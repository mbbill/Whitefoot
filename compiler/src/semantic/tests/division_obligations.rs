//! Static integer-domain obligations for exact `/` and `%` operations [OP-2,
//! ENT-6]. Every exact site must establish its complete typed `.defined`
//! predicate, including nonzero-divisor and signed `MIN / -1` safety. A proof
//! discharges the obligation without adding an effect; an unproved or refuted
//! obligation rejects under OP-2.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, StaticObligationDisposition,
};

use super::super::entailment::{DerivationNode, ObligationFamily, S7DerivationKind};
use super::super::goal::{GoalExpression, GoalOperation};
use super::super::model::{CheckedFunction, CheckedIntegerOperation};
use super::entailment::validate_derivations;
use super::with_semantics;

const DIVISION_FIX: &str = "when the relation must hold, establish the fixed `.defined` normalization with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise use an available total non-exact row or restructure the arithmetic";

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
        .filter(|outcome| {
            outcome.family == ObligationFamily::IntegerDomain
                && matches!(
                    &outcome.canonical_goal,
                    Some(GoalExpression::Operation {
                        row: GoalOperation::Integer {
                            operation: CheckedIntegerOperation::DivideDefined
                                | CheckedIntegerOperation::RemainderDefined,
                            ..
                        },
                        ..
                    })
                )
        })
        .collect()
}

/// A positive-divisor requirement enters the body through S4 and discharges
/// the zero-divisor conjunct of an unsigned exact site.
#[test]
fn a_positive_requirement_discharges_an_unsigned_site() {
    let source = br#"fn ratio(n: own u64, divisor: own u64) -> result: own u64 pure contract {
  requires igt(divisor, 0_u64);
} {
  let q = n / divisor;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a requirement-dominated unsigned site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        let division = division_outcomes(ratio);
        assert_eq!(division.len(), 1, "one source occurrence, one obligation");
        assert_eq!(division[0].components.len(), 2);
        assert!(
            division.iter().all(|outcome| outcome.discharged),
            "the positive requirement derives `divisor != 0` and the unsigned overflow conjunct is ground true",
        );
    });
}

/// A dominating branch spells the divisor's canonical disequality and
/// discharges the exact operation only on the taken edge.
#[test]
fn a_canonical_branch_discharges_the_site() {
    let source = br#"fn ratio(n: own u64, divisor: own u64) -> result: own u64 pure {
  if ine(divisor, 0_u64) {
    let q = n / divisor;
    return q;
  } else {
    return 0_u64;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a branch-backed site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert!(
            division_outcomes(ratio)
                .iter()
                .all(|outcome| outcome.discharged),
            "the branch's established disequality discharges the obligation",
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

/// The same site with a verified entry requirement that bounds the dividend
/// away from the type minimum discharges both conjuncts.
#[test]
fn a_bounded_dividend_over_minus_one_discharges() {
    let source = br#"fn negate(n: own i32) -> result: own i32 pure contract {
  requires igt(n, -100_i32);
} {
  let q = n / -1_i32;
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
/// typed domain predicate. An unrelated effect declaration cannot replace
/// that proof; EFF-2 retains precedence when the row already disagrees.
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
    let extra_effect_row =
        br#"fn ratio(n: own i32, d: own i32) -> result: own i32 allocates(heap) {
  let q = n / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(extra_effect_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unrelated effect cannot replace a static proof: {outcome:?}");
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

/// On the default checker, an unexhibited allocation effect is judged under
/// EFF-2 before the undischarged exact-division obligation is reported.
#[test]
fn effect_mismatch_precedes_static_division_rejection() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> result: own u64 allocates(heap) {
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
            panic!("exact division does not exhibit allocation: {outcome:?}");
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

#[test]
fn unsigned_literal_division_publishes_the_quotient_bound() {
    let source = br#"fn half_floor(count: own u64) -> result: own u64 pure contract {
  ensures ile(result, count);
} {
  let quotient = count / 2_u64;
  return quotient;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the unsigned quotient bound must discharge the postcondition: {outcome:?}");
        };
        let function = named(&checked.data.functions, "half_floor");
        validate_derivations(&function.entailment);
        assert!(function.entailment.postconditions[0].aggregate.discharged);
        assert!(
            function
                .entailment
                .s7_derivations
                .iter()
                .any(|source| matches!(
                    source.kind,
                    S7DerivationKind::UnsignedDivisionBound { divisor: 2, .. }
                ))
        );
    });
}

#[test]
fn unsigned_literal_division_publishes_the_scaled_quotient_image() {
    let source = br#"fn doubled_floor(count: own u64) -> result: own u64 pure contract {
  ensures ile(result, count);
} {
  let quotient = count / 2_u64;
  let doubled = quotient * 2_u64;
  return doubled;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "the scaled quotient image must prove both multiplication and return: {outcome:?}"
            );
        };
        let function = named(&checked.data.functions, "doubled_floor");
        let domains = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .collect::<Vec<_>>();
        assert_eq!(
            domains.len(),
            2,
            "division and multiplication each own one domain"
        );
        assert!(domains.iter().all(|outcome| outcome.discharged));
        assert!(function.entailment.postconditions[0].aggregate.discharged);
    });
}

#[test]
fn signed_literal_division_does_not_publish_unsigned_ordering_images() {
    let source = br#"fn signed_half(value: own i32) -> result: own i32 pure contract {
  ensures ile(result, value);
} {
  let quotient = value / 2_i32;
  return quotient;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("signed truncation does not imply quotient <= dividend: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn9);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedPostcondition(_)
        ));
    });
}

#[test]
fn unsigned_zero_literal_still_fails_the_division_domain() {
    let source = br#"fn invalid_divisor(value: own u64) -> result: own u64 pure {
  let quotient = value / 0_u64;
  return quotient;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("zero remains outside the exact unsigned division domain: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation {
                disposition: StaticObligationDisposition::Refuted,
                ..
            }
        ));
    });
}

#[test]
fn replacing_the_quotient_does_not_transfer_its_old_division_image() {
    let source =
        br#"fn replace_quotient(count: own u64, replacement: own u64) -> result: own u64 pure {
  let quotient = count / 2_u64;
  set quotient = replacement;
  let doubled = quotient * 2_u64;
  return doubled;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a new quotient value must not inherit the old division image: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
}

#[test]
fn replacing_the_dividend_does_not_retarget_the_old_division_image() {
    let source =
        br#"fn replace_dividend(count: own u64, replacement: own u64) -> result: own u64 pure {
  let quotient = count / 2_u64;
  set count = replacement;
  let doubled = quotient * 2_u64;
  let difference = count - doubled;
  return difference;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the old dividend value must not constrain its replacement: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("the rejection must cite the exact arithmetic site");
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(&source[start..end], b"count - doubled");
    });
}

#[test]
fn a_live_alias_keeps_the_old_quotient_value_image_after_set() {
    let source = br#"fn alias_before_set(count: own u64, replacement: own u64) -> result: own u64 pure contract {
  ensures ile(result, count);
} {
  let quotient = count / 2_u64;
  let saved = quotient;
  set quotient = replacement;
  let doubled = saved * 2_u64;
  return doubled;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the image belongs to the saved runtime value, not the overwritten name: {outcome:?}"
        );
    });
}

#[test]
fn independent_branch_images_are_not_merged_without_a_value_transfer_rule() {
    let source =
        br#"fn branch_half(count: own u64, choose_left: own Bool) -> result: own u64 pure {
  let quotient = if choose_left {
    let left = count / 2_u64;
    give left;
  } else {
    let right = count / 2_u64;
    give right;
  }
  let doubled = quotient * 2_u64;
  return doubled;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("branch-local value atoms require an explicit delivery transfer: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
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

/// [OP-2]'s mechanical fixes are writable at a signed type. A verified
/// requirement establishes the complete body fact through S4, while a branch
/// establishes the original divisor's disequality on its taken edge.
#[test]
fn the_signed_zero_divisor_conjunct_is_discharged_by_its_own_mechanical_fix() {
    let required = br#"fn ratio(divisor: own i32) -> result: own i32 pure contract {
  requires ine(divisor, 0_i32);
} {
  let q = 100_i32 / divisor;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(required, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the required disequality must discharge the conjunct: {outcome:?}");
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
    let unproved = br#"fn ratio(d: own i32) -> result: own i32 pure {
  let q = 100_i32 / d;
  return q;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(unproved, |outcome| {
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

#[test]
fn active_invariants_prove_signed_division_and_remainder_domains() {
    let source = br#"fn exact_pairs(dividend_start: own i32, divisor_start: own i32, limit: own u64) -> result: own unit pure contract {
  requires ile(-10_i32, dividend_start);
  requires ile(dividend_start, 100_i32);
  requires ile(1_i32, divisor_start);
  requires ile(divisor_start, 100_i32);
  requires ile(limit, 10_u64);
} {
  let dividend = dividend_start;
  let divisor = divisor_start;
  for @items (
    i in 0_u64..limit,
    invariant dividend_lower: ile(-10_i32, dividend),
    invariant dividend_progress: ile(dividend, dividend_start + i),
    invariant divisor_positive: ile(1_i32, divisor),
    invariant divisor_progress: ile(divisor, divisor_start + i)
  ) {
    let quotient_positive = -2147483648_i32 / divisor;
    let remainder_positive = -2147483648_i32 % divisor;
    let quotient_negative_one = dividend / -1_i32;
    let remainder_negative_one = dividend % -1_i32;
    set dividend = dividend + 1_i32;
    set divisor = divisor + 1_i32;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the active invariants must prove all signed domains: {outcome:?}");
        };
        let function = named(&checked.data.functions, "exact_pairs");
        let domains = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| {
                outcome.family == ObligationFamily::IntegerDomain
                    && matches!(
                        &outcome.canonical_goal,
                        Some(GoalExpression::Operation {
                            row: GoalOperation::Integer {
                                operation: CheckedIntegerOperation::DivideDefined
                                    | CheckedIntegerOperation::RemainderDefined,
                                ..
                            },
                            ..
                        })
                    )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            domains.len(),
            4,
            "two division and two remainder sites remain distinct"
        );
        for domain in domains {
            assert_eq!(domain.components.len(), 3);
            assert!(domain.discharged);
            assert!(domain.residual.is_none());

            let root = domain
                .derivation
                .expect("the accepted signed operation retains a derivation root");
            let mut seen = vec![false; function.entailment.derivations.nodes.len()];
            let mut stack = vec![root];
            let mut used_invariant = false;
            while let Some(node) = stack.pop() {
                let index = node.0 as usize;
                if seen[index] {
                    continue;
                }
                seen[index] = true;
                let retained = &function.entailment.derivations.nodes[index];
                used_invariant |= matches!(
                    retained,
                    DerivationNode::AffineConsequence {
                        premises,
                        ..
                    } if !premises.is_empty()
                );
                stack.extend(retained.parent_ids());
            }
            assert!(
                used_invariant,
                "each signed domain proof must consume a source invariant"
            );
        }
    });
}

#[test]
fn an_indexed_defined_guard_discharges_the_same_structural_exact_operation() {
    let source = br#"fn increment(values: own array<u8, 1>) -> result: own u8 pure {
  if values[0_u64] +defined 1_u8 {
    let result = values[0_u64] + 1_u8;
    return result;
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 1>(0_u8);
  let result = increment(values: move values);
  return exit_status(code: result);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the identical indexed value must retain one structural goal: {outcome:?}");
        };
        let increment = named(&checked.data.functions, "increment");
        let exact = increment
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition retains its OP-2 obligation");
        assert!(exact.discharged);
        let Some(GoalExpression::Operation { arguments, .. }) = &exact.canonical_goal else {
            panic!("every OP-2 occurrence must retain its canonical goal");
        };
        assert!(matches!(
            arguments.as_slice(),
            [
                GoalExpression::Operation {
                    row: GoalOperation::ArrayIndex { .. },
                    ..
                },
                GoalExpression::Datum(_)
            ]
        ));
    });
}

#[test]
fn writing_the_indexed_collection_invalidates_its_old_defined_fact() {
    let source = br#"fn increment_after_write(values: own array<u8, 1>) -> result: own u8 pure {
  if values[0_u64] +defined 1_u8 {
    set values[0_u64] = 255_u8;
    let result = values[0_u64] + 1_u8;
    return result;
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 1>(0_u8);
  let result = increment_after_write(values: move values);
  return exit_status(code: result);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a collection write must kill the old indexed goal: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation {
                disposition: StaticObligationDisposition::Unproved,
                ..
            }
        ));
    });
}

#[test]
fn a_buffer_indexed_defined_guard_discharges_the_same_structural_exact_operation() {
    let source = br#"fn increment['v](values: &'v buffer<u8>) -> result: own u8 reads(values) {
  let room = len(deref(values));
  if ilt(0_u64, room) {
    if deref(values)[0_u64] +defined 1_u8 {
      let result = deref(values)[0_u64] + 1_u8;
      return result;
    } else {
      return 0_u8;
    }
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the identical buffer element must retain one structural goal: {outcome:?}");
        };
        let increment = named(&checked.data.functions, "increment");
        let exact = increment
            .entailment
            .obligations
            .iter()
            .rfind(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition retains its OP-2 obligation");
        assert!(exact.discharged);
        let Some(GoalExpression::Operation { arguments, .. }) = &exact.canonical_goal else {
            panic!("every OP-2 occurrence must retain its canonical goal");
        };
        assert!(matches!(
            arguments.as_slice(),
            [
                GoalExpression::Operation {
                    row: GoalOperation::BufferIndex { .. },
                    ..
                },
                GoalExpression::Datum(_)
            ]
        ));
    });
}

#[test]
fn a_slice_indexed_defined_guard_discharges_the_same_structural_exact_operation() {
    let source = br#"fn increment['v](values: own slice<'v, u8>) -> result: own u8 reads(values) {
  let room = len(values);
  if ilt(0_u64, room) {
    if values[0_u64] +defined 1_u8 {
      let result = values[0_u64] + 1_u8;
      return result;
    } else {
      return 0_u8;
    }
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the identical slice element must retain one structural goal: {outcome:?}");
        };
        let increment = named(&checked.data.functions, "increment");
        let exact = increment
            .entailment
            .obligations
            .iter()
            .rfind(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition retains its OP-2 obligation");
        assert!(exact.discharged);
        let Some(GoalExpression::Operation { arguments, .. }) = &exact.canonical_goal else {
            panic!("every OP-2 occurrence must retain its canonical goal");
        };
        assert!(matches!(
            arguments.as_slice(),
            [
                GoalExpression::Operation {
                    row: GoalOperation::SliceIndex { .. },
                    ..
                },
                GoalExpression::Datum(_)
            ]
        ));
    });
}

#[test]
fn different_index_offsets_do_not_share_a_defined_fact() {
    let source = br#"fn increment_other(values: own array<u8, 2>) -> result: own u8 pure {
  if values[0_u64] +defined 1_u8 {
    let result = values[1_u64] + 1_u8;
    return result;
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("different index identities must not share a defined fact: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation {
                disposition: StaticObligationDisposition::Unproved,
                residual,
                ..
            } if residual == "values[1_u64] +defined 1_u8"
        ));
    });
}

#[test]
fn writing_the_index_binding_invalidates_its_old_indexed_defined_fact() {
    let source =
        br#"fn increment_after_index_write(values: own array<u8, 2>) -> result: own u8 pure {
  let offset = 1_u64;
  if values[offset] +defined 1_u8 {
    set offset = 0_u64;
    let result = values[offset] + 1_u8;
    return result;
  } else {
    return 0_u8;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an index write must kill the old indexed goal: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation {
                disposition: StaticObligationDisposition::Unproved,
                residual,
                ..
            } if residual == "values[offset] +defined 1_u8"
        ));
    });
}
