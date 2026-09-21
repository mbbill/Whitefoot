//! [FN-10] the source conditions for replacing the current activation.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, NodePath, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::model::CheckedMode;
use super::super::places::{PlaceRoot, ResolvedPlace};
use super::{CheckStop, Checker, FunctionSignature, LocalBinding};

impl Checker<'_, '_, '_, '_> {
    pub(super) fn is_musttail_call(&self, node: NodeId) -> Result<bool, CheckStop> {
        for token in self.tree.direct_token_indices(node)? {
            if self.tree.token_bytes(*token)? == b"musttail" {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn invalid_musttail<T>(
        &self,
        node: NodeId,
        condition: &'static str,
        subject: Option<String>,
    ) -> Result<T, CheckStop> {
        self.issue_node(
            SemanticRule::Fn10,
            node,
            SemanticIssueKind::InvalidMusttail { condition, subject },
        )
    }

    /// Check the written position even in clauses and constants, where a call
    /// does not reach the ordinary expression checker.
    pub(super) fn check_musttail_positions(&self) -> Result<(), CheckStop> {
        for call in self
            .tree
            .descendants_with(self.tree.root(), Production::Call)?
        {
            if !self.is_musttail_call(call)? {
                continue;
            }
            let expression = self
                .tree
                .parent(call)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let parent = self
                .tree
                .parent(expression)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if self.tree.production(expression)? != Production::Expr
                || self.tree.production(parent)? != Production::ReturnStmt
                || self.tree.children_with(parent, Production::Expr)?.len() != 1
            {
                return self.invalid_musttail(
                    call,
                    "musttail must be the sole expression of return",
                    None,
                );
            }
        }
        Ok(())
    }

    pub(super) fn check_musttail_callees(
        &self,
        function: &FunctionSignature,
    ) -> Result<(), CheckStop> {
        for call in self
            .tree
            .descendants_with(function.node, Production::Call)?
        {
            if !self.is_musttail_call(call)? {
                continue;
            }
            if self.tree.is_constructor_call(call)? || self.behavior_call_key(call)?.is_some() {
                return self.invalid_musttail(
                    call,
                    "musttail requires a direct call to the enclosing function",
                    None,
                );
            }
            let callee = self
                .tree
                .first_child_with(call, Production::Callee)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let usage = self.use_at_roles(
                callee,
                &[
                    LexicalUseRole::IdentifierCallee,
                    LexicalUseRole::OperationCallee,
                ],
            )?;
            if !matches!(usage.target(), ResolvedTarget::Source { declaration, class: DeclarationClass::Function } if declaration == function.declaration)
            {
                return self.invalid_musttail(
                    call,
                    "musttail requires a direct call to the enclosing function",
                    None,
                );
            }
        }
        Ok(())
    }

    pub(super) fn check_musttail_arguments(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        paths: &[Vec<ResolvedPlace>],
        modes: &[CheckedMode],
    ) -> Result<(), CheckStop> {
        for (ordinal, (paths, mode)) in paths.iter().zip(modes).enumerate() {
            if !mode.is_reference() {
                continue;
            }
            if paths.is_empty()
                || paths.iter().any(|path| {
                    !function.parameters.iter().any(|parameter| {
                        parameter.mode.is_reference()
                            && bindings
                                .get(&parameter.declaration)
                                .is_some_and(|local| path.root == PlaceRoot::Binding(local.binding))
                    })
                })
            {
                return self.invalid_musttail(node, "every musttail reference argument must be rooted at a reference parameter, not current-activation storage", Some(function.parameters[ordinal].name.clone()));
            }
        }
        Ok(())
    }

    pub(super) fn check_musttail_releases(
        &self,
        call: &NodePath,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let mut owners = bindings
            .values()
            .filter(|local| local.live && local.mode == CheckedMode::Own)
            .collect::<Vec<_>>();
        owners.sort_by_key(|owner| std::cmp::Reverse(owner.binding.0));
        for owner in owners {
            if !self.has_nonempty_release(owner.ty)? {
                continue;
            }
            let referenced = bindings.values().any(|local| {
                local.live
                    && local.reference.as_ref().is_some_and(|reference| {
                        reference.is_valid()
                            && reference
                                .paths
                                .iter()
                                .any(|path| path.root == PlaceRoot::Binding(owner.binding))
                    })
            });
            if referenced {
                let name = self
                    .resolved
                    .declarations()
                    .iter()
                    .find(|declaration| declaration.id() == owner.declaration)
                    .map(|declaration| declaration.spelling().to_owned());
                let node = self
                    .tree
                    .node_with_path(call)
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                return self.invalid_musttail(
                    node,
                    "a live reference prevents releasing this owner before the musttail transfer",
                    name,
                );
            }
        }
        Ok(())
    }
}
