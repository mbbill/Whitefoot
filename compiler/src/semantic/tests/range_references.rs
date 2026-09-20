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

use crate::{CallRequirementDisposition, SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_accepts, assert_rule_kind, with_semantics};

/// Asserts that one source is refused by [FN-8] with the exact disposition and
/// the exact instantiated goal text the call owes.
fn assert_call_goal(source: &[u8], disposition: CallRequirementDisposition, goal: &str) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the call requirement must be refused: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!("FN-8 must identify the failing requirement: {issue:?}");
        };
        assert_eq!(detail.instantiated_goal, goal);
        assert_eq!(detail.disposition, disposition);
    });
}

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

/// One callee whose requirement is stated over the range its parameter names.
///
/// [REF-4]: a range reference's "one measure is `len`, equal to `hi - lo`
/// [MSR-1]". [MSR-1] gives `&[T]` its own row, `len` = "range elements,
/// exact", and [OP-15] spells the read `deref(part).len`. So at every call the
/// requirement is instantiated over the actual range's own length and never
/// over the length of the storage that range was formed from: the range
/// `a[2..4]` of a four-element array has two elements, and an offset judged
/// against `a.len` would be admitted and would read outside the range.
const RANGE_OFFSET_CALLEE: &str = r#"fn at(part: &[u8], offset: own u64) -> result: own u8 reads(part) contract {
  requires offset < deref(part).len;
} {
  return deref(part)[offset];
}
"#;

/// A range a binding names: the requirement is that binding's own `len`, and
/// the goal names the place the writer wrote, `deref(view)` [REF-1, OP-15].
#[test]
fn a_requirement_over_a_bound_range_is_that_ranges_length() {
    let body = |offset: &str| {
        format!(
            "{RANGE_OFFSET_CALLEE}
fn main() -> status: own ExitStatus pure {{
  let a = array_filled::<u8, 4>(value: 1_u8);
  let view = &a[2_u64..4_u64];
  let x = at(part: view, offset: {offset});
  return exit_status(code: x);
}}
"
        )
    };
    assert_accepts(body("1_u64").as_bytes());
    assert_call_goal(
        body("3_u64").as_bytes(),
        CallRequirementDisposition::Refuted,
        "3_u64 < deref(view).len",
    );
}

/// A range formed at the call names no binding, so it has no [ENT-2] place of
/// its own. Its goal keeps the formation's range step over the storage path,
/// which is what holds its `len` apart from that storage's own `len`
/// (compiler/checker-facts, pending: "A `&[T]` actual that forms its range at
/// the call keeps that formation's range step over the resolved storage path,
/// as one ordered projection of the goal datum"). The length is the
/// mathematical difference of the two captured endpoints [REF-4], so `a[2..4]`
/// is two elements although `a` is four.
#[test]
fn a_requirement_over_a_range_formed_at_the_call_is_that_ranges_length() {
    let body = |offset: &str| {
        format!(
            "{RANGE_OFFSET_CALLEE}
fn main() -> status: own ExitStatus pure {{
  let a = array_filled::<u8, 4>(value: 1_u8);
  let x = at(part: &a[2_u64..4_u64], offset: {offset});
  return exit_status(code: x);
}}
"
        )
    };
    assert_accepts(body("1_u64").as_bytes());
    assert_call_goal(
        body("3_u64").as_bytes(),
        CallRequirementDisposition::Refuted,
        "3_u64 < a[2..4].len",
    );
}

/// [REF-4]: "Re-slicing is admitted: `&deref(part)[a..b]` under
/// `a <= b <= deref(part).len`." The inner range's `len` is its own `b - a`,
/// not the outer range's and not the array's.
#[test]
fn a_requirement_over_a_reslice_is_the_inner_ranges_length() {
    let body = |offset: &str| {
        format!(
            "{RANGE_OFFSET_CALLEE}
fn main() -> status: own ExitStatus pure {{
  let a = array_filled::<u8, 4>(value: 1_u8);
  let view = &a[0_u64..4_u64];
  let sub = &deref(view)[2_u64..4_u64];
  let x = at(part: sub, offset: {offset});
  return exit_status(code: x);
}}
"
        )
    };
    assert_accepts(body("1_u64").as_bytes());
    assert_call_goal(
        body("3_u64").as_bytes(),
        CallRequirementDisposition::Refuted,
        "3_u64 < deref(sub).len",
    );
}

/// A range forwarded through a `&[T]` parameter: [REF-1] makes "passing a bare
/// reference variable where a `&T` or `&[T]` parameter is expected passes that
/// reference", so the inner call's requirement instantiates over the
/// forwarding function's own parameter and is discharged, or refuted, from
/// that function's entry facts alone [ENT-3.S4].
#[test]
fn a_requirement_over_a_forwarded_range_is_the_forwarded_ranges_length() {
    let body = |offset: &str| {
        format!(
            "{RANGE_OFFSET_CALLEE}
fn forward(part: &[u8]) -> result: own u8 reads(part) contract {{
  requires deref(part).len == 2_u64;
}} {{
  return at(part: part, offset: {offset});
}}

fn main() -> status: own ExitStatus pure {{
  let a = array_filled::<u8, 4>(value: 1_u8);
  let view = &a[2_u64..4_u64];
  let x = forward(part: view);
  return exit_status(code: x);
}}
"
        )
    };
    assert_accepts(body("1_u64").as_bytes());
    assert_call_goal(
        body("3_u64").as_bytes(),
        CallRequirementDisposition::Refuted,
        "3_u64 < deref(part).len",
    );
}

/// [ENT-3.S6]: "`let part = &P[lo..hi];` for a tracked P establishes
/// `deref(part).len = hi - lo` after [REF-4]'s domain goals discharge, over
/// the exact current-value images captured where the endpoints are evaluated."
/// It is an ordinary fact of the state, so a caller that has just formed the
/// range discharges an equality over its length and refutes a different one.
/// `&table[4_u64..24_u64]` is twenty elements however long `table` is.
#[test]
fn a_range_formation_establishes_its_length_equality() {
    let body = |length: &str| {
        format!(
            "fn expects(part: &[u8]) -> result: own u64 reads(part.len) contract {{
  requires deref(part).len == {length};
}} {{
  return deref(part).len;
}}

fn main() -> status: own ExitStatus pure {{
  let table = array_filled::<u8, 32>(value: 0_u8);
  let view = &table[4_u64..24_u64];
  let n = expects(part: view);
  return exit_status(code: 0_u8);
}}
"
        )
    };
    assert_accepts(body("20_u64").as_bytes());
    assert_call_goal(
        body("32_u64").as_bytes(),
        CallRequirementDisposition::Refuted,
        "deref(view).len == 32_u64",
    );
}
