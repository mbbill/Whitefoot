//! [SET-2] affine-place replacement: target class, commit semantics, root
//! liveness, and the ENT-5 kill interaction that keeps a stale length fact
//! from discharging a post-replace subscript.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedSetTarget, CheckedStatement, CheckedType};
use super::{assert_rule, with_semantics};

const HOLDER: &[u8] = br#"struct Holder {
  payload: FixedVector<u8, 4>;
  count: u64;
}

"#;

fn with_holder(rest: &[u8]) -> Vec<u8> {
    let mut source = HOLDER.to_vec();
    source.extend_from_slice(rest);
    source
}

#[test]
fn replace_of_an_affine_field_accepts_and_retains_the_commit() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..4_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 4_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 7_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let second = fixed_vector::<u8, 4>();
  for @fill_second (
    at in 0_u64..2_u64,
    invariant grown: len_of(second) >= at,
    invariant spare: room_of(second) + at >= 2_u64,
    invariant flat: head_of(second) <= 0_u64
  ) {
    set second = place_back(vector: move second, value: 9_u8);
  }
  let old = replace holder.payload = move second;
  let size = len_of(old);
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an affine field replace must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Replace {
            binding, target, ..
        } = &main.body[5]
        else {
            panic!("the sixth statement must be the SET-2 commit");
        };
        let CheckedSetTarget::Place(place) = target else {
            panic!("a field replace target is a Place");
        };
        assert_eq!(place.fields, vec![0]);
        // The fresh binding is a real binding: the later `len_of(old)` read
        // resolves against it, which the accepted outcome already proves.
        assert!(binding.0 > 0);
    });
}

#[test]
fn replace_of_a_copy_place_rejects_citing_set2() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..1_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 1_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 0_u8);
  }
  let holder = Holder(payload: move first, count: 3_u64);
  let old = replace holder.count = 4_u64;
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_rule(
        &source,
        SemanticRule::Set2,
        SemanticIssueKind::InvalidReplaceTarget {
            target_type: "u64".to_owned(),
            mechanical_fix: "use set for a copy place; read the previous value bare",
        },
    );
}

#[test]
fn set_of_an_affine_place_still_rejects_and_names_replace() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..1_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 1_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 0_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let second = fixed_vector::<u8, 4>();
  for @fill_second (
    at in 0_u64..1_u64,
    invariant grown: len_of(second) >= at,
    invariant spare: room_of(second) + at >= 1_u64,
    invariant flat: head_of(second) <= 0_u64
  ) {
    set second = place_back(vector: move second, value: 0_u8);
  }
  set holder.payload = move second;
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_rule(
        &source,
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "FixedVector<u8, 4>".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
}

#[test]
fn replace_kills_the_stale_length_fact_at_the_commit() {
    // Without the ENT-5 SET-2 kill, the pre-replace `size = 4` fact would
    // discharge the post-replace subscript over the two-element run and the
    // accepted program would write out of bounds. Both runs have the same
    // capacity now, because a `FixedVector`'s capacity is its type; what the
    // commit changes is `len_of`, which is the measure the stale fact names.
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..4_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 4_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 7_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let size = len_of(holder.payload);
  let allocated_length = size == 4_u64;
  if allocated_length {
    let second = fixed_vector::<u8, 4>();
    for @fill_second (
      at in 0_u64..2_u64,
      invariant grown: len_of(second) >= at,
      invariant spare: room_of(second) + at >= 2_u64,
      invariant flat: head_of(second) <= 0_u64
    ) {
      set second = place_back(vector: move second, value: 9_u8);
    }
    let old = replace holder.payload = move second;
    set holder.payload[3_u64] = 5_u8;
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the stale-length subscript must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });
}

#[test]
fn the_same_subscript_discharges_without_the_replace() {
    // The control for the kill test: identical program minus the commit.
    // Rejection above plus acceptance here attributes the kill to SET-2.
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..4_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 4_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 7_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let size = len_of(holder.payload);
  let allocated_length = size == 4_u64;
  if allocated_length {
    set holder.payload[3_u64] = 5_u8;
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the control must accept: {outcome:?}"
        );
    });
}

#[test]
fn replace_leaves_the_target_root_live() {
    // The commit is not a consuming use [SET-2, OWN-1]: the root is read,
    // written, and finally moved after the replace.
    let source = with_holder(
        br#"fn consume(h: own Holder) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..2_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 2_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 1_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let second = fixed_vector::<u8, 4>();
  for @fill_second (
    at in 0_u64..3_u64,
    invariant grown: len_of(second) >= at,
    invariant spare: room_of(second) + at >= 3_u64,
    invariant flat: head_of(second) <= 0_u64
  ) {
    set second = place_back(vector: move second, value: 2_u8);
  }
  let old = replace holder.payload = move second;
  set holder.count = 1_u64;
  let observed = holder.count;
  let done = consume(h: move holder);
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the root must stay live after a replace: {outcome:?}"
        );
    });
}

#[test]
fn replace_of_a_dead_root_rejects_citing_own1() {
    let source = with_holder(
        br#"fn sink(h: own Holder) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..2_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 2_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 1_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let gone = sink(h: move holder);
  let second = fixed_vector::<u8, 4>();
  for @fill_second (
    at in 0_u64..1_u64,
    invariant grown: len_of(second) >= at,
    invariant spare: room_of(second) + at >= 1_u64,
    invariant flat: head_of(second) <= 0_u64
  ) {
    set second = place_back(vector: move second, value: 0_u8);
  }
  let old = replace holder.payload = move second;
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a dead-root replace must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Own1);
    });
}

#[test]
fn replace_through_a_shared_borrow_rejects() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..2_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 2_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 1_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  region {
    let view = &holder;
    let second = fixed_vector::<u8, 4>();
    for @fill_second (
      at in 0_u64..1_u64,
      invariant grown: len_of(second) >= at,
      invariant spare: room_of(second) + at >= 1_u64,
      invariant flat: head_of(second) <= 0_u64
    ) {
      set second = place_back(vector: move second, value: 0_u8);
    }
    let old = replace deref(view).payload = move second;
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a shared-holder replace must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Own5);
    });
}

#[test]
fn element_position_replace_accepts_an_affine_element_and_keeps_its_bounds_obligations() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = fixed_vector::<Option<u32>, 4>();
  for @fill (
    at in 0_u64..4_u64,
    invariant grown: len_of(slots) >= at,
    invariant spare: room_of(slots) + at >= 4_u64,
    invariant flat: head_of(slots) <= 0_u64
  ) {
    let empty = None<u32>();
    set slots = place_back(vector: move slots, value: move empty);
  }
  let filled = Some<u32>(value: 7_u32);
  let vacant = replace slots[2_u64] = move filled;
  let none = None<u32>();
  let taken = replace slots[2_u64] = move none;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an affine element replace must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Replace { target, .. } = &main.body[3] else {
            panic!("the fourth statement must be the SET-2 element commit");
        };
        let CheckedSetTarget::RunIndex(target) = target else {
            panic!("an element replace target retains its run root");
        };
        let CheckedType::Nominal(element) = target.element_type else {
            panic!("the element is the affine Option instance");
        };
        assert_eq!(
            checked.data.nominals[element.0 as usize].name,
            "Option<u32>"
        );
        assert!(!target.obligation.components().is_empty());
        assert!(matches!(
            &main.body[5],
            CheckedStatement::Replace {
                target: CheckedSetTarget::RunIndex(_),
                ..
            }
        ));
    });
}

/// B7c4b left this case on the retiring surface, and the reason is a finding
/// rather than an omission: on the container surface neither route to
/// "content reached through a borrow" exists for an affine element. A struct
/// holding a run lent `&uniq` is [BLK-4]'s refusal
/// (`UniqueParameterReachesContainer`), and the exclusive view that replaces
/// it — `&uniq MutSlice<Option<u32>>` — stops as an unsupported composite
/// value, because a view's element domain is flat and `Option<u32>` reaches
/// it only as a nominal a view may not carry. Until one of those two lands,
/// [SET-2]'s sole admitted move of borrowed content has no writable program
/// on the new surface.
#[test]
fn element_position_replace_through_a_unique_holder_accepts() {
    // The DESIGN walkthrough shape: the commit through a live usable `&uniq`
    // holder is [SET-2]'s sole admitted move of content reached through a
    // borrow, and the OP-4 obligation discharges against the held buffer's
    // length fact.
    let source = br#"struct OptVec {
  buf: buffer<Option<u32>>;
  fill: u64;
}

fn push(v: &uniq OptVec, x: own u32) -> result: own unit reads(v.buf, v.fill), writes(v.buf) {
  let count = deref(v).fill;
  let limit = len_of(deref(v).buf);
  let has_room = count < limit;
  if has_room {
    let filled = Some<u32>(value: x);
    let vacant = replace deref(v).buf[count] = move filled;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let empty = buffer_vacant::<u32>(2_u64);
  let v = OptVec(buf: move empty, fill: 0_u64);
  region {
    push(v: &uniq v, x: 5_u32);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("a held element replace must check: {outcome:?}");
        };
    });
}

#[test]
fn element_position_replace_keeps_the_bounds_obligation() {
    let source = br#"fn hollow<const n: u64>() -> result: own unit pure {
  let slots = fixed_vector::<Option<u32>, n>();
  let none = None<u32>();
  let taken = replace slots[0_u64] = move none;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  hollow::<2>();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an undischarged element replace must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });
}

#[test]
fn element_replacement_rhs_must_be_the_exact_element_type() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = fixed_vector::<Option<u32>, 4>();
  for @fill (
    at in 0_u64..4_u64,
    invariant grown: len_of(slots) >= at,
    invariant spare: room_of(slots) + at >= 4_u64,
    invariant flat: head_of(slots) <= 0_u64
  ) {
    let empty = None<u32>();
    set slots = place_back(vector: move slots, value: move empty);
  }
  let taken = replace slots[0_u64] = 3_u32;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a payload-typed replacement must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Type5);
    });
}

#[test]
fn affine_elements_leave_their_slots_only_through_replace() {
    // SET-1 on an affine element names replace [STOR-1].
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let slots = fixed_vector::<Option<u32>, 4>();
  let none = None<u32>();
  set slots[0_u64] = move none;
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "Option<u32>".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    // A bare element read would mint a second owner [OWN-1].
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let slots = fixed_vector::<Option<u32>, 4>();
  let observed = slots[0_u64];
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::BareAffineUse {
            mechanical_fix: "exchange the element with `let old = replace p = e;`",
        },
    );
    // `move` out of a slot is not an admitted element exit [TYPE-2].
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let slots = fixed_vector::<Option<u32>, 4>();
  let observed = move slots[0_u64];
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type2,
        SemanticIssueKind::AffineElementMove {
            mechanical_fix: "exchange the element with `let old = replace p = e;`",
        },
    );
}

#[test]
fn element_position_replace_rejects_while_every_element_is_copy() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 2>();
  let old = replace first[0_u64] = 3_u8;
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_rule(
        &source,
        SemanticRule::Set2,
        SemanticIssueKind::InvalidReplaceTarget {
            target_type: "u8".to_owned(),
            mechanical_fix: "use set for a copy place; read the previous value bare",
        },
    );
}

#[test]
fn replace_rhs_type_mismatch_rejects_citing_type5() {
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = fixed_vector::<u8, 4>();
  for @fill_first (
    at in 0_u64..1_u64,
    invariant grown: len_of(first) >= at,
    invariant spare: room_of(first) + at >= 1_u64,
    invariant flat: head_of(first) <= 0_u64
  ) {
    set first = place_back(vector: move first, value: 0_u8);
  }
  let holder = Holder(payload: move first, count: 0_u64);
  let second = fixed_vector::<u16, 4>();
  let old = replace holder.payload = move second;
  return exit_status(code: 0_u8);
}
"#,
    );
    with_semantics(&source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a wrong-typed replacement must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Type5);
    });
}

/// [SET-2] rejects a region-bearing target type under [STOR-5]'s relation,
/// and that relation names `Slice<'r, T>` and `arena<'r, T>` alike. The two
/// programs differ only in which region-bearing constructor the target has.
///
/// The second stays on `arena<'r, T>`: it is the arena constructor's own half
/// of the relation and retires with the spelling. A store cell is not its
/// twin — `Box<'s, T>` is an ordinary affine target, which is what the
/// positive control below shows.
#[test]
fn replace_of_a_region_bearing_place_rejects_citing_set2() {
    let expected_fix = "a slice's static origin set and an arena's confinement are fixed at \
                        initialization; bind a new slice or arena under a new let";
    assert_rule(
        br#"const left: FixedVector<u8, 2> =[11_u8, 11_u8];

const right: FixedVector<u8, 2> =[29_u8, 29_u8];

command fn main() -> status: own ExitStatus pure {
  region {
    let view = slice_of(&left);
    let previous = replace view = slice_of(&right);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Set2,
        SemanticIssueKind::InvalidReplaceTarget {
            target_type: "Slice<u8>".to_owned(),
            mechanical_fix: expected_fix,
        },
    );
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'r {
    let first = arena_new::<'r, u64>(1_u64);
    let second = arena_new::<'r, u64>(2_u64);
    let previous = replace first = move second;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Set2,
        SemanticIssueKind::InvalidReplaceTarget {
            target_type: "arena<'r, u64>".to_owned(),
            mechanical_fix: expected_fix,
        },
    );
}

/// The positive control for the judgment above: an owning descriptor whose
/// type bears no region is an ordinary affine target, so the exchange is
/// accepted and the fresh binding owns the previous box.
#[test]
fn replace_of_a_cell_descriptor_accepts() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 1_u64) {
      Ok(value: first) => {
        region {
          match heap_box(store: &uniq heap, value: 2_u64) {
            Ok(value: second) => {
              let previous = replace first = move second;
              let old = deref(previous);
              return exit_status(code: 0_u8);
            }
            Err(error: back) => {
              return exit_status(code: 1_u8);
            }
          }
        }
      }
      Err(error: back) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an affine cell descriptor replace must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        assert!(
            replace_reachable(&main.body),
            "the exchange must remain a checked Replace"
        );
    });
}

/// Whether any statement of this body, or of a nested block below it, is a
/// `replace`. The cell allocation is fallible, so the commit sits inside the
/// taken arm rather than at the top of `main`.
fn replace_reachable(body: &[CheckedStatement]) -> bool {
    body.iter().any(|statement| match statement {
        CheckedStatement::Replace { .. } => true,
        CheckedStatement::Region { body, .. } => replace_reachable(body),
        CheckedStatement::Match { arms, .. } => arms.iter().any(|arm| replace_reachable(&arm.body)),
        _ => false,
    })
}
