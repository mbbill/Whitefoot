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

/// [EFF-2] bare infix arithmetic outside [OP-2]'s constant-operand class is
/// the trapping row, so it contributes `traps` exactly as the named `.trap`
/// spelling did. Both operands are non-constant here; a site with a constant
/// operand instead carries the [ENT-6] overflow obligation and contributes
/// nothing, which `arithmetic_obligations` pins.
#[test]
fn bare_arithmetic_contributes_the_traps_effect() {
    let source = br#"fn main() -> own unit pure {
  let a = 20_i32;
  let b = 22_i32;
  let c = a + b;
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

/// [GRAM-5]'s complete set of positions taking a bare `expr`, enumerated from
/// the grammar rather than from whichever tests happened to fail: the `if_stmt`
/// and `value_if` conditions, `ordinary_let_rhs`, `propagate_let_rhs`,
/// `set_stmt`, `return_stmt`, `check_stmt`, `claim_stmt`, `give_stmt`, and the
/// `match_stmt` and `value_match` scrutinees. `expr_stmt := call ";"` takes a
/// `call`, so infix cannot be written there and it is deliberately absent.
///
/// Each source writes one infix over `a` and `b` at the named position and
/// binds the second operand with the exact line [`DISAGREEING_OPERAND`]
/// rewrites, which is what turns every entry into its own negative case.
const EXPRESSION_POSITIONS: [(&str, &str); 11] = [
    (
        "ordinary_let_rhs",
        "fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  let c = a + b;
  return unit;
}
",
    ),
    (
        "propagate_let_rhs",
        "fn step(a: own u64) -> own Result<u64, Overflow> pure {
  let b = 7_u64;
  let c = propagate a +checked b;
  return Ok<u64, Overflow>(value: c);
}

fn main() -> own unit pure {
  return unit;
}
",
    ),
    (
        "set_stmt",
        "fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  set a = a + b;
  return unit;
}
",
    ),
    (
        "return_stmt",
        "fn add(a: own u64) -> own u64 traps {
  let b = 7_u64;
  return a + b;
}

fn main() -> own unit pure {
  return unit;
}
",
    ),
    (
        "check_stmt",
        "fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  check ile(a, b) else trap \"six is at most seven\";
  return unit;
}
",
    ),
    (
        "claim_stmt",
        "fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  claim ordered: ile(a, b) because \"six is at most seven\";
  return unit;
}
",
    ),
    (
        "give_stmt",
        "fn main() -> own unit traps {
  let a = 6_u64;
  let b = 7_u64;
  let f = True();
  let c = if f {
    give a + b;
  } else {
    give a;
  }
  return unit;
}
",
    ),
    (
        "match_stmt scrutinee",
        "fn main() -> own unit pure {
  let a = 6_u64;
  let b = 7_u64;
  match a +checked b {
    Ok(value: v) => {
      return unit;
    }
    Err(error: e) => {
      return unit;
    }
  }
}
",
    ),
    (
        "value_match scrutinee",
        "fn main() -> own unit pure {
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
  return unit;
}
",
    ),
    (
        "if_stmt condition",
        "fn main() -> own unit pure {
  let a = 6_u64;
  let b = 7_u64;
  if ile(a, b) {
    return unit;
  }
  return unit;
}
",
    ),
    (
        "value_if condition",
        "fn main() -> own unit pure {
  let a = 6_u64;
  let b = 7_u64;
  let c = if ile(a, b) {
    give 1_u64;
  } else {
    give 2_u64;
  }
  return unit;
}
",
    ),
];

/// The second operand's binding, and the disagreeing type that replaces it.
const AGREEING_OPERAND: &str = "let b = 7_u64;";
const DISAGREEING_OPERAND: &str = "let b = 7_i32;";

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
        assert_eq!(
            source.matches(AGREEING_OPERAND).count(),
            1,
            "{position} must bind its second operand with the rewritten line",
        );
        let disagreeing = source.replace(AGREEING_OPERAND, DISAGREEING_OPERAND);
        assert_rule_at(disagreeing.as_bytes(), SemanticRule::Type5, "b");
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
    let infix = br#"fn pick['r](x: &'r u64, a: own u64) -> &'r u64 reads('r), traps {
  let b = 7_u64;
  return a + b;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(infix, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
    let plain = br#"fn pick['r](x: &'r u64, a: own u64) -> &'r u64 reads('r), traps {
  return a;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(plain, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}
