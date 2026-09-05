//! [SET-2] affine-place replacement: target class, commit semantics, root
//! liveness, and the ENT-5 kill interaction that keeps a stale length fact
//! from discharging a post-replace subscript.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedFlatElement, CheckedSetTarget, CheckedStatement};
use super::{assert_rule, with_semantics};

const HOLDER: &[u8] = br#"struct Holder {
  payload: buffer<u8>;
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
  let first = buffer_new(4_u64, 7_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(2_u64, 9_u8);
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
        } = &main.body[3]
        else {
            panic!("the fourth statement must be the SET-2 commit");
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
  let first = buffer_new(1_u64, 0_u8);
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
  let first = buffer_new(1_u64, 0_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(1_u64, 0_u8);
  set holder.payload = move second;
  return exit_status(code: 0_u8);
}
"#,
    );
    assert_rule(
        &source,
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "buffer<u8>".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
}

#[test]
fn replace_kills_the_stale_length_fact_at_the_commit() {
    // Without the ENT-5 SET-2 kill, the pre-replace `size = 4` fact would
    // discharge the post-replace subscript over the two-element buffer and
    // the accepted program would write out of bounds.
    let source = with_holder(
        br#"command fn main() -> status: own ExitStatus pure {
  let first = buffer_new(4_u64, 7_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let size = len_of(holder.payload);
  let allocated_length = size == 4_u64;
  if allocated_length {
    let second = buffer_new(2_u64, 9_u8);
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
  let first = buffer_new(4_u64, 7_u8);
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
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(3_u64, 2_u8);
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
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let gone = sink(h: move holder);
  let second = buffer_new(1_u64, 0_u8);
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
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  region {
    let view = &holder;
    let second = buffer_new(1_u64, 0_u8);
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
  let slots = buffer_vacant::<u32>(4_u64);
  let filled = Some<u32>(value: 7_u32);
  let vacant = replace slots[2_u64] = move filled;
  let taken = replace slots[2_u64] = None<u32>();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an affine element replace must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Replace { target, .. } = &main.body[2] else {
            panic!("the third statement must be the SET-2 element commit");
        };
        let CheckedSetTarget::BufferIndex(target) = target else {
            panic!("an element replace target retains its buffer root");
        };
        let CheckedFlatElement::Nominal(element) = target.root.element else {
            panic!("the element is the affine Option instance");
        };
        assert_eq!(
            checked.data.nominals[element.0 as usize].name,
            "Option<u32>"
        );
        assert!(!target.obligation.components().is_empty());
        assert!(matches!(
            &main.body[3],
            CheckedStatement::Replace {
                target: CheckedSetTarget::BufferIndex(_),
                ..
            }
        ));
    });
}

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
    let source = br#"fn hollow(n: own u64) -> result: own unit pure contract {
  requires buffer_fits::<Option<u32>>(n);
} {
  let slots = buffer_vacant::<u32>(n);
  let taken = replace slots[0_u64] = None<u32>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  hollow(n: 2_u64);
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
  let slots = buffer_vacant::<u32>(4_u64);
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
  let slots = buffer_vacant::<u32>(4_u64);
  set slots[0_u64] = None<u32>();
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
  let slots = buffer_vacant::<u32>(4_u64);
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
  let slots = buffer_vacant::<u32>(4_u64);
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
  let first = buffer_new(2_u64, 1_u8);
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
  let first = buffer_new(1_u64, 0_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(1_u64, 0_u16);
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
#[test]
fn replace_of_a_region_bearing_place_rejects_citing_set2() {
    let expected_fix = "a slice's static origin set and an arena's confinement are fixed at \
                        initialization; bind a new slice or arena under a new let";
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  let left = array_new::<u8, 2>(11_u8);
  let right = array_new::<u8, 2>(29_u8);
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
fn replace_of_a_box_descriptor_accepts() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let first = box_new(1_u64);
  let second = box_new(2_u64);
  let previous = replace first = move second;
  let old = deref(previous);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a region-free affine descriptor replace must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        assert!(matches!(main.body[2], CheckedStatement::Replace { .. }));
    });
}
