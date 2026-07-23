use std::fmt::Write;

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedExpression, CheckedFloatOperation, CheckedNumericType, CheckedStatement, FloatType,
    IntegerType,
};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn retains_the_complete_direct_float_operation_family() {
    let source = br#"fn main() -> own unit pure {
  let a: own f32 = fadd.strict<f32>(1.0_f32, 2.0_f32);
  let b: own f32 = fsub.strict<f32>(a, 1.0_f32);
  let c: own f32 = fmul.strict<f32>(a, b);
  let d: own f32 = fdiv.strict<f32>(c, b);
  let e: own Bool = feq<f32>(a, d);
  let f: own Bool = flt<f32>(b, a);
  let g: own Bool = fle<f32>(b, a);
  let h: own Bool = fgt<f32>(a, b);
  let i: own Bool = fge<f32>(a, b);
  let j: own Bool = fne<f32>(a, b);
  let k: own f32 = fneg<f32>(a);
  let l: own f32 = fabs<f32>(k);
  let m: own f32 = fcopysign<f32>(a, k);
  let n: own f32 = fmin<f32>(a, b);
  let o: own f32 = fmax<f32>(a, b);
  let p: own f32 = ffloor<f32>(a);
  let q: own f32 = fceil<f32>(a);
  let r: own f32 = ftrunc<f32>(a);
  let s: own f32 = froundeven<f32>(a);
  let t: own f32 = frem<f32>(a, b);
  let u: own f32 = fsqrt.strict<f32>(a);
  let v: own f32 = ffma.strict<f32>(a, b, c);
  let w: own f32 = finf<f32>();
  let x: own f32 = fnan<f32>();
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("complete direct float family must check: {outcome:?}");
        };
        let operations = checked.data.functions[0]
            .body
            .iter()
            .filter_map(|statement| match statement {
                CheckedStatement::Let {
                    value:
                        CheckedExpression::FloatOperation {
                            operation,
                            operand_type,
                            ..
                        },
                    ..
                } => Some((*operation, *operand_type)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(operations.len(), 24);
        assert_eq!(
            operations[0],
            (CheckedFloatOperation::AddStrict, FloatType::F32)
        );
        assert_eq!(
            operations[21],
            (
                CheckedFloatOperation::FusedMultiplyAddStrict,
                FloatType::F32
            )
        );
        assert_eq!(operations[23], (CheckedFloatOperation::Nan, FloatType::F32));
    });
}

#[test]
fn retains_every_total_conversion_with_a_float_endpoint() {
    let pairs = [
        (
            "i8",
            "f32",
            CheckedNumericType::Integer(IntegerType::I8),
            CheckedNumericType::Float(FloatType::F32),
        ),
        (
            "i16",
            "f32",
            CheckedNumericType::Integer(IntegerType::I16),
            CheckedNumericType::Float(FloatType::F32),
        ),
        (
            "u8",
            "f32",
            CheckedNumericType::Integer(IntegerType::U8),
            CheckedNumericType::Float(FloatType::F32),
        ),
        (
            "u16",
            "f32",
            CheckedNumericType::Integer(IntegerType::U16),
            CheckedNumericType::Float(FloatType::F32),
        ),
        (
            "i8",
            "f64",
            CheckedNumericType::Integer(IntegerType::I8),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "i16",
            "f64",
            CheckedNumericType::Integer(IntegerType::I16),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "i32",
            "f64",
            CheckedNumericType::Integer(IntegerType::I32),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "u8",
            "f64",
            CheckedNumericType::Integer(IntegerType::U8),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "u16",
            "f64",
            CheckedNumericType::Integer(IntegerType::U16),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "u32",
            "f64",
            CheckedNumericType::Integer(IntegerType::U32),
            CheckedNumericType::Float(FloatType::F64),
        ),
        (
            "f32",
            "f64",
            CheckedNumericType::Float(FloatType::F32),
            CheckedNumericType::Float(FloatType::F64),
        ),
    ];
    let mut source = String::new();
    for (source_name, destination_name, _, _) in pairs {
        writeln!(
            source,
            "fn convert_{source_name}_{destination_name}(value: own {source_name}) -> own {destination_name} pure {{\n  return cvt<{source_name}, {destination_name}>(value);\n}}\n"
        )
        .expect("write conversion function");
    }
    source.push_str("fn main() -> own unit pure {\n  return unit;\n}\n");

    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("total floating conversion family must check: {outcome:?}");
        };
        assert_eq!(checked.data.functions.len(), pairs.len() + 1);
        for (function, (_, _, expected_source, expected_destination)) in
            checked.data.functions.iter().zip(pairs)
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
            assert_eq!(*result, expected_destination.ty());
        }
    });
}

#[test]
fn float_literal_and_operation_failures_keep_their_rule_owners() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own f64 = 1.00_f64;\n  return unit;\n}\n",
        SemanticRule::Form7,
        SemanticIssueKind::InvalidFloatLiteral,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own f64 = fadd.strict<i32>(1_i32, 2_i32);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own f64 = fadd.strict<f64>(1.0_f64, 2_i32);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_unsupported(
        b"fn main() -> own unit pure {\n  let value: own Result<f32, NarrowError> = cvt<f64, f32>(1.0_f64);\n  return unit;\n}\n",
        UnsupportedSemanticFeature::FloatingPointConversion,
    );
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
