#![forbid(unsafe_code)]
//! Finite comparison of whole-result lowering and compiler-internal result trees.
//! Research evidence only: logical ownership and lowering events are explicit.

use std::collections::{BTreeMap, BTreeSet};

const LOGICAL_RECORD_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Field {
    Header,
    Payload,
}

impl Field {
    const DROP_ORDER: [Self; 2] = [Self::Payload, Self::Header];
    const fn index(self) -> usize {
        match self {
            Self::Header => 0,
            Self::Payload => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Token(u32);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Part {
    token: Token,
    value: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Record {
    header: Part,
    payload: Part,
}

impl Record {
    fn digest(&self) -> u64 {
        self.header.value.rotate_left(7) ^ self.payload.value.rotate_left(29)
    }
}

#[derive(Clone, Debug)]
enum Action {
    Effect(&'static str),
    ReleaseExisting(usize),
    Init {
        field: Field,
        token: Token,
        value: u64,
    },
}

#[derive(Clone, Debug)]
enum Terminal {
    Fresh,
    ForwardExisting(usize),
    Error(u64),
}

#[derive(Clone, Debug)]
struct Path {
    actions: Vec<Action>,
    terminal: Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResultForm {
    Total,
    Fallible,
}

#[derive(Clone, Debug)]
struct Producer {
    form: ResultForm,
    paths: Vec<Path>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Consumer {
    Place,
    ObserveWholeThenPlace,
}

#[derive(Clone, Debug)]
struct Invocation {
    label: &'static str,
    producer: Producer,
    selected_path: usize,
    consumer: Consumer,
    claim_direct: bool,
    sink: u8,
    existing: Vec<Record>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Outcome {
    Placed { sink: u8, record: Record },
    Failed { code: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Semantics {
    outcome: Outcome,
    effects: Vec<String>,
    releases: Vec<Token>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Cost {
    record_sized_whole_slots: usize,
    whole_record_transfers: usize,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Storage {
    Existing(usize),
    WholeResult(u8),
    Final(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Event {
    AllocateWhole(Storage),
    Effect(&'static str),
    ReleaseExisting(Storage),
    WriteField {
        destination: Storage,
        field: Field,
        part: Part,
    },
    TransferWhole {
        from: Storage,
        to: Storage,
    },
    ObserveWhole(Storage),
    Commit(Storage),
    Error(u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Report {
    semantics: Semantics,
    cost: Cost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Strategy {
    WholeResult,
    ResultTree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ModelError {
    BadPath(usize),
    ErrorInTotal,
    MissingField(Field),
    DuplicateField(Field),
    DuplicateToken(Token),
    DuplicateRelease(usize),
    OwnerAlreadyConsumed(Storage),
    LeakedExisting(usize),
    PartialBeforeForward,
    UnknownExisting(usize),
    UnknownStorage(Storage),
    StorageAlreadyAllocated(Storage),
    DestinationOccupied(Storage),
    UnreleasedOwnership(Storage),
    LeakedToken(Token),
    EventAfterTerminal,
    MissingTerminal,
    UnsupportedDirectGuarantee(&'static str),
}

fn validate_producer(producer: &Producer, existing: &[Record]) -> Result<(), ModelError> {
    let mut input_tokens = BTreeSet::new();
    for record in existing {
        for token in [record.header.token, record.payload.token] {
            if !input_tokens.insert(token) {
                return Err(ModelError::DuplicateToken(token));
            }
        }
    }
    for path in &producer.paths {
        let mut fields = BTreeSet::new();
        let mut tokens = input_tokens.clone();
        let mut remaining: BTreeSet<_> = (0..existing.len()).collect();
        for action in &path.actions {
            match action {
                Action::Effect(_) => {}
                Action::ReleaseExisting(index) => {
                    if *index >= existing.len() {
                        return Err(ModelError::UnknownExisting(*index));
                    }
                    if !remaining.remove(index) {
                        return Err(ModelError::DuplicateRelease(*index));
                    }
                }
                Action::Init { field, token, .. } => {
                    if !fields.insert(*field) {
                        return Err(ModelError::DuplicateField(*field));
                    }
                    if !tokens.insert(*token) {
                        return Err(ModelError::DuplicateToken(*token));
                    }
                }
            }
        }
        match path.terminal {
            Terminal::Fresh => {
                for field in [Field::Header, Field::Payload] {
                    if !fields.contains(&field) {
                        return Err(ModelError::MissingField(field));
                    }
                }
            }
            Terminal::ForwardExisting(index) => {
                if !fields.is_empty() {
                    return Err(ModelError::PartialBeforeForward);
                }
                if index >= existing.len() {
                    return Err(ModelError::UnknownExisting(index));
                }
                if !remaining.remove(&index) {
                    return Err(ModelError::DuplicateRelease(index));
                }
            }
            Terminal::Error(_) if producer.form == ResultForm::Total => {
                return Err(ModelError::ErrorInTotal);
            }
            Terminal::Error(_) => {}
        }
        if let Some(index) = remaining.first() {
            return Err(ModelError::LeakedExisting(*index));
        }
    }
    Ok(())
}

/// Storage-independent source oracle, deliberately map/set based.
fn oracle(invocation: &Invocation) -> Result<Semantics, ModelError> {
    validate_producer(&invocation.producer, &invocation.existing)?;
    let path = invocation
        .producer
        .paths
        .get(invocation.selected_path)
        .ok_or(ModelError::BadPath(invocation.selected_path))?;
    let mut fields = BTreeMap::<Field, Part>::new();
    let mut existing: Vec<_> = invocation.existing.iter().cloned().map(Some).collect();
    let mut tokens = BTreeSet::<Token>::new();
    for record in &invocation.existing {
        tokens.insert(record.header.token);
        tokens.insert(record.payload.token);
    }
    let mut effects = Vec::new();
    let mut releases = Vec::new();
    for action in &path.actions {
        match action {
            Action::Effect(effect) => effects.push((*effect).to_owned()),
            Action::ReleaseExisting(index) => {
                let record = existing
                    .get_mut(*index)
                    .ok_or(ModelError::UnknownExisting(*index))?
                    .take()
                    .ok_or(ModelError::DuplicateRelease(*index))?;
                releases.extend([record.payload.token, record.header.token]);
            }
            Action::Init {
                field,
                token,
                value,
            } => {
                if fields.contains_key(field) {
                    return Err(ModelError::DuplicateField(*field));
                }
                if !tokens.insert(*token) {
                    return Err(ModelError::DuplicateToken(*token));
                }
                fields.insert(
                    *field,
                    Part {
                        token: *token,
                        value: *value,
                    },
                );
            }
        }
    }
    match path.terminal {
        Terminal::Fresh => {
            let record = Record {
                header: fields
                    .remove(&Field::Header)
                    .ok_or(ModelError::MissingField(Field::Header))?,
                payload: fields
                    .remove(&Field::Payload)
                    .ok_or(ModelError::MissingField(Field::Payload))?,
            };
            if invocation.consumer == Consumer::ObserveWholeThenPlace {
                effects.push(format!("observe-whole:{:016x}", record.digest()));
            }
            Ok(Semantics {
                outcome: Outcome::Placed {
                    sink: invocation.sink,
                    record,
                },
                effects,
                releases,
            })
        }
        Terminal::ForwardExisting(index) => {
            if !fields.is_empty() {
                return Err(ModelError::PartialBeforeForward);
            }
            let record = existing
                .get_mut(index)
                .ok_or(ModelError::UnknownExisting(index))?
                .take()
                .ok_or(ModelError::DuplicateRelease(index))?;
            if invocation.consumer == Consumer::ObserveWholeThenPlace {
                effects.push(format!("observe-whole:{:016x}", record.digest()));
            }
            Ok(Semantics {
                outcome: Outcome::Placed {
                    sink: invocation.sink,
                    record,
                },
                effects,
                releases,
            })
        }
        Terminal::Error(code) => {
            releases.extend(
                Field::DROP_ORDER
                    .iter()
                    .filter_map(|field| fields.remove(field).map(|part| part.token)),
            );
            Ok(Semantics {
                outcome: Outcome::Failed { code },
                effects,
                releases,
            })
        }
    }
}

fn direct_eligibility(invocation: &Invocation) -> Result<(), ModelError> {
    if invocation.consumer == Consumer::ObserveWholeThenPlace {
        return Err(ModelError::UnsupportedDirectGuarantee(
            "whole-result observation requires materialization",
        ));
    }
    if invocation
        .producer
        .paths
        .iter()
        .any(|path| matches!(path.terminal, Terminal::ForwardExisting(_)))
    {
        return Err(ModelError::UnsupportedDirectGuarantee(
            "a reachable success forwards an existing owner",
        ));
    }
    Ok(())
}

fn lower(strategy: Strategy, invocation: &Invocation) -> Result<Vec<Event>, ModelError> {
    validate_producer(&invocation.producer, &invocation.existing)?;
    if invocation.claim_direct {
        direct_eligibility(invocation)?;
        if strategy == Strategy::WholeResult {
            return Err(ModelError::UnsupportedDirectGuarantee(
                "whole-result lowering cannot satisfy direct placement",
            ));
        }
    }
    let path = invocation
        .producer
        .paths
        .get(invocation.selected_path)
        .ok_or(ModelError::BadPath(invocation.selected_path))?;
    let whole = Storage::WholeResult(invocation.sink);
    let final_sink = Storage::Final(invocation.sink);
    let needs_whole =
        strategy == Strategy::WholeResult || invocation.consumer == Consumer::ObserveWholeThenPlace;
    let destination = if needs_whole { whole } else { final_sink };
    let mut events = Vec::new();
    if needs_whole {
        events.push(Event::AllocateWhole(whole));
    }
    for action in &path.actions {
        match action {
            Action::Effect(effect) => events.push(Event::Effect(effect)),
            Action::ReleaseExisting(index) => {
                events.push(Event::ReleaseExisting(Storage::Existing(*index)))
            }
            Action::Init {
                field,
                token,
                value,
            } => events.push(Event::WriteField {
                destination,
                field: *field,
                part: Part {
                    token: *token,
                    value: *value,
                },
            }),
        }
    }
    match path.terminal {
        Terminal::Fresh => {
            if invocation.consumer == Consumer::ObserveWholeThenPlace {
                events.push(Event::ObserveWhole(whole));
            }
            if needs_whole {
                events.push(Event::TransferWhole {
                    from: whole,
                    to: final_sink,
                });
            }
            events.push(Event::Commit(final_sink));
        }
        Terminal::ForwardExisting(index) => {
            events.push(Event::TransferWhole {
                from: Storage::Existing(index),
                to: destination,
            });
            if invocation.consumer == Consumer::ObserveWholeThenPlace {
                events.push(Event::ObserveWhole(whole));
            }
            if needs_whole {
                events.push(Event::TransferWhole {
                    from: whole,
                    to: final_sink,
                });
            }
            events.push(Event::Commit(final_sink));
        }
        Terminal::Error(code) => events.push(Event::Error(code)),
    }
    Ok(events)
}

#[derive(Clone, Debug)]
struct Slots([Option<Part>; 2]);

impl Slots {
    fn vacant() -> Self {
        Self(std::array::from_fn(|_| None))
    }
    fn from_record(record: Record) -> Self {
        Self([Some(record.header), Some(record.payload)])
    }
    fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }
    fn take_record(&mut self) -> Result<Record, ModelError> {
        let header = self.0[0]
            .take()
            .ok_or(ModelError::MissingField(Field::Header))?;
        let payload = self.0[1]
            .take()
            .ok_or(ModelError::MissingField(Field::Payload))?;
        Ok(Record { header, payload })
    }
    fn view_record(&self) -> Result<Record, ModelError> {
        Ok(Record {
            header: self.0[0]
                .clone()
                .ok_or(ModelError::MissingField(Field::Header))?,
            payload: self.0[1]
                .clone()
                .ok_or(ModelError::MissingField(Field::Payload))?,
        })
    }
}

/// Destination-indexed machine. Values, cleanup, effects, and costs arise only
/// by executing lowering events.
fn execute(invocation: &Invocation, sinks: &[u8], events: &[Event]) -> Result<Report, ModelError> {
    let mut storage = BTreeMap::<Storage, Slots>::new();
    let mut live = BTreeSet::<Token>::new();
    for sink in sinks {
        storage.insert(Storage::Final(*sink), Slots::vacant());
    }
    for (index, record) in invocation.existing.iter().cloned().enumerate() {
        for token in [record.header.token, record.payload.token] {
            if !live.insert(token) {
                return Err(ModelError::DuplicateToken(token));
            }
        }
        storage.insert(Storage::Existing(index), Slots::from_record(record));
    }
    let mut effects = Vec::new();
    let mut releases = Vec::new();
    let mut cost = Cost::default();
    let mut terminal = None;
    for event in events {
        if terminal.is_some() {
            return Err(ModelError::EventAfterTerminal);
        }
        match event {
            Event::AllocateWhole(place) => {
                if storage.contains_key(place) {
                    return Err(ModelError::StorageAlreadyAllocated(*place));
                }
                storage.insert(*place, Slots::vacant());
                cost.record_sized_whole_slots += 1;
            }
            Event::Effect(effect) => effects.push((*effect).to_owned()),
            Event::ReleaseExisting(place) => {
                let slots = storage
                    .get_mut(place)
                    .ok_or(ModelError::UnknownStorage(*place))?;
                if slots.is_empty() {
                    return Err(ModelError::OwnerAlreadyConsumed(*place));
                }
                let record = slots.take_record()?;
                for token in [record.payload.token, record.header.token] {
                    if !live.remove(&token) {
                        return Err(ModelError::DuplicateToken(token));
                    }
                    releases.push(token);
                }
            }
            Event::WriteField {
                destination,
                field,
                part,
            } => {
                let slots = storage
                    .get_mut(destination)
                    .ok_or(ModelError::UnknownStorage(*destination))?;
                if slots.0[field.index()].is_some() {
                    return Err(ModelError::DuplicateField(*field));
                }
                if !live.insert(part.token) {
                    return Err(ModelError::DuplicateToken(part.token));
                }
                slots.0[field.index()] = Some(part.clone());
            }
            Event::TransferWhole { from, to } => {
                let record = storage
                    .get_mut(from)
                    .ok_or(ModelError::UnknownStorage(*from))?
                    .take_record()?;
                let destination = storage.get_mut(to).ok_or(ModelError::UnknownStorage(*to))?;
                if !destination.is_empty() {
                    return Err(ModelError::DestinationOccupied(*to));
                }
                *destination = Slots::from_record(record);
                cost.whole_record_transfers += 1;
            }
            Event::ObserveWhole(place) => {
                let record = storage
                    .get(place)
                    .ok_or(ModelError::UnknownStorage(*place))?
                    .view_record()?;
                effects.push(format!("observe-whole:{:016x}", record.digest()));
            }
            Event::Commit(place) => {
                let record = storage
                    .get_mut(place)
                    .ok_or(ModelError::UnknownStorage(*place))?
                    .take_record()?;
                for (other, slots) in &storage {
                    if !slots.is_empty() {
                        return Err(ModelError::UnreleasedOwnership(*other));
                    }
                }
                for token in [record.header.token, record.payload.token] {
                    if !live.remove(&token) {
                        return Err(ModelError::DuplicateToken(token));
                    }
                }
                if let Some(token) = live.first() {
                    return Err(ModelError::LeakedToken(*token));
                }
                let Storage::Final(sink) = place else {
                    return Err(ModelError::UnknownStorage(*place));
                };
                terminal = Some(Semantics {
                    outcome: Outcome::Placed {
                        sink: *sink,
                        record,
                    },
                    effects: effects.clone(),
                    releases: releases.clone(),
                });
            }
            Event::Error(code) => {
                for (place, slots) in &storage {
                    if matches!(place, Storage::Existing(_)) && !slots.is_empty() {
                        return Err(ModelError::UnreleasedOwnership(*place));
                    }
                }
                for slots in storage.values_mut() {
                    for field in Field::DROP_ORDER {
                        if let Some(part) = slots.0[field.index()].take() {
                            if !live.remove(&part.token) {
                                return Err(ModelError::DuplicateToken(part.token));
                            }
                            releases.push(part.token);
                        }
                    }
                }
                if let Some(token) = live.first() {
                    return Err(ModelError::LeakedToken(*token));
                }
                terminal = Some(Semantics {
                    outcome: Outcome::Failed { code: *code },
                    effects: effects.clone(),
                    releases: releases.clone(),
                });
            }
        }
    }
    Ok(Report {
        semantics: terminal.ok_or(ModelError::MissingTerminal)?,
        cost,
    })
}

fn run(strategy: Strategy, invocation: &Invocation, sinks: &[u8]) -> Result<Report, ModelError> {
    execute(invocation, sinks, &lower(strategy, invocation)?)
}

fn init(field: Field, token: u32, value: u64) -> Action {
    Action::Init {
        field,
        token: Token(token),
        value,
    }
}

fn fresh_path(seed: u32) -> Path {
    Path {
        actions: vec![
            Action::Effect("begin"),
            init(Field::Header, seed, 0x1000 + u64::from(seed)),
            Action::Effect("after-header"),
            init(Field::Payload, seed + 1, 0x8000 + u64::from(seed)),
        ],
        terminal: Terminal::Fresh,
    }
}

fn existing_record() -> Record {
    Record {
        header: Part {
            token: Token(900),
            value: 0x55,
        },
        payload: Part {
            token: Token(901),
            value: 0xaa,
        },
    }
}

fn cases() -> Vec<Invocation> {
    let total = Producer {
        form: ResultForm::Total,
        paths: vec![fresh_path(10)],
    };
    let fallible = Producer {
        form: ResultForm::Fallible,
        paths: vec![
            fresh_path(20),
            Path {
                actions: vec![Action::Effect("begin")],
                terminal: Terminal::Error(7),
            },
            Path {
                actions: vec![
                    Action::Effect("begin"),
                    init(Field::Header, 30, 0x30),
                    Action::Effect("after-header"),
                    init(Field::Payload, 31, 0x31),
                    Action::Effect("before-error"),
                ],
                terminal: Terminal::Error(9),
            },
        ],
    };
    let mut mixed_fresh = fresh_path(40);
    mixed_fresh.actions.insert(1, Action::ReleaseExisting(0));
    let mixed = Producer {
        form: ResultForm::Fallible,
        paths: vec![
            mixed_fresh,
            Path {
                actions: vec![Action::Effect("forward")],
                terminal: Terminal::ForwardExisting(0),
            },
        ],
    };
    vec![
        Invocation {
            label: "fresh-total",
            producer: total,
            selected_path: 0,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 10,
            existing: vec![],
        },
        Invocation {
            label: "fresh-ok",
            producer: fallible.clone(),
            selected_path: 0,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 11,
            existing: vec![],
        },
        Invocation {
            label: "err-before-fields",
            producer: fallible.clone(),
            selected_path: 1,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 12,
            existing: vec![],
        },
        Invocation {
            label: "err-after-subfields",
            producer: fallible,
            selected_path: 2,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 13,
            existing: vec![],
        },
        Invocation {
            label: "forward-existing-owner",
            producer: mixed.clone(),
            selected_path: 1,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 14,
            existing: vec![existing_record()],
        },
        Invocation {
            label: "forced-whole-observation",
            producer: Producer {
                form: ResultForm::Fallible,
                paths: vec![fresh_path(40)],
            },
            selected_path: 0,
            consumer: Consumer::ObserveWholeThenPlace,
            claim_direct: false,
            sink: 15,
            existing: vec![],
        },
        Invocation {
            label: "mixed-fresh-for-eligibility",
            producer: mixed,
            selected_path: 0,
            consumer: Consumer::Place,
            claim_direct: false,
            sink: 16,
            existing: vec![existing_record()],
        },
    ]
}

fn expect_event_error(
    label: &str,
    invocation: &Invocation,
    sinks: &[u8],
    events: &[Event],
    expected: ModelError,
) -> Result<(), String> {
    let actual = execute(invocation, sinks, events).expect_err("mutated lowering was accepted");
    if actual != expected {
        return Err(format!("{label}: expected {expected:?}, got {actual:?}"));
    }
    Ok(())
}

fn check_mutations(
    base: &Invocation,
    mixed_fresh: &Invocation,
    sinks: &[u8],
) -> Result<(), String> {
    let events = lower(Strategy::ResultTree, base).map_err(|error| format!("{error:?}"))?;
    let mut misrouted = events.clone();
    if let Event::WriteField { destination, .. } = misrouted
        .iter_mut()
        .find(|event| {
            matches!(
                event,
                Event::WriteField {
                    field: Field::Payload,
                    ..
                }
            )
        })
        .unwrap()
    {
        *destination = Storage::Final(11);
    }
    expect_event_error(
        "misroute-field",
        base,
        sinks,
        &misrouted,
        ModelError::MissingField(Field::Payload),
    )?;

    let whole = lower(Strategy::WholeResult, base).map_err(|error| format!("{error:?}"))?;
    let omitted: Vec<_> = whole
        .into_iter()
        .filter(|event| !matches!(event, Event::TransferWhole { .. }))
        .collect();
    expect_event_error(
        "omit-transfer",
        base,
        sinks,
        &omitted,
        ModelError::MissingField(Field::Header),
    )?;

    let mut premature = events.clone();
    let commit = premature.pop().unwrap();
    premature.insert(2, commit);
    expect_event_error(
        "publish-before-complete",
        base,
        sinks,
        &premature,
        ModelError::MissingField(Field::Payload),
    )?;

    let mut duplicate_owner = events.clone();
    let header_token = duplicate_owner
        .iter()
        .find_map(|event| match event {
            Event::WriteField {
                field: Field::Header,
                part,
                ..
            } => Some(part.token),
            _ => None,
        })
        .unwrap();
    if let Event::WriteField { part, .. } = duplicate_owner
        .iter_mut()
        .find(|event| {
            matches!(
                event,
                Event::WriteField {
                    field: Field::Payload,
                    ..
                }
            )
        })
        .unwrap()
    {
        part.token = header_token;
    }
    expect_event_error(
        "duplicate-owner",
        base,
        sinks,
        &duplicate_owner,
        ModelError::DuplicateToken(header_token),
    )?;

    let mut duplicate_sink = events;
    duplicate_sink.push(Event::Commit(Storage::Final(base.sink)));
    expect_event_error(
        "duplicate-sink-publication",
        base,
        sinks,
        &duplicate_sink,
        ModelError::EventAfterTerminal,
    )?;

    let mut duplicate_release =
        lower(Strategy::ResultTree, mixed_fresh).map_err(|error| format!("{error:?}"))?;
    let release_index = duplicate_release
        .iter()
        .position(|event| matches!(event, Event::ReleaseExisting(_)))
        .expect("mixed fresh lowering releases its unused input");
    let release = duplicate_release[release_index].clone();
    duplicate_release.insert(release_index + 1, release);
    expect_event_error(
        "duplicate-release-event",
        mixed_fresh,
        sinks,
        &duplicate_release,
        ModelError::OwnerAlreadyConsumed(Storage::Existing(0)),
    )
}

fn expect_source_error(
    label: &str,
    invocation: &Invocation,
    expected: ModelError,
) -> Result<(), String> {
    let lower_error =
        lower(Strategy::ResultTree, invocation).expect_err("invalid producer was lowered");
    let oracle_error = oracle(invocation).expect_err("invalid producer reached the oracle");
    if lower_error != expected || oracle_error != expected {
        return Err(format!(
            "{label}: expected {expected:?}, lower={lower_error:?}, oracle={oracle_error:?}"
        ));
    }
    Ok(())
}

fn check_invalid_source_paths() -> Result<(), String> {
    let missing = Invocation {
        label: "invalid-unselected-path",
        producer: Producer {
            form: ResultForm::Total,
            paths: vec![
                fresh_path(70),
                Path {
                    actions: vec![init(Field::Header, 80, 1)],
                    terminal: Terminal::Fresh,
                },
            ],
        },
        selected_path: 0,
        consumer: Consumer::Place,
        claim_direct: false,
        sink: 10,
        existing: vec![],
    };
    expect_source_error(
        "missing-field-unselected",
        &missing,
        ModelError::MissingField(Field::Payload),
    )?;

    let leak = Invocation {
        label: "leaked-input-unselected",
        producer: Producer {
            form: ResultForm::Total,
            paths: vec![
                Path {
                    actions: vec![Action::Effect("forward")],
                    terminal: Terminal::ForwardExisting(0),
                },
                fresh_path(81),
            ],
        },
        selected_path: 0,
        consumer: Consumer::Place,
        claim_direct: false,
        sink: 10,
        existing: vec![existing_record()],
    };
    expect_source_error(
        "leaked-input-unselected",
        &leak,
        ModelError::LeakedExisting(0),
    )?;

    let mut repeated_release = fresh_path(82);
    repeated_release
        .actions
        .insert(0, Action::ReleaseExisting(0));
    repeated_release
        .actions
        .insert(1, Action::ReleaseExisting(0));
    let duplicate_release = Invocation {
        label: "duplicate-release-unselected",
        producer: Producer {
            form: ResultForm::Total,
            paths: vec![
                Path {
                    actions: vec![Action::Effect("forward")],
                    terminal: Terminal::ForwardExisting(0),
                },
                repeated_release,
            ],
        },
        selected_path: 0,
        consumer: Consumer::Place,
        claim_direct: false,
        sink: 10,
        existing: vec![existing_record()],
    };
    expect_source_error(
        "duplicate-release-unselected",
        &duplicate_release,
        ModelError::DuplicateRelease(0),
    )?;

    let mut colliding_path = fresh_path(90);
    colliding_path.actions.insert(0, Action::ReleaseExisting(0));
    if let Action::Init { token, .. } = &mut colliding_path.actions[2] {
        *token = Token(900);
    } else {
        return Err("collision fixture lost its header initialization".to_owned());
    }
    let source_collision = Invocation {
        label: "source-input-token-collision",
        producer: Producer {
            form: ResultForm::Total,
            paths: vec![colliding_path],
        },
        selected_path: 0,
        consumer: Consumer::Place,
        claim_direct: false,
        sink: 10,
        existing: vec![existing_record()],
    };
    expect_source_error(
        "source-input-token-collision",
        &source_collision,
        ModelError::DuplicateToken(Token(900)),
    )?;

    let input_collision = Invocation {
        label: "input-token-collision",
        producer: Producer {
            form: ResultForm::Total,
            paths: vec![Path {
                actions: vec![],
                terminal: Terminal::ForwardExisting(0),
            }],
        },
        selected_path: 0,
        consumer: Consumer::Place,
        claim_direct: false,
        sink: 10,
        existing: vec![Record {
            header: Part {
                token: Token(900),
                value: 1,
            },
            payload: Part {
                token: Token(900),
                value: 2,
            },
        }],
    };
    expect_source_error(
        "input-token-collision",
        &input_collision,
        ModelError::DuplicateToken(Token(900)),
    )?;
    Ok(())
}

fn check_direct_guarantees(all: &[Invocation], sinks: &[u8]) -> Result<(), String> {
    let mut fresh = all[0].clone();
    fresh.claim_direct = true;
    if run(Strategy::ResultTree, &fresh, sinks)
        .map_err(|error| format!("{error:?}"))?
        .cost
        != Cost::default()
    {
        return Err("eligible direct construction retained a whole-value event".to_owned());
    }
    if lower(Strategy::WholeResult, &fresh)
        != Err(ModelError::UnsupportedDirectGuarantee(
            "whole-result lowering cannot satisfy direct placement",
        ))
    {
        return Err("whole-result lowering accepted a direct claim".to_owned());
    }
    let mut mixed = all[6].clone();
    mixed.claim_direct = true;
    if lower(Strategy::ResultTree, &mixed)
        != Err(ModelError::UnsupportedDirectGuarantee(
            "a reachable success forwards an existing owner",
        ))
    {
        return Err("a fresh selected path hid a reachable forwarding path".to_owned());
    }
    let mut observed = all[5].clone();
    observed.claim_direct = true;
    if lower(Strategy::ResultTree, &observed)
        != Err(ModelError::UnsupportedDirectGuarantee(
            "whole-result observation requires materialization",
        ))
    {
        return Err("whole observation accepted a direct claim".to_owned());
    }
    Ok(())
}

fn run_suite(print: bool) -> Result<(), String> {
    let all = cases();
    let sinks: Vec<_> = all.iter().map(|case| case.sink).collect();
    if print {
        println!("logical_record_bytes={LOGICAL_RECORD_BYTES}");
        println!("case,strategy,outcome,effects,releases,whole_slots,whole_transfers");
    }
    for invocation in &all {
        let expected = oracle(invocation).map_err(|error| format!("oracle: {error:?}"))?;
        for strategy in [Strategy::WholeResult, Strategy::ResultTree] {
            let events = lower(strategy, invocation)
                .map_err(|error| format!("{}: {error:?}", invocation.label))?;
            let report = execute(invocation, &sinks, &events).map_err(|error| {
                format!(
                    "{} {strategy:?}: {error:?}; events={events:?}",
                    invocation.label
                )
            })?;
            if report.semantics != expected {
                return Err(format!(
                    "{} {strategy:?}: mismatch expected={expected:?} actual={:?}; events={events:?}",
                    invocation.label, report.semantics
                ));
            }
            if print {
                println!(
                    "{},{strategy:?},{:?},{:?},{:?},{},{}",
                    invocation.label,
                    report.semantics.outcome,
                    report.semantics.effects,
                    report.semantics.releases,
                    report.cost.record_sized_whole_slots,
                    report.cost.whole_record_transfers
                );
            }
        }
    }
    if run(Strategy::ResultTree, &all[3], &sinks)
        .map_err(|e| format!("{e:?}"))?
        .semantics
        .releases
        != vec![Token(31), Token(30)]
    {
        return Err("late failure cleanup mismatch".to_owned());
    }
    if run(Strategy::ResultTree, &all[6], &sinks)
        .map_err(|error| format!("mixed fresh release: {error:?}"))?
        .semantics
        .releases
        != vec![Token(901), Token(900)]
    {
        return Err("mixed fresh path did not release its unused input once".to_owned());
    }
    for (index, expected) in [
        (0, Cost::default()),
        (1, Cost::default()),
        (6, Cost::default()),
        (6, Cost::default()),
        (
            4,
            Cost {
                record_sized_whole_slots: 0,
                whole_record_transfers: 1,
            },
        ),
        (
            5,
            Cost {
                record_sized_whole_slots: 1,
                whole_record_transfers: 1,
            },
        ),
    ] {
        let actual = run(Strategy::ResultTree, &all[index], &sinks)
            .map_err(|e| format!("{e:?}"))?
            .cost;
        if actual != expected {
            return Err(format!(
                "{}: expected {expected:?}, got {actual:?}",
                all[index].label
            ));
        }
    }
    let mut renamed = all[0].clone();
    renamed.label = "same-producer-different-name";
    if run(Strategy::ResultTree, &renamed, &sinks) != run(Strategy::ResultTree, &all[0], &sinks) {
        return Err("call-site name selected behavior".to_owned());
    }
    let mixed_fresh = run(Strategy::ResultTree, &all[6], &sinks)
        .map_err(|error| format!("mixed fresh: {error:?}"))?;
    if mixed_fresh.semantics.releases != vec![Token(901), Token(900)] {
        return Err("mixed fresh path did not release its unused input in order".to_owned());
    }
    check_mutations(&all[0], &all[6], &sinks)?;
    check_invalid_source_paths()?;
    check_direct_guarantees(&all, &sinks)?;
    if print {
        println!("checks=oracle,event-costs,all-paths,mutations,direct-eligibility,two-sinks");
    }
    Ok(())
}

fn main() -> Result<(), String> {
    run_suite(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_construction_cases_and_mutations() {
        run_suite(false).unwrap();
    }
}
