use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedExpression, CheckedMode, CheckedSetTarget, CheckedStatement};
use super::{
    assert_rule, assert_rule_extension, assert_rule_kind, assert_unsupported, with_semantics,
    with_semantics_extension,
};

pub(super) const BORROWED_COLUMNS: &[u8] = br#"struct Columns {
  left: buffer<u64>;
  right: buffer<u64>;
}

fn fill['r](left: &uniq 'r buffer<u64>, right: &uniq 'r buffer<u64>, length: own u64) -> function_result: own unit reads(left, right), writes(left, right) {
  let left_room = len_of(deref(left));
  let right_room = len_of(deref(right));
  let index_value = 0_u64;
  loop @fill {
    let done = index_value == length;
    if done {
      break @fill;
    } else {
      let in_left = index_value < left_room;
      if in_left {
        set deref(left)[index_value] = index_value;
      }
      let shifted = index_value +wrap 10_u64;
      let in_right = index_value < right_room;
      if in_right {
        set deref(right)[index_value] = shifted;
      }
      set index_value = index_value +wrap 1_u64;
    }
  }
  return unit;
}

fn fold['r](left: &'r buffer<u64>, right: &'r buffer<u64>, length: own u64) -> function_result: own u64 reads(left, right) {
  let left_room = len_of(deref(left));
  let right_room = len_of(deref(right));
  let index_value = 0_u64;
  let total = 0_u64;
  loop @fold {
    let done = index_value == length;
    if done {
      break @fold;
    } else {
      let in_left = index_value < left_room;
      let left_value = if in_left {
        give deref(left)[index_value];
      } else {
        give 0_u64;
      }
      let in_right = index_value < right_room;
      let right_value = if in_right {
        give deref(right)[index_value];
      } else {
        give 0_u64;
      }
      set total = total +wrap left_value;
      set total = total +wrap right_value;
      set index_value = index_value +wrap 1_u64;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  let length = 4_u64;
  let left = buffer_new(length, 0_u64);
  let right = buffer_new(length, 0_u64);
  let columns = Columns(left: move left, right: move right);
  region {
    let left_out = &uniq columns.left;
    let right_out = &uniq columns.right;
    fill(left: move left_out, right: move right_out, length: length);
  }
  region {
    let left_in = &columns.left;
    let right_in = &columns.right;
    let total = fold(left: left_in, right: right_in, length: length);
  }
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn buffer_borrows_keep_modes_provenance_effects_and_distinct_field_loans() {
    with_semantics(BORROWED_COLUMNS, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("borrowed buffer helpers must check: {outcome:?}");
        };
        let fill = &checked.data.functions[0];
        assert!(matches!(fill.parameters[0].mode, CheckedMode::Unique(_)));
        // The migrated fixture pre-binds both column lengths for its guards,
        // so the loop follows the two length lets and the index let.
        let CheckedStatement::Loop { body, .. } = &fill.body[3] else {
            panic!("fill must retain its loop");
        };
        let CheckedStatement::Match { arms, .. } = &body[1] else {
            panic!("fill loop must retain its terminating match");
        };
        let CheckedStatement::Match {
            arms: guard_arms, ..
        } = &arms[1].body[1]
        else {
            panic!("fill must retain the explicit left bounds guard");
        };
        let target = guard_arms
            .iter()
            .flat_map(|arm| &arm.body)
            .find_map(|statement| match statement {
                CheckedStatement::Set { target, .. } => Some(target),
                _ => None,
            })
            .expect("the true guard arm writes the left borrowed buffer");
        assert!(matches!(target, CheckedSetTarget::BufferIndex(_)));

        let main = &checked.data.functions[2];
        let CheckedStatement::Region { body, .. } = &main.body[4] else {
            panic!("main must retain the fill region");
        };
        assert!(matches!(
            &body[0],
            CheckedStatement::Let {
                value: CheckedExpression::BorrowBuffer { root, .. },
                ..
            } if root.fields == [0]
        ));
        assert!(matches!(
            &body[1],
            CheckedStatement::Let {
                value: CheckedExpression::BorrowBuffer { root, .. },
                ..
            } if root.fields == [1]
        ));
    });
}

#[test]
fn borrowed_column_effect_rows_are_exact() {
    let wrong = BORROWED_COLUMNS
        .windows(b"reads(left, right), writes(left, right)".len())
        .position(|window| window == b"reads(left, right), writes(left, right)")
        .expect("fixture contains fill effects");
    let mut source = BORROWED_COLUMNS.to_vec();
    source.splice(
        wrong..wrong + b"reads(left, right), writes(left, right)".len(),
        b"reads(left, right)".iter().copied(),
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
    let source = br#"fn length(values: &buffer<u8>) -> result: own u64 reads(values) {
  return len_of(deref(values));
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(1_u64, 0_u8);
  region {
    let first = &uniq values;
    let second = &uniq values;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(1_u64, 0_u8);
  region {
    let shared = &values;
    set values[0_u64] = 1_u8;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn user_calls_reject_overlapping_unique_arguments() {
    assert_rule(
        br#"fn two['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(1_u64, 0_u8);
  region {
    two(first: &uniq values, second: &uniq values);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn own_storage_cannot_be_borrowed_into_a_caller_region() {
    assert_rule_kind(
        br#"fn invalid['caller](values: own buffer<u8>, anchor: &'caller u8) -> result: &'caller u8 pure {
  let escaped = &'caller values;
  return anchor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
}

#[test]
fn call_effects_preserve_the_incoming_storage_origin() {
    let source = br#"fn write(out: &uniq buffer<u8>) -> result: own unit reads(out), writes(out) {
  let spare = len_of(deref(out));
  let ok = 0_u64 < spare;
  if ok {
    set deref(out)[0_u64] = 1_u8;
  }
  return unit;
}

fn proxy(out: &uniq buffer<u8>) -> result: own unit reads(out), writes(out) {
  write(out: move out);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("incoming call effects must retain their formal origin: {outcome:?}");
        };
    });
}

#[test]
fn state_paths_separate_identity_from_lifetime_and_substitute_fields() {
    let accepted = br#"struct Pair {
  left: u64;
  right: u64;
}

fn select_left['r](left: &'r Pair, right: &'r Pair) -> result: own u64 reads(left.left) {
  return deref(left).left;
}

fn bump_left(pair: &uniq Pair) -> result: own unit reads(pair.left), writes(pair.left) {
  set deref(pair).left = deref(pair).left +wrap 1_u64;
  return unit;
}

fn forward(pair: &uniq Pair) -> result: own unit reads(pair.left), writes(pair.left) {
  bump_left(pair: move pair);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(accepted, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "parameter and field state paths must check independently of lifetime reuse: {outcome:?}"
        );
    });

    let mut wrong = accepted.to_vec();
    let declaration =
        b"fn forward(pair: &uniq Pair) -> result: own unit reads(pair.left), writes(pair.left)";
    let at = wrong
        .windows(declaration.len())
        .position(|window| window == declaration)
        .expect("fixture contains the forwarding declaration");
    let replacement =
        b"fn forward(pair: &uniq Pair) -> result: own unit reads(pair.left), writes(pair.right)";
    wrong.splice(at..at + declaration.len(), replacement.iter().copied());
    assert_rule_kind(&wrong, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
}

#[test]
fn borrowed_struct_fields_keep_projection_provenance_and_exact_effects() {
    let source = br#"struct Pool {
  left: buffer<u64>;
  right: buffer<u64>;
  count: u64;
}

fn count(pool: &Pool) -> result: own u64 reads(pool.count) {
  return deref(pool).count;
}

fn first(pool: &Pool) -> result: own u64 reads(pool.left) {
  let spare = len_of(deref(pool).left);
  let ok = 0_u64 < spare;
  if ok {
    return deref(pool).left[0_u64];
  } else {
    return 0_u64;
  }
}

fn update(pool: &uniq Pool) -> result: own unit reads(pool.right), writes(pool.right, pool.count) {
  let spare = len_of(deref(pool).right);
  let ok = 0_u64 < spare;
  if ok {
    set deref(pool).right[0_u64] = 9_u64;
  }
  set deref(pool).count = 1_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

        let CheckedStatement::Match { arms, .. } = &checked.data.functions[1].body[2] else {
            panic!("borrowed buffer field read must retain its explicit guard");
        };
        let root = arms
            .iter()
            .find_map(|arm| match arm.body.first() {
                Some(CheckedStatement::Return {
                    value: CheckedExpression::BufferIndex { root, .. },
                    ..
                }) => Some(root),
                _ => None,
            })
            .expect("guarded borrowed buffer field read");
        assert_eq!(root.fields, [0]);

        let update = &checked.data.functions[2];
        let CheckedStatement::Match { arms, .. } = &update.body[2] else {
            panic!("borrowed buffer field write must retain its explicit guard");
        };
        let target = arms
            .iter()
            .find_map(|arm| match arm.body.first() {
                Some(CheckedStatement::Set {
                    target: CheckedSetTarget::BufferIndex(target),
                    ..
                }) => Some(target.as_ref()),
                _ => None,
            })
            .expect("guarded borrowed buffer field write");
        assert_eq!(target.root.fields, [1]);
        let CheckedStatement::Set {
            target: CheckedSetTarget::Place(target),
            ..
        } = &update.body[3]
        else {
            panic!("borrowed copy field write must retain its checked target");
        };
        assert_eq!(target.fields, [2]);
    });
}

/// [SET-1] states the shared-borrow referent among the cases it hands to
/// another rule — "A shared-borrow referent ... is not writable [OWN-5]" —
/// and keeps only the residue of its writability relation. The unified state
/// row rejects the impossible shared write before body checking reaches it.
#[test]
fn shared_struct_borrows_cannot_write_copy_fields() {
    assert_rule_kind(
        br#"struct Counter {
  value: u64;
}

fn invalid(counter: &Counter) -> result: own unit writes(counter) {
  set deref(counter).value = 1_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
}

#[test]
fn struct_borrow_roots_block_owner_access_and_affine_moves() {
    assert_rule(
        br#"struct Pool {
  values: buffer<u64>;
  count: u64;
}

command fn main() -> status: own ExitStatus pure {
  let values = buffer_new(1_u64, 0_u64);
  let pool = Pool(values: move values, count: 0_u64);
  region {
    let view = &pool;
    set pool.count = 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"struct Pool {
  values: buffer<u64>;
}

fn steal(pool: &Pool) -> result: own buffer<u64> pure {
  return move deref(pool).values;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn consume(counter: &uniq Counter, value: own u64) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let counter = Counter(value: 1_u64);
  region {
    consume(counter: &uniq counter, value: counter.value);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );

    with_semantics(
        br#"struct Counter {
  value: u64;
}

fn observe(counter: &Counter, value: own u64) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let counter = Counter(value: 1_u64);
  region {
    observe(counter: &counter, value: counter.value);
  }
  return exit_status(code: 0_u8);
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

fn consume(source: &buffer<u8>, sibling: own buffer<u8>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let source = buffer_new(1_u64, 0_u8);
  let sibling = buffer_new(1_u64, 0_u8);
  let owner = Owner(source: move source, sibling: move sibling);
  region {
    consume(source: &owner.source, sibling: move owner.sibling);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn child_reborrow_shape_and_sibling_exclusivity_follow_own6() {
    let positive = br#"struct Counter {
  value: u64;
}

fn write_byte(out: &uniq buffer<u8>) -> function_result: own unit reads(out), writes(out) {
  let spare = len_of(deref(out));
  let first_ok = 0_u64 < spare;
  if first_ok {
    set deref(out)[0_u64] = 7_u8;
  }
  return unit;
}

fn proxy_byte(out: &uniq buffer<u8>) -> function_result: own unit reads(out), writes(out) {
  region {
    write_byte(out: &uniq deref(out));
  }
  let spare = len_of(deref(out));
  let second_ok = 1_u64 < spare;
  if second_ok {
    set deref(out)[1_u64] = 9_u8;
  }
  return unit;
}

fn bump_counter(counter: &uniq Counter) -> function_result: own unit reads(counter.value), writes(counter.value) {
  let next = deref(counter).value +wrap 1_u64;
  set deref(counter).value = next;
  return unit;
}

fn proxy_counter(counter: &uniq Counter) -> function_result: own unit reads(counter.value), writes(counter.value) {
  region {
    bump_counter(counter: &uniq deref(counter));
  }
  set deref(counter).value = deref(counter).value +wrap 1_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let output = buffer_new(2_u64, 0_u8);
  let counter = Counter(value: 40_u64);
  region {
    proxy_byte(out: &uniq output);
    proxy_counter(counter: &uniq counter);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(positive, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("statement-scoped child reborrows must check: {outcome:?}");
        };
    });

    with_semantics(
        br#"fn observe(out: &buffer<u8>) -> result: own unit pure {
  return unit;
}

fn proxy(out: &buffer<u8>) -> result: own unit pure {
  region {
    observe(out: &deref(out));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared child of a shared holder must check: {outcome:?}");
            };
        },
    );

    assert_rule(
        br#"fn take(out: &uniq buffer<u8>) -> result: own unit pure {
  return unit;
}

fn invalid(out: &buffer<u8>) -> result: own unit pure {
  region {
    take(out: &uniq deref(out));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own6,
        SemanticIssueKind::InvalidChildReborrow {
            mechanical_fix: OWN6_HOLDER,
        },
    );

    assert_rule(
        br#"fn take(out: &uniq buffer<u8>) -> result: own unit pure {
  return unit;
}

fn invalid(out: &uniq buffer<u8>) -> result: own unit pure {
  region {
    take(out: &uniq deref(out));
    take(out: &uniq deref(out));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own6,
        SemanticIssueKind::InvalidChildReborrow {
            mechanical_fix: OWN6_STATEMENT_SCOPE,
        },
    );

    assert_rule(
        br#"fn take_two['r](first: &uniq 'r buffer<u8>, second: &uniq 'r buffer<u8>) -> result: own unit pure {
  return unit;
}

fn invalid(out: &uniq buffer<u8>) -> result: own unit pure {
  region {
    take_two(first: &uniq deref(out), second: &uniq deref(out));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );

    with_semantics(
        br#"fn observe(out: &buffer<u8>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let out = buffer_new(1_u64, 0_u8);
  loop @once {
    region {
      observe(out: &out);
    }
    break @once;
  }
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a loop-local borrow region must check: {outcome:?}");
            };
        },
    );

    assert_rule(
        br#"fn observe(out: &buffer<u8>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let out = buffer_new(1_u64, 0_u8);
  region 'r {
    loop @once {
      observe(out: &'r out);
      break @once;
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
    let source = br#"fn publish(output: &uniq OutputStream, source: &buffer<u8>, count: own u64) -> result: own unit reads(output, source), writes(output) contract {
  define capacity = len_of(deref(source));
  requires count <= capacity;
} {
  region {
    match write_once(output: &uniq deref(output), source: source, start: 0_u64, end: count) {
      Ok(value: written) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let batch = buffer_new(1_u64, 0_u8);
  region {
    publish(output: &uniq out, source: &batch, count: 1_u64);
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

    // The row is checked both ways: source observation and output transition
    // are independent of their loan lifetimes.
    let declared = b"reads(output, source), writes(output)";
    let at = source
        .windows(declared.len())
        .position(|window| window == declared)
        .expect("fixture declares the publish row");
    let mut narrowed = source.to_vec();
    narrowed.splice(at..at + declared.len(), b"reads(source)".iter().copied());
    assert_rule_kind(&narrowed, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
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

fn read_scalar(p: &i32) -> result: own i32 reads(p) {
  return deref(p);
}

fn bump(p: &uniq i32) -> result: own unit writes(p) {
  set deref(p) = 9_i32;
  return unit;
}

fn score(c: &Cell) -> result: own i32 reads(c) {
  match deref(c) {
    Full(v: x) => {
      return deref(x);
    }
    Void() => {
      return 0_i32;
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  let a = 5_i32;
  region {
    let s = &a;
  }
  region {
    let u = &uniq a;
    set deref(u) = 7_i32;
  }
  return exit_status(code: 0_u8);
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
    assert_rule_kind(
        b"fn dangle['r0](x: own i32) -> result: &'r0 i32 pure {\n  return &'r0 x;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    // [OWN-4]: a borrow taken in an inner region cannot be returned as the
    // caller's region. The witness must be in return position. Writing it as
    // `let q = &'s deref(x); return q;` rejects OWN-14 instead, because
    // [OWN-6] admits a reborrow only as a call-argument atom — a plausible
    // simplification that would silently retarget this case.
    assert_rule_kind(
        b"fn leak['r0](x: &'r0 i32) -> result: &'r0 i32 pure {\n  region {\n    return &deref(x);\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own4,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    // [TYPE-7]: no implicit read through a scalar holder.
    assert_rule(
        b"fn read(holder: &i32) -> result: own i32 pure {\n  return holder;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: a bare holder is not an enum value, so it cannot be matched.
    assert_rule(
        b"enum State {\n  Ready();\n  Done();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let state = Ready();\n  region {\n    let holder = &state;\n    match holder {\n      Ready() => {\n      }\n      Done() => {\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: neither is a `borrow_expr`.
    assert_rule(
        b"enum State {\n  Ready();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let state = Ready();\n  region {\n    match &state {\n      Ready() => {\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [TYPE-7]: nor a reference-returning call's result.
    assert_rule(
        b"enum State {\n  Ready();\n}\n\nfn view['r](state: &'r State) -> result: &'r State pure {\n  return state;\n}\n\nfn inspect(state: &State) -> result: own unit pure {\n  match view(state: state) {\n    Ready() => {\n    }\n  }\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    // [OWN-5]: a shared holder never makes its referent writable.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let a = 1_i32;\n  region {\n    let s = &a;\n    set deref(s) = 9_i32;\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // [OWN-5]: two live uniq borrows of one scalar place overlap.
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let a = 3_i32;\n  region {\n    let u1 = &uniq a;\n    let u2 = &uniq a;\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // [OWN-12]: two uniq arguments over one place alias at the call.
    assert_rule(
        b"fn two['r](a: &uniq 'r i32, b: &uniq 'r i32) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let x = 0_i32;\n  region {\n    two(a: &uniq x, b: &uniq x);\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

/// [OWN-3, OWN-4] an enclosing region outlives an inner one, so a borrow of
/// an outer region is legally held by a binding declared any number of
/// blocks deeper: the borrow value stays live for the holder's whole scope.
/// The judgment is the outlives relation over region blocks, not a fixed
/// holder-directly-inside-its-region shape.
#[test]
fn outer_region_borrows_may_be_held_under_inner_regions() {
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  let a = 7_i32;\n  region 'r {\n    region {\n      region {\n        let q = &'r a;\n        let observed = deref(q);\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("an outer-region borrow held two blocks deeper must check: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  let a = 7_i32;\n  region 'r {\n    region {\n      let u = &uniq 'r a;\n      set deref(u) = 8_i32;\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("an outer-region uniq borrow held one block deeper must check: {outcome:?}");
            };
        },
    );
}

/// [EFF-2] §9.1 attributes reads and writes through an incoming borrow
/// parameter to that parameter's formal region, both ways.
#[test]
fn scalar_borrow_parameter_effect_rows_are_exact_in_both_directions() {
    assert_rule_kind(
        b"fn read_scalar(p: &i32) -> result: own i32 pure {\n  return deref(p);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
    assert_rule_kind(
        b"fn bump(p: &uniq i32) -> result: own unit reads(p) {\n  set deref(p) = 9_i32;\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
    assert_rule_kind(
        b"fn quiet(p: &i32) -> result: own unit reads(p) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

/// [OWN-14] admits the written reborrow as the complete return expression
/// from a parameter or let-bound holder with preserved mode; the created
/// borrow then carries the existing [OWN-10]/[OWN-4] region judgments, so a
/// callee-local region cannot leave through the signature.
#[test]
fn returned_reborrows_follow_own14_admission_and_own4_regions() {
    with_semantics(
        b"fn passthru['r0](x: &'r0 i32) -> result: &'r0 i32 pure {\n  return &'r0 deref(x);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared returned reborrow of a parameter must check: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"fn passthru['r0](x: &uniq 'r0 i32) -> result: &uniq 'r0 i32 pure {\n  return &uniq 'r0 deref(x);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a unique returned reborrow of a parameter must check: {outcome:?}");
            };
        },
    );
    // [OWN-4]: the returned borrow's local region cannot reach the written
    // rtype region, in either mode.
    assert_rule_kind(
        b"fn leak['r0](x: &'r0 i32) -> result: &'r0 i32 pure {\n  region {\n    return &deref(x);\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own4,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    assert_rule_kind(
        b"fn leak['r0](x: &uniq 'r0 i32) -> result: &uniq 'r0 i32 pure {\n  region {\n    return &uniq deref(x);\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own4,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
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
        b"fn bind(x: &i32) -> result: own unit pure {\n  region {\n    let y = &deref(x);\n  }\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own14,
        SemanticIssueKind::InvalidReborrowPosition {
            mechanical_fix: RESTRUCTURING,
        },
    );
    // A `shared` result may derive from a `uniq` parameter, so this widening
    // signature has no same-kind candidate and one other parameter naming the
    // result region. v0.32 refuses that boundary at its own `rtype` [FN-1]
    // before OWN-14 judges the return position: the form stays a hard error,
    // and the rule that owns it moves to the declaration.
    assert_rule(
        b"fn down['r0](x: &uniq 'r0 i32) -> result: &'r0 i32 pure {\n  return &'r0 deref(x);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn pick['r](holder: &'r Packet) -> result: &'r i32 reads(holder) {\n  match deref(holder) {\n    Data(value: payload) => {\n      return &'r deref(payload);\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own14,
        SemanticIssueKind::InvalidReborrowPosition {
            mechanical_fix: RESTRUCTURING,
        },
    );
}

/// The companion to the OWN-14 rejections above: a `box` binding is own mode,
/// so a borrow of its content is not a reborrow form at all and never reaches
/// OWN-14's disposition. It is judged by [OWN-10]'s own-mode-binding case —
/// the borrow region must be introduced within the binding's scope and never
/// caller-supplied — and then stops explicitly, because the box binding lowers
/// to the content pointer under the box's own IR type and nothing addresses
/// the content itself. Before the dispatch fix these programs reported TYPE-7
/// "deref requires a borrow holder" against source that wrote no holder.
#[test]
fn box_content_borrows_are_ordinary_borrows_rather_than_reborrows() {
    assert_unsupported(
        br#"fn bump(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 42_i32;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let b = box_new(4_i32);
  region {
    bump(n: &uniq deref(b));
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
    assert_rule_kind(
        br#"fn hold(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 1_i32;
  return unit;
}

fn outer['s](anchor: &'s i32) -> result: &'s i32 pure {
  let b = box_new(4_i32);
  hold(n: &uniq 's deref(b));
  return anchor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
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
        b"enum Packet {\n  Data(value: i32);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let packet = Data(value: 4_i32);\n  region {\n    let holder = &uniq packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        let saved = deref(payload);\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a uniq-match payload read through its binder must check: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"enum Packet {\n  Data(value: i32);\n  Idle();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let packet = Data(value: 4_i32);\n  region {\n    let holder = &packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        let saved = deref(payload);\n      }\n      Idle() => {\n      }\n    }\n    match deref(holder) {\n      Data(value: payload) => {\n        let again = deref(payload);\n      }\n      Idle() => {\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared root is never suspended and matches again: {outcome:?}");
            };
        },
    );
    with_semantics(
        b"enum Inner {\n  Leaf(value: i32);\n}\n\nenum Outer {\n  Wrap(inner: Inner);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let leaf = Leaf(value: 7_i32);\n  let packet = Wrap(inner: move leaf);\n  region {\n    let holder = &packet;\n    match deref(holder) {\n      Wrap(inner: nested) => {\n        match deref(nested) {\n          Leaf(value: payload) => {\n            let saved = deref(payload);\n          }\n        }\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
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
        b"enum Packet {\n  Data(value: i32);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let packet = Data(value: 4_i32);\n  region {\n    let holder = &uniq packet;\n    match deref(holder) {\n      Data(value: payload) => {\n        match deref(holder) {\n          Data(value: other) => {\n          }\n        }\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // Post-match reuse, joined across a binder-creating and a binder-free arm.
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n  Idle();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let packet = Data(value: 4_i32);\n  region {\n    let holder = &uniq packet;\n    match deref(holder) {\n      Data(value: payload) => {\n      }\n      Idle() => {\n      }\n    }\n    match deref(holder) {\n      Data(value: payload) => {\n      }\n      Idle() => {\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    // A returned reborrow is not created through a suspended holder.
    assert_rule(
        b"enum Packet {\n  Data(value: i32);\n}\n\nfn peek['r](holder: &uniq 'r Packet) -> result: &uniq 'r Packet reads(holder) {\n  match deref(holder) {\n    Data(value: payload) => {\n    }\n  }\n  return &uniq 'r deref(holder);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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
        b"fn read(holder: own box<i32>) -> result: own i32 pure {\n  return holder;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        type7.clone(),
    );
    assert_rule(
        b"fn read(holder: own box<i32>) -> result: own i32 pure {\n  return move holder;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        type7.clone(),
    );
    assert_rule(
        b"fn grab(p: &uniq i32) -> result: own i32 pure {\n  return p;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type7,
        type7,
    );
    // The referent is not required here, so TYPE-7 is not established and
    // OWN-1's bare-affine spelling is the sole rejection.
    assert_rule(
        b"fn pass(holder: own box<i32>) -> result: own box<i32> pure {\n  return holder;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own1,
        SemanticIssueKind::BareAffineUse {
            mechanical_fix: "write `move p` for the affine place",
        },
    );
}

// ---------------------------------------------------------------------------
// The reborrow extension: bound call-result borrow holders with unambiguous
// signature provenance, the non-statement-scoped candidate-position child
// reborrow, and the grandchild chains they compose. The shipped switch is on,
// so the extension entry these tests name selects the same judgment as the
// default one; the default-checker tests below pin that the shipped path
// admits the shapes v0.30 rejected.
// ---------------------------------------------------------------------------

const PASSTHRU: &[u8] = b"fn passthru['r0](x: &uniq 'r0 i32) -> result: &uniq 'r0 i32 pure {\n  return &uniq 'r0 deref(x);\n}\n\n";

/// Extension: a borrow-returning call with one same-kind same-region borrow
/// parameter has unambiguous provenance, so its bound result is an ordinary
/// holder over the candidate actual's storage: deref reads and set commits
/// through it check, and a statement-scoped grandchild of the bound result
/// rides the existing OWN-6 child rule.
#[test]
fn extension_binds_call_result_borrows_and_composes_grandchild_chains() {
    // Bind from a candidate-position child reborrow, then write through it.
    let mut chain = PASSTHRU.to_vec();
    chain.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &uniq v;\n    let r = passthru(x: &uniq deref(h));\n    set deref(r) = 9_i32;\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics_extension(&chain, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("a bound call-result borrow with one candidate must check: {outcome:?}");
        };
    });
    // A statement-scoped grandchild of the bound result feeds an
    // own-returning callee under the unchanged v0.7 child rule.
    let mut grandchild = PASSTHRU.to_vec();
    grandchild.extend_from_slice(
        b"fn bump(n: &uniq i32) -> result: own unit writes(n) {\n  set deref(n) = 42_i32;\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &uniq v;\n    let r = passthru(x: &uniq deref(h));\n    region {\n      bump(n: &uniq deref(r));\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics_extension(&grandchild, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("a grandchild chain through a bound result must check: {outcome:?}");
        };
    });
    // A shared bare-holder actual sources a shared result the same way.
    with_semantics_extension(
        b"fn source['r](x: &'r i32) -> result: &'r i32 pure {\n  return x;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &v;\n    let r = source(x: h);\n    let w = deref(r);\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a shared bare-holder-sourced result must check: {outcome:?}");
            };
        },
    );
    // The recursive composition: a candidate child in the caller-supplied
    // parameter region, its bound result, and a returned reborrow of that
    // result — the chain a recursive traversal threads through its frames.
    let mut recursive = PASSTHRU.to_vec();
    recursive.extend_from_slice(
        b"fn twice['q0](x: &uniq 'q0 i32) -> result: &uniq 'q0 i32 pure {\n  let r = passthru(x: &uniq 'q0 deref(x));\n  return &uniq 'q0 deref(r);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics_extension(&recursive, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("a caller-supplied-region chain must check: {outcome:?}");
        };
    });
}

/// Extension: creating the chain suspends the candidate parent holder for
/// the remainder of its life; the borrow may outlive the statement inside
/// the bound result — so a later use through the parent, or a second chain
/// from it, is the OWN-5 suspension rejection.
#[test]
fn extension_chains_suspend_the_candidate_parent_permanently() {
    let mut later_use = PASSTHRU.to_vec();
    later_use.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &uniq v;\n    let r = passthru(x: &uniq deref(h));\n    let w = deref(h);\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule_extension(
        &later_use,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    let mut second_chain = PASSTHRU.to_vec();
    second_chain.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &uniq v;\n    let r = passthru(x: &uniq deref(h));\n    let s = passthru(x: &uniq deref(h));\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule_extension(
        &second_chain,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

/// A callee signature that does not determine one provenance candidate — two
/// same-kind same-region borrow parameters — is rejected at its own boundary
/// citing FN-1, so the reborrow-extension entry never reaches a call whose
/// result it would have to infer an ownership fact for.
#[test]
fn extension_rejects_ambiguous_result_provenance() {
    assert_rule_extension(
        b"fn pick['r](a: &uniq 'r i32, b: &uniq 'r i32) -> result: &uniq 'r i32 pure {\n  return &uniq 'r deref(a);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let x = 1_i32;\n  let y = 2_i32;\n  region {\n    let r = pick(a: &uniq x, b: &uniq y);\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
}

/// Extension: only the candidate position of a borrow-returning call admits
/// a written child reborrow; a child in a non-candidate position keeps
/// OWN-6's own/unit-result rejection.
#[test]
fn extension_keeps_non_candidate_children_rejected() {
    assert_rule_extension(
        b"fn mix['q2](p: &uniq i32, q: &'q2 i32) -> result: &'q2 i32 pure {\n  return &'q2 deref(q);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let x = 1_i32;\n  let y = 2_i32;\n  region 'a {\n    let hx = &uniq x;\n    region {\n      let r = mix(p: &uniq 'a deref(hx), q: &y);\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own6,
        SemanticIssueKind::InvalidChildReborrow {
            mechanical_fix: OWN6_ARGUMENT_POSITION,
        },
    );
}

/// The shipped switch is on, so the default checker and the extension entry
/// are one judgment: the candidate child argument and the bare-holder-sourced
/// binding that v0.30 rejected at OWN-6 and TYPE-5 are admitted through the
/// ordinary `check_semantics` path, not only the test-only entry.
#[test]
fn the_shipped_checker_admits_the_extension_shapes() {
    let mut chain = PASSTHRU.to_vec();
    chain.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &uniq v;\n    let r = passthru(x: &uniq deref(h));\n    set deref(r) = 9_i32;\n  }\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics(&chain, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the candidate child argument is admitted on the shipped path: {outcome:?}",
        );
    });
    with_semantics(
        b"fn source['r](x: &'r i32) -> result: &'r i32 pure {\n  return x;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let v = 5_i32;\n  region {\n    let h = &v;\n    let r = source(x: h);\n    let w = deref(r);\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the bound call-result holder is an ordinary borrow holder: {outcome:?}",
            );
        },
    );
}

/// [ENT-5] a write through a bound call-result holder kills exactly the
/// facts on the candidate actual's storage: the stale bound no longer
/// discharges a later subscript, while the identical program without the
/// write keeps the discharge (the deliberate negative control).
#[test]
fn extension_writes_through_result_holders_kill_source_facts() {
    const HELPER: &[u8] = b"fn passthru['r0](x: &uniq 'r0 u64) -> result: &uniq 'r0 u64 pure {\n  return &uniq 'r0 deref(x);\n}\n\n";
    let mut killed = HELPER.to_vec();
    killed.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let i = 1_u64;\n  let b = buffer_new(4_u64, 0_u64);\n  region {\n    let r = passthru(x: &uniq i);\n    set deref(r) = 9_u64;\n  }\n  let e = b[i];\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics_extension(&killed, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the killed bound must not discharge the subscript: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedBoundsObligation { .. }
        ));
    });
    let mut control = HELPER.to_vec();
    control.extend_from_slice(
        b"command fn main() -> status: own ExitStatus pure {\n  let i = 1_u64;\n  let b = buffer_new(4_u64, 0_u64);\n  region {\n    let r = passthru(x: &uniq i);\n  }\n  let e = b[i];\n  return exit_status(code: 0_u8);\n}\n",
    );
    with_semantics_extension(&control, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("without the write the fact must survive and discharge: {outcome:?}");
        };
    });
}

// ---------------------------------------------------------------------------
// Declaration-site borrow-result provenance [FN-1]: a callable boundary whose
// borrow-mode result has no signature-determined source is rejected at its own
// `rtype`, whether or not it is ever called. The judgment is a boundary rule
// and does not wait for a caller, so OWN-6's binding-side ambiguity rejection
// has no reachable source and no longer exists.
// ---------------------------------------------------------------------------

/// The three exact [OWN-6] restructurings, one per condition the rule carries.
///
/// Spelled out here rather than imported, like the [FN-1] fix above, so a
/// change to the text a writer reads has to be made twice on purpose.
const OWN6_STATEMENT_SCOPE: &str = "a child reborrow's region admits exactly one statement, and a value that statement binds dies at the region's end, so `region 'r { let permit = reserve_handle::<'r>(factory: &uniq 'r holder); match open_...(permit: move permit, ...) { ... } }` is two statements and cannot be repaired by shortening the region. The whole idiom is three parts: move the reserve and the open into one helper that takes the holder as `&uniq 'f` and returns the opened value (`fn open_source_from_factory['f, 'd](factory: &uniq 'f HandleFactory, directory: &'d DirectoryRead) -> result: own Result<DirectorySource, IoError>`); make the single statement of the region the `match` on that helper's call; and write every statement that uses the opened value inside that `match` arm, because the opened value dies with the region (P4 linear threading, P15 recursive walker). The other route, `let stale = replace target = call(...);`, applies only where the call leaves the target's root alive: a call that consumes the target root — one taking `move permit` — rejects OWN-1 instead.";

const OWN6_ARGUMENT_POSITION: &str = "a reborrow is an argument only to a call returning an owned \
     value or unit, or in the one argument position a borrow-returning call takes its result \
     from; pass the holder itself, or bind the result from that position";

const OWN6_HOLDER: &str = "reborrow only a parameter or let-bound holder, take `&uniq` only from \
     a `&uniq` holder, and introduce the child region inside the holder's own region";

/// The exact [FN-1] restructuring the boundary judgment names.
const AMBIGUOUS_PROVENANCE_FIX: &str = "give the source parameter its own region so exactly one parameter shares the result's \
     region and kind, or return the decision as a value and let the caller borrow from the \
     source it names";

/// Two same-kind parameters in the result's region, never called: the
/// declaration itself is the error, because GRAM-9 binds every call result
/// and no caller could bind this one.
#[test]
fn declaration_provenance_rejects_two_same_region_sources_at_the_declaration() {
    assert_rule(
        b"fn pick['r](a: &uniq 'r i32, b: &uniq 'r i32) -> result: &uniq 'r i32 pure {\n  return &uniq 'r deref(a);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
}

/// The same boundary judgment covers every other shape that leaves two
/// possible roots: a same-region parameter of the other borrow kind (a
/// `shared` result may derive from a `uniq` source through a nested
/// borrow-returning call), and a parameter whose written type names the
/// result's region.
#[test]
fn declaration_provenance_rejects_every_undetermined_source_shape() {
    assert_rule(
        b"fn either['r](a: &uniq 'r i32, b: &'r i32) -> result: &'r i32 pure {\n  return &'r deref(b);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
    assert_rule(
        b"fn viewed['r](a: &'r i32, s: own Slice<'r, i32>) -> result: &'r i32 pure {\n  return &'r deref(a);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
}

/// The named mechanical fix works: giving the second parameter its own
/// region leaves exactly one candidate, and the boundary is then accepted
/// with its result fully usable — bound, and written through.
#[test]
fn declaration_provenance_admits_distinct_region_sources_and_keeps_them_usable() {
    with_semantics(
        b"fn pick['r](a: &uniq 'r i32, b: &uniq i32) -> result: &uniq 'r i32 pure {\n  return &uniq 'r deref(a);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let x = 1_i32;\n  let y = 2_i32;\n  region 'a {\n    region {\n      let r = pick(a: &uniq 'a x, b: &uniq y);\n      set deref(r) = 9_i32;\n      let w = deref(r);\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("one candidate per region must check and stay usable: {outcome:?}");
            };
        },
    );
}

/// A borrow-mode result with no candidate parameter at all keeps its
/// boundary: permanently read-only named-const storage is the only source
/// left [CONST-2, OWN-10], so provenance is unique by elimination and FN-1
/// forms no rejection here. The body then meets the checker's missing
/// const-rooted borrow as an explicit capability stop — never an
/// invalid-source verdict, and never the ambiguity rejection.
#[test]
fn declaration_provenance_admits_the_zero_candidate_boundary() {
    with_semantics(
        b"const anchor: i32 = 7_i32;\n\nfn sourced['r](n: own i32) -> result: &'r i32 pure {\n  return &'r anchor;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Unsupported { unsupported } = outcome else {
                panic!("a zero-candidate boundary is legal, not rejected: {outcome:?}");
            };
            assert_eq!(
                unsupported.feature(),
                UnsupportedSemanticFeature::RegionsAndBorrows,
            );
        },
    );
}

/// [FN-1]'s v0.32 conjunct over a result whose written type is not
/// region-free reaches no source through this order. Every written
/// region-bearing type is a `slice` or an `arena` [STOR-5]: the slice shape
/// is FN-1's own borrowed-descriptor rejection above, an arena result is
/// STOR-4's escape — the rule this specification defines first, which the
/// same-node ordering selects — and a borrow-mode arena parameter never
/// reaches a result judgment at all, because borrowing an arena is an
/// explicit capability stop rather than a source rejection. This test pins
/// that order, so restating the arena result as an FN-1 provenance
/// rejection cannot happen silently.
#[test]
fn a_region_bearing_borrow_result_is_owned_by_the_rules_stated_before_it() {
    assert_rule(
        b"fn held['b, 'r](n: own i32) -> result: &'b arena<'r, i32> pure {\n  return n;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Stor4,
        SemanticIssueKind::ArenaEscape {
            mechanical_fix: "keep the arena value inside its region's block; return or deliver its content, or a borrow OWN-10 admits, instead",
        },
    );
    assert_unsupported(
        b"fn held(a: &arena<i32>) -> result: own i32 pure {\n  return 1_i32;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
}

/// The boundary judgment is added to the existing signature-formation order,
/// not ahead of it: a borrowed-slice result still cites FN-1's own
/// slice-descriptor rejection, and a zero-candidate boundary whose body
/// roots the result in callee-local storage still cites OWN-10 at the body.
#[test]
fn declaration_provenance_keeps_the_established_boundary_judgment_order() {
    assert_rule(
        b"fn borrowed_slice['descriptor, 'data](value: &'descriptor Slice<'data, u8>) -> result: &'descriptor Slice<'data, u8> pure {\n  return value;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::BorrowedSliceResult {
            mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
        },
    );
    assert_rule_kind(
        b"fn dangle['r0](x: own i32) -> result: &'r0 i32 pure {\n  return &'r0 x;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
}

/// With the boundary judgment live, the call-site ambiguity state is
/// unreachable: the identical program that v0.31 accepts at the declaration
/// and rejects at the binding is now rejected at the declaration, and the
/// OWN-6 binding diagnostic never runs. Bindable iff usable.
#[test]
fn declaration_provenance_makes_the_binding_side_ambiguity_unreachable() {
    const AMBIGUOUS_CALL: &[u8] = b"fn pick['r](a: &uniq 'r i32, b: &uniq 'r i32) -> result: &uniq 'r i32 pure {\n  return &uniq 'r deref(a);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let x = 1_i32;\n  let y = 2_i32;\n  region {\n    let r = pick(a: &uniq x, b: &uniq y);\n  }\n  return exit_status(code: 0_u8);\n}\n";
    assert_rule(
        AMBIGUOUS_CALL,
        SemanticRule::Fn1,
        SemanticIssueKind::AmbiguousResultProvenance {
            mechanical_fix: AMBIGUOUS_PROVENANCE_FIX,
        },
    );
}
