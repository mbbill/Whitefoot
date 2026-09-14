#![allow(clippy::panic)]

use super::{
    DecisionKind, GrammarNodeKind, LookaheadPredicate, Production, diagnostic_terminal_order,
    grammar_node, productions,
};
use crate::syntax::terminal::{ALL_FIXED_TERMINALS, FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};

/// D7's grammar replaces contract/conform/law declarations with formal/actual
/// groups and function arguments. It has 86 productions, 136 decisions and
/// 6,827 select rows. The independent regeneration audit establishes the link
/// to the active specification; these assertions pin its complete inventory.
#[test]
fn complete_inventory_is_pinned() {
    assert_eq!(productions().len(), 86);
    assert_eq!(DECISIONS.len(), 136);
    assert_eq!(SELECT_ROWS.len(), 6_827);
    assert_eq!(diagnostic_terminal_order().len(), 111);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[11], Production::ContractDefine);
    assert_eq!(productions()[12], Production::RequiresClause);
    assert_eq!(productions()[13], Production::EnsuresClause);
    assert_eq!(productions()[14], Production::ResultRoute);
    assert_eq!(productions()[26], Production::RegionParam);
    assert_eq!(productions()[27], Production::LinearityBound);
    assert_eq!(productions()[46], Production::ForStmt);
    assert_eq!(productions()[47], Production::ForBinding);
    assert_eq!(productions()[48], Production::HeaderInvariant);
    assert_eq!(productions()[49], Production::InvariantStmt);
    assert_eq!(productions()[50], Production::ProofUse);
    assert_eq!(productions()[51], Production::UsePremise);
    assert_eq!(productions()[59], Production::DisposeStmt);
    assert_eq!(productions()[68], Production::CompareOp);
    assert_eq!(productions()[76], Production::ClauseExpr);
    assert_eq!(productions()[77], Production::ClauseOp);
    assert_eq!(productions()[84], Production::Effect);
    assert_eq!(productions()[85], Production::EffectPath);
    assert_eq!(Production::ForStmt.index(), 65);
    assert_eq!(Production::ForBinding.index(), 66);
    assert_eq!(Production::HeaderInvariant.index(), 67);
    assert_eq!(Production::RequiresClause.index(), 68);
    assert_eq!(Production::EnsuresClause.index(), 69);
    assert_eq!(Production::ResultRoute.index(), 70);
    assert_eq!(Production::ReplaceLetRhs.index(), 71);
    assert_eq!(Production::EffectPath.index(), 72);
    assert_eq!(Production::InvariantStmt.index(), 73);
    assert_eq!(Production::AffineExpr.index(), 74);
    assert_eq!(Production::AffineTerm.index(), 75);
    assert_eq!(Production::AffineFactor.index(), 76);
    assert_eq!(Production::AffineAddOp.index(), 77);
    assert_eq!(Production::ProofUse.index(), 78);
    assert_eq!(Production::ClauseExpr.index(), 80);
    assert_eq!(Production::ClauseOp.index(), 81);
    assert_eq!(Production::DisposeStmt.index(), 82);
    assert_eq!(Production::RegionParam.index(), 83);
    assert_eq!(Production::LinearityBound.index(), 84);
    assert_eq!(Production::UsePremise.index(), 85);
    assert_eq!(DECISIONS[72].production(), Production::SetStmt);
    assert_eq!(DECISIONS[72].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[74].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[74].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[75].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[75].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[76].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[76].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[78].production(), Production::ForStmt);
    assert_eq!(DECISIONS[78].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[79].production(), Production::ForStmt);
    assert_eq!(DECISIONS[79].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[81].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[81].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[82].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[82].kind(), DecisionKind::Repeat1);
    assert_eq!(DECISIONS[85].production(), Production::UsePremise);
    assert_eq!(DECISIONS[85].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[90].production(), Production::BreakStmt);
    assert_eq!(DECISIONS[90].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[50].production(), Production::Type);
    assert_eq!(DECISIONS[50].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[51].production(), Production::Type);
    assert_eq!(DECISIONS[51].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[52].production(), Production::Type);
    assert_eq!(DECISIONS[52].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[54].production(), Production::Mode);
    assert_eq!(DECISIONS[54].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[55].production(), Production::Mode);
    assert_eq!(DECISIONS[55].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[91].production(), Production::RegionStmt);
    assert_eq!(DECISIONS[91].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[111].production(), Production::BorrowExpr);
    assert_eq!(DECISIONS[111].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[112].production(), Production::BorrowExpr);
    assert_eq!(DECISIONS[112].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[22].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[22].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[23].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[23].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[24].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[24].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[25].production(), Production::EnsuresClause);
    assert_eq!(DECISIONS[25].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[26].production(), Production::ResultRoute);
    assert_eq!(DECISIONS[26].kind(), DecisionKind::Optional);
}

#[test]
fn active_inventory_has_only_ordinary_function_parameters() {
    assert!(productions().contains(&Production::FnDecl));
    assert!(productions().contains(&Production::FnSig));
    assert!(FixedTerminal::from_spelling(b"command").is_none());
    assert!(FixedTerminal::from_spelling(b"as").is_none());
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
    assert_eq!(decisions, 136);
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
fn fn_decl_opens_with_the_ordinary_fn_terminal() {
    let root = grammar_node(Production::FnDecl.root()).expect("fn_decl root");
    assert_eq!(root.kind(), GrammarNodeKind::Sequence);
    let first = grammar_node(root.children()[0]).expect("first fn_decl child");
    assert_eq!(first.kind(), GrammarNodeKind::TerminalSequence);
    assert_eq!(
        first.terminals(),
        &[LookaheadPredicate::Terminal(TerminalPredicate::Fixed(
            FixedTerminal::Fn
        ))]
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
    assert_eq!(DECISIONS.len(), 136);
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
    assert_eq!(total_rows, 6_827);
    assert!(saw_atom_only);
}
