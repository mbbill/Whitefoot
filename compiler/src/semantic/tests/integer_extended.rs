use crate::{SemanticIssueKind, SemanticOutcome, SemanticRuleV0_14};

use super::super::model::{CheckedExpression, CheckedIntegerOperation, CheckedStatement};
use super::{assert_rule, with_semantics};

#[test]
fn retains_the_complete_nonfloating_integer_family() {
    let source = br#"fn main() -> own unit traps {
  let a: own i32 = idiv.trap<i32>(8_i32, 2_i32);
  let b: own i32 = irem.trap<i32>(9_i32, 2_i32);
  let c: own i32 = iand<i32>(a, b);
  let d: own i32 = ior<i32>(a, b);
  let e: own i32 = ixor<i32>(a, b);
  let f: own i32 = inot<i32>(a);
  let g: own i32 = ishl.wrap<i32>(a, 1_u32);
  let h: own i32 = ishr.wrap<i32>(a, 1_u32);
  let i: own i32 = ishl.trap<i32>(a, 1_u32);
  let j: own i32 = ishr.trap<i32>(a, 1_u32);
  let k: own i32 = irotl<i32>(a, 1_u32);
  let l: own i32 = irotr<i32>(a, 1_u32);
  let m: own u32 = ipopcount<i32>(a);
  let n: own u32 = iclz<i32>(a);
  let o: own u32 = ictz<i32>(a);
  let p: own i32 = ibswap<i32>(a);
  let q: own i32 = imulhi<i32>(a, b);
  let r: own i32 = iadd.sat<i32>(a, b);
  let s: own i32 = isub.sat<i32>(a, b);
  let t: own i32 = imul.sat<i32>(a, b);
  let u: own i32 = imin<i32>(a, b);
  let v: own i32 = imax<i32>(a, b);
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
        assert_eq!(operations.len(), 22);
        assert_eq!(operations[0], CheckedIntegerOperation::DivideTrap);
        assert_eq!(operations[6], CheckedIntegerOperation::ShiftLeftWrap);
        assert_eq!(operations[12], CheckedIntegerOperation::PopulationCount);
        assert_eq!(operations[19], CheckedIntegerOperation::MultiplySaturating);
        assert_eq!(operations[21], CheckedIntegerOperation::Maximum);
    });

    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i8 = ibswap<i8>(1_i8);\n  return unit;\n}\n",
        SemanticRuleV0_14::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i8 = ishl.wrap<i8>(1_i8, 1_i8);\n  return unit;\n}\n",
        SemanticRuleV0_14::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}
