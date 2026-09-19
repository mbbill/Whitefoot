//! [REF-4] range references, and [CALL-3]'s transport of a write through one.
//!
//! This module replaces `tests/slices.rs` (30 tests), whose subject was
//! v0.59's slice and view values. Every rule it exercised is retired, and each
//! retirement has a named successor:
//!
//! - [VIEW-1] view formation and the `slice_of` / `mut_slice_of` formers
//!   retire. The successor is [REF-4]'s `&x[lo..hi]`, a place form under the
//!   obligation `lo <= hi` and `hi <= x.len`, and re-slicing
//!   `&deref(part)[a..b]`.
//! - [VIEW-2] range separation retires as a judgment of its own. The successor
//!   is [OWN-7]'s range-step relation, whose four non-strict orderings are
//!   submitted by [EFF-5] at a call and reported as
//!   `UndischargedRangeSeparation` citing EFF-5.
//! - [VIEW-4] the slice type and [VIEW-6] the slice return ceiling retire.
//!   `Slice<T>` and `MutSlice<T>` are not types at all: `&[T]` is a reference
//!   KIND admitted only in parameter position [TYPE-8, GRAM-2], is never a
//!   stored value, never a result and never a generic type argument, and a
//!   returned reference is refused by [REF-3].
//! - The exclusive/shared distinction over a view retires with the permission
//!   markers: there is one reference kind, and whether a callee writes through
//!   it is stated by its effect row [EFF-1].
//! - Region-scoped loan lifetimes retire with regions [OWN-3, OWN-4, OWN-10,
//!   FORM-8]; the successor is [REF-2] validity, exercised in
//!   `tests/references.rs`.
//!
//! The [OP-4] cases slices.rs carried — the data-dependent scatter and the
//! scalar count bound — keep their live subject rule and belong with the other
//! subscript tests, not with the retired view apparatus.

use crate::{SemanticIssueKind, SemanticRule};

use super::{assert_accepts, assert_rule_kind};

/// [REF-4] `&x[lo..hi]` over an indexable place, re-slicing over another range
/// reference, and the kind's one measure `len` equal to `hi - lo`.
#[test]
fn a_range_reference_forms_and_reslices() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref4-pos-range-reference-and-reslice.wf"
    ));
}

/// [REF-4] a range reference over a `Ring` is a hard error: a wrapped window is
/// two extents and `&[T]` has one `len`.
#[test]
fn a_range_reference_over_a_ring_is_refused() {
    let source = include_bytes!("../../../../tests/conformance/cases/ref4-neg-range-over-ring.wf");
    assert_rule_kind(source, SemanticRule::Ref4, |kind| {
        matches!(kind, SemanticIssueKind::RangeOverRing { .. })
    });
}

/// [REF-4] formation submits `lo <= hi` and `hi <= x.len` to [MSR-4]. An
/// endpoint above the base's length discharges neither, and the rejection
/// carries the residual and the rule's own restructuring.
#[test]
fn an_endpoint_above_the_length_leaves_the_formation_undischarged() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let a = array_filled::<u64, 4>(value: 0_u64);
  let hi = 9_u64;
  let part = &a[0_u64..hi];
  let seen = deref(part).len;
  if seen == 9_u64 {
  } else {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref4, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedRangeFormationObligation { .. }
        )
    });
}

/// [CALL-3] a write through a range reference reaches the range's element
/// storage and no measure of the origin place itself, so after the call the
/// caller still holds both lengths and the bytes the callee wrote.
#[test]
fn a_write_through_a_range_reference_keeps_both_lengths() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/call3-pos-a-fill-through-an-exclusive-view-keeps-both-lengths.wf"
    ));
}

/// [CALL-1] through a reference the callee only reads, every fact survives.
#[test]
fn a_read_only_reference_argument_keeps_every_fact() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/call1-pos-a-shared-borrow-keeps-every-fact.wf"
    ));
}

/// [OWN-7] two index steps whose offsets the fixed [ENT-6] families prove
/// distinct select two different storages, which is the acceptance half the
/// overlap relation must keep.
#[test]
fn proved_distinct_indices_do_not_overlap() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/own7-pos-distinct-noverlap.wf"
    ));
}
