#![allow(clippy::panic)]

use super::{
    DecisionKind, GrammarNodeKind, LookaheadPredicate, Production, diagnostic_terminal_order,
    grammar_node, productions,
};
use crate::syntax::terminal::{ALL_FIXED_TERMINALS, FixedTerminal, TerminalPredicate};

use super::generated::{DECISIONS, SELECT_ROWS};

/// The grammar has 86 productions, 119 decisions and 5,229 select rows.
///
/// The production count is unchanged because the amendment retires exactly as
/// many productions as it adds: `region_params`, `region_param`,
/// `replace_let_rhs`, `region_stmt` and `dispose_stmt` leave, and `heap_decl`,
/// `range_tail`, `epbase`, `epsuffix` and `erange` enter. The decision and row
/// counts fall because the retired statements, the `set` target list, and
/// `borrow_expr`'s two region and permission optionals took decisions with
/// them; `borrow_expr` is now `"&" place` and owns none at all. The later
/// `opaque` modifier on `struct_decl` [GRAM-2, TYPE-2] then added one optional
/// decision and its rows, taking the counts from 118 and 5,218 to 119 and
/// 5,237, and the fixed-atom inventory from 94 to 95 with `diagnostic order`
/// from 101 to 102. x1 then trades one decision for another: `field`'s
/// `"readonly"?` [GRAM-2, TYPE-2] adds an optional and `epbase := IDENT`
/// [EFF-1] drops its choice, so the decision count stands at 119 while the
/// rows fall to 5,229; the atom inventory goes to 96 and `diagnostic order`
/// to 103. These assertions pin the complete generated inventory: the
/// structural checks below prove the table's properties, and an exact pin is
/// what makes the next amendment notice the move.
#[test]
fn complete_inventory_is_pinned() {
    assert_eq!(productions().len(), 86);
    assert_eq!(DECISIONS.len(), 119);
    assert_eq!(SELECT_ROWS.len(), 5_229);
    assert_eq!(diagnostic_terminal_order().len(), 103);
    assert_eq!(productions()[0], Production::Program);
    assert_eq!(productions()[2], Production::HeapDecl);
    assert_eq!(productions()[12], Production::ContractDefine);
    assert_eq!(productions()[13], Production::RequiresClause);
    assert_eq!(productions()[14], Production::EnsuresClause);
    assert_eq!(productions()[15], Production::ResultRoute);
    assert_eq!(productions()[26], Production::LinearityBound);
    assert_eq!(productions()[44], Production::ForStmt);
    assert_eq!(productions()[45], Production::ForBinding);
    assert_eq!(productions()[46], Production::HeaderInvariant);
    assert_eq!(productions()[47], Production::InvariantStmt);
    assert_eq!(productions()[48], Production::ProofUse);
    assert_eq!(productions()[49], Production::UsePremise);
    assert_eq!(productions()[64], Production::CompareOp);
    assert_eq!(productions()[72], Production::ClauseExpr);
    assert_eq!(productions()[73], Production::ClauseOp);
    assert_eq!(productions()[77], Production::RangeTail);
    assert_eq!(productions()[81], Production::Effect);
    assert_eq!(productions()[82], Production::EffectPath);
    assert_eq!(productions()[83], Production::Epbase);
    assert_eq!(productions()[84], Production::Epsuffix);
    assert_eq!(productions()[85], Production::Erange);
    assert_eq!(Production::ForStmt.index(), 63);
    assert_eq!(Production::ForBinding.index(), 64);
    assert_eq!(Production::HeaderInvariant.index(), 65);
    assert_eq!(Production::RequiresClause.index(), 66);
    assert_eq!(Production::EnsuresClause.index(), 67);
    assert_eq!(Production::ResultRoute.index(), 68);
    assert_eq!(Production::EffectPath.index(), 69);
    assert_eq!(Production::InvariantStmt.index(), 70);
    assert_eq!(Production::AffineExpr.index(), 71);
    assert_eq!(Production::AffineTerm.index(), 72);
    assert_eq!(Production::AffineFactor.index(), 73);
    assert_eq!(Production::AffineAddOp.index(), 74);
    assert_eq!(Production::ProofUse.index(), 75);
    assert_eq!(Production::ClauseExpr.index(), 77);
    assert_eq!(Production::ClauseOp.index(), 78);
    assert_eq!(Production::LinearityBound.index(), 79);
    // The five v0.60 productions are appended to the dense enum order, so
    // every surviving production keeps the index it had after the five
    // retirements were removed.
    assert_eq!(Production::HeapDecl.index(), 80);
    assert_eq!(Production::RangeTail.index(), 81);
    assert_eq!(Production::Epbase.index(), 82);
    assert_eq!(Production::Epsuffix.index(), 83);
    assert_eq!(Production::Erange.index(), 84);
    assert_eq!(Production::UsePremise.index(), 85);
    // Index 2 is `struct_decl`'s `"opaque"?` optional [GRAM-2, TYPE-2], so
    // every decision after `item` sits one place later than it did before
    // that modifier entered the grammar.
    // x1 [GRAM-2, TYPE-2]: `field := "readonly"? IDENT ":" type ";"` owns
    // one optional of its own, which is what shifts every decision between
    // it and `epbase` one place later.
    assert_eq!(DECISIONS[7].production(), Production::Field);
    assert_eq!(DECISIONS[7].kind(), DecisionKind::Optional);
    // x1 [EFF-1]: `epbase := IDENT` has one alternative and owns no decision.
    assert!(
        !DECISIONS
            .iter()
            .any(|decision| decision.production() == Production::Epbase)
    );
    assert_eq!(DECISIONS[21].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[21].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[22].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[22].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[23].production(), Production::ContractBlock);
    assert_eq!(DECISIONS[23].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[24].production(), Production::EnsuresClause);
    assert_eq!(DECISIONS[24].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[25].production(), Production::ResultRoute);
    assert_eq!(DECISIONS[25].kind(), DecisionKind::Optional);
    // `param := IDENT ":" (mode type | "&" "[" type "]")`: the range-reference
    // parameter kind is a written choice of its own [GRAM-2, REF-4].
    assert_eq!(DECISIONS[43].production(), Production::Param);
    assert_eq!(DECISIONS[43].kind(), DecisionKind::Choice);
    // `type` keeps its primitive-or-nominal choice and the `targs?` optional;
    // v0.59's three shape optionals retired with `array`, `box` and `arena`.
    assert_eq!(DECISIONS[44].production(), Production::Type);
    assert_eq!(DECISIONS[44].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[45].production(), Production::Type);
    assert_eq!(DECISIONS[45].kind(), DecisionKind::Optional);
    // `mode := "own" | "&"` is one choice where v0.59 had two optionals
    // around the region and permission markers.
    assert_eq!(DECISIONS[46].production(), Production::Mode);
    assert_eq!(DECISIONS[46].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[64].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[64].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[65].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[65].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[66].production(), Production::LoopStmt);
    assert_eq!(DECISIONS[66].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[68].production(), Production::ForStmt);
    assert_eq!(DECISIONS[68].kind(), DecisionKind::Optional);
    assert_eq!(DECISIONS[69].production(), Production::ForStmt);
    assert_eq!(DECISIONS[69].kind(), DecisionKind::Repeat0);
    assert_eq!(DECISIONS[71].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[71].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[72].production(), Production::InvariantStmt);
    assert_eq!(DECISIONS[72].kind(), DecisionKind::Repeat1);
    assert_eq!(DECISIONS[75].production(), Production::UsePremise);
    assert_eq!(DECISIONS[75].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[80].production(), Production::BreakStmt);
    assert_eq!(DECISIONS[80].kind(), DecisionKind::Optional);
    // `psuffix` carries the field, payload and index-or-range choice, and the
    // factored `range_tail?` that keeps the index and range steps
    // strong-LL(2) [GRAM-1, GRAM-5].
    assert_eq!(DECISIONS[103].production(), Production::Psuffix);
    assert_eq!(DECISIONS[103].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[104].production(), Production::Psuffix);
    assert_eq!(DECISIONS[104].kind(), DecisionKind::Optional);
    // `epsuffix` mirrors it inside an effect row [EFF-1]. Its two decisions
    // keep their indices under x1: the `field` optional added ahead of them
    // and the `epbase` choice removed between them cancel exactly.
    assert_eq!(DECISIONS[117].production(), Production::Epsuffix);
    assert_eq!(DECISIONS[117].kind(), DecisionKind::Choice);
    assert_eq!(DECISIONS[118].production(), Production::Epsuffix);
    assert_eq!(DECISIONS[118].kind(), DecisionKind::Optional);
}

/// `borrow_expr` is `"&" place` and owns no decision.
///
/// v0.59 wrote `"&" ("uniq")? (REGIONID)? place`, whose two optionals were the
/// last grammar-visible trace of the permission marker and of regions. Their
/// absence is the property, so it is asserted directly rather than left to the
/// inventory count above [GRAM-5, REF-1].
#[test]
fn borrow_expr_owns_no_decision() {
    assert!(
        !DECISIONS
            .iter()
            .any(|decision| decision.production() == Production::BorrowExpr)
    );
}

#[test]
fn active_inventory_has_only_ordinary_function_parameters() {
    assert!(productions().contains(&Production::FnDecl));
    assert!(productions().contains(&Production::FnSig));
    assert!(FixedTerminal::from_spelling(b"command").is_none());
    assert!(FixedTerminal::from_spelling(b"as").is_none());
}

/// The eleven atoms v0.60 retires are gone and the two it adds are present.
///
/// `from_spelling` is what decides whether a word is a keyword or a name, so
/// a retired atom left in the inventory would keep stealing its spelling from
/// IDENT or TYPEID without any decision table noticing [GRAM-1, FORM-3].
#[test]
fn the_retired_atoms_leave_the_inventory_and_the_new_ones_enter() {
    for retired in [
        b"region".as_slice(),
        b"uniq",
        b"dispose",
        b"replace",
        b"allocates",
        b"array",
        b"box",
        b"arena",
        b"buffer",
        b"Slice",
        b"MutSlice",
    ] {
        assert!(
            FixedTerminal::from_spelling(retired).is_none(),
            "retired atom still in the inventory: {retired:?}"
        );
    }
    assert_eq!(
        FixedTerminal::from_spelling(b"program"),
        Some(FixedTerminal::Program)
    );
    assert_eq!(
        FixedTerminal::from_spelling(b"no_heap"),
        Some(FixedTerminal::NoHeap)
    );
    assert_eq!(FixedTerminal::from_spelling(b"readonly"), Some(FixedTerminal::Readonly));
    assert_eq!(ALL_FIXED_TERMINALS.len(), 96);
    // No fixed atom is capitalized any more, so nothing competes with TYPEID.
    assert!(ALL_FIXED_TERMINALS.iter().all(|terminal| {
        !terminal
            .spelling_bytes()
            .first()
            .is_some_and(u8::is_ascii_uppercase)
    }));
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
    // The same 119 decisions `complete_inventory_is_pinned` reads out of the
    // generated table, counted a second time by walking every production's
    // node tree. `struct_decl`'s `"opaque"?` optional [GRAM-2, TYPE-2] is
    // reachable from `item`, so the walk and the table agree on it; a
    // decision in the table that no production reaches would show up as the
    // two counts disagreeing.
    assert_eq!(decisions, DECISIONS.len());
    assert_eq!(decisions, 119);
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
    assert_eq!(DECISIONS.len(), 119);
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
    // The 5,229 rows `complete_inventory_is_pinned` pins, counted here by
    // summing each decision's own rows: x1's `field` optional [GRAM-2,
    // TYPE-2] brings rows in and `epbase := IDENT` [EFF-1] takes more out
    // with the choice it no longer owns.
    assert_eq!(total_rows, 5_229);
    assert!(saw_atom_only);
}
