use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::CheckedStatement;
use super::{assert_rule, with_semantics};

#[test]
fn checked_requires_block_is_an_executable_function_prologue() {
    let source = include_bytes!("../../../../tests/conformance/cases/fn8-pos-requires-run.wf");
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("valid checked requires prologue must be implemented: {outcome:?}");
        };
        assert_eq!(checked.data.functions[0].requires.len(), 2);
    });
}

#[test]
fn requires_rejects_user_calls_and_trapping_operations() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-user-call.wf"),
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-trapping-op.wf"),
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
}

#[test]
fn requires_check_participates_in_exact_effects_and_op5_typing() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-missing-traps.wf"),
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-non-bool-check.wf"),
        SemanticRule::Op5,
        SemanticIssueKind::InvalidCheckCondition,
    );
}

#[test]
fn requires_locals_are_distinct_from_same_named_body_locals() {
    let source =
        include_bytes!("../../../../tests/conformance/cases/fn8-pos-requires-name-reuse.wf");
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("requires and body scopes must remain disjoint: {outcome:?}");
        };
        let function = &checked.data.functions[0];
        let CheckedStatement::Let {
            binding: requires_binding,
            ..
        } = &function.requires[0]
        else {
            panic!("requires must retain its local");
        };
        let CheckedStatement::Let {
            binding: body_binding,
            ..
        } = &function.body[0]
        else {
            panic!("body must retain its distinct local");
        };
        assert_ne!(requires_binding, body_binding);
    });
}
