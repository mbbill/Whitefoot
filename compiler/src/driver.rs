//! One ordinary active-specification compilation pipeline.
//!
//! The driver keeps source failures, unsupported compiler capabilities,
//! resource failures, invariant failures, lowering failures, and backend
//! failures distinct while returning owned LLVM assembly to callers.

use core::fmt;

use crate::{
    ACTIVE_KERNEL_SPEC_HASH, BackendFailure, CanonicalLimits, CanonicalOutcome, FinalizeLimits,
    FinalizeOutcome, LexLimits, LexOutcome, LoweringFailure, ParseLimits, ParseOutcome,
    ResolutionOutcome, SemanticOutcome, SourceBundle, SourceInput, SourceLimits, TerminalLimits,
    TerminalOutcome, audit_canonical, check_semantics, classify_terminals, emit_llvm, finalize,
    lex, lower_checked, parse, resolve,
};

/// Explicit implementation ceilings for one compiler invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompilerLimits {
    /// Ordered source-envelope limits.
    pub source: SourceLimits,
    /// Lossless lexical limits.
    pub lexer: LexLimits,
    /// Terminal-classification limits.
    pub terminals: TerminalLimits,
    /// Predictive parser limits.
    pub parser: ParseLimits,
    /// Finalized-tree limits.
    pub finalizer: FinalizeLimits,
    /// Canonical-source audit limits.
    pub canonical: CanonicalLimits,
}

impl Default for CompilerLimits {
    fn default() -> Self {
        Self {
            source: SourceLimits {
                max_sources: 1_024,
                max_logical_path_bytes: 4_096,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_binding_bytes: 128 * 1_024 * 1_024,
            },
            lexer: LexLimits {
                max_sources: 1_024,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_token_bytes: 1_024 * 1_024,
                max_tokens: 8 * 1_024 * 1_024,
                max_lexemes: 16 * 1_024 * 1_024,
            },
            terminals: TerminalLimits {
                max_tokens: 8 * 1_024 * 1_024,
            },
            parser: ParseLimits {
                max_work: 256 * 1_024 * 1_024,
                max_tasks: 8 * 1_024 * 1_024,
                max_frames: 65_536,
                max_elements: 16 * 1_024 * 1_024,
            },
            finalizer: FinalizeLimits {
                max_work: 256 * 1_024 * 1_024,
                max_roots: 8 * 1_024 * 1_024,
                max_shape_tasks: 8 * 1_024 * 1_024,
                max_nodes: 8 * 1_024 * 1_024,
                max_child_edges: 8 * 1_024 * 1_024,
                max_terminals: 8 * 1_024 * 1_024,
                max_sources: 1_024,
            },
            canonical: CanonicalLimits {
                max_work: 256 * 1_024 * 1_024,
                max_source_bytes: 16 * 1_024 * 1_024,
                max_total_source_bytes: 64 * 1_024 * 1_024,
                max_gaps: 8 * 1_024 * 1_024,
                max_path_components: 65_536,
            },
        }
    }
}

/// Compiler stage at which one invocation stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilationStage {
    /// PROG-2 source envelope.
    SourceEnvelope,
    /// Raw lossless lexing.
    Lexing,
    /// Context-free terminal membership.
    TerminalClassification,
    /// Strong-LL(2) grammar derivation.
    Parsing,
    /// Finalized production topology.
    Finalization,
    /// Exact FORM-2 source audit.
    CanonicalSource,
    /// Declaration and lexical-use resolution.
    Resolution,
    /// Target-independent semantic checking.
    Semantics,
    /// Checked-program to target-independent IR lowering.
    Lowering,
    /// Selected-target representability and target-domain discharge.
    TargetLayout,
    /// Conservative textual LLVM emission.
    Backend,
}

/// Category of compiler stop, independent of the stage that reported it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompilationFailureKind {
    /// A numbered source-language rule was violated.
    Source,
    /// Valid source requires an unimplemented compiler capability.
    Unsupported,
    /// An explicit implementation ceiling or host storage stopped work.
    Resource,
    /// The caller supplied an invalid compilation envelope or stage identity.
    Invocation,
    /// A trusted compiler invariant failed.
    Compiler,
    /// Checked-program to IR lowering failed internally.
    Lowering,
    /// A statically materialized object is not representable on the selected target.
    TargetLayout,
    /// LLVM emission failed internally.
    Backend,
}

/// One compiler stop with its category preserved in the detail text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationFailure {
    stage: CompilationStage,
    kind: CompilationFailureKind,
    rule_id: Option<&'static str>,
    detail: String,
}

impl CompilationFailure {
    fn new(stage: CompilationStage, kind: CompilationFailureKind, detail: impl fmt::Debug) -> Self {
        Self {
            stage,
            kind,
            rule_id: None,
            detail: format!("{detail:?}"),
        }
    }

    fn semantic_source(issue: crate::SemanticIssue) -> Self {
        Self {
            stage: CompilationStage::Semantics,
            kind: CompilationFailureKind::Source,
            rule_id: Some(issue.rule_id()),
            detail: format!("{issue:?}"),
        }
    }

    /// Returns the stage that did not produce a complete result.
    #[must_use]
    pub const fn stage(&self) -> CompilationStage {
        self.stage
    }

    /// Returns the source/unsupported/resource/invocation/internal category.
    #[must_use]
    pub const fn kind(&self) -> CompilationFailureKind {
        self.kind
    }

    /// Returns the structured debug detail retained by that stage.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Returns the exact numbered source rule when this stop is semantic.
    #[must_use]
    pub const fn rule_id(&self) -> Option<&'static str> {
        self.rule_id
    }
}

impl fmt::Display for CompilationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rule_id) = self.rule_id {
            write!(
                formatter,
                "{:?}/{:?} [{rule_id}]: {}",
                self.stage, self.kind, self.detail
            )
        } else {
            write!(
                formatter,
                "{:?}/{:?}: {}",
                self.stage, self.kind, self.detail
            )
        }
    }
}

impl std::error::Error for CompilationFailure {}

/// Compiles one ordered closed source bundle to conservative textual LLVM.
pub fn compile(
    inputs: &[SourceInput<'_>],
    limits: CompilerLimits,
) -> Result<String, CompilationFailure> {
    let bundle = SourceBundle::with_limits(inputs, limits.source).map_err(|failure| {
        CompilationFailure::new(
            CompilationStage::SourceEnvelope,
            CompilationFailureKind::Invocation,
            failure,
        )
    })?;
    let lexed = match lex(&bundle, limits.lexer) {
        LexOutcome::Complete(complete) => complete,
        LexOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::new(
                CompilationStage::Lexing,
                CompilationFailureKind::Source,
                issue,
            ));
        }
        LexOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Lexing,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        LexOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Lexing,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let classified = match classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals) {
        TerminalOutcome::Complete(complete) => complete,
        TerminalOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Source,
                issue,
            ));
        }
        TerminalOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        TerminalOutcome::InvocationFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Invocation,
                failure,
            ));
        }
        TerminalOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::TerminalClassification,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let parsed = match parse(&classified, limits.parser) {
        ParseOutcome::Complete(complete) => complete,
        ParseOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Source,
                issue,
            ));
        }
        ParseOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        ParseOutcome::InvocationFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Invocation,
                failure,
            ));
        }
        ParseOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Parsing,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let finalized = match finalize(parsed, limits.finalizer) {
        FinalizeOutcome::Complete(complete) => complete,
        FinalizeOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Finalization,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        FinalizeOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::Finalization,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let canonical = match audit_canonical(finalized, limits.canonical) {
        CanonicalOutcome::Complete(complete) => complete,
        CanonicalOutcome::SourceIssue(issue) => {
            return Err(CompilationFailure::new(
                CompilationStage::CanonicalSource,
                CompilationFailureKind::Source,
                issue,
            ));
        }
        CanonicalOutcome::ResourceFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::CanonicalSource,
                CompilationFailureKind::Resource,
                failure,
            ));
        }
        CanonicalOutcome::CompilerFailure(failure) => {
            return Err(CompilationFailure::new(
                CompilationStage::CanonicalSource,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let resolved = match resolve(canonical) {
        ResolutionOutcome::Complete(complete) => complete,
        ResolutionOutcome::SourceIssue { issue, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Resolution,
                CompilationFailureKind::Source,
                issue,
            ));
        }
        ResolutionOutcome::CompilerFailure { failure, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Resolution,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let checked = match check_semantics(resolved) {
        SemanticOutcome::Complete(complete) => *complete,
        SemanticOutcome::SourceIssue { issue, .. } => {
            return Err(CompilationFailure::semantic_source(issue));
        }
        SemanticOutcome::Unsupported { unsupported, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Semantics,
                CompilationFailureKind::Unsupported,
                unsupported,
            ));
        }
        SemanticOutcome::CompilerFailure { failure, .. } => {
            return Err(CompilationFailure::new(
                CompilationStage::Semantics,
                CompilationFailureKind::Compiler,
                failure,
            ));
        }
    };
    let ir = lower_checked(checked).map_err(|failure: LoweringFailure| {
        CompilationFailure::new(
            CompilationStage::Lowering,
            CompilationFailureKind::Lowering,
            failure,
        )
    })?;
    emit_llvm(&ir)
        .map(|module| module.into_string())
        .map_err(|failure: BackendFailure| {
            let (stage, kind) = match failure {
                BackendFailure::TargetLayout(_) => (
                    CompilationStage::TargetLayout,
                    CompilationFailureKind::TargetLayout,
                ),
                _ => (CompilationStage::Backend, CompilationFailureKind::Backend),
            };
            CompilationFailure::new(stage, kind, failure)
        })
}

#[cfg(test)]
mod tests {
    use super::{CompilationFailureKind, CompilationStage, CompilerLimits, compile};
    use crate::SourceInput;

    #[test]
    fn driver_lowers_static_contract_metadata_without_executable_artifacts() {
        let source = b"contract Empty {\n}\n\nconform i32: Empty {\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n";
        let llvm = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect("static contract metadata must use the ordinary lowering path");
        assert!(llvm.contains("define i32 @main()"));
        assert!(!llvm.contains("Empty"));
    }

    #[test]
    fn unrepresentable_array_is_a_target_failure_without_a_source_rule() {
        let source = b"fn main() -> own unit pure {\n  let values: own array<u8, 18446744073709551615> = array_new<u8, 18446744073709551615>(0_u8);\n  return unit;\n}\n";
        let failure = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("the selected target cannot represent the array object");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("Unrepresentable"));
    }

    #[test]
    fn complete_frame_is_checked_after_each_slot_layout_succeeds() {
        let source = b"fn main() -> own unit pure {\n  let left: own array<u8, 4611686018427387904> = array_new<u8, 4611686018427387904>(0_u8);\n  let right: own array<u8, 4611686018427387904> = array_new<u8, 4611686018427387904>(0_u8);\n  return unit;\n}\n";
        let failure = compile(
            &[SourceInput::new("value.wf", source)],
            CompilerLimits::default(),
        )
        .expect_err("two individually representable slots cannot form one target frame");
        assert_eq!(failure.stage(), CompilationStage::TargetLayout);
        assert_eq!(failure.kind(), CompilationFailureKind::TargetLayout);
        assert_eq!(failure.rule_id(), None);
        assert!(failure.detail().contains("StackFrame"));
    }

    #[test]
    fn compiler_independent_negative_cases_keep_their_semantic_rule() {
        for (name, source, rule) in [
            (
                "gram11-neg-misspelled.wf",
                include_bytes!("../../tests/conformance/cases/gram11-neg-misspelled.wf").as_slice(),
                "GRAM-11",
            ),
            (
                "eff2-neg-declared-unexhibited.wf",
                include_bytes!("../../tests/conformance/cases/eff2-neg-declared-unexhibited.wf")
                    .as_slice(),
                "EFF-2",
            ),
            (
                "fn2-neg-implicit-instantiation.wf",
                include_bytes!("../../tests/conformance/cases/fn2-neg-implicit-instantiation.wf")
                    .as_slice(),
                "FN-2",
            ),
            (
                "form7-neg-out-of-range.wf",
                include_bytes!("../../tests/conformance/cases/form7-neg-out-of-range.wf")
                    .as_slice(),
                "FORM-7",
            ),
            (
                "type5-neg-arg-mismatch.wf",
                include_bytes!("../../tests/conformance/cases/type5-neg-arg-mismatch.wf")
                    .as_slice(),
                "TYPE-5",
            ),
            (
                "x-struct-neg-field-order.wf",
                include_bytes!("../../tests/conformance/cases/x-struct-neg-field-order.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-match-gram10-out-of-order-fields.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-match-gram10-out-of-order-fields.wf"
                )
                .as_slice(),
                "GRAM-10",
            ),
            (
                "err2-neg-missing-variant.wf",
                include_bytes!("../../tests/conformance/cases/err2-neg-missing-variant.wf")
                    .as_slice(),
                "ERR-2",
            ),
            (
                "x-ownmove-partial-move-kills-binding.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-ownmove-partial-move-kills-binding.wf"
                )
                .as_slice(),
                "OWN-1",
            ),
            (
                "x-ownmove-payload-binder-consumed-twice.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-ownmove-payload-binder-consumed-twice.wf"
                )
                .as_slice(),
                "OWN-1",
            ),
            (
                "x-gram-construct-repeated-field.wf",
                include_bytes!("../../tests/conformance/cases/x-gram-construct-repeated-field.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-gram-construct-missing-field.wf",
                include_bytes!("../../tests/conformance/cases/x-gram-construct-missing-field.wf")
                    .as_slice(),
                "GRAM-8",
            ),
            (
                "x-typ-match-foreign-variant.wf",
                include_bytes!("../../tests/conformance/cases/x-typ-match-foreign-variant.wf")
                    .as_slice(),
                "TYPE-6",
            ),
            (
                "x-match-give1-wrong-type.wf",
                include_bytes!("../../tests/conformance/cases/x-match-give1-wrong-type.wf")
                    .as_slice(),
                "TYPE-5",
            ),
            (
                "x-integ-give-in-statement-match-rejected.wf",
                include_bytes!(
                    "../../tests/conformance/cases/x-integ-give-in-statement-match-rejected.wf"
                )
                .as_slice(),
                "GIVE-1",
            ),
        ] {
            let failure = compile(&[SourceInput::new(name, source)], CompilerLimits::default())
                .expect_err("negative conformance case must reject");
            assert_eq!(
                failure.stage(),
                CompilationStage::Semantics,
                "{name}: {failure}"
            );
            assert_eq!(
                failure.kind(),
                CompilationFailureKind::Source,
                "{name}: {failure}"
            );
            assert_eq!(failure.rule_id(), Some(rule), "{name}: {failure}");
            assert!(
                failure.to_string().contains(rule),
                "{name}: published diagnostic omitted {rule}: {failure}"
            );
        }
    }
}
