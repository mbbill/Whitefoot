use crate::lowering::{OverlapLowering, lower_checked};
use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::entailment::{CallGoalDisposition, CallGoalEvidence};
use super::super::goal::{GoalDatum, GoalExpression, GoalOperation, GoalProjection};
use super::super::model::{
    CheckedConst, CheckedExpression, CheckedFlatElement, CheckedIntegerOperation,
    CheckedNominalKind, CheckedStatement, CheckedType, CheckedValue, IntegerType,
};
use super::{assert_rule, with_semantics, with_semantics_dark};

#[test]
fn a_non_bool_requires_predicate_cites_op5() {
    assert_rule(
        br#"fn invalid(value: own i32) -> out: own i32 pure contract {
  requires value;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Op5,
        SemanticIssueKind::InvalidPredicateCondition,
    );
}

#[test]
fn plural_requires_keep_every_source_occurrence_at_the_call() {
    let source = br#"fn exact(value: own i32) -> out: own i32 pure contract {
  requires value == 1_i32;
  requires value == 1_i32;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let value = 1_i32;
  let observed = exact(value: value);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("both requirement occurrences must discharge: {outcome:?}");
        };
        let exact = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "exact")
            .expect("exact function");
        assert_eq!(exact.requirements.len(), 2);
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("command main");
        let CheckedStatement::Let {
            value: CheckedExpression::UserCall { requirements, .. },
            ..
        } = &main.body[1]
        else {
            panic!("the second statement must retain the user call");
        };
        assert_eq!(requirements.len(), 2);
        assert_eq!(main.entailment.call_goals.len(), 2);
        assert_ne!(
            main.entailment.call_goals[0].requires_clause,
            main.entailment.call_goals[1].requires_clause
        );
    });
}

#[test]
fn a_later_requires_clause_is_not_dropped_after_an_earlier_success() {
    let source = br#"fn exact(value: own i32) -> out: own i32 pure contract {
  requires value == 1_i32;
  requires value == 2_i32;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let value = 1_i32;
  let observed = exact(value: value);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("the refuted second requirement must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!("FN-8 must identify the failing requirement clause: {issue:?}");
        };
        assert_eq!(
            detail.disposition,
            crate::CallRequirementDisposition::Refuted
        );
        assert!(!detail.requires_clause.components().is_empty());
    });
}

fn instantiated_call_goal_arguments(call: &CheckedExpression) -> &[GoalExpression] {
    let CheckedExpression::UserCall { requirements, .. } = call else {
        panic!("call must carry its instantiated requirement");
    };
    let [requirement] = requirements.as_slice() else {
        panic!("call must carry exactly one instantiated requirement");
    };
    let GoalExpression::Operation { arguments, .. } = &requirement.goal.root else {
        panic!("call requirement must remain an operation goal");
    };
    arguments
}

fn contains_array_fill(expression: &GoalExpression) -> bool {
    match expression {
        GoalExpression::Operation { row, arguments, .. } => {
            matches!(row, GoalOperation::ArrayFill { .. })
                || arguments.iter().any(contains_array_fill)
        }
        GoalExpression::Datum(_) => false,
    }
}

#[test]
fn requires_retains_one_static_goal_without_a_second_expression_tree() {
    let source = br#"fn bounded(x: own i32) -> result: own i32 pure contract {
  define permitted = x >= 0_i32;
  requires permitted;
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  let x = 7_i32;
  let value = bounded(x: x);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("valid checked requirement must be implemented: {outcome:?}");
        };
        let function = &checked.data.functions[0];
        let requirement = function
            .requirements
            .first()
            .expect("checked requires retains its boundary metadata");
        assert!(!requirement.clause.components().is_empty());
    });
}

#[test]
fn requires_rejects_user_calls_and_partial_operations() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-user-call.wf"),
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-partial-op.wf"),
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
}

#[test]
fn requires_is_static_and_keeps_op5_typing() {
    with_semantics(
        br#"fn admitted(value: own i32) -> result: own i32 pure contract {
  requires value >= 0_i32;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a requirement is not an executed EFF-2 occurrence: {outcome:?}");
            };
        },
    );
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn8-neg-requires-non-bool-check.wf"),
        SemanticRule::Op5,
        SemanticIssueKind::InvalidPredicateCondition,
    );
}

/// [FN-8] admits a clause computation in either spelling, and holds both to
/// the same subset.
///
/// The infix shape is the one that hides operands: an `expr`'s own atom is the
/// left operand and the tail carries the operator and the right, so a pass
/// that stops at the first `atom` child validates a third of the expression
/// and admits a proof-required row, a `move`, a borrow, or a subscript in the
/// rest.
#[test]
fn requires_holds_an_infix_row_to_the_same_subset_as_its_named_spelling() {
    with_semantics(
        b"fn f(a: own u64) -> result: own u64 pure contract {\n  \
          define doubled = a *wrap 2_u64;\n  \
          requires doubled <= 16_u64;\n} {\n  \
          return a;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("an infix clause let spells an admitted row: {outcome:?}");
            };
        },
    );
    // A bare `+` is proof-required and therefore unavailable in a definition.
    assert_rule(
        b"fn f(x: own i32) -> result: own i32 pure contract {\n  \
          define raised = x + 1_i32;\n  \
          requires raised > x;\n} {\n  \
          return x;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // Reached only through the infix tail's own atom, never the expr's.
    assert_rule(
        b"fn f(xs: own array<u64, 4>, a: own u64) -> result: own u64 pure contract {\n  \
          define sum = a +wrap xs[1_u64];\n  \
          requires sum <= 8_u64;\n} {\n  \
          return a;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // An initializer is a computation, so a bare atom is not one.
    assert_rule(
        b"fn f(x: own i32) -> result: own i32 pure contract {\n  \
          define candidate = x;\n  \
          requires candidate > 0_i32;\n} {\n  \
          return x;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
}

/// [FN-8]'s clause local is an "own **copy** value", and after the v0.23
/// annotation deletion the only thing that can say so is the derived type.
///
/// The written type used to carry this. Nothing else does: every row below is
/// pure, total and non-trapping, so the admitted-spelling filter passes it,
/// and the value it yields is still not a copy type.
#[test]
fn requires_holds_a_clause_local_to_a_copy_type() {
    // Non-copy by shape: `array_new` is the reachable aggregate row, since
    // `slice_of` needs a borrow operand the clause subset already rejects.
    assert_rule(
        b"fn f(a: own u64) -> result: own u64 pure contract {\n  \
          define xs = array_new::<i32, 4>(0_i32);\n  \
          requires a < 8_u64;\n} {\n  \
          return a;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // The symbolic generic pass must intern the checked-arithmetic Result
    // before FN-8 applies the same copy-local rejection. Returning a compiler
    // failure here would make the generic surface traversal-order dependent.
    assert_rule(
        br#"fn invalid<T: Int>(x: own T) -> result: own T pure contract {
  define raised = x +checked 1_T;
  requires x > 0_T;
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // Non-copy by payload: `Result<i32, Overflow>` has a payload variant, and
    // `CheckedNominal::is_copy` holds only for all-fieldless-variant enums.
    assert_rule(
        b"fn f(x: own i32) -> result: own i32 pure contract {\n  \
          define raised = x +checked 1_i32;\n  \
          requires x > 0_i32;\n} {\n  \
          return x;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
    // The positive control: a copy-typed clause local is still admitted, or
    // the gate above has over-rejected into every clause `let`.
    with_semantics(
        b"fn f(a: own u64) -> result: own u64 pure contract {\n  \
          define ok = a < 8_u64;\n  \
          requires ok;\n} {\n  \
          return a;\n}\n\n\
          command fn main() -> status: own ExitStatus pure {\n  \
          return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("a Bool clause local is a copy value: {outcome:?}");
            };
            let requirement = checked.data.functions[0]
                .requirements
                .first()
                .expect("f carries its admitted requirement");
            assert!(
                !contains_array_fill(&requirement.template.root),
                "ArrayFill is a body-origin operation, never an admitted GoalTemplate row"
            );
        },
    );
}

#[test]
fn resolved_system_calls_in_requires_are_fn8_source_rejections() {
    assert_rule(
        br#"fn invalid() -> result: own ExitStatus pure contract {
  define status = exit_status(code: 0_u8);
  requires 0_u8 == 0_u8;
} {
  return exit_status(code: 0_u8);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn8,
        SemanticIssueKind::InvalidRequires,
    );
}

#[test]
fn requires_locals_are_distinct_from_same_named_body_locals() {
    let source = br#"fn increment(x: own i32) -> result: own i32 pure contract {
  define value = x >= 0_i32;
  requires value;
} {
  let value = x +wrap 1_i32;
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let x = 7_i32;
  let value = increment(x: x);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("requires and body scopes must remain disjoint: {outcome:?}");
        };
        let function = &checked.data.functions[0];
        assert!(!function.requirements.is_empty());
        assert!(matches!(function.body[0], CheckedStatement::Let { .. }));
    });
}

#[test]
fn goal_templates_ignore_clause_spelling_and_local_sharing() {
    let source = br#"fn shared(a: own u64, b: own u64) -> result: own u64 pure contract {
  define sum = a +wrap b;
  define below = sum < 100_u64;
  define above = sum > 0_u64;
  define bounded = band(below, above);
  requires bounded;
} {
  return a;
}

fn duplicated(left: own u64, right: own u64) -> result: own u64 pure contract {
  define first_sum = left +wrap right;
  define low_half = first_sum < 100_u64;
  define second_sum = left +wrap right;
  define high_half = second_sum > 0_u64;
  define complete = band(low_half, high_half);
  requires complete;
} {
  return left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("alpha-equivalent requirements must check: {outcome:?}");
        };
        let requirement = |name: &str| {
            checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .and_then(|function| function.requirements.first())
                .unwrap_or_else(|| panic!("missing requirement for {name}"))
        };
        let shared = requirement("shared");
        let duplicated = requirement("duplicated");
        assert_eq!(shared.template, duplicated.template);
        assert_ne!(shared.clause, duplicated.clause);
    });
}

#[test]
fn goal_templates_retain_order_row_and_named_const_identity() {
    let source = br#"const first_limit: u64 = 8_u64;

const second_limit: u64 = 8_u64;

fn baseline(value: own u64) -> result: own u64 pure contract {
  requires value < first_limit;
} {
  return value;
}

fn swapped(value: own u64) -> result: own u64 pure contract {
  requires first_limit < value;
} {
  return value;
}

fn different_row(value: own u64) -> result: own u64 pure contract {
  requires value <= first_limit;
} {
  return value;
}

fn different_const(value: own u64) -> result: own u64 pure contract {
  requires value < second_limit;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("identity controls must check before comparison: {outcome:?}");
        };
        let template = |name: &str| {
            &checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .and_then(|function| function.requirements.first())
                .unwrap_or_else(|| panic!("missing requirement for {name}"))
                .template
        };
        let baseline = template("baseline");
        assert_ne!(baseline, template("swapped"));
        assert_ne!(baseline, template("different_row"));
        assert_ne!(baseline, template("different_const"));
    });
}

#[test]
fn goal_box_deref_projection_retains_the_selected_referent_type() {
    let source = br#"fn positive(owner: own box<i32>) -> result: own box<i32> pure contract {
  requires deref(owner) > 0_i32;
} {
  return move owner;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an admitted box-referent goal must check: {outcome:?}");
        };
        let root = &checked.data.functions[0]
            .requirements
            .first()
            .expect("positive has a requirement")
            .template
            .root;
        let GoalExpression::Operation { arguments, .. } = root else {
            panic!("comparison must remain an operation goal");
        };
        let GoalExpression::Datum(GoalDatum::Parameter {
            projections, ty, ..
        }) = &arguments[0]
        else {
            panic!("box referent must remain the first formal datum");
        };
        assert_eq!(projections, &[GoalProjection::Deref]);
        assert_eq!(*ty, CheckedType::Integer(IntegerType::I32));
    });
}

#[test]
fn goal_field_projection_retains_the_selected_array_type() {
    let source = br#"struct Envelope {
  values: array<u8, 2>;
}

fn measured(envelope: own Envelope) -> result: own Envelope pure contract {
  define size = len(envelope.values);
  requires size == size;
} {
  return move envelope;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an admitted projected-array goal must check: {outcome:?}");
        };
        let GoalExpression::Operation { arguments, .. } = &checked.data.functions[0]
            .requirements
            .first()
            .expect("measured has a requirement")
            .template
            .root
        else {
            panic!("comparison must remain an operation goal");
        };
        let GoalExpression::Operation {
            arguments: length_arguments,
            ..
        } = &arguments[0]
        else {
            panic!("expanded local must retain its len operation");
        };
        let GoalExpression::Datum(GoalDatum::Parameter {
            projections, ty, ..
        }) = &length_arguments[0]
        else {
            panic!("projected array must remain the formal datum");
        };
        assert_eq!(projections, &[GoalProjection::Field(0)]);
        assert_eq!(
            *ty,
            CheckedType::Array {
                element: CheckedFlatElement::Integer(IntegerType::U8),
                length: CheckedConst::Value(2),
            }
        );
    });
}

#[test]
fn concrete_equal_const_arguments_produce_equal_goal_templates() {
    let source =
        br#"fn left<const n: u64>(value: own array<u8, n>) -> result: own array<u8, n> pure contract {
  define size = len(value);
  requires size == size;
} {
  return move value;
}

fn right<const count: u64>(input: own array<u8, count>) -> result: own array<u8, count> pure contract {
  define extent = len(input);
  requires extent == extent;
} {
  return move input;
}

fn different<const width: u64>(items: own array<u8, width>) -> result: own array<u8, width> pure contract {
  define amount = len(items);
  requires amount == amount;
} {
  return move items;
}

command fn main() -> status: own ExitStatus pure {
  let left_input = array_new::<u8, 2>(1_u8);
  let left_output = left::<2>(value: move left_input);
  let right_input = array_new::<u8, 2>(1_u8);
  let right_output = right::<2>(input: move right_input);
  let different_input = array_new::<u8, 3>(1_u8);
  let different_output = different::<3>(items: move different_input);
  let left_size = len(left_output);
  let right_size = len(right_output);
  let different_size = len(different_output);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("generic requirements must form concrete templates: {outcome:?}");
        };
        let template = |name: &str| {
            &checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .and_then(|function| function.requirements.first())
                .unwrap_or_else(|| panic!("missing concrete requirement for {name}"))
                .template
        };
        assert_eq!(template("left"), template("right"));
        assert_ne!(template("left"), template("different"));
    });
}

#[test]
fn unused_generic_requirement_is_retained_symbolically_without_a_concrete_function() {
    let source = br#"fn positive<T: Int>(value: own T) -> result: own T pure contract {
  requires value > 0_T;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("unused generic requirement must survive symbolic checking: {outcome:?}");
        };
        assert_eq!(checked.data.functions.len(), 1);
        assert_eq!(checked.data.functions[0].name, "main");
        assert_eq!(checked.data.generic_requirements.len(), 1);
        let symbolic = &checked.data.generic_requirements[0];
        let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operand_type: CheckedType::GenericInt(_),
                    ..
                },
            ..
        } = &symbolic.requirement.template.root
        else {
            panic!("retained generic requirement must remain symbolic");
        };
    });
}

#[test]
fn a_nominal_bearing_generic_requirement_survives_the_symbolic_checkpoint_as_metadata() {
    let source = br#"struct Pair<T: Int> {
  left: T;
  right: T;
}

fn need<T: Int>(length: own u64) -> result: own unit pure contract {
  requires buffer_fits::<Pair<T>>(length);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("symbolic nominal requirements must remain valid metadata: {outcome:?}");
        };
        assert_eq!(checked.data.generic_requirements.len(), 1);
        let requirement = &checked.data.generic_requirements[0].requirement;
        let GoalExpression::Operation {
            row: GoalOperation::BufferFits { element, .. },
            ..
        } = &requirement.template.root
        else {
            panic!("the retained requirement must preserve its exact buffer_fits goal");
        };
        let CheckedType::Nominal(pair) = element else {
            panic!("Pair<T> remains a symbolic nominal goal argument");
        };
        assert!(
            pair.0 as usize >= checked.data.executable_nominal_count,
            "metadata-only symbolic nominals must follow the executable prefix"
        );
        let CheckedNominalKind::Struct { fields } = &checked.data.nominals[pair.0 as usize].kind
        else {
            panic!("Pair<T> must retain its checked struct shape");
        };
        assert_eq!(fields.len(), 2);
        assert!(
            fields
                .iter()
                .all(|field| matches!(field.ty, CheckedType::GenericInt(_)))
        );
        lower_checked(*checked, OverlapLowering::Off)
            .expect("metadata-only symbolic nominals must not reach lowering");
    });
}

#[test]
fn a_derived_const_in_generic_requirement_has_checked_program_owned_structure() {
    let source = br#"fn need<const n: u64>(value: own array<u8, n + 1>) -> result: own array<u8, n + 1> pure contract {
  define size = len(value);
  requires size == size;
} {
  return move value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the symbolic const requirement must be retained: {outcome:?}");
        };
        assert_eq!(checked.data.generic_requirements.len(), 1);
        assert_eq!(checked.data.derived_consts.len(), 1);
        let derived = checked.data.derived_consts[0];
        assert!(matches!(derived.left, CheckedConst::Parameter(_)));
        assert_eq!(derived.right, CheckedConst::Value(1));
        let rendered = format!(
            "{:#?}",
            checked.data.generic_requirements[0]
                .requirement
                .template
                .root
        );
        assert!(
            rendered.contains("DerivedConstId") && rendered.contains("0,"),
            "the retained goal must name the checked-program-owned table entry: {rendered}"
        );
        lower_checked(*checked, OverlapLowering::Off)
            .expect("metadata-only derived consts must not enter executable lowering");
    });
}

#[test]
fn called_generic_keeps_concrete_instances_and_one_symbolic_requirement() {
    let source = br#"fn positive<T: Int>(value: own T) -> result: own T pure contract {
  requires value > 0_T;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let narrow = 1_i32;
  let narrow_result = positive::<i32>(value: narrow);
  let wide = 1_i64;
  let wide_result = positive::<i64>(value: wide);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("concrete generic calls and symbolic metadata must coexist: {outcome:?}");
        };
        let concrete = checked
            .data
            .functions
            .iter()
            .filter(|function| function.name == "positive")
            .collect::<Vec<_>>();
        assert_eq!(concrete.len(), 2);
        assert_eq!(checked.data.generic_requirements.len(), 1);
        assert_eq!(
            checked.data.generic_requirements[0].declaration,
            concrete[0].declaration
        );
        assert!(concrete.iter().all(|function| {
            function
                .requirements
                .first()
                .is_some_and(|requirement| requirement.template.root.ty() == CheckedType::Bool)
        }));
    });
}

#[test]
fn generic_to_generic_discovery_does_not_duplicate_symbolic_requirements() {
    let source = br#"fn inner<T: Int>(value: own T) -> result: own T pure contract {
  requires value > 0_T;
} {
  return value;
}

fn outer<T: Int>(value: own T) -> result: own T pure contract {
  requires value > 0_T;
} {
  return inner::<T>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("transitive symbolic validation must retain canonical entries: {outcome:?}");
        };
        assert_eq!(checked.data.functions.len(), 1);
        assert_eq!(checked.data.generic_requirements.len(), 2);
        assert_ne!(
            checked.data.generic_requirements[0].declaration,
            checked.data.generic_requirements[1].declaration
        );
    });
}

#[test]
fn forward_calls_retain_paths_and_exact_literal_place_and_named_const_images() {
    let source = br#"const requirement_limit: u64 = 8_u64;

const equal_value_other_const: u64 = 8_u64;

command fn main() -> status: own ExitStatus pure {
  let local = 3_u64;
  let from_place = below(value: local);
  let from_literal = below(value: 4_u64);
  let from_named = below(value: equal_value_other_const);
  let from_same_named = below(value: requirement_limit);
  return exit_status(code: 0_u8);
}

fn below(value: own u64) -> result: own u64 pure contract {
  requires value < requirement_limit;
} {
  return value;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("forward call goal metadata must check: {outcome:?}");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main function");
        let CheckedStatement::Let { binding: local, .. } = &main.body[0] else {
            panic!("main must bind the place actual");
        };
        let calls = main.body[1..=4]
            .iter()
            .map(|statement| match statement {
                CheckedStatement::Let {
                    value: call @ CheckedExpression::UserCall { .. },
                    ..
                } => call,
                other => panic!("expected retained user call, got {other:?}"),
            })
            .collect::<Vec<_>>();

        for call in &calls {
            let CheckedExpression::UserCall {
                call,
                argument_nodes,
                requirements,
                ..
            } = call
            else {
                unreachable!();
            };
            let [requirement] = requirements.as_slice() else {
                panic!("call must retain exactly one requirement");
            };
            assert_eq!(argument_nodes.len(), 1);
            assert_ne!(*call, argument_nodes[0]);
            assert_ne!(*call, requirement.requires_clause);
        }
        let call_paths = calls
            .iter()
            .map(|call| match call {
                CheckedExpression::UserCall { call, .. } => call,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_ne!(call_paths[0], call_paths[1]);
        assert_ne!(call_paths[1], call_paths[2]);

        assert!(matches!(
            &instantiated_call_goal_arguments(calls[0])[0],
            GoalExpression::Datum(GoalDatum::Place { root, projections, ty })
                if *root == *local && projections.is_empty()
                    && *ty == CheckedType::Integer(IntegerType::U64)
        ));
        assert!(matches!(
            &instantiated_call_goal_arguments(calls[1])[0],
            GoalExpression::Datum(GoalDatum::Literal(CheckedValue::Integer {
                ty: IntegerType::U64,
                bits: 4,
            }))
        ));
        let GoalExpression::Datum(GoalDatum::NamedConst {
            declaration: actual_const,
            ..
        }) = &instantiated_call_goal_arguments(calls[2])[0]
        else {
            panic!("named actual must retain declaration identity");
        };
        let GoalExpression::Datum(GoalDatum::NamedConst {
            declaration: requirement_const,
            ..
        }) = &instantiated_call_goal_arguments(calls[2])[1]
        else {
            panic!("requirement operand must retain declaration identity");
        };
        assert_ne!(actual_const, requirement_const);
        let CheckedExpression::UserCall {
            arguments,
            goal_arguments,
            ..
        } = calls[2]
        else {
            unreachable!();
        };
        assert!(matches!(
            &arguments[0],
            CheckedExpression::NamedConstant { declaration, .. }
                if declaration == actual_const
        ));
        assert_eq!(
            &goal_arguments[0],
            &instantiated_call_goal_arguments(calls[2])[0]
        );

        let same_arguments = instantiated_call_goal_arguments(calls[3]);
        assert_eq!(same_arguments[0], same_arguments[1]);
        assert!(matches!(
            calls[3],
            CheckedExpression::UserCall { arguments, .. }
                if matches!(
                    &arguments[0],
                    CheckedExpression::NamedConstant { declaration, .. }
                        if declaration == requirement_const
                )
        ));
        assert_eq!(main.entailment.call_goals.len(), 4);
        assert_eq!(
            main.entailment
                .call_goals
                .iter()
                .map(|outcome| outcome.disposition)
                .collect::<Vec<_>>(),
            vec![
                CallGoalDisposition::Discharged,
                CallGoalDisposition::Discharged,
                CallGoalDisposition::Refuted,
                CallGoalDisposition::Refuted,
            ]
        );
        assert_eq!(
            main.entailment.call_goals[0].evidence,
            vec![CallGoalEvidence::ExactL0Projection]
        );
        assert_eq!(
            main.entailment.call_goals[2].evidence,
            vec![CallGoalEvidence::NegatedL0Projection]
        );
    });
}

#[test]
fn call_front_end_captures_a_subscript_until_entailment_admits_its_identity() {
    let source = br#"fn positive(value: own u8) -> result: own unit pure contract {
  requires value < 10_u8;
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(3_u8);
  positive(value: values[0_u64]);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("a discharged direct-subscript actual must check: {outcome:?}");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main function");
        let CheckedStatement::Evaluate(call @ CheckedExpression::UserCall { .. }) = &main.body[1]
        else {
            panic!("main must retain the call expression");
        };
        let CheckedExpression::UserCall {
            call: call_path,
            argument_nodes,
            arguments,
            goal_arguments,
            requirements,
            ..
        } = call
        else {
            unreachable!();
        };
        let [requirement] = requirements.as_slice() else {
            panic!("call must retain exactly one requirement");
        };
        assert!(matches!(arguments[0], CheckedExpression::ArrayIndex { .. }));
        assert_eq!(argument_nodes.len(), 1);
        let GoalExpression::Datum(GoalDatum::EvaluatedValue {
            function: caller,
            occurrence:
                super::super::goal::EvaluatedValueOccurrence::CallArgument { call, argument },
            captured_type,
            projections,
            ty,
        }) = &goal_arguments[0]
        else {
            panic!("the front end must retain an occurrence-local pre-admission capture");
        };
        assert_eq!(*caller, main.id);
        assert_eq!(call, call_path);
        assert_eq!(*argument, 0);
        assert_eq!(*captured_type, CheckedType::Integer(IntegerType::U8));
        assert_eq!(*ty, *captured_type);
        assert!(projections.is_empty());
        let GoalExpression::Operation { arguments, .. } = &requirement.goal.root else {
            panic!("positive requirement must remain a comparison");
        };
        assert_eq!(&arguments[0], &goal_arguments[0]);
        assert_eq!(main.entailment.call_goals.len(), 1);
        assert_eq!(
            main.entailment.call_goals[0].disposition,
            CallGoalDisposition::Unproved
        );
        let GoalExpression::Operation { arguments, .. } = &main.entailment.call_goals[0].goal.root
        else {
            panic!("the entailment goal must retain its comparison root");
        };
        assert!(matches!(
            &arguments[0],
            GoalExpression::Operation {
                row: GoalOperation::ArrayIndex { .. },
                ..
            }
        ));
        assert!(main.entailment.call_goals[0].evidence.is_empty());
    });
}

#[test]
fn proved_subscript_actual_reuses_its_stable_goal_identity_at_fn8() {
    let source = br#"fn positive(value: own u8) -> result: own unit pure contract {
  requires value < 10_u8;
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(3_u8);
  if values[0_u64] < 10_u8 {
    positive(value: values[0_u64]);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the admitted indexed value must satisfy the matching requirement: {outcome:?}");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main function");
        let [call] = main.entailment.call_goals.as_slice() else {
            panic!("the true branch must retain one call requirement");
        };
        assert_eq!(call.disposition, CallGoalDisposition::Discharged);
        let GoalExpression::Operation { arguments, .. } = &call.goal.root else {
            panic!("the requirement must retain its comparison root");
        };
        assert!(matches!(
            &arguments[0],
            GoalExpression::Operation {
                row: GoalOperation::ArrayIndex { .. },
                ..
            }
        ));
    });
}

#[test]
fn borrow_substitution_removes_callee_deref_and_retains_caller_opaque_deref() {
    let source = br#"fn observe(value: &u64) -> result: own unit reads(value) contract {
  requires deref(value) > 0_u64;
} {
  let copied = deref(value);
  return unit;
}

fn proxy(value: &u64) -> result: own unit reads(value) {
  region {
    observe(value: &deref(value));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let local = 1_u64;
  region {
    observe(value: &local);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("borrow and reborrow call goals must check: {outcome:?}");
        };
        let proxy = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "proxy")
            .expect("proxy function");
        let CheckedStatement::Region { body, .. } = &proxy.body[0] else {
            panic!("proxy must retain child region");
        };
        let CheckedStatement::Evaluate(CheckedExpression::UserCall { requirements, .. }) = &body[0]
        else {
            panic!("proxy child must retain its call requirement");
        };
        let [requirement] = requirements.as_slice() else {
            panic!("proxy call must retain exactly one requirement");
        };
        let GoalExpression::Operation { arguments, .. } = &requirement.goal.root else {
            panic!("observe requirement must remain a comparison");
        };
        assert!(matches!(
            &arguments[0],
            GoalExpression::Datum(GoalDatum::Place { root, projections, .. })
                if *root == proxy.parameters[0].binding
                    && projections == &[GoalProjection::Deref]
        ));
        assert_eq!(proxy.entailment.call_goals.len(), 1);
        assert_eq!(
            proxy.entailment.call_goals[0].disposition,
            CallGoalDisposition::Unproved
        );

        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main function");
        let CheckedStatement::Let { binding: local, .. } = &main.body[0] else {
            panic!("main local binding");
        };
        let CheckedStatement::Region { body, .. } = &main.body[1] else {
            panic!("main direct region");
        };
        let CheckedStatement::Evaluate(CheckedExpression::UserCall { requirements, .. }) = &body[0]
        else {
            panic!("main direct call requirement");
        };
        let [requirement] = requirements.as_slice() else {
            panic!("main call must retain exactly one requirement");
        };
        let GoalExpression::Operation { arguments, .. } = &requirement.goal.root else {
            panic!("observe requirement must remain a comparison");
        };
        assert!(matches!(
            &arguments[0],
            GoalExpression::Datum(GoalDatum::Place { root, projections, .. })
                if *root == *local && projections.is_empty()
        ));
        assert_eq!(main.entailment.call_goals.len(), 1);
        assert_eq!(
            main.entailment.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            main.entailment.call_goals[0].evidence,
            vec![CallGoalEvidence::ExactL0Projection]
        );
    });
}

#[test]
fn call_goal_substitutes_type_const_and_slice_region_arguments() {
    let source = br#"const bytes: array<u8, 2> =[4_u8, 9_u8];

fn inspect(values: own slice<u8>) -> result: own unit pure contract {
  define size = len(values);
  requires size == size;
} {
  return unit;
}

fn guarded<T: Int, const n: u64>(value: own T, values: own array<u8, n>) -> result: own T pure contract {
  define positive = value > 0_T;
  define size = len(values);
  define exact = size == size;
  define complete = band(positive, exact);
  requires complete;
} {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  region {
    let view = slice_of(&bytes);
    inspect(values: move view);
  }
  let values = array_new::<u8, 3>(1_u8);
  let result = guarded::<i32, 3>(value: 4_i32, values: move values);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("concrete generic and region substitutions must check: {outcome:?}");
        };
        let inspect = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("inspect function");
        let CheckedType::Slice {
            region: inspect_formal_region,
            ..
        } = inspect.parameters[0].ty
        else {
            panic!("inspect parameter must be a slice");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main function");
        let CheckedStatement::Region { body, .. } = &main.body[0] else {
            panic!("main view region");
        };
        let CheckedStatement::Evaluate(CheckedExpression::UserCall {
            goal_arguments,
            requirements,
            ..
        }) = &body[1]
        else {
            panic!("inspect call metadata");
        };
        let [requirement] = requirements.as_slice() else {
            panic!("inspect call must retain exactly one requirement");
        };
        let GoalExpression::Datum(GoalDatum::Place {
            ty:
                CheckedType::Slice {
                    region: caller_region,
                    ..
                },
            ..
        }) = &goal_arguments[0]
        else {
            panic!("slice actual image");
        };
        assert_ne!(*caller_region, inspect_formal_region);
        let GoalExpression::Operation { arguments, .. } = &requirement.goal.root else {
            panic!("slice equality goal");
        };
        for length in arguments {
            let GoalExpression::Operation {
                row: GoalOperation::SliceMeasure { region, .. },
                arguments,
                ..
            } = length
            else {
                panic!("expanded slice length goal");
            };
            assert_eq!(*region, *caller_region);
            assert_eq!(&arguments[0], &goal_arguments[0]);
        }

        let CheckedStatement::Let {
            value:
                CheckedExpression::UserCall {
                    call,
                    argument_nodes,
                    goal_arguments,
                    requirements,
                    ..
                },
            ..
        } = &main.body[2]
        else {
            panic!("guarded call metadata");
        };
        let [requirement] = requirements.as_slice() else {
            panic!("guarded call must retain exactly one requirement");
        };
        assert_eq!(argument_nodes.len(), 2);
        assert!(argument_nodes[0].components() < argument_nodes[1].components());
        assert_ne!(*call, argument_nodes[0]);
        assert_ne!(*call, argument_nodes[1]);
        assert!(matches!(
            goal_arguments[1].ty(),
            CheckedType::Array {
                length: CheckedConst::Value(3),
                ..
            }
        ));
        let GoalExpression::Operation {
            row: GoalOperation::Boolean(_),
            arguments,
            ..
        } = &requirement.goal.root
        else {
            panic!("guarded requirement must remain the complete band goal");
        };
        let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation: CheckedIntegerOperation::Greater,
                    operand_type,
                },
            arguments: positive_arguments,
            ..
        } = &arguments[0]
        else {
            panic!("generic positive subgoal");
        };
        assert_eq!(*operand_type, CheckedType::Integer(IntegerType::I32));
        assert!(matches!(
            &positive_arguments[1],
            GoalExpression::Datum(GoalDatum::Literal(CheckedValue::Integer {
                ty: IntegerType::I32,
                bits: 0,
            }))
        ));
        let GoalExpression::Operation {
            arguments: size_arguments,
            ..
        } = &arguments[1]
        else {
            panic!("generic size subgoal");
        };
        for length in size_arguments {
            assert!(matches!(
                length,
                GoalExpression::Operation {
                    row: GoalOperation::ArrayMeasure {
                        length: CheckedConst::Value(3),
                        ..
                    },
                    ..
                }
            ));
        }
        assert_eq!(main.entailment.call_goals.len(), 2);
        assert_eq!(
            main.entailment.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            main.entailment.call_goals[0].evidence,
            vec![CallGoalEvidence::ExactL0Projection]
        );
        assert_eq!(
            main.entailment.call_goals[1].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            main.entailment.call_goals[1].evidence,
            vec![CallGoalEvidence::BooleanIntroductionPositive]
        );
    });
}

/// The OWN-1 bare-affine rejection inside a static requirement carries the
/// clause-specific repair because [FN-8] rejects `move` inside a contract.
#[test]
fn requires_clause_bare_affine_use_carries_the_static_repair() {
    let expected_fix =
        "restate the definition or clause over copy operands or non-consuming admitted reads";
    assert_rule(
        br#"enum Holder {
  Value(content: u64);
}

fn inspect(holder: own Holder) -> result: own unit pure contract {
  requires eeq(holder, holder);
} {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let holder = Value(content: 4_u64);
  let held = inspect(holder: move holder);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::BareAffineUse {
            mechanical_fix: expected_fix,
        },
    );
}
