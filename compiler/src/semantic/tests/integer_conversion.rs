use std::fmt::Write;

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRuleV0_15};

use super::super::model::{CheckedExpression, CheckedStatement, CheckedType, IntegerType};
use super::{assert_rule, with_semantics};

const INTEGER_TYPES: [(&str, IntegerType); 8] = [
    ("i8", IntegerType::I8),
    ("i16", IntegerType::I16),
    ("i32", IntegerType::I32),
    ("i64", IntegerType::I64),
    ("u8", IntegerType::U8),
    ("u16", IntegerType::U16),
    ("u32", IntegerType::U32),
    ("u64", IntegerType::U64),
];

#[test]
fn classifies_every_distinct_integer_pair_through_one_conversion_judgment() {
    let mut source = String::new();
    let mut expected = Vec::new();
    for (source_name, source_type) in INTEGER_TYPES {
        for (destination_name, destination_type) in INTEGER_TYPES {
            if source_type == destination_type {
                continue;
            }
            let total = source_type.converts_totally_to(destination_type);
            let result = if total {
                destination_name.to_owned()
            } else {
                format!("Result<{destination_name}, NarrowError>")
            };
            writeln!(
                source,
                "fn convert_{source_name}_{destination_name}(value: own {source_name}) -> own {result} pure {{\n  return cvt<{source_name}, {destination_name}>(value);\n}}\n"
            )
            .expect("write generated source");
            expected.push((source_type, destination_type, total));
        }
    }
    source.push_str("fn main() -> own unit pure {\n  return unit;\n}\n");

    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("all integer conversion pairs must check: {outcome:?}");
        };
        assert_eq!(checked.data.functions.len(), expected.len() + 1);
        for (function, (source_type, destination_type, total)) in
            checked.data.functions.iter().zip(expected)
        {
            let [
                CheckedStatement::Return {
                    value:
                        CheckedExpression::IntegerConversion {
                            source,
                            destination,
                            result,
                            ..
                        },
                    ..
                },
            ] = function.body.as_slice()
            else {
                panic!("conversion function must retain one conversion return");
            };
            assert_eq!((*source, *destination), (source_type, destination_type));
            if total {
                assert_eq!(*result, CheckedType::Integer(destination_type));
            } else {
                let CheckedType::Nominal(result) = result else {
                    panic!("checked conversion must return a nominal Result");
                };
                assert_eq!(
                    checked.data.nominals[result.0 as usize].name,
                    format!(
                        "Result<{}, NarrowError>",
                        integer_spelling(destination_type)
                    )
                );
            }
        }
    });
}

#[test]
fn partial_conversion_result_is_available_without_an_explicit_type_annotation() {
    let source = br#"fn main() -> own unit pure {
  match cvt<u64, u8>(65_u64) {
    Ok(value: byte) => {
    }
    Err(error: narrow_error) => {
    }
  }
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a directly matched partial conversion must check: {outcome:?}"
        );
    });
}

#[test]
fn conversion_shape_and_operand_failures_keep_their_rule_owners() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i32 = cvt<i32, i32>(1_i32);\n  return unit;\n}\n",
        SemanticRuleV0_15::Op6,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i64 = cvt<i32, i64>(1_i16);\n  return unit;\n}\n",
        SemanticRuleV0_15::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i64 = cvt<i32>(1_i32);\n  return unit;\n}\n",
        SemanticRuleV0_15::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let flag: own Bool = True();\n  let value: own i32 = cvt<Bool, i32>(flag);\n  return unit;\n}\n",
        SemanticRuleV0_15::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

const fn integer_spelling(ty: IntegerType) -> &'static str {
    match ty {
        IntegerType::I8 => "i8",
        IntegerType::I16 => "i16",
        IntegerType::I32 => "i32",
        IntegerType::I64 => "i64",
        IntegerType::U8 => "u8",
        IntegerType::U16 => "u16",
        IntegerType::U32 => "u32",
        IntegerType::U64 => "u64",
    }
}
