//! Mutable measure facts follow the storage a write may replace [ENT-5, MSR-2].

use super::{assert_accepts, assert_rule_kind};
use crate::SemanticRule;

#[test]
fn a_call_through_a_joined_reference_kills_every_possible_targets_fact() {
    let source = br#"fn write(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 1_u64;
  return unit;
}

fn examine(flag: own u64) -> result: own unit pure {
  let a = 0_u64;
  let b = 0_u64;
  let p = if flag == 1_u64 {
    give &a;
  } else {
    give &b;
  }
  if b == 0_u64 {
    write(target: p);
    invariant stale: b == 0_u64;
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  examine(flag: 0_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Inv1, |_| true);
}

#[test]
fn a_joined_reference_write_cannot_preserve_a_nonzero_divisor_proof() {
    let source = br#"fn zero(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 0_u64;
  return unit;
}

fn examine(flag: own u64) -> result: own u64 pure {
  let a = 1_u64;
  let b = 1_u64;
  let p = if flag == 1_u64 {
    give &a;
  } else {
    give &b;
  }
  if b != 0_u64 {
    zero(target: p);
    return 1_u64 / b;
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  let quotient = examine(flag: 0_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Op2, |_| true);
}

#[test]
fn replacing_an_indexed_window_kills_its_old_length() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let row = slots_new::<u8, 4>();
  place_back(window: &row, value: 7_u8);
  let table = slots_new::<Slots<u8, 4>, 2>();
  place_back(window: &table, value: move row);
  let index = 0_u64;
  if table[index].len == 1_u64 {
    let blank = slots_new::<u8, 4>();
    set table[index] = move blank;
    let observed = table[index][0_u64];
    return exit_status(code: observed);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Op4, |_| true);
}

#[test]
fn a_written_proof_cannot_reuse_a_replaced_elements_length() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let row = slots_new::<u8, 4>();
  place_back(window: &row, value: 7_u8);
  let table = slots_new::<Slots<u8, 4>, 2>();
  place_back(window: &table, value: move row);
  let index = 0_u64;
  if table[index].len == 1_u64 {
    let blank = slots_new::<u8, 4>();
    set table[index] = move blank;
    invariant stale: table[index].len == 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Inv1, |_| true);
}

#[test]
fn replacing_a_distinct_element_preserves_the_measured_element() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let row = slots_new::<u8, 4>();
  place_back(window: &row, value: 7_u8);
  let other = slots_new::<u8, 4>();
  let table = slots_new::<Slots<u8, 4>, 2>();
  place_back(window: &table, value: move row);
  place_back(window: &table, value: move other);
  if table[0_u64].len == 1_u64 {
    let initial = table[0_u64][0_u64];
    let blank = slots_new::<u8, 4>();
    set table[1_u64] = move blank;
    let observed = table[0_u64][0_u64];
    return exit_status(code: observed);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

#[test]
fn changing_length_preserves_a_runtime_capacity_fact() {
    let source = br#"fn check_capacity(value: own u64) -> result: own unit pure contract {
  requires value == 4_u64;
} {
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let window = box_slots_new::<u8>(capacity: 4_u64);
  place_back(window: &window.inner, value: 7_u8);
  check_capacity(value: window.inner.cap);
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}
