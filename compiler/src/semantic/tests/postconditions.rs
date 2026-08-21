use crate::{
    LexicalUseRole, ResolutionIssueKind, ResolutionOutcome, ResolutionRule, SemanticIssueKind,
    SemanticLocation, SemanticOutcome, SemanticRule,
};

use super::{assert_rule, assert_rule_at, with_resolution, with_semantics, with_semantics_dark};
use crate::semantic::entailment::{
    DerivationNode, FlowEventKind, FunctionPostconditionProof, PostconditionDisposition, ProofView,
};
use crate::semantic::model::{CheckedBodyDisposition, CheckedExpression, CheckedStatement};

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
            .postconditions
            .first()
            .cloned()
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

const COMMAND_MAIN: &str =
    "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

#[test]
fn command_entry_smoke() {
    assert_complete(COMMAND_MAIN.as_bytes());
}

#[test]
fn requires_smoke() {
    let source = format!(
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  requires ieq(value, value);\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    assert_complete(source.as_bytes());
}

#[test]
fn ensures_smoke() {
    let source = format!(
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures ieq(out, value);\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_non_bool_ensures_predicate_cites_op5() {
    let source = format!(
        "fn invalid(value: own i32) -> out: own i32 pure contract {{\n  ensures value;\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    assert_rule(
        source.as_bytes(),
        SemanticRule::Op5,
        SemanticIssueKind::InvalidPredicateCondition,
    );
}

#[test]
fn plural_ensures_are_proved_and_published_as_independent_relations() {
    let source = format!(
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures ieq(out, value);\n  ensures ige(out, value);\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("plural ensures fixture must check: {outcome:?}");
        };
        let identity = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "identity")
            .expect("identity function");
        assert_eq!(identity.postconditions.len(), 2);
        assert_eq!(identity.entailment.postconditions.len(), 2);
        for (ordinal, proof) in identity.entailment.postconditions.iter().enumerate() {
            assert_eq!(proof.relation_ordinal as usize, ordinal);
            assert!(proof.complete.discharged);
            assert!(proof.summary.is_some());
        }
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&identity.id))
            .expect("identity component");
        assert_eq!(component.summaries.len(), 2);
        assert_eq!(component.summaries[0].relation_ordinal, 0);
        assert_eq!(component.summaries[1].relation_ordinal, 1);
    });
}

#[test]
fn one_failed_ensure_withholds_every_summary_in_its_component() {
    let source = format!(
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures ieq(out, value);\n  ensures ige(out, 0_i32);\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark plural failure fixture must remain inspectable: {outcome:?}");
        };
        let identity = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "identity")
            .expect("identity function");
        let [first, second] = identity.entailment.postconditions.as_slice() else {
            panic!("both relation proofs must be retained");
        };
        assert!(first.complete.discharged);
        assert!(!second.complete.discharged);
        assert!(first.summary.is_none());
        assert!(second.summary.is_none());
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&identity.id))
            .expect("identity component");
        assert!(component.summaries.is_empty());
    });
    assert_fn9_unproved(source.as_bytes());
}

#[test]
fn one_failed_relation_withholds_every_summary_in_a_mutual_scc() {
    let source = format!(
        "fn left(value: own i32) -> out: own i32 pure contract {{\n  ensures ieq(out, value);\n}} {{\n  let ignored = right(value: value);\n  return value;\n}}\n\nfn right(value: own i32) -> out: own i32 pure contract {{\n  ensures ieq(out, value);\n  ensures ige(out, 0_i32);\n}} {{\n  let ignored = left(value: value);\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    with_semantics_dark(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark mutual failure fixture must remain inspectable: {outcome:?}");
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
                .postconditions
                .iter()
                .all(|proof| proof.summary.is_none())
        }));
        let left = members
            .iter()
            .find(|function| function.name == "left")
            .expect("left function");
        assert!(left.entailment.postconditions[0].complete.discharged);
        let right = members
            .iter()
            .find(|function| function.name == "right")
            .expect("right function");
        assert!(right.entailment.postconditions[0].complete.discharged);
        assert!(!right.entailment.postconditions[1].complete.discharged);
    });
    assert_fn9_unproved(source.as_bytes());
}

#[test]
fn an_inhabited_routed_ensure_without_a_selected_exit_is_rejected() {
    let source = format!(
        "fn only_error(value: own i32) -> out: own Result<i32, i32> pure contract {{\n  ensures when Ok(value: payload): ieq(payload, value);\n}} {{\n  return Err<i32, i32>(error: value);\n}}\n\n{COMMAND_MAIN}"
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("inhabited empty route must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn9);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::NoSelectedNormalExit { .. }
        ));
    });
}

#[test]
fn an_uninhabited_routed_ensure_needs_no_exit_and_publishes_no_summary() {
    let source = format!(
        "fn impossible(value: own i32) -> out: own Result<i32, i32> pure contract {{\n  requires ieq(value, 0_i32);\n  requires ine(value, 0_i32);\n  ensures when Ok(value: payload): ieq(payload, value);\n}} {{\n  return Err<i32, i32>(error: value);\n}}\n\n{COMMAND_MAIN}"
    );
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("uninhabited empty route must be accepted: {outcome:?}");
        };
        let impossible = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "impossible")
            .expect("impossible function");
        assert!(matches!(
            impossible.body_disposition,
            CheckedBodyDisposition::Uninhabited { .. }
        ));
        assert!(impossible.entailment.postconditions.is_empty());
        assert!(
            checked
                .data
                .postcondition_schedule
                .components
                .iter()
                .all(|component| component
                    .summaries
                    .iter()
                    .all(|summary| summary.function != impossible.id))
        );
    });
}

#[test]
fn a_checked_plain_postcondition_is_proved_at_its_selected_exit() {
    let source = br#"fn identity(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
fn body_claims_and_s4_are_routed_to_the_fixed_postcondition_views() {
    let body_claim = br#"fn guarded(value: own i32) -> result: own i32 traps contract {
  ensures ieq(result, 1_i32);
} {
  claim body: ieq(value, 1_i32) because "premises: fixture context: body\nderivation: the fixture supplies the written predicate to exercise the selected checker path\nconclusion: the written predicate holds in the intended fixture state\nchecker gap: the fixture models a proof fact outside the selected checker rules\nconsumers: the following source operation or call is the test subject";
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(body_claim);
    let proof = postcondition_proof(body_claim, "guarded");
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

    let s4 = br#"fn constrained(value: own i32) -> result: own i32 pure contract {
  requires ieq(value, 1_i32);
  ensures ieq(result, 1_i32);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn changed(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  set value = 1_i32;
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn overwrite['r](out: &uniq 'r i32) -> result: own unit writes('r) {
  set deref(out) = 1_i32;
  return unit;
}

fn transfer['r](out: &uniq 'r i32) -> result: own i32 reads('r), writes('r) contract {
  ensures ieq(result, deref(out));
} {
  let before = deref(out);
  overwrite<'r>(out: move out);
  return before;
}

fn plain['r](out: &uniq 'r i32) -> result: own i32 reads('r), writes('r) {
  let before = deref(out);
  overwrite<'r>(out: move out);
  return before;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
            .postconditions
            .first()
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
        assert!(plain.entailment.postconditions.is_empty());
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
    let source = br#"fn looped(value: own i32, stop: own Bool) -> result: own i32 pure contract {
  ensures ieq(result, value);
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
            .postconditions
            .first()
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
    let source = br#"fn append['d, 'm](destination: &uniq 'd buffer<u8>, filled: own u64, text: own slice<'m, u8>) -> result: own u64 reads('d 'm), writes('d) contract {
  define capacity = len(deref(destination));
  define admitted = ile(filled, capacity);
  requires admitted;
  ensures ile(result, capacity);
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let element = br#"fn kept(values: own array<u8, 2>) -> result: own u64 pure contract {
  define size = len(values);
  ensures ieq(result, size);
} {
  set values[0_u64] = 1_u8;
  return len(values);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

    let replacement = br#"fn consume(values: own array<u8, 2>) -> result: own unit pure {
  return unit;
}

fn replaced(values: own array<u8, 2>) -> result: own u64 pure contract {
  define size = len(values);
  ensures ieq(result, size);
} {
  let size = len(values);
  let ignored = consume(values: move values);
  return size;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn branch(value: own i32, choose: own Bool) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  if choose {
    return value;
  } else {
    return value;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let independent = br#"fn callee(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn caller(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = callee(value: value);
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(independent);

    let dependent = br#"fn callee(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn caller(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let called = callee(value: value);
  return called;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(dependent);
}

#[test]
fn an_earlier_ok_summary_is_available_only_at_direct_match_arm_entry() {
    let source = br#"fn callee(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, value);
} {
  return Ok<i32, Overflow>(value: value);
}

fn direct(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
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

fn delivered(value: own i32) -> result: own i32 pure {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_borrowed_formal_substitution_consumes_its_one_formal_deref() {
    let source = br#"struct Pair {
  value: i32;
}

fn observe['r](pair: &'r Pair) -> result: own i32 reads('r) contract {
  ensures ieq(result, deref(pair).value);
} {
  return deref(pair).value;
}

fn caller['r](pair: &'r Pair) -> result: own i32 reads('r) contract {
  ensures ieq(result, deref(pair).value);
} {
  let observed = observe<'r>(pair: pair);
  return observed;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn touch['r](pair: &uniq 'r Pair) -> result: own i32 reads('r), writes('r) contract {
  ensures ieq(result, deref(pair).kept);
} {
  set deref(pair).changed = 1_i32;
  return deref(pair).kept;
}

fn caller(pair: own Pair) -> result: own i32 pure contract {
  ensures ieq(result, pair.kept);
} {
  region 'r {
    let holder = &uniq 'r pair;
    let observed = touch<'r>(pair: move holder);
    return observed;
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
                .postconditions
                .first()
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
        br#"fn observe(value: own i32, owner: own box<i32>) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn caller() -> result: own i32 allocates(heap) contract {
  ensures ieq(result, 1_i32);
} {
  let owner = box_new(1_i32);
  let observed = observe(value: deref(owner), owner: move owner);
  return observed;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn caller() -> result: own unit allocates(heap) {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  sink(owner: move owner);
  guard(left: observed, right: expected);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Fn8,
        "guard(left: observed, right: expected)",
    );
}

#[test]
fn an_ordinary_fallback_survives_when_a_neighboring_s12_candidate_dies() {
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn opaque_identity(value: own i32) -> result: own i32 pure {
  return value;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn caller() -> result: own unit allocates(heap), traps {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  let fallback = opaque_identity(value: expected);
  claim ordinary_fallback: ieq(fallback, expected) because "premises: fallback is returned by opaque_identity, whose body returns its value parameter unchanged\nderivation: the call argument is expected, so fallback equals expected\nconclusion: ieq(fallback, expected) is true\nchecker gap: ENT does not publish an uncontracted user-call result equality\nconsumers: guard requires this equality after the neighboring S12 owner-supported candidate is killed";
  sink(owner: move owner);
  guard(left: fallback, right: expected);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_joined_holder_free_consequence_survives_the_original_owner_move() {
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn caller(choose: own Bool) -> result: own unit allocates(heap) {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_direct_same_binding_call_result_establishes_only_after_the_target_kill() {
    let source = br#"fn choose(ignored: own i32, value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn caller(slot: own i32, replacement: own i32) -> result: own unit pure {
  set slot = choose(ignored: slot, value: replacement);
  guard(left: slot, right: replacement);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn echo(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn choose(first: own i32, second: own i32, value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn mentions_receiver(slot: own i32) -> result: own i32 pure contract {
  ensures ieq(result, slot);
} {
  set slot = echo(value: slot);
  return slot;
}

fn repeated_receiver(slot: own i32, replacement: own i32) -> result: own unit pure {
  set slot = choose(first: slot, second: slot, value: replacement);
  return unit;
}

fn distinct_receiver(slot: own i32, other: own i32, replacement: own i32) -> result: own unit pure {
  set slot = choose(first: other, second: other, value: replacement);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source =
        br#"fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, value);
} {
  return Ok<i32, Overflow>(value: value);
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires ieq(left, right);
} {
  return unit;
}

fn caller(outer: own i32, replacement: own i32) -> result: own unit pure {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, value);
} {
  return Ok<i32, Overflow>(value: value);
}

fn nonfirst(outer: own i32, replacement: own i32) -> result: own unit pure {
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

fn additional_write(outer: own i32, replacement: own i32) -> result: own unit pure {
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

fn call_actual(outer: own i32) -> result: own unit pure {
  match selected(value: outer) {
    Ok(value: payload) => {
      set outer = payload;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn projected(cell: own Cell, replacement: own i32) -> result: own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set cell.value = payload;
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

fn computed(outer: own i32, replacement: own i32) -> result: own unit pure {
  match selected(value: replacement) {
    Ok(value: payload) => {
      set outer = iand(payload, 0_i32);
    }
    Err(error: problem) => {
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source =
        br#"fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, value);
} {
  return Ok<i32, Overflow>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn an_ok_selector_rejects_an_empty_selected_exit_set() {
    let source = br#"fn unselected() -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, 0_i32);
} {
  let error = Overflow();
  return Err<i32, Overflow>(error: error);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn stored(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): ieq(payload, value);
} {
  let outcome = Ok<i32, Overflow>(value: value);
  return move outcome;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "return move outcome;");
}

#[test]
fn relation_length_rejects_a_named_constant_root() {
    let source = br#"const values: array<i32, 1> =[0_i32];

fn length() -> result: own u64 pure contract {
  define size = len(values);
  ensures ieq(result, size);
} {
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "ieq(result, size)");
}

#[test]
fn projected_result_is_rejected_at_the_complete_final_relation() {
    let source = br#"fn projected(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result.field, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "ieq(result.field, value)");
}

#[test]
fn a_nonbare_result_use_in_an_ensures_expression_is_still_rejected() {
    let source = br#"fn hidden(value: own i32) -> result: own i32 pure contract {
  ensures ieq(deref(result), value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "ieq(deref(result), value)");
}

#[test]
fn selected_returns_retain_deref_field_and_field_length_places() {
    let source = br#"struct Pair {
  value: i32;
}

struct Values {
  items: array<u8, 2>;
}

fn from_box(owner: own box<Pair>) -> result: own i32 pure contract {
  ensures ieq(result, deref(owner).value);
} {
  return deref(owner).value;
}

fn from_shared['r](owner: &'r Pair) -> result: own i32 reads('r) contract {
  ensures ieq(result, deref(owner).value);
} {
  return deref(owner).value;
}

fn field_length(values: own Values) -> result: own u64 pure contract {
  define size = len(values.items);
  ensures ieq(result, size);
} {
  return len(values.items);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn a_holder_alias_does_not_change_the_selected_return_term_identity() {
    let source = br#"struct Pair {
  value: i32;
}

fn from_shared_alias['r](owner: &'r Pair) -> result: own i32 reads('r) contract {
  ensures ieq(result, deref(owner).value);
} {
  let alias = owner;
  return deref(alias).value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"fn count<const n: u64>(values: own array<u8, n>) -> result: own u64 pure contract {
  ensures ieq(result, result);
} {
  return len(values);
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 1>(0_u8);
  let one = count<1>(values: move values);
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

#[test]
fn an_ensures_bearing_conformance_binding_is_fn3_before_proof() {
    let source = br#"contract Maker {
  fn make() -> result: own i32 pure;
}

fn make() -> result: own i32 pure contract {
  ensures ieq(result, 1_i32);
} {
  return 1_i32;
}

conform i32: Maker {
  make = make;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn3, "make = make;");
}

#[test]
fn an_invalid_contract_precedes_the_postcondition_proof_boundary() {
    let source = br#"contract Invalid<T> {
}

fn identity(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
  fn combine(x: own u64, y: own u64) -> result: own u64 pure;
  law identity(combine, unit);
}

fn identity(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn invalid() -> result: own unit pure contract {
  ensures ieq(result, missing);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn first(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, missing);
} {
  return value;
}

fn second() -> result: own unit pure contract {
  ensures ieq(result, result);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn unresolved(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, missing);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn poisoned(value: own i32) -> result: own i32 pure contract {
  define ilt = ieq(value, value);
  ensures ieq(result, value);
} {
  return Missing();
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected contract-definition inventory issue, got {outcome:?}");
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
    let source = br#"fn generic<T>(value: own T) -> result: own T pure contract {
  ensures ieq(result, missing);
} {
  return box_new(value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
  ForeignCase(value: i32);
}

fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when ForeignCase(value: payload): ieq(payload, value);
} {
  return Ok<i32, Overflow>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        assert_eq!(&source[start..end], b"ForeignCase(value: payload)");
    });
}

#[test]
fn concrete_generic_instances_do_not_reuse_symbolic_selector_class() {
    let source = br#"fn identity<T>(value: own T) -> result: own T pure contract {
  ensures ieq(result, result);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let good = identity<i32>(value: 1_i32);
  let flag = True();
  let bad = identity<Bool>(value: flag);
  return exit_status(code: 0_u8);
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
    let source = br#"fn probe(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, missing);
} {
  return value;
}

fn main() -> result: own i32 pure {
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
    let source = br#"fn invalid() -> result: own unit pure contract {
  ensures ieq(result, result);
} {
  return unit;
}

fn main() -> result: own i32 pure {
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
    let source = br#"fn invalid<T: Float>(value: own T) -> result: own T pure contract {
  ensures feq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn generic<T>(value: own T) -> result: own T pure contract {
  define ilt = ieq(value, value);
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let unavailable = generic<Missing>(value: unit);
  return exit_status(code: 0_u8);
}
"#;
    with_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!(
                "contract-definition inventory must beat an unavailable type argument: {outcome:?}"
            );
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });
}

#[test]
fn unavailable_const_argument_does_not_invent_a_selector_instance() {
    let source = br#"fn generic<T, const n: u64>(value: own T) -> result: own T pure contract {
  define ilt = ieq(value, value);
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let unavailable = generic<unit, missing>(value: unit);
  return exit_status(code: 0_u8);
}
"#;
    with_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!(
                "contract-definition inventory must beat an unavailable const argument: {outcome:?}"
            );
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });
}

#[test]
fn unrelated_invalid_constant_does_not_suppress_an_independent_selector() {
    let source = br#"const bad: u8 = 1_u16;

fn invalid() -> result: own unit pure contract {
  ensures ieq(result, missing);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn invalid() -> result: own unit pure contract {
  ensures ieq(result, result);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn unavailable<T>(value: own array<T, 1>) -> result: own T pure contract {
  ensures ieq(result, missing);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn unavailable<T>(value: own array<T, 1>) -> result: own T pure contract {
  ensures ieq(result, missing);
} {
  return value;
}

fn invalid() -> result: own unit pure contract {
  ensures ieq(result, result);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn generic<T: Int>(value: own T) -> result: own T pure contract {
  ensures ieq(result, result);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let bad = generic<i32, i32>(value: 1_i32);
  return exit_status(code: 0_u8);
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
    let source = br#"fn broken<const n: Bool>() -> result: own unit pure {
  return unit;
}

fn invalid() -> result: own unit pure contract {
  ensures ieq(result, result);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn probe(value: own Invalid<i32>) -> result: own unit pure contract {
  ensures ieq(result, missing);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn invalid() -> result: own unit pure contract {
  ensures ieq(result, missing);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn top(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = bridge(value: value);
  return value;
}

fn leaf(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn middle(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = leaf(value: value);
  return value;
}

fn bridge(value: own i32) -> result: own i32 pure {
  let ignored = middle(value: value);
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
                .postconditions
                .first()
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
    let source = br#"fn first(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = second(value: value);
  return value;
}

fn second(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = first(value: value);
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
                .postconditions
                .first()
                .expect("mutual proof");
            assert_eq!(proof.summary.as_ref(), Some(summary));
            assert!(proof.complete.discharged);
        }
    });
    assert_complete(source);
}

#[test]
fn an_independently_proved_self_recursive_component_publishes_its_summary() {
    let source = br#"fn recursive(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = recursive(value: value);
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
            .postconditions
            .first()
            .and_then(|proof| proof.summary.as_ref())
            .expect("independent recursive summary publishes");
        assert_eq!(component.summaries, vec![summary.clone()]);
    });
    assert_complete(source);
}

#[test]
fn one_failed_mutual_member_withholds_the_whole_component_summary_batch() {
    let source = br#"fn left(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = right(value: value);
  return value;
}

fn right(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let called = left(value: value);
  return called;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
                .postconditions
                .first()
                .is_some_and(|proof| proof.summary.is_none())
        }));
        assert!(
            members
                .iter()
                .find(|function| function.name == "left")
                .unwrap()
                .entailment
                .postconditions
                .first()
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
                .postconditions
                .first()
                .unwrap()
                .complete
                .discharged
        );
    });
    assert_fn9_unproved(source);
}

#[test]
fn a_seedless_mutual_postcondition_cycle_publishes_no_summary() {
    let source = br#"fn first(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let called = second(value: value);
  return called;
}

fn second(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let called = first(value: value);
  return called;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
            let proof = function.entailment.postconditions.first().unwrap();
            assert!(!proof.complete.discharged);
            assert!(proof.summary.is_none());
        }
    });
    assert_fn9_unproved(source);
}

#[test]
fn concrete_generic_instances_receive_distinct_verified_summary_identities() {
    let source = br#"fn identity<T: Int>(value: own T) -> result: own T pure contract {
  ensures ieq(result, value);
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let small = identity<i32>(value: 1_i32);
  let wide = identity<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
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
                    .postconditions
                    .first()
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
fn a_concrete_instance_named_only_by_an_uninstantiated_generic_still_checks_fn9() {
    let source = br#"fn bad<T: Int>(value: own T) -> result: own T pure contract {
  ensures ilt(result, value);
} {
  return value;
}

fn wrapper<U>() -> result: own unit pure {
  let ignored = bad<u8>(value: 0_u8);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the replayed bad<u8> postcondition must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn9);
        let SemanticIssueKind::UndischargedPostcondition(detail) = issue.kind() else {
            panic!("FN-9 must retain its concrete proof disposition: {issue:?}");
        };
        assert_eq!(
            detail.disposition,
            crate::PostconditionProofDisposition::Refuted
        );
        assert!(detail.concrete_function.contains("bad"));
    });
}

#[test]
fn accepted_provenance_views_use_the_finalized_function_derivation_ids() {
    let source = br#"fn normalized(value: own i32) -> result: own i32 pure contract {
  requires ieq(value, 1_i32);
  ensures ieq(result, value);
} {
  return 1_i32;
}

fn caller() -> result: own i32 pure contract {
  ensures ieq(result, 1_i32);
} {
  let called = normalized(value: 1_i32);
  return called;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let source = br#"fn identity(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  return value;
}

fn relay(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let observed = identity(value: value);
  return observed;
}

fn read(values: own array<u8, 4>, position: own u64) -> result: own u8 traps {
  let room = len(values);
  claim bounded: ilt(position, room) because "premises: fixture context: claimed parameter bound\nderivation: the fixture supplies the written predicate to exercise the selected checker path\nconclusion: the written predicate holds in the intended fixture state\nchecker gap: the fixture models a proof fact outside the selected checker rules\nconsumers: the following source operation or call is the test subject";
  return values[position];
}

command fn main(command.args as args: own Args) -> status: own ExitStatus traps {
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
    let source = br#"fn helper(value: own i32) -> result: own i32 pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let ignored = helper(value: 1_i32);
  return exit_status(code: 0_u8);
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
