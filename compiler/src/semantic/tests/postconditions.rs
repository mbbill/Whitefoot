use crate::{
    LexicalUseRole, ResolutionIssueKind, ResolutionRule, SemanticIssueKind, SemanticLocation,
    SemanticOutcome, SemanticRule,
};

use super::{assert_rule, assert_rule_at, with_semantics, with_semantics_dark};
use crate::semantic::entailment::{
    DerivationNode, FlowEventKind, FunctionPostconditionProof, PostconditionDisposition, ProofView,
};
use crate::semantic::model::{CheckedExpression, CheckedStatement};

fn assert_complete(source: &[u8]) {
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

fn assert_fn9_unproved(source: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("undischarged postcondition must be an FN-9 source issue: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn9);
        let SemanticIssueKind::UndischargedPostcondition(detail) = issue.kind() else {
            panic!("FN-9 issue must carry its proof disposition: {issue:?}");
        };
        assert_eq!(
            detail.disposition,
            crate::PostconditionProofDisposition::Unproved
        );
    });
}

fn postcondition_proof(source: &[u8], function: &str) -> FunctionPostconditionProof {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("postcondition proof fixture must check completely: {outcome:?}");
        };
        checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"))
            .entailment
            .postcondition
            .clone()
            .unwrap_or_else(|| panic!("function {function} must retain a postcondition proof"))
    })
}

fn dispositions(proof: &FunctionPostconditionProof) -> Vec<[PostconditionDisposition; 3]> {
    proof
        .exits
        .iter()
        .map(|exit| {
            assert_eq!(exit.complete.view, ProofView::Complete);
            assert_eq!(exit.unasserted.view, ProofView::Unasserted);
            assert_eq!(exit.s4_blinded.view, ProofView::S4Blinded);
            [
                exit.complete.disposition,
                exit.unasserted.disposition,
                exit.s4_blinded.disposition,
            ]
        })
        .collect()
}

#[test]
fn a_checked_plain_postcondition_is_proved_at_its_selected_exit() {
    let source = br#"fn identity(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
    let proof = postcondition_proof(source, "identity");
    assert_eq!(
        dispositions(&proof),
        vec![[PostconditionDisposition::Discharged; 3]]
    );
    assert!(proof.complete.discharged);
    assert!(proof.unasserted.discharged);
    assert!(proof.s4_blinded.discharged);
}

#[test]
fn body_checks_and_s4_are_routed_to_the_fixed_postcondition_views() {
    let body_check = br#"fn guarded(value: own i32) -> own i32 traps ensures result {
  check ieq(result, 1_i32) else trap "post";
} {
  check ieq(value, 1_i32) else trap "body";
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(body_check);
    let proof = postcondition_proof(body_check, "guarded");
    assert_eq!(
        dispositions(&proof),
        vec![[
            PostconditionDisposition::Discharged,
            PostconditionDisposition::Unproved,
            PostconditionDisposition::Unproved,
        ]]
    );
    assert!(proof.complete.discharged);
    assert!(!proof.unasserted.discharged);
    assert!(!proof.s4_blinded.discharged);

    let s4 = br#"fn constrained(value: own i32) -> own i32 pure requires {
  check ieq(value, 1_i32) else trap "pre";
} ensures result {
  check ieq(result, 1_i32) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(s4);
    let proof = postcondition_proof(s4, "constrained");
    assert_eq!(
        dispositions(&proof),
        vec![[
            PostconditionDisposition::Discharged,
            PostconditionDisposition::Discharged,
            PostconditionDisposition::Unproved,
        ]]
    );
    assert!(proof.complete.discharged);
    assert!(proof.unasserted.discharged);
    assert!(!proof.s4_blinded.discharged);
}

#[test]
fn entry_image_writes_are_retained_and_prevent_false_discharge() {
    let source = br#"fn changed(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  set value = 1_i32;
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let proof = postcondition_proof(source, "changed");
    assert_eq!(
        dispositions(&proof),
        vec![[PostconditionDisposition::Unproved; 3]]
    );
    let image = &proof.exits[0].entry_images[0];
    assert!(image.invalidation.is_some());
    assert!(!proof.complete.discharged);
    assert_rule_at(source, SemanticRule::Fn9, "return value;");
}

#[test]
fn a_moved_holder_consume_precedes_its_projected_call_write() {
    let source = br#"fn overwrite['r](out: &uniq 'r i32) -> own unit writes('r) {
  set deref(out) = 1_i32;
  return unit;
}

fn transfer['r](out: &uniq 'r i32) -> own i32 reads('r), writes('r) ensures result {
  check ieq(result, deref(out)) else trap "post";
} {
  let before = deref(out);
  overwrite<'r>(out: move out);
  return before;
}

fn plain['r](out: &uniq 'r i32) -> own i32 reads('r), writes('r) {
  let before = deref(out);
  overwrite<'r>(out: move out);
  return before;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("moved-holder fixture must check completely: {outcome:?}");
        };
        let transfer = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "transfer")
            .expect("transfer function");
        let proof = transfer
            .entailment
            .postcondition
            .as_ref()
            .expect("transfer postcondition proof");
        let invalidation = proof.exits[0].entry_images[0]
            .invalidation
            .expect("the moved holder invalidates its entry image");
        let event = &transfer.entailment.derivations.events[invalidation.0 as usize];
        assert_eq!(
            event.kind,
            FlowEventKind::PostconditionEntryImageInvalidation
        );

        let (call, carrier) = transfer
            .body
            .iter()
            .find_map(|statement| {
                let call = match statement {
                    CheckedStatement::Evaluate(call)
                    | CheckedStatement::DropExpression { value: call, .. } => call,
                    _ => return None,
                };
                let CheckedExpression::UserCall {
                    call, arguments, ..
                } = call
                else {
                    return None;
                };
                let [
                    CheckedExpression::Binding {
                        carrier,
                        consume_root: true,
                        ..
                    },
                ] = arguments.as_slice()
                else {
                    return None;
                };
                Some((call, carrier))
            })
            .expect("transfer has one direct moved-holder call");
        assert_eq!(event.node_path.as_ref(), Some(carrier));
        assert_ne!(event.node_path.as_ref(), Some(call));

        let plain = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "plain")
            .expect("plain function");
        assert!(plain.entailment.postcondition.is_none());
        assert!(
            plain
                .entailment
                .derivations
                .events
                .iter()
                .all(|event| { event.kind != FlowEventKind::PostconditionEntryImageInvalidation })
        );
    });
}

#[test]
fn an_ordinary_loop_uses_the_exact_first_invalidation_event_without_a_snapshot() {
    let source = br#"fn looped(value: own i32, stop: own Bool) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  loop @again {
    set value = 1_i32;
    set value = 2_i32;
    if stop {
      break @again;
    }
  }
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("ordinary-loop fixture must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "looped")
            .expect("looped function");
        let proof = function
            .entailment
            .postcondition
            .as_ref()
            .expect("looped postcondition proof");
        let invalidation = proof.exits[0].entry_images[0]
            .invalidation
            .expect("the continuing write invalidates the entry image");
        let event = &function.entailment.derivations.events[invalidation.0 as usize];
        assert_eq!(
            event.kind,
            FlowEventKind::PostconditionEntryImageInvalidation
        );
        let set = function
            .body
            .iter()
            .find_map(|statement| match statement {
                CheckedStatement::Loop { body, .. } => body.iter().find_map(|statement| {
                    let CheckedStatement::Set { node_path, .. } = statement else {
                        return None;
                    };
                    Some(node_path)
                }),
                _ => None,
            })
            .expect("loop body set");
        assert_eq!(event.node_path.as_ref(), Some(set));
        assert!(
            function
                .entailment
                .derivations
                .events
                .iter()
                .all(|event| event.kind != FlowEventKind::Snapshot)
        );
    });
}

#[test]
fn counted_append_proves_the_admitted_result_and_refutes_only_the_blinded_invalid_exit() {
    let source = br#"fn append['d, 'm](destination: &uniq 'd buffer<u8>, filled: own u64, text: own slice<'m, u8>) -> own u64 reads('d 'm), writes('d) requires {
  let capacity = len(deref(destination));
  let admitted = ile(filled, capacity);
  check admitted else trap "append filled exceeds destination";
} ensures result {
  let capacity = len(deref(destination));
  check ile(result, capacity) else trap "append result exceeds destination";
} {
  let capacity = len(deref(destination));
  let admitted = ile(filled, capacity);
  let length = len(text);
  if admitted {
    for @append at in filled..capacity {
      let taken = at -wrap filled;
      let done = ige(taken, length);
      if done {
        return at;
      }
      let byte = text[taken];
      set deref(destination)[at] = byte;
    }
    return capacity;
  } else {
    return filled;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
    let proof = postcondition_proof(source, "append");
    assert_eq!(
        dispositions(&proof),
        vec![
            [PostconditionDisposition::Discharged; 3],
            [PostconditionDisposition::Discharged; 3],
            [
                PostconditionDisposition::Discharged,
                PostconditionDisposition::Discharged,
                PostconditionDisposition::Refuted,
            ],
        ]
    );
}

#[test]
fn length_entry_images_ignore_element_writes_but_not_root_replacement() {
    let element = br#"fn kept(values: own array<u8, 2>) -> own u64 pure ensures result {
  let size = len(values);
  check ieq(result, size) else trap "post";
} {
  set values[0_u64] = 1_u8;
  return len(values);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(element);
    let proof = postcondition_proof(element, "kept");
    assert_eq!(
        dispositions(&proof),
        vec![[PostconditionDisposition::Discharged; 3]]
    );
    assert!(
        proof.exits[0]
            .entry_images
            .iter()
            .all(|image| image.invalidation.is_none())
    );

    let replacement = br#"fn consume(values: own array<u8, 2>) -> own unit pure {
  return unit;
}

fn replaced(values: own array<u8, 2>) -> own u64 pure ensures result {
  let size = len(values);
  check ieq(result, size) else trap "post";
} {
  let size = len(values);
  let ignored = consume(values: move values);
  return size;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let proof = postcondition_proof(replacement, "replaced");
    assert_eq!(
        dispositions(&proof),
        vec![[PostconditionDisposition::Unproved; 3]]
    );
    assert!(
        proof.exits[0]
            .entry_images
            .iter()
            .any(|image| image.invalidation.is_some())
    );
}

#[test]
fn selected_exits_aggregate_only_when_every_exit_in_the_view_discharges() {
    let source = br#"fn branch(value: own i32, choose: own Bool) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  if choose {
    return value;
  } else {
    return value;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
    let proof = postcondition_proof(source, "branch");
    assert_eq!(proof.exits.len(), 2);
    assert!(
        proof
            .exits
            .windows(2)
            .all(|pair| pair[0].statement.components() < pair[1].statement.components())
    );
    assert!(proof.complete.discharged);
    assert!(proof.unasserted.discharged);
    assert!(proof.s4_blinded.discharged);
}

#[test]
fn an_earlier_verified_postcondition_discharges_a_fresh_direct_result() {
    let independent = br#"fn callee(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "callee post";
} {
  return value;
}

fn caller(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "caller post";
} {
  let ignored = callee(value: value);
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(independent);

    let dependent = br#"fn callee(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "callee post";
} {
  return value;
}

fn caller(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "caller post";
} {
  let called = callee(value: value);
  return called;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(dependent);
}

#[test]
fn an_earlier_ok_summary_is_available_only_at_direct_match_arm_entry() {
    let source =
        br#"fn callee(value: own i32) -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, value) else trap "callee post";
} {
  return Ok<i32, Overflow>(value: value);
}

fn direct(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "direct post";
} {
  match callee(value: value) {
    Ok(value: payload) => {
      return payload;
    }
    Err(error: problem) => {
      return value;
    }
  }
}

fn delivered(value: own i32) -> own i32 pure {
  let selected = match callee(value: value) {
    Ok(value: payload) => {
      give payload;
    }
    Err(error: problem) => {
      give value;
    }
  }
  return selected;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn a_borrowed_formal_substitution_consumes_its_one_formal_deref() {
    let source = br#"struct Pair {
  value: i32;
}

fn observe['r](pair: &'r Pair) -> own i32 reads('r) ensures result {
  check ieq(result, deref(pair).value) else trap "observe post";
} {
  return deref(pair).value;
}

fn caller['r](pair: &'r Pair) -> own i32 reads('r) ensures result {
  check ieq(result, deref(pair).value) else trap "caller post";
} {
  let observed = observe<'r>(pair: pair);
  return observed;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn a_moved_unique_actual_cannot_publish_a_stale_postcondition_relation() {
    let source = br#"struct Pair {
  kept: i32;
  changed: i32;
}

fn touch['r](pair: &uniq 'r Pair) -> own i32 reads('r), writes('r) ensures result {
  check ieq(result, deref(pair).kept) else trap "touch post";
} {
  set deref(pair).changed = 1_i32;
  return deref(pair).kept;
}

fn caller(pair: own Pair) -> own i32 pure ensures result {
  check ieq(result, pair.kept) else trap "caller post";
} {
  region 'r {
    let holder = &uniq 'r pair;
    let observed = touch<'r>(pair: move holder);
    return observed;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_fn9_unproved(source);
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("moved unique-actual fixture must remain inspectable: {outcome:?}");
        };
        let touch = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "touch")
            .expect("touch function");
        assert!(
            touch
                .entailment
                .postcondition
                .as_ref()
                .is_some_and(|proof| proof.complete.discharged),
            "the callee summary premise is independently available"
        );
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert!(caller.entailment.derivations.nodes.iter().all(|node| {
            !matches!(
                node,
                DerivationNode::PostconditionDirectResult { .. }
                    | DerivationNode::PostconditionDirectMatch { .. }
            )
        }));
    });
}

#[test]
fn a_box_deref_actual_cannot_survive_a_cross_formal_owner_move() {
    let source =
        br#"fn observe(value: own i32, owner: own box<i32>) -> own i32 pure ensures result {
  check ieq(result, value) else trap "observe post";
} {
  return value;
}

fn caller() -> own i32 allocates(heap) ensures result {
  check ieq(result, 1_i32) else trap "caller post";
} {
  let owner = box_new(1_i32);
  let observed = observe(value: deref(owner), owner: move owner);
  return observed;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_fn9_unproved(source);
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("box-deref M fixture must remain inspectable: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert!(
            caller
                .entailment
                .derivations
                .nodes
                .iter()
                .all(|node| { !matches!(node, DerivationNode::PostconditionDirectResult { .. }) })
        );
    });
}

#[test]
fn a_later_owner_move_kills_an_already_published_s12_relation() {
    let source = br#"fn observe(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "observe post";
} {
  return value;
}

fn sink(owner: own box<i32>) -> own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> own unit pure requires {
  check ieq(left, right) else trap "guard pre";
} {
  return unit;
}

fn caller() -> own unit allocates(heap) {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  sink(owner: move owner);
  guard(left: observed, right: expected);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Fn8,
        "guard(left: observed, right: expected)",
    );
}

#[test]
fn an_ordinary_fallback_survives_when_the_same_s12_candidate_dies() {
    let source = br#"fn observe(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "observe post";
} {
  return value;
}

fn sink(owner: own box<i32>) -> own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> own unit pure requires {
  check ieq(left, right) else trap "guard pre";
} {
  return unit;
}

fn caller() -> own unit allocates(heap), traps {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  check ieq(observed, deref(owner)) else trap "ordinary fallback";
  sink(owner: move owner);
  guard(left: observed, right: expected);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn a_joined_holder_free_consequence_survives_the_original_owner_move() {
    let source = br#"fn observe(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "observe post";
} {
  return value;
}

fn sink(owner: own box<i32>) -> own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> own unit pure requires {
  check ieq(left, right) else trap "guard pre";
} {
  return unit;
}

fn caller(choose: own Bool) -> own unit allocates(heap) {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  if choose {
    let left_path = 0_u8;
  } else {
    let right_path = 0_u8;
  }
  sink(owner: move owner);
  guard(left: observed, right: expected);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn a_direct_same_binding_call_result_establishes_only_after_the_target_kill() {
    let source = br#"fn choose(ignored: own i32, value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "choose post";
} {
  return value;
}

fn guard(left: own i32, right: own i32) -> own unit pure requires {
  check ieq(left, right) else trap "guard pre";
} {
  return unit;
}

fn caller(slot: own i32, replacement: own i32) -> own unit pure {
  set slot = choose(ignored: slot, value: replacement);
  guard(left: slot, right: replacement);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("same-binding receiver fixture must remain inspectable: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert_eq!(
            caller
                .entailment
                .derivations
                .nodes
                .iter()
                .filter(|node| matches!(node, DerivationNode::PostconditionDirectReceiver { .. }))
                .count(),
            3,
            "the exact receiver route is retained once per proof view"
        );
    });
}

#[test]
fn direct_same_binding_near_misses_retain_no_receiver_root_or_special_event() {
    let source = br#"fn echo(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "echo post";
} {
  return value;
}

fn choose(first: own i32, second: own i32, value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "choose post";
} {
  return value;
}

fn mentions_receiver(slot: own i32) -> own i32 pure ensures result {
  check ieq(result, slot) else trap "caller post";
} {
  set slot = echo(value: slot);
  return slot;
}

fn repeated_receiver(slot: own i32, replacement: own i32) -> own unit pure {
  set slot = choose(first: slot, second: slot, value: replacement);
  return unit;
}

fn distinct_receiver(slot: own i32, other: own i32, replacement: own i32) -> own unit pure {
  set slot = choose(first: other, second: other, value: replacement);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("same-binding near misses must remain inspectable: {outcome:?}");
        };
        for name in [
            "mentions_receiver",
            "repeated_receiver",
            "distinct_receiver",
        ] {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("{name} function"));
            assert!(function.entailment.derivations.nodes.iter().all(|node| {
                !matches!(node, DerivationNode::PostconditionDirectReceiver { .. })
            }));
            assert!(
                function
                    .entailment
                    .derivations
                    .events
                    .iter()
                    .all(|event| { event.kind != FlowEventKind::PostconditionReceiverWrite })
            );
        }
    });
}

#[test]
fn a_selected_payload_first_set_reestablishes_only_the_result_relation() {
    let source = br#"fn selected(value: own i32) -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, value) else trap "selected post";
} {
  return Ok<i32, Overflow>(value: value);
}

fn guard(left: own i32, right: own i32) -> own unit pure requires {
  check ieq(left, right) else trap "guard pre";
} {
  return unit;
}

fn caller(outer: own i32, replacement: own i32) -> own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set outer = payload;
      guard(left: outer, right: replacement);
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("selected receiver fixture must remain inspectable: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert_eq!(
            caller
                .entailment
                .derivations
                .nodes
                .iter()
                .filter(|node| matches!(node, DerivationNode::PostconditionSelectedReceiver { .. }))
                .count(),
            3,
            "the selected receiver route is retained once per proof view"
        );
    });
}

#[test]
fn selected_receiver_nonfirst_additional_write_and_call_actual_shapes_retain_no_route() {
    let source = br#"struct Cell {
  value: i32;
}

fn selected(value: own i32) -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, value) else trap "selected post";
} {
  return Ok<i32, Overflow>(value: value);
}

fn nonfirst(outer: own i32, replacement: own i32) -> own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      let intervening = 0_i32;
      set outer = payload;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn additional_write(outer: own i32, replacement: own i32) -> own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set outer = payload;
      set outer = replacement;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn call_actual(outer: own i32) -> own unit pure {
  match selected(value: outer) {
    Ok(value: payload) => {
      set outer = payload;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn projected(cell: own Cell, replacement: own i32) -> own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set cell.value = payload;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn computed(outer: own i32, replacement: own i32) -> own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set outer = iand(payload, 0_i32);
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("selected receiver near misses must remain inspectable: {outcome:?}");
        };
        for name in [
            "nonfirst",
            "additional_write",
            "call_actual",
            "projected",
            "computed",
        ] {
            let function = checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("{name} function"));
            assert!(function.entailment.derivations.nodes.iter().all(|node| {
                !matches!(node, DerivationNode::PostconditionSelectedReceiver { .. })
            }));
            assert!(
                function
                    .entailment
                    .derivations
                    .events
                    .iter()
                    .all(|event| { event.kind != FlowEventKind::PostconditionReceiverWrite })
            );
        }
    });
}

#[test]
fn a_checked_ok_postcondition_selects_its_direct_payload() {
    let source = br#"fn selected(value: own i32) -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, value) else trap "post";
} {
  return Ok<i32, Overflow>(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn an_ok_selector_rejects_an_empty_selected_exit_set() {
    let source = br#"fn unselected() -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, 0_i32) else trap "post";
} {
  let error = Overflow();
  return Err<i32, Overflow>(error: error);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::NoSelectedNormalExit {
            residual: "no selected normal exit",
        },
    );
}

#[test]
fn an_ok_selector_rejects_a_stored_whole_result_return() {
    let source =
        br#"fn stored(value: own i32) -> own Result<i32, Overflow> pure ensures Ok(value: result) {
  check ieq(result, value) else trap "post";
} {
  let outcome = Ok<i32, Overflow>(value: value);
  return move outcome;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "return move outcome;");
}

#[test]
fn relation_length_rejects_a_named_constant_root() {
    let source = br#"const values: array<i32, 1> =[0_i32];

fn length() -> own u64 pure ensures result {
  let size = len(values);
  check ieq(result, size) else trap "post";
} {
  return 1_u64;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "ieq(result, size)");
}

#[test]
fn projected_result_is_rejected_at_the_complete_final_relation() {
    let source = br#"fn projected(value: own i32) -> own i32 pure ensures result {
  check ieq(result.field, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "ieq(result.field, value)");
}

#[test]
fn a_nonbare_result_use_in_an_unused_local_is_still_rejected() {
    let source = br#"fn hidden(value: own i32) -> own i32 pure ensures result {
  let ignored = reinterpret<i32, u32>(deref(result));
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Fn9,
        "reinterpret<i32, u32>(deref(result))",
    );
}

#[test]
fn selected_returns_retain_deref_field_and_field_length_places() {
    let source = br#"struct Pair {
  value: i32;
}

struct Values {
  items: array<u8, 2>;
}

fn from_box(owner: own box<Pair>) -> own i32 pure ensures result {
  check ieq(result, deref(owner).value) else trap "post";
} {
  return deref(owner).value;
}

fn from_shared['r](owner: &'r Pair) -> own i32 reads('r) ensures result {
  check ieq(result, deref(owner).value) else trap "post";
} {
  return deref(owner).value;
}

fn field_length(values: own Values) -> own u64 pure ensures result {
  let size = len(values.items);
  check ieq(result, size) else trap "post";
} {
  return len(values.items);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn a_holder_alias_does_not_change_the_selected_return_term_identity() {
    let source = br#"struct Pair {
  value: i32;
}

fn from_shared_alias['r](owner: &'r Pair) -> own i32 reads('r) ensures result {
  check ieq(result, deref(owner).value) else trap "post";
} {
  let alias = owner;
  return deref(alias).value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let proof = postcondition_proof(source, "from_shared_alias");
    assert_eq!(
        dispositions(&proof),
        vec![[PostconditionDisposition::Unproved; 3]]
    );
    assert_rule_at(source, SemanticRule::Fn9, "return deref(alias).value;");
}

#[test]
fn a_concrete_const_substitution_is_retained_with_a_selected_length() {
    let source =
        br#"fn count<const n: u64>(values: own array<u8, n>) -> own u64 pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return len(values);
}

fn main() -> own unit pure {
  let values = array_new<u8, 1>(0_u8);
  let one = count<1>(values: move values);
  return unit;
}
"#;
    assert_complete(source);
}

#[test]
fn an_ensures_bearing_conformance_binding_is_fn3_before_proof() {
    let source = br#"contract Maker {
  fn make() -> own i32 pure;
}

fn make() -> own i32 pure ensures result {
  check ieq(result, 1_i32) else trap "post";
} {
  return 1_i32;
}

conform i32: Maker {
  make = make;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule_at(source, SemanticRule::Fn3, "make = make;");
}

#[test]
fn an_invalid_contract_precedes_the_postcondition_proof_boundary() {
    let source = br#"contract Invalid<T> {
}

fn identity(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::GenericContract,
    );
}

#[test]
fn an_invalid_contract_law_precedes_the_postcondition_proof_boundary() {
    let source = br#"contract InvalidLaw {
  fn combine(x: own u64, y: own u64) -> own u64 pure;
  law identity(combine, unit);
}

fn identity(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn4,
        SemanticIssueKind::InvalidContractLaw,
    );
}

#[test]
fn invalid_selector_precedes_an_unresolved_name_in_its_entry() {
    let source = br#"fn invalid() -> own unit pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn every_concrete_selector_is_admitted_before_any_entry_lookup() {
    let source = br#"fn first(value: own i32) -> own i32 pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return value;
}

fn second() -> own unit pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn admitted_selector_forwards_the_original_entry_lookup_issue() {
    let source = br#"fn unresolved(value: own i32) -> own i32 pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("expected delayed resolution issue, got {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse {
                spelling,
                role: LexicalUseRole::PlaceBase,
                ..
            } if spelling == "missing"
        ));
    });
}

#[test]
fn entry_inventory_precedes_a_poisoned_body_constructor() {
    let source = br#"fn poisoned(value: own i32) -> own i32 pure ensures result {
  let ilt = ieq(result, value);
  check ilt else trap "post";
} {
  return Missing();
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("expected delayed inventory issue, got {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName { spelling, .. } if spelling == "ilt"
        ));
    });
}

#[test]
fn unused_generic_entry_issue_precedes_its_body_semantics() {
    let source = br#"fn generic<T>(value: own T) -> own T pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return box_new(value);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("unused generic entry lookup must win, got {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
    });
}

#[test]
fn a_successfully_resolved_foreign_variant_is_an_fn9_source_issue() {
    let source = br#"enum Foreign {
  Other(value: i32);
}

fn selected(value: own i32) -> own Result<i32, Overflow> pure ensures Other(value: result) {
  check ieq(result, value) else trap "post";
} {
  return Ok<i32, Overflow>(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("foreign selector variant must be FN-9, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn9);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::InvalidPostconditionSelector
        );
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("selector must cite a source node");
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(&source[start..end], b"Other(value: result)");
    });
}

#[test]
fn concrete_generic_instances_do_not_reuse_symbolic_selector_class() {
    let source = br#"fn identity<T>(value: own T) -> own T pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  let good = identity<i32>(value: 1_i32);
  let flag = True();
  let bad = identity<Bool>(value: flag);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn delayed_entry_lookup_precedes_unrelated_entry_form_semantics() {
    let source = br#"fn probe(value: own i32) -> own i32 pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return value;
}

fn main() -> own i32 pure {
  return 0_i32;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("delayed lookup must precede unrelated semantics, got {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, .. } if spelling == "missing"
        ));
    });
}

#[test]
fn selector_preflight_precedes_unrelated_entry_form_semantics() {
    let source = br#"fn invalid() -> own unit pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return unit;
}

fn main() -> own i32 pure {
  return 0_i32;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn unused_numeric_bounds_preserve_selector_class_information() {
    let source = br#"fn invalid<T: Float>(value: own T) -> own T pure ensures result {
  check feq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn unavailable_generic_type_argument_does_not_invent_a_selector_instance() {
    let source = br#"fn generic<T>(value: own T) -> own T pure ensures result {
  let ilt = ieq(result, result);
  check ilt else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  let unavailable = generic<Missing>(value: unit);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("entry inventory must beat an unavailable type argument: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });
}

#[test]
fn unavailable_const_argument_does_not_invent_a_selector_instance() {
    let source = br#"fn generic<T, const n: u64>(value: own T) -> own T pure ensures result {
  let ilt = ieq(result, result);
  check ilt else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  let unavailable = generic<unit, missing>(value: unit);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::ResolutionIssue { issue } = outcome else {
            panic!("entry inventory must beat an unavailable const argument: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });
}

#[test]
fn unrelated_invalid_constant_does_not_suppress_an_independent_selector() {
    let source = br#"const bad: u8 = 1_u16;

fn invalid() -> own unit pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn transitive_invalid_constant_does_not_become_a_compiler_failure() {
    let source = br#"const bad: u8 = 1_u16;

const alias: u8 = bad;

fn invalid() -> own unit pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn unavailable_symbolic_header_does_not_forward_its_entry_issue() {
    let source = br#"fn unavailable<T>(value: own array<T, 1>) -> own T pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("ordinary header issue must win, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Type2);
    });
}

#[test]
fn unavailable_record_does_not_suppress_a_later_independent_selector() {
    let source = br#"fn unavailable<T>(value: own array<T, 1>) -> own T pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return value;
}

fn invalid() -> own unit pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn malformed_trailing_argument_does_not_enter_final_selector_metadata() {
    let source = br#"fn generic<T: Int>(value: own T) -> own T pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  let bad = generic<i32, i32>(value: 1_i32);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("malformed call must remain a source FN-2 issue: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn2, "{issue:?}");
    });
}

#[test]
fn invalid_unrelated_function_template_does_not_suppress_selector_admission() {
    let source = br#"fn broken<const n: Bool>() -> own unit pure {
  return unit;
}

fn invalid() -> own unit pure ensures result {
  check ieq(result, result) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn referenced_generic_nominal_must_pass_its_symbolic_template_judgment() {
    let source = br#"struct Invalid<T> {
  values: array<T, 2>;
}

fn probe(value: own Invalid<i32>) -> own unit pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the referenced symbolic nominal premise must win: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Type2);
    });
}

#[test]
fn unrelated_invalid_generic_nominal_does_not_suppress_selector_admission() {
    let source = br#"struct Invalid<T> {
  values: array<T, 2>;
}

fn invalid() -> own unit pure ensures result {
  check ieq(result, missing) else trap "post";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionSelector,
    );
}

#[test]
fn postcondition_components_are_callee_before_caller_and_publish_atomically() {
    let source = br#"fn top(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "top post";
} {
  let ignored = bridge(value: value);
  return value;
}

fn leaf(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "leaf post";
} {
  return value;
}

fn middle(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "middle post";
} {
  let ignored = leaf(value: value);
  return value;
}

fn bridge(value: own i32) -> own i32 pure {
  let ignored = middle(value: value);
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("independent call chain must check completely: {outcome:?}");
        };
        let mut scheduled = Vec::new();
        for (ordinal, component) in checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .enumerate()
        {
            assert_eq!(component.ordinal as usize, ordinal);
            assert!(
                component
                    .functions
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0)
            );
            assert!(
                component
                    .summaries
                    .windows(2)
                    .all(|pair| pair[0].function.0 < pair[1].function.0)
            );
            assert!(component.summaries.iter().all(|summary| {
                summary.component == component.ordinal
                    && component.functions.contains(&summary.function)
            }));
            scheduled.extend(component.functions.iter().copied());
        }
        scheduled.sort_unstable_by_key(|function| function.0);
        assert_eq!(
            scheduled,
            checked
                .data
                .functions
                .iter()
                .map(|function| function.id)
                .collect::<Vec<_>>()
        );
        let function = |name: &str| {
            checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing {name}"))
        };
        let component = |id| {
            checked
                .data
                .postcondition_schedule
                .components
                .iter()
                .find(|component| component.functions.contains(&id))
                .expect("every concrete function belongs to one component")
        };
        let leaf = function("leaf");
        let middle = function("middle");
        let bridge = function("bridge");
        let top = function("top");
        assert!(component(leaf.id).ordinal < component(middle.id).ordinal);
        assert!(component(middle.id).ordinal < component(bridge.id).ordinal);
        assert!(component(bridge.id).ordinal < component(top.id).ordinal);
        assert!(component(bridge.id).summaries.is_empty());
        for function in [leaf, middle, top] {
            let proof = function
                .entailment
                .postcondition
                .as_ref()
                .expect("postcondition proof");
            let summary = proof.summary.as_ref().expect("published summary");
            assert_eq!(summary.function, function.id);
            assert_eq!(summary.block, proof.block);
            assert_eq!(summary.relation_ordinal, 0);
            assert_eq!(summary.component, component(function.id).ordinal);
            assert_eq!(component(function.id).summaries, vec![summary.clone()]);
        }
    });
    assert_complete(source);
}

#[test]
fn an_independently_proved_mutual_component_publishes_summaries_in_function_order() {
    let source = br#"fn first(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "first post";
} {
  let ignored = second(value: value);
  return value;
}

fn second(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "second post";
} {
  let ignored = first(value: value);
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("independent mutual recursion must check completely: {outcome:?}");
        };
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.len() == 2)
            .expect("one mutual component");
        assert!(
            component
                .functions
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
        );
        assert_eq!(component.summaries.len(), 2);
        assert!(
            component
                .summaries
                .windows(2)
                .all(|pair| pair[0].function.0 < pair[1].function.0)
        );
        for summary in &component.summaries {
            let proof = checked.data.functions[summary.function.0 as usize]
                .entailment
                .postcondition
                .as_ref()
                .expect("mutual proof");
            assert_eq!(proof.summary.as_ref(), Some(summary));
            assert!(proof.complete.discharged);
        }
    });
    assert_complete(source);
}

#[test]
fn an_independently_proved_self_recursive_component_publishes_its_summary() {
    let source = br#"fn recursive(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  let ignored = recursive(value: value);
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("independent recursion must check completely: {outcome:?}");
        };
        let recursive = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "recursive")
            .expect("recursive function");
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&recursive.id))
            .expect("recursive component");
        assert_eq!(component.functions, vec![recursive.id]);
        let summary = recursive
            .entailment
            .postcondition
            .as_ref()
            .and_then(|proof| proof.summary.as_ref())
            .expect("independent recursive summary publishes");
        assert_eq!(component.summaries, vec![summary.clone()]);
    });
    assert_complete(source);
}

#[test]
fn one_failed_mutual_member_withholds_the_whole_component_summary_batch() {
    let source = br#"fn left(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "left post";
} {
  let ignored = right(value: value);
  return value;
}

fn right(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "right post";
} {
  let called = left(value: value);
  return called;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark mutual component must retain failure metadata: {outcome:?}");
        };
        let members = checked
            .data
            .functions
            .iter()
            .filter(|function| matches!(function.name.as_str(), "left" | "right"))
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 2);
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&members[0].id))
            .expect("mutual component");
        assert_eq!(component.functions.len(), 2);
        assert!(component.summaries.is_empty());
        assert!(members.iter().all(|function| {
            function
                .entailment
                .postcondition
                .as_ref()
                .is_some_and(|proof| proof.summary.is_none())
        }));
        assert!(
            members
                .iter()
                .find(|function| function.name == "left")
                .unwrap()
                .entailment
                .postcondition
                .as_ref()
                .unwrap()
                .complete
                .discharged
        );
        assert!(
            !members
                .iter()
                .find(|function| function.name == "right")
                .unwrap()
                .entailment
                .postcondition
                .as_ref()
                .unwrap()
                .complete
                .discharged
        );
    });
    assert_fn9_unproved(source);
}

#[test]
fn a_seedless_mutual_postcondition_cycle_publishes_no_summary() {
    let source = br#"fn first(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "first post";
} {
  let called = second(value: value);
  return called;
}

fn second(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "second post";
} {
  let called = first(value: value);
  return called;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark seedless cycle must retain dispositions: {outcome:?}");
        };
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.len() == 2)
            .expect("seedless mutual component");
        assert!(component.summaries.is_empty());
        for function in component
            .functions
            .iter()
            .map(|id| &checked.data.functions[id.0 as usize])
        {
            let proof = function.entailment.postcondition.as_ref().unwrap();
            assert!(!proof.complete.discharged);
            assert!(proof.summary.is_none());
        }
    });
    assert_fn9_unproved(source);
}

#[test]
fn concrete_generic_instances_receive_distinct_verified_summary_identities() {
    let source = br#"fn identity<T: Int>(value: own T) -> own T pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn main() -> own unit pure {
  let small = identity<i32>(value: 1_i32);
  let wide = identity<u64>(value: 1_u64);
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("concrete generic summaries must check completely: {outcome:?}");
        };
        let instances = checked
            .data
            .functions
            .iter()
            .filter(|function| function.name == "identity")
            .collect::<Vec<_>>();
        assert_eq!(instances.len(), 2);
        let summaries = instances
            .iter()
            .map(|function| {
                function
                    .entailment
                    .postcondition
                    .as_ref()
                    .and_then(|proof| proof.summary.as_ref())
                    .expect("concrete summary")
            })
            .collect::<Vec<_>>();
        assert_ne!(summaries[0].function, summaries[1].function);
        assert_ne!(summaries[0].component, summaries[1].component);
    });
    assert_complete(source);
}

#[test]
fn accepted_provenance_views_use_the_finalized_function_derivation_ids() {
    let source = br#"fn normalized(value: own i32) -> own i32 pure requires {
  check ieq(value, 1_i32) else trap "required";
} ensures result {
  check ieq(result, value) else trap "normalized post";
} {
  return 1_i32;
}

fn caller() -> own i32 pure ensures result {
  check ieq(result, 1_i32) else trap "caller post";
} {
  let called = normalized(value: 1_i32);
  return called;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("finalized-view fixture must be accepted: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        let index = caller.id.0 as usize;
        assert_eq!(
            checked.data.provenance.unasserted[index],
            caller.entailment.unasserted
        );
        assert_eq!(
            checked.data.provenance.s4_blinded[index],
            caller.entailment.s4_blinded
        );
        assert!(
            caller
                .entailment
                .unasserted
                .call_goals
                .iter()
                .chain(&caller.entailment.s4_blinded.call_goals)
                .any(|outcome| outcome.derivation.is_some()),
            "the equality must cover at least one remapped derivation ID"
        );
    });
}

#[test]
fn a_provenance_event_rejects_the_whole_optimistic_postcondition_batch() {
    let source = br#"fn identity(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "identity post";
} {
  return value;
}

fn relay(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "relay post";
} {
  let observed = identity(value: value);
  return observed;
}

fn read(values: own array<u8, 4>, position: own u64) -> own u8 traps {
  let room = len(values);
  claim bounded: ilt(position, room) because "claimed parameter bound";
  return values[position];
}

command fn main(command.args as args: own Args) -> own ExitStatus traps {
  let values = array_new<u8, 4>(0_u8);
  region 'a {
    let position = args_count<'a>(args: &'a args);
    let selected = read(values: move values, position: position);
  }
  return exit_status(code: 0_u8);
}
"#;

    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("PRV event must discard the checked-program batch: {outcome:?}");
        };
        assert_eq!(issue.rule_id(), "PRV-2");
    });
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the dark entailment hook cannot publish a failed PRV batch: {outcome:?}");
        };
        assert_eq!(issue.rule_id(), "PRV-2");
    });
}

#[test]
fn a_unit_without_postconditions_keeps_the_empty_schedule_fast_path() {
    let source = br#"fn helper(value: own i32) -> own i32 pure {
  return value;
}

fn main() -> own unit pure {
  let ignored = helper(value: 1_i32);
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("no-postcondition control must check completely: {outcome:?}");
        };
        assert!(checked.data.postcondition_schedule.components.is_empty());
    });
    assert_complete(source);
}
