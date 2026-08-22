//! [OP-1] (ii) infix resolution: the operator token selects the row, and the
//! row then takes exactly the judgment the named spelling takes.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedExpression, CheckedIntegerOperation, CheckedStatement, CheckedType, IntegerType,
};
use super::{assert_rule, assert_rule_at, with_semantics};

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
    for (operator, expected) in [
        ("+", CheckedIntegerOperation::AddExact),
        ("+wrap", CheckedIntegerOperation::AddWrap),
        ("+defined", CheckedIntegerOperation::AddDefined),
        ("+checked", CheckedIntegerOperation::AddChecked),
        ("+sat", CheckedIntegerOperation::AddSaturating),
        ("-", CheckedIntegerOperation::SubtractExact),
        ("-wrap", CheckedIntegerOperation::SubtractWrap),
        ("-defined", CheckedIntegerOperation::SubtractDefined),
        ("-checked", CheckedIntegerOperation::SubtractChecked),
        ("-sat", CheckedIntegerOperation::SubtractSaturating),
        ("*", CheckedIntegerOperation::MultiplyExact),
        ("*wrap", CheckedIntegerOperation::MultiplyWrap),
        ("*defined", CheckedIntegerOperation::MultiplyDefined),
        ("*checked", CheckedIntegerOperation::MultiplyChecked),
        ("*sat", CheckedIntegerOperation::MultiplySaturating),
        ("/", CheckedIntegerOperation::DivideExact),
        ("/defined", CheckedIntegerOperation::DivideDefined),
        ("/checked", CheckedIntegerOperation::DivideChecked),
        ("%", CheckedIntegerOperation::RemainderExact),
        ("%defined", CheckedIntegerOperation::RemainderDefined),
        ("%checked", CheckedIntegerOperation::RemainderChecked),
    ] {
        // Proof-required exact rows are statically discharged for these
        // constant operands and therefore contribute no runtime effect.
        let source = format!(
            "command fn main() -> status: own ExitStatus pure {{\n  let c = 6_i32 {operator} 7_i32;\n  return exit_status(code: 0_u8);\n}}\n"
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
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let a = 1_i32;
  let b = 2_u64;
  let c = a + b;
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Type5, "b");
}

/// No integer row accepts a Bool, so the selection itself fails and [OP-1]
/// reports it at the whole expression rather than at one operand.
#[test]
fn an_operand_type_outside_every_row_is_an_op1_rejection() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let f = True();
  let g = False();
  let h = f + g;
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Op1, "f + g");
}

/// A bare exact operator is proof-required for every operand shape and never
/// contributes a runtime `traps` effect. Unknown parameters therefore leave a
/// static [OP-2, ENT-6] obligation instead of making the function trapping.
#[test]
fn bare_arithmetic_is_a_static_obligation_not_a_traps_effect() {
    let source = br#"fn add(a: own i32, b: own i32) -> result: own i32 pure {
  return a + b;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unproved exact operation must be rejected statically: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedIntegerDomainObligation { .. }
        ));
    });
}

/// [GRAM-5]'s complete set of positions taking a bare `expr`, enumerated from
/// the grammar rather than from whichever tests happened to fail: the `if_stmt`
/// and `value_if` conditions, `ordinary_let_rhs`, `propagate_let_rhs`,
/// `set_stmt`, `return_stmt`, `claim_stmt`, `give_stmt`, and the `match_stmt`
/// and `value_match` scrutinees. `expr_stmt := call ";"` takes a `call`, so
/// infix cannot be written there and it is deliberately absent. v0.33 has no
/// `check_stmt`; contract clauses are not statements and are covered by the
/// contract tests instead.
///
/// Each source writes one infix over `a` and `b` at the named position and
/// binds the second operand with the exact line [`DISAGREEING_OPERAND`]
/// rewrites, which is what turns every entry into its own negative case.
const EXPRESSION_POSITIONS: [(&str, &str); 10] = [
    (
        "ordinary_let_rhs",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = a +wrap b;
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "propagate_let_rhs",
        "fn step(a: own u64) -> result: own Result<u64, Overflow> pure {
  let b = 7_u64;
  let c = propagate a +checked b;
  return Ok<u64, Overflow>(value: c);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "set_stmt",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  set a = a +wrap b;
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "return_stmt",
        "fn add(a: own u64) -> result: own u64 pure {
  let b = 7_u64;
  return a +wrap b;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "claim_stmt",
        "fn clamp_ten(value: own i64) -> result: own i64 pure {
  let lower = imax(value, -10_i64);
  return imin(lower, 10_i64);
}

fn multiply_seven(input: own i64) -> result: own i64 traps {
  let a = 0_i64;
  loop @select_operand {
    if ieq(a, input) {
      break @select_operand;
    } else if ieq(a, 10_i64) {
      break @select_operand;
    } else {
      set a = a +wrap 1_i64;
    }
  }
  let b = 7_i64;
  claim product_defined: a *defined b because \"premises: a starts at zero, advances by one only on the ordinary-loop backedge, and exits no later than ten\\nderivation: induction over reached loop bodies keeps a between zero and ten, so multiplying it by seven remains in range\\nconclusion: a *defined b is true\\nchecker gap: ENT carries no induction fact across this ordinary-loop backedge\\nconsumers: the following exact a * b operation requires both signed product bounds\";
  return a * b;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "give_stmt",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  let f = True();
  let c = if f {
    give a +wrap b;
  } else {
    give a;
  }
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "match_stmt scrutinee",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  match a +checked b {
    Ok(value: v) => {
      return exit_status(code: 0_u8);
    }
    Err(error: e) => {
      return exit_status(code: 0_u8);
    }
  }
}
",
    ),
    (
        "value_match scrutinee",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = match a +checked b {
    Ok(value: v) => {
      give 1_u64;
    }
    Err(error: e) => {
      give 2_u64;
    }
  }
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "if_stmt condition",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  if a +defined b {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 0_u8);
}
",
    ),
    (
        "value_if condition",
        "command fn main() -> status: own ExitStatus pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = if a +defined b {
    give 1_u64;
  } else {
    give 2_u64;
  }
  return exit_status(code: 0_u8);
}
",
    ),
];

/// The second operand's binding, and the disagreeing type that replaces it.
const AGREEING_OPERAND: &str = "let b = 7_u64;";
const DISAGREEING_OPERAND: &str = "let b = 7_i32;";
const CLAIM_AGREEING_OPERAND: &str = "let b = 7_i64;";
const CLAIM_DISAGREEING_OPERAND: &str = "let b = 7_u64;";

/// Every [GRAM-5] position that admits an infix expression checks one, and
/// none of them fails the tree.
///
/// The `return_stmt` entry is the regression: two `return`-position structural
/// queries read the `expr` node with `only_child`, and `expr := atom
/// infix_tail?` is the one alternative with two children, so every infix
/// return reported `InvalidCanonicalTree` — an internal compiler failure where
/// a source rejection or an accepted program is required.
#[test]
fn infix_is_checked_at_every_expression_position() {
    for (position, source) in EXPRESSION_POSITIONS {
        with_semantics(source.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "infix in {position} position must check: {outcome:?}",
            );
        });
    }
}

/// [OP-2]'s operand judgment runs at every position, not only in the `let`
/// initializer.
///
/// A position that merely stopped failing the tree could still be skipping the
/// judgment, so each source is rewritten to disagree on its second operand and
/// must report TYPE-5 at exactly that operand — the same citation the `let`
/// path produces. A position that did not check the infix cannot produce it.
#[test]
fn a_disagreeing_operand_is_reported_at_that_operand_from_every_position() {
    for (position, source) in EXPRESSION_POSITIONS {
        let (agreeing, disagreeing) = if position == "claim_stmt" {
            (CLAIM_AGREEING_OPERAND, CLAIM_DISAGREEING_OPERAND)
        } else {
            (AGREEING_OPERAND, DISAGREEING_OPERAND)
        };
        assert_eq!(
            source.matches(agreeing).count(),
            1,
            "{position} must bind its second operand with the rewritten line",
        );
        let source = source.replace(agreeing, disagreeing);
        assert_rule_at(source.as_bytes(), SemanticRule::Type5, "b");
    }
}

/// The `return` position has two structural queries, reached under
/// complementary conditions, and each broke on infix independently.
///
/// [TYPE-7]'s implicit read runs only for an `own` result — the table's
/// `return_stmt` entry covers it. [OWN-14]'s returned reborrow runs only for a
/// borrow result, so it needs a borrow-returning function; an infix can never
/// produce a borrow, which makes this an FN-1 rejection rather than an accepted
/// program, and reporting it as a compiler failure was the defect. The control
/// is the same function returning a plain non-borrow atom: it cites FN-1 too,
/// so the infix path is held to the citation the position already produced.
#[test]
fn an_infix_returned_from_a_borrow_result_is_an_fn1_rejection() {
    let infix = br#"fn pick['r](x: &'r u64, a: own u64) -> result: &'r u64 reads('r) {
  let b = 7_u64;
  return a +wrap b;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(infix, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
    let plain = br#"fn pick['r](x: &'r u64, a: own u64) -> result: &'r u64 reads('r) {
  return a;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(plain, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}
