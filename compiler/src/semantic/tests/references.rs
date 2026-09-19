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

/// [REF-2] writing the storage at a reference's own path, or below it, is a
/// content write and invalidates nothing. This is the half of the lattice a
/// conservative implementation would get wrong by invalidating on every write.
#[test]
fn a_content_write_at_or_below_the_path_keeps_the_reference_valid() {
    assert_accepts(include_bytes!(
        "../../../../tests/conformance/cases/ref2-pos-content-write-keeps-validity.wf"
    ));
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
