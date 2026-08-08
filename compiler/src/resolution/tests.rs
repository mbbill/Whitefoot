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
    DependentDeclarationRole, LexicalUseRecord, LexicalUseRole, ResolutionIssue,
    ResolutionIssueKind, ResolutionOutcome, ResolutionRule, ResolvedTarget, resolve,
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
  let value = missing;
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
fn the_kind_declaring_judgment_gates_only_the_system_admission_decision() {
    // A unit with no `program_kind` child is not kind-declaring, so nothing
    // about the entry-form grammar changes its ordinary resolution, and the
    // system domain contributes no entry to it [SYS-3].
    let unlabelled = b"fn main() -> own unit pure {\n  return unit;\n}\n";
    with_one_resolution(unlabelled, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("the unlabelled entry must resolve unchanged: {outcome:?}");
        };
        assert!(resolved.system_declarations().is_empty());
    });

    // One `program_kind` child makes the unit kind-declaring, which admits
    // the complete SYS-2 inventory as a third declaration source [SYS-1]:
    // the entry's system input and result types resolve to system targets.
    let kind_declaring =
        b"command fn main(command.args as args: own Args) -> own ExitStatus pure {\n  return unit;\n}\n";
    with_one_resolution(kind_declaring, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("a kind-declaring unit must resolve system names: {outcome:?}");
        };
        assert_eq!(resolved.system_declarations().len(), 167);
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

    // The judgment is syntactic: a `program_kind` on a declaration that is not
    // the entry still makes the unit kind-declaring, and the unlabelled `main`
    // beside it changes nothing.
    let non_entry_kind = b"command fn helper() -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
    with_one_resolution(non_entry_kind, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("a non-entry program_kind must still be kind-declaring: {outcome:?}");
        };
        assert_eq!(resolved.system_declarations().len(), 167);
    });
}

#[test]
fn fn8_admission_precedes_the_system_admission_decision() {
    // DIAG-1 fixes the stage order: only complete unit-wide FN-8 admission
    // permits the SYS-3 system-admission decision. The FN-8 rejection must
    // therefore win in a kind-declaring unit before any system name enters
    // inventory or lookup.
    let source =
        br#"command fn main(command.args as args: own Args) -> own ExitStatus pure requires {
  doc "not an admitted requires entry";
} {
  return unit;
}
"#;
    with_one_resolution(source, |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the FN-8 defect must outrank system admission: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Fn8);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::RequiresShape(_)
        ));
    });

    // The same kind-declaring entry with an admitted requires block reaches
    // the system-admission decision and resolves with the domain admitted.
    let admitted =
        br#"command fn main(command.args as args: own Args) -> own ExitStatus traps requires {
  check args else trap "present";
} {
  return unit;
}
"#;
    with_one_resolution(admitted, |outcome| {
        let ResolutionOutcome::Complete(resolved) = outcome else {
            panic!("an admitted requires block must reach the SYS-3 decision: {outcome:?}");
        };
        assert_eq!(resolved.system_declarations().len(), 167);
    });
}

#[test]
fn a_kind_declaring_unit_resolves_the_complete_system_lookup_inventory() {
    // Every system nominal type as a `type` TYPEID use, every operation as an
    // IDENT callee, one constructor in construct position, and the
    // `ReadOutcome` variants in arm position, with deterministic [SYS-2]
    // preorder ordinals throughout. Resolution fixes callee targets only;
    // argument-name checking against the [SYS-2] parameter lists is the
    // later typed stage, so the `arg_get` call here omits its second
    // argument: the declared name `index` is a fixed [GRAM-5] atom that
    // [FORM-3] excludes from IDENT, so a complete [GRAM-11] call to
    // `arg_get` is unwritable under v0.18 — a recorded specification
    // finding, not behavior this test may normalize.
    let source = br#"command fn main() -> own ExitStatus pure {
  return exit_status(code: 0_u8);
}

fn types(a: own Args, b: own HostString, c: own RelativePath, d: own DirectoryRead, e: own ReadFile, f: own Output, g: own ExitStatus, h: own ArgError, i: own Utf8Error, j: own CopyError, k: own Utf8CopyError, l: own PathError, m: own ReadOutcome, n: own IoError) -> own unit pure {
  return unit;
}

fn calls(x: own u64) -> own unit pure {
  args_count(args: x);
  arg_get(args: x);
  host_bytes_len(value: x);
  host_copy_bytes(value: x, destination: x, offset: x, capacity: x);
  host_utf8_len(value: x);
  host_copy_utf8(value: x, destination: x, offset: x, capacity: x);
  relative_path(value: x);
  open_read(root: x, path: x);
  read_once(file: x, destination: x, offset: x, capacity: x);
  write_once(output: x, source: x, offset: x, count: x);
  return unit;
}

fn outcomes(m: own ReadOutcome) -> own unit pure {
  let failed = NotFound(code: 1_u32, origin: 0_u8);
  match m {
    ReadBytes(count: got) => {
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
        ] {
            expect(LexicalUseRole::Type, spelling, ordinal);
        }
        for (spelling, ordinal) in [
            ("args_count", 117),
            ("arg_get", 120),
            ("host_bytes_len", 124),
            ("host_copy_bytes", 127),
            ("host_utf8_len", 134),
            ("host_copy_utf8", 137),
            ("relative_path", 144),
            ("open_read", 146),
            ("read_once", 151),
            ("write_once", 158),
            ("exit_status", 165),
        ] {
            expect(LexicalUseRole::IdentifierCallee, spelling, ordinal);
        }
        expect(LexicalUseRole::Construct, "NotFound", 27);
        expect(LexicalUseRole::ArmVariant, "ReadBytes", 22);
        expect(LexicalUseRole::ArmVariant, "ReadEnd", 24);
        expect(LexicalUseRole::ArmVariant, "ReadFailed", 25);
    });
}

#[test]
fn a_system_unadmitted_unit_sees_system_spellings_as_ordinary_undeclared_names() {
    // [SYS-3]: in a unit that is not kind-declaring the system domain
    // contributes no entry, so a system operation spelling is an ordinary
    // undeclared callee decided by the ordinary lexical-use ranks.
    let callee = br#"fn main() -> own unit pure {
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
    let nominal = br#"fn consume(value: own HostString) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
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

fn args_count(args: own u64) -> own u64 pure {
  return args;
}

fn main() -> own unit pure {
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
    let entry =
        "command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    let lookalike = "fn args_count(args: own u64) -> own u64 pure {\n  return args;\n}\n";
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
                DeclarationOrigin::System(id) if id.ordinal() == 117
            ));
        });
    }
}

#[test]
fn system_collisions_cover_every_contributed_domain_and_nested_scopes() {
    let entry =
        "command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

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
            DeclarationOrigin::System(id) if id.ordinal() == 24
        ));
    });

    // A nested declaration collides at rank 5 exactly like a root one
    // ([SYS-1]: at the compilation root and in every nested scope alike);
    // this is a rejection, never a shadow of the system entry.
    let nested = "command fn main() -> own ExitStatus pure {\n  let host_bytes_len = 0_u64;\n  return exit_status(code: 0_u8);\n}\n";
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
            DeclarationOrigin::System(id) if id.ordinal() == 124
        ));
    });
}

#[test]
fn a_prelude_collision_keeps_rank_four_in_a_system_admitted_unit() {
    // [DIAG-1] rank 4 precedes rank 5 at one event: a PRE-1 collision in a
    // kind-declaring unit reports only its PRE-1 conflicts.
    let source = "command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n\nstruct Overflow {\n}\n";
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
    let source = br#"command fn main() -> own ExitStatus pure {
  return exit_status(code: 0_u8);
}

contract Task {
  fn run(value: own u64) -> own u64 pure;
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
        b"command fn main() -> own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
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
            ("exit_status".to_owned(), 165)
        ]
    );
    assert_eq!(first, targets("first.wf"));
    assert_eq!(first, targets("renamed/location.wf"));
}

#[test]
fn requires_locals_do_not_escape_into_the_function_body() {
    let source = br#"fn guarded() -> own unit traps requires {
  let condition = 1_i32;
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
    with_one_resolution(b"fn ilt() -> own unit pure {\n}\n", |outcome| {
        let ResolutionOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("operation name declaration must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), ResolutionRule::Form3);
        assert!(matches!(
            issue.kind(),
            ResolutionIssueKind::ReservedName {
                spelling,
                inventory_ordinal: 18,
                ..
            } if spelling == "ilt"
        ));
    });
}

/// OP-1 (iii): reservation is derived from the op column, so respelling
/// `ieq` to `==` takes it out of `DotlessOperationNames` and frees the name
/// for source. `ilt` keeps its spelling under ruling O1 and stays reserved,
/// which is what the test above pins.
#[test]
fn respelled_comparisons_leave_the_reserved_name_inventory() {
    with_one_resolution(b"fn ieq() -> own unit pure {\n}\n", |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "a respelled comparison name is declarable: {outcome:?}"
        );
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
  let sum = 1_i32 +wrap 2_i32;
  let equal = sum == 3_i32;
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
  fn member['sig](value: &'sig i32) -> own i32 reads('sig);
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

fn user<T: Bound, const n: i32>['call](arg: &'call T) -> &'call T reads('call) {
  return arg;
}

fn numeric<T: Int>() -> own T pure {
  return 0_T;
}

fn main() -> own unit traps {
  let ordinary = 1_i32 +wrap two;
  let made = Package<i32, one>(items: ordinary);
  set deref(made).items = ordinary;
  region 'r {
    let borrowed = &'r ordinary;
    let called = user<i32, 'r, one>(arg: borrowed);
    let view = move called;
    check ordinary == two else trap "bad";
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
  let future = 1_i32;
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
  fn first['r](value: &'r i32) -> own unit pure;
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
    let sibling_branches = br#"fn get(pick: own Bool) -> own unit traps {
  if pick {
    let inside = 1_u64;
    check inside == 1_u64 else trap "left";
  } else {
    let inside = 2_u64;
    check inside == 2_u64 else trap "right";
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

fn get(pick: own Pick) -> own unit traps {
  match pick {
    Left() => {
      let inside = 1_u64;
      check inside == 1_u64 else trap "left";
    }
    Right() => {
      let inside = 2_u64;
      check inside == 2_u64 else trap "right";
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

    let expired_then_enclosing = br#"fn get(pick: own Bool) -> own unit traps {
  if pick {
    let offset = 0_u64;
    check offset == 0_u64 else trap "inner";
  }
  let offset = 1_u64;
  check offset == 1_u64 else trap "outer";
  return unit;
}
"#;
    with_one_resolution(expired_then_enclosing, |outcome| {
        assert!(
            matches!(outcome, ResolutionOutcome::Complete(_)),
            "an expired branch scope may not block a later enclosing binder: {outcome:?}"
        );
    });

    let live_shadow = br#"fn get(pick: own Bool) -> own unit traps {
  let offset = 0_u64;
  if pick {
    let offset = 1_u64;
    check offset == 1_u64 else trap "inner";
  }
  check offset == 0_u64 else trap "outer";
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
  let value = 1_i32;
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
            "fn {helper}() -> own unit pure {{\n}}\n\nfn main() -> own unit pure {{\n  let {local} = 1_i32;\n  {helper}();\n  return {local};\n}}\n"
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

#[test]
fn system_index_helpers_agree_with_the_preorder_entity_map() {
    // The index helpers derive table positions arithmetically from the
    // [SYS-2] preorder; this pins them to `system_entity`, the authoritative
    // ordinal-to-entity map, across every one of the 167 records.
    use super::SystemDeclarationId;
    use super::catalog::{
        SYSTEM_CONSTRUCTORS, SYSTEM_NOMINALS, SYSTEM_OPERATIONS, SystemEntity,
        system_constructor_declaration, system_constructor_index, system_entity,
        system_nominal_index, system_operation_index, system_release_row,
    };

    let mut nominals = 0_usize;
    let mut constructors = 0_usize;
    let mut operations = 0_usize;
    for ordinal in 0..=u8::MAX {
        let id = SystemDeclarationId::new(ordinal);
        match system_entity(id) {
            Some(SystemEntity::Nominal(nominal)) => {
                let index = system_nominal_index(id).expect("nominal index");
                assert_eq!(
                    SYSTEM_NOMINALS[usize::from(index)].spelling,
                    nominal.spelling
                );
                assert!(system_constructor_index(id).is_none());
                assert!(system_operation_index(id).is_none());
                nominals += 1;
            }
            Some(SystemEntity::Constructor(constructor)) => {
                let index = system_constructor_index(id).expect("constructor index");
                assert_eq!(
                    SYSTEM_CONSTRUCTORS[usize::from(index)].spelling,
                    constructor.spelling
                );
                assert_eq!(system_constructor_declaration(index), Some(id));
                assert!(system_nominal_index(id).is_none());
                assert!(system_operation_index(id).is_none());
                constructors += 1;
            }
            Some(SystemEntity::Operation(operation)) => {
                let index = system_operation_index(id).expect("operation index");
                assert_eq!(
                    SYSTEM_OPERATIONS[usize::from(index)].spelling,
                    operation.spelling
                );
                assert!(system_nominal_index(id).is_none());
                assert!(system_constructor_index(id).is_none());
                operations += 1;
            }
            None => {
                assert!(system_constructor_index(id).is_none());
                assert!(system_operation_index(id).is_none());
            }
        }
    }
    assert_eq!(nominals, SYSTEM_NOMINALS.len());
    assert_eq!(constructors, SYSTEM_CONSTRUCTORS.len());
    assert_eq!(operations, SYSTEM_OPERATIONS.len());

    // The [SYS-5] release table: exactly DirectoryRead and ReadFile release
    // with `external, blocks`; every other system nominal's row is empty.
    for (index, nominal) in SYSTEM_NOMINALS.iter().enumerate() {
        let index = u8::try_from(index).expect("nominal table fits u8");
        let row = system_release_row(index);
        let expected = matches!(nominal.spelling, "DirectoryRead" | "ReadFile");
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
