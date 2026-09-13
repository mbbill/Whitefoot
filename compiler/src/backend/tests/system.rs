//! Ordinary callable ABI and prelude library behavior. C2 retires semantic-ID,
//! target-qualification, input-label and implicit-release assertions; the
//! preserved observable cases execute the same ordinary source as the corpus.

use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalOutcome, FinalizeOutcome, IrProgram, LexOutcome,
    OverlapLowering, ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput,
    TerminalLimits, TerminalOutcome, audit_canonical, check_semantics, classify_terminals,
    finalize, lex, lower_checked, parse, resolve,
};

use super::{
    CANONICAL_LIMITS, FINALIZE_LIMITS, LEX_LIMITS, PARSE_LIMITS, SOURCE_LIMITS, compile,
    compile_and_run, compile_and_run_with, compile_rejection, emitted_function,
};

pub(super) fn with_ir<R>(
    source: &[u8],
    run: impl for<'a, 'b, 'c> FnOnce(&IrProgram<'a, 'b, 'c>) -> R,
) -> R {
    with_mutated_ir(source, |program| run(program))
}

pub(super) fn with_mutated_ir<R>(
    source: &[u8],
    run: impl for<'a, 'b, 'c> FnOnce(&mut IrProgram<'a, 'b, 'c>) -> R,
) -> R {
    with_mutated_ir_lowering(source, OverlapLowering::Off, run)
}

pub(super) fn with_parallel_ir<R>(
    source: &[u8],
    run: impl for<'a, 'b, 'c> FnOnce(&IrProgram<'a, 'b, 'c>) -> R,
) -> R {
    with_mutated_ir_lowering(source, OverlapLowering::On, |program| run(program))
}

pub(super) fn with_mutated_ir_lowering<R>(
    source: &[u8],
    overlap: OverlapLowering,
    run: impl for<'a, 'b, 'c> FnOnce(&mut IrProgram<'a, 'b, 'c>) -> R,
) -> R {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_prelude(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("ordinary ABI test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("ordinary ABI test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("ordinary ABI test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("ordinary ABI test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("ordinary ABI test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("ordinary ABI test source must resolve");
    };
    let checked = match check_semantics(resolved) {
        SemanticOutcome::Complete(checked) => checked,
        other => panic!("ordinary ABI test source must check: {other:?}"),
    };
    let mut ir = lower_checked(*checked, overlap).expect("checked program must lower");
    run(&mut ir)
}

pub(super) fn corpus_source(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/conformance/cases")
            .join(format!("{name}.wf")),
    )
    .expect("retained ordinary corpus source")
}

fn run_arguments(name: &str, arguments: &[&[u8]]) {
    let output = compile_and_run_with(&compile(&corpus_source(name)), arguments);
    assert!(output.status.success(), "{name}: {output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn ordinary_declarations_have_no_frame_and_share_the_call_abi() {
    with_ir(
        br#"fn relay(code: own u8) -> result: own ExitStatus pure {
  return exit_status(code: code);
}
"#,
        |program| {
            let declared = program
                .functions()
                .iter()
                .find(|function| function.name() == "exit_status")
                .expect("ordinary prelude signature");
            assert!(declared.blocks().is_empty());
            let plan = crate::backend::storage::FunctionStoragePlan::build(program, declared)
                .expect("a declaration has no activation to allocate");
            assert!(plan.slots().is_empty());
            let llvm = crate::emit_llvm(program)
                .expect("ordinary signature and body emit")
                .into_string();
            assert!(llvm.contains("declare void @wf_exit_status(ptr %wf.result, i8 %v0)"));
            assert!(llvm.contains("call void @wf_exit_status(ptr"));
            assert!(
                !llvm.contains("@main("),
                "a callable unit needs no selected entry"
            );
        },
    );
}

#[test]
fn a_non_utf8_argument_round_trips_its_exact_bytes() {
    run_arguments("run-syshost-nontext-argv-bytes-roundtrip", &[b"a\xffb"]);
}

const COPY_BYTES_WRAPPER: &str = r#"fn copy_bytes(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
} {
  region {
    match host_copy_bytes(value: value, destination: &uniq deref(destination), start: start, end: end) {
      Ok(value: next) => {
        return Ok<u64, CopyError>(value: next);
      }
      Err(error: problem) => {
        return Err<u64, CopyError>(error: move problem);
      }
    }
  }
}

"#;

#[test]
fn a_view_signature_is_identical_for_a_wf_body_and_a_linked_body() {
    let original = String::from_utf8(corpus_source("run-syshost-nontext-argv-bytes-roundtrip"))
        .expect("source is UTF-8");
    let source = format!(
        "{COPY_BYTES_WRAPPER}{}",
        original.replace("host_copy_bytes(", "copy_bytes(")
    );
    with_ir(source.as_bytes(), |program| {
        let signature = |name| {
            let function = program
                .functions()
                .iter()
                .find(|function| function.name() == name)
                .expect("ordinary function exists");
            crate::backend::abi::FunctionAbi::build(program, function).expect("one callable ABI")
        };
        assert_eq!(signature("copy_bytes"), signature("host_copy_bytes"));
    });
    // Exercise each supported command-line configuration through its real
    // driver API: default, explicit --no-overlap, and --par. The first two
    // select Off today; neither is an arithmetic-proof acceptance switch.
    for overlap in [None, Some(OverlapLowering::Off), Some(OverlapLowering::On)] {
        let inputs = [SourceInput::new("test.wf", source.as_bytes())];
        let llvm = match overlap {
            None => crate::compile(&inputs, crate::CompilerLimits::default()),
            Some(overlap) => {
                crate::compile_with_overlap(&inputs, crate::CompilerLimits::default(), overlap)
            }
        };
        let llvm = llvm.expect("ordinary view wrapper compiles in each driver mode");
        let result = compile_and_run_with(&llvm, &[b"a\xffb"]);
        assert!(
            result.status.success(),
            "same-signature view call: {result:?}"
        );
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn behavior_actuals_preserve_ordinary_view_calls_rows_and_contracts() {
    let formal = r#"formal Copier {
  fn transfer(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination) contract {
    requires start <= end;
    requires end <= len_of(deref(destination));
    ensures when Ok(value: next): start <= next;
    ensures when Ok(value: next): next <= end;
  };
}

"#;
    let forwarding = COPY_BYTES_WRAPPER
        .replacen("fn copy_bytes(", "fn copy_through<Copier>(", 1)
        .replace("host_copy_bytes(", "Copier::transfer(");
    let original = String::from_utf8(corpus_source("run-syshost-nontext-argv-bytes-roundtrip"))
        .expect("source is UTF-8");
    let caller = original.replace("host_copy_bytes(", "copy_through::<Selected>(");
    for member in ["host_copy_bytes", "copy_bytes"] {
        let actual = format!("actual Selected : Copier {{\n  transfer = {member};\n}}\n\n");
        let prefix = format!("{formal}{actual}{COPY_BYTES_WRAPPER}{forwarding}");
        let source = format!("{prefix}{caller}");
        let forwarding_name = with_ir(source.as_bytes(), |program| {
            let names = program
                .functions()
                .iter()
                .filter(|function| function.name().starts_with("copy_through$instance$"))
                .map(|function| function.name().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(names.len(), 1, "one concrete forwarding instance");
            names[0].clone()
        });
        for overlap in [None, Some(OverlapLowering::Off), Some(OverlapLowering::On)] {
            let inputs = [SourceInput::new("test.wf", source.as_bytes())];
            let llvm = match overlap {
                None => crate::compile(&inputs, crate::CompilerLimits::default()),
                Some(overlap) => {
                    crate::compile_with_overlap(&inputs, crate::CompilerLimits::default(), overlap)
                }
            }
            .expect("WF and linked actuals compile through the ordinary call path");
            let body = emitted_function(&llvm, &forwarding_name);
            assert!(body.contains(&format!("call void @wf_{member}(")), "{body}");
            let output = compile_and_run_with(&llvm, &[b"a\xffb"]);
            assert!(output.status.success(), "{member}: {output:?}");
            assert!(output.stdout.is_empty());
            assert!(output.stderr.is_empty());
        }

        let out_of_range = format!("{prefix}{}", caller.replace("end: 4_u64", "end: 5_u64"));
        assert_eq!(
            compile_rejection(out_of_range.as_bytes()).rule_id(),
            Some("FN-8")
        );
        for changed_formal in [
            formal.replace(", writes(destination)", ""),
            formal.replace("    ensures when Ok(value: next): next <= end;\n", ""),
        ] {
            // FN-4 checks the binding itself, before any generic caller is
            // needed. Keeping the forwarding body out of these negatives
            // avoids its separate row or postcondition failure under the
            // deliberately narrowed formal boundary.
            let mismatched = format!("{changed_formal}{actual}{COPY_BYTES_WRAPPER}");
            let mismatched = format!("{}\n", mismatched.trim_end());
            let failure = compile_rejection(mismatched.as_bytes());
            assert_eq!(failure.rule_id(), Some("FN-4"), "{member}: {failure:?}");
        }
    }
}

#[test]
fn args_count_reports_the_complete_invocation_vector() {
    run_arguments("run-sysarg-count-and-get", &[b"alpha", b"beta"]);
}

#[test]
fn relative_path_admits_by_construction_and_never_normalizes() {
    run_arguments("run-syspath-relative-basic", &[b"fixture.txt"]);
    run_arguments(
        "run-syspath-dotdot-preserved",
        &[b"./inner/../inner/fixture.txt"],
    );
    run_arguments("run-syspath-absolute-rejected", &[b"/absent"]);
}

#[test]
fn the_text_route_validates_completely_and_preserves_a_refused_destination() {
    run_arguments("run-syshost-nontext-argv-utf8-invalid", &[b"a\xffb"]);
    run_arguments("run-syshost-copyutf8-invalid-unchanged", &[b"a\xffb"]);
}

#[test]
fn a_copy_into_a_short_destination_is_recoverable_and_writes_no_byte() {
    run_arguments("run-syshost-copybytes-toosmall-unchanged", &[b"abcdef"]);
    run_arguments("run-syshost-copyutf8-toosmall-unchanged", &[b"abcdef"]);
}

#[test]
fn an_out_of_range_copy_is_an_ordinary_requirement_rejection() {
    for name in [
        "reject-syshost-copybytes-start-after-end",
        "reject-syshost-copybytes-end-beyond-buffer",
        "reject-syshost-copybytes-start-beyond-buffer",
    ] {
        assert_eq!(
            compile_rejection(&corpus_source(name)).rule_id(),
            Some("FN-8")
        );
    }
}

#[test]
fn a_nonzero_transfer_returns_the_absolute_next_endpoint() {
    run_arguments("v033-run-system-nonzero-next", &[b"AB"]);
}

#[test]
fn an_entry_selecting_no_input_starts_and_returns_its_status() {
    let llvm = compile(
        br#"fn main() -> status: own ExitStatus pure {
  return exit_status(code: 37_u8);
}
"#,
    );
    assert_eq!(compile_and_run(&llvm).status.code(), Some(37));
}

#[test]
fn opaque_drop_has_no_implicit_native_close() {
    let llvm = compile(
        br#"fn main() -> status: own ExitStatus pure {
  let unused = exit_status(code: 9_u8);
  return exit_status(code: 0_u8);
}
"#,
    );
    assert!(!llvm.contains("call void @wf_close_"));
    assert!(compile_and_run(&llvm).status.success());
}
