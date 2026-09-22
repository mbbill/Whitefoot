//! [REF-1] through [REF-3]: a reference is a local name for a path, its
//! validity is a fact, and it never escapes.
//!
//! This module replaces `tests/borrows.rs` (33 tests), whose whole subject was
//! v0.59's borrow apparatus. Every rule it exercised is retired in v0.60 and
//! each retirement has a named successor:
//!
//! - [OWN-2] borrow modes and [OWN-5] the loan-conflict matrix retire: there
//!   is one reference kind, `&p`, with no permission marker, and whether a
//!   callee may write through a reference parameter is stated by its effect
//!   row alone. The successor is [EFF-5]'s pairwise comparison of substituted
//!   paths, exercised in `two_overlapping_substituted_writes_are_refused` below.
//! - [OWN-6] child reborrows and [OWN-14] the admitted reborrow forms retire:
//!   a reference names a resolved path, and a further step below it is another
//!   path, not a derived loan. The successor is [REF-1]'s path resolution.
//! - [OWN-9] holder suspension and [OWN-12] the suspended-parent rules retire.
//!   The successor is [REF-2]: validity is a fact, invalidated by the seven
//!   enumerated events and re-established only by forming the reference again.
//! - [OWN-3], [OWN-4], [OWN-10] and [FORM-8] retire with regions, which v0.60
//!   does not have at all; there is no successor and no replacement spelling.
//! - [VIEW-6]'s slice return ceiling retires; the successor is [REF-3]'s
//!   no-escape refusal at the `return_stmt`, exercised in
//!   `a_returned_reference_is_an_escape` below.
//!
//! The sources are the already-ported v0.60 conformance cases, which fix the
//! exact spelling of each judgment; these tests add the rule and issue kind the
//! corpus manifest does not pin.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_accepts, assert_rule_kind};

/// [REF-1] a reference variable names a path and is not storage of its own, so
/// `&p` where `p` is a reference variable has no path to name that `p` does not
/// already name.
#[test]
fn a_reference_to_a_reference_variable_is_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/ref1-neg-reference-to-reference-variable.wf"
    );
    assert_rule_kind(source, SemanticRule::Ref1, |kind| {
        matches!(kind, SemanticIssueKind::ReferenceToReferenceVariable { .. })
    });
}

/// [REF-2] a proper prefix of the path being written invalidates the
/// reference, and the use afterwards carries the invalidating event.
#[test]
fn a_prefix_write_invalidates_the_reference_at_its_next_use() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/ref2-neg-use-after-prefix-write.wf");
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(
            kind,
            SemanticIssueKind::InvalidReferenceUse { event, .. }
                if event.contains("proper prefix")
        )
    });
}

/// [REF-2] a move never re-roots an existing reference: after `let w = move v;`
/// the references formed from `v` are invalid and are not reinterpreted as
/// references into `w`.
#[test]
fn a_move_of_the_root_does_not_re_root_its_references() {
    let source = include_bytes!("../../../../tests/conformance/cases/ref2-neg-use-after-move.wf");
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// [REF-2] compares evaluated index steps. Replacing the same dynamically
/// indexed owner is a proper-prefix write even though the step came from a
/// binding rather than a literal.
#[test]
fn a_same_dynamic_index_prefix_replacement_invalidates_the_reference() {
    let source = br#"nocopy struct Record {
  value: u8;
}

fn main() -> status: own ExitStatus pure {
  let table = slots_new::<Record, 1>();
  let record = Record(value: 7_u8);
  place_back(window: &table, value: move record);
  let index = 0_u64;
  let p = &table[index].value;
  set table[index] = Record(value: 9_u8);
  return exit_status(code: deref(p));
}
"#;
    assert_rule_kind(
        source,
        SemanticRule::Ref2,
        |kind| matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. } if event.contains("proper prefix")),
    );
}

/// Consuming a whole nocopy owner invalidates every reference rooted there.
#[test]
fn a_whole_nocopy_owner_move_invalidates_its_reference() {
    let source = br#"nocopy struct Record {
  value: u8;
}

fn main() -> status: own ExitStatus pure {
  let record = Record(value: 7_u8);
  let p = &record;
  let moved = move record;
  return exit_status(code: deref(p).value);
}
"#;
    assert_rule_kind(
        source,
        SemanticRule::Ref2,
        |kind| matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. } if event.contains("moved out of")),
    );
}

/// Moving a Box's content consumes the Box root, so a reference to that root
/// cannot survive the unboxing operation.
#[test]
fn consuming_box_content_invalidates_a_reference_to_the_whole_box() {
    let source = br#"nocopy struct Record {
  value: u8;
}

fn main() -> status: own ExitStatus pure {
  let record = Record(value: 7_u8);
  let boxed = box_new::<Record>(value: move record);
  let p = &boxed;
  let extracted = move boxed.inner;
  return exit_status(code: deref(p).inner.value);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// [REF-2] writing the storage at a reference's own path, or below it, is a
/// content write and invalidates nothing. This is the half of the lattice a
/// conservative implementation would get wrong by invalidating on every write.
#[test]
fn a_content_write_at_or_below_the_path_keeps_the_reference_valid() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref2-pos-content-write-keeps-validity.wf"
    ));
}

/// The invalidation boundary is precise: writing the referenced content and
/// replacing a statically distinct indexed owner both preserve the reference.
#[test]
fn exact_content_and_distinct_literal_index_writes_preserve_references() {
    assert_accepts(
        br#"nocopy struct Record {
  value: u8;
}

fn main() -> status: own ExitStatus pure {
  let table = slots_new::<Record, 2>();
  let first = Record(value: 7_u8);
  place_back(window: &table, value: move first);
  let second = Record(value: 8_u8);
  place_back(window: &table, value: move second);
  let p = &table[0_u64].value;
  set table[0_u64].value = 9_u8;
  set table[1_u64] = Record(value: 4_u8);
  return exit_status(code: deref(p));
}
"#,
    );
}

/// [REF-3] a `return_stmt` whose selected expression is a reference is the
/// escape violation, and [FN-1] forms no candidate there.
#[test]
fn a_returned_reference_is_an_escape() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/ref3-neg-returned-reference.wf");
    assert_rule_kind(source, SemanticRule::Ref3, |kind| {
        matches!(kind, SemanticIssueKind::EscapingReference { .. })
    });
}

/// [REF-3] the restructuring the rule states — return an index and let the
/// caller form the reference — is accepted.
#[test]
fn returning_an_index_instead_of_a_reference_is_admitted() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref3-pos-index-result.wf"
    ));
}

/// [EFF-5] clause 1: two substituted effects on overlapping paths where at
/// least one is a write must be proved disjoint; passing one place as both
/// actuals is the refusal at the complete `call`.
#[test]
fn two_overlapping_substituted_writes_are_refused() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/eff5-neg-overlapping-substituted-write.wf"
    );
    assert_rule_kind(source, SemanticRule::Eff5, |kind| {
        matches!(kind, SemanticIssueKind::OverlappingCallEffects { .. })
    });
}

/// [EFF-5] compares every pair of declared effects after substitution, even
/// when one reference parameter supplies all of them.
#[test]
fn one_actual_cannot_supply_overlapping_declared_effects() {
    for source in [
        br#"struct Pair {
  first: u8;
  second: u8;
}

fn act(pair: &Pair) -> result: own unit reads(pair.first), writes(pair) {
  let old = deref(pair).first;
  set deref(pair).second = old;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let pair = Pair(first: 1_u8, second: 2_u8);
  act(pair: &pair);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"struct Pair {
  first: u8;
  second: u8;
}

fn act(pair: &Pair) -> result: own unit writes(pair), writes(pair.first) {
  set deref(pair).first = 7_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let pair = Pair(first: 1_u8, second: 2_u8);
  act(pair: &pair);
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        assert_rule_kind(source, SemanticRule::Eff5, |kind| {
            matches!(kind, SemanticIssueKind::OverlappingCallEffects { .. })
        });
    }
}

/// [EFF-5] substitutes the scalar values of index actuals, not the storage
/// paths from which those values were read. Distinct array elements can both
/// contain zero and therefore select the same written window element.
#[test]
fn substituted_index_values_do_not_inherit_their_actual_storage_separation() {
    let source = br#"fn write_two(window: &Slots<u8, 2>, first: own u64, second: own u64) -> result: own unit reads(window.len), writes(window[first]), writes(window[second]) {
  let length = deref(window).len;
  if first < length {
    if second < length {
      set deref(window)[first] = 1_u8;
      set deref(window)[second] = 2_u8;
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let indices = array_filled::<u64, 2>(value: 0_u64);
  let window = slots_new::<u8, 2>();
  place_back(window: &window, value: 7_u8);
  write_two(window: &window, first: indices[0_u64], second: indices[1_u64]);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff5, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedCallSeparation { residual, .. }
                if residual.contains("captured indices to be distinct")
        )
    });
}

/// [EFF-5] may discharge two indexed writes from the values captured for this
/// call. The strict guard dominates the call, so its disequality is available
/// for the two occurrence-specific actual captures.
#[test]
fn indexed_call_separation_accepts_strict_orderings_and_disequality() {
    let source = br#"fn write_two(window: &Slots<u8, 2>, first: own u64, second: own u64) -> result: own unit reads(window.len), writes(window[first]), writes(window[second]) {
  let length = deref(window).len;
  if first < length {
    if second < length {
      set deref(window)[first] = 1_u8;
      set deref(window)[second] = 2_u8;
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let window = slots_new::<u8, 2>();
  place_back(window: &window, value: 7_u8);
  let i = 0_u64;
  let j = 1_u64;
  if i < j {
    write_two(window: &window, first: i, second: j);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

#[test]
fn indexed_call_separation_does_not_retarget_captured_bindings() {
    let source = br#"fn write_refs(first: &u8, second: &u8) -> result: own unit writes(first), writes(second) {
  set deref(first) = 1_u8;
  set deref(second) = 2_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u8, 2>(value: 0_u8);
  let i = 0_u64;
  let j = 0_u64;
  let first = &values[i];
  let second = &values[j];
  set j = 1_u64;
  if i < j {
    write_refs(first: first, second: second);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff5, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallSeparation { residual, .. }
            if residual.contains("captured indices to be distinct"))
    });
}

#[test]
fn indexed_call_separation_keeps_formation_proof_after_source_writes() {
    let source = br#"fn write_refs(first: &u8, second: &u8) -> result: own unit writes(first), writes(second) {
  set deref(first) = 1_u8;
  set deref(second) = 2_u8;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u8, 2>(value: 0_u8);
  let i = 0_u64;
  let j = 1_u64;
  if i < j {
    let first = &values[i];
    let second = &values[j];
    set j = 0_u64;
    write_refs(first: first, second: second);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

#[test]
fn indexed_call_separation_is_unique_per_call_actual() {
    let source = br#"fn write_two(window: &Slots<u8, 2>, first: own u64, second: own u64) -> result: own unit reads(window.len), writes(window[first]), writes(window[second]) {
  let length = deref(window).len;
  if first < length {
    if second < length {
      set deref(window)[first] = 1_u8;
      set deref(window)[second] = 2_u8;
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let window = slots_new::<u8, 2>();
  place_back(window: &window, value: 7_u8);
  let i = 0_u64;
  let j = 1_u64;
  if i < j {
    write_two(window: &window, first: i, second: j);
  }
  set j = 0_u64;
  write_two(window: &window, first: i, second: j);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff5, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallSeparation { .. })
    });
}

/// [EFF-5] clause 3: a live reference outside the call whose path has a proper
/// prefix among the call's substituted write paths becomes invalid after the
/// call. The citation is [REF-2]'s, because the rejection is at the later use.
#[test]
fn a_call_write_invalidates_the_bystander_reference() {
    let source = include_bytes!(
        "../../../../tests/conformance/cases/eff5-neg-bystander-invalidated-by-call.wf"
    );
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// [EFF-5] substituted paths that are disjoint by different roots are
/// admitted, which is the acceptance half the pairwise comparison must keep.
#[test]
fn disjoint_substituted_paths_are_admitted() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/eff5-pos-substituted-paths-disjoint.wf"
    ));
}

/// [EFF-5] each IDENT index or range endpoint of a declared row takes the value
/// its own argument supplies, evaluated once at the call.
#[test]
fn an_index_endpoint_takes_its_own_arguments_value() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/eff5-pos-index-endpoint-substitution.wf"
    ));
}

/// [TYPE-7] there is no implicit read through a reference: a reference binding
/// used where a value of its referent type is expected is a hard error whose
/// mechanical fix is `deref(.)`.
#[test]
fn reading_through_a_reference_is_explicit() {
    let source = include_bytes!("../../../../tests/conformance/cases/type7-neg-implicit-read.wf");
    assert_rule_kind(source, SemanticRule::Type7, |kind| {
        matches!(kind, SemanticIssueKind::MissingDereference { .. })
    });
}

/// [TYPE-7] `deref` takes a reference and nothing else. A `Box` is not a
/// reference: its content is the field `inner` [TYPE-9], which is what the
/// restructuring says.
#[test]
fn deref_of_a_box_binding_names_no_referent() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/type7-neg-deref-box-binding.wf");
    assert_rule_kind(source, SemanticRule::Type7, |kind| {
        matches!(kind, SemanticIssueKind::MissingDereference { .. })
    });
}

/// [TYPE-7] `deref` of an owned place that is not a reference at all.
#[test]
fn deref_of_a_non_reference_is_refused() {
    let source = include_bytes!("../../../../tests/conformance/cases/type7-neg-deref-nonref.wf");
    assert_rule_kind(source, SemanticRule::Type7, |kind| {
        matches!(kind, SemanticIssueKind::MissingDereference { .. })
    });
}

/// [REF-1] an index expression inside a path is evaluated when the reference is
/// formed; the path records that value, and later assignments to the variables
/// the expression used do not change it.
#[test]
fn an_index_inside_a_path_is_captured_at_formation() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref1-pos-index-captured-at-formation.wf"
    ));
}

/// [REF-1] at a control-flow join a reference variable's target is the union of
/// the path sets its incoming edges may name, and every check on it must hold
/// for every member of that set.
#[test]
fn a_join_takes_the_union_of_the_path_sets() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref1-pos-join-path-set.wf"
    ));
}

/// A direct write through a joined holder must be writable at every possible
/// target. The runtime address is singular, but the constant alternative
/// cannot disappear behind the writable local alternative [REF-1, CONST-2].
#[test]
fn a_joined_dereference_cannot_write_a_possible_constant_target() {
    let source = br#"const permanent: u64 = 1_u64;

fn examine(flag: own Bool) -> result: own unit pure {
  let spare = 0_u64;
  let selected = if flag {
    give &spare;
  } else {
    give &permanent;
  }
  set deref(selected) = 9_u64;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Const2, |kind| {
        matches!(kind, SemanticIssueKind::ImmutableSetTarget)
    });
}

/// [EFF-2] reads through a joined holder exhibit every formal-rooted member.
/// Declaring only one incoming path is narrower than the body access.
#[test]
fn a_joined_dereference_exhibits_every_possible_parameter_read() {
    let source = br#"fn choose(flag: own Bool, a: &u64, b: &u64) -> result: own u64 reads(a) {
  let selected = if flag {
    give a;
  } else {
    give b;
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
}

/// [REF-1, OP-12] two holders with the same singleton resolved path name the
/// same target. The old affine value may therefore enter the updating call
/// through either holder while the commit uses the other holder's runtime
/// address.
#[test]
fn singleton_reference_aliases_name_one_atomic_update_target() {
    let source = br#"nocopy struct Token {
  value: u64;
}

fn retain(old: own Token) -> result: own Token pure {
  return move old;
}

fn main() -> status: own ExitStatus pure {
  let token = Token(value: 7_u64);
  let target = &token;
  let alias = &token;
  set deref(target) = retain(old: move deref(alias));
  if deref(target).value == 7_u64 {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 1_u8);
}
"#;
    assert_accepts(source);
}

/// Sharing one possible member does not make a joined target exact. Selecting
/// that member would make the update atomic only on one runtime branch, so the
/// move through the singleton alias remains [OWN-1]'s refusal.
#[test]
fn an_overlapping_join_is_not_one_atomic_update_target() {
    let source = br#"nocopy struct Token {
  value: u64;
}

fn retain(old: own Token) -> result: own Token pure {
  return move old;
}

fn examine(flag: own Bool) -> result: own unit pure {
  let first = Token(value: 1_u64);
  let second = Token(value: 2_u64);
  let target = if flag {
    give &first;
  } else {
    give &second;
  }
  let alias = &first;
  set deref(target) = retain(old: move deref(alias));
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Own1, |kind| {
        matches!(kind, SemanticIssueKind::MoveThroughReference { .. })
    });
}

/// A call requirement over a joined reference must hold for every path the
/// reference may name. One target carrying the required value cannot hide the
/// other target's contradictory value.
#[test]
fn a_joined_reference_call_actual_checks_every_possible_target() {
    let source = br#"fn needs_one(value: &u64) -> result: own unit reads(value) contract {
  requires deref(value) == 1_u64;
} {
  let observed = deref(value);
  return unit;
}

fn examine(flag: own Bool) -> result: own unit pure {
  let a = 1_u64;
  let b = 0_u64;
  let p = if flag {
    give &a;
  } else {
    give &b;
  }
  let completed = needs_one(value: p);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Fn8, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallRequirement(_))
    });
}

/// Reborrowing a joined reference retains all of its possible write targets.
/// The call may zero `b`, so the earlier nonzero guard cannot authorize the
/// following exact division.
#[test]
fn a_reborrow_of_a_joined_reference_kills_facts_for_every_possible_target() {
    let source = br#"fn zero(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 0_u64;
  return unit;
}

fn examine(flag: own Bool) -> result: own u64 pure {
  let a = 1_u64;
  let b = 1_u64;
  let p = if flag {
    give &a;
  } else {
    give &b;
  }
  let q = &deref(p);
  if b != 0_u64 {
    zero(target: q);
    return 1_u64 / b;
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  let flag = False();
  let value = examine(flag: flag);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Op2, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedIntegerDomainObligation { .. }
        )
    });
}

/// A re-slice formed from a joined range keeps the union of its origins. A
/// proper-prefix replacement of either origin invalidates the derived range.
#[test]
fn a_reslice_of_a_joined_range_is_invalidated_by_either_origin_replacement() {
    let source = br#"fn examine(flag: own Bool) -> result: own u8 pure {
  let a = array_filled::<u8, 2>(value: 1_u8);
  let b = array_filled::<u8, 2>(value: 2_u8);
  let part = if flag {
    give &a[0_u64..2_u64];
  } else {
    give &b[0_u64..2_u64];
  }
  let first = &deref(part)[0_u64..1_u64];
  let replacement = array_filled::<u8, 2>(value: 3_u8);
  set b = replacement;
  return deref(first)[0_u64];
}

fn main() -> status: own ExitStatus pure {
  let flag = False();
  let value = examine(flag: flag);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// A reference delivered from an expression with a returning alternative has
/// the same write reachability as an ordinary reference. The write kills the
/// branch fact before the exact division is checked.
#[test]
fn a_delivered_reference_with_a_returning_alternative_kills_stale_facts() {
    let source = br#"fn zero(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 0_u64;
  return unit;
}

fn examine(flag: own u64) -> result: own u64 pure {
  let value = 1_u64;
  let p = if flag == 1_u64 {
    give &value;
  } else {
    return 0_u64;
  }
  if value != 0_u64 {
    zero(target: p);
    return 1_u64 / value;
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  let value = examine(flag: 1_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Op2, |_| true);
}
#[test]
fn a_written_reference_cannot_select_named_constant_storage() {
    let source = br#"const permanent: u64 = 1_u64;

fn overwrite(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 9_u64;
  return unit;
}

fn examine(flag: own Bool) -> result: own unit pure {
  let spare = 0_u64;
  let original = &permanent;
  let alias = original;
  let selected = if flag {
    give &spare;
  } else {
    give alias;
  }
  overwrite(target: selected);
  return unit;
}
"#;
    super::assert_rule_kind(source, SemanticRule::Const2, |kind| {
        matches!(kind, SemanticIssueKind::ImmutableWrittenArgument { binding, .. }
            if binding == "permanent")
    });
}

#[test]
fn a_constant_can_be_read_by_reference_and_copied_to_writable_storage() {
    let source = br#"const permanent: u64 = 1_u64;

fn observe(value: &u64) -> result: own u64 reads(value) {
  return deref(value);
}

fn overwrite(target: &u64) -> result: own unit writes(target) {
  set deref(target) = 9_u64;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let copied = observe(value: &permanent);
  overwrite(target: &copied);
  return exit_status(code: 0_u8);
}
"#;
    super::assert_accepts(source);
}

/// [REF-2] a selected payload remains in place after its selecting arm ends.
#[test]
fn a_selected_payload_reference_survives_arm_exit() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 reads(packet) {
  let fallback = 7_u64;
  let selected = &fallback;
  match deref(packet) {
    Data(value: payload) => {
      set selected = &deref(payload);
    }
    Idle() => {
    }
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// [REF-2] an `if` branch is a lexical scope just as a loop body is. A local
/// root borrowed into an outer reference dies on the branch edge.
#[test]
fn a_reference_to_an_if_local_dies_at_branch_exit() {
    let source = br#"fn examine(flag: own Bool) -> result: own u64 pure {
  let fallback = 7_u64;
  let selected = &fallback;
  if flag {
    let local = 9_u64;
    set selected = &local;
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. }
            if event.contains("scope"))
    });
}

/// [REF-1] a reference that does not cross a loop backedge may be rebound to
/// another path shape. The new path, kind, and referent type are retained.
#[test]
fn a_non_loop_reference_may_change_path_shape() {
    let source = br#"fn examine() -> result: own u64 pure {
  let first = 7_u64;
  let second = 9_u64;
  let selected = &first;
  set selected = &second;
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// Rebinding changes only the path. It cannot silently change the reference
/// variable's referent type while retaining the old checked type.
#[test]
fn a_reference_rebinding_keeps_its_referent_type() {
    let source = br#"fn examine() -> result: own u64 pure {
  let first = 7_u64;
  let second = 9_u8;
  let selected = &first;
  set selected = &second;
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Type5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

/// A single-place reference and a range reference are different reference
/// kinds even when both name the same element type.
#[test]
fn a_reference_rebinding_keeps_its_reference_kind() {
    let source = br#"fn examine() -> result: own u64 pure {
  let values = array_filled::<u64, 2>(value: 7_u64);
  let selected = &values[0_u64];
  set selected = &values[0_u64..1_u64];
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Type5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
}

/// A direct value-match delivery retains the selected address, without
/// exporting the selecting arm's variant fact.
#[test]
fn a_selected_payload_reference_survives_value_match_delivery() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 reads(packet) {
  let fallback = 7_u64;
  let selected = match deref(packet) {
    Data(value: payload) => {
      give &deref(payload);
    }
    Idle() => {
      give &fallback;
    }
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// A `give` crossing a nested match retains an existing selected payload.
#[test]
fn a_selected_payload_reference_survives_nested_give() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet, choose: own Bool) -> result: own u64 reads(packet) {
  let fallback = 7_u64;
  let selected = if choose {
    match deref(packet) {
      Data(value: payload) => {
        give &deref(payload);
      }
      Idle() => {
        give &fallback;
      }
    }
  } else {
    give &fallback;
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// A reference leaving on a break edge can select another root and payload.
#[test]
fn a_reference_on_a_break_edge_may_change_its_root() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 reads(packet) {
  let fallback = 7_u64;
  let selected = &fallback;
  loop @done {
    match deref(packet) {
      Data(value: payload) => {
        set selected = &deref(payload);
        break @done;
      }
      Idle() => {
        break @done;
      }
    }
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// [REF-1] a loop-carried reference keeps one static path shape while the
/// captured index may change on each backedge. The arbitrary header must
/// therefore admit the prior iteration's path instead of stopping at an
/// ownership-join capability limit.
#[test]
fn a_loop_carried_reference_may_change_its_captured_index() {
    let source = br#"fn inspect(values: &Array<u64, 3>) -> result: own u64 reads(values) {
  let selected = &deref(values)[0_u64];
  let result = 0_u64;
  for (i in 0_u64..3_u64) {
    let current = deref(selected);
    set result = result +wrap current;
    set selected = &deref(values)[i];
  }
  return result;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// A counted header has both its zero-trip preheader edge and every possible
/// backedge. The reference after the loop may consequently name the initial
/// element or the element selected by the last completed iteration; neither
/// edge may be dropped merely because the body has one syntactic rebinding.
#[test]
fn a_counted_reference_continuation_includes_zero_trip_and_backedges() {
    let source = br#"fn select(values: &Array<u64, 3>, count: own u64) -> result: own u64 reads(values) contract {
  requires count <= 2_u64;
} {
  let selected = &deref(values)[0_u64];
  for (i in 0_u64..count) {
    let next = i + 1_u64;
    set selected = &deref(values)[next];
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// [REF-2] the normal backedge can carry an invalid reference to the next
/// iteration. A use before that iteration reforms the reference is therefore
/// invalid even though the preheader supplied a valid path on the first trip.
#[test]
fn a_loop_head_use_observes_a_prior_iteration_window_invalidation() {
    let source =
        br#"fn inspect(owner: &Box<Slots<u64>>) -> result: own u64 writes(owner) contract {
  requires 0_u64 < deref(owner).inner.len;
  requires deref(owner).inner.cap <= 4_u64;
} {
  let selected = &deref(owner).inner[0_u64];
  let result = 0_u64;
  for (i in 0_u64..2_u64) {
    let current = deref(selected);
    set result = result +wrap current;
    let capacity = deref(owner).inner.cap;
    grow(cell: owner, capacity: capacity);
  }
  return result;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. }
            if event.contains("call wrote a proper prefix"))
    });
}

/// Reforming the reference before every use cuts the dependency on the
/// possibly invalid header value. The invalidation at the end of one
/// iteration may therefore flow to the next head without making this body
/// unsafe. A non-continuing invalidation likewise creates no future use.
#[test]
fn reforming_before_use_and_noncontinuing_invalidation_are_valid() {
    let source = br#"fn reform(owner: &Box<Slots<u64>>) -> result: own u64 writes(owner) contract {
  requires 0_u64 < deref(owner).inner.len;
  requires deref(owner).inner.cap <= 4_u64;
} {
  let selected = &deref(owner).inner[0_u64];
  let result = 0_u64;
  for (i in 0_u64..2_u64) {
    let nonempty = 0_u64 < deref(owner).inner.len;
    if nonempty {
      set selected = &deref(owner).inner[0_u64];
      let current = deref(selected);
      set result = result +wrap current;
      let capacity = deref(owner).inner.cap;
      let allocation_fits = capacity <= 4_u64;
      if allocation_fits {
        grow(cell: owner, capacity: capacity);
      }
    }
  }
  return result;
}

fn one_trip(owner: &Box<Slots<u64>>) -> result: own u64 writes(owner) contract {
  requires 0_u64 < deref(owner).inner.len;
  requires deref(owner).inner.cap <= 4_u64;
} {
  let selected = &deref(owner).inner[0_u64];
  loop @done {
    let current = deref(selected);
    let capacity = deref(owner).inner.cap;
    grow(cell: owner, capacity: capacity);
    break @done;
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// A break join includes the branch which never reforms the invalidated
/// reference, even when another branch selects the replacement payload.
#[test]
fn a_break_edge_with_an_unrepaired_invalid_reference_is_rejected() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 writes(packet) {
  match deref(packet) {
    Data(value: outer_payload) => {
      let selected = &deref(outer_payload);
      set deref(packet) = Data(value: 2_u64);
      loop @done {
        match deref(packet) {
          Data(value: inner_payload) => {
            set selected = &deref(inner_payload);
            break @done;
          }
          Idle() => {
            break @done;
          }
        }
      }
      return deref(selected);
    }
    Idle() => {
      return 0_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. }
            if event.contains("proper prefix"))
    });
}

/// [REF-1] a dereferenced joined scrutinee retains every possible enum root.
/// Replacing the second root in one arm must invalidate its payload binder;
/// choosing the first access as a representative would wrongly accept it.
#[test]
fn a_joined_match_scrutinee_keeps_every_payload_origin() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(choose: own Bool) -> result: own u64 pure {
  let first = Data(value: 1_u64);
  let second = Data(value: 2_u64);
  let selected = if choose {
    give &first;
  } else {
    give &second;
  }
  match deref(selected) {
    Data(value: payload) => {
      set second = Idle();
      return deref(payload);
    }
    Idle() => {
      return 0_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// The index operand is evaluated to select the matched range element, but
/// the scalar storage read while doing so is not another enum referent. A
/// later write to that scalar therefore leaves the payload reference valid.
#[test]
fn an_indexed_match_does_not_treat_index_storage_as_an_enum_origin() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine() -> result: own u64 pure {
  let seed = Data(value: 7_u64);
  let packets = array_filled::<Packet, 2>(value: seed);
  set packets[1_u64] = Data(value: 9_u64);
  let index = 1_u64;
  let part = &packets[0_u64..2_u64];
  if index < deref(part).len {
    match deref(part)[index] {
      Data(value: payload) => {
        set index = 0_u64;
        return deref(payload);
      }
      Idle() => {
        return 0_u64;
      }
    }
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// The selected indexed member remains the borrowed-match origin. Replacing
/// that enum invalidates its payload reference even though the index operand
/// itself came from separate scalar storage.
#[test]
fn an_indexed_match_keeps_the_selected_element_as_its_enum_origin() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine() -> result: own u64 pure {
  let seed = Data(value: 7_u64);
  let packets = array_filled::<Packet, 2>(value: seed);
  set packets[1_u64] = Data(value: 9_u64);
  let index = 1_u64;
  let part = &packets[0_u64..2_u64];
  if index < deref(part).len {
    match deref(part)[index] {
      Data(value: payload) => {
        set packets[1_u64] = Idle();
        return deref(payload);
      }
      Idle() => {
        return 0_u64;
      }
    }
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// Re-establishing an already active `(place, variant)` refinement in a
/// nested match does not end the enclosing arm's fact when the inner arm
/// exits.
#[test]
fn an_outer_payload_reference_survives_a_nested_identical_refinement() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 reads(packet), reads(packet.Data.value) {
  match deref(packet) {
    Data(value: outer_payload) => {
      let saved = &deref(outer_payload);
      match deref(packet) {
        Data(value: inner_payload) => {
          let observed = deref(inner_payload);
        }
        Idle() => {
        }
      }
      return deref(saved);
    }
    Idle() => {
      return 0_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// Replacing an enum ends the old selection. Selecting the new payload
/// establishes a new witness which can survive the inner match.
#[test]
fn a_new_selection_survives_the_replaced_outer_refinement() {
    let source = br#"enum Packet {
  Data(value: u64);
  Idle();
}

fn examine(packet: &Packet) -> result: own u64 writes(packet) {
  match deref(packet) {
    Data(value: outer_payload) => {
      let selected = &deref(outer_payload);
      set deref(packet) = Data(value: 2_u64);
      match deref(packet) {
        Data(value: inner_payload) => {
          set selected = &deref(inner_payload);
        }
        Idle() => {
          return 0_u64;
        }
      }
      return deref(selected);
    }
    Idle() => {
      return 0_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_accepts(source);
}

/// A loop body local leaves scope on a nested delivery edge just as it does
/// on the backedge and break edges. The enclosing value initializer cannot
/// publish a reference rooted at that local.
#[test]
fn a_loop_local_reference_cannot_escape_on_a_give_edge() {
    let source = br#"fn examine(flag: own Bool) -> result: own u64 pure {
  let fallback = 7_u64;
  let selected = if flag {
    loop @deliver {
      let local = 9_u64;
      give &local;
    }
    give &fallback;
  } else {
    give &fallback;
  }
  return deref(selected);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { event, .. }
            if event.contains("scope"))
    });
}

const INDEXED_CALL_HELPER: &str = r#"fn write_two(values: &Array<u8, 4>, first: own u64, second: own u64) -> result: own unit writes(values[first]), writes(values[second]) {
  if first < 4_u64 {
    if second < 4_u64 {
      set deref(values)[first] = 1_u8;
      set deref(values)[second] = 2_u8;
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

fn assert_indexed_call_proof(label: &str, source: &[u8], require_affine: bool) {
    super::with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("indexed call separation {label} must be accepted: {outcome:?}");
        };
        let mut found = false;
        for function in &program.data.functions {
            super::entailment::validate_derivations(&function.entailment);
            found |= function.entailment.obligations.iter().any(|outcome| {
                outcome.family == super::super::entailment::ObligationFamily::CallSeparation
                    && outcome.discharged
                    && outcome.derivation.is_some_and(|root| {
                        let Some(super::super::entailment::DerivationNode::IndexSeparation {
                            detail,
                        }) = function.entailment.derivations.nodes.get(root.0 as usize)
                        else {
                            return false;
                        };
                        !require_affine || detail.affine_target.is_some()
                    })
            });
        }
        assert!(
            found,
            "accepted source must retain one exact indexed call proof"
        );
    });
}

#[test]
fn indexed_call_separation_uses_runtime_order_and_disequality_facts() {
    for relation in ["i < j", "j < i", "i != j"] {
        let source = format!(
            "{INDEXED_CALL_HELPER}\nfn ordered(values: &Array<u8, 4>, i: own u64, j: own u64) -> result: own unit writes(values) {{\n  if {relation} {{\n    write_two(values: values, first: i, second: j);\n  }}\n  return unit;\n}}\n"
        );
        assert_indexed_call_proof(relation, source.as_bytes(), false);
    }
    let mixed = format!(
        "{INDEXED_CALL_HELPER}\nfn mixed(values: &Array<u8, 4>, j: own u64) -> result: own unit writes(values) {{\n  if 0_u64 < j {{\n    write_two(values: values, first: 0_u64, second: j);\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("mixed literal", mixed.as_bytes(), false);
    let affine = format!(
        "{INDEXED_CALL_HELPER}\nfn affine(values: &Array<u8, 4>, i: own u64, k: own u64) -> result: own unit writes(values) {{\n  if i < 3_u64 {{\n    if 0_u64 < k {{\n      if k < 3_u64 {{\n        let j = i + k;\n        write_two(values: values, first: i, second: j);\n      }}\n    }}\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("affine successor", affine.as_bytes(), true);
    for body in [
        "  write_two(values: values, first: i, second: j);\n",
        "  if i == j {\n    write_two(values: values, first: i, second: j);\n  }\n",
    ] {
        let source = format!(
            "{INDEXED_CALL_HELPER}\nfn refused(values: &Array<u8, 4>, i: own u64, j: own u64) -> result: own unit writes(values) {{\n{body}  return unit;\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Eff5, |kind| {
            matches!(kind, SemanticIssueKind::UndischargedCallSeparation { .. })
        });
    }
}

#[test]
fn indexed_call_separation_obeys_loop_backedges() {
    let first_visit_only = format!(
        "{INDEXED_CALL_HELPER}\nfn looped(values: &Array<u8, 4>, i: own u64, j: own u64, stop: own Bool) -> result: own unit writes(values) {{\n  if i < j {{\n  }} else {{\n    return unit;\n  }}\n  loop @again {{\n    write_two(values: values, first: i, second: j);\n    set j = i;\n    if stop {{\n      break @again;\n    }}\n  }}\n  return unit;\n}}\n"
    );
    assert_rule_kind(first_visit_only.as_bytes(), SemanticRule::Eff5, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallSeparation { .. })
    });
    let every_visit = format!(
        "{INDEXED_CALL_HELPER}\nfn looped(values: &Array<u8, 4>, i: own u64, j: own u64, stop: own Bool) -> result: own unit writes(values) {{\n  loop @again {{\n    if i < j {{\n      write_two(values: values, first: i, second: j);\n    }}\n    set j = i;\n    if stop {{\n      break @again;\n    }}\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("every loop visit", every_visit.as_bytes(), false);
}

#[test]
fn indexed_call_separation_requires_every_join_predecessor() {
    let one_arm = format!(
        "{INDEXED_CALL_HELPER}\nfn joined(values: &Array<u8, 4>, i: own u64, j: own u64, choose: own Bool) -> result: own unit writes(values) {{\n  if choose {{\n    let branch_marker = i;\n    if i < j {{\n      let observed = i;\n    }} else {{\n      return unit;\n    }}\n  }} else {{\n    let observed = j;\n  }}\n  write_two(values: values, first: i, second: j);\n  return unit;\n}}\n"
    );
    assert_rule_kind(one_arm.as_bytes(), SemanticRule::Eff5, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedCallSeparation { .. })
    });
    let both_arms = format!(
        "{INDEXED_CALL_HELPER}\nfn joined(values: &Array<u8, 4>, i: own u64, j: own u64, choose: own Bool) -> result: own unit writes(values) {{\n  if choose {{\n    let branch_marker = i;\n    if i < j {{\n      let observed = i;\n    }} else {{\n      return unit;\n    }}\n  }} else {{\n    let branch_marker = j;\n    if i < j {{\n      let observed = j;\n    }} else {{\n      return unit;\n    }}\n  }}\n  write_two(values: values, first: i, second: j);\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("both join arms", both_arms.as_bytes(), false);
}

#[test]
fn indexed_call_separation_uses_ordered_nested_candidates() {
    let helper = r#"fn write_nested(values: &Array<Array<u8, 4>, 4>, ao: own u64, ai: own u64, bo: own u64, bi: own u64) -> result: own unit writes(values[ao][ai]), writes(values[bo][bi]) {
  if ao < 4_u64 {
    if ai < 4_u64 {
      if bo < 4_u64 {
        if bi < 4_u64 {
          set deref(values)[ao][ai] = 1_u8;
          set deref(values)[bo][bi] = 2_u8;
        }
      }
    }
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let outer = format!(
        "{helper}\nfn outer(values: &Array<Array<u8, 4>, 4>, i: own u64, j: own u64, k: own u64) -> result: own unit writes(values) {{\n  if i < j {{\n    write_nested(values: values, ao: i, ai: k, bo: j, bi: k);\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("outer nested index", outer.as_bytes(), false);
    let inner = format!(
        "{helper}\nfn inner(values: &Array<Array<u8, 4>, 4>, i: own u64, j: own u64) -> result: own unit writes(values) {{\n  if i < j {{\n    write_nested(values: values, ao: 0_u64, ai: i, bo: 0_u64, bi: j);\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("inner after equal prefix", inner.as_bytes(), false);
    let later = format!(
        "{helper}\nfn later(values: &Array<Array<u8, 4>, 4>, i: own u64, j: own u64, k: own u64, l: own u64) -> result: own unit writes(values) {{\n  if k < l {{\n    write_nested(values: values, ao: i, ai: k, bo: j, bi: l);\n  }}\n  return unit;\n}}\n"
    );
    assert_indexed_call_proof("later nested candidate", later.as_bytes(), false);
}


const BYSTANDER_WRITE_HELPER: &str = r#"struct BystanderRecord {
  value: u8;
}

fn overwrite_bystander(values: &Array<BystanderRecord, 4>, index: own u64) -> result: own unit writes(values[index]) contract {
  requires index < 4_u64;
} {
  set deref(values)[index] = BystanderRecord(value: 9_u8);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

fn assert_bystander_preservation(source: &[u8]) {
    super::with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("bystander preservation must be accepted: {outcome:?}");
        };
        let mut found = false;
        for function in &program.data.functions {
            super::entailment::validate_derivations(&function.entailment);
            found |= function.entailment.obligations.iter().any(|outcome| {
                matches!(outcome.family,
                    super::super::entailment::ObligationFamily::ReferencePreservation(_))
                    && outcome.discharged
                    && outcome.derivation.is_some()
            });
        }
        assert!(found, "a used symbolic preservation retains its derivation");
    });
}

/// REF-2 uses OWN-7's current proof context for a destructive-prefix question,
/// including when the preserved reference is not an argument of the call.
#[test]
fn bystander_reference_survives_a_symbolically_disjoint_call_write() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires i < j;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  overwrite_bystander(values: values, index: j);
  return deref(selected);
}
"#
    );
    assert_bystander_preservation(source.as_bytes());
}

/// A dominating guard supplies the same separation as a requirement.
#[test]
fn bystander_reference_uses_a_dominating_separation_guard() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) {
  if j < 4_u64 {
    if i < j {
      let selected = &deref(values)[i].value;
      overwrite_bystander(values: values, index: j);
      return deref(selected);
    }
  }
  return 0_u8;
}
"#
    );
    assert_bystander_preservation(source.as_bytes());
}

/// Every member of a joined reference's target union must be preserved.
#[test]
fn bystander_reference_separation_checks_every_joined_target() {
    for (second_target, accepted) in [("k", true), ("j", false)] {
        let source = format!(
            "{BYSTANDER_WRITE_HELPER}\nfn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64, k: own u64, choose: own Bool) -> result: own u8 reads(values), writes(values[j]) contract {{\n  requires i < j;\n  requires k < j;\n  requires j < 4_u64;\n}} {{\n  let selected = &deref(values)[i].value;\n  if choose {{\n    set selected = &deref(values)[{second_target}].value;\n  }}\n  overwrite_bystander(values: values, index: j);\n  return deref(selected);\n}}\n"
        );
        if accepted {
            assert_bystander_preservation(source.as_bytes());
        } else {
            assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
                matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
            });
        }
    }
}

/// The selected index is its captured value, even when its source binding
/// changes before the helper call.
#[test]
fn bystander_reference_does_not_retarget_after_an_index_assignment() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires i < j;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  set i = j;
  overwrite_bystander(values: values, index: j);
  return deref(selected);
}
"#
    );
    assert_bystander_preservation(source.as_bytes());

    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires 0_u64 < j;
  requires j < 4_u64;
} {
  let i = j;
  let selected = &deref(values)[i].value;
  set i = 0_u64;
  overwrite_bystander(values: values, index: j);
  return deref(selected);
}
"#
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// A separation learned in a sibling branch cannot preserve a reference
/// across a write reached on both incoming edges.
#[test]
fn bystander_reference_separation_does_not_escape_a_guard() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires i < 4_u64;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  if i < j {
    let observed = i;
  }
  overwrite_bystander(values: values, index: j);
  return deref(selected);
}
"#
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// Invalidating a reference is legal; only its later use is refused.
#[test]
fn an_unused_bystander_does_not_require_call_write_separation() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own unit writes(values[j]) contract {
  requires i < 4_u64;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  overwrite_bystander(values: values, index: j);
  return unit;
}
"#
    );
    assert_accepts(source.as_bytes());
}


/// Separation must hold in the write's entering context; a later guard
/// cannot repair a validity fact that the write already removed.
#[test]
fn a_post_write_guard_cannot_restore_bystander_validity() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires i < 4_u64;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  overwrite_bystander(values: values, index: j);
  if i < j {
    return deref(selected);
  }
  return 0_u8;
}
"#
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}

/// A use before the write in source order still depends on prior iterations'
/// writes. Pending symbolic preservation must survive loop-token elimination.
#[test]
fn a_loop_header_use_demands_bystander_preservation_on_the_backedge() {
    for (order, accepted) in [("  requires i < j;\n", true), ("", false)] {
        let source = format!(
            "{BYSTANDER_WRITE_HELPER}\nfn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own unit reads(values), writes(values[j]) contract {{\n  requires i < 4_u64;\n  requires j < 4_u64;\n{order}}} {{\n  let selected = &deref(values)[i].value;\n  let alias = selected;\n  for (round in 0_u64..2_u64) {{\n    let observed = deref(alias);\n    overwrite_bystander(values: values, index: j);\n  }}\n  return unit;\n}}\n"
        );
        if accepted {
            assert_bystander_preservation(source.as_bytes());
        } else {
            assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
                matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
            });
        }
    }
}

/// Reforming after a write creates a new validity fact, but must not discard
/// an already demanded use of the earlier reference.
#[test]
fn later_reformation_does_not_cancel_a_bystander_use_obligation() {
    let source = format!(
        "{BYSTANDER_WRITE_HELPER}\n{}",
        r#"fn inspect(values: &Array<BystanderRecord, 4>, i: own u64, j: own u64) -> result: own u8 reads(values), writes(values[j]) contract {
  requires i < 4_u64;
  requires j < 4_u64;
} {
  let selected = &deref(values)[i].value;
  overwrite_bystander(values: values, index: j);
  let observed = deref(selected);
  set selected = &deref(values)[i].value;
  return observed;
}
"#
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Ref2, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReferenceUse { .. })
    });
}
