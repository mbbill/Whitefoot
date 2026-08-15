//! Extract and parse the normative EBNF of a Whitefoot kernel specification.
//!
//! Sixty-nine productions live in the four fenced blocks of GRAM-2..GRAM-5; the
//! remaining four (`const`, `cvalue`, `effects`, `effect`) are written inline
//! in the prose of CONST-1, CONST-2, and EFF-1. Scraping only fenced blocks
//! silently loses those four.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner {
    Gram2,
    Gram3,
    Gram4,
    Gram5,
    Const1,
    Const2,
    Eff1,
}

impl Owner {
    pub fn rust(self) -> &'static str {
        match self {
            Owner::Gram2 => "RuleOwner::Gram2",
            Owner::Gram3 => "RuleOwner::Gram3",
            Owner::Gram4 => "RuleOwner::Gram4",
            Owner::Gram5 => "RuleOwner::Gram5",
            Owner::Const1 => "RuleOwner::Const1",
            Owner::Const2 => "RuleOwner::Const2",
            Owner::Eff1 => "RuleOwner::Eff1",
        }
    }
}

/// One raw production: name, right-hand side text, and the rule that owns it.
#[derive(Clone, Debug)]
pub struct RawProduction {
    pub name: String,
    pub body: String,
    pub owner: Owner,
}

/// Returns the fenced code block that follows `marker` at a line start.
fn fenced_after(text: &str, marker: &str) -> String {
    let start = line_index(text, marker).unwrap_or_else(|| panic!("missing {marker}"));
    let rest = &text[start..];
    let open = rest.find("\n```").expect("fenced block opens");
    let after_open = &rest[open + 4..];
    let body_start = after_open.find('\n').expect("fence line ends") + 1;
    let body = &after_open[body_start..];
    let close = body.find("\n```").expect("fenced block closes");
    body[..close].to_string()
}

fn line_index(text: &str, marker: &str) -> Option<usize> {
    text.match_indices(marker)
        .map(|(index, _)| index)
        .find(|index| *index == 0 || text.as_bytes()[index - 1] == b'\n')
}

/// Splits a block of `name := body` lines, honouring continuation lines.
fn split_block(block: &str, owner: Owner, out: &mut Vec<RawProduction>) {
    let mut current: Option<RawProduction> = None;
    for line in block.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // A definition line carries `:=` before any `"`-quoted terminal.
        let define = line.find(":=").filter(|position| {
            line[..*position]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c.is_whitespace())
        });
        match define {
            Some(position) => {
                if let Some(previous) = current.take() {
                    out.push(previous);
                }
                current = Some(RawProduction {
                    name: line[..position].trim().to_string(),
                    body: line[position + 2..].trim().to_string(),
                    owner,
                });
            }
            None => {
                let entry = current.as_mut().expect("continuation without a definition");
                entry.body.push(' ');
                entry.body.push_str(line.trim());
            }
        }
    }
    if let Some(previous) = current.take() {
        out.push(previous);
    }
}

/// Pulls one inline `name := ...` production out of backticked prose.
fn inline(text: &str, rule: &str, name: &str, owner: Owner) -> RawProduction {
    let start = line_index(text, rule).unwrap_or_else(|| panic!("missing {rule}"));
    let paragraph_end = text[start..]
        .find("\n\n")
        .map(|offset| start + offset)
        .unwrap_or(text.len());
    let paragraph = &text[start..paragraph_end];
    let needle = format!("{name} :=");
    let alternate = format!("{name}:=");
    let at = paragraph
        .find(&needle)
        .or_else(|| paragraph.find(&alternate))
        .unwrap_or_else(|| panic!("missing inline production {name} in {rule}"));
    // The production runs to the closing backtick of its code span.
    let open = paragraph[..at].rfind('`').expect("inline span opens");
    let close = at + paragraph[at..].find('`').expect("inline span closes");
    let span = &paragraph[open + 1..close];
    let body = span
        .split_once(":=")
        .expect("inline production has a body")
        .1
        .trim()
        .to_string();
    RawProduction {
        name: name.to_string(),
        body,
        owner,
    }
}

/// Every normative production in specification-definition order.
pub fn productions(spec: &str) -> Vec<RawProduction> {
    let mut out = Vec::new();
    split_block(&fenced_after(spec, "[GRAM-2]"), Owner::Gram2, &mut out);
    split_block(&fenced_after(spec, "[GRAM-3]"), Owner::Gram3, &mut out);
    split_block(&fenced_after(spec, "[GRAM-4]"), Owner::Gram4, &mut out);
    split_block(&fenced_after(spec, "[GRAM-5]"), Owner::Gram5, &mut out);
    assert_eq!(
        out.len(),
        69,
        "the four fenced blocks define 69 productions"
    );
    out.push(inline(spec, "[CONST-1]", "const", Owner::Const1));
    out.push(inline(spec, "[CONST-2]", "cvalue", Owner::Const2));
    out.push(inline(spec, "[EFF-1]", "effects", Owner::Eff1));
    out.push(inline(spec, "[EFF-1]", "effect", Owner::Eff1));
    out
}

// ---------------------------------------------------------------------------
// EBNF syntax tree
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Ast {
    /// One source terminal occurrence; `"&uniq"` expands to two raw tokens.
    Terminal(Vec<String>),
    /// A reference to another production, by name.
    Reference(String),
    Sequence(Vec<Ast>),
    Choice(Vec<Ast>),
    Group(Box<Ast>),
    Optional(Box<Ast>),
    RepeatZero(Box<Ast>),
    RepeatOne(Box<Ast>),
}

struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Parser<'a> {
    fn skip_space(&mut self) {
        while self.at < self.text.len() && self.text[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_space();
        self.text.get(self.at).copied()
    }

    /// alternation := sequence ("|" sequence)*
    fn alternation(&mut self) -> Ast {
        let mut arms = vec![self.sequence()];
        while self.peek() == Some(b'|') {
            self.at += 1;
            arms.push(self.sequence());
        }
        if arms.len() == 1 {
            arms.pop().expect("one arm")
        } else {
            Ast::Choice(arms)
        }
    }

    fn sequence(&mut self) -> Ast {
        let mut items = Vec::new();
        while let Some(byte) = self.peek() {
            if byte == b'|' || byte == b')' {
                break;
            }
            items.push(self.postfix());
        }
        if items.len() == 1 {
            items.pop().expect("one item")
        } else {
            Ast::Sequence(items)
        }
    }

    fn postfix(&mut self) -> Ast {
        let mut node = self.primary();
        loop {
            match self.text.get(self.at).copied() {
                Some(b'?') => {
                    self.at += 1;
                    node = Ast::Optional(Box::new(node));
                }
                Some(b'*') => {
                    self.at += 1;
                    node = Ast::RepeatZero(Box::new(node));
                }
                Some(b'+') => {
                    self.at += 1;
                    node = Ast::RepeatOne(Box::new(node));
                }
                _ => break,
            }
        }
        node
    }

    fn primary(&mut self) -> Ast {
        let byte = self.peek().expect("primary");
        match byte {
            b'(' => {
                self.at += 1;
                let inner = self.alternation();
                assert_eq!(self.peek(), Some(b')'), "group closes");
                self.at += 1;
                Ast::Group(Box::new(inner))
            }
            b'"' => {
                self.at += 1;
                let start = self.at;
                while self.text[self.at] != b'"' {
                    self.at += 1;
                }
                let spelling = std::str::from_utf8(&self.text[start..self.at])
                    .expect("utf8")
                    .to_string();
                self.at += 1;
                Ast::Terminal(split_spelling(&spelling))
            }
            _ => {
                let start = self.at;
                while self.at < self.text.len()
                    && (self.text[self.at].is_ascii_alphanumeric() || self.text[self.at] == b'_')
                {
                    self.at += 1;
                }
                assert!(self.at > start, "a name has at least one byte");
                let name = std::str::from_utf8(&self.text[start..self.at])
                    .expect("utf8")
                    .to_string();
                Ast::Reference(name)
            }
        }
    }
}

/// One written terminal may cover two raw tokens; `&uniq` is the only such form.
fn split_spelling(spelling: &str) -> Vec<String> {
    if spelling == "&uniq" {
        vec!["&".to_string(), "uniq".to_string()]
    } else {
        vec![spelling.to_string()]
    }
}

pub fn parse_body(body: &str) -> Ast {
    let mut parser = Parser {
        text: body.as_bytes(),
        at: 0,
    };
    let ast = parser.alternation();
    parser.skip_space();
    assert_eq!(parser.at, parser.text.len(), "body fully consumed: {body}");
    ast
}

/// Parses every production body, keyed by name, preserving definition order.
pub fn parse_all(raw: &[RawProduction]) -> (Vec<Ast>, BTreeMap<String, usize>) {
    let mut index = BTreeMap::new();
    for (position, production) in raw.iter().enumerate() {
        let previous = index.insert(production.name.clone(), position);
        assert!(
            previous.is_none(),
            "duplicate production {}",
            production.name
        );
    }
    let trees = raw.iter().map(|entry| parse_body(&entry.body)).collect();
    (trees, index)
}
