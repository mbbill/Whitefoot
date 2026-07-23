use crate::{SemanticIssueKind, SemanticOutcome, SemanticRuleV0_14};

use super::super::model::{
    CheckedArrayElement, CheckedArrayRoot, CheckedExpression, CheckedStatement, CheckedType,
    CheckedValue, IntegerType,
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
                element: CheckedArrayElement::Integer(IntegerType::U8),
                length: 4,
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
                        element: CheckedArrayElement::Integer(IntegerType::I32),
                        length: 4,
                    },
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &body[1],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayLength { length: 4, .. },
                ..
            }
        ));
        assert!(matches!(
            &body[2],
            CheckedStatement::Let {
                value: CheckedExpression::ArrayIndex {
                    root: CheckedArrayRoot::Binding(_),
                    length: 4,
                    trap,
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
                    length: 4,
                    trap,
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
        SemanticRuleV0_14::Const1,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"const table: array<u8, 2> = [1_u8];\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_14::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-noneligible.wf"),
        SemanticRuleV0_14::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nconst bad: Cell = unit;\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_14::Const2,
        SemanticIssueKind::InvalidConstValue,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/const2-neg-set.wf"),
        SemanticRuleV0_14::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  let items: own array<u8, 2> = array_new<u8, 2>(0_u8);\n  let value: own u8 = index<u8>(items, 0_u32);\n  return unit;\n}\n",
        SemanticRuleV0_14::Type5,
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
                element: CheckedArrayElement::TagOnlyNominal(checked.data.nominals[0].id),
                length: 2,
            }
        );
    });

    assert_rule(
        b"enum Payload {\n  Item(value: i32);\n}\n\nstruct Holder {\n  values: array<Payload, 2>;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_14::Type2,
        SemanticIssueKind::TypeMismatch,
    );
}
