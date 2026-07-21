//! Closed grammar-region discovery and source-preserving EBNF parsing.

use std::collections::{BTreeMap, BTreeSet};

use crate::document::{Document, Span};
use crate::ebnf::{Node, parse_rhs};
use crate::wire::{Failure, Limits, Work};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SurfaceKind {
    GrammarFence,
    GrammarInline,
    Assignment,
    LexicalCue,
}

impl SurfaceKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::GrammarFence => "grammar-fence",
            Self::GrammarInline => "grammar-inline",
            Self::Assignment => "assignment",
            Self::LexicalCue => "lexical-cue",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Surface<'a> {
    pub(crate) kind: SurfaceKind,
    pub(crate) span: Span,
    pub(crate) owner: &'a str,
}

#[derive(Clone, Debug)]
pub(crate) struct Production<'a> {
    pub(crate) owner: &'a str,
    pub(crate) lhs: &'a str,
    pub(crate) span: Span,
    pub(crate) rhs: Span,
    pub(crate) root: usize,
}

pub(crate) struct Grammar<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
    pub(crate) productions: Vec<Production<'a>>,
    pub(crate) symbols: BTreeMap<&'a str, usize>,
    pub(crate) surfaces: Vec<Surface<'a>>,
    pub(crate) unclassified_count: usize,
}

#[derive(Clone, Copy)]
struct Head {
    lhs_end: usize,
    operator: usize,
}

#[derive(Clone, Copy)]
enum CandidateHead {
    Definition(Head),
    SingleEquals,
}

#[derive(Clone, Copy)]
struct RawDefinition<'a> {
    owner: &'a str,
    lhs: &'a str,
    start: usize,
    operator: usize,
    raw_end: usize,
}

fn grammar_head(line: &[u8]) -> Option<CandidateHead> {
    if !line.first().is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let mut cursor = 1;
    while line
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        cursor += 1;
    }
    let lhs_end = cursor;
    while line.get(cursor) == Some(&b' ') {
        cursor += 1;
    }
    if line.get(cursor..cursor + 2) == Some(b":=") {
        return Some(CandidateHead::Definition(Head {
            lhs_end,
            operator: cursor,
        }));
    }
    while line.get(cursor) == Some(&b'\t') {
        cursor += 1;
    }
    if line.get(cursor) == Some(&b'=')
        && !matches!(line.get(cursor + 1), Some(b'=' | b'>'))
        && line
            .get(cursor + 1..)
            .is_some_and(|tail| tail.iter().any(|byte| !matches!(byte, b' ' | b'\t')))
    {
        return Some(CandidateHead::SingleEquals);
    }
    None
}

fn fence_like(line: &[u8]) -> bool {
    let first = line
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t'))
        .unwrap_or(line.len());
    line.get(first..)
        .is_some_and(|tail| tail.starts_with(b"```"))
}

fn push_surface<'a>(surfaces: &mut Vec<Surface<'a>>, surface: Surface<'a>) -> Result<(), Failure> {
    surfaces.try_reserve(1).map_err(|_| Failure::allocation())?;
    surfaces.push(surface);
    Ok(())
}

fn push_definition<'a>(
    output: &mut Vec<RawDefinition<'a>>,
    definition: RawDefinition<'a>,
    limits: &Limits,
) -> Result<(), Failure> {
    if output.len() >= limits.max_definitions {
        return Err(Failure::resource("max-definitions"));
    }
    output.try_reserve(1).map_err(|_| Failure::allocation())?;
    output.push(definition);
    Ok(())
}

fn scan_fences<'a>(
    document: &Document<'a>,
    limits: &Limits,
    work: &mut Work,
    definitions: &mut Vec<RawDefinition<'a>>,
    surfaces: &mut Vec<Surface<'a>>,
    claimed: &mut BTreeSet<usize>,
) -> Result<Vec<bool>, Failure> {
    let mut masked = Vec::new();
    masked
        .try_reserve_exact(document.lines.len())
        .map_err(|_| Failure::allocation())?;
    masked.resize(document.lines.len(), false);
    let mut index = 0;
    while index < document.lines.len() {
        work.spend(1)?;
        let opening_line = document.lines[index];
        let opening = document.line_content(opening_line);
        if !fence_like(opening) {
            index += 1;
            continue;
        }
        if opening != b"```" {
            return Err(Failure::extraction("grammar-fence-delimiter"));
        }
        let mut close = index + 1;
        while close < document.lines.len() {
            work.spend(1)?;
            let line = document.line_content(document.lines[close]);
            if line == b"```" {
                break;
            }
            if fence_like(line) {
                return Err(Failure::extraction("grammar-fence-delimiter"));
            }
            close += 1;
        }
        if close == document.lines.len() {
            return Err(Failure::extraction("grammar-fence-unterminated"));
        }
        masked[index..=close].fill(true);
        let span = Span {
            start: opening_line.start,
            end: document.lines[close].end,
        };
        let owner = document.owner(span)?;
        let has_head = document.lines[index + 1..close]
            .iter()
            .any(|line| grammar_head(document.line_content(*line)).is_some());
        let grammar_owned = owner.id.starts_with("GRAM-");
        if has_head && !grammar_owned {
            return Err(Failure::extraction("grammar-fence-owner"));
        }
        if grammar_owned {
            push_surface(
                surfaces,
                Surface {
                    kind: SurfaceKind::GrammarFence,
                    span,
                    owner: owner.id,
                },
            )?;
            let mut pending: Vec<(usize, Head)> = Vec::new();
            for line in &document.lines[index + 1..close] {
                let content = document.line_content(*line);
                if content.is_empty() {
                    continue;
                }
                match grammar_head(content) {
                    Some(CandidateHead::Definition(head)) => {
                        pending.try_reserve(1).map_err(|_| Failure::allocation())?;
                        pending.push((line.start, head));
                    }
                    Some(CandidateHead::SingleEquals) => {
                        return Err(Failure::extraction("grammar-assignment-unsupported"));
                    }
                    None if content.first() == Some(&b'\t') => {
                        return Err(Failure::extraction("grammar-continuation-indent"));
                    }
                    None if content.first() == Some(&b' ') && !pending.is_empty() => {}
                    None => return Err(Failure::extraction("grammar-fence-content")),
                }
            }
            if pending.is_empty() {
                return Err(Failure::extraction("grammar-fence-content"));
            }
            for (ordinal, (line_start, head)) in pending.iter().copied().enumerate() {
                let raw_end = pending
                    .get(ordinal + 1)
                    .map_or(document.lines[close].start, |next| next.0);
                let lhs =
                    core::str::from_utf8(&document.bytes[line_start..line_start + head.lhs_end])
                        .map_err(|_| Failure::internal("validated-utf8"))?;
                push_definition(
                    definitions,
                    RawDefinition {
                        owner: owner.id,
                        lhs,
                        start: line_start,
                        operator: line_start + head.operator,
                        raw_end,
                    },
                    limits,
                )?;
                claimed.insert(line_start + head.operator);
                push_surface(
                    surfaces,
                    Surface {
                        kind: SurfaceKind::Assignment,
                        span: Span {
                            start: line_start + head.operator,
                            end: line_start + head.operator + 2,
                        },
                        owner: owner.id,
                    },
                )?;
            }
        }
        index = close + 1;
    }
    Ok(masked)
}

fn close_raw_candidates(
    document: &Document<'_>,
    masked: &[bool],
    work: &mut Work,
) -> Result<usize, Failure> {
    for (index, line) in document.lines.iter().copied().enumerate() {
        if masked[index] {
            continue;
        }
        work.spend(1)?;
        if matches!(
            grammar_head(document.line_content(line)),
            Some(CandidateHead::SingleEquals)
        ) {
            return Err(Failure::extraction("grammar-assignment-unsupported"));
        }
    }
    Ok(0)
}

fn scan_inline<'a>(
    document: &Document<'a>,
    masked: &[bool],
    limits: &Limits,
    work: &mut Work,
    definitions: &mut Vec<RawDefinition<'a>>,
    surfaces: &mut Vec<Surface<'a>>,
    claimed: &mut BTreeSet<usize>,
) -> Result<(), Failure> {
    for (line_index, line) in document.lines.iter().copied().enumerate() {
        if masked[line_index] {
            continue;
        }
        let content = document.line_content(line);
        let mut cursor = 0;
        while cursor < content.len() {
            work.spend(1)?;
            if content[cursor] != b'`' {
                cursor += 1;
                continue;
            }
            let body_start = cursor + 1;
            let close = content[body_start..]
                .iter()
                .position(|byte| *byte == b'`')
                .map(|relative| body_start + relative)
                .ok_or_else(|| Failure::extraction("inline-code-unterminated"))?;
            let body = &content[body_start..close];
            match grammar_head(body) {
                Some(CandidateHead::Definition(head)) => {
                    if body.windows(2).filter(|pair| *pair == b":=").count() != 1 {
                        return Err(Failure::extraction("grammar-inline-operators"));
                    }
                    let span = Span {
                        start: line.start + body_start,
                        end: line.start + close,
                    };
                    let owner = document.owner(span)?;
                    push_surface(
                        surfaces,
                        Surface {
                            kind: SurfaceKind::GrammarInline,
                            span,
                            owner: owner.id,
                        },
                    )?;
                    let lhs = core::str::from_utf8(&body[..head.lhs_end])
                        .map_err(|_| Failure::internal("validated-utf8"))?;
                    let operator = line.start + body_start + head.operator;
                    push_definition(
                        definitions,
                        RawDefinition {
                            owner: owner.id,
                            lhs,
                            start: line.start + body_start,
                            operator,
                            raw_end: line.start + close,
                        },
                        limits,
                    )?;
                    claimed.insert(operator);
                    push_surface(
                        surfaces,
                        Surface {
                            kind: SurfaceKind::Assignment,
                            span: Span {
                                start: operator,
                                end: operator + 2,
                            },
                            owner: owner.id,
                        },
                    )?;
                }
                Some(CandidateHead::SingleEquals) => {
                    return Err(Failure::extraction("grammar-assignment-unsupported"));
                }
                None => {}
            }
            cursor = close + 1;
        }
    }
    Ok(())
}

pub(crate) fn extract<'a>(
    document: &Document<'a>,
    limits: &Limits,
    work: &mut Work,
) -> Result<Grammar<'a>, Failure> {
    let mut definitions = Vec::new();
    let mut surfaces = Vec::new();
    let mut claimed = BTreeSet::new();
    let masked = scan_fences(
        document,
        limits,
        work,
        &mut definitions,
        &mut surfaces,
        &mut claimed,
    )?;
    let unclassified_count = close_raw_candidates(document, &masked, work)?;
    scan_inline(
        document,
        &masked,
        limits,
        work,
        &mut definitions,
        &mut surfaces,
        &mut claimed,
    )?;
    for (offset, pair) in document.bytes.windows(2).enumerate() {
        if pair == b":=" && !claimed.contains(&offset) {
            return Err(Failure::extraction("grammar-unowned-assignment"));
        }
    }
    if definitions.is_empty() {
        return Err(Failure::extraction("grammar-missing"));
    }
    definitions.sort_unstable_by_key(|definition| definition.start);

    let mut nodes = Vec::new();
    let mut productions = Vec::new();
    productions
        .try_reserve_exact(definitions.len())
        .map_err(|_| Failure::allocation())?;
    let mut symbols = BTreeMap::new();
    let mut terminal_count = 0_usize;
    for definition in definitions {
        if definition.lhs.len() > limits.max_symbol_bytes {
            return Err(Failure::resource("max-symbol-bytes"));
        }
        if symbols.contains_key(definition.lhs) {
            return Err(Failure::extraction("production-duplicate"));
        }
        let (root, rhs, count) = parse_rhs(
            document,
            Span {
                start: definition.operator + 2,
                end: definition.raw_end,
            },
            &mut nodes,
            limits,
            work,
        )?;
        terminal_count = terminal_count
            .checked_add(count)
            .ok_or_else(|| Failure::resource("max-terminal-occurrences"))?;
        if terminal_count > limits.max_terminal_occurrences {
            return Err(Failure::resource("max-terminal-occurrences"));
        }
        let index = productions.len();
        symbols.insert(definition.lhs, index);
        productions.push(Production {
            owner: definition.owner,
            lhs: definition.lhs,
            span: Span {
                start: definition.start,
                end: rhs.end,
            },
            rhs,
            root,
        });
    }
    Ok(Grammar {
        nodes,
        productions,
        symbols,
        surfaces,
        unclassified_count,
    })
}

pub(crate) fn reachable_productions(
    grammar: &Grammar<'_>,
    start: &str,
    work: &mut Work,
) -> Result<Vec<bool>, Failure> {
    let start_index = *grammar
        .symbols
        .get(start)
        .ok_or_else(|| Failure::extraction("program-start-missing"))?;
    let mut reachable = Vec::new();
    reachable
        .try_reserve_exact(grammar.productions.len())
        .map_err(|_| Failure::allocation())?;
    reachable.resize(grammar.productions.len(), false);
    let mut productions = Vec::new();
    productions
        .try_reserve(1)
        .map_err(|_| Failure::allocation())?;
    reachable[start_index] = true;
    productions.push(start_index);
    let mut nodes = Vec::new();
    while let Some(production_index) = productions.pop() {
        work.spend(1)?;
        let root = grammar
            .productions
            .get(production_index)
            .ok_or_else(|| Failure::internal("reachability-production"))?
            .root;
        nodes.try_reserve(1).map_err(|_| Failure::allocation())?;
        nodes.push(root);
        while let Some(node_index) = nodes.pop() {
            work.spend(1)?;
            let node = grammar
                .nodes
                .get(node_index)
                .ok_or_else(|| Failure::internal("reachability-node"))?;
            if node.kind == crate::ebnf::NodeKind::Ref {
                let name = node.value.expect("ref value");
                if let Some(target) = grammar.symbols.get(name).copied()
                    && !reachable[target]
                {
                    reachable[target] = true;
                    productions
                        .try_reserve(1)
                        .map_err(|_| Failure::allocation())?;
                    productions.push(target);
                }
            }
            nodes
                .try_reserve(node.children.len())
                .map_err(|_| Failure::allocation())?;
            nodes.extend(node.children.iter().copied());
        }
    }
    Ok(reachable)
}

pub(crate) fn require_program_reachability(
    grammar: &Grammar<'_>,
    work: &mut Work,
) -> Result<(), Failure> {
    if reachable_productions(grammar, "program", work)?
        .into_iter()
        .all(|value| value)
    {
        Ok(())
    } else {
        Err(Failure::extraction("production-unreachable"))
    }
}
