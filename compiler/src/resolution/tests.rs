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
    DependentDeclarationRole, LexicalUseRecord, LexicalUseRole, PostconditionSelectorClass,
    ReservedDeclarationRole, ResolutionIssue, ResolutionIssueKind, ResolutionOutcome,
    ResolutionRule, ResolvedTarget, ScopeKind, resolve,
};

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 16,
    max_logical_path_bytes: 128,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_binding_bytes: 1_048_576,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 16,
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
    max_sources: 16,
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
    let Ok(bundle) = SourceBundle::with_limits(inputs, SOURCE_LIMITS) else {
        panic!("resolver test bundle must be valid");
    };
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
    let source = br#"fn probe() -> result: own unit pure {
  let values = array_new<i32, 4>(0_i32);
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
  define reflexive = ieq(value, value);
  requires reflexive;
  ensures ieq(result, value);
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
        let candidate = postcondition
            .plain_candidate
            .as_ref()
            .expect("plain selector candidate");
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
    for field in ["value", "hostile"] {
        let source = format!(
            "fn selected(value: own i32) -> result: own Result<i32, i32> pure contract {{\n  ensures when Ok({field}: result): ieq(result, value);\n}} {{\n  return Ok<i32, i32>(value: value);\n}}\n"
        );
        with_one_resolution(source.as_bytes(), |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("variant postcondition surface must resolve: {outcome:?}");
            };
            let [postcondition] = resolved.postconditions() else {
                panic!("one private postcondition record is required");
            };
            assert_eq!(postcondition.class, PostconditionSelectorClass::Variant);
            assert!(postcondition.plain_candidate.is_none());
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
    let plain = br#"fn plain(value: own i32) -> ilt: own i32 pure contract {
  ensures ieq(ilt, value);
} {
  return value;
}
"#;
    let variant = br#"fn variant(value: own i32) -> result: own Result<i32, i32> pure contract {
  ensures when Ok(value: ilt): ieq(ilt, value);
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
                    inventory_ordinal: 24,
                    ..
                } if spelling == "ilt" && *declaration_role == expected_role
            ));
        });
    }
}

#[test]
fn postcondition_lookup_waits_for_selector_admission_and_live_conflicts_are_retained() {
    let unresolved = br#"fn unresolved() -> result: own unit pure contract {
  ensures ieq(result, missing);
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
  ensures ieq(result, result);
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
        let candidate = postcondition
            .plain_candidate
            .as_ref()
            .expect("plain selector candidate");
        assert_eq!(candidate.live_conflicts.len(), 1);
        assert!(candidate.later_local_collision.is_none());
        assert_eq!(postcondition.provisional_uses.len(), 1);
        assert_eq!(postcondition.provisional_uses[0].spelling(), "ieq");
        assert_eq!(postcondition.selector_uses.len(), 2);
        assert!(postcondition.entry_resolution_issue.is_none());
        assert!(postcondition.entry_inventory_issue.is_none());
    });
}

#[test]
fn invalid_ensures_local_cannot_poison_an_ordinary_body_lookup() {
    let source = br#"fn poisoned(value: own i32) -> result: own i32 pure contract {
  define ilt = ieq(value, value);
  ensures ieq(result, value);
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
            } if spelling == "ilt"
        ));
    });
}

#[test]
fn unresolved_variant_selector_keeps_its_lookup_verdict_before_entry_inventory() {
    let source =
        br#"fn unresolved(value: own i32) -> result: own Result<i32, Overflow> pure contract {
  ensures when Missing(value: result): ieq(result, value);
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
  requires ieq(pre, value);
  ensures ieq(result, pre);
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
  ensures ieq(result, post);
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

#[test]
fn const_generics_remain_outside_the_ordinary_place_base_domain() {
    let ordinary = br#"fn value<const n: u64>() -> result: own u64 pure {
  return n;
}

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    let postcondition = br#"fn value<const n: u64>() -> result: own u64 pure contract {
  ensures ieq(result, result);
} {
  return n;
}

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    for source in [ordinary.as_slice(), postcondition.as_slice()] {
        with_one_resolution(source, |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a const generic must not become an ordinary pbase: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Type5);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::UnresolvedUse {
                    spelling,
                    role: LexicalUseRole::PlaceBase,
                    admissible,
                    available,
                } if spelling == "n"
                    && !admissible.contains(&DeclarationClass::ConstGeneric)
                    && available.contains(&DeclarationClass::ConstGeneric)
            ));
        });
    }
}

#[test]
fn the_command_kind_gates_only_the_system_admission_decision_in_resolution() {
    // A unit with no `program_kind` child is not kind-declaring, so nothing
    // about the entry-form grammar changes its ordinary resolution, and the
    // system domain contributes no entry to it [SYS-3].
    let ordinary = b"fn probe() -> result: own unit pure {\n  return unit;\n}\n";
    with_one_resolution(ordinary, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("an ordinary internal function must resolve unchanged: {outcome:?}");
        };
        assert!(resolved.system_declarations().is_empty());
    });

    // One `program_kind` child makes the unit kind-declaring, which admits
    // the complete SYS-2 inventory as a third declaration source [SYS-1]:
    // the entry's system input and result types resolve to system targets.
    let kind_declaring =
        b"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    with_one_resolution(kind_declaring, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("a kind-declaring unit must resolve system names: {outcome:?}");
        };
        assert_eq!(resolved.system_declarations().len(), 199);
        for (spelling, ordinal) in [("Args", 0), ("ExitStatus", 6)] {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| usage.spelling() == spelling)
                .unwrap_or_else(|| panic!("missing system type use {spelling}"));
            assert_eq!(usage.role(), LexicalUseRole::Type);
            assert!(matches!(
                usage.target(),
                ResolvedTarget::System(id) if id.ordinal() == ordinal
            ));
        }
    });
}

#[test]
fn fn8_admission_precedes_the_system_admission_decision() {
    // DIAG-1 fixes the stage order: only complete unit-wide FN-8 admission
    // permits the SYS-3 system-admission decision. The FN-8 rejection must
    // therefore win in a kind-declaring unit before any system name enters
    // inventory or lookup.
    let source = br#"fn guarded(value: own i32) -> result: own i32 pure contract {
  define unresolved = missing;
} {
  return value;
}

command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the FN-8 defect must outrank system admission: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Fn8);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ContractShape(_)
        ));
    });

    // The same kind-declaring unit with an admitted internal requirement
    // reaches the system-admission decision and resolves with the domain
    // admitted. The command entry itself carries no contract [FN-7].
    let admitted = br#"fn guarded(value: own i32) -> result: own i32 pure contract {
  requires ieq(value, value);
} {
  return value;
}

command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_one_resolution(admitted, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("an admitted requires block must reach the SYS-3 decision: {outcome:?}");
        };
        assert_eq!(resolved.system_declarations().len(), 199);
    });
}

#[test]
fn a_kind_declaring_unit_resolves_the_complete_system_lookup_inventory() {
    // Every system nominal type as a `type` TYPEID use, every operation as an
    // IDENT callee, one constructor in construct position, and the
    // `ReadOutcome` variants in arm position, with deterministic [SYS-2]
    // preorder ordinals throughout. Resolution fixes callee targets only;
    // argument-name checking against the [SYS-2] parameter lists is the
    // later typed stage; this fixture nevertheless spells every current
    // parameter name so catalog surface changes remain visible here.
    let source = br#"command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}

fn types(a: own Args, b: own HostString, c: own RelativePath, d: own DirectoryRead, e: own ReadFile, f: own Output, g: own ExitStatus, h: own ArgError, i: own Utf8Error, j: own CopyError, k: own Utf8CopyError, l: own PathError, m: own ReadOutcome, n: own IoError, o: own DirectoryList, p: own ListOutcome) -> result: own unit pure {
  return unit;
}

fn calls(x: own u64) -> result: own unit pure {
  args_count(args: x);
  arg_get(args: x, position: x);
  host_bytes_len(value: x);
  host_copy_bytes(value: x, destination: x, start: x, end: x);
  host_utf8_len(value: x);
  host_copy_utf8(value: x, destination: x, start: x, end: x);
  relative_path(value: x);
  open_read(root: x, path: x);
  read_once(file: x, destination: x, start: x, end: x);
  write_once(output: x, source: x, start: x, end: x);
  open_directory(root: x, name: x, start: x, end: x);
  open_list(directory: x);
  list_once(list: x, destination: x, start: x, end: x);
  open_file(root: x, name: x, start: x, end: x);
  return unit;
}

fn outcomes(m: own ReadOutcome) -> result: own unit pure {
  let failed = NotFound(code: 1_u32, origin: 0_u8);
  match m {
    ReadBytes(next: got) => {
      return unit;
    }
    ReadEnd() => {
      return unit;
    }
    ReadFailed(error: cause) => {
      return unit;
    }
  }
}

fn list_outcomes(m: own ListOutcome) -> result: own unit pure {
  match m {
    ListBytes(next: got, entries: count) => {
      return unit;
    }
    ListEnd() => {
      return unit;
    }
    ListFailed(error: cause) => {
      return unit;
    }
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("the complete system fixture must resolve: {outcome:?}");
        };
        let system_target = |usage: &LexicalUseRecord| match usage.target() {
            ResolvedTarget::System(id) => Some(id.ordinal()),
            _ => None,
        };
        let expect = |role: LexicalUseRole, spelling: &str, ordinal: u8| {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| usage.role() == role && usage.spelling() == spelling)
                .unwrap_or_else(|| panic!("missing system use {spelling}"));
            assert_eq!(
                system_target(usage),
                Some(ordinal),
                "wrong target for {spelling}"
            );
        };
        for (spelling, ordinal) in [
            ("Args", 0),
            ("HostString", 1),
            ("RelativePath", 2),
            ("DirectoryRead", 3),
            ("ReadFile", 4),
            ("Output", 5),
            ("ExitStatus", 6),
            ("ArgError", 7),
            ("Utf8Error", 8),
            ("CopyError", 9),
            ("Utf8CopyError", 10),
            ("PathError", 11),
            ("ReadOutcome", 12),
            ("IoError", 13),
            ("DirectoryList", 14),
            ("ListOutcome", 15),
        ] {
            expect(LexicalUseRole::Type, spelling, ordinal);
        }
        for (spelling, ordinal) in [
            ("args_count", 125),
            ("arg_get", 128),
            ("host_bytes_len", 132),
            ("host_copy_bytes", 135),
            ("host_utf8_len", 142),
            ("host_copy_utf8", 145),
            ("relative_path", 152),
            ("open_read", 154),
            ("read_once", 159),
            ("write_once", 166),
            ("exit_status", 173),
            ("open_directory", 175),
            ("open_list", 182),
            ("list_once", 185),
            ("open_file", 192),
        ] {
            expect(LexicalUseRole::IdentifierCallee, spelling, ordinal);
        }
        expect(LexicalUseRole::Construct, "NotFound", 29);
        expect(LexicalUseRole::ArmVariant, "ReadBytes", 24);
        expect(LexicalUseRole::ArmVariant, "ReadEnd", 26);
        expect(LexicalUseRole::ArmVariant, "ReadFailed", 27);
        expect(LexicalUseRole::ArmVariant, "ListBytes", 119);
        expect(LexicalUseRole::ArmVariant, "ListEnd", 122);
        expect(LexicalUseRole::ArmVariant, "ListFailed", 123);
    });
}

#[test]
fn a_system_unadmitted_unit_sees_system_spellings_as_ordinary_undeclared_names() {
    // [SYS-3]: in a unit that is not kind-declaring the system domain
    // contributes no entry, so a system operation spelling is an ordinary
    // undeclared callee decided by the ordinary lexical-use ranks.
    let callee = br#"fn probe() -> result: own unit pure {
  let x = 0_u64;
  args_count(args: x);
  return unit;
}
"#;
    with_one_resolution(callee, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unadmitted system callee must be undeclared: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Op1);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, available, .. }
                if spelling == "args_count" && available.is_empty()
        ));
    });

    // The same holds in the nominal-type domain.
    let nominal = br#"fn consume(value: own HostString) -> result: own unit pure {
  return unit;
}

fn probe() -> result: own unit pure {
  return unit;
}
"#;
    with_one_resolution(nominal, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unadmitted system type must be undeclared: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type5);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, .. } if spelling == "HostString"
        ));
    });
}

#[test]
fn system_lookalike_declarations_in_an_unadmitted_unit_are_ordinary() {
    // [SYS-3]: a source declaration in a system-unadmitted unit may use any
    // system spelling; it collides with nothing and every use resolves to it
    // under the ordinary domains. No source property makes it a system
    // entity.
    let source = br#"struct HostString {
}

enum Outcome {
  ReadEnd();
}

fn args_count(args: own u64) -> result: own u64 pure {
  return args;
}

fn keeper(value: own HostString) -> result: own unit pure {
  return unit;
}

fn probe() -> result: own unit pure {
  let x = args_count(args: 0_u64);
  let s = HostString();
  match ReadEnd() {
    ReadEnd() => {
      return unit;
    }
  }
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("lookalike declarations must be ordinary: {outcome:?}");
        };
        assert!(resolved.system_declarations().is_empty());
        for (role, spelling) in [
            (LexicalUseRole::IdentifierCallee, "args_count"),
            (LexicalUseRole::Type, "HostString"),
            (LexicalUseRole::Construct, "HostString"),
            (LexicalUseRole::ArmVariant, "ReadEnd"),
        ] {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| usage.role() == role && usage.spelling() == spelling)
                .unwrap_or_else(|| panic!("missing lookalike use {spelling}"));
            assert!(
                matches!(usage.target(), ResolvedTarget::Source { .. }),
                "lookalike use {spelling} must resolve to source: {usage:?}"
            );
        }
    });
}

#[test]
fn system_collisions_reject_deterministically_in_both_directions() {
    // [DIAG-1] rank 5: inside a kind-declaring unit a source declaration
    // whose spelling equals a system entry's spelling in the same domain is a
    // deterministic rejection at that source declaration event — before the
    // entry declaration and after it alike — and neither name resolves.
    let entry = "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    let lookalike = "fn args_count(args: own u64) -> result: own u64 pure {\n  return args;\n}\n";
    for source in [
        format!("{lookalike}\n{entry}"),
        format!("{entry}\n{lookalike}"),
    ] {
        with_one_resolution(source.as_bytes(), |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("the system collision must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Type6);
            let ResolutionIssueKind::DeclarationCollision {
                spelling,
                conflicts,
            } = issue.kind()
            else {
                panic!("expected a declaration collision: {issue:?}");
            };
            assert_eq!(spelling, "args_count");
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].domain(), DeclarationDomain::LexicalIdentifier);
            assert_eq!(conflicts[0].class(), DeclarationClass::Function);
            assert!(matches!(
                conflicts[0].origin(),
                DeclarationOrigin::System(id) if id.ordinal() == 125
            ));
        });
    }
}

#[test]
fn system_collisions_cover_every_contributed_domain_and_nested_scopes() {
    let entry = "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

    // Nominal-type domain: a struct reusing an opaque-type spelling. The
    // struct's constructor entry collides with nothing because an opaque
    // type contributes no constructor, so exactly one conflict is reported.
    let nominal = format!("{entry}\nstruct HostString {{\n}}\n");
    with_one_resolution(nominal.as_bytes(), |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the nominal system collision must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        let ResolutionIssueKind::DeclarationCollision {
            spelling,
            conflicts,
        } = issue.kind()
        else {
            panic!("expected a declaration collision: {issue:?}");
        };
        assert_eq!(spelling, "HostString");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].domain(), DeclarationDomain::NominalType);
        assert!(matches!(
            conflicts[0].origin(),
            DeclarationOrigin::System(id) if id.ordinal() == 1
        ));
    });

    // Constructor domain: a source enum variant reusing a system constructor
    // spelling collides even though its enum nominal is fresh.
    let variant = format!("{entry}\nenum Mine {{\n  ReadEnd();\n}}\n");
    with_one_resolution(variant.as_bytes(), |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the constructor system collision must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        let ResolutionIssueKind::DeclarationCollision {
            spelling,
            conflicts,
        } = issue.kind()
        else {
            panic!("expected a declaration collision: {issue:?}");
        };
        assert_eq!(spelling, "ReadEnd");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].domain(), DeclarationDomain::Constructor);
        assert!(matches!(
            conflicts[0].origin(),
            DeclarationOrigin::System(id) if id.ordinal() == 26
        ));
    });

    // A nested declaration collides at rank 5 exactly like a root one
    // ([SYS-1]: at the compilation root and in every nested scope alike);
    // this is a rejection, never a shadow of the system entry.
    let nested = "command fn main() -> status: own ExitStatus pure {\n  let host_bytes_len = 0_u64;\n  return exit_status(code: 0_u8);\n}\n";
    with_one_resolution(nested.as_bytes(), |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the nested system collision must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        let ResolutionIssueKind::DeclarationCollision {
            spelling,
            conflicts,
        } = issue.kind()
        else {
            panic!("expected a declaration collision: {issue:?}");
        };
        assert_eq!(spelling, "host_bytes_len");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].domain(), DeclarationDomain::LexicalIdentifier);
        assert!(matches!(
            conflicts[0].origin(),
            DeclarationOrigin::System(id) if id.ordinal() == 132
        ));
    });
}

#[test]
fn a_prelude_collision_keeps_rank_four_in_a_system_admitted_unit() {
    // [DIAG-1] rank 4 precedes rank 5 at one event: a PRE-1 collision in a
    // kind-declaring unit reports only its PRE-1 conflicts.
    let source = "command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n\nstruct Overflow {\n}\n";
    with_one_resolution(source.as_bytes(), |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the prelude collision must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        let ResolutionIssueKind::DeclarationCollision {
            spelling,
            conflicts,
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
fn a_system_operation_never_satisfies_a_conformance_binding() {
    // [SYS-2]: a system operation is not the right IDENT of an FN-3
    // `fn_bind`; a conformance binds only a top-level source function. The
    // visible system entry still surfaces through the available classes.
    let source = br#"command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}

contract Task {
  fn run(value: own u64) -> result: own u64 pure;
}

conform u64: Task {
  run = args_count;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a system operation must not bind a contract member: {outcome:?}");
        };
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::UnresolvedUse { spelling, available, .. }
                if spelling == "args_count" && available.contains(&DeclarationClass::Function)
        ));
    });
}

#[test]
fn system_resolution_is_deterministic_across_repeated_runs_and_paths() {
    let source =
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    let targets = |path: &str| -> Vec<(String, u8)> {
        with_resolution(&[SourceInput::new(path, source)], |outcome| {
            let ResolutionOutcome::Complete(resolved) = outcome else {
                panic!("the deterministic fixture must resolve: {outcome:?}");
            };
            resolved
                .lexical_uses()
                .iter()
                .filter_map(|usage| match usage.target() {
                    ResolvedTarget::System(id) => Some((usage.spelling().to_owned(), id.ordinal())),
                    _ => None,
                })
                .collect()
        })
    };
    let first = targets("first.wf");
    assert_eq!(
        first,
        vec![
            ("ExitStatus".to_owned(), 6),
            ("exit_status".to_owned(), 173)
        ]
    );
    assert_eq!(first, targets("first.wf"));
    assert_eq!(first, targets("renamed/location.wf"));
}

#[test]
fn requires_locals_do_not_escape_into_the_function_body() {
    let source = br#"fn guarded() -> result: own unit pure contract {
  define condition = 1_i32;
  requires ieq(condition, condition);
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
    with_one_resolution(b"fn ilt() -> result: own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("operation name declaration must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                inventory_ordinal: 24,
                ..
            } if spelling == "ilt"
        ));
    });
}

/// OP-1 (iii): reservation is derived from the op column, so every one of the
/// six comparisons is reserved again after the owner cancelled their infix
/// spellings. The test above pins `ilt`, which never moved; this one pins the
/// four that did, because they were declarable for exactly one revision and a
/// program written against it must stop compiling.
#[test]
fn every_comparison_name_is_reserved_from_source_declarations() {
    for spelling in ["ieq", "ine", "ile", "ige"] {
        let source = format!("fn {spelling}() -> result: own unit pure {{\n}}\n");
        with_one_resolution(source.as_bytes(), |outcome| {
            let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a comparison name declaration must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), ResolutionRule::Form3);
            assert!(matches!(
                issue.kind(),
                ResolutionIssueKind::ReservedName { spelling: declared, .. }
                    if declared == spelling
            ));
        });
    }
}

#[test]
fn region_names_are_unique_across_the_complete_function() {
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
  for @range index in 0_u64..limit {
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
fn counted_range_binder_is_invisible_in_both_endpoints_and_after_the_loop() {
    for source in [
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range index in index..limit {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range index in 0_u64..index {
    break @range;
  }
  return unit;
}
"#
        .as_slice(),
        br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range index in 0_u64..limit {
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
fn counted_range_label_is_non_enclosing_after_the_loop() {
    let source = br#"fn probe(limit: own u64) -> result: own unit pure {
  for @range index in 0_u64..limit {
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
  for @range ilt in 0_u64..limit {
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
            } if spelling == "ilt"
        ));
    });
}

#[test]
fn counted_range_scope_rejects_live_shadowing_and_allows_expired_reuse() {
    let live_outer = br#"fn probe(limit: own u64) -> result: own unit pure {
  let index = 0_u64;
  for @range index in 0_u64..limit {
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
  for @outer index in 0_u64..limit {
    for @inner index in 0_u64..limit {
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
  for @outer outer_index in 0_u64..limit {
    for @inner inner_index in outer_index..limit {
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
  for @range index in 0_u64..limit {
    break @range;
  }
  for @range index in 0_u64..limit {
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
/// The split is smaller than it was: after the owner cancelled the infix
/// comparisons only arithmetic crosses this line, so `ieq` joins `imin` on the
/// named side and is a second control rather than a second subject.
#[test]
fn a_respelled_family_produces_no_lexical_use_at_all() {
    let source = br#"fn probe() -> result: own unit pure {
  let sum = 1_i32 +wrap 2_i32;
  let equal = ieq(sum, 3_i32);
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
        // The two named rows in the same function are the control: they prove
        // the filter finds operation uses at all, so the infix row being
        // absent is the property and not an empty search.
        assert_eq!(operations, ["ieq", "imin"]);
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
/// that A3 deletes, so it now rides a signature-borne `slice<'v, i32>`, which
/// [TYPE-5] keeps written. `OperationCallee` is the OPNAME form specifically
/// (`roles.rs` keys it on `TerminalPredicate::OperationName`), and the
/// fixture's only operation call was `iadd.wrap`, one of the rows [OP-7]
/// respelled — an operator token is never a callee, so a respelled row
/// produces no lexical use at all. It now rides `iabs.checked`, a dotted row
/// that keeps its operation-name route.
#[test]
fn complete_role_fixture_materializes_every_d_u_and_x_family() {
    let source = br#"contract Bound {
  fn member['sig](value: &'sig i32) -> result: own i32 reads('sig);
  law identity(member, 0_i32);
}

contract Numeric<T: Int> {
  fn zero() -> result: own T pure;
  law identity(zero, 0_T);
}

struct Package<T: Bound, const n: i32> {
  items: array<T, n>;
}

enum Choice<T> {
  Absent();
  Present(value: T);
}

const one: i32 = 1_i32;

const two: i32 = one;

fn implementation(value: own i32) -> result: own i32 pure {
  return value;
}

conform Package<i32, one>: Bound {
  member = implementation;
}

fn user<T: Bound, const n: i32>['call](arg: &'call T) -> result: &'call T reads('call) {
  return arg;
}

fn viewer['v](values: own slice<'v, i32>) -> result: own unit reads('v) {
  return unit;
}

fn numeric<T: Int>() -> result: own T pure {
  return 0_T;
}

fn probe() -> result: own unit traps {
  let ordinary = 1_i32 +wrap two;
  let smaller = iabs.checked(ordinary);
  let made = Package<i32, one>(items: ordinary);
  set deref(made).items = ordinary;
  region 'r {
    let borrowed = &'r ordinary;
    let called = user<i32, 'r, one>(arg: borrowed);
    let view = move called;
    claim bad: ieq(ordinary, two) because "bad";
  }
  loop @done {
    break @done;
  }
  for @counted index in 0_u64..2_u64 {
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
            DeclarationRole::Contract,
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
            DependentDeclarationRole::ContractMember,
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
            LexicalUseRole::ConformanceContract,
            LexicalUseRole::Construct,
            LexicalUseRole::ArmVariant,
            LexicalUseRole::TypeRegion,
            LexicalUseRole::ModeRegion,
            LexicalUseRole::TypeArgumentRegion,
            LexicalUseRole::EffectRegion,
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
            DeferredUseRole::ContractBinding,
            DeferredUseRole::LawName,
            DeferredUseRole::LawArgument,
        ] {
            assert!(
                deferred_roles.contains(&role),
                "missing deferred role {role:?}"
            );
        }

        let shared_argument = resolved
            .deferred_uses()
            .iter()
            .find(|usage| usage.spelling() == "0_T")
            .expect("generic law argument must be retained");
        let shared_suffix = resolved
            .lexical_uses()
            .iter()
            .find(|usage| {
                usage.role() == LexicalUseRole::GenericNumericSuffix
                    && usage.origin().node() == shared_argument.origin().node()
            })
            .expect("generic law argument suffix must resolve");
        assert_eq!(
            shared_argument.origin().role_ordinal(),
            shared_suffix.origin().role_ordinal()
        );
        assert_eq!(shared_argument.origin().subtoken_ordinal(), 0);
        assert_eq!(shared_suffix.origin().subtoken_ordinal(), 1);
    });
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
fn approved_duplicate_main_conformance_case_is_type6() {
    let source = include_bytes!("../../../tests/conformance/cases/fn7-neg-two-mains.wf");
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the later main declaration must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::DeclarationCollision { spelling, conflicts }
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
fn sibling_contract_signatures_do_not_share_region_parameters() {
    let source = br#"contract Separate {
  fn first['r](value: &'r i32) -> result: own unit pure;
  fn second() -> result: own slice<'r, i32> pure;
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
    let sibling_branches = br#"fn get(pick: own Bool) -> result: own unit traps {
  if pick {
    let inside = 1_u64;
    claim left: ieq(inside, 1_u64) because "left";
  } else {
    let inside = 2_u64;
    claim right: ieq(inside, 2_u64) because "right";
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

fn get(pick: own Pick) -> result: own unit traps {
  match pick {
    Left() => {
      let inside = 1_u64;
      claim left: ieq(inside, 1_u64) because "left";
    }
    Right() => {
      let inside = 2_u64;
      claim right: ieq(inside, 2_u64) because "right";
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

    let expired_then_enclosing = br#"fn get(pick: own Bool) -> result: own unit traps {
  if pick {
    let offset = 0_u64;
    claim inner: ieq(offset, 0_u64) because "inner";
  }
  let offset = 1_u64;
  claim outer: ieq(offset, 1_u64) because "outer";
  return unit;
}
"#;
    with_one_resolution(expired_then_enclosing, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "an expired branch scope may not block a later enclosing binder: {outcome:?}"
        );
    });

    let live_shadow = br#"fn get(pick: own Bool) -> result: own unit traps {
  let offset = 0_u64;
  if pick {
    let offset = 1_u64;
    claim inner_2: ieq(offset, 1_u64) because "inner";
  }
  claim outer_2: ieq(offset, 0_u64) because "outer";
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

fn ilt() -> result: own unit pure {
}
"#;
    with_one_resolution(later_inventory_error, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("inventory must reject before lookup: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });

    let later_fn8_error = br#"fn ilt() -> result: own unit pure {
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

fn ilt() -> result: own unit pure {
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
    // The callee path now covers the families that keep a name; the twenty-one
    // operator-spelled rows reach their family by operator token and are
    // covered by `a_respelled_family_produces_no_lexical_use_at_all`. The two
    // halves are counted here so that a family silently leaving one for the
    // other cannot pass unnoticed — which is what the owner's cancellation of
    // the infix comparisons did, moving four families back across this line.
    let named: Vec<_> = OPERATION_FAMILIES
        .iter()
        .enumerate()
        .filter(|(_, spelling)| !is_operator_family(spelling))
        .collect();
    assert_eq!(named.len(), OPERATION_FAMILIES.len() - 21);

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

#[test]
fn system_index_helpers_agree_with_the_preorder_entity_map() {
    // The index helpers derive table positions arithmetically from the
    // [SYS-2] preorder; this pins them to `system_entity`, the authoritative
    // ordinal-to-entity map, across every one of the 167 records.
    use super::SystemDeclarationId;
    use super::catalog::{
        SYSTEM_NOMINALS, SystemEntity, system_constructor_declaration, system_constructor_index,
        system_constructors, system_entity, system_nominal_index, system_nominals,
        system_operation_index, system_operations, system_release_row,
    };

    // Every inventory state: a candidate's extra nominal types shift every
    // constructor and operation ordinal, so the helpers must agree with the
    // entity map under each state separately.
    for surface in [
        crate::Inventory::Base,
        crate::Inventory::Traversal,
        crate::Inventory::OpenByName,
    ] {
        let mut nominals = 0_usize;
        let mut constructors = 0_usize;
        let mut operations = 0_usize;
        for ordinal in 0..=u8::MAX {
            let id = SystemDeclarationId::new(ordinal);
            match system_entity(id, surface) {
                Some(SystemEntity::Nominal(nominal)) => {
                    let index = system_nominal_index(id, surface).expect("nominal index");
                    assert_eq!(
                        system_nominals(surface)[usize::from(index)].spelling,
                        nominal.spelling
                    );
                    assert!(system_constructor_index(id, surface).is_none());
                    assert!(system_operation_index(id, surface).is_none());
                    nominals += 1;
                }
                Some(SystemEntity::Constructor(constructor)) => {
                    let index = system_constructor_index(id, surface).expect("constructor index");
                    assert_eq!(
                        system_constructors(surface)[usize::from(index)].spelling,
                        constructor.spelling
                    );
                    assert_eq!(system_constructor_declaration(index, surface), Some(id));
                    assert!(system_nominal_index(id, surface).is_none());
                    assert!(system_operation_index(id, surface).is_none());
                    constructors += 1;
                }
                Some(SystemEntity::Operation(operation)) => {
                    let index = system_operation_index(id, surface).expect("operation index");
                    assert_eq!(
                        system_operations(surface)[usize::from(index)].spelling,
                        operation.spelling
                    );
                    assert!(system_nominal_index(id, surface).is_none());
                    assert!(system_constructor_index(id, surface).is_none());
                    operations += 1;
                }
                None => {
                    assert!(system_constructor_index(id, surface).is_none());
                    assert!(system_operation_index(id, surface).is_none());
                }
            }
        }
        assert_eq!(nominals, system_nominals(surface).len());
        assert_eq!(constructors, system_constructors(surface).len());
        assert_eq!(operations, system_operations(surface).len());
    }

    // The [SYS-5] release table: exactly DirectoryRead, ReadFile, and the
    // candidate DirectoryList release with `external, blocks`; every other
    // system nominal's row is empty.
    for (index, nominal) in SYSTEM_NOMINALS.iter().enumerate() {
        let index = u8::try_from(index).expect("nominal table fits u8");
        let row = system_release_row(index);
        let expected = matches!(
            nominal.spelling,
            "DirectoryRead" | "ReadFile" | "DirectoryList"
        );
        assert_eq!(row.external, expected, "external for {}", nominal.spelling);
        assert_eq!(row.blocks, expected, "blocks for {}", nominal.spelling);
    }
}

#[test]
fn the_system_resource_contracts_equal_the_release_and_backing_tables() {
    use super::catalog::{
        SYSTEM_NOMINALS, SystemReleaseAction, SystemResourceBacking, SystemResourceType,
        system_release_row, system_resource_contract,
    };

    // [SYS-5]'s release table and [HOST-3]'s backing rule, keyed by the
    // [SYS-2] nominal spelling so a reordered inventory cannot silently move
    // a contract onto another type. The seven outcome enums have no release
    // action and take no row in the table, so they carry no contract.
    let expected = [
        (
            "Args",
            SystemResourceType::Args,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        (
            "HostString",
            SystemResourceType::HostString,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::CommandLifetimeLease,
        ),
        (
            "RelativePath",
            SystemResourceType::RelativePath,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::CommandLifetimeLease,
        ),
        (
            "DirectoryRead",
            SystemResourceType::DirectoryRead,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        (
            "ReadFile",
            SystemResourceType::ReadFile,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
        (
            "Output",
            SystemResourceType::Output,
            SystemReleaseAction::SourceDetach,
            SystemResourceBacking::Opaque,
        ),
        (
            "ExitStatus",
            SystemResourceType::ExitStatus,
            SystemReleaseAction::LogicalConsume,
            SystemResourceBacking::Opaque,
        ),
        // The traversal-surface candidate's enumeration handle: an opaque
        // stateful resource whose release is one native close attempt, on the
        // same ground as `ReadFile` [SYS-14].
        (
            "DirectoryList",
            SystemResourceType::DirectoryList,
            SystemReleaseAction::NativeCloseAttempt,
            SystemResourceBacking::Opaque,
        ),
    ];
    let mut covered = 0_usize;
    for (index, nominal) in SYSTEM_NOMINALS.iter().enumerate() {
        let index = u8::try_from(index).expect("nominal table fits u8");
        let contract = system_resource_contract(index);
        let Some(row) = expected
            .iter()
            .find(|(spelling, ..)| *spelling == nominal.spelling)
        else {
            assert!(
                contract.is_none(),
                "{} takes no SYS-5 release row",
                nominal.spelling
            );
            assert!(!nominal.opaque);
            continue;
        };
        covered += 1;
        assert!(nominal.opaque);
        let contract = contract.unwrap_or_else(|| panic!("{} has a contract", nominal.spelling));
        assert_eq!(
            contract.resource, row.1,
            "identity for {}",
            nominal.spelling
        );
        assert_eq!(contract.action, row.2, "action for {}", nominal.spelling);
        assert_eq!(contract.backing, row.3, "backing for {}", nominal.spelling);
        // The row is a function of the action, and the two views agree.
        assert_eq!(contract.row, system_release_row(index));
        assert_eq!(
            contract.row.external,
            row.2 == SystemReleaseAction::NativeCloseAttempt
        );
        assert_eq!(contract.row.blocks, contract.row.external);
    }
    assert_eq!(covered, expected.len());
}
