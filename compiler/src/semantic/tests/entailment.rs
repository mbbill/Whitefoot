//! Unit tests for the dark L0 entailment engine, one family per [ENT] rule,
//! including the adversarial stale-fact and fresh-binding shapes the spec
//! text was reviewed against.
//!
//! Every test observes the engine through the retained obligation
//! dispositions: acceptance is never affected by this slice.

use crate::SemanticOutcome;

use super::super::entailment::ObligationOutcome;
use super::with_semantics;

fn obligations(source: &[u8], function: &str) -> Vec<ObligationOutcome> {
    with_semantics(source, |outcome| {
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      return index<i32>(values, i);
    }
    False() => {
      return index<i32>(values, i);
    }
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

const table: array<u8, count> = [10_u8, 20_u8, 30_u8, 40_u8];

fn read() -> own u8 traps {
  let inside: own u8 = index<u8>(table, 2_u64);
  let outside: own u8 = index<u8>(table, 9_u64);
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let flag: own Bool = ilt<u64>(i, 4_u64);
  match flag {
    True() => {
      return index<i32>(values, i);
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let flag: own Bool = ilt<u64>(i, 4_u64);
  set i = iadd.wrap<u64>(i, 1_u64);
  match flag {
    True() => {
      return index<i32>(values, i);
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 traps {
  match ile<u64>(i, p.count) {
    True() => {
      match ilt<u64>(p.count, 4_u64) {
        True() => {
          return index<i32>(values, i);
        }
        False() => {
          return 0_i32;
        }
      }
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ile<u64>(i, 4_u64) {
    True() => {
      match ieq<u64>(i, 4_u64) {
        True() => {
          return 0_i32;
        }
        False() => {
          return index<i32>(values, i);
        }
      }
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 0_u64) {
    True() => {
      return index<i32>(values, 9_u64);
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 traps {
  match ile<u64>(i, p.count) {
    True() => {
      match ilt<u64>(p.count, 4_u64) {
        True() => {
          eat(p: move p);
          return index<i32>(values, i);
        }
        False() => {
          return 0_i32;
        }
      }
    }
    False() => {
      return 0_i32;
    }
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

fn read(values: own array<i32, count>, p: own Pair) -> own i32 traps {
  match ilt<u64>(p.count, 4_u64) {
    True() => {
      set p.other = 9_u64;
      let kept: own i32 = index<i32>(values, p.count);
      set p.count = 9_u64;
      let lost: own i32 = index<i32>(values, p.count);
      return kept;
    }
    False() => {
      return 0_i32;
    }
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

fn bump ['w](p: &uniq 'w u64) -> own unit writes('w) {
  set deref(p) = 9_u64;
  return unit;
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      region 'w {
        bump<'w>(p: &uniq 'w i);
      }
      return index<i32>(values, i);
    }
    False() => {
      return 0_i32;
    }
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

fn peek ['r](p: &'r u64) -> own u64 reads('r) {
  return deref(p);
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      region 'r {
        let seen: own u64 = peek<'r>(p: &'r i);
      }
      return index<i32>(values, i);
    }
    False() => {
      return 0_i32;
    }
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

fn read(wide: own array<i32, count>, narrow: own array<i32, two>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 2_u64) {
    True() => {
    }
    False() => {
      match ilt<u64>(i, 4_u64) {
        True() => {
        }
        False() => {
          return 0_i32;
        }
      }
    }
  }
  let in_wide: own i32 = index<i32>(wide, i);
  let in_narrow: own i32 = index<i32>(narrow, i);
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
    }
    False() => {
      return 0_i32;
    }
  }
  return index<i32>(values, i);
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

fn read(values: own array<i32, count>, pick: own Bool) -> own i32 traps {
  match pick {
    True() => {
      let j: own u64 = 0_u64;
      match ilt<u64>(j, 4_u64) {
        True() => {
          return index<i32>(values, j);
        }
        False() => {
          return 0_i32;
        }
      }
    }
    False() => {
      let j: own u64 = 9_u64;
      return index<i32>(values, j);
    }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  region 'a {
    match ilt<u64>(i, 4_u64) {
      True() => {
      }
      False() => {
        return 0_i32;
      }
    }
  }
  return index<i32>(values, i);
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  loop @l {
    match ilt<u64>(i, 4_u64) {
      True() => {
        break @l;
      }
      False() => {
        return 0_i32;
      }
    }
  }
  return index<i32>(values, i);
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  loop @l {
    match ilt<u64>(i, 4_u64) {
      True() => {
        set i = iadd.wrap<u64>(i, 1_u64);
        break @l;
      }
      False() => {
        return 0_i32;
      }
    }
  }
  return index<i32>(values, i);
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let picked: own i32 = match ilt<u64>(i, 4_u64) {
    True() => {
      give index<i32>(values, i);
    }
    False() => {
      give 0_i32;
    }
  }
  let after: own i32 = index<i32>(values, i);
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
  match flag {
    True() => {
      return Ok(value: 1_u64);
    }
    False() => {
      let bad: own Fail = Bad();
      return Err(error: bad);
    }
  }
}

fn read(values: own array<i32, count>, i: own u64, flag: own Bool) -> own Result<i32, Fail> traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      let v: own u64 = propagate source(flag: flag);
      let a: own i32 = index<i32>(values, i);
      return Ok(value: a);
    }
    False() => {
      return Ok(value: 0_i32);
    }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
    }
    False() => {
      return 0_i32;
    }
  }
  let before: own i32 = index<i32>(values, i);
  loop @l {
    let inside: own i32 = index<i32>(values, i);
    set i = iadd.wrap<u64>(i, 1_u64);
    match ilt<u64>(i, 4_u64) {
      True() => {
      }
      False() => {
        break @l;
      }
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
    }
    False() => {
      return 0_i32;
    }
  }
  loop @l {
    let inside: own i32 = index<i32>(values, i);
    break @l;
  }
  return index<i32>(values, i);
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

// ---------------------------------------------------------------------
// [ENT-6] obligations and residual rendering
// ---------------------------------------------------------------------

#[test]
fn a_struct_field_base_renders_its_canonical_place_in_the_residual() {
    let source = br#"const count: u64 = 4_u64;

struct Holder {
  data: array<u8, count>;
}

fn read(h: own Holder, i: own u64) -> own u8 traps {
  return index<u8>(h.data, i);
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

fn read(lens: own array<u8, count>, order: own array<u64, count>, j: own u64) -> own u8 traps {
  match ilt<u64>(j, 4_u64) {
    True() => {
      return index<u8>(lens, index<u64>(order, j));
    }
    False() => {
      return 0_u8;
    }
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
        Some("index<u64>(order, j) < len(lens)")
    );
}

#[test]
fn a_buffer_obligation_stays_undischarged_without_the_length_fact_sources() {
    // S6 allocation-length equalities are a later additive slice; until they
    // land, a buffer bound is honestly underivable [ENT-1].
    let source = br#"fn read() -> own u8 allocates(heap), traps {
  let b: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);
  return index<u8>(b, 0_u64);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(!outcomes[0].discharged);
    assert_eq!(outcomes[0].residual.as_deref(), Some("0_u64 < len(b)"));
}

#[test]
fn set_targets_carry_the_same_obligation_in_target_position() {
    let source = br#"const count: u64 = 4_u64;

fn write(values: own array<u16, count>, i: own u64) -> own u16 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      set index<u16>(values, i) = 9_u16;
      return 1_u16;
    }
    False() => {
      set index<u16>(values, i) = 9_u16;
      return 0_u16;
    }
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
