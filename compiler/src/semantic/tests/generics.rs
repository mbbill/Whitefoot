use crate::{
    SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature, lower_checked,
    lowering::OverlapLowering,
};

use super::super::model::{CheckedConst, CheckedNominalKind, CheckedType, IntegerType};
use super::{assert_rule, assert_rule_kind, assert_unsupported, with_semantics};

#[test]
fn brand_parameters_include_phantom_and_every_nested_name_position() {
    let source = br#"struct Phantom['a, 'b] {
  tag: u64;
}

struct Wrapped['a, 'b] {
  value: Phantom<'a, 'b>;
}

fn read['a, 'b](value: &Wrapped<'a, 'b>) -> result: own u64 reads(value.value.tag) {
  return deref(value).value.tag;
}

command fn main() -> status: own ExitStatus pure {
  region 'outer {
    region 'inner {
      let value = Phantom<'inner, 'outer>(tag: 17_u64);
      let wrapped = Wrapped(value: move value);
      region {
        let actual = read(value: &wrapped);
        if actual == 17_u64 {
          return exit_status(code: 0_u8);
        }
      }
    }
  }
  return exit_status(code: 1_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn brand_parameters_follow_explicit_nested_container_arguments() {
    let source = br#"fn read['a, 'b](values: &FixedVector<Vector<'a, Box<'b, u8>>, 2>) -> result: own u64 reads(values) {
  return len_of(deref(values));
}

fn relay['x, 'y](values: &FixedVector<Vector<'x, Box<'y, u8>>, 2>) -> result: own u64 reads(values) {
  return read(values: values);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn brand_parameters_preserve_resolved_elided_store_positions() {
    let source = br#"struct Wrap['s] {
  value: Vector<u8>;
}

fn package['s](value: own Vector<'s, u8>) -> result: own Wrap<'s> pure {
  return Wrap(value: move value);
}

fn entry_heap_reader(value: &Vector<u8>) -> result: own u64 reads(value) {
  return len_of(deref(value));
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn brand_parameters_do_not_change_single_loan_spelling_or_legacy_arena_support() {
    assert_rule_kind(
        br#"fn read['r](value: &'r u64) -> result: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Form8,
        |kind| matches!(kind, SemanticIssueKind::RegionSpelling { .. }),
    );
    assert_unsupported(
        br#"fn read(value: &arena<u64>) -> result: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
}

#[test]
fn brand_parameters_do_not_leak_nominal_defaults_into_nested_declarations() {
    for (field, region_parameters) in [("Inner<u64>", ""), ("Inner<'s, 's, u64>", "['a, 'b]")] {
        // Inner's symbolic instance is checked first; Outer later completes
        // concrete Inner<u64> under its sole-region field context. Both
        // Inner's fields and a later parameter keep their entry-heap brand.
        let source = format!(
            "struct Inner<T: affine>{region_parameters} {{\n  values: Vector<u8>;\n  payload: T;\n}}\n\nstruct Outer['s] {{\n  inner: {field};\n}}\n\nfn read(value: &Vector<u8>) -> result: own u64 reads(value) {{\n  return len_of(deref(value));\n}}\n\nfn inspect['s](value: &Outer<'s>) -> result: own u64 pure {{\n  return 0_u64;\n}}\n\ncommand fn main(command.heap as heap: own Heap) -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n"
        );
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("nested defaults must remain declaration-local: {outcome:?}");
            };
            let inner = checked
                .data
                .nominals
                .iter()
                .filter(|nominal| nominal.name.starts_with("Inner<"))
                .collect::<Vec<_>>();
            assert!(!inner.is_empty());
            for nominal in inner {
                let CheckedNominalKind::Struct { fields } = &nominal.kind else {
                    panic!("Inner is a struct");
                };
                assert!(
                    matches!(
                        fields[0].ty,
                        CheckedType::Vector {
                            region: crate::DeclarationId::ENTRY_HEAP_REGION,
                            ..
                        }
                    ),
                    "a zero-/two-region declaration keeps the entry-heap default: {:?}",
                    fields[0].ty
                );
            }
            let read = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == "read")
                .unwrap();
            assert!(matches!(
                read.parameters[0].ty,
                CheckedType::Vector {
                    region: crate::DeclarationId::ENTRY_HEAP_REGION,
                    ..
                }
            ));
        });
    }
}

#[test]
fn brand_parameters_reject_different_actuals_inside_one_operand() {
    for (declarations, expected, actual) in [
        (
            "struct Pair['a, 'b] {\n  value: u64;\n}\n\n",
            "Pair<'s, 's>",
            "Pair<'a, 'b>",
        ),
        (
            "",
            "FixedVector<Vector<'s, Box<'s, u8>>, 2>",
            "FixedVector<Vector<'a, Box<'b, u8>>, 2>",
        ),
    ] {
        let source = format!(
            "{declarations}fn inspect['s](value: &{expected}) -> result: own u64 pure {{\n  return 0_u64;\n}}\n\nfn caller['a, 'b](value: &{actual}) -> result: own u64 pure {{\n  return inspect(value: value);\n}}\n\ncommand fn main() -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Type5, |kind| {
            matches!(kind, SemanticIssueKind::TypeMismatch { .. })
        });
    }
}

#[test]
fn brand_parameters_keep_opaque_type_arguments_out_of_region_inference() {
    let source = br#"struct Mark['s] {
  value: u64;
}

struct Wrap<T: affine>['s] {
  payload: T;
}

fn pack<T: affine>['s](value: own T) -> result: own Wrap<'s, T> pure {
  return Wrap<'s, T>(payload: move value);
}

command fn main() -> status: own ExitStatus pure {
  region 'outer {
    let value = Mark<'outer>(value: 9_u64);
    region 'inner {
      let wrapped = pack::<'inner, Mark<'outer>>(value: move value);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
    let trailing = std::str::from_utf8(source).unwrap().replace(
        "pack::<'inner, Mark<'outer>>",
        "pack::<Mark<'outer>, 'inner>",
    );
    assert_rule_kind(trailing.as_bytes(), SemanticRule::Fn2, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

#[test]
fn brand_parameters_cannot_be_shortened_by_a_mode_loan_in_either_order() {
    for reversed in [false, true] {
        let (parameters, arguments) = if reversed {
            (
                "loan: &'s u64, marker: own Mark<'s>",
                "loan: &number, marker: move marker",
            )
        } else {
            (
                "marker: own Mark<'s>, loan: &'s u64",
                "marker: move marker, loan: &number",
            )
        };
        let source = format!(
            "struct Mark['s] {{\n  value: u64;\n}}\n\nfn carry['s]({parameters}) -> result: own Mark<'s> pure {{\n  return move marker;\n}}\n\ncommand fn main() -> status: own ExitStatus pure {{\n  region 'outer {{\n    let marker = Mark<'outer>(value: 7_u64);\n    let number = 9_u64;\n    region {{\n      let result = carry({arguments});\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Own4, |kind| {
            matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. })
        });
    }
}

#[test]
fn brand_parameters_allow_longer_mode_loans_in_either_order() {
    for reversed in [false, true] {
        let (parameters, arguments) = if reversed {
            (
                "loan: &'s u64, marker: own Mark<'s>",
                "loan: &'outer number, marker: move marker",
            )
        } else {
            (
                "marker: own Mark<'s>, loan: &'s u64",
                "marker: move marker, loan: &'outer number",
            )
        };
        let source = format!(
            "struct Mark['s] {{\n  value: u64;\n}}\n\nfn carry['s]({parameters}) -> result: own Mark<'s> pure {{\n  return move marker;\n}}\n\ncommand fn main() -> status: own ExitStatus pure {{\n  let number = 9_u64;\n  region 'outer {{\n    region 'inner {{\n      let marker = Mark<'inner>(value: 7_u64);\n      let result = carry({arguments});\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
        );
        with_semantics(source.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{outcome:?}"
            )
        });
    }
}

#[test]
fn brand_parameters_do_not_infer_input_brands_from_a_result() {
    let source = br#"struct Mark['s] {
  value: u64;
}

fn make['r](loan: &'r u64) -> result: own Mark<'r> reads(loan) {
  return Mark<'r>(value: deref(loan));
}

fn add['r](left: &'r u64, right: &'r u64) -> result: own u64 reads(left, right) {
  let first = deref(left);
  let second = deref(right);
  return first +wrap second;
}

command fn main() -> status: own ExitStatus pure {
  let first = 7_u64;
  region 'outer {
    let second = 9_u64;
    region {
      let marker = make(loan: &second);
      let sum = add(left: &'outer first, right: &second);
      if sum == 16_u64 {
        return exit_status(code: 0_u8);
      }
    }
  }
  return exit_status(code: 1_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn leading_call_regions_do_not_change_generic_cycle_judgments() {
    let source = br#"struct Mark<T: affine>['s] {
  value: T;
}

fn recur<T: affine, const n: u64>['s](value: own T) -> result: own Mark<'s, T> pure {
  return recur::<'s, T, n>(value: move value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
    let expanded = std::str::from_utf8(source)
        .unwrap()
        .replace("recur::<'s, T, n>", "recur::<'s, FixedVector<T, n>, n>");
    assert_rule_kind(expanded.as_bytes(), SemanticRule::Fn6, |kind| {
        matches!(kind, SemanticIssueKind::PolymorphicRecursion { .. })
    });
}

#[test]
fn a_captured_generic_box_brand_does_not_infer_a_different_store() {
    let source = br#"fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn wrong['a, 'b](wanted: &Box<'a, u64>, given: own Box<'b, u64>, witness: &Box<'b, u64>) -> result: own Box<'a, u64> pure {
  return pass::<Box<'a, u64>>(value: move given);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Type5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

#[test]
fn explicit_int_generic_function_builds_each_reachable_concrete_instance() {
    let source = br#"fn identity<T: Int>(value: own T) -> result: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let first = identity::<u32>(value: 7_u32);
  let second = identity::<i64>(value: -9_i64);
  return exit_status(code: 0_u8);
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
    let source = br#"fn maximum<T: Int>(left: own T, right: own T) -> result: own T pure {
  return imax(left, right);
}

command fn main() -> status: own ExitStatus pure {
  let small = maximum::<u8>(left: 4_u8, right: 9_u8);
  let signed = maximum::<i64>(left: -7_i64, right: -2_i64);
  return exit_status(code: 0_u8);
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
    let source = br#"fn nudge<T: Float>(value: own T) -> result: own T pure {
  let zero = 0_T;
  let one = 1_T;
  let shifted = fadd.strict(value, one);
  return fadd.strict(zero, shifted);
}

command fn main() -> status: own ExitStatus pure {
  let single = nudge::<f32>(value: 2.0_f32);
  let double = nudge::<f64>(value: 4.0_f64);
  return exit_status(code: 0_u8);
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
    let source = br#"fn identity<T: Float>(value: own T) -> result: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let invalid = identity::<u32>(value: 7_u32);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Fn3, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

#[test]
fn numeric_identity_requires_an_int_or_float_bound() {
    let source = br#"fn invalid<T: affine>() -> result: own T pure {
  return 0_T;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Form5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

#[test]
fn int_bound_identity_is_concretized_before_lowering() {
    let source = br#"fn one<T: Int>() -> result: own T pure {
  return 1_T;
}

command fn main() -> status: own ExitStatus pure {
  let value = one::<u16>();
  return exit_status(code: 0_u8);
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
    let source = br#"fn convert<T: Int>(value: own T) -> result: own unit pure {
  cvt::<T, u64>(value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::Generics);
}

#[test]
fn int_bound_rejects_a_non_integer_explicit_argument_under_fn3() {
    let source = br#"fn identity<T: Int>(value: own T) -> result: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let input = True();
  let invalid = identity::<Bool>(value: input);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Fn3, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

/// The positive control for the three [FN-6] rejections below: a cycle whose
/// every call does instantiate the callee at exactly the caller's own
/// parameters is *permitted* by FN-6, and it is also finite — the call mints
/// no instance the caller is not already at — so it monomorphizes to the one
/// instance the program reaches rather than stopping as an unimplemented
/// capability.
#[test]
fn a_generic_call_cycle_at_the_callers_own_parameters_monomorphizes() {
    let source = br#"fn recursive<T: Int>(value: own T) -> result: own T pure {
  return recursive::<T>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  let seen = recursive::<u16>(value: 1_u16);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a cycle at the caller's own parameters must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 2);
    });
}

/// The one generic cycle that still stops: [FN-6]'s syntactic rule speaks of
/// *type* parameters, so it does not refuse a call that derives its const
/// argument from the caller's own const parameter. Each such call mints a
/// second instance, which mints a third, and the instantiation worklist does
/// not terminate. That is an unimplemented capability of this compiler and is
/// reported as one, never as a source rejection.
#[test]
fn a_generic_cycle_varying_a_const_argument_stops_before_instance_enumeration() {
    let source = br#"fn grow<const n: u64>(at: own u64) -> total: own u64 pure {
  let done = at == 0_u64;
  if done {
    return 0_u64;
  }
  let next = at -wrap 1_u64;
  let rest = grow::<n + 1>(at: next);
  return rest +wrap 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  let total = grow::<1>(at: 3_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::Generics);
}

/// [FN-6] recursion is permitted; polymorphic recursion is rejected by a
/// syntactic rule. A green run here establishes only that these three written
/// shapes are attributed to FN-6 at the offending call with the cycle named;
/// it says nothing about monomorphizing the cycles FN-6 permits, which the
/// control above still reports as unimplemented.
#[test]
fn polymorphic_recursion_is_rejected_at_the_call_that_leaves_the_caller_parameters() {
    let fixed_type = SemanticIssueKind::PolymorphicRecursion {
        cycle: "poly -> poly".to_owned(),
        mechanical_fix: "instantiate every call on the cycle at exactly the caller's own type parameters, or move the differently instantiated call off the cycle",
    };
    // The conformance corpus's own case bytes: the recursive call instantiates
    // the callee at a fixed `i32` instead of the caller's `T`.
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn6-neg-polymorphic-recursion.wf"),
        SemanticRule::Fn6,
        fixed_type.clone(),
    );
    // A growing argument is the shape that would actually diverge: each
    // instance would demand a strictly larger one.
    assert_rule(
        br#"fn poly<T: affine>(x: own T) -> result: own T pure {
  let y = poly::<FixedVector<T, 2>>(x: x);
  return x;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn6,
        fixed_type,
    );
    // A permutation cycle terminates, and FN-6 is deliberately stronger than
    // finiteness requires, so it is rejected all the same.
    assert_rule(
        br#"fn left<A: affine, B: affine>(first: own A, second: own B) -> result: own A pure {
  let swapped = right::<B, A>(first: second, second: first);
  return first;
}

fn right<A: affine, B: affine>(first: own A, second: own B) -> result: own A pure {
  let back = left::<A, B>(first: first, second: second);
  return first;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn6,
        SemanticIssueKind::PolymorphicRecursion {
            cycle: "left -> right -> left".to_owned(),
            mechanical_fix: "instantiate every call on the cycle at exactly the caller's own type parameters, or move the differently instantiated call off the cycle",
        },
    );
}

/// A cycle through a nongeneric participant is not a cycle *among generic
/// functions*, and it cannot diverge: a nongeneric caller has no type
/// parameter to write, so its written argument is fixed and the instance set
/// is finite. FN-6 therefore forms no candidate, and the stop stays the
/// unimplemented-capability report.
#[test]
fn a_cycle_through_a_nongeneric_caller_is_not_polymorphic_recursion() {
    let source = br#"fn poly<T: affine>(x: own T) -> result: own T pure {
  let back = trampoline();
  return x;
}

fn trampoline() -> result: own i32 pure {
  let forward = poly::<i32>(x: 0_i32);
  return forward;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::Generics);
}

#[test]
fn unused_int_generic_body_is_checked_for_the_complete_bound_domain() {
    let source = br#"fn invalid<T: Int>(value: own T) -> result: own T pure {
  return 0_u8;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}

/// [FN-2, OWN-1, S37] the template is the spelling authority.
///
/// This test asserted the opposite until the owner's 2026-09-05 ruling: an
/// `affine`-bounded body writing `move value` was an [OWN-1] `MoveOfCopy`
/// rejection at every copy instance, so no generic body could serve a copy
/// type and an affine type, and the library dodged it by instantiating only at
/// affine types. Under [S37] the body is checked once at the symbolic instance
/// under its written bound, and the concrete-instance recheck does not
/// re-judge the [OWN-1]/[FORM-1] spelling: `move` of a template-affine value
/// at a copy instance denotes a copy. The instance recheck still rejects what
/// is invalid *at the instance* — the conformance corpus keeps that in
/// `const1-neg-eval-overflow`, whose body is admitted symbolically and refused
/// at the instantiation whose value leaves the const domain.
#[test]
fn a_move_in_an_affine_bounded_body_denotes_a_copy_at_a_copy_instance() {
    let source = br#"fn transfer<T: affine>(value: own T) -> result: own T pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let copied = transfer::<u8>(value: 7_u8);
  let payload = Some<u8>(value: 3_u8);
  let held = transfer::<Option<u8>>(value: move payload);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("one affine-bounded body must serve a copy and an affine instance: {outcome:?}");
        };
        // The template, its copy instance, its affine instance, and `main`.
        assert_eq!(checked.function_count(), 3);
    });
}

#[test]
fn nested_generic_calls_discover_reachable_instances_after_template_checking() {
    let source = br#"fn select<T: Int>(value: own T) -> result: own T pure {
  return imax(value, value);
}

fn forward<T: Int>(value: own T) -> result: own T pure {
  return select::<T>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  let small = forward::<u8>(value: 7_u8);
  let signed = forward::<i64>(value: -9_i64);
  return exit_status(code: 0_u8);
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
    let source =
        br#"fn preserve<const n: u64>(value: own FixedVector<u8, n>) -> result: own FixedVector<u8, n> reads(value) {
  let size = len_of(value);
  return move value;
}

fn forward<const n: u64>(value: own FixedVector<u8, n>) -> result: own FixedVector<u8, n> reads(value) {
  return preserve::<n>(value: move value);
}

command fn main() -> status: own ExitStatus pure {
  let small_input = fixed_vector::<u8, 2>();
  let small = forward::<2>(value: move small_input);
  let large_input = fixed_vector::<u8, 5>();
  let large = forward::<5>(value: move large_input);
  let small_held = len_of(small);
  if 1_u64 < small_held {
    let first = small[1_u64];
  }
  let large_held = len_of(large);
  if 4_u64 < large_held {
    let second = large[4_u64];
  }
  return exit_status(code: 0_u8);
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
    let source = br#"fn marker<T: affine>() -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  marker::<u8>();
  marker::<Bool>();
  return exit_status(code: 0_u8);
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
    assert_rule_kind(
        br#"fn marker<T: affine>() -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  marker::<4>();
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule_kind(
        br#"fn sized<const n: u64>() -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  sized::<u8>();
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule(
        br#"fn invalid<const n: Bool>() -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn duplicate<T: Int>(value: own T) -> result: own Pair<T> pure {
  return Pair<T>(left: value, right: value);
}

command fn main() -> status: own ExitStatus pure {
  let small = duplicate::<u8>(value: 7_u8);
  let wide = duplicate::<i64>(value: -9_i64);
  let small_left = small.left;
  let wide_right = wide.right;
  return exit_status(code: 0_u8);
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

command fn main() -> status: own ExitStatus pure {
  let small = Present<u8>(value: 3_u8);
  match small {
    Missing() => {
      let ignored = unit;
    }
    Present(value: observed) => {
      let retained = observed;
    }
  }
  let wide = Present<i64>(value: -5_i64);
  match wide {
    Missing() => {
      let ignored = unit;
    }
    Present(value: observed) => {
      let retained = observed;
    }
  }
  return exit_status(code: 0_u8);
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
  bytes: FixedVector<u8, n>;
}

struct Holder<T: affine> {
  value: T;
}

command fn main() -> status: own ExitStatus pure {
  let short_bytes = fixed_vector::<u8, 2>();
  let short = Packet<2>(bytes: move short_bytes);
  let long_bytes = fixed_vector::<u8, 5>();
  let long = Packet<5>(bytes: move long_bytes);
  let held = Holder<Packet<2>>(value: move short);
  return exit_status(code: 0_u8);
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
                    CheckedType::FixedVector {
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

/// [FN-2] substitutes a callee's actual region through every result position,
/// including a nominal stored as one flat element of a run and the
/// compiler-owned result-list nominal around a multi-result return. The two
/// types in the `replace_one` call must therefore name `compose`'s `'s`, even
/// though the printed formal spelling is identical either way.
#[test]
fn call_results_substitute_regions_inside_flat_run_elements() {
    let source = br#"struct Entry['s] {
  payload: Box<'s, u64>;
}

fn build['s](first: own Box<'s, u64>, replacement: own Box<'s, u64>) -> (slots: own FixedVector<Option<Entry<'s>>, 1>, returned: own Box<'s, u64>) pure contract {
  ensures len_of(slots) == 1_u64;
} {
  let entry = Entry(payload: move first);
  let occupied = Some<Entry<'s>>(value: move entry);
  let slots = fixed_vector::<Option<Entry<'s>>, 1>();
  set slots = place_back(vector: move slots, value: move occupied);
  return move slots, move replacement;
}

fn replace_one['s](slots: own FixedVector<Option<Entry<'s>>, 1>, replacement: own Entry<'s>) -> (updated: own FixedVector<Option<Entry<'s>>, 1>, previous: own Option<Entry<'s>>) reads(slots), writes(slots) contract {
  requires 1_u64 <= len_of(slots);
  ensures len_of(updated) == len_of(slots);
} {
  let occupied = Some<Entry<'s>>(value: move replacement);
  let previous = replace slots[0_u64] = move occupied;
  return move slots, move previous;
}

fn compose['s](first: own Box<'s, u64>, replacement: own Box<'s, u64>) -> (updated: own FixedVector<Option<Entry<'s>>, 1>, previous: own Option<Entry<'s>>) reads(first), writes(first) {
  let (slots, returned) = build(first: move first, replacement: move replacement);
  let entry = Entry(payload: move returned);
  let (updated, previous) = replace_one(slots: move slots, replacement: move entry);
  return move updated, move previous;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("nested result-region substitution must check: {outcome:?}");
        };
    });
}

/// [FORM-8] reads every result ordinal as an output position. Each region in
/// this swapped pair occurs first at an input and again at the opposite result
/// ordinal; in particular, `'left` receives its required second occurrence
/// only from ordinal one.
#[test]
fn multi_result_region_spelling_reads_every_ordinal() {
    let source = br#"struct Holder['s] {
  cell: Box<'s, u64>;
}

fn reverse['left, 'right](left: own Holder<'left>, right: own Holder<'right>) -> (first: own Holder<'right>, second: own Holder<'left>) pure {
  return move right, move left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("every result ordinal must participate in FORM-8: {outcome:?}");
        };
    });
}

/// The same declaration still writes its region-parameter list in first-use
/// order. Swapping the list is a FORM-8 rejection even though the result
/// ordinals themselves intentionally reverse the value flow.
#[test]
fn multi_result_region_spelling_keeps_first_occurrence_order() {
    assert_rule_kind(
        br#"struct Holder['s] {
  cell: Box<'s, u64>;
}

fn reverse['right, 'left](left: own Holder<'left>, right: own Holder<'right>) -> (first: own Holder<'right>, second: own Holder<'left>) pure {
  return move right, move left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Form8,
        |kind| matches!(kind, SemanticIssueKind::RegionSpelling { .. }),
    );
}

/// The same recursive position is an input position for [FORM-8], so its
/// region is inferred from the argument and writing it at the call is the
/// ordinary canonical-spelling rejection.
#[test]
fn nested_nominal_parameter_regions_are_inferred_at_calls() {
    assert_rule_kind(
        br#"struct Entry['s] {
  payload: Box<'s, u64>;
}

fn pass['s](slots: own FixedVector<Option<Entry<'s>>, 1>) -> result: own FixedVector<Option<Entry<'s>>, 1> pure {
  return move slots;
}

fn caller['s](slots: own FixedVector<Option<Entry<'s>>, 1>) -> result: own FixedVector<Option<Entry<'s>>, 1> pure {
  return pass::<'s>(slots: move slots);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Form8,
        |kind| matches!(kind, SemanticIssueKind::RegionSpelling { .. }),
    );
}

/// The inferred store region is substituted through the complete parameter
/// type before TYPE-5 compares it with each actual. Reusing one helper at
/// different indexed places must not depend on the first nominal occurrence.
#[test]
fn nested_borrowed_parameter_regions_are_inferred_at_each_call_position() {
    let source = br#"struct Entry['s] {
  payload: Box<'s, u64>;
}

fn inspect['s](entry: &Option<Entry<'s>>, same: &Option<Entry<'s>>) -> result: own u64 pure {
  return 0_u64;
}

fn inspect_positions['s](entries: own FixedVector<Option<Entry<'s>>, 4>) -> result: own FixedVector<Option<Entry<'s>>, 4> pure contract {
  requires 3_u64 <= len_of(entries);
} {
  region {
    let first = inspect(entry: &entries[0_u64], same: &entries[0_u64]);
    let later = inspect(entry: &entries[2_u64], same: &entries[2_u64]);
  }
  return move entries;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("nested borrowed parameter regions must check at every call: {outcome:?}");
        };
    });
}

/// Two distinct store regions survive the same nested result substitution in
/// declaration order. Every explicit brand position of the nested nominal
/// determines its formal, so neither region is written at the call.
#[test]
fn nested_results_preserve_two_distinct_region_arguments() {
    let source = br#"struct Pair['left, 'right] {
  left: Box<'left, u64>;
  right: Box<'right, u64>;
}

fn build['left, 'right](left: own Box<'left, u64>, right: own Box<'right, u64>) -> slots: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  let pair = Pair(left: move left, right: move right);
  let occupied = Some<Pair<'left, 'right>>(value: move pair);
  let slots = fixed_vector::<Option<Pair<'left, 'right>>, 1>();
  set slots = place_back(vector: move slots, value: move occupied);
  return move slots;
}

fn pass['left, 'right](slots: own FixedVector<Option<Pair<'left, 'right>>, 1>) -> result: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  return move slots;
}

fn compose['left, 'right](left: own Box<'left, u64>, right: own Box<'right, u64>) -> slots: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  let slots = build(left: move left, right: move right);
  return pass(slots: move slots);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("both nested result regions must retain their positions: {outcome:?}");
        };
    });
}

/// An independent anchor fixes the relationship between the nested brands.
/// Swapping the owners cannot relabel them: exact [TYPE-5] rejects the handoff.
#[test]
fn nested_results_reject_crossed_region_handoffs() {
    assert_rule_kind(
        br#"struct Pair['left, 'right] {
  left: Box<'left, u64>;
  right: Box<'right, u64>;
}

fn build['left, 'right](left: own Box<'left, u64>, right: own Box<'right, u64>) -> slots: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  let pair = Pair(left: move left, right: move right);
  let occupied = Some<Pair<'left, 'right>>(value: move pair);
  let slots = fixed_vector::<Option<Pair<'left, 'right>>, 1>();
  set slots = place_back(vector: move slots, value: move occupied);
  return move slots;
}

fn pass['left, 'right](slots: own FixedVector<Option<Pair<'left, 'right>>, 1>, anchor: &Box<'left, u64>) -> result: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  return move slots;
}

fn crossed['left, 'right](left: own Box<'left, u64>, right: own Box<'right, u64>, witness: &Box<'left, u64>) -> slots: own FixedVector<Option<Pair<'left, 'right>>, 1> pure {
  let slots = build(left: move right, right: move left);
  return pass(slots: move slots, anchor: witness);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn source_nominal_argument_arity_and_kinds_are_exact() {
    assert_rule_kind(
        br#"struct Pair<T: affine> {
  value: T;
}

command fn main() -> status: own ExitStatus pure {
  let invalid = Pair<u8, u16>(value: 1_u8);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule_kind(
        br#"struct Packet<const n: u64> {
  bytes: FixedVector<u8, n>;
}

command fn main() -> status: own ExitStatus pure {
  let bytes = fixed_vector::<u8, 1>();
  let invalid = Packet<u8>(bytes: move bytes);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn constructor_only_generic_instances_still_reach_normal_type_diagnostics() {
    assert_rule(
        br#"struct Holder<T: affine> {
  value: T;
}

command fn main() -> status: own ExitStatus pure {
  return Holder<u8>(value: 1_u8);
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
}

/// TYPE-2 admits a symbolic owning array element under its declared bound.
/// The former flat-only rejection is superseded by the full-array amendment.
#[test]
fn unused_generic_array_members_admit_their_declared_owning_bound() {
    with_semantics(
        br#"struct Holder<T: affine> {
  values: array<T, 2>;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "generic owning array: {outcome:?}"
            )
        },
    );
}

#[test]
fn recursive_generic_nominal_layouts_stop_before_concrete_enumeration() {
    assert_unsupported(
        br#"struct Recursive<T: affine> {
  next: Recursive<T>;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"fn checked_sum<T: Int>(left: own T, right: own T) -> result: own Result<T, Overflow> pure {
  return left +checked right;
}

command fn main() -> status: own ExitStatus pure {
  let small = checked_sum::<u8>(left: 1_u8, right: 2_u8);
  let wide = checked_sum::<i64>(left: -3_i64, right: 5_i64);
  return exit_status(code: 0_u8);
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
fn numeric_and_const_parameters_flow_through_container_operations() {
    let source = br#"fn filled_run<T: Int, const n: u64>(value: own T) -> result: own FixedVector<T, n> pure contract {
  ensures len_of(result) >= n;
} {
  let built = fixed_vector::<T, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len_of(built) >= at,
    invariant spare: room_of(built) + at >= n,
    invariant flat: head_of(built) <= 0_u64
  ) {
    set built = place_back(vector: move built, value: value);
  }
  return move built;
}

fn filled_float_run<T: Float, const n: u64>(value: own T) -> result: own FixedVector<T, n> pure contract {
  ensures len_of(result) >= n;
} {
  let built = fixed_vector::<T, n>();
  for @fill (
    at in 0_u64..n,
    invariant grown: len_of(built) >= at,
    invariant spare: room_of(built) + at >= n,
    invariant flat: head_of(built) <= 0_u64
  ) {
    set built = place_back(vector: move built, value: value);
  }
  return move built;
}

fn store_run<T: Int>(store: &uniq Heap, length: own u64) -> result: own u64 reads(store), writes(store), allocates(store) contract {
  requires buffer_fits::<T>(length);
} {
  region {
    match heap_vector::<T>(store: &uniq deref(store), count: length) {
      Some(value: fresh) => {
        return cap_of(fresh);
      }
      None() => {
        return 0_u64;
      }
    }
  }
}

fn float_store_run<T: Float>(store: &uniq Heap, length: own u64) -> result: own u64 reads(store), writes(store), allocates(store) contract {
  requires buffer_fits::<T>(length);
} {
  region {
    match heap_vector::<T>(store: &uniq deref(store), count: length) {
      Some(value: fresh) => {
        return cap_of(fresh);
      }
      None() => {
        return 0_u64;
      }
    }
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let bytes = filled_run::<u8, 2>(value: 7_u8);
  let words = filled_run::<i64, 3>(value: -5_i64);
  let byte = bytes[1_u64];
  let word = words[2_u64];
  let samples = filled_float_run::<f32, 2>(value: 1.5_f32);
  let sample = samples[1_u64];
  region {
    let storage_room = store_run::<u16>(store: &uniq heap, length: 2_u64);
  }
  region {
    let weights_room = float_store_run::<f64>(store: &uniq heap, length: 2_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic container rows must check and concretize: {outcome:?}");
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
        br#"fn instantiate<T: affine>() -> result: own unit pure {
  return unit;
}

fn invalid() -> result: own unit pure {
  instantiate::<Slice<u8>>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"struct Marker<T: affine> {
}

fn invalid(value: own Marker<Slice<u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid(value: own Option<Slice<u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        expected.clone(),
    );
    assert_rule(
        br#"fn instantiate<T: affine>() -> result: own unit pure {
  return unit;
}

fn invalid() -> result: own unit pure {
  instantiate::<Arena<64, 8>>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn2,
        expected,
    );
}

/// A concrete nominal written inside an otherwise symbolic function remains a
/// concrete descendant of that source schema.  Rebuilding the concrete
/// inventory must therefore retain both the nominal and the callee instance.
#[test]
fn schema_written_concrete_nominal_arguments_are_rebuilt_after_the_symbolic_checkpoint() {
    let source = br#"struct Pair<T: Int> {
  value: T;
}

fn consume<T: affine>(value: own T) -> result: own unit pure {
  return unit;
}

fn wrapper<U: affine>() -> result: own unit pure {
  let pair = Pair<u8>(value: 1_u8);
  consume::<Pair<u8>>(value: move pair);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the concrete nominal substitution must be rebuilt: {outcome:?}");
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

/// A partially concrete call chain contributes only the nominal arguments
/// whose complete type is known at the symbolic checkpoint.
#[test]
fn partial_schema_rebuild_keeps_only_the_truly_concrete_nominal_instance() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn sink<T: affine>() -> result: own unit pure {
  return unit;
}

fn middle<A: Int, B: affine>() -> result: own unit pure {
  sink::<Pair<A>>();
  return unit;
}

fn wrapper<U: affine>() -> result: own unit pure {
  middle::<u8, U>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("partial rebuilding must retain concrete descendants: {outcome:?}");
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
        lower_checked(*program, OverlapLowering::Off)
            .expect("the concrete-only rebuilt inventory must lower");
    });
}

/// A symbolic argument in one position must not hide an independent concrete
/// descendant in another position of the same call.
#[test]
fn partial_schema_rebuild_still_discovers_an_independent_concrete_descendant() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn sink<T: affine>() -> result: own unit pure {
  return unit;
}

fn next<X: affine, Y: affine>() -> result: own unit pure {
  sink::<Y>();
  return unit;
}

fn middle<A: Int>() -> result: own unit pure {
  next::<Pair<A>, u8>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("partial rebuilding must project its concrete descendant: {outcome:?}");
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
        lower_checked(*program, OverlapLowering::Off)
            .expect("the concrete descendant inventory must lower");
    });
}

/// Source-order diagnostics stay source-stable even when an earlier generic
/// body receives a dense concrete instance identity during inventory build.
#[test]
fn ordinary_admission_diagnostics_prefer_source_order_over_instance_identity() {
    let source =
        br#"fn earlier<T: affine>(values: own FixedVector<u8, 4>, index: own u64) -> result: own u8 reads(values) {
  return values[index];
}

fn later(values: own FixedVector<u8, 4>, index: own u64) -> result: own u8 reads(values) {
  return values[index];
}

command fn main() -> status: own ExitStatus pure {
  let first_values = fixed_vector::<u8, 4>();
  let second_values = fixed_vector::<u8, 4>();
  earlier::<u8>(values: move first_values, index: 5_u64);
  later(values: move second_values, index: 5_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("both bodies contain an OP-4 rejection: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
        let crate::SemanticLocation::SourceNode(path, _) = issue.location() else {
            panic!("OP-4 must cite the source operation");
        };
        assert_eq!(path.components().first(), Some(&0));
    });
}

/// A wrapper's type argument can carry its entire region axis. The physical
/// family erases that axis and defers release selection; ordinary aliases must
/// still distinguish the Box and Vector cleanup classes nested inside it.
#[test]
fn nominal_physical_families_reach_regions_inside_type_arguments_and_result_lists() {
    let source = br#"struct Wrapped<T: linear> {
  value: T;
}

fn first['a: affine](value: own Wrapped<Result<Box<'a, u64>, Vector<'a, u64>>>, spare: own Box<'a, u64>) -> (back: own Wrapped<Result<Box<'a, u64>, Vector<'a, u64>>>, spare: own Box<'a, u64>) pure {
  return move value, move spare;
}

fn second['b: affine](value: own Wrapped<Result<Box<'b, u64>, Vector<'b, u64>>>, spare: own Box<'b, u64>) -> (back: own Wrapped<Result<Box<'b, u64>, Vector<'b, u64>>>, spare: own Box<'b, u64>) pure {
  return move value, move spare;
}

fn general['g](value: own Wrapped<Result<Box<'g, u64>, Vector<'g, u64>>>, spare: own Box<'g, u64>) -> (back: own Wrapped<Result<Box<'g, u64>, Vector<'g, u64>>>, spare: own Box<'g, u64>) pure {
  return move value, move spare;
}

fn renamed['g](value: own Wrapped<Result<Box<'g, u64>, Vector<'g, u64>>>, spare: own Box<'g, u64>) -> (other: own Wrapped<Result<Box<'g, u64>, Vector<'g, u64>>>, spare: own Box<'g, u64>) pure {
  return move value, move spare;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the wrapper and result declarations must check: {outcome:?}");
        };
        let function = |name: &str| {
            checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("each declaration has a checked function")
        };
        for ty in [
            |function: &crate::semantic::model::CheckedFunction| function.parameters[0].ty,
            |function: &crate::semantic::model::CheckedFunction| function.result,
        ] {
            let id = |name| match ty(function(name)) {
                CheckedType::Nominal(id) => id,
                other => panic!("the wrapper or result list must be nominal: {other:?}"),
            };
            let (first, second, general) = (id("first"), id("second"), id("general"));
            assert_ne!(first, second, "source region identities remain distinct");
            assert_eq!(
                checked.data.nominal_lowering_alias[first.0 as usize],
                checked.data.nominal_lowering_alias[second.0 as usize],
                "equal release classes share the ordinary lowering alias"
            );
            assert_ne!(
                checked.data.nominal_lowering_alias[first.0 as usize],
                checked.data.nominal_lowering_alias[general.0 as usize],
                "nested General and Extent releases remain separate"
            );
            assert_eq!(
                checked.data.nominal_physical_alias[first.0 as usize],
                checked.data.nominal_physical_alias[general.0 as usize],
                "one physical family is specialized by the closed release environment"
            );
        }
        let (CheckedType::Nominal(general), CheckedType::Nominal(renamed)) =
            (function("general").result, function("renamed").result)
        else {
            panic!("both multi-result types must be nominal");
        };
        assert_ne!(
            checked.data.nominal_physical_alias[general.0 as usize],
            checked.data.nominal_physical_alias[renamed.0 as usize],
            "result-list ordinal names remain part of the family"
        );
    });
}

/// Layout alone cannot erase source declaration identity or phantom generic
/// arguments. Each of these empty structs has the same storage layout.
#[test]
fn nominal_physical_families_preserve_declarations_and_phantom_arguments() {
    let source = br#"struct Marker<T: affine, const n: u64> {
}

struct AlternateMarker<T: affine, const n: u64> {
}

fn base(value: own Marker<u8, 1>) -> back: own Marker<u8, 1> pure {
  return move value;
}

fn element(value: own Marker<u16, 1>) -> back: own Marker<u16, 1> pure {
  return move value;
}

fn count(value: own Marker<u8, 2>) -> back: own Marker<u8, 2> pure {
  return move value;
}

fn declaration(value: own AlternateMarker<u8, 1>) -> back: own AlternateMarker<u8, 1> pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the empty generic structs must check: {outcome:?}");
        };
        let mut aliases = Vec::new();
        for name in ["base", "element", "count", "declaration"] {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("each declaration has a checked function");
            let CheckedType::Nominal(id) = function.parameters[0].ty else {
                panic!("the input must be an empty nominal");
            };
            let family = checked.data.nominal_physical_alias[id.0 as usize];
            assert!(
                !aliases.contains(&family),
                "{name} must keep a distinct family"
            );
            aliases.push(family);
        }
    });
}

/// This acyclic graph exceeds both former implementation depth cutoffs.
/// Accepted source depth must not change which region instances share a family.
#[test]
fn nominal_physical_families_complete_deep_finite_type_graphs() {
    let mut source = String::from("struct Layer0['s] {\n  cell: Box<'s, u64>;\n}\n\n");
    for depth in 1..=80 {
        source.push_str(&format!(
            "struct Layer{depth}['s] {{\n  inner: Layer{}<'s>;\n}}\n\n",
            depth - 1
        ));
    }
    source.push_str(
        "fn first['a: affine](value: own Layer80<'a>) -> back: own Layer80<'a> pure {\n  return move value;\n}\n\n\
         fn second['b: affine](value: own Layer80<'b>) -> back: own Layer80<'b> pure {\n  return move value;\n}\n\n\
         command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the finite nominal graph must check: {outcome:?}");
        };
        let ids = ["first", "second"].map(|name| {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("both declarations have checked functions");
            let CheckedType::Nominal(id) = function.parameters[0].ty else {
                panic!("the input must be a Layer80 nominal");
            };
            id
        });
        assert_ne!(ids[0], ids[1]);
        for aliases in [
            &checked.data.nominal_lowering_alias,
            &checked.data.nominal_physical_alias,
        ] {
            assert_eq!(aliases[ids[0].0 as usize], aliases[ids[1].0 as usize]);
        }
    });
}

/// A back edge is only one obligation: the release class on another field
/// must still be checked even after the recursive pair has been visited.
#[test]
fn nominal_physical_families_complete_cycles_and_check_remaining_fields() {
    let source = br#"enum Tree['s, 't] {
  Leaf();
  Branch(next: Box<'s, Tree<'s, 't>>, values: Vector<'t, u64>);
}

fn first['a: affine, 'b: affine](value: own Tree<'a, 'b>) -> back: own Tree<'a, 'b> pure {
  return move value;
}

fn second['c: affine, 'd: affine](value: own Tree<'c, 'd>) -> back: own Tree<'c, 'd> pure {
  return move value;
}

fn mixed['e: affine, 'f](value: own Tree<'e, 'f>) -> back: own Tree<'e, 'f> pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the recursive nominal graph must check: {outcome:?}");
        };
        let ids = ["first", "second", "mixed"].map(|name| {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("each declaration has a checked function");
            let CheckedType::Nominal(id) = function.parameters[0].ty else {
                panic!("the input must be a Tree nominal");
            };
            id
        });
        assert_eq!(
            checked.data.nominal_lowering_alias[ids[0].0 as usize],
            checked.data.nominal_lowering_alias[ids[1].0 as usize]
        );
        assert_ne!(
            checked.data.nominal_lowering_alias[ids[0].0 as usize],
            checked.data.nominal_lowering_alias[ids[2].0 as usize],
            "the nonrecursive Vector field still has a different release action"
        );
        assert_eq!(
            checked.data.nominal_physical_alias[ids[0].0 as usize],
            checked.data.nominal_physical_alias[ids[2].0 as usize]
        );
    });
}

/// Concrete generic calls are rebuilt after the symbolic inventory is rolled
/// back. A Box type argument must retain its source store identity through that
/// rebuild, including when the call is inside an uncalled ordinary helper.
#[test]
fn generic_replay_preserves_box_store_brands_and_legacy_boxes() {
    use crate::semantic::model::CheckedReleaseClass;

    let source = br#"fn pass<T: linear>(value: own T) -> result: own T pure {
  return move value;
}

fn first_general['s](cell: own Box<'s, u64>) -> result: own Box<'s, u64> pure {
  return pass::<Box<'s, u64>>(value: move cell);
}

fn second_general['s](cell: own Box<'s, u64>) -> result: own Box<'s, u64> pure {
  return pass::<Box<'s, u64>>(value: move cell);
}

fn extent['s: affine](cell: own Box<'s, u64>) -> result: own Box<'s, u64> pure {
  return pass::<Box<'s, u64>>(value: move cell);
}

fn legacy(cell: own box<u64>) -> result: own box<u64> pure {
  return pass::<box<u64>>(value: move cell);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic replay must preserve each Box type argument: {outcome:?}");
        };
        let mut brands = Vec::new();
        for (name, expected_release, branded) in [
            ("first_general", CheckedReleaseClass::General, true),
            ("second_general", CheckedReleaseClass::General, true),
            ("extent", CheckedReleaseClass::Extent, true),
            ("legacy", CheckedReleaseClass::General, false),
        ] {
            let relay = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("every written relay is checked even though main does not call it");
            let box_type = relay.parameters[0].ty;
            let CheckedType::Nominal(id) = box_type else {
                panic!("{name} must take a Box nominal");
            };
            let CheckedNominalKind::Box {
                region,
                release,
                referent,
            } = checked.data.nominals[id.0 as usize].kind
            else {
                panic!("{name} must retain its Box kind");
            };
            assert_eq!(referent, CheckedType::Integer(IntegerType::U64));
            assert_eq!(release, expected_release);
            if branded {
                let [expected_region] = relay.region_parameters.as_slice() else {
                    panic!("{name} declares one store region");
                };
                assert_eq!(region, Some(*expected_region));
                assert!(
                    !brands.contains(expected_region),
                    "the same written region name in another relay is a different store"
                );
                brands.push(*expected_region);
            } else {
                assert_eq!(region, None, "legacy box has no store brand");
            }
            assert_eq!(relay.result, box_type);
            assert!(
                checked.data.functions.iter().any(|function| {
                    function.name == "pass"
                        && function.parameters[0].ty == box_type
                        && function.result == box_type
                }),
                "{name}'s explicit generic argument must produce an exact pass instance"
            );
        }
    });
}

#[test]
fn general_elements_retain_deep_runs_through_generic_replay_and_nominal_fields() {
    let source = br#"struct Wrapped {
  values: FixedVector<FixedVector<FixedVector<u64, 2>, 2>, 2>;
}

fn pass<T: affine>(value: own T) -> result: own T pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let empty_leaf = fixed_vector::<u64, 2>();
  let leaf = place_back(vector: move empty_leaf, value: 7_u64);
  let empty_middle = fixed_vector::<FixedVector<u64, 2>, 2>();
  let middle = place_back(vector: move empty_middle, value: move leaf);
  let empty_outer = fixed_vector::<FixedVector<FixedVector<u64, 2>, 2>, 2>();
  let outer = place_back(vector: move empty_outer, value: move middle);
  let returned = pass::<FixedVector<FixedVector<FixedVector<u64, 2>, 2>, 2>>(value: move outer);
  let wrapped = Wrapped(values: move returned);
  let retained = pass::<Wrapped>(value: move wrapped);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("deep run elements and the nominal control must check: {outcome:?}");
        };
        let wrapper = checked
            .data
            .nominals
            .iter()
            .find(|nominal| nominal.name == "Wrapped")
            .expect("the source nominal remains in the ordinary inventory");
        let CheckedNominalKind::Struct { fields } = &wrapper.kind else {
            panic!("the wrapper remains a source struct");
        };
        let mut ty = fields[0].ty;
        for _ in 0..3 {
            let CheckedType::FixedVector { element, length } = ty else {
                panic!("every declared nested run must survive the checked graph");
            };
            assert_eq!(length, CheckedConst::Value(2));
            ty = checked
                .element_type(element)
                .expect("a complete checked element");
        }
        assert_eq!(ty, CheckedType::Integer(IntegerType::U64));
    });
}

#[test]
fn general_elements_reify_nominal_children_after_the_schema_checkpoint() {
    let source = br#"struct Pair<T: Int> {
  value: T;
}

fn consume<T: affine>(value: own T) -> result: own unit pure {
  return unit;
}

fn wrapper<U: affine>() -> result: own unit pure {
  let pair = Pair<u8>(value: 7_u8);
  let empty_inner = fixed_vector::<Pair<u8>, 1>();
  let inner = place_back(vector: move empty_inner, value: move pair);
  let empty_outer = fixed_vector::<FixedVector<Pair<u8>, 1>, 1>();
  let outer = place_back(vector: move empty_outer, value: move inner);
  consume::<FixedVector<FixedVector<Pair<u8>, 1>, 1>>(value: move outer);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("schema-created nominal element children must be reified: {outcome:?}");
        };
        let consume = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "consume")
            .expect("the schema-written concrete call is retained");
        let mut ty = consume.parameters[0].ty;
        for _ in 0..2 {
            let CheckedType::FixedVector { element, length } = ty else {
                panic!("the concrete argument retains both structural layers");
            };
            assert_eq!(length, CheckedConst::Value(1));
            ty = checked
                .element_type(element)
                .expect("a reified element handle");
        }
        let CheckedType::Nominal(id) = ty else {
            panic!("the terminal element remains a source nominal");
        };
        assert!((id.0 as usize) < checked.data.executable_nominal_count);
        let nominal = &checked.data.nominals[id.0 as usize];
        assert!(nominal.name.starts_with("Pair<"));
        let CheckedNominalKind::Struct { fields } = &nominal.kind else {
            panic!("the terminal nominal must retain its original family");
        };
        assert_eq!(fields[0].ty, CheckedType::Integer(IntegerType::U8));
    });
}
