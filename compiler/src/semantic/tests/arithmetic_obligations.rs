//! Static integer-domain obligations for exact `+`, `-`, and `*` operations
//! [OP-2, ENT-6]. Every exact site must establish its canonical `.defined`
//! predicate. A proof discharges the obligation without adding an effect; an
//! unproved or refuted obligation rejects under OP-2. Total suffixed modes do
//! not carry this obligation.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, StaticObligationDisposition,
};

use super::super::entailment::{
    DerivationNode, ObligationFamily, OverflowConjuncts, overflow_conjuncts_for_values,
};
use super::super::model::{CheckedFunction, CheckedIntegerOperation, IntegerType};
use super::with_semantics;

const OVERFLOW_FIX: &str = "establish the fixed `.defined` normalization with a dominating branch, a verified requirement, a source invariant, or explicit finite proof steps; otherwise use an available total non-exact row or restructure the arithmetic";

fn named<'functions>(
    functions: &'functions [CheckedFunction],
    name: &str,
) -> &'functions CheckedFunction {
    functions
        .iter()
        .find(|function| function.name == name)
        .expect("named function is checked")
}

/// A verified callable-boundary requirement enters the body as an ordinary S4
/// fact and discharges both conjuncts of the exact addition.
#[test]
fn a_verified_requirement_discharges_the_literal_site() {
    let source = br#"fn bump(x: own u64) -> result: own u64 pure contract {
  requires ilt(x, 1000_u64);
} {
  let y = x + 1_u64;
  return y;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a requirement-dominated literal site must be accepted: {outcome:?}");
        };
        let bump = named(&checked.data.functions, "bump");
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
            "both conjuncts discharge: the requirement bounds the operand and the \
             implicit type bound closes the trivial side",
        );
        super::entailment::validate_derivations(&bump.entailment);
        let root = overflow[0]
            .derivation
            .expect("the accepted OP-2 site retains one proof root");
        let DerivationNode::IntegerDomain {
            goal: Some(_),
            parents,
        } = &bump.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the finite OP-2 route concludes the canonical integer-domain goal");
        };
        let [parent] = parents.as_slice() else {
            panic!("the finite OP-2 route retains one normalized-goal proof");
        };
        assert!(matches!(
            bump.entailment.derivations.nodes[parent.0 as usize],
            DerivationNode::GoalNormalization { .. }
        ));
    });
}

/// The loop-counter shape from the recorded measurements: the counted
/// binder's `binder < upper_capture` body fact plus the capture's implicit
/// type bound discharge `i + 1_u64` by the same transitive closure the
/// index obligation uses.
#[test]
fn the_counted_binder_increment_discharges_by_transitive_closure() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let n = 10_u64;
  for @steps i in 0_u64..n {
    let next = i + 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
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
/// residual. Exact arithmetic contributes no runtime effect, so `pure` is the
/// correct row for this body.
#[test]
fn an_unbounded_literal_site_rejects_citing_op2_with_the_folded_residual() {
    let source = br#"fn bump(x: own u64) -> result: own u64 pure {
  let y = x + 1_u64;
  return y;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
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

/// The taken edge of a written comparison is an ordinary S1 fact, so the
/// exact operation is accepted only in the branch where its domain holds.
#[test]
fn a_dominating_branch_discharges_the_site() {
    let source = br#"fn bump(x: own u64) -> result: own u64 pure {
  if ile(x, 100_u64) {
    let y = x + 1_u64;
    return y;
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
            panic!("a branch-dominated literal site must be accepted: {outcome:?}");
        };
        let bump = named(&checked.data.functions, "bump");
        assert!(
            bump.entailment
                .obligations
                .iter()
                .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
                .all(|outcome| outcome.discharged),
            "the taken branch's established fact discharges the obligation",
        );
    });
}

/// `.wrap` is total: no exact-domain obligation attaches, the operation stays
/// pure, and the checked program keeps its wrap identity.
#[test]
fn a_wrap_site_attaches_no_obligation() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let x = 6_u64;
  let y = x +wrap 1_u64;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
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

/// Two local operands retain their exact affine values, so a fixed in-range
/// sum discharges without a written goal. Unbounded parameters retain no such
/// upper interval and still require proof at the exact site.
#[test]
fn exact_local_values_discharge_a_two_variable_sum_but_parameters_remain_bounded() {
    let exact_locals = br#"command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = a + b;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(exact_locals, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the retained exact values prove the local sum: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        let addition = main
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition still owns one OP-2 obligation");
        assert!(addition.discharged);
    });

    let unbounded_parameters = br#"fn add(a: own u64, b: own u64) -> result: own u64 pure {
  let result = a + b;
  return result;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(unbounded_parameters, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("unbounded parameter intervals cannot prove the sum: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
}

/// Two constant operands make the obligation ground: an in-range result
/// discharges, while an inevitable overflow is a compile-time rejection.
#[test]
fn a_ground_obligation_discharges_in_range_and_rejects_on_inevitable_overflow() {
    let in_range = br#"command fn main() -> status: own ExitStatus pure {
  let x = 254_u8 + 1_u8;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(in_range, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an in-range constant fold must be accepted: {outcome:?}");
        };
        let main = named(&checked.data.functions, "main");
        let overflow: Vec<_> = main
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .collect();
        assert_eq!(overflow.len(), 1, "one exact site, one obligation");
        assert!(overflow[0].discharged, "the ground obligation is true");
    });
    let overflowing = br#"command fn main() -> status: own ExitStatus pure {
  let x = 255_u8 + 1_u8;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(overflowing, |outcome| {
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
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let a = array_new<u8, 2>(7_u8);
  let y = a[0_u64] + 1_u8;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
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

/// Rule precedence is stable on the default semantic path: an unexhibited
/// allocation effect rejects under EFF-2 before an unproved exact-site
/// obligation, while the matching `pure` row reaches OP-2.
#[test]
fn effect_mismatch_precedes_static_integer_domain_rejection() {
    let extra_effect_row = br#"fn bump(x: own u64) -> result: own u64 allocates(heap) {
  let y = x + 1_u64;
  return y;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(extra_effect_row, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an exact site does not exhibit allocation: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
        assert!(
            matches!(issue.kind(), SemanticIssueKind::EffectMismatch { .. }),
            "expected EffectMismatch, got {:?}",
            issue.kind()
        );
    });
    let pure_row = br#"fn bump(x: own u64) -> result: own u64 pure {
  let y = x + 1_u64;
  return y;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let ground = br#"command fn main() -> status: own ExitStatus pure {
  let x = 255_u8 + 1_u8;
  return exit_status(code: 0_u8);
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
