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
        // The migrated fixture pre-binds both column lengths for its claims,
        // so the loop follows the two length lets and the index let.
        let CheckedStatement::Loop { body, .. } = &fill.body[3] else {
            panic!("fill must retain its loop");
        };
        let CheckedStatement::Match { arms, .. } = &body[1] else {
            panic!("fill loop must retain its terminating match");
        };
        let CheckedStatement::Set { target, .. } = &arms[1].body[2] else {
            panic!("fill must write the left borrowed buffer after its claim");
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
    let source = br#"fn length['r](values: &'r buffer<u8>) -> own u64 reads('r) {
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
    set values[0_u64] = 1_u8;
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
        br#"fn two['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> own unit pure {
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
        br#"fn invalid['caller](values: own buffer<u8>) -> own unit pure {
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
    let source =
        br#"fn write['r](out: &uniq 'r buffer<u8>) -> own unit reads('r), writes('r), traps {
  let room: own u64 = len<u8>(deref(out));
  let ok: own Bool = ilt<u64>(0_u64, room);
  claim has_room: ok because "callers pass a nonempty buffer";
  set deref(out)[0_u64] = 1_u8;
  return unit;
}

fn proxy['r](out: &uniq 'r buffer<u8>) -> own unit reads('r), writes('r), traps {
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

fn count['r](pool: &'r Pool) -> own u64 reads('r) {
  return deref(pool).count;
}

fn first['r](pool: &'r Pool) -> own u64 reads('r), traps {
  let room: own u64 = len<u64>(deref(pool).left);
  let ok: own Bool = ilt<u64>(0_u64, room);
  claim left_nonempty: ok because "callers pool at least one element per column";
  return deref(pool).left[0_u64];
}

fn update['r](pool: &uniq 'r Pool) -> own unit reads('r), writes('r), traps {
  let room: own u64 = len<u64>(deref(pool).right);
  let ok: own Bool = ilt<u64>(0_u64, room);
  claim right_nonempty: ok because "callers pool at least one element per column";
  set deref(pool).right[0_u64] = 9_u64;
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

        // The subscripting helpers open with a length let, a comparison
        // let, and the discharging claim; their subscript statements follow.
        let CheckedStatement::Return {
            value: CheckedExpression::BufferIndex { root, .. },
            ..
        } = &checked.data.functions[1].body[3]
        else {
            panic!("borrowed buffer field read must retain its checked root");
        };
        assert_eq!(root.fields, [0]);

        let update = &checked.data.functions[2];
        let CheckedStatement::Set {
            target: CheckedSetTarget::BufferIndex(target),
            ..
        } = &update.body[3]
        else {
            panic!("borrowed buffer field write must retain its checked target");
        };
        assert_eq!(target.root.fields, [1]);
        let CheckedStatement::Set {
            target: CheckedSetTarget::Place(target),
            ..
        } = &update.body[4]
        else {
            panic!("borrowed copy field write must retain its checked target");
        };
        assert_eq!(target.fields, [2]);
    });
}

/// [SET-1] states the shared-borrow referent among the cases it hands to
/// another rule — "A shared-borrow referent ... is not writable [OWN-5]" —
/// and keeps only the residue of its writability relation. The rejection is
/// unconditional either way; the citation is [OWN-5].
#[test]
fn shared_struct_borrows_cannot_write_copy_fields() {
    assert_rule(
        br#"struct Counter {
  value: u64;
}

fn invalid['r](counter: &'r Counter) -> own unit writes('r) {
  set deref(counter).value = 1_u64;
  return unit;
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

fn steal['r](pool: &'r Pool) -> own buffer<u64> pure {
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

fn consume['r](counter: &uniq 'r Counter, value: own u64) -> own unit pure {
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

fn observe['r](counter: &'r Counter, value: own u64) -> own unit pure {
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

    assert_rule(
        br#"struct Owner {
  source: buffer<u8>;
  sibling: buffer<u8>;
}

fn consume['r](source: &'r buffer<u8>, sibling: own buffer<u8>) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  region 'r {
    consume<'r>(source: &'r owner.source, sibling: move owner.sibling);
  }
  return unit;
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
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
        br#"fn observe['r](out: &'r buffer<u8>) -> own unit pure {
  return unit;
}

fn proxy['r](out: &'r buffer<u8>) -> own unit pure {
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
        br#"fn take['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid['r](out: &'r buffer<u8>) -> own unit pure {
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
        br#"fn take['r](out: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid['r](out: &uniq 'r buffer<u8>) -> own unit pure {
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
        br#"fn take_two['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> own unit pure {
  return unit;
}

fn invalid['r](out: &uniq 'r buffer<u8>) -> own unit pure {
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
        br#"fn observe['r](out: &'r buffer<u8>) -> own unit pure {
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
        br#"fn observe['r](out: &'r buffer<u8>) -> own unit pure {
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

#[test]
fn borrow_mode_parameters_of_system_types_carry_the_ordinary_borrow_judgments() {
    // [SYS-4] gives every first-slice system type shared borrows and gives a
    // stateful resource `&uniq`, and [FN-1] attaches no type condition to a
    // parameter mode, so a user signature admits a borrowed system value on
    // the normal path. A statement-scoped child reborrow [OWN-6] then carries
    // it into a system operation whose own parameter is that same mode
    // [SYS-2]. An opaque resource has no source-visible content, so its
    // borrow is the value itself.
    let source = br#"fn publish['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, count: own u64) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: count) {
      Ok(value: written) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  region 'publication {
    publish<'publication, 'publication>(output: &uniq 'publication out, source: &'publication batch, count: 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a borrowed system parameter must check: {outcome:?}");
        };
        let publish = &checked.data.functions[0];
        assert!(matches!(publish.parameters[0].mode, CheckedMode::Unique(_)));
        let CheckedStatement::Region { body, .. } = &publish.body[0] else {
            panic!("publish must retain its attempt region");
        };
        let CheckedStatement::Match { scrutinee, .. } = &body[0] else {
            panic!("publish must retain its outcome match");
        };
        let CheckedExpression::SystemCall { arguments, .. } = scrutinee else {
            panic!("the scrutinee must be the system call");
        };
        assert!(matches!(
            &arguments[0],
            CheckedExpression::BorrowSystemResource { .. }
        ));
    });

    // The row is checked both ways over the borrowed parameter's region: the
    // write the operation performs through `&uniq 'o` is attributed to the
    // caller-supplied region, exactly as it is for a borrowed buffer [EFF-2].
    let declared = b"reads('o 's), writes('o), external";
    let at = source
        .windows(declared.len())
        .position(|window| window == declared)
        .expect("fixture declares the publish row");
    let mut narrowed = source.to_vec();
    narrowed.splice(
        at..at + declared.len(),
        b"reads('o 's), external".iter().copied(),
    );
    assert_rule(
        &narrowed,
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

/// General borrow-mode parameters and `let` borrows: scalar and enum content
/// is borrowed by the same machinery the buffer, struct, box, and system
/// families already use [OWN-2, TYPE-7, OWN-13].
#[test]
fn scalar_and_enum_borrows_check_read_write_and_match_through_the_holder() {
    let source = br#"enum Cell {
  Full(v: i32);
  Void();
}

fn read_scalar['r](p: &'r i32) -> own i32 reads('r) {
  return deref(p);
}

fn bump['r](p: &uniq 'r i32) -> own unit writes('r) {
  set deref(p) = 9_i32;
  return unit;
}

fn score['r](c: &'r Cell) -> own i32 reads('r), traps {
  match deref(c) {
    Full(v: x) => {
      return iadd.trap<i32>(deref(x), 1_i32);
    }
    Void() => {
      return 0_i32;
    }
  }
}

fn main() -> own unit traps {
  let a: own i32 = 5_i32;
  region 'r {
    let s: &'r i32 = &'r a;
    check ieq<i32>(deref(s), 5_i32) else trap "read";
  }
  region 'q {
    let u: &uniq 'q i32 = &uniq 'q a;
    set deref(u) = 7_i32;
  }
  check ieq<i32>(a, 7_i32) else trap "write";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("general borrow-mode parameters must check: {outcome:?}");
        };
        assert!(matches!(
            checked.data.functions[0].parameters[0].mode,
            CheckedMode::Shared(_)
        ));
        assert!(matches!(
            checked.data.functions[1].parameters[0].mode,
            CheckedMode::Unique(_)
        ));
        // The read through the holder is its own checked node, so lowering
        // never confuses the holder with its referent.
        let CheckedStatement::Return {
            value: CheckedExpression::DerefAddressed { .. },
            ..
        } = &checked.data.functions[0].body[0]
        else {
            panic!("a scalar deref read must retain its checked node");
        };
        // Matching through `&'r` derives shared payload binders [OWN-13].
        let CheckedStatement::Match { arms, .. } = &checked.data.functions[2].body[0] else {
            panic!("score must retain its borrowed match");
        };
        assert!(matches!(arms[0].binders[0].mode, CheckedMode::Shared(_)));
        // The borrowed root carries the address; the holder is that address.
        let main = &checked.data.functions[3];
        let CheckedStatement::Region { body, .. } = &main.body[1] else {
            panic!("main must retain its first region");
        };
        assert!(matches!(
            &body[0],
            CheckedStatement::Let {
                value: CheckedExpression::BorrowAddressed { .. },
                ..
            }
        ));
    });
}

/// The near misses the same admission must keep rejecting.
#[test]
fn general_borrows_keep_their_escape_read_and_exclusivity_rejections() {
    // [OWN-10]: a caller-supplied region outlives the frame that owns `x`.
    assert_rule(
        b"fn dangle['r0](x: own i32) -> &'r0 i32 pure {\n  return &'r0 x;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own10,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
    // [OWN-4]: a borrow narrowed to an inner region cannot be returned as the
    // caller's region.
    assert_rule(
        b"fn leak['r0](x: &'r0 i32) -> &'r0 i32 pure {\n  region 's {\n    let q: &'s i32 = x;\n    return q;\n  }\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own4,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
    // [TYPE-7]: no implicit read through a scalar holder.
    assert_rule(
        b"fn read['r](holder: &'r i32) -> own i32 pure {\n  return holder;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: a bare holder is not an enum value, so it cannot be matched.
    assert_rule(
        b"enum State {\n  Ready();\n  Done();\n}\n\nfn main() -> own unit pure {\n  let state: own State = Ready();\n  region 'r {\n    let holder: &'r State = &'r state;\n    match holder {\n      Ready() => {\n      }\n      Done() => {\n      }\n    }\n  }\n  return unit;\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: neither is a `borrow_expr`.
    assert_rule(
        b"enum State {\n  Ready();\n}\n\nfn main() -> own unit pure {\n  let state: own State = Ready();\n  region 'r {\n    match &'r state {\n      Ready() => {\n      }\n    }\n  }\n  return unit;\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: nor a reference-returning call's result.
    assert_rule(
        b"enum State {\n  Ready();\n}\n\nfn view['r](state: &'r State) -> &'r State pure {\n  return state;\n}\n\nfn inspect['r](state: &'r State) -> own unit pure {\n  match view<'r>(state: state) {\n    Ready() => {\n    }\n  }\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [OWN-5]: a shared holder never makes its referent writable.
    assert_rule(
        b"fn main() -> own unit pure {\n  let a: own i32 = 1_i32;\n  region 'r {\n    let s: &'r i32 = &'r a;\n    set deref(s) = 9_i32;\n  }\n  return unit;\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // [OWN-5]: two live uniq borrows of one scalar place overlap.
    assert_rule(
        b"fn main() -> own unit pure {\n  let a: own i32 = 3_i32;\n  region 'r {\n    let u1: &uniq 'r i32 = &uniq 'r a;\n    let u2: &uniq 'r i32 = &uniq 'r a;\n  }\n  return unit;\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // [OWN-12]: two uniq arguments over one place alias at the call.
    assert_rule(
        b"fn two['r](a: &uniq 'r i32, b: &uniq 'r i32) -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  let x: own i32 = 0_i32;\n  region 'r {\n    two<'r>(a: &uniq 'r x, b: &uniq 'r x);\n  }\n  return unit;\n}\n",
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

/// [EFF-2] §9.1 attributes reads and writes through an incoming borrow
/// parameter to that parameter's formal region, both ways.
#[test]
fn scalar_borrow_parameter_effect_rows_are_exact_in_both_directions() {
    assert_rule(
        b"fn read_scalar['r](p: &'r i32) -> own i32 pure {\n  return deref(p);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn bump['r](p: &uniq 'r i32) -> own unit reads('r) {\n  set deref(p) = 9_i32;\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn quiet['r](p: &'r i32) -> own unit reads('r) {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

/// [OWN-14] admits the written reborrow as the complete return expression
/// from a parameter or let-bound holder with preserved mode; the created
/// borrow then carries the existing [OWN-10]/[OWN-4] region judgments, so a
/// callee-local region cannot leave through the signature.
#[test]
fn returned_reborrows_follow_own14_admission_and_own4_regions() {
    with_semantics(
        b"fn passthru['r0](x: &'r0 i32) -> &'r0 i32 pure {\n  return &'r0 deref(x);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared returned reborrow of a parameter must check: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"fn passthru['r0](x: &uniq 'r0 i32) -> &uniq 'r0 i32 pure {\n  return &uniq 'r0 deref(x);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a unique returned reborrow of a parameter must check: {outcome:?}");
            };
        },
    );
    // [OWN-4]: the returned borrow's local region cannot reach the written
    // rtype region, in either mode.
    assert_rule(
        b"fn leak['r0](x: &'r0 i32) -> &'r0 i32 pure {\n  region 's {\n    return &'s deref(x);\n  }\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own4,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
    assert_rule(
        b"fn leak['r0](x: &uniq 'r0 i32) -> &uniq 'r0 i32 pure {\n  region 's {\n    return &uniq 's deref(x);\n  }\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own4,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
}

/// [OWN-14] rejects every written reborrow form outside its two admitted
/// positions, and a return-position reborrow failing the admission: a bound
/// reborrow, a mode-downgrading returned reborrow, and a `match`-binder
/// holder are each the OWN-14 hard error with its restructuring.
#[test]
fn non_admitted_reborrow_forms_are_own14_hard_errors() {
    const RESTRUCTURING: &str = "pass the reborrow as a statement-scoped child in argument position, \
         return it as the complete return expression from a parameter or let-bound holder, \
         or return the holder itself";
    assert_rule(
        b"fn bind['r](x: &'r i32) -> own unit pure {\n  region 'c {\n    let y: &'c i32 = &'c deref(x);\n  }\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own14,
        SemanticIssueKind::InvalidReborrowPosition {
            mechanical_fix: RESTRUCTURING,
        },
    );
    assert_rule(
        b"fn down['r0](x: &uniq 'r0 i32) -> &'r0 i32 pure {\n  return &'r0 deref(x);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own14,
        SemanticIssueKind::InvalidReborrowPosition {
            mechanical_fix: RESTRUCTURING,
        },
    );
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn pick['r](holder: &'r Packet) -> &'r i32 reads('r) {\n  match deref(holder) {\n    Data(value: payload) => {\n      return &'r deref(payload);\n    }\n  }\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own14,
        SemanticIssueKind::InvalidReborrowPosition {
            mechanical_fix: RESTRUCTURING,
        },
    );
}

/// [OWN-13] the borrow-mode payload binder is an arm-scoped child reborrow of
/// the scrutinee place's root binding: usable within its arm from a `uniq`
/// root, whose suspension the binder creation establishes; shared roots stay
/// plain overlapping shared borrows, and a shared binder roots the next
/// scrutinee in a chain.
#[test]
fn arm_scoped_child_reborrows_admit_payload_uses() {
    with_semantics(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn main() -> own unit pure {\n  let packet: own Packet = Data(value: 4_i32);\n  region 'r {\n    let holder: &uniq 'r Packet = &uniq 'r packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        let saved: own i32 = deref(payload);\n      }\n    }\n  }\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a uniq-match payload read through its binder must check: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"enum Packet {\n  Data(value: i32);\n  Idle();\n}\n\nfn main() -> own unit pure {\n  let packet: own Packet = Data(value: 4_i32);\n  region 'r {\n    let holder: &'r Packet = &'r packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        let saved: own i32 = deref(payload);\n      }\n      Idle() => {\n      }\n    }\n    match deref(holder) {\n      Data(value: payload) => {\n        let again: own i32 = deref(payload);\n      }\n      Idle() => {\n      }\n    }\n  }\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared root is never suspended and matches again: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"enum Inner {\n  Leaf(value: i32);\n}\n\nenum Outer {\n  Wrap(inner: Inner);\n}\n\nfn main() -> own unit pure {\n  let leaf: own Inner = Leaf(value: 7_i32);\n  let packet: own Outer = Wrap(inner: move leaf);\n  region 'r {\n    let holder: &'r Outer = &'r packet;\n    match deref(holder) {\n      Wrap(inner: nested) => {\n        match deref(nested) {\n          Leaf(value: payload) => {\n            let saved: own i32 = deref(payload);\n          }\n        }\n      }\n    }\n  }\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared binder must root the next scrutinee: {outcome:?}");
            };
        },
    );
}

/// [OWN-13] a matched-through `uniq` root does not resume within its region:
/// its post-match use is the OWN-5 suspension rejection on every path, in-arm
/// use is rejected the same way, and the suspension joins across arms that
/// did and did not create binders.
#[test]
fn suspended_uniq_match_roots_do_not_resume() {
    // In-arm reuse of the suspended root.
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn main() -> own unit pure {\n  let packet: own Packet = Data(value: 4_i32);\n  region 'r {\n    let holder: &uniq 'r Packet = &uniq 'r packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        match deref(holder) {\n          Data(value: other) => {\n          }\n        }\n      }\n    }\n  }\n  return unit;\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // Post-match reuse, joined across a binder-creating and a binder-free arm.
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n  Idle();\n}\n\nfn main() -> own unit pure {\n  let packet: own Packet = Data(value: 4_i32);\n  region 'r {\n    let holder: &uniq 'r Packet = &uniq 'r packet;\n    match deref(holder) {\n      Data(value: payload) => {\n      }\n      Idle() => {\n      }\n    }\n    match deref(holder) {\n      Data(value: payload) => {\n      }\n      Idle() => {\n      }\n    }\n  }\n  return unit;\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // A returned reborrow is not created through a suspended holder.
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn peek['r](holder: &uniq 'r Packet) -> &uniq 'r Packet reads('r) {\n  match deref(holder) {\n    Data(value: payload) => {\n    }\n  }\n  return &uniq 'r deref(holder);\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

/// [DIAG-1] one offending use establishing several rejections cites the
/// first-defined established rule: a holder returned where the written
/// `rtype` requires its referent value cites TYPE-7 (with FN-1 forming no
/// candidate), ahead of OWN-1's spelling judgments at the same node; a
/// holder returned where the holder itself is required stays OWN-1.
#[test]
fn same_node_return_rejections_cite_the_first_defined_rule() {
    let type7 = SemanticIssueKind::MissingDereference {
        mechanical_fix: "write `deref(holder)`",
    };
    assert_rule(
        b"fn read(holder: own box<i32>) -> own i32 pure {\n  return holder;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type7,
        type7.clone(),
    );
    assert_rule(
        b"fn read(holder: own box<i32>) -> own i32 pure {\n  return move holder;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type7,
        type7.clone(),
    );
    assert_rule(
        b"fn grab['r](p: &uniq 'r i32) -> own i32 pure {\n  return p;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type7,
        type7,
    );
    // The referent is not required here, so TYPE-7 is not established and
    // OWN-1's bare-affine spelling is the sole rejection.
    assert_rule(
        b"fn pass(holder: own box<i32>) -> own box<i32> pure {\n  return holder;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Own1,
        SemanticIssueKind::BareAffineUse {
            mechanical_fix: "write `move p` for the affine place",
        },
    );
}
