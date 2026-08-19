//! Terminal predicates, grammar nodes, and the strong-LL(2) SELECT machinery.

use std::collections::{BTreeMap, BTreeSet};

use crate::ebnf::Ast;

// ---------------------------------------------------------------------------
// Predicates
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pred {
    Fixed(&'static str),
    Identifier,
    TypeIdentifier,
    RegionIdentifier,
    Label,
    OperationName,
    Literal,
    String,
    Digits,
    SourceEnd,
}

impl Pred {
    pub fn rust(self) -> String {
        match self {
            Pred::SourceEnd => "LookaheadPredicate::SourceEnd".to_string(),
            Pred::Fixed(name) => format!(
                "LookaheadPredicate::Terminal(TerminalPredicate::Fixed(FixedTerminal::{name}))"
            ),
            other => format!(
                "LookaheadPredicate::Terminal(TerminalPredicate::{})",
                other.bare()
            ),
        }
    }

    pub fn bare(self) -> &'static str {
        match self {
            Pred::Identifier => "Identifier",
            Pred::TypeIdentifier => "TypeIdentifier",
            Pred::RegionIdentifier => "RegionIdentifier",
            Pred::Label => "Label",
            Pred::OperationName => "OperationName",
            Pred::Literal => "Literal",
            Pred::String => "String",
            Pred::Digits => "Digits",
            Pred::Fixed(name) => name,
            Pred::SourceEnd => "SourceEnd",
        }
    }

    /// The DIAG-1 name predicate this terminal is, when it is one.
    pub fn name_predicate(self) -> Option<&'static str> {
        match self {
            Pred::Identifier => Some("Identifier"),
            Pred::TypeIdentifier => Some("TypeIdentifier"),
            Pred::RegionIdentifier => Some("RegionIdentifier"),
            Pred::Label => Some("Label"),
            Pred::OperationName => Some("OperationName"),
            _ => None,
        }
    }
}

/// Maps a written terminal spelling to its `FixedTerminal` variant.
pub fn fixed_terminal(spelling: &str) -> Pred {
    const TABLE: &[(&str, &str)] = &[
        ("struct", "Struct"),
        ("{", "LeftBrace"),
        ("}", "RightBrace"),
        (":", "Colon"),
        (";", "Semicolon"),
        ("enum", "Enum"),
        ("(", "LeftParen"),
        (")", "RightParen"),
        (",", "Comma"),
        ("deny_claims", "DenyClaims"),
        ("fn", "Fn"),
        ("->", "ThinArrow"),
        ("requires", "Requires"),
        ("ensures", "Ensures"),
        ("contract", "Contract"),
        ("law", "Law"),
        ("conform", "Conform"),
        ("const", "Const"),
        ("=", "Equal"),
        ("doc", "Doc"),
        ("<", "LeftAngle"),
        (">", "RightAngle"),
        ("[", "LeftBracket"),
        ("]", "RightBracket"),
        ("i8", "I8"),
        ("i16", "I16"),
        ("i32", "I32"),
        ("i64", "I64"),
        ("u8", "U8"),
        ("u16", "U16"),
        ("u32", "U32"),
        ("u64", "U64"),
        ("f32", "F32"),
        ("f64", "F64"),
        ("unit", "Unit"),
        ("array", "Array"),
        ("slice", "Slice"),
        ("box", "Box"),
        ("arena", "Arena"),
        ("buffer", "Buffer"),
        ("own", "Own"),
        ("&", "Ampersand"),
        ("uniq", "Uniq"),
        ("let", "Let"),
        ("propagate", "Propagate"),
        ("replace", "Replace"),
        ("set", "Set"),
        ("return", "Return"),
        ("loop", "Loop"),
        ("break", "Break"),
        ("region", "Region"),
        ("check", "Check"),
        ("else", "Else"),
        ("trap", "Trap"),
        ("give", "Give"),
        ("match", "Match"),
        ("=>", "FatArrow"),
        ("move", "Move"),
        ("deref", "Deref"),
        (".", "Dot"),
        ("pure", "Pure"),
        ("reads", "Reads"),
        ("writes", "Writes"),
        ("allocates", "Allocates"),
        ("heap", "Heap"),
        ("traps", "Traps"),
        ("as", "As"),
        ("external", "External"),
        ("blocks", "Blocks"),
        ("claim", "Claim"),
        ("because", "Because"),
        // FLOOR-5 additions: `if` plus the twenty `infix_op` spellings.
        // `else` already exists (check_stmt). Verified against the fixed
        // delta's [GRAM-5] block, not guessed.
        ("if", "If"),
        ("+", "Plus"),
        ("+defined", "PlusDefined"),
        ("+wrap", "PlusWrap"),
        ("+checked", "PlusChecked"),
        ("+sat", "PlusSat"),
        ("-", "Minus"),
        ("-defined", "MinusDefined"),
        ("-wrap", "MinusWrap"),
        ("-checked", "MinusChecked"),
        ("-sat", "MinusSat"),
        ("*", "Star"),
        ("*defined", "StarDefined"),
        ("*wrap", "StarWrap"),
        ("*checked", "StarChecked"),
        ("*sat", "StarSat"),
        ("/", "Slash"),
        ("/defined", "SlashDefined"),
        ("/checked", "SlashChecked"),
        ("%", "Percent"),
        ("%defined", "PercentDefined"),
        ("%checked", "PercentChecked"),
        ("for", "For"),
        ("in", "In"),
        ("..", "DotDot"),
    ];
    if spelling == "[0-9]+" {
        return Pred::Digits;
    }
    for (text, variant) in TABLE {
        if *text == spelling {
            return Pred::Fixed(variant);
        }
    }
    panic!("no FixedTerminal for spelling {spelling:?}");
}

/// Bare EBNF names that denote a terminal predicate rather than a production.
pub fn bare_terminal(name: &str) -> Option<Pred> {
    match name {
        "IDENT" => Some(Pred::Identifier),
        "TYPEID" => Some(Pred::TypeIdentifier),
        "REGIONID" => Some(Pred::RegionIdentifier),
        "LABEL" => Some(Pred::Label),
        "OPNAME" => Some(Pred::OperationName),
        "STRING" => Some(Pred::String),
        "literal" => Some(Pred::Literal),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Node arena
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Terminal(Vec<Pred>),
    Production(usize),
    Sequence,
    Choice,
    Group,
    Optional,
    RepeatZero,
    RepeatOne,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub children: Vec<usize>,
    pub decision: Option<usize>,
    pub atom_only: bool,
}

pub struct Grammar {
    pub nodes: Vec<Node>,
    /// Node id per decision slot, in decision-index order.
    pub decision_order: Vec<usize>,
    /// Root node per production, in specification-definition order.
    pub roots: Vec<usize>,
    pub index: BTreeMap<String, usize>,
}

impl Grammar {
    pub fn build(trees: &[Ast], names: &[String], index: &BTreeMap<String, usize>) -> Self {
        let mut grammar = Grammar {
            nodes: Vec::new(),
            decision_order: Vec::new(),
            roots: vec![usize::MAX; trees.len()],
            index: index.clone(),
        };
        for (position, tree) in trees.iter().enumerate() {
            let owner = names[position].as_str();
            let root = grammar.lower(tree, owner);
            grammar.roots[position] = root;
        }
        grammar
    }

    /// Assigns decision slots: historical slots first, then new ones in node
    /// order.
    pub fn assign_decisions(&mut self, historical: &[usize]) {
        let mut order: Vec<usize> = historical
            .iter()
            .copied()
            .filter(|id| {
                self.nodes.get(*id).is_some_and(|node| {
                    matches!(
                        node.kind,
                        Kind::Choice | Kind::Optional | Kind::RepeatZero | Kind::RepeatOne
                    )
                })
            })
            .collect();
        for (id, node) in self.nodes.iter().enumerate() {
            if matches!(
                node.kind,
                Kind::Choice | Kind::Optional | Kind::RepeatZero | Kind::RepeatOne
            ) && !order.contains(&id)
            {
                order.push(id);
            }
        }
        for (slot, id) in order.iter().enumerate() {
            self.nodes[*id].decision = Some(slot);
        }
        self.decision_order = order;
    }

    fn push(&mut self) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind: Kind::Sequence,
            children: Vec::new(),
            decision: None,
            atom_only: false,
        });
        id
    }

    fn lower(&mut self, ast: &Ast, owner: &str) -> usize {
        let id = self.push();
        let (kind, children, atom_only) = match ast {
            Ast::Terminal(spellings) => {
                let preds = spellings
                    .iter()
                    .map(|spelling| fixed_terminal(spelling))
                    .collect();
                (Kind::Terminal(preds), Vec::new(), false)
            }
            Ast::Reference(name) => {
                if let Some(pred) = bare_terminal(name) {
                    (Kind::Terminal(vec![pred]), Vec::new(), false)
                } else {
                    let target = *self
                        .index
                        .get(name)
                        .unwrap_or_else(|| panic!("unknown production reference {name}"));
                    // GRAM-9: every `atom` reference outside `expr` is an
                    // atom-only position.
                    let atom_only = name == "atom" && owner != "expr";
                    (Kind::Production(target), Vec::new(), atom_only)
                }
            }
            Ast::Sequence(items) => {
                let kids = items
                    .iter()
                    .map(|item| self.lower(item, owner))
                    .collect::<Vec<_>>();
                (Kind::Sequence, kids, false)
            }
            Ast::Choice(arms) => {
                let kids = arms
                    .iter()
                    .map(|arm| self.lower(arm, owner))
                    .collect::<Vec<_>>();
                (Kind::Choice, kids, false)
            }
            Ast::Group(inner) => {
                let kid = self.lower(inner, owner);
                (Kind::Group, vec![kid], false)
            }
            Ast::Optional(inner) => {
                let kid = self.lower(inner, owner);
                (Kind::Optional, vec![kid], false)
            }
            Ast::RepeatZero(inner) => {
                let kid = self.lower(inner, owner);
                (Kind::RepeatZero, vec![kid], false)
            }
            Ast::RepeatOne(inner) => {
                let kid = self.lower(inner, owner);
                (Kind::RepeatOne, vec![kid], false)
            }
        };
        self.nodes[id].kind = kind;
        self.nodes[id].children = children;
        self.nodes[id].atom_only = atom_only;
        id
    }
}

// ---------------------------------------------------------------------------
// Words with provenance
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tok {
    pub pred: Pred,
    pub prov: Option<usize>,
    pub tname: Option<&'static str>,
    pub atom_only: bool,
    pub inside: bool,
}

pub type Word = Vec<Tok>;
pub type WordSet = BTreeSet<Word>;

fn truncate(mut word: Word) -> Word {
    word.truncate(2);
    word
}

/// Two-truncated concatenation of word sets.
pub fn concat(left: &WordSet, right: &WordSet) -> WordSet {
    let mut out = WordSet::new();
    for first in left {
        if first.len() >= 2 {
            out.insert(first.clone());
            continue;
        }
        for second in right {
            let mut joined = first.clone();
            joined.extend_from_slice(second);
            out.insert(truncate(joined));
        }
    }
    out
}

fn clear_names(set: &WordSet) -> WordSet {
    set.iter()
        .map(|word| {
            word.iter()
                .map(|token| Tok {
                    tname: None,
                    ..*token
                })
                .collect()
        })
        .collect()
}

fn mark_atom_only(set: &WordSet) -> WordSet {
    set.iter()
        .map(|word| {
            word.iter()
                .map(|token| Tok {
                    atom_only: true,
                    ..*token
                })
                .collect()
        })
        .collect()
}

pub fn mark_outside(set: &WordSet) -> WordSet {
    set.iter()
        .map(|word| {
            word.iter()
                .map(|token| Tok {
                    inside: false,
                    ..*token
                })
                .collect()
        })
        .collect()
}

/// FIRST(2) per node and the repetition-tail set per repeat node.
pub struct First {
    pub node: Vec<WordSet>,
    /// For a repeat node, the set of zero-or-more further iterations.
    pub tail: Vec<WordSet>,
}

pub fn first_sets(grammar: &Grammar) -> First {
    let count = grammar.nodes.len();
    let mut first = First {
        node: vec![WordSet::new(); count],
        tail: vec![WordSet::new(); count],
    };
    let empty: Word = Vec::new();
    loop {
        let mut changed = false;
        for id in (0..count).rev() {
            let node = &grammar.nodes[id];
            let computed = match &node.kind {
                Kind::Terminal(preds) => {
                    let word: Word = preds
                        .iter()
                        .map(|pred| Tok {
                            pred: *pred,
                            prov: Some(id),
                            tname: pred.name_predicate(),
                            atom_only: false,
                            inside: true,
                        })
                        .collect();
                    let mut set = WordSet::new();
                    set.insert(truncate(word));
                    set
                }
                Kind::Production(target) => {
                    let root = grammar.roots[*target];
                    if node.atom_only {
                        mark_atom_only(&first.node[root])
                    } else {
                        first.node[root].clone()
                    }
                }
                Kind::Sequence => {
                    let mut set = WordSet::new();
                    set.insert(empty.clone());
                    for child in &node.children {
                        set = concat(&set, &first.node[*child]);
                    }
                    set
                }
                Kind::Choice => {
                    let mut set = WordSet::new();
                    for child in &node.children {
                        set.extend(first.node[*child].iter().cloned());
                    }
                    clear_names(&set)
                }
                Kind::Group => first.node[node.children[0]].clone(),
                Kind::Optional => {
                    let mut set = first.node[node.children[0]].clone();
                    set.insert(empty.clone());
                    set
                }
                Kind::RepeatZero | Kind::RepeatOne => {
                    // tail = zero or more further iterations.
                    let mut tail = WordSet::new();
                    tail.insert(empty.clone());
                    let inner = &first.node[node.children[0]];
                    tail.extend(concat(inner, &first.tail[id]));
                    if first.tail[id] != tail {
                        first.tail[id] = tail.clone();
                        changed = true;
                    }
                    if matches!(node.kind, Kind::RepeatZero) {
                        tail
                    } else {
                        concat(inner, &tail)
                    }
                }
            };
            if first.node[id] != computed {
                first.node[id] = computed;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    first
}

/// The continuation set after each node, and after each production.
pub struct Follow {
    pub node: Vec<WordSet>,
    pub production: Vec<WordSet>,
}

pub fn follow_sets(grammar: &Grammar, first: &First, start: usize) -> Follow {
    let count = grammar.nodes.len();
    let mut follow = Follow {
        node: vec![WordSet::new(); count],
        production: vec![WordSet::new(); grammar.roots.len()],
    };
    // The start production is followed by end of source.
    let mut initial = WordSet::new();
    initial.insert(Vec::new());
    follow.production[start] = initial;

    loop {
        let mut changed = false;
        // Push production-level continuations down to roots.
        for (production, root) in grammar.roots.iter().enumerate() {
            let value = follow.production[production].clone();
            if follow.node[*root] != value {
                let merged: WordSet = follow.node[*root].union(&value).cloned().collect();
                if follow.node[*root] != merged {
                    follow.node[*root] = merged;
                    changed = true;
                }
            }
        }
        for id in 0..count {
            let node = &grammar.nodes[id];
            let outer = follow.node[id].clone();
            match &node.kind {
                Kind::Sequence => {
                    for (position, child) in node.children.iter().enumerate() {
                        let mut set = outer.clone();
                        for later in node.children[position + 1..].iter().rev() {
                            set = concat(&first.node[*later], &set);
                        }
                        changed |= merge(&mut follow.node[*child], &set);
                    }
                }
                Kind::Choice | Kind::Group | Kind::Optional => {
                    for child in &node.children {
                        changed |= merge(&mut follow.node[*child], &outer);
                    }
                }
                Kind::RepeatZero | Kind::RepeatOne => {
                    let set = concat(&first.tail[id], &outer);
                    changed |= merge(&mut follow.node[node.children[0]], &set);
                }
                Kind::Production(target) => {
                    let value = outer.clone();
                    changed |= merge(&mut follow.production[*target], &value);
                }
                Kind::Terminal(_) => {}
            }
        }
        if !changed {
            break;
        }
    }
    follow
}

fn merge(into: &mut WordSet, from: &WordSet) -> bool {
    let before = into.len();
    into.extend(from.iter().cloned());
    into.len() != before
}

/// Pads a word to exactly two tokens with the source-end sentinel.
pub fn pad(word: &Word) -> (Tok, Tok) {
    let end = Tok {
        pred: Pred::SourceEnd,
        prov: None,
        tname: None,
        atom_only: false,
        inside: false,
    };
    let first = word.first().copied().unwrap_or(end);
    let second = word.get(1).copied().unwrap_or(end);
    (first, second)
}
