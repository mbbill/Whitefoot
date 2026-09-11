//! [EFF-2] state-parameter effects and release attribution.
//!
//! The exhibited row is the union of the syntactic contribution and the
//! release contribution: the effect rows of every compiler-derived release
//! that may run on a normal control-flow edge, scoped by [STOR-3] to the
//! system resource families whose [SYS-5] contract fixes a nonempty row.

use crate::semantic::model::CheckedStateStep;
use crate::semantic::state_origins::StateOriginPrecision;
use crate::{SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule};

use super::super::model::{CheckedResultStateOrigin, CheckedResultStatePath};
use super::{assert_rule, assert_rule_kind, with_semantics};

const RELEASE_FIX: &str = "declare the release effects of every resource this function may release, or move the owner out";

#[test]
fn owning_array_and_optional_cleanup_select_only_released_state() {
    let array = r#"fn release(values: own array<ReadFile, 2>) -> result: own unit writes(values) {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(array.as_bytes());
    let omitted = array.replace("writes(values)", "pure");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::ReleaseEffectMismatch { .. })
    });
    let zero = omitted.replace("array<ReadFile, 2>", "array<ReadFile, 0>");
    assert_complete(zero.as_bytes());

    let optional = r#"struct Entry {
  file: ReadFile;
  spare: box<u64>;
}

fn release(file: own ReadFile, spare: own box<u64>) -> result: own unit writes(file) {
  let stored_entry = Entry(file: move file, spare: move spare);
  let present = Some<Entry>(value: move stored_entry);
  return unit;
}

fn empty() -> result: own unit pure {
  let absent = None<Entry>();
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(optional.as_bytes());
    let omitted = optional.replace("writes(file)", "pure");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::ReleaseEffectMismatch { .. })
    });
    let spurious = optional.replace("writes(file)", "writes(file, spare)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra == &["writes(spare)"])
    });
}

#[test]
fn known_back_positions_survive_helpers_and_unrelated_replacement() {
    let source = r#"struct Packed {
  values: FixedVector<box<u64>, 2>;
}

fn pack(first: own box<u64>, second: own box<u64>) -> (values: own FixedVector<box<u64>, 2>, stamp: own u64) reads(first), writes(first) contract {
  ensures len_of(values) == 2_u64;
} {
  let values = fixed_vector::<box<u64>, 2>();
  region {
    place_back(vector: &uniq values, value: move first);
  }
  region {
    place_back(vector: &uniq values, value: move second);
  }
  return move values, 7_u64;
}

fn read(value: &box<u64>) -> result: own u64 reads(value) {
  return deref(deref(value));
}

fn observe(first: own box<u64>, second: own box<u64>, replacement: own box<u64>) -> result: own u64 reads(first, second), writes(first) {
  let (values, stamp) = pack(first: move first, second: move second);
  let packed = Packed(values: move values);
  let old = replace packed.values[0_u64] = move replacement;
  region {
    return read(value: &packed.values[1_u64]);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
    let omitted = source.replace("reads(first, second)", "reads(first)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing == &["reads(second)"])
    });
    let spurious = source.replace("reads(first, second)", "reads(first, second, replacement)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra == &["reads(replacement)"])
    });
}

#[test]
fn acquired_run_success_carries_length_without_giving_it_to_refusal() {
    let source = r#"fn pack['s](store: &uniq STORE, value: own box<u64>) -> result: own Option<Vector<'s, box<u64>>> reads(store), writes(store), allocates(store) {
  region {
    match ACQUIRE::<box<u64>>(store: &uniq deref(store), count: 1_u64) {
      None() => {
        return None<Vector<'s, box<u64>>>();
      }
      Some(value: empty) => {
        region {
          place_back(vector: &uniq empty, value: move value);
        }
        let full = move empty;
        return Some<Vector<'s, box<u64>>>(value: move full);
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for (store, acquire) in [
        ("Heap<'s>", "heap_vector"),
        ("Arena<'s, 64, 16>", "arena_vector"),
    ] {
        let source = source.replace("STORE", store).replace("ACQUIRE", acquire);
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("acquired run endpoints must check: {outcome:?}");
            };
            let image = crate::semantic::model::CheckedStateOrigins::instantiate(
                &program.data.functions[0].result_state_origin,
                &[None, None],
            );
            assert_eq!(image.clone().enum_payload(1, 0).run_lengths.root(), Some(1));
            assert_eq!(image.enum_payload(0, 0).run_lengths.root(), None);
        });
    }
}

#[test]
fn known_back_extraction_returns_the_corresponding_owner() {
    let source = r#"fn tail(first: own box<u64>, second: own box<u64>) -> result: own box<u64> reads(first, second), writes(first, second) {
  let values = fixed_vector::<box<u64>, 2>();
  region {
    place_back(vector: &uniq values, value: move first);
  }
  region {
    place_back(vector: &uniq values, value: move second);
  }
  region {
    let last = take_back(vector: &uniq values);
    let rest = move values;
    RETURN
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for (tail, parameter) in [
        ("return move last;", 1),
        (
            "region {\n      let previous = take_back(vector: &uniq rest);\n      return move previous;\n    }",
            0,
        ),
    ] {
        let source = source.replace("RETURN", tail);
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("known endpoint extraction must check: {outcome:?}");
            };
            assert_eq!(
                program.data.functions[0].result_state_origin,
                CheckedResultStateOrigin::Finite {
                    formals: vec![root(parameter)],
                    run_lengths: Default::default(),
                }
            );
        });
    }
}

#[test]
fn run_length_joins_do_not_enumerate_capacity_or_preserve_different_lengths() {
    let source = br#"fn choose(first: own box<u64>, second: own box<u64>, extra: own Bool) -> result: own FixedVector<box<u64>, 1000000000> reads(first), writes(first) {
  let values = fixed_vector::<box<u64>, 1000000000>();
  region {
    place_back(vector: &uniq values, value: move first);
  }
  if extra {
    region {
      place_back(vector: &uniq values, value: move second);
    }
    return move values;
  }
  return move values;
}

fn rotate(value: own box<u64>) -> result: own FixedVector<box<u64>, 1000000000> reads(value), writes(value) {
  let values = fixed_vector::<box<u64>, 1000000000>();
  region {
    place_back(vector: &uniq values, value: move value);
  }
  for (
    round in 0_u64..8_u64,
    invariant at_most_one: len_of(values) <= 1_u64,
    invariant at_least_one: len_of(values) >= 1_u64
  ) {
    let first = take_front(vector: &uniq values);
    let rest = move values;
    region {
      place_back(vector: &uniq rest, value: move first);
    }
    set values = move rest;
  }
  return move values;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("finite branch and loop images must check: {outcome:?}");
        };
        let CheckedResultStateOrigin::Finite { run_lengths, .. } =
            &program.data.functions[0].result_state_origin
        else {
            panic!("the joined owner image must remain finite");
        };
        assert_eq!(run_lengths.root(), None);
        let CheckedResultStateOrigin::Finite { formals, .. } =
            &program.data.functions[1].result_state_origin
        else {
            panic!("the rotating owner image must remain finite");
        };
        assert!(formals.iter().any(|route| route.parameter == 0));
    });
}

#[test]
fn dynamic_element_queries_keep_field_effects_separate() {
    let source = r#"struct Slot {
  key: u64;
  payload: box<u64>;
}

fn read_key(slot: &Slot) -> result: own u64 reads(slot.key) {
  return deref(slot).key;
}

fn read_payload(slot: &Slot) -> result: own u64 reads(slot.payload) {
  return deref(deref(slot).payload);
}

fn observe(values: own array<Slot, 2>, first: own box<u64>, second: own box<u64>, index: own u64) -> result: own u64 reads(values), writes(values) contract {
  requires index < 2_u64;
} {
  let first_slot = Slot(key: 7_u64, payload: move first);
  let second_slot = Slot(key: 9_u64, payload: move second);
  let old_first = replace values[0_u64] = move first_slot;
  let old_second = replace values[1_u64] = move second_slot;
  OBSERVE
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for observation in [
        "return values[index].key;",
        "region {\n    return read_key(slot: &values[index]);\n  }",
    ] {
        let key = source.replace("OBSERVE", observation);
        assert_complete(key.as_bytes());
        let spurious = key.replace("reads(values)", "reads(values, first, second)");
        assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
                if extra == &["reads(first)", "reads(second)"])
        });
    }
    let payload = source
        .replace(
            "OBSERVE",
            "region {\n    return read_payload(slot: &values[index]);\n  }",
        )
        .replace("reads(values)", "reads(values, first, second)");
    assert_complete(payload.as_bytes());
    for incomplete in ["reads(values, first)", "reads(values, second)"] {
        let missing = payload.replace("reads(values, first, second)", incomplete);
        assert_rule_kind(missing.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                if missing.len() == 1 && matches!(missing[0].as_str(), "reads(first)" | "reads(second)"))
        });
    }
    let nested_key = source
        .replace("array<Slot, 2>", "array<array<Slot, 2>, 2>")
        .replace("values[0_u64]", "values[0_u64][0_u64]")
        .replace("values[1_u64]", "values[0_u64][1_u64]")
        .replace("OBSERVE", "return values[index][index].key;");
    assert_complete(nested_key.as_bytes());
    let fixed_key = source
        .replace("array<Slot, 2>", "FixedVector<Slot, 2>")
        .replace(
            "requires index < 2_u64;",
            "requires index < 2_u64;\n  requires len_of(values) == 2_u64;",
        )
        .replace("OBSERVE", "return values[index].key;");
    assert_complete(fixed_key.as_bytes());
}

#[test]
fn typed_selected_results_keep_unrelated_fields_separate_across_helpers() {
    let source = r#"struct Slot {
  file: ReadFile;
  scratch: FixedVector<ReadFile, 1>;
}

fn exchange(slots: own array<Slot, 2>, replacement: own Slot, index: own u64) -> (updated: own array<Slot, 2>, previous: own Slot) reads(slots), writes(slots) contract {
  requires index < 2_u64;
} {
  let previous = replace slots[index] = move replacement;
  return move slots, move previous;
}

fn relay(slots: own array<Slot, 2>, replacement: own Slot, index: own u64) -> (updated: own array<Slot, 2>, previous: own Slot) reads(slots), writes(slots) contract {
  requires index < 2_u64;
} {
  let (updated, previous) = exchange(slots: move slots, replacement: move replacement, index: index);
  return move updated, move previous;
}

fn consume(first: own ReadFile, second: own ReadFile, incoming: own Slot, index: own u64) -> (updated: own array<Slot, 2>, previous_file: own ReadFile) reads(first, second), writes(first, second) contract {
  requires index < 2_u64;
} {
  let first_empty = fixed_vector::<ReadFile, 1>();
  let second_empty = fixed_vector::<ReadFile, 1>();
  let first_slot = Slot(file: move first, scratch: move first_empty);
  let second_slot = Slot(file: move second, scratch: move second_empty);
  let values = fixed_vector::<Slot, 2>();
  region {
    place_back(vector: &uniq values, value: move first_slot);
  }
  region {
    place_back(vector: &uniq values, value: move second_slot);
  }
  let slots = array_from_fixed(vector: move values);
  EXCHANGE
  let Slot(file: previous_file, scratch: unused) = move previous;
  return move updated, move previous_file;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for operation in [
        "let previous = replace slots[index] = move incoming;\n  let updated = move slots;",
        "let (updated, previous) = exchange(slots: move slots, replacement: move incoming, index: index);",
        "let (updated, previous) = relay(slots: move slots, replacement: move incoming, index: index);",
    ] {
        let source = source.replace("EXCHANGE", operation);
        assert_complete(source.as_bytes());
        let omitted = source.replace("writes(first, second)", "writes(first)");
        assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                if missing == &["writes(second)"])
        });
        let spurious = source.replace(
            "reads(first, second)",
            "reads(first, second, incoming.file)",
        );
        assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
                if extra == &["reads(incoming.file)"])
        });
    }
}

#[test]
fn whole_effect_union_requires_independently_established_possible_sources() {
    let source = r#"fn read(value: &box<u64>) -> result: own u64 reads(value) {
  return deref(deref(value));
}

fn observe(values: own array<box<u64>, 2>, incoming: own box<u64>, spare: own box<u64>, index: own u64) -> result: own u64 reads(values, incoming), writes(values) contract {
  requires index < 2_u64;
} {
  ESTABLISH
  let previous = replace values[index] = move incoming;
  region {
    return read(value: &values[0_u64]);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let established = source.replace("ESTABLISH", "let observed = deref(incoming);");
    assert_complete(established.as_bytes());
    let omitted = established.replace("reads(values, incoming)", "reads(values)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(incoming)"))
    });
    let spurious = established.replace("reads(values, incoming)", "reads(values, incoming, spare)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra.iter().any(|effect| effect == "reads(spare)"))
    });
    for uncovered in [
        source.replace("  ESTABLISH\n", ""),
        source
            .replace("ESTABLISH", "set deref(incoming) = 9_u64;")
            .replace("writes(values)", "writes(values, incoming)"),
    ] {
        // The written row covers the upper bound, but does not prove that
        // incoming is read. A known write cannot establish a possible read.
        with_semantics(uncovered.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Unsupported { ref unsupported }
                    if unsupported.feature() == crate::UnsupportedSemanticFeature::OwnerStateRouting),
                "uncovered effect possibilities remain a capability gap: {outcome:?}"
            );
        });
    }
}

#[test]
fn whole_effect_union_keeps_struct_fields_distinct_from_their_root() {
    let source = br#"struct Pair {
  left: box<u64>;
  right: box<u64>;
}

fn read(value: &Pair) -> result: own u64 reads(value.left) {
  return deref(deref(value).left);
}

fn observe(values: own array<Pair, 2>, incoming: own Pair, replacement: own Pair, index: own u64) -> result: own u64 reads(values, incoming), writes(values, incoming) contract {
  requires index < 2_u64;
} {
  let supplied = replace incoming = move replacement;
  let previous = replace values[index] = move supplied;
  region {
    return read(value: &values[0_u64]);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // The initial product image already retains both field sources. A root
    // declaration cannot stand in for those discrete established atoms.
    assert_rule_kind(source, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, extra, .. }
            if missing.iter().any(|effect| effect == "reads(incoming.left)")
                && missing.iter().any(|effect| effect == "reads(incoming.right)")
                && extra.iter().any(|effect| effect == "reads(incoming)"))
    });
    let precise = std::str::from_utf8(source)
        .unwrap()
        .replace(
            "reads(values, incoming)",
            "reads(values, incoming.left, incoming.right)",
        )
        .replace(
            "writes(values, incoming)",
            "writes(values, incoming.left, incoming.right)",
        );
    assert_complete(precise.as_bytes());
}

#[test]
fn dynamic_slot_mutation_frames_siblings_through_owner_and_borrowed_helpers() {
    let source = r#"struct Slots {
  values: array<box<u64>, 2>;
  sibling: box<u64>;
}

MUTATOR

fn observe(values: own array<box<u64>, 2>, sibling: own box<u64>, incoming: own box<u64>, index: own u64) -> result: own u64 reads(values, sibling), writes(values) contract {
  requires index < 2_u64;
} {
  let state = Slots(values: move values, sibling: move sibling);
  INVOKE
  return deref(OBSERVED);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for (mutator, invoke, target, observed) in [
        (
            r#"fn mutate(state: own Slots, incoming: own box<u64>, index: own u64) -> result: own Slots reads(state.values), writes(state.values) contract {
  requires index < 2_u64;
} {
  OPERATION
  return move state;
}"#,
            "let updated = mutate(state: move state, incoming: move incoming, index: index);",
            "state.values[index]",
            "updated.sibling",
        ),
        (
            r#"fn mutate(state: &uniq Slots, incoming: own box<u64>, index: own u64) -> result: own unit reads(state.values), writes(state.values) contract {
  requires index < 2_u64;
} {
  OPERATION
  return unit;
}"#,
            "region {\n    let done = mutate(state: &uniq state, incoming: move incoming, index: index);\n  }",
            "deref(state).values[index]",
            "state.sibling",
        ),
    ] {
        for (operation, legal) in [
            (
                format!("let previous = replace {target} = move incoming;"),
                true,
            ),
            (format!("set {target} = move incoming;"), false),
        ] {
            let source = source
                .replace("MUTATOR", &mutator.replace("OPERATION", &operation))
                .replace("INVOKE", invoke)
                .replace("OBSERVED", observed);
            if !legal {
                // A live affine element still needs replace. Preserving a
                // known prefix adds no new read-out or overwrite permission.
                assert_rule_kind(source.as_bytes(), SemanticRule::Stor1, |kind| {
                    matches!(kind, SemanticIssueKind::AffineSetTarget { .. })
                });
                continue;
            }
            assert_complete(source.as_bytes());
            let omitted = source.replace("reads(values, sibling)", "reads(values)");
            assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
                matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                    if missing.iter().any(|effect| effect == "reads(sibling)"))
            });
            let spurious =
                source.replace("reads(values, sibling)", "reads(values, sibling, incoming)");
            assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
                matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
                    if extra.iter().any(|effect| effect == "reads(incoming)"))
            });
        }
    }
}

#[test]
fn precise_run_contents_separate_descriptor_and_release_effects() {
    let length = br#"fn length(value: own box<u64>) -> result: own u64 pure {
  let empty = fixed_vector::<box<u64>, 1>();
  region {
    place_back(vector: &uniq empty, value: move value);
  }
  let one = move empty;
  return len_of(one);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // The descriptor is fresh even though its sole element is imported.
    // Exact element placement now resolves the former capability sentinel.
    assert_complete(length);
    let spurious = String::from_utf8_lossy(length).replace("u64 pure {", "u64 reads(value) {");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra == &["reads(value)"])
    });

    let release = br#"struct Entry {
  file: ReadFile;
  spare: box<u64>;
}

fn release(file: own ReadFile, spare: own box<u64>) -> result: own unit writes(file) {
  let stored_entry = Entry(file: move file, spare: move spare);
  let empty = fixed_vector::<Entry, 1>();
  region {
    place_back(vector: &uniq empty, value: move stored_entry);
  }
  let one = move empty;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // Typed selection now resolves the release sentinel as well. Dropping
    // the ordinary memory-only sibling contributes no resource write.
    assert_complete(release);
    let omitted = String::from_utf8_lossy(release).replace("writes(file)", "pure");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::ReleaseEffectMismatch { .. })
    });
    let spurious = String::from_utf8_lossy(release).replace("writes(file)", "writes(file, spare)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra == &["writes(spare)"])
    });
}

#[test]
fn whole_run_transport_projects_all_suppliers_and_only_the_selected_component() {
    let source = br#"struct Packet {
  values: FixedVector<box<u64>, 2>;
  spare: box<u64>;
}

fn pack(first: own box<u64>, second: own box<u64>, spare: own box<u64>) -> (values: own FixedVector<box<u64>, 2>, returned: own box<u64>) reads(first), writes(first) contract {
  ensures len_of(values) == 2_u64;
} {
  let empty = fixed_vector::<box<u64>, 2>();
  region {
    place_back(vector: &uniq empty, value: move first);
  }
  let one = move empty;
  region {
    place_front(vector: &uniq one, value: move second);
  }
  let two = move one;
  return move two, move spare;
}

fn convert(packet: own Packet) -> result: own array<box<u64>, 2> reads(packet.values) contract {
  requires len_of(packet.values) == 2_u64;
} {
  return array_from_fixed(vector: move packet.values);
}

fn relay(first: own box<u64>, second: own box<u64>, spare: own box<u64>) -> result: own array<box<u64>, 2> reads(first, second), writes(first) {
  let (values, returned) = pack(first: move first, second: move second, spare: move spare);
  let packet = Packet(values: move values, spare: move returned);
  return convert(packet: move packet);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    let source = std::str::from_utf8(source).unwrap();
    let omitted = source.replace("reads(first, second)", "reads(first)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(second)"))
    });
    let omitted = source.replace("reads(first, second)", "reads(second)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(first)"))
    });
    let spurious = source.replace("reads(first, second)", "reads(first, second, spare)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra.iter().any(|effect| effect == "reads(spare)"))
    });
}

#[test]
fn replacing_one_literal_element_preserves_its_siblings_owner() {
    let source = br#"fn read['s](value: &Box<'s, u64>) -> result: own u64 reads(value) {
  return deref(deref(value));
}

fn exchange['s](slots: own array<Box<'s, u64>, 2>, incoming: own Box<'s, u64>) -> (updated: own array<Box<'s, u64>, 2>, previous: own Box<'s, u64>, sibling: own u64) reads(slots), writes(slots) {
  let previous = replace slots[0_u64] = move incoming;
  region {
    let sibling = read(value: &slots[1_u64]);
    return move slots, move previous, sibling;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    let source = std::str::from_utf8(source).unwrap();
    let spurious = source.replace("reads(slots)", "reads(slots, incoming)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra.iter().any(|effect| effect == "reads(incoming)"))
    });
    let replaced = source.replace("read(value: &slots[1_u64])", "read(value: &slots[0_u64])");
    assert_rule_kind(replaced.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(incoming)"))
    });
}

#[test]
fn nested_box_content_replacement_preserves_the_incoming_owner() {
    let source = br#"fn exchange(storage: &uniq box<box<u64>>, incoming: own box<u64>) -> previous: own box<u64> reads(storage), writes(storage) {
  let previous = replace deref(deref(storage)) = move incoming;
  return move previous;
}

fn observe(storage: own box<box<u64>>, incoming: own box<u64>) -> result: own u64 reads(storage, incoming), writes(storage) {
  region {
    let previous = exchange(storage: &uniq storage, incoming: move incoming);
    let old = deref(previous);
    let current = deref(deref(storage));
    return old +wrap current;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the outer allocation must retain its new content's origin: {outcome:?}"
        );
    });
    let omitted = std::str::from_utf8(source)
        .unwrap()
        .replace("reads(storage, incoming)", "reads(storage)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(incoming)"))
    });
}

#[test]
fn nested_box_successive_exchanges_do_not_retain_the_displaced_owner() {
    let source = br#"fn exchange(storage: &uniq box<box<u64>>, incoming: own box<u64>) -> previous: own box<u64> reads(storage), writes(storage) {
  let previous = replace deref(deref(storage)) = move incoming;
  return move previous;
}

fn twice(storage: &uniq box<box<u64>>, first: own box<u64>, second: own box<u64>) -> previous: own box<u64> reads(storage, first), writes(storage, first) {
  region {
    let old = exchange(storage: &uniq deref(storage), incoming: move first);
    let previous = exchange(storage: &uniq deref(storage), incoming: move second);
    return move previous;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("successive interior updates must check: {outcome:?}");
        };
        let twice = &program.data.functions[1];
        assert_eq!(
            twice.result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(1)]
            }
        );
        let CheckedResultStateOrigin::Finite { formals, .. } =
            &twice.borrowed_state_origins[0].origin
        else {
            panic!("the final containing owner must have a finite image");
        };
        assert!(formals.iter().all(|route| route.parameter != 1));
        assert!(formals.iter().any(|route| route.parameter == 0
            && route.exclusions == vec![vec![CheckedStateStep::Referent]]));
        assert!(
            formals.iter().any(|route| route.parameter == 2
                && route.result_fields == vec![CheckedStateStep::Referent])
        );
    });
}

#[test]
fn arena_cell_content_preserves_imported_owners_through_direct_and_helper_replacement() {
    let source = r#"fn exchange['s](storage: &uniq Box<'s, box<u64>>, incoming: own box<u64>) -> previous: own box<u64> reads(storage), writes(storage) {
  let previous = replace deref(deref(storage)) = move incoming;
  return move previous;
}

fn observe(initial: own box<u64>, incoming: own box<u64>) -> result: own u64 reads(initial, incoming), writes(initial) {
  region 'a {
    let store = arena_frame::<64, 16, 'a>();
    region {
      match arena_box(store: &uniq store, value: move initial) {
        Ok(value: storage) => {
          region {
            let previous = replace deref(storage) = move incoming;
            let old = deref(previous);
            let current = deref(deref(storage));
            return old +wrap current;
          }
        }
        Err(error: back) => {
          return deref(back);
        }
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for source in [
        source.to_owned(),
        source.replace(
            "replace deref(storage) = move incoming",
            "exchange(storage: &uniq storage, incoming: move incoming)",
        ),
    ] {
        assert_complete(source.as_bytes());
        for (written, omitted, expected) in [
            (
                "reads(initial, incoming)",
                "reads(initial)",
                "reads(incoming)",
            ),
            (
                "reads(initial, incoming)",
                "reads(incoming)",
                "reads(initial)",
            ),
            (
                "reads(initial, incoming), writes(initial)",
                "reads(initial, incoming)",
                "writes(initial)",
            ),
        ] {
            let omitted = source.replace(written, omitted);
            with_semantics(omitted.as_bytes(), |outcome| {
                let SemanticOutcome::SourceIssue { issue } = outcome else {
                    panic!("the omitted effect must reject: {outcome:?}");
                };
                assert_eq!(issue.rule(), SemanticRule::Eff2);
                assert!(
                    matches!(issue.kind(), SemanticIssueKind::EffectMismatch { missing, .. }
                    if missing.iter().any(|effect| effect == expected)),
                    "{issue:?}"
                );
            });
        }
    }
}

#[test]
fn consuming_a_cell_preserves_its_payloads_origin_directly_and_through_a_helper() {
    let source = r#"fn extract['s](cell: own Box<'s, box<u64>>) -> inner: own box<u64> pure {
  let Box(value: inner) = move cell;
  return move inner;
}

fn observe(value: own box<u64>) -> result: own u64 reads(value) {
  region 'a {
    let store = arena_frame::<64, 16, 'a>();
    region {
      match arena_box(store: &uniq store, value: move value) {
        Ok(value: cell) => {
          let Box(value: inner) = move cell;
          return deref(inner);
        }
        Err(error: back) => {
          return 0_u64;
        }
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  let value = box_new(37_u64);
  let result = observe(value: move value);
  if result != 37_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for source in [
        source.to_owned(),
        source.replace(
            "          let Box(value: inner) = move cell;",
            "          let inner = extract(cell: move cell);",
        ),
    ] {
        assert_complete(source.as_bytes());
        let omitted = source.replace("reads(value)", "pure");
        assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                if missing.iter().any(|effect| effect == "reads(value)"))
        });
    }
}

#[test]
fn front_insertion_cannot_reuse_the_previous_logical_slot_origins() {
    let source = br#"fn shift(slots: own FixedVector<box<u64>, 2>, incoming: own box<u64>) -> result: own box<u64> reads(slots, incoming), writes(slots, incoming) contract {
  requires len_of(slots) == 1_u64;
} {
  let displaced = replace slots[0_u64] = move incoming;
  let first = box_new(7_u64);
  region {
    place_front(vector: &uniq slots, value: move first);
  }
  let both = move slots;
  let empty = box_new(0_u64);
  let recovered = replace both[1_u64] = move empty;
  return move recovered;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // The returned owner is incoming, which moved from logical slot zero to
    // one. The complete effect union is now known, but the returned image
    // must retain an incoming bound instead of selecting the old slot one.
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("established effects allow the bounded result: {outcome:?}");
        };
        let CheckedResultStateOrigin::Finite { formals, .. } =
            &program.data.functions[0].result_state_origin
        else {
            panic!("the transferred suppliers are finite");
        };
        assert!(formals.iter().any(|route| route.parameter == 1));
        assert!(formals.iter().all(|route| {
            route.precision == StateOriginPrecision::Bound
                && route.parameter < 2
                && route.result_fields.is_empty()
        }));
    });
}

#[test]
fn bounded_run_helpers_preserve_imported_sources_and_resolve_fresh_actuals() {
    let helpers = r#"fn pack(value: own box<u64>) -> result: own FixedVector<box<u64>, 1> pure contract {
  ensures len_of(result) == 1_u64;
} {
  let empty = fixed_vector::<box<u64>, 1>();
  region {
    place_back(vector: &uniq empty, value: move value);
  }
  let full = move empty;
  return move full;
}

fn relay(value: own box<u64>) -> result: own FixedVector<box<u64>, 1> pure contract {
  ensures len_of(result) == 1_u64;
} {
  let full = pack(value: move value);
  return move full;
}
"#;
    let fresh = format!(
        r#"{helpers}
command fn main() -> status: own ExitStatus pure {{
  let value = box_new(37_u64);
  let full = relay(value: move value);
  region {{
    let extracted = take_back(vector: &uniq full);
    let empty = move full;
    if deref(extracted) != 37_u64 {{
      return exit_status(code: 1_u8);
    }}
    return exit_status(code: 0_u8);
  }}
}}
"#
    );
    assert_complete(fresh.as_bytes());
    let imported = format!(
        r#"{helpers}
fn observe(value: own box<u64>) -> result: own u64 pure {{
  let full = relay(value: move value);
  region {{
    let extracted = take_back(vector: &uniq full);
    let empty = move full;
    return deref(extracted);
  }}
}}

command fn main() -> status: own ExitStatus pure {{
  let value = box_new(37_u64);
  let observed = observe(value: move value);
  return exit_status(code: 0_u8);
}}
"#
    );
    // Removal establishes the imported source's complete read/write row;
    // the bounded later read cannot wash it away at the helper boundary.
    assert_rule_kind(imported.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing == &["reads(value)", "writes(value)"])
    });
}

#[test]
fn storage_read_out_captures_the_value_image_before_a_helper_call() {
    use super::super::model::{CheckedExpression, CheckedStatement};
    let source = br#"fn keep(value: own box<u64>) -> result: own box<u64> pure {
  return move value;
}

fn carry(slots: own FixedVector<box<u64>, 2>) -> result: own FixedVector<box<u64>, 2> reads(slots), writes(slots) contract {
  requires 0_u64 < len_of(slots);
} {
  set slots[0_u64] = keep(value: move slots[0_u64]);
  return move slots;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the transport itself needs no precise displaced slot: {outcome:?}");
        };
        let function = &program.data.functions[1];
        let CheckedStatement::Set {
            value: CheckedExpression::UserCall { arguments, .. },
            ..
        } = &function.body[0]
        else {
            panic!("expected the ordinary read-out call");
        };
        let CheckedExpression::ReadStorage {
            state_origins: Some(image),
            ..
        } = &arguments[0]
        else {
            panic!("read-out must capture its typed storage's value image");
        };
        assert!(!image.lacks_exact_origins());
        assert!(
            image
                .formals
                .iter()
                .any(|origin| origin.precision == StateOriginPrecision::Exact
                    && origin.source.root == function.parameters[0].declaration)
        );
        assert_eq!(
            image.formals[0].source_value_fields,
            vec![CheckedStateStep::Element(0)]
        );
    });
    // LIV-2 identifies read-out offsets only by equal literals. A bound
    // index is a normative rejection, not a missing origin image.
    let dynamic = std::str::from_utf8(source).unwrap().replace(
        "  set slots[0_u64] = keep(value: move slots[0_u64]);",
        "  let index = 0_u64;\n  set slots[index] = keep(value: move slots[index]);",
    );
    assert_rule_kind(dynamic.as_bytes(), SemanticRule::Type2, |kind| {
        matches!(kind, SemanticIssueKind::AffineElementMove { .. })
    });
}

#[test]
fn fresh_outer_storage_keeps_an_inserted_formal_through_an_extracting_helper() {
    let source = br#"fn extract(storage: own box<box<u64>>) -> previous: own box<u64> reads(storage), writes(storage) {
  let empty = box_new(0_u64);
  let previous = replace deref(storage) = move empty;
  return move previous;
}

fn observe(incoming: own box<u64>) -> value: own u64 reads(incoming), writes(incoming) {
  let empty = box_new(0_u64);
  let storage = box_new(move empty);
  let old = replace deref(storage) = move incoming;
  let previous = extract(storage: move storage);
  return deref(previous);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    let omitted = std::str::from_utf8(source)
        .unwrap()
        .replace("reads(incoming), writes(incoming)", "writes(incoming)");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(incoming)"))
    });
}

#[test]
fn recursive_contained_selectors_remain_an_explicit_finite_summary_gap() {
    let source = br#"enum Chain {
  End(value: box<u64>);
  Link(next: box<Chain>);
}

fn extract(value: own Chain) -> result: own box<u64> reads(value), writes(value) {
  match value {
    End(value: leaf) => {
      return move leaf;
    }
    Link(next: child) => {
      let zero = box_new(0_u64);
      let empty = End(value: move zero);
      let next = replace deref(child) = move empty;
      return extract(value: move next);
    }
  }
}

fn observe(value: own Chain) -> result: own u64 reads(value), writes(value) {
  let leaf = extract(value: move value);
  return deref(leaf);
}

command fn main() -> status: own ExitStatus pure {
  let number = box_new(37_u64);
  let last = End(value: move number);
  let child = box_new(move last);
  let chain = Link(next: move child);
  let result = observe(value: move chain);
  if result != 37_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Unsupported { unsupported } = outcome else {
            panic!("recursive contained selectors need an explicit capability stop: {outcome:?}");
        };
        assert_eq!(
            unsupported.feature(),
            crate::UnsupportedSemanticFeature::OwnerStateRouting
        );
        assert_eq!(
            unsupported.node.components(),
            &[2, 0, 4, 0, 0, 0],
            "the unresolved returned owner stops at observe's deref(leaf), not an unrelated operation"
        );
    });
    // The same recursive type and its directly selected owned payload must
    // still work when this function does not recursively extract a child.
    let direct = std::str::from_utf8(source).unwrap().replace(
        "return extract(value: move next);",
        "let fallback = box_new(37_u64);\n      return move fallback;",
    );
    assert_complete(direct.as_bytes());
}

#[test]
fn whole_unique_borrow_results_preserve_ordinary_box_writeback() {
    for body in [
        "return move value;",
        "return &uniq 'r deref(value);",
        "region {\n    pause(value: &uniq deref(value));\n  }\n  return move value;",
    ] {
        let source = r#"fn pause(value: &uniq box<u64>) -> result: own unit pure {
  return unit;
}

fn alias['r](value: &uniq 'r box<u64>) -> result: &uniq 'r box<u64> pure {
  BODY
}

fn exchange(target: &uniq box<u64>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe_previous(target: &uniq box<u64>, incoming: own box<u64>) -> result: own u64 reads(target), writes(target) {
  let previous = exchange(target: move target, incoming: move incoming);
  let old = deref(previous);
  return old;
}

fn observe(owner: own box<u64>, incoming: own box<u64>) -> result: own u64 reads(owner, incoming), writes(owner) {
  region {
    let holder = alias(value: &uniq owner);
    region {
      let old = observe_previous(target: &uniq deref(holder), incoming: move incoming);
    }
  }
  return deref(owner);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
        .replace("BODY", body);
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("whole unique borrow writeback must check: {outcome:?}");
            };
            let helper = &program.data.functions[3];
            assert_eq!(
                helper.result_state_origin,
                CheckedResultStateOrigin::NoState
            );
            assert_eq!(
                helper.borrowed_state_origins[0].origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(1)]
                }
            );
        });
        // v0.55 permits the original direct form: only the temporary loan
        // ends at the call statement; the region and displaced owner remain.
        let direct_region = source.replace(
            "let old = observe_previous(target: &uniq deref(holder), incoming: move incoming);",
            "let previous = exchange(target: &uniq deref(holder), incoming: move incoming);\n      let old = deref(previous);",
        );
        with_semantics(direct_region.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "the direct displaced-owner read must check: {outcome:?}"
            );
        });
        let omitted_direct = direct_region.replace("reads(owner, incoming)", "reads(owner)");
        assert_rule_kind(omitted_direct.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                if missing.iter().any(|effect| effect == "reads(incoming)"))
        });
        let omitted = source.replace("reads(owner, incoming)", "reads(owner)");
        assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                if missing.iter().any(|effect| effect == "reads(incoming)"))
        });
    }
}

#[test]
fn a_whole_unique_result_does_not_make_an_inexact_actual_exact() {
    let source = br#"struct Holder {
  value: box<u64>;
}

fn field['r](owner: &uniq 'r Holder) -> result: &uniq 'r box<u64> pure {
  return &uniq 'r deref(owner).value;
}

fn identity['r](value: &uniq 'r box<u64>) -> result: &uniq 'r box<u64> pure {
  return move value;
}

fn exchange(target: &uniq box<u64>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe(owner: own Holder, incoming: own box<u64>) -> result: own box<u64> reads(owner.value), writes(owner.value) {
  region {
    let child = field(owner: &uniq owner);
    let forwarded = identity(value: move child);
    return exchange(target: move forwarded, incoming: move incoming);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Unsupported { ref unsupported }
                if unsupported.feature() == crate::UnsupportedSemanticFeature::OwnerStateRouting),
            "identity forwarding cannot certify an inexact actual's location: {outcome:?}"
        );
    });
}

#[test]
fn nested_dereference_is_not_a_returned_reborrow_form() {
    let source = br#"struct Node {
  next: box<Node>;
  value: u64;
}

fn child['r](owner: &uniq 'r box<Node>) -> result: &uniq 'r box<Node> pure {
  return &uniq 'r deref(deref(owner)).next;
}

fn exchange(target: &uniq box<Node>, incoming: own box<Node>) -> previous: own box<Node> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe(owner: own box<Node>, incoming: own box<Node>) -> result: own box<Node> reads(owner), writes(owner) {
  region {
    let descendant = child(owner: &uniq owner);
    return exchange(target: move descendant, incoming: move incoming);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // OWN-14 permits deref(holder) followed by field/subscript suffixes.
    // The extra deref is not such a suffix, so this source cannot witness
    // returned-location routing: it must fail the earlier language rule.
    assert_rule_kind(source, SemanticRule::Own14, |kind| {
        matches!(kind, SemanticIssueKind::InvalidReborrowPosition { .. })
    });
    let invalid_lifetime = std::str::from_utf8(source)
        .expect("ASCII fixture")
        .replace(
            "fn child['r](owner: &uniq 'r box<Node>)",
            "fn child['s](owner: &uniq box<Node>)",
        )
        .replace(
            "-> result: &uniq 'r box<Node>",
            "-> result: &uniq 's box<Node>",
        )
        .replace(
            "return &uniq 'r deref(deref(owner)).next;",
            "return &uniq 's deref(deref(owner)).next;",
        );
    assert_rule_kind(invalid_lifetime.as_bytes(), SemanticRule::Own10, |kind| {
        matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. })
    });
    let suspended = std::str::from_utf8(source)
        .expect("ASCII fixture")
        .replace(
            "fn child['r]",
            "fn identity['r](owner: &uniq 'r box<Node>) -> result: &uniq 'r box<Node> pure {\n  return move owner;\n}\n\nfn child['r]",
        )
        .replace(
            "return &uniq 'r deref(deref(owner)).next;",
            "let selected = identity(owner: &uniq 'r deref(owner));\n  return &uniq 'r deref(deref(owner)).next;",
        );
    assert_rule_kind(suspended.as_bytes(), SemanticRule::Own5, |kind| {
        matches!(kind, SemanticIssueKind::BorrowConflict)
    });
}

#[test]
fn recursive_types_do_not_establish_whole_result_locations() {
    let source = br#"struct Node {
  next: box<Node>;
  value: u64;
}

fn identity['r](owner: &uniq 'r box<Node>) -> result: &uniq 'r box<Node> pure {
  return move owner;
}

fn exchange(target: &uniq box<Node>, incoming: own box<Node>) -> previous: own box<Node> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe(owner: own box<Node>, incoming: own box<Node>) -> result: own box<Node> reads(owner), writes(owner) {
  region {
    let selected = identity(owner: &uniq owner);
    return exchange(target: move selected, incoming: move incoming);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // The declaration-only type walk is deliberately conservative through
    // owning recursion. It does not inspect this identity body for precision.
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Unsupported { ref unsupported }
                if unsupported.feature() == crate::UnsupportedSemanticFeature::OwnerStateRouting),
            "a recursive type does not certify a whole candidate: {outcome:?}"
        );
    });
}

fn root(parameter: u32) -> CheckedResultStatePath {
    CheckedResultStatePath {
        precision: StateOriginPrecision::Exact,
        result_fields: Vec::new(),
        exclusions: Vec::new(),
        parameter,
        parameter_fields: Vec::new(),
    }
}

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "expected acceptance, got {outcome:?}"
        );
    });
}

/// Asserts an EFF-2 release-attributed rejection at the function's effects
/// node, rendering the owner whose release contributed the category.
fn assert_release_mismatch(source: &[u8], owner: &str, located: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected an EFF-2 release mismatch, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::ReleaseEffectMismatch {
                owner: owner.to_owned(),
                mechanical_fix: RELEASE_FIX,
            }
        );
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("EFF-2 must use a source node, got {:?}", issue.location());
        };
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits usize");
        assert_eq!(
            std::str::from_utf8(&source[start..end]),
            std::str::from_utf8(located),
            "the mismatch must locate the effects node"
        );
    });
}

const CANONICAL_ACCEPT: &[u8] = b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CANONICAL_REJECT: &[u8] = b"fn release_read_file(file: own ReadFile) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn the_canonical_release_case_holds_exactly() {
    // A nongeneric function whose only parameter is `own ReadFile` and whose
    // complete body is exactly `return unit;` exhibits `writes(file)`:
    // its whole row is the release contribution of that parameter's
    // compiler-derived close attempt on the function-return edge
    // [EFF-2, STOR-3, SYS-5].
    assert_complete(CANONICAL_ACCEPT);
    // Declaring `pure` is an undeclared-but-exhibited rejection at that
    // function's `effects` node, rendering the owning parameter.
    assert_release_mismatch(CANONICAL_REJECT, "file", b"pure");
}

const BORROWED_ACCEPT: &[u8] = b"fn touch_read_file(file: &ReadFile) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const BORROWED_REJECT: &[u8] = b"fn touch_read_file(file: &ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_borrowed_resource_parameter_contributes_no_release_row() {
    // The exact contrast with the canonical case above, and the whole reason a
    // helper may touch a system value without inheriting its owner's row: the
    // release contribution collects compiler-derived *releases*, and only an
    // owner has one [EFF-2, STOR-3]. The same body under a borrowed parameter
    // is therefore exactly `pure`.
    assert_complete(BORROWED_ACCEPT);
    // A shared loan cannot authorize a state transition. The signature is
    // rejected at EFF-1 before release attribution is considered.
    assert_rule_kind(BORROWED_REJECT, SemanticRule::Eff1, |kind| {
        matches!(kind, SemanticIssueKind::InvalidEffectRow { .. })
    });
}

#[test]
fn over_declaring_the_release_row_rejects_likewise() {
    // Preserve the old test's declared-but-unexhibited direction under the
    // state-row model: Args release is empty and the body performs no
    // state action, so `reads(args)` is an exact over-declaration.
    assert_rule_kind(
        b"fn ignore_arguments(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn file_reservation_and_open_project_only_their_explicit_inputs() {
    assert_complete(
        br#"command fn main(command.cwd as cwd: own DirectoryRead, command.handles as files: own HandleFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    match reserve_handle(factory: &uniq files) {
      Ok(value: permit) => {
        let opened = open_directory_source(permit: move permit, directory: &cwd);
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn file_reservation_projects_the_factory_without_an_open() {
    assert_complete(
        br#"command fn main(command.handles as files: own HandleFactory) -> status: own ExitStatus reads(files), writes(files) {
  region {
    match reserve_handle(factory: &uniq files) {
      Ok(value: permit) => {
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn unused_file_authority_releases_by_logical_consume() {
    assert_complete(
        br#"fn discard_factory(factory: own HandleFactory) -> result: own unit pure {
  return unit;
}

fn discard_permit(permit: own HandlePermit) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn an_immutable_borrowing_helper_names_only_the_snapshot_state() {
    // The local borrow region still does not escape into the row. The new
    // authority component is `reads(args)`, independently of that lifetime.
    assert_complete(
        b"fn count_arguments(args: own Args) -> result: own u64 reads(args) {\n  region {\n    let total = args_count(args: &args);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

const CONDITIONAL_UNION_ACCEPT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> result: own unit writes(outcome) {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

const CONDITIONAL_UNION_REJECT: &[u8] = b"fn dispose_open_outcome(outcome: own Result<ReadFile, IoError>) -> result: own unit pure {\n  match outcome {\n    Ok(value: file) => {\n      return unit;\n    }\n    Err(error: problem) => {\n      return unit;\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn a_release_on_one_match_arm_contributes_its_row() {
    // The release contribution is the union over every normal edge of the
    // conservative structural graph [FN-1]: only the `Ok` arm ever holds a
    // `ReadFile`, and `IoError` has no release action [SYS-5], yet the
    // one-arm release still contributes its exact state write.
    assert_complete(CONDITIONAL_UNION_ACCEPT);
    // Running on only some paths never weakens the contribution: omitting
    // the row is an undeclared-but-exhibited rejection naming the arm
    // binder whose release contributed it.
    assert_release_mismatch(CONDITIONAL_UNION_REJECT, "file", b"pure");
}

#[test]
fn a_pure_contract_member_cannot_bind_a_release_effectful_function() {
    // [FN-3] normalizes state identities and compares `external` and `blocks`
    // by presence: a `pure` member cannot bind a function that exhibits a
    // category only through release.
    assert_rule(
        b"contract Disposer {\n  fn release(file: own ReadFile) -> result: own unit pure;\n}\n\nconform u64: Disposer {\n  release = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn3,
        SemanticIssueKind::IncompatibleConformanceFunction,
    );
    // The same member row binds the same function when both declare the two
    // categories, so the presence comparison admits as well as rejects.
    assert_complete(
        b"contract Disposer {\n  fn release(item: own ReadFile) -> result: own unit writes(item);\n}\n\nconform u64: Disposer {\n  release = release_read_file;\n}\n\nfn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn memory_reclamation_contributes_no_release_row() {
    // The [STOR-3] scope limit, as a facts-off-style regression: a
    // `buffer<T>` drop, a `box<T>` drop, and every frame-resident drop carry
    // the empty release row, so no pre-existing accepted program's legal row
    // changes — these v0.17-legal rows stay exact.
    assert_complete(
        b"fn consume(data: own buffer<u8>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_complete(
        b"command fn main() -> status: own ExitStatus pure {\n  let boxed = box_new(0_u64);\n  let stored = buffer_new(4_u64, 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn release_attribution_is_transitive_over_owned_content() {
    // Release of a value is release of its components [SYS-5]: a
    // `box<ReadFile>` drop frees the box with the empty row and releases the
    // boxed `ReadFile` with its fixed state-release row, so the row is
    // exhibited through the indirection.
    assert_complete(
        b"fn stash(file: own ReadFile) -> result: own unit writes(file) {\n  let boxed = box_new(move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn live_effect_categories_keep_eff1_canonical_order_and_multiplicity() {
    // The replacement keeps the same canonical-order and multiplicity
    // coverage over the live categories: reads, writes, and allocates.
    //
    // `pure` combined with a second category is refused at the same rule but
    // at the earlier stage: `effects := "pure" | effect ("," effect)*` cannot
    // derive it, and regenerating the tables for [S23]'s `allocates` entry
    // tightened the decision that used to admit the bytes and leave the
    // refusal to the checker. The conformance corpus keeps its recorded
    // `reject EFF-1` verdict for the same program either way; what moved is
    // the stage, so this assertion moves with it.
    super::assert_parse_rule(
        b"fn probe(file: own ReadFile) -> result: own unit pure, writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        crate::SyntaxRule::Eff1,
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), writes(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), reads(file) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
}

#[test]
fn user_calls_substitute_state_formals_to_actual_origins() {
    // A moved state keeps its direct formal origin, and the callee's
    // state subject projects back to that caller formal.
    assert_complete(
        b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> result: own unit writes(file) {\n  let moved = move file;\n  release_read_file(file: move moved);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_rule_kind(
        b"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {\n  return unit;\n}\n\nfn forward(file: own ReadFile) -> result: own unit pure {\n  release_read_file(file: move file);\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

const PASS_OUTPUT_PREFIX: &str = r#"fn pass_output(output: own OutputStream) -> result: own OutputStream pure {
  return move output;
}

"#;

fn pass_output_program(effects: &str) -> Vec<u8> {
    format!(
        "{PASS_OUTPUT_PREFIX}command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus {effects} {{\n  let same = pass_output(output: move out);\n  let bytes = buffer_new(1_u64, 65_u8);\n  region {{\n    let written = write_once(output: &uniq same, source: &bytes, start: 0_u64, end: 1_u64);\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

#[test]
fn a_user_result_cannot_wash_an_output_formal_origin() {
    let accepted = pass_output_program("reads(out), writes(out)");
    assert_complete(&accepted);
    let washed = pass_output_program("pure");
    assert_rule_kind(&washed, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
    with_semantics(&accepted, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the pass-through program must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn a_passed_through_read_file_close_keeps_its_release_write() {
    let accepted = br#"fn pass_file(file: own ReadFile) -> result: own ReadFile pure {
  return move file;
}

fn close_after_pass(file: own ReadFile) -> result: own unit writes(file) {
  let same = pass_file(file: move file);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(accepted);
    let rejected = br#"fn pass_file(file: own ReadFile) -> result: own ReadFile pure {
  return move file;
}

fn close_after_pass(file: own ReadFile) -> result: own unit pure {
  let same = pass_file(file: move file);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_release_mismatch(rejected, "same", b"pure");
}

fn choose_output_program(effects: &str, delivered: bool) -> Vec<u8> {
    let chooser = if delivered {
        r#"fn choose_output(left: own OutputStream, right: own OutputStream, take_left: own Bool) -> result: own OutputStream pure {
  let selected = if take_left {
    give move left;
  } else {
    give move right;
  }
  return move selected;
}

fn forward_choice(left: own OutputStream, right: own OutputStream, take_left: own Bool) -> result: own OutputStream pure {
  return choose_output(left: move left, right: move right, take_left: take_left);
}

"#
    } else {
        r#"fn forward_choice(left: own OutputStream, right: own OutputStream, take_left: own Bool) -> result: own OutputStream pure {
  if take_left {
    return move left;
  } else {
    return move right;
  }
}

"#
    };
    format!(
        "{chooser}command fn main(command.stdout as out: own OutputStream, command.stderr as err: own OutputStream) -> status: own ExitStatus {effects} {{\n  let flag = True();\n  let selected = forward_choice(left: move out, right: move err, take_left: flag);\n  let bytes = buffer_new(1_u64, 65_u8);\n  region {{\n    let written = write_once(output: &uniq selected, source: &bytes, start: 0_u64, end: 1_u64);\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

#[test]
fn a_control_flow_result_projects_to_every_possible_formal() {
    let accepted = choose_output_program("reads(out, err), writes(out, err)", false);
    assert_complete(&accepted);
    let narrowed = choose_output_program("reads(out), writes(out)", false);
    assert_rule_kind(&narrowed, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { .. })
    });
    with_semantics(&accepted, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the finite-origin choice must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0), root(1)],
            }
        );
    });
}

#[test]
fn value_if_delivery_and_a_multihop_wrapper_preserve_the_same_formal() {
    let source = br#"fn delivered(output: own OutputStream, first: own Bool) -> result: own OutputStream pure {
  let selected = if first {
    give move output;
  } else {
    give move output;
  }
  return move selected;
}

fn delivered_wrapper(output: own OutputStream, first: own Bool) -> result: own OutputStream pure {
  return delivered(output: move output, first: first);
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let flag = True();
  let selected = delivered_wrapper(output: move out, first: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      let written = write_once(output: &uniq 'o selected, source: &bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_recursive_pass_through_reaches_the_formal_fixed_point() {
    let source = br#"fn recursive_pass(output: own OutputStream, stop: own Bool) -> result: own OutputStream pure {
  if stop {
    return move output;
  } else {
    return recursive_pass(output: move output, stop: stop);
  }
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let flag = True();
  let selected = recursive_pass(output: move out, stop: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      let written = write_once(output: &uniq 'o selected, source: &bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_mutually_recursive_pass_through_reaches_the_same_fixed_point() {
    let source = br#"fn mutual_a(output: own OutputStream, stop: own Bool) -> result: own OutputStream pure {
  if stop {
    return move output;
  } else {
    return mutual_b(output: move output, stop: stop);
  }
}

fn mutual_b(output: own OutputStream, stop: own Bool) -> result: own OutputStream pure {
  return mutual_a(output: move output, stop: stop);
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let flag = True();
  let selected = mutual_b(output: move out, stop: flag);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      let written = write_once(output: &uniq 'o selected, source: &bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_fresh_and_formal_result_join_remains_a_finite_origin_set() {
    let source = br#"fn choose_file(existing: own FileOpenOutcome, permit: own HandlePermit, root: &DirectoryRead, path: &RelativePath, fresh: own Bool) -> result: own FileOpenOutcome reads(permit, root, path), writes(existing, permit) {
  if fresh {
    return open_read(permit: move permit, root: root, path: path);
  } else {
    return move existing;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("fresh/formal origin union must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn an_unclosed_recursive_origin_no_longer_creates_a_language_stop() {
    let source = br#"fn unclosed(output: own OutputStream) -> result: own OutputStream pure {
  return unclosed(output: move output);
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus pure {
  let result = unclosed(output: move out);
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_multi_state_aggregate_releases_each_structural_leaf() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn pass_pair(pair: own Pair) -> result: own Pair pure {
  return move pair;
}

fn release_pair(pair: own Pair) -> result: own unit writes(pair.first, pair.second) {
  let same = pass_pair(pair: move pair);
  return unit;
}

fn pack(first: own ReadFile, second: own ReadFile) -> result: own Pair pure {
  return Pair(first: move first, second: move second);
}

fn dispose_inputs(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
  let pair = pack(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("two structural state leaves must survive an ordinary result move: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(0)],
                        exclusions: Vec::new(),
                        parameter: 0,
                        parameter_fields: vec![CheckedStateStep::Field(0)],
                    },
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(1)],
                        exclusions: Vec::new(),
                        parameter: 0,
                        parameter_fields: vec![CheckedStateStep::Field(1)],
                    },
                ],
            }
        );
    });
}

#[test]
fn ordinary_affine_types_and_embedded_copy_fields_preserve_result_identity() {
    let source = br#"struct Record {
  label: HostString;
  count: u64;
}

fn pass_host(value: own HostString) -> result: own HostString pure {
  return move value;
}

fn pass_path(value: own RelativePath) -> result: own RelativePath pure {
  return move value;
}

fn pass_factory(value: own HandleFactory) -> result: own HandleFactory pure {
  return move value;
}

fn pass_buffer(value: own buffer<u8>) -> result: own buffer<u8> pure {
  return move value;
}

fn pass_record(value: own Record) -> result: own Record pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("ordinary affine pass-throughs must check: {outcome:?}");
        };
        for name in ["pass_host", "pass_path", "pass_factory", "pass_buffer"] {
            let function = program
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing function {name}"));
            assert_eq!(
                function.result_state_origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(0)],
                },
                "{name} must use the ordinary owner-transfer route"
            );
        }
        let record = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "pass_record")
            .expect("pass_record function");
        assert_eq!(
            record.result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(0)],
                        exclusions: Vec::new(),
                        parameter: 0,
                        parameter_fields: vec![CheckedStateStep::Field(0)],
                    },
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(1)],
                        exclusions: Vec::new(),
                        parameter: 0,
                        parameter_fields: vec![CheckedStateStep::Field(1)],
                    },
                ],
            },
            "the copy field remains a structural leaf inside an affine owner"
        );
    });
}

#[test]
fn replace_routes_the_old_field_and_residual_releases_independently() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn replace_first(pair: own Pair, replacement: own ReadFile) -> result: own ReadFile reads(pair.first), writes(pair.first, pair.second, replacement) {
  let previous = replace pair.first = move replacement;
  return move previous;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("field replacement must keep the old and residual owners distinct: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![CheckedResultStatePath {
                    precision: StateOriginPrecision::Exact,
                    result_fields: Vec::new(),
                    exclusions: Vec::new(),
                    parameter: 0,
                    parameter_fields: vec![CheckedStateStep::Field(0)],
                }],
            }
        );
    });
}

#[test]
fn direct_aggregate_construction_releases_every_input_leaf() {
    let accepted = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn release_both(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
  let pair = Pair(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(accepted);
    let missing_second = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn release_both(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first) {
  let pair = Pair(first: move first, second: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_release_mismatch(missing_second, "pair", b"writes(first)");
}

#[test]
fn match_payload_binders_receive_only_the_selected_constructor_payload() {
    let source = br#"enum Choice {
  Left(file: ReadFile);
  Right(file: ReadFile);
}

fn select_left(left: own ReadFile, unrelated: own ReadFile) -> result: own ReadFile writes(unrelated) {
  let choice = Left(file: move left);
  match move choice {
    Left(file: selected) => {
      return move selected;
    }
    Right(file: impossible) => {
      return move impossible;
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!(
                "an unselected payload must not inherit the selected payload's origin: {outcome:?}"
            );
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn a_source_error_still_precedes_state_origin_analysis() {
    let source = br#"struct Pair {
  first: ReadFile;
  second: ReadFile;
}

fn broken(pair: own Pair) -> result: own unit pure {
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(source, SemanticRule::Fn1, SemanticIssueKind::ReturnMismatch);
}

#[test]
fn an_unrelated_loop_does_not_destroy_a_formal_origin() {
    let source = br#"fn through_loop(output: own OutputStream) -> result: own OutputStream pure {
  loop @once {
    break @once;
  }
  return move output;
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let selected = through_loop(output: move out);
  let bytes = buffer_new(1_u64, 65_u8);
  region 'o {
    region {
      let written = write_once(output: &uniq 'o selected, source: &bytes, start: 0_u64, end: 1_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn loop_carried_box_calls_wait_for_their_origin_summary() {
    for body in [
        "for (index in 0_u64..2_u64) {\n    set value = relay(value: move value);\n  }",
        "let index = 0_u64;\n  loop {\n    let done = index == 2_u64;\n    if done {\n      break;\n    }\n    set value = relay(value: move value);\n    set index = index +wrap 1_u64;\n  }",
    ] {
        let source = r#"fn relay(value: own box<u64>) -> result: own box<u64> pure {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  let value = box_new(17_u64);
  BODY
  let observed = deref(value);
  let wrong = observed != 17_u64;
  if wrong {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#
        .replace("BODY", body);
        assert_complete(source.as_bytes());
    }
}

#[test]
fn copy_run_take_preserves_the_remainder_origin_through_loops_and_helpers() {
    for operation in ["take_front", "take_back"] {
        let source = r#"fn take(vector: own FixedVector<u64, 2>) -> (rest: own FixedVector<u64, 2>, value: own u64) reads(vector), writes(vector) contract {
  requires len_of(vector) >= 1_u64;
  ensures len_of(rest) + 1_u64 == len_of(vector);
} {
  region {
    let value = TAKE(vector: &uniq vector);
    let rest = move vector;
    return move rest, value;
  }
}

fn drain(vector: own FixedVector<u64, 2>) -> result: own FixedVector<u64, 2> reads(vector), writes(vector) contract {
  requires len_of(vector) == 2_u64;
} {
  for (
    index in 0_u64..2_u64,
    invariant left: len_of(vector) + index >= 2_u64
  ) {
    set (vector, observed) = take(vector: move vector);
  }
  return move vector;
}

fn observe(vector: own FixedVector<u64, 2>) -> result: own u64 reads(vector) {
  return len_of(vector);
}

fn after_drain(vector: own FixedVector<u64, 2>) -> result: own u64 reads(vector), writes(vector) contract {
  requires len_of(vector) == 2_u64;
} {
  let empty = drain(vector: move vector);
  return observe(vector: move empty);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
        .replace("TAKE", operation);
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("copy run take must preserve its remaining owner: {outcome:?}");
            };
            let mut rest = root(0);
            rest.result_fields = vec![CheckedStateStep::Field(0)];
            assert_eq!(
                program.data.functions[0].result_state_origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![rest],
                }
            );
            assert_eq!(
                program.data.functions[1].result_state_origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(0)],
                }
            );
        });
    }
}

#[test]
fn affine_run_take_does_not_use_the_copy_result_shortcut() {
    for operation in ["take_front", "take_back"] {
        let source = r#"fn take(vector: own FixedVector<box<u64>, 1>) -> (rest: own FixedVector<box<u64>, 1>, value: own box<u64>) reads(vector), writes(vector) contract {
  requires len_of(vector) >= 1_u64;
} {
  region {
    let value = TAKE(vector: &uniq vector);
    let rest = move vector;
    return move rest, move value;
  }
}

fn observe(vector: own FixedVector<box<u64>, 1>) -> result: own u64 reads(vector), writes(vector) contract {
  requires len_of(vector) >= 1_u64;
} {
  region {
    let taken = TAKE(vector: &uniq vector);
    let rest = move vector;
    return deref(taken);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
        .replace("TAKE", operation);
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("the first take establishes all later possible effects: {outcome:?}");
            };
            let mut rest = root(0);
            rest.precision = StateOriginPrecision::Bound;
            rest.result_fields = vec![CheckedStateStep::Field(0)];
            let mut value = root(0);
            value.precision = StateOriginPrecision::Bound;
            value.result_fields = vec![CheckedStateStep::Field(1)];
            assert_eq!(
                program.data.functions[0].result_state_origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![rest, value],
                }
            );
        });
    }
}

#[test]
fn changing_loop_origins_do_not_hide_a_later_iterations_read() {
    let source =
        br#"fn cycle(first: own box<u64>, second: own box<u64>) -> result: own unit reads(first), writes(first, second) {
  for (iteration in 0_u64..2_u64) {
    let observed = deref(first);
    set (first, second) = move second, move first;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, extra, .. }
            if missing == &["reads(second)"] && extra.is_empty())
    });
    let complete = std::str::from_utf8(source)
        .unwrap()
        .replace("reads(first)", "reads(first, second)");
    assert_complete(complete.as_bytes());
}

#[test]
fn counted_exhaustion_retains_entry_and_backedge_sources() {
    let source = br#"fn after(first: own box<u64>, second: own box<u64>, count: own u64) -> result: own u64 reads(first), writes(first, second) {
  for (iteration in 0_u64..count) {
    set (first, second) = move second, move first;
  }
  return deref(first);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, extra, .. }
            if missing == &["reads(second)"] && extra.is_empty())
    });
    let complete = std::str::from_utf8(source)
        .unwrap()
        .replace("reads(first)", "reads(first, second)");
    assert_complete(complete.as_bytes());
}

#[test]
fn a_loop_break_join_retains_the_two_origins_an_update_can_select() {
    let source = br#"fn loop_choice(selected: own Result<ReadFile, IoError>, factory: own HandleFactory, root: own DirectoryRead, path: &RelativePath, refresh: own Bool) -> result: own Result<ReadFile, IoError> reads(selected, factory, root, path), writes(selected, factory, root) {
  loop @once {
    if refresh {
      region {
        match reserve_handle(factory: &uniq factory) {
          Ok(value: permit) => {
            region {
              match open_read(permit: move permit, root: &root, path: path) {
                FileOpened(value: got) => {
                  let discarded = replace selected = Ok<ReadFile, IoError>(value: move got);
                }
                FileOpenFailed(error: problem, permit: refused) => {
                  let discarded = replace selected = Err<ReadFile, IoError>(error: move problem);
                }
              }
            }
          }
          Err(error: spent) => {
            return Err<ReadFile, IoError>(error: move spent);
          }
        }
      }
    }
    break @once;
  }
  return move selected;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("loop origin update must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)],
            }
        );
    });
}

#[test]
fn an_optional_state_result_can_prove_that_its_only_route_is_absent() {
    let source = br#"fn no_file() -> result: own Result<ReadFile, IoError> pure {
  let problem = Other(code: 0_u32, origin: 0_u8);
  return Err<ReadFile, IoError>(error: move problem);
}

command fn main() -> status: own ExitStatus pure {
  let result = no_file();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a proved Err-only optional state must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: Vec::new(),
            }
        );
    });
}

#[test]
fn an_optional_result_projects_its_present_formal_and_keeps_its_absent_route() {
    let source = br#"fn maybe_output(output: own OutputStream, present: own Bool) -> result: own Result<OutputStream, IoError> pure {
  if present {
    return Ok<OutputStream, IoError>(value: move output);
  } else {
    let problem = Other(code: 0_u32, origin: 0_u8);
    return Err<OutputStream, IoError>(error: move problem);
  }
}

command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out) {
  let flag = True();
  match maybe_output(output: move out, present: flag) {
    Ok(value: selected) => {
      let bytes = buffer_new(1_u64, 65_u8);
      region 'o {
        region {
          let written = write_once(output: &uniq 'o selected, source: &bytes, start: 0_u64, end: 1_u64);
        }
      }
    }
    Err(error: problem) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("optional formal origin must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![CheckedResultStatePath {
                    precision: StateOriginPrecision::Exact,
                    result_fields: vec![CheckedStateStep::VariantField {
                        variant: 0,
                        field: 0
                    }],
                    exclusions: Vec::new(),
                    parameter: 0,
                    parameter_fields: Vec::new(),
                }],
            }
        );
    });
}

#[test]
fn a_copy_only_parameter_is_a_valid_path_but_must_be_exhibited() {
    assert_rule_kind(
        b"fn probe(value: own u64) -> result: own unit reads(value) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn external_and_blocks_are_ordinary_function_and_parameter_names() {
    assert_complete(
        b"fn external(blocks: own Args) -> result: own u64 reads(blocks) {\n  region {\n    let total = args_count(args: &blocks);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn borrowing_one_owned_struct_field_projects_only_that_field_effect() {
    assert_complete(
        br#"struct Pair {
  first: buffer<u8>;
  second: buffer<u8>;
}

fn length(value: &buffer<u8>) -> result: own u64 reads(value) {
  return len_of(deref(value));
}

fn read_second(pair: own Pair) -> result: own unit reads(pair.second) {
  region {
    let count = length(value: &pair.second);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn general_elements_retain_deep_resource_release_effects() {
    let accepted = b"fn release_files(files: own FixedVector<FixedVector<FixedVector<ReadFile, 2>, 2>, 2>) -> result: own unit writes(files) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    let rejected = b"fn release_files(files: own FixedVector<FixedVector<FixedVector<ReadFile, 2>, 2>, 2>) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    assert_complete(accepted);
    assert_release_mismatch(rejected, "files", b"pure");
}

const RESOURCE_FIELD_BORROW: &str = r#"struct Holder {
  before: u64;
  file: ReadFile;
  after: u64;
}

struct Nested {
  before: u64;
  holder: Holder;
  after: u64;
}

fn exchange(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  let displaced = replace deref(target) = move incoming;
  return move displaced;
}

fn inspect(holder: own Holder, incoming: own ReadFile) -> result: own Holder reads(holder.file), writes(holder.file) {
  region {
    let previous = exchange(target: &uniq holder.file, incoming: move incoming);
  }
  return move holder;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn resource_field_borrow_projects_only_the_selected_state() {
    assert_complete(RESOURCE_FIELD_BORROW.as_bytes());
}

#[test]
fn resource_field_borrow_projects_a_nested_selected_state() {
    let source = RESOURCE_FIELD_BORROW
        .replace("holder: own Holder", "holder: own Nested")
        .replace("result: own Holder", "result: own Nested")
        .replace("holder.file", "holder.holder.file");
    assert_complete(source.as_bytes());
}

#[test]
fn resource_field_borrow_child_projects_only_the_selected_state() {
    let source = RESOURCE_FIELD_BORROW
        .replace("holder: own Holder", "holder: &uniq Holder")
        .replace("result: own Holder", "result: own unit")
        .replace("&uniq holder.file", "&uniq deref(holder).file")
        .replace("return move holder;", "return unit;");
    assert_complete(source.as_bytes());
}

#[test]
fn resource_field_borrow_cannot_omit_its_selected_read_or_write() {
    for row in ["reads(holder.file)", "writes(holder.file)", "pure"] {
        let source = RESOURCE_FIELD_BORROW.replace("reads(holder.file), writes(holder.file)", row);
        assert_rule_kind(source.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { .. })
        });
    }
}

#[test]
fn resource_field_borrow_child_projects_a_nested_selected_state() {
    let source = RESOURCE_FIELD_BORROW
        .replace("holder: own Holder", "holder: &uniq Nested")
        .replace("result: own Holder", "result: own unit")
        .replace("holder.file", "holder.holder.file")
        .replace(
            "&uniq holder.holder.file",
            "&uniq deref(holder).holder.file",
        )
        .replace("return move holder;", "return unit;");
    assert_complete(source.as_bytes());
}

#[test]
fn resource_field_borrow_rejects_unobserved_sibling_or_whole_root_rows() {
    for row in [
        "reads(holder.before, holder.file), writes(holder.file)",
        "reads(holder.file), writes(holder.file, holder.after)",
        "reads(holder), writes(holder)",
    ] {
        let source = RESOURCE_FIELD_BORROW.replace("reads(holder.file), writes(holder.file)", row);
        assert_rule_kind(source.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { .. })
        });
    }
}

#[test]
fn resource_field_borrow_preserves_a_whole_resource_root() {
    let source = RESOURCE_FIELD_BORROW
        .replace("holder: own Holder", "holder: own ReadFile")
        .replace("result: own Holder", "result: own ReadFile")
        .replace("holder.file", "holder");
    assert_complete(source.as_bytes());
}

#[test]
fn resource_field_borrow_projects_the_returned_displaced_owner_summary() {
    let source = RESOURCE_FIELD_BORROW
        .replace("holder: own Holder", "holder: &uniq Holder")
        .replace("result: own Holder", "result: own ReadFile")
        .replace(
            "let previous = exchange(target: &uniq holder.file, incoming: move incoming);",
            "return exchange(target: &uniq deref(holder).file, incoming: move incoming);",
        )
        .replace("  return move holder;\n", "");
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the selected field must retain its callable result origin: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[1].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![CheckedResultStatePath {
                    precision: StateOriginPrecision::Exact,
                    result_fields: Vec::new(),
                    exclusions: Vec::new(),
                    parameter: 0,
                    parameter_fields: vec![CheckedStateStep::Field(1)],
                }],
            }
        );
    });
}

#[test]
fn ordinary_box_owner_transfer_preserves_incoming_reads_across_helpers() {
    let legacy = r#"fn exchange(target: &uniq box<u64>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe(owner: own box<u64>, incoming: own box<u64>) -> result: own u64 reads(owner, incoming), writes(owner) {
  region {
    let previous = exchange(target: &uniq owner, incoming: move incoming);
  }
  return deref(owner);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let modern = r#"fn exchange['s](target: &uniq Box<'s, u64>, incoming: own Box<'s, u64>) -> previous: own Box<'s, u64> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn observe['s](owner: own Box<'s, u64>, incoming: own Box<'s, u64>, store: &uniq Heap<'s>) -> result: own u64 reads(owner, incoming), writes(owner, store) {
  region {
    let previous = exchange(target: &uniq owner, incoming: move incoming);
  }
  return deref(owner);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for helper in [legacy, modern] {
        let direct = helper.replace(
            "  region {\n    let previous = exchange(target: &uniq owner, incoming: move incoming);\n  }",
            "  let previous = replace owner = move incoming;",
        );
        for source in [helper, direct.as_str()] {
            assert_complete(source.as_bytes());
            let omitted = source.replace("reads(owner, incoming)", "reads(owner)");
            assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
                matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                    if missing.iter().any(|effect| effect == "reads(incoming)"))
            });
        }
    }
}

const OWNER_WRITEBACK: &str = r#"fn exchange(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn install(target: &uniq ReadFile, incoming: own ReadFile) -> result: own unit reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return unit;
}

fn release_all(file: own ReadFile, first: own ReadFile, second: own ReadFile) -> result: own unit reads(file, first), writes(file, first, second) {
  region {
    let previous = exchange(target: &uniq file, incoming: move first);
  }
  region {
    let installed = install(target: &uniq file, incoming: move second);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  doc "FN-1, EFF-2, SET-2: two exclusive-borrowed exchanges leave successive incoming owners in the same location. The first returned owner is the original file, the second helper releases first and returns unit, and the caller releases second. Both result and exclusive-storage exit state use the entry image; an empty result component cannot erase writeback. These uncalled declarations test checked callable composition without opening host files.";
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn owner_writeback_tracks_two_exchanges_and_a_unit_result() {
    with_semantics(OWNER_WRITEBACK.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("joint exit routes must check: {outcome:?}");
        };
        for function in &program.data.functions[..2] {
            assert_eq!(function.borrowed_state_origins.len(), 1);
            assert_eq!(function.borrowed_state_origins[0].parameter, 0);
            assert_eq!(
                function.borrowed_state_origins[0].origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(1)]
                }
            );
        }
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)]
            }
        );
        assert_eq!(
            program.data.functions[1].result_state_origin,
            CheckedResultStateOrigin::NoState
        );
    });
}

#[test]
fn owner_writeback_cannot_hide_the_installed_owners_release() {
    let source = OWNER_WRITEBACK.replace("writes(file, first, second)", "writes(file, first)");
    assert_rule_kind(source.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(
            kind,
            SemanticIssueKind::EffectMismatch { .. }
                | SemanticIssueKind::ReleaseEffectMismatch { .. }
        )
    });
}

#[test]
fn owner_writeback_follows_nested_reborrowed_fields() {
    let source = OWNER_WRITEBACK.replace("fn release_all(", "struct Pair {\n  before: u64;\n  file: ReadFile;\n  after: u64;\n}\n\nstruct Outer {\n  pair: Pair;\n}\n\nfn release_all(")
        .replace("file: own ReadFile, first:", "file: &uniq Outer, first:")
        .replace("reads(file, first), writes(file, first, second)", "reads(file.pair.file, first), writes(file.pair.file, first)")
        .replace("&uniq file,", "&uniq deref(file).pair.file,");
    assert_complete(source.as_bytes());
    let rejected = source.replace("writes(file.pair.file, first)", "writes(file.pair.file)");
    assert_rule_kind(rejected.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(
            kind,
            SemanticIssueKind::EffectMismatch { .. }
                | SemanticIssueKind::ReleaseEffectMismatch { .. }
        )
    });
}

#[test]
fn owner_writeback_accepts_whole_replacement_through_a_generic_unique_parameter() {
    let source = br#"fn exchange<T: affine>(target: &uniq T, incoming: own T) -> previous: own T reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn copy_instance(value: &uniq u64) -> result: own u64 reads(value), writes(value) {
  region {
    let old = exchange::<u64>(target: &uniq deref(value), incoming: 7_u64);
  }
  return deref(value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn owner_writeback_whole_affine_replacement_agrees_with_direct_code() {
    let source = br#"struct Cell {
  count: u64;
  owner: box<u64>;
}

fn fresh_cell() -> result: own Cell pure {
  let owner = box_new(0_u64);
  return Cell(count: 0_u64, owner: move owner);
}

fn exchange(target: &uniq Cell, incoming: own Cell) -> previous: own Cell reads(target.count, target.owner), writes(target.count, target.owner) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn direct(cell: own Cell) -> result: own Cell reads(cell.count, cell.owner), writes(cell.count, cell.owner) {
  let fresh = fresh_cell();
  let previous = replace cell = move fresh;
  return move cell;
}

fn indirect(cell: own Cell) -> result: own Cell reads(cell.count, cell.owner), writes(cell.count, cell.owner) {
  let fresh = fresh_cell();
  region {
    let previous = exchange(target: &uniq cell, incoming: move fresh);
  }
  return move cell;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("whole-value transfer must check: {outcome:?}");
        };
        for function in &program.data.functions[2..4] {
            assert_eq!(
                function.result_state_origin,
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: Vec::new()
                }
            );
        }
    });
}

#[test]
fn owner_writeback_copy_field_assignment_preserves_the_existing_aggregate() {
    let source = br#"struct Cell {
  count: u64;
  owner: box<u64>;
}

fn set_count(cell: &uniq Cell) -> result: own unit writes(cell.count) {
  set deref(cell).count = 7_u64;
  return unit;
}

fn change(cell: own Cell) -> result: own Cell writes(cell.count) {
  region {
    let changed = set_count(cell: &uniq cell);
  }
  return move cell;
}

fn repack(cell: &Cell) -> result: own Cell reads(cell.count) {
  let count = deref(cell).count;
  let owner = box_new(0_u64);
  return Cell(count: count, owner: move owner);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("copy and aggregate identities must differ: {outcome:?}");
        };
        let expected = vec![0, 1]
            .into_iter()
            .map(|field| CheckedResultStatePath {
                precision: StateOriginPrecision::Exact,
                result_fields: vec![CheckedStateStep::Field(field)],
                exclusions: Vec::new(),
                parameter: 0,
                parameter_fields: vec![CheckedStateStep::Field(field)],
            })
            .collect();
        assert_eq!(
            program.data.functions[1].result_state_origin,
            CheckedResultStateOrigin::Finite {
                formals: expected,
                run_lengths: Default::default()
            }
        );
        assert_eq!(
            program.data.functions[2].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: Vec::new()
            }
        );
    });
}

#[test]
fn owner_writeback_evaluate_and_discarded_result_keep_call_updates() {
    for source in [
        OWNER_WRITEBACK.replace("let installed = ", ""),
        OWNER_WRITEBACK.replace("let previous = exchange", "exchange"),
    ] {
        assert_complete(source.as_bytes());
    }
}

#[test]
fn owner_writeback_keeps_the_release_capability_requirement_for_dispose() {
    let source = OWNER_WRITEBACK.replace(
        "incoming: move first);",
        "incoming: move first);\n    dispose previous;",
    );
    assert_rule_kind(source.as_bytes(), SemanticRule::Prov6, |kind| {
        matches!(kind, SemanticIssueKind::DisposeWithoutCapabilityLeaf { .. })
    });
}

#[test]
fn owner_writeback_substitutes_disjoint_outputs_from_one_entry_image() {
    let source = br#"struct Pair {
  left: ReadFile;
  right: ReadFile;
}

fn rotate(left: &uniq ReadFile, right: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(left, right), writes(left, right) {
  let old_left = replace deref(left) = move incoming;
  let old_right = replace deref(right) = move old_left;
  return move old_right;
}

fn apply(pair: own Pair, incoming: own ReadFile) -> result: own Pair reads(pair.left, pair.right), writes(pair.left, pair.right) {
  region {
    let previous = rotate(left: &uniq pair.left, right: &uniq pair.right, incoming: move incoming);
  }
  return move pair;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("simultaneous disjoint substitution must check: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0]
                .borrowed_state_origins
                .iter()
                .map(|image| image.origin.clone())
                .collect::<Vec<_>>(),
            vec![
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(2)]
                },
                CheckedResultStateOrigin::Finite {
                    run_lengths: Default::default(),
                    formals: vec![root(0)]
                }
            ]
        );
        assert_eq!(
            program.data.functions[1].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(0)],
                        exclusions: Vec::new(),
                        parameter: 1,
                        parameter_fields: Vec::new()
                    },
                    CheckedResultStatePath {
                        precision: StateOriginPrecision::Exact,
                        result_fields: vec![CheckedStateStep::Field(1)],
                        exclusions: Vec::new(),
                        parameter: 0,
                        parameter_fields: vec![CheckedStateStep::Field(0)]
                    },
                ]
            }
        );
    });
}

#[test]
fn owner_writeback_affine_generic_own_exchange_supports_copy_instantiation() {
    let source = br#"fn exchange_owned<T: affine>(target: own T, incoming: own T) -> (current: own T, previous: own T) reads(target), writes(target) {
  let previous = replace target = move incoming;
  return move target, move previous;
}

command fn main() -> status: own ExitStatus pure {
  let (current, previous) = exchange_owned::<u64>(target: 1_u64, incoming: 2_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn owner_writeback_recursive_exchange_reaches_a_joint_fixed_point() {
    let source = br#"fn recursive_exchange(target: &uniq ReadFile, incoming: own ReadFile, stop: own Bool) -> previous: own ReadFile reads(target), writes(target) {
  if stop {
    let previous = replace deref(target) = move incoming;
    return move previous;
  } else {
    region {
      return recursive_exchange(target: &uniq deref(target), incoming: move incoming, stop: stop);
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("joint recursion must reach its finite routes: {outcome:?}");
        };
        assert_eq!(
            program.data.functions[0].result_state_origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(0)]
            }
        );
        assert_eq!(
            program.data.functions[0].borrowed_state_origins[0].origin,
            CheckedResultStateOrigin::Finite {
                run_lengths: Default::default(),
                formals: vec![root(1)]
            }
        );
    });
}

#[test]
fn whole_unique_resource_result_preserves_owner_writeback() {
    let source = br#"fn alias['r](value: &uniq 'r ReadFile) -> result: &uniq 'r ReadFile pure {
  return &uniq 'r deref(value);
}

fn exchange(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn apply(file: own ReadFile, incoming: own ReadFile) -> result: own unit reads(file), writes(file, incoming) {
  region {
    let holder = alias(value: &uniq file);
    region {
      let previous = exchange(target: &uniq deref(holder), incoming: move incoming);
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // This unchanged source has a unique, same-typed whole candidate, not a
    // selected subplace. The field/recursive controls above retain the
    // distinction between an inexact ceiling and an exact returned location.
    assert_complete(source);
}

#[test]
fn an_uncalled_nonreturning_box_helper_needs_no_chosen_origin_encoding() {
    // FN-1 describes returned values, not the analysis representation of a
    // function that never returns one. This source supplies no counterexample
    // to an empty may-origin summary; the returning controls below do.
    let source = br#"fn unclosed(value: own box<u64>) -> result: own box<u64> pure {
  let next = unclosed(value: move value);
  return move next;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_returning_recursive_box_helper_preserves_the_input_read_effect() {
    let source = r#"fn recur(value: own box<u64>, stop: own Bool) -> result: own box<u64> pure {
  if stop {
    return move value;
  } else {
    let done = True();
    let next = recur(value: move value, stop: done);
    return move next;
  }
}

fn observe(value: own box<u64>, stop: own Bool) -> result: own u64 reads(value) {
  let returned = recur(value: move value, stop: stop);
  return deref(returned);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
    let omitted = source.replace("reads(value)", "pure");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(value)"))
    });
}

#[test]
fn a_nonreturning_box_helper_keeps_its_structural_read_effect() {
    let source = r#"fn unclosed(value: own box<u64>) -> result: own box<u64> reads(value) {
  let observed = deref(value);
  let next = unclosed(value: move value);
  return move next;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
    let omitted = source.replace("reads(value)", "pure");
    assert_rule_kind(omitted.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
            if missing.iter().any(|effect| effect == "reads(value)"))
    });
}

#[test]
fn ordinary_displaced_box_result_keeps_the_extracted_owners_origin() {
    let source = br#"fn exchange(target: &uniq box<box<u64>>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let previous = replace deref(deref(target)) = move incoming;
  return move previous;
}

fn observe(owner: own box<box<u64>>, incoming: own box<u64>) -> result: own u64 reads(owner), writes(owner) {
  region {
    let previous = exchange(target: &uniq owner, incoming: move incoming);
    return deref(previous);
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    let spurious = std::str::from_utf8(source)
        .unwrap()
        .replace("reads(owner)", "reads(owner, incoming)");
    assert_rule_kind(spurious.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra.iter().any(|effect| effect == "reads(incoming)"))
    });
}

#[test]
fn ordinary_displaced_run_keeps_its_origin_and_inherent_capacity() {
    let helper = br#"fn extract(target: own box<FixedVector<u64, 0>>, incoming: own FixedVector<u64, 0>) -> previous: own FixedVector<u64, 0> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn convert(owner: own box<FixedVector<u64, 0>>, incoming: own FixedVector<u64, 0>) -> result: own array<u64, 0> reads(owner), writes(owner) {
  let previous = extract(target: move owner, incoming: move incoming);
  return array_from_fixed(vector: move previous);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(helper);
    let not_always_full = std::str::from_utf8(helper)
        .unwrap()
        .replace("FixedVector<u64, 0>", "FixedVector<u64, 1>")
        .replace("array<u64, 0>", "array<u64, 1>");
    assert_rule_kind(not_always_full.as_bytes(), SemanticRule::Blk0, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedKernelRequirement(_))
    });
    assert_complete(br#"fn convert(owner: own box<FixedVector<u64, 0>>, incoming: own FixedVector<u64, 0>) -> result: own array<u64, 0> reads(owner), writes(owner) {
  let previous = replace deref(owner) = move incoming;
  return array_from_fixed(vector: move previous);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#);
}

#[test]
fn loop_origin_sets_ignore_route_enumeration_order() {
    // Ordinary owning array elements already admit this transfer. Repeated
    // movement through the held slot exposes the same possible sources in
    // a different traversal order at the arbitrary backedge.
    let source = r#"struct Resource {
  id: u64;
  data0: u64;
  data1: u64;
  data2: u64;
  data3: u64;
}

enum Slot['s] {
  Vacant();
  Deleted();
  Occupied(fingerprint: u8, key: u64, payload: Box<'s, Resource>);
}

struct Progress['s] {
  held: Slot<'s>;
  source_next: u64;
  next_probe: u64;
  phase: u8;
}

fn occupied['s](slot: &Slot<'s>) -> full: own Bool reads(slot) {
  match deref(slot) {
    Occupied(fingerprint: tag, key: present, payload: owner) => {
      return True();
    }
    Vacant() => {
      return False();
    }
    Deleted() => {
      return False();
    }
  }
}

fn slot_key['s](slot: &Slot<'s>) -> key: own u64 reads(slot) {
  match deref(slot) {
    Occupied(fingerprint: tag, key: present, payload: owner) => {
      return deref(present);
    }
    Vacant() => {
      return 0_u64;
    }
    Deleted() => {
      return 0_u64;
    }
  }
}

fn probe_index(home: own u64, step: own u64, count: own u64) -> index: own u64 pure contract {
  requires home < count;
  requires step < count;
  ensures index < count;
} {
  let until_end = count - home;
  if step >= until_end {
    let wrapped = step - until_end;
    return wrapped;
  }
  let straight = home + step;
  return straight;
}

fn advance['s](store: &uniq Heap<'s>, source: &uniq array<Slot<'s>, 4>, target: &uniq array<Slot<'s>, 4>, progress: &uniq Progress<'s>, budget: own u64) -> (examined: own u64, moved: own u64) reads(source, target, progress.held, progress.source_next, progress.next_probe, progress.phase), writes(store, source, target, progress.held, progress.source_next, progress.next_probe, progress.phase) {
  let inspected = 0_u64;
  let transfers = 0_u64;
  if deref(progress).phase < 2_u8 {
    for (
      tick in 0_u64..budget,
      invariant work_min: inspected >= tick,
      invariant work_max: inspected <= tick,
      invariant moved_bound: transfers <= inspected
    ) {
      if deref(progress).phase == 0_u8 {
        let count = len_of(deref(source));
        if deref(progress).source_next >= count {
          set deref(progress).phase = 3_u8;
          break;
        }
        let index = deref(progress).source_next;
        set deref(progress).source_next = deref(progress).source_next + 1_u64;
        region {
          let full = occupied(slot: &deref(source)[index]);
          if full {
            let deleted = Deleted<'s>();
            let held_slot = replace deref(source)[index] = move deleted;
            let previous_held = replace deref(progress).held = move held_slot;
            dispose previous_held;
            set deref(progress).next_probe = 0_u64;
            set deref(progress).phase = 1_u8;
          }
        }
      } else {
        let count = len_of(deref(target));
        if deref(progress).next_probe >= count {
          set deref(progress).phase = 2_u8;
          break;
        }
        region {
          let key = slot_key(slot: &deref(progress).held);
          let home = key % count;
          let index = probe_index(home: home, step: deref(progress).next_probe, count: count);
          set deref(progress).next_probe = deref(progress).next_probe + 1_u64;
          let full = occupied(slot: &deref(target)[index]);
          if full {
          } else {
            let vacant = Vacant<'s>();
            let held_slot = replace deref(progress).held = move vacant;
            let previous = replace deref(target)[index] = move held_slot;
            dispose previous;
            set transfers = transfers + 1_u64;
            set deref(progress).phase = 0_u8;
          }
        }
      }
      set inspected = inspected + 1_u64;
    }
    if deref(progress).phase == 0_u8 {
      let source_count = len_of(deref(source));
      if deref(progress).source_next == source_count {
        set deref(progress).phase = 3_u8;
      }
    }
  }
  return inspected, transfers;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
}
