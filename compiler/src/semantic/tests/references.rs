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

use crate::{SemanticIssueKind, SemanticRule};

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
            SemanticIssueKind::OverlappingCallEffects { first, second, .. }
                if first == second
        )
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
