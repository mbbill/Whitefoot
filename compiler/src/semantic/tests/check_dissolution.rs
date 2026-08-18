//! Check dissolution (#47): `claim` [CLM-1] is the sole writer-stated trap
//! construct, and the body `check` statement retires.
//!
//! v0.32 removes `check_stmt` from the [GRAM-4] `stmt` alternation, so the
//! retirement is a parse rejection rather than a semantic one; that half is
//! pinned in `syntax::parser::tests`. What remains checkable here is the two
//! things the grammar removal must *not* have broken: the contract finals of
//! `requires`/`ensures` blocks, which [GRAM-2] now admits directly at
//! `requires_entry`/`ensures_entry`, and S3 claim establishment, which
//! carries the signed Boolean decomposition that retired [ENT-3.S2] used to.
//! Design: `research/investigations/check-dissolution/SPEC-DELTA.md`.

use crate::SemanticOutcome;

use super::with_semantics;

/// The `requires` and `ensures` finals are contract syntax owned by FN-8 and
/// FN-9, not body statements. `check_stmt` survives as exactly that form, so
/// a requirement-bearing, postcondition-bearing program stays accepted even
/// though no body may spell `check`.
#[test]
fn contract_final_checks_survive_the_body_statement_retirement() {
    let source = br#"fn pick(table: own array<u8, 8>, index: own u64) -> own u8 pure requires {
  let admitted = ilt(index, 8_u64);
  check admitted else trap "index in range";
} {
  let value = table[index];
  return value;
}

fn identity(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "contract finals must stay accepted after the body check retires: {outcome:?}"
        );
    });
}

/// S3 claim establishment carries the signed Boolean decomposition: the
/// `band` claim's positive conjuncts discharge both guarded subscripts with
/// no body check anywhere. This is the recorded S2-retirement condition —
/// decomposition attaches at establishment sources, not at the retired
/// statement form.
#[test]
fn a_band_claim_discharges_decomposed_bounds_without_any_body_check() {
    let source =
        br#"fn read_pair(table: own array<u8, 8>, low: own u64, high: own u64) -> own u8 traps {
  let low_ok = ilt(low, 8_u64);
  let high_ok = ilt(high, 8_u64);
  let both = band(low_ok, high_ok);
  claim pair_in_range: both because "both offsets were compared against the table length";
  let first = table[low];
  let second = table[high];
  return second;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the S3 band claim must discharge both subscripts: {outcome:?}"
        );
    });
}
