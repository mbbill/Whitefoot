use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, lower_checked};

use super::super::model::{CheckedContractLawKind, CheckedIntegerOperation};
use super::{assert_rule, with_semantics};

fn assert_issue_slice(source: &[u8], rule: SemanticRule, kind: SemanticIssueKind, expected: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("contract diagnostics are source-node diagnostics");
        };
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits");
        assert_eq!(&source[start..end], expected);
    });
}

#[test]
fn static_contract_metadata_is_complete_and_non_executable() {
    let source = include_bytes!("../../../../tests/conformance/cases/fn3-pos-contract-conform.wf");
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("valid static conformance must check: {outcome:?}");
        };
        assert_eq!(checked.data.contracts.len(), 1);
        assert_eq!(checked.data.contracts[0].name, "Zeroed");
        assert_eq!(checked.data.contracts[0].members.len(), 1);
        assert_eq!(checked.data.contracts[0].members[0].name, "zero");
        assert!(checked.data.contracts[0].laws.is_empty());
        assert_eq!(checked.data.conformances.len(), 1);
        assert_eq!(checked.data.conformances[0].bindings.len(), 1);
        assert!(checked.data.law_derivations.is_empty());

        // The bound function is still present exactly once in the ordinary
        // function table. Contract metadata contributes no executable function.
        assert_eq!(checked.data.functions.len(), 2);
        let lowered = lower_checked(*checked)
            .expect("static contract metadata must not alter ordinary lowering");
        assert_eq!(lowered.functions().len(), 2);
    });
}

#[test]
fn marker_contract_and_empty_conformance_are_valid() {
    let source = br#"contract Marker {
}

conform i32: Marker {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("empty marker conformance must check: {outcome:?}");
        };
        assert!(checked.data.contracts[0].members.is_empty());
        assert!(checked.data.conformances[0].bindings.is_empty());
    });
}

#[test]
fn conformance_subject_materializes_its_only_generic_nominal_instance() {
    let source = br#"struct Wrapper<T> {
  value: T;
}

contract Marker {
}

conform Wrapper<i32>: Marker {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("contract-only generic subject must check: {outcome:?}");
        };
        assert_eq!(checked.data.conformances.len(), 1);
        assert!(
            checked
                .data
                .nominals
                .iter()
                .any(|nominal| nominal.name.starts_with("Wrapper<"))
        );
        let semantic_nominal_count = checked.data.nominals.len();
        let lowered = lower_checked(*checked)
            .expect("conformance-only nominal metadata must not affect ordinary lowering");
        assert_eq!(lowered.nominals().len() + 1, semantic_nominal_count);
    });
}

#[test]
fn contract_member_materializes_its_only_generic_nominal_instance() {
    let source = br#"struct Wrapper<T> {
  value: T;
}

contract Factory {
  fn make() -> own Wrapper<i32> pure;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("contract-only generic member type must check: {outcome:?}");
        };
        assert_eq!(checked.data.contracts[0].members.len(), 1);
        assert!(
            checked
                .data
                .nominals
                .iter()
                .any(|nominal| nominal.name.starts_with("Wrapper<"))
        );
        let semantic_nominal_count = checked.data.nominals.len();
        let lowered = lower_checked(*checked)
            .expect("contract-only nominal metadata must not affect ordinary lowering");
        assert_eq!(lowered.nominals().len() + 1, semantic_nominal_count);
    });
}

#[test]
fn affine_const_is_not_usable_as_an_owned_law_identity() {
    let source = br#"const zero: array<u8, 1> = [0_u8];

contract InvalidIdentity {
  fn combine(x: own array<u8, 1>, y: own array<u8, 1>) -> own array<u8, 1> pure;
  law identity(combine, zero);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn4,
        SemanticIssueKind::InvalidContractLaw,
        b"law identity(combine, zero);",
    );
}

#[test]
fn protected_fn3_rejections_keep_their_rule() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn3-neg-two-conformances.wf"),
        SemanticRule::Fn3,
        SemanticIssueKind::DuplicateConformance,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn3-neg-requires-member.wf"),
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn3-neg-signature-effect-mismatch.wf"),
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
}

#[test]
fn protected_fn4_cases_discharge_only_the_closed_table() {
    with_semantics(
        include_bytes!("../../../../tests/conformance/cases/fn4-pos-law-in-contract.wf"),
        |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("law without conformance is a valid obligation: {outcome:?}");
            };
            assert_eq!(checked.data.contracts[0].laws.len(), 1);
            assert!(checked.data.law_derivations.is_empty());
        },
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn4-neg-bad-lawname.wf"),
        SemanticRule::Fn4,
        SemanticIssueKind::InvalidContractLaw,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn4-neg-law-refuted-signedness.wf"),
        SemanticRule::Fn4,
        SemanticIssueKind::UndischargedContractLaw,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn4-neg-law-undischarged.wf"),
        SemanticRule::Fn4,
        SemanticIssueKind::UndischargedContractLaw,
    );

    with_semantics(
        include_bytes!("../../../../tests/conformance/cases/fn4-pos-law-discharged.wf"),
        |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("closed unsigned saturating-add laws must discharge: {outcome:?}");
            };
            assert_eq!(checked.data.law_derivations.len(), 3);
            assert_eq!(
                checked.data.law_derivations[0].law,
                CheckedContractLawKind::Associative
            );
            assert_eq!(
                checked.data.law_derivations[1].law,
                CheckedContractLawKind::Commutative
            );
            assert_eq!(
                checked.data.law_derivations[2].law,
                CheckedContractLawKind::Identity
            );
            assert!(
                checked
                    .data
                    .law_derivations
                    .iter()
                    .all(|record| record.operation == CheckedIntegerOperation::AddSaturating)
            );
        },
    );
}

#[test]
fn identity_wrong_literal_type_is_an_fn4_role_error() {
    let source = br#"contract BadIdentity {
  fn combine(x: own u64, y: own u64) -> own u64 pure;
  law identity(combine, unit);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn4,
        SemanticIssueKind::InvalidContractLaw,
        b"law identity(combine, unit);",
    );
}

#[test]
fn earlier_same_typed_zero_constant_discharges_identity() {
    let source = br#"const zero: u64 = 0_u64;

contract AddIdentity {
  fn combine(x: own u64, y: own u64) -> own u64 pure;
  law identity(combine, zero);
}

fn saturating_add(x: own u64, y: own u64) -> own u64 pure {
  return iadd.sat<u64>(x, y);
}

conform u64: AddIdentity {
  combine = saturating_add;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("same-typed earlier zero constant must discharge identity: {outcome:?}");
        };
        assert_eq!(checked.data.law_derivations.len(), 1);
        assert_eq!(
            checked.data.law_derivations[0].law,
            CheckedContractLawKind::Identity
        );
    });
}

#[test]
fn contract_generics_point_at_the_generic_child() {
    let source = br#"contract Generic<T> {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::GenericContract,
        b"<T>",
    );
}

#[test]
fn repeated_member_points_at_the_later_signature() {
    let source = br#"contract Repeated {
  fn value() -> own i32 pure;
  fn value() -> own i32 pure;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::DuplicateContractMember {
            member: "value".to_owned(),
        },
        b"fn value() -> own i32 pure;",
    );
}

#[test]
fn prelude_contract_points_at_its_type_identifier_token() {
    let source = br#"conform i32: Int {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::InvalidConformanceContract,
        b"Int",
    );
}

#[test]
fn contract_arguments_point_at_the_targs_child() {
    let source = br#"contract Plain {
}

conform i32: Plain<i32> {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::ConformanceContractArguments,
        b"<i32>",
    );
}

#[test]
fn duplicate_key_points_at_the_later_conformance() {
    let source = br#"contract Marker {
}

conform i32: Marker {
}

conform i32: Marker {
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::DuplicateConformance,
        b"conform i32: Marker {\n}",
    );
}

#[test]
fn incompatible_and_out_of_order_bindings_point_at_the_fn_bind() {
    let source = br#"contract Pair {
  fn first() -> own i32 pure;
  fn second() -> own i32 pure;
}

fn make_first() -> own i32 pure {
  return 1_i32;
}

fn make_second() -> own i32 pure {
  return 2_i32;
}

conform i32: Pair {
  second = make_second;
  first = make_first;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::InvalidConformanceBinding {
            expected_member: Some("first".to_owned()),
        },
        b"second = make_second;",
    );
}

#[test]
fn missing_binding_points_at_the_conformance_closing_brace() {
    let source = br#"contract Pair {
  fn first() -> own i32 pure;
  fn second() -> own i32 pure;
}

fn make_first() -> own i32 pure {
  return 1_i32;
}

conform i32: Pair {
  first = make_first;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::MissingConformanceBinding {
            member: "second".to_owned(),
        },
        b"}",
    );
}

#[test]
fn source_contract_bound_points_at_the_bound_type_identifier() {
    let source = br#"contract Marker {
}

fn generic<T: Marker>() -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::SourceContractGenericBound,
        b"Marker",
    );
}

#[test]
fn positional_region_alpha_equality_covers_modes_and_normalized_effect_sets() {
    let source = br#"contract LengthSum {
  fn sum ['left, 'right](x: &'left buffer<u8>, y: &'right buffer<u8>) -> own u64 reads('left 'right);
}

fn add_lengths ['a, 'b](first: &'a buffer<u8>, second: &'b buffer<u8>) -> own u64 reads('b 'a) {
  let first_length: own u64 = len<u8>(deref(first));
  let second_length: own u64 = len<u8>(deref(second));
  return iadd.wrap<u64>(first_length, second_length);
}

conform buffer<u8>: LengthSum {
  sum = add_lengths;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("positional region alpha equality must check: {outcome:?}");
        };
        assert_eq!(checked.data.conformances.len(), 1);
    });
}

#[test]
fn positional_region_ordinal_swap_is_not_alpha_equal() {
    let source = br#"contract FirstLength {
  fn length ['left, 'right](x: &'left buffer<u8>, y: &'right buffer<u8>) -> own u64 reads('left);
}

fn second_length ['a, 'b](first: &'a buffer<u8>, second: &'b buffer<u8>) -> own u64 reads('b) {
  return len<u8>(deref(second));
}

conform buffer<u8>: FirstLength {
  length = second_length;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
        b"length = second_length;",
    );
}
