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
      return values[i];
    }
    False() => {
      return values[i];
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

const table: array<u8, count> =[10_u8, 20_u8, 30_u8, 40_u8];

fn read() -> own u8 traps {
  let inside: own u8 = table[2_u64];
  let outside: own u8 = table[9_u64];
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
      return values[i];
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
      return values[i];
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
          return values[i];
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
          return values[i];
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
      return values[9_u64];
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
          return values[i];
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
      let kept: own i32 = values[p.count];
      set p.count = 9_u64;
      let lost: own i32 = values[p.count];
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

fn bump['w](p: &uniq 'w u64) -> own unit writes('w) {
  set deref(p) = 9_u64;
  return unit;
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      region 'w {
        bump<'w>(p: &uniq 'w i);
      }
      return values[i];
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

fn peek['r](p: &'r u64) -> own u64 reads('r) {
  return deref(p);
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      region 'r {
        let seen: own u64 = peek<'r>(p: &'r i);
      }
      return values[i];
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
  let in_wide: own i32 = wide[i];
  let in_narrow: own i32 = narrow[i];
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

fn read(values: own array<i32, count>, pick: own Bool) -> own i32 traps {
  match pick {
    True() => {
      let j: own u64 = 0_u64;
      match ilt<u64>(j, 4_u64) {
        True() => {
          return values[j];
        }
        False() => {
          return 0_i32;
        }
      }
    }
    False() => {
      let j: own u64 = 9_u64;
      return values[j];
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let picked: own i32 = match ilt<u64>(i, 4_u64) {
    True() => {
      give values[i];
    }
    False() => {
      give 0_i32;
    }
  }
  let after: own i32 = values[i];
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
      let a: own i32 = values[i];
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
  let before: own i32 = values[i];
  loop @l {
    let inside: own i32 = values[i];
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
    let inside: own i32 = values[i];
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

fn read(lens: own array<u8, count>, order: own array<u64, count>, j: own u64) -> own u8 traps {
  match ilt<u64>(j, 4_u64) {
    True() => {
      return lens[order[j]];
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
        Some("order[j] < len(lens)")
    );
}

#[test]
fn a_buffer_or_slice_offset_renders_the_same_subscript_spelling() {
    let source = br#"const count: u64 = 4_u64;

fn from_buffer(values: own array<u8, count>) -> own u8 allocates(heap), traps {
  let b: own buffer<u64> = buffer_new<u64>(4_u64, 0_u64);
  return values[b[0_u64]];
}

fn from_slice['r](values: own array<u8, count>, order: own slice<'r, u64>) -> own u8 reads('r), traps {
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
  let b: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);
  return b[3_u64];
}

fn runtime(n: own u64) -> own u8 allocates(heap), traps {
  let b: own buffer<u8> = buffer_new<u8>(n, 0_u8);
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
  let b: own buffer<u8> = buffer_new<u8>(n, 0_u8);
  let m: own u64 = len<u8>(b);
  match ilt<u64>(i, m) {
    True() => {
      return b[i];
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
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

#[test]
fn a_slice_of_carries_its_source_length() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>) -> own u8 traps {
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
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
  let b: own buffer<u8> = buffer_new<u8>(n, 0_u8);
  match ilt<u64>(3_u64, n) {
    True() => {
      set b[0_u64] = 1_u8;
      return b[3_u64];
    }
    False() => {
      return 0_u8;
    }
  }
}

fn killed(n: own u64) -> own u8 allocates(heap), traps {
  let b: own buffer<u8> = buffer_new<u8>(n, 0_u8);
  match ilt<u64>(3_u64, n) {
    True() => {
      set n = 0_u64;
      return b[3_u64];
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
  let b: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);
  let m: own u64 = len<u8>(b);
  let sample: own u8 = other[m];
  eat(b: move b);
  return sample;
}

fn killed(other: own array<u8, wide>) -> own u8 allocates(heap), traps {
  let b: own buffer<u8> = buffer_new<u8>(4_u64, 0_u8);
  let m: own u64 = len<u8>(b);
  eat(b: move b);
  let sample: own u8 = other[m];
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

fn write(values: own array<u16, count>, i: own u64) -> own u16 traps {
  match ilt<u64>(i, 4_u64) {
    True() => {
      set values[i] = 9_u16;
      return 1_u16;
    }
    False() => {
      set values[i] = 9_u16;
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

// ---------------------------------------------------------------------
// [ENT-3] S2 check facts
// ---------------------------------------------------------------------

#[test]
fn a_passed_check_establishes_its_comparison_on_the_continuation() {
    let source = br#"const count: u64 = 4_u64;

fn direct(values: own array<i32, count>, i: own u64) -> own i32 traps {
  check ilt<u64>(i, 4_u64) else trap "i must be in range";
  return values[i];
}

fn through_origin(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let ok: own Bool = ilt<u64>(i, 4_u64);
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
fn a_check_without_comparison_origin_establishes_nothing() {
    // `band` has no comparison origin in this version [ENT-3], so its passed
    // check contributes no fact even though it constrains the run.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let low: own Bool = ilt<u64>(i, 4_u64);
  let high: own Bool = ige<u64>(i, 0_u64);
  check band<Bool>(low, high) else trap "i must be in range";
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

fn read(values: own array<i32, count>) -> own i32 traps {
  let k: own u64 = 2_u64;
  let j: own u64 = k;
  let narrow: own u16 = 3_u16;
  let widened: own u64 = cvt<u16, u64>(narrow);
  let first: own i32 = values[j];
  let second: own i32 = values[widened];
  return iadd.wrap<i32>(first, second);
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

fn read(values: own array<i32, count>, n: own u64) -> own i32 traps {
  match ilt<u64>(n, 4_u64) {
    True() => {
      match cvt<u64, u8>(n) {
        Ok(value: small) => {
          let widened: own u64 = cvt<u8, u64>(small);
          return values[widened];
        }
        Err(error: narrowed) => {
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
  match ilt<u64>(i, 3_u64) {
    True() => {
      let next: own u64 = iadd.trap<u64>(i, 1_u64);
      return values[next];
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
        "the executed contract check is the proof [ENT-3] S7"
    );
}

#[test]
fn a_wrapping_offset_establishes_only_where_the_range_is_already_proved() {
    // The wrap has no runtime check, so the equality holds only where the
    // closed state already proves the unwrapped result stays in range.
    let source = br#"const count: u64 = 4_u64;

fn guarded(values: own array<i32, count>, p: own u64) -> own i32 traps {
  match ilt<u64>(p, 4_u64) {
    True() => {
      match ige<u64>(p, 1_u64) {
        True() => {
          let s: own u64 = isub.wrap<u64>(p, 1_u64);
          return values[s];
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

fn unguarded(values: own array<i32, count>, p: own u64) -> own i32 traps {
  match ilt<u64>(p, 4_u64) {
    True() => {
      let s: own u64 = isub.wrap<u64>(p, 1_u64);
      return values[s];
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

fn direct(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 3_u64) {
    True() => {
      match iadd.checked<u64>(i, 1_u64) {
        Ok(value: next) => {
          return values[next];
        }
        Err(error: overflowed) => {
          return 0_i32;
        }
      }
    }
    False() => {
      return 0_i32;
    }
  }
}

fn through_binding(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 3_u64) {
    True() => {
      let outcome: own Result<u64, Overflow> = iadd.checked<u64>(i, 1_u64);
      match outcome {
        Ok(value: next) => {
          return values[next];
        }
        Err(error: overflowed) => {
          return 0_i32;
        }
      }
    }
    False() => {
      return 0_i32;
    }
  }
}

fn killed(values: own array<i32, count>, i: own u64) -> own i32 traps {
  match ilt<u64>(i, 3_u64) {
    True() => {
      let outcome: own Result<u64, Overflow> = iadd.checked<u64>(i, 1_u64);
      set i = 9_u64;
      match outcome {
        Ok(value: next) => {
          return values[next];
        }
        Err(error: overflowed) => {
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

fn low(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let bound: own u64 = inside[i];
  return values[bound];
}

fn high(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let bound: own u64 = outside[i];
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

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps requires {
  let ok: own Bool = ilt<u64>(i, 4_u64);
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
        "the prologue's checked relation enters the body's entry state"
    );
}

#[test]
fn a_requires_chain_substitutes_repeatedly_and_reads_a_length_call() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps requires {
  let n: own u64 = len<i32>(values);
  let ok: own Bool = ilt<u64>(i, n);
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

fn read(values: own array<i32, count>) -> own i32 traps requires {
  let n: own u64 = len<i32>(values);
  let ok: own Bool = ilt<u64>(n, n);
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
fn a_requires_shape_outside_the_admitted_comparison_establishes_nothing() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps requires {
  let low: own Bool = ilt<u64>(i, 4_u64);
  let high: own Bool = ige<u64>(i, 0_u64);
  let ok: own Bool = band<Bool>(low, high);
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
        "a band result has no comparison origin; the prologue still executes"
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
        let sample: own u8 = table[written];
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
        let sample: own u8 = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let table: own array<u8, count> = array_new<u8, count>(0_u8);
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
  let table: own array<u8, count> = array_new<u8, count>(0_u8);
  let sink: own buffer<u8> = buffer_new<u8>(8_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd sink, offset: 0_u64, capacity: 3_u64) {
              Ok(value: copied) => {
                let good: own u8 = table[copied];
              }
              Err(error: problem) => {
                match problem {
                  CopyTooSmall(required: needed) => {
                    let bad: own u8 = table[needed];
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
fn a_let_bound_transfer_outcome_carries_the_same_count_bound() {
    // The bare IDENT form of [ENT-3] S10, under the same no-kill, no-`set`
    // path discipline as S7's checked-arithmetic origin.
    let source = br#"const count: u64 = 4_u64;

fn deferred['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome: own Result<u64, IoError> = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: 3_u64);
    match outcome {
      Ok(value: written) => {
        let sample: own u8 = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

fn killed['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome: own Result<u64, IoError> = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: limit);
    set limit = 9_u64;
    match outcome {
      Ok(value: written) => {
        let sample: own u8 = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let table: own array<u8, count> = array_new<u8, count>(0_u8);
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
  let table: own array<u8, count> = array_new<u8, count>(0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                match open_read<'c, 'p>(root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes: own buffer<u8> = buffer_new<u8>(64_u64, 0_u8);
                    region 'f {
                      region 'd {
                        match read_once<'f, 'd>(file: &uniq 'f file, destination: &uniq 'd bytes, offset: 0_u64, capacity: 3_u64) {
                          ReadBytes(count: n) => {
                            let sample: own u8 = table[n];
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
