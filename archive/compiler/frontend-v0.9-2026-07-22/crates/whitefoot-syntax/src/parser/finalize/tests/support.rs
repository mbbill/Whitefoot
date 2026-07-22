use whitefoot_contract::{KERNEL_SPEC_V0_9_HASH, SourceBundle, SourceInput, SourceLimits};
use whitefoot_lexer::{LexLimits, LexOutcome, lex_v0_9};

use crate::{ClassifiedBundle, TerminalLimits, TerminalOutcome, classify_terminals_v0_9};

use super::super::{CanonicalLimits, FinalizeLimits};
use crate::parser::{ParseLimits, ParseOutcome, ParsedBundle, parse_v0_9};

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

pub(super) const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 8_000_000,
    max_roots: 131_072,
    max_shape_tasks: 131_072,
    max_nodes: 131_072,
    max_child_edges: 131_072,
    max_terminals: 131_072,
    max_sources: 16,
};

pub(super) const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 8_000_000,
    max_source_bytes: 262_144,
    max_total_source_bytes: 524_288,
    max_gaps: 131_072,
    max_path_components: 8_192,
};

pub(super) fn with_parsed<ResultValue>(
    inputs: &[SourceInput<'_>],
    run: impl for<'classified, 'lexed, 'source> FnOnce(
        ParsedBundle<'classified, 'lexed, 'source>,
    ) -> ResultValue,
) -> ResultValue {
    let Ok(bundle) = SourceBundle::with_limits(inputs, SOURCE_LIMITS) else {
        panic!("test source bundle must be valid");
    };
    let LexOutcome::Complete(lexed) = lex_v0_9(&bundle, LEX_LIMITS) else {
        panic!("test source must lex");
    };
    let TerminalOutcome::Complete(classified) = classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        panic!("test source must classify");
    };
    let ParseOutcome::Complete(parsed) = parse_v0_9(&classified, PARSE_LIMITS) else {
        panic!("test source must derive");
    };
    run(parsed)
}

pub(super) fn source_offsets(classified: &ClassifiedBundle<'_, '_>) -> Vec<usize> {
    classified.source_offsets.clone()
}

pub(super) fn reaches_canonical_syntax(source: &[u8]) -> bool {
    let inputs = [SourceInput::new("generated.wf", source)];
    let Ok(bundle) = SourceBundle::with_limits(&inputs, SOURCE_LIMITS) else {
        panic!("generated source envelope must remain valid");
    };
    let lexed = match lex_v0_9(&bundle, LEX_LIMITS) {
        LexOutcome::Complete(lexed) => lexed,
        LexOutcome::SourceIssue(_) => return false,
        other => panic!("generated source must not hit a non-source lex outcome: {other:?}"),
    };
    let classified = match classify_terminals_v0_9(
        &lexed,
        KERNEL_SPEC_V0_9_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) {
        TerminalOutcome::Complete(classified) => classified,
        TerminalOutcome::SourceIssue(_) => return false,
        other => panic!("generated source must not hit a non-source terminal outcome: {other:?}"),
    };
    let parsed = match parse_v0_9(&classified, PARSE_LIMITS) {
        ParseOutcome::Complete(parsed) => parsed,
        ParseOutcome::SourceIssue(_) => return false,
        other => panic!("generated source must not hit a non-source parse outcome: {other:?}"),
    };
    let finalized = match super::super::finalize_v0_9(parsed, FINALIZE_LIMITS) {
        super::super::FinalizeOutcome::Complete(finalized) => finalized,
        other => panic!("trusted generated derivation must finalize: {other:?}"),
    };
    match super::super::audit_canonical_v0_9(finalized, CANONICAL_LIMITS) {
        super::super::CanonicalOutcome::Complete(_) => true,
        super::super::CanonicalOutcome::SourceIssue(_) => false,
        other => panic!("generated source must not hit an internal canonical outcome: {other:?}"),
    }
}
