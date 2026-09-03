//! Focused PRF-1 evidence for finite source-written affine proofs.

use crate::{
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, SourceProofObligation,
};

use super::super::entailment::affine::MAX_CERTIFICATE_PREMISES;
use super::super::entailment::{
    CallGoalDisposition, CallGoalEvidence, DerivationNode, ObligationFamily, SourceAffineFactRef,
    SourceProofCertificateFailure,
};
use super::{with_semantics, with_semantics_dark};

const COMMAND_MAIN: &str =
    "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[derive(Clone, Copy)]
enum ExpectedProofIssueNode<'source> {
    Invariant,
    Use {
        source: &'source str,
        /// Zero-based occurrence of this exact `use` spelling after the
        /// owning invariant begins. This distinguishes repeated entries.
        occurrence: usize,
    },
}

fn assert_prf1_issue(
    source: &[u8],
    expected: SourceProofObligation,
    expected_node: ExpectedProofIssueNode<'_>,
) {
    assert_prf1_issue_named(source, expected, "upper_bound", expected_node);
}

fn assert_prf1_issue_named(
    source: &[u8],
    expected: SourceProofObligation,
    expected_name: &str,
    expected_node: ExpectedProofIssueNode<'_>,
) {
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
        assert_eq!(name, expected_name);
        assert_eq!(*obligation, expected);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("PRF-1 must cite the complete invariant statement");
        };
        let start = usize::try_from(coordinate.start().value()).expect("source offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("source offset fits usize");
        let cited = std::str::from_utf8(&source[start..end]).expect("proof source is UTF-8");
        match expected_node {
            ExpectedProofIssueNode::Invariant => {
                assert!(
                    cited.starts_with(&format!("invariant {expected_name}: ")),
                    "PRF-1 cited {cited:?} instead of the complete invariant statement"
                );
                assert!(cited.ends_with('}'));
            }
            ExpectedProofIssueNode::Use {
                source: expected_source,
                occurrence,
            } => {
                assert_eq!(cited, expected_source);
                let text = std::str::from_utf8(source).expect("proof source is UTF-8");
                let owner_start = text
                    .find(&format!("invariant {expected_name}:"))
                    .expect("owning invariant exists");
                let expected_start = text[owner_start..]
                    .match_indices(expected_source)
                    .nth(occurrence)
                    .map(|(offset, _)| owner_start + offset)
                    .expect("expected use occurrence exists");
                assert_eq!((start, end), (expected_start, expected_start + cited.len()));
            }
        }
    });
}

/// AUTO owns the old two-premise shape. This preserves the OP-2 consumer
/// coverage while separately letting the certificate test below exercise the
/// first shape that actually needs written `use` steps.
#[test]
fn an_automatic_pair_invariant_discharges_op2_after_a_middle_write() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8, replacement: own u8) -> result: own u8 pure contract {{
  requires x <= middle;
  requires middle <= 254_u8;
}} {{
  invariant upper_bound: x <= 254_u8;
  set middle = replacement;
  let result = x + 1_u8;
  return result;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the automatic pair invariant must discharge OP-2: {outcome:?}");
        };
        let increment = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "increment")
            .expect("increment function exists");
        let addition = increment
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("the exact addition retains one OP-2 obligation");
        assert!(addition.discharged);
    });
}

#[test]
fn an_explicit_three_premise_invariant_survives_source_writes() {
    let source = format!(
        r#"fn preserve(a: own u64, a_limit: own u64, b: own u64, b_limit: own u64, c: own u64, c_limit: own u64, replacement: own u64) -> result: own unit pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  let left = a;
  let left_limit = a_limit;
  let middle = b;
  let middle_limit = b_limit;
  let right = c;
  let right_limit = c_limit;
  invariant total: left + middle + right <= left_limit + middle_limit + right_limit {{
    use left <= left_limit;
    use middle <= middle_limit;
    use right <= right_limit;
  }}
  set a = replacement;
  set a_limit = replacement;
  set b = replacement;
  set b_limit = replacement;
  set c = replacement;
  set c_limit = replacement;
  invariant retained_scaled: 3_u64 * left + 3_u64 * middle + 3_u64 * right <= 3_u64 * left_limit + 3_u64 * middle_limit + 3_u64 * right_limit {{
    use 3 * total;
  }}
  for (
    seed in 0_u64..0_u64,
    invariant retained: 3_u64 * left + 3_u64 * middle + 3_u64 * right <= 3_u64 * left_limit + 3_u64 * middle_limit + 3_u64 * right_limit
  ) {{
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "the explicit three-premise invariant must survive the source writes: {outcome:?}"
            );
        };
        let preserve = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve")
            .expect("preserve function exists");
        super::entailment::validate_derivations(&preserve.entailment);

        let [total, retained_scaled] = preserve.entailment.source_proofs.as_slice() else {
            panic!("preserve retains the original and post-write source proofs");
        };
        assert_eq!(total.name, "total");
        assert_eq!(total.check.premises, [true, true, true]);
        assert!(total.check.combination);
        assert!(total.check.discharged());
        assert_eq!(retained_scaled.name, "retained_scaled");
        assert_eq!(retained_scaled.check.premises, [true]);
        assert!(retained_scaled.check.combination);
        assert!(retained_scaled.check.discharged());
        let retained = preserve
            .entailment
            .loop_invariants
            .iter()
            .find(|invariant| invariant.name == "retained")
            .expect("the post-write loop invariant exists");
        assert!(retained.proof.base);
    });
}

#[test]
fn the_first_unproved_use_is_reported_in_source_order() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires middle <= 254_u8;
}} {{
  invariant upper_bound: x <= 254_u8 {{
    use x <= middle;
    use middle <= 254_u8;
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::Premise(0),
        ExpectedProofIssueNode::Use {
            source: "use x <= middle;",
            occurrence: 0,
        },
    );
}

#[test]
fn the_second_unproved_use_is_reported_in_source_order() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires x <= middle;
}} {{
  invariant upper_bound: x <= 254_u8 {{
    use x <= middle;
    use middle <= 254_u8;
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::Premise(1),
        ExpectedProofIssueNode::Use {
            source: "use middle <= 254_u8;",
            occurrence: 0,
        },
    );
}

#[test]
fn proved_premises_cannot_strengthen_their_written_sum() {
    let source = format!(
        r#"fn increment(x: own u8, middle: own u8) -> result: own u8 pure contract {{
  requires x <= middle;
  requires middle <= 254_u8;
}} {{
  invariant upper_bound: x <= 253_u8 {{
    use x <= middle;
    use middle <= 254_u8;
  }}
  return x;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::Combination,
        ExpectedProofIssueNode::Invariant,
    );
}

#[test]
fn proved_premises_may_weaken_their_written_sum_deterministically() {
    let source = format!(
        r#"fn retain(a: own u64, a_limit: own u64, b: own u64, b_limit: own u64, c: own u64, c_limit: own u64) -> result: own unit pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  invariant upper_bound: a + b + c <= a_limit + b_limit + c_limit + 1_u64 {{
    use a <= a_limit;
    use b <= b_limit;
    use c <= c_limit;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the fixed residual check must admit the weaker target: {outcome:?}");
        };
        let retain = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "retain")
            .expect("retain function exists");
        let [proof] = retain.entailment.source_proofs.as_slice() else {
            panic!("retain has one source proof");
        };
        assert_eq!(proof.check.premises, [true, true, true]);
        assert!(proof.check.combination);
        assert!(proof.check.discharged());
    });
}

/// The two branches write the same canonical inequality in different forms.
/// One uses the expression directly; the other names its exact affine value
/// with a local binder. Numeric intersection keeps the fact, while the joined
/// source identity records both real proof statements for diagnostics.
#[test]
fn equivalent_expression_and_binder_proofs_survive_a_branch_join() {
    let source = format!(
        r#"fn need(value: own u32, limit: own u32) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  return unit;
}}

fn combine(flag: own Bool, a: own u8, a_limit: own u8, b: own u8, b_limit: own u8, c: own u8, c_limit: own u8) -> result: own u32 pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  let left = cvt::<u8, u32>(a);
  let left_limit = cvt::<u8, u32>(a_limit);
  let middle = cvt::<u8, u32>(b);
  let middle_limit = cvt::<u8, u32>(b_limit);
  let right = cvt::<u8, u32>(c);
  let right_limit = cvt::<u8, u32>(c_limit);
  let first_sum = left + middle;
  let total = first_sum + right;
  let first_limit_sum = left_limit + middle_limit;
  let total_limit = first_limit_sum + right_limit;
  if flag {{
    invariant expression_form: left + middle + right <= left_limit + middle_limit + right_limit {{
      use left <= left_limit;
      use middle <= middle_limit;
      use right <= right_limit;
    }}
  }} else {{
    invariant binder_form: total <= total_limit {{
      use left <= left_limit;
      use middle <= middle_limit;
      use right <= right_limit;
    }}
  }}
  need(value: total, limit: total_limit);
  return total;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("equivalent source facts must survive the branch join: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine function exists");
        super::entailment::validate_derivations(&combine.entailment);

        assert_eq!(combine.entailment.source_proofs.len(), 2);
        assert!(
            combine
                .entailment
                .source_proofs
                .iter()
                .all(|proof| proof.check.discharged())
        );
        let [joined] = combine.entailment.joined_source_proofs.as_slice() else {
            panic!("the branch join retains one diagnostic provenance node");
        };
        assert_eq!(
            joined.predecessors.as_ref(),
            [
                SourceAffineFactRef::SourceProof { source_ordinal: 0 },
                SourceAffineFactRef::SourceProof { source_ordinal: 1 },
            ]
        );
        assert!(combine.entailment.derivations.nodes.iter().any(|node| {
            matches!(
                node,
                DerivationNode::AffineConsequence { premises, .. }
                    if premises.iter().any(|premise| premise.source
                        == SourceAffineFactRef::JoinedSourceProof { join_ordinal: 0 })
            )
        }));
    });
}

#[test]
fn a_common_source_proof_reference_is_reused_across_a_branch_join() {
    let source = format!(
        r#"fn need(value: own u32, limit: own u32) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  return unit;
}}

fn combine(flag: own Bool, a: own u8, a_limit: own u8, b: own u8, b_limit: own u8, c: own u8, c_limit: own u8) -> result: own u32 pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  let left = cvt::<u8, u32>(a);
  let left_limit = cvt::<u8, u32>(a_limit);
  let middle = cvt::<u8, u32>(b);
  let middle_limit = cvt::<u8, u32>(b_limit);
  let right = cvt::<u8, u32>(c);
  let right_limit = cvt::<u8, u32>(c_limit);
  let first_sum = left + middle;
  let total = first_sum + right;
  let first_limit_sum = left_limit + middle_limit;
  let total_limit = first_limit_sum + right_limit;
  invariant common_bound: total <= total_limit {{
    use left <= left_limit;
    use middle <= middle_limit;
    use right <= right_limit;
  }}
  if flag {{
    let marker = 0_u32;
  }} else {{
    let marker = 1_u32;
  }}
  need(value: total, limit: total_limit);
  return total;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the common source fact must survive unchanged: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine function exists");
        super::entailment::validate_derivations(&combine.entailment);
        assert!(combine.entailment.joined_source_proofs.is_empty());
        assert!(combine.entailment.derivations.nodes.iter().any(|node| {
            matches!(
                node,
                DerivationNode::AffineConsequence { premises, .. }
                    if premises.iter().any(|premise| premise.source
                        == SourceAffineFactRef::SourceProof { source_ordinal: 0 })
            )
        }));
    });
}

#[test]
fn different_canonical_inequalities_do_not_merge_at_a_branch_join() {
    let source = format!(
        r#"fn combine(flag: own Bool, a: own u8, b: own u8, c: own u8, x: own u8, p: own u8, q: own u8) -> result: own u8 pure {{
  let a_wide = cvt::<u8, u16>(a);
  let b_wide = cvt::<u8, u16>(b);
  let c_wide = cvt::<u8, u16>(c);
  let x_wide = cvt::<u8, u16>(x);
  let p_wide = cvt::<u8, u16>(p);
  let q_wide = cvt::<u8, u16>(q);
  let first_link = a_wide + x_wide;
  let second_link = b_wide + p_wide;
  let third_link = c_wide + q_wide;
  let ceiling_link = 255_u16 + x_wide;
  let first_holds = first_link <= p_wide;
  if first_holds {{
    let second_holds = second_link <= q_wide;
    if second_holds {{
      let third_holds = third_link <= ceiling_link;
      if third_holds {{
        if flag {{
          invariant exact: a_wide + b_wide + c_wide <= 255_u16 {{
            use a_wide + x_wide <= p_wide;
            use b_wide + p_wide <= q_wide;
            use c_wide + q_wide <= 255_u16 + x_wide;
          }}
        }} else {{
          invariant weaker: a_wide + b_wide + c_wide <= 256_u16 {{
            use a_wide + x_wide <= p_wide;
            use b_wide + p_wide <= q_wide;
            use c_wide + q_wide <= 255_u16 + x_wide;
          }}
        }}
        let first = a + b;
        let result = first + c;
        return result;
      }} else {{
        return 0_u8;
      }}
    }} else {{
      return 0_u8;
    }}
  }} else {{
    return 0_u8;
  }}
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("different canonical facts must not become one joined fact: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation { .. }
        ));
    });
}

/// A redundant earlier fact has the target's coefficient vector but an upper
/// bound weaker by one. Exhaustive pair checking must still find the two later
/// exact components; adding the weaker fact cannot turn acceptance into
/// rejection.
#[test]
fn an_earlier_weaker_fact_does_not_hide_a_later_automatic_pair() {
    let source = format!(
        r#"fn preserve(first: own u64, first_limit: own u64, second: own u64, second_limit: own u64, replacement: own u64) -> result: own unit pure contract {{
  requires first <= first_limit;
  requires second <= second_limit;
}} {{
  for (
    weak_seed in 0_u64..0_u64,
    invariant weaker_first: first + second <= first_limit + second_limit + 1_u64
  ) {{
  }}
  for (
    first_seed in 0_u64..0_u64,
    invariant first_part: first <= first_limit
  ) {{
  }}
  for (
    second_seed in 0_u64..0_u64,
    invariant second_part: second <= second_limit
  ) {{
  }}
  let left = first * 1_u64;
  let left_limit = first_limit * 1_u64;
  let right = second * 1_u64;
  let right_limit = second_limit * 1_u64;
  set first = replacement;
  set first_limit = replacement;
  set second = replacement;
  set second_limit = replacement;
  for (
    check_seed in 0_u64..0_u64,
    invariant combined: left + right <= left_limit + right_limit
  ) {{
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );

    // The old two-use certificate is retained as negative evidence: AUTO now
    // owns this exact pair, so spelling the same selection is a PRF-1 error.
    const REDUNDANT: &str = "  invariant exact_parts: left + right <= left_limit + right_limit {\n    use left <= left_limit;\n    use right <= right_limit;\n  }\n";
    let with_redundant = source.replacen(
        "  for (\n    check_seed",
        &format!("{REDUNDANT}  for (\n    check_seed"),
        1,
    );
    assert_prf1_issue_named(
        with_redundant.as_bytes(),
        SourceProofObligation::RedundantUseBlock,
        "exact_parts",
        ExpectedProofIssueNode::Invariant,
    );

    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the exhaustive automatic pair must establish the invariant base: {outcome:?}");
        };
        let preserve = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve")
            .expect("preserve function exists");
        assert!(preserve.entailment.source_proofs.is_empty());
        let combined = preserve
            .entailment
            .loop_invariants
            .iter()
            .find(|invariant| invariant.name == "combined")
            .expect("the target invariant is retained");
        assert!(combined.proof.base);
        assert!(combined.proof.discharged());
    });
}

/// One checked source fact may serve every later goal in its dominance region;
/// the checker does not rebuild a writer assertion or a per-consumer proof
/// channel.
#[test]
fn one_source_proof_fact_discharges_multiple_bounds_and_a_call_requirement() {
    let source = format!(
        r#"fn need(index: own u8) -> result: own unit pure contract {{
  requires index <= 254_u8;
}} {{
  return unit;
}}

fn read(values: own array<u8, 255>, first: own u8, first_limit: own u8, second: own u8, second_limit: own u8, third: own u8, third_limit: own u8) -> result: own u8 pure contract {{
  requires first <= first_limit;
  requires second <= second_limit;
  requires third <= third_limit;
  requires first_limit <= 80_u8;
  requires second_limit <= 80_u8;
  requires third_limit <= 93_u8;
}} {{
  invariant component_sum: first + second + third <= first_limit + second_limit + third_limit + 1_u8 {{
    use first <= first_limit;
    use second <= second_limit;
    use third <= third_limit;
  }}
  invariant limit_sum: first_limit + second_limit + third_limit <= 253_u8;
  invariant in_range: first + second + third <= 254_u8;
  let first_two = first + second;
  let index = first_two + third;
  let array_index = cvt::<u8, u64>(index);
  let loaded_first = values[array_index];
  let loaded_second = values[array_index];
  need(index: index);
  return loaded_second;
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
        r#"fn bounded(first: own u8, first_limit: own u8, second: own u8, second_limit: own u8, third: own u8, third_limit: own u8) -> result: own u8 pure contract {{
  requires first <= first_limit;
  requires second <= second_limit;
  requires third <= third_limit;
  requires first_limit <= 80_u8;
  requires second_limit <= 80_u8;
  requires third_limit <= 93_u8;
  ensures result <= 254_u8;
}} {{
  invariant component_sum: first + second + third <= first_limit + second_limit + third_limit + 1_u8 {{
    use first <= first_limit;
    use second <= second_limit;
    use third <= third_limit;
  }}
  invariant limit_sum: first_limit + second_limit + third_limit <= 253_u8;
  invariant total_bound: first + second + third <= 254_u8;
  let first_two = first + second;
  let result = first_two + third;
  return result;
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
fn assignment_does_not_rebind_a_source_proof_to_the_new_value() {
    let source = format!(
        r#"fn increment(first: own u8, first_limit: own u8, second: own u8, second_limit: own u8, third: own u8, third_limit: own u8, replacement: own u8) -> result: own u8 pure contract {{
  requires first <= first_limit;
  requires second <= second_limit;
  requires third <= third_limit;
  requires first_limit <= 80_u8;
  requires second_limit <= 80_u8;
  requires third_limit <= 93_u8;
}} {{
  invariant component_sum: first + second + third <= first_limit + second_limit + third_limit + 1_u8 {{
    use first <= first_limit;
    use second <= second_limit;
    use third <= third_limit;
  }}
  invariant limit_sum: first_limit + second_limit + third_limit <= 253_u8;
  invariant total_bound: first + second + third <= 254_u8;
  set first = replacement;
  let result = first + second;
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
        let component_sum = increment
            .entailment
            .source_proofs
            .iter()
            .find(|proof| proof.name == "component_sum")
            .expect("the pre-assignment source proof remains recorded");
        assert!(component_sum.check.discharged());
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
  requires x <= 0_u64;
}} {{
  for (
    seed in 0_u64..0_u64,
    invariant large_nonpositive: 9223372036854775808_u64 *(18446744073709551615_u64 * x) <= 0_u8
  ) {{
  }}
  let left = x;
  for (
    i in 0_u64..1_u64,
    invariant scaled_order: 18446744073709551615_u64 * left <= 18446744073709551615_u64 * x
  ) {{
    invariant carried_order: 18446744073709551615_u64 * left <= 18446744073709551615_u64 * x;
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
        assert!(proof.check.premises.is_empty());
        assert!(proof.check.combination);
    });
}

#[test]
fn three_written_uses_follow_the_certificate_when_auto_stops_at_two() {
    let source = format!(
        r#"fn combine(a: own u64, a_limit: own u64, b: own u64, b_limit: own u64, c: own u64, c_limit: own u64) -> result: own unit pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  invariant total: a + b + c <= a_limit + b_limit + c_limit {{
    use a <= a_limit;
    use b <= b_limit;
    use c <= c_limit;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("three explicit uses must discharge the non-AUTO target: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine function exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert_eq!(proof.check.premises, [true, true, true]);
        assert!(proof.check.combination);
        assert!(!proof.check.redundant);
        assert!(proof.check.discharged());
    });
}

/// The binary-search midpoint. The written sum proves `2*(mid - hi) <= -1`,
/// which over the integers is exactly `mid < hi`; without the integer
/// tightening the halved target is outside every fixed residual rule.
#[test]
fn a_midpoint_certificate_halves_its_doubled_sum_and_discharges_the_subscript() {
    let source = format!(
        r#"fn probe['t](table: &'t buffer<u8>, lo: own u64, hi: own u64) -> found: own u8 reads(table) contract {{
  define room = len(deref(table));
  requires lo < hi;
  requires hi <= room;
}} {{
  let span = hi - lo;
  let half = span / 2_u64;
  let mid = lo + half;
  invariant inside: mid < hi {{
    use lo < hi;
    use 2_u64 * half <= span;
  }}
  let byte = deref(table)[mid];
  return byte;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the halved midpoint certificate must discharge OP-4: {outcome:?}");
        };
        let probe = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "probe")
            .expect("probe function exists");
        let [proof] = probe.entailment.source_proofs.as_slice() else {
            panic!("probe retains one local certificate");
        };
        assert_eq!(proof.check.premises, [true, true]);
        assert!(proof.check.combination);
        assert!(!proof.check.redundant);
        assert!(proof.check.discharged());
        assert!(
            probe
                .entailment
                .obligations
                .iter()
                .all(|outcome| outcome.discharged)
        );
    });
}

/// A signed certificate whose doubled sum has the odd bound -5. The target
/// holds exactly at the mathematical floor -3 of -5/2; truncation toward zero
/// would stop at -2 and lose it.
#[test]
fn a_signed_certificate_floors_its_halved_bound_toward_negative_infinity() {
    let source = |slack: &str| {
        format!(
            r#"fn ordered(a: own i32, b: own i32, c: own i32, d: own i32, e: own i32, f: own i32) -> result: own unit pure contract {{
  requires a < b;
  requires c < d;
  requires e < f;
}} {{
  invariant doubled: 2_i32 * a + 1_i32 <= 2_i32 * b;
  invariant total: a + c + e + {slack}_i32 <= b + d + f {{
    use doubled;
    use 2 * (c < d);
    use 2 * (e < f);
  }}
  return unit;
}}

{COMMAND_MAIN}"#
        )
    };
    with_semantics(source("3").as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the floored halved bound must discharge the target: {outcome:?}");
        };
        let ordered = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "ordered")
            .expect("ordered function exists");
        let proof = ordered
            .entailment
            .source_proofs
            .iter()
            .find(|proof| proof.name == "total")
            .expect("the written certificate is retained");
        assert_eq!(proof.check.premises, [true, true, true]);
        assert!(proof.check.combination);
        assert!(!proof.check.redundant);
        assert!(proof.check.discharged());
    });
    assert_prf1_issue_named(
        source("4").as_bytes(),
        SourceProofObligation::Combination,
        "total",
        ExpectedProofIssueNode::Invariant,
    );
}

#[test]
fn an_auto_provable_target_rejects_its_whole_use_block_as_redundant() {
    let source = format!(
        r#"fn retain(value: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant upper_bound: value <= limit {{
    use value <= limit;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::RedundantUseBlock,
        ExpectedProofIssueNode::Invariant,
    );
}

#[test]
fn repeated_normalized_uses_require_one_explicit_multiplier() {
    let source = format!(
        r#"fn combine(value: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant upper_bound: 3_u64 * value <= 3_u64 * limit {{
    use value <= limit;
    use value <= limit;
    use value <= limit;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::RepeatedUse {
            first: 0,
            repeated: 1,
        },
        ExpectedProofIssueNode::Use {
            source: "use value <= limit;",
            occurrence: 1,
        },
    );
}

#[test]
fn use_capacity_cites_the_first_entry_beyond_the_admitted_prefix() {
    let written_use = "    use value <= limit;\n";
    let uses = written_use.repeat(MAX_CERTIFICATE_PREMISES + 1);
    let source = format!(
        "fn combine(value: own u64, limit: own u64, other: own u64, other_limit: own u64, final_value: own u64, final_limit: own u64) -> result: own unit pure {{\n  invariant upper_bound: value + other + final_value <= limit + other_limit + final_limit {{\n{uses}  }}\n  return unit;\n}}\n\n{COMMAND_MAIN}"
    );
    let maximum = u32::try_from(MAX_CERTIFICATE_PREMISES).expect("capacity fits u32");
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::UseCapacity {
            maximum,
            actual: maximum + 1,
        },
        ExpectedProofIssueNode::Use {
            source: written_use.trim(),
            occurrence: MAX_CERTIFICATE_PREMISES,
        },
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the capacity result: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert_eq!(proof.check.certificate_failure_use_index, Some(maximum));
        assert_eq!(proof.use_node_paths.len(), MAX_CERTIFICATE_PREMISES + 1);
    });
}

#[test]
fn explicit_factors_apply_to_relation_and_named_uses() {
    let source = format!(
        r#"fn relation_scale(value: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant scaled: 4_u64 * value <= 4_u64 * limit {{
    use 4 * (value <= limit);
  }}
  return unit;
}}

fn named_scale(value: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant unit_bound: value <= limit;
  invariant scaled: 4_u64 * value <= 4_u64 * limit {{
    use 4 * unit_bound;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("both explicit-factor source forms must check: {outcome:?}");
        };
        for (name, count) in [("relation_scale", 1), ("named_scale", 2)] {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("focused function exists");
            assert_eq!(function.entailment.source_proofs.len(), count);
            assert!(
                function
                    .entailment
                    .source_proofs
                    .iter()
                    .all(|proof| proof.check.discharged())
            );
        }
    });
}

#[test]
fn a_named_use_keeps_the_published_value_image_across_set() {
    let source = format!(
        r#"fn update(value: own u64, replacement: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant before: value <= limit;
  set value = replacement;
  invariant after: 4_u64 * value <= 4_u64 * limit {{
    use 4 * before;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the old theorem image must not rebind to the replacement: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Prf1);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedSourceProof {
                name,
                obligation: SourceProofObligation::Combination,
                ..
            } if name == "after"
        ));
    });
}

#[test]
fn all_ordered_invariant_roots_normalize_to_their_written_direction() {
    let source = format!(
        r#"fn ordered(a: own i32, b: own i32, c: own i32, d: own i32, e: own i32, f: own i32, g: own i32, h: own i32) -> result: own unit pure contract {{
  requires a <= b;
  requires c < d;
  requires e >= f;
  requires g > h;
}} {{
  invariant le: a <= b;
  invariant lt: c < d;
  invariant ge: e >= f;
  invariant gt: g > h;
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("all four ordered roots must retain their exact direction: {outcome:?}");
        };
        let ordered = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "ordered")
            .expect("ordered function exists");
        assert_eq!(ordered.entailment.source_proofs.len(), 4);
        assert!(
            ordered
                .entailment
                .source_proofs
                .iter()
                .all(|proof| proof.check.discharged())
        );
    });
}

#[test]
fn an_explicit_factor_one_is_not_canonical_source() {
    let source = format!(
        r#"fn scale(value: own u64, limit: own u64) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  invariant scaled: 4_u64 * value <= 4_u64 * limit {{
    use 1 * (value <= limit);
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an explicit factor one must reject canonically: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Prf1);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::InvalidSourceProof {
                reason: "an explicitly written use multiplier one is not canonical",
                ..
            }
        ));
    });
}

#[test]
fn a_composite_requirement_uses_affine_invariant_leaves() {
    let source = format!(
        r#"fn need(value: own u32, limit: own u32, enabled: own Bool) -> result: own unit pure contract {{
  define ordered = value <= limit;
  define accepted = band(ordered, enabled);
  requires accepted;
}} {{
  return unit;
}}

fn caller(enabled: own Bool, a: own u8, a_limit: own u8, b: own u8, b_limit: own u8, c: own u8, c_limit: own u8) -> result: own unit pure contract {{
  requires enabled;
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  let left = cvt::<u8, u32>(a);
  let left_limit = cvt::<u8, u32>(a_limit);
  let middle = cvt::<u8, u32>(b);
  let middle_limit = cvt::<u8, u32>(b_limit);
  let right = cvt::<u8, u32>(c);
  let right_limit = cvt::<u8, u32>(c_limit);
  let first_sum = left + middle;
  let value = first_sum + right;
  let first_limit_sum = left_limit + middle_limit;
  let limit = first_limit_sum + right_limit;
  invariant total: value <= limit {{
    use left <= left_limit;
    use middle <= middle_limit;
    use right <= right_limit;
  }}
  let called = need(value: value, limit: limit, enabled: enabled);
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "the affine relation and ordinary Boolean fact must prove the conjunction: {outcome:?}"
            );
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller exists");
        super::entailment::validate_derivations(&caller.entailment);
        assert_eq!(caller.entailment.call_goals.len(), 1);
        assert_eq!(
            caller.entailment.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            caller.entailment.call_goals[0].evidence,
            [CallGoalEvidence::AffinePositive]
        );
        assert!(caller.entailment.derivations.nodes.iter().any(|node| {
            matches!(
                node,
                DerivationNode::BooleanIntroduction { parents, .. } if parents.len() == 2
            )
        }));
    });
}

/// A predecessor that closes to L0 contradiction contributes no runtime state
/// to a join. In particular, its write cannot erase the live predecessor's
/// exact value image, published name, or canonical affine theorem.
#[test]
fn a_contradictory_predecessor_is_neutral_to_an_affine_join() {
    let source = format!(
        r#"fn need(value: own u32, limit: own u32) -> result: own unit pure contract {{
  requires value <= limit;
}} {{
  return unit;
}}

fn retain(a: own u8, a_limit: own u8, b: own u8, b_limit: own u8, c: own u8, c_limit: own u8) -> result: own unit pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  let left = cvt::<u8, u32>(a);
  let left_limit = cvt::<u8, u32>(a_limit);
  let middle = cvt::<u8, u32>(b);
  let middle_limit = cvt::<u8, u32>(b_limit);
  let right = cvt::<u8, u32>(c);
  let right_limit = cvt::<u8, u32>(c_limit);
  let first_sum = left + middle;
  let value = first_sum + right;
  let first_limit_sum = left_limit + middle_limit;
  let limit = first_limit_sum + right_limit;
  if a < a {{
    set value = 0_u32;
  }} else {{
    invariant total: value <= limit {{
      use left <= left_limit;
      use middle <= middle_limit;
      use right <= right_limit;
    }}
  }}
  need(value: value, limit: limit);
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the contradictory predecessor must be neutral: {outcome:?}");
        };
        let retain = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "retain")
            .expect("retain exists");
        super::entailment::validate_derivations(&retain.entailment);
        let [call] = retain.entailment.call_goals.as_slice() else {
            panic!("retain has one requirement call");
        };
        assert_eq!(call.disposition, CallGoalDisposition::Discharged);
        assert_eq!(call.evidence, [CallGoalEvidence::AffinePositive]);
    });
}

#[test]
fn an_unpublished_named_header_source_is_not_reproved_by_a_later_guard() {
    let source = format!(
        r#"fn named_source(value: own u64, limit: own u64) -> result: own unit pure {{
  loop (
    invariant header_bound: value <= limit
  ) {{
    if value <= limit {{
      invariant scaled_named: 3_u64 * value <= 3_u64 * limit {{
        use 3 * header_bound;
      }}
    }}
    break;
  }}
  return unit;
}}

fn relation_source(value: own u64, limit: own u64) -> result: own unit pure {{
  if value <= limit {{
    invariant scaled_relation: 3_u64 * value <= 3_u64 * limit {{
      use 3 * (value <= limit);
    }}
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain both source forms: {outcome:?}");
        };

        let named = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "named_source")
            .expect("named_source exists");
        let named = named
            .entailment
            .source_proofs
            .iter()
            .find(|proof| proof.name == "scaled_named")
            .expect("the named certificate is retained");
        assert!(named.check.source_failure.is_none());
        assert!(named.check.certificate_failure.is_none());
        assert_eq!(named.check.premises, [false]);
        assert_eq!(named.check.first_unproved_premise, Some(0));
        assert_eq!(named.use_node_paths.len(), 1);
        assert!(!named.check.discharged());

        let relation = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "relation_source")
            .expect("relation_source exists");
        let relation = relation
            .entailment
            .source_proofs
            .iter()
            .find(|proof| proof.name == "scaled_relation")
            .expect("the relation certificate is retained");
        assert_eq!(relation.check.premises, [true]);
        assert_eq!(relation.check.first_unproved_premise, None);
        assert!(relation.check.combination);
        assert!(relation.check.discharged());
    });
}

#[test]
fn repeated_unpublished_named_uses_retain_the_structural_failure() {
    let source = format!(
        r#"fn combine(value: own u64, limit: own u64) -> result: own unit pure {{
  loop (
    invariant header_bound: value <= limit
  ) {{
    invariant scaled: 3_u64 * value <= 3_u64 * limit {{
      use header_bound;
      use 2 * header_bound;
    }}
    break;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the duplicate result: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert!(proof.check.source_failure.is_none());
        assert_eq!(proof.check.premises, [false, false]);
        assert_eq!(
            proof.check.certificate_failure,
            Some(SourceProofCertificateFailure::RepeatedUse {
                first: 0,
                repeated: 1,
            })
        );
        assert_eq!(proof.check.certificate_failure_use_index, Some(1));
        assert!(!proof.check.discharged());
    });
}

#[test]
fn an_unpublished_named_use_does_not_hide_scaled_sum_overflow() {
    let source = format!(
        r#"fn combine(value: own u64, limit: own u64) -> result: own unit pure {{
  loop (
    invariant doubled: 2_u64 * value <= 2_u64 * limit
  ) {{
    invariant scaled: 2_u64 * value <= 2_u64 * limit {{
      use 170141183460469231731687303715884105727 * doubled;
    }}
    break;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the arithmetic result: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert!(proof.check.source_failure.is_none());
        assert_eq!(proof.check.premises, [false]);
        assert_eq!(
            proof.check.certificate_failure,
            Some(SourceProofCertificateFailure::ArithmeticOverflow)
        );
        assert_eq!(proof.check.certificate_failure_use_index, Some(0));
        assert!(!proof.check.discharged());
    });
}

#[test]
fn source_order_sum_overflow_cites_the_use_that_triggers_it() {
    let source = format!(
        r#"fn combine(a: own u64, a_limit: own u64, b: own u64, b_limit: own u64, c: own u64, c_limit: own u64) -> result: own unit pure contract {{
  requires a <= a_limit;
  requires b <= b_limit;
  requires c <= c_limit;
}} {{
  invariant upper_bound: a + b + c <= a_limit + b_limit + c_limit {{
    use a <= a_limit;
    use 170141183460469231731687303715884105727 * (2_u64 * b <= 2_u64 * b_limit);
    use c <= c_limit;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue(
        source.as_bytes(),
        SourceProofObligation::CertificateArithmeticOverflow,
        ExpectedProofIssueNode::Use {
            source: "use 170141183460469231731687303715884105727 * (2_u64 * b <= 2_u64 * b_limit);",
            occurrence: 0,
        },
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the indexed arithmetic result: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert_eq!(
            proof.check.certificate_failure,
            Some(SourceProofCertificateFailure::ArithmeticOverflow)
        );
        assert_eq!(proof.check.certificate_failure_use_index, Some(1));
        assert_eq!(proof.use_node_paths.len(), 3);
    });
}

#[test]
fn an_unpublished_named_use_does_not_stop_later_duplicate_detection() {
    let source = format!(
        r#"fn combine(value: own u64, limit: own u64, part: own u64, part_limit: own u64) -> result: own unit pure {{
  loop (
    invariant header_bound: value <= limit
  ) {{
    invariant combined: value + 2_u64 * part <= limit + 2_u64 * part_limit {{
      use header_bound;
      use part <= part_limit;
      use part <= part_limit;
    }}
    break;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the later duplicate result: {outcome:?}");
        };
        let combine = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "combine")
            .expect("combine exists");
        let [proof] = combine.entailment.source_proofs.as_slice() else {
            panic!("combine retains one local certificate");
        };
        assert!(proof.check.source_failure.is_none());
        assert_eq!(proof.check.premises, [false, false, false]);
        assert_eq!(
            proof.check.certificate_failure,
            Some(SourceProofCertificateFailure::RepeatedUse {
                first: 1,
                repeated: 2,
            })
        );
        assert_eq!(proof.check.certificate_failure_use_index, Some(2));
        assert!(!proof.check.discharged());
    });
}

#[test]
fn current_value_image_overflow_precedes_redundant_block_detection() {
    let source = format!(
        r#"fn expand(value: own u64) -> result: own unit pure contract {{
  requires value <= 0_u64;
}} {{
  let scaled = 18446744073709551615_u64 * value;
  invariant unchanged: scaled <= scaled {{
    use 18446744073709551615_u64 * scaled <= 0_u64;
  }}
  return unit;
}}

{COMMAND_MAIN}"#
    );
    assert_prf1_issue_named(
        source.as_bytes(),
        SourceProofObligation::CertificateArithmeticOverflow,
        "unchanged",
        ExpectedProofIssueNode::Use {
            source: "use 18446744073709551615_u64 * scaled <= 0_u64;",
            occurrence: 0,
        },
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark analysis must retain the source-formation result: {outcome:?}");
        };
        let expand = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "expand")
            .expect("expand exists");
        let [proof] = expand.entailment.source_proofs.as_slice() else {
            panic!("expand retains one local certificate");
        };
        assert_eq!(
            proof.check.source_failure,
            Some(SourceProofCertificateFailure::ArithmeticOverflow)
        );
        assert_eq!(proof.check.source_failure_use_index, Some(0));
        assert!(!proof.check.discharged());
    });
}
