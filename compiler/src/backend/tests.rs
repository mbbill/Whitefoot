#![allow(clippy::panic)]

mod arenas;
mod arithmetic_obligations;
mod arrays;
mod base64;
mod buffers;
mod checked_division;
mod cost_shape;
mod counted_ranges;
mod deterministic_target;
mod division_obligations;
mod effect_attributes;
mod float_conversion;
mod floating;
mod integer_absolute;
mod integer_conversion;
mod integer_extended;
mod integer_negation;
mod options;
mod propagation;
mod reborrows;
mod reinterpret;
mod requires;
mod resource_enums;
mod slices;
mod system;
mod system_io;

use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome,
    HOST_OPTIMIZATION_ARGUMENTS, ParseLimits, ParseOutcome, ResolutionOutcome, SemanticOutcome,
    SourceBundle, SourceInput, SourceLimits, TerminalLimits, TerminalOutcome, audit_canonical,
    check_semantics, check_semantics_arithmetic_obligations, check_semantics_division_obligations,
    classify_terminals, compile as compile_program, emit_llvm, finalize, lower_checked, parse,
    resolve,
};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 4,
    max_logical_path_bytes: 128,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_binding_bytes: 1_048_576,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 4,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_token_bytes: 16_384,
    max_tokens: 131_072,
    max_lexemes: 262_144,
};

const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 8_000_000,
    max_tasks: 131_072,
    max_frames: 8_192,
    max_elements: 262_144,
};

const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 8_000_000,
    max_roots: 131_072,
    max_shape_tasks: 131_072,
    max_nodes: 131_072,
    max_child_edges: 131_072,
    max_terminals: 131_072,
    max_sources: 4,
};

const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 8_000_000,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_gaps: 131_072,
    max_path_components: 8_192,
};

static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

fn emit(source: &[u8]) -> String {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("backend test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("backend test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("backend test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("backend test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("backend test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("backend test source must resolve");
    };
    let SemanticOutcome::Complete(checked) = check_semantics(resolved) else {
        panic!("backend test source must check");
    };
    let ir = lower_checked(*checked).expect("checked program must lower");
    emit_llvm(&ir)
        .expect("lowered program must emit")
        .into_string()
}

/// [`emit`] through the test-only checker entry that forces the
/// arithmetic-mode dissolution switch on [OP-2, ENT-6], so the emitted
/// module of the v0.31 candidate judgment can be compared against the
/// default v0.30 emission of the same source.
fn emit_arithmetic_obligations(source: &[u8]) -> String {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("backend test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("backend test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("backend test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("backend test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("backend test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("backend test source must resolve");
    };
    let SemanticOutcome::Complete(checked) = check_semantics_arithmetic_obligations(resolved)
    else {
        panic!("backend test source must check under the arithmetic switch");
    };
    let ir = lower_checked(*checked).expect("checked program must lower");
    emit_llvm(&ir)
        .expect("lowered program must emit")
        .into_string()
}

/// [`emit`] through the test-only checker entry that forces the division
/// dissolution switch on [OP-2, ENT-6], so the emitted module of the v0.32
/// candidate judgment can be compared against the default v0.31 emission of
/// the same source.
fn emit_division_obligations(source: &[u8]) -> String {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("backend test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("backend test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("backend test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("backend test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("backend test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("backend test source must resolve");
    };
    let SemanticOutcome::Complete(checked) = check_semantics_division_obligations(resolved) else {
        panic!("backend test source must check under the division switch");
    };
    let ir = lower_checked(*checked).expect("checked program must lower");
    emit_llvm(&ir)
        .expect("lowered program must emit")
        .into_string()
}

fn compile(source: &[u8]) -> String {
    compile_sources(&[("test.wf", source)])
}

/// [`emit`] through the test-only reborrow-extension checker, so execution
/// tests can run the implemented v0.31-candidate chains while the shipped
/// switch keeps v0.30 semantics [OWN-6, OWN-14].
fn emit_reborrow_extension(source: &[u8]) -> String {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("backend test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("backend test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("backend test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("backend test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("backend test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("backend test source must resolve");
    };
    let checked = match crate::semantic::check_semantics_reborrow_extension(resolved) {
        SemanticOutcome::Complete(checked) => checked,
        outcome => {
            panic!("backend test source must check under the reborrow extension: {outcome:?}")
        }
    };
    let ir = lower_checked(*checked).expect("checked program must lower");
    emit_llvm(&ir)
        .expect("lowered program must emit")
        .into_string()
}

/// Compiles a source that must be rejected, returning the failure for rule
/// and detail assertions.
fn compile_rejection(source: &[u8]) -> crate::CompilationFailure {
    let inputs = [SourceInput::new("test.wf", source)];
    compile_program(&inputs, crate::CompilerLimits::default()).expect_err("source must be rejected")
}

fn compile_sources(sources: &[(&str, &[u8])]) -> String {
    let inputs = sources
        .iter()
        .map(|(path, source)| SourceInput::new(path, source))
        .collect::<Vec<_>>();
    compile_program(&inputs, crate::CompilerLimits::default())
        .expect("normal compiler pipeline must emit")
}

fn compile_and_run(llvm: &str) -> Output {
    compile_and_run_with(llvm, &[])
}

/// Creates one fresh directory for a test's own artifacts.
fn test_directory() -> PathBuf {
    let sequence = NEXT_TEST.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-backend-test-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("unique backend test directory");
    directory
}

/// Links one emitted module into an executable inside `directory`.
fn build_executable(llvm: &str, directory: &Path) -> PathBuf {
    build_linked_executable(llvm, None, directory)
}

/// Links one emitted module, optionally with one host translation unit, into
/// an executable inside `directory`.
///
/// A program emitted for the native target links against the host's own
/// facilities and passes no extra unit; a program emitted for the
/// deterministic test target supplies the unit that answers its scripted
/// facilities [QUAL-1].
fn build_linked_executable(llvm: &str, host: Option<&str>, directory: &Path) -> PathBuf {
    let module = directory.join("program.ll");
    let executable = directory.join("program");
    std::fs::write(&module, llvm).expect("write backend test module");
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-x").arg("ir").arg(&module);
    let host_unit = host.map(|source| {
        let path = directory.join("host.c");
        std::fs::write(&path, source).expect("write deterministic host unit");
        path
    });
    if let Some(path) = host_unit.as_ref() {
        command.arg("-x").arg("c").arg(path);
    }
    let compile = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    if !compile.status.success() {
        panic!(
            "clang rejected emitted LLVM:\n{}\n{}",
            String::from_utf8_lossy(&compile.stderr),
            llvm
        );
    }
    std::fs::remove_file(&module).expect("remove backend test module");
    if let Some(path) = host_unit {
        std::fs::remove_file(path).expect("remove deterministic host unit");
    }
    executable
}

/// Runs one emitted module with exact argument bytes.
///
/// The bytes are passed as raw `OsStr`s so a test can hand the program an
/// argument that is not valid text [HOST-1].
fn compile_and_run_with(llvm: &str, arguments: &[&[u8]]) -> Output {
    compile_link_and_run(llvm, None, arguments)
}

/// Runs one emitted module, optionally linked against one host translation
/// unit.
fn compile_link_and_run(llvm: &str, host: Option<&str>, arguments: &[&[u8]]) -> Output {
    let directory = test_directory();
    let executable = build_linked_executable(llvm, host, &directory);
    let output = Command::new(&executable)
        .args(
            arguments
                .iter()
                .map(|bytes| std::ffi::OsStr::from_bytes(bytes)),
        )
        .output()
        .expect("run backend test executable");
    std::fs::remove_file(&executable).expect("remove backend test executable");
    std::fs::remove_dir(&directory).expect("remove backend test directory");
    output
}

/// Returns the module as the host optimizer leaves it at the shipped level.
fn host_optimized_module(llvm: &str) -> String {
    let mut child = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg("-")
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-S")
        .arg("-emit-llvm")
        .arg("-o")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("invoke host clang");
    child
        .stdin
        .take()
        .expect("clang stdin must be available")
        .write_all(llvm.as_bytes())
        .expect("send module to host clang");
    let output = child.wait_with_output().expect("wait for host clang");
    if !output.status.success() {
        panic!(
            "clang rejected emitted LLVM:\n{}\n{llvm}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).expect("optimized module is UTF-8")
}

/// Returns the definition of `main` inside one optimized module.
fn optimized_main(module: &str) -> &str {
    let start = module
        .match_indices(" @main(")
        .find_map(|(symbol_start, _)| {
            let line_start = module[..symbol_start]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line_start..symbol_start]
                .starts_with("define")
                .then_some(line_start)
        })
        .expect("optimized module must still define main");
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 2)
        .expect("main definition must close");
    &module[start..end]
}

fn emitted_function<'module>(module: &'module str, name: &str) -> &'module str {
    let symbol = format!(" @wf_{name}(");
    let function_start = module
        .match_indices(&symbol)
        .find_map(|(symbol_start, _)| {
            let line_start = module[..symbol_start]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line_start..symbol_start]
                .starts_with("define internal")
                .then_some(line_start)
        })
        .unwrap_or_else(|| panic!("missing emitted function {name}"));
    let function_end = module[function_start..]
        .find("\n}\n\n")
        .map(|offset| function_start + offset + 3)
        .expect("source function definition must close");
    &module[function_start..function_end]
}

fn emitted_drop_ids(function: &str) -> Vec<u32> {
    function
        .lines()
        .filter_map(|line| line.strip_prefix("  ; drop %v"))
        .map(|ordinal| ordinal.parse().expect("drop value must have an ordinal"))
        .collect()
}

#[test]
fn emitted_module_retains_checks_and_avoids_undefined_overflow_flags() {
    let source = br#"fn add(x: own i32, y: own i32) -> own i32 traps {
  return x + y;
}

fn main() -> own unit traps {
  let answer = add(x: 40_i32, y: 2_i32);
  claim wrong_answer: ieq(answer, 42_i32) because "wrong answer";
  return unit;
}
"#;
    let llvm = emit(source);
    assert!(llvm.contains("@llvm.sadd.with.overflow.i32"));
    assert!(llvm.contains("br i1"));
    assert!(llvm.contains("call void @wf_trap"));
    assert!(!llvm.contains(" nsw "));
    assert!(!llvm.contains(" nuw "));
    assert!(!llvm.contains("llvm.assume"));
}

#[test]
fn nominal_lowering_keeps_selected_tag_widths_and_initialized_payloads() {
    let source = br#"enum Flag {
  Off();
  On();
}

enum Payload {
  Empty();
  Value(number: i32);
}

fn main() -> own unit pure {
  let flag = On();
  match flag {
    Off() => {
    }
    On() => {
    }
  }
  let payload = Value(number: 42_i32);
  match payload {
    Empty() => {
    }
    Value(number: value) => {
    }
  }
  return unit;
}
"#;
    let llvm = emit(source);
    assert!(llvm.contains("switch i1"));
    assert!(llvm.contains("switch i32"));
    assert!(llvm.contains("insertvalue %wf.t1 zeroinitializer, i32 1, 0"));
    assert!(llvm.contains("call void @abort()"));
    assert!(!llvm.contains("%wf.t0 = type"));
}

#[test]
fn checked_affine_cleanup_survives_lowering_and_emission() {
    let source = br#"struct Cell {
  value: i32;
}

struct Inner {
  selected: Cell;
  sibling: Cell;
}

struct Outer {
  inner: Inner;
  sibling: Cell;
}

enum Holder {
  Held(cell: Cell);
  Empty();
}

fn make() -> own Cell pure {
  let cell = Cell(value: 1_i32);
  return move cell;
}

fn cleanup() -> own unit pure {
  make();
  let first = Cell(value: 2_i32);
  let second = Cell(value: 3_i32);
  let selected = Cell(value: 4_i32);
  let inner_sibling = Cell(value: 5_i32);
  let inner = Inner(selected: move selected, sibling: move inner_sibling);
  let outer_sibling = Cell(value: 6_i32);
  let outer = Outer(inner: move inner, sibling: move outer_sibling);
  let taken = move outer.inner.selected;
  return unit;
}

fn cleanup_match(value: own Holder, flag: own Bool) -> own i32 pure {
  match move value {
    Held(cell: item) => {
    }
    Empty() => {
    }
  }
  let selected = if flag {
    let temporary = Cell(value: 7_i32);
    give 1_i32;
  } else {
    give 0_i32;
  }
  return selected;
}

fn main() -> own unit pure {
  cleanup();
  let cell = Cell(value: 8_i32);
  let holder = Held(cell: move cell);
  let flag = True();
  cleanup_match(value: move holder, flag: flag);
  return unit;
}
"#;
    let llvm = emit(source);
    assert!(emitted_drop_ids(emitted_function(&llvm, "make")).is_empty());

    let cleanup = emitted_function(&llvm, "cleanup");
    let cleanup_drops = emitted_drop_ids(cleanup);
    assert_eq!(cleanup_drops.len(), 6);
    assert!(cleanup_drops[3] > cleanup_drops[4]);
    assert!(cleanup_drops[4] > cleanup_drops[5]);
    assert!(cleanup.contains("; ownership-consuming projection"));

    let cleanup_match = emitted_function(&llvm, "cleanup_match");
    assert_eq!(emitted_drop_ids(cleanup_match).len(), 2);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn copy_place_set_executes_for_root_and_nested_struct_fields() {
    let source = br#"struct Inner {
  value: i32;
}

struct Outer {
  inner: Inner;
  other: i32;
}

fn main() -> own unit traps {
  let number = 1_i32;
  let inner = Inner(value: 2_i32);
  let outer = Outer(inner: move inner, other: 7_i32);
  let flag = True();
  if flag {
    set number = 42_i32;
    set outer.inner.value = number;
  } else {
    set number = 9_i32;
    set outer.inner.value = number;
  }
  let observed = outer.inner.value;
  claim nested_set_failed: ieq(observed, 42_i32) because "nested set failed";
  let preserved = outer.other;
  claim sibling_changed: ieq(preserved, 7_i32) because "sibling changed";
  let selected = if flag {
    set number = 43_i32;
    give number;
  } else {
    set number = 10_i32;
    give number;
  }
  claim value_match_result_failed: ieq(selected, 43_i32) because "value match result failed";
  claim value_match_set_failed: ieq(number, 43_i32) because "value match set failed";
  return unit;
}
"#;
    let llvm = emit(source);
    let main = emitted_function(&llvm, "main");
    assert!(main.contains(" = phi i32 "));
    assert!(main.contains(" = insertvalue %wf.t1"));
    assert!(main.contains(" = insertvalue %wf.t0"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [GRAM-6] the Bool conditional lowers through the Bool `match` path it
/// checks into, so no lowering, cleanup, or drop change is owed. Every branch
/// here is observable: a wrong one leaves a flag false and the check traps.
#[test]
fn bool_conditionals_execute_through_the_existing_match_lowering() {
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  let other = False();
  let seen = False();
  if flag {
    set seen = True();
  }
  claim the_else_free_if_did_not_run: seen because "the else-free if did not run";
  let untouched = True();
  if other {
    set untouched = False();
  }
  claim the_else_free_if_ran_when_it_should_not: untouched because "the else-free if ran when it should not";
  let taken = if flag {
    give True();
  } else {
    give False();
  }
  claim the_value_if_took_the_wrong_branch: taken because "the value_if took the wrong branch";
  let chained = if other {
    give False();
  } else if flag {
    give True();
  } else {
    give False();
  }
  claim the_else_if_chain_took_the_wrong_branch: chained because "the else-if chain took the wrong branch";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [STOR-2] a box whose nominal is derived rather than written lowers and
/// runs like any other: it is in the executable prefix, it allocates, its
/// content reads back, and it is released. `box<u64>` is spelled nowhere.
#[test]
fn a_derived_box_nominal_allocates_reads_back_and_releases() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let flag = True();
  let owner = box_new(flag);
  let loaded = deref(owner);
  claim the_derived_box_did_not_read_back: loaded because "the derived box did not read back";
  return unit;
}
"#;
    let llvm = compile(source);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [OP-1] (ii) the infix spelling executes the row its operator names, and
/// the modes stay distinguishable: bare `+` traps, `+wrap` wraps, `+sat`
/// saturates. Every result is checked, so a mis-selected row traps here.
#[test]
fn infix_operators_execute_the_rows_they_name() {
    let source = br#"fn main() -> own unit traps {
  let a = 20_i32;
  let b = a + 22_i32;
  let want = 42_i32;
  let sum_ok = ieq(b, want);
  claim bare_plus_is_the_trapping_add: sum_ok because "bare plus is the trapping add";
  let hi = 2147483647_i32;
  let wrapped = hi +wrap 1_i32;
  let low = -2147483648_i32;
  let wrap_ok = ieq(wrapped, low);
  claim wrap_wraps: wrap_ok because "+wrap wraps";
  let saturated = hi +sat 1_i32;
  let sat_ok = ieq(saturated, hi);
  claim sat_saturates: sat_ok because "+sat saturates";
  let quotient = 43_i32 / 2_i32;
  let q_ok = ieq(quotient, 21_i32);
  claim bare_slash_divides: q_ok because "bare slash divides";
  let rest = 43_i32 % 2_i32;
  let r_ok = ieq(rest, 1_i32);
  claim bare_percent_remainders: r_ok because "bare percent remainders";
  let differ = ine(a, b);
  claim not_equal: differ because "not equal";
  let ordered = ile(a, b);
  claim less_equal: ordered because "less equal";
  let reversed = ige(b, a);
  claim greater_equal: reversed because "greater equal";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [OP-1] (ii) an infix returned directly from a function lowers and executes,
/// not merely type-checks.
///
/// `return a + b;` failed semantic checking outright, so no lowering evidence
/// existed for the shape. Both an arithmetic and a comparison result are
/// returned and consumed at the call site, and the false comparison gates a
/// trap, so a lost or inverted result fails here rather than passing quietly.
#[test]
fn an_infix_returned_from_a_function_executes() {
    let source = br#"fn add(a: own i32, b: own i32) -> own i32 traps {
  return a + b;
}

fn eq(a: own i32, b: own i32) -> own Bool pure {
  return ieq(a, b);
}

fn main() -> own unit traps {
  let sum = add(a: 20_i32, b: 22_i32);
  let sum_ok = ieq(sum, 42_i32);
  claim the_returned_sum_is_wrong: sum_ok because "the returned sum is wrong";
  let same = eq(a: 7_i32, b: 7_i32);
  claim the_returned_comparison_is_wrong: same because "the returned comparison is wrong";
  let differ = eq(a: 7_i32, b: 8_i32);
  let impossible = eq(a: 0_i32, b: 1_i32);
  if differ {
    claim a_false_returned_comparison_must_not_be_true: impossible because "a false returned comparison must not be true";
  }
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [OP-2] bare infix arithmetic outside the constant-operand class keeps the
/// trapping semantics its named `.trap` spelling had: the required check is
/// not lost to the shorter form. Both operands are bound rather than written
/// as literals, so the site stays in the retained trapping class.
#[test]
fn bare_infix_overflow_traps_at_runtime() {
    let source = br#"fn main() -> own unit traps {
  let hi = 2147483647_i32;
  let one = 1_i32;
  let overflowed = hi + one;
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with("{\"rule_id\":\"OP-2\",\"message\":\"integer overflow\""));
}

#[test]
fn compiler_independent_scalar_cases_execute_through_host_llvm() {
    for source in [
        include_bytes!("../../../tests/conformance/cases/scope3-pos-defined-run.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/gram11-pos-named-args.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/form7-pos-in-range.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/op1-pos-table-op.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-const-scalar-u64-width.wf").as_slice(),
        include_bytes!(
            "../../../tests/conformance/cases/x-arith-iadd-wrap-overflow-to-negative.wf"
        )
        .as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-arith-isub-wrap-min-roundtrip-runs.wf")
            .as_slice(),
    ] {
        let output = compile_and_run(&compile(source));
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn compiler_independent_loop_accumulator_executes_through_host_llvm() {
    for source in [
        include_bytes!("../../../tests/conformance/cases/gram6-pos-no-operators.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/own1-pos-tagonly-copy.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/type2-pos-twostate-enum-i1.wf").as_slice(),
    ] {
        let llvm = compile(source);
        let main = emitted_function(&llvm, "main");
        assert!(main.contains(" = phi "));

        let output = compile_and_run(&llvm);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn result_values_checked_arithmetic_and_propagation_execute_through_host_llvm() {
    let source = br#"enum StepError {
  Failed();
}

struct Pair {
  left: i32;
  right: i32;
}

struct Envelope {
  result: Result<i32, StepError>;
  residue: Pair;
}

fn step(value: own i32) -> own Result<i32, StepError> pure {
  if ilt(value, 0_i32) {
    let error = Failed();
    return Err<i32, StepError>(error: error);
  } else {
    return Ok<i32, StepError>(value: value);
  }
}

fn forward(value: own i32) -> own Result<i64, StepError> pure {
  let result = step(value: value);
  let accepted = propagate result;
  return Ok<i64, StepError>(value: 42_i64);
}

fn forward_field(value: own i32) -> own Result<i64, StepError> pure {
  let result = step(value: value);
  let residue = Pair(left: 1_i32, right: 2_i32);
  let envelope = Envelope(result: move result, residue: move residue);
  let accepted = propagate envelope.result;
  return Ok<i64, StepError>(value: 42_i64);
}

fn make_pair() -> own Result<Pair, StepError> pure {
  let pair = Pair(left: 20_i32, right: 22_i32);
  return Ok<Pair, StepError>(value: move pair);
}

fn main() -> own unit traps {
  let arithmetic_result = 2147483647_i32 +checked 1_i32;
  match move arithmetic_result {
    Ok(value: sum) => {
      claim checked_overflow_took_ok: False() because "checked overflow took Ok";
    }
    Err(error: overflow) => {
    }
  }
  let subtract_result = 0_u8 -checked 1_u8;
  match move subtract_result {
    Ok(value: difference) => {
      claim checked_underflow_took_ok: False() because "checked underflow took Ok";
    }
    Err(error: underflow) => {
    }
  }
  let multiply_result = 6_i16 *checked 7_i16;
  match move multiply_result {
    Ok(value: product) => {
      claim checked_product_drift: ieq(product, 42_i16) because "checked product drift";
    }
    Err(error: product_error) => {
      claim checked_product_took_err: False() because "checked product took Err";
    }
  }
  let success = forward(value: 7_i32);
  match move success {
    Ok(value: answer) => {
      claim propagated_ok_payload_drift: ieq(answer, 42_i64) because "propagated Ok payload drift";
    }
    Err(error: failure_error) => {
      claim unexpected_propagated_err: False() because "unexpected propagated Err";
    }
  }
  let failure = forward(value: -1_i32);
  match move failure {
    Ok(value: unexpected) => {
      claim propagated_err_became_ok: False() because "propagated Err became Ok";
    }
    Err(error: forwarded_error) => {
    }
  }
  let field_success = forward_field(value: 7_i32);
  match move field_success {
    Ok(value: field_answer) => {
      claim field_propagation_drift: ieq(field_answer, 42_i64) because "field propagation drift";
    }
    Err(error: field_failure) => {
      claim unexpected_field_propagation_error: False() because "unexpected field propagation error";
    }
  }
  let field_failure = forward_field(value: -1_i32);
  match move field_failure {
    Ok(value: field_unexpected) => {
      claim field_propagation_lost_err: False() because "field propagation lost Err";
    }
    Err(error: field_forwarded_error) => {
    }
  }
  let pair_result = make_pair();
  match move pair_result {
    Ok(value: pair) => {
      let total = pair.left +wrap pair.right;
      claim aggregate_result_payload_drift: ieq(total, 42_i32) because "aggregate Result payload drift";
    }
    Err(error: pair_error) => {
      claim unexpected_aggregate_result_error: False() because "unexpected aggregate Result error";
    }
  }
  return unit;
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("@llvm.sadd.with.overflow.i32"));
    assert!(llvm.contains("@llvm.usub.with.overflow.i8"));
    assert!(llvm.contains("@llvm.smul.with.overflow.i16"));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    for independent in [
        include_bytes!("../../../tests/conformance/cases/err1-pos-result-value-match.wf")
            .as_slice(),
        include_bytes!("../../../tests/conformance/cases/pre1-pos-prelude-enums.wf").as_slice(),
        include_bytes!(
            "../../../tests/conformance/cases/x-arith-iadd-checked-overflow-err-arm-runs.wf"
        )
        .as_slice(),
        include_bytes!("../../../tests/conformance/cases/run-ex2-loop-trap-folds.wf").as_slice(),
    ] {
        let output = compile_and_run(&compile(independent));
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn nested_loop_labels_route_breaks_to_the_resolved_exit() {
    let source = br#"fn main() -> own unit traps {
  let outer = 0_i32;
  loop @outer_loop {
    set outer = outer +wrap 1_i32;
    let inner = 0_i32;
    loop @inner_loop {
      if ige(outer, 3_i32) {
        break @outer_loop;
      }
      if ige(inner, 2_i32) {
        break @inner_loop;
      }
      set inner = inner +wrap 1_i32;
    }
  }
  claim wrong_outer_exit: ieq(outer, 3_i32) because "wrong outer exit";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_nominal_data_cases_execute_through_host_llvm() {
    for source in [
        include_bytes!("../../../tests/conformance/cases/x-struct-construct-read-field.wf")
            .as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-struct-cross-fn.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-struct-mixed-width.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-struct-nested-field.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-struct-set-field.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-enum-payload-give.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-enum-multiwidth-dispatch.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-enum-stmt-payload-check.wf").as_slice(),
        include_bytes!(
            "../../../tests/conformance/cases/x-ownmove-copy-reused-affine-consumed-once.wf"
        )
        .as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-ownmove-owned-temporary-scrutinee.wf")
            .as_slice(),
        include_bytes!("../../../tests/conformance/cases/op1-pos-bool-enum-equality.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/op1-pos-tag-enum-equality.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/type2-pos-enum.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/gram8-pos-construct.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/err2-pos-exhaustive-match.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/fn5-pos-match-dispatch.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-nominal-bool-ops-run.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/x-nominal-multifield-payload-run.wf")
            .as_slice(),
    ] {
        let output = compile_and_run(&compile(source));
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn every_lowered_integer_mode_and_comparison_executes_with_exact_width_and_sign() {
    let source = br#"fn main() -> own unit traps {
  let aw = 127_i8 +wrap 1_i8;
  let sw = 0_u8 -wrap 1_u8;
  let mw = 65535_u16 *wrap 2_u16;
  let ast = -10_i16 + 3_i16;
  let aut = 10_u16 + 3_u16;
  let sst = 10_i32 - 3_i32;
  let sut = 10_u32 - 3_u32;
  let mst = 6_i64 * 7_i64;
  let mut = 6_u64 * 7_u64;
  claim signed_add_wrap_drift: ieq(aw, -128_i8) because "signed add wrap drift";
  claim unsigned_subtract_wrap_drift: ieq(sw, 255_u8) because "unsigned subtract wrap drift";
  claim unsigned_multiply_wrap_drift: ieq(mw, 65534_u16) because "unsigned multiply wrap drift";
  claim signed_add_trap_drift: ieq(ast, -7_i16) because "signed add trap drift";
  claim unsigned_add_trap_drift: ieq(aut, 13_u16) because "unsigned add trap drift";
  claim signed_subtract_trap_drift: ieq(sst, 7_i32) because "signed subtract trap drift";
  claim unsigned_subtract_trap_drift: ieq(sut, 7_u32) because "unsigned subtract trap drift";
  claim signed_multiply_trap_drift: ieq(mst, 42_i64) because "signed multiply trap drift";
  claim unsigned_multiply_trap_drift: ieq(mut, 42_u64) because "unsigned multiply trap drift";
  claim ine_drift: ine(1_i32, 2_i32) because "ine drift";
  claim signed_ilt_drift: ilt(-1_i32, 0_i32) because "signed ilt drift";
  claim unsigned_ile_drift: ile(1_u32, 1_u32) because "unsigned ile drift";
  claim signed_igt_drift: igt(1_i32, -1_i32) because "signed igt drift";
  claim unsigned_ige_drift: ige(1_u32, 1_u32) because "unsigned ige drift";
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn unit_is_a_first_class_parameter_result_and_local() {
    let source = br#"fn identity(value: own unit) -> own unit pure {
  return value;
}

fn main() -> own unit pure {
  let value = identity(value: unit);
  return value;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The [OP-5] record shape, including the exact [FORM-5] decoding of a
/// message carrying an embedded quote and newline.
///
/// v0.32 retires the body `check` statement, so the entry requirement's
/// final `check_stmt` is the sole remaining [OP-5] record carrier whose
/// `message` is a writer-chosen STRING — a migrated body check becomes a
/// [CLM-1] record whose `message` is an IDENT and can carry neither byte.
/// The record shape and the message decoding are unchanged across the
/// version, which is what this pins.
#[test]
fn a_failing_entry_requirement_emits_the_exact_mandatory_record_shape() {
    let source = b"fn main() -> own unit pure requires {\n  check ieq(0_u8, 1_u8) else trap \"bad \\\"quote\\\"\\nline\";\n} {\n  return unit;\n}\n";
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(
        stderr.starts_with(
            "{\"rule_id\":\"OP-5\",\"message\":\"bad \\\"quote\\\"\\nline\",\"function\":\"main\",\"node_path\":["
        ),
        "unexpected record: {stderr}"
    );
    assert!(stderr.ends_with("]}\n"));
    assert_eq!(stderr.lines().count(), 1);
}

/// The [CLM-1] record a migrated body check now emits: same mandatory
/// [DIAG-3] field order and framing, `rule_id` `CLM-1`, and `message` the
/// claim's IDENT spelling rather than its `because` justification.
#[test]
fn a_failing_claim_emits_the_exact_mandatory_record_shape() {
    let source =
        b"fn main() -> own unit traps {\n  claim bad_quote_line: False() because \"bad \\\"quote\\\"\\nline\";\n  return unit;\n}\n";
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(
        stderr.starts_with(
            "{\"rule_id\":\"CLM-1\",\"message\":\"bad_quote_line\",\"function\":\"main\",\"node_path\":["
        ),
        "unexpected record: {stderr}"
    );
    assert!(stderr.ends_with("]}\n"));
    assert_eq!(stderr.lines().count(), 1);
}

/// [DIAG-3] record fields carry their exact bytes for every scalar, not only
/// for ASCII.
///
/// This exercises the record encoder directly because [FORM-5] still admits no
/// non-ASCII byte in a STRING, so no source program can reach the case through
/// a `trap` message yet. The encoder is nonetheless the real emission path for
/// every record, and it was silently lossy: byte iteration re-encoded each
/// continuation byte as its Latin-1 scalar, so `"é"` (2 bytes) left as 4 and
/// `"日"` (3 bytes) as 6. The assertion is exact bytes rather than a length,
/// because a length check passes on mojibake of the right size.
///
/// A green run establishes that the encoder preserves and escapes correctly;
/// it does not establish that any source program can produce such a message.
#[test]
fn a_diag3_record_preserves_the_exact_utf8_bytes_of_its_message() {
    let record = crate::backend::emitter::trap_record(&crate::IrTrapSite {
        rule_id: "OP-5",
        // One two-byte scalar, one three-byte scalar, one four-byte scalar,
        // and both characters that still need a JSON escape.
        message: "é 日 \u{1F600} \"q\"\nl".to_owned(),
        function: "main".to_owned(),
        node_path: vec![0, 1],
    });
    assert_eq!(
        record,
        "{\"rule_id\":\"OP-5\",\"message\":\"é 日 \u{1F600} \\\"q\\\"\\nl\",\
         \"function\":\"main\",\"node_path\":[0,1]}\n"
            .as_bytes()
    );
    // The record is exactly the bytes the message was written with: no
    // expansion, no replacement scalar, and no encoding split across the
    // escape boundary.
    assert!(String::from_utf8(record.clone()).is_ok());
    assert_eq!(
        record.iter().filter(|byte| !byte.is_ascii()).count(),
        "é日\u{1F600}".len()
    );
}

#[test]
fn integer_overflow_reports_op2_before_abort() {
    // Both operands are bound, keeping the site in [OP-2]'s retained
    // trapping class: a written literal operand would carry a compile-time
    // overflow obligation instead and could never reach a runtime record.
    let source = br#"fn main() -> own unit traps {
  let hi = 127_i8;
  let one = 1_i8;
  let overflow = hi + one;
  return unit;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-2\",\"message\":\"integer overflow\",\"function\":\"main\",\"node_path\":["
    ));
    assert!(stderr.ends_with("]}\n"));
}

#[test]
fn required_check_survives_host_optimization_of_an_unfoldable_loop() {
    // The loop multiplies by an odd constant and mixes in the counter, so no
    // closed form exists for the host optimizer to fold, and the iteration
    // count is far past any full-unroll budget. The failing condition
    // therefore cannot be decided before execution: whatever the optimizer
    // does, the check has to run.
    let source = br#"fn main() -> own unit traps {
  doc "A mixing chain the host optimizer cannot fold feeds one required check.";
  let step = 0_u64;
  let state = 14695981039346656037_u64;
  loop @mix {
    if ige(step, 4096_u64) {
      break @mix;
    }
    let mixed = ixor(state, step);
    set state = mixed *wrap 1099511628211_u64;
    set step = step + 1_u64;
  }
  claim mixing_chain_drift: ieq(state, 1_u64) because "mixing chain drift";
  return unit;
}
"#;
    let llvm = compile(source);
    let optimized = host_optimized_module(&llvm);
    // Unoptimized `main` is a bare wrapper around `wf_main`, so finding the
    // loop and its trap edge inside `main` also witnesses that the shared
    // optimization arguments really reached the host compiler.
    let main = optimized_main(&optimized);
    assert!(
        main.contains("1099511628211"),
        "the host optimizer folded the mixing chain, so the check is decidable:\n{main}"
    );
    assert!(
        main.contains("br i1"),
        "the check became unconditional under host optimization:\n{main}"
    );
    assert!(
        main.contains("@wf_trap"),
        "host optimization dropped the trap edge of a required check:\n{main}"
    );

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    assert_eq!(
        output.status.code(),
        None,
        "a trap aborts instead of exiting normally"
    );
    assert_eq!(
        output.stderr,
        b"{\"rule_id\":\"OP-5\",\"message\":\"mixing chain drift\",\"function\":\"main\",\"node_path\":[0,0,6,0]}\n"
    );
    assert!(output.stdout.is_empty());
}
