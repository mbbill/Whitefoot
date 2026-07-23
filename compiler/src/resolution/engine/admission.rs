use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{Production, SyntaxCoordinate};

use super::super::scopes::ScopeBuild;
use super::super::{
    RequiresShapeIssue, ResolutionCompilerFailure, ResolutionIssue, ResolutionIssueKind,
    ResolutionRule, SourceOrigin,
};
use super::EventKey;

pub(super) fn check_requires_blocks(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let mut candidates = Vec::new();
    for (index, node) in topology.nodes.iter().enumerate() {
        if node.production != Production::RequiresBlock {
            continue;
        }
        let id = NodeId::from_index(index).ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        let entries = topology
            .node_children(id)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let mut all_ordinary = true;
        let mut selected = None;
        for (entry_index, entry) in entries.iter().enumerate() {
            let kind = requires_entry_kind(topology, *entry)?;
            match kind {
                RequiresEntryKind::OrdinaryLet => {}
                RequiresEntryKind::Check if entry_index + 1 == entries.len() => {
                    all_ordinary = false;
                }
                _ => {
                    selected = Some((*entry, RequiresShapeIssue::InvalidEntry));
                    break;
                }
            }
        }
        if selected.is_none() && (entries.is_empty() || all_ordinary) {
            selected = Some((id, RequiresShapeIssue::MissingFinalCheck));
        }
        if let Some((issue_node, issue_kind)) = selected {
            let origin = node_origin(topology, scopes, issue_node)?;
            candidates.push(ResolutionIssue {
                rule: ResolutionRule::Fn8,
                origin,
                kind: ResolutionIssueKind::RequiresShape(issue_kind),
            });
        }
    }
    candidates.sort_by_key(|issue| EventKey::from_origin(&issue.origin));
    Ok(candidates.into_iter().next())
}

#[derive(Clone, Copy)]
enum RequiresEntryKind {
    OrdinaryLet,
    Check,
    Other,
}

fn requires_entry_kind(
    topology: &FinalizedTopology,
    entry: NodeId,
) -> Result<RequiresEntryKind, ResolutionCompilerFailure> {
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
        return Ok(RequiresEntryKind::Other);
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
        Production::CheckStmt => Ok(RequiresEntryKind::Check),
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
                RequiresEntryKind::OrdinaryLet
            } else {
                RequiresEntryKind::Other
            })
        }
        _ => Ok(RequiresEntryKind::Other),
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
