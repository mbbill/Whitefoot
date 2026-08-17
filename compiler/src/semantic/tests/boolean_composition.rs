//! O11 signed-Boolean-decomposition tests, in both directions.
//!
//! Direction one: establishment records and establishes exactly the sound
//! decomposition set — `+band` and `-bor` decompose into their signed
//! children recursively, `bnot` flips, and `-band`, `+bor`, and `bxor` on
//! either sign contribute nothing (the classic asymmetry: the other sign's
//! content is genuinely disjunctive). Direction two: decomposition never runs
//! upward — children still establish nothing about a parent, so a caller
//! discharges a composed requirement only by exact-whole-tree evidence.
//!
//! The obligations these guards protect now discharge, which is the rule's
//! purpose: the [ENT-3] members are facts at their establishment point.
//! Design: `research/investigations/o11-composition/DESIGN.md`.

use crate::SemanticOutcome;

use super::super::entailment::{
    BooleanGoalDecomposition, FunctionEntailment, GoalSign, Relation, TermKind,
};
use super::super::goal::{GoalExpression, GoalOperation};
use super::super::model::CheckedBooleanOperation;
use super::{with_semantics, with_semantics_dark};

fn entailment(source: &[u8], function: &str) -> FunctionEntailment {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("boolean-composition test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.clone()
    })
}

/// The recorded parent's retained expression must have the expected Boolean
/// root; returns the entry for further member assertions.
fn entry_with_root(
    summary: &FunctionEntailment,
    operation: CheckedBooleanOperation,
    sign: GoalSign,
) -> &BooleanGoalDecomposition {
    summary
        .boolean_decompositions
        .iter()
        .find(|candidate| {
            candidate.sign == sign
                && matches!(
                    &summary.inventory.goals[candidate.parent.0 as usize].expression,
                    GoalExpression::Operation {
                        row: GoalOperation::Boolean(root),
                        ..
                    } if *root == operation
                )
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a recorded {operation:?}/{sign:?} decomposition, got {:?}",
                summary.boolean_decompositions
            )
        })
}

/// Asserts one member is a comparison goal whose retained projection is the
/// exact normalized bound `place - constant <= bound` in operand order.
fn assert_comparison_member(
    summary: &FunctionEntailment,
    member: (super::super::entailment::GoalId, GoalSign),
    sign: GoalSign,
    constant: i128,
    bound: i128,
) {
    assert_eq!(member.1, sign);
    let retained = &summary.inventory.goals[member.0.0 as usize];
    assert!(
        matches!(
            &retained.expression,
            GoalExpression::Operation {
                row: GoalOperation::Integer { .. },
                ..
            }
        ),
        "member must be a comparison goal: {retained:?}"
    );
    let Some(Relation::Bound {
        left,
        right,
        bound: held,
    }) = &retained.projection
    else {
        panic!("comparison member must retain its exact projection: {retained:?}");
    };
    assert!(
        matches!(
            summary.inventory.terms[left.0 as usize],
            TermKind::Place(..)
        ),
        "projection left operand must be the tracked place"
    );
    assert_eq!(
        summary.inventory.terms[right.0 as usize],
        TermKind::Constant(constant)
    );
    assert_eq!(*held, bound);
}

/// S2: a passed `check` on a `band` tree establishes the positive conjuncts
/// with their exact projections, so both guarded subscripts discharge from
/// the one conjoined guard.
#[test]
fn passed_band_check_establishes_positive_conjuncts_and_discharges_both() {
    let source =
        br#"fn read_pair(table: own array<u8, 8>, low: own u64, high: own u64) -> own u8 traps {
  let low_ok = ilt(low, 8_u64);
  let high_ok = ilt(high, 8_u64);
  let both = band(low_ok, high_ok);
  check both else trap "pair in range";
  let first = table[low];
  let second = table[high];
  return second;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read_pair");
    // Each conjunct is established, so each subscript discharges.
    assert_eq!(summary.obligations.len(), 2);
    assert!(summary.obligations.iter().all(|o| o.discharged));
    // Exactly one band entry with the two conjuncts.
    assert_eq!(summary.boolean_decompositions.len(), 1);
    let entry = entry_with_root(&summary, CheckedBooleanOperation::And, GoalSign::Positive);
    assert_eq!(entry.members.len(), 2);
    // ilt(low, 8) normalizes to low - 8 <= -1; same for high.
    assert_comparison_member(&summary, entry.members[0], GoalSign::Positive, 8, -1);
    assert_comparison_member(&summary, entry.members[1], GoalSign::Positive, 8, -1);
    assert_ne!(entry.members[0].0, entry.members[1].0);
}

/// S1: the ruled-flip guard shape. The `bor` else edge establishes both
/// negative disjuncts (whose projections negate to the two-sided bound) and
/// contributes nothing on the true edge, so the guarded subscript discharges
/// on the edge the guard protects.
#[test]
fn bor_guard_false_edge_establishes_negative_disjuncts_and_discharges() {
    let source = br#"fn get(table: own array<u8, 4>, symbol: own u64) -> own u8 pure {
  let below = ilt(symbol, 0_u64);
  let above = ige(symbol, 4_u64);
  let invalid = bor(below, above);
  if invalid {
    return 0_u8;
  } else {
    return table[symbol];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "get");
    assert_eq!(summary.obligations.len(), 1);
    assert!(summary.obligations[0].discharged);
    // Only the false edge decomposes a disjunction; +bor contributes nothing.
    assert_eq!(summary.boolean_decompositions.len(), 1);
    let entry = entry_with_root(&summary, CheckedBooleanOperation::Or, GoalSign::Negative);
    assert_eq!(entry.members.len(), 2);
    // -ilt(symbol, 0): projection symbol - 0 <= -1, negated at activation.
    assert_comparison_member(&summary, entry.members[0], GoalSign::Negative, 0, -1);
    // -ige(symbol, 4): projection normalizes ge by operand swap, 4 - symbol <= 0.
    let above = &summary.inventory.goals[entry.members[1].0.0 as usize];
    let Some(Relation::Bound { left, bound, .. }) = &above.projection else {
        panic!("the ige member must retain its projection: {above:?}");
    };
    assert_eq!(
        summary.inventory.terms[left.0 as usize],
        TermKind::Constant(4)
    );
    assert_eq!(*bound, 0);
    // The ruled flip, now live: this is the shape the owner's 2026-08-09
    // ruling (2) disposed. The protected corpus case
    // `ent3-neg-bor-no-comparison-origin.wf` still declares the v0.30
    // rejection and needs its owner-approved rewrite before the coverage
    // gate agrees with this verdict.
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the guarded subscript discharges from the decomposed else edge",
        );
    });
}

/// The asymmetry, pinned: `-band` and `+bor` carry only disjunctive content
/// and record nothing, and `bxor` records nothing on either sign.
#[test]
fn disjunctive_signs_and_bxor_record_nothing() {
    let source = br#"fn classify(a: own u64, b: own u64) -> own u64 pure {
  let a_small = ilt(a, 16_u64);
  let b_small = ilt(b, 16_u64);
  let both = band(a_small, b_small);
  let either = bor(a_small, b_small);
  let mixed = bxor(a_small, b_small);
  if both {
    return 0_u64;
  } else if either {
    return 1_u64;
  } else if mixed {
    return 2_u64;
  } else {
    return 3_u64;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "classify");
    // Exactly the +band then-edge and the -bor false-edge entries exist: no
    // -band, no +bor, and no bxor entry on either sign.
    assert_eq!(summary.boolean_decompositions.len(), 2);
    let conjunction = entry_with_root(&summary, CheckedBooleanOperation::And, GoalSign::Positive);
    assert_eq!(conjunction.members.len(), 2);
    let disjunction = entry_with_root(&summary, CheckedBooleanOperation::Or, GoalSign::Negative);
    assert_eq!(disjunction.members.len(), 2);
    assert!(
        conjunction
            .members
            .iter()
            .all(|member| member.1 == GoalSign::Positive)
    );
    assert!(
        disjunction
            .members
            .iter()
            .all(|member| member.1 == GoalSign::Negative)
    );
}

/// `bnot` flips the sign both ways, and the De Morgan shape falls out of
/// recursion, never rewriting: `+bnot(bor(A, B))` records `-bor`, `-A`,
/// `-B`; `-bnot(bor(A, B))` flips to the disjunctive sign and stops after
/// the child.
#[test]
fn bnot_flips_recursively_without_rewriting() {
    let source = br#"fn guard(table: own array<u8, 8>, index: own u64) -> own u8 pure {
  let low = ilt(index, 4_u64);
  let high = ige(index, 8_u64);
  let outside = bor(low, high);
  let inside = bnot(outside);
  if inside {
    return table[index];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "guard");
    // The recursion reaches `-ilt(index, 4)`, which discharges the subscript.
    assert_eq!(summary.obligations.len(), 1);
    assert!(summary.obligations[0].discharged);
    assert_eq!(summary.boolean_decompositions.len(), 2);
    let positive = entry_with_root(&summary, CheckedBooleanOperation::Not, GoalSign::Positive);
    assert_eq!(positive.members.len(), 3);
    let inner = &summary.inventory.goals[positive.members[0].0.0 as usize];
    assert_eq!(positive.members[0].1, GoalSign::Negative);
    assert!(matches!(
        &inner.expression,
        GoalExpression::Operation {
            row: GoalOperation::Boolean(CheckedBooleanOperation::Or),
            ..
        }
    ));
    assert_comparison_member(&summary, positive.members[1], GoalSign::Negative, 4, -1);
    assert_eq!(positive.members[2].1, GoalSign::Negative);
    // The false edge flips to +bor, which is disjunctive: one member only.
    let negative = entry_with_root(&summary, CheckedBooleanOperation::Not, GoalSign::Negative);
    assert_eq!(negative.members.len(), 1);
    assert_eq!(negative.members[0].1, GoalSign::Positive);
    assert_eq!(negative.members[0].0, positive.members[0].0);
}

/// S4: a `band` requirement goal establishes its conjuncts at body entry, so
/// the body's own subscript discharges, while the caller still discharges the
/// requirement only by exact-whole-tree evidence — the preserved
/// no-upward-composition direction.
#[test]
fn band_requirement_establishes_body_conjuncts_and_composes_nothing_upward() {
    let source =
        br#"fn pick(table: own array<u8, 8>, low: own u64, high: own u64) -> own u8 pure requires {
  let low_ok = ilt(low, 8_u64);
  let high_ok = ilt(high, 8_u64);
  let both = band(low_ok, high_ok);
  check both else trap "pair in range";
} {
  let first = table[low];
  return first;
}

fn caller(table: own array<u8, 8>, low: own u64, high: own u64) -> own u8 traps {
  let low_ok = ilt(low, 8_u64);
  let high_ok = ilt(high, 8_u64);
  let both = band(low_ok, high_ok);
  check both else trap "caller proof";
  let value = pick(table: move table, low: low, high: high);
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let callee = entailment(source, "pick");
    assert_eq!(callee.obligations.len(), 1);
    assert!(callee.obligations[0].discharged);
    let entry = entry_with_root(&callee, CheckedBooleanOperation::And, GoalSign::Positive);
    assert_eq!(entry.members.len(), 2);
    assert_comparison_member(&callee, entry.members[0], GoalSign::Positive, 8, -1);
    assert_comparison_member(&callee, entry.members[1], GoalSign::Positive, 8, -1);
    // The caller discharges the whole conjunction by exact-tree evidence:
    // children never compose a parent, so the discharge must come from the
    // established whole goal, and it does.
    let caller = entailment(source, "caller");
    assert_eq!(caller.call_goals.len(), 1);
    assert!(
        matches!(
            caller.call_goals[0].disposition,
            super::super::entailment::CallGoalDisposition::Discharged
        ),
        "whole-tree caller evidence must discharge: {:?}",
        caller.call_goals[0]
    );
}
