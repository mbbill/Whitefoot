#![allow(clippy::panic)]

use crate::lexer::{LexLimits, LexOutcome, lex_v0_12};
use crate::{
    CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome, KERNEL_SPEC_V0_12_HASH,
    ParseLimits, ParseOutcome, ResolutionOutcome, SemanticIssueKind, SemanticLocation,
    SemanticOutcome, SemanticRuleV0_12, SourceBundle, SourceInput, SourceLimits, TerminalLimits,
    TerminalOutcome, UnsupportedSemanticFeatureV0_12, audit_canonical_v0_12, check_semantics_v0_12,
    classify_terminals_v0_12, finalize_v0_12, parse_v0_12, resolve_v0_12,
};

use super::model::{CheckedExpression, CheckedStatement};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 4,
    max_logical_path_bytes: 128,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_binding_bytes: 1_048_576,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 4,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_token_bytes: 16_384,
    max_tokens: 131_072,
    max_lexemes: 262_144,
};

const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 8_000_000,
    max_tasks: 131_072,
    max_frames: 8_192,
    max_elements: 262_144,
};

const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 8_000_000,
    max_roots: 131_072,
    max_shape_tasks: 131_072,
    max_nodes: 131_072,
    max_child_edges: 131_072,
    max_terminals: 131_072,
    max_sources: 4,
};

const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 8_000_000,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_gaps: 131_072,
    max_path_components: 8_192,
};

fn with_semantics<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex_v0_12(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_12(
        &lexed,
        KERNEL_SPEC_V0_12_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse_v0_12(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize_v0_12(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical_v0_12(finalized, CANONICAL_LIMITS)
    else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve_v0_12(canonical) else {
        panic!("semantic test source must resolve");
    };
    run(check_semantics_v0_12(resolved))
}

fn assert_rule(source: &[u8], rule: SemanticRuleV0_12, kind: SemanticIssueKind) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
    });
}

fn assert_unsupported(source: &[u8], feature: UnsupportedSemanticFeatureV0_12) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Unsupported { unsupported, .. } = outcome else {
            panic!("expected unsupported {feature:?}, got {outcome:?}");
        };
        assert_eq!(unsupported.feature(), feature);
    });
}

#[test]
fn scalar_constants_calls_operations_and_checks_publish_one_checked_program() {
    let source = br#"const base: i32 = 40_i32;

fn add(x: own i32, y: own i32) -> own i32 pure {
  return iadd.wrap<i32>(x, y);
}

fn main() -> own unit traps {
  let result: own i32 = add(x: base, y: 2_i32);
  check ieq<i32>(result, 42_i32) else trap "wrong answer";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("complete scalar family must check: {outcome:?}");
        };
        assert_eq!(checked.function_count(), 2);
        assert_eq!(checked.entry_function_name(), "main");
    });
}

#[test]
fn semantic_rule_owners_remain_distinct() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i8 = 128_i8;\n  return unit;\n}\n",
        SemanticRuleV0_12::Form7,
        SemanticIssueKind::InvalidIntegerLiteral,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  return 0_i32;\n}\n",
        SemanticRuleV0_12::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  check 1_i32 else trap \"bad\";\n  return unit;\n}\n",
        SemanticRuleV0_12::Op5,
        SemanticIssueKind::InvalidCheckCondition,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  check True() else trap \"bad\";\n  return unit;\n}\n",
        SemanticRuleV0_12::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  return unit;\n}\n",
        SemanticRuleV0_12::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn function_control_and_main_contract_are_checked_before_lowering() {
    assert_rule(
        b"fn main() -> own unit pure {\n}\n",
        SemanticRuleV0_12::Fn1,
        SemanticIssueKind::FunctionFallthrough,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  return unit;\n  return unit;\n}\n",
        SemanticRuleV0_12::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
    assert_rule(
        b"fn main(value: own i32) -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_12::Fn7,
        SemanticIssueKind::InvalidMain,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  loop @done {\n    break @done;\n    return unit;\n  }\n  return unit;\n}\n",
        SemanticRuleV0_12::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
}

#[test]
fn loops_enforce_own11_for_outer_affine_moves() {
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/own11-neg-move-outer-in-loop.wf"),
        SemanticRuleV0_12::Own11,
        SemanticIssueKind::MoveOuterBindingInLoop {
            mechanical_fix: "move the binding before the loop or declare and consume it inside the loop body",
        },
    );
    assert_unsupported(
        b"fn main() -> own unit pure {\n  loop @forever {\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeatureV0_12::StructuredControlFlow,
    );
}

#[test]
fn loop_break_and_backedge_cleanup_is_explicit() {
    let source = br#"struct Cell {
  value: i32;
}

fn main() -> own unit pure {
  loop @again {
    let first: own Cell = Cell(value: 1_i32);
    match True() {
      True() => {
        break @again;
      }
      False() => {
      }
    }
    let second: own Cell = Cell(value: 2_i32);
  }
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("loop cleanup source must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Loop {
            body,
            backedge_drops,
            ..
        } = &main.body[0]
        else {
            panic!("first statement must be the checked loop");
        };
        assert_eq!(backedge_drops.len(), 2);
        assert!(backedge_drops[0].binding.0 > backedge_drops[1].binding.0);
        let CheckedStatement::Match { arms, .. } = &body[1] else {
            panic!("second loop statement must be the match");
        };
        let CheckedStatement::Break { drops, .. } = &arms[0].body[0] else {
            panic!("True arm must contain the checked break");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].binding, backedge_drops[1].binding);
    });
}

#[test]
fn named_arguments_and_copy_move_spelling_are_checked_generally() {
    let wrong_name = br#"fn take(value: own i32) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  take(other: 1_i32);
  return unit;
}
"#;
    assert_rule(
        wrong_name,
        SemanticRuleV0_12::Gram11,
        SemanticIssueKind::InvalidNamedArguments {
            callee: "take".to_owned(),
            declared_parameters: vec!["value".to_owned()],
        },
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let a: own i32 = 1_i32;\n  let b: own i32 = move a;\n  return unit;\n}\n",
        SemanticRuleV0_12::Own1,
        SemanticIssueKind::MoveOfCopy {
            mechanical_fix: "use the copy place without `move`",
        },
    );
}

#[test]
fn operation_call_shapes_keep_their_exact_rule_owners() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i32 = iadd.wrap(1_i32, 2_i32);\n  return unit;\n}\n",
        SemanticRuleV0_12::Fn2,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value: own i32 = iadd.wrap<i32>(left: 1_i32, right: 2_i32);\n  return unit;\n}\n",
        SemanticRuleV0_12::Gram11,
        SemanticIssueKind::InvalidNamedArguments {
            callee: "iadd.wrap".to_owned(),
            declared_parameters: Vec::new(),
        },
    );
}

#[test]
fn effect_mismatch_is_located_at_the_written_effect_row() {
    let source =
        b"fn main() -> own unit pure {\n  check True() else trap \"bad\";\n  return unit;\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected EFF-2 mismatch, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRuleV0_12::Eff2);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("EFF-2 must use the source effects node");
        };
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits usize");
        assert_eq!(&source[start..end], b"pure");
    });
}

#[test]
fn invalid_generic_main_is_fn7_not_an_unsupported_generic() {
    assert_rule(
        b"fn main<T>() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_12::Fn7,
        SemanticIssueKind::InvalidMain,
    );
}

#[test]
fn unimplemented_contract_family_is_not_a_source_rejection() {
    let source = b"contract Empty {\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::Unsupported { unsupported, .. } = outcome else {
            panic!("contract semantics must be explicitly unsupported: {outcome:?}");
        };
        assert_eq!(
            unsupported.feature(),
            UnsupportedSemanticFeatureV0_12::ContractsAndConformances
        );
    });
}

#[test]
fn nominal_diagnostics_retain_required_lists_and_repairs() {
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-struct-neg-field-order.wf"),
        SemanticRuleV0_12::Gram8,
        SemanticIssueKind::InvalidConstructionFields {
            constructor: "Pair".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-match-gram10-out-of-order-fields.wf"),
        SemanticRuleV0_12::Gram10,
        SemanticIssueKind::InvalidMatchFields {
            variant: "Both".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/err2-neg-missing-variant.wf"),
        SemanticRuleV0_12::Err2,
        SemanticIssueKind::NonExhaustiveMatch {
            missing_variants: vec!["Blue".to_owned()],
        },
    );
    assert_rule(
        b"struct Pair {\n  x: i32;\n  x: i32;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRuleV0_12::Type6,
        SemanticIssueKind::DuplicateFieldLabel {
            label: "x".to_owned(),
        },
    );
    assert_rule(
        b"enum Pairing {\n  Both(a: i32, b: i32);\n}\n\nfn main() -> own unit pure {\n  let pair: own Pairing = Both(a: 1_i32, b: 2_i32);\n  match move pair {\n    Both(a: first) => {\n    }\n  }\n  return unit;\n}\n",
        SemanticRuleV0_12::Gram10,
        SemanticIssueKind::InvalidMatchFields {
            variant: "Both".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
}

#[test]
fn give_completeness_rejects_each_structural_failure() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let flag: own Bool = True();\n  let result: own i32 = match flag {\n    True() => {\n    }\n    False() => {\n      give 0_i32;\n    }\n  }\n  return unit;\n}\n",
        SemanticRuleV0_12::Give1,
        SemanticIssueKind::InvalidGive,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let flag: own Bool = True();\n  let result: own i32 = match flag {\n    True() => {\n      give 1_i32;\n      give 2_i32;\n    }\n    False() => {\n      give 0_i32;\n    }\n  }\n  return unit;\n}\n",
        SemanticRuleV0_12::Give1,
        SemanticIssueKind::InvalidGive,
    );
}

#[test]
fn enum_equality_exclusions_reach_the_intended_rule() {
    assert_rule(
        b"enum PayloadEq {\n  PayloadEmpty();\n  PayloadValue(value: u32);\n}\n\nfn main() -> own unit pure {\n  let left: own PayloadEq = PayloadEmpty();\n  let right: own PayloadEq = PayloadEmpty();\n  let equal: own Bool = eeq<PayloadEq>(move left, move right);\n  return unit;\n}\n",
        SemanticRuleV0_12::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"enum LeftEq {\n  LeftFirst();\n}\n\nenum RightEq {\n  RightFirst();\n}\n\nfn main() -> own unit pure {\n  let left: own LeftEq = LeftFirst();\n  let right: own RightEq = RightFirst();\n  let equal: own Bool = eeq<LeftEq>(left, right);\n  return unit;\n}\n",
        SemanticRuleV0_12::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn nominal_adjacent_unimplemented_behavior_stays_non_language_failure() {
    with_semantics(
        include_bytes!("../../../tests/conformance/cases/x-struct-set-field.wf"),
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    assert_unsupported(
        include_bytes!("../../../tests/conformance/cases/x-enum-borrow-payload-live.wf"),
        UnsupportedSemanticFeatureV0_12::RegionsAndBorrows,
    );
    assert_unsupported(
        b"struct Node {\n  next: Node;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        UnsupportedSemanticFeatureV0_12::RecursiveNominalLayout,
    );
    assert_unsupported(
        b"enum Flag {\n  A();\n  B();\n}\n\nfn main() -> own unit pure {\n  let flag: own Flag = A();\n  match flag {\n    A() => {\n    }\n    A() => {\n    }\n    B() => {\n    }\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeatureV0_12::DuplicateMatchArm,
    );
    assert_unsupported(
        b"struct Cell {\n  value: i32;\n}\n\nfn main() -> own unit pure {\n  let cell: own Cell = Cell(value: 1_i32);\n  let flag: own Bool = True();\n  match flag {\n    True() => {\n      let consumed: own Cell = move cell;\n    }\n    False() => {\n    }\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeatureV0_12::OwnershipJoin,
    );
}

#[test]
fn result_construction_and_propagation_keep_context_and_rule_owners() {
    let source = br#"enum StepError {
  Failed();
}

struct Pair {
  value: i32;
}

fn step(value: own i32) -> own Result<i32, StepError> pure {
  return Ok(value: value);
}

fn forward(value: own i32) -> own Result<Pair, StepError> pure {
  let accepted: own i32 = propagate step(value: value);
  let pair: own Pair = Pair(value: accepted);
  return Ok(value: move pair);
}

fn direct(error: own StepError) -> own Result<Pair, StepError> pure {
  let accepted: own i32 = propagate Err(error: error);
  let pair: own Pair = Pair(value: accepted);
  return Ok(value: move pair);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("Result family must check: {outcome:?}");
        };
        let forward = &checked.data.functions[1];
        let CheckedStatement::PropagateLet {
            ok_type, context, ..
        } = &forward.body[0]
        else {
            panic!("forward must retain its checked propagation edge");
        };
        assert_eq!(
            *ok_type,
            super::model::CheckedType::Integer(super::model::IntegerType::I32)
        );
        assert_eq!(context.function, "forward");
        assert!(!context.node_path.components().is_empty());
    });

    assert_rule(
        include_bytes!("../../../tests/conformance/cases/err3-neg-error-type-mismatch.wf"),
        SemanticRuleV0_12::Err3,
        SemanticIssueKind::InvalidPropagation,
    );
    assert_rule(
        br#"enum Flag {
  First();
  Second();
}

fn main() -> own unit pure {
  let flag: own Flag = First();
  match Err(error: flag) {
    Ok(value: ok_value) => {
    }
    Err(error: err_value) => {
    }
  }
  return unit;
}
"#,
        SemanticRuleV0_12::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-enum-result-payload-type-mismatch.wf"),
        SemanticRuleV0_12::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn set_retains_checked_copy_places_for_root_and_nested_field_updates() {
    let source = br#"struct Inner {
  value: i32;
}

struct Outer {
  inner: Inner;
  other: i32;
}

fn main() -> own unit pure {
  let number: own i32 = 1_i32;
  set number = 2_i32;
  let inner: own Inner = Inner(value: 3_i32);
  let outer: own Outer = Outer(inner: move inner, other: 4_i32);
  set outer.inner.value = number;
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("copy-place set must check: {outcome:?}");
        };
        let body = &checked.data.functions[0].body;
        let CheckedStatement::Set { target, .. } = &body[1] else {
            panic!("second statement must be the root set");
        };
        assert!(target.fields.is_empty());
        let CheckedStatement::Set { target, .. } = &body[4] else {
            panic!("fifth statement must be the nested-field set");
        };
        assert_eq!(target.fields, vec![0, 0]);
    });
}

#[test]
fn set_rejections_keep_their_exact_rule_owners() {
    assert_rule(
        b"const answer: i32 = 1_i32;\n\nfn main() -> own unit pure {\n  set answer = 2_i32;\n  return unit;\n}\n",
        SemanticRuleV0_12::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nfn main() -> own unit pure {\n  let left: own Cell = Cell(value: 1_i32);\n  let right: own Cell = Cell(value: 2_i32);\n  set left = move right;\n  return unit;\n}\n",
        SemanticRuleV0_12::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "Cell".to_owned(),
            mechanical_fix:
                "construct a fresh owner under a new let; do not replace an affine place",
        },
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let number: own i32 = 1_i32;\n  set number = True();\n  return unit;\n}\n",
        SemanticRuleV0_12::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn set_revalidates_the_target_after_rhs_ownership_changes() {
    let source = br#"struct Cell {
  value: i32;
}

fn take(cell: own Cell) -> own i32 pure {
  return cell.value;
}

fn main() -> own unit pure {
  let cell: own Cell = Cell(value: 1_i32);
  set cell.value = take(cell: move cell);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRuleV0_12::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}

#[test]
fn checked_cleanup_edges_cover_every_current_affine_exit() {
    let source = br#"struct Cell {
  value: i32;
}

struct Inner {
  selected: Cell;
  sibling: Cell;
}

struct Outer {
  inner: Inner;
  sibling: Cell;
}

enum Holder {
  Held(cell: Cell);
  Empty();
}

fn make() -> own Cell pure {
  let cell: own Cell = Cell(value: 1_i32);
  return move cell;
}

fn discard_call() -> own unit pure {
  make();
  return unit;
}

fn drop_binder(value: own Holder) -> own unit pure {
  match move value {
    Held(cell: item) => {
    }
    Empty() => {
    }
  }
  return unit;
}

fn drop_before_give(flag: own Bool) -> own i32 pure {
  let selected: own i32 = match flag {
    True() => {
      let temporary: own Cell = Cell(value: 2_i32);
      give 1_i32;
    }
    False() => {
      give 0_i32;
    }
  }
  return selected;
}

fn move_through_give(flag: own Bool) -> own Cell pure {
  let selected: own Cell = match flag {
    True() => {
      let temporary: own Cell = Cell(value: 3_i32);
      give move temporary;
    }
    False() => {
      let temporary: own Cell = Cell(value: 4_i32);
      give move temporary;
    }
  }
  return move selected;
}

fn reverse_order() -> own unit pure {
  let first: own Cell = Cell(value: 5_i32);
  let second: own Cell = Cell(value: 6_i32);
  return unit;
}

fn consume_projection() -> own unit pure {
  let selected: own Cell = Cell(value: 7_i32);
  let inner_sibling: own Cell = Cell(value: 8_i32);
  let inner: own Inner = Inner(selected: move selected, sibling: move inner_sibling);
  let outer_sibling: own Cell = Cell(value: 9_i32);
  let outer: own Outer = Outer(inner: move inner, sibling: move outer_sibling);
  let taken: own Cell = move outer.inner.selected;
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("cleanup fixture must check: {outcome:?}");
        };
        let function = |name: &str| {
            checked
                .data
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("missing checked function {name}"))
        };

        let make = function("make");
        let CheckedStatement::Return { drops, .. } = &make.body[1] else {
            panic!("make must end in return");
        };
        assert!(drops.is_empty(), "returned affine value must not also drop");

        let discard = function("discard_call");
        assert!(matches!(
            discard.body[0],
            CheckedStatement::DropExpression(_)
        ));

        let drop_binder = function("drop_binder");
        let CheckedStatement::Match { arms, .. } = &drop_binder.body[0] else {
            panic!("drop_binder must start with match");
        };
        assert_eq!(arms[0].fallthrough_drops.len(), 1);
        assert!(arms[1].fallthrough_drops.is_empty());

        let drop_before_give = function("drop_before_give");
        let CheckedStatement::ValueMatchLet { arms, .. } = &drop_before_give.body[0] else {
            panic!("drop_before_give must start with value match");
        };
        let CheckedStatement::Give { drops, .. } = &arms[0].body[1] else {
            panic!("first arm must end in give");
        };
        assert_eq!(drops.len(), 1);

        let move_through_give = function("move_through_give");
        let CheckedStatement::ValueMatchLet { arms, .. } = &move_through_give.body[0] else {
            panic!("move_through_give must start with value match");
        };
        for arm in arms {
            let CheckedStatement::Give { drops, .. } = &arm.body[1] else {
                panic!("each arm must end in give");
            };
            assert!(drops.is_empty(), "given affine value must not also drop");
        }

        let reverse = function("reverse_order");
        let CheckedStatement::Return { drops, .. } = &reverse.body[2] else {
            panic!("reverse_order must end in return");
        };
        assert_eq!(drops.len(), 2);
        assert!(drops[0].binding.0 > drops[1].binding.0);

        let projection = function("consume_projection");
        let CheckedStatement::Let {
            binding: taken,
            value:
                CheckedExpression::Project {
                    consume_root: true,
                    residual_drops,
                    ..
                },
        } = &projection.body[5]
        else {
            panic!("affine field move must consume its root");
        };
        assert_eq!(residual_drops.len(), 2);
        assert_eq!(residual_drops[0].fields, vec![1]);
        assert_eq!(residual_drops[1].fields, vec![0, 1]);
        let CheckedStatement::Return { drops, .. } = &projection.body[6] else {
            panic!("consume_projection must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].binding, *taken);
    });
}
