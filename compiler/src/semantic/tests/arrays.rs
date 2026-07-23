use crate::{SemanticIssueKind, SemanticOutcome, SemanticRuleV0_15};

use super::super::model::{
    CheckedArrayRoot, CheckedConst, CheckedExpression, CheckedFlatElement, CheckedSetTarget,
    CheckedStatement, CheckedTargetDomainObligation, CheckedType, CheckedValue, IntegerType,
};
use super::{assert_rule, with_semantics};

#[test]
fn constants_fill_length_and_index_share_exact_array_types() {
    let source = br#"const count: u64 = 4_u64;

const table: array<u8, count> = [10_u8, 20_u8, 30_u8, 40_u8];

fn main() -> own unit traps {
  let values: own array<i32, count> = array_new<i32, count>(7_i32);
  let length: own u64 = len<i32>(values);
  let local: own i32 = index<i32>(values, 2_u64);
  let stored: own u8 = index<u8>(table, 2_u64);
  check ieq<u64>(length, 4_u64) else trap "length drift";
  check ieq<i32>(local, 7_i32) else trap "fill drift";
  check ieq<u8>(stored, 30_u8) else trap "const drift";
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
                    root: CheckedArrayRoot::Binding(_),
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
        SemanticRuleV0_15::Const1,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"const table: array<u8, 2> = [1_u8];\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_15::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-noneligible.wf"),
        SemanticRuleV0_15::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nconst bad: Cell = unit;\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_15::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-set.wf"),
        SemanticRuleV0_15::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  let items: own array<u8, 2> = array_new<u8, 2>(0_u8);\n  let value: own u8 = index<u8>(items, 0_u32);\n  return unit;\n}\n",
        SemanticRuleV0_15::Type5,
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
        SemanticRuleV0_15::Type2,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn indexed_set_retains_its_pre_rhs_guard_and_copy_target() {
    let source = br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  set index<u8>(values, 1_u64) = 9_u8;
  let stored: own u8 = index<u8>(values, 1_u64);
  check ieq<u8>(stored, 9_u8) else trap "set drift";
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
    assert_rule(
        b"fn main() -> own unit pure {\n  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);\n  set index<u8>(values, 0_u64) = 1_u8;\n  return unit;\n}\n",
        SemanticRuleV0_15::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);\n  set index<u8>(values, 0_u64) = 1_u16;\n  return unit;\n}\n",
        SemanticRuleV0_15::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        b"fn consume(values: own array<u8, 2>) -> own u8 pure {\n  return 1_u8;\n}\n\nfn main() -> own unit traps {\n  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);\n  set index<u8>(values, 0_u64) = consume(values: move values);\n  return unit;\n}\n",
        SemanticRuleV0_15::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}
