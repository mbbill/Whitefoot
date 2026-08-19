//! The arithmetic-mode dissolution behind its integration switch [OP-2,
//! ENT-6]: a bare `+`/`-`/`*` with a constant operand carries an overflow
//! obligation judged like a subscript bounds obligation — discharged sites
//! lose their runtime check and `traps` contribution, undischarged sites
//! reject citing OP-2 — while two-variable bare sites and every suffixed
//! mode keep their v0.30 semantics. The default-switch controls at the end
//! pin the v0.30 behavior the shipped compiler retains.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, StaticObligationDisposition,
};

use super::super::entailment::{
    ObligationFamily, OverflowConjuncts, overflow_conjuncts_for_values,
};
use super::super::model::{
    CheckedExpression, CheckedFunction, CheckedIntegerOperation, CheckedStatement, IntegerType,
};
use super::{with_semantics, with_semantics_arithmetic};

const OVERFLOW_FIX: &str = "add a dominating `claim` of the `.defined` predicate or a dominating branch establishing its fixed normalization, or use an available total non-exact row";

/// The trap disposition of every bare-add site in one checked function.
fn add_trap_records(function: &CheckedFunction) -> Vec<bool> {
    let mut records = Vec::new();
    for statement in &function.body {
        let CheckedStatement::Let { value, .. } = statement else {
            continue;
        };
        if let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::AddExact,
            ..
        } = value
        {
            records.push(false);
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

/// A dominating branch-class fact source (here a `check`) discharges the
/// literal-operand site: the program is accepted, both overflow conjuncts
/// are discharged, and the site retains no trap record — the runtime check
/// is gone in every build mode.
#[test]
fn a_dominating_check_discharges_the_literal_site_and_drops_its_check() {
    let source = br#"fn bump(x: own u64) -> own u64 traps {
  claim bounded_input: ilt(x, 1000_u64) because "bounded input";
  let y = x + 1_u64;
  return y;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a check-dominated literal site must be accepted: {outcome:?}");
        };
        let bump = named(&checked.data.functions, "bump");
        assert_eq!(
            add_trap_records(bump),
            vec![false],
            "the discharged class site must drop its trap record",
        );
        let overflow: Vec<_> = bump
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .collect();
        assert_eq!(overflow.len(), 1, "one source occurrence, one obligation");
        assert_eq!(overflow[0].components.len(), 2);
        assert!(
            overflow.iter().all(|outcome| outcome.discharged),
            "both conjuncts discharge: the check bounds the operand and the \
             implicit type bound closes the trivial side",
        );
    });
}

/// The loop-counter shape from the recorded measurements: the counted
/// binder's `binder < upper_capture` body fact plus the capture's implicit
/// type bound discharge `i + 1_u64` by the same transitive closure the
/// index obligation uses.
#[test]
fn the_counted_binder_increment_discharges_by_transitive_closure() {
    let source = br#"fn main() -> own unit pure {
  let n = 10_u64;
  for @steps i in 0_u64..n {
    let next = i + 1_u64;
  }
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a counted-binder increment must be accepted: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        assert!(
            main.entailment
                .obligations
                .iter()
                .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
                .all(|outcome| outcome.discharged),
            "the binder increment discharges inside the counted body",
        );
    });
}

/// An operand nothing bounds leaves the binding conjunct underivable: the
/// program rejects citing OP-2 at the `infix` node with the exact folded
/// residual, and — because the class site contributes no `traps` — the
/// `pure` effect row is the correct row for this body.
#[test]
fn an_unbounded_literal_site_rejects_citing_op2_with_the_folded_residual() {
    let source = br#"fn bump(x: own u64) -> own u64 pure {
  let y = x + 1_u64;
  return y;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unbounded literal site must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "x +defined 1_u64".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: OVERFLOW_FIX,
            },
        );
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("expected a source-node citation: {:?}", issue.location());
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(
            std::str::from_utf8(&source[start..end]).expect("cited bytes are text"),
            "x + 1_u64",
            "the rejection lands on the infix node",
        );
    });
}

/// A dominating claim is the named runtime backstop: it establishes the
/// residual, the site discharges and loses its own check, and the claim —
/// which still executes — carries the function's `traps` effect.
#[test]
fn a_dominating_claim_discharges_the_site_and_carries_the_trap() {
    let source = br#"fn bump(x: own u64) -> own u64 traps {
  claim small: ile(x, 100_u64) because "callers pass a byte count";
  let y = x + 1_u64;
  return y;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a claim-backed literal site must be accepted: {outcome:?}");
        };
        let bump = named(&checked.data.functions, "bump");
        assert_eq!(
            add_trap_records(bump),
            vec![false],
            "the claim carries the runtime check; the site itself has none",
        );
        assert!(
            bump.entailment
                .obligations
                .iter()
                .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
                .all(|outcome| outcome.discharged),
            "the claim's established fact discharges the obligation",
        );
    });
}

/// `.wrap` is untouched by the dissolution: no obligation attaches, the
/// operation stays pure, and the checked program keeps its wrap identity.
#[test]
fn a_wrap_site_attaches_no_obligation() {
    let source = br#"fn main() -> own unit pure {
  let x = 6_u64;
  let y = x +wrap 1_u64;
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a wrap site must be accepted: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        assert!(
            main.entailment
                .obligations
                .iter()
                .all(|outcome| outcome.family == ObligationFamily::Bounds),
            "no overflow obligation attaches to a wrap site",
        );
    });
}

/// A bare site with two non-constant operands has no L0 normalization. Its
/// canonical `.defined` goal is still required, independent of effects.
#[test]
fn a_two_variable_site_requires_its_canonical_goal() {
    let pure_row = br#"fn main() -> own unit pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = a + b;
  return unit;
}
"#;
    with_semantics_arithmetic(pure_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a two-variable exact site must require proof: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
    let traps_row = br#"fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  let c = a + b;
  return unit;
}
"#;
    with_semantics_arithmetic(traps_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("declaring traps cannot bypass the proof obligation: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
}

/// Two constant operands make the obligation ground: an in-range result
/// discharges (and drops the check), while an inevitable overflow is a
/// compile-time rejection — there is no accepted always-trapping bare
/// spelling in the class.
#[test]
fn a_ground_obligation_discharges_in_range_and_rejects_on_inevitable_overflow() {
    let in_range = br#"fn main() -> own unit pure {
  let x = 254_u8 + 1_u8;
  return unit;
}
"#;
    with_semantics_arithmetic(in_range, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an in-range constant fold must be accepted: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        assert_eq!(add_trap_records(main), vec![false]);
    });
    let overflowing = br#"fn main() -> own unit pure {
  let x = 255_u8 + 1_u8;
  return unit;
}
"#;
    with_semantics_arithmetic(overflowing, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an inevitable constant overflow must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "255_u8 +defined 1_u8".to_owned(),
                disposition: StaticObligationDisposition::Refuted,
                mechanical_fix: OVERFLOW_FIX,
            },
        );
    });
}

/// A subscripted class operand is outside ENT-2's term vocabulary: the
/// relation is underivable, never ill-formed, and the rejection renders the
/// subscripted operand exactly — the one-`let` rebinding fallback then
/// makes the operand a term, mirroring the subscript-offset fallback.
#[test]
fn a_subscripted_class_operand_is_underivable_and_rejects() {
    let source = br#"fn main() -> own unit pure {
  let a = array_new<u8, 2>(7_u8);
  let y = a[0_u64] + 1_u8;
  return unit;
}
"#;
    with_semantics_arithmetic(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a non-term class operand must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "a[0_u64] +defined 1_u8".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: OVERFLOW_FIX,
            },
        );
    });
}

/// The shipped switch is on: the default `check_semantics` path is the
/// candidate judgment, not a v0.30 fallback. A `traps` row whose only trap
/// contributor was a constant-operand-class site now disagrees with the
/// exhibited row under EFF-2, the `pure` spelling of the same body reaches
/// the OP-2 rejection its undischarged obligation earns, and an inevitable
/// constant overflow is no longer an accepted always-trapping call.
#[test]
fn the_shipped_switch_selects_the_candidate_judgment() {
    let traps_row = br#"fn bump(x: own u64) -> own u64 traps {
  let y = x + 1_u64;
  return y;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(traps_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the class site no longer exhibits traps: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
        assert_eq!(issue.kind(), &SemanticIssueKind::EffectMismatch);
    });
    let pure_row = br#"fn bump(x: own u64) -> own u64 pure {
  let y = x + 1_u64;
  return y;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(pure_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the unbounded class site must reject on OP-2: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::UndischargedIntegerDomainObligation {
                residual: "x +defined 1_u64".to_owned(),
                disposition: StaticObligationDisposition::Unproved,
                mechanical_fix: OVERFLOW_FIX,
            },
        );
    });
    let ground = br#"fn main() -> own unit pure {
  let x = 255_u8 + 1_u8;
  return unit;
}
"#;
    with_semantics(ground, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an inevitable constant overflow must reject: {outcome:?}");
        };
        assert_eq!(
            issue.rule(),
            SemanticRule::Op2,
            "there is no accepted always-trapping bare spelling",
        );
    });
}

/// The folded conjunct constants over the signed corners, checked directly
/// against the [ENT-6] table: the sign reversals of negative multiplication
/// constants and the exact floor/ceil quotients are where the arithmetic
/// can silently go wrong.
#[test]
fn the_conjunct_fold_matches_the_ent6_table() {
    let conjuncts = |operation: CheckedIntegerOperation,
                     constant: i128,
                     constant_is_left: bool,
                     ty: IntegerType| {
        let OverflowConjuncts {
            upper,
            lower,
            ground,
            ..
        } = overflow_conjuncts_for_values(
            operation,
            constant_is_left.then_some(constant),
            (!constant_is_left).then_some(constant),
            ty,
        )
        .expect("one constant operand has a fixed normalization");
        (upper, lower, ground)
    };
    // t + 200 over u8: t <= 55; the lower side is the implicit type bound.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::AddExact,
            200,
            false,
            IntegerType::U8,
        ),
        (55, 200, false),
    );
    // t + (-3) over i8: t >= -125 binds; the upper side is relaxed.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::AddExact,
            -3,
            false,
            IntegerType::I8,
        ),
        (130, 125, false),
    );
    // t - 3 over u8: 3 <= t binds.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::SubtractExact,
            3,
            false,
            IntegerType::U8,
        ),
        (258, -3, false),
    );
    // (-100) - t over i8: t <= 28 binds.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::SubtractExact,
            -100,
            true,
            IntegerType::I8,
        ),
        (28, 227, false),
    );
    // t * 2 over i8: -64 <= t <= 63.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::MultiplyExact,
            2,
            false,
            IntegerType::I8,
        ),
        (63, 64, false),
    );
    // t * (-2) over i8: the bounds swap ends: -63 <= t <= 64.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::MultiplyExact,
            -2,
            false,
            IntegerType::I8,
        ),
        (64, 63, false),
    );
    // t * 0 is zero for every operand: ground and in range.
    assert_eq!(
        conjuncts(
            CheckedIntegerOperation::MultiplyExact,
            0,
            false,
            IntegerType::I8,
        ),
        (0, 0, true),
    );
    // Ground u64 * u64 at the magnitude ceiling: exact, and out of range.
    let max = i128::from(u64::MAX);
    let result = overflow_conjuncts_for_values(
        CheckedIntegerOperation::MultiplyExact,
        Some(max),
        Some(max),
        IntegerType::U64,
    )
    .expect("two constants have a ground normalization");
    assert_eq!((result.upper, result.lower, result.ground), (-1, -1, true));
    assert_eq!(
        result
            .ground_result
            .expect("ground obligations retain their exact result")
            .render(),
        "340282366920938463426481119284349108225",
    );
}
