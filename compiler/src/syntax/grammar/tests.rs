#![allow(clippy::panic)]

use super::{
    DecisionKind, GrammarNodeKind, LookaheadPredicate, Production, diagnostic_terminal_order,
    grammar_node, productions,
};
use crate::syntax::terminal::{ALL_FIXED_TERMINALS, FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};

/// D7's grammar replaces contract/conform/law declarations with formal/actual
/// groups and function arguments. It has 88 productions, 138 decisions and
/// 6,849 select rows. The independent regeneration audit establishes the link
/// to the active specification; these assertions pin its complete inventory.
#[test]
fn complete_inventory_is_pinned() {
    assert_eq!(productions().len(), 88);
    assert_eq!(DECISIONS.len(), 138);
    assert_eq!(SELECT_ROWS.len(), 6_849);
    assert_eq!(diagnostic_terminal_order().len(), 113);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[12], Production::ContractDefine);
    assert_eq!(productions()[13], Production::RequiresClause);
    assert_eq!(productions()[14], Production::EnsuresClause);
    assert_eq!(productions()[15], Production::ResultRoute);
    assert_eq!(productions()[27], Production::RegionParam);
    assert_eq!(productions()[28], Production::LinearityBound);
    assert_eq!(productions()[48], Production::ForStmt);
    assert_eq!(productions()[49], Production::ForBinding);
    assert_eq!(productions()[50], Production::HeaderInvariant);
    assert_eq!(productions()[51], Production::InvariantStmt);
    assert_eq!(productions()[52], Production::ProofUse);
    assert_eq!(productions()[53], Production::UsePremise);
    assert_eq!(productions()[61], Production::DisposeStmt);
    assert_eq!(productions()[70], Production::CompareOp);
    assert_eq!(productions()[78], Production::ClauseExpr);
    assert_eq!(productions()[79], Production::ClauseOp);
    assert_eq!(productions()[86], Production::Effect);
    assert_eq!(productions()[87], Production::EffectPath);
    assert_eq!(Production::ForStmt.index(), 67);
    assert_eq!(Production::ForBinding.index(), 68);
    assert_eq!(Production::HeaderInvariant.index(), 69);
    assert_eq!(Production::RequiresClause.index(), 70);
    assert_eq!(Production::EnsuresClause.index(), 71);
    assert_eq!(Production::ResultRoute.index(), 72);
    assert_eq!(Production::ReplaceLetRhs.index(), 73);
    assert_eq!(Production::EffectPath.index(), 74);
    assert_eq!(Production::InvariantStmt.index(), 75);
    assert_eq!(Production::AffineExpr.index(), 76);
    assert_eq!(Production::AffineTerm.index(), 77);
    assert_eq!(Production::AffineFactor.index(), 78);
    assert_eq!(Production::AffineAddOp.index(), 79);
    assert_eq!(Production::ProofUse.index(), 80);
    assert_eq!(Production::ClauseExpr.index(), 82);
    assert_eq!(Production::ClauseOp.index(), 83);
    assert_eq!(Production::DisposeStmt.index(), 84);
    assert_eq!(Production::RegionParam.index(), 85);
    assert_eq!(Production::LinearityBound.index(), 86);
    assert_eq!(Production::UsePremise.index(), 87);
    assert_eq!(DECISIONS[74].production(), Production::SetStmt);
    assert_eq!(DECISIONS[74].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[76].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[76].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[77].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[77].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[78].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[78].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[80].production(), Production::ForStmt);
    assert_eq!(DECISIONS[80].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[81].production(), Production::ForStmt);
    assert_eq!(DECISIONS[81].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[83].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[83].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[84].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[84].kind(), DecisionKind::Repeat1);
    assert_eq!(DECISIONS[87].production(), Production::UsePremise);
    assert_eq!(DECISIONS[87].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[92].production(), Production::BreakStmt);
    assert_eq!(DECISIONS[92].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[52].production(), Production::Type);
    assert_eq!(DECISIONS[52].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[53].production(), Production::Type);
    assert_eq!(DECISIONS[53].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[54].production(), Production::Type);
    assert_eq!(DECISIONS[54].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[56].production(), Production::Mode);
    assert_eq!(DECISIONS[56].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[57].production(), Production::Mode);
    assert_eq!(DECISIONS[57].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[93].production(), Production::RegionStmt);
    assert_eq!(DECISIONS[93].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[113].production(), Production::BorrowExpr);
    assert_eq!(DECISIONS[113].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[114].production(), Production::BorrowExpr);
    assert_eq!(DECISIONS[114].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[23].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[23].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[24].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[24].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[25].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[25].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[26].production(), Production::EnsuresClause);
    assert_eq!(DECISIONS[26].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[27].production(), Production::ResultRoute);
    assert_eq!(DECISIONS[27].kind(), DecisionKind::Optional);
}

#[test]
fn active_inventory_carries_the_system_interface_grammar() {
    assert!(productions().contains(&Production::ProgramKind));
    assert!(productions().contains(&Production::InputLabel));
    let predicate = LookaheadPredicate::Terminal(TerminalPredicate::Fixed(FixedTerminal::As));
    assert!(diagnostic_terminal_order().contains(&predicate));
}

#[test]
fn fixed_terminal_inventory_follows_first_grammar_occurrence() {
    let derived = diagnostic_terminal_order()
        .iter()
        .filter_map(|predicate| match predicate {
            LookaheadPredicate::Terminal(TerminalPredicate::Fixed(terminal)) => Some(*terminal),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(derived.as_slice(), ALL_FIXED_TERMINALS.as_slice());
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
    assert_eq!(decisions, 138);
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
fn fn_decl_opens_with_the_optional_program_kind() {
    let Some(root) = grammar_node(Production::FnDecl.root()) else {
        panic!("fn_decl root must exist");
    };
    assert_eq!(root.kind(), GrammarNodeKind::Sequence);
    let [program_kind, ..] = root.children() else {
        panic!("fn_decl root must have children");
    };
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
    assert_eq!(DECISIONS.len(), 138);
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
    assert_eq!(total_rows, 6_849);
    assert!(saw_atom_only);
}
