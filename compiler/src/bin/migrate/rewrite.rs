//! The textual pre-pass: v0.22 spellings to parseable v0.23 ones.
//!
//! Every class is expressed as an edit over the original bytes, keyed off the
//! compiler's own token stream. Layout is deliberately not the pre-pass's
//! concern — the result only has to lex and parse, because the renderer
//! decides every byte of spacing afterwards.

use whitefoot::TokenKind;

use super::OwnedLexeme;

/// One replacement of `source[start..end]`.
struct Edit {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct Counts {
    /// [TYPE-5] `let` annotations deleted.
    pub(crate) annotations: usize,
    /// Prelude constructors given the arguments the annotation carried.
    pub(crate) constructors: usize,
    /// Prelude constructors left bare because no annotation supplied a type.
    pub(crate) bare_constructors: usize,
}

impl Counts {
    pub(crate) fn add(&mut self, other: &Self) {
        self.annotations += other.annotations;
        self.constructors += other.constructors;
        self.bare_constructors += other.bare_constructors;
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "{} annotation(s), {} constructor(s) written, {} left bare",
            self.annotations, self.constructors, self.bare_constructors
        )
    }
}

/// The prelude variant constructors that must write their nominal's arguments.
const PRELUDE_CONSTRUCTORS: [&[u8]; 4] = [b"None", b"Some", b"Ok", b"Err"];

/// The generic prelude nominals whose arguments those constructors carry.
const PRELUDE_GENERICS: [&[u8]; 2] = [b"Option", b"Result"];

pub(crate) fn pre_pass(
    source: &[u8],
    lexemes: &[OwnedLexeme],
) -> Result<(Vec<u8>, Counts), String> {
    let tokens: Vec<OwnedLexeme> = lexemes
        .iter()
        .copied()
        .filter(|lexeme| lexeme.kind.is_some())
        .collect();
    let mut edits = Vec::new();
    let mut counts = Counts::default();
    let mut index = 0;
    while index < tokens.len() {
        if let Some(consumed) = annotated_let(source, &tokens, index, &mut edits, &mut counts)? {
            index = consumed;
            continue;
        }
        index += 1;
    }
    Ok((apply(source, edits), counts))
}

/// [TYPE-5] `let IDENT : mode type = ...` becomes `let IDENT = ...`.
///
/// Returns the index just past the statement's `=` when one was rewritten.
/// A `let` that already carries no annotation is left alone, which is what
/// makes the tool re-runnable.
fn annotated_let(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) -> Result<Option<usize>, String> {
    if bytes(source, tokens, index) != b"let" {
        return Ok(None);
    }
    let Some(colon) = tokens.get(index + 2) else {
        return Ok(None);
    };
    if colon.kind != Some(TokenKind::Colon) {
        return Ok(None);
    }
    let Some(equal) = statement_equal(tokens, index + 3) else {
        return Err("a `let` annotation reached no `=`".to_owned());
    };
    // The annotation is `: mode type`; the type is everything after the mode
    // word, and it is what a bare prelude constructor on the right needs.
    let annotation = tokens
        .get(index + 4)
        .zip(tokens.get(equal - 1))
        .map(|(first, last)| &source[first.start..last.end]);
    edits.push(Edit {
        start: colon.start,
        end: tokens[equal].start,
        replacement: b" ".to_vec(),
    });
    counts.annotations += 1;
    write_constructor_arguments(source, tokens, equal, annotation, edits, counts);
    Ok(Some(equal + 1))
}

/// The `=` that ends a `let` binder, skipping any inside its written type.
fn statement_equal(tokens: &[OwnedLexeme], from: usize) -> Option<usize> {
    let mut depth = 0_i32;
    for (offset, token) in tokens.iter().enumerate().skip(from) {
        match token.kind {
            Some(TokenKind::LeftAngle | TokenKind::LeftParen | TokenKind::LeftBracket) => {
                depth += 1;
            }
            Some(TokenKind::RightAngle | TokenKind::RightParen | TokenKind::RightBracket) => {
                depth -= 1;
            }
            Some(TokenKind::Equal) if depth == 0 => return Some(offset),
            Some(TokenKind::Semicolon) if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

/// Gives a bare prelude constructor the arguments its binder's annotation
/// carried, before that annotation is dropped.
///
/// `let r: own Result<u8, E> = Ok(value: v);` becomes
/// `let r = Ok<u8, E>(value: v);`. A constructor that already writes its
/// arguments, or whose annotation names no generic prelude nominal, is left
/// exactly as written.
fn write_constructor_arguments(
    source: &[u8],
    tokens: &[OwnedLexeme],
    equal: usize,
    annotation: Option<&[u8]>,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) {
    let Some(callee) = tokens.get(equal + 1) else {
        return;
    };
    if callee.kind != Some(TokenKind::UpperWordForm) {
        return;
    }
    let spelling = &source[callee.start..callee.end];
    if !PRELUDE_CONSTRUCTORS.contains(&spelling) {
        return;
    }
    // Already written, so the annotation has nothing to supply.
    if tokens.get(equal + 2).map(|next| next.kind) != Some(Some(TokenKind::LeftParen)) {
        return;
    }
    let Some(arguments) = annotation.and_then(prelude_arguments) else {
        counts.bare_constructors += 1;
        return;
    };
    edits.push(Edit {
        start: callee.end,
        end: callee.end,
        replacement: arguments.to_vec(),
    });
    counts.constructors += 1;
}

/// The `<...>` group of an `Option<..>` or `Result<..>` annotation.
fn prelude_arguments(annotation: &[u8]) -> Option<&[u8]> {
    let nominal_end = annotation.iter().position(|byte| *byte == b'<')?;
    let nominal = annotation.get(..nominal_end)?;
    if !PRELUDE_GENERICS.contains(&nominal) {
        return None;
    }
    let mut depth = 0_i32;
    for (offset, byte) in annotation.iter().enumerate().skip(nominal_end) {
        match byte {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return annotation.get(nominal_end..=offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn bytes<'source>(source: &'source [u8], tokens: &[OwnedLexeme], index: usize) -> &'source [u8] {
    tokens
        .get(index)
        .map_or(&[][..], |token| &source[token.start..token.end])
}

/// Applies every edit to the original bytes in source order.
fn apply(source: &[u8], mut edits: Vec<Edit>) -> Vec<u8> {
    edits.sort_by_key(|edit| edit.start);
    let mut out = Vec::with_capacity(source.len());
    let mut cursor = 0;
    for edit in edits {
        if edit.start < cursor {
            continue;
        }
        out.extend_from_slice(&source[cursor..edit.start]);
        out.extend_from_slice(&edit.replacement);
        cursor = edit.end;
    }
    out.extend_from_slice(&source[cursor..]);
    out
}
