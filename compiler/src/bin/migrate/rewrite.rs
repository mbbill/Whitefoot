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
    /// [OP-1] named rows respelled infix.
    pub(crate) respelled: usize,
    /// [TYPE-5] written type arguments deleted from a de-argumented row.
    pub(crate) arguments: usize,
    /// Prelude constructors given the arguments the annotation carried.
    pub(crate) constructors: usize,
    /// Prelude constructors left bare because no annotation supplied a type.
    pub(crate) bare_constructors: usize,
}

impl Counts {
    pub(crate) fn add(&mut self, other: &Self) {
        self.annotations += other.annotations;
        self.respelled += other.respelled;
        self.arguments += other.arguments;
        self.constructors += other.constructors;
        self.bare_constructors += other.bare_constructors;
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "{} annotation(s), {} respell(s), {} argument list(s), {} constructor(s) written, {} left bare",
            self.annotations,
            self.respelled,
            self.arguments,
            self.constructors,
            self.bare_constructors
        )
    }
}

/// The prelude variant constructors that must write their nominal's arguments.
const PRELUDE_CONSTRUCTORS: [&[u8]; 4] = [b"None", b"Some", b"Ok", b"Err"];

/// The generic prelude nominals whose arguments those constructors carry.
const PRELUDE_GENERICS: [&[u8]; 2] = [b"Option", b"Result"];

/// [OP-1] the v0.22 named rows that became infix operators, and the operator
/// each one spells.
///
/// The tool carries this itself rather than reading the compiler's catalog:
/// the catalog is already v0.23, so the *source* spellings this migration
/// reads no longer exist anywhere in the tree.
const INFIX_RESPELL: [(&[u8], &[u8]); 20] = [
    (b"iadd.wrap", b"+wrap"),
    (b"isub.wrap", b"-wrap"),
    (b"imul.wrap", b"*wrap"),
    (b"iadd.trap", b"+"),
    (b"isub.trap", b"-"),
    (b"imul.trap", b"*"),
    (b"iadd.checked", b"+checked"),
    (b"isub.checked", b"-checked"),
    (b"imul.checked", b"*checked"),
    (b"iadd.sat", b"+sat"),
    (b"isub.sat", b"-sat"),
    (b"imul.sat", b"*sat"),
    (b"idiv.trap", b"/"),
    (b"irem.trap", b"%"),
    (b"idiv.checked", b"/checked"),
    (b"irem.checked", b"%checked"),
    (b"ieq", b"=="),
    (b"ine", b"!="),
    (b"ile", b"<="),
    (b"ige", b">="),
];

/// [TYPE-5] the closed retained-argument class: the rows whose written type
/// argument survives, because no operand can supply it.
const RETAINED_ARGUMENTS: [&[u8]; 6] = [
    b"cvt",
    b"reinterpret",
    b"array_new",
    b"arena_new",
    b"finf",
    b"fnan",
];

/// Every other v0.22 operation family, which loses its written argument.
///
/// A generic *user* function keeps its arguments under [FN-2], so the type
/// arguments are stripped only from a callee this list names.
const DEARGUMENTED: [&[u8]; 57] = [
    b"ineg.wrap",
    b"ineg.trap",
    b"ineg.checked",
    b"ilt",
    b"igt",
    b"eeq",
    b"ene",
    b"fadd.strict",
    b"fsub.strict",
    b"fmul.strict",
    b"fdiv.strict",
    b"feq",
    b"flt",
    b"fle",
    b"fgt",
    b"fge",
    b"fne",
    b"band",
    b"bor",
    b"bxor",
    b"bnot",
    b"len",
    b"slice_of",
    b"box_new",
    b"buffer_new",
    b"iand",
    b"ior",
    b"ixor",
    b"inot",
    b"ishl.wrap",
    b"ishr.wrap",
    b"ishl.trap",
    b"ishr.trap",
    b"irotl",
    b"irotr",
    b"ipopcount",
    b"iclz",
    b"ictz",
    b"ibswap",
    b"imulhi",
    b"imin",
    b"imax",
    b"iabs.wrap",
    b"iabs.trap",
    b"iabs.checked",
    b"fneg",
    b"fabs",
    b"fcopysign",
    b"fmin",
    b"fmax",
    b"ffloor",
    b"fceil",
    b"ftrunc",
    b"froundeven",
    b"frem",
    b"fsqrt.strict",
    b"ffma.strict",
];

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
    // The declared result type of the function being walked. A `return` of a
    // bare prelude constructor takes its arguments from here: the signature
    // survives A3, so this is a textual lookup and not an inference.
    let mut result_type: Option<(usize, usize)> = None;
    let mut index = 0;
    while index < tokens.len() {
        if bytes(source, &tokens, index) == b"fn" {
            result_type = declared_result(&tokens, index);
        }
        if let Some(consumed) =
            returned_constructor(source, &tokens, index, result_type, &mut edits, &mut counts)
        {
            index = consumed;
            continue;
        }
        if let Some(consumed) = annotated_let(source, &tokens, index, &mut edits, &mut counts)? {
            index = consumed;
            continue;
        }
        // [OP-1] the operation call classes. Respelling subsumes
        // de-argumenting for the rows that become operators, because it
        // deletes the whole call form, so it is asked first.
        if let Some(consumed) = operation_call(source, &tokens, index, &mut edits, &mut counts)? {
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

/// [OP-1] one operation call: respelled infix, or stripped of the written
/// type argument the operands now supply.
///
/// Returns the index just past the call when one was rewritten.
fn operation_call(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) -> Result<Option<usize>, String> {
    let callee = tokens.get(index).copied();
    let Some(callee) = callee else {
        return Ok(None);
    };
    if !matches!(
        callee.kind,
        Some(TokenKind::LowerWordForm | TokenKind::OperationNameForm)
    ) {
        return Ok(None);
    }
    let spelling = &source[callee.start..callee.end];
    if RETAINED_ARGUMENTS.contains(&spelling) {
        return Ok(None);
    }
    let respell = INFIX_RESPELL
        .iter()
        .find(|(name, _)| *name == spelling)
        .map(|(_, operator)| *operator);
    if respell.is_none() && !DEARGUMENTED.contains(&spelling) {
        return Ok(None);
    }
    // `callee targs? "("` — the written arguments are optional even in v0.22
    // for the rows whose operand already fixed the type.
    let mut cursor = index + 1;
    let mut arguments = None;
    if tokens.get(cursor).map(|token| token.kind) == Some(Some(TokenKind::LeftAngle)) {
        let close = group_end(tokens, cursor, TokenKind::LeftAngle, TokenKind::RightAngle)
            .ok_or_else(|| "an operation's type arguments reached no `>`".to_owned())?;
        arguments = Some((cursor, close));
        cursor = close + 1;
    }
    if tokens.get(cursor).map(|token| token.kind) != Some(Some(TokenKind::LeftParen)) {
        return Ok(None);
    }
    let open = cursor;
    let close = group_end(tokens, open, TokenKind::LeftParen, TokenKind::RightParen)
        .ok_or_else(|| "an operation call reached no `)`".to_owned())?;

    let Some(operator) = respell else {
        if let Some((start, end)) = arguments {
            edits.push(Edit {
                start: tokens[start].start,
                end: tokens[end].end,
                replacement: Vec::new(),
            });
            counts.arguments += 1;
        }
        return Ok(Some(close + 1));
    };

    // [GRAM-9] the operands are atoms, so the infix form is exactly the two
    // of them with the operator between: delete the callee through `(`,
    // replace the separating comma, and delete `)`.
    let Some(comma) = separator(tokens, open, close) else {
        return Ok(Some(close + 1));
    };
    let head_end = arguments.map_or(tokens[open].end, |_| tokens[open].end);
    edits.push(Edit {
        start: callee.start,
        end: head_end,
        replacement: Vec::new(),
    });
    let mut replacement = b" ".to_vec();
    replacement.extend_from_slice(operator);
    replacement.push(b' ');
    edits.push(Edit {
        start: tokens[comma].start,
        end: tokens[comma].end,
        replacement,
    });
    edits.push(Edit {
        start: tokens[close].start,
        end: tokens[close].end,
        replacement: Vec::new(),
    });
    counts.respelled += 1;
    Ok(Some(close + 1))
}

/// The index of the token closing a group opened at `open`.
fn group_end(
    tokens: &[OwnedLexeme],
    open: usize,
    opening: TokenKind,
    closing: TokenKind,
) -> Option<usize> {
    let mut depth = 0_i32;
    for (offset, token) in tokens.iter().enumerate().skip(open) {
        match token.kind {
            Some(kind) if kind == opening => depth += 1,
            Some(kind) if kind == closing => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// The one comma separating a two-operand call's atoms, or `None` when the
/// call does not have exactly two.
fn separator(tokens: &[OwnedLexeme], open: usize, close: usize) -> Option<usize> {
    let mut depth = 0_i32;
    let mut found = None;
    for (offset, token) in tokens.iter().enumerate().take(close).skip(open) {
        match token.kind {
            Some(TokenKind::LeftParen | TokenKind::LeftAngle | TokenKind::LeftBracket) => {
                depth += 1;
            }
            Some(TokenKind::RightParen | TokenKind::RightAngle | TokenKind::RightBracket) => {
                depth -= 1;
            }
            Some(TokenKind::Comma) if depth == 1 && found.replace(offset).is_some() => {
                return None;
            }
            _ => {}
        }
    }
    found
}

/// The byte range of a function's declared result type, from its `->`.
///
/// The type is one name plus its argument group when it has one, which is
/// exactly where the effect row begins.
fn declared_result(tokens: &[OwnedLexeme], index: usize) -> Option<(usize, usize)> {
    let arrow = tokens
        .iter()
        .enumerate()
        .skip(index)
        .take_while(|(_, token)| token.kind != Some(TokenKind::LeftBrace))
        .find(|(_, token)| token.kind == Some(TokenKind::ThinArrow))
        .map(|(offset, _)| offset)?;
    // `-> mode type`, so the name follows the mode word.
    let name = tokens.get(arrow + 2)?;
    let end = match tokens.get(arrow + 3).map(|token| token.kind) {
        Some(Some(TokenKind::LeftAngle)) => {
            let close = group_end(
                tokens,
                arrow + 3,
                TokenKind::LeftAngle,
                TokenKind::RightAngle,
            )?;
            tokens.get(close)?.end
        }
        _ => name.end,
    };
    Some((name.start, end))
}

/// `return Ok(..)` in a function returning `Result<T, E>` becomes
/// `return Ok<T, E>(..)`.
///
/// This is where the class actually lives: a `let` annotation supplies one
/// site in the corpus, and the returned position supplies almost all of the
/// rest.
fn returned_constructor(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
    result_type: Option<(usize, usize)>,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) -> Option<usize> {
    if bytes(source, tokens, index) != b"return" {
        return None;
    }
    let callee = tokens.get(index + 1)?;
    if callee.kind != Some(TokenKind::UpperWordForm) {
        return None;
    }
    if !PRELUDE_CONSTRUCTORS.contains(&&source[callee.start..callee.end]) {
        return None;
    }
    if tokens.get(index + 2).map(|token| token.kind) != Some(Some(TokenKind::LeftParen)) {
        return None;
    }
    let (start, end) = result_type?;
    let Some(arguments) = prelude_arguments(&source[start..end]) else {
        counts.bare_constructors += 1;
        return Some(index + 2);
    };
    edits.push(Edit {
        start: callee.end,
        end: callee.end,
        replacement: arguments.to_vec(),
    });
    counts.constructors += 1;
    Some(index + 2)
}
