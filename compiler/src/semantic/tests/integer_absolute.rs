use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedExpression, CheckedIntegerOperation, CheckedStatement};
use super::{assert_rule, with_semantics};

#[test]
fn retains_each_mode_and_rejects_unsigned_types() {
    let source = br#"fn main() -> own unit pure {
  let wrapped = iabs.wrap(-128_i8);
  let exact = iabs(-42_i16);
  let absolute_value_is_defined = iabs.defined(-42_i64);
  let absolute_result = iabs.checked(-42_i32);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("integer absolute-value family must check: {outcome:?}");
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
        assert_eq!(
            operations,
            [
                CheckedIntegerOperation::AbsoluteWrap,
                CheckedIntegerOperation::AbsoluteExact,
                CheckedIntegerOperation::AbsoluteDefined,
                CheckedIntegerOperation::AbsoluteChecked,
            ]
        );
    });

    assert_rule(
        b"fn main() -> own unit pure {\n  let value = iabs.wrap(1_u8);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}
