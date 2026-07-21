//! Bounded tokenization and construction of the closed EBNF tree.

use std::collections::BTreeSet;

use crate::document::{Document, Span};
use crate::wire::{Failure, Limits, Work};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeKind {
    Ref,
    Fixed,
    Pattern,
    Sequence,
    Choice,
    Group,
    Optional,
    Repeat0,
    Repeat1,
}

impl NodeKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Ref => "ref",
            Self::Fixed => "fixed",
            Self::Pattern => "pattern",
            Self::Sequence => "sequence",
            Self::Choice => "choice",
            Self::Group => "group",
            Self::Optional => "optional",
            Self::Repeat0 => "repeat0",
            Self::Repeat1 => "repeat1",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node<'a> {
    pub(crate) kind: NodeKind,
    pub(crate) span: Span,
    pub(crate) value: Option<&'a str>,
    pub(crate) children: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
enum TokenKind<'a> {
    Name(&'a str),
    Quoted(&'a str),
    Left,
    Right,
    Choice,
    Optional,
    Repeat0,
    Repeat1,
}

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    kind: TokenKind<'a>,
    span: Span,
}

fn symbol_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn symbol_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn tokenize<'a>(
    document: &Document<'a>,
    span: Span,
    limits: &Limits,
    work: &mut Work,
) -> Result<Vec<Token<'a>>, Failure> {
    let mut output = Vec::new();
    let mut cursor = span.start;
    while cursor < span.end {
        work.spend(1)?;
        match document.bytes[cursor] {
            b' ' | b'\n' => cursor += 1,
            b'\t' => return Err(Failure::extraction("ebnf-layout")),
            b'#' => {
                if cursor == span.start || document.bytes[cursor - 1] != b' ' {
                    return Err(Failure::extraction("ebnf-comment-position"));
                }
                cursor += 1;
                while cursor < span.end && document.bytes[cursor] != b'\n' {
                    cursor += 1;
                }
            }
            b'"' => {
                let start = cursor;
                cursor += 1;
                let value_start = cursor;
                while cursor < span.end && document.bytes[cursor] != b'"' {
                    if !(0x20..=0x7e).contains(&document.bytes[cursor])
                        || document.bytes[cursor] == b'\\'
                    {
                        return Err(Failure::extraction("quoted-byte"));
                    }
                    cursor += 1;
                }
                if cursor == span.end {
                    return Err(Failure::extraction("quoted-unterminated"));
                }
                if cursor == value_start {
                    return Err(Failure::extraction("quoted-empty"));
                }
                let value = core::str::from_utf8(&document.bytes[value_start..cursor])
                    .map_err(|_| Failure::internal("validated-utf8"))?;
                if value.as_bytes().first() == Some(&b'[')
                    && value.as_bytes().contains(&b']')
                    && value != "[0-9]+"
                {
                    return Err(Failure::extraction("quoted-pattern"));
                }
                cursor += 1;
                output.try_reserve(1).map_err(|_| Failure::allocation())?;
                output.push(Token {
                    kind: TokenKind::Quoted(value),
                    span: Span { start, end: cursor },
                });
            }
            byte if symbol_start(byte) => {
                let start = cursor;
                cursor += 1;
                while cursor < span.end && symbol_continue(document.bytes[cursor]) {
                    cursor += 1;
                }
                let value = core::str::from_utf8(&document.bytes[start..cursor])
                    .map_err(|_| Failure::internal("validated-utf8"))?;
                if value.len() > limits.max_symbol_bytes {
                    return Err(Failure::resource("max-symbol-bytes"));
                }
                output.try_reserve(1).map_err(|_| Failure::allocation())?;
                output.push(Token {
                    kind: TokenKind::Name(value),
                    span: Span { start, end: cursor },
                });
            }
            byte @ (b'(' | b')' | b'|' | b'?' | b'*' | b'+') => {
                let kind = match byte {
                    b'(' => TokenKind::Left,
                    b')' => TokenKind::Right,
                    b'|' => TokenKind::Choice,
                    b'?' => TokenKind::Optional,
                    b'*' => TokenKind::Repeat0,
                    b'+' => TokenKind::Repeat1,
                    _ => unreachable!(),
                };
                output.try_reserve(1).map_err(|_| Failure::allocation())?;
                output.push(Token {
                    kind,
                    span: Span {
                        start: cursor,
                        end: cursor + 1,
                    },
                });
                cursor += 1;
            }
            _ => return Err(Failure::extraction("ebnf-byte")),
        }
    }
    Ok(output)
}

struct Parser<'a, 'b> {
    tokens: &'b [Token<'a>],
    cursor: usize,
    nodes: &'b mut Vec<Node<'a>>,
    limits: &'b Limits,
    work: &'b mut Work,
}

impl<'a> Parser<'a, '_> {
    fn parse(mut self) -> Result<usize, Failure> {
        let root = self.choice(1)?;
        if self.cursor != self.tokens.len() {
            return Err(Failure::extraction("ebnf-trailing"));
        }
        Ok(root)
    }

    fn add(
        &mut self,
        kind: NodeKind,
        span: Span,
        value: Option<&'a str>,
        children: Vec<usize>,
    ) -> Result<usize, Failure> {
        if self.nodes.len() >= self.limits.max_grammar_nodes {
            return Err(Failure::resource("max-grammar-nodes"));
        }
        self.work.spend(1)?;
        self.nodes
            .try_reserve(1)
            .map_err(|_| Failure::allocation())?;
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind,
            span,
            value,
            children,
        });
        Ok(id)
    }

    fn choice(&mut self, depth: usize) -> Result<usize, Failure> {
        self.depth(depth)?;
        let mut children = vec![self.sequence(depth + 1)?];
        while matches!(self.peek(), Some(TokenKind::Choice)) {
            self.cursor += 1;
            children.try_reserve(1).map_err(|_| Failure::allocation())?;
            children.push(self.sequence(depth + 1)?);
        }
        if children.len() == 1 {
            return Ok(children[0]);
        }
        let span = self.cover(&children)?;
        self.add(NodeKind::Choice, span, None, children)
    }

    fn sequence(&mut self, depth: usize) -> Result<usize, Failure> {
        self.depth(depth)?;
        let mut children = Vec::new();
        while !matches!(
            self.peek(),
            None | Some(TokenKind::Choice | TokenKind::Right)
        ) {
            children.try_reserve(1).map_err(|_| Failure::allocation())?;
            children.push(self.postfix(depth + 1)?);
        }
        if children.is_empty() {
            return Err(Failure::extraction("ebnf-empty-branch"));
        }
        if children.len() == 1 {
            return Ok(children[0]);
        }
        let span = self.cover(&children)?;
        self.add(NodeKind::Sequence, span, None, children)
    }

    fn postfix(&mut self, depth: usize) -> Result<usize, Failure> {
        self.depth(depth)?;
        let mut node = self.atom(depth + 1)?;
        let kind = match self.peek() {
            Some(TokenKind::Optional) => Some(NodeKind::Optional),
            Some(TokenKind::Repeat0) => Some(NodeKind::Repeat0),
            Some(TokenKind::Repeat1) => Some(NodeKind::Repeat1),
            _ => None,
        };
        if let Some(kind) = kind {
            let suffix = self.tokens[self.cursor].span;
            self.cursor += 1;
            let child_span = self.nodes[node].span;
            node = self.add(
                kind,
                Span {
                    start: child_span.start,
                    end: suffix.end,
                },
                None,
                vec![node],
            )?;
            if matches!(
                self.peek(),
                Some(TokenKind::Optional | TokenKind::Repeat0 | TokenKind::Repeat1)
            ) {
                return Err(Failure::extraction("ebnf-double-postfix"));
            }
        }
        Ok(node)
    }

    fn atom(&mut self, depth: usize) -> Result<usize, Failure> {
        self.depth(depth)?;
        let token = *self
            .tokens
            .get(self.cursor)
            .ok_or_else(|| Failure::extraction("ebnf-atom"))?;
        match token.kind {
            TokenKind::Name(value) => {
                self.cursor += 1;
                self.add(NodeKind::Ref, token.span, Some(value), Vec::new())
            }
            TokenKind::Quoted(value) => {
                self.cursor += 1;
                let kind = if value == "[0-9]+" {
                    NodeKind::Pattern
                } else {
                    NodeKind::Fixed
                };
                self.add(kind, token.span, Some(value), Vec::new())
            }
            TokenKind::Left => {
                self.cursor += 1;
                let inner = self.choice(depth + 1)?;
                let close = *self
                    .tokens
                    .get(self.cursor)
                    .ok_or_else(|| Failure::extraction("ebnf-group-close"))?;
                if !matches!(close.kind, TokenKind::Right) {
                    return Err(Failure::extraction("ebnf-group-close"));
                }
                self.cursor += 1;
                self.add(
                    NodeKind::Group,
                    Span {
                        start: token.span.start,
                        end: close.span.end,
                    },
                    None,
                    vec![inner],
                )
            }
            _ => Err(Failure::extraction("ebnf-atom")),
        }
    }

    fn peek(&self) -> Option<TokenKind<'a>> {
        self.tokens.get(self.cursor).map(|token| token.kind)
    }

    fn cover(&self, children: &[usize]) -> Result<Span, Failure> {
        let first = self
            .nodes
            .get(children[0])
            .ok_or_else(|| Failure::internal("ebnf-child"))?;
        let last = self
            .nodes
            .get(*children.last().expect("nonempty"))
            .ok_or_else(|| Failure::internal("ebnf-child"))?;
        Ok(Span {
            start: first.span.start,
            end: last.span.end,
        })
    }

    fn depth(&self, depth: usize) -> Result<(), Failure> {
        if depth > self.limits.max_ebnf_depth {
            Err(Failure::resource("max-ebnf-depth"))
        } else {
            Ok(())
        }
    }
}

pub(crate) fn parse_rhs<'a>(
    document: &Document<'a>,
    raw: Span,
    nodes: &mut Vec<Node<'a>>,
    limits: &Limits,
    work: &mut Work,
) -> Result<(usize, Span, usize), Failure> {
    let tokens = tokenize(document, raw, limits, work)?;
    if tokens.is_empty() {
        return Err(Failure::extraction("ebnf-empty"));
    }
    let terminal_count = tokens
        .iter()
        .filter(|token| matches!(token.kind, TokenKind::Quoted(_)))
        .count();
    let rhs = Span {
        start: tokens.first().expect("nonempty").span.start,
        end: tokens.last().expect("nonempty").span.end,
    };
    let root = Parser {
        tokens: &tokens,
        cursor: 0,
        nodes,
        limits,
        work,
    }
    .parse()?;
    Ok((root, rhs, terminal_count))
}

pub(crate) fn fixed_expansion(spelling: &str) -> String {
    fn shape(part: &str) -> &'static str {
        let bytes = part.as_bytes();
        if bytes.first().is_some_and(u8::is_ascii_lowercase)
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
        {
            "lowerword"
        } else if bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            "identifier"
        } else {
            match part {
                "&" => "ampersand",
                "->" => "thin-arrow",
                "=>" => "fat-arrow",
                _ => "punctuation",
            }
        }
    }
    let bytes = spelling.as_bytes();
    let split = bytes.iter().position(|byte| symbol_start(*byte));
    let parts: Vec<&str> = match split {
        Some(index)
            if index > 0
                && bytes[..index].iter().all(|byte| !symbol_continue(*byte))
                && bytes[index..].iter().all(|byte| symbol_continue(*byte)) =>
        {
            vec![&spelling[..index], &spelling[index..]]
        }
        _ => vec![spelling],
    };
    let mut descriptor = String::new();
    for (index, part) in parts.iter().enumerate() {
        if index != 0 {
            descriptor.push(',');
        }
        descriptor.push_str(shape(part));
        descriptor.push(':');
        descriptor.push_str(&crate::hash::hex(part.as_bytes()));
    }
    descriptor
}

pub(crate) fn expanded_fixed_lowerwords<'a>(nodes: &'a [Node<'a>]) -> BTreeSet<&'a str> {
    let mut output = BTreeSet::new();
    for node in nodes {
        if node.kind != NodeKind::Fixed {
            continue;
        }
        let spelling = node.value.expect("fixed value");
        let bytes = spelling.as_bytes();
        let Some(index) = bytes.iter().position(|byte| symbol_start(*byte)) else {
            continue;
        };
        if bytes[index].is_ascii_lowercase()
            && bytes[index..].iter().all(|byte| symbol_continue(*byte))
        {
            output.insert(&spelling[index..]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::fixed_expansion;

    #[test]
    fn compound_fixed_atom_expands_generically() {
        assert_eq!(fixed_expansion("&uniq"), "ampersand:26,lowerword:756e6971");
        assert_eq!(fixed_expansion("->"), "thin-arrow:2d3e");
    }
}
