//! Typed-IR shape assertions for [SYS-2] resource identities and the
//! compiler-derived releases [STOR-3] places on normal control-flow edges.
//!
//! These read the IR directly rather than emitted code: target qualification
//! and native emission for the system interface are later stages, and the
//! question here is whether the IR carries the facts they will need.

#![allow(clippy::panic)]

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome,
    OverlapLowering, ParseLimits, ParseOutcome, ResolutionOutcome, SYSTEM_OPERATIONS,
    SemanticOutcome, SourceBundle, SourceInput, SourceLimits, SystemReleaseAction,
    SystemReleaseRow, SystemResourceBacking, SystemResourceType, TerminalLimits, TerminalOutcome,
    audit_canonical, check_semantics, classify_terminals, finalize, parse, resolve,
};

use super::{
    IrBlock, IrDrop, IrEntry, IrFunction, IrInstruction, IrIntegerOperation, IrNominalKind,
    IrOperation, IrProgram, IrTerminator, IrType, IrValueId, lower_checked,
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

/// The valid `command` entry these whole-program lowering fixtures need [FN-7].
const COMMAND_ENTRY: &str =
    "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

fn with_ir<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_ir_mode(source, OverlapLowering::Off, run)
}

fn with_ir_mode<ResultValue>(
    source: &[u8],
    overlap: OverlapLowering,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        &IrProgram<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("lowering test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("lowering test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("lowering test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("lowering test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("lowering test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("lowering test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("lowering test source must resolve");
    };
    let outcome = check_semantics(resolved);
    let SemanticOutcome::Complete(checked) = outcome else {
        panic!("lowering test source must check: {outcome:?}");
    };
    let ir = lower_checked(*checked, overlap).expect("checked system program must lower");
    run(&ir)
}

fn function<'program>(
    program: &'program IrProgram<'_, '_, '_>,
    name: &str,
) -> &'program IrFunction {
    program
        .functions()
        .iter()
        .find(|function| function.name() == name)
        .unwrap_or_else(|| panic!("lowered program must contain {name}"))
}

/// The one block of a straight-line function.
fn only_block(function: &IrFunction) -> &IrBlock {
    let [block] = function.blocks() else {
        panic!("expected one block, got {}", function.blocks().len());
    };
    block
}

fn return_drops(function: &IrFunction) -> &[IrDrop] {
    let IrTerminator::Return { drops, .. } = only_block(function).terminator() else {
        panic!("expected a return terminator");
    };
    drops
}

#[test]
fn counted_range_cfg_emits_with_distinct_header_update_and_exit_interfaces() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  for @items (i in 18446744073709551614_u64..18446744073709551615_u64) {
    set total = i;
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("counted IR must emit: {error:?}\n{counted:#?}");
        }
        assert!(counted.blocks().len() >= 6);
        assert!(counted.blocks().iter().any(|block| {
            matches!(
                block.terminator(),
                IrTerminator::Match {
                    enum_type: super::IrEnumType::Bool,
                    ..
                }
            )
        }));
        let hidden_updates = counted
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::Integer {
                            operation: IrIntegerOperation::AddWrap,
                            operand_type: IrType::Integer {
                                width: 64,
                                signed: false,
                            },
                            ..
                        },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(
            hidden_updates, 1,
            "MAX-1..MAX must retain exactly the compiler-owned unit update"
        );
    });
}

#[test]
fn counted_break_and_return_edges_do_not_enter_the_hidden_update() {
    let source = br#"fn leave_by_break(stop: own Bool) -> result: own u64 pure {
  for @scan (i in 0_u64..2_u64) {
    if stop {
      break @scan;
    }
  }
  return 7_u64;
}

fn leave_by_return(stop: own Bool) -> result: own u64 pure {
  for @scan (i in 0_u64..2_u64) {
    if stop {
      return 9_u64;
    }
  }
  return 7_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let hidden_update_count = |function: &IrFunction| {
            function
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .filter(|instruction| {
                    matches!(
                        instruction,
                        IrInstruction::Define {
                            operation: IrOperation::Integer {
                                operation: IrIntegerOperation::AddWrap,
                                operand_type: IrType::Integer {
                                    width: 64,
                                    signed: false,
                                },
                                ..
                            },
                            ..
                        }
                    )
                })
                .count()
        };

        let breaking = function(program, "leave_by_break");
        assert_eq!(
            hidden_update_count(breaking),
            1,
            "the normal fallthrough keeps one hidden update"
        );
        let exit_blocks = breaking
            .blocks()
            .iter()
            .enumerate()
            .filter_map(|(index, block)| {
                matches!(block.terminator(), IrTerminator::Return { .. }).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            exit_blocks.len(),
            1,
            "the break fixture has one final return"
        );
        let exit = exit_blocks[0];
        let jumps_to_exit = breaking
            .blocks()
            .iter()
            .filter(|block| {
                matches!(
                    block.terminator(),
                    IrTerminator::Jump { target, .. } if target.index() == exit
                )
            })
            .count();
        assert_eq!(
            jumps_to_exit, 2,
            "the false header and break edge must reach the exit directly"
        );

        let returning = function(program, "leave_by_return");
        assert_eq!(
            hidden_update_count(returning),
            1,
            "the non-returning branch keeps one hidden update"
        );
        let returns = returning
            .blocks()
            .iter()
            .filter(|block| matches!(block.terminator(), IrTerminator::Return { .. }))
            .count();
        assert_eq!(
            returns, 2,
            "the body return must remain a return edge beside the false-header exit"
        );
    });
}

#[test]
fn counted_range_carries_one_stable_binder_address_for_body_local_shared_borrows() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  let upper = 2_u64;
  for @items (i in 0_u64..upper) {
    region {
      let held = &i;
      let seen = deref(held);
      set total = total +wrap seen;
    }
    set upper = 0_u64;
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("addressed counted binder IR must emit: {error:?}\n{counted:#?}");
        }
        let address_count = counted
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::AddressOf { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(address_count, 1, "the binder storage is allocated once");
    });
}

#[test]
fn nested_counted_breaks_keep_each_exit_interface_local_to_its_range() {
    let source = br#"fn count() -> result: own u64 pure {
  let total = 0_u64;
  for @outer (i in 0_u64..4_u64) {
    for @inner (j in 0_u64..4_u64) {
      set total = total +wrap 1_u64;
      break @inner;
    }
    if i == 1_u64 {
      break @outer;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let counted = function(program, "count");
        if let Err(error) = crate::emit_llvm(program) {
            panic!("nested counted IR must emit: {error:?}\n{counted:#?}");
        }
    });
}

/// The [SYS-2] resource identity the IR records for one drop's type.
fn dropped_resource(program: &IrProgram<'_, '_, '_>, drop: IrDrop) -> Option<SystemResourceType> {
    let IrType::Nominal(id) = drop.ty() else {
        return None;
    };
    match program.nominal(id).expect("drop type must exist").kind() {
        IrNominalKind::SystemResource(contract) => Some(contract.resource),
        _ => None,
    }
}

/// Asserts the complete [SYS-5]/[HOST-3] contract the IR records for one
/// opaque system nominal, located by the identity rather than by a spelling.
fn assert_contract(
    program: &IrProgram<'_, '_, '_>,
    resource: SystemResourceType,
    action: SystemReleaseAction,
    row: SystemReleaseRow,
    backing: SystemResourceBacking,
) {
    let contract = program
        .nominals()
        .iter()
        .find_map(|nominal| match nominal.kind() {
            IrNominalKind::SystemResource(contract) if contract.resource == resource => {
                Some(*contract)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("the IR must carry a {resource:?} resource identity"));
    assert_eq!(contract.action, action);
    assert_eq!(contract.row, row);
    assert_eq!(contract.backing, backing);
}

const fn close_row() -> SystemReleaseRow {
    SystemReleaseRow {
        target_action: crate::TargetAction::MAY_SUSPEND,
        state_write: true,
    }
}

#[test]
fn every_system_type_carries_its_release_contract_and_one_release_edge() {
    // One release helper per opaque [SYS-2] type. Each function's whole
    // exhibited row is the release contribution of its own parameter, so the
    // declared rows below are exactly [SYS-5]'s table read back through
    // [EFF-2]: only the two native close attempts carry a category.
    let source = format!(
        "fn release_args(value: own Args) -> result: own unit pure {{\n  return unit;\n}}\n\n\
         fn release_host_string(value: own HostString) -> result: own unit pure {{\n  return unit;\n}}\n\n\
         fn release_relative_path(value: own RelativePath) -> result: own unit pure {{\n  return unit;\n}}\n\n\
         fn release_directory_read(value: own DirectoryRead) -> result: own unit writes(value) {{\n  return unit;\n}}\n\n\
         fn release_read_file(value: own ReadFile) -> result: own unit writes(value) {{\n  return unit;\n}}\n\n\
         fn release_output(value: own Output) -> result: own unit pure {{\n  return unit;\n}}\n\n\
         fn release_exit_status(value: own ExitStatus) -> result: own unit pure {{\n  return unit;\n}}\n\n\
         fn release_directory_source(value: own DirectorySource) -> result: own unit writes(value) {{\n  return unit;\n}}\n\n\
         {COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        use SystemReleaseAction::{LogicalConsume, NativeCloseAttempt, SourceDetach};
        use SystemResourceBacking::{CommandLifetimeLease, Opaque};

        // The seven identities and their complete contracts. `HostString` and
        // `RelativePath` are inline leases over command-lifetime backing
        // [HOST-3]; the fact is retained for auditing and lowering and
        // refuses no program.
        assert_contract(
            program,
            SystemResourceType::Args,
            LogicalConsume,
            SystemReleaseRow::EMPTY,
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::HostString,
            LogicalConsume,
            SystemReleaseRow::EMPTY,
            CommandLifetimeLease,
        );
        assert_contract(
            program,
            SystemResourceType::RelativePath,
            LogicalConsume,
            SystemReleaseRow::EMPTY,
            CommandLifetimeLease,
        );
        assert_contract(
            program,
            SystemResourceType::DirectoryRead,
            NativeCloseAttempt,
            close_row(),
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::ReadFile,
            NativeCloseAttempt,
            close_row(),
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::Output,
            SourceDetach,
            SystemReleaseRow::EMPTY,
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::ExitStatus,
            LogicalConsume,
            SystemReleaseRow::EMPTY,
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::DirectorySource,
            NativeCloseAttempt,
            close_row(),
            Opaque,
        );

        // Each helper carries exactly one release on its return edge, with
        // the action its type fixes. A logical consume is an explicit release
        // operation too, even though its row is empty and it makes no call.
        for (name, resource, action) in [
            ("release_args", SystemResourceType::Args, LogicalConsume),
            (
                "release_host_string",
                SystemResourceType::HostString,
                LogicalConsume,
            ),
            (
                "release_relative_path",
                SystemResourceType::RelativePath,
                LogicalConsume,
            ),
            (
                "release_directory_read",
                SystemResourceType::DirectoryRead,
                NativeCloseAttempt,
            ),
            (
                "release_read_file",
                SystemResourceType::ReadFile,
                NativeCloseAttempt,
            ),
            ("release_output", SystemResourceType::Output, SourceDetach),
            (
                "release_exit_status",
                SystemResourceType::ExitStatus,
                LogicalConsume,
            ),
            (
                "release_directory_source",
                SystemResourceType::DirectorySource,
                NativeCloseAttempt,
            ),
        ] {
            let function = function(program, name);
            let [drop] = return_drops(function) else {
                panic!("{name} must release its one owner on the return edge");
            };
            assert_eq!(dropped_resource(program, *drop), Some(resource));
            assert_eq!(drop.release().action, Some(action));
            assert_eq!(
                drop.release().row,
                if action == NativeCloseAttempt {
                    close_row()
                } else {
                    SystemReleaseRow::EMPTY
                }
            );
            // The released value is the parameter itself: nothing between the
            // entry and the release replaced the resource identity.
            assert_eq!(drop.value(), function.parameters()[0].0);
        }
    });
}

#[test]
fn a_move_keeps_the_resource_identity_and_its_release() {
    let source = format!(
        "fn release_after_move(file: own ReadFile) -> result: own unit writes(file) {{\n  \
         let moved = move file;\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "release_after_move");
        let [drop] = return_drops(function) else {
            panic!("the moved owner must be released exactly once");
        };
        assert_eq!(
            dropped_resource(program, *drop),
            Some(SystemResourceType::ReadFile)
        );
        assert_eq!(
            drop.release().action,
            Some(SystemReleaseAction::NativeCloseAttempt)
        );
        // A move rebinds without materializing another value, so the release
        // still names the incoming resource.
        assert_eq!(drop.value(), function.parameters()[0].0);
    });
}

#[test]
fn a_struct_field_release_reaches_the_contained_resource() {
    let source = format!(
        "struct Holder {{\n  file: ReadFile;\n}}\n\n\
         fn release_holder(holder: own Holder) -> result: own unit writes(holder.file) {{\n  return unit;\n}}\n\n\
         {COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "release_holder");
        let drops = return_drops(function);
        // The struct decomposes into its field's release and its own, in that
        // order. The field release names the `ReadFile` itself, projected out
        // of the aggregate: storing a resource in a field never loses its
        // identity or downgrades its action to a memory-only drop.
        let [field, aggregate] = drops else {
            panic!("expected the field release then the struct's own");
        };
        assert_eq!(
            dropped_resource(program, *field),
            Some(SystemResourceType::ReadFile)
        );
        assert_eq!(
            field.release().action,
            Some(SystemReleaseAction::NativeCloseAttempt)
        );
        assert_eq!(field.release().row, close_row());
        assert_ne!(field.value(), function.parameters()[0].0);
        // The struct itself has no release action of its own, and its row is
        // the union of what its owned content may run.
        assert_eq!(dropped_resource(program, *aggregate), None);
        assert_eq!(aggregate.release().action, None);
        assert_eq!(aggregate.release().row, close_row());
        assert_eq!(aggregate.value(), function.parameters()[0].0);
    });
}

#[test]
fn an_enum_release_carries_the_union_of_its_components_rows() {
    let source = format!(
        "enum Holder {{\n  Empty();\n  Full(file: ReadFile);\n}}\n\n\
         fn release_holder(holder: own Holder) -> result: own unit writes(holder) {{\n  return unit;\n}}\n\n\
         {COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "release_holder");
        let [drop] = return_drops(function) else {
            panic!("the enum owner must be released once");
        };
        // Release of an outcome value is release of its components [SYS-5]:
        // the enum itself has no release action of its own, and its row is
        // the union of the rows its variants may run.
        assert_eq!(dropped_resource(program, *drop), None);
        assert_eq!(drop.release().action, None);
        assert_eq!(drop.release().row, close_row());
    });
}

#[test]
fn an_unused_state_writing_call_reaches_ir() {
    let source = format!(
        "struct Pair {{\n  left: u64;\n}}\n\n\
         fn mutate(pair: &uniq Pair) -> result: own unit writes(pair.left) {{\n  \
         set deref(pair).left = 1_u64;\n  return unit;\n}}\n\n\
         fn wrapper(pair: &uniq Pair) -> result: own unit writes(pair.left) {{\n  \
         mutate(pair: move pair);\n  return unit;\n}}\n\n\
         {COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let wrapper = function(program, "wrapper");
        assert!(wrapper.blocks().iter().any(|block| {
            block.instructions().iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::Call { function: 0, .. },
                        ..
                    }
                )
            })
        }));
    });
}

#[test]
fn one_match_arm_releases_its_binder_on_that_arms_normal_edge() {
    let source = format!(
        "enum Holder {{\n  Empty();\n  Full(file: ReadFile);\n}}\n\n\
         fn release_arm(holder: own Holder) -> result: own unit writes(holder) {{\n  \
         match move holder {{\n    Empty() => {{\n    }}\n    Full(file: opened) => {{\n    }}\n  }}\n  \
         return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "release_arm");
        let arm_releases: Vec<Vec<IrDrop>> = function
            .blocks()
            .iter()
            .filter_map(|block| match block.terminator() {
                IrTerminator::Jump { drops, .. } => Some(drops.clone()),
                _ => None,
            })
            .collect();
        // Exactly one arm holds a resource, so exactly one arm edge carries a
        // release; the other arm's edge carries none. A release derived on
        // only one arm is still explicit on that arm's own normal edge.
        let released: Vec<&IrDrop> = arm_releases
            .iter()
            .flatten()
            .filter(|drop| drop.release().action.is_some())
            .collect();
        let [drop] = released.as_slice() else {
            panic!("exactly one match arm must release the bound resource");
        };
        assert_eq!(
            dropped_resource(program, **drop),
            Some(SystemResourceType::ReadFile)
        );
        assert_eq!(
            drop.release().action,
            Some(SystemReleaseAction::NativeCloseAttempt)
        );
    });
}

#[test]
fn returning_or_passing_an_owner_derives_no_release_here() {
    let source = format!(
        "fn pass_through(file: own ReadFile) -> result: own ReadFile pure {{\n  return move file;\n}}\n\n\
         fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {{\n  return unit;\n}}\n\n\
         fn hand_off(file: own ReadFile) -> result: own unit writes(file) {{\n  \
         release_read_file(file: move file);\n  return unit;\n}}\n\n\
         fn receive(file: own ReadFile) -> result: own unit writes(file) {{\n  \
         let received = pass_through(file: move file);\n  return unit;\n}}\n\n\
         {COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        // An owner returned across the function boundary is not released by
        // the callee: its identity leaves with the value.
        assert!(return_drops(function(program, "pass_through")).is_empty());
        // Nor is an owner moved into a call released by the caller; the
        // callee's own edge owns that release.
        assert!(return_drops(function(program, "hand_off")).is_empty());
        // The value that comes back across a call boundary is released by
        // the receiver, with the identity and action the type fixes.
        let receive = function(program, "receive");
        let [drop] = return_drops(receive) else {
            panic!("the received owner must be released once");
        };
        assert_eq!(
            dropped_resource(program, *drop),
            Some(SystemResourceType::ReadFile)
        );
        assert_eq!(
            drop.release().action,
            Some(SystemReleaseAction::NativeCloseAttempt)
        );
        // It is the call result, not the incoming parameter.
        assert_ne!(drop.value(), receive.parameters()[0].0);
    });
}

#[test]
fn releases_keep_reverse_declaration_order_on_the_normal_edge() {
    let source = format!(
        "fn ordered(first: own ReadFile, second: own ReadFile) \
         -> result: own unit writes(first, second) {{\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "ordered");
        let block = only_block(function);
        // Releases are not ordinary instructions interleaved with the body;
        // they belong to the normal return edge.
        assert!(
            block
                .instructions()
                .iter()
                .all(|instruction| !matches!(instruction, IrInstruction::Drop(_)))
        );
        // Both releases sit on the one normal edge, in the reverse
        // declaration order [STOR-3] fixes, which is the order [EFF-5]
        // requires of every conforming lowering.
        let drops = return_drops(function);
        let ordered: Vec<IrValueId> = drops.iter().map(|drop| drop.value()).collect();
        assert_eq!(
            ordered,
            vec![function.parameters()[1].0, function.parameters()[0].0]
        );
        assert!(
            drops
                .iter()
                .all(|drop| drop.release().action == Some(SystemReleaseAction::NativeCloseAttempt))
        );
    });
}

#[test]
fn a_system_call_carries_its_semantic_identity_and_precedes_the_releases() {
    let source = "command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output, command.files as files: own FileFactory) -> status: own ExitStatus writes(cwd) {\n  return exit_status(code: 0_u8);\n}\n";
    with_ir(source.as_bytes(), |program| {
        let main = function(program, "main");
        let block = only_block(main);
        // The identity is the specification's own inventory index, resolved
        // in the system declaration domain — no source spelling reaches the
        // IR [QUAL-1].
        let expected = SYSTEM_OPERATIONS
            .iter()
            .position(|operation| operation.spelling == "exit_status")
            .and_then(|index| u8::try_from(index).ok())
            .expect("the SYS-2 inventory declares exit_status");
        let calls: Vec<u8> = block
            .instructions()
            .iter()
            .filter_map(|instruction| match instruction {
                IrInstruction::Define {
                    operation: IrOperation::SystemCall { operation, .. },
                    ..
                } => Some(operation.ordinal()),
                _ => None,
            })
            .collect();
        assert_eq!(calls, vec![expected]);

        // Every release occupies the position its normal edge gives it,
        // after the operations that precede it in source order [EFF-5], and
        // the five standard inputs release in reverse declaration order with
        // the actions [SYS-5] fixes for their types.
        let drops = return_drops(main);
        let actions: Vec<Option<SystemReleaseAction>> =
            drops.iter().map(|drop| drop.release().action).collect();
        assert_eq!(
            actions,
            vec![
                Some(SystemReleaseAction::LogicalConsume),
                Some(SystemReleaseAction::SourceDetach),
                Some(SystemReleaseAction::SourceDetach),
                Some(SystemReleaseAction::NativeCloseAttempt),
                Some(SystemReleaseAction::LogicalConsume),
            ]
        );
        let resources: Vec<Option<SystemResourceType>> = drops
            .iter()
            .map(|drop| dropped_resource(program, *drop))
            .collect();
        assert_eq!(
            resources,
            vec![
                Some(SystemResourceType::FileFactory),
                Some(SystemResourceType::Output),
                Some(SystemResourceType::Output),
                Some(SystemResourceType::DirectoryRead),
                Some(SystemResourceType::Args),
            ]
        );
    });
}

#[test]
fn the_entry_retains_distinct_standard_input_rows_without_alias_metadata() {
    let both = "command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_ir(both.as_bytes(), |program| {
        let IrEntry::Command { inputs } = program.entry();
        assert_eq!(inputs, &vec![2, 3]);
    });

    let one = "command fn main(command.stdout as out: own Output) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_ir(one.as_bytes(), |program| {
        let IrEntry::Command { inputs } = program.entry();
        assert_eq!(inputs, &vec![2]);
    });
}

#[test]
fn ordinary_requires_is_not_lowered_as_a_callee_prologue() {
    let source = br#"fn bounded(value: own u64) -> result: own u64 pure contract {
  requires value < 8_u64;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let value = 4_u64;
  let result = bounded(value: value);
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let bounded = function(program, "bounded");
        assert!(
            bounded
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .next()
                .is_none(),
            "an ordinary requirement is a call-site obligation and contributes no executable callee prologue"
        );
    });
}

#[test]
fn source_proof_is_erased_before_typed_ir() {
    let source = br#"fn plain(left: own u64, left_limit: own u64, middle: own u64, middle_limit: own u64, right: own u64, right_limit: own u64) -> result: own unit pure contract {
  requires left <= left_limit;
  requires middle <= middle_limit;
  requires right <= right_limit;
} {
  return unit;
}

fn prove_only(left: own u64, left_limit: own u64, middle: own u64, middle_limit: own u64, right: own u64, right_limit: own u64) -> result: own unit pure contract {
  requires left <= left_limit;
  requires middle <= middle_limit;
  requires right <= right_limit;
} {
  invariant combined: left + middle + right <= left_limit + middle_limit + right_limit {
    use left <= left_limit;
    use middle <= middle_limit;
    use right <= right_limit;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let plain = function(program, "plain");
        let proved = function(program, "prove_only");
        assert_eq!(
            proved.parameters(),
            plain.parameters(),
            "the erased proof contributes no parameter or value dependency"
        );
        assert_eq!(proved.result(), plain.result());
        assert_eq!(
            proved.blocks(),
            plain.blocks(),
            "PRF-1 contributes no instruction, effect, branch, runtime check, or terminator change"
        );
        assert_eq!(proved.overlaps(), plain.overlaps());
        assert_eq!(proved.completion_steps(), plain.completion_steps());
        assert_eq!(proved.completion_pipeline(), plain.completion_pipeline());
    });
}

#[test]
fn staged_permission_reaches_a_complete_depth_one_driver_by_checked_loop_identity() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @plain (index in 0_u64..1_u64) {
    set total = total +wrap 1_u64;
  }
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file(factory: &uniq files);
      region {
        match open_file(permit: move permit, root: &'f cwd, name: &name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir_mode(source, OverlapLowering::Completion, |program| {
        let main = function(program, "main");
        let pipeline = main
            .completion_pipeline()
            .expect("the permitted staged loop must reach IR");
        assert_eq!(
            pipeline.source_loop().0,
            1,
            "the pure first loop has id 0; only the permitted I/O loop's checked identity may select the descriptor"
        );
        assert!(
            pipeline.entry().ordinal() > 0,
            "the descriptor must name the selected loop's preheader, not the function entry or the first loop"
        );
        assert!(
            pipeline.driver_ready(),
            "the feeder-drain edge is a complete depth-one driver"
        );
        let plan = pipeline
            .planned_driver()
            .expect("the eligible result dispatch must have a materialized one-slot plan");
        assert!(
            pipeline.carries(plan.feeder()),
            "the feeder must carry its submitted operation across the mandatory drain edge"
        );
        assert!(
            !pipeline.carries(plan.drain()),
            "the drain must retire the operation before dispatching its result"
        );
        assert!(!pipeline.drains(plan.feeder()));
        assert!(pipeline.drains(plan.drain()));
        assert_eq!(pipeline.slots(), 1);
        assert!(pipeline.slot_index(plan.feeder()).is_none());
        assert!(pipeline.slot_index(plan.drain()).is_none());
        assert!(
            plan.feeder().ordinal() < plan.drain().ordinal(),
            "the K=1 feeder must be emitted before the exact drain that owns its result"
        );
        let feeder = &main.blocks()[plan.feeder().index()];
        let IrTerminator::Jump { target, .. } = feeder.terminator() else {
            panic!("the feeder must have exactly one edge to its drain");
        };
        assert_eq!(*target, plan.drain());
        let drain = &main.blocks()[plan.drain().index()];
        let IrTerminator::Match { scrutinee, .. } = drain.terminator() else {
            panic!("the drain must own the original result dispatch");
        };
        assert_eq!(
            *scrutinee,
            plan.result(),
            "the result is consumed only after the feeder's mandatory drain edge"
        );
        let llvm = crate::emit_llvm(program)
            .expect("a complete depth-one staged descriptor must emit")
            .into_string();
        assert!(
            llvm.contains("@wf__completion_window("),
            "the production driver must ask for its bounded window once at loop entry"
        );
        assert!(
            llvm.contains("@wf__completion_file_open_at_submit(")
                && llvm.contains("@wf__completion_file_open_join("),
            "the staged cut must use one typed submission followed by its mandatory depth-one retirement"
        );
    });
}

#[test]
fn direct_staged_loop_builds_a_two_slot_issue_and_drain_driver() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let opened = 0_u64;
  let name = buffer_new(4_u64, 97_u8);
  for @scan (index in 0_u64..4_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          set opened = opened +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir_mode(source, OverlapLowering::Completion, |program| {
        let main = function(program, "main");
        let pipeline = main
            .completion_pipeline()
            .expect("the direct staged loop must reach IR");
        let driver = pipeline
            .planned_batch_driver()
            .expect("the direct result dispatch must use the bounded batch driver");
        assert_eq!(pipeline.slots(), 2);
        assert!(pipeline.window_value().is_some());
        assert!(pipeline.carries(driver.feeder()));
        assert!(!pipeline.carries(driver.drain()));
        assert!(!pipeline.drains(driver.feeder()));
        assert!(pipeline.drains(driver.drain()));
        assert!(pipeline.slot_index(driver.feeder()).is_some());
        assert!(pipeline.slot_index(driver.drain()).is_some());
        let llvm = crate::emit_llvm(program)
            .expect("a source-derived two-slot driver must emit valid LLVM text")
            .into_string();
        assert!(llvm.contains("call i64 @wf__completion_window(i64 4, i64 0, i64 2)"));
        assert!(llvm.contains("%wf.frame = alloca {"));
        assert!(llvm.contains("[2 x [2 x i64]]"));
        assert!(llvm.contains("getelementptr inbounds [2 x [2 x i64]]"));
        assert!(llvm.contains("call i32 @wf__completion_file_open_at_submit("));
        assert!(llvm.contains("call void @wf__completion_file_open_join("));
    });
}

#[test]
fn two_staged_loops_in_one_function_leave_both_on_the_ordinary_path() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let opened = 0_u64;
  let name = buffer_new(4_u64, 97_u8);
  for @first (index in 0_u64..3_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          set opened = opened +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
  }
  for @second (index in 0_u64..3_u64) {
    let permit = reserve_file(factory: &uniq files);
    region {
      match open_file(permit: move permit, root: &cwd, name: &name, start: 0_u64, end: 4_u64) {
        Ok(value: handle) => {
          set opened = opened +wrap 1_u64;
        }
        Err(error: problem) => {
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_ir_mode(source, OverlapLowering::Completion, |program| {
        let main = function(program, "main");
        assert!(
            main.completion_pipeline().is_none(),
            "a function-level descriptor must not partially transform one of two independently permitted loops"
        );
        let llvm = crate::emit_llvm(program)
            .expect("both loops must remain valid on the ordinary target path")
            .into_string();
        assert!(!llvm.contains("@wf__completion_window("));
    });
}

#[test]
fn buffer_allocations_lower_the_source_proved_length_ceiling_into_target_obligations() {
    let source = br#"fn allocate(n: own u64) -> result: own unit pure contract {
  requires n <= 1000_u64;
} {
  let filled = buffer_new(n, 7_u16);
  let vacant = buffer_vacant::<u16>(n);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  allocate(n: 4_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let allocate = function(program, "allocate");
        let bounds = allocate
            .blocks()
            .iter()
            .flat_map(IrBlock::instructions)
            .filter_map(|instruction| {
                let IrInstruction::Define { operation, .. } = instruction else {
                    return None;
                };
                match operation {
                    IrOperation::BufferFill { target_domains, .. }
                    | IrOperation::BufferVacant { target_domains, .. } => {
                        Some(target_domains.source_length_upper_bound())
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(bounds, vec![1000, 1000]);
    });
}

#[test]
fn an_uninhabited_function_keeps_its_abi_and_lowers_to_one_unreachable_block() {
    let source = br#"fn impossible(value: own i32) -> out: own i32 pure contract {
  requires value == 0_i32;
  requires value != 0_i32;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_ir(source, |program| {
        let impossible = function(program, "impossible");
        assert_eq!(impossible.parameters().len(), 1);
        assert_eq!(
            impossible.result(),
            IrType::Integer {
                width: 32,
                signed: true,
            }
        );
        let [entry] = impossible.blocks() else {
            panic!("an uninhabited function must lower to exactly one block");
        };
        assert!(entry.instructions().is_empty());
        assert_eq!(entry.terminator(), &IrTerminator::Unreachable);
    });
}

#[test]
fn a_memory_only_release_carries_no_system_action_or_row() {
    // The negative boundary: a `buffer` release is compiler-derived too, and
    // it must stay distinguishable from a system release [STOR-3].
    with_ir(
        b"fn drop_buffer(values: own buffer<u8>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |program| {
            let [drop] = return_drops(function(program, "drop_buffer")) else {
                panic!("the buffer owner must be released once");
            };
            assert_eq!(drop.release().action, None);
            assert_eq!(drop.release().row, SystemReleaseRow::EMPTY);
        },
    );
}

/// The recognized byte-walk loop for the wide-probe tests, with `{MIDDLE}`
/// and `{STEP}` varied per case.
fn byte_walk_source(middle: &str, step: &str) -> Vec<u8> {
    format!(
        "command fn main() -> status: own ExitStatus pure {{\n  let data = buffer_new(64_u64, 97_u8);\n  let mark = 88_u8;\n  let seen = 0_u64;\n  let stop = len_of(data);\n  let cursor = 0_u64;\n  loop @walk {{\n    let done = cursor >= stop;\n    if done {{\n      break @walk;\n    }}\n    let byte = data[cursor];\n{middle}    set cursor = cursor +wrap {step};\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

const NEUTRAL_MIDDLE: &str = "    let newline = byte == 10_u8;\n    if newline {\n      set seen = seen +wrap 1_u64;\n    }\n    let lead = byte == mark;\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";

fn probe_needle_counts(program: &IrProgram<'_, '_, '_>) -> Vec<usize> {
    program
        .functions()
        .iter()
        .flat_map(IrFunction::blocks)
        .flat_map(IrBlock::instructions)
        .filter_map(|instruction| {
            let IrInstruction::Define {
                operation: IrOperation::BufferProbeSkip { needles, .. },
                ..
            } = instruction
            else {
                return None;
            };
            Some(needles.len())
        })
        .collect()
}

#[test]
fn a_recognized_byte_walk_gains_one_wide_probe_with_its_needles() {
    with_ir(&byte_walk_source(NEUTRAL_MIDDLE, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), vec![2]);
    });
}

#[test]
fn an_effect_on_the_quiet_path_declines_the_wide_probe() {
    let middle = format!("{NEUTRAL_MIDDLE}    set seen = seen +wrap 1_u64;\n");
    with_ir(&byte_walk_source(&middle, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}

#[test]
fn a_non_single_step_increment_declines_the_wide_probe() {
    with_ir(&byte_walk_source(NEUTRAL_MIDDLE, "2_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}

#[test]
fn a_needle_declared_inside_the_loop_declines_the_wide_probe() {
    let middle = "    let inner_mark = 88_u8;\n    let lead = byte == inner_mark;\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";
    with_ir(&byte_walk_source(middle, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}
