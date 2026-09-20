//! [WIN-1] through [WIN-3], [TYPE-9], [TYPE-10] and the window, construction
//! and release operations [OP-10, OP-13, OP-14, OP-15].
//!
//! This module replaces `tests/buffers.rs` (22 tests), whose subject was
//! v0.59's `buffer` storage class and the store apparatus around it. Each
//! retirement has a named successor:
//!
//! - The storage names `buffer`, `FixedVector`, `Vector<'s, T>`, `arena<'r, T>`
//!   and `box<T>` retire with [TYPE-2]'s and [GRAM-3]'s alternatives. The
//!   successor is [TYPE-9]: `Array`, `Slots` and `Ring` in a constant-capacity
//!   and a runtime-capacity form each, and the cell `Box<T>` whose content is
//!   its field `inner`.
//! - [BLK-1]'s two-run inventory, [BLK-2]'s formation rows, [BLK-3]'s refusal
//!   of swap, growth and middle removal, [BLK-4]'s confinement and [PROV-1]'s
//!   store/provider/capability table retire. The successor is the ordinary
//!   [PRE-1] records: the nine window operations [OP-10], the nine
//!   construction functions [OP-13], `swap` [OP-11] and `free_empty` [OP-14],
//!   over one heap [STOR-8].
//! - `buffer_new`, `buffer_vacant`, `array_new` and `box_new`'s old spelling
//!   leave [OP-1]'s table; `buffer_fits` and its static allocation-fit
//!   predicate leave with them. The successor of the fit predicate is [OP-9]'s
//!   allocation-size obligation, still an `AllocationFit` obligation family in
//!   the checker but now stated over `stride_ceiling(T) * n` staying inside
//!   u64, carried by each runtime-capacity construction and by `grow`.
//! - `dispose` retires with the capability leaves. The successor is [OP-14]
//!   `free_empty(window: move r)` on a proved-empty window, and for an affine
//!   value, moving it into a function that consumes it [PROV-6].
//! - The `len_of` / `cap_of` / `vacant_of` / `extent_of` former family retires.
//!   The successor is [OP-15]: `r.len`, `r.cap` and `r.head` are place forms,
//!   not calls.
//!
//! The sources are the already-ported v0.60 conformance cases; these tests add
//! the rule and issue kind the corpus manifest does not pin.
//!
//! Two of the judgments below are stated by the specification and are NOT
//! implemented by the checker as this package lands. Each assertion here is
//! the normative verdict, not the current one, because an unimplemented
//! feature is not a source-language rejection and must not rewrite a
//! normative expectation. The two gaps, with the code that owns each:
//!
//! 1. [OP-10]'s compiler-owned window type parameter is not inferred from the
//!    operand. `check/generics.rs` (`generic_substitution`) refuses any call
//!    to a callee with type parameters and no written argument list, citing
//!    FN-2, while [OP-10] states that "a window operation, `swap` [OP-11],
//!    and `free_empty` [OP-14] therefore write no type arguments at a call:
//!    every type parameter of those rows is supplied by an operand".
//! 2. A runtime-capacity `Slots<T>` or `Ring<T>`, and a constant-capacity
//!    `Ring<T, N>`, stop as an unimplemented compiler capability at
//!    `check/types.rs`.
//!
//! The judgments this package's port closed, and which are now ordinary
//! assertions above: a `Box`'s content reached by the bare field step
//! `b.inner` and consumed by `let n = move b.inner;` [TYPE-9, WIN-3]; a move
//! out of a window slot citing WIN-3 with `MoveOutOfSlot`; [OP-14]'s own site
//! for an undischarged `free_empty` requirement; [OP-9]'s allocation-size
//! obligation at each runtime-capacity construction and at `grow`, rendered
//! as its defining comparison rather than as the retired `buffer_fits` row;
//! and [OP-15]'s measure member read in expression position, including one
//! over a subscripted place.

use crate::{SemanticIssueKind, SemanticRule};

use super::{assert_accepts, assert_rule_kind};

/// [WIN-1] a `Slots` is a run of `cap` slots whose initialized storage is the
/// `len` slots beginning at `head`, and an `Array` has no window at all.
#[test]
fn a_window_is_its_len_slots_and_an_array_has_none() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/win1-pos-window-is-len-slots.wf"
    ));
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/win1-pos-array-len-equals-cap.wf"
    ));
}

/// [WIN-1] a subscript `r[i]` carries [OP-4]'s obligation `i < r.len`, stated
/// against `len` and never against `cap` or `head`.
#[test]
fn a_subscript_above_the_length_is_undischarged() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/win1-neg-subscript-above-len.wf");
    assert_rule_kind(source, SemanticRule::Op4, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedBoundsObligation { .. })
    });
}

/// [WIN-2] the four window parts are vocabulary for effect rows and the
/// overlap judgment only: a declared row may name one, and no program writes
/// one.
#[test]
fn a_window_part_is_row_vocabulary_and_not_a_place() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/win2-pos-part-effect-row.wf"
    ));
    let written =
        include_bytes!("../../../../tests/conformance/cases/win2-neg-window-part-write-target.wf");
    assert_rule_kind(written, SemanticRule::Type10, |kind| {
        matches!(kind, SemanticIssueKind::ReservedPseudoField { .. })
    });
}

/// [TYPE-2] a measure is a readonly field of the shape's [PRE-1]
/// declaration: it is read like any field and is never a write target, and
/// only [OP-10] and [OP-13] change one.
#[test]
fn a_measure_is_read_only() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/type10-pos-measures-are-read-only.wf"
    ));
    let written = include_bytes!(
        "../../../../tests/conformance/cases/type2-neg-readonly-measure-write-target.wf"
    );
    assert_rule_kind(written, SemanticRule::Type2, |kind| {
        matches!(kind, SemanticIssueKind::ReadonlyWriteTarget { .. })
    });
}

/// x1 [TYPE-10]: the measure and window-part spellings reserve nothing, so a
/// source struct of another type declares fields named `len` and `next` and
/// bindings carry the part spellings.
#[test]
fn member_spellings_reserve_nothing() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/type10-pos-member-spellings-reserve-nothing.wf"
    ));
}

/// [TYPE-2] a `set` whose target ends at or passes through a readonly field
/// is refused in a source struct exactly as in a prelude one, and an argument
/// naming such a path at a written reference parameter is refused with it.
#[test]
fn a_readonly_field_is_never_a_write_target() {
    for source in [
        include_bytes!(
            "../../../../tests/conformance/cases/type2-neg-readonly-field-set-target.wf"
        )
        .as_slice(),
        include_bytes!(
            "../../../../tests/conformance/cases/type2-neg-readonly-path-passes-through.wf"
        )
        .as_slice(),
        include_bytes!(
            "../../../../tests/conformance/cases/type2-neg-readonly-field-written-argument.wf"
        )
        .as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Type2, |kind| {
            matches!(kind, SemanticIssueKind::ReadonlyWriteTarget { .. })
        });
    }
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/type2-pos-readonly-field-read-and-whole-replace.wf"
    ));
}

/// [TYPE-2] readonly provenance follows references and reborrows. A written
/// actual cannot hide the readonly field by naming an alias or by stepping
/// through a reference to the enclosing value.
#[test]
fn readonly_provenance_survives_reference_aliases_and_reborrows() {
    for source in [
        br#"struct Record {
  readonly value: u8;
}

fn put(cell: &u8) -> result: own unit writes(cell) {
  set deref(cell) = 9_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let record = Record(value: 1_u8);
  let p = &record.value;
  put(cell: p);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"struct Record {
  readonly value: u8;
}

fn put(cell: &u8) -> result: own unit writes(cell) {
  set deref(cell) = 9_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let record = Record(value: 1_u8);
  let p = &record;
  put(cell: &deref(p).value);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"struct Record {
  readonly value: u8;
}

fn put(cell: &u8) -> result: own unit writes(cell) {
  set deref(cell) = 9_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let record = Record(value: 1_u8);
  let p = &record.value;
  put(cell: &deref(p));
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Type2, |kind| {
            matches!(kind, SemanticIssueKind::ReadonlyWriteTarget { .. })
        });
    }
}

/// [OP-15] a measure read is a place form whose exact type is `own u64`, so
/// the ordinary [TYPE-5] check applies at each use.
#[test]
fn a_measure_read_is_a_place_of_type_u64() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op15-pos-measure-read-is-a-place.wf"
    ));
    let mistyped =
        include_bytes!("../../../../tests/conformance/cases/op15-neg-measure-type-is-u64.wf");
    assert_rule_kind(mistyped, SemanticRule::Type5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

/// [WIN-3] a move out of a window slot is a hard error; the restructuring is
/// `take_back`, `remove_at`, or `swap`.
#[test]
fn a_move_out_of_a_window_slot_is_refused() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/win3-neg-move-out-of-window-slot.wf");
    assert_rule_kind(source, SemanticRule::Win3, |kind| {
        matches!(kind, SemanticIssueKind::MoveOutOfSlot { .. })
    });
}

/// [WIN-3] assigning over an owned place releases the old value when it is
/// affine and is a hard error at the target `place` when it is linear.
#[test]
fn assignment_releases_an_affine_old_value_and_refuses_a_linear_one() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/win3-pos-overwrite-releases-affine.wf"
    ));
    let linear =
        include_bytes!("../../../../tests/conformance/cases/win3-neg-overwrite-linear-place.wf");
    assert_rule_kind(linear, SemanticRule::Win3, |kind| {
        matches!(kind, SemanticIssueKind::LinearAssignmentTarget { .. })
    });
}

/// [WIN-3] a destructuring consume binds the fields it names and covers the
/// rest with `..`, which is the route a remaining linear part must take.
#[test]
fn a_destructuring_consume_takes_the_whole_owner() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/win3-pos-destructuring-consume.wf"
    ));
}

/// [OP-10] `place_back` fills the append slot and moves `r.len` up by one;
/// `take_back` empties the last filled slot and moves it back down.
#[test]
fn the_back_operations_move_the_boundary() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op10-pos-place-back-take-back.wf"
    ));
}

/// [OP-10] `insert_at`, `remove_at`, `append` and `split_off` each shift or
/// move a run of elements and move the boundary.
#[test]
fn the_shifting_operations_move_runs_and_boundaries() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op10-pos-insert-remove-append-split.wf"
    ));
}

/// [OP-10] `place_front` and `take_front` admit `Ring` alone and shift every
/// logical index.
#[test]
fn the_front_operations_admit_a_ring_alone() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op10-pos-front-operations-on-ring.wf"
    ));
}

/// [OP-10] `grow` is defined on `Box<Slots<T>>` alone and remakes the cell's
/// content whole, carrying [OP-9]'s obligation.
#[test]
fn grow_remakes_a_boxed_window() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op10-pos-boxed-window-grow.wf"
    ));
}

/// [OP-10] `place_back`'s `requires deref(window).len < deref(window).cap` is an ordinary
/// [FN-8] requirement, so a full window is refused at the call.
#[test]
fn place_back_on_a_full_window_is_an_undischarged_requirement() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/op10-neg-place-back-full-window.wf");
    assert_rule_kind(source, SemanticRule::Fn8, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallRequirement { .. })
    });
}

/// [OP-13] the nine construction functions are the only way to build a shape;
/// there is no `Type::name` spelling and no element-list literal in expression
/// position.
#[test]
fn the_construction_functions_build_every_shape() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op13-pos-construction-functions.wf"
    ));
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op13-pos-slots-from-and-into-array.wf"
    ));
}

/// [OP-9] each runtime-capacity construction carries the static
/// allocation-size obligation over its own count.
#[test]
fn a_runtime_capacity_construction_owes_the_size_obligation() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/op13-neg-runtime-capacity-size-obligation.wf"
    );
    assert_rule_kind(source, SemanticRule::Op9, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedAllocationFitObligation { .. }
        )
    });
}

/// [OP-14] `free_empty` consumes any window proved empty, an affine element
/// type and a linear one alike, and its boxed argument frees the cell with it.
#[test]
fn free_empty_consumes_a_proved_empty_window() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op14-pos-free-empty-empty-window.wf"
    ));
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op14-pos-free-empty-boxed-window.wf"
    ));
}

/// [OP-14] an undischarged `requires window.len == 0_u64` is a hard error at
/// the complete `call`, rendering the residual.
#[test]
fn free_empty_of_a_nonempty_window_is_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/op14-neg-free-empty-nonempty-window.wf"
    );
    assert_rule_kind(source, SemanticRule::Op14, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedEmptyRunRelease { .. })
    });
}

/// [OP-14] applies to both window shapes, static and runtime capacities, and
/// copy and linear elements. Direct static shapes and boxed runtime shapes are
/// admitted, and every admitted operand must have proved current length zero.
#[test]
fn free_empty_uses_the_current_length_for_every_window_shape() {
    for source in [
        br#"fn main() -> status: own ExitStatus pure {
  let window = box_slots_new::<u8>(capacity: 2_u64);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn main() -> status: own ExitStatus pure {
  let window = box_ring_new::<u8>(capacity: 2_u64);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn main() -> status: own ExitStatus pure {
  let slots = slots_new::<u8, 4>();
  free_empty(window: move slots);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn main() -> status: own ExitStatus pure {
  let ring = ring_new::<u8, 4>();
  free_empty(window: move ring);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"nodrop enum Ticket {
  Mark();
}

fn main() -> status: own ExitStatus pure {
  let window = box_slots_new::<Ticket>(capacity: 2_u64);
  let ticket = Mark();
  place_back(window: &window.inner, value: move ticket);
  let taken = take_back(window: &window.inner);
  match move taken {
    Mark() => {
      free_empty(window: move window);
      return exit_status(code: 0_u8);
    }
  }
}
"#
        .as_slice(),
    ] {
        assert_accepts(source);
    }

    for source in [
        br#"fn main() -> status: own ExitStatus pure {
  let window = box_slots_new::<u8>(capacity: 4_u64);
  place_back(window: &window.inner, value: 7_u8);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"nodrop struct Token {
  value: u8;
}

fn main() -> status: own ExitStatus pure {
  let window = box_slots_new::<Token>(capacity: 1_u64);
  let token = Token(value: 7_u8);
  place_back(window: &window.inner, value: move token);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn main() -> status: own ExitStatus pure {
  let window = box_ring_new::<Box<u8>>(capacity: 4_u64);
  let value = box_new::<u8>(value: 7_u8);
  place_back(window: &window.inner, value: move value);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn release(window: own Box<Slots<u8>>) -> result: own unit pure {
  free_empty(window: move window);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Op14, |kind| {
            matches!(kind, SemanticIssueKind::UndischargedEmptyRunRelease { .. })
        });
    }

    for source in [
        br#"fn main() -> status: own ExitStatus pure {
  let slots = slots_new::<u8, 4>();
  let window = box_new::<Slots<u8, 4>>(value: move slots);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"fn main() -> status: own ExitStatus pure {
  let ring = ring_new::<u8, 4>();
  let window = box_new::<Ring<u8, 4>>(value: move ring);
  free_empty(window: move window);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Op14, |kind| {
            matches!(kind, SemanticIssueKind::TypeMismatch { .. })
        });
    }
}

/// [TYPE-9] the three shapes have two placements each: constant capacity
/// inline in the owner, runtime capacity only as `Box` content.
#[test]
fn the_two_placements_are_admitted_where_the_rule_admits_them() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/type9-pos-constant-and-runtime-capacity-placement.wf"
    ));
}

/// [TYPE-9] a runtime-capacity form outside a `Box` is a hard error at the
/// complete `type`.
#[test]
fn a_runtime_capacity_shape_outside_a_box_is_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/type9-neg-runtime-capacity-outside-box.wf"
    );
    assert_rule_kind(source, SemanticRule::Type9, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

/// [TYPE-2] each shape is one of the prelude's opaque structs now, and an
/// opaque struct's constructor entry exists to be refused: a shape is built by
/// a construction function [OP-13], never by a constructor `call`.
#[test]
fn a_compiler_owned_constructor_call_is_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/type2-neg-storage-shape-constructor-call.wf"
    );
    assert_rule_kind(source, SemanticRule::Type2, |kind| {
        matches!(kind, SemanticIssueKind::ContainerConstruction { .. })
    });
}

/// [TYPE-2] a `Box` constructor `call` is refused in the same words, its
/// content being supplied by `box_new` and its friends.
#[test]
fn a_box_constructor_call_is_refused() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/type2-neg-box-constructor-call.wf");
    assert_rule_kind(source, SemanticRule::Type2, |kind| {
        matches!(kind, SemanticIssueKind::ContainerConstruction { .. })
    });
}

/// [TYPE-9] the content is reached by the ordinary field step `b.inner`, and
/// `let n = move b.inner;` consumes the `Box`, yields its content and frees
/// the cell.
#[test]
fn box_content_is_read_written_and_unboxed_through_its_field() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/type9-pos-box-content-read-write-unbox.wf"
    ));
}

/// [TYPE-9] a `move` of a runtime-capacity content is a hard error: the `Box`
/// must release it at scope exit, or the window must be emptied and the cell
/// consumed by `free_empty`.
#[test]
fn a_move_of_runtime_capacity_content_is_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/type9-neg-move-runtime-capacity-content.wf"
    );
    assert_rule_kind(source, SemanticRule::Type9, |kind| {
        matches!(kind, SemanticIssueKind::InlineRuntimeCapacityShape { .. })
    });
}

/// [STOR-8] a compilation unit carrying the no-heap declaration cannot name
/// `Box` or a runtime-capacity shape, and cannot call an allocating row.
#[test]
fn a_no_heap_unit_names_no_box_and_calls_no_allocating_row() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/stor8-pos-no-heap-program.wf"
    ));
    let named =
        include_bytes!("../../../../tests/conformance/cases/stor8-neg-no-heap-names-box.wf");
    assert_rule_kind(named, SemanticRule::Stor8, |kind| {
        matches!(kind, SemanticIssueKind::HeapTypeUnderNoHeap { .. })
    });
    let called = include_bytes!(
        "../../../../tests/conformance/cases/stor8-neg-no-heap-calls-allocating-row.wf"
    );
    assert_rule_kind(called, SemanticRule::Stor8, |kind| {
        matches!(kind, SemanticIssueKind::HeapTypeUnderNoHeap { .. })
    });
}

/// [STOR-8] allocation is total in the source: it never returns a failure and
/// no construction carries a `Result`.
#[test]
fn allocation_is_total() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/stor8-pos-allocation-is-total.wf"
    ));
}

/// [OP-11] `swap` exchanges the values at two owned places of one type,
/// consumes neither root, and admits its two arguments naming one place.
#[test]
fn swap_exchanges_two_owned_places() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op11-pos-swap-exchanges-values.wf"
    ));
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/op11-pos-swap-same-place-admission.wf"
    ));
}

/// [OP-11] a `swap` over a copy place is a hard error at the first
/// `borrow_expr`; the restructuring is to read the two values and assign them
/// back.
#[test]
fn swap_over_a_copy_place_is_refused() {
    let source = include_bytes!("../../../../tests/conformance/cases/op11-neg-swap-copy-place.wf");
    assert_rule_kind(source, SemanticRule::Op11, |kind| {
        matches!(kind, SemanticIssueKind::SwapOverCopyPlace { .. })
    });
}
