//! Emission of exact arithmetic [OP-2, ENT-6]: an accepted site has a proof of
//! its integer domain and compiles to the plain exact operation with no
//! overflow branch. The shipped emission and the forced-on entry must agree
//! exactly.

use super::{emit, emit_arithmetic_obligations};

/// The normalizer publishes its verified result bound. The caller consumes
/// that exact summary directly to discharge the addition; the final value
/// check is an ordinary test oracle.
const PROVED_EXACT: &[u8] =
    br#"fn clamp_below_thousand(value: own u64) -> result: own u64 pure contract {
  ensures result < 1000_u64;
} {
  if value < 1000_u64 {
    return value;
  } else {
    return 999_u64;
  }
}

fn increment(x: own u64) -> result: own u64 pure {
  let bounded = clamp_below_thousand(value: x);
  let stepped = bounded + 1_u64;
  return stepped;
}

fn main() -> status: own ExitStatus pure {
  let total = increment(x: 6_u64);
  if total != 7_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;

fn overflow_call_count(module: &str) -> usize {
    module
        .lines()
        .filter(|line| line.contains("call") && line.contains(".with.overflow."))
        .count()
}

/// The proved exact site is one `add` and has no overflow carrier. The
/// shipped emission and the forced-on entry are byte-identical: there is one
/// acceptance and lowering path, not a switchable pair.
///
/// The `add` carries `nuw` because the `exact` family is exactly the one
/// whose [OP-2] domain obligation the checker discharged before lowering, and
/// the backend states a proved fact rather than leaving the optimizer to
/// rediscover it (compiler/backend-facts). The flag is a statement about the
/// same one instruction, not a second branch.
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
            .any(|line| line.trim_start().starts_with('%') && line.contains("= add nuw i64")),
        "the discharged site compiles to one exact add carrying its proved domain",
    );
    assert_eq!(
        shipped,
        emit_arithmetic_obligations(PROVED_EXACT),
        "the shipped path and the forced-on entry are one judgment",
    );
}
