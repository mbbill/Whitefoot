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

/// [FN-8] admits a clause computation in either spelling, and holds both to
/// the same subset.
///
/// The infix shape is the one that hides operands: an `expr`'s own atom is the
/// left operand and the tail carries the operator and the right, so a pass
/// that stops at the first `atom` child validates a third of the expression
/// and admits a trapping row, a `move`, a borrow, or a subscript in the rest.
#[test]
fn requires_holds_an_infix_row_to_the_same_subset_as_its_named_spelling() {
    with_semantics(
        b"fn f(a: own u64) -> own u64 traps requires {\n  \
          let ok = a <= 8_u64;\n  \
          check ok else trap \"bounded\";\n} {\n  \
          return a;\n}\n\n\
          fn main() -> own unit pure {\n  \
          return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("an infix clause let spells an admitted row: {outcome:?}");
            };
        },
    );
    // A bare `+` carries the trapping mode with no `.trap` in its spelling.
    assert_rule(
        b"fn f(x: own i32) -> own i32 traps requires {\n  \
          let raised = x + 1_i32;\n  \
          check igt(raised, x) else trap \"increases\";\n} {\n  \
          return x;\n}\n\n\
          fn main() -> own unit pure {\n  \
          return unit;\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // Reached only through the infix tail's own atom, never the expr's.
    assert_rule(
        b"fn f(xs: own array<u64, 4>, a: own u64) -> own u64 traps requires {\n  \
          check a <= xs[1_u64] else trap \"bounded\";\n} {\n  \
          return a;\n}\n\n\
          fn main() -> own unit pure {\n  \
          return unit;\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // An initializer is a computation, so a bare atom is not one.
    assert_rule(
        b"fn f(x: own i32) -> own i32 traps requires {\n  \
          let candidate = x;\n  \
          check igt(candidate, 0_i32) else trap \"positive\";\n} {\n  \
          return x;\n}\n\n\
          fn main() -> own unit pure {\n  \
          return unit;\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
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
