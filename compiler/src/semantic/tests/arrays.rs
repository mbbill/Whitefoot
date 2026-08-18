use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedArrayRoot, CheckedConst, CheckedExpression, CheckedFlatElement, CheckedSetTarget,
    CheckedStatement, CheckedTargetDomainObligation, CheckedType, CheckedValue, IntegerType,
};
use super::{assert_rule, with_semantics};

#[test]
fn constants_fill_length_and_index_share_exact_array_types() {
    let source = br#"const count: u64 = 4_u64;

const table: array<u8, count> =[10_u8, 20_u8, 30_u8, 40_u8];

fn main() -> own unit traps {
  let values = array_new<i32, count>(7_i32);
  let length = len(values);
  let local = values[2_u64];
  let stored = table[2_u64];
  claim length_drift: ieq(length, 4_u64) because "length drift";
  claim fill_drift: ieq(local, 7_i32) because "fill drift";
  claim const_drift: ieq(stored, 30_u8) because "const drift";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("fixed-array family must check: {outcome:?}");
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
            &body[0],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayFill {
                    ty: CheckedType::Array {
                        element: CheckedFlatElement::Integer(IntegerType::I32),
                        length: CheckedConst::Value(4),
                    },
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &body[1],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayLength {
                    length: CheckedConst::Value(4),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &body[2],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayIndex {
                    root: CheckedArrayRoot::Binding { .. },
                    length: CheckedConst::Value(4),
                    trap,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            } if trap.rule_id == "OP-4"
        ));
        assert!(matches!(
            &body[3],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayIndex {
                    root: CheckedArrayRoot::Constant(_),
                    length: CheckedConst::Value(4),
                    trap,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            } if trap.rule_id == "OP-4"
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
        b"const table: array<u8, 2> =[1_u8];\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-noneligible.wf"),
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nconst bad: Cell = unit;\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-set.wf"),
        SemanticRule::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  let items = array_new<u8, 2>(0_u8);\n  let value = items[0_u32];\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
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
  flags: array<Flag, count>;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "nominal array fields must use earlier lengths and completed enum layouts: {outcome:?}"
            );
        };
        let super::super::model::CheckedNominalKind::Struct { fields } =
            &checked.data.nominals[1].kind
        else {
            panic!("Holder must remain a struct");
        };
        assert_eq!(
            fields[0].ty,
            CheckedType::Array {
                element: CheckedFlatElement::TagOnlyNominal(checked.data.nominals[0].id),
                length: CheckedConst::Value(2),
            }
        );
    });

    assert_rule(
        b"enum Payload {\n  Item(value: i32);\n}\n\nstruct Holder {\n  values: array<Payload, 2>;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn indexed_set_retains_its_pre_rhs_guard_and_copy_target() {
    let source = br#"fn main() -> own unit traps {
  let values = array_new<u8, 2>(0_u8);
  set values[1_u64] = 9_u8;
  let stored = values[1_u64];
  claim set_drift: ieq(stored, 9_u8) because "set drift";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("indexed fixed-array set must check: {outcome:?}");
        };
        let CheckedStatement::Set { target, .. } = &checked.data.functions[0].body[1] else {
            panic!("second statement must be the indexed set");
        };
        let CheckedSetTarget::ArrayIndex(target) = target else {
            panic!("indexed set must retain an array-index target");
        };
        assert_eq!(
            target.array_type,
            CheckedType::Array {
                element: CheckedFlatElement::Integer(IntegerType::U8),
                length: CheckedConst::Value(2),
            }
        );
        assert_eq!(target.element_type, CheckedType::Integer(IntegerType::U8));
        assert_eq!(target.length, CheckedConst::Value(2));
        assert_eq!(target.offset.ty(), CheckedType::Integer(IntegerType::U64));
        assert_eq!(target.trap.rule_id, "OP-4");
        assert_eq!(
            target.target_domain,
            CheckedTargetDomainObligation::ElementAddress
        );
    });
}

#[test]
fn indexed_set_rechecks_type_effect_and_root_liveness() {
    // A discharged subscript is not an [EFF-2] trap source: the indexed set
    // with a constant in-range offset is accepted in a `pure` function.
    with_semantics(
        b"fn main() -> own unit pure {\n  let values = array_new<u8, 2>(0_u8);\n  set values[0_u64] = 1_u8;\n  return unit;\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "a discharged indexed set must not force a traps row: {outcome:?}"
            );
        },
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  let values = array_new<u8, 2>(0_u8);\n  set values[0_u64] = 1_u16;\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        b"fn consume(values: own array<u8, 2>) -> own u8 pure {\n  return 1_u8;\n}\n\nfn main() -> own unit traps {\n  let values = array_new<u8, 2>(0_u8);\n  set values[0_u64] = consume(values: move values);\n  return unit;\n}\n",
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}

#[test]
fn nested_struct_array_places_retain_their_complete_paths() {
    let source = br#"struct Inner {
  values: array<u8, 2>;
}

struct Outer {
  inner: Inner;
}

fn main() -> own unit traps {
  let values = array_new<u8, 2>(0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  let length = len(outer.inner.values);
  set outer.inner.values[1_u64] = 9_u8;
  let stored = outer.inner.values[1_u64];
  claim length_drift: ieq(length, 2_u64) because "length drift";
  claim set_drift: ieq(stored, 9_u8) because "set drift";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("nested struct array places must check: {outcome:?}");
        };
        let body = &checked.data.functions[0].body;
        let CheckedStatement::Set { target, .. } = &body[4] else {
            panic!("fifth statement must be the projected indexed set");
        };
        let CheckedSetTarget::ArrayIndex(target) = target else {
            panic!("set must retain one checked array-index target");
        };
        assert_eq!(target.fields, vec![0, 0]);
        assert!(matches!(
            &body[5],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayIndex {
                    root: CheckedArrayRoot::Binding { fields, .. },
                    ..
                },
                ..
            } if fields == &[0, 0]
        ));
    });

    assert_rule(
        br#"struct Inner {
  values: array<u8, 2>;
}

struct Outer {
  inner: Inner;
}

fn replacement(value: own Outer) -> own u8 pure {
  return 9_u8;
}

fn main() -> own unit traps {
  let values = array_new<u8, 2>(0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  set outer.inner.values[1_u64] = replacement(value: move outer);
  return unit;
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );

    assert_rule(
        br#"struct Inner {
  values: array<u8, 2>;
}

struct Outer {
  inner: Inner;
}

fn main() -> own unit traps {
  let values = array_new<u8, 2>(0_u8);
  let inner = Inner(values: move values);
  let outer = Outer(inner: move inner);
  region 'view {
    let held = &'view outer;
    set outer.inner.values[1_u64] = 9_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn region_bearing_array_content_rejects_under_stor5() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid['r](value: own array<slice<'r, u8>, 1>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> own unit pure {
  array_new<slice<'r, u8>, 1>(move value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected,
    );
}
