//! Extraction locks over the [ENT-3] fact-source envelope.
//!
//! The twelve L0 fact sources were markdown bullets with no id syntax for
//! several versions: cited seventy-one times across task records, nameable by
//! nothing, and restated in English inside the compiler because there was no id
//! to point at. v0.30 gave them the sub-rule ids `[ENT-3.S1]`…`[ENT-3.S12]`
//! plus a machine-readable `retired: S8` line. This module is what makes those
//! ids load-bearing rather than decorative: it locks the defined label set, the
//! retirement, and the per-view source split against `FlowEventKind` and
//! `ProofView`.
//!
//! A green run establishes that the compiler models exactly the sources the
//! specification defines, that a retired label is not silently reused, and that
//! the view definitions and the compiler's three views agree on which sources
//! each view removes. It does not establish that any source establishes the
//! right facts — the flow tests in `entailment.rs` cover that — and it does not
//! re-derive the behavioural consequence of the unasserted split, which
//! `strict::an_outside_call_does_not_demand_its_actual_expression_obligations_in_u`
//! already gates by compiling a program whose only support is an S2 check.

use std::collections::BTreeSet;

use super::super::entailment::{FlowEventKind, ProofView};

/// The body of [ENT-3], from its own rule id to the next rule definition.
fn ent3_body() -> &'static str {
    let after = crate::ACTIVE_KERNEL_SPEC_TEXT
        .split_once("\n[ENT-3] ")
        .expect("the active specification defines ENT-3")
        .1;
    after
        .split_once("\n[ENT-4]")
        .expect("ENT-3 is followed by ENT-4")
        .0
}

/// Every `[ENT-3.Sk]` sub-rule label defined at a line start, as its number.
fn defined_sources() -> BTreeSet<u8> {
    ent3_body()
        .lines()
        .filter_map(|line| line.strip_prefix("[ENT-3.S"))
        .map(|rest| {
            rest.strip_suffix(']')
                .expect("an ENT-3 sub-rule label is closed on its own line")
                .parse()
                .expect("an ENT-3 sub-rule label ends in its source number")
        })
        .collect()
}

/// Every label the `retired:` envelope key withdraws.
fn retired_sources() -> BTreeSet<u8> {
    ent3_body()
        .lines()
        .filter_map(|line| line.strip_prefix("retired: "))
        .flat_map(|list| list.split(", "))
        .map(|label| {
            label
                .strip_prefix('S')
                .expect("a retired label is S-prefixed")
                .parse()
                .expect("a retired label ends in its source number")
        })
        .collect()
}

/// The source number of one flow event, or `None` for a synthetic event.
///
/// Exhaustive with no wildcard: a new event kind is a compile error here, so it
/// cannot be added without deciding whether it is an ENT-3 source.
const fn event_source(kind: FlowEventKind) -> Option<u8> {
    match kind {
        FlowEventKind::S1 => Some(1),
        FlowEventKind::S3 => Some(3),
        FlowEventKind::S4 => Some(4),
        FlowEventKind::S5 => Some(5),
        FlowEventKind::S6 => Some(6),
        FlowEventKind::S7 => Some(7),
        FlowEventKind::S9 => Some(9),
        FlowEventKind::S10 => Some(10),
        FlowEventKind::S11 => Some(11),
        // S12 has no event kind of its own: FN-9 publishes a verified callee
        // summary through the postcondition group below, under a formula whose
        // establishment point is a call rather than a statement.
        FlowEventKind::Join
        | FlowEventKind::Snapshot
        | FlowEventKind::PostconditionEntryImageInvalidation
        | FlowEventKind::PostconditionCallConsume
        | FlowEventKind::PostconditionCallWrite
        | FlowEventKind::PostconditionReceiverWrite
        | FlowEventKind::PostconditionGive
        | FlowEventKind::PostconditionDeliveryJoin => None,
    }
}

const FLOW_EVENT_KINDS: [FlowEventKind; 17] = [
    FlowEventKind::S1,
    FlowEventKind::S3,
    FlowEventKind::S4,
    FlowEventKind::S5,
    FlowEventKind::S6,
    FlowEventKind::S7,
    FlowEventKind::S9,
    FlowEventKind::S10,
    FlowEventKind::S11,
    FlowEventKind::Join,
    FlowEventKind::Snapshot,
    FlowEventKind::PostconditionEntryImageInvalidation,
    FlowEventKind::PostconditionCallConsume,
    FlowEventKind::PostconditionCallWrite,
    FlowEventKind::PostconditionReceiverWrite,
    FlowEventKind::PostconditionGive,
    FlowEventKind::PostconditionDeliveryJoin,
];

const PROOF_VIEWS: [ProofView; 3] = [
    ProofView::Complete,
    ProofView::Unasserted,
    ProofView::S4Blinded,
];

/// The spelling [FN-1] writes for each view, exhaustively.
const fn view_spelling(view: ProofView) -> &'static str {
    match view {
        ProofView::Complete => "complete",
        ProofView::Unasserted => "unasserted",
        ProofView::S4Blinded => "S4-blinded",
    }
}

/// The source number S12 occupies. It is defined by the specification and
/// modelled by the postcondition event group rather than by an `Sk` variant.
const POSTCONDITION_SOURCE: u8 = 12;

/// Every source the specification defines has an `Sk` event kind, and the one
/// exception is the one the specification itself describes differently.
#[test]
fn the_ent3_sub_rules_and_the_compilers_flow_events_name_the_same_sources() {
    let defined = defined_sources();
    assert_eq!(
        defined,
        BTreeSet::from([1, 3, 4, 5, 6, 7, 9, 10, 11, 12]),
        "ENT-3 defines ten sources; S8 and S2 are retired"
    );

    let modelled: BTreeSet<u8> = FLOW_EVENT_KINDS
        .into_iter()
        .filter_map(event_source)
        .collect();
    assert_eq!(
        modelled.len(),
        FLOW_EVENT_KINDS
            .into_iter()
            .filter(|kind| event_source(*kind).is_some())
            .count(),
        "two event kinds claim one source number"
    );

    let mut expected = defined.clone();
    assert!(
        expected.remove(&POSTCONDITION_SOURCE),
        "S12 must be a defined source"
    );
    assert_eq!(
        modelled, expected,
        "the ENT-3 sub-rules and the Sk flow events disagree"
    );

    // S12's absence from the Sk variants is deliberate, so the group that does
    // carry it must be present rather than empty: an empty postcondition group
    // would satisfy the equality above while modelling nothing.
    assert_eq!(
        FLOW_EVENT_KINDS
            .into_iter()
            .filter(|kind| event_source(*kind).is_none())
            .count(),
        8
    );
}

/// A retired label is withdrawn, not reused.
#[test]
fn the_retired_source_label_is_never_a_defined_sub_rule() {
    let retired = retired_sources();
    assert_eq!(retired, BTreeSet::from([8]));
    let defined = defined_sources();
    assert!(
        retired.is_disjoint(&defined),
        "a retired label is redefined as a sub-rule"
    );
    assert!(
        FLOW_EVENT_KINDS
            .into_iter()
            .filter_map(event_source)
            .all(|source| !retired.contains(&source)),
        "the compiler models a retired source"
    );
    // The retirement paragraph states the same thing in prose; both must be
    // present, because the key alone would let the prose be deleted and the
    // prose alone is what the key replaced.
    assert!(
        ent3_body().contains("The label S8 is retired, not reused"),
        "ENT-3 states its retirement in prose as well as in the envelope key"
    );
}

/// The unasserted and S4-blinded view definitions partition the sources, and
/// the compiler has exactly the three views they define.
#[test]
fn the_view_definitions_and_the_compilers_proof_views_agree() {
    let spec = crate::ACTIVE_KERNEL_SPEC_TEXT;

    // The three views [FN-1] names, in the order it writes them, are exactly
    // the compiler's three; a fourth variant or a renamed one fails here.
    let written = spec
        .split_once(" dispositions")
        .expect("FN-1 names the view dispositions")
        .0;
    let written = &written[written.len() - "complete/unasserted/S4-blinded".len()..];
    assert_eq!(
        written,
        PROOF_VIEWS
            .into_iter()
            .map(view_spelling)
            .collect::<Vec<_>>()
            .join("/"),
        "FN-1's view vocabulary and ProofView disagree"
    );
    assert!(spec.contains("The **unasserted state** U is that flow recomputed"));
    assert!(spec.contains("The **S4-blinded state** B is U with"));

    let removal = spec
        .split_once("The unasserted state removes exactly ")
        .expect("the specification states the unasserted removal exactly")
        .1
        .split_once('\n')
        .expect("that sentence ends")
        .0;
    let removed = source_numbers(removal);
    assert_eq!(
        removed,
        BTreeSet::from([3]),
        "the unasserted state removes exactly S3"
    );

    let retained = source_numbers(
        spec.split_once("The unasserted state removes exactly S3 claim establishment.\n")
            .expect("the removal sentence is followed by the retention sentence")
            .1
            .split_once('\n')
            .expect("that sentence ends")
            .0,
    );

    let mut sources = defined_sources();
    assert!(sources.remove(&POSTCONDITION_SOURCE));
    assert!(
        removed.is_disjoint(&retained),
        "a source is both removed from and retained in U"
    );
    assert_eq!(
        removed.union(&retained).copied().collect::<BTreeSet<u8>>(),
        sources,
        "the two view sentences do not account for every flow source"
    );

    // B is U plus S4's own blinding, so exactly one further source moves.
    let blinding = spec
        .split_once("The **S4-blinded state** B is U with ")
        .expect("the specification defines B")
        .1
        .split_once('\n')
        .expect("that sentence ends")
        .0;
    assert!(
        blinding.contains("S4 goal"),
        "B is defined by blinding S4, not by removing another source"
    );
    assert!(retained.contains(&4), "S4 is retained in U itself");
}

/// Every `Sk` mentioned in one sentence, as numbers.
fn source_numbers(sentence: &str) -> BTreeSet<u8> {
    let bytes = sentence.as_bytes();
    let mut numbers = BTreeSet::new();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'S' {
            continue;
        }
        // A source mention starts a word, so `S` after a letter or digit — as
        // in `SCC` prefixes or `ENT-3.S1` cross-references — is not one.
        if index > 0 && bytes[index - 1].is_ascii_alphanumeric() {
            continue;
        }
        let digits: String = bytes[index + 1..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .map(|byte| char::from(*byte))
            .collect();
        if let Ok(number) = digits.parse::<u8>() {
            numbers.insert(number);
        }
    }
    numbers
}
