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
    DependentDeclarationRole, LexicalUseRole, ResolutionIssue, ResolutionIssueKind,
    ResolutionOutcome, ResolutionRule, ResolvedTarget, resolve,
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
    with_one_resolution(b"fn main() -> own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("minimal canonical function must resolve: {outcome:?}");
        };
        assert_eq!(resolved.prelude_declarations().len(), 24);
        assert_eq!(resolved.declarations().len(), 1);
        assert_eq!(resolved.declarations()[0].role(), DeclarationRole::Function);
        assert_eq!(resolved.declarations()[0].spelling(), "main");
        assert!(resolved.scopes().len() >= 3);
    });
}

#[test]
fn top_level_functions_are_visible_throughout_the_closed_unit() {
    let source = br#"fn main() -> own unit pure {
  helper();
}

fn helper() -> own unit pure {
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
    let source = br#"fn main() -> own unit pure {
  let values: own array<i32, 4> = array_new<i32, 4>(0_i32);
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
    let source = br#"fn consume(value: own Later) -> own unit pure {
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
    let source = br#"fn guarded() -> own unit traps requires {
  let value: own Missing = missing;
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
            ResolutionIssueKind::RequiresShape(_)
        ));
    });
}

#[test]
fn requires_locals_do_not_escape_into_the_function_body() {
    let source = br#"fn guarded() -> own unit traps requires {
  let condition: own i32 = 1_i32;
  check condition else trap "failed";
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
    let source = br#"fn value() -> own unit pure {
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
    with_one_resolution(b"fn ieq() -> own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("operation name declaration must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                inventory_ordinal: 16,
                ..
            } if spelling == "ieq"
        ));
    });
}

#[test]
fn region_names_are_unique_across_the_complete_function() {
    let source = br#"fn nested() -> own unit pure {
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
    let source = br#"fn main() -> own unit pure {
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
fn dotless_and_dotted_operations_resolve_by_exact_op1_spelling() {
    let source = br#"fn main() -> own unit pure {
  let sum: own i32 = iadd.wrap(1_i32, 2_i32);
  let equal: own Bool = ieq<i32>(sum, 3_i32);
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("closed operations must resolve: {outcome:?}");
        };
        for spelling in ["iadd.wrap", "ieq"] {
            let usage = resolved
                .lexical_uses()
                .iter()
                .find(|usage| usage.spelling() == spelling)
                .unwrap_or_else(|| panic!("missing operation use {spelling}"));
            assert!(matches!(usage.target(), ResolvedTarget::Operation(_)));
        }
    });
}

#[test]
fn match_binder_cannot_equal_its_paired_field_name() {
    let source = br#"fn main() -> own unit pure {
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

fn main() -> own unit pure {
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

#[test]
fn complete_role_fixture_materializes_every_d_u_and_x_family() {
    let source = br#"contract Bound {
  fn member ['sig](value: &'sig i32) -> own i32 reads('sig);
  law identity(member, 0_i32);
}

contract Numeric<T: Int> {
  fn zero() -> own T pure;
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

fn implementation(value: own i32) -> own i32 pure {
  return value;
}

conform Package<i32, one>: Bound {
  member = implementation;
}

fn user<T: Bound, const n: i32> ['call](arg: &'call T) -> &'call T reads('call) {
  return arg;
}

fn numeric<T: Int>() -> own T pure {
  return 0_T;
}

fn main() -> own unit traps {
  let ordinary: own i32 = iadd.wrap(1_i32, two);
  let made: own Package<i32, one> = Package<i32, one>(items: ordinary);
  set deref(made).items = ordinary;
  region 'r {
    let borrowed: &'r i32 = &'r ordinary;
    let called: &'r i32 = user<i32, 'r, one>(arg: borrowed);
    let view: own slice<'r, i32> = move called;
    check ieq<i32>(ordinary, two) else trap "bad";
  }
  loop @done {
    break @done;
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
    let source = br#"fn main() -> own unit pure {
  let future: own i32 = 1_i32;
  return unit;
}

fn future() -> own unit pure {
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
  fn first ['r](value: &'r i32) -> own unit pure;
  fn second() -> own slice<'r, i32> pure;
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

#[test]
fn semantic_stage_order_precedes_source_position_and_inventory_rank_is_event_local() {
    let later_inventory_error = br#"fn main() -> own unit pure {
  missing();
}

fn ieq() -> own unit pure {
}
"#;
    with_one_resolution(later_inventory_error, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("inventory must reject before lookup: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
    });

    let later_fn8_error = br#"fn ieq() -> own unit pure {
}

fn guarded() -> own unit traps requires {
  let value: own i32 = 1_i32;
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

    let earlier_lower_rank = br#"fn value() -> own unit pure {
}

const value: i32 = 1_i32;

fn ieq() -> own unit pure {
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
            "fn {helper}() -> own unit pure {{\n}}\n\nfn main() -> own unit pure {{\n  let {local}: own i32 = 1_i32;\n  {helper}();\n  return {local};\n}}\n"
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
    let accepted = br#"fn helper() -> own unit pure {
}

fn main() -> own unit pure {
  helper();
}
"#;
    with_one_resolution(accepted, |outcome| {
        assert!(matches!(outcome, ResolutionOutcome::Complete(_)));
    });

    let mutated = br#"fn helper() -> own unit pure {
}

fn main() -> own unit pure {
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
    let source = b"fn main() -> own unit pure {\n  missing();\n}\n";
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

    let first = SourceInput::new("left/name.wf", b"fn same() -> own unit pure {\n}\n");
    let second = SourceInput::new("right/name.wf", b"fn same() -> own unit pure {\n}\n");
    with_resolution(&[first, second], |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("logical paths must not create function namespaces: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Type6);
    });
}

#[test]
fn every_distinct_op1_family_resolves_through_the_normal_callee_path() {
    let mut source = String::from("fn main() -> own unit pure {\n");
    for operation in OPERATION_FAMILIES {
        source.push_str("  ");
        source.push_str(operation);
        source.push_str("<i32>(1_i32);\n");
    }
    source.push_str("}\n");

    with_one_resolution(source.as_bytes(), |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("every closed OP-1 family must resolve: {outcome:?}");
        };
        let operations: Vec<_> = resolved
            .lexical_uses()
            .iter()
            .filter(|usage| matches!(usage.target(), ResolvedTarget::Operation(_)))
            .collect();
        assert_eq!(operations.len(), OPERATION_FAMILIES.len());
        for (ordinal, operation) in operations.into_iter().enumerate() {
            let ResolvedTarget::Operation(id) = operation.target() else {
                unreachable!();
            };
            assert_eq!(usize::from(id.ordinal()), ordinal);
            assert_eq!(operation.spelling(), OPERATION_FAMILIES[ordinal]);
        }
    });
}
