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
