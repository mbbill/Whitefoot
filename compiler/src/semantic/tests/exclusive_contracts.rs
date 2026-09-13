//! Exclusive entry/exit measure contracts share the normal proof flow.

use crate::{SemanticOutcome, SemanticRule};

use super::with_semantics;

fn assert_rule(source: &[u8], rule: SemanticRule) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule, "{issue:?}");
    });
}

fn assert_complete(source: &str) {
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

#[test]
fn generic_unit_helper_publishes_only_proved_exclusive_state_relations() {
    let source = r#"fn touch<T: affine>(values: &uniq FixedVector<T, 4>) -> result: own unit reads(values), writes(values) contract {
  requires len_of(deref(values)) >= 1_u64;
  ensures len_of(deref(values)) == len_of(deref(entry(values)));
} {
  region {
    let value = take_back(vector: &uniq deref(values));
    place_back(vector: &uniq deref(values), value: move value);
    return unit;
  }
}

fn exercise<T: affine>(values: &uniq FixedVector<T, 4>) -> result: own unit reads(values), writes(values) contract {
  requires len_of(deref(values)) >= 1_u64;
} {
  region {
    touch::<T>(values: &uniq deref(values));
    let value = take_back(vector: &uniq deref(values));
    place_back(vector: &uniq deref(values), value: move value);
    return unit;
  }
}

fn main() -> status: own ExitStatus pure {
  let values = fixed_vector::<u64, 4>();
  region {
    place_back(vector: &uniq values, value: 7_u64);
    exercise::<u64>(values: &uniq values);
    return exit_status(code: 0_u8);
  }
}
"#;
    assert_complete(source);
    // A return must still prove the promised state, and a call with no
    // ensures must still lose the old length fact at its write boundary.
    // Keep the complete pop/push body and make its promise false. This
    // isolates return-proof checking; disposing an opaque affine T would
    // instead test the independent PROV-6 capability-release restriction.
    assert_rule(
        source
            .replace(
                "ensures len_of(deref(values)) == len_of(deref(entry(values)));",
                "ensures len_of(deref(values)) == len_of(deref(entry(values))) + 1_u64;",
            )
            .as_bytes(),
        SemanticRule::Fn9,
    );
    assert_rule(
        source
            .replace(
                "  ensures len_of(deref(values)) == len_of(deref(entry(values)));\n",
                "",
            )
            .as_bytes(),
        SemanticRule::Blk0,
    );
    // Unit is still not an admitted result datum.
    assert_rule(
        source
            .replace(
                "ensures len_of(deref(values)) == len_of(deref(entry(values)));",
                "ensures result == unit;",
            )
            .as_bytes(),
        SemanticRule::Fn9,
    );
}

const PUSH: &str = r#"fn push(values: &uniq FixedVector<u64, 4>, value: own u64) -> result: own unit reads(values), writes(values) contract {
  requires room_of(deref(values)) > 0_u64;
  ensures len_of(deref(values)) == len_of(deref(entry(values))) + 1_u64;
  ensures cap_of(deref(values)) == cap_of(deref(entry(values)));
} {
  region {
    place_back(vector: &uniq deref(values), value: value);
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = fixed_vector::<u64, 4>();
  region {
    push(values: &uniq values, value: 7_u64);
    invariant upper: len_of(values) <= 1_u64;
    invariant lower: len_of(values) >= 1_u64;
    let last = take_back(vector: &uniq values);
    invariant empty: len_of(values) <= 0_u64;
    return exit_status(code: 0_u8);
  }
}
"#;

#[test]
fn exclusive_push_proves_entry_and_exit_and_publishes_without_result() {
    assert_complete(PUSH);
    with_semantics(PUSH.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("{outcome:?}");
        };
        for function in &checked.data.functions {
            super::entailment::validate_derivations(&function.entailment);
            let mut state_clauses = std::collections::HashSet::new();
            for root in &function.entailment.derivations.roots {
                if matches!(
                    root.kind,
                    super::super::entailment::DerivationRootKind::PostconditionState { .. }
                ) {
                    let super::super::entailment::DerivationNode::PostconditionCall { detail } =
                        &function.entailment.derivations.nodes[root.node.0 as usize]
                    else {
                        panic!("a state root retains its call evidence");
                    };
                    assert!(
                        state_clauses.insert((
                            detail.call.clone(),
                            detail.summary.clone(),
                            detail.relation.clone()
                        )),
                        "binding a take result must not republish its state clauses"
                    );
                }
            }
        }
    });
}

#[test]
fn exit_state_cannot_be_its_own_increment() {
    let source = PUSH.replace(
        "len_of(deref(entry(values))) + 1_u64",
        "len_of(deref(values)) + 1_u64",
    );
    assert_rule(source.as_bytes(), SemanticRule::Call6);
}

#[test]
fn entry_former_rejects_body_and_nonexclusive_parameter() {
    let body = PUSH.replace(
        "return unit;",
        "let size = len_of(deref(entry(values)));\n  return unit;",
    );
    assert_rule(body.as_bytes(), SemanticRule::Msr3);
    let shared = r#"fn observe(values: &FixedVector<u64, 4>) -> result: own u64 reads(values) contract {
  ensures result == len_of(deref(entry(values)));
} {
  return len_of(deref(values));
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(shared.as_bytes(), SemanticRule::Msr3);
}

#[test]
fn an_exclusive_call_without_ensures_requires_a_runtime_reread() {
    let source = PUSH
        .replace(
            "  ensures len_of(deref(values)) == len_of(deref(entry(values))) + 1_u64;\n",
            "",
        )
        .replace(
            "  ensures cap_of(deref(values)) == cap_of(deref(entry(values)));\n",
            "",
        );
    assert_rule(source.as_bytes(), SemanticRule::Inv1);
    let guarded = source.replace(
        "    invariant upper: len_of(values) <= 1_u64;\n    invariant lower: len_of(values) >= 1_u64;\n    let last = take_back(vector: &uniq values);\n    invariant empty: len_of(values) <= 0_u64;",
        "    let size = len_of(values);\n    if size > 0_u64 {\n      let last = take_back(vector: &uniq values);\n    }",
    );
    assert_complete(&guarded);
}

#[test]
fn whole_referent_replacement_kills_the_old_window_facts() {
    let source = r#"fn clear(values: &uniq FixedVector<u64, 4>) -> result: own unit reads(values), writes(values) {
  let empty = fixed_vector::<u64, 4>();
  let old = replace deref(values) = move empty;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = fixed_vector::<u64, 4>();
  region {
    place_back(vector: &uniq values, value: 7_u64);
    clear(values: &uniq values);
    let last = take_back(vector: &uniq values);
    return exit_status(code: 0_u8);
  }
}
"#;
    assert_rule(source.as_bytes(), SemanticRule::Blk0);
}

#[test]
fn exclusive_exit_facts_publish_beside_multiple_results() {
    let source = r#"fn pop(values: &uniq FixedVector<u64, 4>) -> (value: own u64, count: own u64) reads(values), writes(values) contract {
  requires len_of(deref(values)) > 0_u64;
  ensures len_of(deref(values)) == len_of(deref(entry(values))) - 1_u64;
  ensures count == len_of(deref(values));
} {
  region {
    let value = take_back(vector: &uniq deref(values));
    let count = len_of(deref(values));
    return value, count;
  }
}

fn main() -> status: own ExitStatus pure {
  let values = fixed_vector::<u64, 4>();
  region {
    place_back(vector: &uniq values, value: 7_u64);
    let (value, count) = pop(values: &uniq values);
    invariant empty: len_of(values) <= 0_u64;
    invariant reported: count <= 0_u64;
    if value != 7_u64 {
      return exit_status(code: 1_u8);
    }
    return exit_status(code: 0_u8);
  }
}
"#;
    assert_complete(source);
}

#[test]
fn exclusive_nested_field_effects_preserve_disjoint_support() {
    let source = r#"struct Pair {
  changed: FixedVector<u64, 4>;
  untouched: FixedVector<u64, 4>;
}

fn push(pair: &uniq Pair, value: own u64) -> result: own unit reads(pair.changed), writes(pair.changed) contract {
  requires room_of(deref(pair).changed) > 0_u64;
  ensures len_of(deref(pair).changed) == len_of(deref(entry(pair)).changed) + 1_u64;
} {
  region {
    place_back(vector: &uniq deref(pair).changed, value: value);
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let changed = fixed_vector::<u64, 4>();
  let untouched = fixed_vector::<u64, 4>();
  let pair = Pair(changed: move changed, untouched: move untouched);
  region {
    push(pair: &uniq pair, value: 7_u64);
    invariant changed: len_of(pair.changed) >= 1_u64;
    invariant unchanged: len_of(pair.untouched) <= 0_u64;
    return exit_status(code: 0_u8);
  }
}
"#;
    assert_complete(source);
}

#[test]
fn an_exclusive_run_call_conflicts_with_a_surviving_view() {
    let source = PUSH.replace(
        "    push(values: &uniq values, value: 7_u64);",
        "    let view = slice_of(&values);\n    push(values: &uniq values, value: 7_u64);\n    let size = len_of(view);",
    );
    assert_rule(source.as_bytes(), SemanticRule::Own5);
}

#[test]
fn an_exit_only_clause_does_not_use_an_ordinary_borrow_result_as_a_datum() {
    let source = r#"fn retain['r](values: &uniq FixedVector<u64, 4>, saved: &'r u64) -> result: &'r u64 pure contract {
  ensures len_of(deref(values)) == len_of(deref(entry(values)));
} {
  return saved;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn replacing_the_actual_after_a_call_kills_its_exit_only_relation() {
    let helper = PUSH
        .split("fn main()")
        .next()
        .unwrap()
        .replace("-> result: own unit", "-> result: own FixedVector<u64, 4>")
        .replace("  return unit;", "  return fixed_vector::<u64, 4>();");
    let source = format!(
        "{helper}{}",
        r#"fn overwrite(values: &uniq FixedVector<u64, 4>) -> result: own unit reads(values), writes(values) contract {
  requires room_of(deref(values)) > 0_u64;
} {
  region {
    let replacement = push(values: &uniq deref(values), value: 7_u64);
    let old = replace deref(values) = move replacement;
  }
  region {
    let last = take_back(vector: &uniq deref(values));
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
    );
    assert_rule(source.as_bytes(), SemanticRule::Blk0);
}

#[test]
fn exclusive_equality_requires_both_affine_bounds() {
    let source = r#"fn fill['s](slots: &uniq Vector<'s, u8>, count: own u64) -> result: own unit reads(slots), writes(slots) contract {
  requires len_of(deref(slots)) == 0_u64;
  requires head_of(deref(slots)) <= 0_u64;
  requires room_of(deref(slots)) >= count;
  ensures len_of(deref(slots)) == count;
} {
  for (
    index in 0_u64..count,
    invariant filled: len_of(deref(slots)) >= index,
    invariant bounded: len_of(deref(slots)) <= index,
    invariant spare: room_of(deref(slots)) + index >= count,
    invariant flat: head_of(deref(slots)) <= 0_u64
  ) {
    place_back(vector: &uniq deref(slots), value: 0_u8);
  }
  invariant complete_min: len_of(deref(slots)) >= count;
  invariant complete_max: len_of(deref(slots)) <= count;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    for (header, fact) in [
        (
            "    invariant filled: len_of(deref(slots)) >= index,\n",
            "  invariant complete_min: len_of(deref(slots)) >= count;\n",
        ),
        (
            "    invariant bounded: len_of(deref(slots)) <= index,\n",
            "  invariant complete_max: len_of(deref(slots)) <= count;\n",
        ),
    ] {
        let missing_direction = source.replace(header, "").replace(fact, "");
        assert_rule(missing_direction.as_bytes(), SemanticRule::Fn9);
    }
}

#[test]
fn exclusive_boxed_run_holder_publishes_to_the_typed_referent() {
    let helper = PUSH.split("fn main()").next().unwrap();
    let source = format!(
        "{helper}{}",
        r#"fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u64, 4>();
  let owner = box_new(move empty);
  let room = room_of(deref(owner));
  if room > 0_u64 {
    region {
      let held = &uniq deref(owner);
      push(values: move held, value: 7_u64);
      invariant changed: len_of(deref(owner)) >= 1_u64;
    }
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 1_u8);
}
"#
    );
    assert_complete(&source);
    let surviving = source.replace(
        "      let held = &uniq deref(owner);\n      push(values: move held, value: 7_u64);\n      invariant changed: len_of(deref(owner)) >= 1_u64;",
        "      let held = &deref(owner);\n      push(values: &uniq deref(owner), value: 7_u64);\n      let observed = len_of(deref(held));",
    );
    assert_rule(surviving.as_bytes(), SemanticRule::Own5);
}
