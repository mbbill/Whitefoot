#![allow(clippy::panic)]

use super::{
    DecisionKindV0_15, GrammarNodeKindV0_15, LookaheadPredicateV0_15, ProductionV0_15,
    SYNTAX_DATA_SPEC_V0_15, diagnostic_terminal_order_v0_15, grammar_node_v0_15, productions_v0_15,
};
use crate::KERNEL_SPEC_V0_15_HASH;
use crate::syntax::terminal::{FixedTerminalV0_15, TerminalPredicateV0_15};

use super::generated::{DECISIONS, SELECT_ROWS};

#[test]
fn complete_inventory_is_bound_to_exact_v0_15() {
    assert_eq!(SYNTAX_DATA_SPEC_V0_15, KERNEL_SPEC_V0_15_HASH);
    assert_eq!(productions_v0_15().len(), 62);
    assert_eq!(diagnostic_terminal_order_v0_15().len(), 72);
    assert_eq!(productions_v0_15()[0], ProductionV0_15::Program);
    assert_eq!(productions_v0_15()[61], ProductionV0_15::Effect);
}

#[test]
fn every_decision_has_two_position_rows_and_complete_arm_coverage() {
    let mut decisions = 0_usize;
    for production in productions_v0_15() {
        let mut stack = vec![production.root()];
        while let Some(node_id) = stack.pop() {
            let Some(node) = grammar_node_v0_15(node_id) else {
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
    assert_eq!(decisions, 72);
}

#[test]
fn program_is_one_repeat_decision_over_items() {
    let Some(root) = grammar_node_v0_15(ProductionV0_15::Program.root()) else {
        panic!("program root must exist");
    };
    assert_eq!(root.kind(), GrammarNodeKindV0_15::RepeatZero);
    let Some(decision) = root.decision() else {
        panic!("program repetition must own a decision");
    };
    assert_eq!(decision.kind(), DecisionKindV0_15::Repeat0);
    assert_eq!(decision.arm_count(), 2);
}

#[test]
fn diagnostic_order_contains_no_source_end() {
    assert!(
        diagnostic_terminal_order_v0_15()
            .iter()
            .all(|item| !matches!(item, LookaheadPredicateV0_15::SourceEnd))
    );
}

fn overlaps(left: LookaheadPredicateV0_15, right: LookaheadPredicateV0_15) -> bool {
    if left == right {
        return true;
    }
    matches!(
        (left, right),
        (
            LookaheadPredicateV0_15::Terminal(TerminalPredicateV0_15::Fixed(
                FixedTerminalV0_15::Unit
            )),
            LookaheadPredicateV0_15::Terminal(TerminalPredicateV0_15::Literal)
        ) | (
            LookaheadPredicateV0_15::Terminal(TerminalPredicateV0_15::Literal),
            LookaheadPredicateV0_15::Terminal(TerminalPredicateV0_15::Fixed(
                FixedTerminalV0_15::Unit
            ))
        )
    )
}

#[test]
fn all_detailed_rows_retain_provenance_and_remain_cross_arm_disjoint() {
    assert_eq!(DECISIONS.len(), 72);
    assert_eq!(SELECT_ROWS.len(), 1_839);
    let mut saw_atom_only = false;
    for decision in DECISIONS {
        for row in decision.rows() {
            for position in 0..2 {
                let Some(atom) = row.position(position) else {
                    panic!("every row has exactly two atoms");
                };
                match atom.predicate() {
                    LookaheadPredicateV0_15::Terminal(_) => assert!(atom.provenance().is_some()),
                    LookaheadPredicateV0_15::SourceEnd => assert!(atom.provenance().is_none()),
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
                        .unwrap_or(LookaheadPredicateV0_15::SourceEnd),
                    right
                        .position(0)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicateV0_15::SourceEnd),
                );
                let second_overlaps = overlaps(
                    left.position(1)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicateV0_15::SourceEnd),
                    right
                        .position(1)
                        .map(|atom| atom.predicate())
                        .unwrap_or(LookaheadPredicateV0_15::SourceEnd),
                );
                assert!(!(first_overlaps && second_overlaps));
            }
        }
    }
    assert!(saw_atom_only);
}
