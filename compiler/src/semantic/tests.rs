#![allow(clippy::panic)]

mod arenas;
mod arithmetic_obligations;
mod arrays;
mod boolean_composition;
mod borrows;
mod boxes;
mod buffers;
mod check_dissolution;
mod checked_division;
mod conditionals;
mod const_eval;
mod contracts;
mod counted_ranges;
mod derivation;
mod division_obligations;
mod entailment;
mod entailment_sources;
mod entry_form;
mod float_conversion;
mod floating;
mod generics;
mod infix;
mod integer_absolute;
mod integer_conversion;
mod integer_extended;
mod integer_negation;
mod operation_table;
mod options;
mod postconditions;
mod provenance;
mod reinterpret;
mod replace;
mod requires;
mod slices;
mod strict;
mod system_effects;

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_TEXT, CanonicalLimits, CanonicalOutcome,
    FinalizeLimits, FinalizeOutcome, ParseLimits, ParseOutcome, ResolutionOutcome,
    SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule, SourceBundle, SourceInput,
    SourceLimits, TerminalLimits, TerminalOutcome, UnsupportedSemanticFeature, audit_canonical,
    check_semantics, classify_terminals, finalize, parse, resolve,
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
    with_semantics_inputs(&inputs, run)
}

fn with_semantics_inputs<ResultValue>(
    inputs: &[SourceInput<'_>],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_semantics_inputs_for(inputs, crate::Inventory::ACTIVE, run)
}

/// [`with_semantics_inputs`] against one named [SYS-2] inventory state.
///
/// A frozen real source that uses a candidate operation names the inventory
/// that declares it; every other caller takes the active one.
fn with_semantics_inputs_for<ResultValue>(
    inputs: &[SourceInput<'_>],
    inventory: crate::Inventory,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let Ok(bundle) = SourceBundle::with_limits(inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = crate::resolve_with_inventory(canonical, inventory)
    else {
        panic!("semantic test source must resolve");
    };
    run(check_semantics(resolved))
}

/// [`with_semantics`] through the test-only dark checker, which retains
/// every [ENT-6] obligation and [CLM-2] claim disposition instead of
/// rejecting at the first one, so engine unit tests can observe complete
/// per-function derivations.
fn with_semantics_dark<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("semantic test source must resolve");
    };
    run(super::check::check_semantics_dark(resolved))
}

/// [`with_semantics`] through the test-only entry that forces the
/// arithmetic-mode dissolution switch on [OP-2, ENT-6]. The shipped switch is
/// on too, so this entry selects the same judgment as the default one and
/// records which judgment its callers mean.
fn with_semantics_arithmetic<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("semantic test source must resolve");
    };
    run(super::check::check_semantics_arithmetic_obligations(
        resolved,
    ))
}

/// [`with_semantics`] through the test-only entry that forces the division
/// dissolution switch on [OP-2, ENT-6]. The shipped switch is on too, so this
/// entry selects the same judgment as the default one and records which
/// judgment its callers mean.
fn with_semantics_division<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("semantic test source must resolve");
    };
    run(super::check::check_semantics_division_obligations(resolved))
}

/// [`with_semantics`] through the test-only extension checker, which admits
/// the reborrow extension [OWN-6, OWN-14]. The shipped switch admits it too,
/// so this entry selects the same judgment as the default one and records
/// which judgment its callers mean.
fn with_semantics_extension<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_semantics_entry(
        source,
        super::check::check_semantics_reborrow_extension,
        run,
    )
}

/// One single-source frontend pass delivered to the named checker entry.
/// The pipeline values borrow one another down the stack, so the entry is
/// selected by parameter rather than by returning the resolved unit.
fn with_semantics_entry<ResultValue>(
    source: &[u8],
    check: for<'classified, 'lexed, 'source> fn(
        crate::ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    )
        -> SemanticOutcome<'classified, 'lexed, 'source>,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        SemanticOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("semantic test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("semantic test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("semantic test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("semantic test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("semantic test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("semantic test source must be canonical");
    };
    let ResolutionOutcome::Complete(resolved) = resolve(canonical) else {
        panic!("semantic test source must resolve");
    };
    run(check(resolved))
}

fn assert_rule(source: &[u8], rule: SemanticRule, kind: SemanticIssueKind) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
    });
}

/// [`assert_rule`] under the reborrow extension [OWN-6, OWN-14].
fn assert_rule_extension(source: &[u8], rule: SemanticRule, kind: SemanticIssueKind) {
    with_semantics_extension(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
    });
}

/// Asserts a rejection and the exact source bytes it cites, for the rules
/// that name *which* operand or node they land on.
fn assert_rule_at(source: &[u8], rule: SemanticRule, cited: &str) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?} at {cited:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!(
                "expected a source-node citation, got {:?}",
                issue.location()
            );
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        let actual = std::str::from_utf8(&source[start..end]).expect("cited bytes must be text");
        assert_eq!(actual, cited, "citation landed on the wrong node");
    });
}

fn assert_unsupported(source: &[u8], feature: UnsupportedSemanticFeature) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Unsupported { unsupported, .. } = outcome else {
            panic!("expected unsupported {feature:?}, got {outcome:?}");
        };
        assert_eq!(unsupported.feature(), feature);
    });
}

#[test]
fn a_claim_statement_is_an_accepted_named_runtime_check() {
    // CLM-1: a conforming claim is accepted and always retained. A
    // constructed `True()` predicate has no comparison origin, so it is
    // neither redundant nor refutable [CLM-2].
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  claim held: flag because "constructed true";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("expected acceptance, got {outcome:?}");
        };
        assert!(program.data.claim_advisories.is_empty());
    });
}

#[test]
fn a_repeated_claim_name_is_a_clm1_rejection_at_the_later_claim() {
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  claim held: flag because "first";
  claim held: flag because "second";
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Clm1,
        SemanticIssueKind::DuplicateClaimName {
            name: "held".to_owned(),
        },
    );
}

#[test]
fn a_non_bool_claim_condition_is_a_clm1_rejection() {
    let source = br#"fn main() -> own unit traps {
  let value = 3_u64;
  claim held: value because "not a Bool";
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Clm1,
        SemanticIssueKind::InvalidCheckCondition,
    );
}

#[test]
fn scalar_constants_calls_operations_and_checks_publish_one_checked_program() {
    let source = br#"const base: i32 = 40_i32;

fn add(x: own i32, y: own i32) -> own i32 pure {
  return x +wrap y;
}

fn main() -> own unit traps {
  let result = add(x: base, y: 2_i32);
  claim wrong_answer: ieq(result, 42_i32) because "wrong answer";
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
        b"fn main() -> own unit pure {\n  let value = 128_i8;\n  return unit;\n}\n",
        SemanticRule::Form7,
        SemanticIssueKind::InvalidIntegerLiteral,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  return 0_i32;\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  claim bad: 1_i32 because \"bad\";\n  return unit;\n}\n",
        SemanticRule::Clm1,
        SemanticIssueKind::InvalidCheckCondition,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  claim bad: True() because \"bad\";\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit traps {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn function_control_and_main_contract_are_checked_before_lowering() {
    assert_rule(
        b"fn main() -> own unit pure {\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::FunctionFallthrough,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  return unit;\n  return unit;\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
    assert_rule(
        b"fn main(value: own i32) -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  loop @done {\n    break @done;\n    return unit;\n  }\n  return unit;\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
}

#[test]
fn loops_enforce_own11_for_outer_affine_moves() {
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/own11-neg-move-outer-in-loop.wf"),
        SemanticRule::Own11,
        SemanticIssueKind::MoveOuterBindingInLoop {
            mechanical_fix: "move the binding before the loop or declare and consume it inside the loop body",
        },
    );
    assert_unsupported(
        b"fn main() -> own unit pure {\n  loop @forever {\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeature::StructuredControlFlow,
    );
}

#[test]
fn loop_break_and_backedge_cleanup_is_explicit() {
    let source = br#"struct Cell {
  value: i32;
}

fn main() -> own unit pure {
  loop @again {
    let first = Cell(value: 1_i32);
    if True() {
      break @again;
    }
    let second = Cell(value: 2_i32);
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
        SemanticRule::Gram11,
        SemanticIssueKind::InvalidNamedArguments {
            callee: "take".to_owned(),
            declared_parameters: vec!["value".to_owned()],
        },
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let a = 1_i32;\n  let b = move a;\n  return unit;\n}\n",
        SemanticRule::Own1,
        SemanticIssueKind::MoveOfCopy {
            mechanical_fix: "use the copy place without `move`",
        },
    );
}

/// Both shapes are carried on rows that keep their callee name. [OP-7]'s
/// one-spelling rule moved the twenty respelled rows out of the callee-name
/// inventory, so `iadd.wrap` — which carried both shapes in v0.22 — is no
/// longer a name at all and reaches OP-1 at resolution before either shape can
/// be judged. The concerns survive on the rows that still have names: a call
/// missing the arguments its row mandates, and one written with named
/// arguments.
#[test]
fn operation_call_shapes_keep_their_exact_rule_owners() {
    // A retained-argument row with its mandatory arguments absent. This was
    // recorded as FN-2 and flagged then as a witness that would move if the
    // citation question were settled the other way; it was, so it did.
    // [DIAG-1] gives a table operation the rule [OP-2] selects and never FN-2,
    // and [TYPE-5] is what mandates these arguments, so their absence is its
    // violation — the reading `finf`/`fnan` already carried.
    assert_rule(
        b"fn main() -> own unit pure {\n  let value = 4_i32;\n  let narrowed = cvt(value);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let left = 1_i32;\n  let right = 2_i32;\n  let value = imin(left: left, right: right);\n  return unit;\n}\n",
        SemanticRule::Gram11,
        SemanticIssueKind::InvalidNamedArguments {
            callee: "imin".to_owned(),
            declared_parameters: Vec::new(),
        },
    );
}

/// [DIAG-1] "The cited rule is the rule selected by the callee's class": FN-2
/// for a user-generic call, and for a table operation the rule [OP-2] selects.
/// The compiler used to choose from the *kind* of argument problem instead, so
/// it was wrong in both directions at once — a table operation missing its
/// mandatory arguments cited FN-2, and a user-generic call missing or
/// miscounting its arguments cited TYPE-5.
///
/// One argument problem is held fixed across the two classes so that only the
/// callee varies: each pair below is the same failure, so the rule must follow
/// the callee and nothing else.
#[test]
fn the_cited_rule_follows_the_callee_class_and_not_the_argument_problem() {
    // Missing the arguments the callee's class mandates.
    assert_rule(
        b"struct Held {\n  v: i32;\n}\n\nfn pick<T>(value: own T) -> own T pure {\n  return move value;\n}\n\nfn main() -> own unit pure {\n  let a = Held(v: 1_i32);\n  let b = pick(value: move a);\n  return unit;\n}\n",
        SemanticRule::Fn2,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value = 4_i32;\n  let narrowed = cvt(value);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );

    // A wrong-count argument list, the same failure on both classes.
    assert_rule(
        b"struct Held {\n  v: i32;\n}\n\nfn pick<T>(value: own T) -> own T pure {\n  return move value;\n}\n\nfn main() -> own unit pure {\n  let a = Held(v: 1_i32);\n  let b = pick<Held, Held>(value: move a);\n  return unit;\n}\n",
        SemanticRule::Fn2,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let value = 4_i32;\n  let narrowed = cvt<i32>(value);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );

    // A generic nominal's construct is a third class: [TYPE-5] owns its
    // written arguments, and it shares the argument-list reader with the
    // user-generic call, so it is the control that the rule is not simply
    // keyed on that reader.
    assert_rule(
        b"struct Pair<T> {\n  v: T;\n}\n\nfn main() -> own unit pure {\n  let p = Pair(v: 1_i32);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn effect_mismatch_is_located_at_the_written_effect_row() {
    let source =
        b"fn main() -> own unit pure {\n  claim bad: True() because \"bad\";\n  return unit;\n}\n";
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected EFF-2 mismatch, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
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
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
    );
}

#[test]
fn nominal_diagnostics_retain_required_lists_and_repairs() {
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-struct-neg-field-order.wf"),
        SemanticRule::Gram8,
        SemanticIssueKind::InvalidConstructionFields {
            constructor: "Pair".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-match-gram10-out-of-order-fields.wf"),
        SemanticRule::Gram10,
        SemanticIssueKind::InvalidMatchFields {
            variant: "Both".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/err2-neg-missing-variant.wf"),
        SemanticRule::Err2,
        SemanticIssueKind::NonExhaustiveMatch {
            missing_variants: vec!["Blue".to_owned()],
        },
    );
    assert_rule(
        b"struct Pair {\n  x: i32;\n  x: i32;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Type6,
        SemanticIssueKind::DuplicateFieldLabel {
            label: "x".to_owned(),
        },
    );
    assert_rule(
        b"enum Pairing {\n  Both(a: i32, b: i32);\n}\n\nfn main() -> own unit pure {\n  let pair = Both(a: 1_i32, b: 2_i32);\n  match move pair {\n    Both(a: first) => {\n    }\n  }\n  return unit;\n}\n",
        SemanticRule::Gram10,
        SemanticIssueKind::InvalidMatchFields {
            variant: "Both".to_owned(),
            declared_fields: vec!["a".to_owned(), "b".to_owned()],
        },
    );
}

#[test]
fn give_completeness_rejects_each_structural_failure() {
    assert_rule(
        b"fn main() -> own unit pure {\n  let flag = True();\n  let result = if flag {\n  } else {\n    give 0_i32;\n  }\n  return unit;\n}\n",
        SemanticRule::Give1,
        SemanticIssueKind::InvalidGive,
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let flag = True();\n  let result = if flag {\n    give 1_i32;\n    give 2_i32;\n  } else {\n    give 0_i32;\n  }\n  return unit;\n}\n",
        SemanticRule::Give1,
        SemanticIssueKind::InvalidGive,
    );
}

#[test]
fn enum_equality_exclusions_reach_the_intended_rule() {
    assert_rule(
        b"enum PayloadEq {\n  PayloadEmpty();\n  PayloadValue(value: u32);\n}\n\nfn main() -> own unit pure {\n  let left = PayloadEmpty();\n  let right = PayloadEmpty();\n  let equal = eeq(move left, move right);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"enum LeftEq {\n  LeftFirst();\n}\n\nenum RightEq {\n  RightFirst();\n}\n\nfn main() -> own unit pure {\n  let left = LeftFirst();\n  let right = RightFirst();\n  let equal = eeq(left, right);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn nominal_adjacent_unimplemented_behavior_stays_non_language_failure() {
    // The TYPE-5 set-field control is written inline rather than read from
    // `x-struct-set-field.wf`: that case's `c.n + 1_i32` is a v0.31
    // constant-operand-class site whose overflow obligation nothing
    // discharges, so the case now rejects on OP-2 with residual
    // `c.n <= 2147483646` and needs the owner-approved corpus migration the
    // arith delta assigns to the activation packet. The capability this
    // control exists to demonstrate — set a struct field, read it back — is
    // unaffected.
    with_semantics(
        b"struct Counter {\n  n: i32;\n}\n\nfn main() -> own unit traps {\n  let c = Counter(n: 1_i32);\n  set c.n = 41_i32;\n  let v = c.n;\n  claim set_field_drift: ieq(v, 41_i32) because \"set field drift\";\n  return unit;\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    // Borrow-mode parameters and `let` borrows of scalars and enums, and the
    // [OWN-13] borrowed match this exercises, are on the normal path now.
    // Also written inline rather than read from
    // `x-enum-borrow-payload-live.wf` for the same reason: that case's
    // `deref(x) + 1_i32` is an undischarged v0.31 class site (residual
    // `deref(x) <= 2147483646`), so it too awaits the owner-approved corpus
    // migration. The shape kept here is the one under test — a payload enum
    // borrow-matched through `&'r` whose scrutinee stays live for a second
    // read, with each derived binder explicitly dereferenced.
    with_semantics(
        b"enum Cell {\n  Full(v: i32);\n  Void();\n}\n\nfn main() -> own unit traps {\n  let c = Full(v: 20_i32);\n  region 'r {\n    let p = &'r c;\n    let a = match deref(p) {\n      Full(v: x) => {\n        give deref(x);\n      }\n      Void() => {\n        give 0_i32;\n      }\n    }\n    let q = &'r c;\n    let b = match deref(q) {\n      Full(v: y) => {\n        give deref(y);\n      }\n      Void() => {\n        give 0_i32;\n      }\n    }\n    claim borrow_payload_drift: ieq(a, 20_i32) because \"borrow payload drift\";\n    claim second_read_drift: ieq(b, 20_i32) because \"second read drift\";\n  }\n  return unit;\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    assert_unsupported(
        b"struct Node {\n  next: Node;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        UnsupportedSemanticFeature::RecursiveNominalLayout,
    );
    assert_unsupported(
        b"enum Flag {\n  A();\n  B();\n}\n\nfn main() -> own unit pure {\n  let flag = A();\n  match flag {\n    A() => {\n    }\n    A() => {\n    }\n    B() => {\n    }\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeature::DuplicateMatchArm,
    );
    assert_unsupported(
        b"struct Cell {\n  value: i32;\n}\n\nfn main() -> own unit pure {\n  let cell = Cell(value: 1_i32);\n  let flag = True();\n  if flag {\n    let consumed = move cell;\n  }\n  return unit;\n}\n",
        UnsupportedSemanticFeature::OwnershipJoin,
    );
}

#[test]
fn undeclared_system_effect_categories_reject_both_row_directions() {
    // The two payload-free categories are checked exactly like every other
    // category [EFF-1, EFF-2]: a non-kind-declaring unit can never exhibit
    // them, so declaring either is declared-but-unexhibited.
    assert_rule(
        b"fn probe() -> own unit external {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn probe() -> own unit blocks {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn checked_system_programs_complete_semantic_checking() {
    // The system semantic family — [SYS-2] call typing, [EFF-2] effect
    // attribution, and the release contribution — is implemented, so a
    // conforming kind-declaring unit completes semantic checking; the
    // remaining system boundary is lowering's explicit unsupported stop.
    with_semantics(
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "a conforming command entry must check: {outcome:?}"
            );
        },
    );
}

#[test]
fn propagate_of_a_box_holder_is_a_type7_missing_dereference() {
    // ERR-3: a borrow or box holder used without `deref` retains its TYPE-7
    // judgment; the propagate path previously fell through to ERR-3
    // invalid-propagation for a box<Result<..>> operand (task 0019, bucket 4).
    assert_rule(
        br#"enum StepError {
  Failed();
}

fn unwrap(holder: own box<Result<i32, StepError>>) -> own Result<i32, StepError> pure {
  let accepted = propagate holder;
  return Ok<i32, StepError>(value: accepted);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
}

#[test]
fn match_and_index_of_a_box_holder_are_type7_missing_dereferences() {
    // TYPE-7 owns the implicit-read case exclusively at every position that
    // states the exclusivity, so a box holder written where its referent enum
    // or its referent indexable would be required cites TYPE-7 and the
    // position's own wrong-type judgment forms no rejection.
    assert_rule(
        br#"enum State {
  Ready();
}

fn inspect(holder: own box<State>) -> own unit pure {
  match holder {
    Ready() => {
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    assert_rule(
        br#"fn read(holder: own box<buffer<u8>>) -> own u8 traps {
  return holder[0_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
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
  return Ok<i32, StepError>(value: value);
}

fn forward(value: own i32) -> own Result<Pair, StepError> pure {
  let accepted = propagate step(value: value);
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
}

fn direct(error: own StepError) -> own Result<Pair, StepError> pure {
  let accepted = propagate Err<i32, StepError>(error: error);
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
}

fn bare(result: own Result<i32, StepError>) -> own Result<Pair, StepError> pure {
  let accepted = propagate result;
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
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
        br#"enum StepError {
  Failed();
}

fn reuse(result: own Result<i32, StepError>) -> own Result<i32, StepError> pure {
  let accepted = propagate result;
  match result {
    Ok(value: second_value) => {
    }
    Err(error: second_error) => {
    }
  }
  return Ok<i32, StepError>(value: accepted);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );

    assert_rule(
        include_bytes!("../../../tests/conformance/cases/err3-neg-error-type-mismatch.wf"),
        SemanticRule::Err3,
        SemanticIssueKind::InvalidPropagation,
    );
    assert_rule(
        br#"enum Flag {
  First();
  Second();
}

fn main() -> own unit pure {
  let flag = First();
  match Err(error: flag) {
    Ok(value: ok_value) => {
    }
    Err(error: err_value) => {
    }
  }
  return unit;
}
"#,
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
    assert_rule(
        include_bytes!("../../../tests/conformance/cases/x-enum-result-payload-type-mismatch.wf"),
        SemanticRule::Type5,
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
  let number = 1_i32;
  set number = 2_i32;
  let inner = Inner(value: 3_i32);
  let outer = Outer(inner: move inner, other: 4_i32);
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
        let super::model::CheckedSetTarget::Place(target) = target else {
            panic!("root set must retain an ordinary writable place");
        };
        assert!(target.fields.is_empty());
        let CheckedStatement::Set { target, .. } = &body[4] else {
            panic!("fifth statement must be the nested-field set");
        };
        let super::model::CheckedSetTarget::Place(target) = target else {
            panic!("nested set must retain an ordinary writable place");
        };
        assert_eq!(target.fields, vec![0, 0]);
    });
}

#[test]
fn set_rejections_keep_their_exact_rule_owners() {
    assert_rule(
        b"const answer: i32 = 1_i32;\n\nfn main() -> own unit pure {\n  set answer = 2_i32;\n  return unit;\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\nfn main() -> own unit pure {\n  let left = Cell(value: 1_i32);\n  let right = Cell(value: 2_i32);\n  set left = move right;\n  return unit;\n}\n",
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "Cell".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    assert_rule(
        b"fn main() -> own unit pure {\n  let number = 1_i32;\n  set number = True();\n  return unit;\n}\n",
        SemanticRule::Type5,
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
  let cell = Cell(value: 1_i32);
  set cell.value = take(cell: move cell);
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Own1,
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
  let cell = Cell(value: 1_i32);
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
  let selected = if flag {
    let temporary = Cell(value: 2_i32);
    give 1_i32;
  } else {
    give 0_i32;
  }
  return selected;
}

fn move_through_give(flag: own Bool) -> own Cell pure {
  let selected = if flag {
    let temporary = Cell(value: 3_i32);
    give move temporary;
  } else {
    let temporary = Cell(value: 4_i32);
    give move temporary;
  }
  return move selected;
}

fn reverse_order() -> own unit pure {
  let first = Cell(value: 5_i32);
  let second = Cell(value: 6_i32);
  return unit;
}

fn consume_projection() -> own unit pure {
  let selected = Cell(value: 7_i32);
  let inner_sibling = Cell(value: 8_i32);
  let inner = Inner(selected: move selected, sibling: move inner_sibling);
  let outer_sibling = Cell(value: 9_i32);
  let outer = Outer(inner: move inner, sibling: move outer_sibling);
  let taken = move outer.inner.selected;
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
            CheckedStatement::DropExpression { .. }
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
            ..
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

/// [DIAG-1] the same-node citation rank is the rules' definition order in
/// the active specification; `SemanticRule::definition_rank` must agree with
/// the specification bytes for every citable rule.
///
/// The set under check is **walked from the enum**, not listed here. Both
/// `next_in_definition_order` and `definition_rank` are exhaustive matches, so
/// a new variant does not compile until it appears in each; this test then
/// makes the two check each other, since walking the chain must yield the
/// ranks 0, 1, 2, … in order. A hand-maintained list stood here until
/// 2026-08-08 and silently omitted `Gram6`, reporting every rule verified
/// while one was not.
#[test]
fn definition_rank_matches_the_active_specification() {
    let mut all = Vec::new();
    let mut rule = Some(SemanticRule::FIRST);
    while let Some(current) = rule {
        assert!(
            !all.contains(&current),
            "the definition-order chain revisits {}",
            current.id()
        );
        all.push(current);
        rule = current.next_in_definition_order();
    }

    // The chain and the rank table are separate exhaustive matches; this is
    // where they are made to agree, so neither can drift alone.
    for (position, rule) in all.iter().enumerate() {
        assert_eq!(
            rule.definition_rank(),
            position,
            "{} sits at chain position {position} but ranks {}",
            rule.id(),
            rule.definition_rank()
        );
    }

    let definition_line = |rule: SemanticRule| {
        let prefix = format!("[{}]", rule.id());
        ACTIVE_KERNEL_SPEC_TEXT
            .lines()
            .position(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("no definition line for {}", rule.id()))
    };
    let mut by_specification = all.clone();
    by_specification.sort_by_key(|rule| definition_line(*rule));
    for (walked, specified) in all.iter().zip(&by_specification) {
        assert_eq!(
            walked.id(),
            specified.id(),
            "definition_rank disagrees with the active specification order"
        );
    }
}
