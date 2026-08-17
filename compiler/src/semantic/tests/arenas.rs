use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::{assert_rule, assert_unsupported, with_semantics};

/// [STOR-4] a value of type `arena<'r, T>` may not be returned, so a result
/// type naming an arena is rejected at the callable boundary — and the
/// rejection is reported even though the unit also lacks `main`, because a
/// declaration's own established violation is ordered before the FN-7
/// whole-unit rejection [DIAG-1]. This is the stor4-neg-arena-escape
/// conformance case byte for byte.
#[test]
fn arena_results_reject_citing_stor4_before_missing_main() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/stor4-neg-arena-escape.wf"),
        SemanticRule::Stor4,
        SemanticIssueKind::ArenaEscape {
            mechanical_fix: "keep the arena value inside its region's block; \
     return or deliver its content, or a borrow OWN-10 admits, instead",
        },
    );
}

/// A contract member signature is judged at the same callable boundary, so an
/// arena member result is the same STOR-4 rejection.
#[test]
fn contract_member_arena_results_reject_citing_stor4() {
    assert_rule(
        br#"contract Maker {
  fn make['r]() -> own arena<'r, i32> allocates(arena 'r);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor4,
        SemanticIssueKind::ArenaEscape {
            mechanical_fix: "keep the arena value inside its region's block; \
     return or deliver its content, or a borrow OWN-10 admits, instead",
        },
    );
}

/// The missing-`main` salvage is not a rewrite: a unit whose declarations all
/// check clean still reports the FN-7 whole-unit rejection at `BundleRoot`.
#[test]
fn missing_main_still_rejects_when_nothing_else_does() {
    with_semantics(
        b"fn quiet() -> own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::SourceIssue { issue } = outcome else {
                panic!("a main-less unit must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn7);
            assert_eq!(issue.kind(), &SemanticIssueKind::MissingMain);
        },
    );
}

/// An unsupported capability in a main-less unit must not mask the definite
/// FN-7 violation [DIAG-1]: the salvage pre-pass only promotes established
/// source rejections.
#[test]
fn missing_main_wins_over_an_unsupported_capability() {
    with_semantics(
        b"fn quiet['r](storage: own arena<'r, i32>) -> own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::SourceIssue { issue } = outcome else {
                panic!("a main-less unit must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn7);
            assert_eq!(issue.kind(), &SemanticIssueKind::MissingMain);
        },
    );
}

/// [FN-1] an `arena<'r, U>` parameter is not an input-slice supplier: a view
/// formed over its content has a resolved source-place origin outside the
/// return-origin ceiling, rejected at the `return_stmt`. This is the
/// fn1-neg-returned-slice-arena-origin conformance case byte for byte.
#[test]
fn arena_content_views_stay_outside_the_slice_return_ceiling() {
    assert_rule(
        include_bytes!(
            "../../../../tests/conformance/cases/fn1-neg-returned-slice-arena-origin.wf"
        ),
        SemanticRule::Fn1,
        SemanticIssueKind::InvalidSliceReturnOrigin {
            mechanical_fix: "accept an exact direct input slice in the result region or keep \
                             the newly formed view in its caller; do not return a view of raw \
                             callee storage",
        },
    );
}

/// [OWN-10] a borrow of arena content uses source region 'r: a view formed
/// under an incomparable caller region fails closed [OWN-3].
#[test]
fn arena_content_borrows_obey_own10_with_the_arena_region() {
    assert_rule(
        br#"fn views['r, 's](storage: own arena<'r, array<u8, 2>>) -> own slice<'s, u8> pure {
  let view = slice_of(&'s deref(storage));
  return move view;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own10,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
}

/// A within-region view of arena content checks; the function then stops at
/// the explicit temporary arena-runtime capability gate rather than lowering
/// wrong code, and the manifest keeps such positives pending until the
/// region-tied release lowering lands.
#[test]
fn checked_arena_parameters_stop_at_the_explicit_runtime_gate() {
    assert_unsupported(
        br#"fn views['r](storage: own arena<'r, array<u8, 2>>) -> own unit pure {
  let view = slice_of(&'r deref(storage));
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}

/// A view over a *local* arena's content reaches no later arena gate, so it
/// stops where it is formed. Without that stop it published a checked program
/// carrying a slice source no IR builder can lower, and compilation died as an
/// internal `InvalidCheckedProgram` on source the checker had accepted.
#[test]
fn local_arena_content_views_stop_at_the_explicit_runtime_gate() {
    assert_unsupported(
        br#"fn main() -> own unit pure {
  let values = array_new<u8, 2>(7_u8);
  region 'r {
    let a = arena_new<'r, array<u8, 2>>(move values);
    let view = slice_of(&'r deref(a));
  }
  return unit;
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}

/// [STOR-4] a value delivery whose destination binding lies outside the
/// arena's region block moves the value out of its region and rejects.
#[test]
fn arena_deliveries_may_not_leave_their_region_block() {
    assert_rule(
        br#"fn main() -> own unit pure {
  let flag = True();
  let escaped = if flag {
    region 'r {
      let a = arena_new<'r, i32>(1_i32);
      give move a;
    }
  } else {
    return unit;
  }
  return unit;
}
"#,
        SemanticRule::Stor4,
        SemanticIssueKind::ArenaEscape {
            mechanical_fix: "keep the arena value inside its region's block; \
     return or deliver its content, or a borrow OWN-10 admits, instead",
        },
    );
}

/// [STOR-2, TYPE-5] `arena_new<'r, T>(v)` requires `v` to produce exactly T.
#[test]
fn arena_new_operands_must_match_the_written_content_type() {
    assert_rule(
        br#"fn main() -> own unit pure {
  region 'r {
    let a = arena_new<'r, i32>(4_u64);
  }
  return unit;
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

/// Allocation into a caller-supplied region and content carrying its own
/// release action both stay explicit capability stops rather than silent
/// leaks or fabricated rejections.
#[test]
fn caller_region_allocation_and_owning_content_stop_at_the_runtime_gate() {
    assert_unsupported(
        br#"fn fill['r]() -> own unit allocates(arena 'r) {
  let a = arena_new<'r, i32>(1_i32);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
    assert_unsupported(
        br#"fn main() -> own unit allocates(heap) {
  let boxed = box_new(9_i32);
  region 'r {
    let a = arena_new<'r, box<i32>>(move boxed);
  }
  return unit;
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}
