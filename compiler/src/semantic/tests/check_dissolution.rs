//! Check dissolution (#47): `claim` [CLM-1] is the sole writer-stated trap
//! construct, and the body `check` statement retires.
//!
//! Active v0.33 removes `check_stmt` from the [GRAM-4] `stmt`
//! alternation, so the
//! retirement is a parse rejection rather than a semantic one; that half is
//! pinned in `syntax::parser::tests`. What remains checkable here is the two
//! things the grammar removal must *not* have broken: the non-executable
//! clauses of the unified contract block, and S3 claim establishment, which
//! carries signed Boolean decomposition without any retired body assertion.
//! Design: `research/investigations/check-dissolution/SPEC-DELTA.md`.

use crate::SemanticOutcome;

use super::with_semantics;

/// `requires` and `ensures` clauses are static contract syntax owned by FN-8
/// and FN-9, not body statements or runtime checks.
#[test]
fn contract_clauses_survive_the_body_statement_retirement() {
    let source =
        br#"fn pick(table: own array<u8, 8>, index: own u64) -> value: own u8 pure contract {
  define admitted = ilt(index, 8_u64);
  requires admitted;
} {
  let value = table[index];
  return value;
}

fn identity(value: own i32) -> out: own i32 pure contract {
  ensures ieq(out, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "static contract clauses must stay accepted after the body check retires: {outcome:?}"
        );
    });
}

/// S3 claim establishment carries the signed Boolean decomposition: the
/// `band` claim's positive conjuncts discharge both guarded subscripts with
/// no body check anywhere. Decomposition attaches at establishment sources,
/// not at the retired statement form.
#[test]
fn a_band_claim_discharges_decomposed_bounds_without_any_body_check() {
    let source =
        br#"fn read_pair(table: own array<u8, 8>, low: own u64, high: own u64) -> out: own u8 traps {
  let low_ok = ilt(low, 8_u64);
  let high_ok = ilt(high, 8_u64);
  let both = band(low_ok, high_ok);
  claim pair_in_range: both because "both offsets were compared against the table length";
  let first = table[low];
  let second = table[high];
  return second;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the S3 band claim must discharge both subscripts: {outcome:?}"
        );
    });
}
