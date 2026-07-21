//! Exact membership and pairwise intersections for every accepted terminal predicate.

use std::collections::{BTreeMap, BTreeSet};

use crate::ebnf::{NodeKind, expanded_fixed_lowerwords};
use crate::float_contract::{self, Dialect as FloatDialect};
use crate::grammar::Grammar;
use crate::lexical::{LexKind, Lexical};
use crate::wire::{Failure, Work};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Predicate {
    Fixed(String),
    Digits,
    Lex(String),
    End,
}

impl Predicate {
    pub(crate) fn descriptor(&self) -> String {
        match self {
            Self::Fixed(spelling) => format!("fixed:{}", crate::hash::hex(spelling.as_bytes())),
            Self::Digits => "pattern:digits".to_owned(),
            Self::Lex(name) => format!("lex:{}", crate::hash::hex(name.as_bytes())),
            Self::End => "end".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Word(pub(crate) Vec<Predicate>);

impl Word {
    pub(crate) fn descriptor(&self) -> String {
        if self.0.is_empty() {
            return "empty".to_owned();
        }
        self.0
            .iter()
            .map(Predicate::descriptor)
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub(crate) struct Context {
    ident_excludes_fixed: bool,
    float_dialect: Option<FloatDialect>,
    lowerwords: BTreeSet<String>,
    tables: BTreeMap<String, BTreeSet<String>>,
}

impl Context {
    pub(crate) fn new(grammar: &Grammar<'_>, lexical: &[Lexical<'_>]) -> Self {
        let lowerwords = expanded_fixed_lowerwords(&grammar.nodes)
            .into_iter()
            .map(str::to_owned)
            .collect();
        let tables = lexical
            .iter()
            .filter(|item| !item.table_spellings.is_empty())
            .map(|item| {
                (
                    item.name.to_owned(),
                    item.table_spellings
                        .iter()
                        .map(|value| (*value).to_owned())
                        .collect(),
                )
            })
            .collect();
        Self {
            ident_excludes_fixed: lexical
                .iter()
                .any(|item| item.name == "IDENT" && item.excludes_fixed_lowerwords),
            float_dialect: lexical.iter().find_map(|item| match item.kind {
                LexKind::LiteralUnion(dialect) => Some(dialect),
                LexKind::Regex | LexKind::ByteString | LexKind::ClosedTable => None,
            }),
            lowerwords,
            tables,
        }
    }
}

fn lower_identifier(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn string_literal(bytes: &[u8]) -> bool {
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return false;
    }
    let mut cursor = 1;
    while cursor + 1 < bytes.len() {
        let byte = bytes[cursor];
        if byte == b'\\' {
            if !matches!(bytes.get(cursor + 1), Some(b'\\' | b'"' | b'n')) {
                return false;
            }
            cursor += 2;
        } else if !(0x20..=0x7e).contains(&byte) || matches!(byte, b'"' | b'\\') {
            return false;
        } else {
            cursor += 1;
        }
    }
    cursor + 1 == bytes.len()
}

fn integer_literal(bytes: &[u8]) -> bool {
    let Some(split) = bytes.iter().rposition(|byte| *byte == b'_') else {
        return false;
    };
    let suffix = &bytes[split + 1..];
    if !matches!(
        suffix,
        b"i8" | b"i16" | b"i32" | b"i64" | b"u8" | b"u16" | b"u32" | b"u64"
    ) {
        return false;
    }
    let negative = bytes.first() == Some(&b'-');
    let digits = &bytes[usize::from(negative)..split];
    !digits.is_empty() && digits.iter().all(u8::is_ascii_digit)
}

fn literal(bytes: &[u8], float_dialect: FloatDialect) -> bool {
    matches!(bytes, b"unit" | b"0_T" | b"1_T")
        || integer_literal(bytes)
        || float_contract::accepts(bytes, float_dialect)
}

pub(crate) fn accepts(predicate: &Predicate, spelling: &[u8], context: &Context) -> bool {
    match predicate {
        Predicate::Fixed(value) => spelling == value.as_bytes(),
        Predicate::Digits => !spelling.is_empty() && spelling.iter().all(u8::is_ascii_digit),
        Predicate::End => spelling.is_empty(),
        Predicate::Lex(name) => match name.as_str() {
            "IDENT" => {
                lower_identifier(spelling)
                    && (!context.ident_excludes_fixed
                        || !context
                            .lowerwords
                            .contains(core::str::from_utf8(spelling).unwrap_or("")))
            }
            "TYPEID" => {
                spelling.first().is_some_and(u8::is_ascii_uppercase)
                    && spelling[1..].iter().all(u8::is_ascii_alphanumeric)
            }
            "REGIONID" => spelling.first() == Some(&b'\'') && lower_identifier(&spelling[1..]),
            "LABEL" => spelling.first() == Some(&b'@') && lower_identifier(&spelling[1..]),
            "OPNAME" => [
                b".wrap".as_slice(),
                b".trap".as_slice(),
                b".checked".as_slice(),
                b".sat".as_slice(),
                b".strict".as_slice(),
            ]
            .iter()
            .any(|suffix| spelling.strip_suffix(*suffix).is_some_and(lower_identifier)),
            "literal" => context
                .float_dialect
                .is_some_and(|dialect| literal(spelling, dialect)),
            "STRING" => string_literal(spelling),
            table => context.tables.get(table).is_some_and(|spellings| {
                core::str::from_utf8(spelling)
                    .ok()
                    .is_some_and(|value| spellings.contains(value))
            }),
        },
    }
}

fn identifier_witness(context: &Context) -> Vec<u8> {
    let mut candidate = "a".to_owned();
    while context.ident_excludes_fixed && context.lowerwords.contains(&candidate) {
        candidate.push('a');
    }
    candidate.into_bytes()
}

fn lex_intersection(left: &str, right: &str, context: &Context) -> Option<Vec<u8>> {
    if left == right {
        return match left {
            "IDENT" => Some(identifier_witness(context)),
            "TYPEID" => Some(b"A".to_vec()),
            "REGIONID" => Some(b"'a".to_vec()),
            "LABEL" => Some(b"@a".to_vec()),
            "OPNAME" => Some(b"a.wrap".to_vec()),
            "literal" => Some(b"unit".to_vec()),
            "STRING" => Some(b"\"\"".to_vec()),
            table => context
                .tables
                .get(table)
                .and_then(|values| values.first())
                .map(|value| value.as_bytes().to_vec()),
        };
    }
    let pair = if left < right {
        (left, right)
    } else {
        (right, left)
    };
    let known = match pair {
        ("IDENT", "literal") => {
            accepts(&Predicate::Lex("IDENT".to_owned()), b"unit", context).then(|| b"unit".to_vec())
        }
        _ => None,
    };
    if known.is_some() {
        return known;
    }
    match (context.tables.get(left), context.tables.get(right)) {
        (Some(left_values), Some(right_values)) => left_values
            .intersection(right_values)
            .next()
            .map(|value| value.as_bytes().to_vec()),
        (Some(values), None) => values
            .iter()
            .find(|value| accepts(&Predicate::Lex(right.to_owned()), value.as_bytes(), context))
            .map(|value| value.as_bytes().to_vec()),
        (None, Some(values)) => values
            .iter()
            .find(|value| accepts(&Predicate::Lex(left.to_owned()), value.as_bytes(), context))
            .map(|value| value.as_bytes().to_vec()),
        (None, None) => None,
    }
}

pub(crate) fn intersection(
    left: &Predicate,
    right: &Predicate,
    context: &Context,
) -> Option<Vec<u8>> {
    match (left, right) {
        (Predicate::End, Predicate::End) => Some(Vec::new()),
        (Predicate::End, _) | (_, Predicate::End) => None,
        (Predicate::Fixed(left), Predicate::Fixed(right)) => {
            (left == right).then(|| left.as_bytes().to_vec())
        }
        (Predicate::Fixed(value), predicate) | (predicate, Predicate::Fixed(value)) => {
            accepts(predicate, value.as_bytes(), context).then(|| value.as_bytes().to_vec())
        }
        (Predicate::Digits, Predicate::Digits) => Some(b"0".to_vec()),
        (Predicate::Digits, Predicate::Lex(_)) | (Predicate::Lex(_), Predicate::Digits) => None,
        (Predicate::Lex(left), Predicate::Lex(right)) => lex_intersection(left, right, context),
    }
}

pub(crate) fn fixed_predicates(spelling: &str) -> Vec<Predicate> {
    let bytes = spelling.as_bytes();
    let split = bytes
        .iter()
        .position(|byte| byte.is_ascii_alphabetic() || *byte == b'_');
    match split {
        Some(index)
            if index > 0
                && bytes[..index]
                    .iter()
                    .all(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
                && bytes[index..]
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_') =>
        {
            vec![
                Predicate::Fixed(spelling[..index].to_owned()),
                Predicate::Fixed(spelling[index..].to_owned()),
            ]
        }
        _ => vec![Predicate::Fixed(spelling.to_owned())],
    }
}

pub(crate) fn predicate_universe(
    grammar: &Grammar<'_>,
    lexical: &[Lexical<'_>],
) -> BTreeSet<Predicate> {
    let mut predicates = BTreeSet::from([Predicate::End]);
    for node in &grammar.nodes {
        match node.kind {
            NodeKind::Fixed => {
                predicates.extend(fixed_predicates(node.value.expect("fixed value")));
            }
            NodeKind::Pattern => {
                predicates.insert(Predicate::Digits);
            }
            NodeKind::Ref
            | NodeKind::Sequence
            | NodeKind::Choice
            | NodeKind::Group
            | NodeKind::Optional
            | NodeKind::Repeat0
            | NodeKind::Repeat1 => {}
        }
    }
    predicates.extend(
        lexical
            .iter()
            .map(|definition| Predicate::Lex(definition.name.to_owned())),
    );
    predicates
}

fn lower_identifier_end(source: &[u8], start: usize) -> Option<usize> {
    if !source.get(start).is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let mut cursor = start + 1;
    while source
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        cursor += 1;
    }
    Some(cursor)
}

fn string_prefix_end(source: &[u8], start: usize) -> Option<usize> {
    if source.get(start) != Some(&b'"') {
        return None;
    }
    let mut cursor = start + 1;
    while cursor < source.len() {
        match source[cursor] {
            b'"' => return Some(cursor + 1),
            b'\\' if matches!(source.get(cursor + 1), Some(b'\\' | b'"' | b'n')) => cursor += 2,
            byte if (0x20..=0x7e).contains(&byte) && byte != b'\\' => cursor += 1,
            _ => return None,
        }
    }
    None
}

fn suffix_end(source: &[u8], start: usize, suffixes: &[&[u8]]) -> Option<usize> {
    suffixes
        .iter()
        .filter_map(|suffix| {
            source
                .get(start..)
                .is_some_and(|tail| tail.starts_with(suffix))
                .then_some(start + suffix.len())
        })
        .max()
}

fn integer_prefix_end(source: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start;
    if source.get(cursor) == Some(&b'-') {
        cursor += 1;
    }
    let digits_start = cursor;
    while source.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == digits_start || source.get(cursor) != Some(&b'_') {
        return None;
    }
    suffix_end(
        source,
        cursor,
        &[
            b"_i8", b"_i16", b"_i32", b"_i64", b"_u8", b"_u16", b"_u32", b"_u64",
        ],
    )
}

fn literal_prefix_end(source: &[u8], start: usize, float_dialect: FloatDialect) -> Option<usize> {
    let mut end = integer_prefix_end(source, start)
        .into_iter()
        .chain(float_contract::prefix_end(source, start, float_dialect))
        .max();
    for spelling in [b"unit".as_slice(), b"0_T", b"1_T"] {
        if source
            .get(start..)
            .is_some_and(|tail| tail.starts_with(spelling))
        {
            end = Some(end.map_or(start + spelling.len(), |old| {
                old.max(start + spelling.len())
            }));
        }
    }
    end
}

fn lexical_prefix_end(name: &str, source: &[u8], start: usize, context: &Context) -> Option<usize> {
    match name {
        "IDENT" => {
            let end = lower_identifier_end(source, start)?;
            accepts(
                &Predicate::Lex("IDENT".to_owned()),
                &source[start..end],
                context,
            )
            .then_some(end)
        }
        "TYPEID" => {
            if !source.get(start).is_some_and(u8::is_ascii_uppercase) {
                return None;
            }
            let mut cursor = start + 1;
            while source.get(cursor).is_some_and(u8::is_ascii_alphanumeric) {
                cursor += 1;
            }
            Some(cursor)
        }
        "REGIONID" => (source.get(start) == Some(&b'\''))
            .then(|| lower_identifier_end(source, start + 1))
            .flatten(),
        "LABEL" => (source.get(start) == Some(&b'@'))
            .then(|| lower_identifier_end(source, start + 1))
            .flatten(),
        "OPNAME" => {
            let base_end = lower_identifier_end(source, start)?;
            suffix_end(
                source,
                base_end,
                &[b".wrap", b".trap", b".checked", b".sat", b".strict"],
            )
        }
        "literal" => context
            .float_dialect
            .and_then(|dialect| literal_prefix_end(source, start, dialect)),
        "STRING" => string_prefix_end(source, start),
        table => context.tables.get(table).and_then(|spellings| {
            spellings
                .iter()
                .filter_map(|spelling| {
                    source
                        .get(start..)
                        .is_some_and(|tail| tail.starts_with(spelling.as_bytes()))
                        .then_some(start + spelling.len())
                })
                .max()
        }),
    }
}

fn predicate_prefix_end(
    predicate: &Predicate,
    source: &[u8],
    start: usize,
    context: &Context,
) -> Option<usize> {
    match predicate {
        Predicate::Fixed(spelling) => source
            .get(start..)
            .is_some_and(|tail| tail.starts_with(spelling.as_bytes()))
            .then_some(start + spelling.len()),
        Predicate::Digits => {
            if !source.get(start).is_some_and(u8::is_ascii_digit) {
                return None;
            }
            let mut cursor = start + 1;
            while source.get(cursor).is_some_and(u8::is_ascii_digit) {
                cursor += 1;
            }
            Some(cursor)
        }
        Predicate::Lex(name) => lexical_prefix_end(name, source, start, context),
        Predicate::End => (start == source.len()).then_some(start),
    }
}

pub(crate) fn source_lookahead<'a>(
    source: &'a [u8],
    grammar: &Grammar<'_>,
    lexical: &[Lexical<'_>],
    work: &mut Work,
) -> Result<Option<Vec<&'a [u8]>>, Failure> {
    let context = Context::new(grammar, lexical);
    let predicates = predicate_universe(grammar, lexical);
    let mut output = Vec::new();
    output
        .try_reserve_exact(2)
        .map_err(|_| Failure::allocation())?;
    let mut cursor = 0;
    while output.len() < 2 {
        while matches!(source.get(cursor), Some(b' ' | b'\n')) {
            work.spend(1)?;
            cursor += 1;
        }
        if cursor == source.len() {
            output.push(&source[source.len()..]);
            continue;
        }
        let mut longest = None;
        for predicate in &predicates {
            work.spend(1)?;
            if let Some(end) = predicate_prefix_end(predicate, source, cursor, &context) {
                longest = Some(longest.map_or(end, |old: usize| old.max(end)));
            }
        }
        let Some(end) = longest.filter(|end| *end > cursor) else {
            return Ok(None);
        };
        output.push(&source[cursor..end]);
        cursor = end;
    }
    Ok(Some(output))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        Predicate, integer_literal, literal, predicate_universe, source_lookahead, string_literal,
    };
    use crate::document::Span;
    use crate::ebnf::{Node, NodeKind};
    use crate::float_contract::Dialect as FloatDialect;
    use crate::grammar::Grammar;
    use crate::lexical::{LexKind, Lexical};
    use crate::wire::Work;

    #[test]
    fn exact_literal_membership_rejects_near_misses() {
        assert!(integer_literal(b"-128_i8"));
        assert!(integer_literal(b"-129_i8"));
        assert!(integer_literal(b"-1_u8"));
        assert!(integer_literal(b"01_i32"));
        assert!(!integer_literal(b"1_f32"));
        assert!(literal(b"1.5_f64", FloatDialect::V08));
        assert!(literal(b"6.022e23_f64", FloatDialect::Successor));
        assert!(literal(b"01.5_f64", FloatDialect::V08));
        assert!(!literal(b"01.5_f64", FloatDialect::Successor));
        assert!(!literal(b"1.5e+2_f64", FloatDialect::V08));
        assert!(string_literal(b"\"a\\n\""));
        assert!(!string_literal(b"\"a\\t\""));
        assert!(!literal(b"\"\"", FloatDialect::V08));
    }

    #[test]
    fn lookahead_uses_exact_maximal_munch_predicates() {
        let fixed = |value| Node {
            kind: NodeKind::Fixed,
            span: Span { start: 0, end: 1 },
            value: Some(value),
            children: Vec::new(),
        };
        let grammar = Grammar {
            nodes: vec![fixed("->"), fixed(".")],
            productions: Vec::new(),
            symbols: BTreeMap::new(),
            surfaces: Vec::new(),
            unclassified_count: 0,
        };
        let lexical = ["IDENT", "OPNAME"]
            .into_iter()
            .map(|name| Lexical {
                owner: "FORM-3",
                name,
                kind: LexKind::Regex,
                span: Span { start: 0, end: 1 },
                predicate: String::new(),
                excludes_fixed_lowerwords: false,
                table_spellings: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert!(
            predicate_universe(&grammar, &lexical).contains(&Predicate::Lex("IDENT".to_owned()))
        );

        let scan = |source| {
            let mut work = Work::new(1_000);
            source_lookahead(source, &grammar, &lexical, &mut work)
                .expect("bounded scan")
                .expect("complete prefix")
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>()
        };
        assert_eq!(scan(b"->"), vec![b"->".to_vec(), Vec::new()]);
        assert_eq!(scan(b"p.field"), vec![b"p".to_vec(), b".".to_vec()]);
        assert_eq!(
            scan(b"iadd.checked"),
            vec![b"iadd.checked".to_vec(), Vec::new()]
        );

        let literal = Lexical {
            owner: "FORM-5",
            name: "literal",
            kind: LexKind::LiteralUnion(FloatDialect::V08),
            span: Span { start: 0, end: 1 },
            predicate: String::new(),
            excludes_fixed_lowerwords: false,
            table_spellings: Vec::new(),
        };
        let mut work = Work::new(1_000);
        assert_eq!(
            source_lookahead(b"1.5e-2_f64", &grammar, &[literal], &mut work)
                .expect("bounded scan")
                .expect("complete prefix")
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>(),
            vec![b"1.5e-2_f64".to_vec(), Vec::new()]
        );
    }
}
