use crate::{SemanticOutcome, SourceInput};

use super::super::entailment::CallGoalDisposition;
use super::super::model::{CheckedProgramData, CheckedStatement, FunctionId};
use super::super::provenance::{
    DatumSelector, FunctionDependencies, ParameterDatum, StructuralPredecessor, SubjectPredecessor,
    ValueDependencies,
};
use super::{with_semantics, with_semantics_inputs};

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
        .parameters
        .datums
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
                dependencies.result.components[0].parameters.datums, result_parameters,
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
            assert_eq!(dependencies.writes[0].datums, expected);
            assert!(dependencies.writes[1].datums.is_empty());
        }
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
            dependencies.writes[0].datums,
            vec![ParameterDatum {
                ordinal: 1,
                selector: DatumSelector::Plain,
            }]
        );
        assert!(dependencies.writes[1].datums.is_empty());
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
                dependencies.writes[0].datums,
                vec![ParameterDatum {
                    ordinal: 1,
                    selector: DatumSelector::Plain,
                }]
            );
            assert!(dependencies.writes[1].datums.is_empty());
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
                dependencies.result.components[0].parameters.datums,
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
            dependencies.writes[0].datums.is_empty(),
            "a system write does not derive the written resource from call arguments"
        );
        for binder in arms.iter().flat_map(|arm| &arm.binders) {
            let value = dependencies.bindings[binder.binding.0 as usize]
                .as_ref()
                .expect("system result binder dependencies");
            assert!(
                value
                    .components
                    .iter()
                    .all(|component| component.parameters.datums.is_empty()),
                "a system result payload does not derive from call arguments"
            );
        }
    });
}

#[test]
fn canonical_deflate_retains_one_subject_bridge_and_three_unasserted_calls() {
    let inputs = [
        SourceInput::new(
            "raw_deflate.wf",
            include_bytes!("../../../../tests/programs/raw_deflate.wf"),
        ),
        SourceInput::new(
            "raw_deflate_dynamic.wf",
            include_bytes!("../../../../tests/programs/raw_deflate_dynamic.wf"),
        ),
        SourceInput::new(
            "raw_deflate_dynamic_decode.wf",
            include_bytes!("../../../../tests/programs/raw_deflate_dynamic_decode.wf"),
        ),
        SourceInput::new(
            "raw_deflate_boundary.wf",
            include_bytes!("../../../../tests/programs/raw_deflate_boundary.wf"),
        ),
    ];
    with_semantics_inputs(&inputs, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("canonical raw-deflate bundle must remain accepted: {outcome:?}");
        };
        let store = function(&program.data, "store_dynamic_length");
        let decode = function(&program.data, "decode_dynamic");
        let metadata = &program.data.provenance;

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
            .data
            .functions
            .iter()
            .find(|function| function.id == store)
            .expect("store_dynamic_length function");
        assert!(
            store_function
                .entailment
                .claims
                .iter()
                .any(|claim| { claim.name == "distance_position_in_lengths" })
        );
    });
}
