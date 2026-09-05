use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::{assert_rule, assert_rule_kind, assert_unsupported, with_semantics};

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
  fn make['r]() -> result: own arena<'r, i32> allocates(arena 'r);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        b"fn quiet() -> result: own unit pure {\n  return unit;\n}\n",
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
        b"fn quiet(storage: own arena<i32>) -> result: own unit pure {\n  return unit;\n}\n",
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
    assert_rule_kind(
        br#"fn views['s](storage: own arena<array<u8, 2>>) -> result: own slice<'s, u8> pure {
  let view = slice_of(&'s deref(storage));
  return move view;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
}

/// A within-region view of arena content checks; the function then stops at
/// the explicit temporary arena-runtime capability gate rather than lowering
/// wrong code, and the manifest keeps such positives pending until the
/// region-tied release lowering lands.
#[test]
fn checked_arena_parameters_stop_at_the_explicit_runtime_gate() {
    assert_unsupported(
        br#"fn views(storage: own arena<array<u8, 2>>) -> result: own unit pure {
  region {
    let view = slice_of(&deref(storage));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(7_u8);
  region 'r {
    let a = arena_new::<'r, array<u8, 2>>(move values);
    let view = slice_of(&deref(a));
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}

/// [OWN-14] defines a reborrow form by its root binding's *mode*, so a borrow
/// of a local arena's content — an own-mode binding — is an ordinary borrow,
/// never a reborrow. Dispatching on the `deref` spelling alone demanded a
/// borrow holder these programs never wrote and reported spec-legal source as
/// an OWN-6, OWN-14, or TYPE-7 violation. Each shape now reaches [OWN-10]'s
/// arena case with source region `'r` [STOR-4] and then stops at the explicit
/// arena-runtime gate, because no checked expression addresses arena content.
#[test]
fn arena_content_borrows_are_ordinary_borrows_rather_than_reborrows() {
    // A `uniq` child in the arena's own region, and the same borrow under a
    // nested region: `'r` outlives-or-equals both.
    assert_unsupported(
        br#"fn bump(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 42_i32;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    bump(n: &uniq deref(a));
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
    assert_unsupported(
        br#"fn bump(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 42_i32;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    region {
      bump(n: &uniq deref(a));
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
    // A shared borrow in argument position, and a `let`-bound holder, which
    // OWN-14 rejected outright as a non-argument reborrow position.
    assert_unsupported(
        br#"fn peek(n: &i32) -> result: own i32 reads(n) {
  return deref(n);
}

command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    let v = peek(n: &deref(a));
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
    assert_unsupported(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    let h = &uniq deref(a);
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}

/// The ordinary judgments the routed borrow now reaches still reject: [OWN-10]
/// with the arena's source region for a borrow region `'r` does not
/// outlive-or-equal, and [OWN-11] for a region introduced outside the loop.
#[test]
fn arena_content_borrows_keep_their_region_rejections() {
    // An enclosing region outlives the arena's, so its storage is too
    // short-lived for the borrow.
    assert_rule_kind(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'o {
    region 'r {
      let a = arena_new::<'r, i32>(4_i32);
      let h = &uniq 'o deref(a);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    // A caller-supplied region is never comparable to a local arena's [OWN-3].
    assert_rule_kind(
        br#"fn hold(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 1_i32;
  return unit;
}

fn outer['s](anchor: &'s i32) -> result: &'s i32 pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    hold(n: &uniq 's deref(a));
  }
  return anchor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    // [OWN-11] a loop body hosts its own borrows, so reaching the arena's own
    // region from inside the body means naming it.
    assert_rule(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    loop @once {
      let h = &uniq 'r deref(a);
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

/// The set-target twin of the borrow dispatch above. [SET-1] makes a `deref`
/// target writable through a live own-mode binding whose storage is
/// arena-owned [STOR-1] just as it does through a live usable `&uniq` holder,
/// and an `arena_new` result is own mode. Resolving a holder for every
/// `deref` target reported this spec-legal target as TYPE-7 "deref requires a
/// borrow holder"; it is now judged as the own-rooted target it is — the
/// content is copy, so SET-1 admits it — and stops at the arena-runtime gate,
/// because arena storage has no runtime to store into.
#[test]
fn arena_content_set_targets_are_own_rooted_rather_than_holder_derefs() {
    assert_unsupported(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_i32);
    set deref(a) = 7_i32;
  }
  return exit_status(code: 0_u8);
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
        br#"command fn main() -> status: own ExitStatus pure {
  let flag = True();
  let escaped = if flag {
    region 'r {
      let a = arena_new::<'r, i32>(1_i32);
      give move a;
    }
  } else {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor4,
        SemanticIssueKind::ArenaEscape {
            mechanical_fix: "keep the arena value inside its region's block; \
     return or deliver its content, or a borrow OWN-10 admits, instead",
        },
    );
}

/// [STOR-2, TYPE-5] `arena_new::<'r, T>(v)` requires `v` to produce exactly T.
#[test]
fn arena_new_operands_must_match_the_written_content_type() {
    assert_rule_kind(
        br#"command fn main() -> status: own ExitStatus pure {
  region 'r {
    let a = arena_new::<'r, i32>(4_u64);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

/// Allocation into a caller-supplied region and content carrying its own
/// release action both stay explicit capability stops rather than silent
/// leaks or fabricated rejections.
#[test]
fn caller_region_allocation_and_owning_content_stop_at_the_runtime_gate() {
    assert_unsupported(
        br#"fn fill['r]() -> result: own unit allocates(arena 'r) {
  let a = arena_new::<'r, i32>(1_i32);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
    assert_unsupported(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let boxed = box_new(9_i32);
  region 'r {
    let a = arena_new::<'r, box<i32>>(move boxed);
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::ArenaRuntime,
    );
}

/// [PROV-6, STOR-1, STOR-3] a store-backed run's release class is decided from
/// its store region's declaration alone and travels in its type.
///
/// No heap value exists in this version, so nothing releases through a free
/// yet and no program can observe the difference at run time. The
/// classification is what a heap-backed run's lowering will select between, so
/// it is pinned here rather than left to the version that first spends it: an
/// `affine`-bounded region parameter and a `region_stmt` region are bump
/// extents whose reclamation is the region's own reset, and the entry heap, an
/// unbounded region parameter and a `linear`-bounded one are general stores.
#[test]
fn a_runs_release_class_is_read_off_its_store_regions_declaration() {
    let source = br#"fn from_extent['s: affine](run: own Vector<'s, u64>) -> back: own Vector<'s, u64> pure {
  doc "An affine-bounded region parameter is a bump extent.";
  return move run;
}

fn from_general['s: linear](run: own Vector<'s, u64>) -> back: own Vector<'s, u64> pure {
  doc "A linear-bounded region parameter is a general store.";
  return move run;
}

fn from_unconstrained['s](run: own Vector<'s, u64>) -> back: own Vector<'s, u64> pure {
  doc "An unbounded region parameter is a general store, fail-closed.";
  return move run;
}

fn from_entry_heap(run: own Vector<u64>) -> back: own Vector<u64> pure {
  doc "An elided store brand at a parameter position is the entry heap's store region.";
  return move run;
}

command fn main() -> status: own ExitStatus pure {
  doc "The four declarations are checked; none is called, because no program can produce a general store's run yet.";
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the four run declarations must check: {outcome:?}");
        };
        let class = |name: &str| {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .expect("each declaration is one checked function");
            match function.result {
                crate::semantic::CheckedType::Vector { release, .. } => release,
                other => panic!("{name} must return a run, got {other:?}"),
            }
        };
        assert_eq!(
            class("from_extent"),
            crate::semantic::CheckedReleaseClass::Extent
        );
        assert_eq!(
            class("from_general"),
            crate::semantic::CheckedReleaseClass::General
        );
        assert_eq!(
            class("from_unconstrained"),
            crate::semantic::CheckedReleaseClass::General
        );
        assert_eq!(
            class("from_entry_heap"),
            crate::semantic::CheckedReleaseClass::General
        );
    });
}
