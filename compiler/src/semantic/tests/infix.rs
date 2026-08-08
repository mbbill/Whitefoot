//! [OP-1] (ii) infix resolution: the operator token selects the row, and the
//! row then takes exactly the judgment the named spelling takes.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedExpression, CheckedIntegerOperation, CheckedStatement, CheckedType, IntegerType,
};
use super::{assert_rule_at, with_semantics};

/// The operation and selected type of the one integer operation in `main`.
fn sole_operation(source: &[u8]) -> (CheckedIntegerOperation, CheckedType) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("infix source must check: {outcome:?}");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main is checked");
        let mut found = None;
        for statement in &main.body {
            let CheckedStatement::Let { value, .. } = statement else {
                continue;
            };
            if let CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                ..
            } = value
            {
                assert!(found.is_none(), "exactly one integer operation");
                found = Some((*operation, *operand_type));
            }
        }
        found.expect("one integer operation")
    })
}

#[test]
fn every_operator_token_selects_its_row() {
    for (operator, expected, traps) in [
        ("+", CheckedIntegerOperation::AddTrap, true),
        ("+wrap", CheckedIntegerOperation::AddWrap, false),
        ("+checked", CheckedIntegerOperation::AddChecked, false),
        ("+sat", CheckedIntegerOperation::AddSaturating, false),
        ("-", CheckedIntegerOperation::SubtractTrap, true),
        ("-wrap", CheckedIntegerOperation::SubtractWrap, false),
        ("-checked", CheckedIntegerOperation::SubtractChecked, false),
        ("-sat", CheckedIntegerOperation::SubtractSaturating, false),
        ("*", CheckedIntegerOperation::MultiplyTrap, true),
        ("*wrap", CheckedIntegerOperation::MultiplyWrap, false),
        ("*checked", CheckedIntegerOperation::MultiplyChecked, false),
        ("*sat", CheckedIntegerOperation::MultiplySaturating, false),
        ("/", CheckedIntegerOperation::DivideTrap, true),
        ("/checked", CheckedIntegerOperation::DivideChecked, false),
        ("%", CheckedIntegerOperation::RemainderTrap, true),
        ("%checked", CheckedIntegerOperation::RemainderChecked, false),
        ("==", CheckedIntegerOperation::Equal, false),
        ("!=", CheckedIntegerOperation::NotEqual, false),
        ("<=", CheckedIntegerOperation::LessEqual, false),
        (">=", CheckedIntegerOperation::GreaterEqual, false),
    ] {
        // [EFF-2] the row is exact, so only the trapping operators may
        // declare `traps`.
        let effects = if traps { "traps" } else { "pure" };
        let source = format!(
            "fn main() -> own unit {effects} {{\n  let a = 6_i32;\n  let b = 7_i32;\n  let c = a {operator} b;\n  return unit;\n}}\n"
        );
        let (operation, operand_type) = sole_operation(source.as_bytes());
        assert_eq!(operation, expected, "operator {operator:?} selects its row");
        assert_eq!(
            operand_type,
            CheckedType::Integer(IntegerType::I32),
            "operator {operator:?} derives its selected type from the operands",
        );
    }
}

/// [OP-2] the selection comes from the first operand, and every later operand
/// is held to the row's argument type for it — so the second atom is where a
/// disagreement is reported.
#[test]
fn a_disagreeing_second_operand_is_a_type5_rejection_at_that_operand() {
    let source = br#"fn main() -> own unit traps {
  let a = 1_i32;
  let b = 2_u64;
  let c = a + b;
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Type5, "b");
}

/// No integer row accepts a Bool, so the selection itself fails and [OP-1]
/// reports it at the whole expression rather than at one operand.
#[test]
fn an_operand_type_outside_every_row_is_an_op1_rejection() {
    let source = br#"fn main() -> own unit traps {
  let f = True();
  let g = False();
  let h = f + g;
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Op1, "f + g");
}

/// [EFF-2] bare infix arithmetic is the trapping row, so it contributes
/// `traps` exactly as the named `.trap` spelling did.
#[test]
fn bare_arithmetic_contributes_the_traps_effect() {
    let source = br#"fn main() -> own unit pure {
  let a = 20_i32;
  let b = a + 22_i32;
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a pure function may not trap: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
        assert_eq!(issue.kind(), &SemanticIssueKind::EffectMismatch);
    });
}
