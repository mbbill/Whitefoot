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
    ParseLimits, ParseOutcome, ResolutionOutcome, SYSTEM_OPERATIONS, SemanticOutcome, SourceBundle,
    SourceInput, SourceLimits, SystemReleaseAction, SystemReleaseRow, SystemResourceBacking,
    SystemResourceType, TerminalLimits, TerminalOutcome, audit_canonical, check_semantics,
    classify_terminals, finalize, parse, resolve,
};

use super::{
    IrBlock, IrDrop, IrEntry, IrFunction, IrInstruction, IrNominalKind, IrOperation, IrProgram,
    IrTerminator, IrType, IrValueId, lower_checked,
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

/// The `command` entry every system-admitted test unit needs [SYS-3, FN-7].
const COMMAND_ENTRY: &str =
    "command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

fn with_ir<ResultValue>(
    source: &[u8],
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
    let ir = lower_checked(*checked).expect("checked system program must lower");
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
    let source = br#"fn count() -> own u64 pure {
  let total = 0_u64;
  for @items i in 0_u64..2_u64 {
    set total = total +wrap 1_u64;
  }
  return total;
}

command fn main() -> own ExitStatus pure {
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
    });
}

#[test]
fn counted_range_carries_one_stable_binder_address_for_body_local_shared_borrows() {
    let source = br#"fn count() -> own u64 pure {
  let total = 0_u64;
  let upper = 2_u64;
  for @items i in 0_u64..upper {
    region 'r {
      let held = &'r i;
      let seen = deref(held);
      set total = total +wrap seen;
    }
    set upper = 0_u64;
  }
  return total;
}

command fn main() -> own ExitStatus pure {
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
    let source = br#"fn count() -> own u64 pure {
  let total = 0_u64;
  for @outer i in 0_u64..4_u64 {
    for @inner j in 0_u64..4_u64 {
      set total = total +wrap 1_u64;
      break @inner;
    }
    if ieq(i, 1_u64) {
      break @outer;
    }
  }
  return total;
}

command fn main() -> own ExitStatus pure {
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

const CLOSE_ROW: SystemReleaseRow = SystemReleaseRow {
    external: true,
    blocks: true,
};

#[test]
fn every_system_type_carries_its_release_contract_and_one_release_edge() {
    // One release helper per opaque [SYS-2] type. Each function's whole
    // exhibited row is the release contribution of its own parameter, so the
    // declared rows below are exactly [SYS-5]'s table read back through
    // [EFF-2]: only the two native close attempts carry a category.
    let source = format!(
        "fn release_args(value: own Args) -> own unit pure {{\n  return unit;\n}}\n\n\
         fn release_host_string(value: own HostString) -> own unit pure {{\n  return unit;\n}}\n\n\
         fn release_relative_path(value: own RelativePath) -> own unit pure {{\n  return unit;\n}}\n\n\
         fn release_directory_read(value: own DirectoryRead) -> own unit external, blocks {{\n  return unit;\n}}\n\n\
         fn release_read_file(value: own ReadFile) -> own unit external, blocks {{\n  return unit;\n}}\n\n\
         fn release_output(value: own Output) -> own unit pure {{\n  return unit;\n}}\n\n\
         fn release_exit_status(value: own ExitStatus) -> own unit pure {{\n  return unit;\n}}\n\n\
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
            CLOSE_ROW,
            Opaque,
        );
        assert_contract(
            program,
            SystemResourceType::ReadFile,
            NativeCloseAttempt,
            CLOSE_ROW,
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
                    CLOSE_ROW
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
        "fn release_after_move(file: own ReadFile) -> own unit external, blocks {{\n  \
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
         fn release_holder(holder: own Holder) -> own unit external, blocks {{\n  return unit;\n}}\n\n\
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
        assert_eq!(field.release().row, CLOSE_ROW);
        assert_ne!(field.value(), function.parameters()[0].0);
        // The struct itself has no release action of its own, and its row is
        // the union of what its owned content may run.
        assert_eq!(dropped_resource(program, *aggregate), None);
        assert_eq!(aggregate.release().action, None);
        assert_eq!(aggregate.release().row, CLOSE_ROW);
        assert_eq!(aggregate.value(), function.parameters()[0].0);
    });
}

#[test]
fn an_enum_release_carries_the_union_of_its_components_rows() {
    let source = format!(
        "enum Holder {{\n  Empty();\n  Full(file: ReadFile);\n}}\n\n\
         fn release_holder(holder: own Holder) -> own unit external, blocks {{\n  return unit;\n}}\n\n\
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
        assert_eq!(drop.release().row, CLOSE_ROW);
    });
}

#[test]
fn one_match_arm_releases_its_binder_on_that_arms_normal_edge() {
    let source = format!(
        "enum Holder {{\n  Empty();\n  Full(file: ReadFile);\n}}\n\n\
         fn release_arm(holder: own Holder) -> own unit external, blocks {{\n  \
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
        "fn pass_through(file: own ReadFile) -> own ReadFile pure {{\n  return move file;\n}}\n\n\
         fn release_read_file(file: own ReadFile) -> own unit external, blocks {{\n  return unit;\n}}\n\n\
         fn hand_off(file: own ReadFile) -> own unit external, blocks {{\n  \
         release_read_file(file: move file);\n  return unit;\n}}\n\n\
         fn receive(file: own ReadFile) -> own unit external, blocks {{\n  \
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
fn releases_keep_reverse_declaration_order_and_never_sit_on_a_trapping_edge() {
    let source = format!(
        "fn ordered(first: own ReadFile, second: own ReadFile, ready: own Bool) \
         -> own unit external, blocks, traps {{\n  \
         claim ordering_probe: ready because \"ordering probe\";\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    with_ir(source.as_bytes(), |program| {
        let function = function(program, "ordered");
        let block = only_block(function);
        // A trap runs no language cleanup [TRAP-1, EFF-4]: the retained check
        // is reached with no release before it, and the IR gives a check no
        // edge to carry one.
        assert!(
            block
                .instructions()
                .iter()
                .all(|instruction| !matches!(instruction, IrInstruction::Drop(_)))
        );
        assert!(
            block
                .instructions()
                .iter()
                .any(|instruction| matches!(instruction, IrInstruction::Check { .. }))
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
    let source = "command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus external, blocks {\n  return exit_status(code: 0_u8);\n}\n";
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
        // the four standard inputs release in reverse declaration order with
        // the actions [SYS-5] fixes for their types.
        let drops = return_drops(main);
        let actions: Vec<Option<SystemReleaseAction>> =
            drops.iter().map(|drop| drop.release().action).collect();
        assert_eq!(
            actions,
            vec![
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
                Some(SystemResourceType::Output),
                Some(SystemResourceType::Output),
                Some(SystemResourceType::DirectoryRead),
                Some(SystemResourceType::Args),
            ]
        );
    });
}

#[test]
fn the_entry_retains_the_standard_input_rows_and_the_may_alias_pair() {
    let both = "command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_ir(both.as_bytes(), |program| {
        let IrEntry::Command { inputs, aliases } = program.entry() else {
            panic!("a kind-declaring entry must lower as a command entry");
        };
        assert_eq!(inputs, &vec![2, 3]);
        assert!(program.entry_goal().is_none());
        // [SYS-12]: redirection may make the two `Output` owners one sink.
        // Nothing here reads the link; it is retained so a later
        // cross-resource reordering fact cannot treat them as disjoint.
        let [alias] = aliases.as_slice() else {
            panic!("the two output owners must carry one retained alias link");
        };
        assert_eq!((alias.left(), alias.right()), (2, 3));
    });

    let one = "command fn main(command.stdout as out: own Output) -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_ir(one.as_bytes(), |program| {
        let IrEntry::Command { inputs, aliases } = program.entry() else {
            panic!("a kind-declaring entry must lower as a command entry");
        };
        assert_eq!(inputs, &vec![2]);
        assert!(program.entry_goal().is_none());
        // One selected sink has nothing to alias with.
        assert!(aliases.is_empty());
    });

    with_ir(
        b"fn main() -> own unit pure {\n  return unit;\n}\n",
        |program| {
            assert_eq!(program.entry(), &IrEntry::Unlabelled);
            assert!(program.entry_goal().is_none());
        },
    );
}

#[test]
fn ordinary_requires_is_not_lowered_as_a_callee_prologue() {
    let source = br#"fn bounded(value: own u64) -> own u64 pure requires {
  check ilt(value, 8_u64) else trap "bounded";
} {
  return value;
}

fn main() -> own unit traps {
  let value = 4_u64;
  claim caller_evidence: ilt(value, 8_u64) because "caller evidence";
  let result = bounded(value: value);
  return unit;
}
"#;
    with_ir(source, |program| {
        let bounded = function(program, "bounded");
        assert!(
            bounded
                .blocks()
                .iter()
                .flat_map(IrBlock::instructions)
                .all(|instruction| !matches!(instruction, IrInstruction::Check { .. })),
            "an ordinary requirement is a call-site obligation and S4 axiom, not executable IR"
        );
    });
}

#[test]
fn program_start_retains_one_private_straight_line_requirement_goal() {
    let source = br#"const expected: u8 = 7_u8;

fn main() -> own unit pure requires {
  let same = ieq(expected, 7_u8);
  check same else trap "entry requirement";
} {
  return unit;
}
"#;
    with_ir(source, |program| {
        let goal = program
            .entry_goal()
            .expect("a required entry must retain one wrapper goal");
        assert_eq!(goal.ty(goal.condition()), Some(IrType::Bool));
        assert_eq!(goal.trap().rule_id, "OP-5");
        assert_eq!(goal.trap().message, "entry requirement");
        assert_eq!(goal.trap().function, "main");
        assert!(
            goal.definitions()
                .iter()
                .all(|definition| !matches!(definition.operation(), IrOperation::ArrayFill { .. }))
        );
    });
}

#[test]
fn a_memory_only_release_carries_no_system_action_or_row() {
    // The negative boundary: a `buffer` release is compiler-derived too, and
    // it must stay distinguishable from a system release [STOR-3].
    with_ir(
        b"fn drop_buffer(values: own buffer<u8>) -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
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
        "fn main() -> own unit allocates(heap), traps {{\n  let data = buffer_new(64_u64, 97_u8);\n  let mark = 88_u8;\n  let seen = 0_u64;\n  let stop = len(data);\n  let cursor = 0_u64;\n  loop @walk {{\n    let done = ige(cursor, stop);\n    if done {{\n      break @walk;\n    }}\n    let byte = data[cursor];\n{middle}    set cursor = cursor +wrap {step};\n  }}\n  claim walk_drift: ilt(seen, 1000_u64) because \"walk drift\";\n  return unit;\n}}\n"
    )
    .into_bytes()
}

const NEUTRAL_MIDDLE: &str = "    let newline = ieq(byte, 10_u8);\n    if newline {\n      set seen = seen +wrap 1_u64;\n    }\n    let lead = ieq(byte, mark);\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";

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
    let middle = "    let inner_mark = 88_u8;\n    let lead = ieq(byte, inner_mark);\n    if lead {\n      set seen = seen +wrap 2_u64;\n    }\n";
    with_ir(&byte_walk_source(middle, "1_u64"), |program| {
        assert_eq!(probe_needle_counts(program), Vec::<usize>::new());
    });
}
