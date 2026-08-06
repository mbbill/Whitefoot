#![allow(clippy::panic)]

use super::{
    DecisionKind, GrammarNodeKind, GrammarStage, LookaheadPredicate, Production,
    SYNTAX_DATA_SPEC_HASH, diagnostic_terminal_order, grammar_node, productions,
};
use crate::ACTIVE_KERNEL_SPEC_HASH;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};
use super::staged::{STAGED_DECISIONS, STAGED_SELECT_ROWS};

#[test]
fn complete_inventory_is_bound_to_exact() {
    assert_eq!(SYNTAX_DATA_SPEC_HASH, ACTIVE_KERNEL_SPEC_HASH);
    assert_eq!(productions().len(), 62);
    assert_eq!(diagnostic_terminal_order().len(), 72);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[61], Production::Effect);
}

#[test]
fn staged_inventory_extends_the_active_grammar() {
    let staged = GrammarStage::Staged;
    assert_eq!(staged.productions().len(), 64);
    assert_eq!(staged.diagnostic_terminal_order().len(), 75);
    assert!(staged.productions().contains(&Production::ProgramKind));
    assert!(staged.productions().contains(&Production::InputLabel));
    // The two staged productions are absent from the active tables.
    assert!(
        GrammarStage::Active
            .production_root(Production::ProgramKind)
            .is_none()
    );
    assert!(
        GrammarStage::Active
            .production_root(Production::InputLabel)
            .is_none()
    );
    assert!(staged.production_root(Production::ProgramKind).is_some());
    assert!(staged.production_root(Production::InputLabel).is_some());
    // The staged diagnostic order carries the three staged spellings.
    for terminal in [
        FixedTerminal::As,
        FixedTerminal::External,
        FixedTerminal::Blocks,
    ] {
        let predicate = LookaheadPredicate::Terminal(TerminalPredicate::Fixed(terminal));
        assert!(staged.diagnostic_terminal_order().contains(&predicate));
        assert!(!diagnostic_terminal_order().contains(&predicate));
    }
}

fn count_stage_decisions(stage: GrammarStage) -> usize {
    let mut decisions = 0_usize;
    for production in stage.productions() {
        let Some(root) = stage.production_root(*production) else {
            panic!("every stage production must have a root");
        };
        let mut stack = vec![root];
        while let Some(node_id) = stack.pop() {
            let Some(node) = stage.node(node_id) else {
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
    decisions
}

#[test]
fn every_decision_has_two_position_rows_and_complete_arm_coverage() {
    assert_eq!(count_stage_decisions(GrammarStage::Active), 72);
    assert_eq!(count_stage_decisions(GrammarStage::Staged), 74);
}

#[test]
fn program_is_one_repeat_decision_over_items() {
    let Some(root_id) = Production::Program.root() else {
        panic!("program must have an active root");
    };
    let Some(root) = grammar_node(root_id) else {
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
fn staged_fn_decl_opens_with_an_optional_program_kind() {
    let staged = GrammarStage::Staged;
    let Some(root_id) = staged.production_root(Production::FnDecl) else {
        panic!("staged fn_decl must have a root");
    };
    let Some(root) = staged.node(root_id) else {
        panic!("staged fn_decl root must exist");
    };
    assert_eq!(root.kind(), GrammarNodeKind::Sequence);
    let Some(first) = root.children().first().copied() else {
        panic!("staged fn_decl root must have children");
    };
    let Some(first) = staged.node(first) else {
        panic!("staged fn_decl first child must exist");
    };
    assert_eq!(first.kind(), GrammarNodeKind::Optional);
    let Some(content) = first.children().first().copied() else {
        panic!("the optional must contain the program_kind reference");
    };
    let Some(content) = staged.node(content) else {
        panic!("the program_kind reference must exist");
    };
    assert_eq!(
        content.kind(),
        GrammarNodeKind::Production(Production::ProgramKind)
    );
}

#[test]
fn diagnostic_order_contains_no_source_end() {
    for stage in [GrammarStage::Active, GrammarStage::Staged] {
        assert!(
            stage
                .diagnostic_terminal_order()
                .iter()
                .all(|item| !matches!(item, LookaheadPredicate::SourceEnd))
        );
    }
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

fn assert_detailed_rows(decisions: &[super::Decision], expected_rows: usize) {
    let mut total_rows = 0_usize;
    let mut saw_atom_only = false;
    for decision in decisions {
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
    assert_eq!(total_rows, expected_rows);
    assert!(saw_atom_only);
}

#[test]
fn all_detailed_rows_retain_provenance_and_remain_cross_arm_disjoint() {
    assert_eq!(DECISIONS.len(), 72);
    assert_eq!(SELECT_ROWS.len(), 1_839);
    assert_detailed_rows(&DECISIONS, 1_839);
}

#[test]
fn all_staged_rows_retain_provenance_and_remain_cross_arm_disjoint() {
    assert_eq!(STAGED_DECISIONS.len(), 74);
    assert_eq!(STAGED_SELECT_ROWS.len(), 1_925);
    assert_detailed_rows(&STAGED_DECISIONS, 1_925);
}
