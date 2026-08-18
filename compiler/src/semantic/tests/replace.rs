//! [SET-2] affine-place replacement: target class, commit semantics, root
//! liveness, and the ENT-5 kill interaction that keeps a stale length fact
//! from discharging a post-replace subscript.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedSetTarget, CheckedStatement};
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(4_u64, 7_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(2_u64, 9_u8);
  let old = replace holder.payload = move second;
  let size = len(old);
  check ieq(size, 4_u64) else trap "previous buffer length";
  return unit;
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
        // The fresh binding is a real binding: the later `len(old)` read
        // resolves against it, which the accepted outcome already proves.
        assert!(binding.0 > 0);
    });
}

#[test]
fn replace_of_a_copy_place_rejects_citing_set2() {
    let source = with_holder(
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(1_u64, 0_u8);
  let holder = Holder(payload: move first, count: 3_u64);
  let old = replace holder.count = 4_u64;
  return unit;
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(1_u64, 0_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(1_u64, 0_u8);
  set holder.payload = move second;
  return unit;
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(4_u64, 7_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let size = len(holder.payload);
  check ieq(size, 4_u64) else trap "allocated length";
  let second = buffer_new(2_u64, 9_u8);
  let old = replace holder.payload = move second;
  set holder.payload[3_u64] = 5_u8;
  return unit;
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(4_u64, 7_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let size = len(holder.payload);
  check ieq(size, 4_u64) else trap "allocated length";
  set holder.payload[3_u64] = 5_u8;
  return unit;
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
        br#"fn consume(h: own Holder) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(3_u64, 2_u8);
  let old = replace holder.payload = move second;
  set holder.count = 1_u64;
  let observed = holder.count;
  check ieq(observed, 1_u64) else trap "root stays live";
  let done = consume(h: move holder);
  return unit;
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
        br#"fn sink(h: own Holder) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let gone = sink(h: move holder);
  let second = buffer_new(1_u64, 0_u8);
  let old = replace holder.payload = move second;
  return unit;
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(2_u64, 1_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  region 'r {
    let view = &'r holder;
    let second = buffer_new(1_u64, 0_u8);
    let old = replace deref(view).payload = move second;
  }
  return unit;
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
fn element_position_replace_rejects_while_every_element_is_copy() {
    let source = with_holder(
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(2_u64, 1_u8);
  let old = replace first[0_u64] = 3_u8;
  return unit;
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
        br#"fn main() -> own unit allocates(heap), traps {
  let first = buffer_new(1_u64, 0_u8);
  let holder = Holder(payload: move first, count: 0_u64);
  let second = buffer_new(1_u64, 0_u16);
  let old = replace holder.payload = move second;
  return unit;
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
