use crate::syntax::terminal::TerminalPredicateV0_14;
use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{
    NodePath, ProductionV0_14, ResolvedSyntaxUnit, SemanticCompilerFailure, SyntaxCoordinate,
};

pub(super) struct TreeView<'unit, 'classified, 'lexed, 'source> {
    resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    paths: Vec<NodePath>,
    direct_terminals: Vec<Vec<usize>>,
}

impl<'unit, 'classified, 'lexed, 'source> TreeView<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn new(
        resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    ) -> Result<Self, SemanticCompilerFailure> {
        let topology = Self::topology_of(resolved);
        let mut paths = Vec::with_capacity(topology.nodes.len());
        for index in 0..topology.nodes.len() {
            let mut node =
                NodeId::from_index(index).ok_or(SemanticCompilerFailure::CounterOverflow)?;
            let mut components = Vec::new();
            while node != topology.root {
                let record = topology
                    .node(node)
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                components.push(record.child_ordinal);
                node = record
                    .parent
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            }
            components.reverse();
            paths.push(NodePath { components });
        }

        let mut direct_terminals = vec![Vec::new(); topology.nodes.len()];
        for (terminal_index, terminal) in topology.terminals.iter().enumerate() {
            let owner = terminal
                .owner
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            direct_terminals
                .get_mut(owner.index())
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
                .push(terminal_index);
        }
        Ok(Self {
            resolved,
            paths,
            direct_terminals,
        })
    }

    pub(super) fn topology(&self) -> &FinalizedTopology {
        Self::topology_of(self.resolved)
    }

    fn topology_of<'resolved>(
        resolved: &'resolved ResolvedSyntaxUnit<'_, '_, '_>,
    ) -> &'resolved FinalizedTopology {
        &resolved.syntax().finalized.topology
    }

    pub(super) fn root(&self) -> NodeId {
        self.topology().root
    }

    pub(super) fn production(
        &self,
        node: NodeId,
    ) -> Result<ProductionV0_14, SemanticCompilerFailure> {
        self.topology()
            .node(node)
            .map(|record| record.production)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn children(&self, node: NodeId) -> Result<&[NodeId], SemanticCompilerFailure> {
        self.topology()
            .node_children(node)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn children_with(
        &self,
        node: NodeId,
        production: ProductionV0_14,
    ) -> Result<Vec<NodeId>, SemanticCompilerFailure> {
        Ok(self
            .children(node)?
            .iter()
            .copied()
            .filter(|child| {
                self.production(*child)
                    .is_ok_and(|actual| actual == production)
            })
            .collect())
    }

    pub(super) fn only_child(&self, node: NodeId) -> Result<NodeId, SemanticCompilerFailure> {
        let [child] = self.children(node)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree);
        };
        Ok(*child)
    }

    pub(super) fn first_child_with(
        &self,
        node: NodeId,
        production: ProductionV0_14,
    ) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        for child in self.children(node)? {
            if self.production(*child)? == production {
                return Ok(Some(*child));
            }
        }
        Ok(None)
    }

    pub(super) fn path(&self, node: NodeId) -> Result<&NodePath, SemanticCompilerFailure> {
        self.paths
            .get(node.index())
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn coordinate(
        &self,
        node: NodeId,
    ) -> Result<SyntaxCoordinate, SemanticCompilerFailure> {
        let record = self
            .topology()
            .node(node)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let FinalizedExtent::Source { source, start, end } = record.extent else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree);
        };
        Ok(SyntaxCoordinate::new(source, start, end))
    }

    pub(super) fn closing_brace_coordinate(
        &self,
        node: NodeId,
    ) -> Result<SyntaxCoordinate, SemanticCompilerFailure> {
        let terminal = self
            .topology()
            .node(node)
            .and_then(|record| record.body_close)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| {
                self.resolved
                    .syntax()
                    .classified_bundle()
                    .tokens()
                    .get(index)
            })
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
            .token()
            .id();
        Ok(SyntaxCoordinate::new(
            terminal.source(),
            terminal.start(),
            terminal.end(),
        ))
    }

    pub(super) fn direct_token_indices(
        &self,
        node: NodeId,
    ) -> Result<&[usize], SemanticCompilerFailure> {
        self.direct_terminals
            .get(node.index())
            .map(Vec::as_slice)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn token_bytes(
        &self,
        terminal: usize,
    ) -> Result<&'source [u8], SemanticCompilerFailure> {
        self.resolved
            .syntax()
            .classified_bundle()
            .tokens()
            .get(terminal)
            .map(|token| token.token().span().bytes())
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn direct_spelling(&self, node: NodeId) -> Result<Vec<u8>, SemanticCompilerFailure> {
        let mut spelling = Vec::new();
        for terminal in self.direct_token_indices(node)? {
            spelling.extend_from_slice(self.token_bytes(*terminal)?);
        }
        Ok(spelling)
    }

    pub(super) fn direct_token_with(
        &self,
        node: NodeId,
        predicate: TerminalPredicateV0_14,
    ) -> Result<Option<usize>, SemanticCompilerFailure> {
        let classified = self.resolved.syntax().classified_bundle();
        let mut found = None;
        for terminal in self.direct_token_indices(node)? {
            let token = classified
                .tokens()
                .get(*terminal)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if token.terminals().contains(predicate) && found.replace(*terminal).is_some() {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree);
            }
        }
        Ok(found)
    }
}
