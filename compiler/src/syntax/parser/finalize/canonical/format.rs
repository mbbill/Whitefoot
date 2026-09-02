use crate::syntax::grammar::Production;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};

use super::{AuditWork, Stop};
use crate::syntax::parser::finalize::outcome::{
    CanonicalCompilerFailure, CanonicalLimit, CanonicalLimits, CanonicalResourceFailure,
    CanonicalStorage,
};
use crate::syntax::parser::finalize::topology::{FinalizedExtent, FinalizedTopology, NodeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum GapStyle {
    Inline,
    Spaced,
    Break,
    HeaderBreak,
    Blank,
}

fn is_line_bearing(topology: &FinalizedTopology, node: NodeId) -> Result<bool, Stop> {
    let record = topology
        .node(node)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    let fixed = matches!(
        record.production,
        Production::Field
            | Production::Variant
            | Production::FnSig
            | Production::Law
            | Production::FnBind
            | Production::ConstDecl
            | Production::Doc
            | Production::ContractDefine
            | Production::RequiresClause
            | Production::EnsuresClause
            | Production::SetStmt
            | Production::ExprStmt
            | Production::ReturnStmt
            | Production::BreakStmt
            | Production::ProofUse
            | Production::GiveStmt
    );
    if fixed || (record.production == Production::InvariantStmt && record.body_open.is_none()) {
        return Ok(true);
    }
    if record.production != Production::LetStmt {
        return Ok(false);
    }
    let children = topology
        .node_children(node)
        .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
    Ok(children.iter().any(|child| {
        topology.node(*child).is_some_and(|nested| {
            matches!(
                nested.production,
                Production::OrdinaryLetRhs
                    | Production::PropagateLetRhs
                    | Production::ReplaceLetRhs
            )
        })
    }))
}

fn is_block_bearing(record: &crate::syntax::parser::finalize::topology::NodeRecord) -> bool {
    matches!(
        record.production,
        Production::StructDecl
            | Production::EnumDecl
            | Production::ContractDecl
            | Production::ConformDecl
            | Production::FnDecl
            | Production::ContractBlock
            | Production::LoopStmt
            | Production::ForStmt
            | Production::RegionStmt
            | Production::MatchStmt
            | Production::ValueMatch
            | Production::Arm
            | Production::IfStmt
            | Production::ValueIf
    ) || (record.production == Production::InvariantStmt && record.body_open.is_some())
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
        if !is_block_bearing(record) {
            if record.body_open.is_some() || record.body_close.is_some() {
                return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
            }
            continue;
        }
        let ranges = record.body_ranges();
        if ranges[0].is_none() {
            return Err(CanonicalCompilerFailure::InvalidFinalizedTree.into());
        }
        for (index, (open, close)) in ranges.iter().flatten().copied().enumerate() {
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
            // A clause block joins the following clause or function body, and an
            // `if` joins its continuation as `} else {` or `} else if … {`.
            // Both keep the close brace and what follows on one line, so the
            // break after the close is suppressed exactly there. The last
            // block of a construct always breaks.
            let joins_a_continuation = matches!(record.production, Production::ContractBlock)
                || (index == 0 && record.has_else);
            if !joins_a_continuation {
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

        if matches!(
            record.production,
            Production::ForStmt | Production::LoopStmt
        ) {
            let children = topology
                .node_children(node)
                .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
            let mut header_items = 0_u32;
            for child in children {
                let child = topology
                    .node(*child)
                    .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
                if matches!(
                    child.production,
                    Production::ForBinding | Production::HeaderInvariant
                ) {
                    header_items = header_items
                        .checked_add(1)
                        .ok_or(CanonicalCompilerFailure::CounterOverflow)?;
                    mark_before(
                        &mut gaps,
                        topology,
                        child.first_terminal,
                        GapStyle::HeaderBreak,
                    )?;
                }
            }
            if header_items != 0 {
                let first_header = children
                    .iter()
                    .filter_map(|child| topology.node(*child))
                    .find(|child| {
                        matches!(
                            child.production,
                            Production::ForBinding | Production::HeaderInvariant
                        )
                    })
                    .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
                let open = first_header
                    .first_terminal
                    .checked_sub(1)
                    .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
                mark_before(&mut gaps, topology, open, GapStyle::Spaced)?;
                let close = record
                    .body_open
                    .and_then(|open| open.checked_sub(1))
                    .ok_or(CanonicalCompilerFailure::InvalidFinalizedTree)?;
                mark_before(&mut gaps, topology, close, GapStyle::Break)?;
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

fn left_attaches(predicate: TerminalPredicate) -> bool {
    matches!(
        predicate,
        TerminalPredicate::Fixed(
            FixedTerminal::LeftParen
                | FixedTerminal::LeftBracket
                | FixedTerminal::LeftAngle
                | FixedTerminal::Ampersand
                | FixedTerminal::Dot
                | FixedTerminal::DotDot
        )
    )
}

fn right_attaches(predicate: TerminalPredicate) -> bool {
    matches!(
        predicate,
        TerminalPredicate::Fixed(
            FixedTerminal::RightParen
                | FixedTerminal::RightBracket
                | FixedTerminal::RightAngle
                | FixedTerminal::Comma
                | FixedTerminal::Semicolon
                | FixedTerminal::Dot
                | FixedTerminal::Colon
                | FixedTerminal::LeftParen
                | FixedTerminal::LeftAngle
                | FixedTerminal::LeftBracket
                | FixedTerminal::DotDot
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

/// The exact canonical bytes of one terminal gap: newlines, then spaces.
///
/// Every FORM-2 gap has this shape, so one value describes it completely. The
/// auditor compares source bytes against it and the renderer emits it, which is
/// what keeps the two from drifting: there is one rule, not two that agree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CanonicalGap {
    newlines: usize,
    spaces: usize,
}

impl CanonicalGap {
    /// Yields the gap's bytes in order.
    pub(super) fn bytes(self) -> impl Iterator<Item = u8> {
        core::iter::repeat_n(b'\n', self.newlines).chain(core::iter::repeat_n(b' ', self.spaces))
    }

    /// The gap as text, for a rejection to print beside what it found.
    pub(super) fn text(self) -> String {
        self.bytes().map(char::from).collect()
    }

    /// Returns the gap's byte length.
    pub(super) fn len(self) -> Result<usize, Stop> {
        self.newlines
            .checked_add(self.spaces)
            .ok_or_else(|| CanonicalCompilerFailure::CounterOverflow.into())
    }
}

/// Computes the canonical gap a terminal boundary must carry.
pub(super) fn canonical_gap(
    style: GapStyle,
    depth: u32,
    left: Option<TerminalPredicate>,
    right: Option<TerminalPredicate>,
) -> Result<CanonicalGap, Stop> {
    Ok(match style {
        GapStyle::Inline => {
            let space = matches!((left, right), (Some(left), Some(right)) if !left_attaches(left) && !right_attaches(right));
            CanonicalGap {
                newlines: 0,
                spaces: usize::from(space),
            }
        }
        GapStyle::Spaced => CanonicalGap {
            newlines: 0,
            spaces: 1,
        },
        GapStyle::Break => CanonicalGap {
            newlines: 1,
            spaces: usize::try_from(depth)
                .ok()
                .and_then(|value| value.checked_mul(2))
                .ok_or(CanonicalCompilerFailure::CounterOverflow)?,
        },
        GapStyle::HeaderBreak => CanonicalGap {
            newlines: 1,
            spaces: usize::try_from(
                depth
                    .checked_add(1)
                    .ok_or(CanonicalCompilerFailure::CounterOverflow)?,
            )
            .ok()
            .and_then(|value| value.checked_mul(2))
            .ok_or(CanonicalCompilerFailure::CounterOverflow)?,
        },
        GapStyle::Blank => CanonicalGap {
            newlines: 2,
            spaces: 0,
        },
    })
}

pub(super) fn gap_matches(
    actual: &[u8],
    style: GapStyle,
    depth: u32,
    left: Option<TerminalPredicate>,
    right: Option<TerminalPredicate>,
    work: &mut AuditWork,
) -> Result<(bool, u64, CanonicalGap), Stop> {
    let gap = canonical_gap(style, depth, left, right)?;
    let expected_len = gap.len()?;
    let matches = bytes_match(actual, gap.bytes(), expected_len, work)?;
    let expected_len =
        u64::try_from(expected_len).map_err(|_| CanonicalCompilerFailure::CounterOverflow)?;
    Ok((matches, expected_len, gap))
}
