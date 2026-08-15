use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{Production, SyntaxCoordinate};

use super::super::scopes::ScopeBuild;
use super::super::{
    EnsuresShapeIssue, RequiresShapeIssue, ResolutionCompilerFailure, ResolutionIssue,
    ResolutionIssueKind, ResolutionRule, SourceOrigin,
};
use super::EventKey;

pub(super) fn check_clause_blocks(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let mut candidates = Vec::new();
    for (index, node) in topology.nodes.iter().enumerate() {
        let clause = match node.production {
            Production::RequiresBlock => ClauseKind::Requires,
            Production::EnsuresBlock => ClauseKind::Ensures,
            _ => continue,
        };
        let id = NodeId::from_index(index).ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        let children = topology
            .node_children(id)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let entries = match clause {
            ClauseKind::Requires => children,
            ClauseKind::Ensures => {
                let [selector, entries @ ..] = children else {
                    return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
                };
                if topology
                    .node(*selector)
                    .is_none_or(|record| record.production != Production::EnsuresSelector)
                {
                    return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
                }
                entries
            }
        };
        let expected_entry = match clause {
            ClauseKind::Requires => Production::RequiresEntry,
            ClauseKind::Ensures => Production::EnsuresEntry,
        };
        if entries.iter().any(|entry| {
            topology
                .node(*entry)
                .is_none_or(|record| record.production != expected_entry)
        }) {
            return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
        }
        let mut all_ordinary = true;
        let mut selected = None;
        for (entry_index, entry) in entries.iter().enumerate() {
            let kind = clause_entry_kind(topology, *entry)?;
            match kind {
                ClauseEntryKind::OrdinaryLet => {}
                ClauseEntryKind::Check if entry_index + 1 == entries.len() => {
                    all_ordinary = false;
                }
                _ => {
                    selected = Some((*entry, ShapeIssue::InvalidEntry));
                    break;
                }
            }
        }
        if selected.is_none() && (entries.is_empty() || all_ordinary) {
            selected = Some((id, ShapeIssue::MissingFinalCheck));
        }
        if let Some((issue_node, issue_kind)) = selected {
            let origin = node_origin(topology, scopes, issue_node)?;
            let (rule, kind) = match (clause, issue_kind) {
                (ClauseKind::Requires, ShapeIssue::MissingFinalCheck) => (
                    ResolutionRule::Fn8,
                    ResolutionIssueKind::RequiresShape(RequiresShapeIssue::MissingFinalCheck),
                ),
                (ClauseKind::Requires, ShapeIssue::InvalidEntry) => (
                    ResolutionRule::Fn8,
                    ResolutionIssueKind::RequiresShape(RequiresShapeIssue::InvalidEntry),
                ),
                (ClauseKind::Ensures, ShapeIssue::MissingFinalCheck) => (
                    ResolutionRule::Fn9,
                    ResolutionIssueKind::EnsuresShape(EnsuresShapeIssue::MissingFinalCheck),
                ),
                (ClauseKind::Ensures, ShapeIssue::InvalidEntry) => (
                    ResolutionRule::Fn9,
                    ResolutionIssueKind::EnsuresShape(EnsuresShapeIssue::InvalidEntry),
                ),
            };
            candidates.push(ResolutionIssue { rule, origin, kind });
        }
    }
    candidates.sort_by_key(|issue| EventKey::from_origin(&issue.origin));
    Ok(candidates.into_iter().next())
}

#[derive(Clone, Copy)]
enum ClauseKind {
    Requires,
    Ensures,
}

#[derive(Clone, Copy)]
enum ShapeIssue {
    MissingFinalCheck,
    InvalidEntry,
}

#[derive(Clone, Copy)]
enum ClauseEntryKind {
    OrdinaryLet,
    Check,
    Other,
}

fn clause_entry_kind(
    topology: &FinalizedTopology,
    entry: NodeId,
) -> Result<ClauseEntryKind, ResolutionCompilerFailure> {
    let [selected] = topology
        .node_children(entry)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
    else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    let selected_record = topology
        .node(*selected)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    if selected_record.production != Production::Stmt {
        return Ok(ClauseEntryKind::Other);
    }
    let [statement] = topology
        .node_children(*selected)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
    else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    let statement_record = topology
        .node(*statement)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    match statement_record.production {
        Production::CheckStmt => Ok(ClauseEntryKind::Check),
        Production::LetStmt => {
            let ordinary = topology
                .node_children(*statement)
                .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
                .iter()
                .any(|child| {
                    topology
                        .node(*child)
                        .is_some_and(|record| record.production == Production::OrdinaryLetRhs)
                });
            Ok(if ordinary {
                ClauseEntryKind::OrdinaryLet
            } else {
                ClauseEntryKind::Other
            })
        }
        _ => Ok(ClauseEntryKind::Other),
    }
}

fn node_origin(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    node: NodeId,
) -> Result<SourceOrigin, ResolutionCompilerFailure> {
    let record = topology
        .node(node)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    let FinalizedExtent::Source { source, start, end } = record.extent else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    Ok(SourceOrigin {
        node: scopes.path(node)?.clone(),
        coordinate: SyntaxCoordinate::new(source, start, end),
        role_ordinal: 0,
        subtoken_ordinal: 0,
    })
}
