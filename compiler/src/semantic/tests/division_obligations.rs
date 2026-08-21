//! Static integer-domain obligations for exact `/` and `%` operations [OP-2,
//! ENT-6]. Every exact site must establish its complete typed `.defined`
//! predicate, including nonzero-divisor and signed `MIN / -1` safety. A proof
//! discharges the obligation without adding an effect; an unproved or refuted
//! obligation rejects under OP-2.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, StaticObligationDisposition,
};

use super::super::entailment::ObligationFamily;
use super::super::model::CheckedFunction;
use super::with_semantics;

const DIVISION_FIX: &str = "add a dominating `claim` of the `.defined` predicate or a dominating branch establishing its fixed normalization, or use an available total non-exact row";

fn named<'functions>(
    functions: &'functions [CheckedFunction],
    name: &str,
) -> &'functions CheckedFunction {
    functions
        .iter()
        .find(|function| function.name == name)
        .expect("named function is checked")
}

fn division_outcomes(
    function: &CheckedFunction,
) -> Vec<&super::super::entailment::ObligationOutcome> {
    function
        .entailment
        .obligations
        .iter()
        .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
        .collect()
}

/// A reviewed residual theorem about an uncontracted normalizer discharges
/// the zero-divisor conjunct of an unsigned site: the program is accepted and
/// both conjuncts are proved.
#[test]
fn a_stronger_claim_discharges_an_unsigned_site() {
    let source = br#"fn reviewed_positive(value: own u64) -> result: own u64 pure {
  return imax(value, 1_u64);
}

fn ratio(n: own u64, d: own u64) -> result: own u64 traps {
  let divisor = reviewed_positive(value: d);
  claim positive_divisor: igt(divisor, 0_u64) because "premises: divisor is returned by reviewed_positive, whose body computes imax(d, 1_u64)\nderivation: imax(d, 1_u64) is at least 1_u64, which is strictly greater than 0_u64\nconclusion: igt(divisor, 0_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following n / divisor exact division requires a nonzero divisor for its OP-2 domain obligation";
  let q = n / divisor;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a claim-dominated unsigned site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        let division = division_outcomes(ratio);
        assert_eq!(division.len(), 1, "one source occurrence, one obligation");
        assert_eq!(division[0].components.len(), 2);
        assert!(
            division.iter().all(|outcome| outcome.discharged),
            "the claim derives `divisor != 0` and the unsigned overflow conjunct is ground true",
        );
    });
}

/// A reviewed residual theorem spells the selected normalized divisor's
/// canonical disequality and discharges the obligation. The claim itself
/// remains the function's `traps` effect source.
#[test]
fn a_canonical_claim_discharges_the_site() {
    let source = br#"fn reviewed_nonzero(value: own u64) -> result: own u64 pure {
  return imax(value, 1_u64);
}

fn ratio(n: own u64, d: own u64) -> result: own u64 traps {
  let divisor = reviewed_nonzero(value: d);
  claim nonzero: ine(divisor, 0_u64) because "premises: divisor is returned by reviewed_nonzero, whose body computes imax(d, 1_u64)\nderivation: imax(d, 1_u64) is at least 1_u64 and therefore cannot equal 0_u64\nconclusion: ine(divisor, 0_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result disequality\nconsumers: the following n / divisor exact division requires this disequality for its OP-2 domain obligation";
  let q = n / divisor;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a claim-backed site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert!(
            division_outcomes(ratio)
                .iter()
                .all(|outcome| outcome.discharged),
            "the claim's established disequality discharges the obligation",
        );
    });
}

/// A divisor nothing bounds leaves the zero-divisor conjunct underivable:
/// the program rejects citing OP-2 at the `infix` node with the exact
/// residual `d != 0`. Exact division contributes no runtime effect, so the
/// `pure` row is correct for this body.
#[test]
fn an_unconstrained_divisor_rejects_citing_op2_with_the_exact_residual() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> result: own u64 pure {
  let q = n / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "n /defined d".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: DIVISION_FIX,
            },
        );
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("expected a source-node citation: {:?}", issue.location());
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(
            std::str::from_utf8(&source[start..end]).expect("cited bytes are text"),
            "n / d",
            "the rejection lands on the infix node",
        );
    });
}

/// The remainder row carries the identical obligation: both `/` and `%`
/// fail on exactly the same two inputs, so one class and one pair of
/// conjuncts serve both.
#[test]
fn the_remainder_row_carries_the_same_obligation() {
    let source = br#"fn residue(n: own u64, d: own u64) -> result: own u64 pure {
  let r = n % d;
  return r;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained remainder divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "n %defined d".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// A nonzero constant divisor makes the zero-divisor conjunct ground true
/// and, over a signed type, decides the `-1` half of the exceptional pair, so
/// the site discharges with no fact source.
#[test]
fn a_nonzero_constant_divisor_discharges_with_no_fact_source() {
    let source = br#"fn halve(n: own i32) -> result: own i32 pure {
  let q = n / 2_i32;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a nonzero constant divisor must be accepted: {outcome:?}");
        };
        let halve = named(&checked.data.functions, "halve");
        assert!(
            division_outcomes(halve)
                .iter()
                .all(|outcome| outcome.discharged),
        );
    });
}

/// A constant zero divisor instantiates a ground false zero-divisor
/// conjunct and is therefore rejected at every non-contradictory point.
#[test]
fn a_constant_zero_divisor_is_rejected_everywhere() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let x = 10_i32;
  let q = x / 0_i32;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a constant zero divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "x /defined 0_i32".to_owned(),
                disposition: StaticObligationDisposition::Refuted,
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// The one expressible signed-overflow shape: a constant `-1` divisor
/// reduces the disjunction to `dividend != iK::MIN`, one disequality the
/// fragment states. Nothing bounds the dividend, so conjunct one is the
/// rejection.
#[test]
fn a_minus_one_divisor_demands_the_dividend_disequality() {
    let source = br#"fn negate(n: own i32) -> result: own i32 pure {
  let q = n / -1_i32;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained dividend over -1 must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "n /defined -1_i32".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// The same site with a reviewed, uncontracted clamp that bounds the dividend
/// away from the type minimum discharges both conjuncts.
#[test]
fn a_bounded_dividend_over_minus_one_discharges() {
    let source = br#"fn clamp_above_minus_hundred(value: own i32) -> result: own i32 pure {
  return imax(value, -99_i32);
}

fn negate(n: own i32) -> result: own i32 traps {
  let bounded = clamp_above_minus_hundred(value: n);
  claim bounded_input: igt(bounded, -100_i32) because "premises: bounded is returned by clamp_above_minus_hundred, whose body computes imax(n, -99_i32)\nderivation: imax(n, -99_i32) is at least -99_i32, which is strictly greater than -100_i32\nconclusion: igt(bounded, -100_i32) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following bounded / -1_i32 exact division requires exclusion of i32::MIN for its OP-2 domain obligation";
  let q = bounded / -1_i32;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a bounded dividend over -1 must be accepted: {outcome:?}");
        };
        let negate = named(&checked.data.functions, "negate");
        assert!(
            division_outcomes(negate)
                .iter()
                .all(|outcome| outcome.discharged),
        );
    });
}

/// A signed site with two non-constant operands still requires its complete
/// typed domain predicate. A `traps` declaration cannot replace that proof;
/// EFF-2 retains precedence when the declared row already disagrees.
#[test]
fn a_signed_two_variable_site_requires_static_domain_proof() {
    let pure_row = br#"fn ratio(n: own i32, d: own i32) -> result: own i32 pure {
  let q = n / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(pure_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a signed two-variable site must require proof: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
    let traps_row = br#"fn ratio(n: own i32, d: own i32) -> result: own i32 traps {
  let q = n / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(traps_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a traps row cannot restore a runtime fallback: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
}

/// `/checked` is total: no exact-domain obligation attaches, the row stays a
/// `Result`-returning operation, and a zero divisor remains a recoverable
/// value rather than a source rejection.
#[test]
fn a_checked_division_attaches_no_obligation() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let n = 10_i64;
  let d = 0_i64;
  match n /checked d {
    Ok(value: v) => {
      return exit_status(code: 0_u8);
    }
    Err(error: e) => {
      return exit_status(code: 0_u8);
    }
  }
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a checked division must be accepted: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        assert!(
            division_outcomes(main).is_empty(),
            "no division obligation attaches to a checked site",
        );
    });
}

/// On the default checker, an unjustified `traps` declaration is judged under
/// EFF-2 before the undischarged exact-division obligation is reported.
#[test]
fn effect_mismatch_precedes_static_division_rejection() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> result: own u64 traps {
  let q = n / d;
  let r = n % d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("exact division does not justify a traps row: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
}

/// The default checker reaches the same ground-false zero-divisor conjunct as
/// the obligation-focused test entry.
#[test]
fn the_default_checker_rejects_a_constant_zero_divisor() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let x = 10_i32;
  let q = x / 0_i32;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the default path rejects a constant zero divisor: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "x /defined 0_i32".to_owned(),
                disposition: StaticObligationDisposition::Refuted,
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// The default checker accepts a `pure` body whose exact-division obligation
/// is statically discharged.
#[test]
fn the_default_checker_accepts_a_discharged_exact_division() {
    let source = br#"fn halve(n: own i32) -> result: own i32 pure {
  let q = n / 2_i32;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a discharged exact site has no runtime effect, so `pure` is correct: {outcome:?}",
        );
    });
}

/// A generic exact division is pure only when its complete typed domain
/// predicate is a static requirement. The one `/defined` goal covers both
/// zero divisors and the signed `MIN / -1` case; concrete callers discharge
/// that same template for either instance.
#[test]
fn a_generic_divisor_site_uses_one_static_domain_requirement() {
    let source = br#"fn ratio<T: Int>(n: own T, d: own T) -> result: own T pure contract {
  requires n /defined d;
} {
  let q = n / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  let a = 10_i32;
  let b = 3_i32;
  let signed = ratio<i32>(n: a, d: b);
  let x = 10_u32;
  let y = 3_u32;
  let unsigned = ratio<u32>(n: x, d: y);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the generic domain requirement must discharge both concrete instances: {outcome:?}"
        );
    });
}

/// [OP-2]'s own mechanical fix must be writable at a signed type. The claim
/// route first obtains a genuinely nonzero divisor from an uncontracted
/// normalizer, then states the selected operand's residual disequality; the
/// branch route establishes the original divisor's disequality directly.
/// Both routes compare the selected divisor against a written `0_i32`, which
/// is the same mathematical value as the zero term the conjunct is stated
/// against and therefore the same [ENT-2] term; the unsigned routes, which
/// reach the same conjunct through the type's own lower bound, are unchanged.
#[test]
fn the_signed_zero_divisor_conjunct_is_discharged_by_its_own_mechanical_fix() {
    let claimed = br#"fn reviewed_nonzero(value: own i32) -> result: own i32 pure {
  return imax(value, 1_i32);
}

fn ratio(d: own i32) -> result: own i32 traps {
  let divisor = reviewed_nonzero(value: d);
  claim nonzero: ine(divisor, 0_i32) because "premises: divisor is returned by reviewed_nonzero, whose body computes imax(d, 1_i32)\nderivation: imax(d, 1_i32) is at least 1_i32 and therefore cannot equal 0_i32\nconclusion: ine(divisor, 0_i32) is true\nchecker gap: ENT does not publish an uncontracted user-call result disequality\nconsumers: the following 100_i32 / divisor exact division requires this disequality for its OP-2 domain obligation";
  let q = 100_i32 / divisor;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(claimed, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the claimed disequality must discharge the conjunct: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert!(
            division_outcomes(ratio)
                .iter()
                .all(|outcome| outcome.discharged),
        );
    });
    let branched = br#"fn ratio(d: own i32) -> result: own i32 pure {
  if ine(d, 0_i32) {
    let q = 100_i32 / d;
    return q;
  } else {
    return 0_i32;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(branched, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the dominating branch must discharge the conjunct: {outcome:?}");
        };
        // The site sits inside the taken arm, so the obligation outcomes,
        // not the top-level statement walk, carry the verdict here.
        let ratio = named(&checked.data.functions, "ratio");
        let discharged = division_outcomes(ratio);
        assert_eq!(discharged.len(), 1, "one source occurrence, one obligation");
        assert!(discharged.iter().all(|outcome| outcome.discharged));
    });
    let unclaimed = br#"fn ratio(d: own i32) -> result: own i32 pure {
  let q = 100_i32 / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(unclaimed, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("without either route the conjunct must stay undischarged: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "100_i32 /defined d".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}
