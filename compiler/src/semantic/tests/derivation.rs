//! [TYPE-5] and [GIVE-1] derivation: a `let` binder's mode and type come
//! from its selected right-hand side, never from a written annotation, and a
//! value initializer's come from its delivery set.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_rule, with_semantics};

#[test]
fn an_ordinary_let_takes_the_type_its_right_hand_side_produces() {
    let source = br#"fn answer() -> own i32 pure {
  let value = 40_i32;
  return value;
}

fn main() -> own unit pure {
  return unit;
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
        br#"fn answer() -> own i32 pure {
  let value = 40_i64;
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
}

#[test]
fn a_value_match_derives_its_binding_from_the_delivery_set() {
    let source = br#"fn choose(flag: own Option<i32>) -> own i32 pure {
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

fn main() -> own unit pure {
  return unit;
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
        br#"fn choose(flag: own Option<i32>) -> own unit pure {
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

fn main() -> own unit pure {
  return unit;
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
        br#"fn choose(flag: own Option<i32>) -> own i32 pure {
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

fn main() -> own unit pure {
  return unit;
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
    let source = br#"fn main() -> own unit pure {
  let absent = None<buffer<u8>>();
  let present = Some<i32>(value: 7_i32);
  return unit;
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
        br#"fn main() -> own unit pure {
  let absent = None();
  return unit;
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn a_result_construction_writes_both_of_its_arguments() {
    let source = br#"fn main() -> own unit pure {
  let good = Ok<i32, Overflow>(value: 1_i32);
  let flag = Overflow();
  let bad = Err<i32, Overflow>(error: flag);
  return unit;
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
