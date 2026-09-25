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

/// [GRAM-2] "A `heap_decl` is admitted at most once in a compilation unit and
/// only as the first `item` of the first source record [PROG-2]; a second
/// `heap_decl`, or one at any later item position, is a hard error citing
/// GRAM-2 at that `heap_decl` node."
///
/// The parser is table-driven and has no compilation-unit position rule, so
/// the grammar admits a `heap_decl` at every item position and this whole-unit
/// judgment is made here. [DIAG-1] states that [FN-8]'s contract-presence
/// judgment runs first after canonical FORM-2 and fixes nothing else about
/// this one, so it runs immediately after that and before inventory.
pub(super) fn check_heap_declaration(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let items = topology
        .node_children(topology.root)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    let mut admitted: Option<SourceOrigin> = None;
    for (position, item) in items.iter().enumerate() {
        let Some(declaration) = heap_declaration(topology, *item)? else {
            continue;
        };
        let origin = node_origin(topology, scopes, declaration)?;
        // The unit's items are in [PROG-2] source-record order, so the first
        // item of the first source record is the first child of the root.
        if position == 0 && admitted.is_none() {
            admitted = Some(origin);
            continue;
        }
        return Ok(Some(ResolutionIssue {
            rule: ResolutionRule::Gram2,
            origin,
            kind: ResolutionIssueKind::MisplacedHeapDeclaration {
                admitted: admitted.clone(),
            },
        }));
    }
    Ok(None)
}

/// The `heap_decl` one `item` selects, when it selects one.
fn heap_declaration(
    topology: &FinalizedTopology,
    item: NodeId,
) -> Result<Option<NodeId>, ResolutionCompilerFailure> {
    if topology
        .node(item)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
        .production
        != Production::Item
    {
        return Ok(None);
    }
    Ok(topology
        .node_children(item)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
        .iter()
        .copied()
        .find(|child| {
            topology
                .node(*child)
                .is_some_and(|record| record.production == Production::HeapDecl)
        }))
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
        extent: SyntaxCoordinate::new(source, start, end),
        role_ordinal: 0,
        subtoken_ordinal: 0,
    })
}
