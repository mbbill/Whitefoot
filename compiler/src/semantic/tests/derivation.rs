//! [TYPE-5] and [GIVE-1] derivation: a `let` binder's mode and type come
//! from its selected right-hand side, never from a written annotation, and a
//! value initializer's come from its delivery set.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_rule, assert_rule_at, with_semantics};

#[test]
fn an_ordinary_let_takes_the_type_its_right_hand_side_produces() {
    let source = br#"fn answer() -> result: own i32 pure {
  let value = 40_i32;
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("an unannotated let must derive its binder: {outcome:?}");
        };
    });
}

/// The derived type is the operand's own, so a right-hand side that produces
/// a different type than the use site wants is still caught — by the
/// consuming construct's rule ([FN-1] at `return`), not by the vanished
/// annotation.
#[test]
fn a_derived_binding_still_faces_its_consumer_s_exactness_rule() {
    assert_rule(
        br#"fn answer() -> result: own i32 pure {
  let value = 40_i64;
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
}

#[test]
fn a_value_match_derives_its_binding_from_the_delivery_set() {
    let source = br#"fn choose(flag: own Option<i32>) -> result: own i32 pure {
  let picked = match flag {
    Some(value: inner) => {
      give inner;
    }
    None() => {
      give 0_i32;
    }
  }
  return picked;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("a delivery set of one exact type must derive: {outcome:?}");
        };
    });
}

/// [GIVE-1] derivation is agreement over the closed delivery set, never a
/// join or a widening, and the citation lands on the *later* `give`.
#[test]
fn a_second_give_of_another_type_rejects_at_that_give() {
    assert_rule(
        br#"fn choose(flag: own Option<i32>) -> result: own unit pure {
  let picked = match flag {
    Some(value: inner) => {
      give inner;
    }
    None() => {
      give 0_i64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Give1,
        SemanticIssueKind::TypeMismatch,
    );
}

/// [GIVE-1] an empty delivery set — every arm leaves by `return` — has no
/// type to derive, and its mechanical fix is the statement form with the
/// binding dropped, so the citation is at the `let_stmt` rather than at any
/// arm.
#[test]
fn an_empty_delivery_set_rejects_at_the_let_statement() {
    assert_rule(
        br#"fn choose(flag: own Option<i32>) -> result: own i32 pure {
  let picked = match flag {
    Some(value: inner) => {
      return inner;
    }
    None() => {
      return 0_i32;
    }
  }
  return picked;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Give1,
        SemanticIssueKind::InvalidGive,
    );
}

/// The round-3 blocker's own witness. `None()` carries no operand, and after
/// the annotation is deleted the written arguments are the only supply there
/// is; [TYPE-5] therefore makes them mandatory in every position.
#[test]
fn a_nullary_prelude_construction_types_itself_from_written_arguments() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let absent = None<buffer<u8>>();
  let present = Some<i32>(value: 7_i32);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("written prelude arguments must type the construction: {outcome:?}");
        };
        let names = checked
            .data
            .nominals
            .iter()
            .map(|nominal| nominal.name.as_str())
            .collect::<Vec<_>>();
        for expected in ["Option<buffer<u8>>", "Option<i32>"] {
            assert!(
                names.contains(&expected),
                "missing instance {expected} derived from written arguments: {names:?}"
            );
        }
    });
}

#[test]
fn a_prelude_construction_without_its_arguments_rejects_at_the_construct() {
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let absent = None();
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn a_result_construction_writes_both_of_its_arguments() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let good = Ok<i32, Overflow>(value: 1_i32);
  let flag = Overflow();
  let bad = Err<i32, Overflow>(error: flag);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("written Result arguments must type both variants: {outcome:?}");
        };
        let names = checked
            .data
            .nominals
            .iter()
            .map(|nominal| nominal.name.as_str())
            .collect::<Vec<_>>();
        assert!(
            names.contains(&"Result<i32, Overflow>"),
            "missing Result instance derived from written arguments: {names:?}"
        );
    });
}

#[test]
fn a_table_operation_selects_its_row_from_its_operands() {
    let source = br#"fn smaller(x: own i32, y: own i32) -> result: own i32 pure {
  return imin(x, y);
}

fn widest(x: own u64, y: own u64) -> result: own u64 pure {
  return imin(x, y);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("one spelling must select two rows by operand type: {outcome:?}");
        };
    });
}

/// [OP-2] a written type argument on a deleted-class operation cites OP-1.
/// This is the judgment that inverted: v0.22 cited FN-2 for its *absence*.
#[test]
fn a_written_type_argument_on_a_derived_operation_rejects() {
    assert_rule(
        // The written argument is the violation, so deleting it — which is
        // what A1 does to a legal call — leaves nothing to cite.
        br#"fn smaller(x: own i32, y: own i32) -> result: own i32 pure {
  return imin<i32>(x, y);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

/// [OP-2] "Operands of two different exact types are a hard error citing
/// TYPE-5 at the second operand atom in source order" — so the citation is
/// pinned to those bytes, not merely to the rule.
#[test]
fn disagreeing_operands_cite_type5_at_the_second_operand_atom() {
    assert_rule_at(
        br#"fn smaller(x: own i32, y: own i64) -> result: own i32 pure {
  return imin(x, y);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        "y",
    );
}

/// The first operand fixes the row, so an operand type outside the closed
/// integer set is the *selection* failing, which cites OP-1 rather than the
/// per-operand TYPE-5.
#[test]
fn a_first_operand_outside_the_closed_set_cites_op1() {
    assert_rule(
        br#"fn smaller(x: own Bool, y: own Bool) -> result: own Bool pure {
  return imin(x, y);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

/// [OP-9] `buffer_new(n, v)` is the one deleted-class row that selects from
/// its second operand, and `len` then derives from the place it is given.
#[test]
fn buffer_new_selects_its_element_from_the_fill_value() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let data = buffer_new(4_u64, 7_u8);
  let count = len(data);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("buffer_new must select its element from the fill value: {outcome:?}");
        };
    });
}

/// [STOR-5] the written referent type used to carry the box-content
/// judgment. With it deleted the derived referent carries it, cited at the
/// operand that supplied it.
#[test]
fn box_content_that_bears_a_region_still_rejects_under_stor5() {
    assert_rule_at(
        br#"fn invalid['r](value: own slice<'r, u8>) -> result: own unit allocates(heap) {
  box_new(move value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        "move value",
    );
}
