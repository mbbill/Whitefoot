//! O11 signed-Boolean-decomposition tests, in both directions.
//!
//! Direction one: ordinary establishment records and establishes exactly the
//! sound decomposition set — `+band` and `-bor` decompose into their signed
//! children recursively, `bnot` flips, and `-band`, `+bor`, and `bxor` on
//! either sign contribute nothing (the classic asymmetry: the other sign's
//! content is genuinely disjunctive). A residual claim instead publishes its
//! canonical contribution components and reconstructs only the finite parent
//! already present in the goal universe.
//!
//! The obligations these guards protect now discharge, which is the rule's
//! purpose: the [ENT-3] members are facts at their establishment point.
//! Design: `research/investigations/o11-composition/DESIGN.md`.

use crate::SemanticOutcome;

use super::super::entailment::{
    BooleanGoalDecomposition, FunctionEntailment, GoalSign, Relation, TermKind,
};
use super::super::goal::{GoalDatum, GoalExpression, GoalOperation};
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

/// The recorded entry whose parent is the `own Bool` binding a guard or claim
/// named, rather than that binding's expanded Boolean tree. Both parents are
/// established, so both are recorded; this one carries the conjuncts in the
/// operand forms their own comparison bindings recorded.
fn binding_rooted_entry(summary: &FunctionEntailment, sign: GoalSign) -> &BooleanGoalDecomposition {
    summary
        .boolean_decompositions
        .iter()
        .find(|candidate| {
            candidate.sign == sign
                && matches!(
                    &summary.inventory.goals[candidate.parent.0 as usize].expression,
                    GoalExpression::Datum(GoalDatum::Place { .. })
                )
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a recorded binding-rooted {sign:?} decomposition, got {:?}",
                summary.boolean_decompositions
            )
        })
}

/// Asserts every member of one entry is a bare `own Bool` comparison binding
/// carried at the expected sign — the shape that delivers its relation from
/// `state.origins` rather than from a projection of its own.
fn assert_binding_members(
    summary: &FunctionEntailment,
    entry: &BooleanGoalDecomposition,
    sign: GoalSign,
) {
    for member in &entry.members {
        assert_eq!(member.1, sign);
        assert!(
            matches!(
                &summary.inventory.goals[member.0.0 as usize].expression,
                GoalExpression::Datum(GoalDatum::Place { projections, .. })
                    if projections.is_empty()
            ),
            "member {member:?} must be a bare Bool binding: {:?}",
            summary.inventory.goals[member.0.0 as usize]
        );
    }
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
        // A written zero is the zero term: one term per mathematical value.
        if constant == 0 {
            TermKind::Zero
        } else {
            TermKind::Constant(constant)
        }
    );
    assert_eq!(*held, bound);
}

/// S3: a genuine residual on a `band` tree publishes two canonical
/// contribution components, so both guarded subscripts discharge from one
/// reviewed theorem without treating the parent as an ordinary decomposition.
#[test]
fn passed_band_claim_establishes_positive_conjuncts_and_discharges_both() {
    let source =
        br#"fn clamp_seven(value: own u64) -> result: own u64 pure {
  return imin(value, 7_u64);
}

fn read_pair(table: own array<u8, 8>, low: own u64, high: own u64) -> result: own u8 traps {
  let low_bounded = 0_u64;
  loop @select_low {
    if ieq(low_bounded, low) {
      break @select_low;
    } else if ieq(low_bounded, 7_u64) {
      break @select_low;
    } else {
      set low_bounded = low_bounded +wrap 1_u64;
    }
  }
  let high_bounded = 0_u64;
  loop @select_high {
    if ieq(high_bounded, high) {
      break @select_high;
    } else if ieq(high_bounded, 7_u64) {
      break @select_high;
    } else {
      set high_bounded = high_bounded +wrap 1_u64;
    }
  }
  let low_ok = ilt(low_bounded, 8_u64);
  let high_ok = ilt(high_bounded, 8_u64);
  let both = band(low_ok, high_ok);
  claim pair_in_range: both because "premises: each bounded value starts at zero, advances by one only on its own ordinary-loop backedge, and exits no later than seven\nderivation: induction over the two current-function loops keeps both bounded values between zero and seven inclusive\nconclusion: both is true\nchecker gap: ENT carries no induction fact across either ordinary-loop backedge\nconsumers: the following two table subscripts consume the respective bounds";
  let first = table[low_bounded];
  let second = table[high_bounded];
  return second;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(source, "read_pair");
    // Each conjunct is established, so each subscript discharges.
    assert_eq!(summary.obligations.len(), 2);
    assert!(summary.obligations.iter().all(|o| o.discharged));
    // Claim components are S3 authorities, not ordinary Boolean
    // decompositions; the retained outcome records both canonical facts.
    assert!(summary.boolean_decompositions.is_empty());
    assert_eq!(summary.claims[0].components.len(), 2);
}

/// S1: the ruled-flip guard shape. The `bor` else edge establishes both
/// negative disjuncts (whose projections negate to the two-sided bound) and
/// contributes nothing on the true edge, so the guarded subscript discharges
/// on the edge the guard protects.
#[test]
fn bor_guard_false_edge_establishes_negative_disjuncts_and_discharges() {
    let source = br#"fn get(table: own array<u8, 4>, symbol: own u64) -> result: own u8 pure {
  let below = ilt(symbol, 0_u64);
  let above = ige(symbol, 4_u64);
  let invalid = bor(below, above);
  if invalid {
    return 0_u8;
  } else {
    return table[symbol];
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(source, "get");
    assert_eq!(summary.obligations.len(), 1);
    assert!(summary.obligations[0].discharged);
    // Only the false edge decomposes a disjunction; +bor contributes nothing.
    // The edge establishes the named binding and its expanded tree, so the
    // false edge records both parents and the true edge still records neither.
    assert_eq!(summary.boolean_decompositions.len(), 2);
    let named = binding_rooted_entry(&summary, GoalSign::Negative);
    assert_eq!(named.members.len(), 2);
    assert_binding_members(&summary, named, GoalSign::Negative);
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
    // The ruled flip, now live: this is the shape the 2026-08-09 decision
    // disposed. The protected corpus case records the corresponding current
    // expectation independently.
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
    let source = br#"fn classify(a: own u64, b: own u64) -> result: own u64 pure {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(source, "classify");
    // Exactly the +band then-edge and the -bor false-edge entries exist, each
    // recorded at both its named binding and its expanded tree: no -band, no
    // +bor, and no bxor entry on either sign or either parent shape — the
    // `mixed` edges establish their bindings and still decompose to nothing.
    assert_eq!(summary.boolean_decompositions.len(), 4);
    let conjunction = entry_with_root(&summary, CheckedBooleanOperation::And, GoalSign::Positive);
    assert_eq!(conjunction.members.len(), 2);
    let disjunction = entry_with_root(&summary, CheckedBooleanOperation::Or, GoalSign::Negative);
    assert_eq!(disjunction.members.len(), 2);
    let named_conjunction = binding_rooted_entry(&summary, GoalSign::Positive);
    assert_eq!(named_conjunction.members.len(), 2);
    assert_binding_members(&summary, named_conjunction, GoalSign::Positive);
    let named_disjunction = binding_rooted_entry(&summary, GoalSign::Negative);
    assert_eq!(named_disjunction.members.len(), 2);
    assert_binding_members(&summary, named_disjunction, GoalSign::Negative);
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
    let source = br#"fn guard(table: own array<u8, 8>, index: own u64) -> result: own u8 pure {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(source, "guard");
    // The recursion reaches `-ilt(index, 4)`, which discharges the subscript.
    assert_eq!(summary.obligations.len(), 1);
    assert!(summary.obligations[0].discharged);
    // Both edges record both parents: the named `inside` binding and its
    // expanded `bnot` tree.
    assert_eq!(summary.boolean_decompositions.len(), 4);
    let named_positive = binding_rooted_entry(&summary, GoalSign::Positive);
    assert_eq!(named_positive.members.len(), 3);
    assert_binding_members(&summary, named_positive, GoalSign::Negative);
    let named_negative = binding_rooted_entry(&summary, GoalSign::Negative);
    assert_eq!(named_negative.members.len(), 1);
    assert_binding_members(&summary, named_negative, GoalSign::Positive);
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
/// the body's own subscript discharges. At the caller, two genuine claim
/// components reconstruct the already-interned exact parent requirement.
#[test]
fn band_requirement_establishes_body_conjuncts_and_claim_components_reconstruct_the_parent() {
    let source =
        br#"fn clamp_seven(value: own u64) -> result: own u64 pure {
  return imin(value, 7_u64);
}

fn pick(table: own array<u8, 8>, low: own u64, high: own u64) -> result: own u8 pure contract {
  define low_ok = ilt(low, 8_u64);
  define high_ok = ilt(high, 8_u64);
  define both = band(low_ok, high_ok);
  requires both;
} {
  let first = table[low];
  return first;
}

fn caller(table: own array<u8, 8>, low: own u64, high: own u64) -> result: own u8 traps {
  let low_bounded = 0_u64;
  loop @select_low {
    if ieq(low_bounded, low) {
      break @select_low;
    } else if ieq(low_bounded, 7_u64) {
      break @select_low;
    } else {
      set low_bounded = low_bounded +wrap 1_u64;
    }
  }
  let high_bounded = 0_u64;
  loop @select_high {
    if ieq(high_bounded, high) {
      break @select_high;
    } else if ieq(high_bounded, 7_u64) {
      break @select_high;
    } else {
      set high_bounded = high_bounded +wrap 1_u64;
    }
  }
  let low_ok = ilt(low_bounded, 8_u64);
  let high_ok = ilt(high_bounded, 8_u64);
  let both = band(low_ok, high_ok);
  claim caller_proof: both because "premises: each bounded value starts at zero, advances by one only on its own ordinary-loop backedge, and exits no later than seven\nderivation: induction over the two current-function loops keeps both bounded values between zero and seven inclusive\nconclusion: both is true\nchecker gap: ENT carries no induction fact across either ordinary-loop backedge\nconsumers: the two following table subscripts consume the components and pick requires the reconstructed exact conjunction";
  let low_probe = table[low_bounded];
  let high_probe = table[high_bounded];
  let value = pick(table: move table, low: low_bounded, high: high_bounded);
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let callee = entailment(source, "pick");
    assert_eq!(callee.obligations.len(), 1);
    assert!(callee.obligations[0].discharged);
    let entry = entry_with_root(&callee, CheckedBooleanOperation::And, GoalSign::Positive);
    assert_eq!(entry.members.len(), 2);
    assert_comparison_member(&callee, entry.members[0], GoalSign::Positive, 8, -1);
    assert_comparison_member(&callee, entry.members[1], GoalSign::Positive, 8, -1);
    // The caller discharges the exact conjunction reconstructed from the two
    // canonical claim components.
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

/// The band/derived-index discharge asymmetry, pinned. A conjunct that bounds
/// a let-bound derived value keeps the relation its own comparison binding
/// recorded, so the conjoined claim discharges exactly what the equivalent
/// pair of single-bound claims discharges. Before the members were read
/// through their bindings, the expanded conjunct read `at +wrap 1 < len(..)`,
/// whose arithmetic root has no term form, and the second subscript's
/// obligation survived while the first discharged.
#[test]
fn band_conjunct_over_a_derived_binding_discharges_like_the_single_bound_pair() {
    let conjoined = br#"fn read_pair['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i), traps {
  let next = at +wrap 1_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  let both = band(at_ok, next_ok);
  claim pair_in_range: both because "pair in range";
  let first = deref(input)[at];
  let second = deref(input)[next];
  return first +wrap second;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let separate = br#"fn read_pair['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i), traps {
  let next = at +wrap 1_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  claim at_in_range: at_ok because "first in range";
  claim next_in_range: next_ok because "second in range";
  let first = deref(input)[at];
  let second = deref(input)[next];
  return first +wrap second;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for source in [conjoined.as_slice(), separate.as_slice()] {
        let summary = entailment(source, "read_pair");
        assert_eq!(summary.obligations.len(), 2);
        assert!(
            summary.obligations.iter().all(|o| o.discharged),
            "both subscripts must discharge: {:?}",
            summary.obligations
        );
    }
    // The conjunct that bounds the derived value is carried as its own
    // comparison binding, not as an expanded arithmetic comparison.
    let summary = entailment(conjoined, "read_pair");
    let named = binding_rooted_entry(&summary, GoalSign::Positive);
    assert_eq!(named.members.len(), 2);
    assert_binding_members(&summary, named, GoalSign::Positive);
}

/// The same widening under a branch guard rather than a claim, which is the
/// shape that leaves the function claim-free: `if band(..)` admits both
/// subscripts on the true edge and neither on the false edge, because `-band`
/// carries only disjunctive content.
#[test]
fn band_guard_over_a_derived_binding_admits_the_true_edge_only() {
    let source =
        br#"fn window['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i) {
  let next = at +wrap 1_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  let both = band(at_ok, next_ok);
  if both {
    let first = deref(input)[at];
    let second = deref(input)[next];
    return first +wrap second;
  }
  return 0_u8;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(source, "window");
    assert_eq!(summary.obligations.len(), 2);
    assert!(summary.obligations.iter().all(|o| o.discharged));
    // No claim site, so the guarded form is the one that keeps a caller's
    // sibling pair eligible.
    assert!(summary.claims.is_empty());
    let else_edge =
        br#"fn window['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i) {
  let next = at +wrap 1_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  let both = band(at_ok, next_ok);
  if both {
    return 0_u8;
  } else {
    return deref(input)[next];
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(else_edge, "window");
    assert_eq!(summary.obligations.len(), 1);
    assert!(
        !summary.obligations[0].discharged,
        "a false `band` bounds neither conjunct: {:?}",
        summary.obligations[0]
    );
}

/// The negative twin the widening must keep failing: reading the conjuncts
/// through their bindings proves exactly the two bounds the band names and no
/// third one, so a subscript by an index the band never bounded still carries
/// its obligation, and a disjunction still bounds neither side.
#[test]
fn band_over_derived_bindings_proves_no_unnamed_bound() {
    let uncovered = br#"fn read_three['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i), traps {
  let next = at +wrap 1_u64;
  let far = at +wrap 2_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  let both = band(at_ok, next_ok);
  claim pair_in_range: both because "pair in range";
  let first = deref(input)[at];
  let third = deref(input)[far];
  return first +wrap third;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(uncovered, "read_three");
    assert_eq!(summary.obligations.len(), 2);
    assert_eq!(
        summary.obligations.iter().filter(|o| o.discharged).count(),
        1,
        "only the named bound discharges: {:?}",
        summary.obligations
    );
    let disjoined = br#"fn read_pair['i](input: &'i buffer<u8>, at: own u64) -> result: own u8 reads('i), traps {
  let next = at +wrap 1_u64;
  let room = len(deref(input));
  let at_ok = ilt(at, room);
  let next_ok = ilt(next, room);
  let either = bor(at_ok, next_ok);
  claim one_in_range: either because "one in range";
  let second = deref(input)[next];
  return second;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let summary = entailment(disjoined, "read_pair");
    assert_eq!(summary.obligations.len(), 1);
    assert!(
        !summary.obligations[0].discharged,
        "a disjunction bounds neither side: {:?}",
        summary.obligations[0]
    );
}
