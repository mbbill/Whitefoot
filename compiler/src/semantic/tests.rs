#![allow(clippy::panic)]

mod arenas;
mod arithmetic_obligations;
mod arrays;
mod boolean_composition;
mod borrows;
mod boxes;
mod buffers;
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
mod loop_invariants;
mod loop_permission;
mod operation_table;
mod options;
mod originating_acceptance;
mod permission;
mod postconditions;
mod reinterpret;
mod replace;
mod requires;
mod slices;
mod source_proofs;
mod staged_permission;
mod staged_permission_corpus;
mod system_effects;
mod target_action;

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

// These are the harness's own resource ceilings, well below the driver's
// [`crate::driver`], and they bound how large a semantic-test source may be
// rather than what the language admits. [GRAM-4]'s `affine_factor` now reaches
// its IDENT through an `atom` and a `place` [MSR-5], which is three derivation
// elements per affine atom where it was one, and the certificate-capacity
// fixture writes four thousand `use` steps over two atoms each. The parse and
// finalize ceilings rise so that fixture still reaches the checker; no
// judgment and no language rule changes with them.
const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 32_000_000,
    max_tasks: 524_288,
    max_frames: 8_192,
    max_elements: 524_288,
};

const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 32_000_000,
    max_roots: 262_144,
    max_shape_tasks: 262_144,
    max_nodes: 262_144,
    max_child_edges: 262_144,
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

/// Runs the ordinary front end through canonicalization and exposes the
/// resolver outcome directly. Contract definitions now precede every
/// postcondition in one block, so lookup failures in those definitions are
/// resolver-owned rather than delayed semantic entry failures.
fn with_resolution<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        ResolutionOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let inputs = [SourceInput::new("test.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("resolution test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("resolution test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("resolution test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("resolution test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("resolution test derivation must finalize");
    };
    let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, CANONICAL_LIMITS) else {
        panic!("resolution test source must be canonical");
    };
    run(resolve(canonical))
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
/// Asserts that one source is refused at the parse stage citing one rule.
///
/// A grammar the tables cannot derive is refused before the checker sees it,
/// and [DIAG-1] cites the production's own rule there. This is the assertion
/// for a source whose defect the grammar itself decides.
fn assert_parse_rule(source: &[u8], rule: crate::SyntaxRule) {
    let Ok(bundle) = SourceBundle::with_limits(&[SourceInput::new("parse.wf", source)], SOURCE_LIMITS) else {
        panic!("parse test bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("parse test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("parse test source must classify");
    };
    let outcome = parse(&classified, PARSE_LIMITS);
    let ParseOutcome::SourceIssue(issue) = outcome else {
        panic!("parse test source must be refused at the parse stage");
    };
    assert_eq!(issue.rule(), rule);
}

/// A frozen real source may name the inventory that first declared an
/// operation; every other caller takes the active one.
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
    let outcome = crate::resolve_with_inventory(canonical, inventory);
    let ResolutionOutcome::Complete(resolved) = outcome else {
        panic!("semantic test source must resolve: {outcome:?}");
    };
    run(check_semantics(resolved))
}

/// [`with_semantics`] through the test-only dark checker, which retains every
/// [ENT-6] obligation instead of rejecting at the first one, so engine unit
/// tests can observe complete per-function derivations.
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

/// Asserts a rejection's rule and which issue kind it cited, without pinning a
/// payload the call site does not state.
///
/// These call sites predate the payloads batch 0100 gave `TypeMismatch`,
/// `EffectMismatch`, `InvalidEffectRow`, and `InvalidBorrowLifetime`, and each
/// asserts here exactly what it asserted when those kinds were unit variants:
/// which rule rejected, and which kind it cited. Nothing was narrowed. The
/// payload text those kinds carry is pinned by
/// `driver::pinned_sentences`, one row per sentence, which is where a change
/// to the wording has to be made deliberately.
fn assert_rule_kind(source: &[u8], rule: SemanticRule, kind: fn(&SemanticIssueKind) -> bool) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected {rule:?} with a matching kind, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert!(kind(issue.kind()), "unexpected kind {:?}", issue.kind());
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
fn a_branch_fact_discharges_the_protected_array_read() {
    let source = br#"fn read(values: own array<i32, 8>, i: own u64) -> result: own i32 pure {
  let length = len_of(values);
  if i < length {
    return values[i];
  } else {
    return values[0_u64];
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("expected acceptance, got {outcome:?}");
        };
    });
}

#[test]
fn repeated_normalized_uses_are_a_prf1_rejection() {
    let source = br#"fn combine(value: own u64, limit: own u64) -> result: own unit pure contract {
  requires value <= limit;
} {
  invariant scaled: 3_u64 * value <= 3_u64 * limit {
    use value <= limit;
    use value <= limit;
    use value <= limit;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Prf1, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedSourceProof { .. })
    });
}

#[test]
fn a_non_ordered_local_invariant_target_is_an_inv1_rejection() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  invariant held: 0_u64 == 0_u64;
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Inv1, |kind| {
        matches!(kind, SemanticIssueKind::InvalidInvariant { .. })
    });
}

#[test]
fn an_unproved_blockless_local_invariant_target_is_an_inv1_rejection() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  invariant impossible: 1_u64 <= 0_u64;
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Inv1, |kind| {
        matches!(
            kind,
            SemanticIssueKind::UndischargedLocalInvariant { name, .. }
                if name == "impossible"
        )
    });
}

#[test]
fn a_non_ordered_use_relation_is_a_prf1_rejection() {
    let source = br#"fn check(value: own u64, limit: own u64) -> result: own unit pure {
  invariant scaled: 2_u64 * value <= 2_u64 * limit {
    use value == limit;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule_kind(source, SemanticRule::Prf1, |kind| {
        matches!(kind, SemanticIssueKind::InvalidSourceProof { .. })
    });
}

#[test]
fn scalar_constants_calls_and_operations_publish_one_checked_program() {
    let source = br#"const base: i32 = 40_i32;

fn add(x: own i32, y: own i32) -> result: own i32 pure {
  return x +wrap y;
}

command fn main() -> status: own ExitStatus pure {
  let result = add(x: base, y: 2_i32);
  return exit_status(code: 0_u8);
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
        b"command fn main() -> status: own ExitStatus pure {\n  let value = 128_i8;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Form7,
        SemanticIssueKind::InvalidIntegerLiteral,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return 0_i32;\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::ReturnMismatch,
    );
    assert_rule_kind(
        b"command fn main() -> status: own ExitStatus pure {\n  invariant bad: 0_u64 == 0_u64;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Inv1,
        |kind| matches!(kind, SemanticIssueKind::InvalidInvariant { .. }),
    );
    // [S23] both EFF-2 arms name a provider path: the first declares less
    // than the body exhibits, the second more.
    assert_rule_kind(
        b"fn helper(heap: &uniq Heap) -> result: own unit reads(heap), writes(heap) {\n  region {\n    match heap_vector::<u8>(store: &uniq deref(heap), count: 1_u64) {\n      None() => {\n        return unit;\n      }\n      Some(value: run) => {\n        return unit;\n      }\n    }\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
    assert_rule_kind(
        b"fn helper(heap: &uniq Heap) -> result: own unit allocates(heap) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn function_control_and_main_contract_are_checked_before_lowering() {
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::FunctionFallthrough,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
    assert_rule(
        b"fn main(value: own i32) -> result: own unit pure {\n  return unit;\n}\n",
        SemanticRule::Fn7,
        SemanticIssueKind::InvalidMain,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  loop @done {\n    break @done;\n    return exit_status(code: 0_u8);\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn1,
        SemanticIssueKind::UnreachableStatement,
    );
}

/// [OWN-11] the per-iteration judgment, which is [LIV-1]'s liveness agreement
/// read at a loop head.
///
/// A body that leaves an outer binding dead on the backedge is the rejection;
/// a body that moves one and commits a value back into it before the backedge
/// agrees with the entering edge and is accepted.
#[test]
fn loops_enforce_own11_for_outer_affine_moves() {
    assert_rule(
        br#"fn measure(cell: own buffer<u8>) -> size: own u64 reads(cell) {
  let n = len_of(cell);
  return n;
}

command fn main() -> status: own ExitStatus pure {
  let c = buffer_new(4_u64, 0_u8);
  for (i in 0_u64..2_u64) {
    let taken = measure(cell: move c);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::MoveOuterBindingInLoop {
            binding: "c".to_owned(),
            mechanical_fix: "one iteration must leave every outer binding in the status the next \
                             one starts from: commit a value back into it before the backedge, or \
                             declare and consume it inside the body",
        },
    );
    with_semantics(
        br#"fn measure(cell: own buffer<u8>) -> size: own u64 reads(cell) {
  let n = len_of(cell);
  return n;
}

command fn main() -> status: own ExitStatus pure {
  let c = buffer_new(4_u64, 0_u8);
  for (i in 0_u64..2_u64) {
    let taken = measure(cell: move c);
    set c = buffer_new(4_u64, 0_u8);
  }
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!(
                    "a body that reinitializes the outer binding agrees at the backedge: {outcome:?}"
                );
            };
        },
    );
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  loop @forever {\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "a break-free loop has a contradictory continuation rather than an unsupported shape: {outcome:?}"
            );
        },
    );
}

#[test]
fn loop_break_and_backedge_cleanup_is_explicit() {
    let source = br#"struct Cell {
  value: i32;
}

command fn main() -> status: own ExitStatus pure {
  loop @again {
    let first = Cell(value: 1_i32);
    if True() {
      break @again;
    }
    let second = Cell(value: 2_i32);
  }
  return exit_status(code: 0_u8);
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
    let wrong_name = br#"fn take(value: own i32) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  take(other: 1_i32);
  return exit_status(code: 0_u8);
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
        b"command fn main() -> status: own ExitStatus pure {\n  let a = 1_i32;\n  let b = move a;\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main() -> status: own ExitStatus pure {\n  let value = 4_i32;\n  let narrowed = cvt(value);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let left = 1_i32;\n  let right = 2_i32;\n  let value = imin(left: left, right: right);\n  return exit_status(code: 0_u8);\n}\n",
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
    assert_rule_kind(
        b"struct Held {\n  v: i32;\n}\n\nfn pick<T: affine>(value: own T) -> result: own T pure {\n  return move value;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let a = Held(v: 1_i32);\n  let b = pick(value: move a);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn2,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let value = 4_i32;\n  let narrowed = cvt(value);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );

    // A wrong-count argument list, the same failure on both classes.
    assert_rule_kind(
        b"struct Held {\n  v: i32;\n}\n\nfn pick<T: affine>(value: own T) -> result: own T pure {\n  return move value;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let a = Held(v: 1_i32);\n  let b = pick::<Held, Held>(value: move a);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Fn2,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let value = 4_i32;\n  let narrowed = cvt::<i32>(value);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );

    // A generic nominal's construct is a third class: [TYPE-5] owns its
    // written arguments, and it shares the argument-list reader with the
    // user-generic call, so it is the control that the rule is not simply
    // keyed on that reader.
    assert_rule_kind(
        b"struct Pair<T: affine> {\n  v: T;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let p = Pair(v: 1_i32);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn effect_mismatch_is_located_at_the_written_effect_row() {
    let source = b"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus pure {\n  region {\n    match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {\n      None() => {\n        return exit_status(code: 1_u8);\n      }\n      Some(value: run) => {\n        return exit_status(code: 0_u8);\n      }\n    }\n  }\n}\n";
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
        b"fn main<T: affine>() -> result: own unit pure {\n  return unit;\n}\n",
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
        b"struct Pair {\n  x: i32;\n  x: i32;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type6,
        SemanticIssueKind::DuplicateFieldLabel {
            label: "x".to_owned(),
        },
    );
    assert_rule(
        b"enum Pairing {\n  Both(a: i32, b: i32);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let pair = Both(a: 1_i32, b: 2_i32);\n  match move pair {\n    Both(a: first) => {\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
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
        b"command fn main() -> status: own ExitStatus pure {\n  let flag = True();\n  let result = if flag {\n  } else {\n    give 0_i32;\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Give1,
        SemanticIssueKind::InvalidGive,
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus pure {\n  let flag = True();\n  let result = if flag {\n    give 1_i32;\n    give 2_i32;\n  } else {\n    give 0_i32;\n  }\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Give1,
        SemanticIssueKind::InvalidGive,
    );
}

#[test]
fn enum_equality_exclusions_reach_the_intended_rule() {
    assert_rule(
        b"enum PayloadEq {\n  PayloadEmpty();\n  PayloadValue(value: u32);\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let left = PayloadEmpty();\n  let right = PayloadEmpty();\n  let equal = eeq(move left, move right);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
    assert_rule_kind(
        b"enum LeftEq {\n  LeftFirst();\n}\n\nenum RightEq {\n  RightFirst();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let left = LeftFirst();\n  let right = RightFirst();\n  let equal = eeq(left, right);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

#[test]
fn nominal_adjacent_unimplemented_behavior_stays_non_language_failure() {
    // Keep the capability control smaller than the conformance program: this
    // test isolates set-field construction and read-back, while
    // `x-struct-set-field.wf` additionally proves its exact increment from
    // the S5 post-write image.
    with_semantics(
        b"struct Counter {\n  n: i32;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let c = Counter(n: 1_i32);\n  set c.n = 41_i32;\n  let v = c.n;\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    // Borrow-mode parameters and `let` borrows of scalars and enums, and the
    // [OWN-13] borrowed match this exercises, are on the normal path now.
    // Also written inline rather than read from
    // `x-enum-borrow-payload-live.wf` for the same reason: that case's
    // `deref(x) + 1_i32` is an undischarged v0.31 class site (residual
    // `deref(x) <= 2147483646`), so it too stays outside this capability
    // control. The shape kept here is the one under test — a payload enum
    // borrow-matched through `&'r` whose scrutinee stays live for a second
    // read, with each derived binder explicitly dereferenced.
    with_semantics(
        b"enum Cell {\n  Full(v: i32);\n  Void();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let c = Full(v: 20_i32);\n  region {\n    let p = &c;\n    let a = match deref(p) {\n      Full(v: x) => {\n        give deref(x);\n      }\n      Void() => {\n        give 0_i32;\n      }\n    }\n    let q = &c;\n    let b = match deref(q) {\n      Full(v: y) => {\n        give deref(y);\n      }\n      Void() => {\n        give 0_i32;\n      }\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    assert_unsupported(
        b"struct Node {\n  next: Node;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        UnsupportedSemanticFeature::RecursiveNominalLayout,
    );
    assert_unsupported(
        b"enum Flag {\n  A();\n  B();\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let flag = A();\n  match flag {\n    A() => {\n    }\n    A() => {\n    }\n    B() => {\n    }\n  }\n  return exit_status(code: 0_u8);\n}\n",
        UnsupportedSemanticFeature::DuplicateMatchArm,
    );
    // [LIV-1] a liveness disagreement at a join is a source rejection now, so
    // the capability limit this control pins is the state a join still cannot
    // merge: the loop's entering value carries a fresh owner's attribution and
    // its committed value carries the callee's, which no rule of this version
    // joins.
    assert_unsupported(
        br#"fn consume(cell: own buffer<u8>) -> out: own buffer<u8> pure {
  return move cell;
}

command fn main() -> status: own ExitStatus pure {
  let c = buffer_new(4_u64, 0_u8);
  for (i in 0_u64..2_u64) {
    set c = consume(cell: move c);
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::OwnershipJoin,
    );
}

#[test]
fn undeclared_system_effect_categories_reject_both_row_directions() {
    // Capability effects are checked in both directions [EFF-1, EFF-2].
    // First an unexhibited declaration, then an undeclared exhibited read.
    assert_rule_kind(
        b"fn probe(args: own Args) -> result: own unit reads(args) {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
    assert_rule_kind(
        b"fn probe(args: own Args) -> result: own u64 pure {\n  region {\n    let total = args_count(args: &args);\n    return total;\n  }\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        |kind| matches!(kind, SemanticIssueKind::EffectMismatch { .. }),
    );
}

#[test]
fn checked_system_programs_complete_semantic_checking() {
    // The system semantic family — [SYS-2] call typing, [EFF-2] effect
    // attribution, and the release contribution — is implemented, so a
    // conforming kind-declaring unit completes semantic checking; the
    // remaining system boundary is lowering's explicit unsupported stop.
    with_semantics(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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

fn unwrap(holder: own box<Result<i32, StepError>>) -> result: own Result<i32, StepError> pure {
  let accepted = propagate holder;
  return Ok<i32, StepError>(value: accepted);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn inspect(holder: own box<State>) -> result: own unit pure {
  match holder {
    Ready() => {
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type7,
        SemanticIssueKind::MissingDereference {
            mechanical_fix: "write `deref(holder)`",
        },
    );
    assert_rule(
        br#"fn read(holder: own box<buffer<u8>>) -> result: own u8 pure {
  return holder[0_u64];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn step(value: own i32) -> result: own Result<i32, StepError> pure {
  return Ok<i32, StepError>(value: value);
}

fn forward(value: own i32) -> result: own Result<Pair, StepError> pure {
  let accepted = propagate step(value: value);
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
}

fn direct(error: own StepError) -> result: own Result<Pair, StepError> pure {
  let accepted = propagate Err<i32, StepError>(error: error);
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
}

fn bare(outcome: own Result<i32, StepError>) -> result: own Result<Pair, StepError> pure {
  let accepted = propagate outcome;
  let pair = Pair(value: accepted);
  return Ok<Pair, StepError>(value: move pair);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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

fn reuse(outcome: own Result<i32, StepError>) -> result: own Result<i32, StepError> pure {
  let accepted = propagate outcome;
  match outcome {
    Ok(value: second_value) => {
    }
    Err(error: second_error) => {
    }
  }
  return Ok<i32, StepError>(value: accepted);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    assert_rule_kind(
        br#"enum Flag {
  First();
  Second();
}

command fn main() -> status: own ExitStatus pure {
  let flag = First();
  match Err(error: flag) {
    Ok(value: ok_value) => {
    }
    Err(error: err_value) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
    assert_rule_kind(
        include_bytes!("../../../tests/conformance/cases/x-enum-result-payload-type-mismatch.wf"),
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
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

command fn main() -> status: own ExitStatus pure {
  let number = 1_i32;
  set number = 2_i32;
  let inner = Inner(value: 3_i32);
  let outer = Outer(inner: move inner, other: 4_i32);
  set outer.inner.value = number;
  return exit_status(code: 0_u8);
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
        b"const answer: i32 = 1_i32;\n\ncommand fn main() -> status: own ExitStatus pure {\n  set answer = 2_i32;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Const2,
        SemanticIssueKind::ImmutableSetTarget,
    );
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let left = Cell(value: 1_i32);\n  let right = Cell(value: 2_i32);\n  set left = move right;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "Cell".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    assert_rule_kind(
        b"command fn main() -> status: own ExitStatus pure {\n  let number = 1_i32;\n  set number = True();\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        |kind| matches!(kind, SemanticIssueKind::TypeMismatch { .. }),
    );
}

/// [LIV-2] an affine `set` target is admitted exactly when it is dead at the
/// commit.
///
/// The two halves of the rule's first condition are checked side by side: a
/// live affine target whose previous value the right-hand side does not read
/// out keeps [STOR-1]'s rejection and its one restructuring, and the same
/// statement whose right-hand side consumes that value is the read-out and is
/// accepted. The second program is probe `q9`'s shape, which [STOR-1] refused
/// before this rule and which offered a fresh-`let` restructuring that the
/// rule makes unnecessary.
#[test]
fn an_affine_set_is_admitted_exactly_when_its_target_is_dead_at_the_commit() {
    assert_rule(
        b"struct Cell {\n  value: i32;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  let left = Cell(value: 1_i32);\n  let right = Cell(value: 2_i32);\n  set left = move right;\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "Cell".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    with_semantics(
        br#"struct Counts {
  lines: u64;
  bytes: u64;
}

fn walk(running: own Counts) -> result: own Counts pure {
  return move running;
}

command fn main() -> status: own ExitStatus pure {
  let totals = Counts(lines: 0_u64, bytes: 0_u64);
  set totals = walk(running: move totals);
  let lines = totals.lines;
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("the read-out and its commit must check: {outcome:?}");
            };
        },
    );
    // The two-statement form stays accepted: [LIV-2] adds a spelling and
    // removes none.
    with_semantics(
        br#"struct Counts {
  lines: u64;
  bytes: u64;
}

fn walk(running: own Counts) -> result: own Counts pure {
  return move running;
}

command fn main() -> status: own ExitStatus pure {
  let totals = Counts(lines: 0_u64, bytes: 0_u64);
  let sub = walk(running: move totals);
  let lines = sub.lines;
  let bytes = sub.bytes;
  let total = lines +wrap bytes;
  let empty = total == 0_u64;
  if empty {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 1_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("the two-statement form must check: {outcome:?}");
            };
        },
    );
}

/// [LIV-2] after its read-out the target is dead for the remainder of the
/// right-hand side.
///
/// Every shape that would consume one target's value twice is a rejection: the
/// same place moved twice, the same field moved twice, and a field read out
/// beside a move of the whole root. Without the sentence the first of these
/// compiled and freed one run twice.
#[test]
fn a_read_out_target_is_dead_for_the_rest_of_the_right_hand_side() {
    let expected = SemanticIssueKind::UseAfterMove {
        mechanical_fix: "introduce a new `let` binding before reuse",
    };
    assert_rule(
        br#"fn pair(left: own buffer<u8>, right: own buffer<u8>) -> out: own buffer<u8> pure {
  return move left;
}

command fn main() -> status: own ExitStatus pure {
  let c = buffer_new(4_u64, 0_u8);
  set c = pair(left: move c, right: move c);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        expected.clone(),
    );
    assert_rule(
        br#"struct Holder {
  run: buffer<u8>;
}

fn pair(left: own buffer<u8>, right: own buffer<u8>) -> out: own buffer<u8> pure {
  return move left;
}

command fn main() -> status: own ExitStatus pure {
  let first = buffer_new(4_u64, 0_u8);
  let holder = Holder(run: move first);
  set holder.run = pair(left: move holder.run, right: move holder.run);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        expected.clone(),
    );
    assert_rule(
        br#"struct Holder {
  run: buffer<u8>;
  spare: buffer<u8>;
}

fn take(left: own buffer<u8>, right: own Holder) -> out: own buffer<u8> pure {
  return move left;
}

command fn main() -> status: own ExitStatus pure {
  let first = buffer_new(4_u64, 0_u8);
  let second = buffer_new(4_u64, 0_u8);
  let holder = Holder(run: move first, spare: move second);
  set holder.run = take(left: move holder.run, right: move holder);
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        expected,
    );
}

#[test]
fn set_revalidates_the_target_after_rhs_ownership_changes() {
    let source = br#"struct Cell {
  value: i32;
}

fn take(cell: own Cell) -> result: own i32 pure {
  return cell.value;
}

command fn main() -> status: own ExitStatus pure {
  let cell = Cell(value: 1_i32);
  set cell.value = take(cell: move cell);
  return exit_status(code: 0_u8);
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

fn make() -> result: own Cell pure {
  let cell = Cell(value: 1_i32);
  return move cell;
}

fn discard_call() -> result: own unit pure {
  make();
  return unit;
}

fn drop_binder(value: own Holder) -> result: own unit pure {
  match move value {
    Held(cell: item) => {
    }
    Empty() => {
    }
  }
  return unit;
}

fn drop_before_give(flag: own Bool) -> result: own i32 pure {
  let selected = if flag {
    let temporary = Cell(value: 2_i32);
    give 1_i32;
  } else {
    give 0_i32;
  }
  return selected;
}

fn move_through_give(flag: own Bool) -> result: own Cell pure {
  let selected = if flag {
    let temporary = Cell(value: 3_i32);
    give move temporary;
  } else {
    let temporary = Cell(value: 4_i32);
    give move temporary;
  }
  return move selected;
}

fn reverse_order() -> result: own unit pure {
  let first = Cell(value: 5_i32);
  let second = Cell(value: 6_i32);
  return unit;
}

fn consume_projection() -> result: own unit pure {
  let selected = Cell(value: 7_i32);
  let inner_sibling = Cell(value: 8_i32);
  let inner = Inner(selected: move selected, sibling: move inner_sibling);
  let outer_sibling = Cell(value: 9_i32);
  let outer = Outer(inner: move inner, sibling: move outer_sibling);
  let taken = move outer.inner.selected;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
