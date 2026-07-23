use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedExpression, CheckedFloatOperation, CheckedStatement, FloatType};
use super::{assert_rule, with_semantics};

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
}
