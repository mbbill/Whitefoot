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
    ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput, TerminalLimits,
    TerminalOutcome, audit_canonical, check_semantics, classify_terminals, finalize, lex,
    lower_checked, parse,
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
    let SemanticOutcome::Complete(checked) = check_semantics(resolved) else {
        panic!("system test source must check");
    };
    let mut ir = lower_checked(*checked).expect("checked system program must lower");
    run(&mut ir)
}

/// Reads one argument's bytes and returns their wrapping sum as the status.
const ARGUMENT_CHECKSUM: &[u8] = br#"fn checksum(value: own HostString) -> result: own u64 allocates(heap), traps {
  region 'v {
    let length = host_bytes_len<'v>(value: &'v value);
    let bytes = buffer_new(length, 0_u8);
    region 'd {
      let copied = host_copy_bytes<'v, 'd>(value: &'v value, destination: &uniq 'd bytes, start: 0_u64, end: length);
      match move copied {
        Ok(value: next) => {
        }
        Err(error: problem) => {
          claim exact_capacity_must_fit: False() because "exact capacity must fit";
        }
      }
    }
    let total = 0_u64;
    let cursor = 0_u64;
    loop @sum {
      let done = ieq(cursor, length);
      if done {
        break @sum;
      }
      let sum_ok = ilt(cursor, length);
      claim cursor_in_bytes: sum_ok because "the sum stops at the copied length";
      let byte = bytes[cursor];
      let widened = cvt<u8, u64>(byte);
      set total = total +wrap widened;
      set cursor = cursor +wrap 1_u64;
    }
    return total;
  }
}

command fn main(command.args as args: own Args) -> status: own ExitStatus allocates(heap), traps {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        let total = checksum(value: move text);
        let narrowed = cvt<u64, u8>(total);
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
fn the_argument_lease_path_allocates_nothing_and_dispatches_on_nothing() {
    // Only the lease operations: no buffer, no copy, no text route.
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    let total = args_count<'a>(args: &'a args);
    match arg_get<'a>(args: &'a args, position: total) {
      Ok(value: text) => {
        region 'v {
          let length = host_bytes_len<'v>(value: &'v text);
          match relative_path(value: move text) {
            Ok(value: path) => {
              let narrowed = cvt<u64, u8>(length);
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
    // A missing argument is `InvalidIndex()` and returns no value [SYS-6].
    let absent = compile_and_run_with(&llvm, &[]);
    assert_eq!(absent.status.code(), Some(252));
}

#[test]
fn args_count_reports_the_complete_invocation_vector() {
    let source =
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    let total = args_count<'a>(args: &'a args);
    let narrowed = cvt<u64, u8>(total);
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
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
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
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        region 'v {
          match host_utf8_len<'v>(value: &'v text) {
            Ok(value: length) => {
              let narrowed = cvt<u64, u8>(length);
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
    let source = br#"command fn main(command.args as args: own Args) -> status: own ExitStatus allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        let bytes = buffer_new(2_u64, 7_u8);
        region 'v {
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd bytes, start: 0_u64, end: 2_u64) {
              Ok(value: next) => {
                return exit_status(code: 10_u8);
              }
              Err(error: problem) => {
                match move problem {
                  CopyTooSmall(required: needed) => {
                    let untouched = bytes[0_u64];
                    if ieq(untouched, 7_u8) {
                      let narrowed = cvt<u64, u8>(needed);
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
    let source = br#"command fn main(command.args as args: own Args) -> status: own ExitStatus allocates(heap) {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        let bytes = buffer_new(2_u64, 7_u8);
        region 'v {
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd bytes, start: 1_u64, end: 5_u64) {
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
    assert!(failure.detail().contains("5_u64 <= len(buffer)"));
}

#[test]
fn every_release_action_emits_exactly_its_contract() {
    let source = br#"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus external, blocks {
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    // The one native close attempt is the `DirectoryRead` release; `Args`,
    // both `Output` owners, and the returned `ExitStatus` release with no host
    // call at all [SYS-5].
    assert_eq!(llvm.matches("call i32 @close(i32").count(), 1);
    assert!(llvm.contains("declare i32 @close(i32)"));
    // A logical consume and a source detach are explicit releases that emit
    // no code; the drop marker is still present for each owner.
    assert_eq!(llvm.matches("  ; drop %v").count(), 4);
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
    assert!(llvm.contains("define i32 @main(i32 %argc, ptr %argv) {"));
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
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    let total = args_count<'a>(args: &'a args);
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
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead) -> status: own ExitStatus allocates(heap), external, blocks {
  let name = buffer_new(1_u64, 65_u8);
  region 'c {
    region 'n {
      match open_file<'c, 'n>(root: &'c cwd, name: &'n name, start: 0_u64, end: 1_u64) {
        Ok(value: file) => {
        }
        Err(error: problem) => {
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
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
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
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {
  let bytes = buffer_new(3_u64, 65_u8);
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 1_u64, end: 3_u64) {
        Ok(value: next) => {
          if ieq(next, 3_u64) {
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
    assert!(!llvm.contains("wf_trap"));
    assert!(llvm.contains("%next = add nuw i64 %start, %accepted"));
    let output = compile_and_run_with(&llvm, &[]);
    assert_eq!(output.stdout, b"AA");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn linux_enumeration_facility_without_an_abi_mapping_is_missing_mapping() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead) -> status: own ExitStatus external, blocks {
  region 'c {
    match open_list<'c>(directory: &'c cwd) {
      Ok(value: list) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let linux = SystemTarget::for_triple("x86_64-unknown-linux-gnu")
            .expect("Linux is a recognized system target");
        let failure = qualify_program(linux, program)
            .expect_err("Linux has enumeration but no approved getdents ABI mapping");
        assert!(matches!(
            failure,
            crate::BackendFailure::TargetQualification(QualificationFailure::MissingMapping(_))
        ));
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
            "fstat",
            144,
            4,
        ),
        (
            "x86_64-apple-darwin",
            1023,
            0x0010_0000,
            0x0010_0100,
            0x0000_0104,
            "fstat$INODE64",
            144,
            4,
        ),
        (
            "aarch64-unknown-linux-gnu",
            255,
            0x0000_4000,
            0x0000_c000,
            0x0000_8800,
            "fstat",
            128,
            16,
        ),
        (
            "x86_64-unknown-linux-gnu",
            255,
            0x0001_0000,
            0x0003_0000,
            0x0002_0800,
            "fstat",
            144,
            24,
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus external, blocks {
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
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
        br#"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  region 'a {
    let count = args_count<'a>(args: &'a args);
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

    let composite = br#"command fn main(command.args as args: own Args, command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
      }
      Err(error: absent) => {
      }
    }
  }
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s bytes, start: 0_u64, end: 1_u64) {
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
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead) -> status: own ExitStatus allocates(heap), external, blocks {
  let name = buffer_new(1_u64, 65_u8);
  region 'c {
    region 'n {
      match open_file<'c, 'n>(root: &'c cwd, name: &'n name, start: 0_u64, end: 1_u64) {
        Ok(value: file) => {
        }
        Err(error: problem) => {
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
            let status = target.file_status_symbol();
            assert!(llvm.contains(&format!(
                "call i32 @{status}(i32 %descriptor, ptr %file.status)"
            )));
            assert!(llvm.contains("%file.kind = and i32 %mode, 61440"));
            assert!(llvm.contains("%regular = icmp eq i32 %file.kind, 32768"));
            assert!(llvm.contains("br i1 %regular, label %live, label %kind.failure"));
            assert_eq!(
                llvm.matches("call i32 @close(i32 %descriptor)").count(),
                2,
                "each provisional-error path must make one close attempt"
            );
            assert!(llvm.contains(
                "%inspection.close = call i32 @close(i32 %descriptor)\n  \
                 br label %inspection.error"
            ));
            assert!(llvm.contains(
                "%kind.close = call i32 @close(i32 %descriptor)\n  \
                 br label %kind.select"
            ));
            assert!(!llvm.contains("%inspection.released"));
            assert!(!llvm.contains("%kind.released"));
            assert!(!llvm.contains("tcb.defect:"));
            let _optimized = host_optimized_module(&llvm);
        }
    });
}

#[test]
fn darwin_list_once_keeps_range_and_record_extents_distinct_and_verifiable() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead) -> status: own ExitStatus allocates(heap), external, blocks {
  let destination = buffer_new(64_u64, 0_u8);
  region 'c {
    match open_list<'c>(directory: &'c cwd) {
      Ok(value: list) => {
        region 'l {
          region 'd {
            let outcome = list_once<'l, 'd>(list: &uniq 'l list, destination: &uniq 'd destination, start: 0_u64, end: 64_u64);
          }
        }
      }
      Err(error: problem) => {
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
            .expect("Darwin must qualify and emit list_once")
            .into_string()
    });
    assert_eq!(llvm.matches("%record.extent = zext").count(), 1);
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
    let optimized = host_optimized_module(&llvm);
    assert!(optimized.contains("@__getdirentries64"));
}
