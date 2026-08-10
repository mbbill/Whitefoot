//! Unit tests for the L0 entailment engine, one family per [ENT] rule,
//! including the adversarial stale-fact and fresh-binding shapes the spec
//! text was reviewed against.
//!
//! Derivation tests observe the engine through the retained obligation and
//! claim dispositions via the test-only dark checker, which skips the
//! [OP-4]/[CLM-2] rejection so a function's complete summary stays
//! observable. The rejection behavior itself is tested at the end of this
//! file through the ordinary acceptance path.

use crate::{CallRequirementDisposition, SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::entailment::{
    CallGoalDisposition, CallGoalEvidence, CallGoalOutcome, ClaimDisposition, ClaimOutcome,
    ObligationOutcome,
};
use super::{assert_rule, with_semantics, with_semantics_dark};

fn obligations(source: &[u8], function: &str) -> Vec<ObligationOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.obligations.clone()
    })
}

fn claims(source: &[u8], function: &str) -> Vec<ClaimOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.claims.clone()
    })
}

fn call_goals(source: &[u8], function: &str) -> Vec<CallGoalOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("call-goal test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.call_goals.clone()
    })
}

fn discharge_flags(source: &[u8], function: &str) -> Vec<bool> {
    obligations(source, function)
        .iter()
        .map(|outcome| outcome.discharged)
        .collect()
}

// ---------------------------------------------------------------------
// [ENT-3] S1 branch facts and their exact negation
// ---------------------------------------------------------------------

#[test]
fn a_dominating_branch_discharges_the_guarded_index_and_not_the_other_arm() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    return values[i];
  } else {
    return values[i];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes[0].discharged, "True arm carries i < 4 = len");
    assert!(!outcomes[1].discharged, "False arm carries only i >= 4");
    assert_eq!(outcomes[1].residual.as_deref(), Some("i < len(values)"));
}

#[test]
fn a_constant_offset_discharges_against_a_const_array_and_a_too_large_one_reports() {
    let source = br#"const count: u64 = 4_u64;

const table: array<u8, count> =[10_u8, 20_u8, 30_u8, 40_u8];

fn read() -> own u8 pure {
  let inside = table[2_u64];
  let outside = table[9_u64];
  return inside;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "2 < 4 by the implicit length equality"
    );
    assert!(!outcomes[1].discharged, "9 < 4 is not derivable");
    assert_eq!(outcomes[1].residual.as_deref(), Some("9_u64 < len(table)"));
}

// ---------------------------------------------------------------------
// [ENT-3] comparison origin (b) and its path validity
// ---------------------------------------------------------------------

#[test]
fn a_bool_binding_carries_its_comparison_to_the_match_when_no_kill_intervenes() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let flag = ilt(i, 4_u64);
  if flag {
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

#[test]
fn a_set_between_initializer_and_use_invalidates_the_comparison_origin() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let flag = ilt(i, 4_u64);
  set i = i +wrap 1_u64;
  if flag {
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the origin's operand fact was killed by the assignment"
    );
}

// ---------------------------------------------------------------------
// [ENT-4] closure: transitivity, strengthening, contradiction, and the
// flow/closure boundary
// ---------------------------------------------------------------------

#[test]
fn transitivity_composes_branch_facts_through_a_middle_term() {
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 pure {
  if ile(i, p.count) {
    if ilt(p.count, 4_u64) {
      return values[i];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "i <= p.count and p.count < 4 compose to i < len(values)"
    );
}

#[test]
fn disequality_strengthens_a_weak_bound_to_a_strict_one() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ile(i, 4_u64) {
    if ieq(i, 4_u64) {
      return 0_i32;
    } else {
      return values[i];
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "i <= 4 with i != 4 strengthens to i <= 3 < len(values)"
    );
}

#[test]
fn a_contradictory_state_discharges_every_obligation() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 0_u64) {
    return values[9_u64];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].discharged, "i < 0 for u64 contradicts i >= 0");
    assert!(outcomes[0].contradictory);
}

#[test]
fn a_kill_between_establishment_and_query_breaks_an_underived_chain() {
    // The flow carries established facts and closure happens at the query
    // [ENT-3, ENT-4]: consuming the middle term's root before the query
    // leaves the endpoints unrelated, because i - Z was never established
    // as its own fact on this straight-line path.
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn eat(p: own Pair) -> own unit pure {
  return unit;
}

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 pure {
  if ile(i, p.count) {
    if ilt(p.count, 4_u64) {
      eat(p: move p);
      return values[i];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "consuming p kills both links before any join materializes i <= 3"
    );
}

// ---------------------------------------------------------------------
// [ENT-5] kills: assignment overlap and effect-row write projection
// ---------------------------------------------------------------------

#[test]
fn an_assignment_to_a_sibling_field_keeps_facts_and_to_the_fact_field_kills_them() {
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn read(values: own array<i32, count>, p: own Pair) -> own i32 pure {
  if ilt(p.count, 4_u64) {
    set p.other = 9_u64;
    let kept = values[p.count];
    set p.count = 9_u64;
    let lost = values[p.count];
    return kept;
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false],
        "OWN-7 overlap: p.other is disjoint from p.count; p.count is not"
    );
}

#[test]
fn a_callee_writing_through_a_unique_borrow_kills_facts_on_that_place() {
    let source = br#"const count: u64 = 4_u64;

fn bump['w](p: &uniq 'w u64) -> own unit writes('w) {
  set deref(p) = 9_u64;
  return unit;
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    region 'w {
      bump<'w>(p: &uniq 'w i);
    }
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the callee's writes row projects onto the unique actual's place"
    );
}

#[test]
fn a_callee_with_no_writes_row_kills_nothing() {
    let source = br#"const count: u64 = 4_u64;

fn peek['r](p: &'r u64) -> own u64 reads('r) {
  return deref(p);
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    region 'r {
      let seen = peek<'r>(p: &'r i);
    }
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "a call whose row carries no writes kills nothing"
    );
}

// ---------------------------------------------------------------------
// [ENT-5] joins and scope-exit ordering
// ---------------------------------------------------------------------

#[test]
fn a_join_keeps_the_weakest_bound_held_on_every_continuing_arm() {
    let source = br#"const two: u64 = 2_u64;

const count: u64 = 4_u64;

fn read(wide: own array<i32, count>, narrow: own array<i32, two>, i: own u64) -> own i32 pure {
  if ilt(i, 2_u64) {
  } else if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  let in_wide = wide[i];
  let in_narrow = narrow[i];
  return in_wide;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false],
        "the join keeps i <= 3 (weakest across arms), not the True arm's i <= 1"
    );
}

#[test]
fn an_arm_that_leaves_by_return_contributes_nothing_to_the_join() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "only the True arm reaches the continuation, so its fact survives"
    );
}

#[test]
fn a_fresh_binding_reusing_an_expired_spelling_inherits_no_stale_fact() {
    // The stale-fact/fresh-binding attack shape: each arm declares its own
    // `j`; the second is a distinct declaration event [ENT-2] and no fact
    // established for the first may attach to it.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, pick: own Bool) -> own i32 pure {
  if pick {
    let j = 0_u64;
    if ilt(j, 4_u64) {
      return values[j];
    } else {
      return 0_i32;
    }
  } else {
    let j = 9_u64;
    return values[j];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes[0].discharged, "the first j is branch-guarded");
    assert!(
        !outcomes[1].discharged,
        "the second j is a fresh declaration event with no facts"
    );
}

#[test]
fn a_fact_about_an_outer_binding_survives_a_region_exit() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  region 'a {
    if ilt(i, 4_u64) {
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "scope-exit kills reach only bindings whose scope ends at the edge"
    );
}

// ---------------------------------------------------------------------
// [ENT-5] break, give, and propagate edges
// ---------------------------------------------------------------------

#[test]
fn a_break_edge_carries_surviving_facts_to_the_loop_continuation() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  loop @l {
    if ilt(i, 4_u64) {
      break @l;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the break edge exits only loop-local scopes; the fact on i survives"
    );
}

#[test]
fn a_kill_before_the_break_edge_leaves_the_continuation_unproved() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  loop @l {
    if ilt(i, 4_u64) {
      set i = i +wrap 1_u64;
      break @l;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the assignment killed the branch fact before the break edge"
    );
}

#[test]
fn give_edges_join_at_the_value_match_continuation_with_arm_facts_dead() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let picked = if ilt(i, 4_u64) {
    give values[i];
  } else {
    give 0_i32;
  }
  let after = values[i];
  return picked;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "inside the True arm the S1 fact discharges the given index"
    );
    assert!(
        !outcomes[1].discharged,
        "after the give join neither arm's exclusive fact survives"
    );
}

#[test]
fn a_propagate_continuation_keeps_prior_facts_when_the_call_writes_nothing() {
    let source = br#"const count: u64 = 4_u64;

enum Fail {
  Bad();
}

fn source(flag: own Bool) -> own Result<u64, Fail> pure {
  if flag {
    return Ok<u64, Fail>(value: 1_u64);
  } else {
    let bad = Bad();
    return Err<u64, Fail>(error: bad);
  }
}

fn read(values: own array<i32, count>, i: own u64, flag: own Bool) -> own Result<i32, Fail> pure {
  if ilt(i, 4_u64) {
    let v = propagate source(flag: flag);
    let a = values[i];
    return Ok<i32, Fail>(value: a);
  } else {
    return Ok<i32, Fail>(value: 0_i32);
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the Err edge leaves the function; the normal continuation keeps i < 4"
    );
}

// ---------------------------------------------------------------------
// [ENT-5] the no-induction loop rule
// ---------------------------------------------------------------------

#[test]
fn a_loop_body_kill_removes_the_fact_from_every_iteration_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  let before = values[i];
  loop @l {
    let inside = values[i];
    set i = i +wrap 1_u64;
    if ilt(i, 4_u64) {
    } else {
      break @l;
    }
  }
  return before;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false],
        "the head state subtracts every fact the body's assignment may kill"
    );
}

#[test]
fn a_kill_free_loop_body_keeps_the_entry_fact_at_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let inside = values[i];
    break @l;
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, true],
        "no kill event in the body touches i, so the fact holds at the head"
    );
}

#[test]
fn d1h_and_d1i_distinguish_a_return_inside_the_loop_from_one_after_it() {
    let source = br#"const count: u64 = 4_u64;

fn return_inside(values: own array<i32, count>, i: own u64, stop: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    if stop {
      return at_head;
    }
    break @l;
  }
  return 0_i32;
}

fn return_after(values: own array<i32, count>, i: own u64, stop: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    break @l;
  }
  if stop {
    return 1_i32;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "return_inside"),
        vec![true],
        "D1h: a return edge reaches no later head, so it cannot erase the entry fact"
    );
    assert_eq!(
        discharge_flags(source, "return_after"),
        vec![true],
        "D1i control: moving the same return after the loop stays discharged"
    );
}

#[test]
fn a_kill_followed_only_by_the_current_loop_break_does_not_poison_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    set i = i +wrap 1_u64;
    break @l;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the set's only successor leaves this loop, so no later head observes it"
    );
}

#[test]
fn a_kill_followed_only_by_an_enclosing_break_does_not_poison_the_inner_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave_outer: own Bool, leave_inner: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    loop @inner {
      let at_head = values[i];
      if leave_outer {
        set i = i +wrap 1_u64;
        break @outer;
      }
      if leave_inner {
        break @inner;
      }
    }
    break @outer;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the assignment reaches only the enclosing loop's continuation, not the inner head"
    );
}

#[test]
fn a_propagate_error_edge_does_not_poison_the_loop_head() {
    let source = br#"const count: u64 = 4_u64;

enum Fail {
  Bad();
}

fn source(fail: own Bool) -> own Result<u64, Fail> pure {
  if fail {
    let bad = Bad();
    return Err<u64, Fail>(error: bad);
  }
  return Ok<u64, Fail>(value: 1_u64);
}

fn read(values: own array<i32, count>, i: own u64, fail: own Bool, leave: own Bool) -> own Result<i32, Fail> pure {
  if ilt(i, 4_u64) {
  } else {
    return Ok<i32, Fail>(value: 0_i32);
  }
  loop @l {
    let value = propagate source(fail: fail);
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return Ok<i32, Fail>(value: 0_i32);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "only propagate's Ok edge can return to the head; its Err edge leaves the function"
    );
}

#[test]
fn an_else_free_continuing_kill_still_poisons_the_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, mutate: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    if mutate {
      set i = i +wrap 1_u64;
    }
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the mutating arm and the else-free false edge both remain inside the body"
    );
}

#[test]
fn a_give_to_an_initializer_inside_the_loop_carries_its_kill_to_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, mutate: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let picked = if mutate {
      set i = i +wrap 1_u64;
      give 1_i32;
    } else {
      give 0_i32;
    }
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the give reaches an initializer continuation inside the body and then the backedge"
    );
}

#[test]
fn a_mixed_branch_ignores_the_return_only_kill_but_keeps_the_continuing_one() {
    let source = br#"const count: u64 = 4_u64;

fn read(left: own array<i32, count>, right: own array<i32, count>, i: own u64, j: own u64, stop: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  if ilt(j, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    if stop {
      set i = i +wrap 1_u64;
      return 0_i32;
    } else {
      set j = j +wrap 1_u64;
    }
    let left_value = left[i];
    let right_value = right[j];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false],
        "the returning arm's i kill is non-continuing, while the other arm's j kill reaches the backedge"
    );
}

#[test]
fn a_nested_loop_own_break_carries_kills_to_the_outer_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave_outer: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    let at_head = values[i];
    loop @inner {
      set i = i +wrap 1_u64;
      break @inner;
    }
    if leave_outer {
      break @outer;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the inner break reaches a continuation inside the outer body and then its backedge"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S11 counted-range structural facts
// ---------------------------------------------------------------------

#[test]
fn a_counted_range_discharges_its_binder_and_safe_predecessor_indices() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let total = 0_i32;
  for @items i in 1_u64..4_u64 {
    let previous = i -wrap 1_u64;
    let current_value = values[i];
    let previous_value = values[previous];
    set total = total +wrap current_value;
    set total = total +wrap previous_value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, true],
        "S11 proves i < upper and lower <= i; S7 derives the safe predecessor"
    );
}

#[test]
fn a_counted_range_does_not_prove_the_next_index_or_an_unrelated_carried_index() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, j: own u64) -> own i32 pure {
  let total = 0_i32;
  for @items i in 0_u64..4_u64 {
    let next = i +wrap 1_u64;
    let current_value = values[i];
    let next_value = values[next];
    let unrelated = values[j];
    set total = total +wrap current_value;
    set total = total +wrap next_value;
    set total = total +wrap unrelated;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false, false],
        "S11 is not general induction: next may equal upper and j is unrelated"
    );
}

#[test]
fn a_counted_upper_needs_an_independent_relation_to_the_storage_length() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, upper: own u64) -> own i32 pure {
  let total = 0_i32;
  for @items i in 0_u64..upper {
    let value = values[i];
    set total = total +wrap value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "i < captured upper does not imply upper <= len(values)"
    );
}

#[test]
fn a_counted_preheader_closes_snapshot_consequences_before_body_kills() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let upper = 4_u64;
  let total = 0_i32;
  for @items i in 0_u64..upper {
    set upper = 0_u64;
    let value = values[i];
    set total = total +wrap value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the capture equals four before the mutable source changes; that closed snapshot consequence remains true"
    );
}

#[test]
fn a_break_free_zero_trip_counted_continuation_is_reachable_not_contradictory() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  for @empty i in 4_u64..4_u64 {
    let ignored = i;
  }
  return values[9_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the real false-header edge prevents the ordinary break-free-loop contradictory join"
    );
}

#[test]
fn a_counted_body_fact_does_not_escape_through_the_zero_trip_edge() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  for @maybe n in 0_u64..1_u64 {
    if ilt(i, 4_u64) {
      let ignored = n;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the structural false edge contributes the pre-body state to the continuation join"
    );
}

#[test]
fn a_nested_counted_loop_kill_can_reach_an_outer_ordinary_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    for @inner n in 0_u64..1_u64 {
      set i = i +wrap 1_u64;
      let ignored = n;
    }
    let at_head = values[i];
    if leave {
      break @outer;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "nested counted exhaustion reaches the outer continuation and then its backedge"
    );
}

// ---------------------------------------------------------------------
// [ENT-6] obligations and residual rendering
// ---------------------------------------------------------------------

#[test]
fn a_struct_field_base_renders_its_canonical_place_in_the_residual() {
    let source = br#"const count: u64 = 4_u64;

struct Holder {
  data: array<u8, count>;
}

fn read(h: own Holder, i: own u64) -> own u8 pure {
  return h.data[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(!outcomes[0].discharged);
    assert_eq!(outcomes[0].residual.as_deref(), Some("i < len(h.data)"));
}

#[test]
fn a_nested_index_offset_is_no_term_and_renders_its_canonical_bytes() {
    let source = br#"const count: u64 = 4_u64;

fn read(lens: own array<u8, count>, order: own array<u64, count>, j: own u64) -> own u8 pure {
  if ilt(j, 4_u64) {
    return lens[order[j]];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2, "inner offset first, then the outer site");
    assert!(
        outcomes[0].discharged,
        "the inner index over order discharges"
    );
    assert!(
        !outcomes[1].discharged,
        "an index-bearing offset is no term [ENT-2], so the outer obligation is underivable"
    );
    assert_eq!(
        outcomes[1].residual.as_deref(),
        Some("order[j] < len(lens)")
    );
}

#[test]
fn a_buffer_or_slice_offset_renders_the_same_subscript_spelling() {
    let source = br#"const count: u64 = 4_u64;

fn from_buffer(values: own array<u8, count>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u64);
  return values[b[0_u64]];
}

fn from_slice['r](values: own array<u8, count>, order: own slice<'r, u64>) -> own u8 reads('r) {
  return values[order[0_u64]];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let buffer = obligations(source, "from_buffer");
    assert_eq!(buffer.len(), 2, "inner offset first, then the outer site");
    assert!(
        buffer[0].discharged,
        "the S6 allocation equality proves the inner offset"
    );
    assert_eq!(
        buffer[1].residual.as_deref(),
        Some("b[0_u64] < len(values)")
    );

    let slice = obligations(source, "from_slice");
    assert_eq!(slice.len(), 2, "inner offset first, then the outer site");
    assert_eq!(slice[0].residual.as_deref(), Some("0_u64 < len(order)"));
    assert_eq!(
        slice[1].residual.as_deref(),
        Some("order[0_u64] < len(values)")
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S6 length facts
// ---------------------------------------------------------------------

#[test]
fn an_allocation_length_equality_proves_a_constant_offset_and_a_runtime_length_does_not() {
    let source = br#"fn sized() -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  return b[3_u64];
}

fn runtime(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  return b[3_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "sized"),
        vec![true],
        "len(b) = 4 makes 3 < len(b) derivable [ENT-3] S6"
    );
    let runtime = obligations(source, "runtime");
    assert!(
        !runtime[0].discharged,
        "len(b) = n bounds nothing without a fact about n"
    );
    assert_eq!(runtime[0].residual.as_deref(), Some("3_u64 < len(b)"));
}

#[test]
fn an_allocation_length_binding_carries_the_length_into_a_branch() {
    // `let m = len<T>(P)` establishes m = len(P), so a branch over m is a
    // branch over the length itself [ENT-3] S6.
    let source = br#"fn read(n: own u64, i: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  let m = len(b);
  if ilt(i, m) {
    return b[i];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

#[test]
fn a_slice_of_carries_its_source_length() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>) -> own u8 pure {
  region 'view {
    let window = slice_of(&'view values);
    return window[3_u64];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "len(window) = len(values) = 4 [ENT-3] S6"
    );
}

#[test]
fn an_element_write_keeps_the_allocation_equality_that_a_write_to_its_length_kills() {
    // [ENT-5]: a buffer's length is fixed at allocation, so an element write
    // never kills its length fact; a write to the term the equality is held
    // against does. A buffer place is affine [STOR-1], so writing the root
    // binding itself is not a source shape the engine can be shown.
    let source = br#"fn kept(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  if ilt(3_u64, n) {
    set b[0_u64] = 1_u8;
    return b[3_u64];
  } else {
    return 0_u8;
  }
}

fn killed(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  if ilt(3_u64, n) {
    set n = 0_u64;
    return b[3_u64];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let kept = obligations(source, "kept");
    assert!(
        kept.last().is_some_and(|outcome| outcome.discharged),
        "an element write leaves len(b) = n alive"
    );
    let killed = obligations(source, "killed");
    assert!(
        !killed.last().is_some_and(|outcome| outcome.discharged),
        "writing n kills the allocation equality held against it"
    );
}

#[test]
fn consuming_the_buffer_kills_a_length_binding_that_survives_otherwise() {
    // The support of len(b) is b's root binding, so a consuming use kills
    // every fact holding it, including the equality a length binding carries
    // away from it [ENT-5](c).
    let source = br#"const wide: u64 = 8_u64;

fn eat(b: own buffer<u8>) -> own unit pure {
  return unit;
}

fn kept(other: own array<u8, wide>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  let m = len(b);
  let sample = other[m];
  eat(b: move b);
  return sample;
}

fn killed(other: own array<u8, wide>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  let m = len(b);
  eat(b: move b);
  let sample = other[m];
  return sample;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "kept"),
        vec![true],
        "m = len(b) = 4 < 8 while b is live"
    );
    assert_eq!(
        discharge_flags(source, "killed"),
        vec![false],
        "the consuming use kills m's tie to the allocation length"
    );
}

#[test]
fn set_targets_carry_the_same_obligation_in_target_position() {
    let source = br#"const count: u64 = 4_u64;

fn write(values: own array<u16, count>, i: own u64) -> own u16 pure {
  if ilt(i, 4_u64) {
    set values[i] = 9_u16;
    return 1_u16;
  } else {
    set values[i] = 9_u16;
    return 0_u16;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "write");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "target-position discharge is identical"
    );
    assert!(!outcomes[1].discharged);
    assert_eq!(outcomes[1].residual.as_deref(), Some("i < len(values)"));
}

// ---------------------------------------------------------------------
// [ENT-3] S2 check facts
// ---------------------------------------------------------------------

#[test]
fn a_passed_check_establishes_its_comparison_on_the_continuation() {
    let source = br#"const count: u64 = 4_u64;

fn direct(values: own array<i32, count>, i: own u64) -> own i32 traps {
  check ilt(i, 4_u64) else trap "i must be in range";
  return values[i];
}

fn through_origin(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let ok = ilt(i, 4_u64);
  check ok else trap "i must be in range";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "direct"), vec![true]);
    assert_eq!(
        discharge_flags(source, "through_origin"),
        vec![true],
        "the check reads comparison-origin shape (b) exactly as a match does"
    );
}

#[test]
fn a_check_without_comparison_origin_establishes_no_l0_fact() {
    // `band` has no comparison projection [ENT-3], so its passed check
    // establishes the exact whole goal but no child comparison relation.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let low = ilt(i, 4_u64);
  let high = ige(i, 0_u64);
  check band(low, high) else trap "i must be in range";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![false]);
}

// ---------------------------------------------------------------------
// [ENT-3] S5 copy and conversion equalities
// ---------------------------------------------------------------------

#[test]
fn a_literal_a_copy_and_a_total_conversion_carry_the_value_forward() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let k = 2_u64;
  let j = k;
  let narrow = 3_u16;
  let widened = cvt<u16, u64>(narrow);
  let first = values[j];
  let second = values[widened];
  return first +wrap second;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, true],
        "j = k = 2 and widened = narrow = 3, both below len(values)"
    );
}

#[test]
fn a_narrowing_conversion_carries_no_equality_into_its_ok_arm() {
    // [OP-6] narrowing is not a total pair, so [ENT-3] S5 does not apply and
    // the `Ok` binder inherits only its own type range.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, n: own u64) -> own i32 pure {
  if ilt(n, 4_u64) {
    match cvt<u64, u8>(n) {
      Ok(value: small) => {
        let widened = cvt<u8, u64>(small);
        return values[widened];
      }
      Err(error: narrowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "small keeps no tie to n, so widened is bounded only by u8's range"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S7 constant-offset arithmetic
// ---------------------------------------------------------------------

#[test]
fn a_trapping_offset_establishes_its_equality_unconditionally() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 3_u64) {
    let next = i + 1_u64;
    return values[next];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the executed contract check is the proof [ENT-3] S7"
    );
}

#[test]
fn a_wrapping_offset_establishes_only_where_the_range_is_already_proved() {
    // The wrap has no runtime check, so the equality holds only where the
    // closed state already proves the unwrapped result stays in range.
    let source = br#"const count: u64 = 4_u64;

fn guarded(values: own array<i32, count>, p: own u64) -> own i32 pure {
  if ilt(p, 4_u64) {
    if ige(p, 1_u64) {
      let s = p -wrap 1_u64;
      return values[s];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn unguarded(values: own array<i32, count>, p: own u64) -> own i32 pure {
  if ilt(p, 4_u64) {
    let s = p -wrap 1_u64;
    return values[s];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "guarded"),
        vec![true],
        "p >= 1 proves p - 1 does not underflow, so s = p - 1 <= 2"
    );
    assert_eq!(
        discharge_flags(source, "unguarded"),
        vec![false],
        "p may be 0, where the wrap reaches u64::MAX"
    );
}

#[test]
fn a_checked_offset_establishes_in_the_ok_arm_only_and_dies_with_its_base() {
    let source = br#"const count: u64 = 4_u64;

fn direct(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    match i +checked 1_u64 {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn through_binding(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    let outcome = i +checked 1_u64;
    match outcome {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn killed(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    let outcome = i +checked 1_u64;
    set i = 9_u64;
    match outcome {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "direct"), vec![true]);
    assert_eq!(
        discharge_flags(source, "through_binding"),
        vec![true],
        "a bare IDENT naming the outcome carries the same fact"
    );
    assert_eq!(
        discharge_flags(source, "killed"),
        vec![false],
        "writing the base between the initializer and the match ends the origin"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S9 const-array element ranges
// ---------------------------------------------------------------------

#[test]
fn a_const_array_element_carries_its_declared_value_range() {
    let source = br#"const count: u64 = 4_u64;

const inside: array<u64, count> =[0_u64, 1_u64, 3_u64, 2_u64];

const outside: array<u64, count> =[0_u64, 1_u64, 4_u64, 2_u64];

fn low(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let bound = inside[i];
  return values[bound];
}

fn high(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let bound = outside[i];
  return values[bound];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let low = obligations(source, "low");
    assert_eq!(low.len(), 2, "the element read carries its own obligation");
    assert!(
        !low[0].discharged,
        "the index into the const table is judged separately and unaffected"
    );
    assert!(
        low[1].discharged,
        "every declared element is at most 3 < len(values)"
    );
    let high = obligations(source, "high");
    assert!(
        !high[1].discharged,
        "a declared element of 4 reaches len(values)"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S4 requires facts
// ---------------------------------------------------------------------

#[test]
fn a_requires_check_establishes_its_substituted_relation_at_body_entry() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let ok = ilt(i, 4_u64);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the requirement relation is available at body entry"
    );
}

#[test]
fn a_requires_chain_substitutes_repeatedly_and_reads_a_length_call() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let n = len(values);
  let ok = ilt(i, n);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "ok substitutes to the comparison, then n to the length term itself"
    );
}

#[test]
fn every_occurrence_of_a_requires_local_substitutes() {
    // Both operands name the same clause local. Expanding only one would
    // leave a non-term operand and establish nothing; expanding both derives
    // len(values) < len(values), a contradictory entry state [ENT-4].
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure requires {
  let n = len(values);
  let ok = ilt(n, n);
  check ok else trap "unsatisfiable by construction";
} {
  return values[9_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(
        outcomes[0].contradictory,
        "both occurrences expanded to the same length term"
    );
    assert!(outcomes[0].discharged);
}

#[test]
fn a_noncomparison_s4_goal_establishes_no_child_l0_fact() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let low = ilt(i, 4_u64);
  let high = ige(i, 0_u64);
  let ok = band(low, high);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "S4 establishes the atomic band, not either comparison child"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S10 boundary count facts
// ---------------------------------------------------------------------

#[test]
fn a_transfer_count_is_bounded_by_its_bounding_actual_and_not_beyond_it() {
    // The bound is `w <= k` against the actual bound to the operation's own
    // bounding parameter, so a count equal to the length proves nothing.
    let source = br#"const count: u64 = 4_u64;

fn under['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, table: own array<u8, count>) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: 3_u64) {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

fn exact['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, table: own array<u8, count>) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: 4_u64) {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch = buffer_new(1_u64, 0_u8);
  let table = array_new<u8, count>(0_u8);
  region 'publication {
    under<'publication, 'publication>(output: &uniq 'publication out, source: &'publication batch, table: move table);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "under"),
        vec![true],
        "written <= 3 < len(table)"
    );
    assert_eq!(
        discharge_flags(source, "exact"),
        vec![false],
        "written <= 4 admits written = len(table)"
    );
}

#[test]
fn a_transfer_count_bound_enters_the_observing_arm_only() {
    // `Ok(value: w)` observes the count bound; the error arm's own u64 payload
    // is an unrelated required size and gains nothing [ENT-3] S10.
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let table = array_new<u8, count>(0_u8);
  let sink = buffer_new(8_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd sink, offset: 0_u64, capacity: 3_u64) {
              Ok(value: copied) => {
                let good = table[copied];
              }
              Err(error: problem) => {
                match problem {
                  CopyTooSmall(required: needed) => {
                    let bad = table[needed];
                  }
                }
              }
            }
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true, false],
        "only the success arm's binder carries the capacity bound"
    );
}

#[test]
fn a_host_copy_utf8_success_count_is_bounded_by_capacity() {
    // The UTF-8 copy producer carries the same S10 success-count bound as the
    // byte-preserving copy producer: copied <= 3 < len(table).
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let table = array_new<u8, count>(0_u8);
  let sink = buffer_new(8_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            match host_copy_utf8<'v, 'd>(value: &'v text, destination: &uniq 'd sink, offset: 0_u64, capacity: 3_u64) {
              Ok(value: copied) => {
                let good = table[copied];
              }
              Err(error: problem) => {
              }
            }
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true],
        "host_copy_utf8's Ok payload is at most the capacity actual"
    );
}

#[test]
fn a_let_bound_transfer_outcome_carries_the_same_count_bound() {
    // The bare IDENT form of [ENT-3] S10, under the same no-kill, no-`set`
    // path discipline as S7's checked-arithmetic origin.
    let source = br#"const count: u64 = 4_u64;

fn deferred['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: 3_u64);
    match outcome {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

fn killed['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: limit);
    set limit = 9_u64;
    match outcome {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch = buffer_new(1_u64, 0_u8);
  let table = array_new<u8, count>(0_u8);
  region 'publication {
    deferred<'publication>(output: move out, source: &'publication batch, table: move table, limit: 3_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(discharge_flags(source, "deferred"), vec![true]);
    assert_eq!(
        discharge_flags(source, "killed"),
        vec![false],
        "writing the bounding actual before the match ends the origin"
    );
}

#[test]
fn a_read_once_count_is_observed_on_its_own_outcome_variant() {
    // `read_once` reports through `ReadBytes(count: w)` rather than a
    // `Result`, so the observing arm is named per operation [ENT-3] S10.
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead) -> own ExitStatus allocates(heap), external, blocks, traps {
  let table = array_new<u8, count>(0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                match open_read<'c, 'p>(root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(64_u64, 0_u8);
                    region 'f {
                      region 'd {
                        match read_once<'f, 'd>(file: &uniq 'f file, destination: &uniq 'd bytes, offset: 0_u64, capacity: 3_u64) {
                          ReadBytes(count: n) => {
                            let sample = table[n];
                          }
                          ReadEnd() => {
                          }
                          ReadFailed(error: problem) => {
                          }
                        }
                      }
                    }
                  }
                  Err(error: unopened) => {
                  }
                }
              }
            }
          }
          Err(error: unresolved) => {
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true],
        "the ReadBytes count is at most the capacity actual"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S3 claim facts
// ---------------------------------------------------------------------

#[test]
fn a_passed_claim_establishes_its_fact_on_the_continuation() {
    let source = br#"fn read(values: own buffer<i32>, i: own u64) -> own i32 traps {
  let n = len(values);
  let inside = ilt(i, n);
  claim in_range: inside because "the caller walks 0..len";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(
        outcomes[0].discharged,
        "S3: the passed claim predicate discharges the following subscript"
    );
    let claim_outcomes = claims(source, "read");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].name, "in_range");
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Retained);
}

#[test]
fn a_claim_without_comparison_origin_is_retained_and_never_judged() {
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  claim held: flag because "constructed";
  return unit;
}
"#;
    let claim_outcomes = claims(source, "main");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Retained);
}

// ---------------------------------------------------------------------
// [CLM-2] redundancy and refutation
// ---------------------------------------------------------------------

#[test]
fn a_derivable_claim_is_redundant_and_reports_the_advisory_without_rejecting() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 4_u64) {
    claim proven: ilt(i, 4_u64) because "already branched";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a redundant claim must not reject, got {outcome:?}");
        };
        assert_eq!(program.data.claim_advisories.len(), 1);
        assert_eq!(program.data.claim_advisories[0].function, "read");
        assert_eq!(program.data.claim_advisories[0].name, "proven");
    });
}

#[test]
fn a_refuted_claim_is_a_clm2_rejection_with_predicate_and_negation() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ige(i, 4_u64) {
    claim in_range: ilt(i, 4_u64) because "refuted by the branch";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("a refuted claim must reject, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::RefutedClaim(detail) = issue.kind() else {
            panic!("expected the refutation payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.name, "in_range");
    });
}

#[test]
fn a_contradictory_state_never_refutes_a_claim() {
    // [ENT-4]: after a loop with no break the continuation state is
    // contradictory; every relation is derivable there, so the claim is
    // redundant, never rejected. The loop must terminate for the checker's
    // reachability rules, so it returns from inside.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 0_u64) {
    claim absurd: ilt(i, 4_u64) because "under a false branch";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    // i < 0 is unsatisfiable for u64: the True arm's state is contradictory,
    // so the claim is redundant there and the subscript discharges.
    let claim_outcomes = claims(source, "read");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Redundant);
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

// ---------------------------------------------------------------------
// [OP-4] discharge-or-reject through the ordinary acceptance path
// ---------------------------------------------------------------------

#[test]
fn an_undischarged_subscript_is_an_op4_rejection_with_the_exact_residual() {
    let source = br#"fn read(values: own buffer<i32>, i: own u64) -> own i32 pure {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Op4,
        SemanticIssueKind::UndischargedBoundsObligation {
            residual: "i < len(values)".to_owned(),
            mechanical_fix: "add a dominating `claim` of the residual or a dominating branch establishing it",
        },
    );
}

#[test]
fn a_discharged_program_accepts_and_retains_its_derivations() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  return values[2_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a discharged subscript must accept, got {outcome:?}");
        };
        let function = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("read must exist");
        assert_eq!(function.entailment.obligations.len(), 1);
        assert!(function.entailment.obligations[0].discharged);
    });
}

#[test]
fn counted_sha256_discharges_all_nine_indices_without_claims() {
    let source = include_bytes!("../../../../tests/programs/sha256_abc.wf");
    let outcomes = obligations(source, "sha256_abc_word_zero");
    assert_eq!(outcomes.len(), 9);
    assert!(outcomes.iter().all(|outcome| outcome.discharged));
    assert!(claims(source, "sha256_abc_word_zero").is_empty());
}

#[test]
fn counted_range_reads_a_dereferenced_projected_endpoint_as_an_s11_term() {
    let source = br#"struct Holder {
  value: box<u64>;
}

fn probe(holder: own Holder) -> own unit traps {
  for @items i in deref(holder.value)..1_u64 {
    claim impossible: ine(i, 0_u64) because "the true edge fixes i to zero";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_kills_a_borrowed_projected_endpoint_after_a_write() {
    let source = br#"struct Limit {
  upper: u64;
}

fn probe(limit: own Limit) -> own unit traps {
  region 'r {
    let holder = &uniq 'r limit;
    for @items i in 0_u64..deref(holder).upper {
      set deref(holder).upper = 0_u64;
      claim safe: ige(i, deref(holder).upper) because "the reread is not the captured endpoint";
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, ClaimDisposition::Retained);
}

#[test]
fn counted_range_preserves_multiple_deref_projections_in_one_endpoint_term() {
    let source = br#"fn probe(holder: own box<box<u64>>) -> own unit traps {
  for @items i in deref(deref(holder))..1_u64 {
    claim impossible: ine(i, 0_u64) because "the true edge fixes i to zero";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_restores_a_borrow_holder_deref_before_nested_box_derefs() {
    let source = br#"fn probe['r](holder: &'r box<box<u64>>) -> own unit reads('r), traps {
  for @items i in deref(deref(deref(holder)))..1_u64 {
    claim impossible: igt(deref(deref(deref(holder))), i) because "the header proves the opposite";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    let ClaimDisposition::Refuted {
        predicate,
        negation,
    } = &outcomes[0].disposition
    else {
        panic!(
            "the opposite of the counted guard must be refuted: {:?}",
            outcomes[0].disposition
        );
    };
    assert!(predicate.contains("deref(deref(deref(holder)))"));
    assert!(negation.contains("deref(deref(deref(holder)))"));
}

#[test]
fn counted_range_does_not_treat_a_read_only_box_deref_as_a_consume() {
    let source = br#"fn probe(holder: own box<u64>) -> own unit traps {
  for @items i in deref(holder)..1_u64 {
    claim impossible: igt(deref(holder), i) because "the captured lower endpoint cannot exceed the binder";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_does_not_duplicate_the_deref_of_a_let_bound_owning_box() {
    let source = br#"fn probe() -> own unit allocates(heap), traps {
  let holder = box_new(0_u64);
  for @items i in deref(holder)..1_u64 {
    claim impossible: igt(deref(holder), i) because "the captured lower endpoint cannot exceed the binder";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    let ClaimDisposition::Refuted { predicate, .. } = &outcomes[0].disposition else {
        panic!("the opposite of the counted lower bound must be refuted");
    };
    assert!(predicate.contains("deref(holder)"));
    assert!(!predicate.contains("deref(deref(holder))"));
}

// ---------------------------------------------------------------------
// [ENT-2..ENT-5, FN-8] exact signed goals and ordinary calls
// ---------------------------------------------------------------------

#[test]
fn whole_goal_sources_discharge_atomically_while_children_do_not() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn from_branch(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn from_check(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
  guarded(value: value);
  return unit;
}

fn from_claim(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  claim established: complete because "complete";
  guarded(value: value);
  return unit;
}

fn from_children(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  if positive {
    if small {
      guarded(value: value);
    } else {
      return unit;
    }
  } else {
    return unit;
  }
  return unit;
}

fn from_false(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    return unit;
  } else {
    guarded(value: value);
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    for function in ["from_branch", "from_check", "from_claim"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
        assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    }
    assert_eq!(
        claims(source, "from_claim")[0].disposition,
        ClaimDisposition::Retained,
        "CLM-2 remains comparison-origin-only even though S3 establishes +G"
    );

    let children = call_goals(source, "from_children");
    assert_eq!(children[0].disposition, CallGoalDisposition::Unproved);
    assert!(children[0].evidence.is_empty());

    let negative = call_goals(source, "from_false");
    assert_eq!(negative[0].disposition, CallGoalDisposition::Refuted);
    assert_eq!(negative[0].evidence, vec![CallGoalEvidence::OpaqueNegative]);
}

#[test]
fn an_exact_comparison_call_retains_every_positive_derivation_ground() {
    let source = br#"fn below(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn exact(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn projected(value: own u64) -> own unit traps {
  let at_most_nine = ile(value, 9_u64);
  if at_most_nine {
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let exact = call_goals(source, "exact");
    assert_eq!(exact[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        exact[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    let projected = call_goals(source, "projected");
    assert_eq!(projected[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        projected[0].evidence,
        vec![CallGoalEvidence::ExactL0Projection]
    );
}

#[test]
fn joined_whole_goals_require_the_same_sign_on_every_reachable_input() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn both(value: own u64, choose: own Bool) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if choose {
    check complete else trap "left";
  } else {
    check complete else trap "right";
  }
  guarded(value: value);
  return unit;
}

fn one(value: own u64, choose: own Bool) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if choose {
    check complete else trap "left";
  } else {
    check True() else trap "other";
  }
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let both = call_goals(source, "both");
    assert_eq!(both[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(both[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    let one = call_goals(source, "one");
    assert_eq!(one[0].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn a_computed_bool_truth_survives_an_origin_write_but_its_expansion_does_not() {
    let source = br#"fn need_true(value: own Bool) -> own unit traps requires {
  check value else trap "true";
} {
  check True() else trap "body";
  return unit;
}

fn below(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn probe(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    set value = 20_u64;
    need_true(value: small);
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "probe");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn a_copy_referent_read_through_an_affine_box_is_an_exact_goal_origin() {
    let source = br#"fn observe['r](value: &'r box<i32>) -> own unit reads('r), traps requires {
  let positive = igt(deref(deref(value)), 0_i32);
  let small = ilt(deref(deref(value)), 10_i32);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  let seen = deref(deref(value));
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit allocates(heap), traps {
  let owner = box_new(5_i32);
  let positive = igt(deref(owner), 0_i32);
  let small = ilt(deref(owner), 10_i32);
  let complete = band(positive, small);
  if complete {
    region 'r {
      observe<'r>(value: &'r owner);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
}

#[test]
fn setting_an_intermediate_bool_binding_stops_later_origin_expansion() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  check igt(value, 0_u64) else trap "positive";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let alias = positive;
  set positive = False();
  if alias {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Unproved);
    assert!(outcomes[0].evidence.is_empty());
}

#[test]
fn resolved_writes_stop_future_expansion_of_the_written_origin_binding() {
    let source = br#"fn need() -> own unit traps requires {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let complete = band(first, second);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn mutate['r](value: &uniq 'r Bool) -> own unit writes('r) {
  set deref(value) = False();
  return unit;
}

fn through_holder() -> own unit traps {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let source = band(first, second);
  region 'r {
    let holder = &uniq 'r source;
    set deref(holder) = False();
  }
  let alias = source;
  if alias {
    need();
  } else {
    return unit;
  }
  return unit;
}

fn through_call() -> own unit traps {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let source = band(first, second);
  region 'r {
    mutate<'r>(value: &uniq 'r source);
  }
  let alias = source;
  if alias {
    need();
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["through_holder", "through_call"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Unproved);
        assert!(outcomes[0].evidence.is_empty());
    }
}

#[test]
fn combined_contradiction_is_absorbing_before_goal_and_l0_support_kills() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn signed(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "first";
  if complete {
    return unit;
  } else {
    set value = 20_u64;
    guarded(value: value);
    claim unreachable: ilt(value, 1_u64) because "combined contradiction";
  }
  return unit;
}

fn l0(value: own u64) -> own unit traps {
  check ilt(value, 5_u64) else trap "low";
  check ige(value, 5_u64) else trap "high";
  set value = 20_u64;
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["signed", "l0"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
        assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::AllDerivable]);
    }
    assert!(!matches!(
        claims(source, "signed")[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn a_discharged_whole_goal_is_accepted_on_the_ordinary_path() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the established whole goal must accept its call: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert_eq!(caller.entailment.call_goals.len(), 1);
        assert_eq!(
            caller.entailment.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
    });
}

#[test]
fn fn8_call_rejection_carries_the_complete_deterministic_payload() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unproved complete goal must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!(
                "FN-8 must carry the ordinary-call payload: {:?}",
                issue.kind()
            );
        };
        assert_eq!(detail.concrete_callee, "guarded");
        assert!(!detail.final_check.components().is_empty());
        assert!(detail.instantiated_goal.contains("Boolean(And)"));
        assert!(detail.instantiated_goal.contains("Integer(U64)"));
        assert_eq!(detail.disposition, CallRequirementDisposition::Unproved);
        assert_eq!(
            detail.mechanical_fix,
            "establish the complete callee requirement with one dominating branch, check, or claim before the call"
        );
        let crate::SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("FN-8 must cite the source call");
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(&source[start..end], b"guarded(value: value)");
    });
}

#[test]
fn actual_obligations_precede_fn8_and_ephemeral_goals_use_the_stronger_fix() {
    let admitted_actual = br#"fn positive(value: own u8) -> own unit traps requires {
  check ilt(value, 10_u8) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit traps {
  let values = array_new<u8, 2>(3_u8);
  positive(value: values[0_u64]);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(admitted_actual, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the ephemeral goal is not source-establishable: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!("expected FN-8 payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.disposition, CallRequirementDisposition::Unproved);
        assert!(
            detail
                .instantiated_goal
                .contains("argument #0 pre-transfer value")
        );
        assert_eq!(
            detail.mechanical_fix,
            "bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow"
        );
    });

    let failed_actual = br#"fn positive(value: own u8) -> own unit traps requires {
  check ilt(value, 10_u8) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit traps {
  let values = array_new<u8, 2>(3_u8);
  positive(value: values[9_u64]);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(failed_actual, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the actual's own OP-4 failure must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedBoundsObligation { .. }
        ));
    });
    assert!(
        call_goals(failed_actual, "caller").is_empty(),
        "FN-8 judgment begins only after every actual obligation succeeds"
    );
}

#[test]
fn a_call_is_judged_before_its_callee_write_and_that_write_kills_the_second_call() {
    let source =
        br#"fn update['r](value: &uniq 'r u64) -> own unit reads('r), writes('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  let old = deref(value);
  set deref(value) = old;
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    region 'first {
      update<'first>(value: &uniq 'first value);
    }
    region 'second {
      update<'second>(value: &uniq 'second value);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn s4_discharges_the_body_call_until_a_body_write_kills_it() {
    let source = br#"fn observe['r](value: &'r u64) -> own unit reads('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  let seen = deref(value);
  check True() else trap "body";
  return unit;
}

fn update['r](value: &uniq 'r u64) -> own unit reads('r), writes('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  region 'first {
    observe<'first>(value: &'first deref(value));
  }
  let old = deref(value);
  set deref(value) = old;
  region 'second {
    observe<'second>(value: &'second deref(value));
  }
  check True() else trap "body";
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "update");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn an_element_write_keeps_a_whole_goal_supported_only_by_length() {
    let source = br#"fn sized(values: own array<u8, 2>) -> own unit traps requires {
  let size = len(values);
  let exact = ieq(size, 2_u64);
  let complete = band(exact, exact);
  check complete else trap "sized";
} {
  check True() else trap "body";
  return unit;
}

fn caller(values: own array<u8, 2>) -> own unit traps {
  let size = len(values);
  let exact = ieq(size, 2_u64);
  let complete = band(exact, exact);
  if complete {
    set values[0_u64] = 9_u8;
    sized(values: move values);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "length-only support survives the element commit: {outcome:?}"
        );
    });
}

#[test]
fn array_fill_participates_only_in_body_origin_expansion() {
    let source = br#"fn need_true(value: own Bool) -> own unit traps requires {
  check value else trap "true";
} {
  check True() else trap "body";
  return unit;
}

fn probe() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  let first_size = len(values);
  let first_exact = ieq(first_size, 4_u64);
  let first = band(first_exact, first_exact);
  if first {
    let second_size = len(values);
    let second_exact = ieq(second_size, 4_u64);
    let second = band(second_exact, second_exact);
    if second {
      return unit;
    } else {
      let impossible = False();
      need_true(value: impossible);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::AllDerivable]);
}

#[test]
fn s4_is_independent_of_forward_and_mutually_recursive_traversal_order() {
    let source = br#"fn first(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  second(value: value);
  check True() else trap "body";
  return unit;
}

fn second(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  first(value: value);
  check True() else trap "body";
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["first", "second"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
        assert_eq!(
            outcomes[0].evidence,
            vec![
                CallGoalEvidence::OpaquePositive,
                CallGoalEvidence::ExactL0Projection,
            ]
        );
    }
}

#[test]
fn a_forward_concrete_generic_call_uses_its_substituted_goal() {
    let source = br#"fn main() -> own unit pure {
  return unit;
}

fn caller(value: own i32) -> own unit traps {
  let positive = igt(value, 0_i32);
  if positive {
    let result = guarded<i32>(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn guarded<T: Int>(value: own T) -> own T traps requires {
  check igt(value, 0_T) else trap "positive";
} {
  check True() else trap "body";
  return value;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ],
        "concrete generic goal: {:#?}",
        outcomes[0].goal
    );
}
