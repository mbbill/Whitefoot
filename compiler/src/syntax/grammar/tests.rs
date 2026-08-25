#![allow(clippy::panic)]

use super::{
    DecisionKind, GrammarNodeKind, LookaheadPredicate, Production, diagnostic_terminal_order,
    grammar_node, productions,
};
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};

/// The committed inventory's own shape. That this data belongs to the active
/// specification is checked by regenerating it from the active grammar, in
/// `committed_tables_are_derived_from_the_active_grammar`.
#[test]
fn complete_inventory_is_pinned() {
    assert_eq!(productions().len(), 74);
    assert_eq!(DECISIONS.len(), 93);
    assert_eq!(SELECT_ROWS.len(), 3_659);
    assert_eq!(diagnostic_terminal_order().len(), 103);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[12], Production::ContractDefine);
    assert_eq!(productions()[13], Production::RequiresClause);
    assert_eq!(productions()[14], Production::EnsuresClause);
    assert_eq!(productions()[15], Production::ResultRoute);
    assert_eq!(productions()[46], Production::ForStmt);
    assert_eq!(productions()[73], Production::Effect);
    assert_eq!(Production::ForStmt.index(), 69);
    assert_eq!(Production::RequiresClause.index(), 70);
    assert_eq!(Production::EnsuresClause.index(), 71);
    assert_eq!(Production::ResultRoute.index(), 72);
    assert_eq!(Production::ReplaceLetRhs.index(), 73);
    assert_eq!(DECISIONS[55].production(), Production::ForStmt);
    assert_eq!(DECISIONS[55].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[18].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[18].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[19].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[19].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[20].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[20].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[21].production(), Production::EnsuresClause);
    assert_eq!(DECISIONS[21].kind(), DecisionKind::Optional);
}

#[test]
fn active_inventory_carries_the_system_interface_grammar() {
    assert!(productions().contains(&Production::ProgramKind));
    assert!(productions().contains(&Production::InputLabel));
    let predicate = LookaheadPredicate::Terminal(TerminalPredicate::Fixed(FixedTerminal::As));
    assert!(diagnostic_terminal_order().contains(&predicate));
    for terminal in [FixedTerminal::External, FixedTerminal::Blocks] {
        let predicate = LookaheadPredicate::Terminal(TerminalPredicate::Fixed(terminal));
        assert!(!diagnostic_terminal_order().contains(&predicate));
    }
}

#[test]
fn every_decision_has_two_position_rows_and_complete_arm_coverage() {
    let mut decisions = 0_usize;
    for production in productions() {
        let mut stack = vec![production.root()];
        while let Some(node_id) = stack.pop() {
            let Some(node) = grammar_node(node_id) else {
                panic!("generated node must exist");
            };
            if let Some(decision) = node.decision() {
                decisions += 1;
                let mut seen = vec![false; usize::from(decision.arm_count())];
                for row in decision.rows() {
                    assert!(row.position(0).is_some());
                    assert!(row.position(1).is_some());
                    seen[usize::from(row.arm())] = true;
                }
                assert!(seen.into_iter().all(|value| value));
            }
            stack.extend_from_slice(node.children());
        }
    }
    assert_eq!(decisions, 93);
}

#[test]
fn program_is_one_repeat_decision_over_items() {
    let Some(root) = grammar_node(Production::Program.root()) else {
        panic!("program root must exist");
    };
    assert_eq!(root.kind(), GrammarNodeKind::RepeatZero);
    let Some(decision) = root.decision() else {
        panic!("program repetition must own a decision");
    };
    assert_eq!(decision.kind(), DecisionKind::Repeat0);
    assert_eq!(decision.arm_count(), 2);
}

#[test]
fn fn_decl_opens_with_the_marker_then_the_optional_program_kind() {
    let Some(root) = grammar_node(Production::FnDecl.root()) else {
        panic!("fn_decl root must exist");
    };
    assert_eq!(root.kind(), GrammarNodeKind::Sequence);
    let [marker, program_kind, ..] = root.children() else {
        panic!("fn_decl root must have children");
    };
    let Some(marker) = grammar_node(*marker) else {
        panic!("fn_decl marker child must exist");
    };
    assert_eq!(marker.kind(), GrammarNodeKind::Optional);
    let Some(marker_content) = marker.children().first().copied() else {
        panic!("the marker optional must contain deny_claims");
    };
    let Some(marker_content) = grammar_node(marker_content) else {
        panic!("the marker terminal must exist");
    };
    assert_eq!(marker_content.kind(), GrammarNodeKind::TerminalSequence);
    assert_eq!(
        marker_content.terminals(),
        &[LookaheadPredicate::Terminal(TerminalPredicate::Fixed(
            FixedTerminal::DenyClaims
        ))]
    );

    let Some(program_kind) = grammar_node(*program_kind) else {
        panic!("fn_decl program_kind child must exist");
    };
    assert_eq!(program_kind.kind(), GrammarNodeKind::Optional);
    let Some(content) = program_kind.children().first().copied() else {
        panic!("the second optional must contain the program_kind reference");
    };
    let Some(content) = grammar_node(content) else {
        panic!("the program_kind reference must exist");
    };
    assert_eq!(
        content.kind(),
        GrammarNodeKind::Production(Production::ProgramKind)
    );
}

#[test]
fn diagnostic_order_contains_no_source_end() {
    assert!(
        diagnostic_terminal_order()
            .iter()
            .all(|item| !matches!(item, LookaheadPredicate::SourceEnd))
    );
}

fn overlaps(left: LookaheadPredicate, right: LookaheadPredicate) -> bool {
    if left == right {
        return true;
    }
    matches!(
        (left, right),
        (
            LookaheadPredicate::Terminal(TerminalPredicate::Fixed(FixedTerminal::Unit)),
            LookaheadPredicate::Terminal(TerminalPredicate::Literal)
        ) | (
            LookaheadPredicate::Terminal(TerminalPredicate::Literal),
            LookaheadPredicate::Terminal(TerminalPredicate::Fixed(FixedTerminal::Unit))
        )
    )
}

#[test]
fn all_detailed_rows_retain_provenance_and_remain_cross_arm_disjoint() {
    assert_eq!(DECISIONS.len(), 93);
    let mut total_rows = 0_usize;
    let mut saw_atom_only = false;
    for decision in &DECISIONS {
        total_rows += decision.rows().len();
        for row in decision.rows() {
            for position in 0..2 {
                let Some(atom) = row.position(position) else {
                    panic!("every row has exactly two atoms");
                };
                match atom.predicate() {
                    LookaheadPredicate::Terminal(_) => assert!(atom.provenance().is_some()),
                    LookaheadPredicate::SourceEnd => assert!(atom.provenance().is_none()),
                }
                saw_atom_only |= atom.is_atom_only();
            }
        }
        for (left_index, left) in decision.rows().iter().enumerate() {
            for right in &decision.rows()[left_index + 1..] {
                if left.arm() == right.arm() {
                    continue;
                }
                let first_overlaps = overlaps(
                    left.position(0)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicate::SourceEnd),
                    right
                        .position(0)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicate::SourceEnd),
                );
                let second_overlaps = overlaps(
                    left.position(1)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicate::SourceEnd),
                    right
                        .position(1)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicate::SourceEnd),
                );
                assert!(!(first_overlaps && second_overlaps));
            }
        }
    }
    assert_eq!(total_rows, 3_659);
    assert!(saw_atom_only);
}
