//! Emitted-shape and behaviour evidence for the qualified [SYS-2] system
//! interface: the [QUAL-1] qualification table, the [QUAL-3] command
//! bootstrap, and the argument/path operation cluster.
//!
//! The cost-shape assertions read the module the host optimizer leaves, which
//! is what [QUAL-3] says establishes the required emitted shape: inspection of
//! emitted code and symbols, not a machine-checked language judgment.

use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::{
    CodeUnitFamily, Facility, QualificationFailure, SystemTarget, TargetGuarantee, qualify_program,
};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalOutcome, FinalizeOutcome, IrProgram, LexOutcome,
    OverlapLowering, ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput,
    SystemIntegerResultBound, TerminalLimits, TerminalOutcome, audit_canonical, check_semantics,
    classify_terminals, finalize, lex, lower_checked, parse,
};

use super::{
    CANONICAL_LIMITS, FINALIZE_LIMITS, LEX_LIMITS, PARSE_LIMITS, SOURCE_LIMITS, compile,
    compile_and_run_with, compile_rejection, host_optimized_module, optimized_main,
};

pub(super) fn with_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_ir_for(source, crate::Inventory::ACTIVE, run)
}

pub(super) fn with_mutated_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &mut IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_mutated_ir_for(source, crate::Inventory::ACTIVE, run)
}

/// [`with_ir`] against one named [SYS-2] inventory state.
///
/// The cost-shape anchor is a real corpus program, and that program now uses
/// active `open_file` [SYS-11], so it names the inventory that declares it.
/// Every other caller takes the active one.
pub(super) fn with_ir_for<ResultValue>(
    source: &[u8],
    inventory: crate::Inventory,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_mutated_ir_for(source, inventory, |program| run(program))
}

fn with_mutated_ir_for<ResultValue>(
    source: &[u8],
    inventory: crate::Inventory,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &mut IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_mutated_ir_for_overlap(source, inventory, OverlapLowering::Off, run)
}

/// [`with_mutated_ir`] under the shipped completion lowering.
///
/// `OverlapLowering::Off` consults no permission group at all, so a program
/// lowered that way carries no completion schedule to mutate. A probe of the
/// completion emitter needs the lowering the compiler actually ships, which is
/// the one that actualizes direct target operations and no compute group.
pub(super) fn with_mutated_completion_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &mut IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_mutated_ir_for_overlap(
        source,
        crate::Inventory::ACTIVE,
        OverlapLowering::Completion,
        run,
    )
}

/// [`with_ir`] under the opt-in compute overlap lowering.
///
/// Target-shape tests use this to inspect the same handed-out IR that
/// `whitefootc --par` emits without changing the shared host-target helpers.
pub(super) fn with_parallel_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_mutated_ir_for_overlap(
        source,
        crate::Inventory::ACTIVE,
        OverlapLowering::On,
        |program| run(program),
    )
}

fn with_mutated_ir_for_overlap<ResultValue>(
    source: &[u8],
    inventory: crate::Inventory,
    overlap: OverlapLowering,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &mut IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS).expect("valid test bundle");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("system test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("system test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("system test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("system test source must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("system test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = crate::resolve_with_inventory(canonical, inventory)
    else {
        panic!("system test source must resolve");
    };
    let checked = match check_semantics(resolved) {
        SemanticOutcome::Complete(checked) => checked,
        other => panic!("system test source must check: {other:?}"),
    };
    let mut ir = lower_checked(*checked, overlap).expect("checked system program must lower");
    run(&mut ir)
}

/// Reads one argument's bytes and returns their wrapping sum as the status.
const ARGUMENT_CHECKSUM: &[u8] = br#"fn checksum(value: own HostString) -> result: own u64 reads(value), allocates(heap) {
  region 'v {
    let length = host_bytes_len(value: &value);
    let bytes = buffer_new(length, 0_u8);
    region {
      let copied = host_copy_bytes(value: &'v value, destination: &uniq bytes, start: 0_u64, end: length);
      match move copied {
        Ok(value: next) => {
        }
        Err(error: problem) => {
          return 18446744073709551615_u64;
        }
      }
    }
    let total = 0_u64;
    let cursor = 0_u64;
    loop @sum {
      let done = cursor == length;
      if done {
        break @sum;
      }
      let sum_ok = cursor < length;
      if sum_ok {
        let byte = bytes[cursor];
        let widened = cvt::<u8, u64>(byte);
        set total = total +wrap widened;
        set cursor = cursor +wrap 1_u64;
      } else {
        return 18446744073709551615_u64;
      }
    }
    return total;
  }
}

command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        let total = checksum(value: move text);
        let narrowed = cvt::<u64, u8>(total);
        match narrowed {
          Ok(value: code) => {
            return exit_status(code: code);
          }
          Err(error: overflowed) => {
            return exit_status(code: 254_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 252_u8);
      }
    }
  }
}
"#;

#[test]
fn host_bytes_len_qualification_bounds_the_exact_semantic_result() {
    with_ir(ARGUMENT_CHECKSUM, |program| {
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        let qualification =
            qualify_program(target, program).expect("the argument program must qualify");
        let mut found = false;
        for operation in program
            .functions()
            .iter()
            .flat_map(|function| function.blocks())
            .flat_map(|block| block.instructions())
            .filter_map(|instruction| match instruction {
                crate::IrInstruction::Define {
                    operation: crate::IrOperation::SystemCall { operation, .. },
                    ..
                } => Some(*operation),
                _ => None,
            })
        {
            let implementation = qualification
                .operation(operation)
                .expect("the used semantic identity has one selected row");
            let catalog_bound =
                crate::SYSTEM_OPERATIONS[usize::from(operation.ordinal())].integer_result_bound;
            assert_eq!(
                implementation.integer_result_bound(),
                catalog_bound,
                "qualification must copy semantic ID {}'s fixed result contract",
                operation.ordinal()
            );
            if operation.ordinal() == 2 {
                found = true;
                assert_eq!(
                    implementation.integer_result_bound(),
                    Some(SystemIntegerResultBound::AddressIndexMaximum)
                );
            } else {
                assert_eq!(implementation.integer_result_bound(), None);
            }
        }
        assert!(found, "the fixture must contain semantic ID 2");
    });
}

#[test]
fn the_argument_lease_path_allocates_nothing_and_dispatches_on_nothing() {
    // Only the lease operations: no buffer, no copy, no text route.
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    let total = args_count(args: &args);
    match arg_get(args: &args, position: total) {
      Ok(value: text) => {
        region {
          let length = host_bytes_len(value: &text);
          match relative_path(value: move text) {
            Ok(value: path) => {
              let narrowed = cvt::<u64, u8>(length);
              match narrowed {
                Ok(value: code) => {
                  return exit_status(code: code);
                }
                Err(error: overflowed) => {
                  return exit_status(code: 200_u8);
                }
              }
            }
            Err(error: rejected) => {
              return exit_status(code: 201_u8);
            }
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;
    let llvm = compile(source);
    // Selection is static for the whole build: each semantic identity resolved
    // to exactly one private ABI symbol before emission [QUAL-1].
    assert!(llvm.contains("; QUAL-1 semantic id 0 -> @wf.sys.args_count.v1"));
    assert!(llvm.contains("; QUAL-1 semantic id 1 -> @wf.sys.arg_get.v1"));
    assert!(llvm.contains("; QUAL-1 semantic id 2 -> @wf.sys.host_bytes_len.v1"));
    assert!(llvm.contains("; QUAL-1 semantic id 6 -> @wf.sys.relative_path.v1"));
    assert!(llvm.contains("; QUAL-1 resource HostString -> InlineLease"));
    // No emitted program contains a runtime operation-ID switch, target tag,
    // per-call dispatch table, or handle-table lookup [QUAL-3].
    assert!(!llvm.contains("@wf.sys.dispatch"));

    let optimized = host_optimized_module(&llvm);
    let entry = optimized_main(&optimized);
    // The compiler wrapper is inlined, which is the condition of
    // qualification [QUAL-3].
    assert!(
        !entry.contains("@wf.sys."),
        "no approved-implementation wrapper survives on the lease path:\n{entry}"
    );
    // A lease is an inline pointer and length: no allocation, no byte copy,
    // and no handle lookup [HOST-3, SYS-9, PATH-1].
    for forbidden in ["@malloc", "@free", "@llvm.memcpy", "@realloc"] {
        assert!(
            !entry.contains(forbidden),
            "the lease path must not contain {forbidden}:\n{entry}"
        );
    }
    // No indirect call: every call names a symbol.
    assert!(
        !entry.contains("call void %") && !entry.contains("call i64 %"),
        "the lease path must contain no indirect call:\n{entry}"
    );
}

#[test]
fn a_non_utf8_argument_round_trips_its_exact_bytes() {
    let llvm = compile(ARGUMENT_CHECKSUM);
    // The selected operation contract and the retained source allocation
    // proof discharge target representability before this module is emitted.
    assert!(!llvm.contains("wf_target_domain_abort"));
    assert!(!llvm.contains("target-domain"));
    // A zero-length buffer may legally carry a null allocation result.  The
    // two host-copy wrappers therefore form their zero-byte destination with
    // ordinary pointer arithmetic; an `inbounds` null-plus-zero expression
    // would be poison even though the following memcpy touches no byte.
    let target = "%target = getelementptr i8, ptr %base, i64 %start";
    let inbounds = "%target = getelementptr inbounds i8, ptr %base, i64 %start";
    assert_eq!(llvm.matches(target).count(), 1);
    assert!(!llvm.contains(inbounds));
    let utf8 = compile(include_bytes!(
        "../../../../tests/conformance/cases/run-syshost-copyutf8-toosmall-unchanged.wf"
    ));
    assert_eq!(utf8.matches(target).count(), 1);
    assert!(!utf8.contains(inbounds));
    // 0xff is not valid UTF-8 anywhere; the lossless route reports and copies
    // the target's own code units with no validation [HOST-2, SYS-9].
    let single = compile_and_run_with(&llvm, &[&[0xff]]);
    assert_eq!(single.status.code(), Some(255));
    // Two bytes, neither valid text on its own: 0x80 + 0x41 = 193.
    let pair = compile_and_run_with(&llvm, &[&[0x80, 0x41]]);
    assert_eq!(pair.status.code(), Some(193));
    // An ordinary text argument travels the same route: 'a' + 'b' = 195.
    let text = compile_and_run_with(&llvm, &[b"ab"]);
    assert_eq!(text.status.code(), Some(195));
    let empty = compile_and_run_with(&llvm, &[b""]);
    assert_eq!(empty.status.code(), Some(0));
    // A missing argument is `InvalidIndex()` and returns no value [SYS-6].
    let absent = compile_and_run_with(&llvm, &[]);
    assert_eq!(absent.status.code(), Some(252));
}

#[test]
fn args_count_reports_the_complete_invocation_vector() {
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    let total = args_count(args: &args);
    let narrowed = cvt::<u64, u8>(total);
    match narrowed {
      Ok(value: code) => {
        return exit_status(code: code);
      }
      Err(error: overflowed) => {
        return exit_status(code: 255_u8);
      }
    }
  }
}
"#;
    let llvm = compile(source);
    // The snapshot is the complete native argument vector the invocation
    // supplied, including the invoked name at position 0: nothing is dropped
    // or truncated on the way to source [FN-7, HOST-1].
    assert_eq!(compile_and_run_with(&llvm, &[]).status.code(), Some(1));
    assert_eq!(
        compile_and_run_with(&llvm, &[b"one", b"two"]).status.code(),
        Some(3)
    );
}

#[test]
fn relative_path_admits_by_construction_and_never_normalizes() {
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            return exit_status(code: 10_u8);
          }
          Err(error: rejected) => {
            return exit_status(code: 20_u8);
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 30_u8);
      }
    }
  }
}
"#;
    let llvm = compile(source);
    // `.` and `..` components and every separator are preserved exactly and
    // admitted; only a target-root prefix is refused [PATH-1].
    assert_eq!(
        compile_and_run_with(&llvm, &[b"../a/./b"]).status.code(),
        Some(10)
    );
    assert_eq!(
        compile_and_run_with(&llvm, &[b"plain"]).status.code(),
        Some(10)
    );
    // The empty sequence contains no NUL and begins with no root prefix.
    assert_eq!(compile_and_run_with(&llvm, &[b""]).status.code(), Some(10));
    // One leading separator is this family's target-root prefix.
    assert_eq!(
        compile_and_run_with(&llvm, &[b"/abs"]).status.code(),
        Some(20)
    );
    // Construction consumes its input on failure too, so nothing leaks.
    let optimized = host_optimized_module(&llvm);
    assert!(!optimized_main(&optimized).contains("@malloc"));
}

#[test]
fn the_text_route_validates_completely_and_reports_the_exact_encoded_length() {
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        region {
          match host_utf8_len(value: &text) {
            Ok(value: length) => {
              let narrowed = cvt::<u64, u8>(length);
              match narrowed {
                Ok(value: code) => {
                  return exit_status(code: code);
                }
                Err(error: overflowed) => {
                  return exit_status(code: 200_u8);
                }
              }
            }
            Err(error: invalid) => {
              return exit_status(code: 201_u8);
            }
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;
    let llvm = compile(source);
    for (argument, status) in [
        (b"abc".as_slice(), 3),
        // Two-byte, three-byte, and four-byte encodings.
        ("é".as_bytes(), 2),
        ("€".as_bytes(), 3),
        ("😀".as_bytes(), 4),
        // A lone continuation byte, an overlong encoding, a surrogate, and a
        // value above U+10FFFF are each rejected without a replacement code
        // point and without a truncated encoding [HOST-2].
        (&[0x80], 201),
        (&[0xc0, 0xaf], 201),
        (&[0xed, 0xa0, 0x80], 201),
        (&[0xf5, 0x80, 0x80, 0x80], 201),
        // A truncated sequence at the end of the value is invalid, not short.
        (&[0xe2, 0x82], 201),
    ] {
        assert_eq!(
            compile_and_run_with(&llvm, &[argument]).status.code(),
            Some(status),
            "argument {argument:02x?}"
        );
    }
}

#[test]
fn a_copy_into_a_short_destination_is_recoverable_and_writes_no_byte() {
    let source = br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        let bytes = buffer_new(2_u64, 7_u8);
        region 'v {
          region {
            match host_copy_bytes(value: &'v text, destination: &uniq bytes, start: 0_u64, end: 2_u64) {
              Ok(value: next) => {
                return exit_status(code: 10_u8);
              }
              Err(error: problem) => {
                match move problem {
                  CopyTooSmall(required: needed) => {
                    let untouched = bytes[0_u64];
                    if untouched == 7_u8 {
                      let narrowed = cvt::<u64, u8>(needed);
                      match narrowed {
                        Ok(value: code) => {
                          return exit_status(code: code);
                        }
                        Err(error: overflowed) => {
                          return exit_status(code: 200_u8);
                        }
                      }
                    } else {
                      return exit_status(code: 201_u8);
                    }
                  }
                }
              }
            }
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 202_u8);
      }
    }
  }
}
"#;
    let llvm = compile(source);
    // A destination exactly large enough succeeds.
    assert_eq!(
        compile_and_run_with(&llvm, &[b"ab"]).status.code(),
        Some(10)
    );
    // A short destination reports the exact required length and leaves the
    // whole destination buffer unchanged [SYS-8].
    assert_eq!(
        compile_and_run_with(&llvm, &[b"abcde"]).status.code(),
        Some(5)
    );
}

#[test]
fn an_out_of_range_copy_is_a_static_sys8_rejection() {
    let source = br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args), allocates(heap) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Ok(value: text) => {
        let bytes = buffer_new(2_u64, 7_u8);
        region 'v {
          region {
            match host_copy_bytes(value: &'v text, destination: &uniq bytes, start: 1_u64, end: 5_u64) {
              Ok(value: next) => {
                return exit_status(code: 10_u8);
              }
              Err(error: problem) => {
                return exit_status(code: 20_u8);
              }
            }
          }
        }
      }
      Err(error: absent) => {
        return exit_status(code: 30_u8);
      }
    }
  }
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("SYS-8"));
    // The residual names the caller's own buffer, not the operation's
    // declared parameter: with two buffers in scope, `len(buffer)` did not say
    // which one the bound is about.
    assert!(failure.detail().contains("5_u64 <= len(bytes)"));
}

#[test]
fn every_release_action_emits_exactly_its_contract() {
    let source = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus writes(cwd) {
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    // The one native close attempt is the `DirectoryRead` release; `Args`,
    // both `Output` owners, `FileFactory`, and the returned `ExitStatus`
    // release with no host call at all [SYS-5].
    assert_eq!(
        llvm.matches("call i32 @wf__completion_file_close_direct(i32")
            .count(),
        1
    );
    assert!(llvm.contains("declare i32 @wf__completion_file_close_direct(i32)"));
    // A logical consume and a source detach are explicit releases that emit
    // no code; the drop marker is still present for each owner.
    assert_eq!(llvm.matches("  ; drop %v").count(), 5);
    assert!(
        llvm.contains("i32 %cwd, i32 1, i32 2, i1 true"),
        "the bootstrap must supply the proof-only FileFactory after stderr"
    );
}

#[test]
fn the_command_bootstrap_normalizes_once_and_maps_the_returned_status_exactly() {
    let source =
        br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus pure {
  return exit_status(code: 42_u8);
}
"#;
    let llvm = compile(source);
    // One-time per-invocation normalization belongs to the bootstrap before
    // entry, never to a transfer [QUAL-3].
    assert_eq!(llvm.matches("@signal(i32 13,").count(), 1);
    assert!(llvm.contains("br i1 %installed, label %inputs, label %start.failure"));
    // The [QUAL-2] backing guarantee is established before entry, and a target
    // that cannot establish it refuses startup rather than entering.
    assert!(llvm.contains("%backing = and i1 %argv.present, %argc.counted"));
    assert!(llvm.contains("call void @exit(i32 71)"));
    // The entry is invoked once and its returned code becomes the process
    // status exactly [PROG-3, SYS-13].
    assert_eq!(llvm.matches("call i8 @wf_main(").count(), 1);
    assert!(llvm.contains("%code = zext i8 %status to i32"));
    assert_eq!(compile_and_run_with(&llvm, &[]).status.code(), Some(42));
}

#[test]
fn an_entry_selecting_no_input_still_starts_and_returns_its_status() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 7_u8);
}
"#;
    let llvm = compile(source);
    assert!(!llvm.contains("@open("));
    assert_eq!(compile_and_run_with(&llvm, &[]).status.code(), Some(7));
}

#[test]
fn a_no_input_entry_still_uses_the_command_bootstrap() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv) #0 {"));
    assert_eq!(llvm.matches("@signal(i32 13,").count(), 1);
    assert!(llvm.contains("%backing = and i1 %argv.present, %argc.counted"));
    assert_eq!(compile_and_run_with(&llvm, &[]).status.code(), Some(0));
}

#[test]
fn a_command_without_system_operations_still_crosses_target_qualification() {
    let source = br#"fn spin() -> status: own ExitStatus pure {
  return spin();
}

command fn main() -> status: own ExitStatus pure {
  return spin();
}
"#;
    with_ir(source, |program| {
        let system_calls = program
            .functions()
            .iter()
            .flat_map(|function| function.blocks())
            .flat_map(|block| block.instructions())
            .filter(|instruction| {
                matches!(
                    instruction,
                    crate::IrInstruction::Define {
                        operation: crate::IrOperation::SystemCall { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(system_calls, 0);
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        qualify_program(target, program)
            .expect("the command entry is qualified even without an operation row");
    });
}

#[test]
fn a_target_without_the_argument_backing_guarantee_fails_qualification() {
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    let total = args_count(args: &args);
    return exit_status(code: 0_u8);
  }
}
"#;
    with_ir(source, |program| {
        // The qualified host target accepts the same program.
        let qualified = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe base triple is a qualified target");
        qualify_program(qualified, program).expect("a qualified target admits the program");

        // A target that can supply neither stable native argument backing nor
        // a pre-entry snapshot fails qualification for the command entry and
        // for argument access [QUAL-2]; it never enters under a weaker
        // guarantee and never narrows the semantic ID.
        let unbacked = SystemTarget::probe(Some(CodeUnitFamily::Unix), false, true);
        let failure = qualify_program(unbacked, program)
            .expect_err("a target without command-lifetime argument backing must fail");
        assert!(matches!(
            failure,
            crate::BackendFailure::TargetQualification(QualificationFailure::UnmetGuarantee { .. })
        ));

        // A target belonging to no lossless code-unit family still qualifies
        // for argument counting: [HOST-1] withholds exactly the host-string
        // and path semantic IDs, not the whole interface.
        let lossy = SystemTarget::probe(None, true, true);
        qualify_program(lossy, program)
            .expect("counting arguments needs no lossless code-unit family");
    });

    // The same lossy target fails for a program that leases code units.
    let leases =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    match arg_get(args: &args, position: 0_u64) {
      Ok(value: text) => {
        return exit_status(code: 0_u8);
      }
      Err(error: absent) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#;
    with_ir(leases, |program| {
        let lossy = SystemTarget::probe(None, true, true);
        let failure = qualify_program(lossy, program)
            .expect_err("a target with no lossless code-unit family cannot lease code units");
        assert!(matches!(
            failure,
            crate::BackendFailure::TargetQualification(QualificationFailure::UnmetGuarantee { .. })
        ));
    });
}

#[test]
fn a_target_without_directory_relative_resolution_rejects_component_opening() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let name = buffer_new(1_u64, 65_u8);
  region 'c {
    region {
      match reserve_file(factory: &uniq 'c files) {
        Ok(value: permit) => {
          match open_file(permit: move permit, root: &'c cwd, name: &name, start: 0_u64, end: 1_u64) {
            FileOpened(value: file) => {
            }
            FileOpenFailed(error: problem, permit: refused_2) => {
            }
          }
        }
        Err(error: spent) => {
          return exit_status(code: 8_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let target = SystemTarget::probe(Some(CodeUnitFamily::Unix), true, false);
        assert_eq!(
            qualify_program(target, program),
            Err(crate::BackendFailure::TargetQualification(
                QualificationFailure::UnmetGuarantee {
                    facility: Facility::Resource(crate::SystemResourceType::DirectoryRead),
                    guarantee: TargetGuarantee::DirectoryRelativeResolution,
                }
            ))
        );
    });
}

#[test]
fn every_semantic_identity_resolves_before_layout_and_emission() {
    // The qualification table is consulted once, after the exact target is
    // selected and before any use of an operation is emitted, and it now has
    // an approved implementation for every [SYS-2] identity on this target.
    // The I/O cluster's own emission evidence is in `system_io.rs`.
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &bytes, start: 0_u64, end: 1_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;
    let inputs = [SourceInput::new("test.wf", source)];
    let llvm = crate::compile(&inputs, crate::CompilerLimits::default())
        .expect("a qualified writing command must emit");
    assert!(llvm.contains("; QUAL-1 semantic id 9 -> @wf.sys.write_once.v1"));
    let output = compile_and_run_with(&llvm, &[]);
    assert_eq!(output.stdout, b"A");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_nonzero_transfer_returns_the_absolute_next_endpoint() {
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &bytes, start: 1_u64, end: 3_u64) {
        Ok(value: next) => {
          if next == 3_u64 {
            return exit_status(code: 0_u8);
          } else {
            return exit_status(code: 2_u8);
          }
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;
    let llvm = compile(source);
    assert!(llvm.contains("%bounded = icmp ule i64 %accepted, %extent"));
    assert!(llvm.contains("br i1 %bounded, label %ok, label %tcb.defect"));
    // Every writer-controlled partial operation in this source was proved
    // before lowering, so the transfer path may not gain a source-originated
    // runtime check. Heap availability remains a separate resource path, and
    // the qualified host wrapper retains its internal consistency failure.
    assert!(!llvm.contains("call void @wf_trap(ptr @.wf_trap."));
    assert!(llvm.contains("call void @wf_resource_abort()"));
    assert!(llvm.contains("tcb.defect:\n  call void @abort()\n  unreachable"));
    assert!(llvm.contains("%next = add nuw i64 %start, %accepted"));
    let output = compile_and_run_with(&llvm, &[]);
    assert_eq!(output.stdout, b"AA");
    assert_eq!(output.status.code(), Some(0));
}

/// Every triple this compiler recognizes now has an approved [SYS-14]
/// enumeration row, and a target with no enumeration facility at all still
/// fails qualification for those semantic IDs rather than having a scan built
/// for it out of other operations [QUAL-2].
///
/// This case superseded the one that recorded the opposite for Linux. That
/// case asserted `MissingMapping(Operation(12))` on a Linux triple because
/// this compiler had no `getdents64` record model; the model landed with the
/// Linux row, so the condition it named no longer exists on any recognized
/// triple. What remains true, and is what a table without a Linux row was
/// only ever standing in for, is the refusal below.
#[test]
fn a_target_without_an_enumeration_facility_fails_the_enumeration_guarantee() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        match open_directory_source(permit: move permit, directory: &cwd) {
          SourceOpened(value: list) => {
          }
          SourceOpenFailed(error: problem, permit: refused_2) => {
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        for triple in [
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
        ] {
            let target = SystemTarget::for_triple(triple).expect("a recognized system target");
            qualify_program(target, program)
                .unwrap_or_else(|failure| panic!("{triple} must qualify: {failure:?}"));
        }
        let bare = SystemTarget::probe_without_enumeration();
        assert_eq!(
            qualify_program(bare, program),
            Err(crate::BackendFailure::TargetQualification(
                QualificationFailure::UnmetGuarantee {
                    facility: Facility::Operation(12),
                    guarantee: TargetGuarantee::DirectoryEnumeration,
                }
            ))
        );
    });
}

/// The mapping refusal is distinct from the guarantee refusal: a family that
/// has the facility, on a compiler holding no approved ABI record for it,
/// fails the enumeration semantic IDs with `MissingMapping` [QUAL-1]. No
/// recognized triple is in this state since the Linux row landed, so the
/// arm is reached through a probe rather than left unexecuted.
#[test]
fn a_facility_without_an_approved_record_is_a_missing_mapping() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        match open_directory_source(permit: move permit, directory: &cwd) {
          SourceOpened(value: list) => {
          }
          SourceOpenFailed(error: problem, permit: refused_2) => {
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let unmapped = SystemTarget::probe_without_enumeration_record();
        assert_eq!(
            qualify_program(unmapped, program),
            Err(crate::BackendFailure::TargetQualification(
                QualificationFailure::MissingMapping(Facility::Operation(12))
            ))
        );
    });
}

#[test]
fn component_open_flags_and_status_abis_are_target_exact() {
    let cases = [
        (
            "aarch64-apple-darwin",
            1023,
            0x0010_0000,
            0x0010_0100,
            0x0000_0104,
            "wf__completion_file_status_direct",
            144,
            4,
        ),
        (
            "x86_64-apple-darwin",
            1023,
            0x0010_0000,
            0x0010_0100,
            0x0000_0104,
            "wf__completion_file_status_direct",
            144,
            4,
        ),
        (
            "aarch64-unknown-linux-gnu",
            255,
            0x0000_4000,
            0x0000_c000,
            0x0000_8800,
            "wf__completion_file_status_direct",
            128,
            16,
        ),
        (
            "x86_64-unknown-linux-gnu",
            255,
            0x0001_0000,
            0x0003_0000,
            0x0002_0800,
            "wf__completion_file_status_direct",
            144,
            24,
        ),
        (
            "x86_64-pc-windows-msvc",
            510,
            0,
            1,
            1,
            "wf__completion_file_status_direct",
            8,
            0,
        ),
    ];
    for (
        triple,
        component_limit,
        directory,
        component_directory,
        component_file,
        status,
        size,
        mode,
    ) in cases
    {
        let target = SystemTarget::for_triple(triple).expect("a supported target row");
        assert_eq!(target.component_limit(), component_limit, "{triple}");
        assert_eq!(target.directory_open_flags(), directory, "{triple}");
        assert_eq!(
            target.component_directory_open_flags(),
            component_directory,
            "{triple}"
        );
        assert_eq!(target.file_open_flags(), 0, "{triple}");
        assert_eq!(
            target.component_file_open_flags(),
            component_file,
            "{triple}"
        );
        assert_eq!(target.file_status_symbol(), status, "{triple}");
        assert_eq!(target.file_status_size(), size, "{triple}");
        assert_eq!(target.file_status_mode_offset(), mode, "{triple}");
    }
}

#[test]
fn command_entry_rejects_abi_equivalent_but_semantically_wrong_ir_types() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus writes(cwd) {
  return exit_status(code: 0_u8);
}
"#;
    with_mutated_ir(source, |program| {
        let main = &program.functions()[program.main_ordinal() as usize];
        let wrong_resource = main.parameters()[1].1;
        assert!(program.retype_main_parameter_for_test(0, wrong_resource));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });

    with_mutated_ir(source, |program| {
        assert!(program.retype_main_result_for_test(crate::IrType::Integer {
            width: 8,
            signed: false,
        }));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });
}

#[test]
fn system_calls_reject_abi_equivalent_but_semantically_wrong_ir_arguments() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(cwd, out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &bytes, start: 0_u64, end: 1_u64) {
        Ok(value: next) => {
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_mutated_ir(source, |program| {
        let main = &program.functions()[program.main_ordinal() as usize];
        let directory = main.parameters()[0].1;
        assert!(program.retype_first_system_argument_for_test(0, directory));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });

    with_mutated_ir(source, |program| {
        assert!(program.retype_first_system_argument_for_test(
            1,
            crate::IrType::Buffer {
                element: crate::IrFlatElement::Integer {
                    width: 8,
                    signed: true,
                },
            },
        ));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });

    with_mutated_ir(source, |program| {
        assert!(program.retype_first_system_argument_for_test(
            2,
            crate::IrType::Integer {
                width: 64,
                signed: true,
            },
        ));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });
}

#[test]
fn system_calls_reject_wrong_scalar_and_composite_result_identities() {
    let scalar =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    let count = args_count(args: &args);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_mutated_ir(scalar, |program| {
        assert!(
            program.retype_first_system_result_for_test(crate::IrType::Integer {
                width: 64,
                signed: true,
            },)
        );
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });

    let composite = br#"command fn main(command.args as args: own Args, command.stdout as out: own Output) -> status: own ExitStatus reads(args, out), writes(out), allocates(heap) {
  region {
    match arg_get(args: &args, position: 0_u64) {
      Ok(value: text) => {
      }
      Err(error: absent) => {
      }
    }
  }
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &bytes, start: 0_u64, end: 1_u64) {
        Ok(value: next) => {
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_mutated_ir(composite, |program| {
        let first_result = program
            .functions()
            .iter()
            .flat_map(|function| function.blocks())
            .flat_map(|block| block.instructions())
            .find_map(|instruction| {
                let crate::IrInstruction::Define {
                    ty,
                    operation: crate::IrOperation::SystemCall { .. },
                    ..
                } = instruction
                else {
                    return None;
                };
                Some(*ty)
            })
            .expect("the probe contains a system call");
        let wrong_result = program
            .nominals()
            .iter()
            .find(|nominal| {
                nominal.identity() == crate::IrNominalIdentity::PreludeResult
                    && crate::IrType::Nominal(nominal.id()) != first_result
            })
            .map(|nominal| crate::IrType::Nominal(nominal.id()))
            .expect("the probe instantiates two distinct Result shapes");
        assert!(program.retype_first_system_result_for_test(wrong_result));
        let target = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("the probe target is qualified");
        assert!(matches!(
            emit_llvm_for_target(program, target),
            Err(crate::BackendFailure::InvalidIr)
        ));
    });
}

#[test]
fn open_file_validates_a_provisional_descriptor_before_publishing_it() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let name = buffer_new(1_u64, 65_u8);
  region 'c {
    region {
      match reserve_file(factory: &uniq 'c files) {
        Ok(value: permit) => {
          match open_file(permit: move permit, root: &'c cwd, name: &name, start: 0_u64, end: 1_u64) {
            FileOpened(value: file) => {
            }
            FileOpenFailed(error: problem, permit: refused_2) => {
            }
          }
        }
        Err(error: spent) => {
          return exit_status(code: 8_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        for triple in [
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
        ] {
            let target = SystemTarget::for_triple(triple).expect("a supported target row");
            let llvm = emit_llvm_for_target(program, target)
                .expect("open_file must qualify and emit")
                .into_string();
            // The permit is one credit of the factory's capacity [SYS-10]: the
            // wrapper answers the `Result` shape from the floor's count and
            // performs no host call.
            assert!(llvm.contains("@wf.sys.reserve_file.v1() alwaysinline"));
            assert!(llvm.contains("call i32 @wf__file_reserve()"));
            assert!(llvm.contains("@wf.sys.reserve_file.v1()"));
            assert!(
                !llvm.contains("@wf.sys.open_file.v1(i1"),
                "the proof-only permit must not enter the qualified open ABI"
            );
            assert!(llvm.contains("i32 1, ptr %open.error.slot, ptr %open.outcome.slot)"));
            assert!(llvm.contains("@wf.sys.open_file.completion"));
            assert!(llvm.contains("i32 3, label %kind.directory.return"));
            assert!(llvm.contains("i32 4, label %kind.other.return"));
            // The kind decision moved into the one shared rule every target
            // answers with, so the check that a provisional descriptor is
            // classified from its own mode now reads the shared header.
            assert!(crate::COMPLETION_FILE_ADAPTER_HEADER.contains("S_ISREG(file_mode)"));
            assert_eq!(
                crate::COMPLETION_FILE_ADAPTER_SOURCE
                    .matches("(void)close(descriptor);")
                    .count(),
                2,
                "status failure and nonregular classification each close once"
            );
            let _optimized = host_optimized_module(&llvm);
        }
    });
}

#[test]
fn darwin_directory_next_keeps_range_and_record_extents_distinct_and_verifiable() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let destination = buffer_new(64_u64, 0_u8);
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        match open_directory_source(permit: move permit, directory: &cwd) {
          SourceOpened(value: list) => {
            region 'l {
              region {
                let outcome = directory_next(source: &uniq 'l list, destination: &uniq destination, start: 0_u64, end: 64_u64);
              }
            }
          }
          SourceOpenFailed(error: problem, permit: refused_2) => {
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = with_ir(source, |program| {
        let darwin = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("Darwin is a recognized system target");
        emit_llvm_for_target(program, darwin)
            .expect("Darwin must qualify and emit directory_next")
            .into_string()
    });
    assert_eq!(
        llvm.matches("%record.extent = zext").count(),
        2,
        "the direct wrapper and completion mapper validate the same native record"
    );
    assert!(llvm.contains("%bounded.batch = icmp ule i64 %filled, %extent"));
    assert!(llvm.contains("br i1 %bounded.batch, label %normalize, label %tcb.defect"));
    assert!(llvm.contains("%sized = icmp uge i64 %record.extent, %needed"));
    assert!(llvm.contains("%bounded = icmp ule i64 %record.extent, %remaining"));
    assert!(llvm.contains("%source.next = add i64 %source, %record.extent"));
    assert!(llvm.contains("%fits = icmp ule i64 %after, %extent"));
    assert!(
        llvm.contains("%target.named.low = getelementptr inbounds i8, ptr %target.record, i64 1")
    );
    assert!(
        llvm.contains("%target.named.high = getelementptr inbounds i8, ptr %target.record, i64 2")
    );
    assert!(llvm.contains("%named.high.part = lshr i16 %named.short, 8"));
    assert!(llvm.contains("%target.name = getelementptr inbounds i8, ptr %target.record, i64 3"));
    assert!(llvm.contains("%copy.invalid = or i1 %copy.nul, %copy.separator"));
    assert!(llvm.contains("tcb.defect:\n  call void @abort()\n  unreachable"));
    assert!(llvm.contains("@wf__completion_directory_next_direct"));
    assert!(!llvm.contains("call i64 @__getdirentries64"));
    let optimized = host_optimized_module(&llvm);
    assert!(optimized.contains("@wf__completion_directory_next_direct"));
}

/// The Linux row derives a name's length by one scan bounded by the record's
/// own extent, because `struct linux_dirent64` states no length [SYS-14].
///
/// This is the whole difference between the two approved enumeration rows, so
/// it is asserted on the emitted text of both: the Linux shim must hold the
/// bounded scan and must load no name-length field, and the Darwin shim must
/// hold the field load and no scan. Everything else about the walk — the
/// extent validation, the in-place rewrite, the defect arm — is one text both
/// rows embed, which the two counted assertions below state.
#[test]
fn linux_directory_next_derives_the_name_length_by_a_bounded_scan() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let destination = buffer_new(64_u64, 0_u8);
  region {
    match reserve_file(factory: &uniq files) {
      Ok(value: permit) => {
        match open_directory_source(permit: move permit, directory: &cwd) {
          SourceOpened(value: list) => {
            region 'l {
              region {
                let outcome = directory_next(source: &uniq 'l list, destination: &uniq destination, start: 0_u64, end: 64_u64);
              }
            }
          }
          SourceOpenFailed(error: problem, permit: refused_2) => {
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let (linux, darwin) = with_ir(source, |program| {
        let linux = SystemTarget::for_triple("x86_64-unknown-linux-gnu")
            .expect("Linux is a recognized system target");
        let darwin = SystemTarget::for_triple("aarch64-apple-darwin")
            .expect("Darwin is a recognized system target");
        (
            emit_llvm_for_target(program, linux)
                .expect("Linux must qualify and emit directory_next")
                .into_string(),
            emit_llvm_for_target(program, darwin)
                .expect("Darwin must qualify and emit directory_next")
                .into_string(),
        )
    });

    // `struct linux_dirent64`: d_reclen at 16, d_type at 18, d_name at 19.
    assert!(
        linux.contains("%record.extent.at = getelementptr inbounds i8, ptr %entry.record, i64 16")
    );
    assert!(linux.contains("%kind.at = getelementptr inbounds i8, ptr %entry.record, i64 18"));
    assert!(linux.contains("%name.base = getelementptr inbounds i8, ptr %entry.record, i64 19"));
    // The scan is bounded by the record's own extent, and the extent is
    // checked against the reported batch before one name byte is read.
    assert!(linux.contains("%name.bounded = icmp ule i64 %record.extent, %remaining"));
    assert!(linux.contains("%name.present = icmp ugt i64 %record.extent, 19"));
    assert!(linux.contains("%name.span = sub nuw i64 %record.extent, 19"));
    assert!(linux.contains("%name.unterminated = icmp uge i64 %name.scanned, %name.span"));
    assert!(linux.contains("br i1 %name.unterminated, label %tcb.defect, label %name.scan.step"));
    // No length field is read, because the record states none.
    assert!(!linux.contains("%named.native = load i16"));
    // The component limit is the Linux family's own [SYS-14].
    assert!(linux.contains("%nameable = icmp ule i64 %named, 255"));
    assert!(darwin.contains("%nameable = icmp ule i64 %named, 1023"));
    // The Darwin row reads its field and needs no scan at all.
    assert!(darwin.contains("%named.at = getelementptr inbounds i8, ptr %entry.record, i64 18"));
    assert!(darwin.contains("%named.native = load i16, ptr %named.at, align 1"));
    assert!(!darwin.contains("%name.span = sub nuw"));
    // Both rows embed one walk, in both the direct route and the completion
    // mapper: the record model varies, the normalization does not.
    for module in [&linux, &darwin] {
        assert_eq!(module.matches("%record.extent = zext").count(), 2);
        assert_eq!(
            module
                .matches("%source.next = add i64 %source, %record.extent")
                .count(),
            2
        );
        assert_eq!(
            module
                .matches("%fits = icmp ule i64 %after, %extent")
                .count(),
            2
        );
        assert!(module.contains("tcb.defect:\n  call void @abort()\n  unreachable"));
        assert!(module.contains("@wf__completion_directory_next_direct"));
    }
    assert!(!linux.contains("call i64 @getdents64"));
}
