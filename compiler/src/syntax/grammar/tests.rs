#![allow(clippy::panic)]

use super::{
    DecisionContext, DecisionKind, GrammarNodeKind, LookaheadPredicate, Production,
    diagnostic_terminal_order, grammar_node, productions,
};
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};

/// The committed inventory's own shape. That this data belongs to the active
/// specification is checked by regenerating it from the active grammar, in
/// `committed_tables_are_derived_from_the_active_grammar`.
#[test]
fn complete_inventory_is_pinned() {
    assert_eq!(productions().len(), 74);
    assert_eq!(DECISIONS.len(), 96);
    assert_eq!(SELECT_ROWS.len(), 3_788);
    assert_eq!(diagnostic_terminal_order().len(), 99);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[12], Production::EnsuresBlock);
    assert_eq!(productions()[13], Production::EnsuresSelector);
    assert_eq!(productions()[14], Production::EnsuresEntry);
    assert_eq!(productions()[45], Production::ForStmt);
    assert_eq!(productions()[73], Production::Effect);
    assert_eq!(Production::ForStmt.index(), 69);
    assert_eq!(Production::EnsuresBlock.index(), 70);
    assert_eq!(Production::EnsuresSelector.index(), 71);
    assert_eq!(Production::EnsuresEntry.index(), 72);
    assert_eq!(Production::ReplaceLetRhs.index(), 73);
    assert_eq!(DECISIONS[84].production(), Production::ForStmt);
    assert_eq!(DECISIONS[84].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[85].production(), Production::FnDecl);
    assert_eq!(DECISIONS[85].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[86].production(), Production::EnsuresBlock);
    assert_eq!(DECISIONS[86].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[87].production(), Production::EnsuresSelector);
    assert_eq!(DECISIONS[87].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[88].production(), Production::EnsuresSelector);
    assert_eq!(DECISIONS[88].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[89].production(), Production::EnsuresEntry);
    assert_eq!(DECISIONS[89].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[90].production(), Production::FnDecl);
    assert_eq!(DECISIONS[90].kind(), DecisionKind::Optional);
}

#[test]
fn v028_decision_slots_retain_their_exact_shapes() {
    macro_rules! shape {
        ($production:ident, $kind:ident, $context:ident, $arms:literal) => {
            (
                Production::$production,
                DecisionKind::$kind,
                DecisionContext::$context,
                $arms,
            )
        };
    }
    let expected = [
        shape!(Program, Repeat0, ProgramItems, 2),
        shape!(Item, Choice, ConstructEntry, 6),
        shape!(StructDecl, Optional, Ordinary, 2),
        shape!(StructDecl, Optional, Ordinary, 2),
        shape!(StructDecl, Repeat0, Ordinary, 2),
        shape!(EnumDecl, Optional, Ordinary, 2),
        shape!(EnumDecl, Optional, Ordinary, 2),
        shape!(EnumDecl, Repeat0, Ordinary, 2),
        shape!(Variant, Optional, Ordinary, 2),
        shape!(VfieldList, Repeat0, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(FnDecl, Repeat0, ConstructEntry, 2),
        shape!(RequiresBlock, Repeat0, ConstructEntry, 2),
        // v0.32 admits the contract final `check_stmt` directly at the
        // entry, so this choice carries `doc | stmt | check_stmt`.
        shape!(RequiresEntry, Choice, ConstructEntry, 3),
        shape!(ContractDecl, Optional, Ordinary, 2),
        shape!(ContractDecl, Optional, Ordinary, 2),
        shape!(ContractDecl, Repeat0, Ordinary, 2),
        shape!(ContractDecl, Repeat0, Ordinary, 2),
        shape!(FnSig, Optional, Ordinary, 2),
        shape!(FnSig, Optional, Ordinary, 2),
        shape!(Law, Optional, Ordinary, 2),
        shape!(Law, Repeat0, Ordinary, 2),
        shape!(LawArg, Choice, Ordinary, 2),
        shape!(ConformDecl, Optional, Ordinary, 2),
        shape!(ConformDecl, Optional, Ordinary, 2),
        shape!(ConformDecl, Repeat0, Ordinary, 2),
        shape!(Generics, Repeat0, Ordinary, 2),
        shape!(Gparam, Choice, Ordinary, 2),
        shape!(Gparam, Optional, Ordinary, 2),
        shape!(RegionParams, Repeat0, Ordinary, 2),
        shape!(ParamList, Repeat0, Ordinary, 2),
        shape!(Param, Optional, Ordinary, 2),
        shape!(Type, Choice, Ordinary, 17),
        shape!(Type, Optional, Ordinary, 2),
        shape!(Mode, Choice, Ordinary, 3),
        shape!(Targs, Repeat0, Ordinary, 2),
        shape!(Targ, Choice, Ordinary, 3),
        // v0.32 retires the body `check` statement: `check_stmt` left
        // this alternation for the two contract entries.
        shape!(Stmt, Choice, ConstructEntry, 12),
        shape!(InfixOp, Choice, Ordinary, 16),
        shape!(Callee, Choice, Ordinary, 2),
        shape!(Place, Repeat0, Ordinary, 2),
        shape!(Pbase, Choice, Ordinary, 2),
        // v0.31 adds `replace_let_rhs` as a fifth let_stmt alternative;
        // the decision slot and shape are otherwise the v0.28 record.
        shape!(LetStmt, Choice, Ordinary, 5),
        shape!(IfStmt, Repeat0, ConstructEntry, 2),
        shape!(IfStmt, Optional, Ordinary, 2),
        shape!(IfStmt, Choice, Ordinary, 2),
        shape!(IfStmt, Repeat0, ConstructEntry, 2),
        shape!(ValueIf, Repeat0, ConstructEntry, 2),
        shape!(ValueIf, Choice, Ordinary, 2),
        shape!(ValueIf, Repeat0, ConstructEntry, 2),
        shape!(LoopStmt, Repeat0, ConstructEntry, 2),
        shape!(RegionStmt, Repeat0, ConstructEntry, 2),
        shape!(MatchStmt, Repeat1, Ordinary, 2),
        shape!(ValueMatch, Repeat1, Ordinary, 2),
        shape!(Arm, Optional, Ordinary, 2),
        shape!(Arm, Repeat0, ConstructEntry, 2),
        shape!(FieldbindList, Repeat0, Ordinary, 2),
        shape!(Expr, Choice, Ordinary, 3),
        shape!(Expr, Optional, Ordinary, 2),
        shape!(Atom, Choice, Ordinary, 4),
        shape!(Call, Optional, Ordinary, 2),
        shape!(Call, Optional, Ordinary, 2),
        shape!(Call, Choice, Ordinary, 2),
        shape!(Construct, Optional, Ordinary, 2),
        shape!(Construct, Optional, Ordinary, 2),
        shape!(FieldinitList, Repeat0, Ordinary, 2),
        shape!(BorrowExpr, Choice, Ordinary, 2),
        shape!(AtomList, Repeat0, Ordinary, 2),
        shape!(Psuffix, Choice, Ordinary, 2),
        shape!(Const, Choice, Ordinary, 2),
        shape!(Cvalue, Choice, Ordinary, 4),
        shape!(Cvalue, Repeat0, Ordinary, 2),
        shape!(Effects, Choice, Ordinary, 2),
        shape!(Effects, Repeat0, Ordinary, 2),
        shape!(Effect, Choice, Ordinary, 6),
        shape!(Effect, Repeat1, Ordinary, 2),
        shape!(Effect, Repeat1, Ordinary, 2),
        shape!(Effect, Repeat1, Ordinary, 2),
        shape!(Effect, Choice, Ordinary, 2),
        shape!(ForStmt, Repeat0, ConstructEntry, 2),
        shape!(FnDecl, Optional, Ordinary, 2),
        shape!(EnsuresBlock, Repeat0, ConstructEntry, 2),
        shape!(EnsuresSelector, Choice, Ordinary, 2),
        shape!(EnsuresSelector, Optional, Ordinary, 2),
        shape!(EnsuresEntry, Choice, ConstructEntry, 3),
    ];
    assert_eq!(expected.len(), 90);
    for (slot, expected) in expected.into_iter().enumerate() {
        let decision = DECISIONS[slot];
        assert_eq!(
            (
                decision.production(),
                decision.kind(),
                decision.context(),
                decision.arm_count(),
            ),
            expected,
            "historical decision slot {slot} changed shape"
        );
    }
}

#[test]
fn active_inventory_carries_the_system_interface_grammar() {
    assert!(productions().contains(&Production::ProgramKind));
    assert!(productions().contains(&Production::InputLabel));
    for terminal in [
        FixedTerminal::As,
        FixedTerminal::External,
        FixedTerminal::Blocks,
    ] {
        let predicate = LookaheadPredicate::Terminal(TerminalPredicate::Fixed(terminal));
        assert!(diagnostic_terminal_order().contains(&predicate));
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
    assert_eq!(decisions, 96);
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
    assert_eq!(DECISIONS.len(), 96);
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
    assert_eq!(total_rows, 3_788);
    assert!(saw_atom_only);
}
