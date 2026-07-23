use std::fmt::Write;

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedExpression, CheckedNumericType, CheckedStatement, CheckedType, FloatType, IntegerType,
};
use super::{assert_rule, with_semantics};

const NUMERIC_TYPES: [(&str, CheckedNumericType); 10] = [
    ("i8", CheckedNumericType::Integer(IntegerType::I8)),
    ("i16", CheckedNumericType::Integer(IntegerType::I16)),
    ("i32", CheckedNumericType::Integer(IntegerType::I32)),
    ("i64", CheckedNumericType::Integer(IntegerType::I64)),
    ("u8", CheckedNumericType::Integer(IntegerType::U8)),
    ("u16", CheckedNumericType::Integer(IntegerType::U16)),
    ("u32", CheckedNumericType::Integer(IntegerType::U32)),
    ("u64", CheckedNumericType::Integer(IntegerType::U64)),
    ("f32", CheckedNumericType::Float(FloatType::F32)),
    ("f64", CheckedNumericType::Float(FloatType::F64)),
];

#[test]
fn classifies_every_distinct_pair_with_a_float_endpoint() {
    let mut source = String::new();
    let mut expected = Vec::new();
    for (source_name, source_type) in NUMERIC_TYPES {
        for (destination_name, destination_type) in NUMERIC_TYPES {
            if source_type == destination_type
                || !matches!(source_type, CheckedNumericType::Float(_))
                    && !matches!(destination_type, CheckedNumericType::Float(_))
            {
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
            .expect("write conversion function");
            expected.push((source_type, destination_type, total, destination_name));
        }
    }
    source.push_str("fn main() -> own unit pure {\n  return unit;\n}\n");

    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("all concrete float-endpoint conversion pairs must check: {outcome:?}");
        };
        assert_eq!(expected.len(), 34);
        assert_eq!(checked.data.functions.len(), expected.len() + 1);
        for (function, (expected_source, expected_destination, total, destination_name)) in
            checked.data.functions.iter().zip(expected)
        {
            let [
                CheckedStatement::Return {
                    value:
                        CheckedExpression::NumericConversion {
                            source,
                            destination,
                            result,
                            ..
                        },
                    ..
                },
            ] = function.body.as_slice()
            else {
                panic!("conversion function must retain one numeric conversion");
            };
            assert_eq!(
                (*source, *destination),
                (expected_source, expected_destination)
            );
            if total {
                assert_eq!(*result, expected_destination.ty());
            } else {
                let CheckedType::Nominal(result) = result else {
                    panic!("partial conversion must return a nominal Result");
                };
                assert_eq!(
                    checked.data.nominals[result.0 as usize].name,
                    format!("Result<{destination_name}, NarrowError>")
                );
            }
        }
    });
}

#[test]
fn partial_float_conversion_result_is_available_without_an_annotation() {
    let source = br#"fn main() -> own unit pure {
  match cvt<f32, u8>(1.0_f32) {
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
            "a directly matched partial float conversion must check: {outcome:?}"
        );
    });
}

#[test]
fn float_conversion_operand_failures_keep_their_rule_owners() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own f32 = cvt<f32, f32>(1.0_f32);\n  return unit;\n}\n",
        SemanticRule::Op6,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own f64 = cvt<u32, f64>(1_u16);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}
