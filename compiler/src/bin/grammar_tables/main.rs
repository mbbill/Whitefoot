//! Derives `src/syntax/grammar/generated.rs` from the active specification's
//! normative EBNF.
//!
//! The committed strong-LL(2) tables are not hand-editable. They contain
//! hundreds of nodes and decisions plus thousands of provenance-retaining
//! SELECT rows. They
//! have always been produced by a generator, but each grammar task built one
//! offline and deleted it, so nothing checked that the committed tables were
//! still the tables the specification implies. Task 0031 added 84 `]`-closing
//! follow rows by hand; 70 of the resulting predicate pairs place `]` where the
//! grammar cannot derive it, and the whole batch names the wrong provenance
//! node. Nothing detected that for a version.
//!
//! This binary closes that hole. `--check` regenerates from the active
//! specification and compares against the committed file, and the test below
//! runs the same comparison inside `make -C compiler check`, so the invariant
//! "the committed tables are the tables the specification's grammar implies" is
//! machine-checked from now on.
//!
//! Agreement here does not prove the tables are *complete* for a grammar the
//! corpus has never exercised; it proves the committed data and the normative
//! EBNF agree. Parsing real programs remains the completeness oracle.

mod ebnf;
mod model;

use std::collections::BTreeMap;
use std::path::Path;

use model::{First, Follow, Grammar, Kind, Pred, Tok, WordSet, concat, mark_outside, pad};
use whitefoot::{ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT};

/// The committed tables this generator must reproduce.
const COMMITTED_TABLES: &str = include_str!("../../syntax/grammar/generated.rs");

/// `Production` declaration order is the dense index for the current grammar.
/// Retired productions leave this inventory instead of surviving as dormant
/// parser concepts. Specification-definition order is carried by
/// `PRODUCTIONS`.
const ENUM_ORDER: &[&str] = &[
    "program",
    "item",
    "struct_decl",
    "field",
    "enum_decl",
    "variant",
    "vfield_list",
    "vfield",
    "fn_decl",
    "result_binding",
    "contract_block",
    "contract_decl",
    "fn_sig",
    "law",
    "law_arg",
    "conform_decl",
    "const_decl",
    "fn_bind",
    "doc",
    "generics",
    "gparam",
    "region_params",
    "param_list",
    "param",
    "type",
    "rtype",
    "mode",
    "targs",
    "targ",
    "stmt",
    "let_stmt",
    "ordinary_let_rhs",
    "propagate_let_rhs",
    "set_stmt",
    "expr_stmt",
    "return_stmt",
    "loop_stmt",
    "break_stmt",
    "region_stmt",
    "contract_define",
    "give_stmt",
    "match_stmt",
    "value_match",
    "arm",
    "fieldbind_list",
    "fieldbind",
    "expr",
    "atom",
    "call",
    "callee",
    "construct",
    "fieldinit_list",
    "fieldinit",
    "borrow_expr",
    "atom_list",
    "place",
    "pbase",
    "psuffix",
    "const",
    "cvalue",
    "effects",
    "effect",
    "program_kind",
    "input_label",
    "if_stmt",
    "value_if",
    "infix_tail",
    "infix_op",
    "for_stmt",
    "requires_clause",
    "ensures_clause",
    "result_route",
    "replace_let_rhs",
    "effect_path",
    "invariant_stmt",
    "affine_expr",
    "affine_term",
    "affine_factor",
    "affine_add_op",
    "proof_stmt",
    "proof_premise",
];

/// v0.33 deliberately replaces the old pseudo-statement contract grammar.
/// Decision identities are regenerated from source order because none is a
/// source- or artifact-visible language identity.
const HISTORICAL_DECISIONS: &[(usize, usize)] = &[];

/// Productions whose entry frontier carries DIAG-1 construct-entry behaviour.
const CONSTRUCT_ENTRY: &[&str] = &[
    "item",
    "stmt",
    "contract_define",
    "requires_clause",
    "ensures_clause",
];

fn camel(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

struct Row {
    arm: u8,
    first: Tok,
    second: Tok,
}

struct DecisionRecord {
    node: usize,
    production: usize,
    kind: &'static str,
    context: &'static str,
    arms: usize,
    rows: Vec<Row>,
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let mut check = false;
    let mut source: Option<String> = None;
    let mut output: Option<String> = None;
    while let Some(argument) = arguments.next() {
        if argument == "--check" {
            check = true;
        } else if argument == "--output" {
            output = arguments.next();
            if output.is_none() {
                eprintln!("whitefoot-grammar-tables: --output requires a path");
                std::process::exit(1);
            }
        } else {
            source = Some(argument);
        }
    }
    let (path, text) = match source {
        Some(path) => {
            let text = match std::fs::read_to_string(Path::new(&path)) {
                Ok(text) => text,
                Err(error) => {
                    eprintln!("whitefoot-grammar-tables: cannot read {path}: {error}");
                    std::process::exit(1);
                }
            };
            (path, text)
        }
        None => (
            ACTIVE_KERNEL_SPEC_PATH.to_string(),
            ACTIVE_KERNEL_SPEC_TEXT.to_string(),
        ),
    };
    let derived = generate(&path, &text);
    if check {
        if derived == COMMITTED_TABLES {
            println!(
                "committed grammar tables match the grammar of {path}: {} bytes",
                derived.len()
            );
        } else {
            eprintln!(
                "whitefoot-grammar-tables: the committed tables are not the tables the grammar of {path} implies; regenerate them"
            );
            std::process::exit(1);
        }
    } else if let Some(output) = output {
        if let Err(error) = std::fs::write(&output, derived) {
            eprintln!("whitefoot-grammar-tables: cannot write {output}: {error}");
            std::process::exit(1);
        }
    } else {
        print!("{derived}");
    }
}

/// Derives the complete contents of `generated.rs` from one specification.
fn generate(path: &str, specification: &str) -> String {
    let raw = ebnf::productions(specification);
    let names: Vec<String> = raw.iter().map(|entry| entry.name.clone()).collect();
    let (trees, index) = ebnf::parse_all(&raw);
    let mut grammar = Grammar::build(&trees, &names, &index);
    let historical_decisions: Vec<usize> = HISTORICAL_DECISIONS
        .iter()
        .map(|(production_slot, offset)| {
            let name = ENUM_ORDER
                .get(*production_slot)
                .expect("historical production slot exists");
            let production = index[*name];
            grammar.roots[production]
                .checked_add(*offset)
                .expect("historical decision node fits in the arena")
        })
        .collect();
    grammar.assign_decisions(&historical_decisions);
    let grammar = grammar;

    let start = index["program"];
    let first = model::first_sets(&grammar);
    let follow = model::follow_sets(&grammar, &first, start);
    let decisions = build_decisions(&grammar, &first, &follow, &index);
    emit(path, &grammar, &raw, &names, &decisions)
}

/// Which production a node belongs to; roots run in definition order and node
/// ids are definition pre-order, so a production owns a contiguous range.
fn owning_productions(grammar: &Grammar) -> Vec<usize> {
    let mut owner = vec![usize::MAX; grammar.nodes.len()];
    for (production, root) in grammar.roots.iter().enumerate() {
        let end = grammar
            .roots
            .get(production + 1)
            .copied()
            .unwrap_or(grammar.nodes.len());
        for slot in owner.iter_mut().take(end).skip(*root) {
            *slot = production;
        }
    }
    owner
}

fn build_decisions(
    grammar: &Grammar,
    first: &First,
    follow: &Follow,
    index: &BTreeMap<String, usize>,
) -> Vec<DecisionRecord> {
    let owner = owning_productions(grammar);
    let program = index["program"];
    let entry_roots: Vec<usize> = CONSTRUCT_ENTRY
        .iter()
        .map(|name| grammar.roots[index[*name]])
        .collect();
    let entry_targets: Vec<usize> = CONSTRUCT_ENTRY.iter().map(|name| index[*name]).collect();

    let mut out = Vec::new();
    for id in grammar.decision_order.iter().copied() {
        let node = &grammar.nodes[id];
        let outer = mark_outside(&follow.node[id]);
        let (kind, arm_sets): (&'static str, Vec<WordSet>) = match &node.kind {
            Kind::Choice => (
                "DecisionKind::Choice",
                node.children
                    .iter()
                    .map(|child| concat(&first.node[*child], &outer))
                    .collect(),
            ),
            Kind::Optional => (
                "DecisionKind::Optional",
                vec![concat(&first.node[node.children[0]], &outer), outer.clone()],
            ),
            Kind::RepeatZero | Kind::RepeatOne => {
                // Entering consumes one iteration; what follows it is any
                // further iteration and then the continuation, both outside
                // the selected arm.
                let tail = mark_outside(&concat(&first.tail[id], &follow.node[id]));
                let enter = concat(&first.node[node.children[0]], &tail);
                let kind = if matches!(node.kind, Kind::RepeatZero) {
                    "DecisionKind::Repeat0"
                } else {
                    "DecisionKind::Repeat1"
                };
                (kind, vec![enter, outer.clone()])
            }
            _ => unreachable!("only branching nodes own decisions"),
        };

        let context = if owner[id] == program && matches!(node.kind, Kind::RepeatZero) {
            "DecisionContext::ProgramItems"
        } else if entry_roots.contains(&id) || repeats_construct(grammar, node, &entry_targets) {
            "DecisionContext::ConstructEntry"
        } else {
            "DecisionContext::Ordinary"
        };

        let mut rows: Vec<Row> = Vec::new();
        for (arm, set) in arm_sets.iter().enumerate() {
            let arm = u8::try_from(arm).expect("an arm index fits in a byte");
            let mut seen = std::collections::BTreeSet::new();
            for word in set {
                let (first, second) = pad(word);
                if seen.insert((first, second)) {
                    rows.push(Row { arm, first, second });
                }
            }
        }
        rows.sort_by(|left, right| {
            (left.arm, left.first, left.second).cmp(&(right.arm, right.first, right.second))
        });
        out.push(DecisionRecord {
            node: id,
            production: owner[id],
            kind,
            context,
            arms: arm_sets.len(),
            rows,
        });
    }
    out
}

/// Whether a repetition repeats one of the construct-entry productions.
fn repeats_construct(grammar: &Grammar, node: &model::Node, targets: &[usize]) -> bool {
    if !matches!(node.kind, Kind::RepeatZero | Kind::RepeatOne) {
        return false;
    }
    matches!(
        grammar.nodes[node.children[0]].kind,
        Kind::Production(target) if targets.contains(&target)
    )
}

fn atom_text(token: Tok) -> String {
    let provenance = match token.prov {
        Some(node) => format!("Some(GrammarNodeId::new({node}))"),
        None => "None".to_string(),
    };
    let name = match token.tname {
        Some(name) => format!("Some(NamePredicate::{name})"),
        None => "None".to_string(),
    };
    format!(
        "SelectAtom::new({}, {provenance}, {}, {name}, {})",
        token.pred.rust(),
        token.inside,
        token.atom_only
    )
}

fn table<T>(
    out: &mut String,
    name: &str,
    element: &str,
    items: &[T],
    render: impl Fn(&T) -> String,
) {
    emit_table(out, "const", name, element, items, render);
}

/// A table too large for a `const`, whose every use is a borrow.
///
/// A `const` array is re-materialized at each use, so clippy's
/// `large_const_arrays` rejects one past its size threshold; `SELECT_ROWS`
/// crossed it at v0.23. The storage class is written at the call site rather
/// than guessed from a threshold the generator cannot see.
fn static_table<T>(
    out: &mut String,
    name: &str,
    element: &str,
    items: &[T],
    render: impl Fn(&T) -> String,
) {
    emit_table(out, "static", name, element, items, render);
}

fn emit_table<T>(
    out: &mut String,
    storage: &str,
    name: &str,
    element: &str,
    items: &[T],
    render: impl Fn(&T) -> String,
) {
    out.push_str(&format!(
        "#[rustfmt::skip]\npub(crate) {storage} {name}: [{element}; {}] = [\n",
        items.len()
    ));
    for item in items {
        out.push_str(&format!("    {},\n", render(item)));
    }
    out.push_str("];\n\n");
}

fn emit(
    path: &str,
    grammar: &Grammar,
    raw: &[ebnf::RawProduction],
    names: &[String],
    decisions: &[DecisionRecord],
) -> String {
    let mut children_arena: Vec<usize> = Vec::new();
    let mut terminal_arena: Vec<Pred> = Vec::new();
    let mut ranges: Vec<(usize, usize)> = vec![(0, 0); grammar.nodes.len()];
    for (id, node) in grammar.nodes.iter().enumerate() {
        match &node.kind {
            Kind::Terminal(preds) => {
                ranges[id] = (terminal_arena.len(), preds.len());
                terminal_arena.extend(preds.iter().copied());
            }
            Kind::Production(_) => ranges[id] = (0, 0),
            _ => {
                ranges[id] = (children_arena.len(), node.children.len());
                children_arena.extend(node.children.iter().copied());
            }
        }
    }

    let mut enum_names: Vec<String> = ENUM_ORDER
        .iter()
        .filter(|name| names.iter().any(|entry| entry == *name))
        .map(|name| (*name).to_string())
        .collect();
    for name in names {
        if !enum_names.contains(name) {
            enum_names.push(name.clone());
        }
    }
    let position = |name: &String| names.iter().position(|entry| entry == name).expect("known");

    let mut out = String::new();
    out.push_str(&format!("// Generated from the grammar in {path}.\n"));
    out.push_str(
        "use crate::syntax::grammar::{\n    Decision, DecisionContext, DecisionKind, GrammarNode, GrammarNodeId, GrammarNodeKind,\n    LookaheadPredicate, NamePredicate, RuleOwner, SelectAtom, SelectRow,\n};\nuse crate::syntax::terminal::{FixedTerminal, TerminalPredicate};\n\n",
    );
    out.push_str("/// One normative production of the active specification grammar.\n///\n/// The declaration order is the dense compiler-local index for the current\n/// grammar. Retired productions leave the inventory; these indices are never\n/// serialized. Specification-definition order is carried by `PRODUCTIONS`.\n#[derive(Clone, Copy, Debug, Eq, PartialEq)]\npub enum Production {\n");
    for name in &enum_names {
        out.push_str(&format!("    /// The `{name}` production.\n"));
        out.push_str(&format!("    {},\n", camel(name)));
    }
    out.push_str("}\n\nimpl Production {\n    pub(crate) const fn index(self) -> usize {\n        self as usize\n    }\n}\n\n");

    table(&mut out, "PRODUCTIONS", "Production", names, |name| {
        format!("Production::{}", camel(name))
    });
    table(
        &mut out,
        "PRODUCTION_ROOTS",
        "GrammarNodeId",
        &enum_names,
        |name| format!("GrammarNodeId::new({})", grammar.roots[position(name)]),
    );
    table(
        &mut out,
        "PRODUCTION_OWNERS",
        "RuleOwner",
        &enum_names,
        |name| raw[position(name)].owner.rust().to_string(),
    );
    table(
        &mut out,
        "GRAMMAR_CHILDREN",
        "GrammarNodeId",
        &children_arena,
        |child| format!("GrammarNodeId::new({child})"),
    );
    table(
        &mut out,
        "GRAMMAR_TERMINALS",
        "LookaheadPredicate",
        &terminal_arena,
        |pred| pred.rust(),
    );

    let node_rows: Vec<String> = grammar
        .nodes
        .iter()
        .enumerate()
        .map(|(id, node)| {
            let kind = match &node.kind {
                Kind::Terminal(_) => "GrammarNodeKind::TerminalSequence".to_string(),
                Kind::Production(target) => format!(
                    "GrammarNodeKind::Production(Production::{})",
                    camel(&names[*target])
                ),
                Kind::Sequence => "GrammarNodeKind::Sequence".to_string(),
                Kind::Choice => "GrammarNodeKind::Choice".to_string(),
                Kind::Group => "GrammarNodeKind::Group".to_string(),
                Kind::Optional => "GrammarNodeKind::Optional".to_string(),
                Kind::RepeatZero => "GrammarNodeKind::RepeatZero".to_string(),
                Kind::RepeatOne => "GrammarNodeKind::RepeatOne".to_string(),
            };
            let decision = match node.decision {
                Some(value) => format!("Some({value})"),
                None => "None".to_string(),
            };
            let (start, len) = ranges[id];
            format!(
                "GrammarNode::new({kind}, {start}, {len}, {decision}, {})",
                node.atom_only
            )
        })
        .collect();
    table(
        &mut out,
        "GRAMMAR_NODES",
        "GrammarNode",
        &node_rows,
        |row| row.clone(),
    );

    let mut atoms: Vec<Tok> = Vec::new();
    let mut atom_index: BTreeMap<Tok, usize> = BTreeMap::new();
    let mut rows: Vec<String> = Vec::new();
    let mut records: Vec<String> = Vec::new();
    let mut start = 0_usize;
    for decision in decisions {
        for row in &decision.rows {
            let mut resolve = |token: Tok| -> usize {
                *atom_index.entry(token).or_insert_with(|| {
                    atoms.push(token);
                    atoms.len() - 1
                })
            };
            let first = resolve(row.first);
            let second = resolve(row.second);
            rows.push(format!("SelectRow::new({}, {first}, {second})", row.arm));
        }
        records.push(format!(
            "Decision::new(GrammarNodeId::new({}), Production::{}, {}, {}, {}, {start}, {})",
            decision.node,
            camel(&names[decision.production]),
            decision.kind,
            decision.context,
            decision.arms,
            decision.rows.len()
        ));
        start += decision.rows.len();
    }
    table(&mut out, "DECISIONS", "Decision", &records, |row| {
        row.clone()
    });
    table(&mut out, "SELECT_ATOMS", "SelectAtom", &atoms, |token| {
        atom_text(*token)
    });
    static_table(&mut out, "SELECT_ROWS", "SelectRow", &rows, |row| {
        row.clone()
    });

    let mut order: Vec<Pred> = Vec::new();
    for pred in &terminal_arena {
        if !order.contains(pred) {
            order.push(*pred);
        }
    }
    table(
        &mut out,
        "DIAGNOSTIC_ORDER",
        "LookaheadPredicate",
        &order,
        |pred| pred.rust(),
    );
    // The final table ends the file; drop its trailing blank line.
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT, COMMITTED_TABLES, generate};

    /// The committed tables must be the tables the active grammar implies.
    ///
    /// This is the check that was missing when task 0031 hand-added 84
    /// `]`-closing follow rows, 70 of whose predicate pairs the grammar cannot
    /// derive. Regenerate with `cargo run --bin whitefoot-grammar-tables`.
    #[test]
    fn committed_tables_are_derived_from_the_active_grammar() {
        let derived = generate(ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT);
        assert_eq!(
            derived, COMMITTED_TABLES,
            "committed grammar tables differ from the active specification grammar"
        );
    }
}
