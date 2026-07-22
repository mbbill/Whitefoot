use whitefoot_language_data::{FixedTerminalV0_9, TerminalPredicateV0_9};
use whitefoot_syntax_data::ProductionV0_9;

use super::{AuditWork, Stop};
use crate::parser::finalize::outcome::{
    CanonicalCompilerFailure, CanonicalLimit, CanonicalLimits, CanonicalResourceFailure,
    CanonicalStorage,
};
use crate::parser::finalize::topology::{FinalizedExtent, FinalizedTopology, NodeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum GapStyle {
    Inline,
    Break,
    Blank,
}

fn is_line_bearing(topology: &FinalizedTopology, node: NodeId) -> Result<bool, Stop> {
    let record = topology
        .node(node)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    let fixed = matches!(
        record.production,
        ProductionV0_9::Field
            | ProductionV0_9::Variant
            | ProductionV0_9::FnSig
            | ProductionV0_9::Law
            | ProductionV0_9::FnBind
            | ProductionV0_9::ConstDecl
            | ProductionV0_9::Doc
            | ProductionV0_9::SetStmt
            | ProductionV0_9::ExprStmt
            | ProductionV0_9::ReturnStmt
            | ProductionV0_9::BreakStmt
            | ProductionV0_9::CheckStmt
            | ProductionV0_9::GiveStmt
    );
    if fixed {
        return Ok(true);
    }
    if record.production != ProductionV0_9::LetStmt {
        return Ok(false);
    }
    let children = topology
        .node_children(node)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    Ok(children.iter().any(|child| {
        topology.node(*child).is_some_and(|nested| {
            matches!(
                nested.production,
                ProductionV0_9::OrdinaryLetRhs | ProductionV0_9::TryLetRhs
            )
        })
    }))
}

fn is_block_bearing(production: ProductionV0_9) -> bool {
    matches!(
        production,
        ProductionV0_9::StructDecl
            | ProductionV0_9::EnumDecl
            | ProductionV0_9::ContractDecl
            | ProductionV0_9::ConformDecl
            | ProductionV0_9::FnDecl
            | ProductionV0_9::RequiresBlock
            | ProductionV0_9::LoopStmt
            | ProductionV0_9::RegionStmt
            | ProductionV0_9::MatchStmt
            | ProductionV0_9::ValueMatch
            | ProductionV0_9::Arm
    )
}

fn same_source(topology: &FinalizedTopology, left: u64, right: u64) -> bool {
    let Ok(left_index) = usize::try_from(left) else {
        return false;
    };
    let Ok(right_index) = usize::try_from(right) else {
        return false;
    };
    matches!(
        (topology.terminals.get(left_index), topology.terminals.get(right_index)),
        (Some(left), Some(right)) if left.source == right.source
    )
}

fn mark_before(
    gaps: &mut [GapStyle],
    topology: &FinalizedTopology,
    terminal: u64,
    style: GapStyle,
) -> Result<(), Stop> {
    if terminal == 0 || !same_source(topology, terminal - 1, terminal) {
        return Ok(());
    }
    let index = usize::try_from(terminal).map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    let gap = gaps
        .get_mut(index)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    *gap = (*gap).max(style);
    Ok(())
}

pub(super) fn build_gap_styles(
    topology: &FinalizedTopology,
    limits: CanonicalLimits,
    work: &mut AuditWork,
) -> Result<Vec<GapStyle>, Stop> {
    let count = u64::try_from(topology.terminals.len())
        .map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    if count > limits.max_gaps {
        return Err(CanonicalResourceFailure::LimitExceeded {
            limit: CanonicalLimit::Gaps,
            maximum: limits.max_gaps,
            actual: count,
        }
        .into());
    }
    let mut gaps = Vec::new();
    gaps.try_reserve_exact(topology.terminals.len())
        .map_err(|_| CanonicalResourceFailure::StorageUnavailable {
            storage: CanonicalStorage::Gaps,
            requested: count,
        })?;
    gaps.resize(topology.terminals.len(), GapStyle::Inline);

    for (index, record) in topology.nodes.iter().enumerate() {
        work.spend(1)?;
        let node = NodeId::from_index(index).ok_or(CanonicalCompilerFailure::CounterOverflow)?;
        if is_line_bearing(topology, node)? {
            let last = record
                .last_terminal()
                .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
            let next = last
                .checked_add(1)
                .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
            if usize::try_from(next)
                .ok()
                .is_some_and(|value| value < gaps.len())
            {
                mark_before(&mut gaps, topology, next, GapStyle::Break)?;
            }
        }
        if !is_block_bearing(record.production) {
            if record.body_open.is_some() || record.body_close.is_some() {
                return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
            }
            continue;
        }
        let (Some(open), Some(close)) = (record.body_open, record.body_close) else {
            return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
        };
        if open >= close
            || record.first_terminal > open
            || record.last_terminal().is_none_or(|last| close > last)
        {
            return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
        }
        let after_open = open
            .checked_add(1)
            .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
        mark_before(&mut gaps, topology, after_open, GapStyle::Break)?;
        mark_before(&mut gaps, topology, close, GapStyle::Break)?;
        if record.production != ProductionV0_9::RequiresBlock {
            let after_close = close
                .checked_add(1)
                .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
            if usize::try_from(after_close)
                .ok()
                .is_some_and(|value| value < gaps.len())
            {
                mark_before(&mut gaps, topology, after_close, GapStyle::Break)?;
            }
        }
    }

    let root_children = topology
        .node_children(topology.root)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    for pair in root_children.windows(2) {
        let left = topology
            .node(pair[0])
            .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
        let right = topology
            .node(pair[1])
            .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
        let (
            FinalizedExtent::Source {
                source: left_source,
                ..
            },
            FinalizedExtent::Source {
                source: right_source,
                ..
            },
        ) = (left.extent, right.extent)
        else {
            return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
        };
        if left_source == right_source {
            mark_before(&mut gaps, topology, right.first_terminal, GapStyle::Blank)?;
        }
    }
    Ok(gaps)
}

fn left_attaches(predicate: TerminalPredicateV0_9) -> bool {
    matches!(
        predicate,
        TerminalPredicateV0_9::Fixed(
            FixedTerminalV0_9::LeftParen
                | FixedTerminalV0_9::LeftBracket
                | FixedTerminalV0_9::LeftAngle
                | FixedTerminalV0_9::Ampersand
                | FixedTerminalV0_9::Dot
        )
    )
}

fn right_attaches(predicate: TerminalPredicateV0_9) -> bool {
    matches!(
        predicate,
        TerminalPredicateV0_9::Fixed(
            FixedTerminalV0_9::RightParen
                | FixedTerminalV0_9::RightBracket
                | FixedTerminalV0_9::RightAngle
                | FixedTerminalV0_9::Comma
                | FixedTerminalV0_9::Semicolon
                | FixedTerminalV0_9::Dot
                | FixedTerminalV0_9::Colon
                | FixedTerminalV0_9::LeftParen
                | FixedTerminalV0_9::LeftAngle
        )
    )
}

pub(super) fn bytes_match(
    actual: &[u8],
    expected: impl Iterator<Item = u8>,
    expected_len: usize,
    work: &mut AuditWork,
) -> Result<bool, Stop> {
    let actual_work =
        u64::try_from(actual.len()).map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    let expected_work =
        u64::try_from(expected_len).map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    work.spend(actual_work.max(expected_work))?;
    if actual.len() != expected_len {
        return Ok(false);
    }
    Ok(actual.iter().copied().eq(expected))
}

pub(super) fn gap_matches(
    actual: &[u8],
    style: GapStyle,
    depth: u32,
    left: Option<TerminalPredicateV0_9>,
    right: Option<TerminalPredicateV0_9>,
    work: &mut AuditWork,
) -> Result<(bool, u64), Stop> {
    let (matches, expected_len) = match style {
        GapStyle::Inline => {
            let space = matches!((left, right), (Some(left), Some(right)) if !left_attaches(left) && !right_attaches(right));
            let expected_len = usize::from(space);
            (
                bytes_match(
                    actual,
                    core::iter::once(b' ').take(expected_len),
                    expected_len,
                    work,
                )?,
                expected_len,
            )
        }
        GapStyle::Break => {
            let spaces = usize::try_from(depth)
                .ok()
                .and_then(|value| value.checked_mul(2))
                .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
            let length = spaces
                .checked_add(1)
                .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
            (
                bytes_match(
                    actual,
                    core::iter::once(b'\n').chain(core::iter::repeat_n(b' ', spaces)),
                    length,
                    work,
                )?,
                length,
            )
        }
        GapStyle::Blank => (bytes_match(actual, [b'\n', b'\n'].into_iter(), 2, work)?, 2),
    };
    let expected_len =
        u64::try_from(expected_len).map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    Ok((matches, expected_len))
}
