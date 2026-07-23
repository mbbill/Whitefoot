#![allow(clippy::panic)]

mod checked_division;
mod integer_absolute;
mod integer_negation;

use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::lexer::{LexLimits, LexOutcome, lex_v0_14};
use crate::{
    CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome, KERNEL_SPEC_V0_14_HASH,
    ParseLimits, ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput,
    SourceLimits, TerminalLimits, TerminalOutcome, audit_canonical_v0_14, check_semantics_v0_14,
    classify_terminals_v0_14, compile_v0_14, emit_llvm_v0_14, finalize_v0_14, lower_checked_v0_14,
    parse_v0_14, resolve_v0_14,
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
    let LexOutcome::Complete(lexed) = lex_v0_14(&bundle, LEX_LIMITS) else {
        panic!("backend test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_14(
        &lexed,
        KERNEL_SPEC_V0_14_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("backend test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse_v0_14(&classified, PARSE_LIMITS) else {
        panic!("backend test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize_v0_14(parsed, FINALIZE_LIMITS) else {
        panic!("backend test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical_v0_14(finalized, CANONICAL_LIMITS)
    else {
        panic!("backend test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve_v0_14(canonical) else {
        panic!("backend test source must resolve");
    };
    let SemanticOutcome::Complete(checked) = check_semantics_v0_14(resolved) else {
        panic!("backend test source must check");
    };
    let ir = lower_checked_v0_14(*checked).expect("checked program must lower");
    emit_llvm_v0_14(&ir)
        .expect("lowered program must emit")
        .into_string()
}

fn compile(source: &[u8]) -> String {
    compile_v0_14(
        &[SourceInput::new("test.wf", source)],
        crate::CompilerLimits::default(),
    )
    .expect("normal compiler pipeline must emit")
}

fn compile_and_run(llvm: &str) -> Output {
    let sequence = NEXT_TEST.fetch_add(1, Ordering::Relaxed);
    let directory =
        std::env::temp_dir().join(format!("whitefoot-v014-{}-{sequence}", std::process::id()));
    std::fs::create_dir(&directory).expect("unique backend test directory");
    let module = directory.join("program.ll");
    let executable = directory.join("program");
    std::fs::write(&module, llvm).expect("write backend test module");
    let compile = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg(&module)
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
    let output = Command::new(&executable)
        .output()
        .expect("run backend test executable");
    std::fs::remove_file(&executable).expect("remove backend test executable");
    std::fs::remove_file(&module).expect("remove backend test module");
    std::fs::remove_dir(&directory).expect("remove backend test directory");
    output
}

fn emitted_function<'module>(module: &'module str, name: &str) -> &'module str {
    let symbol = format!(" @wf_{name}(");
    let symbol_start = module
        .find(&symbol)
        .unwrap_or_else(|| panic!("missing emitted function {name}"));
    let function_start = module[..symbol_start]
        .rfind("define internal")
        .expect("source function must have an internal definition");
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
  return iadd.trap<i32>(x, y);
}

fn main() -> own unit traps {
  let answer: own i32 = add(x: 40_i32, y: 2_i32);
  check ieq<i32>(answer, 42_i32) else trap "wrong answer";
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
  let flag: own Flag = On();
  match flag {
    Off() => {
    }
    On() => {
    }
  }
  let payload: own Payload = Value(number: 42_i32);
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
  let cell: own Cell = Cell(value: 1_i32);
  return move cell;
}

fn cleanup() -> own unit pure {
  make();
  let first: own Cell = Cell(value: 2_i32);
  let second: own Cell = Cell(value: 3_i32);
  let selected: own Cell = Cell(value: 4_i32);
  let inner_sibling: own Cell = Cell(value: 5_i32);
  let inner: own Inner = Inner(selected: move selected, sibling: move inner_sibling);
  let outer_sibling: own Cell = Cell(value: 6_i32);
  let outer: own Outer = Outer(inner: move inner, sibling: move outer_sibling);
  let taken: own Cell = move outer.inner.selected;
  return unit;
}

fn cleanup_match(value: own Holder, flag: own Bool) -> own i32 pure {
  match move value {
    Held(cell: item) => {
    }
    Empty() => {
    }
  }
  let selected: own i32 = match flag {
    True() => {
      let temporary: own Cell = Cell(value: 7_i32);
      give 1_i32;
    }
    False() => {
      give 0_i32;
    }
  }
  return selected;
}

fn main() -> own unit pure {
  cleanup();
  let cell: own Cell = Cell(value: 8_i32);
  let holder: own Holder = Held(cell: move cell);
  let flag: own Bool = True();
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
  let number: own i32 = 1_i32;
  let inner: own Inner = Inner(value: 2_i32);
  let outer: own Outer = Outer(inner: move inner, other: 7_i32);
  let flag: own Bool = True();
  match flag {
    True() => {
      set number = 42_i32;
      set outer.inner.value = number;
    }
    False() => {
      set number = 9_i32;
      set outer.inner.value = number;
    }
  }
  let observed: own i32 = outer.inner.value;
  check ieq<i32>(observed, 42_i32) else trap "nested set failed";
  let preserved: own i32 = outer.other;
  check ieq<i32>(preserved, 7_i32) else trap "sibling changed";
  let selected: own i32 = match flag {
    True() => {
      set number = 43_i32;
      give number;
    }
    False() => {
      set number = 10_i32;
      give number;
    }
  }
  check ieq<i32>(selected, 43_i32) else trap "value match result failed";
  check ieq<i32>(number, 43_i32) else trap "value match set failed";
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

#[test]
fn compiler_independent_scalar_cases_execute_through_host_llvm() {
    for source in [
        include_bytes!("../../../tests/conformance/cases/scope3-pos-defined-run.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/type5-pos-explicit.wf").as_slice(),
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
  match ilt<i32>(value, 0_i32) {
    True() => {
      let error: own StepError = Failed();
      return Err(error: error);
    }
    False() => {
      return Ok(value: value);
    }
  }
}

fn forward(value: own i32) -> own Result<i64, StepError> pure {
  let result: own Result<i32, StepError> = step(value: value);
  let accepted: own i32 = propagate result;
  return Ok(value: 42_i64);
}

fn forward_field(value: own i32) -> own Result<i64, StepError> pure {
  let result: own Result<i32, StepError> = step(value: value);
  let residue: own Pair = Pair(left: 1_i32, right: 2_i32);
  let envelope: own Envelope = Envelope(result: move result, residue: move residue);
  let accepted: own i32 = propagate envelope.result;
  return Ok(value: 42_i64);
}

fn make_pair() -> own Result<Pair, StepError> pure {
  let pair: own Pair = Pair(left: 20_i32, right: 22_i32);
  return Ok(value: move pair);
}

fn main() -> own unit traps {
  let arithmetic_result: own Result<i32, Overflow> = iadd.checked<i32>(2147483647_i32, 1_i32);
  match move arithmetic_result {
    Ok(value: sum) => {
      check False() else trap "checked overflow took Ok";
    }
    Err(error: overflow) => {
    }
  }
  let subtract_result: own Result<u8, Overflow> = isub.checked<u8>(0_u8, 1_u8);
  match move subtract_result {
    Ok(value: difference) => {
      check False() else trap "checked underflow took Ok";
    }
    Err(error: underflow) => {
    }
  }
  let multiply_result: own Result<i16, Overflow> = imul.checked<i16>(6_i16, 7_i16);
  match move multiply_result {
    Ok(value: product) => {
      check ieq<i16>(product, 42_i16) else trap "checked product drift";
    }
    Err(error: product_error) => {
      check False() else trap "checked product took Err";
    }
  }
  let success: own Result<i64, StepError> = forward(value: 7_i32);
  match move success {
    Ok(value: answer) => {
      check ieq<i64>(answer, 42_i64) else trap "propagated Ok payload drift";
    }
    Err(error: failure_error) => {
      check False() else trap "unexpected propagated Err";
    }
  }
  let failure: own Result<i64, StepError> = forward(value: -1_i32);
  match move failure {
    Ok(value: unexpected) => {
      check False() else trap "propagated Err became Ok";
    }
    Err(error: forwarded_error) => {
    }
  }
  let field_success: own Result<i64, StepError> = forward_field(value: 7_i32);
  match move field_success {
    Ok(value: field_answer) => {
      check ieq<i64>(field_answer, 42_i64) else trap "field propagation drift";
    }
    Err(error: field_failure) => {
      check False() else trap "unexpected field propagation error";
    }
  }
  let field_failure: own Result<i64, StepError> = forward_field(value: -1_i32);
  match move field_failure {
    Ok(value: field_unexpected) => {
      check False() else trap "field propagation lost Err";
    }
    Err(error: field_forwarded_error) => {
    }
  }
  let pair_result: own Result<Pair, StepError> = make_pair();
  match move pair_result {
    Ok(value: pair) => {
      let total: own i32 = iadd.wrap<i32>(pair.left, pair.right);
      check ieq<i32>(total, 42_i32) else trap "aggregate Result payload drift";
    }
    Err(error: pair_error) => {
      check False() else trap "unexpected aggregate Result error";
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
  let outer: own i32 = 0_i32;
  loop @outer_loop {
    set outer = iadd.wrap<i32>(outer, 1_i32);
    let inner: own i32 = 0_i32;
    loop @inner_loop {
      match ige<i32>(outer, 3_i32) {
        True() => {
          break @outer_loop;
        }
        False() => {
        }
      }
      match ige<i32>(inner, 2_i32) {
        True() => {
          break @inner_loop;
        }
        False() => {
        }
      }
      set inner = iadd.wrap<i32>(inner, 1_i32);
    }
  }
  check ieq<i32>(outer, 3_i32) else trap "wrong outer exit";
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
  let aw: own i8 = iadd.wrap<i8>(127_i8, 1_i8);
  let sw: own u8 = isub.wrap<u8>(0_u8, 1_u8);
  let mw: own u16 = imul.wrap<u16>(65535_u16, 2_u16);
  let ast: own i16 = iadd.trap<i16>(-10_i16, 3_i16);
  let aut: own u16 = iadd.trap<u16>(10_u16, 3_u16);
  let sst: own i32 = isub.trap<i32>(10_i32, 3_i32);
  let sut: own u32 = isub.trap<u32>(10_u32, 3_u32);
  let mst: own i64 = imul.trap<i64>(6_i64, 7_i64);
  let mut: own u64 = imul.trap<u64>(6_u64, 7_u64);
  check ieq<i8>(aw, -128_i8) else trap "signed add wrap drift";
  check ieq<u8>(sw, 255_u8) else trap "unsigned subtract wrap drift";
  check ieq<u16>(mw, 65534_u16) else trap "unsigned multiply wrap drift";
  check ieq<i16>(ast, -7_i16) else trap "signed add trap drift";
  check ieq<u16>(aut, 13_u16) else trap "unsigned add trap drift";
  check ieq<i32>(sst, 7_i32) else trap "signed subtract trap drift";
  check ieq<u32>(sut, 7_u32) else trap "unsigned subtract trap drift";
  check ieq<i64>(mst, 42_i64) else trap "signed multiply trap drift";
  check ieq<u64>(mut, 42_u64) else trap "unsigned multiply trap drift";
  check ine<i32>(1_i32, 2_i32) else trap "ine drift";
  check ilt<i32>(-1_i32, 0_i32) else trap "signed ilt drift";
  check ile<u32>(1_u32, 1_u32) else trap "unsigned ile drift";
  check igt<i32>(1_i32, -1_i32) else trap "signed igt drift";
  check ige<u32>(1_u32, 1_u32) else trap "unsigned ige drift";
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
  let value: own unit = identity(value: unit);
  return value;
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_check_failure_emits_the_exact_mandatory_record_shape() {
    let source = b"fn main() -> own unit traps {\n  check False() else trap \"bad \\\"quote\\\"\\nline\";\n  return unit;\n}\n";
    let output = compile_and_run(&compile(source));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-5\",\"message\":\"bad \\\"quote\\\"\\nline\",\"function\":\"main\",\"node_path\":["
    ));
    assert!(stderr.ends_with("]}\n"));
    assert_eq!(stderr.lines().count(), 1);
}

#[test]
fn integer_overflow_reports_op2_before_abort() {
    let source = br#"fn main() -> own unit traps {
  let overflow: own i8 = iadd.trap<i8>(127_i8, 1_i8);
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
