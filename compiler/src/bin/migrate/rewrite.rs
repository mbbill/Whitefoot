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
    /// [OP-1] infix comparisons returned to their named calls.
    pub(crate) unspelled: usize,
    /// [TYPE-5] written type arguments deleted from a de-argumented row.
    pub(crate) arguments: usize,
    /// [GRAM-6] Bool matches rewritten as `if`.
    pub(crate) conditionals: usize,
    /// [GRAM-6] else blocks flattened to `else if`.
    pub(crate) flattened: usize,
    /// Prelude constructors given the arguments the annotation carried.
    pub(crate) constructors: usize,
    /// Prelude constructors left bare because no annotation supplied a type.
    pub(crate) bare_constructors: usize,
}

impl Counts {
    pub(crate) fn add(&mut self, other: &Self) {
        self.annotations += other.annotations;
        self.respelled += other.respelled;
        self.unspelled += other.unspelled;
        self.arguments += other.arguments;
        self.conditionals += other.conditionals;
        self.flattened += other.flattened;
        self.constructors += other.constructors;
        self.bare_constructors += other.bare_constructors;
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "{} annotation(s), {} respell(s), {} comparison(s) unspelled, {} argument list(s), {} constructor(s) written, {} left bare, {} conditional(s) ({} flattened)",
            self.annotations,
            self.respelled,
            self.unspelled,
            self.arguments,
            self.constructors,
            self.bare_constructors,
            self.conditionals,
            self.flattened
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
const INFIX_RESPELL: [(&[u8], &[u8]); 16] = [
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
];

/// [OP-1] the infix comparison spellings the owner cancelled on 2026-08-08,
/// and the named call each one returns to.
///
/// This is the reverse direction of [`INFIX_RESPELL`], and it exists because
/// the corpus was already migrated to the spelling that was cancelled. It is
/// reached from the same pre-pass as every other class, so the `.wf` corpus
/// and the fixtures embedded in Rust literals share one implementation.
///
/// The operator is matched on its bytes rather than on one token kind, so the
/// class keeps working after the lexer stops forming the compound tokens:
/// `==`, `<=`, and `>=` then arrive as two byte-adjacent punctuation tokens.
/// `!=` has no such fallback — `!` is no token at all in v0.22 — so a source
/// still carrying one after that point fails to lex, loudly, rather than
/// migrating silently.
const COMPARISON_UNSPELL: [(&[u8], &[u8]); 4] = [
    (b"==", b"ieq"),
    (b"!=", b"ine"),
    (b"<=", b"ile"),
    (b">=", b"ige"),
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
const DEARGUMENTED: [&[u8]; 61] = [
    b"ineg.wrap",
    b"ineg.trap",
    b"ineg.checked",
    b"ieq",
    b"ine",
    b"ilt",
    b"ile",
    b"igt",
    b"ige",
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
        if let Some(consumed) =
            annotated_let(source, &tokens, index, result_type, &mut edits, &mut counts)?
        {
            index = consumed;
            continue;
        }
        if let Some(consumed) = bool_match(source, &tokens, index, &mut edits, &mut counts)? {
            // Deliberately not skipping the arms: a nested Bool match inside
            // them is found by continuing through them, and its edits fall
            // strictly inside this one's.
            let _ = consumed;
            index += 1;
            continue;
        }
        // [OP-1] the operation call classes. Respelling subsumes
        // de-argumenting for the rows that become operators, because it
        // deletes the whole call form, so it is asked first.
        if let Some(consumed) = operation_call(source, &tokens, index, &mut edits, &mut counts)? {
            index = consumed;
            continue;
        }
        if let Some(consumed) = infix_comparison(source, &tokens, index, &mut edits, &mut counts)? {
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
    result: Option<(usize, usize)>,
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
    write_delivered_constructor_arguments(source, tokens, equal, annotation, edits, counts);
    write_propagated_constructor_arguments(
        source, tokens, equal, annotation, result, edits, counts,
    );
    Ok(Some(equal + 1))
}

/// [GIVE-1] a bare prelude constructor inside a value initializer's `give`.
///
/// The binder's annotation is the delivered type, so every `give Ok(..)` in the
/// initializer needs exactly the arguments [`write_constructor_arguments`]
/// gives a directly assigned one. The constructor sits inside an arm rather
/// than after the `=`, which is why the direct rule never reached it.
fn write_delivered_constructor_arguments(
    source: &[u8],
    tokens: &[OwnedLexeme],
    equal: usize,
    annotation: Option<&[u8]>,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) {
    let Some(arguments) = annotation.and_then(prelude_arguments) else {
        return;
    };
    let Some(end) = initializer_end(tokens, equal) else {
        return;
    };
    for offset in equal + 1..end {
        if bytes(source, tokens, offset) != b"give" {
            continue;
        }
        if let Some(callee) = bare_prelude_constructor(source, tokens, offset + 1) {
            edits.push(Edit {
                start: callee.end,
                end: callee.end,
                replacement: arguments.to_vec(),
            });
            counts.constructors += 1;
        }
    }
}

/// [ERR-3] a bare prelude constructor propagated into a binder.
///
/// `let x: own T = propagate Err(error: e);` inside a function returning
/// `Result<_, E>` needs `Err<T, E>`: the Ok half is the binder's annotation and
/// the error half is the function's declared result error. This is the one
/// position whose arguments come from two places, which is why it is separate
/// from the annotation-copying rules above.
fn write_propagated_constructor_arguments(
    source: &[u8],
    tokens: &[OwnedLexeme],
    equal: usize,
    annotation: Option<&[u8]>,
    result: Option<(usize, usize)>,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) {
    if bytes(source, tokens, equal + 1) != b"propagate" {
        return;
    }
    let Some(callee) = bare_prelude_constructor(source, tokens, equal + 2) else {
        return;
    };
    let (Some(ok), Some((start, end))) = (annotation, result) else {
        counts.bare_constructors += 1;
        return;
    };
    let Some(error) = prelude_arguments(&source[start..end]).and_then(error_argument) else {
        counts.bare_constructors += 1;
        return;
    };
    let mut written = b"<".to_vec();
    written.extend_from_slice(ok);
    written.extend_from_slice(b", ");
    written.extend_from_slice(error);
    written.push(b'>');
    edits.push(Edit {
        start: callee.end,
        end: callee.end,
        replacement: written,
    });
    counts.constructors += 1;
}

/// The error half of a written `<T, E>` argument group.
fn error_argument(arguments: &[u8]) -> Option<&[u8]> {
    let inner = arguments.get(1..arguments.len().checked_sub(1)?)?;
    let mut depth = 0_i32;
    for (offset, byte) in inner.iter().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b',' if depth == 0 => {
                return inner.get(offset + 1..).map(|rest| {
                    let skipped = rest.iter().take_while(|byte| **byte == b' ').count();
                    &rest[skipped..]
                });
            }
            _ => {}
        }
    }
    None
}

/// The prelude constructor at `index`, when it is written without arguments.
fn bare_prelude_constructor(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
) -> Option<OwnedLexeme> {
    let callee = *tokens.get(index)?;
    if callee.kind != Some(TokenKind::UpperWordForm) {
        return None;
    }
    if !PRELUDE_CONSTRUCTORS.contains(&&source[callee.start..callee.end]) {
        return None;
    }
    (tokens.get(index + 1)?.kind == Some(TokenKind::LeftParen)).then_some(callee)
}

/// The token index just past a `let` initializer that opens a block.
///
/// A value initializer's extent is its brace group; a plain expression has
/// none, and returns `None` because it holds no `give`.
fn initializer_end(tokens: &[OwnedLexeme], equal: usize) -> Option<usize> {
    let brace = tokens
        .iter()
        .enumerate()
        .skip(equal + 1)
        .take_while(|(_, token)| token.kind != Some(TokenKind::Semicolon))
        .find(|(_, token)| token.kind == Some(TokenKind::LeftBrace))
        .map(|(offset, _)| offset)?;
    group_end(tokens, brace, TokenKind::LeftBrace, TokenKind::RightBrace)
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

/// [OP-1] one infix comparison returned to its named call: `a <= b` becomes
/// `ile(a, b)`.
///
/// Returns the index just past the right operand when one was rewritten.
///
/// The operator is the anchor and both operands are recovered from it, because
/// unlike every other class this one has no callee token to key on. [GRAM-9]
/// makes each operand exactly one `atom`, so the recovery walks the atom's own
/// forms — place suffixes, a `deref(…)` group, `move`, and a borrow prefix —
/// and nothing else. An operand that reaches a `(` or `<` is a call or a
/// construct, which [GRAM-9] forbids in an atom position: the tool refuses it
/// by name rather than emitting a rewrite around a boundary it guessed.
fn infix_comparison(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) -> Result<Option<usize>, String> {
    let Some((operator, last_operator_token)) = comparison_operator(source, tokens, index) else {
        return Ok(None);
    };
    let callee = COMPARISON_UNSPELL
        .iter()
        .find(|(spelling, _)| *spelling == operator)
        .map(|(_, callee)| *callee)
        .ok_or_else(|| "an unlisted comparison operator reached the rewrite".to_owned())?;
    if index == 0 {
        return Err("an infix comparison began a source".to_owned());
    }
    let left = atom_start(source, tokens, index - 1)
        .ok_or_else(|| "an infix comparison's left operand has no atom start".to_owned())?;
    let right = atom_end(source, tokens, last_operator_token + 1)
        .ok_or_else(|| "an infix comparison's right operand has no atom end".to_owned())?;
    let mut head = callee.to_vec();
    head.push(b'(');
    edits.push(Edit {
        start: tokens[left].start,
        end: tokens[left].start,
        replacement: head,
    });
    edits.push(Edit {
        start: tokens[index].start,
        end: tokens[last_operator_token].end,
        replacement: b", ".to_vec(),
    });
    edits.push(Edit {
        start: tokens[right].end,
        end: tokens[right].end,
        replacement: b")".to_vec(),
    });
    counts.unspelled += 1;
    Ok(Some(right + 1))
}

/// The comparison operator starting at `index`, and its last token.
///
/// One compound token while the lexer still forms it, or two byte-adjacent
/// punctuation tokens after it stops. Adjacency is what makes the two-token
/// reading safe: [FORM-2] renders every canonical `=` with a space on each
/// side, so two touching punctuation bytes never arise any other way.
fn comparison_operator<'source>(
    source: &'source [u8],
    tokens: &[OwnedLexeme],
    index: usize,
) -> Option<(&'source [u8], usize)> {
    let first = tokens.get(index)?;
    let single = &source[first.start..first.end];
    if COMPARISON_UNSPELL
        .iter()
        .any(|(spelling, _)| *spelling == single)
    {
        return Some((single, index));
    }
    let second = tokens.get(index + 1)?;
    if second.start != first.end {
        return None;
    }
    let pair = &source[first.start..second.end];
    COMPARISON_UNSPELL
        .iter()
        .any(|(spelling, _)| *spelling == pair)
        .then_some((pair, index + 1))
}

/// The first token of the `atom` whose last token is `last`.
fn atom_start(source: &[u8], tokens: &[OwnedLexeme], last: usize) -> Option<usize> {
    let mut start = last;
    loop {
        start = match tokens.get(start)?.kind {
            Some(TokenKind::RightParen) => {
                group_start(tokens, start, TokenKind::LeftParen, TokenKind::RightParen)?
            }
            Some(TokenKind::RightBracket) => group_start(
                tokens,
                start,
                TokenKind::LeftBracket,
                TokenKind::RightBracket,
            )?,
            _ => start,
        };
        if start == 0 {
            return Some(0);
        }
        let previous = start - 1;
        // A suffixed place carries its base leftwards; `move`, `deref`, and a
        // borrow prefix carry their own.
        let extends = match tokens.get(previous)?.kind {
            Some(TokenKind::Dot | TokenKind::Ampersand | TokenKind::RegionForm) => true,
            Some(TokenKind::LowerWordForm) => {
                let spelling = bytes(source, tokens, previous);
                spelling == b"move"
                    || spelling == b"uniq"
                    || spelling == b"deref"
                    || suffix_start(tokens, start)
            }
            Some(TokenKind::RightParen | TokenKind::RightBracket) => suffix_start(tokens, start),
            _ => false,
        };
        if !extends {
            return Some(start);
        }
        start = previous;
    }
}

/// The last token of the `atom` whose first token is `first`.
fn atom_end(source: &[u8], tokens: &[OwnedLexeme], first: usize) -> Option<usize> {
    let mut end = first;
    loop {
        end = match tokens.get(end)?.kind {
            Some(TokenKind::LeftParen) => {
                group_end(tokens, end, TokenKind::LeftParen, TokenKind::RightParen)?
            }
            Some(TokenKind::LeftBracket) => {
                group_end(tokens, end, TokenKind::LeftBracket, TokenKind::RightBracket)?
            }
            _ => end,
        };
        let next = end + 1;
        if next >= tokens.len() {
            return Some(end);
        }
        let extends = match tokens[end].kind {
            Some(TokenKind::Dot | TokenKind::Ampersand | TokenKind::RegionForm) => true,
            Some(TokenKind::LowerWordForm) => {
                let spelling = bytes(source, tokens, end);
                spelling == b"move"
                    || spelling == b"uniq"
                    || spelling == b"deref"
                    || suffix_start(tokens, next)
            }
            Some(TokenKind::RightParen | TokenKind::RightBracket) => suffix_start(tokens, next),
            _ => false,
        };
        if !extends {
            return Some(end);
        }
        end = next;
    }
}

/// Whether the token at `index` begins a `psuffix` — `.` IDENT or `[` atom `]`.
fn suffix_start(tokens: &[OwnedLexeme], index: usize) -> bool {
    matches!(
        tokens.get(index).and_then(|token| token.kind),
        Some(TokenKind::Dot | TokenKind::LeftBracket)
    )
}

/// The index of the token opening the group closed at `close`.
fn group_start(
    tokens: &[OwnedLexeme],
    close: usize,
    opening: TokenKind,
    closing: TokenKind,
) -> Option<usize> {
    let mut depth = 0_i32;
    for offset in (0..=close).rev() {
        match tokens[offset].kind {
            Some(kind) if kind == closing => depth += 1,
            Some(kind) if kind == opening => {
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

/// [GRAM-6] a Bool-scrutinee `match` becomes `if`/`else`.
///
/// This is the one class that reshapes rather than respells, and on the token
/// stream the reshape is small because layout is not its problem: the arms'
/// own braces become the `if`'s, so only the match's outer braces and the two
/// arm headers move.
///
/// ```text
/// match S { True() => { A } False() => { B } }  ->  if S { A } else { B }
/// ```
fn bool_match(
    source: &[u8],
    tokens: &[OwnedLexeme],
    index: usize,
    edits: &mut Vec<Edit>,
    counts: &mut Counts,
) -> Result<Option<usize>, String> {
    if bytes(source, tokens, index) != b"match" {
        return Ok(None);
    }
    let open = tokens
        .iter()
        .enumerate()
        .skip(index)
        .find(|(_, token)| token.kind == Some(TokenKind::LeftBrace))
        .map(|(offset, _)| offset)
        .ok_or_else(|| "a `match` reached no `{`".to_owned())?;
    let close = group_end(tokens, open, TokenKind::LeftBrace, TokenKind::RightBrace)
        .ok_or_else(|| "a `match` reached no `}`".to_owned())?;
    let Some(first) = arm(source, tokens, open + 1) else {
        return Ok(None);
    };
    let Some(second) = arm(source, tokens, first.close + 1) else {
        return Ok(None);
    };
    if second.close + 1 != close {
        return Ok(None);
    }
    // Every Bool match in the corpus writes `True` first; a `False`-first one
    // would need its bodies exchanged, so it is reported rather than guessed.
    if !(first.is_true && !second.is_true) {
        if second.is_true && !first.is_true {
            return Err(
                "a Bool `match` writes `False` first; exchange its arms by hand".to_owned(),
            );
        }
        return Ok(None);
    }
    // A value initializer's `else` is mandatory by grammar, so an empty one is
    // kept: dropping it would silently demote the initializer to a statement.
    let value_position = index
        .checked_sub(1)
        .and_then(|previous| tokens.get(previous))
        .is_some_and(|previous| previous.kind == Some(TokenKind::Equal));
    let alternative_is_empty = second.open + 1 == second.close;

    edits.push(Edit {
        start: tokens[index].start,
        end: tokens[index].end,
        replacement: b"if".to_vec(),
    });
    edits.push(Edit {
        start: tokens[open].start,
        end: tokens[open].end,
        replacement: Vec::new(),
    });
    edits.push(Edit {
        start: tokens[first.header].start,
        end: tokens[first.arrow].end,
        replacement: Vec::new(),
    });
    if alternative_is_empty && !value_position {
        // [ERR-2] the else-free `if` is the one spelling of the empty
        // alternative, and GRAM-6 rejects the empty `else`.
        edits.push(Edit {
            start: tokens[second.header].start,
            end: tokens[second.close].end,
            replacement: Vec::new(),
        });
    } else {
        edits.push(Edit {
            start: tokens[second.header].start,
            end: tokens[second.arrow].end,
            replacement: b"else".to_vec(),
        });
        // [GRAM-6] an `else` block holding exactly one `if` must flatten to
        // `else if`, so the block's own braces go.
        if sole_nested_match(source, tokens, second.open, second.close) {
            edits.push(Edit {
                start: tokens[second.open].start,
                end: tokens[second.open].end,
                replacement: Vec::new(),
            });
            edits.push(Edit {
                start: tokens[second.close].start,
                end: tokens[second.close].end,
                replacement: Vec::new(),
            });
            counts.flattened += 1;
        }
    }
    edits.push(Edit {
        start: tokens[close].start,
        end: tokens[close].end,
        replacement: Vec::new(),
    });
    counts.conditionals += 1;
    Ok(Some(close + 1))
}

/// One `True()`/`False()` arm, located by its header tokens.
struct Arm {
    is_true: bool,
    header: usize,
    arrow: usize,
    open: usize,
    close: usize,
}

fn arm(source: &[u8], tokens: &[OwnedLexeme], index: usize) -> Option<Arm> {
    let name = tokens.get(index)?;
    if name.kind != Some(TokenKind::UpperWordForm) {
        return None;
    }
    let is_true = match &source[name.start..name.end] {
        b"True" => true,
        b"False" => false,
        _ => return None,
    };
    if tokens.get(index + 1)?.kind != Some(TokenKind::LeftParen)
        || tokens.get(index + 2)?.kind != Some(TokenKind::RightParen)
        || tokens.get(index + 3)?.kind != Some(TokenKind::FatArrow)
        || tokens.get(index + 4)?.kind != Some(TokenKind::LeftBrace)
    {
        return None;
    }
    let close = group_end(
        tokens,
        index + 4,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
    )?;
    Some(Arm {
        is_true,
        header: index,
        arrow: index + 3,
        open: index + 4,
        close,
    })
}

/// Whether a block holds exactly one `match` and nothing else, which is the
/// shape [GRAM-6] requires flattened once it becomes an `if`.
fn sole_nested_match(source: &[u8], tokens: &[OwnedLexeme], open: usize, close: usize) -> bool {
    if bytes(source, tokens, open + 1) != b"match" {
        return false;
    }
    tokens
        .get(open + 1)
        .and_then(|_| {
            let brace = tokens
                .iter()
                .enumerate()
                .skip(open + 1)
                .find(|(_, token)| token.kind == Some(TokenKind::LeftBrace))
                .map(|(offset, _)| offset)?;
            group_end(tokens, brace, TokenKind::LeftBrace, TokenKind::RightBrace)
        })
        .is_some_and(|nested_close| nested_close + 1 == close)
}
