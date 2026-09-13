//! Ordinary storage-place effects, ownership transfer and run proofs.
//!
//! C2 removes value-history routing and opaque implicit release. These tests
//! retain the ordinary source assertions; CASES.md records retired metadata
//! assertions and the rule that removes their meaning.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::{assert_rule_kind, with_semantics};

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "ordinary source must check: {outcome:?}"
        );
    });
}

#[test]
fn memory_reclamation_contributes_no_release_row() {
    assert_complete(
        b"fn consume(data: own buffer<u8>) -> result: own unit pure {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert_complete(
        b"fn main() -> status: own ExitStatus pure {\n  let boxed = box_new(0_u64);\n  let stored = buffer_new(4_u64, 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
    );
}

#[test]
fn live_effect_categories_keep_eff1_canonical_order_and_multiplicity() {
    super::assert_parse_rule(
        b"fn probe(file: own ReadFile) -> result: own unit pure, writes(file) {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        crate::SyntaxRule::Gram2,
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), writes(file) {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
    assert_rule_kind(
        b"fn probe(file: own ReadFile) -> result: own unit writes(file), reads(file) {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff1,
        |kind| matches!(kind, SemanticIssueKind::InvalidEffectRow { .. }),
    );
}

#[test]
fn a_copy_only_parameter_is_a_valid_path_but_must_be_exhibited() {
    assert_rule_kind(
        b"fn probe(value: own u64) -> result: own unit reads(value) {\n  return unit;\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn external_and_blocks_are_ordinary_function_and_parameter_names() {
    assert_complete(
        b"fn external(blocks: own Args) -> result: own u64 reads(blocks) {\n  region {\n    let total = args_count(args: &blocks);\n    return total;\n  }\n}\n\nfn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
    );
}

#[test]
fn owner_writeback_affine_generic_own_exchange_supports_copy_instantiation() {
    let source = br#"fn exchange_owned<T: affine>(target: own T, incoming: own T) -> (current: own T, previous: own T) reads(target), writes(target) {
  let previous = replace target = move incoming;
  return move target, move previous;
}

fn main() -> status: own ExitStatus pure {
  let (current, previous) = exchange_owned::<u64>(target: 1_u64, incoming: 2_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn an_uncalled_recursive_owner_transfer_needs_no_routing_summary() {
    let source = br#"fn unclosed(value: own box<u64>) -> result: own box<u64> pure {
  let next = unclosed(value: move value);
  return move next;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_nonreturning_box_helper_keeps_its_structural_read_effect() {
    let source = r#"fn unclosed(value: own box<u64>) -> result: own box<u64> reads(value) {
  let observed = deref(value);
  let next = unclosed(value: move value);
  return move next;
}

fn main() -> status: own ExitStatus pure {
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
fn ordinary_displaced_box_result_uses_only_target_storage_effects() {
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

fn main() -> status: own ExitStatus pure {
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
fn ordinary_displaced_run_keeps_its_inherent_capacity() {
    let helper = br#"fn extract(target: own box<FixedVector<u64, 0>>, incoming: own FixedVector<u64, 0>) -> previous: own FixedVector<u64, 0> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn convert(owner: own box<FixedVector<u64, 0>>, incoming: own FixedVector<u64, 0>) -> result: own array<u64, 0> reads(owner), writes(owner) {
  let previous = extract(target: move owner, incoming: move incoming);
  return array_from_fixed(vector: move previous);
}

fn main() -> status: own ExitStatus pure {
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

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#);
}

#[test]
fn owning_sparse_rehash_keeps_ordinary_loop_proofs() {
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

fn main['heap](heap: own Heap<'heap>) -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
}

#[test]
fn run_length_joins_and_counted_rotation_need_no_owner_summary() {
    let source = br#"fn choose(first: own box<u64>, second: own box<u64>, extra: own Bool) -> result: own FixedVector<box<u64>, 1000000000> pure {
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

fn rotate(value: own box<u64>) -> result: own FixedVector<box<u64>, 1000000000> pure {
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

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn recursive_exclusive_replacement_returns_both_linear_owners() {
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

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn affine_opaque_drop_and_linear_transfer_use_the_ordinary_rules() {
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/pre1-pos-affine-opaque-empty-drop.wf"
    ));
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/pre1-pos-linear-opaque-ordinary-transfer.wf"
    ));
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/accept-sysrelease-return-unit-declared.wf"
    ));
    assert_rule_kind(
        include_bytes!("../../../../tests/conformance/cases/reject-syseff-return-unit-pure.wf"),
        SemanticRule::Prov6,
        |_| true,
    );
}

#[test]
fn an_explicit_close_on_one_arm_contributes_ordinary_factory_effects() {
    assert_complete(include_bytes!(
        "../../../../tests/conformance/cases/accept-syseff-conditional-release-union.wf"
    ));
    assert_rule_kind(
        include_bytes!(
            "../../../../tests/conformance/cases/reject-syseff-conditional-release-narrow.wf"
        ),
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn a_pure_formal_does_not_cover_an_actuals_explicit_close_effects() {
    assert_rule_kind(
        include_bytes!("../../../../tests/conformance/cases/fn4-neg-pure-member-binds-release.wf"),
        SemanticRule::Fn4,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn a_local_owner_has_no_history_effect_on_its_former_parameter() {
    let source = r#"fn relay(value: own box<u64>) -> result: own box<u64> pure {
  return move value;
}

fn observe(value: own box<u64>) -> result: own u64 pure {
  let local = relay(value: move value);
  return deref(local);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
    let extra = source.replace("-> result: own u64 pure", "-> result: own u64 reads(value)");
    assert_rule_kind(extra.as_bytes(), SemanticRule::Eff2, |kind| {
        matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
            if extra == &["reads(value)"])
    });
}

#[test]
fn dynamic_slot_reads_do_not_import_the_replacement_inputs_history() {
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
  let left = Slot(key: 7_u64, payload: move first);
  let right = Slot(key: 9_u64, payload: move second);
  let old_left = replace values[0_u64] = move left;
  let old_right = replace values[1_u64] = move right;
  region {
    return OBSERVE;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for observation in [
        "values[index].key",
        "read_key(slot: &values[index])",
        "read_payload(slot: &values[index])",
    ] {
        let accepted = source.replace("OBSERVE", observation);
        assert_complete(accepted.as_bytes());
        let extra = accepted.replace("reads(values)", "reads(values, first, second)");
        assert_rule_kind(extra.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
                if extra == &["reads(first)", "reads(second)"])
        });
    }
}

const FIELD_REPLACEMENT: &str = r#"struct Holder {
  before: u64;
  value: box<u64>;
  after: u64;
}

struct Nested {
  before: u64;
  holder: Holder;
  after: u64;
}

fn exchange(target: &uniq box<u64>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let displaced = replace deref(target) = move incoming;
  return move displaced;
}

fn inspect(holder: own Holder, incoming: own box<u64>) -> result: own Holder reads(holder.value), writes(holder.value) {
  region {
    let previous = exchange(target: &uniq holder.value, incoming: move incoming);
  }
  return move holder;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

#[test]
fn a_replacement_projects_only_the_selected_ordinary_field() {
    assert_complete(FIELD_REPLACEMENT.as_bytes());
    assert_complete(
        FIELD_REPLACEMENT
            .replace("holder: own Holder", "holder: own Nested")
            .replace("result: own Holder", "result: own Nested")
            .replace("holder.value", "holder.holder.value")
            .as_bytes(),
    );
}

#[test]
fn nested_reborrows_project_the_same_field_effects() {
    for (referent, path) in [
        ("Holder", "holder.value"),
        ("Nested", "holder.holder.value"),
    ] {
        let source = FIELD_REPLACEMENT
            .replace("holder: own Holder", &format!("holder: &uniq {referent}"))
            .replace("result: own Holder", "result: own unit")
            .replace("holder.value", path)
            .replace(
                &format!("&uniq {path}"),
                &format!("&uniq deref(holder){}", &path[6..]),
            )
            .replace("return move holder;", "return unit;");
        assert_complete(source.as_bytes());
    }
}

#[test]
fn field_effects_reject_missing_selected_access_and_extra_siblings() {
    for row in [
        "reads(holder.value)",
        "writes(holder.value)",
        "pure",
        "reads(holder.before, holder.value), writes(holder.value)",
        "reads(holder.value), writes(holder.value, holder.after)",
        "reads(holder), writes(holder.value)",
    ] {
        let source = FIELD_REPLACEMENT.replace("reads(holder.value), writes(holder.value)", row);
        assert_rule_kind(source.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { .. })
        });
    }
}

#[test]
fn opaque_fields_have_the_same_replacement_row_and_must_return_the_previous_owner() {
    let source = FIELD_REPLACEMENT
        .replace("box<u64>", "ReadFile")
        .replace("result: own Holder", "(result: own Holder, old: own ReadFile)")
        .replace(
            "let previous = exchange(target: &uniq holder.value, incoming: move incoming);\n  }\n  return move holder;",
            "let previous = exchange(target: &uniq holder.value, incoming: move incoming);\n    return move holder, move previous;\n  }",
        );
    assert_complete(source.as_bytes());
    let missing_result =
        source.replace("return move holder, move previous;", "return move holder;");
    assert_rule_kind(missing_result.as_bytes(), SemanticRule::Fn1, |_| true);
    let missing_consume = missing_result.replace(
        "(result: own Holder, old: own ReadFile)",
        "result: own Holder",
    );
    assert_rule_kind(missing_consume.as_bytes(), SemanticRule::Prov6, |_| true);
}

#[test]
fn generic_exclusive_replacement_needs_no_result_routing_summary() {
    let source = r#"fn exchange<T: linear>(target: &uniq T, incoming: own T) -> previous: own T reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn transfer(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  region {
    return exchange::<ReadFile>(target: &uniq deref(target), incoming: move incoming);
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source.as_bytes());
}

#[test]
fn ordinary_numeric_result_facts_survive_a_later_outcome_match() {
    assert_complete(
        br#"enum TestError {
  Failed();
}

fn provide(end: own u64) -> (result: own Result<unit, TestError>, next: own u64, count: own u64) pure contract {
  ensures next <= end;
} {
  return Ok<unit, TestError>(value: unit), end, 0_u64;
}

fn main() -> status: own ExitStatus pure {
  let (outcome, next, count) = provide(end: 4096_u64);
  match move outcome {
    Ok(value: done) => {
      invariant bounded: next <= 4096_u64;
    }
    Err(error: problem) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
}
