//! The division dissolution behind its integration switch [OP-2, ENT-6]: a
//! bare `/` or `%` in the divisor class carries a division obligation judged
//! like a subscript bounds obligation — discharged sites lose their runtime
//! check and `traps` contribution, undischarged sites reject citing OP-2 —
//! while a signed site with two non-constant operands keeps its v0.31
//! trapping semantics, because its `iK::MIN / -1` safe condition is a
//! disjunction the [ENT-4] fragment cannot state. The default-switch
//! controls at the end pin the v0.31 behavior the shipped compiler retains.

use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule};

use super::super::entailment::ObligationFamily;
use super::super::model::{
    CheckedExpression, CheckedFunction, CheckedIntegerOperation, CheckedStatement,
};
use super::{with_semantics, with_semantics_division};

const DIVISION_FIX: &str = "add a dominating `claim` of the residual or a dominating branch establishing it, or respell the operation `checked`";

/// The trap disposition of every bare divide/remainder site in one checked
/// function, in source order.
fn division_trap_records(function: &CheckedFunction) -> Vec<bool> {
    let mut records = Vec::new();
    for statement in &function.body {
        let CheckedStatement::Let { value, .. } = statement else {
            continue;
        };
        if let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::DivideTrap | CheckedIntegerOperation::RemainderTrap,
            trap,
            ..
        } = value
        {
            records.push(trap.is_some());
        }
    }
    records
}

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
        .filter(|outcome| outcome.family == ObligationFamily::Division)
        .collect()
}

/// A dominating branch-class fact source discharges the zero-divisor
/// conjunct of an unsigned site: the program is accepted, both conjuncts
/// discharge, and the site retains no trap record — the runtime check is
/// gone in every build mode, and the row is `pure`.
#[test]
fn a_dominating_check_discharges_an_unsigned_site_and_drops_its_check() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> own u64 traps {
  claim positive_divisor: igt(d, 0_u64) because "positive divisor";
  let q = n / d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a check-dominated unsigned site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert_eq!(
            division_trap_records(ratio),
            vec![false],
            "the discharged class site must drop its trap record",
        );
        let division = division_outcomes(ratio);
        assert_eq!(division.len(), 2, "one obligation, two conjuncts");
        assert_eq!(
            (division[0].conjunct, division[1].conjunct),
            (0, 1),
            "zero-divisor conjunct at ordinal zero, signed overflow at one",
        );
        assert!(
            division.iter().all(|outcome| outcome.discharged),
            "the check derives `d != 0` and the unsigned overflow conjunct is ground true",
        );
    });
}

/// The same discharge through the named runtime backstop: a dominating
/// claim establishes the disequality, the site loses its own check, and the
/// claim carries the function's `traps` effect.
#[test]
fn a_dominating_claim_discharges_the_site_and_carries_the_trap() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> own u64 traps {
  claim nonzero: ine(d, 0_u64) because "callers pass a nonzero stride";
  let q = n / d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a claim-backed site must be accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert_eq!(
            division_trap_records(ratio),
            vec![false],
            "the claim carries the runtime check; the site itself has none",
        );
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
/// residual `d != 0`, and — because the class site contributes no `traps` —
/// the `pure` row is the correct row for this body.
#[test]
fn an_unconstrained_divisor_rejects_citing_op2_with_the_exact_residual() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> own u64 pure {
  let q = n / d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedDivisionObligation {
                residual: "d != 0".to_owned(),
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
    let source = br#"fn residue(n: own u64, d: own u64) -> own u64 pure {
  let r = n % d;
  return r;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained remainder divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedDivisionObligation {
                residual: "d != 0".to_owned(),
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// A nonzero constant divisor makes the zero-divisor conjunct ground true
/// and — over a signed type — decides the `-1` half of the trapping pair,
/// so the site discharges with no fact source at all and needs no `traps`.
#[test]
fn a_nonzero_constant_divisor_discharges_with_no_fact_source() {
    let source = br#"fn halve(n: own i32) -> own i32 pure {
  let q = n / 2_i32;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a nonzero constant divisor must be accepted: {outcome:?}");
        };
        let halve = named(&checked.data.functions, "halve");
        assert_eq!(
            division_trap_records(halve),
            vec![false],
            "a constant divisor leaves nothing to test at runtime",
        );
        assert!(
            division_outcomes(halve)
                .iter()
                .all(|outcome| outcome.discharged),
        );
    });
}

/// A constant zero divisor instantiates a ground false zero-divisor
/// conjunct and is therefore rejected at every non-contradictory point:
/// there is no accepted always-trapping bare spelling.
#[test]
fn a_constant_zero_divisor_is_rejected_everywhere() {
    let source = br#"fn main() -> own unit traps {
  let x = 10_i32;
  let q = x / 0_i32;
  claim unreachable: igt(q, 0_i32) because "unreachable";
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a constant zero divisor must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedDivisionObligation {
                residual: "0_i32 != 0".to_owned(),
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
    let source = br#"fn negate(n: own i32) -> own i32 pure {
  let q = n / -1_i32;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unconstrained dividend over -1 must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedDivisionObligation {
                residual: "n != -2147483648".to_owned(),
                mechanical_fix: DIVISION_FIX,
            },
        );
    });
}

/// The same site with the dividend bounded away from the type minimum
/// discharges both conjuncts and drops the whole trap.
#[test]
fn a_bounded_dividend_over_minus_one_discharges() {
    let source = br#"fn negate(n: own i32) -> own i32 traps {
  claim bounded_input: igt(n, -100_i32) because "bounded input";
  let q = n / -1_i32;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a bounded dividend over -1 must be accepted: {outcome:?}");
        };
        let negate = named(&checked.data.functions, "negate");
        assert_eq!(division_trap_records(negate), vec![false]);
        assert!(
            division_outcomes(negate)
                .iter()
                .all(|outcome| outcome.discharged),
        );
    });
}

/// A signed site with two non-constant operands is outside the class: its
/// safe condition `dividend != iK::MIN or divisor != -1` is a disjunction
/// the conjunctive fragment cannot state, so it keeps its complete trap
/// record, still exhibits `traps`, and attaches no obligation.
#[test]
fn a_signed_two_variable_site_retains_the_trap_and_its_effect() {
    let pure_row = br#"fn ratio(n: own i32, d: own i32) -> own i32 pure {
  let q = n / d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(pure_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a signed two-variable site still exhibits traps: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
    let traps_row = br#"fn ratio(n: own i32, d: own i32) -> own i32 traps {
  let q = n / d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_division(traps_row, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the retained trapping class must stay accepted: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert_eq!(
            division_trap_records(ratio),
            vec![true],
            "the retained class keeps its runtime trap",
        );
        assert!(
            division_outcomes(ratio).is_empty(),
            "no obligation attaches outside the class",
        );
    });
}

/// `/checked` is untouched by the dissolution: no obligation attaches, the
/// row stays a total `Result`-returning operation, and the zero divisor
/// remains a recoverable value rather than a source rejection.
#[test]
fn a_checked_division_attaches_no_obligation() {
    let source = br#"fn main() -> own unit pure {
  let n = 10_i64;
  let d = 0_i64;
  match n /checked d {
    Ok(value: v) => {
      return unit;
    }
    Err(error: e) => {
      return unit;
    }
  }
}
"#;
    with_semantics_division(source, |outcome| {
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

/// Default-switch control: with [`DIVISION_OBLIGATIONS`] off, the shipped
/// compiler keeps the active-v0.31 judgment — every bare site retains its
/// trap record and no division obligation exists anywhere.
///
/// [`DIVISION_OBLIGATIONS`]: super::super::check::DIVISION_OBLIGATIONS
#[test]
fn the_default_switch_keeps_every_bare_site_trapping() {
    let source = br#"fn ratio(n: own u64, d: own u64) -> own u64 traps {
  let q = n / d;
  let r = n % d;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the shipped path accepts an unconstrained divisor: {outcome:?}");
        };
        let ratio = named(&checked.data.functions, "ratio");
        assert_eq!(
            division_trap_records(ratio),
            vec![true, true],
            "with the switch off every bare site keeps its trap record",
        );
        assert!(
            division_outcomes(ratio).is_empty(),
            "with the switch off no division obligation is attached",
        );
    });
}

/// Default-switch control on the constant-zero case: the active
/// specification still accepts `x / 0_i32` as a well-typed always-trapping
/// call, which is exactly what the candidate changes.
#[test]
fn the_default_switch_still_accepts_a_constant_zero_divisor() {
    let source = br#"fn main() -> own unit traps {
  let x = 10_i32;
  let q = x / 0_i32;
  claim unreachable: igt(q, 0_i32) because "unreachable";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the shipped path accepts a constant zero divisor: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        assert_eq!(division_trap_records(main), vec![true]);
    });
}

/// The one direction in which the candidate accepts more: a body whose only
/// trap contributor was a divisor-class bare site can now write the narrower
/// `pure` row, which the active specification rejects under EFF-2. This is
/// the exact converse of the effect-row rejection the acceptance-set
/// analysis records, and it is the only newly accepted class.
#[test]
fn the_default_switch_rejects_a_pure_row_the_candidate_accepts() {
    let source = br#"fn halve(n: own i32) -> own i32 pure {
  let q = n / 2_i32;
  return q;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("v0.31 makes every bare division exhibit traps: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
    with_semantics_division(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the discharged class site contributes no traps, so `pure` is correct",
        );
    });
}
