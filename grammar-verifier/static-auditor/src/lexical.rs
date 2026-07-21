//! Exact lexical-declaration handlers and external-reference closure.

use std::collections::BTreeSet;

use crate::document::{Document, Span};
use crate::ebnf::NodeKind;
use crate::float_contract::{self, Dialect as FloatDialect};
use crate::grammar::{Grammar, Surface, SurfaceKind};
use crate::wire::{Failure, Limits, Work};

const CLASSES_CUE: &[u8] = b"Lexical classes:";
const LITERALS_CUE: &[u8] = b"Literals, exhaustively:";
const TABLE_SUFFIX: &[u8] = b" is a closed table:";
const IDENT_EXCLUSION: &[u8] =
    b" excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar";

const REGION_ANNOTATION: &[u8] = b" (apostrophe-prefixed, the only region spelling)";
const OPNAME_V08_ANNOTATION: &[u8] = concat!(
    " (single token; the base is an IDENT and the mode suffix is a closed word set, ",
    "so an OPNAME can never maximal-munch a field-access place `p.field` [GRAM-5]; ",
    "e.g. `iadd.checked`)"
)
.as_bytes();
const OPNAME_PARTITIONED_ANNOTATION: &[u8] = concat!(
    " (single token; the base has the raw lowercase-word shape used by IDENT and the mode ",
    "suffix is a closed word set, so an OPNAME can never maximal-munch a valid field-access ",
    "place `p.field`: all five suffix words are reserved from field binding [OP-1, GRAM-5]; ",
    "e.g. `iadd.checked`)"
)
.as_bytes();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LexKind {
    Regex,
    LiteralUnion(FloatDialect),
    ByteString,
    ClosedTable,
}

impl LexKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Regex => "regex",
            Self::LiteralUnion(_) => "literal-union",
            Self::ByteString => "byte-string",
            Self::ClosedTable => "closed-table",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Lexical<'a> {
    pub(crate) owner: &'a str,
    pub(crate) name: &'a str,
    pub(crate) kind: LexKind,
    pub(crate) span: Span,
    pub(crate) predicate: String,
    pub(crate) excludes_fixed_lowerwords: bool,
    pub(crate) table_spellings: Vec<&'a str>,
}

fn find_unique(bytes: &[u8], needle: &[u8], code: &'static str) -> Result<usize, Failure> {
    let mut found = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == needle).then_some(offset));
    let first = found.next().ok_or_else(|| Failure::extraction(code))?;
    if found.next().is_some() {
        return Err(Failure::extraction(code));
    }
    Ok(first)
}

fn line_bounds(document: &Document<'_>, offset: usize) -> Result<Span, Failure> {
    document
        .lines
        .iter()
        .find(|line| line.start <= offset && offset < line.end)
        .map(|line| Span {
            start: line.start,
            end: line.end - 1,
        })
        .ok_or_else(|| Failure::internal("lexical-line"))
}

fn push_lexical<'a>(
    output: &mut Vec<Lexical<'a>>,
    value: Lexical<'a>,
    limits: &Limits,
) -> Result<(), Failure> {
    if output.len() >= limits.max_lexical_definitions {
        return Err(Failure::resource("max-lexical-definitions"));
    }
    output.try_reserve(1).map_err(|_| Failure::allocation())?;
    output.push(value);
    Ok(())
}

fn surface<'a>(
    grammar: &mut Grammar<'a>,
    document: &Document<'a>,
    span: Span,
) -> Result<&'a str, Failure> {
    let owner = document.owner(span)?.id;
    grammar
        .surfaces
        .try_reserve(1)
        .map_err(|_| Failure::allocation())?;
    grammar.surfaces.push(Surface {
        kind: SurfaceKind::LexicalCue,
        span,
        owner,
    });
    Ok(owner)
}

fn extract_classes<'a>(
    document: &Document<'a>,
    grammar: &mut Grammar<'a>,
    limits: &Limits,
    output: &mut Vec<Lexical<'a>>,
) -> Result<(), Failure> {
    let cue = find_unique(document.bytes, CLASSES_CUE, "lexical-classes-missing")?;
    let line = line_bounds(document, cue)?;
    let owner = surface(
        grammar,
        document,
        Span {
            start: cue,
            end: cue + CLASSES_CUE.len(),
        },
    )?;
    const SLOTS: [(&str, &[u8], &[u8]); 5] = [
        ("IDENT", b"[a-z][a-z0-9_]*", b""),
        ("TYPEID", b"[A-Z][A-Za-z0-9]*", b""),
        ("REGIONID", b"'[a-z][a-z0-9_]*", REGION_ANNOTATION),
        ("LABEL", b"@[a-z][a-z0-9_]*", b""),
        (
            "OPNAME",
            b"[a-z][a-z0-9_]*\\.(wrap|trap|checked|sat|strict)",
            OPNAME_V08_ANNOTATION,
        ),
    ];
    let mut cursor = cue + CLASSES_CUE.len();
    if document.bytes.get(cursor) != Some(&b' ') {
        return Err(Failure::extraction("lexical-classes-structure"));
    }
    cursor += 1;
    let mut fixed_lowerword_partition = false;
    for (index, (name, pattern, default_annotation)) in SLOTS.into_iter().enumerate() {
        let start = cursor;
        if document.bytes.get(cursor..cursor + name.len()) != Some(name.as_bytes()) {
            return Err(Failure::extraction("lexical-class-name"));
        }
        cursor += name.len();
        if document.bytes.get(cursor..cursor + 2) != Some(b" `") {
            return Err(Failure::extraction("lexical-class-pattern"));
        }
        cursor += 2;
        if document.bytes.get(cursor..cursor + pattern.len()) != Some(pattern) {
            return Err(Failure::extraction("lexical-class-pattern"));
        }
        cursor += pattern.len();
        if document.bytes.get(cursor) != Some(&b'`') {
            return Err(Failure::extraction("lexical-class-pattern"));
        }
        cursor += 1;
        let modifier = name == "IDENT"
            && document
                .bytes
                .get(cursor..)
                .is_some_and(|tail| tail.starts_with(IDENT_EXCLUSION));
        if modifier {
            cursor += IDENT_EXCLUSION.len();
            fixed_lowerword_partition = true;
        }
        let semantic_end = cursor;
        let annotation = if name == "OPNAME" && fixed_lowerword_partition {
            OPNAME_PARTITIONED_ANNOTATION
        } else {
            default_annotation
        };
        if document.bytes.get(cursor..cursor + annotation.len()) != Some(annotation) {
            return Err(Failure::extraction("lexical-class-annotation"));
        }
        cursor += annotation.len();
        let exclude = if modifier { "fixed-lowerwords" } else { "none" };
        let pattern = core::str::from_utf8(pattern)
            .map_err(|_| Failure::internal("lexical-pattern-ascii"))?;
        push_lexical(
            output,
            Lexical {
                owner,
                name,
                kind: LexKind::Regex,
                span: Span {
                    start,
                    end: semantic_end,
                },
                predicate: format!("pattern={pattern};exclude={exclude}"),
                excludes_fixed_lowerwords: modifier,
                table_spellings: Vec::new(),
            },
            limits,
        )?;
        let delimiter = if index + 1 == SLOTS.len() {
            b".".as_slice()
        } else {
            b"; ".as_slice()
        };
        if document.bytes.get(cursor..cursor + delimiter.len()) != Some(delimiter) {
            return Err(Failure::extraction("lexical-classes-tail"));
        }
        cursor += delimiter.len();
    }
    if cursor != line.end {
        return Err(Failure::extraction("lexical-classes-tail"));
    }
    Ok(())
}

fn extract_literals<'a>(
    document: &Document<'a>,
    grammar: &mut Grammar<'a>,
    limits: &Limits,
    output: &mut Vec<Lexical<'a>>,
) -> Result<(), Failure> {
    let cue = find_unique(document.bytes, LITERALS_CUE, "literal-cue-missing")?;
    let line = line_bounds(document, cue)?;
    let float_dialect = float_contract::extract(document)?;
    let owner = surface(
        grammar,
        document,
        Span {
            start: cue,
            end: cue + LITERALS_CUE.len(),
        },
    )?;
    push_lexical(
        output,
        Lexical {
            owner,
            name: "literal",
            kind: LexKind::LiteralUnion(float_dialect),
            span: Span {
                start: cue,
                end: line.end,
            },
            predicate: format!(
                "integer=-?[0-9]+_TYPE;{};unit=unit;generic=0_T,1_T",
                float_dialect.predicate()
            ),
            excludes_fixed_lowerwords: false,
            table_spellings: Vec::new(),
        },
        limits,
    )?;
    let string_start = cue
        + find_unique(
            &document.bytes[cue..line.end],
            b"STRING `\"...\"`",
            "string-start",
        )?;
    let end_marker = b"non-ASCII diagnostic text is DEFERRED.";
    let string_end = cue
        + find_unique(&document.bytes[cue..line.end], end_marker, "string-end")?
        + end_marker.len();
    push_lexical(
        output,
        Lexical {
            owner,
            name: "STRING",
            kind: LexKind::ByteString,
            span: Span {
                start: string_start,
                end: string_end,
            },
            predicate: "range=32-126;exclude=34,92;escapes=backslash,quote,n;contexts=doc,check"
                .to_owned(),
            excludes_fixed_lowerwords: false,
            table_spellings: Vec::new(),
        },
        limits,
    )
}

fn table_name_start(bytes: &[u8], suffix_start: usize) -> Option<usize> {
    let start = bytes[..suffix_start]
        .iter()
        .rposition(|byte| !byte.is_ascii_uppercase() && !byte.is_ascii_digit() && *byte != b'_')
        .map_or(0, |value| value + 1);
    (start < suffix_start
        && bytes[start].is_ascii_uppercase()
        && bytes[start..suffix_start]
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_'))
    .then_some(start)
}

fn valid_table_signature(bytes: &[u8]) -> Option<usize> {
    let open = bytes.iter().position(|byte| *byte == b'(')?;
    if open == 0
        || bytes.last() != Some(&b')')
        || !bytes[0].is_ascii_lowercase()
        || !bytes[..open]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        return None;
    }
    let parameters = &bytes[open + 1..bytes.len() - 1];
    if parameters.is_empty() {
        return None;
    }
    let mut cursor = 0;
    loop {
        if !parameters.get(cursor).is_some_and(u8::is_ascii_lowercase) {
            return None;
        }
        cursor += 1;
        if cursor == parameters.len() {
            return Some(open);
        }
        if parameters.get(cursor..cursor + 2) != Some(b", ") {
            return None;
        }
        cursor += 2;
    }
}

fn extract_tables<'a>(
    document: &Document<'a>,
    grammar: &mut Grammar<'a>,
    limits: &Limits,
    output: &mut Vec<Lexical<'a>>,
) -> Result<(), Failure> {
    let suffixes = document
        .bytes
        .windows(TABLE_SUFFIX.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == TABLE_SUFFIX).then_some(offset))
        .collect::<Vec<_>>();
    for suffix_start in suffixes {
        let name_start = table_name_start(document.bytes, suffix_start)
            .ok_or_else(|| Failure::extraction("closed-table-name"))?;
        if grammar.surfaces.iter().any(|surface| {
            surface.kind == SurfaceKind::GrammarFence
                && surface.span.start <= name_start
                && name_start < surface.span.end
        }) {
            continue;
        }
        let line = line_bounds(document, name_start)?;
        if !document.bytes[line.start..name_start]
            .iter()
            .filter(|byte| **byte == b'`')
            .count()
            .is_multiple_of(2)
        {
            continue;
        }
        let name = core::str::from_utf8(&document.bytes[name_start..suffix_start])
            .map_err(|_| Failure::internal("validated-utf8"))?;
        let cue_end = suffix_start + TABLE_SUFFIX.len();
        let owner = surface(
            grammar,
            document,
            Span {
                start: name_start,
                end: cue_end,
            },
        )?;
        let mut cursor = cue_end;
        if document.bytes.get(cursor..cursor + 2) != Some(b" `") {
            return Err(Failure::extraction("closed-table-structure"));
        }
        cursor += 1;
        let mut spellings = Vec::new();
        let mut signatures = Vec::new();
        loop {
            if document.bytes.get(cursor) != Some(&b'`') {
                return Err(Failure::extraction("closed-table-entry"));
            }
            let signature_start = cursor + 1;
            let signature_end = document.bytes[signature_start..line.end]
                .iter()
                .position(|byte| *byte == b'`')
                .map(|relative| signature_start + relative)
                .ok_or_else(|| Failure::extraction("closed-table-entry"))?;
            let signature = &document.bytes[signature_start..signature_end];
            let spelling_end = valid_table_signature(signature)
                .ok_or_else(|| Failure::extraction("closed-table-signature"))?;
            let spelling = core::str::from_utf8(&signature[..spelling_end])
                .map_err(|_| Failure::internal("validated-utf8"))?;
            if spellings.contains(&spelling) {
                return Err(Failure::extraction("closed-table-duplicate"));
            }
            spellings.push(spelling);
            signatures.push(
                core::str::from_utf8(signature)
                    .map_err(|_| Failure::internal("validated-utf8"))?
                    .replace(", ", ","),
            );
            cursor = signature_end + 1;
            if document.bytes.get(cursor) == Some(&b'.') && cursor + 1 == line.end {
                cursor += 1;
                break;
            }
            if document.bytes.get(cursor..cursor + 3) != Some(b", `") {
                return Err(Failure::extraction("closed-table-tail"));
            }
            cursor += 2;
        }
        if cursor != line.end {
            return Err(Failure::extraction("closed-table-tail"));
        }
        push_lexical(
            output,
            Lexical {
                owner,
                name,
                kind: LexKind::ClosedTable,
                span: Span {
                    start: name_start,
                    end: line.end,
                },
                predicate: signatures.join(","),
                excludes_fixed_lowerwords: false,
                table_spellings: spellings,
            },
            limits,
        )?;
    }
    Ok(())
}

fn close_candidates(document: &Document<'_>) -> Result<(), Failure> {
    let required = [CLASSES_CUE, LITERALS_CUE];
    for marker in [
        b"Lexical class".as_slice(),
        b"Lexical classes".as_slice(),
        b"Token class".as_slice(),
        b"Token classes".as_slice(),
        b"Literals, exhaustively".as_slice(),
    ] {
        for (offset, _) in document
            .bytes
            .windows(marker.len())
            .enumerate()
            .filter(|(_, window)| *window == marker)
        {
            if !required.iter().any(|exact| {
                document
                    .bytes
                    .get(offset..)
                    .is_some_and(|tail| tail.starts_with(exact))
            }) {
                return Err(Failure::extraction("lexical-candidate-unsupported"));
            }
        }
    }
    let signal = b" is a closed table";
    for (offset, _) in document
        .bytes
        .windows(signal.len())
        .enumerate()
        .filter(|(_, window)| *window == signal)
    {
        if table_name_start(document.bytes, offset).is_none()
            || document.bytes.get(offset..offset + TABLE_SUFFIX.len()) != Some(TABLE_SUFFIX)
        {
            return Err(Failure::extraction("lexical-candidate-unsupported"));
        }
    }
    Ok(())
}

pub(crate) fn extract<'a>(
    document: &Document<'a>,
    grammar: &mut Grammar<'a>,
    limits: &Limits,
    work: &mut Work,
) -> Result<Vec<Lexical<'a>>, Failure> {
    work.spend(document.bytes.len())?;
    close_candidates(document)?;
    let mut output = Vec::new();
    extract_classes(document, grammar, limits, &mut output)?;
    extract_literals(document, grammar, limits, &mut output)?;
    extract_tables(document, grammar, limits, &mut output)?;
    let mut names = BTreeSet::new();
    for item in &output {
        if !names.insert(item.name) {
            return Err(Failure::extraction("lexical-duplicate"));
        }
    }
    for node in &grammar.nodes {
        if node.kind != NodeKind::Ref {
            continue;
        }
        let name = node.value.expect("ref value");
        if !grammar.symbols.contains_key(name) && !names.contains(name) {
            return Err(Failure::extraction("symbol-unbound"));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{IDENT_EXCLUSION, valid_table_signature};

    #[test]
    fn closed_lexical_shapes_have_no_fallback() {
        assert!(IDENT_EXCLUSION.starts_with(b" excluding every lowercase token"));
        assert_eq!(valid_table_signature(b"identity(f, e)"), Some(8));
        assert_eq!(valid_table_signature(b"identity(foo)"), None);
    }
}
