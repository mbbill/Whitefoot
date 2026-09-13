#![allow(clippy::panic)]

use crate::lexer::{LexLimits, LexOutcome, lex};
use crate::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalLimits, CanonicalOutcome, FinalizeLimits, FinalizeOutcome,
    ParseLimits, ParseOutcome, SourceBundle, SourceInput, SourceLimits, TerminalLimits,
    TerminalOutcome, audit_canonical, classify_terminals, finalize, parse,
};

use super::catalog::OPERATION_FAMILIES;
use super::{
    DeclarationClass, DeclarationDomain, DeclarationOrigin, DeclarationRole, DeferredUseRole,
    DependentDeclarationRole, LexicalUseRole, PostconditionSelectorClass, ReservedDeclarationRole,
    ResolutionIssue, ResolutionIssueKind, ResolutionOutcome, ResolutionRule, ResolvedTarget,
    ScopeKind, resolve,
};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 64,
    max_logical_path_bytes: 128,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_binding_bytes: 1_048_576,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 64,
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
    max_sources: 64,
};

const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 8_000_000,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_gaps: 131_072,
    max_path_components: 8_192,
};

fn with_resolution<ResultValue>(
    inputs: &[SourceInput<'_>],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        ResolutionOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_resolution_sources(inputs, false, run)
}

fn with_resolution_sources<ResultValue>(
    inputs: &[SourceInput<'_>],
    include_prelude: bool,
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        ResolutionOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let bundle = if include_prelude {
        SourceBundle::with_prelude(inputs, SOURCE_LIMITS)
    } else {
        SourceBundle::with_limits(inputs, SOURCE_LIMITS)
    }
    .expect("resolver test bundle must be valid");
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        panic!("resolver test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("resolver test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        panic!("resolver test source must parse");
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        panic!("resolver test derivation must finalize");
    };
    let canonical = audit_canonical(finalized, CANONICAL_LIMITS);
    let CanonicalOutcome::Complete(syntax) = canonical else {
        panic!("resolver test source must use exact FORM-2 formatting: {canonical:?}");
    };
    run(resolve(syntax))
}

fn with_one_resolution<ResultValue>(
    source: &[u8],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        ResolutionOutcome<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    with_resolution(&[SourceInput::new("test.wf", source)], run)
}

#[test]
fn minimal_function_publishes_the_closed_prelude_and_source_declaration() {
    with_one_resolution(b"fn probe() -> result: own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("minimal canonical function must resolve: {outcome:?}");
        };
        assert_eq!(resolved.prelude_declarations().len(), 24);
        assert_eq!(resolved.declarations().len(), 1);
        assert_eq!(resolved.declarations()[0].role(), DeclarationRole::Function);
        assert_eq!(resolved.declarations()[0].spelling(), "probe");
        assert!(resolved.scopes().len() >= 3);
    });
}

#[test]
fn top_level_functions_are_visible_throughout_the_closed_unit() {
    let source = br#"fn probe() -> result: own unit pure {
  helper();
}

fn helper() -> result: own unit pure {
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("later function must be globally visible: {outcome:?}");
        };
        let helper = resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.spelling() == "helper")
            .expect("helper declaration must exist");
        let call = resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.spelling() == "helper")
            .expect("helper call must exist");
        assert_eq!(call.role(), LexicalUseRole::IdentifierCallee);
        assert_eq!(
            call.target(),
            ResolvedTarget::Source {
                declaration: helper.id(),
                class: DeclarationClass::Function,
            }
        );
    });
}

#[test]
fn named_constants_remain_lexically_declaration_before_use() {
    let source = b"const first: i32 = second;\n\nconst second: i32 = 2_i32;\n";
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("later named constant must not be visible: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Const2);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "second"
        ));
    });
}

#[test]
fn decimal_array_sizes_need_no_lexical_target() {
    let source = br#"const values: FixedVector<i32, 4> =[0_i32, 0_i32, 0_i32, 0_i32];

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("a decimal const expression must resolve without a name role: {outcome:?}");
        };
        assert!(
            resolved
                .lexical_uses()
                .iter()
                .all(|usage| usage.role() != LexicalUseRole::Const)
        );
    });
}

#[test]
fn source_nominals_are_not_visible_before_their_declaration() {
    let source = br#"fn consume(value: own Later) -> result: own unit pure {
}

struct Later {
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("later nominal must not be visible: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "Later"
        ));
    });
}

#[test]
fn requires_shape_is_checked_before_names_inside_the_invalid_block() {
    let source = br#"fn guarded() -> result: own unit pure contract {
  define value = missing;
} {
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("invalid requires block must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Fn8);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ContractShape(_)
        ));
    });
}

#[test]
fn contract_structural_admission_selects_the_earliest_source() {
    let first_empty = br#"fn first() -> result: own i32 pure contract {
} {
  return 0_i32;
}
"#;
    let second_define_only = br#"fn second() -> result: own unit pure contract {
  define unresolved = missing;
} {
  return unit;
}
"#;

    for (first, second) in [
        (first_empty.as_slice(), second_define_only.as_slice()),
        (second_define_only.as_slice(), first_empty.as_slice()),
    ] {
        with_resolution(
            &[
                SourceInput::new("first.wf", first),
                SourceInput::new("second.wf", second),
            ],
            |outcome| {
                let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                    panic!("an invalid clause block must reject: {outcome:?}");
                };
                assert_eq!(issue.rule(), ResolutionRule::Fn8);
                assert_eq!(issue.origin().coordinate().source().ordinal(), 0);
            },
        );
    }
}

#[test]
fn plain_postcondition_selector_is_private_and_definitions_share_one_contract_scope() {
    let source = br#"fn relation(value: own i32) -> result: own i32 pure contract {
  define reflexive = value == value;
  requires reflexive;
  ensures result == value;
} {
  return value;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("plain postcondition surface must resolve: {outcome:?}");
        };
        let postcondition = resolved
            .postconditions()
            .first()
            .expect("one private postcondition record");
        assert_eq!(postcondition.class, PostconditionSelectorClass::Plain);
        let [candidate] = postcondition.result_binders.as_slice() else {
            panic!("one result ordinal binder is required");
        };
        assert_eq!(candidate.spelling, "result");
        assert_eq!(candidate.origin.role_ordinal(), 0);
        assert!(candidate.paired_field.is_none());
        assert!(candidate.live_conflicts.is_empty());
        assert!(candidate.later_local_collision.is_none());
        assert!(postcondition.fields.is_empty());
        assert!(postcondition.variant_target.is_none());
        assert!(postcondition.entry_inventory_issue.is_none());
        assert!(postcondition.entry_resolution_issue.is_none());
        assert_eq!(postcondition.selector_uses.len(), 1);
        assert_eq!(postcondition.selector_uses[0].spelling, "result");
        assert!(
            resolved
                .declarations()
                .iter()
                .all(|declaration| declaration.spelling() != "result")
        );
        assert!(
            resolved
                .lexical_uses()
                .iter()
                .all(|usage| usage.spelling() != "result")
        );

        let contracts = resolved
            .scopes()
            .iter()
            .filter(|scope| scope.kind() == ScopeKind::ContractBlock)
            .collect::<Vec<_>>();
        assert_eq!(contracts.len(), 1, "requires and ensures share one scope");
    });
}

#[test]
fn variant_postcondition_selector_preserves_prelude_identity_without_match_roles() {
    for field in ["value", "alternate"] {
        let source = format!(
            "fn selected(value: own i32) -> result: own Result<i32, i32> pure contract {{\n  ensures when Ok({field}: result): result == value;\n}} {{\n  return Ok<i32, i32>(value: value);\n}}\n"
        );
        with_one_resolution(source.as_bytes(), |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("variant postcondition surface must resolve: {outcome:?}");
            };
            let [postcondition] = resolved.postconditions() else {
                panic!("one private postcondition record is required");
            };
            assert_eq!(postcondition.class, PostconditionSelectorClass::Variant);
            // [CALL-4] the declaration writes one result, so exactly one
            // ordinal binder is a candidate; a routed clause names it
            // through its route rather than as a plain selector.
            assert_eq!(postcondition.result_binders.len(), 1);
            assert!(postcondition.route_ordinal.is_none());
            assert!(matches!(
                postcondition.variant_target,
                Some(ResolvedTarget::Prelude(id)) if id.ordinal() == 11
            ));
            let [selector_field] = postcondition.fields.as_slice() else {
                panic!("one selector field is required");
            };
            assert_eq!(selector_field.spelling, field);
            assert_eq!(selector_field.origin.role_ordinal(), 0);
            assert_eq!(selector_field.candidate.spelling, "result");
            assert_eq!(selector_field.candidate.origin.role_ordinal(), 1);
            assert_eq!(
                selector_field.candidate.paired_field.as_deref(),
                Some(field)
            );
            assert!(
                resolved
                    .declarations()
                    .iter()
                    .all(|declaration| declaration.role() != DeclarationRole::MatchBinder)
            );
            assert!(
                resolved
                    .deferred_uses()
                    .iter()
                    .all(|usage| usage.role() != DeferredUseRole::MatchField)
            );
        });
    }
}

#[test]
fn selector_candidates_use_their_exact_form3_reservation_roles() {
    let plain = br#"fn plain(value: own i32) -> cvt: own i32 pure contract {
  ensures cvt == value;
} {
  return value;
}
"#;
    let variant = br#"fn variant(value: own i32) -> result: own Result<i32, i32> pure contract {
  ensures when Ok(value: cvt): cvt == value;
} {
  return Ok<i32, i32>(value: value);
}
"#;
    for (source, expected_role, role_ordinal) in [
        (
            plain.as_slice(),
            ReservedDeclarationRole::PlainResultSelector,
            0,
        ),
        (
            variant.as_slice(),
            ReservedDeclarationRole::VariantResultSelector,
            1,
        ),
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("reserved selector candidate must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Form3);
            assert_eq!(issue.origin().role_ordinal(), role_ordinal);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::ReservedName {
                    spelling,
                    declaration_role,
                    inventory_ordinal: 44,
                    ..
                } if spelling == "cvt" && *declaration_role == expected_role
            ));
        });
    }
}

#[test]
fn postcondition_lookup_waits_for_selector_admission_and_live_conflicts_are_retained() {
    let unresolved = br#"fn unresolved() -> result: own unit pure contract {
  ensures result == missing;
} {
  return unit;
}
"#;
    with_one_resolution(unresolved, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("entry lookup must be retained behind selector admission: {outcome:?}");
        };
        let [postcondition] = resolved.postconditions() else {
            panic!("one private postcondition record is required");
        };
        assert!(postcondition.entry_inventory_issue.is_none());
        assert!(postcondition.provisional_uses.is_empty());
        assert!(matches!(
            postcondition.entry_resolution_issue.as_ref(),
            Some(issue)
                if issue.rule() == ResolutionRule::Type5
                    && matches!(
                        issue.kind(),
                        ResolutionIssueKind::UnresolvedUse { spelling, .. }
                            if spelling == "missing"
                    )
        ));
    });

    let inventory_conflict = br#"fn conflict(result: own i32) -> result: own i32 pure contract {
  ensures result == result;
} {
  return result;
}
"#;
    with_one_resolution(inventory_conflict, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("entry inventory must be retained behind selector admission: {outcome:?}");
        };
        let [postcondition] = resolved.postconditions() else {
            panic!("one private postcondition record is required");
        };
        let [candidate] = postcondition.result_binders.as_slice() else {
            panic!("one result ordinal binder is required");
        };
        assert_eq!(candidate.live_conflicts.len(), 1);
        assert!(candidate.later_local_collision.is_none());
        // The comparison is an operator token since v0.41 and produces no
        // lexical use; only the two `result` operands remain as selector uses.
        assert_eq!(postcondition.provisional_uses.len(), 0);
        assert_eq!(postcondition.selector_uses.len(), 2);
        assert!(postcondition.entry_resolution_issue.is_none());
        assert!(postcondition.entry_inventory_issue.is_none());
    });
}

#[test]
fn invalid_ensures_local_cannot_poison_an_ordinary_body_lookup() {
    let source = br#"fn poisoned(value: own i32) -> result: own i32 pure contract {
  define cvt = value == value;
  ensures result == value;
} {
  return value;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an invalid contract definition must reject before body lookup: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                declaration_role: ReservedDeclarationRole::Let,
                ..
            } if spelling == "cvt"
        ));
    });
}

#[test]
fn unresolved_variant_selector_keeps_its_lookup_verdict_before_entry_inventory() {
    let source =
        br#"fn unresolved(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Missing(value: result): result == value;
} {
  return Ok<i32, Overflow>(value: value);
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!(
                "leading selector lookup must reject before delayed entry inventory: {outcome:?}"
            );
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse {
                spelling,
                role: LexicalUseRole::EnsuresVariant,
                ..
            } if spelling == "Missing"
        ));
    });
}

#[test]
fn contract_definitions_are_shared_across_clauses_but_do_not_reach_the_body() {
    let requires_into_ensures = br#"fn isolated(value: own i32) -> result: own i32 pure contract {
  define pre = value;
  requires pre == value;
  ensures result == pre;
} {
  return value;
}
"#;
    with_one_resolution(requires_into_ensures, |outcome| {
        let ResolutionOutcome::Complete(_) = outcome else {
            panic!("one shared contract definition must reach both clause kinds: {outcome:?}");
        };
    });

    let ensures_into_body = br#"fn isolated(value: own i32) -> result: own i32 pure contract {
  define post = value;
  ensures result == post;
} {
  return post;
}
"#;
    with_one_resolution(ensures_into_body, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("ensures local must not reach the body: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "post"
        ));
    });
}

/// [MSR-6] a `pbase` admits an in-scope const generic.
///
/// Until v0.45 this test asserted the opposite: `available: [ConstGeneric]`
/// with the class inadmissible, which is the rejection the containers design
/// recorded as probe `q10`. That rejection is what [MSR-6] removes, so the
/// test now pins the admission and the declaration it resolves to rather than
/// being deleted.
#[test]
fn a_const_generic_resolves_as_an_ordinary_place_base() {
    let ordinary = br#"fn value<const n: u64>() -> result: own u64 pure {
  return n;
}

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    let postcondition = br#"fn value<const n: u64>() -> result: own u64 pure contract {
  ensures result == result;
} {
  return n;
}

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    for source in [ordinary.as_slice(), postcondition.as_slice()] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("a const generic is an ordinary pbase: {outcome:?}");
            };
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage: &&crate::resolution::LexicalUseRecord| {
                    usage.role() == LexicalUseRole::PlaceBase && usage.spelling() == "n"
                })
                .expect("the place base `n` is a resolved use");
            assert!(matches!(
                usage.target(),
                ResolvedTarget::Source {
                    class: DeclarationClass::ConstGeneric,
                    ..
                }
            ));
        });
    }
}

#[test]
fn fn8_admission_precedes_declaration_inventory() {
    // DIAG-1 fixes the stage order: complete unit-wide FN-8 admission precedes
    // declaration inventory. The FN-8 rejection therefore wins before the
    // complete ordinary declaration collection is installed for lookup.
    let source = br#"fn guarded(value: own i32) -> result: own i32 pure contract {
  define unresolved = missing;
} {
  return value;
}

fn main() -> result: own unit pure {
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the FN-8 defect must outrank declaration inventory: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Fn8);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ContractShape(_)
        ));
    });

    // The same unit with an admitted internal requirement reaches declaration
    // inventory and resolves normally.
    let admitted = br#"fn guarded(value: own i32) -> result: own i32 pure contract {
  requires value == value;
} {
  return value;
}

fn main() -> result: own unit pure {
  return unit;
}
"#;
    with_one_resolution(admitted, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("an admitted requires block must reach declaration inventory: {outcome:?}");
        };
        assert!(
            resolved
                .declarations()
                .iter()
                .any(|declaration| declaration.spelling() == "guarded")
        );
    });
}

#[test]
fn a_builtin_prelude_collision_reports_the_prelude_origin() {
    // [DIAG-1, PRE-1] a built-in nominal collision reports its prelude origin.
    let source =
        "fn main() -> result: own unit pure {\n  return unit;\n}\n\nstruct Overflow {\n}\n";
    with_one_resolution(source.as_bytes(), |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the prelude collision must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        let ResolutionIssueKind::DeclarationCollision {
            spelling,
            conflicts,
            ..
        } = issue.kind()
        else {
            panic!("expected a declaration collision: {issue:?}");
        };
        assert_eq!(spelling, "Overflow");
        assert!(
            conflicts
                .iter()
                .all(|conflict| matches!(conflict.origin(), DeclarationOrigin::Prelude(_)))
        );
    });
}

#[test]
fn requires_locals_do_not_escape_into_the_function_body() {
    let source = br#"fn guarded() -> result: own unit pure contract {
  define condition = 1_i32;
  requires condition == condition;
} {
  return condition;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("requires local must not reach the body: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "condition"
        ));
    });
}

#[test]
fn root_identifier_collisions_are_rejected_in_inventory_order() {
    let source = br#"fn value() -> result: own unit pure {
}

const value: i32 = 1_i32;
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("function and const must share the lexical namespace: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "value"
        ));
    });
}

#[test]
fn dotless_operation_names_are_reserved_from_source_declarations() {
    with_one_resolution(b"fn cvt() -> result: own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("operation name declaration must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                inventory_ordinal: 44,
                ..
            } if spelling == "cvt"
        ));
    });
}

/// OP-1: reservation is derived from the op column, so the six retired
/// comparison names left `DotlessOperationNames` the moment v0.41 respelled
/// their rows as operators, and a program may declare them like any other
/// identifier. The operator spellings themselves are not identifiers and
/// cannot be declared at all, so nothing is reserved on their behalf.
#[test]
fn every_retired_comparison_name_is_a_free_identifier() {
    for spelling in ["ieq", "ine", "ilt", "ile", "igt", "ige"] {
        let source = format!("fn {spelling}() -> result: own unit pure {{\n}}\n");
        with_one_resolution(source.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, ResolutionOutcome::Complete(_)),
                "a retired comparison name must be declarable: {outcome:?}"
            );
        });
    }
}

#[test]
fn a_dotless_operation_name_is_reserved_from_header_invariant_declarations() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for (
    index in 0_u64..limit,
    invariant cvt: index <= limit
  ) {
    break;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a reserved header invariant name must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                declaration_role: ReservedDeclarationRole::Invariant,
                ..
            } if spelling == "cvt"
        ));
    });
}

#[test]
fn a_dotless_operation_name_is_reserved_from_body_invariant_declarations() {
    let source = br#"fn probe(value: own u64) -> result: own unit pure {
  invariant cvt: value <= value;
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a reserved body invariant name must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                declaration_role: ReservedDeclarationRole::Invariant,
                ..
            } if spelling == "cvt"
        ));
    });
}

#[test]
fn region_names_are_unique_across_the_complete_function() {
    // Both blocks write `'r`, which [FORM-8] would separately reject because
    // neither body references it. Resolution runs first and owns the repeated
    // region name, which is the judgment under test.
    let source = br#"fn nested() -> result: own unit pure {
  region 'r {
    give unit;
  }
  region 'r {
    give unit;
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("repeated function region must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Own3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::RepeatedRegion { spelling, .. } if spelling == "'r"
        ));
    });
}

#[test]
fn a_break_label_must_lexically_enclose_the_break() {
    let source = br#"fn probe() -> result: own unit pure {
  loop @done {
    break @done;
  }
  break @done;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("out-of-scope label must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::NonEnclosingLabel { spelling, .. } if spelling == "@done"
        ));
    });
}

#[test]
fn counted_range_binder_and_label_are_visible_only_in_the_body() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in 0_u64..limit) {
    let copied = index;
    break @range;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("counted binder and label must resolve in their body: {outcome:?}");
        };
        let binder = resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.role() == DeclarationRole::CountedBinder)
            .expect("counted binder declaration must exist");
        assert_eq!(binder.spelling(), "index");
        let label = resolved
            .declarations()
            .iter()
            .find(|declaration| {
                declaration.role() == DeclarationRole::LoopLabel
                    && declaration.spelling() == "@range"
            })
            .expect("counted label declaration must exist");
        let binder_use = resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.spelling() == "index")
            .expect("body must use the counted binder");
        assert_eq!(binder_use.role(), LexicalUseRole::PlaceBase);
        assert_eq!(
            binder_use.target(),
            ResolvedTarget::Source {
                declaration: binder.id(),
                class: DeclarationClass::Value,
            }
        );
        let break_use = resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.spelling() == "@range")
            .expect("body break must use the counted label");
        assert_eq!(break_use.role(), LexicalUseRole::BreakLabel);
        assert_eq!(
            break_use.target(),
            ResolvedTarget::Source {
                declaration: label.id(),
                class: DeclarationClass::Label,
            }
        );
    });
}

#[test]
fn unlabeled_loops_keep_the_counted_binder_without_creating_label_records() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  loop {
    break;
  }
  for (
    index in 0_u64..limit,
    invariant ceiling: index <= limit
  ) {
    break;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("unlabeled loop forms must resolve structurally: {outcome:?}");
        };
        assert!(
            resolved
                .declarations()
                .iter()
                .all(|declaration| declaration.role() != DeclarationRole::LoopLabel)
        );
        assert!(
            resolved
                .lexical_uses()
                .iter()
                .all(|usage| usage.role() != LexicalUseRole::BreakLabel)
        );

        let binder = resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.role() == DeclarationRole::CountedBinder)
            .expect("an unlabeled counted loop still declares its binder");
        assert_eq!(binder.spelling(), "index");
        assert!(resolved.lexical_uses().iter().any(|usage| {
            usage.role() == LexicalUseRole::InvariantValue
                && usage.spelling() == "index"
                && usage.target()
                    == ResolvedTarget::Source {
                        declaration: binder.id(),
                        class: DeclarationClass::Value,
                    }
        }));
    });
}

#[test]
fn invariant_names_declare_facts_and_affine_locals_resolve_as_invariant_values() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (
    index in 0_u64..limit,
    invariant ceiling: index <= limit
  ) {
    break @range;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("invariant value uses must resolve in the counted body: {outcome:?}");
        };
        let binder = resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.role() == DeclarationRole::CountedBinder)
            .expect("counted binder declaration exists");
        let parameter = resolved
            .declarations()
            .iter()
            .find(|declaration| {
                declaration.role() == DeclarationRole::Parameter
                    && declaration.spelling() == "limit"
            })
            .expect("limit parameter declaration exists");
        let invariant_uses = resolved
            .lexical_uses()
            .iter()
            .filter(|usage| usage.role() == LexicalUseRole::InvariantValue)
            .collect::<Vec<_>>();
        assert_eq!(invariant_uses.len(), 2);
        assert_eq!(invariant_uses[0].spelling(), "index");
        assert_eq!(
            invariant_uses[0].target(),
            ResolvedTarget::Source {
                declaration: binder.id(),
                class: DeclarationClass::Value,
            }
        );
        assert_eq!(invariant_uses[1].spelling(), "limit");
        assert_eq!(
            invariant_uses[1].target(),
            ResolvedTarget::Source {
                declaration: parameter.id(),
                class: DeclarationClass::Value,
            }
        );
        assert!(resolved.declarations().iter().any(|declaration| {
            declaration.role() == DeclarationRole::Invariant && declaration.spelling() == "ceiling"
        }));
        assert!(
            resolved
                .lexical_uses()
                .iter()
                .all(|usage| usage.spelling() != "ile"),
            "the invariant relation carrier must not create a lexical use"
        );
    });
}

#[test]
fn an_unresolved_affine_local_is_reported_as_an_invariant_value() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (
    index in 0_u64..limit,
    invariant ceiling: index <= missing
  ) {
    break @range;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unresolved invariant value must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Inv1);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse {
                spelling,
                role: LexicalUseRole::InvariantValue,
                ..
            } if spelling == "missing"
        ));
    });
}

#[test]
fn counted_range_binder_is_invisible_in_both_endpoints_and_after_the_loop() {
    for source in [
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in index..limit) {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in 0_u64..index) {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in 0_u64..limit) {
    break @range;
  }
  let after = index;
  return unit;
}
"#
        .as_slice(),
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("counted binder must be invisible outside its body: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Type5);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "index"
            ));
        });
    }
}

#[test]
fn counted_range_endpoints_with_an_outer_same_name_still_enforce_no_shadowing() {
    for source in [
        br#"fn probe(limit: own u64) -> result: own unit pure {
  let index = 0_u64;
  for @range (index in index..limit) {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(limit: own u64) -> result: own unit pure {
  let index = 0_u64;
  for @range (index in 0_u64..index) {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("the live outer declaration must still enforce no-shadowing: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Type6);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::DeclarationCollision { spelling, .. }
                    if spelling == "index"
            ));
        });
    }
}

#[test]
fn invariant_fact_names_resolve_only_after_their_complete_declaration() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for (
    index in 0_u64..limit,
    invariant ceiling: index <= limit
  ) {
    invariant repeated: index <= limit {
      use ceiling;
      use (index <= limit);
    }
    invariant chained: index <= limit {
      use repeated;
    }
    break;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("published invariant names must resolve in their dominance region: {outcome:?}");
        };
        let named = resolved
            .declarations()
            .iter()
            .find(|declaration| {
                declaration.role() == DeclarationRole::Invariant
                    && declaration.spelling() == "ceiling"
            })
            .expect("header invariant declaration exists");
        let local = resolved
            .declarations()
            .iter()
            .find(|declaration| {
                declaration.role() == DeclarationRole::Invariant
                    && declaration.spelling() == "repeated"
            })
            .expect("local invariant declaration exists");
        for (spelling, declaration) in [("ceiling", named.id()), ("repeated", local.id())] {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| {
                    usage.role() == LexicalUseRole::InvariantFact && usage.spelling() == spelling
                })
                .expect("named use must become an invariant-fact use");
            assert_eq!(
                usage.target(),
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Invariant,
                }
            );
        }
        assert_eq!(
            resolved
                .lexical_uses()
                .iter()
                .filter(|usage| usage.role() == LexicalUseRole::InvariantValue)
                .count(),
            6
        );
        assert_eq!(
            resolved
                .lexical_uses()
                .iter()
                .filter(|usage| usage.role() == LexicalUseRole::ProofValue)
                .count(),
            2
        );
    });

    let self_reference = br#"fn probe(value: own i32) -> result: own unit pure {
  invariant same: value <= value {
    use same;
  }
  return unit;
}
"#;
    with_one_resolution(self_reference, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an invariant name must not resolve inside its own certificate: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Inv1);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse {
                spelling,
                role: LexicalUseRole::InvariantFact,
                ..
            } if spelling == "same"
        ));
    });
}

#[test]
fn an_unresolved_relation_use_value_is_reported_by_prf1() {
    let source = br#"fn probe(value: own u64, limit: own u64) -> result: own unit pure {
  invariant scaled: 3_u64 * value <= 3_u64 * limit {
    use (value <= missing);
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unresolved use relation value must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Prf1);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse {
                spelling,
                role: LexicalUseRole::ProofValue,
                ..
            } if spelling == "missing"
        ));
    });
}

#[test]
fn repeated_header_and_local_invariant_names_are_reported_by_inv1() {
    for source in [
        br#"fn probe(value: own u64) -> result: own unit pure {
  invariant same: value <= value;
  invariant same: value <= value;
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(value: own u64) -> result: own unit pure {
  loop (
    invariant same: value <= value,
    invariant same: value <= value
  ) {
    break;
  }
  return unit;
}
"#
        .as_slice(),
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a repeated invariant name must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Inv1);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::DeclarationCollision { spelling, .. }
                    if spelling == "same"
            ));
        });
    }
}

#[test]
fn header_invariant_names_are_invisible_after_their_loop() {
    for source in [
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for (
    index in 0_u64..limit,
    invariant ceiling: index <= limit
  ) {
  }
  invariant after: 0_u64 <= limit {
    use ceiling;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(value: own u64) -> result: own unit pure {
  loop (
    invariant stable: value <= value
  ) {
    break;
  }
  invariant after: value <= value {
    use stable;
  }
  return unit;
}
"#
        .as_slice(),
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a header invariant name must end with its loop: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Inv1);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::InvisibleUse {
                    role: LexicalUseRole::InvariantFact,
                    ..
                }
            ));
        });
    }
}

#[test]
fn counted_range_label_is_non_enclosing_after_the_loop() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in 0_u64..limit) {
    break @range;
  }
  break @range;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("counted label must not escape its loop: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::NonEnclosingLabel { spelling, .. } if spelling == "@range"
        ));
    });
}

#[test]
fn counted_range_binder_uses_the_for_binder_reservation_role() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (cvt in 0_u64..limit) {
    break @range;
  }
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("reserved counted binder must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                declaration_role: ReservedDeclarationRole::ForBinder,
                ..
            } if spelling == "cvt"
        ));
    });
}

#[test]
fn counted_range_scope_rejects_live_shadowing_and_allows_expired_reuse() {
    let live_outer = br#"fn probe(limit: own u64) -> result: own unit pure {
  let index = 0_u64;
  for @range (index in 0_u64..limit) {
    break @range;
  }
  return unit;
}
"#;
    with_one_resolution(live_outer, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("counted binder must not shadow a live outer binding: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "index"
        ));
    });

    let nested = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @outer (index in 0_u64..limit) {
    for @inner (index in 0_u64..limit) {
      break @inner;
    }
    break @outer;
  }
  return unit;
}
"#;
    with_one_resolution(nested, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("nested counted binder must not shadow its live parent: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "index"
        ));
    });

    let nested_distinct = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @outer (outer_index in 0_u64..limit) {
    for @inner (inner_index in outer_index..limit) {
      let copied = inner_index;
      break @inner;
    }
    break @outer;
  }
  return unit;
}
"#;
    with_one_resolution(nested_distinct, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "distinct nested counted declarations must resolve: {outcome:?}"
        );
    });

    let reused = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range (index in 0_u64..limit) {
    break @range;
  }
  for @range (index in 0_u64..limit) {
    break @range;
  }
  let index = 7_u64;
  let copied = index;
  return unit;
}
"#;
    with_one_resolution(reused, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "expired counted binder and label spellings may be reused: {outcome:?}"
        );
    });
}

/// Whether a family is spelled as an operator rather than as a callee name.
///
/// [OP-7] respelled twenty rows, and an operator token is never a declaration,
/// callee IDENT, or OPNAME [OP-1], so the two halves of the inventory now
/// resolve by different mechanisms and cannot be asserted together.
fn is_operator_family(spelling: &str) -> bool {
    !spelling.as_bytes()[0].is_ascii_alphabetic()
}

#[test]
fn dotless_and_dotted_operations_resolve_by_exact_op1_spelling() {
    let source = br#"fn probe() -> result: own unit pure {
  let negated = ineg(1_i32);
  let smaller = imin(negated, 2_i32);
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("closed operations must resolve: {outcome:?}");
        };
        for spelling in ["ineg", "imin"] {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| usage.spelling() == spelling)
                .unwrap_or_else(|| panic!("missing operation use {spelling}"));
            assert!(matches!(usage.target(), ResolvedTarget::Operation(_)));
        }
    });
}

/// [OP-1] "infix resolution consults no name domain, and an operator token is
/// never a declaration, callee IDENT, or OPNAME". The respelling therefore did
/// not move these rows to a different name — it took them out of the name
/// domain entirely, which is a property worth a gate of its own.
///
/// Since v0.41 the integer comparisons cross this line with the arithmetic:
/// `==` is a second respelled subject, and `imin` is the one named control.
#[test]
fn a_respelled_family_produces_no_lexical_use_at_all() {
    let source = br#"fn probe() -> result: own unit pure {
  let sum = 1_i32 +wrap 2_i32;
  let equal = sum == 3_i32;
  let named = imin(sum, 3_i32);
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("infix operations must resolve: {outcome:?}");
        };
        let mut operations: Vec<_> = resolved
            .lexical_uses()
            .iter()
            .filter(|usage| matches!(usage.target(), ResolvedTarget::Operation(_)))
            .map(|usage| usage.spelling())
            .collect();
        operations.sort_unstable();
        // The named row in the same function is the control: it proves the
        // filter finds operation uses at all, so the two infix rows being
        // absent is the property and not an empty search.
        assert_eq!(operations, ["imin"]);
        assert!(
            !resolved
                .lexical_uses()
                .iter()
                .any(|usage| usage.spelling() == "=="),
            "a respelled comparison must produce no lexical use"
        );
        assert!(
            !resolved
                .lexical_uses()
                .iter()
                .any(|usage| usage.spelling() == "+wrap"),
            "a respelled family must produce no lexical use"
        );
    });
}

#[test]
fn match_binder_cannot_equal_its_paired_field_name() {
    let source = br#"fn probe() -> result: own unit pure {
  match unit {
    Some(value: value) => {
      return unit;
    }
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("non-fresh match binder must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Gram10);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::MatchBinderFreshness { spelling, .. } if spelling == "value"
        ));
    });
}

#[test]
fn arm_lookup_does_not_accept_a_struct_constructor() {
    let source = br#"struct Boxed {
}

fn probe() -> result: own unit pure {
  match unit {
    Boxed() => {
      return unit;
    }
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("arm must require an enum variant: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, available, .. }
                if spelling == "Boxed" && available.contains(&DeclarationClass::StructConstructor)
        ));
    });
}

/// Three roles moved position under v0.23 and the fixture follows them rather
/// than the assertions moving. `TypeRegion` came only from a `let` annotation
/// that A3 deletes, so it now rides a signature-borne `Slice<'v, i32>`, which
/// [TYPE-5] keeps written. `OperationCallee` is the OPNAME form specifically
/// (`roles.rs` keys it on `TerminalPredicate::OperationName`), and the
/// fixture's only operation call was `iadd.wrap`, one of the rows [OP-7]
/// respelled — an operator token is never a callee, so a respelled row
/// produces no lexical use at all. It now rides `iabs.checked`, a dotted row
/// that keeps its operation-name route.
#[test]
fn complete_role_fixture_materializes_every_d_u_and_x_family() {
    // B7c4b left `viewer` on the retiring surface: the fixture has to
    // materialize `LexicalUseRole::EffectAllocationRegion`, and the only
    // spelling that produces it is the region-keyed `allocates(arena 'r)`
    // entry, which has no path form on the container surface and retires with
    // `arena<'r, T>` itself.
    let source = br#"formal Bound {
  fn member(value: &i32) -> result: own i32 reads(value);
}

formal Numeric<T: Int> {
  fn zero() -> result: own T pure;
}

struct Package<T: affine, const n: i32> {
  items: FixedVector<T, n>;
}

enum Choice<T: affine> {
  Absent();
  Present(value: T);
}

const one: i32 = 1_i32;

const two: i32 = one;

fn implementation(value: own i32) -> result: own i32 pure {
  return value;
}

actual Implementation : Bound {
  member = implementation;
}

fn user<T: affine, const n: i32>['call](arg: &'call T) -> result: &'call T reads(arg) {
  return arg;
}

fn grouped<Bound>() -> result: own i32 pure {
  let called = Bound::member(value: 1_i32);
  return called;
}

fn viewer['v](values: own Slice<'v, i32>, capability: own Args) -> result: own unit reads(values, capability), allocates(arena 'v) {
  let held = arena_new::<'v, i32>(1_i32);
  return unit;
}

fn numeric<T: Int>() -> result: own T pure {
  return 0_T;
}

fn probe() -> result: own unit pure {
  let ordinary = 1_i32 +wrap two;
  let smaller = iabs.checked(ordinary);
  let made = Package<i32, one>(items: ordinary);
  set deref(made).items = ordinary;
  region 'outer {
    let borrowed = &ordinary;
    let called = user::<i32, one>(arg: borrowed);
    let view = move called;
    let comparison = ordinary == two;
    region {
      let outward = &'outer ordinary;
    }
  }
  loop @done {
    break @done;
  }
  for @counted (index in 0_u64..2_u64) {
    let observed = index;
    break @counted;
  }
  match ordinary {
    Present(value: payload) => {
      give payload;
    }
    Absent() => {
      return unit;
    }
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("complete role fixture must resolve: {outcome:?}");
        };

        let declaration_roles: Vec<_> = resolved
            .declarations()
            .iter()
            .map(|declaration| declaration.role())
            .collect();
        for role in [
            DeclarationRole::Function,
            DeclarationRole::Struct,
            DeclarationRole::Enum,
            DeclarationRole::Variant,
            DeclarationRole::Formal,
            DeclarationRole::Actual,
            DeclarationRole::FunctionParameter,
            DeclarationRole::NamedConst,
            DeclarationRole::GenericType,
            DeclarationRole::ConstGeneric,
            DeclarationRole::RegionParameter,
            DeclarationRole::Parameter,
            DeclarationRole::Let,
            DeclarationRole::LoopLabel,
            DeclarationRole::LocalRegion,
            DeclarationRole::MatchBinder,
            DeclarationRole::CountedBinder,
        ] {
            assert!(
                declaration_roles.contains(&role),
                "missing declaration role {role:?}"
            );
        }

        let dependent_roles: Vec<_> = resolved
            .dependent_declarations()
            .iter()
            .map(|declaration| declaration.role())
            .collect();
        for role in [
            DependentDeclarationRole::Field,
            DependentDeclarationRole::VariantField,
        ] {
            assert!(
                dependent_roles.contains(&role),
                "missing dependent role {role:?}"
            );
        }

        let lexical_roles: Vec<_> = resolved
            .lexical_uses()
            .iter()
            .map(|usage| usage.role())
            .collect();
        for role in [
            LexicalUseRole::Type,
            LexicalUseRole::GenericBound,
            LexicalUseRole::FormalGroup,
            LexicalUseRole::Construct,
            LexicalUseRole::ArmVariant,
            LexicalUseRole::TypeRegion,
            LexicalUseRole::ModeRegion,
            LexicalUseRole::TypeArgumentRegion,
            LexicalUseRole::EffectAllocationRegion,
            LexicalUseRole::EffectRoot,
            LexicalUseRole::BorrowRegion,
            LexicalUseRole::BreakLabel,
            LexicalUseRole::Const,
            LexicalUseRole::ConstValue,
            LexicalUseRole::PlaceBase,
            LexicalUseRole::IdentifierCallee,
            LexicalUseRole::OperationCallee,
            LexicalUseRole::FunctionBinding,
            LexicalUseRole::GenericNumericSuffix,
        ] {
            assert!(
                lexical_roles.contains(&role),
                "missing lexical role {role:?}"
            );
        }

        let deferred_roles: Vec<_> = resolved
            .deferred_uses()
            .iter()
            .map(|usage| usage.role())
            .collect();
        for role in [
            DeferredUseRole::FieldInitializer,
            DeferredUseRole::MatchField,
            DeferredUseRole::ProjectedField,
            DeferredUseRole::FunctionBinding,
            DeferredUseRole::FunctionMember,
        ] {
            assert!(
                deferred_roles.contains(&role),
                "missing deferred role {role:?}"
            );
        }

        // D7 retires law arguments; numeric literals retain their own
        // suffix use, and qualified calls add the deferred member use above.
        let suffix = resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.role() == LexicalUseRole::GenericNumericSuffix)
            .expect("generic literal suffix must resolve");
        assert_eq!(suffix.origin().subtoken_ordinal(), 1);
    });
}

#[test]
fn effect_paths_resolve_the_exact_formal_parameter_and_retain_fields() {
    let source = b"struct Holder {\n  output: OutputStream;\n}\n\nfn publish(holder: own Holder) -> result: own unit writes(holder.output) {\n  return unit;\n}\n";
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("a parameter-rooted state path must resolve: {outcome:?}");
        };
        let parameter = resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.role() == DeclarationRole::Parameter)
            .expect("the parameter declaration exists");
        let usage = resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.role() == LexicalUseRole::EffectRoot)
            .expect("the effect root use exists");
        assert_eq!(usage.spelling(), "holder");
        assert_eq!(
            usage.target(),
            ResolvedTarget::Source {
                declaration: parameter.id(),
                class: DeclarationClass::Value,
            }
        );
        let field = resolved
            .deferred_uses()
            .iter()
            .find(|usage| usage.role() == DeferredUseRole::EffectField)
            .expect("the static effect field is retained");
        assert_eq!(field.spelling(), "output");
    });
}

#[test]
fn unresolved_and_body_local_effect_targets_reject_under_eff1() {
    for source in [
        &b"fn probe() -> result: own unit reads(missing) {\n  return unit;\n}\n"[..],
        &b"fn probe() -> result: own unit reads(local) {\n  let local = 0_u64;\n  return unit;\n}\n"[..],
    ] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a non-formal effect target must reject in resolution: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Eff1);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::UnresolvedUse {
                    role: LexicalUseRole::EffectRoot,
                    ..
                }
                    | ResolutionIssueKind::InvisibleUse {
                        role: LexicalUseRole::EffectRoot,
                        ..
                    }
            ));
        });
    }
}

#[test]
fn existing_positive_conformance_programs_resolve_without_fixture_rewrites() {
    for source in [
        include_bytes!("../../../tests/conformance/cases/fn3-pos-contract-conform.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/ex1-pos-worked-example.wf").as_slice(),
        include_bytes!("../../../tests/conformance/cases/gram10-pos-named-binders.wf").as_slice(),
    ] {
        with_one_resolution(source, |outcome| {
            assert!(
                matches!(outcome, ResolutionOutcome::Complete(_)),
                "positive compiler-independent conformance source must resolve: {outcome:?}"
            );
        });
    }
}

#[test]
fn existing_requires_scope_conformance_case_reaches_type5_resolution() {
    let source =
        include_bytes!("../../../tests/conformance/cases/fn8-neg-requires-local-in-body.wf");
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("requires-scope conformance case must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
    });
}

#[test]
fn prelude_collision_payload_keeps_both_ordered_struct_domains() {
    with_one_resolution(b"struct Overflow {\n}\n", |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("prelude collision must reject: {outcome:?}");
        };
        let ResolutionIssueKind::DeclarationCollision { conflicts, .. } = issue.kind() else {
            panic!("expected a declaration collision: {issue:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].domain(), DeclarationDomain::NominalType);
        assert_eq!(conflicts[1].domain(), DeclarationDomain::Constructor);
        assert!(
            matches!(conflicts[0].origin(), DeclarationOrigin::Prelude(id) if id.ordinal() == 15)
        );
        assert!(
            matches!(conflicts[1].origin(), DeclarationOrigin::Prelude(id) if id.ordinal() == 16)
        );
    });
}

#[test]
fn duplicate_main_conformance_case_is_type6() {
    let source = include_bytes!("../../../tests/conformance/cases/fn7-neg-two-mains.wf");
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the later main declaration must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, conflicts, .. }
                if spelling == "main" && conflicts.len() == 1
        ));
    });
}

#[test]
fn nested_declarations_cannot_shadow_source_later_global_functions() {
    let source = br#"fn probe() -> result: own unit pure {
  let future = 1_i32;
  return unit;
}

fn future() -> result: own unit pure {
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("whole-unit function visibility must prevent shadowing: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "future"
        ));
    });
}

#[test]
fn sibling_formal_signatures_do_not_share_region_parameters() {
    let source = br#"formal Separate {
  fn first(value: &i32) -> result: own unit pure;
  fn second() -> result: own Slice<'r, i32> pure;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("sibling member region must not participate: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Own3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, .. } if spelling == "'r"
        ));
    });
}

/// A conditional's two blocks are two lexical blocks [TYPE-6], and they are
/// the one construct whose blocks hang off the same node [GRAM-4], so the
/// scope tree has to separate them by brace pair rather than by production.
///
/// The two rejecting sources are the controls that make the two accepting ones
/// mean something: `match` arms, which are separate productions, must reach the
/// same answer as the `if` branches; and a genuine shadow of a live enclosing
/// binder must still be rejected, since giving each branch its own scope would
/// otherwise hide it.
#[test]
fn conditional_branches_are_separate_lexical_scopes() {
    let sibling_branches = br#"fn get(pick: own Bool) -> result: own unit pure {
  if pick {
    let inside = 1_u64;
    let observed = inside;
  } else {
    let inside = 2_u64;
    let observed = inside;
  }
  return unit;
}
"#;
    with_one_resolution(sibling_branches, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "disjoint sibling branches may reuse a spelling: {outcome:?}"
        );
    });

    let arm_control = br#"enum Pick {
  Left();
  Right();
}

fn get(pick: own Pick) -> result: own unit pure {
  match pick {
    Left() => {
      let inside = 1_u64;
      let observed = inside;
    }
    Right() => {
      let inside = 2_u64;
      let observed = inside;
    }
  }
  return unit;
}
"#;
    with_one_resolution(arm_control, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "the arm spelling of the same program must resolve too: {outcome:?}"
        );
    });

    let expired_then_enclosing = br#"fn get(pick: own Bool) -> result: own unit pure {
  if pick {
    let offset = 0_u64;
    let inner = offset;
  }
  let offset = 1_u64;
  let outer = offset;
  return unit;
}
"#;
    with_one_resolution(expired_then_enclosing, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "an expired branch scope may not block a later enclosing binder: {outcome:?}"
        );
    });

    let live_shadow = br#"fn get(pick: own Bool) -> result: own unit pure {
  let offset = 0_u64;
  if pick {
    let offset = 1_u64;
    let inner = offset;
  }
  let outer = offset;
  return unit;
}
"#;
    with_one_resolution(live_shadow, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a branch binder may not shadow a live enclosing one: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "offset"
        ));
    });
}

#[test]
fn semantic_stage_order_precedes_source_position_and_inventory_rank_is_event_local() {
    let later_inventory_error = br#"fn probe() -> result: own unit pure {
  missing();
}

fn cvt() -> result: own unit pure {
}
"#;
    with_one_resolution(later_inventory_error, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("inventory must reject before lookup: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });

    let later_fn8_error = br#"fn cvt() -> result: own unit pure {
}

fn guarded() -> result: own unit pure contract {
  define value = 1_i32;
} {
  return unit;
}
"#;
    with_one_resolution(later_fn8_error, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("FN-8 must reject before inventory: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Fn8);
    });

    let earlier_lower_rank = br#"fn value() -> result: own unit pure {
}

const value: i32 = 1_i32;

fn cvt() -> result: own unit pure {
}
"#;
    with_one_resolution(earlier_lower_rank, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("minimum declaration event must win before rank: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, .. } if spelling == "value"
        ));
    });
}

#[test]
fn identifier_renaming_preserves_general_resolution_structure() {
    for (helper, local) in [
        ("a", "x"),
        ("helper_name", "local_value"),
        ("function_27", "binding_42"),
    ] {
        let source = format!(
            "fn {helper}() -> result: own unit pure {{\n}}\n\nfn probe() -> result: own unit pure {{\n  let {local} = 1_i32;\n  {helper}();\n  return {local};\n}}\n"
        );
        with_one_resolution(source.as_bytes(), |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("ordinary renaming must preserve resolution: {outcome:?}");
            };
            assert_eq!(resolved.declarations().len(), 3);
            assert!(resolved.lexical_uses().iter().any(|usage| {
                usage.spelling() == helper
                    && matches!(
                        usage.target(),
                        ResolvedTarget::Source {
                            class: DeclarationClass::Function,
                            ..
                        }
                    )
            }));
            assert!(resolved.lexical_uses().iter().any(|usage| {
                usage.spelling() == local
                    && matches!(
                        usage.target(),
                        ResolvedTarget::Source {
                            class: DeclarationClass::Value,
                            ..
                        }
                    )
            }));
        });
    }
}

#[test]
fn one_name_mutation_changes_a_complete_call_into_an_op1_rejection() {
    let accepted = br#"fn helper() -> result: own unit pure {
}

fn probe() -> result: own unit pure {
  helper();
}
"#;
    with_one_resolution(accepted, |outcome| {
        assert!(matches!(outcome, ResolutionOutcome::Complete(_)));
    });

    let mutated = br#"fn helper() -> result: own unit pure {
}

fn probe() -> result: own unit pure {
  missing();
}
"#;
    with_one_resolution(mutated, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("mutated call must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Op1);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, .. } if spelling == "missing"
        ));
    });
}

#[test]
fn diagnostics_ignore_logical_paths_and_repeat_byte_for_byte() {
    let source = b"fn probe() -> result: own unit pure {\n  missing();\n}\n";
    let issue = |path: &str| -> ResolutionIssue {
        with_resolution(&[SourceInput::new(path, source)], |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("missing call must reject: {outcome:?}");
            };
            issue
        })
    };
    assert_eq!(issue("first.wf"), issue("renamed/location.wf"));
    assert_eq!(issue("first.wf"), issue("first.wf"));
}

#[test]
fn source_record_order_controls_const_visibility_but_paths_create_no_namespace() {
    let use_source = SourceInput::new("consumer/first.wf", b"const first: i32 = second;\n");
    let declaration_source = SourceInput::new("library/second.wf", b"const second: i32 = 2_i32;\n");
    with_resolution(&[use_source, declaration_source], |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("later-source const must be invisible: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Const2);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::InvisibleUse { spelling, .. } if spelling == "second"
        ));
    });

    with_resolution(&[declaration_source, use_source], |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "earlier source record must make the const visible: {outcome:?}"
        );
    });

    let first = SourceInput::new("left/name.wf", b"fn same() -> result: own unit pure {\n}\n");
    let second = SourceInput::new(
        "right/name.wf",
        b"fn same() -> result: own unit pure {\n}\n",
    );
    with_resolution(&[first, second], |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("logical paths must not create function namespaces: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
    });
}

#[test]
fn every_distinct_op1_family_resolves_through_the_normal_callee_path() {
    // The callee path now covers the families that keep a name; the
    // twenty-seven operator-spelled rows — twenty-one arithmetic since v0.23
    // and the six integer comparisons since v0.41 — reach their family by
    // operator token and are covered by
    // `a_respelled_family_produces_no_lexical_use_at_all`. The two halves are
    // counted here so that a family silently leaving one for the other cannot
    // pass unnoticed.
    let named: Vec<_> = OPERATION_FAMILIES
        .iter()
        .enumerate()
        .filter(|(_, spelling)| !is_operator_family(spelling))
        .collect();
    assert_eq!(named.len(), OPERATION_FAMILIES.len() - 27);

    let mut source = String::from("fn probe() -> result: own unit pure {\n");
    for (_, operation) in &named {
        source.push_str("  ");
        source.push_str(operation);
        source.push_str("(1_i32);\n");
    }
    source.push_str("}\n");

    with_one_resolution(source.as_bytes(), |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("every named OP-1 family must resolve: {outcome:?}");
        };
        let operations: Vec<_> = resolved
            .lexical_uses()
            .iter()
            .filter(|usage| matches!(usage.target(), ResolvedTarget::Operation(_)))
            .collect();
        assert_eq!(operations.len(), named.len());
        // The identity check is against the family's own position in the
        // inventory, not against the order it happens to be written in, so
        // filtering the source cannot make the ordinals agree by accident.
        for (usage, (index, spelling)) in operations.into_iter().zip(named.iter()) {
            let ResolvedTarget::Operation(id) = usage.target() else {
                unreachable!();
            };
            assert_eq!(usize::from(id.ordinal()), *index);
            assert_eq!(usage.spelling(), **spelling);
        }
    });
}

// v0.58 retires SYS-1/SYS-2/SYS-3, QUAL-1 and SYS-5 ordinal/release-table
// assertions. PRE-1 now supplies ordinary declarations; these witnesses retain
// visibility, collision and callable-binding coverage without an external domain.
#[test]
fn parsed_prelude_declarations_are_ordinary_visible_targets() {
    let source = b"fn inspect(args: &Args) -> result: own u64 reads(args) {\n  return args_count(args: args);\n}\n";
    with_resolution_sources(
        &[SourceInput::new("ordinary.wf", source)],
        true,
        |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("ordinary prelude declaration resolution: {outcome:?}");
            };
            for (spelling, class) in [
                ("Args", DeclarationClass::NominalType),
                ("args_count", DeclarationClass::Function),
            ] {
                assert!(resolved.lexical_uses().iter().any(|usage| {
                usage.origin().coordinate().source().ordinal() == 0 && usage.spelling() == spelling
                    && matches!(usage.target(), ResolvedTarget::Source { class: actual, .. } if actual == class)
            }));
            }
        },
    );
}

#[test]
fn ordinary_prelude_names_cannot_be_shadowed_and_opaque_types_have_no_constructor() {
    for source in [
        "struct HostString {\n}\n",
        "enum Collision {\n  NotFound();\n}\n",
        "fn helper() -> result: own unit pure {\n  let args_count = 0_u64;\n  return unit;\n}\n",
    ] {
        with_resolution_sources(
            &[SourceInput::new("collision.wf", source.as_bytes())],
            true,
            |outcome| {
                let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                    panic!("PRE-1 collision must reject: {outcome:?}");
                };
                assert!(matches!(
                    issue.kind(),
                    ResolutionIssueKind::DeclarationCollision { .. }
                ));
            },
        );
    }
    let source = b"fn fabricate() -> result: own HostString pure {\n  return HostString();\n}\n";
    with_resolution_sources(&[SourceInput::new("opaque.wf", source)], true, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an ordinary opaque nominal has no constructor: {outcome:?}");
        };
        assert!(
            matches!(issue.kind(), ResolutionIssueKind::UnresolvedUse { spelling, .. } if spelling == "HostString")
        );
    });
}

#[test]
fn an_ordinary_prelude_signature_is_eligible_for_an_actual_member() {
    let source = b"formal Counter {\n  fn count(args: &Args) -> result: own u64 reads(args);\n}\n\nactual Selected : Counter {\n  count = args_count;\n}\n";
    with_resolution_sources(&[SourceInput::new("actual.wf", source)], true, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("FN-4 admits ordinary declarations: {outcome:?}");
        };
        assert!(resolved.lexical_uses().iter().any(|usage| {
            usage.origin().coordinate().source().ordinal() == 0
                && usage.spelling() == "args_count"
                && matches!(
                    usage.target(),
                    ResolvedTarget::Source {
                        class: DeclarationClass::Function,
                        ..
                    }
                )
        }));
    });
}

#[test]
fn supplied_signature_locals_do_not_capture_writer_global_names() {
    // PRE-1 is an outer ordinary declaration environment. Its local `f` and
    // `input` parameter names cannot reserve those names in the writer unit.
    let source = b"const input: u64 = 7_u64;\n\nfn f() -> result: own u64 pure {\n  return input;\n}\n";
    with_resolution_sources(&[SourceInput::new("ordinary.wf", source)], true, |outcome| {
        assert!(matches!(outcome, ResolutionOutcome::Complete(_)), "{outcome:?}");
    });
}
