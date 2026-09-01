//! Focused PRF-1 evidence for finite source-written affine proofs.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, SourceProofObligation,
};

use super::super::entailment::{DerivationNode, ObligationFamily, SourceAffineFactRef};
use super::{with_semantics, with_semantics_dark};

const COMMAND_MAIN: &str =
    "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

fn assert_prf1_issue(source: &[u8], expected: SourceProofObligation) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected a PRF-1 source rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Prf1);
        let SemanticIssueKind::UndischargedSourceProof {
            name, obligation, ..
        } = issue.kind()
        else {
            panic!(
                "expected an undischarged source proof, got {:?}",
                issue.kind()
            );
        };
        assert_eq!(name, "upper_bound");
        assert_eq!(*obligation, expected);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("PRF-1 must cite the complete prove statement");
        };
        let start = usize::try_from(coordinate.start().value()).expect("source offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("source offset fits usize");
        let cited = std::str::from_utf8(&source[start..end]).expect("proof source is UTF-8");
        assert!(
            cited.starts_with("prove upper_bound: ile("),
            "PRF-1 cited {cited:?} instead of the complete prove statement"
        );
        assert!(cited.ends_with('}'));
    });
}

#[test]
fn an_exact_source_proof_discharges_a_following_integer_domain() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8, replacement: own u8) -> result: own u8 pure contract {{
  requires ile(x, middle);
  requires ile(middle, 254_u8);
}} {{
  prove upper_bound: ile(x, 254_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  set middle = replacement;
  let result = x + 1_u8;
  return result;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the exact source proof must discharge the addition: {outcome:?}");
        };
        let increment = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "increment")
            .expect("increment function exists");
        super::entailment::validate_derivations(&increment.entailment);

        let [proof] = increment.entailment.source_proofs.as_slice() else {
            panic!("increment retains one PRF-1 outcome");
        };
        assert_eq!(proof.name, "upper_bound");
        assert_eq!(proof.check.premises, [true, true]);
        assert!(proof.check.combination);
        assert!(proof.check.discharged());

        let addition = increment
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition retains one OP-2 obligation");
        assert!(addition.discharged);
        assert!(
            increment.entailment.derivations.nodes.iter().any(|node| {
                matches!(
                    node,
                    DerivationNode::AffineConsequence {
                        premise: Some(SourceAffineFactRef::SourceProof { source_ordinal: 0 }),
                        ..
                    }
                )
            }),
            "retained derivations: {:#?}",
            increment.entailment.derivations.nodes
        );
    });
}

#[test]
fn the_first_unproved_use_is_reported_in_source_order() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires ile(middle, 254_u8);
}} {{
  prove upper_bound: ile(x, 254_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(source.as_bytes(), SourceProofObligation::Premise(0));
}

#[test]
fn the_second_unproved_use_is_reported_in_source_order() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires ile(x, middle);
}} {{
  prove upper_bound: ile(x, 254_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(source.as_bytes(), SourceProofObligation::Premise(1));
}

#[test]
fn proved_premises_must_sum_to_the_written_target_exactly() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires ile(x, middle);
  requires ile(middle, 254_u8);
}} {{
  prove upper_bound: ile(x, 253_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(source.as_bytes(), SourceProofObligation::Combination);
}

/// One checked source fact may serve every later goal in its dominance region;
/// the checker does not rebuild a writer assertion or a per-consumer proof
/// channel.
#[test]
fn one_source_proof_fact_discharges_multiple_bounds_and_a_call_requirement() {
    let source = format!(
        r#"fn need(index: own u64) -> result: own unit pure contract {{
  requires ile(index, 7_u64);
}} {{
  return unit;
}}

fn read(values: own array<u8, 8>, index: own u64, middle: own u64) -> result: own u8 pure contract {{
  requires ile(index, middle);
  requires ile(middle, 7_u64);
}} {{
  prove in_range: ile(index, 7_u64) {{
    use ile(index, middle);
    use ile(middle, 7_u64);
  }}
  let first = values[index];
  let second = values[index];
  need(index: index);
  return second;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "one checked proof fact must serve each dominated goal: {outcome:?}"
        );
    });
}

/// The originating proof context also proves FN-9. A source proof that reaches
/// the selected return is therefore available to the written `ensures`
/// without a separate postcondition or provenance replay.
#[test]
fn a_source_proof_fact_discharges_the_selected_return_postcondition() {
    let source = format!(
        r#"fn bounded(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires ile(x, middle);
  requires ile(middle, 254_u8);
  ensures ile(result, 254_u8);
}} {{
  prove upper_bound: ile(x, 254_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the selected return must use its originating source-proof fact: {outcome:?}"
        );
    });
}

#[test]
fn assignment_invalidates_a_source_proof_about_the_previous_value() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8, replacement: own u8) -> result: own u8 pure contract {{
  requires ile(x, middle);
  requires ile(middle, 254_u8);
}} {{
  prove upper_bound: ile(x, 254_u8) {{
    use ile(x, middle);
    use ile(middle, 254_u8);
  }}
  set x = replacement;
  let result = x + 1_u8;
  return result;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the old proof fact must not authorize the assigned value: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation { .. }
        ));
    });

    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the rejected obligation: {outcome:?}");
        };
        let increment = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "increment")
            .expect("increment function exists");
        let [proof] = increment.entailment.source_proofs.as_slice() else {
            panic!("the pre-assignment proof is still checked once");
        };
        assert!(proof.check.discharged());
        let addition = increment
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the post-assignment addition retains its obligation");
        assert!(!addition.discharged);
    });
}

/// Candidate affine facts are visited in one fixed order. An earlier fact
/// whose coefficient-one residual is not representable in i128 cannot grant
/// authority, but it also cannot conceal a later exact invariant fact.
#[test]
fn an_unrepresentable_irrelevant_residual_does_not_hide_a_later_fact() {
    let source = format!(
        r#"fn preserve_zero(x: own u64) -> result: own unit pure contract {{
  requires ile(x, 0_u64);
}} {{
  for seed in 0_u64..0_u64 {{
    invariant large_nonpositive: ile(9223372036854775808_u64 *(18446744073709551615_u64 * x), 0_u8);
  }}
  let left = x;
  for i in 0_u64..1_u64 {{
    invariant scaled_order: ile(18446744073709551615_u64 * left, 18446744073709551615_u64 * x);
    prove carried_order: ile(18446744073709551615_u64 * left, 18446744073709551615_u64 * x) {{
      use ile(18446744073709551615_u64 * left, 18446744073709551615_u64 * x);
    }}
    set left = x;
    break;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the later exact invariant must remain visible: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve_zero")
            .expect("preserve_zero function exists");
        let [proof] = function.entailment.source_proofs.as_slice() else {
            panic!("preserve_zero retains one source proof");
        };
        assert_eq!(proof.name, "carried_order");
        assert_eq!(proof.check.premises, [true]);
        assert!(proof.check.combination);
    });
}
