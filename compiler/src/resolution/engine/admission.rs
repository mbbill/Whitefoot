use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{Production, SyntaxCoordinate};

use super::super::scopes::ScopeBuild;
use super::super::{
    ContractShapeIssue, ResolutionCompilerFailure, ResolutionIssue, ResolutionIssueKind,
    ResolutionRule, SourceOrigin,
};
use super::EventKey;

pub(super) fn check_clause_blocks(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let mut candidates = Vec::new();
    for (index, record) in topology.nodes.iter().enumerate() {
        if record.production != Production::ContractBlock {
            continue;
        }
        let id = NodeId::from_index(index).ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        let has_clause = topology
            .node_children(id)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .iter()
            .any(|child| {
                topology.node(*child).is_some_and(|child| {
                    matches!(
                        child.production,
                        Production::RequiresClause | Production::EnsuresClause
                    )
                })
            });
        if !has_clause {
            candidates.push(ResolutionIssue {
                rule: ResolutionRule::Fn8,
                origin: node_origin(topology, scopes, id)?,
                kind: ResolutionIssueKind::ContractShape(ContractShapeIssue::MissingClause),
            });
        }
    }
    candidates.sort_by_key(|issue| EventKey::from_origin(&issue.origin));
    Ok(candidates.into_iter().next())
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
