use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedConst, CheckedNominalKind, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn explicit_int_generic_function_builds_each_reachable_concrete_instance() {
    let source = br#"fn identity<T: Int>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit traps {
  let first = identity<u32>(value: 7_u32);
  let second = identity<i64>(value: -9_i64);
  check ieq(first, 7_u32) else trap "u32 generic instance";
  check ieq(second, -9_i64) else trap "i64 generic instance";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("explicit generic instances must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 3);
        assert_eq!(checked.entry_function_name(), "main");
    });
}

#[test]
fn int_bound_selects_the_same_operation_row_for_every_concrete_instance() {
    let source = br#"fn maximum<T: Int>(left: own T, right: own T) -> own T pure {
  return imax(left, right);
}

fn main() -> own unit traps {
  let small = maximum<u8>(left: 4_u8, right: 9_u8);
  let signed = maximum<i64>(left: -7_i64, right: -2_i64);
  check ieq(small, 9_u8) else trap "u8 generic maximum";
  check ieq(signed, -2_i64) else trap "i64 generic maximum";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("Int-bound operation must check for each instance: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 3);
    });
}

#[test]
fn float_bound_selects_operations_and_identities_for_every_concrete_instance() {
    let source = br#"fn affine<T: Float>(value: own T) -> own T pure {
  let zero = 0_T;
  let one = 1_T;
  let shifted = fadd.strict(value, one);
  return fadd.strict(zero, shifted);
}

fn main() -> own unit traps {
  let single = affine<f32>(value: 2.0_f32);
  let double = affine<f64>(value: 4.0_f64);
  check feq(single, 3.0_f32) else trap "f32 generic operation";
  check feq(double, 5.0_f64) else trap "f64 generic operation";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("Float-bound operations must check for each instance: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 3);
    });
}

#[test]
fn float_bound_rejects_a_non_float_explicit_argument_under_fn3() {
    let source = br#"fn identity<T: Float>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit pure {
  let invalid = identity<u32>(value: 7_u32);
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Fn3, SemanticIssueKind::TypeMismatch);
}

#[test]
fn numeric_identity_requires_an_int_or_float_bound() {
    let source = br#"fn invalid<T>() -> own T pure {
  return 0_T;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Form5, SemanticIssueKind::TypeMismatch);
}

#[test]
fn int_bound_identity_is_concretized_before_lowering() {
    let source = br#"fn one<T: Int>() -> own T pure {
  return 1_T;
}

fn main() -> own unit traps {
  let value = one<u16>();
  check ieq(value, 1_u16) else trap "generic integer identity";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("Int-bound identity must check and concretize: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 2);
    });
}

#[test]
fn generic_conversion_is_reported_as_unsupported_instead_of_invalid_source() {
    let source = br#"fn convert<T: Int>(value: own T) -> own unit pure {
  cvt<T, u64>(value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::Generics);
}

#[test]
fn int_bound_rejects_a_non_integer_explicit_argument_under_fn3() {
    let source = br#"fn identity<T: Int>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit pure {
  let input = True();
  let invalid = identity<Bool>(value: input);
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Fn3, SemanticIssueKind::TypeMismatch);
}

#[test]
fn generic_call_cycle_stops_before_concrete_instance_enumeration() {
    let source = br#"fn recursive<T: Int>(value: own T) -> own T pure {
  return recursive<T>(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::Generics);
}

#[test]
fn unused_int_generic_body_is_checked_for_the_complete_bound_domain() {
    let source = br#"fn invalid<T: Int>(value: own T) -> own T pure {
  return 0_u8;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(source, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}

#[test]
fn concretely_invalid_generic_body_is_rejected_during_instance_rechecking() {
    let source = br#"fn transfer<T>(value: own T) -> own T pure {
  return move value;
}

fn main() -> own unit pure {
  let copied = transfer<u8>(value: 7_u8);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Own1,
        SemanticIssueKind::MoveOfCopy {
            mechanical_fix: "use the copy place without `move`",
        },
    );
}

#[test]
fn nested_generic_calls_discover_reachable_instances_after_template_checking() {
    let source = br#"fn select<T: Int>(value: own T) -> own T pure {
  return imax(value, value);
}

fn forward<T: Int>(value: own T) -> own T pure {
  return select<T>(value: value);
}

fn main() -> own unit traps {
  let small = forward<u8>(value: 7_u8);
  let signed = forward<i64>(value: -9_i64);
  check ieq(small, 7_u8) else trap "nested u8 instance";
  check ieq(signed, -9_i64) else trap "nested i64 instance";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("nested generic calls must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 5);
    });
}

#[test]
fn const_parameters_forward_symbolically_and_instantiate_at_reachable_sizes() {
    let source = br#"fn preserve<const n: u64>(value: own array<u8, n>) -> own array<u8, n> pure {
  let size = len(value);
  return move value;
}

fn forward<const n: u64>(value: own array<u8, n>) -> own array<u8, n> pure {
  return preserve<n>(value: move value);
}

fn main() -> own unit traps {
  let small_input = array_new<u8, 2>(7_u8);
  let small = forward<2>(value: move small_input);
  let large_input = array_new<u8, 5>(9_u8);
  let large = forward<5>(value: move large_input);
  let first = small[1_u64];
  let second = large[4_u64];
  check ieq(first, 7_u8) else trap "small const instance";
  check ieq(second, 9_u8) else trap "large const instance";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("forwarded const instances must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 5);
    });
}

#[test]
fn unbounded_type_parameters_build_only_explicit_reachable_instances() {
    let source = br#"fn marker<T>() -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  marker<u8>();
  marker<Bool>();
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("unbounded marker instances must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 3);
    });
}

/// The first two are wrong-kind arguments on a **user-generic call**, so
/// [DIAG-1] gives them to FN-2: "the cited rule is the rule selected by the
/// callee's class". They recorded TYPE-5 while the compiler chose its rule
/// from the kind of argument problem instead of the callee, and they move with
/// the 2026-08-08 ruling that settled the question — TYPE-5 governs whether an
/// argument's type matches its parameter, not the argument list itself. The
/// third is CONST-1's own violation and is unaffected.
#[test]
fn generic_argument_kinds_and_const_parameter_types_are_checked() {
    assert_rule(
        br#"fn marker<T>() -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  marker<4>();
  return unit;
}
"#,
        SemanticRule::Fn2,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        br#"fn sized<const n: u64>() -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  sized<u8>();
  return unit;
}
"#,
        SemanticRule::Fn2,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        br#"fn invalid<const n: Bool>() -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Const1,
        SemanticIssueKind::InvalidConstValue,
    );
}

#[test]
fn source_generic_structs_are_checked_symbolically_and_rechecked_per_instance() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn duplicate<T: Int>(value: own T) -> own Pair<T> pure {
  return Pair<T>(left: value, right: value);
}

fn main() -> own unit traps {
  let small = duplicate<u8>(value: 7_u8);
  let wide = duplicate<i64>(value: -9_i64);
  let small_left = small.left;
  let wide_right = wide.right;
  check ieq(small_left, 7_u8) else trap "small generic struct";
  check ieq(wide_right, -9_i64) else trap "wide generic struct";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic struct instances must check: {outcome:?}");
        };
        let pair_instances = checked
            .data
            .nominals
            .iter()
            .filter(|nominal| nominal.name.starts_with("Pair<"))
            .collect::<Vec<_>>();
        assert_eq!(pair_instances.len(), 2);
        assert_ne!(pair_instances[0].id, pair_instances[1].id);
        assert_eq!(checked.function_count(), 3);
    });
}

#[test]
fn source_generic_enums_use_the_concrete_instance_member_table() {
    let source = br#"enum Choice<T: Int> {
  Missing();
  Present(value: T);
}

fn main() -> own unit traps {
  let small = Present<u8>(value: 3_u8);
  match small {
    Missing() => {
      check False() else trap "unexpected missing";
    }
    Present(value: observed) => {
      check ieq(observed, 3_u8) else trap "wrong payload";
    }
  }
  let wide = Present<i64>(value: -5_i64);
  match wide {
    Missing() => {
      check False() else trap "unexpected missing";
    }
    Present(value: observed) => {
      check ieq(observed, -5_i64) else trap "wrong payload";
    }
  }
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic enum instances must check: {outcome:?}");
        };
        assert_eq!(
            checked
                .data
                .nominals
                .iter()
                .filter(|nominal| nominal.name.starts_with("Choice<"))
                .count(),
            2
        );
    });
}

#[test]
fn const_and_nested_source_nominal_instances_are_fully_substituted() {
    let source = br#"struct Packet<const n: u64> {
  bytes: array<u8, n>;
}

struct Holder<T> {
  value: T;
}

fn main() -> own unit pure {
  let short_bytes = array_new<u8, 2>(7_u8);
  let short = Packet<2>(bytes: move short_bytes);
  let long_bytes = array_new<u8, 5>(11_u8);
  let long = Packet<5>(bytes: move long_bytes);
  let held = Holder<Packet<2>>(value: move short);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("const and nested nominal instances must check: {outcome:?}");
        };
        let mut packet_lengths = checked
            .data
            .nominals
            .iter()
            .filter(|nominal| nominal.name.starts_with("Packet<"))
            .map(|nominal| match &nominal.kind {
                CheckedNominalKind::Struct { fields } => match fields[0].ty {
                    CheckedType::Array {
                        length: CheckedConst::Value(length),
                        ..
                    } => length,
                    other => panic!("Packet field must be a concrete array: {other:?}"),
                },
                other => panic!("Packet must remain a struct: {other:?}"),
            })
            .collect::<Vec<_>>();
        packet_lengths.sort_unstable();
        assert_eq!(packet_lengths, [2, 5]);
        assert_eq!(
            checked
                .data
                .nominals
                .iter()
                .filter(|nominal| nominal.name.starts_with("Holder<"))
                .count(),
            1
        );
    });
}

#[test]
fn source_nominal_argument_arity_and_kinds_are_exact() {
    assert_rule(
        br#"struct Pair<T> {
  value: T;
}

fn main() -> own unit pure {
  let invalid = Pair<u8, u16>(value: 1_u8);
  return unit;
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        br#"struct Packet<const n: u64> {
  bytes: array<u8, n>;
}

fn main() -> own unit pure {
  let bytes = array_new<u8, 1>(0_u8);
  let invalid = Packet<u8>(bytes: move bytes);
  return unit;
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn constructor_only_generic_instances_still_reach_normal_type_diagnostics() {
    assert_rule(
        br#"struct Holder<T> {
  value: T;
}

fn main() -> own unit pure {
  return Holder<u8>(value: 1_u8);
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
}

#[test]
fn unused_generic_nominal_members_are_checked_under_their_declared_bounds() {
    assert_rule(
        br#"struct Invalid<T> {
  values: array<T, 2>;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Type2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn recursive_generic_nominal_layouts_stop_before_concrete_enumeration() {
    assert_unsupported(
        br#"struct Recursive<T> {
  next: Recursive<T>;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::RecursiveNominalLayout,
    );
}

#[test]
fn generic_nominals_may_contain_symbolic_prelude_instances() {
    let source = br#"struct Wrapped<T: Int> {
  value: Option<T>;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("symbolic prelude fields must receive template coverage: {outcome:?}");
        };
    });
}

#[test]
fn checked_integer_results_are_available_during_template_and_concrete_rechecking() {
    let source =
        br#"fn checked_sum<T: Int>(left: own T, right: own T) -> own Result<T, Overflow> pure {
  return left +checked right;
}

fn main() -> own unit pure {
  let small = checked_sum<u8>(left: 1_u8, right: 2_u8);
  let wide = checked_sum<i64>(left: -3_i64, right: 5_i64);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("checked generic results must check through both stages: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 3);
    });
}

#[test]
fn numeric_and_const_parameters_flow_through_flat_storage_operations() {
    let source = br#"fn filled_array<T: Int, const n: u64>(value: own T) -> own array<T, n> pure {
  return array_new<T, n>(value);
}

fn filled_buffer<T: Int>(length: own u64, value: own T) -> own buffer<T> allocates(heap), traps {
  return buffer_new(length, value);
}

fn filled_float_array<T: Float, const n: u64>(value: own T) -> own array<T, n> pure {
  return array_new<T, n>(value);
}

fn filled_float_buffer<T: Float>(length: own u64, value: own T) -> own buffer<T> allocates(heap), traps {
  return buffer_new(length, value);
}

fn main() -> own unit allocates(heap), traps {
  let bytes = filled_array<u8, 2>(value: 7_u8);
  let words = filled_array<i64, 3>(value: -5_i64);
  let byte = bytes[1_u64];
  let word = words[2_u64];
  let storage = filled_buffer<u16>(length: 2_u64, value: 9_u16);
  let storage_room = len(storage);
  let storage_ok = ilt(1_u64, storage_room);
  claim storage_sized: storage_ok because "filled_buffer allocates its length argument";
  let buffered = storage[1_u64];
  let samples = filled_float_array<f32, 2>(value: 1.5_f32);
  let sample = samples[1_u64];
  let weights = filled_float_buffer<f64>(length: 2_u64, value: 2.5_f64);
  let weights_room = len(weights);
  let weights_ok = ilt(1_u64, weights_room);
  claim weights_sized: weights_ok because "filled_float_buffer allocates its length argument";
  let weight = weights[1_u64];
  check ieq(byte, 7_u8) else trap "generic array";
  check ieq(word, -5_i64) else trap "generic const array";
  check ieq(buffered, 9_u16) else trap "generic buffer";
  check feq(sample, 1.5_f32) else trap "generic float array";
  check feq(weight, 2.5_f64) else trap "generic float buffer";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic flat storage must check and concretize: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 6);
    });
}

#[test]
fn region_bearing_function_and_nominal_arguments_reject_under_fn2() {
    let expected = SemanticIssueKind::RegionBearingGenericArgument {
        mechanical_fix: "make the slice or arena a direct written parameter or result instead of a generic argument",
    };
    assert_rule(
        br#"fn instantiate<T>() -> own unit pure {
  return unit;
}

fn invalid['r]() -> own unit pure {
  instantiate<slice<'r, u8>>();
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"struct Marker<T> {
}

fn invalid['r](value: own Marker<slice<'r, u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['r](value: own Option<slice<'r, u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"fn instantiate<T>() -> own unit pure {
  return unit;
}

fn invalid['r]() -> own unit pure {
  instantiate<arena<'r, u8>>();
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn2,
        expected,
    );
}
