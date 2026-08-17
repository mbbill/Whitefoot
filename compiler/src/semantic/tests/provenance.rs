use std::cmp::Ordering;

use crate::SemanticOutcome;

use super::super::entailment::CallGoalDisposition;
use super::super::model::{CheckedProgramData, CheckedStatement, FunctionId};
use super::super::provenance::{
    CallArgumentProvenanceDisposition, CarrierCallRole, CarrierRoute, CarrierWriteContext,
    DatumDependencies, DatumSelector, FunctionDependencies, LocalLeafProvenanceDisposition,
    ParameterDatum, ProvenanceDependency, ProvenanceGoalObservation, StructuralPredecessor,
    SubjectPredecessor, SystemResultProvenance, ValueDependencies, carrier_route_cmp,
    system_external_writes, system_result_provenance,
};
use super::with_semantics;

/// One extracted row of the [SYS-2] `wf-prov` table.
#[derive(Debug, Eq, PartialEq)]
struct ProvRow {
    operation: String,
    result_class: String,
    parameter_class: String,
}

/// The `wf-prov` table, extracted from the active specification by fence info
/// string.
///
/// This is the table's first extraction lock of any kind. Its rows were
/// hand-transcribed into `semantic::provenance` as bare numeric operation
/// ordinals — the one payload in the specification with a machine consumer and
/// no check that the consumer still agreed with it.
fn prov_rows() -> Vec<ProvRow> {
    let mut fences = crate::ACTIVE_KERNEL_SPEC_TEXT.split("\n```wf-prov\n");
    let _before = fences.next().expect("the split always yields a first part");
    let body = fences
        .next()
        .expect("the active specification has one wf-prov fence")
        .split_once("\n```")
        .expect("the wf-prov fence is terminated")
        .0;
    assert!(
        fences.next().is_none(),
        "the wf-prov schema names exactly one table"
    );

    let mut lines = body.lines();
    assert_eq!(
        lines.next(),
        Some("| operation | result component class | writable `&uniq` parameter component class |"),
        "the first row of a wf-prov fence is its column schema"
    );
    assert_eq!(lines.next(), Some("|---|---|---|"));

    lines
        .map(|line| {
            let cells: Vec<&str> = line
                .strip_prefix("| ")
                .and_then(|rest| rest.strip_suffix(" |"))
                .expect("a wf-prov row is pipe-delimited")
                .split(" | ")
                .collect();
            let [operation, result_class, parameter_class] = cells.as_slice() else {
                panic!(
                    "a wf-prov row has exactly three cells, found {}",
                    cells.len()
                );
            };
            ProvRow {
                operation: operation
                    .split('`')
                    .nth(1)
                    .expect("a wf-prov operation cell is backticked")
                    .to_owned(),
                result_class: (*result_class).to_owned(),
                parameter_class: (*parameter_class).to_owned(),
            }
        })
        .collect()
}

/// Every `wf-prov` row's two class cells decide what the compiler does.
///
/// The operation column is locked to `SYSTEM_OPERATIONS` order, so the numeric
/// ordinals the compiler dispatches on cannot drift from the row they name —
/// the failure that a bare ordinal table makes silent, because a
/// misattributed external class still produces a well-formed provenance
/// judgment for some other operation.
///
/// A green run establishes that each row's result class and writable-parameter
/// class are the ones the compiler applies, and that the two orders coincide.
/// It does not establish that an external class produces the right downstream
/// [PRV-2] demand; the provenance tests below cover that.
#[test]
fn every_wf_prov_row_decides_the_compilers_system_provenance() {
    let rows = prov_rows();
    assert_eq!(
        rows.len(),
        crate::SYSTEM_OPERATIONS.len(),
        "the wf-prov table has one row per SYS-2 operation"
    );

    let mut result_classes = std::collections::HashSet::new();
    let mut writing_rows = 0;
    for (ordinal, row) in rows.iter().enumerate() {
        let index = u8::try_from(ordinal).expect("eleven operations fit a u8");
        assert_eq!(
            row.operation,
            crate::SYSTEM_OPERATIONS[ordinal].spelling,
            "wf-prov row {ordinal} and SYSTEM_OPERATIONS[{ordinal}] name different operations"
        );

        // The result-component cell's own vocabulary decides the class.
        let expected = match row.result_class.as_str() {
            "plain result external" | "`Ok(value:)` external; `Err(error:)` external" => {
                SystemResultProvenance::AllExternal
            }
            "`Ok(value:)` internal; `Err(error:)` external" => {
                SystemResultProvenance::ErrorPayloadOnly
            }
            "`ReadBytes(count:)` internal; `ReadFailed(error:)` external; `ReadEnd()` carries no result component" => {
                SystemResultProvenance::ReadFailedPayloadOnly
            }
            "plain result internal" => SystemResultProvenance::NoneExternal,
            other => panic!(
                "{} writes an unmodelled result class {other}",
                row.operation
            ),
        };
        result_classes.insert(expected);
        assert_eq!(
            system_result_provenance(index),
            Some(expected),
            "{}'s result class is written `{}`",
            row.operation,
            row.result_class
        );

        // The writable-parameter cell names the parameters by their declared
        // name, so the expected ordinals come from the operation's own
        // parameter list rather than from a second hand-written list.
        let declared = crate::SYSTEM_OPERATIONS[ordinal].parameters;
        let expected_writes: Vec<usize> = if row.parameter_class == "—" {
            Vec::new()
        } else {
            let mut ordinals: Vec<usize> = row
                .parameter_class
                .split("; ")
                .map(|entry| {
                    let (name, class) = entry
                        .split_once(' ')
                        .unwrap_or_else(|| panic!("{entry} names a parameter and a class"));
                    assert_eq!(class, "external", "{entry} is not an external write");
                    let name = name.trim_matches('`');
                    declared
                        .iter()
                        .position(|parameter| parameter.name == name)
                        .unwrap_or_else(|| panic!("{} declares no parameter {name}", row.operation))
                })
                .collect();
            ordinals.sort_unstable();
            writing_rows += 1;
            ordinals
        };
        assert_eq!(
            system_external_writes(index).expect("a declared operation ordinal"),
            expected_writes.as_slice(),
            "{}'s writable-parameter class is written `{}`",
            row.operation,
            row.parameter_class
        );
    }

    // All four result classes and the four writing rows appear, so a table
    // that collapsed to one class would not pass the loop vacuously.
    assert_eq!(result_classes.len(), 4);
    assert_eq!(writing_rows, 4);
    // An ordinal past the inventory fails closed rather than defaulting to a
    // class, in both directions the dispatch can be wrong.
    let past = u8::try_from(crate::SYSTEM_OPERATIONS.len()).expect("eleven fits a u8");
    assert_eq!(system_result_provenance(past), None);
    assert!(system_external_writes(past).is_err());
}

fn checked(source: &[u8], run: impl FnOnce(&CheckedProgramData)) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("provenance test source must be accepted: {outcome:?}");
        };
        run(&program.data);
    });
}

#[test]
fn a_bridge_subject_excludes_the_requirement_bound_and_protected_base() {
    let source = br#"const count: u64 = 4_u64;

fn read_below(values: own array<u8, count>, limit: own u64, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ile(limit, room) else trap "limit in values";
} {
  if ilt(position, limit) {
    return values[position];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let metadata = &program.provenance;
        assert_eq!(metadata.structural_bridges.len(), 1);
        assert_eq!(metadata.subject_bridges.len(), 1);
        assert_eq!(
            metadata.subject_bridges[0].subject,
            ParameterDatum {
                ordinal: 2,
                selector: DatumSelector::Plain,
            },
            "only the indexed position is the constrained subject"
        );
    });
}

#[test]
fn full_only_check_and_claim_calls_differ_from_an_unasserted_branch_call() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "read bound";
} {
  return values[position];
}

fn from_check(values: own array<u8, count>, position: own u64) -> own u8 traps {
  let room = len(values);
  check ilt(position, room) else trap "checked";
  return read(values: move values, position: position);
}

fn from_claim(values: own array<u8, count>, position: own u64) -> own u8 traps {
  let room = len(values);
  claim bounded: ilt(position, room) because "claimed";
  return read(values: move values, position: position);
}

fn from_branch(values: own array<u8, count>, position: own u64) -> own u8 pure {
  let room = len(values);
  if ilt(position, room) {
    return read(values: move values, position: position);
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let check = function(program, "from_check");
        let claim = function(program, "from_claim");
        let branch = function(program, "from_branch");
        let metadata = &program.provenance;
        assert_eq!(metadata.calls.len(), 3);

        for caller in [check, claim] {
            let call = metadata
                .calls
                .iter()
                .find(|call| call.caller == caller)
                .expect("full-only call link");
            assert_eq!(call.full.goal_disposition, CallGoalDisposition::Discharged);
            assert_ne!(
                call.unasserted.goal_disposition,
                CallGoalDisposition::Discharged
            );
            assert_ne!(
                call.s4_blinded.goal_disposition,
                CallGoalDisposition::Discharged
            );
            assert!(call.upstream_requirement.is_none());
        }

        let call = metadata
            .calls
            .iter()
            .find(|call| call.caller == branch)
            .expect("branch call link");
        assert_eq!(
            call.unasserted.goal_disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            call.s4_blinded.goal_disposition,
            CallGoalDisposition::Discharged
        );
        assert!(call.upstream_requirement.is_none());

        assert_eq!(metadata.call_argument_dispositions.len(), 6);
        assert!(metadata.call_argument_dispositions.iter().all(|argument| {
            argument.disposition == CallArgumentProvenanceDisposition::NoEvent
                && matches!(argument.complete, ProvenanceGoalObservation::Evaluated(_))
                && matches!(argument.unasserted, ProvenanceGoalObservation::Evaluated(_))
                && matches!(argument.s4_blinded, ProvenanceGoalObservation::Evaluated(_))
        }));
    });
}

fn function(program: &CheckedProgramData, name: &str) -> FunctionId {
    program
        .functions
        .iter()
        .find(|function| function.name == name)
        .map(|function| function.id)
        .unwrap_or_else(|| panic!("missing function {name}"))
}

fn dependencies<'program>(
    program: &'program CheckedProgramData,
    name: &str,
) -> &'program FunctionDependencies {
    let function = function(program, name);
    program
        .provenance
        .functions
        .iter()
        .find(|dependencies| dependencies.function == function)
        .unwrap_or_else(|| panic!("missing dependencies for {name}"))
}

fn projection(value: &ValueDependencies, selector: DatumSelector) -> &[ParameterDatum] {
    &value
        .components
        .iter()
        .find(|component| component.selector == selector)
        .unwrap_or_else(|| panic!("missing value projection {selector:?}"))
        .dependency
        .parameters
        .datums
}

#[test]
fn an_exact_missing_component_selector_fails_closed() {
    let value = ValueDependencies {
        components: vec![DatumDependencies {
            selector: DatumSelector::Plain,
            dependency: ProvenanceDependency::default(),
        }],
    };
    assert_eq!(
        value.selected(DatumSelector::EnumPayload {
            variant: 0,
            field: 0,
        }),
        Err(crate::SemanticCompilerFailure::InvalidResolution)
    );
}

#[test]
fn system_result_and_write_origins_are_each_one_edge_and_tie_only_by_call_path() {
    let result = CarrierRoute::call_terminal(
        crate::NodePath {
            components: vec![1],
        },
        DatumSelector::Plain,
        CarrierCallRole::SystemResult,
        None,
    );
    let write = CarrierRoute::call_terminal(
        crate::NodePath {
            components: vec![2],
        },
        DatumSelector::Plain,
        CarrierCallRole::SystemWrite,
        Some(CarrierWriteContext {
            parameter: 1,
            actual: crate::NodePath {
                components: vec![9],
            },
        }),
    );
    assert_eq!(result.steps().len(), 1);
    assert_eq!(write.steps().len(), 1);
    assert_eq!(carrier_route_cmp(&result, &write), Ordering::Less);

    let same_path_result = CarrierRoute::call_terminal(
        crate::NodePath {
            components: vec![2],
        },
        DatumSelector::Plain,
        CarrierCallRole::SystemResult,
        None,
    );
    assert_eq!(
        carrier_route_cmp(&same_path_result, &write),
        Ordering::Equal,
        "role and write context are diagnostic identity, not extra edges or tie keys"
    );
}

fn assert_provenance_rule_at(source: &[u8], rule: &str, located: &[u8]) {
    inspect_provenance_issue(source, rule, located, |_| {});
}

fn inspect_provenance_issue(
    source: &[u8],
    rule: &str,
    located: &[u8],
    run: impl FnOnce(&crate::SemanticIssueKind),
) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule} provenance rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule_id(), rule);
        let crate::SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("{rule} must cite a source node, got {:?}", issue.location());
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits usize");
        assert_eq!(&source[start..end], located);
        run(issue.kind());
    });
}

fn coordinate_bytes<'source>(
    source: &'source [u8],
    coordinate: &crate::SyntaxCoordinate,
) -> &'source [u8] {
    let start = usize::try_from(coordinate.start().value()).expect("offset fits usize");
    let end = usize::try_from(coordinate.end().value()).expect("offset fits usize");
    &source[start..end]
}

#[test]
fn an_external_system_result_cannot_use_a_claim_to_authorize_a_local_subscript() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let room = len(values);
    claim bounded: ilt(position, room) because "claimed external bound";
    let selected = values[position];
  }
  return exit_status(code: 0_u8);
}
"#;

    assert_provenance_rule_at(source, "PRV-3", b"[position]");
}

#[test]
fn an_external_nested_give_reaches_the_outer_value_binding_and_is_rejected() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let derived = if True() {
      if True() {
        give position;
      } else {
        give 0_u64;
      }
    } else {
      give 0_u64;
    }
    let room = len(values);
    claim bounded: ilt(derived, room) because "claimed external bound";
    let selected = values[derived];
  }
  return exit_status(code: 0_u8);
}
"#;

    assert_provenance_rule_at(source, "PRV-3", b"[derived]");
}

#[test]
fn a_direct_parameter_demand_rejects_the_external_actual_at_its_argument() {
    let source = br#"fn read(values: own array<u8, 4>, position: own u64) -> own u8 traps {
  let room = len(values);
  claim bounded: ilt(position, room) because "claimed parameter bound";
  return values[position];
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let selected = read(values: move values, position: position);
  }
  return exit_status(code: 0_u8);
}
"#;

    assert_provenance_rule_at(source, "PRV-2", b"position");
}

#[test]
fn base_op4_precedes_a_local_prv3_candidate() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus pure {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let selected = values[position];
    return exit_status(code: selected);
  }
}
"#;

    assert_provenance_rule_at(source, "OP-4", b"[position]");
}

#[test]
fn base_fn8_precedes_a_call_argument_prv2_candidate() {
    let source = br#"fn read(values: own array<u8, 4>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "bound";
} {
  return values[position];
}

command fn main(command.args as args: own Args) -> own ExitStatus pure {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let selected = read(values: move values, position: position);
    return exit_status(code: selected);
  }
}
"#;

    assert_provenance_rule_at(
        source,
        "FN-8",
        b"read(values: move values, position: position)",
    );
}

#[test]
fn a_bridge_converts_to_direct_and_crosses_a_requirement_free_call() {
    let source = br#"fn leaf(values: own array<u8, 4>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "leaf bound";
} {
  return values[position];
}

fn wrapper(values: own array<u8, 4>, position: own u64) -> own u8 traps {
  let room = len(values);
  check ilt(position, room) else trap "wrapper assertion";
  return leaf(values: move values, position: position);
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let selected = wrapper(values: move values, position: position);
    return exit_status(code: selected);
  }
}
"#;

    inspect_provenance_issue(source, "PRV-2", b"position", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedCallArgument(detail) = kind else {
            panic!("expected structured PRV-2 detail: {kind:?}");
        };
        assert_eq!(detail.targets.len(), 1);
        let target = &detail.targets[detail.selected_target as usize];
        assert_eq!(target.demand_kind, crate::ProvenanceDemandKind::Direct);
        assert_eq!(target.boundaries.len(), 2);
        assert_eq!(
            target.boundaries[0].callee.demand_kind,
            crate::ProvenanceDemandKind::RequirementBridge
        );
        assert_eq!(
            target.boundaries[0]
                .caller_continuation
                .as_ref()
                .expect("bridge converts to a caller state")
                .demand_kind,
            crate::ProvenanceDemandKind::Direct
        );
        assert_eq!(
            target.boundaries[1].callee.demand_kind,
            crate::ProvenanceDemandKind::Direct
        );
        assert!(target.boundaries[1].caller_continuation.is_none());
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_two_hop_bridge_diagnostic_retains_every_boundary_and_terminal_origin() {
    let source = include_bytes!("../../../../tests/conformance/cases/prv2-neg-two-hop-bridge.wf");
    inspect_provenance_issue(source, "PRV-2", b"index", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedCallArgument(detail) = kind else {
            panic!("expected structured PRV-2 detail: {kind:?}");
        };
        let target = &detail.targets[detail.selected_target as usize];
        assert_eq!(
            target.demand_kind,
            crate::ProvenanceDemandKind::RequirementBridge
        );
        assert_eq!(target.boundaries.len(), 3);
        assert!(
            target.boundaries[..2]
                .iter()
                .all(|boundary| boundary.caller_continuation.is_some())
        );
        assert!(target.boundaries[2].caller_continuation.is_none());
        assert!(target.boundaries.iter().all(|boundary| {
            boundary.callee.demand_kind == crate::ProvenanceDemandKind::RequirementBridge
                && boundary.callee.requirement.is_some()
                && boundary.callee.parameter.ordinal == 1
        }));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
        assert!(target.witness.len() > target.carrier.len());
        assert!(target.target_repair.contains("rejecting caller"));
        assert!(detail.restructure_alternative.contains("explicit dataflow"));
    });
}

#[test]
fn a_command_entry_bridge_terminates_at_its_call_argument_without_upstream_continuation() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/prv2-neg-entry-system-result-bridge.wf"
    );
    inspect_provenance_issue(source, "PRV-2", b"index", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedCallArgument(detail) = kind else {
            panic!("expected structured PRV-2 detail: {kind:?}");
        };
        let target = &detail.targets[detail.selected_target as usize];
        assert_eq!(
            target.demand_kind,
            crate::ProvenanceDemandKind::RequirementBridge
        );
        assert_eq!(target.boundaries.len(), 1);
        assert!(target.boundaries[0].caller_continuation.is_none());
        assert_eq!(target.boundaries[0].callee.parameter.ordinal, 1);
        assert!(target.boundaries[0].callee.requirement.is_some());
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_recursive_direct_route_uses_complete_state_identity_and_stays_finite() {
    let source = br#"fn rotate(values: own array<u8, 4>, current: own u64, future: own u64, again: own Bool) -> own u8 traps {
  if again {
    let stop = False();
    return rotate(values: move values, current: future, future: current, again: stop);
  } else {
    let room = len(values);
    claim bounded: ilt(current, room) because "recursive direct leaf";
    return values[current];
  }
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let go = True();
    let selected = rotate(values: move values, current: 0_u64, future: position, again: go);
    return exit_status(code: selected);
  }
}
"#;

    inspect_provenance_issue(source, "PRV-2", b"position", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedCallArgument(detail) = kind else {
            panic!("expected structured PRV-2 detail: {kind:?}");
        };
        let target = &detail.targets[detail.selected_target as usize];
        assert_eq!(target.demand_kind, crate::ProvenanceDemandKind::Direct);
        assert_eq!(target.boundaries.len(), 2);
        assert!(target.boundaries.iter().all(|boundary| {
            boundary.callee.demand_kind == crate::ProvenanceDemandKind::Direct
        }));
        assert_eq!(target.boundaries[0].callee.parameter.ordinal, 1);
        assert_eq!(
            target.boundaries[0]
                .caller_continuation
                .as_ref()
                .expect("recursive permutation continues")
                .parameter
                .ordinal,
            2
        );
        assert_eq!(target.boundaries[1].callee.parameter.ordinal, 2);
        assert!(target.boundaries[1].caller_continuation.is_none());
    });
}

#[test]
fn a_cross_function_system_result_retains_every_result_and_let_carrier() {
    let source = br#"fn count_arguments(args: own Args) -> own u64 pure {
  region 'a {
    let total = args_count<'a>(args: &'a args);
    return total;
  }
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  let position = count_arguments(args: move args);
  let room = len(values);
  claim bounded: ilt(position, room) because "cross-function result";
  let selected = values[position];
  return exit_status(code: selected);
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"count_arguments(args: move args)"
        }));
        assert!(
            target
                .carrier
                .iter()
                .any(|step| { coordinate_bytes(source, &step.coordinate) == b"return total;" })
        );
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate)
                == b"let total = args_count<'a>(args: &'a args);"
        }));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_cross_function_system_write_keeps_write_context_before_the_true_origin() {
    let source = br#"fn copy_host['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>) -> own u64 reads('v 'd), writes('d), traps {
  region 'c {
    match host_copy_bytes<'v, 'c>(value: value, destination: &uniq 'c deref(destination), offset: 0_u64, capacity: 4_u64) {
      Ok(value: copied) => {
        return copied;
      }
      Err(error: problem) => {
        return 0_u64;
      }
    }
  }
}

command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let bytes = buffer_new(4_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            let copied = copy_host<'v, 'd>(value: &'v text, destination: &uniq 'd bytes);
          }
        }
      }
      Err(error: absent) => {
      }
    }
  }
  let raw = bytes[0_u64];
  let position = cvt<u8, u64>(raw);
  let room = len(bytes);
  claim bounded: ilt(position, room) because "cross-function write";
  let selected = bytes[position];
  return exit_status(code: selected);
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"host_copy_bytes<'v, 'c>(value: value, destination: &uniq 'c deref(destination), offset: 0_u64, capacity: 4_u64)"
        );
        assert!(target.carrier.last().is_some_and(|step| {
            step.call_role == Some(crate::ProvenanceCarrierCallRole::SystemWrite)
                && coordinate_bytes(source, &step.coordinate)
                    == b"host_copy_bytes<'v, 'c>(value: value, destination: &uniq 'c deref(destination), offset: 0_u64, capacity: 4_u64)"
        }));
        let system_write = target.carrier.last().expect("system write origin");
        let system_context = system_write
            .write_context
            .as_ref()
            .expect("system write context");
        assert_eq!(system_context.parameter, 1);
        assert_eq!(
            coordinate_bytes(source, &system_context.actual_coordinate),
            b"&uniq 'c deref(destination)"
        );
        let user_write = target
            .carrier
            .iter()
            .find(|step| step.call_role == Some(crate::ProvenanceCarrierCallRole::UserWrite))
            .expect("cross-function projected-write edge");
        let user_context = user_write
            .write_context
            .as_ref()
            .expect("user write context");
        assert_eq!(user_context.parameter, 1);
        assert_eq!(
            coordinate_bytes(source, &user_context.actual_coordinate),
            b"&uniq 'd bytes"
        );
        assert_eq!(
            target
                .carrier
                .iter()
                .filter(|step| {
                    step.call_role == Some(crate::ProvenanceCarrierCallRole::SystemWrite)
                })
                .count(),
            1,
            "the exact writable actual is nested context, not a second edge"
        );
    });
}

#[test]
fn an_alias_whole_place_write_taints_the_resolved_owner_and_retains_its_set_carrier() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let values = buffer_new(4_u64, 0_u8);
  let position = 0_u64;
  region 'a {
    let external_value = args_count<'a>(args: &'a args);
    region 'write {
      let holder = &uniq 'write position;
      set deref(holder) = external_value;
    }
    let room = len(values);
    claim bounded: ilt(position, room) because "borrowed whole write";
    let selected = values[position];
    return exit_status(code: selected);
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"set deref(holder) = external_value;"
        }));
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate)
                == b"let external_value = args_count<'a>(args: &'a args);"
        }));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_simple_borrow_holder_route_keeps_the_holder_let_and_borrow_atom() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus traps {
  region 'a {
    let raw = args_count<'a>(args: &'a args);
    region 'r {
      let holder = &'r raw;
      let position = deref(holder);
      let values = array_new<u8, 4>(0_u8);
      let room = len(values);
      claim bounded: ilt(position, room) because "borrowed scalar";
      let selected = values[position];
      return exit_status(code: selected);
    }
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"let holder = &'r raw;"
        }));
        assert!(
            target
                .carrier
                .iter()
                .any(|step| coordinate_bytes(source, &step.coordinate) == b"&'r raw")
        );
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_borrowed_match_payload_deref_reconstructs_a_prv3_carrier() {
    let source = br#"enum Choice {
  Item(value: u64);
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  region 'a {
    let raw = args_count<'a>(args: &'a args);
    let choice = Item(value: raw);
    region 'r {
      let holder = &'r choice;
      match deref(holder) {
        Item(value: selected) => {
          let index = deref(selected);
          let values = array_new<u8, 4>(0_u8);
          let room = len(values);
          claim bounded: ilt(index, room) because "borrowed payload";
          let value = values[index];
          return exit_status(code: value);
        }
      }
    }
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[index]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"let holder = &'r choice;"
        }));
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"let index = deref(selected);"
        }));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_value_match_payload_storage_route_keeps_its_fieldbind_carrier() {
    let source = br#"const count: u64 = 4_u64;

enum Wrap {
  Data(values: array<u64, count>);
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  region 'a {
    let raw = args_count<'a>(args: &'a args);
    let seeded = array_new<u64, count>(raw);
    let wrapped = Data(values: move seeded);
    let selected = match move wrapped {
      Data(values: payload) => {
        let position = payload[0_u64];
        let output = array_new<u8, count>(0_u8);
        let room = len(output);
        claim bounded: ilt(position, room) because "value-match payload";
        let value = output[position];
        give value;
      }
    }
    return exit_status(code: selected);
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(
            target
                .carrier
                .iter()
                .any(|step| { coordinate_bytes(source, &step.coordinate) == b"values: payload" })
        );
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_parameter_backed_user_result_keeps_distinct_result_and_substitution_edges() {
    let source = br#"fn relay(value: own u64) -> own u64 pure {
  let copied = value;
  return copied;
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let outside = args_count<'a>(args: &'a args);
    let position = relay(value: outside);
    let room = len(values);
    claim bounded: ilt(position, room) because "user result carrier";
    let selected = values[position];
    return exit_status(code: selected);
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[detail.selected_target as usize];
        let call_steps = target
            .carrier
            .iter()
            .filter(|step| coordinate_bytes(source, &step.coordinate) == b"relay(value: outside)")
            .collect::<Vec<_>>();
        assert_eq!(call_steps.len(), 2);
        assert_eq!(
            call_steps[0].call_role,
            Some(crate::ProvenanceCarrierCallRole::UserResult)
        );
        assert_eq!(
            call_steps[1].call_role,
            Some(crate::ProvenanceCarrierCallRole::UserSubstitution)
        );
        assert!(call_steps.iter().all(|step| step.write_context.is_none()));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_parameter_backed_user_write_keeps_distinct_write_and_substitution_edges() {
    let source = br#"fn store['r](output: &uniq 'r u64, value: own u64) -> own unit writes('r) {
  set deref(output) = value;
  return unit;
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  let saved = 0_u64;
  region 'a {
    let outside = args_count<'a>(args: &'a args);
    region 'w {
      store<'w>(output: &uniq 'w saved, value: outside);
    }
  }
  let position = saved;
  let room = len(values);
  claim bounded: ilt(position, room) because "user write carrier";
  let selected = values[position];
  return exit_status(code: selected);
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[position]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[detail.selected_target as usize];
        let call_steps = target
            .carrier
            .iter()
            .filter(|step| {
                coordinate_bytes(source, &step.coordinate)
                    == b"store<'w>(output: &uniq 'w saved, value: outside)"
            })
            .collect::<Vec<_>>();
        assert_eq!(call_steps.len(), 2);
        assert_eq!(
            call_steps[0].call_role,
            Some(crate::ProvenanceCarrierCallRole::UserWrite)
        );
        let context = call_steps[0]
            .write_context
            .as_ref()
            .expect("the projected-write edge retains its destination context");
        assert_eq!(context.parameter, 0);
        assert_eq!(
            coordinate_bytes(source, &context.actual_coordinate),
            b"&uniq 'w saved"
        );
        assert_eq!(
            call_steps[1].call_role,
            Some(crate::ProvenanceCarrierCallRole::UserSubstitution)
        );
        assert!(call_steps[1].write_context.is_none());
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_selected_payload_witness_never_uses_an_external_sibling_root_path() {
    let source = br#"enum Choice {
  First(value: u64);
  Second(value: u64);
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  region 'a {
    let first = args_count<'a>(args: &'a args);
    let second = args_count<'a>(args: &'a args);
    let choose_first = True();
    let choice = if choose_first {
      give First(value: first);
    } else {
      give Second(value: second);
    }
    match choice {
      First(value: selected) => {
        let values = array_new<u8, 4>(0_u8);
        let room = len(values);
        claim bounded: ilt(selected, room) because "selected sibling only";
        let value = values[selected];
        return exit_status(code: value);
      }
      Second(value: ignored) => {
        return exit_status(code: 0_u8);
      }
    }
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[selected]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        let first_call = source
            .windows(b"args_count<'a>(args: &'a args)".len())
            .position(|window| window == b"args_count<'a>(args: &'a args)")
            .expect("first system call") as u64;
        assert_eq!(target.origin_coordinate.start().value(), first_call);
        assert!(target.carrier.iter().any(|step| {
            step.selector
                == crate::ProvenanceDatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                }
        }));
    });
}

#[test]
fn an_external_result_payload_keeps_selectors_through_value_delivery_and_outer_enum() {
    let source = br#"enum Wrapped {
  Present(value: u64);
  Missing();
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  region 'a {
    let raw = args_count<'a>(args: &'a args);
    let wrapped = match cvt<u64, u8>(raw) {
      Ok(value: small) => {
        let widened = cvt<u8, u64>(small);
        give Present(value: widened);
      }
      Err(error: narrow) => {
        give Missing();
      }
    }
    match wrapped {
      Present(value: selected) => {
        let values = array_new<u8, 4>(0_u8);
        let room = len(values);
        claim bounded: ilt(selected, room) because "payload carrier";
        let value = values[selected];
        return exit_status(code: value);
      }
      Missing() => {
        return exit_status(code: 0_u8);
      }
    }
  }
}
"#;

    inspect_provenance_issue(source, "PRV-3", b"[selected]", |kind| {
        let crate::SemanticIssueKind::ExternalProtectedSubject(detail) = kind else {
            panic!("expected structured PRV-3 detail: {kind:?}");
        };
        let target = &detail.targets[0];
        assert!(
            target
                .carrier
                .iter()
                .filter(|step| matches!(
                    step.selector,
                    crate::ProvenanceDatumSelector::EnumPayload { .. }
                ))
                .count()
                >= 3
        );
        assert!(target.carrier.iter().any(|step| {
            coordinate_bytes(source, &step.coordinate) == b"give Present(value: widened);"
        }));
        assert_eq!(
            coordinate_bytes(source, &target.origin_coordinate),
            b"args_count<'a>(args: &'a args)"
        );
    });
}

#[test]
fn a_real_branch_discharges_the_same_external_subject_without_a_provenance_rejection() {
    let source = br#"command fn main(command.args as args: own Args) -> own ExitStatus pure {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let room = len(values);
    if ilt(position, room) {
      let selected = values[position];
      return exit_status(code: selected);
    } else {
      return exit_status(code: 1_u8);
    }
  }
}
"#;

    checked(source, |program| {
        assert_eq!(program.provenance.local_leaf_dispositions.len(), 1);
        let disposition = &program.provenance.local_leaf_dispositions[0];
        assert!(disposition.complete_discharged);
        assert!(disposition.unasserted_discharged);
        assert!(disposition.s4_blinded_discharged);
        assert_eq!(
            disposition.disposition,
            LocalLeafProvenanceDisposition::BlindedDischarged
        );
    });
}

#[test]
fn a_local_s4_bounds_leaf_retains_only_its_offset_subject() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "position in values";
} {
  return values[position];
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let metadata = &program.provenance;
        assert_eq!(metadata.structural_bridges.len(), 1);
        assert!(matches!(
            metadata.structural_bridges[0].predecessor,
            StructuralPredecessor::Local
        ));
        assert_eq!(metadata.subject_bridges.len(), 1);
        assert_eq!(
            metadata.subject_bridges[0].subject,
            ParameterDatum {
                ordinal: 1,
                selector: DatumSelector::Plain,
            }
        );
        assert!(matches!(
            metadata.subject_bridges[0].predecessor,
            SubjectPredecessor::Local
        ));
        assert!(metadata.calls.is_empty());
    });
}

#[test]
fn a_subjectless_leaf_still_retains_its_structural_bridge() {
    let source = br#"fn first(values: own buffer<u8>) -> own u8 pure requires {
  let room = len(values);
  check ilt(0_u64, room) else trap "nonempty";
} {
  return values[0_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        assert_eq!(program.provenance.structural_bridges.len(), 1);
        assert!(program.provenance.subject_bridges.is_empty());
    });
}

#[test]
fn bridges_compose_two_hops_and_through_a_local_value_transform() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "read bound";
} {
  return values[position];
}

fn relay(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "relay bound";
} {
  return read(values: move values, position: position);
}

fn transformed(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "outer bound";
} {
  let shifted = position +wrap 0_u64;
  return relay(values: move values, position: shifted);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let read = function(program, "read");
        let relay = function(program, "relay");
        let transformed = function(program, "transformed");
        let metadata = &program.provenance;

        assert_eq!(metadata.structural_bridges.len(), 3);
        let read_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == read)
            .expect("read local bridge");
        assert!(matches!(
            read_bridge.predecessor,
            StructuralPredecessor::Local
        ));
        let relay_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == relay)
            .expect("relay bridge");
        assert!(matches!(
            &relay_bridge.predecessor,
            StructuralPredecessor::Call {
                downstream_requirement,
                ..
            } if downstream_requirement.function == read
        ));
        let transformed_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == transformed)
            .expect("transformed bridge");
        assert!(matches!(
            &transformed_bridge.predecessor,
            StructuralPredecessor::Call {
                downstream_requirement,
                ..
            } if downstream_requirement.function == relay
        ));

        assert_eq!(metadata.subject_bridges.len(), 3);
        assert!(metadata.subject_bridges.iter().all(|bridge| {
            bridge.subject
                == ParameterDatum {
                    ordinal: 1,
                    selector: DatumSelector::Plain,
                }
        }));
        assert_eq!(metadata.calls.len(), 2);
        assert!(metadata.calls.iter().all(|call| {
            call.full.actual_obligations_ok
                && call.unasserted.actual_obligations_ok
                && call.s4_blinded.actual_obligations_ok
                && call.full.goal_disposition
                    == super::super::entailment::CallGoalDisposition::Discharged
                && call.unasserted.goal_disposition
                    == super::super::entailment::CallGoalDisposition::Discharged
                && call.s4_blinded.goal_disposition
                    != super::super::entailment::CallGoalDisposition::Discharged
                && call.upstream_requirement.is_some()
        }));
    });
}

#[test]
fn an_equal_length_bridge_diamond_derives_its_legacy_predecessor_from_the_full_route() {
    let source = br#"const count: u64 = 4_u64;

fn leaf(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "leaf bound";
} {
  return values[position];
}

fn left(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "left bound";
} {
  return leaf(values: move values, position: position);
}

fn right(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "right bound";
} {
  return leaf(values: move values, position: position);
}

fn diamond(values: own array<u8, count>, position: own u64, choose_left: own Bool) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "diamond bound";
} {
  if choose_left {
    return left(values: move values, position: position);
  } else {
    return right(values: move values, position: position);
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let diamond = function(program, "diamond");
        let leaf = function(program, "leaf");
        let bridge = program
            .provenance
            .subject_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == diamond && bridge.leaf.function == leaf)
            .expect("diamond bridge to the shared leaf");
        assert_eq!(bridge.boundaries.len(), 2);
        let boundary = bridge.boundaries.last().expect("immediate predecessor");
        let SubjectPredecessor::Call {
            call,
            argument,
            downstream_requirement,
            downstream_subject,
        } = &bridge.predecessor
        else {
            panic!("diamond must select one call predecessor");
        };
        assert_eq!(call, &boundary.call);
        assert_eq!(*argument, boundary.argument);
        let super::super::provenance::DemandState::Bridge {
            requirement,
            subject,
            ..
        } = &boundary.callee
        else {
            panic!("subject bridge boundary must retain a bridge callee state");
        };
        assert_eq!(downstream_requirement, requirement);
        assert_eq!(downstream_subject, subject);
        assert_eq!(
            boundary.caller_continuation,
            Some(super::super::provenance::DemandState::Bridge {
                requirement: bridge.requirement.clone(),
                subject: bridge.subject,
                leaf: bridge.leaf.clone(),
            })
        );
    });
}

#[test]
fn recursive_and_mutually_recursive_bridges_converge_from_local_seeds() {
    let source = br#"const count: u64 = 4_u64;

fn self_read(values: own array<u8, count>, position: own u64, again: own Bool) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "self bound";
} {
  if again {
    let stop = False();
    return self_read(values: move values, position: position, again: stop);
  } else {
    return values[position];
  }
}

fn left(values: own array<u8, count>, position: own u64, again: own Bool) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "left bound";
} {
  return right(values: move values, position: position, again: again);
}

fn right(values: own array<u8, count>, position: own u64, again: own Bool) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "right bound";
} {
  if again {
    let stop = False();
    return left(values: move values, position: position, again: stop);
  } else {
    return values[position];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let self_read = function(program, "self_read");
        let left = function(program, "left");
        let right = function(program, "right");
        let metadata = &program.provenance;

        let self_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == self_read)
            .expect("recursive local bridge");
        assert!(matches!(
            self_bridge.predecessor,
            StructuralPredecessor::Local
        ));
        let left_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == left)
            .expect("mutually recursive inherited bridge");
        assert!(matches!(
            &left_bridge.predecessor,
            StructuralPredecessor::Call {
                downstream_requirement,
                ..
            } if downstream_requirement.function == right
        ));
        let right_bridge = metadata
            .structural_bridges
            .iter()
            .find(|bridge| bridge.requirement.function == right)
            .expect("mutually recursive local bridge");
        assert!(matches!(
            right_bridge.predecessor,
            StructuralPredecessor::Local
        ));

        assert_eq!(metadata.structural_bridges.len(), 3);
        assert_eq!(metadata.subject_bridges.len(), 3);
        assert_eq!(metadata.calls.len(), 3);
        assert!(
            metadata
                .calls
                .iter()
                .all(|call| call.upstream_requirement.is_some())
        );

        let result_parameters = [
            ParameterDatum {
                ordinal: 0,
                selector: DatumSelector::Plain,
            },
            ParameterDatum {
                ordinal: 1,
                selector: DatumSelector::Plain,
            },
        ];
        for id in [self_read, left, right] {
            let dependencies = metadata
                .functions
                .iter()
                .find(|dependencies| dependencies.function == id)
                .expect("recursive dependency summary");
            assert_eq!(
                dependencies.result.components[0]
                    .dependency
                    .parameters
                    .datums,
                result_parameters,
                "result composition reaches the seeded read but excludes the control Bool"
            );
        }
    });
}

#[test]
fn a_seedless_requirement_cycle_has_no_bridge_or_call_link() {
    let source = br#"fn empty_left(value: own u64) -> own unit pure requires {
  check ilt(value, 10_u64) else trap "left bound";
} {
  return empty_right(value: value);
}

fn empty_right(value: own u64) -> own unit pure requires {
  check ilt(value, 10_u64) else trap "right bound";
} {
  return empty_left(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        assert!(program.provenance.structural_bridges.is_empty());
        assert!(program.provenance.subject_bridges.is_empty());
        assert!(program.provenance.calls.is_empty());
    });
}

#[test]
fn a_counterfactual_call_goal_keeps_actual_obligation_failure_separate() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>, position: own u64) -> own u8 pure requires {
  let room = len(values);
  check ilt(position, room) else trap "read bound";
} {
  return values[position];
}

fn counterfactual(values: own array<u8, count>, positions: own array<u64, count>, selector: own u64) -> own u8 traps {
  check ilt(selector, 0_u64) else trap "unreachable call";
  return read(values: move values, position: positions[selector]);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let caller = function(program, "counterfactual");
        let call = program
            .provenance
            .calls
            .iter()
            .find(|call| call.caller == caller)
            .expect("accepted full-state call link");
        assert!(call.full.actual_obligations_ok);
        assert!(!call.unasserted.actual_obligations_ok);
        assert!(!call.s4_blinded.actual_obligations_ok);
        assert_ne!(
            call.unasserted.goal_disposition,
            CallGoalDisposition::Discharged
        );
        assert!(call.upstream_requirement.is_none());
    });
}

#[test]
fn call_write_dependencies_compose_through_a_moved_unique_parameter() {
    let source = br#"fn write['r](out: &uniq 'r u64, value: own u64) -> own unit writes('r) {
  set deref(out) = value;
  return unit;
}

fn proxy['r](out: &uniq 'r u64, value: own u64) -> own unit writes('r) {
  write<'r>(out: move out, value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let expected = vec![ParameterDatum {
            ordinal: 1,
            selector: DatumSelector::Plain,
        }];
        for name in ["write", "proxy"] {
            let id = function(program, name);
            let dependencies = program
                .provenance
                .functions
                .iter()
                .find(|dependencies| dependencies.function == id)
                .unwrap_or_else(|| panic!("missing dependencies for {name}"));
            assert_eq!(dependencies.writes[0].parameters.datums, expected);
            assert!(dependencies.writes[1].parameters.datums.is_empty());
        }
        assert_eq!(program.provenance.call_argument_dispositions.len(), 2);
        assert!(
            program
                .provenance
                .call_argument_dispositions
                .iter()
                .all(|argument| {
                    argument.disposition == CallArgumentProvenanceDisposition::NoEvent
                        && argument.complete == ProvenanceGoalObservation::NotApplicable
                        && argument.unasserted == ProvenanceGoalObservation::NotApplicable
                        && argument.s4_blinded == ProvenanceGoalObservation::NotApplicable
                })
        );
    });
}

#[test]
fn a_unique_match_payload_write_resolves_to_the_scrutinee_root() {
    let source = br#"enum Payload {
  Item(value: u64);
}

fn update['r](input: &uniq 'r Payload, replacement: own u64) -> own unit reads('r), writes('r) {
  match deref(input) {
    Item(value: selected) => {
      set deref(selected) = replacement;
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let update = function(program, "update");
        let dependencies = program
            .provenance
            .functions
            .iter()
            .find(|dependencies| dependencies.function == update)
            .expect("update dependencies");
        assert_eq!(
            dependencies.writes[0].parameters.datums,
            vec![ParameterDatum {
                ordinal: 1,
                selector: DatumSelector::Plain,
            }]
        );
        assert!(dependencies.writes[1].parameters.datums.is_empty());
    });
}

#[test]
fn a_unique_boxed_match_payload_write_resolves_to_the_outer_scrutinee_root() {
    let source = br#"enum Payload {
  Item(value: u64);
}

fn update['r](input: &uniq 'r box<Payload>, replacement: own u64) -> own unit reads('r), writes('r) {
  match deref(deref(input)) {
    Item(value: selected) => {
      set deref(selected) = replacement;
    }
  }
  return unit;
}

fn proxy['r](input: &uniq 'r box<Payload>, replacement: own u64) -> own unit reads('r), writes('r) {
  update<'r>(input: move input, replacement: replacement);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        for name in ["update", "proxy"] {
            let function = function(program, name);
            let dependencies = program
                .provenance
                .functions
                .iter()
                .find(|dependencies| dependencies.function == function)
                .unwrap_or_else(|| panic!("missing dependencies for {name}"));
            assert_eq!(
                dependencies.writes[0].parameters.datums,
                vec![ParameterDatum {
                    ordinal: 1,
                    selector: DatumSelector::Plain,
                }]
            );
            assert!(dependencies.writes[1].parameters.datums.is_empty());
        }
    });
}

#[test]
fn direct_enum_payload_dependencies_survive_match_and_call_composition() {
    let source = br#"enum Choice {
  Value(data: u64);
  Empty();
}

fn choose(input: own Choice, fallback: own u64) -> own u64 pure {
  match move input {
    Value(data: selected) => {
      return selected;
    }
    Empty() => {
      return fallback;
    }
  }
}

fn relay(input: own Choice, fallback: own u64) -> own u64 pure {
  return choose(input: move input, fallback: fallback);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let expected = vec![
            ParameterDatum {
                ordinal: 0,
                selector: DatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                },
            },
            ParameterDatum {
                ordinal: 1,
                selector: DatumSelector::Plain,
            },
        ];
        for name in ["choose", "relay"] {
            let id = function(program, name);
            let dependencies = program
                .provenance
                .functions
                .iter()
                .find(|dependencies| dependencies.function == id)
                .unwrap_or_else(|| panic!("missing dependencies for {name}"));
            assert_eq!(
                dependencies.result.components[0]
                    .dependency
                    .parameters
                    .datums,
                expected
            );
        }
    });
}

#[test]
fn checked_arithmetic_and_partial_conversion_seed_only_the_ok_projection() {
    let source = br#"fn add(value: own u64) -> own Result<u64, Overflow> pure {
  return value +checked 1_u64;
}

fn narrow(value: own u64) -> own Result<u8, NarrowError> pure {
  return cvt<u64, u8>(value);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let expected = [ParameterDatum {
            ordinal: 0,
            selector: DatumSelector::Plain,
        }];
        for name in ["add", "narrow"] {
            let result = &dependencies(program, name).result;
            assert_eq!(
                projection(
                    result,
                    DatumSelector::EnumPayload {
                        variant: 0,
                        field: 0,
                    },
                ),
                expected
            );
            assert!(
                projection(
                    result,
                    DatumSelector::EnumPayload {
                        variant: 1,
                        field: 0,
                    },
                )
                .is_empty(),
                "{name}'s error projection is not data-derived from its operand"
            );
        }
    });
}

#[test]
fn propagation_keeps_ok_and_error_dependencies_componentwise() {
    let source =
        br#"fn forward(input: own Result<u8, NarrowError>) -> own Result<u8, NarrowError> pure {
  let value = propagate input;
  return Ok<u8, NarrowError>(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let result = &dependencies(program, "forward").result;
        assert_eq!(
            projection(
                result,
                DatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                },
            ),
            [ParameterDatum {
                ordinal: 0,
                selector: DatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                },
            }]
        );
        assert_eq!(
            projection(
                result,
                DatumSelector::EnumPayload {
                    variant: 1,
                    field: 0,
                },
            ),
            [ParameterDatum {
                ordinal: 0,
                selector: DatumSelector::EnumPayload {
                    variant: 1,
                    field: 0,
                },
            }]
        );
    });
}

#[test]
fn a_nested_payload_aggregate_seeds_every_direct_payload_projection() {
    let source = br#"enum Inner {
  Left(value: u64);
  Right(value: u64);
}

enum Outer {
  Wrap(value: Inner);
}

fn unwrap(input: own Outer) -> own Inner pure {
  match move input {
    Wrap(value: nested) => {
      return move nested;
    }
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let result = &dependencies(program, "unwrap").result;
        let expected = [ParameterDatum {
            ordinal: 0,
            selector: DatumSelector::EnumPayload {
                variant: 0,
                field: 0,
            },
        }];
        assert_eq!(
            projection(
                result,
                DatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                },
            ),
            expected
        );
        assert_eq!(
            projection(
                result,
                DatumSelector::EnumPayload {
                    variant: 1,
                    field: 0,
                },
            ),
            expected,
            "the aggregate nested payload seeds every direct Inner projection"
        );
    });
}

#[test]
fn a_counted_binder_depends_on_its_lower_endpoint_only() {
    let source = br#"fn walk(lower: own u64, upper: own u64) -> own unit pure {
  for @items position in lower..upper {
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    checked(source, |program| {
        let walk = program
            .functions
            .iter()
            .find(|function| function.name == "walk")
            .expect("walk function");
        let CheckedStatement::CountedRange { binder, .. } = &walk.body[0] else {
            panic!("walk keeps its counted range");
        };
        let binder = dependencies(program, "walk").bindings[binder.0 as usize]
            .as_ref()
            .expect("counted binder dependencies");
        assert_eq!(
            projection(binder, DatumSelector::Plain),
            [ParameterDatum {
                ordinal: 0,
                selector: DatumSelector::Plain,
            }]
        );
    });
}

#[test]
fn system_results_and_writes_add_no_parameter_datum() {
    let source = br#"fn publish['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, count: own u64) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: count) {
      Ok(value: written) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch = buffer_new(1_u64, 0_u8);
  region 'publication {
    publish<'publication, 'publication>(output: &uniq 'publication out, source: &'publication batch, count: 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;

    checked(source, |program| {
        let publish = program
            .functions
            .iter()
            .find(|function| function.name == "publish")
            .expect("publish function");
        let CheckedStatement::Region { body, .. } = &publish.body[0] else {
            panic!("publish keeps its attempt region");
        };
        let CheckedStatement::Match { arms, .. } = &body[0] else {
            panic!("publish keeps its system outcome match");
        };
        let dependencies = dependencies(program, "publish");
        assert!(
            dependencies.writes[0].parameters.datums.is_empty(),
            "a system write does not derive the written resource from call arguments"
        );
        assert!(
            dependencies.writes[0].unconditional_external,
            "the SYS-2 write_once output component is unconditional external"
        );
        for arm in arms {
            for binder in &arm.binders {
                let value = dependencies.bindings[binder.binding.0 as usize]
                    .as_ref()
                    .expect("system result binder dependencies");
                assert!(
                    value.components.iter().all(|component| component
                        .dependency
                        .parameters
                        .datums
                        .is_empty()),
                    "a system result payload does not derive from call arguments"
                );
                assert_eq!(
                    value.components[0].dependency.unconditional_external,
                    arm.tag == 1,
                    "write_once Ok(value:) is internal and only Err(error:) is SYS-2 external"
                );
            }
        }
    });
}

/// The canonical raw-DEFLATE provenance gate: one subject bridge and three
/// unasserted downstream calls.
///
/// This runs inside `entailment.rs`'s frozen real-source corpus walk so the
/// two gates share one front-end pass over the same 56 KB four-file bundle;
/// the standalone `#[ignore]`d test that re-analyzed it was removed on
/// 2026-08-16. If the DEFLATE bundle ever leaves that corpus, restore a
/// standalone test here that analyzes the bundle and calls this function.
pub(super) fn assert_canonical_deflate_provenance(program: &CheckedProgramData) {
    let store = function(program, "store_dynamic_length");
    let decode = function(program, "decode_dynamic");
    let metadata = &program.provenance;

    assert_eq!(metadata.structural_bridges.len(), 1);
    assert_eq!(metadata.structural_bridges[0].requirement.function, store);
    assert!(matches!(
        metadata.structural_bridges[0].predecessor,
        StructuralPredecessor::Local
    ));
    assert_eq!(metadata.subject_bridges.len(), 1);
    assert_eq!(
        metadata.subject_bridges[0].subject,
        ParameterDatum {
            ordinal: 3,
            selector: DatumSelector::Plain,
        }
    );

    let calls = metadata
        .calls
        .iter()
        .filter(|call| call.caller == decode && call.downstream_requirement.function == store)
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 3);
    for call in calls {
        assert!(call.full.actual_obligations_ok);
        assert!(call.unasserted.actual_obligations_ok);
        assert!(call.s4_blinded.actual_obligations_ok);
        assert_eq!(
            call.unasserted.goal_disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            call.s4_blinded.goal_disposition,
            CallGoalDisposition::Discharged
        );
        assert!(call.upstream_requirement.is_none());
        assert_eq!(call.subjects.len(), 1);
        assert_eq!(call.subjects[0].argument, 3);
        assert_eq!(call.subjects[0].callee_subject.ordinal, 3);
    }

    let store_function = program
        .functions
        .iter()
        .find(|function| function.id == store)
        .expect("store_dynamic_length function");
    assert!(
        !store_function
            .entailment
            .claims
            .iter()
            .any(|claim| claim.name == "distance_position_in_lengths")
    );
    assert_eq!(
        program
            .functions
            .iter()
            .map(|function| function.entailment.claims.len())
            .sum::<usize>(),
        12
    );
}
