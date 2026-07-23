use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedExpression, CheckedMode, CheckedSetTarget, CheckedStatement};
use super::{assert_rule, with_semantics};

pub(super) const BORROWED_COLUMNS: &[u8] =
    include_bytes!("../../../../tests/conformance/cases/x-buffer-borrowed-columns-run.wf");

#[test]
fn buffer_borrows_keep_modes_provenance_effects_and_distinct_field_loans() {
    with_semantics(BORROWED_COLUMNS, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("borrowed buffer helpers must check: {outcome:?}");
        };
        let fill = &checked.data.functions[0];
        assert!(matches!(fill.parameters[0].mode, CheckedMode::Unique(_)));
        let CheckedStatement::Loop { body, .. } = &fill.body[1] else {
            panic!("fill must retain its loop");
        };
        let CheckedStatement::Match { arms, .. } = &body[1] else {
            panic!("fill loop must retain its terminating match");
        };
        let CheckedStatement::Set { target, .. } = &arms[1].body[0] else {
            panic!("fill must write the left borrowed buffer");
        };
        assert!(matches!(target, CheckedSetTarget::BufferIndex(_)));

        let main = &checked.data.functions[2];
        let CheckedStatement::Region { body, .. } = &main.body[4] else {
            panic!("main must retain the fill region");
        };
        assert!(matches!(
            &body[0],
            CheckedStatement::Let {
                value: CheckedExpression::BorrowBuffer { root },
                ..
            } if root.fields == [0]
        ));
        assert!(matches!(
            &body[1],
            CheckedStatement::Let {
                value: CheckedExpression::BorrowBuffer { root },
                ..
            } if root.fields == [1]
        ));
    });
}

#[test]
fn borrowed_column_effect_rows_are_exact() {
    let wrong = BORROWED_COLUMNS
        .windows(b"writes('r), traps".len())
        .position(|window| window == b"writes('r), traps")
        .expect("fixture contains fill effects");
    let mut source = BORROWED_COLUMNS.to_vec();
    source.splice(
        wrong..wrong + b"writes('r), traps".len(),
        b"traps".iter().copied(),
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("missing write effect must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
}

#[test]
fn borrowed_buffer_length_exhibits_a_read_of_its_storage_origin() {
    let source = br#"fn length ['r](values: &'r buffer<u8>) -> own u64 reads('r) {
  return len<u8>(deref(values));
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("borrowed length must exhibit its incoming region read: {outcome:?}");
        };
    });
}

#[test]
fn live_buffer_loans_reject_overlapping_borrows_and_owner_writes() {
    assert_rule(
        br#"fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  region 'r {
    let first: &uniq 'r buffer<u8> = &uniq 'r values;
    let second: &uniq 'r buffer<u8> = &uniq 'r values;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  region 'r {
    let shared: &'r buffer<u8> = &'r values;
    set index<u8>(values, 0_u64) = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn user_calls_reject_overlapping_unique_arguments() {
    assert_rule(
        br#"fn two ['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  region 'r {
    two<'r>(first: &uniq 'r values, second: &uniq 'r values);
  }
  return unit;
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn own_storage_cannot_be_borrowed_into_a_caller_region() {
    assert_rule(
        br#"fn invalid ['caller](values: own buffer<u8>) -> own unit pure {
  let escaped: &'caller buffer<u8> = &'caller values;
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own10,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
}

#[test]
fn call_effects_preserve_the_incoming_storage_origin() {
    let source = br#"fn write ['r](out: &uniq 'r buffer<u8>) -> own unit writes('r), traps {
  set index<u8>(deref(out), 0_u64) = 1_u8;
  return unit;
}

fn proxy ['r](out: &uniq 'r buffer<u8>) -> own unit writes('r), traps {
  write<'r>(out: move out);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("incoming call effects must retain their formal origin: {outcome:?}");
        };
        assert!(checked.data.functions[1].declared_traps);
    });
}

#[test]
fn borrowed_struct_fields_keep_projection_provenance_and_exact_effects() {
    let source = br#"struct Pool {
  left: buffer<u64>;
  right: buffer<u64>;
  count: u64;
}

fn count ['r](pool: &'r Pool) -> own u64 reads('r) {
  return deref(pool).count;
}

fn first ['r](pool: &'r Pool) -> own u64 reads('r), traps {
  return index<u64>(deref(pool).left, 0_u64);
}

fn update ['r](pool: &uniq 'r Pool) -> own unit writes('r), traps {
  set index<u64>(deref(pool).right, 0_u64) = 9_u64;
  set deref(pool).count = 1_u64;
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("borrowed struct projections must check: {outcome:?}");
        };

        let CheckedStatement::Return {
            value:
                CheckedExpression::Project {
                    fields,
                    consume_root,
                    ..
                },
            ..
        } = &checked.data.functions[0].body[0]
        else {
            panic!("copy field read must retain one checked projection");
        };
        assert_eq!(fields, &[2]);
        assert!(!consume_root);

        let CheckedStatement::Return {
            value: CheckedExpression::BufferIndex { root, .. },
            ..
        } = &checked.data.functions[1].body[0]
        else {
            panic!("borrowed buffer field read must retain its checked root");
        };
        assert_eq!(root.fields, [0]);

        let update = &checked.data.functions[2];
        let CheckedStatement::Set {
            target: CheckedSetTarget::BufferIndex(target),
            ..
        } = &update.body[0]
        else {
            panic!("borrowed buffer field write must retain its checked target");
        };
        assert_eq!(target.root.fields, [1]);
        let CheckedStatement::Set {
            target: CheckedSetTarget::Place(target),
            ..
        } = &update.body[1]
        else {
            panic!("borrowed copy field write must retain its checked target");
        };
        assert_eq!(target.fields, [2]);
    });
}

#[test]
fn shared_struct_borrows_cannot_write_copy_fields() {
    assert_rule(
        br#"struct Counter {
  value: u64;
}

fn invalid ['r](counter: &'r Counter) -> own unit writes('r) {
  set deref(counter).value = 1_u64;
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Set1,
        SemanticIssueKind::InvalidSetTarget {
            root_class: "shared borrow".to_owned(),
            required_classes: "live own storage or a live usable &uniq referent",
        },
    );
}

#[test]
fn struct_borrow_roots_block_owner_access_and_affine_moves() {
    assert_rule(
        br#"struct Pool {
  values: buffer<u64>;
  count: u64;
}

fn main() -> own unit allocates(heap), traps {
  let values: own buffer<u64> = buffer_new<u64>(1_u64, 0_u64);
  let pool: own Pool = Pool(values: move values, count: 0_u64);
  region 'r {
    let view: &'r Pool = &'r pool;
    set pool.count = 1_u64;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"struct Pool {
  values: buffer<u64>;
}

fn steal ['r](pool: &'r Pool) -> own buffer<u64> pure {
  return move deref(pool).values;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn call_scoped_struct_loans_are_checked_against_later_place_arguments() {
    assert_rule(
        br#"struct Counter {
  value: u64;
}

fn consume ['r](counter: &uniq 'r Counter, value: own u64) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  let counter: own Counter = Counter(value: 1_u64);
  region 'r {
    consume<'r>(counter: &uniq 'r counter, value: counter.value);
  }
  return unit;
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );

    with_semantics(
        br#"struct Counter {
  value: u64;
}

fn observe ['r](counter: &'r Counter, value: own u64) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  let counter: own Counter = Counter(value: 1_u64);
  region 'r {
    observe<'r>(counter: &'r counter, value: counter.value);
  }
  return unit;
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared call loan permits a later overlapping read: {outcome:?}");
            };
        },
    );
}

#[test]
fn child_reborrow_shape_and_sibling_exclusivity_follow_own6() {
    let positive = include_bytes!("../../../../tests/conformance/cases/x-child-reborrow-run.wf");
    with_semantics(positive, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("statement-scoped child reborrows must check: {outcome:?}");
        };
    });

    with_semantics(
        br#"fn observe ['r](out: &'r buffer<u8>) -> own unit pure {
  return unit;
}

fn proxy ['r](out: &'r buffer<u8>) -> own unit pure {
  region 'child {
    observe<'child>(out: &'child deref(out));
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared child of a shared holder must check: {outcome:?}");
            };
        },
    );

    assert_rule(
        br#"fn take ['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid ['r](out: &'r buffer<u8>) -> own unit pure {
  region 'child {
    take<'child>(out: &uniq 'child deref(out));
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own6,
        SemanticIssueKind::InvalidChildReborrow,
    );

    assert_rule(
        br#"fn take ['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid ['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  region 'child {
    take<'child>(out: &uniq 'child deref(out));
    take<'child>(out: &uniq 'child deref(out));
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own6,
        SemanticIssueKind::InvalidChildReborrow,
    );

    assert_rule(
        br#"fn take_two ['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid ['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  region 'child {
    take_two<'child>(first: &uniq 'child deref(out), second: &uniq 'child deref(out));
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );

    with_semantics(
        br#"fn observe ['r](out: &'r buffer<u8>) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let out: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  loop @once {
    region 'inside {
      observe<'inside>(out: &'inside out);
    }
    break @once;
  }
  return unit;
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a loop-local borrow region must check: {outcome:?}");
            };
        },
    );

    assert_rule(
        br#"fn observe ['r](out: &'r buffer<u8>) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let out: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  region 'outside {
    loop @once {
      observe<'outside>(out: &'outside out);
      break @once;
    }
  }
  return unit;
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::BorrowRegionOutsideLoop {
            mechanical_fix: "introduce the borrow region inside the enclosing loop body",
        },
    );
}
