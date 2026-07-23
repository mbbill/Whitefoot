use crate::{
    SemanticIssueKind, SemanticOutcome, SemanticRuleV0_15, UnsupportedSemanticFeatureV0_15,
};

use super::super::model::{CheckedConst, CheckedNominalKind, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn explicit_int_generic_function_builds_each_reachable_concrete_instance() {
    let source = br#"fn identity<T: Int>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit traps {
  let first: own u32 = identity<u32>(value: 7_u32);
  let second: own i64 = identity<i64>(value: -9_i64);
  check ieq<u32>(first, 7_u32) else trap "u32 generic instance";
  check ieq<i64>(second, -9_i64) else trap "i64 generic instance";
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
  return imax<T>(left, right);
}

fn main() -> own unit traps {
  let small: own u8 = maximum<u8>(left: 4_u8, right: 9_u8);
  let signed: own i64 = maximum<i64>(left: -7_i64, right: -2_i64);
  check ieq<u8>(small, 9_u8) else trap "u8 generic maximum";
  check ieq<i64>(signed, -2_i64) else trap "i64 generic maximum";
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
fn generic_conversion_is_reported_as_unsupported_instead_of_invalid_source() {
    let source = br#"fn convert<T: Int>(value: own T) -> own unit pure {
  cvt<T, u64>(value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeatureV0_15::Generics);
}

#[test]
fn int_bound_rejects_a_non_integer_explicit_argument_under_fn3() {
    let source = br#"fn identity<T: Int>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit pure {
  let input: own Bool = True();
  let invalid: own Bool = identity<Bool>(value: input);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRuleV0_15::Fn3,
        SemanticIssueKind::TypeMismatch,
    );
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
    assert_unsupported(source, UnsupportedSemanticFeatureV0_15::Generics);
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
    assert_rule(
        source,
        SemanticRuleV0_15::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
}

#[test]
fn concretely_invalid_generic_body_is_rejected_during_instance_rechecking() {
    let source = br#"fn transfer<T>(value: own T) -> own T pure {
  return move value;
}

fn main() -> own unit pure {
  let copied: own u8 = transfer<u8>(value: 7_u8);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRuleV0_15::Own1,
        SemanticIssueKind::MoveOfCopy {
            mechanical_fix: "use the copy place without `move`",
        },
    );
}

#[test]
fn nested_generic_calls_discover_reachable_instances_after_template_checking() {
    let source = br#"fn select<T: Int>(value: own T) -> own T pure {
  return imax<T>(value, value);
}

fn forward<T: Int>(value: own T) -> own T pure {
  return select<T>(value: value);
}

fn main() -> own unit traps {
  let small: own u8 = forward<u8>(value: 7_u8);
  let signed: own i64 = forward<i64>(value: -9_i64);
  check ieq<u8>(small, 7_u8) else trap "nested u8 instance";
  check ieq<i64>(signed, -9_i64) else trap "nested i64 instance";
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
  let size: own u64 = len<u8>(value);
  return move value;
}

fn forward<const n: u64>(value: own array<u8, n>) -> own array<u8, n> pure {
  return preserve<n>(value: move value);
}

fn main() -> own unit traps {
  let small_input: own array<u8, 2> = array_new<u8, 2>(7_u8);
  let small: own array<u8, 2> = forward<2>(value: move small_input);
  let large_input: own array<u8, 5> = array_new<u8, 5>(9_u8);
  let large: own array<u8, 5> = forward<5>(value: move large_input);
  let first: own u8 = index<u8>(small, 1_u64);
  let second: own u8 = index<u8>(large, 4_u64);
  check ieq<u8>(first, 7_u8) else trap "small const instance";
  check ieq<u8>(second, 9_u8) else trap "large const instance";
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
        SemanticRuleV0_15::Type5,
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
        SemanticRuleV0_15::Type5,
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
        SemanticRuleV0_15::Const1,
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
  let small: own Pair<u8> = duplicate<u8>(value: 7_u8);
  let wide: own Pair<i64> = duplicate<i64>(value: -9_i64);
  let small_left: own u8 = small.left;
  let wide_right: own i64 = wide.right;
  check ieq<u8>(small_left, 7_u8) else trap "small generic struct";
  check ieq<i64>(wide_right, -9_i64) else trap "wide generic struct";
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
  let small: own Choice<u8> = Present<u8>(value: 3_u8);
  match small {
    Missing() => {
      check False() else trap "unexpected missing";
    }
    Present(value: observed) => {
      check ieq<u8>(observed, 3_u8) else trap "wrong payload";
    }
  }
  let wide: own Choice<i64> = Present<i64>(value: -5_i64);
  match wide {
    Missing() => {
      check False() else trap "unexpected missing";
    }
    Present(value: observed) => {
      check ieq<i64>(observed, -5_i64) else trap "wrong payload";
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
  let short_bytes: own array<u8, 2> = array_new<u8, 2>(7_u8);
  let short: own Packet<2> = Packet<2>(bytes: move short_bytes);
  let long_bytes: own array<u8, 5> = array_new<u8, 5>(11_u8);
  let long: own Packet<5> = Packet<5>(bytes: move long_bytes);
  let held: own Holder<Packet<2>> = Holder<Packet<2>>(value: move short);
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
  let invalid: own Pair<u8, u16> = Pair<u8, u16>(value: 1_u8);
  return unit;
}
"#,
        SemanticRuleV0_15::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        br#"struct Packet<const n: u64> {
  bytes: array<u8, n>;
}

fn main() -> own unit pure {
  let bytes: own array<u8, 1> = array_new<u8, 1>(0_u8);
  let invalid: own Packet<u8> = Packet<u8>(bytes: move bytes);
  return unit;
}
"#,
        SemanticRuleV0_15::Type5,
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
        SemanticRuleV0_15::Fn1,
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
        SemanticRuleV0_15::Type2,
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
        UnsupportedSemanticFeatureV0_15::RecursiveNominalLayout,
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
  return iadd.checked<T>(left, right);
}

fn main() -> own unit pure {
  let small: own Result<u8, Overflow> = checked_sum<u8>(left: 1_u8, right: 2_u8);
  let wide: own Result<i64, Overflow> = checked_sum<i64>(left: -3_i64, right: 5_i64);
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
fn int_and_const_parameters_flow_through_flat_storage_operations() {
    let source = br#"fn filled_array<T: Int, const n: u64>(value: own T) -> own array<T, n> pure {
  return array_new<T, n>(value);
}

fn filled_buffer<T: Int>(length: own u64, value: own T) -> own buffer<T> allocates(heap), traps {
  return buffer_new<T>(length, value);
}

fn main() -> own unit allocates(heap), traps {
  let bytes: own array<u8, 2> = filled_array<u8, 2>(value: 7_u8);
  let words: own array<i64, 3> = filled_array<i64, 3>(value: -5_i64);
  let byte: own u8 = index<u8>(bytes, 1_u64);
  let word: own i64 = index<i64>(words, 2_u64);
  let storage: own buffer<u16> = filled_buffer<u16>(length: 2_u64, value: 9_u16);
  let buffered: own u16 = index<u16>(storage, 1_u64);
  check ieq<u8>(byte, 7_u8) else trap "generic array";
  check ieq<i64>(word, -5_i64) else trap "generic const array";
  check ieq<u16>(buffered, 9_u16) else trap "generic buffer";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic flat storage must check and concretize: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 4);
    });
}
