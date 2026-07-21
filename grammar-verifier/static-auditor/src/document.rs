//! Closed document validation and numbered-rule ownership.

use crate::wire::{Failure, Limits, Work};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Line {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Rule<'a> {
    pub(crate) id: &'a str,
    pub(crate) span: Span,
}

pub(crate) struct Document<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) lines: Vec<Line>,
    pub(crate) rules: Vec<Rule<'a>>,
}

impl<'a> Document<'a> {
    pub(crate) fn parse(
        bytes: &'a [u8],
        limits: &Limits,
        work: &mut Work,
    ) -> Result<Self, Failure> {
        if bytes.is_empty() || !bytes.ends_with(b"\n") {
            return Err(Failure::input("document-terminal-lf"));
        }
        if bytes.len() > limits.max_document_bytes {
            return Err(Failure::resource("max-document-bytes"));
        }
        work.spend(bytes.len())?;
        if core::str::from_utf8(bytes).is_err() {
            return Err(Failure::input("document-utf8"));
        }
        if bytes.contains(&b'\r') {
            return Err(Failure::input("document-lf"));
        }

        let count = bytes.iter().filter(|byte| **byte == b'\n').count();
        if count > limits.max_lines {
            return Err(Failure::resource("max-lines"));
        }
        let mut lines = Vec::new();
        lines
            .try_reserve_exact(count)
            .map_err(|_| Failure::allocation())?;
        let mut start = 0;
        for (offset, byte) in bytes.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            if offset - start > limits.max_line_bytes {
                return Err(Failure::resource("max-line-bytes"));
            }
            if matches!(bytes.get(offset.wrapping_sub(1)), Some(b' ' | b'\t')) && offset > start {
                return Err(Failure::input("document-trailing-whitespace"));
            }
            lines.push(Line {
                start,
                end: offset + 1,
            });
            start = offset + 1;
        }

        let starts = discover_rule_starts(bytes, &lines, limits, work)?;
        let mut rules = Vec::new();
        rules
            .try_reserve_exact(starts.len())
            .map_err(|_| Failure::allocation())?;
        for (index, (id, rule_start)) in starts.iter().copied().enumerate() {
            let end = starts.get(index + 1).map_or(bytes.len(), |item| item.1);
            rules.push(Rule {
                id,
                span: Span {
                    start: rule_start,
                    end,
                },
            });
        }
        Ok(Self {
            bytes,
            lines,
            rules,
        })
    }

    pub(crate) fn line_content(&self, line: Line) -> &'a [u8] {
        self.bytes[line.start..line.end]
            .strip_suffix(b"\n")
            .expect("validated line has LF")
    }

    pub(crate) fn owner(&self, span: Span) -> Result<Rule<'a>, Failure> {
        self.rules
            .iter()
            .copied()
            .find(|rule| rule.span.start <= span.start && span.end <= rule.span.end)
            .ok_or_else(|| Failure::extraction("grammar-owner"))
    }
}

fn fence_like(line: &[u8]) -> bool {
    let first = line
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t'))
        .unwrap_or(line.len());
    line.get(first..)
        .is_some_and(|tail| tail.starts_with(b"```"))
}

fn valid_rule_id(bytes: &[u8]) -> bool {
    let Some(dash) = bytes.iter().position(|byte| *byte == b'-') else {
        return false;
    };
    dash > 0
        && dash + 1 < bytes.len()
        && bytes[..dash]
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        && bytes[dash + 1..].iter().all(u8::is_ascii_digit)
}

fn rule_id(line: &[u8]) -> Option<&str> {
    if line.first() != Some(&b'[') {
        return None;
    }
    let close = line.iter().position(|byte| *byte == b']')?;
    if !valid_rule_id(&line[1..close]) || !matches!(line.get(close + 1), None | Some(b' ')) {
        return None;
    }
    core::str::from_utf8(&line[1..close]).ok()
}

fn discover_rule_starts<'a>(
    bytes: &'a [u8],
    lines: &[Line],
    limits: &Limits,
    work: &mut Work,
) -> Result<Vec<(&'a str, usize)>, Failure> {
    let mut starts = Vec::new();
    starts
        .try_reserve(limits.max_rules.min(lines.len()))
        .map_err(|_| Failure::allocation())?;
    let mut in_fence = false;
    for line in lines {
        work.spend(1)?;
        let content = &bytes[line.start..line.end - 1];
        if fence_like(content) {
            if !in_fence {
                in_fence = true;
            } else if content == b"```" {
                in_fence = false;
            }
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(id) = rule_id(content) {
            if starts.len() >= limits.max_rules {
                return Err(Failure::resource("max-rules"));
            }
            starts.push((id, line.start));
        }
    }
    if starts.is_empty() {
        return Err(Failure::extraction("rules-missing"));
    }
    let mut ids: Vec<&str> = starts.iter().map(|item| item.0).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Failure::extraction("rule-duplicate"));
    }
    Ok(starts)
}

#[cfg(test)]
mod tests {
    use super::valid_rule_id;

    #[test]
    fn rule_identifiers_are_closed() {
        assert!(valid_rule_id(b"GRAM-1"));
        assert!(valid_rule_id(b"Z9-88"));
        assert!(!valid_rule_id(b"GRAM-X"));
        assert!(!valid_rule_id(b"gram-1"));
    }
}
