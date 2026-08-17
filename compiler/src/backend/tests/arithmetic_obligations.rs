//! Emission of the arithmetic-mode dissolution [OP-2, ENT-6]: a discharged
//! constant-operand-class site compiles to the plain exact operation with no
//! overflow branch, while the same source under the default v0.30 switch
//! keeps every runtime overflow check. Same-source differential, so the
//! only variable is the integration switch.

use super::{emit, emit_arithmetic_obligations};

/// One function holding both classes: the `check` bounds `x`, so the
/// literal-operand `x + 1_u64` discharges, while `y + x` has two
/// non-constant operands and keeps its trap. The `check` keeps the `traps`
/// effect row correct under both switches.
const BOTH_CLASSES: &[u8] = br#"fn combine(x: own u64, y: own u64) -> own u64 traps {
  check ilt(x, 1000_u64) else trap "bounded input";
  let stepped = x + 1_u64;
  let joined = y + stepped;
  return joined;
}

fn main() -> own unit traps {
  let total = combine(x: 6_u64, y: 7_u64);
  check ieq(total, 14_u64) else trap "combined total";
  return unit;
}
"#;

fn overflow_call_count(module: &str) -> usize {
    module
        .lines()
        .filter(|line| line.contains("call") && line.contains(".with.overflow."))
        .count()
}

/// The switch is the only difference between the two emissions: v0.30 keeps
/// two overflow branches, the dissolution keeps exactly the two-variable
/// one and emits the discharged site as a plain `add`.
#[test]
fn a_discharged_class_site_emits_no_overflow_branch() {
    let baseline = emit(BOTH_CLASSES);
    assert_eq!(
        overflow_call_count(&baseline),
        2,
        "v0.30 checks both bare additions",
    );
    let dissolved = emit_arithmetic_obligations(BOTH_CLASSES);
    assert_eq!(
        overflow_call_count(&dissolved),
        1,
        "the discharged literal-operand site loses its overflow branch",
    );
    assert!(
        dissolved
            .lines()
            .any(|line| line.trim_start().starts_with('%') && line.contains("= add i64")),
        "the discharged site compiles to the plain exact add",
    );
}
