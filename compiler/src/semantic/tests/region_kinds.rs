//! Complete-unit memory/world region-kind inference and diagnostic ownership.

use crate::{SemanticIssueKind, SemanticRule};

use super::{assert_rule, assert_rule_at};

const COMMAND_ENTRY: &str =
    "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_declaration_cannot_be_world_then_memory_kind() {
    let source = format!(
        "fn mixed['r](output: own Output<'r, 'r>, value: &'r u64) -> result: own unit pure {{\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        source.as_bytes(),
        SemanticRule::Own3,
        SemanticIssueKind::RegionKindConflict {
            region: "'r".to_owned(),
            first: "world",
            second: "memory",
            mechanical_fix: "split the memory lifetime and world identity into two region parameters",
        },
    );
}

#[test]
fn an_unanchored_region_and_forwarding_cycle_reject_at_own3() {
    let direct = format!(
        "fn unanchored['r](value: own u64) -> result: own u64 pure {{\n  return value;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        direct.as_bytes(),
        SemanticRule::Own3,
        SemanticIssueKind::UnresolvedRegionKind {
            region: "'r".to_owned(),
        },
    );

    let cycle = format!(
        "fn left['a]() -> result: own unit pure {{\n  let done = right<'a>();\n  return unit;\n}}\n\nfn right['b]() -> result: own unit pure {{\n  let done = left<'b>();\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        cycle.as_bytes(),
        SemanticRule::Own3,
        SemanticIssueKind::UnresolvedRegionKind {
            region: "'a".to_owned(),
        },
    );
}

#[test]
fn an_effect_row_does_not_invent_a_region_kind() {
    for row in ["reads('r)", "writes('r)"] {
        let source = format!(
            "fn unanchored['r](value: own u64) -> result: own u64 {row} {{\n  return value;\n}}\n\n{COMMAND_ENTRY}"
        );
        assert_rule(
            source.as_bytes(),
            SemanticRule::Own3,
            SemanticIssueKind::UnresolvedRegionKind {
                region: "'r".to_owned(),
            },
        );
    }
}

#[test]
fn an_invalid_effect_row_precedes_a_later_call_edge_diagnostic() {
    let source = br#"fn invalid['r, 's](value: own u64) -> result: own u64 reads('r), reads('s) {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let value = invalid(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        source,
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
}

#[test]
fn a_wrong_kind_user_call_argument_is_owned_by_fn2() {
    let source = br#"fn observe['m](value: &'m u64) -> result: own u64 reads('m) {
  return deref(value);
}

command fn main['q, 'o](command.stdout as out: own Output<'q, 'o>) -> status: own ExitStatus pure {
  let value = 1_u64;
  region 'a {
    let seen = observe<'q>(value: &'a value);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Fn2, SemanticIssueKind::TypeMismatch);
}

#[test]
fn a_forward_call_uses_the_later_callee_anchor_before_linking() {
    let source = br#"fn forward['q, 'o](output: own Output<'q, 'o>) -> result: own unit pure {
  callee<'q>();
  return unit;
}

fn callee['m]() -> result: own unit allocates(arena 'm) {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn2, "callee<'q>()");
}

#[test]
fn the_first_call_edge_joining_opposite_components_owns_fn2() {
    let source = br#"fn memory['a](value: &'a u64) -> result: own unit reads('a) {
  bridge<'a>();
  return unit;
}

fn bridge['b]() -> result: own unit pure {
  world<'b>();
  return unit;
}

fn world['c](output: own Output<'c, 'c>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn2, "world<'b>()");
}

#[test]
fn a_bare_system_nominal_vector_is_owned_by_sys2() {
    let bare = format!(
        "fn invalid(output: own Output) -> result: own unit pure {{\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        bare.as_bytes(),
        SemanticRule::Sys2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn a_wrong_count_system_nominal_vector_is_owned_by_sys2() {
    let wrong_count = format!(
        "fn invalid['q](output: own Output<'q>) -> result: own unit pure {{\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        wrong_count.as_bytes(),
        SemanticRule::Sys2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn a_memory_argument_in_a_system_nominal_vector_is_owned_by_sys2() {
    let memory_argument = format!(
        "fn invalid['m](value: &'m u64, output: own Output<'m, 'm>) -> result: own unit reads('m) {{\n  let observed = deref(value);\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        memory_argument.as_bytes(),
        SemanticRule::Sys2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn a_wrong_kind_system_call_argument_is_owned_by_sys2() {
    let source = br#"command fn main['q, 'o](command.stdout as out: own Output<'q, 'o>) -> status: own ExitStatus writes('q 'o), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region 'm {
    match write_once<'q, 'm, 'q, 'o>(output: &uniq 'm out, source: &'m bytes, start: 0_u64, end: 1_u64) {
      Ok(value: next) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Sys2, SemanticIssueKind::TypeMismatch);
}

#[test]
fn a_world_region_in_an_arena_allocation_row_is_owned_by_eff1() {
    let source = format!(
        "fn invalid['q, 'o](output: own Output<'q, 'o>) -> result: own unit allocates(arena 'q) {{\n  return unit;\n}}\n\n{COMMAND_ENTRY}"
    );
    assert_rule(
        source.as_bytes(),
        SemanticRule::Eff1,
        SemanticIssueKind::InvalidEffectRow,
    );
}

#[test]
fn a_memory_kind_command_parameter_is_owned_by_fn7() {
    let source = br#"command fn main['m](command.args as args: own Args) -> status: own ExitStatus reads('m) {
  let count = args_count<'m>(args: &'m args);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Fn7, SemanticIssueKind::InvalidMain);
}

#[test]
fn contract_alpha_equality_preserves_the_ordered_region_kinds() {
    let source = br#"contract Kinded {
  fn apply['m, 'q, 'o](value: &'m u64, output: own Output<'q, 'o>) -> result: own unit pure;
}

conform u64: Kinded {
  apply = implementation;
}

fn implementation['q, 'm, 'o](value: &'m u64, output: own Output<'q, 'o>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
}
