#![forbid(unsafe_code)]

//! Migrates v0.22 Whitefoot source to the v0.23 spelling.
//!
//! The shape is forced and is the same for every transform class: a textual
//! pre-pass rewrites the token stream into *parseable* v0.23, then the source
//! is parsed and re-rendered. The pre-pass owns spelling and never layout;
//! [`render_canonical`] owns layout and never spelling. Nothing here inspects
//! whitespace to decide what a construct is.
//!
//! The pre-pass reads the compiler's own lexemes rather than raw text, so a
//! spelling inside a string or a comment is never rewritten: those are single
//! lexemes and the walk only ever edits token spans.
//!
//! It is re-runnable. A file already in v0.23 has no class to rewrite, so the
//! pre-pass is the identity on it and the render reproduces its own bytes.

use std::path::{Path, PathBuf};

use whitefoot::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalLimits, FinalizeLimits, FinalizeOutcome, LexLimits,
    LexOutcome, Lexeme, ParseLimits, ParseOutcome, RenderOutcome, SourceBundle, SourceInput,
    SourceLimits, TerminalLimits, TerminalOutcome, TokenKind, classify_terminals, finalize, lex,
    parse, render_canonical,
};

mod rewrite;

const SOURCE_LIMITS: SourceLimits = SourceLimits {
    max_sources: 2,
    max_logical_path_bytes: 256,
    max_source_bytes: 1_048_576,
    max_total_source_bytes: 2_097_152,
    max_binding_bytes: 4_194_304,
};

const LEX_LIMITS: LexLimits = LexLimits {
    max_sources: 2,
    max_source_bytes: 1_048_576,
    max_total_source_bytes: 2_097_152,
    max_token_bytes: 16_384,
    max_tokens: 262_144,
    max_lexemes: 524_288,
};

const PARSE_LIMITS: ParseLimits = ParseLimits {
    max_work: 16_000_000,
    max_tasks: 262_144,
    max_frames: 16_384,
    max_elements: 524_288,
};

const FINALIZE_LIMITS: FinalizeLimits = FinalizeLimits {
    max_work: 16_000_000,
    max_roots: 262_144,
    max_shape_tasks: 262_144,
    max_nodes: 262_144,
    max_child_edges: 262_144,
    max_terminals: 262_144,
    max_sources: 2,
};

const CANONICAL_LIMITS: CanonicalLimits = CanonicalLimits {
    max_work: 16_000_000,
    max_source_bytes: 1_048_576,
    max_total_source_bytes: 2_097_152,
    max_gaps: 262_144,
    max_path_components: 16_384,
};

fn main() {
    if let Err(message) = run() {
        eprintln!("whitefoot-migrate: {message}");
        std::process::exit(1);
    }
}

struct Options {
    check_only: bool,
    sources: Vec<PathBuf>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut check_only = false;
        let mut sources = Vec::new();
        for argument in arguments {
            match argument.as_str() {
                "--check" => check_only = true,
                other if other.starts_with("--") => {
                    return Err(format!("unknown option {other}"));
                }
                other => sources.push(PathBuf::from(other)),
            }
        }
        if sources.is_empty() {
            return Err("usage: whitefoot-migrate [--check] <file.wf>...".to_owned());
        }
        Ok(Self {
            check_only,
            sources,
        })
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let options = Options::parse(&arguments)?;
    let mut changed = 0_usize;
    let mut counts = rewrite::Counts::default();
    for path in &options.sources {
        let original = std::fs::read(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let (migrated, file_counts) = migrate(&original, path)?;
        counts.add(&file_counts);
        if migrated != original {
            changed += 1;
            if !options.check_only {
                std::fs::write(path, &migrated)
                    .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
            }
        }
    }
    println!(
        "{} file(s), {changed} changed; {}",
        options.sources.len(),
        counts.summary()
    );
    Ok(())
}

/// Rewrites one source and returns its canonical v0.23 bytes.
fn migrate(original: &[u8], path: &Path) -> Result<(Vec<u8>, rewrite::Counts), String> {
    let display = path.display().to_string();
    let lexemes = lex_source(original, &display)?;
    let (pre_pass, counts) = rewrite::pre_pass(original, &lexemes)?;
    let rendered = render(&pre_pass, &display)?;
    Ok((rendered, counts))
}

/// Lexes with the compiler's own lexer, which accepts the v0.22 token
/// spellings unchanged — only the parser rejects them.
fn lex_source(source: &[u8], display: &str) -> Result<Vec<OwnedLexeme>, String> {
    let inputs = [SourceInput::new("input.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS)
        .map_err(|failure| format!("{display}: source bundle: {failure:?}"))?;
    match lex(&bundle, LEX_LIMITS) {
        LexOutcome::Complete(lexed) => Ok(lexed.lexemes().iter().map(OwnedLexeme::new).collect()),
        other => Err(format!("{display}: does not lex: {other:?}")),
    }
}

/// One lexeme reduced to what the pre-pass reads: its byte range, and its
/// token kind when it is a token rather than trivia.
#[derive(Clone, Copy)]
pub(crate) struct OwnedLexeme {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: Option<TokenKind>,
}

impl OwnedLexeme {
    fn new(lexeme: &Lexeme<'_>) -> Self {
        let (span, kind) = match lexeme {
            Lexeme::Token(token) => (token.span(), Some(token.kind())),
            Lexeme::Trivia(trivia) => (trivia.span(), None),
        };
        Self {
            start: usize::try_from(span.start().value()).unwrap_or(usize::MAX),
            end: usize::try_from(span.end().value()).unwrap_or(usize::MAX),
            kind,
        }
    }
}

/// Parses the pre-passed bytes and renders them canonically.
///
/// A file that does not parse here is a defect in the pre-pass, never a file
/// to repair by hand, so the failure names the stage that rejected it.
fn render(source: &[u8], display: &str) -> Result<Vec<u8>, String> {
    let inputs = [SourceInput::new("input.wf", source)];
    let bundle = SourceBundle::with_limits(&inputs, SOURCE_LIMITS)
        .map_err(|failure| format!("{display}: pre-passed source bundle: {failure:?}"))?;
    let LexOutcome::Complete(lexed) = lex(&bundle, LEX_LIMITS) else {
        return Err(format!("{display}: pre-passed source does not lex"));
    };
    let TerminalOutcome::Complete(classified) = classify_terminals(
        &lexed,
        ACTIVE_KERNEL_SPEC_HASH,
        TerminalLimits {
            max_tokens: LEX_LIMITS.max_tokens,
        },
    ) else {
        return Err(format!("{display}: pre-passed source does not classify"));
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, PARSE_LIMITS) else {
        return Err(format!(
            "{display}: pre-passed source does not parse — a pre-pass defect"
        ));
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
        return Err(format!("{display}: pre-passed source does not finalize"));
    };
    match render_canonical(&finalized, CANONICAL_LIMITS) {
        RenderOutcome::Complete(sources) => sources
            .first()
            .map(|rendered| rendered.bytes.clone())
            .ok_or_else(|| format!("{display}: renderer produced no source")),
        other => Err(format!("{display}: does not render: {other:?}")),
    }
}

#[cfg(test)]
mod tests;
