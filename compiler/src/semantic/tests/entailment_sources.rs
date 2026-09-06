//! Extraction locks over the [ENT-3] fact-source envelope.
//!
//! These tests connect the specification's source labels to the constructors
//! that can actually add facts to the originating semantic flow. They do not
//! replay a second proof view or decide acceptance.

use std::collections::BTreeSet;

use super::super::entailment::FlowEventKind;

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

/// Every `[ENT-3.Sk]` sub-rule label defined at a line start.
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

/// The source number of one fact-establishing event. Synthetic diagnostic and
/// invalidation events return `None` and never add a fact by themselves.
const fn fact_event_source(kind: FlowEventKind) -> Option<u8> {
    match kind {
        FlowEventKind::S1 => Some(1),
        FlowEventKind::S4 => Some(4),
        FlowEventKind::S5 => Some(5),
        FlowEventKind::S6 => Some(6),
        FlowEventKind::S7 => Some(7),
        FlowEventKind::S9 => Some(9),
        FlowEventKind::S10 => Some(10),
        FlowEventKind::S11 => Some(11),
        FlowEventKind::S13 => Some(13),
        FlowEventKind::S14 => Some(14),
        _ => None,
    }
}

const FACT_EVENT_KINDS: [FlowEventKind; 10] = [
    FlowEventKind::S1,
    FlowEventKind::S4,
    FlowEventKind::S5,
    FlowEventKind::S6,
    FlowEventKind::S7,
    FlowEventKind::S9,
    FlowEventKind::S10,
    FlowEventKind::S11,
    FlowEventKind::S13,
    FlowEventKind::S14,
];

/// S12 is published from an already verified FN-9 summary at the call transfer,
/// so it deliberately has no `FlowEventKind::S12` statement event.
const POSTCONDITION_SOURCE: u8 = 12;

#[test]
fn ent3_labels_and_fact_event_constructors_name_the_same_sources() {
    let defined = defined_sources();
    assert_eq!(
        defined,
        BTreeSet::from([1, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14]),
        "ENT-3 defines the eleven originating fact sources"
    );

    let modelled = FACT_EVENT_KINDS
        .into_iter()
        .filter_map(fact_event_source)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        modelled.len(),
        FACT_EVENT_KINDS.len(),
        "two fact event constructors must not reuse one source number"
    );

    let mut expected = defined;
    assert!(
        expected.remove(&POSTCONDITION_SOURCE),
        "S12 must be the separately published FN-9 source"
    );
    assert_eq!(
        modelled, expected,
        "ENT-3 labels and originating flow constructors disagree"
    );
}

#[test]
fn retired_fact_source_labels_are_never_reused() {
    let retired = retired_sources();
    assert_eq!(
        retired,
        BTreeSet::from([8]),
        "S8 remains the sole reserved fact-source label"
    );
    let defined = defined_sources();
    assert!(
        retired.is_disjoint(&defined),
        "a retired fact-source label is defined again"
    );
    assert!(
        FACT_EVENT_KINDS
            .into_iter()
            .filter_map(fact_event_source)
            .all(|source| !retired.contains(&source)),
        "an originating fact constructor uses a retired source label"
    );
}
