use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedExpression, CheckedStatement, CheckedType, CheckedValue, IntegerType,
};
use super::{assert_rule, with_semantics};

const ENDPOINT_TERM_FIX: &str =
    "bind the computed u64 value with one preceding ordinary let and use that term as the endpoint";

fn assert_checks(source: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("counted range must check: {outcome:?}");
        };
    });
}

#[test]
fn counted_range_retains_checked_inputs_binder_and_real_exhaustion() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  for @items i in 2_u64..1_u64 {
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("break-free zero-trip counted range must check: {outcome:?}");
        };
        let CheckedStatement::CountedRange {
            node_path,
            binder,
            lower,
            upper,
            body,
            backedge_drops,
            ..
        } = &checked.data.functions[0].body[0]
        else {
            panic!("counted source must retain its dedicated checked node");
        };
        assert!(!node_path.components().is_empty());
        assert_eq!(binder.0, 0);
        assert!(matches!(
            lower,
            CheckedExpression::Constant(CheckedValue::Integer {
                ty: IntegerType::U64,
                bits: 2
            })
        ));
        assert!(matches!(
            upper,
            CheckedExpression::Constant(CheckedValue::Integer {
                ty: IntegerType::U64,
                bits: 1
            })
        ));
        assert!(body.is_empty());
        assert!(backedge_drops.is_empty());
        assert!(!checked.data.functions[0].declared_traps);
    });

    assert_checks(
        br#"command fn main() -> status: own ExitStatus pure {
  for @items i in 18446744073709551614_u64..18446744073709551615_u64 {
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn counted_endpoints_require_exact_own_u64_with_type7_exclusive() {
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  for @items i in 0_u32..1_u64 {
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );

    assert_rule(
        br#"fn walk['r](start: &'r u64) -> result: own unit pure {
  for @items i in start..1_u64 {
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );

    assert_rule(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let start = box_new(0_u64);
  for @items i in start..1_u64 {
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );

    assert_rule(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let start = box_new(0_u64);
  loop @outer {
    for @items i in start..1_u64 {
    }
    break @outer;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );

    assert_checks(
        br#"fn walk['l, 'u](lower: &'l u64, upper: &'u u64) -> result: own unit reads(lower, upper) {
  for @items i in deref(lower)..deref(upper) {
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn counted_endpoints_require_a_preceding_term_or_constant() {
    let subscript = br#"fn probe(bounds: own array<u64, 2>) -> result: own unit pure {
  for @items i in bounds[0_u64]..bounds[1_u64] {
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        subscript,
        SemanticRule::Ent2,
        SemanticIssueKind::InvalidCountedEndpoint {
            mechanical_fix: ENDPOINT_TERM_FIX,
        },
    );
    super::assert_rule_at(subscript, SemanticRule::Ent2, "bounds[0_u64]");

    assert_checks(
        br#"struct Bounds {
  lower: u64;
}

fn probe['r](bounds: own Bounds, upper: &'r u64) -> result: own unit reads(bounds.lower, upper) {
  for @items i in bounds.lower..deref(upper) {
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn counted_binder_is_not_source_writable_or_uniquely_borrowable() {
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  for @items i in 0_u64..1_u64 {
    set i = 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Set1,
        SemanticIssueKind::InvalidSetTarget {
            root_class: "compiler-updated counted binder".to_owned(),
            required_classes: "source-writable live own storage or a live usable &uniq referent",
        },
    );

    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  for @items i in 0_u64..1_u64 {
    region 'body {
      let exclusive = &uniq 'body i;
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::BorrowConflict,
    );

    assert_rule(
        br#"fn overwrite['r](target: &uniq 'r u64) -> result: own unit writes(target) {
  set deref(target) = 9_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  for @items i in 0_u64..1_u64 {
    region 'body {
      overwrite<'body>(target: &uniq 'body i);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn counted_body_inherits_own11_and_accepts_body_local_ownership() {
    assert_rule(
        br#"struct Token {
  value: u64;
}

command fn main() -> status: own ExitStatus pure {
  let token = Token(value: 1_u64);
  for @items i in 0_u64..1_u64 {
    let consumed = move token;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::MoveOuterBindingInLoop {
            mechanical_fix: "move the binding before the loop or declare and consume it inside the loop body",
        },
    );

    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let value = 0_u64;
  region 'outer {
    for @items i in 0_u64..1_u64 {
      let shared = &'outer value;
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::BorrowRegionOutsideLoop {
            mechanical_fix: "introduce the borrow region inside the enclosing loop body",
        },
    );

    assert_checks(
        br#"struct Token {
  value: u64;
}

command fn main() -> status: own ExitStatus pure {
  for @items i in 0_u64..1_u64 {
    region 'body {
      let shared = &'body i;
    }
    let token = Token(value: i);
    let consumed = move token;
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn counted_cleanup_is_attached_only_to_taken_body_exits() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  for @items i in 0_u64..1_u64 {
    let values = buffer_new(1_u64, 0_u8);
    break @items;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("counted break cleanup must check: {outcome:?}");
        };
        let CheckedStatement::CountedRange {
            body,
            backedge_drops,
            ..
        } = &checked.data.functions[0].body[0]
        else {
            panic!("expected counted range");
        };
        assert!(backedge_drops.is_empty());
        let CheckedStatement::Break { drops, .. } = &body[1] else {
            panic!("expected local counted break");
        };
        assert_eq!(drops.len(), 1);
        assert!(matches!(drops[0].ty, CheckedType::Buffer { .. }));
    });
}

#[test]
fn counted_return_and_propagate_edges_reuse_exact_cleanup() {
    let source = br#"enum Fail {
  Bad();
}

fn source() -> result: own Result<u64, Fail> pure {
  return Ok<u64, Fail>(value: 1_u64);
}

fn leave() -> result: own unit allocates(heap) {
  for @items i in 0_u64..1_u64 {
    let values = buffer_new(1_u64, 0_u8);
    return unit;
  }
  return unit;
}

fn forward() -> result: own Result<unit, Fail> allocates(heap) {
  for @items i in 0_u64..1_u64 {
    let values = buffer_new(1_u64, 0_u8);
    let value = propagate source();
  }
  return Ok<unit, Fail>(value: unit);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("counted return and propagation cleanup must check: {outcome:?}");
        };
        let leave = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "leave")
            .expect("leave must be checked");
        let CheckedStatement::CountedRange {
            body,
            backedge_drops,
            ..
        } = &leave.body[0]
        else {
            panic!("leave must retain its counted range");
        };
        assert!(backedge_drops.is_empty());
        let CheckedStatement::Return { drops, .. } = &body[1] else {
            panic!("leave must retain its return edge");
        };
        assert_eq!(drops.len(), 1);
        assert!(matches!(drops[0].ty, CheckedType::Buffer { .. }));

        let forward = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "forward")
            .expect("forward must be checked");
        let CheckedStatement::CountedRange {
            body,
            backedge_drops,
            ..
        } = &forward.body[0]
        else {
            panic!("forward must retain its counted range");
        };
        assert_eq!(backedge_drops.len(), 1);
        assert!(matches!(backedge_drops[0].ty, CheckedType::Buffer { .. }));
        let CheckedStatement::PropagateLet { error_drops, .. } = &body[1] else {
            panic!("forward must retain its propagation edge");
        };
        assert_eq!(error_drops.len(), 1);
        assert!(matches!(error_drops[0].ty, CheckedType::Buffer { .. }));
        assert_eq!(error_drops[0].binding, backedge_drops[0].binding);
    });
}

#[test]
fn counted_range_forwards_breaks_to_an_enclosing_loop() {
    assert_checks(
        br#"command fn main() -> status: own ExitStatus pure {
  loop @outer {
    for @items i in 0_u64..1_u64 {
      break @outer;
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}
