use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, lower_checked};

use super::super::entailment::{
    ClaimComponentFact, ClaimMaskedDisposition, ClaimTerminalOwner, ClaimTerminalRoot,
    DerivationRootKind, ProofView, Relation,
};
use super::super::model::{CheckedNominalKind, CheckedType, IntegerType};

use super::with_semantics;

/// Structurally valid review data used only in fixtures whose source must be
/// rejected before it could become an owner-approved theorem.
const REJECTION_REVIEW: &str = "premises: this negative fixture supplies a structurally complete review record\\nderivation: the tested machine judgment rejects before external theorem approval\\nconclusion: this occurrence must not enter a checked program\\nchecker gap: this field is present only to reach the rejection under test\\nconsumers: no approved program consumes this negative fixture";

const CLAMP_LT_EIGHT_REVIEW: &str = "premises: values has length 8 and bounded is returned by clamp_seven, whose body computes imin(index, 7_u64)\\nderivation: bounded is at most 7_u64 and therefore strictly less than size\\nconclusion: ilt(bounded, size) is true\\nchecker gap: ENT does not publish an uncontracted user-call result bound\\nconsumers: the following length-eight array subscript uses bounded";

const PAIR_CLAMP_REVIEW: &str = "premises: table has length 8, and low_bounded and high_bounded are returned by clamp_seven for their respective inputs\\nderivation: both results are at most 7_u64 and therefore strictly below size\\nconclusion: band(low_ok, high_ok) is true\\nchecker gap: ENT does not publish either uncontracted user-call result bound\\nconsumers: the two following array subscripts use the respective bounded values";

const GENERIC_MAX_NONNEGATIVE_REVIEW: &str = "premises: nonnegative is imax(value, 0_T) for an integer type T\\nderivation: imax returns an operand no smaller than 0_T\\nconclusion: ige(nonnegative, 0_T) is true\\nchecker gap: ENT does not derive the result range of imax for GenericInt\\nconsumers: the following FN-8 requirement needs nonnegative at least 0_T";

fn assert_source_rule(source: &[u8], expected: SemanticRule) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {expected:?} source rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), expected);
    });
}

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "expected a checked proof residual, got {outcome:?}"
        );
    });
}

#[test]
fn a_structured_unknown_load_bearing_claim_remains_an_executed_residual() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {{
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  claim in_range: inside because "{CLAMP_LT_EIGHT_REVIEW}";
  return values[bounded];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a genuine residual must remain accepted: {outcome:?}");
        };
        let read = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("read function");
        assert_eq!(read.entailment.claims.len(), 1);
        let entry = program
            .data
            .claim_ledger
            .entries
            .iter()
            .find(|entry| entry.name == "in_range")
            .expect("retained claim ledger entry");
        assert_eq!(entry.source.declaration, read.declaration);
        assert_eq!(entry.source.function, read.id);
        let proof = entry.proof.as_ref().expect("concrete claim proof packet");
        assert_ne!(proof.images.direct, proof.images.expanded);
        assert_eq!(proof.components.len(), 1);
        assert_eq!(proof.components[0].ordinal, 0);
        assert_eq!(entry.residual_witnesses.len(), 2);
        assert_eq!(entry.residual_witnesses[0].component, Some(0));
        assert_eq!(entry.residual_witnesses[1].component, None);
        assert_eq!(entry.uses.len(), 1);
        let used = &entry.uses[0];
        assert!(used.query_noncontradictory);
        assert!(used.non_explosive);
        assert_eq!(used.component_premises.len(), 1);
        assert_eq!(used.component_premises[0].component, 0);
        assert!(matches!(used.root, DerivationRootKind::BoundsObligation(_)));
        assert!(matches!(
            used.terminal,
            ClaimTerminalRoot::Obligation {
                owner: ClaimTerminalOwner::Concrete(owner),
                ..
            } if owner == read.id
        ));
        assert!(
            entry
                .residual_witnesses
                .iter()
                .all(|witness| matches!(witness.masked, ClaimMaskedDisposition::Obligation { .. }))
        );
    });
}

#[test]
fn local_array_construction_does_not_hide_a_length_relation_claim_component() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn read(index: own u64) -> result: own u8 traps {
  let values = array_new<u8, 4>(0_u8);
  let bounded = clamp_three(value: index);
  let room = len(values);
  let inside = ilt(bounded, room);
  claim in_range: inside because "premises: values has length 4 and bounded is returned by clamp_three, whose body computes imin(index, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than room\nconclusion: ilt(bounded, room) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following values[bounded] subscript requires this exact bound";
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn expanding_a_conversion_origin_does_not_hide_a_load_bearing_relation_component() {
    let source = br#"fn clamp_three(value: own u8) -> result: own u8 pure {
  return imin(value, 3_u8);
}

fn read(value: own u8) -> result: own u8 traps {
  let values = array_new<u8, 4>(0_u8);
  let small = clamp_three(value: value);
  let wide = cvt<u8, u64>(small);
  let inside = ilt(wide, 4_u64);
  claim in_range: inside because "premises: small is returned by clamp_three, whose body computes imin(value, 3_u8), and cvt<u8, u64> preserves that nonnegative value\nderivation: wide is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: ilt(wide, 4_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound through conversion\nconsumers: the following values[wide] subscript requires this exact bound";
  return values[wide];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn expanding_a_conversion_origin_cannot_hide_a_checker_known_component_in_a_bundle() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(small: own u8, index: own u64) -> result: own unit traps {{
  let wide = cvt<u8, u64>(small);
  let known = ilt(wide, 256_u64);
  let bounded = clamp_three(value: index);
  let residual = ilt(bounded, 4_u64);
  let both = band(known, residual);
  claim bundled: both because "{REJECTION_REVIEW}";
  need(flag: both);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_claim_component_uses_the_checked_snapshot_leaf_not_a_mutated_origin() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {
  let source = clamp_three(value: input);
  let snapshot = source;
  let inside = ilt(snapshot, 4_u64);
  claim in_range: inside because "premises: source is returned by clamp_three, whose body computes imin(input, 3_u64), and snapshot copies source before its later write\nderivation: snapshot is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: ilt(snapshot, 4_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following values[snapshot] subscript requires this exact snapshot bound";
  set source = 100_u64;
  return values[snapshot];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn the_support_snapshot_is_an_exact_claim_lifecycle_image() {
    let source = format!(
        r#"fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u64) -> result: own unit traps {{
  let snapshot = ixor(input, 123_u64);
  let different = ine(snapshot, 42_u64);
  claim first: different because "{REJECTION_REVIEW}";
  need(flag: different);
  let same = ieq(snapshot, 42_u64);
  claim second: same because "{REJECTION_REVIEW}";
  need(flag: same);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the support-snapshot negation must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::RefutedClaim(detail) = issue.kind() else {
            panic!(
                "the exact support image must produce a refutation: {:?}",
                issue.kind()
            );
        };
        assert_eq!(detail.name, "second");
    });
}

#[test]
fn opposite_signs_across_exact_lifecycle_images_make_the_claim_path_vacuous() {
    let source = format!(
        r#"fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u8) -> result: own unit traps {{
  let first_value = ixor(input, 123_u8);
  let inside = ilt(first_value, 4_u8);
  claim first: inside because "{REJECTION_REVIEW}";
  need(flag: inside);
  let second_value = ixor(input, 123_u8);
  let second_inside = ilt(second_value, 4_u8);
  let outside = bnot(second_inside);
  if outside {{
    claim repeated: inside because "{REJECTION_REVIEW}";
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("opposite exact images must make the claim path vacuous: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a vacuous-claim payload: {:?}", issue.kind());
        };
        assert_eq!(detail.name, "repeated");
        assert_eq!(detail.classification, "vacuous");
    });
}

#[test]
fn complete_structural_expansion_still_finds_a_repeated_computation_overlap() {
    let source = format!(
        r#"fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {{
  let first = ixor(input, 123_u64);
  let first_inside = ilt(first, 4_u64);
  if first_inside {{
    let second = ixor(input, 123_u64);
    let second_inside = ilt(second, 4_u64);
    claim overlap: second_inside because "{REJECTION_REVIEW}";
    return values[second];
  }} else {{
    return 0_u8;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_structurally_known_leaf_cannot_hide_inside_a_partly_residual_bundle() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u64, other: own u64) -> result: own unit traps {{
  let first = ixor(input, 123_u64);
  let first_inside = ilt(first, 4_u64);
  if first_inside {{
    let second = ixor(input, 123_u64);
    let known = ilt(second, 4_u64);
    let bounded = clamp_three(value: other);
    let residual = ilt(bounded, 4_u64);
    let both = band(known, residual);
    claim bundled: both because "{REJECTION_REVIEW}";
    need(flag: both);
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_structurally_known_defined_leaf_maps_to_each_frontier_component() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own i8, other: own u64) -> result: own unit traps {{
  let first = ixor(input, 123_i8);
  let first_ok = first *defined 2_i8;
  if first_ok {{
    let second = ixor(input, 123_i8);
    let known = second *defined 2_i8;
    let bounded = clamp_three(value: other);
    let residual = ilt(bounded, 4_u64);
    let both = band(known, residual);
    claim bundled: both because "{REJECTION_REVIEW}";
    need(flag: both);
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_structurally_known_equality_maps_to_each_directed_bound_component() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u64, target: own u64, other: own u64) -> result: own unit traps {{
  let first = ixor(input, 123_u64);
  let first_equal = ieq(first, target);
  if first_equal {{
    let second = ixor(input, 123_u64);
    let known = ieq(second, target);
    let bounded = clamp_three(value: other);
    let residual = ilt(bounded, 4_u64);
    let both = band(known, residual);
    claim bundled: both because "{REJECTION_REVIEW}";
    need(flag: both);
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn opposite_equivalent_component_manifestations_make_the_claim_path_vacuous() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u64, other: own u64) -> result: own unit traps {{
  let first = ixor(input, 123_u64);
  let first_inside = ilt(first, 4_u64);
  if first_inside {{
    let second = ixor(input, 123_u64);
    let second_outside = ige(second, 4_u64);
    if second_outside {{
      let inside = ilt(second, 4_u64);
      let bounded = clamp_three(value: other);
      let residual = ilt(bounded, 4_u64);
      let both = band(inside, residual);
      claim impossible_path: both because "{REJECTION_REVIEW}";
      need(flag: both);
    }}
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the claim-local contradiction must be a source rejection: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a vacuous-claim payload: {:?}", issue.kind());
        };
        assert_eq!(detail.classification, "vacuous");
    });
}

#[test]
fn claim_local_contradiction_precedes_an_earlier_component_overlap() {
    let source = format!(
        r#"fn clamp_three(value: own u64) -> result: own u64 pure {{
  return imin(value, 3_u64);
}}

fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(input: own u64, other: own u64) -> result: own unit traps {{
  let first = ixor(input, 123_u64);
  let first_inside = ilt(first, 4_u64);
  if first_inside {{
    let second = ixor(input, 123_u64);
    let second_outside = ige(second, 4_u64);
    if second_outside {{
      let already_known = True();
      let inside = ilt(second, 4_u64);
      let bounded = clamp_three(value: other);
      let residual = ilt(bounded, 4_u64);
      let tail = band(inside, residual);
      let all = band(already_known, tail);
      claim impossible_path: all because "{REJECTION_REVIEW}";
      need(flag: all);
    }}
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("claim-local contradiction must win over component order: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a vacuous-claim payload: {:?}", issue.kind());
        };
        assert_eq!(detail.classification, "vacuous");
    });
}

#[test]
fn a_defined_predicate_with_an_uncontracted_true_range_remains_a_residual() {
    let source = br#"fn reviewed_small(value: own i8) -> result: own i8 pure {
  let lower = imax(value, -10_i8);
  return imin(lower, 10_i8);
}

fn double(value: own i8) -> result: own i8 traps {
  let bounded = reviewed_small(value: value);
  let safe = bounded *defined 2_i8;
  claim product_is_defined: safe because "premises: bounded is returned by reviewed_small, whose body clamps value to the closed interval -10_i8 through 10_i8\nderivation: multiplying any integer in that interval by 2_i8 yields an i8 value in -20_i8 through 20_i8\nconclusion: bounded *defined 2_i8 is true\nchecker gap: ENT does not publish the range of an uncontracted user-call result\nconsumers: the following exact multiplication requires both signed product bounds";
  return bounded * 2_i8;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the reviewed product domain must remain residual: {outcome:?}");
        };
        let entry = program
            .data
            .claim_ledger
            .entries
            .iter()
            .find(|entry| entry.name == "product_is_defined")
            .expect("product-domain claim ledger entry");
        let proof = entry.proof.as_ref().expect("concrete proof packet");
        assert_eq!(proof.components.len(), 2);
        assert_eq!(
            entry
                .residual_witnesses
                .iter()
                .map(|witness| witness.component)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), None]
        );
    });
}

#[test]
fn a_claim_with_an_unstructured_because_string_is_a_clm1_error() {
    let source = br#"fn probe(flag: own Bool) -> result: own unit traps {
  claim malformed: flag because "trust me";
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_source_rule(source, SemanticRule::Clm1);
}

#[test]
fn literal_true_is_a_redundant_claim_source_error() {
    let source = format!(
        r#"fn probe() -> result: own unit traps {{
  claim redundant: True() because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn literal_false_is_a_refuted_claim_source_error() {
    let source = format!(
        r#"fn probe() -> result: own unit traps {{
  claim refuted: False() because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn an_unknown_claim_without_a_terminal_admission_consumer_is_rejected() {
    let source = format!(
        r#"fn probe(flag: own Bool) -> result: own unit traps {{
  claim unused_theorem: flag because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_user_call_cannot_hide_program_behavior_in_a_claim_predicate() {
    let source = format!(
        r#"fn theorem() -> result: own Bool pure {{
  return True();
}}

fn probe() -> result: own unit traps {{
  claim called: theorem() because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm1);
}

#[test]
fn a_checker_known_component_cannot_be_bundled_with_a_residual_component() {
    let source = format!(
        r#"fn need_both(left: own Bool, right: own Bool) -> result: own unit pure contract {{
  requires band(left, right);
}} {{
  return unit;
}}

fn probe(right: own Bool) -> result: own unit traps {{
  let left = True();
  let both = band(left, right);
  claim overlap: both because "{REJECTION_REVIEW}";
  need_both(left: left, right: right);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn every_unknown_conjunction_component_can_close_its_own_terminal_root() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn read_pair(table: own array<u8, 8>, low: own u64, high: own u64) -> result: own u8 traps {{
  let size = len(table);
  let low_bounded = clamp_seven(value: low);
  let high_bounded = clamp_seven(value: high);
  let low_ok = ilt(low_bounded, size);
  let high_ok = ilt(high_bounded, size);
  let both = band(low_ok, high_ok);
  claim pair_in_range: both because "{PAIR_CLAMP_REVIEW}";
  let first = table[low_bounded];
  let second = table[high_bounded];
  return second;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("both human-proved components must remain residual: {outcome:?}");
        };
        let function = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "read_pair")
            .expect("read_pair function");
        let entry = program
            .data
            .claim_ledger
            .entries
            .iter()
            .find(|entry| entry.name == "pair_in_range")
            .expect("pair claim entry");
        let proof = entry.proof.as_ref().expect("pair proof packet");
        assert_eq!(proof.components.len(), 2);
        assert_eq!(entry.residual_witnesses.len(), 3);
        assert_eq!(
            entry
                .residual_witnesses
                .iter()
                .map(|witness| witness.component)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), None]
        );
        assert_eq!(entry.uses.len(), 2);
        assert!(entry.uses.iter().any(|used| {
            used.component_premises
                .iter()
                .any(|premise| premise.component == 0)
        }));
        assert!(entry.uses.iter().any(|used| {
            used.component_premises
                .iter()
                .any(|premise| premise.component == 1)
        }));
        assert_eq!(
            function
                .entailment
                .derivations
                .roots
                .iter()
                .filter(|root| matches!(root.kind, DerivationRootKind::ClaimComponent { .. }))
                .count(),
            2
        );
        assert_eq!(
            function
                .entailment
                .derivations
                .roots
                .iter()
                .filter(|root| matches!(root.kind, DerivationRootKind::ClaimReconstruction { .. }))
                .count(),
            2
        );
    });
}

#[test]
fn duplicate_conjunction_members_have_one_normative_component_identity() {
    let source = r#"fn clamp_seven(value: own u64) -> result: own u64 pure {
  return imin(value, 7_u64);
}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  let repeated = band(inside, inside);
  claim in_range: repeated because "premises: values has length 8 and bounded is returned by clamp_seven, whose body computes imin(index, 7_u64)\nderivation: bounded is at most 7_u64, so both duplicate inside conjuncts are true\nconclusion: band(inside, inside) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following length-eight array subscript uses bounded";
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_complete(source.as_bytes());
}

#[test]
fn an_expanded_binding_origin_disappears_with_its_masked_component() {
    let source = r#"fn clamp_seven(value: own u64) -> result: own u64 pure {
  return imin(value, 7_u64);
}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  let copied = inside;
  claim in_range: copied because "premises: values has length 8, bounded is returned by clamp_seven, and copied is ilt(bounded, size)\nderivation: bounded is at most 7_u64 and therefore strictly below size\nconclusion: copied is true\nchecker gap: ENT expands copied but does not publish an uncontracted user-call result bound\nconsumers: the following array subscript uses bounded";
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_complete(source.as_bytes());
}

#[test]
fn an_independently_reproved_fact_makes_the_earlier_claim_non_residual() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {{
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  claim premature: inside because "{CLAMP_LT_EIGHT_REVIEW}";
  if inside {{
    return values[bounded];
  }} else {{
    return 0_u8;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_positive_disjunction_can_be_an_exact_fn8_residual() {
    let source = r#"fn need_either(flag: own Bool) -> result: own unit pure contract {
  requires flag;
} {
  return unit;
}

fn probe(index: own u64, right: own Bool) -> result: own unit traps {
  let bounded = imin(index, 7_u64);
  let left = ilt(bounded, 8_u64);
  let either = bor(left, right);
  claim reviewed_disjunction: either because "premises: bounded is imin(index, 7_u64), so left is true regardless of right\nderivation: a disjunction with the true left operand is true\nconclusion: bor(left, right) is true\nchecker gap: ENT preserves the exact disjunction but does not derive the result range of imin\nconsumers: the following FN-8 requirement needs the exact either predicate";
  need_either(flag: either);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_complete(source.as_bytes());
}

#[test]
fn one_residual_component_combines_with_a_checker_fact_for_an_exact_fn8_parent() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn need_both(known: own Bool, residual: own Bool) -> result: own unit pure contract {
  define both = band(known, residual);
  requires both;
} {
  return unit;
}

fn probe(input: own u64) -> result: own unit traps {
  let known = True();
  if known {
    let bounded = clamp_three(value: input);
    let residual = ilt(bounded, 4_u64);
    claim bounded_result: residual because "premises: bounded is returned by clamp_three, whose body computes imin(input, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: residual is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: need_both combines this residual with the branch-established true component in its exact conjunction requirement";
    need_both(known: known, residual: residual);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn xor_is_not_silently_treated_as_an_indivisible_residual() {
    let source = format!(
        r#"fn need_mixed(left: own Bool, right: own Bool) -> result: own unit pure contract {{
  requires bxor(left, right);
}} {{
  return unit;
}}

fn probe(left: own Bool, right: own Bool) -> result: own unit traps {{
  let mixed = bxor(left, right);
  claim unsupported_basis: mixed because "{REJECTION_REVIEW}";
  need_mixed(left: left, right: right);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn an_integer_equality_cannot_hide_one_checker_known_direction() {
    let source = format!(
        r#"fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {{
  requires ieq(left, right);
}} {{
  return unit;
}}

fn probe(left: own i32, right: own i32) -> result: own unit traps {{
  if ile(left, right) {{
    claim overlap: ieq(left, right) because "{REJECTION_REVIEW}";
    need_same(left: left, right: right);
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_claim_can_be_load_bearing_only_for_the_complete_fn9_postcondition() {
    let source = r#"fn reviewed_one(value: own i32) -> result: own i32 pure {
  let upper = imin(value, 1_i32);
  return imax(upper, 1_i32);
}

fn reviewed(value: own i32) -> result: own i32 traps contract {
  ensures ieq(result, 1_i32);
} {
  let normalized = reviewed_one(value: value);
  claim result_is_one: ieq(normalized, 1_i32) because "premises: normalized is returned by reviewed_one, whose body computes imax(imin(value, 1_i32), 1_i32)\nderivation: the inner minimum is at most 1_i32 and the outer maximum with 1_i32 is exactly 1_i32\nconclusion: ieq(normalized, 1_i32) is true\nchecker gap: ENT does not publish an uncontracted user-call result equality\nconsumers: the complete FN-9 selected-return proof needs result equal to 1_i32";
  return normalized;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the true imax theorem must support FN-9: {outcome:?}");
        };
        let entry = program
            .data
            .claim_ledger
            .entries
            .iter()
            .find(|entry| entry.name == "result_is_one")
            .expect("FN-9 claim ledger entry");
        assert_eq!(entry.uses.len(), 1);
        assert!(matches!(
            entry.uses[0].root,
            DerivationRootKind::PostconditionAggregate {
                view: ProofView::Complete,
                ..
            }
        ));
        assert!(matches!(
            entry.uses[0].terminal,
            ClaimTerminalRoot::Postcondition { .. }
        ));
        assert!(
            entry
                .residual_witnesses
                .iter()
                .all(|witness| matches!(witness.terminal, ClaimTerminalRoot::Postcondition { .. }))
        );
    });
}

#[test]
fn an_ordinary_admission_error_precedes_non_residuality() {
    let source = format!(
        r#"fn read(values: own buffer<i32>, i: own u64, theorem: own Bool) -> result: own i32 traps {{
  claim unrelated: theorem because "{REJECTION_REVIEW}";
  return values[i];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Op4);
}

#[test]
fn exact_claim_lifecycle_precedes_a_later_ordinary_admission_error() {
    let source = format!(
        r#"fn read(values: own buffer<i32>, i: own u64) -> result: own i32 traps {{
  claim redundant: True() because "{REJECTION_REVIEW}";
  return values[i];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_masked_actual_obligation_may_remove_later_root_records_without_internal_failure() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn consume(value: own u8, proof: own u8) -> result: own unit pure contract {{
  requires ige(proof, 0_u8);
}} {{
  return unit;
}}

fn read(values: own array<u8, 8>, index: own u64) -> result: own unit traps {{
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  claim in_range: inside because "{CLAMP_LT_EIGHT_REVIEW}";
  consume(value: values[bounded], proof: 0_u8);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn an_unreachable_claim_is_rejected_by_the_existing_fn1_body_rule() {
    let source = format!(
        r#"fn probe(flag: own Bool) -> result: own unit traps {{
  return unit;
  claim dead: flag because "{REJECTION_REVIEW}";
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Fn1);
}

#[test]
fn a_value_join_with_multiple_predicate_origins_is_not_an_opaque_residual() {
    let source = format!(
        r#"fn need(value: own Bool) -> result: own unit pure contract {{
  requires value;
}} {{
  return unit;
}}

fn probe(left: own Bool, right: own Bool, choose_left: own Bool) -> result: own unit traps {{
  let picked = if choose_left {{
    give left;
  }} else {{
    give right;
  }}
  claim ambiguous: picked because "{REJECTION_REVIEW}";
  need(value: picked);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn contradiction_precedes_ambiguous_origin_classification() {
    let source = format!(
        r#"fn probe(left: own Bool, right: own Bool, choose_left: own Bool) -> result: own unit traps {{
  if False() {{
    let picked = if choose_left {{
      give left;
    }} else {{
      give right;
    }}
    claim ambiguous: picked because "{REJECTION_REVIEW}";
  }}
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("a contradictory claim path must be rejected as vacuous: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a vacuous claim diagnostic: {:?}", issue.kind());
        };
        assert_eq!(detail.classification, "vacuous");
        assert_eq!(detail.component, None);
        assert_eq!(detail.reason, "the pre-claim state is contradictory");
    });
}

#[test]
fn an_unrelated_contradictory_obligation_does_not_poison_a_residual_witness() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {
  if False() {
    let unreachable = values[4_u64];
  }
  let bounded = clamp_three(value: input);
  let inside = ilt(bounded, 4_u64);
  claim in_range: inside because "premises: values has length 4 and bounded is returned by clamp_three, whose body computes imin(input, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: ilt(bounded, 4_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the final array subscript uses bounded";
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn an_unrelated_all_derivable_call_does_not_poison_a_residual_witness() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn need(flag: own Bool) -> result: own unit pure contract {
  requires flag;
} {
  return unit;
}

fn prove(input: own u64) -> result: own unit traps {
  let impossible = False();
  if False() {
    need(flag: impossible);
  }
  let bounded = clamp_three(value: input);
  let inside = ilt(bounded, 4_u64);
  claim in_range: inside because "premises: bounded is returned by clamp_three, whose body computes imin(input, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: inside is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the final need call requires inside";
  need(flag: inside);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn an_unrelated_explosive_postcondition_does_not_poison_a_residual_witness() {
    let source = br#"fn impossible_exit(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  if False() {
    return 0_i32;
  } else {
    return value;
  }
}

fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn read(values: own array<u8, 4>, input: own u64) -> result: own u8 traps {
  let bounded = clamp_three(value: input);
  let inside = ilt(bounded, 4_u64);
  claim in_range: inside because "premises: values has length 4 and bounded is returned by clamp_three, whose body computes imin(input, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: inside is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the final array subscript uses bounded";
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_later_masked_explosive_root_is_not_hidden_by_an_earlier_counterfactual_witness() {
    for consumers in [
        "  let first = five[bounded];\n  return ten[bounded];",
        "  let first = ten[bounded];\n  return five[bounded];",
    ] {
        let source = format!(
            r#"fn clamp_four(value: own u64) -> result: own u64 pure {{
  return imin(value, 4_u64);
}}

fn read(five: own array<u8, 5>, ten: own array<u8, 10>, input: own u64) -> result: own u8 traps {{
  let bounded = clamp_four(value: input);
  let weak = ilt(bounded, 10_u64);
  let always = True();
  if always {{
    if weak {{
    }} else {{
      return 0_u8;
    }}
  }}
  claim strong: ilt(bounded, 5_u64) because "premises: bounded is returned by clamp_four, whose body computes imin(input, 4_u64)\nderivation: bounded is at most 4_u64 and therefore strictly less than 5_u64\nconclusion: bounded is below five\nchecker gap: ENT does not publish the result bound of clamp_four\nconsumers: the five-element subscript consumes the strong bound";
{consumers}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
        );
        with_semantics(source.as_bytes(), |outcome| {
            assert!(
                matches!(
                    outcome,
                    SemanticOutcome::CompilerFailure {
                        failure: crate::SemanticCompilerFailure::InvalidResolution,
                    }
                ),
                "an explosive masked proof must fail independently of root order: {outcome:?}"
            );
        });
    }
}

#[test]
fn a_masked_component_is_non_residual_when_an_unmasked_component_reconstructs_it() {
    let source = r#"fn need(flag: own Bool) -> result: own unit pure contract {
  requires flag;
} {
  return unit;
}

fn clamp_four(value: own u64) -> result: own u64 pure {
  return imin(value, 4_u64);
}

fn probe(value: own u64) -> result: own unit traps {
  let bounded = clamp_four(value: value);
  let tight = ilt(bounded, 5_u64);
  let weak = ilt(bounded, 10_u64);
  let both = band(tight, weak);
  claim overcomplete: both because "premises: bounded is returned by clamp_four, whose body computes imin(value, 4_u64)\nderivation: bounded is at most 4_u64 and therefore strictly below both 5_u64 and 10_u64\nconclusion: band(tight, weak) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following FN-8 requirement names the exact conjunction";
  need(flag: both);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn residuality_uses_one_fixed_eligible_set_instead_of_selecting_survivors() {
    let source = r#"fn clamp_four(value: own u64) -> result: own u64 pure {
  return imin(value, 4_u64);
}

fn need_under_ten(value: own u64) -> result: own unit pure contract {
  requires ilt(value, 10_u64);
} {
  return unit;
}

fn prove(input: own u64) -> result: own unit traps {
  let bounded = clamp_four(value: input);
  claim under_seven: ilt(bounded, 7_u64) because "premises: bounded is returned by clamp_four, whose body computes imin(input, 4_u64)\nderivation: bounded is at most 4_u64 and therefore strictly below 7_u64\nconclusion: ilt(bounded, 7_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following FN-8 requirement can be discharged from this bound";
  claim under_five: ilt(bounded, 5_u64) because "premises: bounded is returned by clamp_four, whose body computes imin(input, 4_u64)\nderivation: bounded is at most 4_u64 and therefore strictly below 5_u64\nconclusion: ilt(bounded, 5_u64) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the following FN-8 requirement can be discharged from this stronger bound";
  need_under_ten(value: bounded);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_hidden_contradictory_join_predecessor_is_not_a_residual_witness() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {{
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  let always = True();
  if always {{
    claim joined: inside because "{CLAMP_LT_EIGHT_REVIEW}";
  }}
  return values[bounded];
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_claim_need_only_dominate_the_mutually_exclusive_lineage_that_consumes_it() {
    let source = format!(
        r#"fn clamp_seven(value: own u64) -> result: own u64 pure {{
  return imin(value, 7_u64);
}}

fn read(values: own array<u8, 8>, index: own u64, choose: own Bool) -> result: own u8 traps {{
  if choose {{
    let size = len(values);
    let bounded = clamp_seven(value: index);
    let inside = ilt(bounded, size);
    claim branch_only: inside because "{CLAMP_LT_EIGHT_REVIEW}";
    return values[bounded];
  }} else {{
    return 0_u8;
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_write_through_a_moved_unique_holder_kills_an_earlier_claim_fact() {
    let source = r#"fn overwrite['r](target: &uniq 'r u64) -> result: own unit writes('r) {
  set deref(target) = 100_u64;
  return unit;
}

fn clamp_seven(value: own u64) -> result: own u64 pure {
  return imin(value, 7_u64);
}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {
  let size = len(values);
  let bounded = clamp_seven(value: index);
  let inside = ilt(bounded, size);
  claim initially_inside: inside because "premises: bounded is returned by clamp_seven, whose body computes imin(index, 7_u64), and values has length 8\nderivation: bounded is at most 7_u64 and therefore strictly below size at the claim\nconclusion: ilt(bounded, size) is true at this statement\nchecker gap: ENT does not publish an uncontracted user-call result bound\nconsumers: the later subscript would consume this fact only if no intervening write killed it";
  region 'r {
    let holder = &uniq 'r bounded;
    overwrite<'r>(target: move holder);
  }
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_source_rule(source.as_bytes(), SemanticRule::Op4);
}

#[test]
fn a_copy_read_through_a_projected_affine_holder_is_a_valid_proof_predicate() {
    let source = r#"struct Holder {
  value: box<u64>;
}

fn clamped_holder(value: own u64) -> result: own Holder allocates(heap) {
  let bounded = imin(value, 7_u64);
  let owner = box_new(bounded);
  return Holder(value: move owner);
}

fn read(values: own array<u8, 8>, index: own u64) -> result: own u8 allocates(heap), traps {
  let size = len(values);
  let holder = clamped_holder(value: index);
  claim projected: ilt(deref(holder.value), size) because "premises: holder is returned by clamped_holder, whose body stores imin(index, 7_u64), and values has length 8\nderivation: deref(holder.value) is at most 7_u64 and therefore strictly below size\nconclusion: ilt(deref(holder.value), size) is true\nchecker gap: ENT accepts the projected affine-holder copy read but does not publish the uncontracted call result invariant\nconsumers: the following length-eight array subscript uses deref(holder.value)";
  return values[deref(holder.value)];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_complete(source.as_bytes());
}

#[test]
fn boolean_equivalence_waits_for_a_canonical_conjunctive_basis() {
    let source = format!(
        r#"fn need(flag: own Bool) -> result: own unit pure contract {{
  requires flag;
}} {{
  return unit;
}}

fn probe(left: own Bool, right: own Bool) -> result: own unit traps {{
  let same = eeq(left, right);
  claim unsupported_equivalence: same because "{REJECTION_REVIEW}";
  need(flag: same);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn symmetric_disequality_components_deduplicate_by_normative_fact_identity() {
    let source = r#"fn reviewed_nonzero(value: own u64) -> result: own u64 pure {
  return imax(value, 1_u64);
}

fn ratio(n: own u64, d: own u64) -> result: own u64 traps {
  let divisor = reviewed_nonzero(value: d);
  let forward = ine(divisor, 0_u64);
  let reverse = ine(0_u64, divisor);
  let repeated = band(forward, reverse);
  claim nonzero: repeated because "premises: divisor is returned by reviewed_nonzero, whose body computes imax(d, 1_u64)\nderivation: divisor is at least 1_u64, hence nonzero in either written orientation, and their conjunction is true\nconclusion: band(forward, reverse) is true\nchecker gap: ENT does not publish an uncontracted user-call result bound but canonicalizes symmetric disequality identity\nconsumers: the following exact unsigned division requires divisor nonzero";
  return n / divisor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the symmetric true disequalities must deduplicate: {outcome:?}");
        };
        let entry = program
            .data
            .claim_ledger
            .entries
            .iter()
            .find(|entry| entry.name == "nonzero")
            .expect("nonzero claim entry");
        let proof = entry.proof.as_ref().expect("nonzero proof packet");
        assert_eq!(proof.components.len(), 1);
        let ClaimComponentFact::Relation(Relation::Distinct { left, right }) =
            proof.components[0].fact
        else {
            panic!("the retained component must be one disequality");
        };
        assert_ne!(left.0, 0);
        assert_eq!(right.0, 0, "the first written orientation must be retained");
    });
}

#[test]
fn an_uninstantiated_generic_claim_still_needs_a_source_schema_consumer() {
    let source = format!(
        r#"fn unused<T>(flag: own Bool) -> result: own unit traps {{
  claim no_consumer: flag because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn an_uninstantiated_generic_schema_still_checks_ordinary_admission_roots() {
    let source = br#"fn clamp_three(value: own u64) -> result: own u64 pure {
  return imin(value, 3_u64);
}

fn unused<T>(safe: own array<u8, 4>, unsafe_values: own array<u8, 4>, input: own u64, index: own u64) -> result: own u8 traps {
  let bounded = clamp_three(value: input);
  claim in_range: ilt(bounded, 4_u64) because "premises: bounded is returned by clamp_three, whose body computes imin(input, 3_u64)\nderivation: bounded is at most 3_u64 and therefore strictly less than 4_u64\nconclusion: bounded is below four\nchecker gap: ENT does not publish the result bound of clamp_three\nconsumers: the first array subscript consumes this exact bound";
  let first = safe[bounded];
  return unsafe_values[index];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_source_rule(source, SemanticRule::Op4);
}

#[test]
fn an_uninstantiated_generic_literal_true_is_redundant_in_its_source_schema() {
    let source = format!(
        r#"fn unused<T>() -> result: own unit traps {{
  claim known: True() because "{REJECTION_REVIEW}";
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn an_uninstantiated_generic_exact_fn8_residual_has_one_source_schema_report() {
    let source = format!(
        r#"fn need<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn prove<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim theorem: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need<T>(value: nonnegative);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the symbolic FN-8 consumer must retain the schema claim: {outcome:?}");
        };
        let schema = program
            .data
            .generic_claim_schemas
            .iter()
            .find(|schema| !schema.claims.is_empty())
            .expect("generic claim schema");
        assert_eq!(schema.display_symbol, "prove");
        assert_eq!(schema.claims.len(), 1);
        assert!(schema.claims[0].proof.is_none());
        let schema_proof = schema.claims[0]
            .schema_proof
            .as_ref()
            .expect("stable symbolic proof summary");
        assert!(!schema_proof.direct_image.is_empty());
        assert_ne!(schema_proof.direct_image, schema_proof.expanded_image);
        assert_eq!(schema_proof.expanded_image, schema_proof.complete_image);
        assert!(schema_proof.reconstruction_succeeded);
        assert_eq!(schema.claims[0].residual_witnesses.len(), 2);
        assert!(schema.claims[0].residual_witnesses.iter().all(|witness| {
            matches!(
                &witness.terminal,
                ClaimTerminalRoot::Call {
                    owner: ClaimTerminalOwner::Schema(owner),
                    function_symbol,
                    callee: ClaimTerminalOwner::Schema(_),
                    callee_symbol,
                    ..
                } if *owner == schema.declaration
                    && function_symbol == "prove"
                    && callee_symbol == "need"
            )
        }));
        let stable_debug = format!("{schema:?}");
        for scratch_identity in [
            "$instance$",
            "FunctionId(",
            "NominalId(",
            "GoalId(",
            "TermId(",
            "DerivationId(",
            "DerivedConstId(",
        ] {
            assert!(
                !stable_debug.contains(scratch_identity),
                "schema leaked scratch identity {scratch_identity}: {stable_debug}"
            );
        }
        assert!(program.data.claim_ledger.entries.is_empty());
    });
}

#[test]
fn nominal_bearing_generic_schema_diagnostics_and_proofs_use_source_stable_renderings() {
    let rejected = br#"enum Flag<T: Int> {
  Off();
  On();
}

fn need<T: Int>(left: own Flag<T>, right: own Flag<T>) -> result: own unit pure contract {
  requires eeq(left, right);
} {
  return unit;
}

fn unused<T: Int>(left: own Flag<T>, right: own Flag<T>) -> result: own unit pure {
  need<T>(left: left, right: right);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(rejected, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the unproved symbolic requirement must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!("the generic source-body rejection must retain its FN-8 payload");
        };
        assert!(detail.instantiated_goal.contains("Flag<"));
        for scratch_identity in ["$instance$", "FunctionId(", "NominalId("] {
            assert!(
                !detail.instantiated_goal.contains(scratch_identity),
                "schema diagnostic leaked scratch identity {scratch_identity}: {}",
                detail.instantiated_goal
            );
        }
    });

    let accepted = br#"enum Flag<T: Int> {
  Off();
  On();
}

fn hidden<T: Int>() -> result: own Flag<T> pure {
  return Off<T>();
}

fn need<T: Int>(left: own Flag<T>, right: own Flag<T>) -> result: own unit pure contract {
  requires eeq(left, right);
} {
  return unit;
}

fn prove<T: Int>() -> result: own unit traps {
  let left = hidden<T>();
  let right = hidden<T>();
  claim same: eeq(left, right) because "premises: left and right are returned by hidden<T>, whose body returns Off<T>()\nderivation: both values therefore have the same tag-only Flag<T> value\nconclusion: eeq(left, right) is true\nchecker gap: schema ENT does not publish uncontracted generic call-result equality\nconsumers: the following FN-8 requirement needs the exact Flag<T> equality";
  need<T>(left: left, right: right);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(accepted, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the nominal-bearing source schema residual must check: {outcome:?}");
        };
        let schema = program
            .data
            .generic_claim_schemas
            .iter()
            .find(|schema| schema.display_symbol == "prove")
            .expect("prove schema report");
        assert_eq!(schema.claims.len(), 1);
        let stable_debug = format!("{schema:?}");
        assert!(stable_debug.contains("Flag<"));
        for scratch_identity in [
            "$instance$",
            "FunctionId(",
            "NominalId(",
            "GoalId(",
            "TermId(",
            "DerivationId(",
            "DerivedConstId(",
        ] {
            assert!(
                !stable_debug.contains(scratch_identity),
                "schema proof leaked scratch identity {scratch_identity}: {stable_debug}"
            );
        }
    });
}

#[test]
fn a_generic_schema_links_each_inhabited_concrete_claim_report_in_stable_order() {
    let source = format!(
        r#"fn need_nonnegative<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn prove<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim theorem: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

command fn main() -> status: own ExitStatus traps {{
  prove<i8>(value: -1_i8);
  prove<i16>(value: -1_i16);
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("both signed instances must retain the true imax theorem: {outcome:?}");
        };
        let schema = program
            .data
            .generic_claim_schemas
            .iter()
            .find(|schema| schema.claims.iter().any(|claim| claim.name == "theorem"))
            .expect("prove source schema");
        assert_eq!(schema.concrete_reports.len(), 2);
        assert!(
            schema
                .concrete_reports
                .windows(2)
                .all(|pair| pair[0].function.0 < pair[1].function.0)
        );
        let entries = program
            .data
            .claim_ledger
            .entries
            .iter()
            .filter(|entry| {
                entry.source.declaration == schema.declaration && entry.name == "theorem"
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
        for (link, entry) in schema.concrete_reports.iter().zip(entries) {
            assert_eq!(link.function, entry.source.function);
            assert_eq!(link.claim, entry.source.node_path);
            assert_eq!(link.name, entry.name);
            assert!(entry.residual_witnesses.iter().all(|witness| matches!(
                witness.terminal,
                ClaimTerminalRoot::Call {
                    owner: ClaimTerminalOwner::Concrete(owner),
                    ..
                } if owner == entry.source.function
            )));
        }
    });
}

#[test]
fn checker_strengthening_in_one_generic_instance_rejects_the_shared_source_claim() {
    let source = format!(
        r#"fn need_nonnegative<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn clamp_and_prove<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim theorem: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

command fn main() -> status: own ExitStatus traps {{
  clamp_and_prove<i8>(value: -1_i8);
  clamp_and_prove<u8>(value: 1_u8);
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn a_concrete_instance_discovered_only_through_an_uninstantiated_generic_is_rechecked() {
    let source = format!(
        r#"fn need_nonnegative<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn prove<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim theorem: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

fn wrapper<U>() -> result: own unit traps {{
  prove<u8>(value: 1_u8);
  return unit;
}}

command fn main() -> status: own ExitStatus pure {{
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the explicit u8 instance must reject the shared claim: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a concrete redundancy payload: {:?}", issue.kind());
        };
        assert_eq!(detail.name, "theorem");
        assert_eq!(detail.classification, "redundant");
        assert_eq!(detail.instance.as_deref(), Some("prove<u8>"));
        assert!(!format!("{detail:?}").contains("$instance$"));
    });
}

#[test]
fn schema_written_concrete_nominal_arguments_are_rebuilt_after_the_symbolic_checkpoint() {
    let source = br#"struct Pair<T: Int> {
  value: T;
}

fn consume<T>(value: own T) -> result: own unit pure {
  return unit;
}

fn wrapper<U>() -> result: own unit pure {
  let pair = Pair<u8>(value: 1_u8);
  consume<Pair<u8>>(value: move pair);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the concrete nominal substitution must be replayed: {outcome:?}");
        };
        assert!(
            program
                .data
                .functions
                .iter()
                .any(|function| function.name == "consume")
        );
        assert!(
            program
                .data
                .nominals
                .iter()
                .any(|nominal| nominal.name.starts_with("Pair<"))
        );
    });
}

#[test]
fn partial_schema_replay_keeps_only_the_truly_concrete_nominal_instance() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn sink<T>() -> result: own unit pure {
  return unit;
}

fn middle<A: Int, B>() -> result: own unit pure {
  sink<Pair<A>>();
  return unit;
}

fn wrapper<U>() -> result: own unit pure {
  middle<u8, U>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("partial replay must retain only concrete descendants: {outcome:?}");
        };
        let pairs = program
            .data
            .nominals
            .iter()
            .take(program.data.executable_nominal_count)
            .filter(|nominal| nominal.name.starts_with("Pair<"))
            .collect::<Vec<_>>();
        let [pair] = pairs.as_slice() else {
            panic!("exactly one concrete Pair<u8> must survive: {pairs:?}");
        };
        let CheckedNominalKind::Struct { fields } = &pair.kind else {
            panic!("Pair is a struct nominal")
        };
        assert_eq!(fields.len(), 2);
        assert!(
            fields
                .iter()
                .all(|field| field.ty == CheckedType::Integer(IntegerType::U8))
        );
        assert_eq!(
            program
                .data
                .functions
                .iter()
                .filter(|function| function.name == "sink")
                .count(),
            1
        );
        lower_checked(*program).expect("a concrete-only replay inventory must lower");
    });
}

#[test]
fn partial_schema_replay_still_discovers_an_independent_concrete_descendant() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn sink<T>() -> result: own unit pure {
  return unit;
}

fn next<X, Y>() -> result: own unit pure {
  sink<Y>();
  return unit;
}

fn middle<A: Int>() -> result: own unit pure {
  next<Pair<A>, u8>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("partial replay must project its concrete descendant: {outcome:?}");
        };
        assert!(
            program
                .data
                .nominals
                .iter()
                .take(program.data.executable_nominal_count)
                .all(|nominal| !nominal.name.starts_with("Pair<"))
        );
        assert_eq!(
            program
                .data
                .functions
                .iter()
                .filter(|function| function.name == "sink")
                .count(),
            1
        );
        assert!(
            program
                .data
                .functions
                .iter()
                .all(|function| function.name != "next")
        );
        lower_checked(*program).expect("the concrete descendant inventory must lower");
    });
}

#[test]
fn a_concrete_fragment_fn9_root_can_consume_an_uninstantiated_generic_schema_claim() {
    let source = r#"fn reviewed_one(value: own u64) -> result: own u64 pure {
  let upper = imin(value, 1_u64);
  return imax(upper, 1_u64);
}

fn reviewed<T>(value: own u64) -> result: own u64 traps contract {
  ensures ieq(result, 1_u64);
} {
  let normalized = reviewed_one(value: value);
  claim result_is_one: ieq(normalized, 1_u64) because "premises: normalized is returned by reviewed_one, whose body computes imax(imin(value, 1_u64), 1_u64)\nderivation: the inner minimum is at most 1_u64 and the outer maximum with 1_u64 is exactly 1_u64\nconclusion: ieq(normalized, 1_u64) is true\nchecker gap: schema ENT does not publish an uncontracted user-call result equality\nconsumers: the complete concrete-u64 FN-9 selected-return proof needs result equal to 1_u64";
  return normalized;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the concrete-fragment FN-9 schema root must check: {outcome:?}");
        };
        let schema = program
            .data
            .generic_claim_schemas
            .iter()
            .find(|schema| schema.display_symbol == "reviewed")
            .expect("reviewed source schema");
        assert!(schema.claims[0].residual_witnesses.iter().all(|witness| {
            matches!(
                &witness.terminal,
                ClaimTerminalRoot::Postcondition {
                    owner: ClaimTerminalOwner::Schema(owner),
                    function_symbol,
                    ..
                } if *owner == schema.declaration && function_symbol == "reviewed"
            )
        }));
        assert!(!format!("{schema:?}").contains("$instance$"));
    });
}

#[test]
fn a_generic_int_fn9_root_is_concrete_instance_only_not_a_schema_consumer() {
    let source = r#"fn reviewed<T: Int>(value: own T) -> result: own T traps contract {
  ensures ige(result, 1_T);
} {
  let normalized = imax(value, 1_T);
  claim result_at_least_one: ige(normalized, 1_T) because "premises: normalized is imax(value, 1_T) for every integer T\nderivation: imax returns an operand no smaller than 1_T\nconclusion: ige(normalized, 1_T) is true\nchecker gap: schema ENT does not treat GenericInt as an L0 FN-9 fragment\nconsumers: only a generic-T FN-9 clause would consume this theorem, and that root is concrete-instance-only";
  return normalized;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.to_string();
    assert_source_rule(source.as_bytes(), SemanticRule::Clm2);
}

#[test]
fn an_entry_uninhabited_generic_instance_produces_no_concrete_claim_report() {
    let source = format!(
        r#"fn need_nonnegative<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn guarded<T: Int>(guard: own T, input: own T) -> result: own unit traps contract {{
  requires ilt(guard, 0_T);
}} {{
  let nonnegative = imax(input, 0_T);
  claim source_theorem: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

command fn main() -> status: own ExitStatus traps {{
  let never = False();
  if never {{
    guarded<u8>(guard: 0_u8, input: 5_u8);
  }}
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!(
                "the uninhabited concrete instance must not reject the schema claim: {outcome:?}"
            );
        };
        assert_eq!(
            program
                .data
                .generic_claim_schemas
                .iter()
                .map(|schema| schema.claims.len())
                .sum::<usize>(),
            1
        );
        assert!(program.data.claim_ledger.entries.is_empty());
    });
}

#[test]
fn generic_diagnostics_order_source_occurrences_before_concrete_instances() {
    let source = format!(
        r#"fn need_nonnegative<T: Int>(value: own T) -> result: own unit pure contract {{
  requires ige(value, 0_T);
}} {{
  return unit;
}}

fn first_proof<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim first_nonnegative: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

fn second_proof<T: Int>(value: own T) -> result: own unit traps {{
  let nonnegative = imax(value, 0_T);
  claim second_nonnegative: ige(nonnegative, 0_T) because "{GENERIC_MAX_NONNEGATIVE_REVIEW}";
  need_nonnegative<T>(value: nonnegative);
  return unit;
}}

command fn main() -> status: own ExitStatus traps {{
  second_proof<u8>(value: 1_u8);
  first_proof<u8>(value: 1_u8);
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("mixed generic instances must reject one shared claim: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::InvalidClaim(detail) = issue.kind() else {
            panic!("expected a redundancy payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.name, "first_nonnegative");
        assert_eq!(detail.classification, "redundant");
        assert_eq!(detail.instance.as_deref(), Some("first_proof<u8>"));
    });
}

#[test]
fn ordinary_admission_diagnostics_order_source_before_dense_instance_identity() {
    let source =
        br#"fn earlier<T>(values: own array<u8, 4>, index: own u64) -> result: own u8 pure {
  return values[index];
}

fn later(values: own array<u8, 4>, index: own u64) -> result: own u8 pure {
  return values[index];
}

command fn main() -> status: own ExitStatus pure {
  let first_values = array_new<u8, 4>(0_u8);
  let second_values = array_new<u8, 4>(0_u8);
  earlier<u8>(values: move first_values, index: 5_u64);
  later(values: move second_values, index: 5_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("both bodies contain an OP-4 rejection: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
        let crate::SemanticLocation::SourceNode(path, _) = issue.location() else {
            panic!("OP-4 must cite the source operation");
        };
        assert_eq!(path.components().first(), Some(&0));
    });
}
