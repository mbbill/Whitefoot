use crate::syntax::{FinalizedTopology, NodeId};
use crate::{NodePath, Production};

use super::{ResolutionCompilerFailure, ScopeId, ScopeKind, ScopeRecord};

pub(crate) struct ScopeBuild {
    pub(crate) records: Vec<ScopeRecord>,
    node_scopes: Vec<Option<ScopeId>>,
    declaration_scopes: Vec<Option<ScopeId>>,
    paths: Vec<Option<NodePath>>,
}

impl ScopeBuild {
    pub(crate) fn build(topology: &FinalizedTopology) -> Result<Self, ResolutionCompilerFailure> {
        let mut build = Self {
            records: Vec::new(),
            node_scopes: vec![None; topology.nodes.len()],
            declaration_scopes: vec![None; topology.nodes.len()],
            paths: vec![None; topology.nodes.len()],
        };
        let root_path = NodePath {
            components: Vec::new(),
        };
        let unit = build.push_scope(None, ScopeKind::CompilationUnit, root_path.clone())?;
        let mut tasks = vec![(topology.root, unit, root_path)];
        while let Some((node_id, current_scope, path)) = tasks.pop() {
            if build
                .node_scopes
                .get(node_id.index())
                .and_then(|scope| *scope)
                .is_some()
            {
                return Err(ResolutionCompilerFailure::InvalidScopeTree);
            }
            let node = topology
                .node(node_id)
                .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
            build.node_scopes[node_id.index()] = Some(current_scope);
            build.paths[node_id.index()] = Some(path.clone());
            let children = topology
                .node_children(node_id)
                .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;

            let mut child_scopes = vec![current_scope; children.len()];
            match node.production {
                Production::StructDecl | Production::EnumDecl | Production::ContractDecl => {
                    if children.iter().any(|child| {
                        topology
                            .node(*child)
                            .is_some_and(|record| record.production == Production::Generics)
                    }) {
                        let generic = build.push_scope(
                            Some(current_scope),
                            ScopeKind::DeclarationGenerics,
                            path.clone(),
                        )?;
                        child_scopes.fill(generic);
                    }
                }
                Production::FnDecl => {
                    let generic = if children.iter().any(|child| {
                        topology
                            .node(*child)
                            .is_some_and(|record| record.production == Production::Generics)
                    }) {
                        build.push_scope(
                            Some(current_scope),
                            ScopeKind::DeclarationGenerics,
                            path.clone(),
                        )?
                    } else {
                        current_scope
                    };
                    let signature = build.push_scope(
                        Some(generic),
                        ScopeKind::FunctionSignature,
                        path.clone(),
                    )?;
                    let body =
                        build.push_scope(Some(signature), ScopeKind::FunctionBody, path.clone())?;
                    for (index, child) in children.iter().enumerate() {
                        let production = topology
                            .node(*child)
                            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
                            .production;
                        child_scopes[index] = match production {
                            Production::Generics => generic,
                            Production::RequiresBlock => build.push_scope(
                                Some(signature),
                                ScopeKind::RequiresBlock,
                                path.clone(),
                            )?,
                            Production::Doc | Production::Stmt => body,
                            _ => signature,
                        };
                    }
                }
                Production::FnSig => {
                    let signature = build.push_scope(
                        Some(current_scope),
                        ScopeKind::ContractSignature,
                        path.clone(),
                    )?;
                    child_scopes.fill(signature);
                }
                Production::LoopStmt => {
                    let label = build.push_scope(
                        Some(current_scope),
                        ScopeKind::LoopLabel,
                        path.clone(),
                    )?;
                    let body =
                        build.push_scope(Some(label), ScopeKind::NestedBody, path.clone())?;
                    build.declaration_scopes[node_id.index()] = Some(label);
                    assign_nested_body_scopes(topology, children, &mut child_scopes, label, body)?;
                }
                Production::ForStmt => {
                    let range = build.push_scope(
                        Some(current_scope),
                        ScopeKind::CountedRange,
                        path.clone(),
                    )?;
                    let body =
                        build.push_scope(Some(range), ScopeKind::NestedBody, path.clone())?;
                    // The direct LABEL and IDENT declarations map to `range`,
                    // while both endpoint atoms deliberately stay in the
                    // enclosing scope. Only statements enter the body scope,
                    // making both declarations body-only without a second
                    // grammar production or a multi-scope owner table.
                    build.declaration_scopes[node_id.index()] = Some(range);
                    assign_counted_range_scopes(
                        topology,
                        children,
                        &mut child_scopes,
                        current_scope,
                        body,
                    )?;
                }
                Production::RegionStmt => {
                    let region = build.push_scope(
                        Some(current_scope),
                        ScopeKind::LocalRegion,
                        path.clone(),
                    )?;
                    let body =
                        build.push_scope(Some(region), ScopeKind::NestedBody, path.clone())?;
                    build.declaration_scopes[node_id.index()] = Some(region);
                    assign_nested_body_scopes(topology, children, &mut child_scopes, region, body)?;
                }
                Production::Arm => {
                    let arm =
                        build.push_scope(Some(current_scope), ScopeKind::Arm, path.clone())?;
                    let body = build.push_scope(Some(arm), ScopeKind::NestedBody, path.clone())?;
                    build.declaration_scopes[node_id.index()] = Some(arm);
                    assign_nested_body_scopes(topology, children, &mut child_scopes, arm, body)?;
                }
                // A conditional's two blocks are two lexical blocks, and they
                // are the one construct whose blocks are not nested
                // productions: [GRAM-4] hangs both `stmt*` sequences off the
                // same `if_stmt` or `value_if` node, so their statements can
                // only be told apart by the brace pair each falls inside.
                // Without this, a `let` in either block would declare into the
                // enclosing block and collide with a sibling branch's binder or
                // with a later binder of the same spelling in the enclosing
                // block, both of which [TYPE-6] admits as disjoint scopes.
                Production::IfStmt | Production::ValueIf => {
                    let [Some(then_range), else_range] = node.body_ranges() else {
                        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
                    };
                    let then_body = build.push_scope(
                        Some(current_scope),
                        ScopeKind::NestedBody,
                        path.clone(),
                    )?;
                    // Absent for the else-free `if` and for an `else if` chain,
                    // whose alternative is the nested conditional node rather
                    // than a block this node owns.
                    let else_body = match else_range {
                        Some(_) => Some(build.push_scope(
                            Some(current_scope),
                            ScopeKind::NestedBody,
                            path.clone(),
                        )?),
                        None => None,
                    };
                    for (index, child) in children.iter().enumerate() {
                        let child_record = topology
                            .node(*child)
                            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
                        // The condition `expr` and a chained conditional both
                        // read the enclosing scope, so only statements move.
                        if child_record.production != Production::Stmt {
                            continue;
                        }
                        child_scopes[index] = if within(
                            child_record.first_terminal,
                            child_record.last_terminal(),
                            then_range,
                        ) {
                            then_body
                        } else {
                            else_body.ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
                        };
                    }
                }
                _ => {}
            }

            for (index, child) in children.iter().enumerate().rev() {
                let child_record = topology
                    .node(*child)
                    .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
                let mut child_path = path.clone();
                child_path.components.push(child_record.child_ordinal);
                tasks.push((*child, child_scopes[index], child_path));
            }
        }
        if build.node_scopes.iter().any(Option::is_none) || build.paths.iter().any(Option::is_none)
        {
            return Err(ResolutionCompilerFailure::InvalidScopeTree);
        }
        Ok(build)
    }

    fn push_scope(
        &mut self,
        parent: Option<ScopeId>,
        kind: ScopeKind,
        owner: NodePath,
    ) -> Result<ScopeId, ResolutionCompilerFailure> {
        let id = ScopeId::from_index(self.records.len())
            .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        self.records.push(ScopeRecord {
            id,
            parent,
            kind,
            owner,
        });
        Ok(id)
    }

    pub(crate) fn node_scope(&self, node: NodeId) -> Result<ScopeId, ResolutionCompilerFailure> {
        self.node_scopes
            .get(node.index())
            .and_then(|scope| *scope)
            .ok_or(ResolutionCompilerFailure::InvalidScopeTree)
    }

    pub(crate) fn declaration_scope(
        &self,
        node: NodeId,
    ) -> Result<ScopeId, ResolutionCompilerFailure> {
        self.declaration_scopes
            .get(node.index())
            .and_then(|scope| *scope)
            .ok_or(ResolutionCompilerFailure::InvalidScopeTree)
    }

    pub(crate) fn path(&self, node: NodeId) -> Result<&NodePath, ResolutionCompilerFailure> {
        self.paths
            .get(node.index())
            .and_then(Option::as_ref)
            .ok_or(ResolutionCompilerFailure::InvalidScopeTree)
    }

    pub(crate) fn is_ancestor(&self, ancestor: ScopeId, mut scope: ScopeId) -> bool {
        loop {
            if scope == ancestor {
                return true;
            }
            let Some(parent) = self
                .records
                .get(scope.index())
                .and_then(ScopeRecord::parent)
            else {
                return false;
            };
            scope = parent;
        }
    }
}

/// Whether a node's complete terminal run lies strictly inside a brace pair.
fn within(first_terminal: u64, last_terminal: Option<u64>, (open, close): (u64, u64)) -> bool {
    first_terminal > open && last_terminal.is_some_and(|last| last < close)
}

fn assign_nested_body_scopes(
    topology: &FinalizedTopology,
    children: &[NodeId],
    child_scopes: &mut [ScopeId],
    introduced: ScopeId,
    body: ScopeId,
) -> Result<(), ResolutionCompilerFailure> {
    for (index, child) in children.iter().enumerate() {
        let production = topology
            .node(*child)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .production;
        child_scopes[index] = if production == Production::Stmt {
            body
        } else {
            introduced
        };
    }
    Ok(())
}

fn assign_counted_range_scopes(
    topology: &FinalizedTopology,
    children: &[NodeId],
    child_scopes: &mut [ScopeId],
    outer: ScopeId,
    body: ScopeId,
) -> Result<(), ResolutionCompilerFailure> {
    let mut endpoint_count = 0_u8;
    for (index, child) in children.iter().enumerate() {
        let production = topology
            .node(*child)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .production;
        child_scopes[index] = match production {
            Production::Atom => {
                endpoint_count = endpoint_count
                    .checked_add(1)
                    .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
                outer
            }
            Production::Stmt => body,
            _ => return Err(ResolutionCompilerFailure::InvalidCanonicalTree),
        };
    }
    if endpoint_count != 2 {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    }
    Ok(())
}
