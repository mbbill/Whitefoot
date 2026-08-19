//! Emission of exact arithmetic [OP-2, ENT-6]: an accepted site has a proof of
//! its integer domain and compiles to the plain exact operation with no
//! overflow branch. The shipped emission and the forced-on entry must agree
//! exactly.

use super::{emit, emit_arithmetic_obligations};

/// The claim bounds `x`, so the exact `x + 1_u64` domain is proved before
/// lowering. The claim remains the only runtime rejection point.
const PROVED_EXACT: &[u8] = br#"fn increment(x: own u64) -> own u64 traps {
  claim bounded_input: ilt(x, 1000_u64) because "bounded input";
  let stepped = x + 1_u64;
  return stepped;
}

fn main() -> own unit traps {
  let total = increment(x: 6_u64);
  claim incremented_total: ieq(total, 7_u64) because "incremented total";
  return unit;
}
"#;

fn overflow_call_count(module: &str) -> usize {
    module
        .lines()
        .filter(|line| line.contains("call") && line.contains(".with.overflow."))
        .count()
}

/// The proved exact site is a plain `add` and has no overflow carrier. The
/// shipped emission and the forced-on entry are byte-identical: there is one
/// acceptance and lowering path, not a switchable pair.
#[test]
fn a_proved_exact_site_emits_no_overflow_branch() {
    let shipped = emit(PROVED_EXACT);
    assert_eq!(
        overflow_call_count(&shipped),
        0,
        "an accepted exact add has no runtime overflow branch",
    );
    assert!(
        shipped
            .lines()
            .any(|line| line.trim_start().starts_with('%') && line.contains("= add i64")),
        "the discharged site compiles to the plain exact add",
    );
    assert_eq!(
        shipped,
        emit_arithmetic_obligations(PROVED_EXACT),
        "the shipped path and the forced-on entry are one judgment",
    );
}
