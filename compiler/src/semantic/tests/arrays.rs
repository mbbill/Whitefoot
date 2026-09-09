use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedConst, CheckedContainerRoot, CheckedExpression, CheckedFlatElement, CheckedPlaceStep,
    CheckedSetTarget, CheckedStatement, CheckedTargetDomainObligation, CheckedType, CheckedValue,
    IntegerType,
};
use super::{assert_rule, assert_rule_kind, with_semantics};

/// [TYPE-7, MSR-1] an explicit dereference of an own Box is a measured place,
/// not the implicit read of a borrow holder. The checked path retains the Box
/// step so lowering follows the allocation rather than measuring the pointer
/// slot or a copied run value.
#[test]
fn an_owned_box_referent_is_an_admitted_measured_place() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  region 'a {
    let store = arena_frame::<64, 8, 'a>();
    region {
      let empty = fixed_vector::<u8, 4>();
      let one = place_back(vector: move empty, value: 1_u8);
      match arena_box(store: &uniq store, value: move one) {
        Err(error: back) => {
          return exit_status(code: 1_u8);
        }
        Ok(value: block) => {
          let capacity = cap_of(deref(block));
          let length = len_of(deref(block));
          return exit_status(code: 0_u8);
        }
      }
    }
  }
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an own Box referent is a measure place: {outcome:?}");
        };
        let arm = &checked.data.functions[0].body[0];
        let CheckedStatement::Region { body, .. } = arm else {
            panic!("outer store region must remain checked");
        };
        let CheckedStatement::Region { body, .. } = &body[1] else {
            panic!("inner borrow region must remain checked");
        };
        let CheckedStatement::Match { arms, .. } = &body[2] else {
            panic!("arena_box result must remain a checked match");
        };
        let CheckedStatement::Let {
            value: CheckedExpression::ContainerMeasure { root, .. },
            ..
        } = &arms[1].body[0]
        else {
            panic!("capacity must lower as a container measure");
        };
        assert!(matches!(
            root.path.as_slice(),
            [CheckedPlaceStep::BoxReferent(_)]
        ));
    });
}

#[test]
fn a_bare_owned_box_still_requires_explicit_dereference_to_measure_its_referent() {
    assert_rule_kind(
        br#"command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 4>();
  let block = box_new(move empty);
  let capacity = cap_of(block);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type7,
        |kind| matches!(kind, SemanticIssueKind::MissingDereference { .. }),
    );
}

/// A fact about the old referent must die when the whole Box owner is
/// replaced. Otherwise its old nonempty length could authorize an element
/// read from the new empty run.
#[test]
fn replacing_an_owned_box_kills_its_referents_old_measure() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  region 'a {
    let store = arena_frame::<128, 8, 'a>();
    region {
      let old_empty = fixed_vector::<u8, 1>();
      let old_run = place_back(vector: move old_empty, value: 7_u8);
      match arena_box(store: &uniq store, value: move old_run) {
        Err(error: back) => {
          return exit_status(code: 1_u8);
        }
        Ok(value: block) => {
          region {
            let fresh_run = fixed_vector::<u8, 1>();
            match arena_box(store: &uniq store, value: move fresh_run) {
              Err(error: back) => {
                return exit_status(code: 2_u8);
              }
              Ok(value: fresh) => {
                let old_length = len_of(deref(block));
                let in_old = 0_u64 < old_length;
                if in_old {
                  let previous = replace block = move fresh;
                  let invalid = deref(block)[0_u64];
                  return exit_status(code: invalid);
                } else {
                  return exit_status(code: 3_u8);
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;
    assert_rule_kind(source, SemanticRule::Op4, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedBoundsObligation { .. })
    });
}

/// B7c4b moved this module off `array<T, n>` and `array_new`. The [S34] const
/// run keeps the array place as its storage type — four exact constants over a
/// run of `n` slots, materialized from the type — so a const's checked type is
/// still `CheckedType::Array` and its subscript is still `ArrayIndex` rooted at
/// the constant. Everything a program builds is a `FixedVector`, whose
/// capacity is standing and whose `len_of`, `room_of` and `head_of` are
/// descriptor words, so a built run's measure is `ContainerMeasure`, its
/// subscript is `ReadStorage`, and its indexed commit is `CheckedSetTarget::
/// Storage`, each retaining the complete typed projection.
#[test]
fn constants_fill_length_and_index_share_exact_run_types() {
    let source = br#"const count: u64 = 4_u64;

const table: FixedVector<u8, count> =[10_u8, 20_u8, 30_u8, 40_u8];

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<i32, count>();
  let one = place_back(vector: move empty, value: 7_i32);
  let values = place_back(vector: move one, value: 7_i32);
  let length = len_of(values);
  let local = values[1_u64];
  let stored = table[2_u64];
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("fixed-run family must check: {outcome:?}");
        };
        assert_eq!(checked.data.constants.len(), 2);
        assert_eq!(
            checked.data.constants[1].ty,
            CheckedType::Array {
                element: CheckedFlatElement::Integer(IntegerType::U8),
                length: CheckedConst::Value(4),
            }
        );
        let CheckedValue::Array { elements, .. } = &checked.data.constants[1].value else {
            panic!("table must retain its complete checked initializer");
        };
        assert_eq!(elements.len(), 4);

        let body = &checked.data.functions[0].body;
        assert!(matches!(
            &body[3],
            CheckedStatement::Let {
                value: CheckedExpression::ContainerMeasure {
                    root: CheckedContainerRoot {
                        ty: CheckedType::FixedVector {
                            element,
                            length: CheckedConst::Value(4),
                        },
                        ..
                    },
                    ..
                },
                ..
            } if checked.element_type(*element) == Some(CheckedType::Integer(IntegerType::I32))
        ));
        assert!(matches!(
            &body[4],
            CheckedStatement::Let {
                value: CheckedExpression::ReadStorage {
                    root: CheckedContainerRoot {
                        ty: CheckedType::Integer(IntegerType::I32),
                        path,
                        ..
                    },
                    ..
                },
                ..
            } if matches!(path.as_slice(), [CheckedPlaceStep::Subscript(index)]
                if matches!(index.base_type, CheckedType::FixedVector { length: CheckedConst::Value(4), .. })
                && index.target_domain == CheckedTargetDomainObligation::ElementAddress
                && !index.obligation.components().is_empty())
        ));
        assert!(matches!(
            &body[5],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayIndex {
                    root: super::super::model::CheckedArrayRoot::Constant(_),
                    length: CheckedConst::Value(4),
                    obligation,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            } if !obligation.components().is_empty()
        ));
    });
}

#[test]
fn const_expression_and_const_value_failures_keep_their_rule_owners() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const1-neg-noninteger.wf"),
        SemanticRule::Const1,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"const table: FixedVector<u8, 2> =[1_u8];\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-noneligible.wf"),
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nconst bad: Cell = unit;\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-set.wf"),
        SemanticRule::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule_kind(
        b"command fn main() -> status: own ExitStatus pure {\n  let items = fixed_vector::<u8, 2>();\n  let value = items[0_u32];\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn named_lengths_and_tag_only_enum_elements_work_in_nominal_layouts() {
    let source = br#"const count: u64 = 2_u64;

enum Flag {
  Off();
  On();
}

struct Holder {
  flags: FixedVector<Flag, count>;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "nominal run fields must use earlier lengths and completed enum layouts: {outcome:?}"
            );
        };
        let super::super::model::CheckedNominalKind::Struct { fields } =
            &checked.data.nominals[1].kind
        else {
            panic!("Holder must remain a struct");
        };
        let CheckedType::FixedVector { element, length } = fields[0].ty else {
            panic!("field must be a fixed run");
        };
        assert_eq!(length, CheckedConst::Value(2));
        assert_eq!(
            checked.element_type(element),
            Some(CheckedType::Nominal(checked.data.nominals[0].id))
        );
    });

    // B7c4b left this negative on the retiring surface: [TYPE-2]'s
    // flat-element restriction is `array<T, n>`'s own. A run's element domain
    // is `CheckedElement`, which admits a region-free affine nominal stored by
    // value, so `FixedVector<Payload, 2>` is an accepted field and this
    // property has no twin to be rewritten as. It retires with `array<T, n>`.
    assert_rule_kind(
        b"enum Payload {\n  Item(value: i32);\n}\n\nstruct Holder {\n  values: array<Payload, 2>;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type2,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn indexed_set_retains_its_pre_rhs_guard_and_copy_target() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  set values[1_u64] = 9_u8;
  let stored = values[1_u64];
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("indexed fixed-run set must check: {outcome:?}");
        };
        let CheckedStatement::Set { target, .. } = &checked.data.functions[0].body[3] else {
            panic!("fourth statement must be the indexed set");
        };
        let CheckedSetTarget::Storage(target) = target else {
            panic!("indexed set must retain a run-index target");
        };
        let [CheckedPlaceStep::Subscript(index)] = target.path.as_slice() else {
            panic!("the complete target must retain its subscript");
        };
        let CheckedType::FixedVector { element, length } = index.base_type else {
            panic!("index base must be a fixed run");
        };
        assert_eq!(length, CheckedConst::Value(2));
        assert_eq!(
            checked.element_type(element),
            Some(CheckedType::Integer(IntegerType::U8))
        );
        assert_eq!(target.ty, CheckedType::Integer(IntegerType::U8));
        assert_eq!(index.offset.ty(), CheckedType::Integer(IntegerType::U64));
        assert!(!index.obligation.components().is_empty());
        assert_eq!(
            index.target_domain,
            CheckedTargetDomainObligation::ElementAddress
        );
    });
}

#[test]
fn indexed_set_rechecks_type_effect_and_root_liveness() {
    // A discharged subscript adds no runtime effect: the indexed set with a
    // constant in-range offset is accepted in a `pure` function.
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  let empty = fixed_vector::<u8, 2>();\n  let values = place_back(vector: move empty, value: 0_u8);\n  set values[0_u64] = 1_u8;\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "a discharged indexed set must remain pure: {outcome:?}"
            );
        },
    );
    assert_rule_kind(
        b"command fn main() -> status: own ExitStatus pure {\n  let empty = fixed_vector::<u8, 2>();\n  let values = place_back(vector: move empty, value: 0_u8);\n  set values[0_u64] = 1_u16;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule(
        b"fn consume(values: own FixedVector<u8, 2>) -> result: own u8 pure {\n  return 1_u8;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let empty = fixed_vector::<u8, 2>();\n  let values = place_back(vector: move empty, value: 0_u8);\n  set values[0_u64] = consume(values: move values);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}

#[test]
fn nested_struct_run_places_retain_their_complete_paths() {
    let source = br#"struct Inner {
  values: FixedVector<u8, 2>;
}

struct Outer {
  inner: Inner;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  let length = len_of(outer.inner.values);
  set outer.inner.values[1_u64] = 9_u8;
  let stored = outer.inner.values[1_u64];
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("nested struct run places must check: {outcome:?}");
        };
        let body = &checked.data.functions[0].body;
        let CheckedStatement::Set { target, .. } = &body[6] else {
            panic!("seventh statement must be the projected indexed set");
        };
        let CheckedSetTarget::Storage(target) = target else {
            panic!("set must retain one checked run-index target");
        };
        assert!(matches!(
            target.path.as_slice(),
            [
                CheckedPlaceStep::Field(0),
                CheckedPlaceStep::Field(0),
                CheckedPlaceStep::Subscript(_)
            ]
        ));
        assert!(matches!(
            &body[7],
            CheckedStatement::Let {
                value: CheckedExpression::ReadStorage {
                    root: CheckedContainerRoot { path, .. },
                    ..
                },
                ..
            } if matches!(path.as_slice(), [CheckedPlaceStep::Field(0), CheckedPlaceStep::Field(0), CheckedPlaceStep::Subscript(_)])
        ));
    });

    assert_rule(
        br#"struct Inner {
  values: FixedVector<u8, 2>;
}

struct Outer {
  inner: Inner;
}

fn replacement(value: own Outer) -> result: own u8 pure {
  return 9_u8;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  set outer.inner.values[1_u64] = replacement(value: move outer);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );

    assert_rule(
        br#"struct Inner {
  values: FixedVector<u8, 2>;
}

struct Outer {
  inner: Inner;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 2>();
  let one = place_back(vector: move empty, value: 0_u8);
  let values = place_back(vector: move one, value: 0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  region {
    let held = &outer;
    set outer.inner.values[1_u64] = 9_u8;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

/// B7c4b left this case on the retiring surface. [STOR-5]'s refusal of
/// region-bearing content *inside an array* has no run twin: a run parameter
/// whose element is a view stops earlier as an unsupported composite value,
/// and `fixed_vector::<Slice<u8>, 1>()` is refused at [OP-1] rather than
/// [STOR-5], so neither program reaches this rule. It retires with
/// `array<T, n>`.
#[test]
fn region_bearing_array_content_rejects_under_stor5() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice, arena, or provider as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid(value: own array<Slice<u8>, 1>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid(value: own Slice<u8>) -> result: own unit pure {
  array_new::<Slice<u8>, 1>(move value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected,
    );
}

#[test]
fn general_elements_allow_an_array_value_inside_a_run_slot() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let row = array_new::<u64, 2>(7_u64);
  let empty = fixed_vector::<array<u64, 2>, 2>();
  let rows = place_back(vector: move empty, value: move row);
  set rows[0_u64][1_u64] = 9_u64;
  let value = rows[0_u64][1_u64];
  let width = len_of(rows[0_u64]);
  let capacity = cap_of(rows[0_u64]);
  let head = head_of(rows[0_u64]);
  let room = room_of(rows[0_u64]);
  invariant width_lower: width >= 2_u64;
  invariant width_upper: width <= 2_u64;
  invariant capacity_lower: capacity >= 2_u64;
  invariant capacity_upper: capacity <= 2_u64;
  invariant head_zero: head <= 0_u64;
  invariant room_zero: room <= 0_u64;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an array is a complete nameable run element: {outcome:?}");
        };
        let main = &checked.data.functions[checked.data.main.0 as usize];
        let CheckedStatement::Set {
            target: CheckedSetTarget::Storage(target),
            ..
        } = &main.body[3]
        else {
            panic!("nested array mutation must use the ordinary typed storage path");
        };
        assert_eq!(target.ty, CheckedType::Integer(IntegerType::U64));
        assert!(
            matches!(target.path.as_slice(), [CheckedPlaceStep::Subscript(_), CheckedPlaceStep::Subscript(index)]
            if matches!(index.base_type, CheckedType::Array { length: CheckedConst::Value(2), .. }))
        );
    });
}

#[test]
fn general_elements_reject_deep_stored_views_and_providers() {
    for source in [
        br#"fn invalid(value: own FixedVector<FixedVector<FixedVector<Slice<u8>, 1>, 1>, 1>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
        br#"command fn main() -> status: own ExitStatus pure {
  let invalid = fixed_vector::<FixedVector<FixedVector<Heap, 1>, 1>, 1>();
  return exit_status(code: 0_u8);
}
"#.as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Stor5, |kind| matches!(kind, SemanticIssueKind::RegionBearingStorage { .. }));
    }
}
