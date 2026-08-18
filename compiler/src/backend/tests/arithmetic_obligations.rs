//! Emission of the arithmetic-mode dissolution [OP-2, ENT-6]: a discharged
//! constant-operand-class site compiles to the plain exact operation with no
//! overflow branch, while a two-non-constant site keeps its runtime overflow
//! check. The switch is on, so the shipped emission is this one; the
//! forced-on entry must agree with it exactly.

use super::{emit, emit_arithmetic_obligations};

/// One function holding both classes: the `check` bounds `x`, so the
/// literal-operand `x + 1_u64` discharges, while `y + x` has two
/// non-constant operands and keeps its trap. The `check` keeps the `traps`
/// effect row correct under both switches.
const BOTH_CLASSES: &[u8] = br#"fn combine(x: own u64, y: own u64) -> own u64 traps {
  claim bounded_input: ilt(x, 1000_u64) because "bounded input";
  let stepped = x + 1_u64;
  let joined = y + stepped;
  return joined;
}

fn main() -> own unit traps {
  let total = combine(x: 6_u64, y: 7_u64);
  claim combined_total: ieq(total, 14_u64) because "combined total";
  return unit;
}
"#;

fn overflow_call_count(module: &str) -> usize {
    module
        .lines()
        .filter(|line| line.contains("call") && line.contains(".with.overflow."))
        .count()
}

/// Exactly the two-variable overflow branch survives, and the discharged
/// literal-operand site is a plain `add`. The shipped emission and the
/// forced-on entry are byte-identical: there is one acceptance and lowering
/// path, not a switchable pair.
#[test]
fn a_discharged_class_site_emits_no_overflow_branch() {
    let shipped = emit(BOTH_CLASSES);
    assert_eq!(
        overflow_call_count(&shipped),
        1,
        "the discharged literal-operand site loses its overflow branch",
    );
    assert!(
        shipped
            .lines()
            .any(|line| line.trim_start().starts_with('%') && line.contains("= add i64")),
        "the discharged site compiles to the plain exact add",
    );
    assert_eq!(
        shipped,
        emit_arithmetic_obligations(BOTH_CLASSES),
        "the shipped path and the forced-on entry are one judgment",
    );
}
