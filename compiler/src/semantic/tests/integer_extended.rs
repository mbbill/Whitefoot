use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedExpression, CheckedIntegerOperation, CheckedStatement};
use super::{assert_rule, with_semantics};

#[test]
fn retains_the_complete_nonfloating_integer_family() {
    let source = br#"fn main() -> own unit pure {
  let a = 8_i32 / 2_i32;
  let b = 9_i32 % 2_i32;
  let c = iand(a, b);
  let d = ior(a, b);
  let e = ixor(a, b);
  let f = inot(a);
  let g = ishl.wrap(a, 1_u32);
  let h = ishr.wrap(a, 1_u32);
  let i = ishl(a, 1_u32);
  let j = ishr(a, 1_u32);
  let left_is_defined = ishl.defined(a, 1_u32);
  let right_is_defined = ishr.defined(a, 1_u32);
  let k = irotl(a, 1_u32);
  let l = irotr(a, 1_u32);
  let m = ipopcount(a);
  let n = iclz(a);
  let o = ictz(a);
  let p = ibswap(a);
  let q = imulhi(a, b);
  let r = a +sat b;
  let s = a -sat b;
  let t = a *sat b;
  let u = imin(a, b);
  let v = imax(a, b);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("complete integer family must check: {outcome:?}");
        };
        let operations = checked.data.functions[0]
            .body
            .iter()
            .filter_map(|statement| match statement {
                CheckedStatement::Let {
                    value: CheckedExpression::IntegerOperation { operation, .. },
                    ..
                } => Some(*operation),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(operations.len(), 24);
        assert_eq!(operations[0], CheckedIntegerOperation::DivideExact);
        assert_eq!(operations[6], CheckedIntegerOperation::ShiftLeftWrap);
        assert_eq!(operations[10], CheckedIntegerOperation::ShiftLeftDefined);
        assert_eq!(operations[11], CheckedIntegerOperation::ShiftRightDefined);
        assert_eq!(operations[14], CheckedIntegerOperation::PopulationCount);
        assert_eq!(operations[21], CheckedIntegerOperation::MultiplySaturating);
        assert_eq!(operations[23], CheckedIntegerOperation::Maximum);
    });

    assert_rule(
        b"fn main() -> own unit pure {\n  let value = ibswap(1_i8);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value = ishl.wrap(1_i8, 1_i8);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}
