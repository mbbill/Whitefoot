use crate::syntax::terminal::TerminalPredicate;
use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{
    ByteOffset, NodePath, Production, ResolvedSyntaxUnit, SemanticCompilerFailure, SyntaxCoordinate,
};

/// [GRAM-4] one conditional node's then-block and its alternative.
pub(super) struct ConditionalBlocks {
    pub(super) then_statements: Vec<NodeId>,
    pub(super) alternative: ConditionalAlternative,
}

/// [GRAM-6] the three shapes an `if` alternative can take.
pub(super) enum ConditionalAlternative {
    /// No `else`: the else-free `if`, whose alternative delivers and does
    /// nothing. [ERR-2] makes this the one spelling of the empty alternative.
    Absent,
    /// A braced `else` and the statements it owns.
    Block(Vec<NodeId>),
    /// `else if`: the nested conditional owning the rest of the chain.
    Chain(NodeId),
}

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

    pub(super) fn production(&self, node: NodeId) -> Result<Production, SemanticCompilerFailure> {
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
        production: Production,
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

    /// The one child holding an `expr`'s complete written content, or `None`
    /// when the expression is the infix shape.
    ///
    /// [GRAM-5] `expr := atom infix_tail? | call | construct`. Three of those
    /// alternatives are a single child that names what the expression is
    /// written as; the infix one is two children producing a fresh operation
    /// result that is no written atom, call, or construct. A structural query
    /// asking which of the three shapes an expression has therefore has no
    /// answer for infix and must answer `None` — never fail the tree, which
    /// would turn a legal expression into an internal compiler failure.
    pub(super) fn sole_expression_child(
        &self,
        expression: NodeId,
    ) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        if self
            .first_child_with(expression, Production::InfixTail)?
            .is_some()
        {
            return Ok(None);
        }
        self.only_child(expression).map(Some)
    }

    pub(super) fn first_child_with(
        &self,
        node: NodeId,
        production: Production,
    ) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        for child in self.children(node)? {
            if self.production(*child)? == production {
                return Ok(Some(*child));
            }
        }
        Ok(None)
    }

    pub(super) fn descendants_with(
        &self,
        node: NodeId,
        production: Production,
    ) -> Result<Vec<NodeId>, SemanticCompilerFailure> {
        let mut matches = Vec::new();
        let mut pending = self
            .children(node)?
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>();
        while let Some(candidate) = pending.pop() {
            if self.production(candidate)? == production {
                matches.push(candidate);
            }
            pending.extend(self.children(candidate)?.iter().rev().copied());
        }
        Ok(matches)
    }

    /// [GRAM-4] the statement groups an `if_stmt` or `value_if` owns.
    ///
    /// Both blocks are `Stmt` children of the one conditional node, so only
    /// the brace pairs separate them: the then-block is the group inside the
    /// first pair and a braced `else` is the group inside the second. An
    /// `else if` chain owns one pair and an `else` terminal, and reaches its
    /// alternative through the nested conditional node instead.
    pub(super) fn conditional_blocks(
        &self,
        node: NodeId,
    ) -> Result<ConditionalBlocks, SemanticCompilerFailure> {
        let record = self
            .topology()
            .node(node)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let [Some(then_range), else_range] = record.body_ranges() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree);
        };
        let mut then_statements = Vec::new();
        let mut else_statements = Vec::new();
        for statement in self.children_with(node, Production::Stmt)? {
            if self.within(statement, then_range)? {
                then_statements.push(statement);
            } else {
                else_statements.push(statement);
            }
        }
        // Only a braced `else` owns statements directly; with no second pair
        // every statement of this node belongs to the then-block.
        if else_range.is_none() && !else_statements.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree);
        }
        let alternative = match (else_range, record.has_else) {
            (Some(_), _) => ConditionalAlternative::Block(else_statements),
            (None, true) => {
                let nested = self
                    .children(node)?
                    .iter()
                    .copied()
                    .find(|child| {
                        self.production(*child).is_ok_and(|production| {
                            matches!(production, Production::IfStmt | Production::ValueIf)
                        })
                    })
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                ConditionalAlternative::Chain(nested)
            }
            (None, false) => ConditionalAlternative::Absent,
        };
        Ok(ConditionalBlocks {
            then_statements,
            alternative,
        })
    }

    fn within(
        &self,
        node: NodeId,
        (open, close): (u64, u64),
    ) -> Result<bool, SemanticCompilerFailure> {
        let record = self
            .topology()
            .node(node)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        Ok(record.first_terminal > open && record.last_terminal().is_some_and(|last| last < close))
    }

    pub(super) fn path(&self, node: NodeId) -> Result<&NodePath, SemanticCompilerFailure> {
        self.paths
            .get(node.index())
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)
    }

    pub(super) fn node_with_path(&self, path: &NodePath) -> Option<NodeId> {
        self.paths
            .iter()
            .position(|candidate| candidate == path)
            .and_then(NodeId::from_index)
    }

    pub(super) fn parent(&self, node: NodeId) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        self.topology()
            .node(node)
            .map(|record| record.parent)
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

    /// Copies the exact canonical source spelling owned by one production
    /// node. Semantic metadata uses this only while the source bundle is
    /// live; it is not a portable source identity or a second parser.
    pub(super) fn source_spelling(&self, node: NodeId) -> Result<String, SemanticCompilerFailure> {
        let coordinate = self.coordinate(node)?;
        let span = self
            .resolved
            .syntax()
            .classified_bundle()
            .source_bundle()
            .span(coordinate.source(), coordinate.start(), coordinate.end())
            .map_err(|_| SemanticCompilerFailure::InvalidCanonicalTree)?;
        std::str::from_utf8(span.bytes())
            .map(str::to_owned)
            .map_err(|_| SemanticCompilerFailure::InvalidCanonicalTree)
    }

    /// Resolves one checked node path to its bundle-local logical source and
    /// exact byte extent. The pair is stable only within this checked program.
    pub(super) fn source_identity(
        &self,
        path: &NodePath,
    ) -> Result<(String, SyntaxCoordinate), SemanticCompilerFailure> {
        let node = self
            .node_with_path(path)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let coordinate = self.coordinate(node)?;
        let logical_path = self
            .resolved
            .syntax()
            .classified_bundle()
            .source_bundle()
            .file(coordinate.source())
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
            .logical_path()
            .as_str()
            .to_owned();
        Ok((logical_path, coordinate))
    }

    /// Resolves one checked node path to its bundle-local logical source and
    /// one-based line number.
    ///
    /// The line is developer-channel presentation only: the non-normative
    /// permission ledger prints it. No mandatory record and no normative
    /// output reads it.
    pub(super) fn source_line(
        &self,
        path: &NodePath,
    ) -> Result<(String, u64), SemanticCompilerFailure> {
        let (logical_path, coordinate) = self.source_identity(path)?;
        let prefix = self
            .resolved
            .syntax()
            .classified_bundle()
            .source_bundle()
            .span(coordinate.source(), ByteOffset::new(0), coordinate.start())
            .map_err(|_| SemanticCompilerFailure::InvalidCanonicalTree)?;
        let newlines = prefix.bytes().iter().filter(|byte| **byte == b'\n').count();
        let line = u64::try_from(newlines)
            .map_err(|_| SemanticCompilerFailure::InvalidCanonicalTree)?
            .saturating_add(1);
        Ok((logical_path, line))
    }

    /// [`Self::source_spelling`] reached by node path rather than by node.
    pub(super) fn path_spelling(&self, path: &NodePath) -> Result<String, SemanticCompilerFailure> {
        let node = self
            .node_with_path(path)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.source_spelling(node)
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

    /// Returns every IDENT token owned directly by one node, in source order.
    ///
    /// The single-token reader rejects a second match, which is right for the
    /// productions carrying one name; an `input_label` carries two [GRAM-2],
    /// and its prefix and tail are distinct checked table facts [FN-7].
    pub(super) fn direct_identifiers(
        &self,
        node: NodeId,
    ) -> Result<Vec<usize>, SemanticCompilerFailure> {
        let classified = self.resolved.syntax().classified_bundle();
        let mut identifiers = Vec::new();
        for terminal in self.direct_token_indices(node)? {
            let token = classified
                .tokens()
                .get(*terminal)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if token.terminals().contains(TerminalPredicate::Identifier) {
                identifiers.push(*terminal);
            }
        }
        Ok(identifiers)
    }

    /// Every direct token of one node matching any of the given predicates,
    /// in source order. The single-token reader below rejects a second match
    /// of one predicate, which suits the productions carrying one such
    /// token; a candidate-grammar `const` operation carries two terms of one
    /// terminal class [CONST-1].
    pub(super) fn direct_tokens_matching(
        &self,
        node: NodeId,
        predicates: &[TerminalPredicate],
    ) -> Result<Vec<usize>, SemanticCompilerFailure> {
        let classified = self.resolved.syntax().classified_bundle();
        let mut matches = Vec::new();
        for terminal in self.direct_token_indices(node)? {
            let token = classified
                .tokens()
                .get(*terminal)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if predicates
                .iter()
                .any(|predicate| token.terminals().contains(*predicate))
            {
                matches.push(*terminal);
            }
        }
        Ok(matches)
    }

    pub(super) fn direct_token_with(
        &self,
        node: NodeId,
        predicate: TerminalPredicate,
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
