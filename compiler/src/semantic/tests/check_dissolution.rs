//! The check-dissolution candidate behind its default-off integration
//! switch (#47): with the switch on, a body `check_stmt` is rejected
//! because `claim` [CLM-1] is the sole writer-stated trap construct, while
//! the contract finals of `requires`/`ensures` blocks [FN-8, FN-9] are
//! untouched in either switch position. S3 claim establishment carries the
//! signed Boolean decomposition without any body check, so goal
//! decomposition survives S2 retirement.
//! Design: `research/investigations/check-dissolution/SPEC-DELTA.md`.

use crate::SemanticOutcome;

use super::super::{SemanticIssueKind, SemanticRule};
use super::{with_semantics, with_semantics_check_dissolution};

/// The one v0.31/v0.32-candidate acceptance flip: a body check is accepted
/// with the switch off and retires at its exact statement node with the
/// switch on.
#[test]
fn a_body_check_is_accepted_off_switch_and_retires_on_switch() {
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  check flag else trap "body check";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the shipped v0.31 path must keep accepting a body check: {outcome:?}"
        );
    });
    with_semantics_check_dissolution(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the dissolution switch must retire the body check: {outcome:?}");
        };
        assert_eq!(issue.rule, SemanticRule::Op5);
        assert!(matches!(
            issue.kind,
            SemanticIssueKind::RetiredCheckStatement { .. }
        ));
    });
}

/// The `requires` and `ensures` finals are contract syntax owned by FN-8
/// and FN-9, not body statements; the dissolution switch must leave a
/// requirement-bearing, postcondition-bearing program accepted.
#[test]
fn contract_final_checks_survive_the_dissolution_switch() {
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
    with_semantics_check_dissolution(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "contract finals must stay accepted under dissolution: {outcome:?}"
        );
    });
}

/// S3 claim establishment carries the signed Boolean decomposition: the
/// `band` claim's positive conjuncts discharge both guarded subscripts with
/// no body check anywhere, in both switch positions. This is the recorded
/// S2-retirement condition — decomposition attaches at establishment
/// sources, not at the retired statement form.
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
    with_semantics_check_dissolution(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "S3 decomposition must survive the dissolution switch: {outcome:?}"
        );
    });
}
