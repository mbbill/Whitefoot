use crate::{
    LexicalUseRole, ResolutionIssueKind, ResolutionOutcome, ResolutionRule, SemanticIssueKind,
    SemanticLocation, SemanticOutcome, SemanticRule,
};

use super::{assert_rule, assert_rule_at, with_resolution, with_semantics, with_semantics_dark};
use crate::semantic::entailment::{
    CallGoalDisposition, DerivationNode, FlowEventKind, FunctionPostconditionProof,
    PostconditionDisposition,
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

fn dispositions(proof: &FunctionPostconditionProof) -> Vec<PostconditionDisposition> {
    proof.exits.iter().map(|exit| exit.disposition).collect()
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
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  requires value == value;\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    assert_complete(source.as_bytes());
}

#[test]
fn ensures_smoke() {
    let source = format!(
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures out == value;\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
    );
    assert_complete(source.as_bytes());
}

#[test]
fn a_computed_constant_offset_is_not_an_fn9_relation_operand() {
    let source = br#"fn shifted(value: own u8) -> result: own u8 pure contract {
  define next = value +wrap 1_u8;
  ensures result == next;
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
        SemanticIssueKind::InvalidPostconditionRelation,
    );
}

#[test]
fn a_true_computed_constant_offset_is_still_outside_the_fn9_relation_form() {
    let source = br#"fn shifted(value: own u8) -> result: own u8 pure contract {
  define next = value -wrap 1_u8;
  ensures result == next;
} {
  let next = value -wrap 1_u8;
  return next;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        source,
        SemanticRule::Fn9,
        SemanticIssueKind::InvalidPostconditionRelation,
    );
}

#[test]
fn an_uncomputed_fn9_relation_still_publishes_to_its_caller() {
    let source = br#"fn identity(value: own u64) -> result: own u64 pure contract {
  ensures result == value;
} {
  return value;
}

fn select(values: own array<u8, 8>, index: own u64) -> result: own u8 pure contract {
  requires index < 8_u64;
} {
  let selected = identity(value: index);
  return values[selected];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

/// Contract clauses remain static proof input: `requires` admits the first
/// protected read, and the proved `ensures` relation admits the caller's
/// second protected read.
#[test]
fn contract_clauses_remain_available_to_the_originating_proof_context() {
    let source =
        br#"fn pick(table: own array<u8, 8>, index: own u64) -> value: own u64 pure contract {
  requires index < 8_u64;
  ensures value <= 7_u64;
} {
  let selected = table[index];
  let widened = cvt::<u8, u64>(selected);
  if widened <= 7_u64 {
    return widened;
  } else {
    return 7_u64;
  }
}

fn caller(table: own array<u8, 8>, lookup: own array<u8, 8>) -> result: own u8 pure {
  let value = pick(table: move table, index: 0_u64);
  return lookup[value];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
}

/// FN-9 publishes exactly the relations a callee proves.  The caller can use
/// both equality and a weaker ordering relation without restating either as a
/// body assertion.
#[test]
fn verified_exact_and_weak_ensures_are_consumed_by_the_caller() {
    let source = br#"fn exact(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn weak(value: own i32) -> result: own i32 pure contract {
  ensures result <= value;
} {
  return value;
}

fn need_same(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
} {
  return unit;
}

fn need_ordered(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left <= right;
} {
  return unit;
}

fn caller(value: own i32) -> result: own unit pure {
  let exact_result = exact(value: value);
  need_same(left: exact_result, right: value);
  let weak_result = weak(value: value);
  need_ordered(left: weak_result, right: value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
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
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures out == value;\n  ensures out >= value;\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
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
            assert!(proof.aggregate.discharged);
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
        "fn identity(value: own i32) -> out: own i32 pure contract {{\n  ensures out == value;\n  ensures out >= 0_i32;\n}} {{\n  return value;\n}}\n\n{COMMAND_MAIN}"
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
        assert!(first.aggregate.discharged);
        assert!(!second.aggregate.discharged);
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
        "fn left(value: own i32) -> out: own i32 pure contract {{\n  ensures out == value;\n}} {{\n  let ignored = right(value: value);\n  return value;\n}}\n\nfn right(value: own i32) -> out: own i32 pure contract {{\n  ensures out == value;\n  ensures out >= 0_i32;\n}} {{\n  let ignored = left(value: value);\n  return value;\n}}\n\n{COMMAND_MAIN}"
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
        assert!(left.entailment.postconditions[0].aggregate.discharged);
        let right = members
            .iter()
            .find(|function| function.name == "right")
            .expect("right function");
        assert!(right.entailment.postconditions[0].aggregate.discharged);
        assert!(!right.entailment.postconditions[1].aggregate.discharged);
    });
    assert_fn9_unproved(source.as_bytes());
}

#[test]
fn an_inhabited_routed_ensure_without_a_selected_exit_is_rejected() {
    let source = format!(
        "fn only_error(value: own i32) -> out: own Result<i32, i32> pure contract {{\n  ensures when Ok(value: payload): payload == value;\n}} {{\n  return Err<i32, i32>(error: value);\n}}\n\n{COMMAND_MAIN}"
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
        "fn impossible(value: own i32) -> out: own Result<i32, i32> pure contract {{\n  requires value == 0_i32;\n  requires value != 0_i32;\n  ensures when Ok(value: payload): payload == value;\n}} {{\n  return Err<i32, i32>(error: value);\n}}\n\n{COMMAND_MAIN}"
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
  ensures result == value;
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
        vec![PostconditionDisposition::Discharged]
    );
    assert!(proof.aggregate.discharged);
}
#[test]
fn entry_requirements_prove_postconditions_in_the_originating_context() {
    let source = br#"fn constrained(value: own i32) -> result: own i32 pure contract {
  requires value == 1_i32;
  ensures result == 1_i32;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(source);
    let proof = postcondition_proof(source, "constrained");
    assert_eq!(
        dispositions(&proof),
        vec![PostconditionDisposition::Discharged]
    );
    assert!(proof.aggregate.discharged);
}

#[test]
fn entry_image_writes_are_retained_and_prevent_false_discharge() {
    let source = br#"fn changed(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
        vec![PostconditionDisposition::Unproved]
    );
    let image = &proof.exits[0].entry_images[0];
    assert!(image.invalidation.is_some());
    assert!(!proof.aggregate.discharged);
    assert_rule_at(source, SemanticRule::Fn9, "return value;");
}

#[test]
fn a_moved_holder_consume_precedes_its_projected_call_write() {
    let source = br#"fn overwrite(out: &uniq i32) -> result: own unit writes(out) {
  set deref(out) = 1_i32;
  return unit;
}

fn transfer(out: &uniq i32) -> result: own i32 reads(out), writes(out) contract {
  ensures result == deref(out);
} {
  let before = deref(out);
  overwrite(out: move out);
  return before;
}

fn plain(out: &uniq i32) -> result: own i32 reads(out), writes(out) {
  let before = deref(out);
  overwrite(out: move out);
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
  ensures result == value;
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
    let source = br#"fn append(destination: &uniq buffer<u8>, capacity: own u64, filled: own u64, text: own slice<u8>) -> result: own u64 reads(destination, text), writes(destination) contract {
  requires capacity == len_of(deref(destination));
  requires filled <= capacity;
  ensures result <= capacity;
} {
  let spare = len_of(deref(destination));
  let admitted = filled <= spare;
  let length = len_of(text);
  if admitted {
    for @append (at in filled..spare) {
      let taken = at -wrap filled;
      let done = taken >= length;
      if done {
        return at;
      }
      let byte = text[taken];
      set deref(destination)[at] = byte;
    }
    return spare;
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
            PostconditionDisposition::Discharged,
            PostconditionDisposition::Discharged,
            PostconditionDisposition::Discharged,
        ]
    );
}

/// [MSR-3] a measure of a parameter is that parameter's entry datum, which
/// contains no place and which no [ENT-5] event kills, so neither an element
/// write nor the consume of the parameter's own root takes it away. What the
/// second half of this case pinned before the entry placement landed — an
/// unproved relation after the root was consumed — was the cost of reading
/// the live term where the rule says the entry value; the relation is now
/// discharged, and it is the same relation the writer stated.
///
/// A parameter of fragment type is not a measured value and has no measure
/// datum, so its entry image still invalidates; the third half is that
/// boundary, and the conformance case
/// `msr3-neg-a-parameter-value-written-back-loses-its-entry-image` carries
/// the same judgment as a source verdict.
#[test]
fn measure_entry_datums_survive_element_writes_and_root_replacement() {
    let element = br#"fn kept(values: own array<u8, 2>) -> result: own u64 pure contract {
  define size = len_of(values);
  ensures result == size;
} {
  set values[0_u64] = 1_u8;
  return len_of(values);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(element);
    let proof = postcondition_proof(element, "kept");
    assert_eq!(
        dispositions(&proof),
        vec![PostconditionDisposition::Discharged]
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
  define size = len_of(values);
  ensures result == size;
} {
  let size = len_of(values);
  let ignored = consume(values: move values);
  return size;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_complete(replacement);
    let proof = postcondition_proof(replacement, "replaced");
    assert_eq!(
        dispositions(&proof),
        vec![PostconditionDisposition::Discharged]
    );
    assert!(
        proof.exits[0]
            .entry_images
            .iter()
            .all(|image| image.invalidation.is_none())
    );

    // A fragment parameter has no measure and therefore no entry datum: its
    // entry image is the live place, and writing the parameter back takes it.
    let fragment = br#"fn shifted(count: own u64) -> result: own u64 pure contract {
  ensures result == count;
} {
  set count = 1_u64;
  return count;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let proof = postcondition_proof(fragment, "shifted");
    assert_eq!(
        dispositions(&proof),
        vec![PostconditionDisposition::Unproved]
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
  ensures result == value;
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
    assert!(proof.aggregate.discharged);
}

#[test]
fn an_earlier_verified_postcondition_discharges_a_fresh_direct_result() {
    let independent = br#"fn callee(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn caller(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
  ensures result == value;
} {
  return value;
}

fn caller(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
  ensures when Ok(value: payload): payload == value;
} {
  return Ok<i32, Overflow>(value: value);
}

fn direct(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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

fn observe(pair: &Pair) -> result: own i32 reads(pair.value) contract {
  ensures result == deref(pair).value;
} {
  return deref(pair).value;
}

fn caller(pair: &Pair) -> result: own i32 reads(pair.value) contract {
  ensures result == deref(pair).value;
} {
  let observed = observe(pair: pair);
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

fn touch(pair: &uniq Pair) -> result: own i32 reads(pair.kept), writes(pair.changed) contract {
  ensures result == deref(pair).kept;
} {
  set deref(pair).changed = 1_i32;
  return deref(pair).kept;
}

fn caller(pair: own Pair) -> result: own i32 reads(pair.kept), writes(pair.changed) contract {
  ensures result == pair.kept;
} {
  region {
    let holder = &uniq pair;
    let observed = touch(pair: move holder);
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
                .is_some_and(|proof| proof.aggregate.discharged),
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

/// [MSR-3] an `own` operand of a published relation denotes that call's call
/// datum — the operand's value at the pre-transfer point — so the relation
/// means what it reads as even when the same statement consumes the holder
/// the actual was reached through.
///
/// Until v0.44 this program was rejected: the relation's operand was the
/// place `deref(owner)`, `owner: move owner` consumed its holder in the same
/// statement, and `M(c,q)` failed. That refusal was conservative and not
/// sound-critical — the value `observe` received is 1 whatever later happens
/// to the box — and the datum is exactly the term that says so. The test is
/// kept as the positive it became, so the change is pinned rather than
/// silently absorbed.
#[test]
fn a_box_deref_actual_survives_a_cross_formal_owner_move_as_a_call_datum() {
    let source =
        br#"fn observe(value: own i32, owner: own box<i32>) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn caller() -> result: own i32 allocates(heap) contract {
  ensures result == 1_i32;
} {
  let owner = box_new(1_i32);
  let observed = observe(value: deref(owner), owner: move owner);
  return observed;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // `caller`'s own `ensures result == 1_i32` stays unproved for an
    // unrelated reason — `box_new` establishes no value equality on its
    // referent — so the program is still rejected. What changed is the route
    // below: the callee's relation now reaches the caller at all.
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
                .any(|node| { matches!(node, DerivationNode::PostconditionDirectResult { .. }) }),
            "the call datum publishes the relation the consumed holder used to delete"
        );
    });
}

/// P0 closes the two pre-move equalities while their common box referent is
/// still live. The resulting `observed == expected` theorem names only the two
/// copied scalar bindings, so consuming the original owner cannot invalidate
/// it.
#[test]
fn an_owner_move_preserves_a_materialized_holder_free_s12_consequence() {
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
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
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a holder-free pre-move consequence must survive: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function exists");
        super::entailment::validate_derivations(&caller.entailment);
        let [goal] = caller.entailment.call_goals.as_slice() else {
            panic!("caller retains exactly the guard requirement");
        };
        assert_eq!(goal.disposition, CallGoalDisposition::Discharged);
        let root = goal
            .derivation
            .expect("the discharged guard retains its derivation");
        let mut seen = vec![false; caller.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut materialized = false;
        let mut postcondition = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &caller.entailment.derivations.nodes[index];
            materialized |= matches!(retained, DerivationNode::MaterializedBound { .. });
            postcondition |= matches!(retained, DerivationNode::PostconditionCall { .. });
            stack.extend(retained.parent_ids());
        }
        assert!(
            materialized,
            "the surviving scalar equality is pre-kill materialized"
        );
        assert!(
            postcondition,
            "the equality retains the verified callee result as a parent"
        );
    });
}

#[test]
fn a_referent_write_kills_an_s12_relation_that_still_reads_that_referent() {
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
} {
  return unit;
}

fn caller() -> result: own unit pure {
  let source = 1_i32;
  let observed = observe(value: source);
  region {
    let writer = &uniq source;
    set deref(writer) = 2_i32;
  }
  guard(left: observed, right: source);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(
        source,
        SemanticRule::Fn8,
        "guard(left: observed, right: source)",
    );
}

#[test]
fn an_ordinary_fallback_survives_when_a_neighboring_s12_candidate_dies() {
    let source = br#"fn observe(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
} {
  return unit;
}

fn caller() -> result: own unit allocates(heap) {
  let owner = box_new(1_i32);
  let expected = deref(owner);
  let observed = observe(value: deref(owner));
  if observed == expected {
    sink(owner: move owner);
    guard(left: observed, right: expected);
  } else {
    sink(owner: move owner);
    return unit;
  }
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
  ensures result == value;
} {
  return value;
}

fn sink(owner: own box<i32>) -> result: own unit pure {
  return unit;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
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
  ensures result == value;
} {
  return value;
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
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
            1,
            "the exact receiver route is retained once in the source context"
        );
    });
}

#[test]
fn direct_same_binding_near_misses_retain_no_receiver_root_or_special_event() {
    let source = br#"fn echo(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn choose(first: own i32, second: own i32, value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn mentions_receiver(slot: own i32) -> result: own i32 pure contract {
  ensures result == slot;
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
  ensures when Ok(value: payload): payload == value;
} {
  return Ok<i32, Overflow>(value: value);
}

fn guard(left: own i32, right: own i32) -> result: own unit pure contract {
  requires left == right;
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
            1,
            "the selected receiver route is retained once in the source context"
        );
    });
}

#[test]
fn selected_receiver_nonfirst_additional_write_and_call_actual_shapes_retain_no_route() {
    let source = br#"struct Cell {
  value: i32;
}

fn selected(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Ok(value: payload): payload == value;
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

fn projected(cell: own Cell, replacement: own i32) -> result: own unit writes(cell.value) {
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
  ensures when Ok(value: payload): payload == value;
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
  ensures when Ok(value: payload): payload == 0_i32;
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
  ensures when Ok(value: payload): payload == value;
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
  define size = len_of(values);
  ensures result == size;
} {
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "result == size");
}

#[test]
fn projected_result_is_rejected_at_the_complete_final_relation() {
    let source = br#"fn projected(value: own i32) -> result: own i32 pure contract {
  ensures result.field == value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "result.field == value");
}

#[test]
fn a_nonbare_result_use_in_an_ensures_expression_is_still_rejected() {
    let source = br#"fn hidden(value: own i32) -> result: own i32 pure contract {
  ensures deref(result) == value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_at(source, SemanticRule::Fn9, "deref(result) == value");
}

#[test]
fn selected_returns_retain_deref_field_and_field_length_places() {
    let source = br#"struct Pair {
  value: i32;
}

struct Values {
  items: array<u8, 2>;
}

fn from_box(owner: own box<Pair>) -> result: own i32 reads(owner) contract {
  ensures result == deref(owner).value;
} {
  return deref(owner).value;
}

fn from_shared(owner: &Pair) -> result: own i32 reads(owner.value) contract {
  ensures result == deref(owner).value;
} {
  return deref(owner).value;
}

fn field_length(values: own Values) -> result: own u64 pure contract {
  define size = len_of(values.items);
  ensures result == size;
} {
  return len_of(values.items);
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

fn from_shared_alias(owner: &Pair) -> result: own i32 reads(owner.value) contract {
  ensures result == deref(owner).value;
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
        vec![PostconditionDisposition::Unproved]
    );
    assert_rule_at(source, SemanticRule::Fn9, "return deref(alias).value;");
}

#[test]
fn a_concrete_const_substitution_is_retained_with_a_selected_length() {
    let source =
        br#"fn count<const n: u64>(values: own array<u8, n>) -> result: own u64 pure contract {
  ensures result == result;
} {
  return len_of(values);
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 1>(0_u8);
  let one = count::<1>(values: move values);
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
  ensures result == 1_i32;
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
    let source = br#"contract Invalid<T: affine> {
}

fn identity(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
  ensures result == value;
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
  ensures result == missing;
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
  ensures result == missing;
} {
  return value;
}

fn second() -> result: own unit pure contract {
  ensures result == result;
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
  ensures result == missing;
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
  define cvt = value == value;
  ensures result == value;
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
            ResolutionIssueKind::ReservedName { spelling, .. } if spelling == "cvt"
        ));
    });
}

#[test]
fn unused_generic_entry_issue_precedes_its_body_semantics() {
    let source = br#"fn generic<T: affine>(value: own T) -> result: own T pure contract {
  ensures result == missing;
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
  ensures when ForeignCase(value: payload): payload == value;
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
    let source = br#"fn identity<T: affine>(value: own T) -> result: own T pure contract {
  ensures result == result;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let good = identity::<i32>(value: 1_i32);
  let flag = True();
  let bad = identity::<Bool>(value: flag);
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
  ensures result == missing;
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
  ensures result == result;
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
    let source = br#"fn generic<T: affine>(value: own T) -> result: own T pure contract {
  define cvt = value == value;
  ensures result == value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let unavailable = generic::<Missing>(value: unit);
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
    let source =
        br#"fn generic<T: affine, const n: u64>(value: own T) -> result: own T pure contract {
  define cvt = value == value;
  ensures result == value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let unavailable = generic::<unit, missing>(value: unit);
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
  ensures result == missing;
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
  ensures result == result;
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
    let source =
        br#"fn unavailable<T: affine>(value: own array<T, 1>) -> result: own T pure contract {
  ensures result == missing;
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
    let source =
        br#"fn unavailable<T: affine>(value: own array<T, 1>) -> result: own T pure contract {
  ensures result == missing;
} {
  return value;
}

fn invalid() -> result: own unit pure contract {
  ensures result == result;
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
  ensures result == result;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let bad = generic::<i32, i32>(value: 1_i32);
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
  ensures result == result;
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
    let source = br#"struct Invalid<T: affine> {
  values: array<T, 2>;
}

fn probe(value: own Invalid<i32>) -> result: own unit pure contract {
  ensures result == missing;
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
    let source = br#"struct Invalid<T: affine> {
  values: array<T, 2>;
}

fn invalid() -> result: own unit pure contract {
  ensures result == missing;
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
  ensures result == value;
} {
  let ignored = bridge(value: value);
  return value;
}

fn leaf(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  return value;
}

fn middle(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
  ensures result == value;
} {
  let ignored = second(value: value);
  return value;
}

fn second(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
            assert!(proof.aggregate.discharged);
        }
    });
    assert_complete(source);
}

#[test]
fn an_independently_proved_self_recursive_component_publishes_its_summary() {
    let source = br#"fn recursive(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
  ensures result == value;
} {
  let ignored = right(value: value);
  return value;
}

fn right(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
                .aggregate
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
                .aggregate
                .discharged
        );
    });
    assert_fn9_unproved(source);
}

#[test]
fn a_seedless_mutual_postcondition_cycle_publishes_no_summary() {
    let source = br#"fn first(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
} {
  let called = second(value: value);
  return called;
}

fn second(value: own i32) -> result: own i32 pure contract {
  ensures result == value;
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
            assert!(!proof.aggregate.discharged);
            assert!(proof.summary.is_none());
        }
    });
    assert_fn9_unproved(source);
}

#[test]
fn concrete_generic_instances_receive_distinct_verified_summary_identities() {
    let source = br#"fn identity<T: Int>(value: own T) -> result: own T pure contract {
  ensures result == value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let small = identity::<i32>(value: 1_i32);
  let wide = identity::<u64>(value: 1_u64);
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
  ensures result < value;
} {
  return value;
}

fn wrapper<U: affine>() -> result: own unit pure {
  let ignored = bad::<u8>(value: 0_u8);
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
