use crate::{SemanticIssueKind, SemanticOutcome, SemanticRuleV0_15};

use super::super::model::{CheckedExpression, CheckedIntegerOperation, CheckedStatement};
use super::{assert_rule, with_semantics};

#[test]
fn retains_each_mode_and_rejects_unsigned_types() {
    let source = br#"fn main() -> own unit traps {
  let wrapped: own i8 = ineg.wrap<i8>(-128_i8);
  let trapped: own i16 = ineg.trap<i16>(-42_i16);
  let negation_result: own Result<i32, Overflow> = ineg.checked<i32>(-42_i32);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("integer negation family must check: {outcome:?}");
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
                CheckedIntegerOperation::NegateWrap,
                CheckedIntegerOperation::NegateTrap,
                CheckedIntegerOperation::NegateChecked,
            ]
        );
    });

    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own u8 = ineg.wrap<u8>(1_u8);\n  return unit;\n}\n",
        SemanticRuleV0_15::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}
