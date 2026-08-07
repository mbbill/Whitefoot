use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::model::{
    BindingId, CheckedMode, CheckedNominalKind, CheckedStatement, CheckedType, PropagationContext,
};
use super::super::{CheckStop, Checker, FunctionSignature, LocalBinding, PreludeType};
use super::{ControlScope, StatementResult};

// [DIAG-1] the return position asks TYPE-7's implicit read before the
// operand's OWN-1 spelling judgments because TYPE-7 is defined first.
const _: () = assert!(
    SemanticRule::Type7.definition_rank() < SemanticRule::Own1.definition_rank(),
    "check_return_implicit_read precedes the operand's OWN-1 judgments"
);

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_propagate_let(
        &self,
        function: &FunctionSignature,
        propagate: NodeId,
        expected: CheckedType,
        declaration: DeclarationId,
        binding: BindingId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let expression_node = self
            .tree
            .first_child_with(propagate, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expected_operand = match function.result {
            CheckedType::Nominal(return_nominal) => match self.prelude_type(return_nominal) {
                Some(PreludeType::Result(_, error_type)) => Some(CheckedType::Nominal(
                    self.prelude_nominal(PreludeType::Result(expected, error_type))?,
                )),
                _ => None,
            },
            _ => None,
        };
        let value = self.check_consuming_expression_with_expected(
            function,
            expression_node,
            bindings,
            scope.loops.len(),
            expected_operand,
        )?;
        let holder_without_deref = value.mode != CheckedMode::Own
            || match value.expression.ty() {
                CheckedType::Nominal(nominal) => {
                    matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Box { .. })
                }
                _ => false,
            };
        if holder_without_deref {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        let CheckedType::Nominal(result_nominal) = value.expression.ty() else {
            return self.invalid_propagation(propagate);
        };
        let Some(PreludeType::Result(ok_type, error_type)) = self.prelude_type(result_nominal)
        else {
            return self.invalid_propagation(propagate);
        };
        let CheckedType::Nominal(return_nominal) = function.result else {
            return self.invalid_propagation(propagate);
        };
        let Some(PreludeType::Result(_, return_error_type)) = self.prelude_type(return_nominal)
        else {
            return self.invalid_propagation(propagate);
        };
        if ok_type != expected || error_type != return_error_type {
            return self.invalid_propagation(propagate);
        }
        let error_drops = self.live_affine_drops(bindings, &HashSet::new())?;
        if bindings
            .insert(
                declaration,
                LocalBinding {
                    binding,
                    declaration,
                    mode: CheckedMode::Own,
                    ty: expected,
                    live: true,
                    loop_depth: scope.loops.len(),
                    borrow: None,
                    slice: None,
                    slice_loans: Vec::new(),
                    suspended: false,
                },
            )
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(Self::continuing_statement(
            CheckedStatement::PropagateLet {
                binding,
                scrutinee: value.expression,
                result_nominal,
                return_nominal,
                ok_type,
                error_type,
                error_drops,
                context: PropagationContext {
                    function: function.name.clone(),
                    node_path: self.tree.path(propagate)?.clone(),
                },
            },
            value.effects,
        ))
    }

    fn invalid_propagation<ResultValue>(&self, node: NodeId) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Err3,
            node,
            SemanticIssueKind::InvalidPropagation,
        )
    }

    /// [TYPE-7]'s implicit read at return position: a live borrow-mode or
    /// box binding used where the written `rtype` requires its referent
    /// value is rejected citing TYPE-7, and [FN-1] forms no candidate for
    /// that use. TYPE-7's definition precedes OWN-1's spelling judgments,
    /// so this same-node rejection event is cited first
    /// [DIAG-1, `SemanticRule::definition_rank`].
    pub(super) fn check_return_implicit_read(
        &self,
        function: &FunctionSignature,
        expression_node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        if function.result_mode != CheckedMode::Own {
            return Ok(());
        }
        let atom = self.tree.only_child(expression_node)?;
        if self.tree.production(atom)? != Production::Atom {
            return Ok(());
        }
        let Some(place) = self.tree.first_child_with(atom, Production::Place)? else {
            return Ok(());
        };
        let Some(pbase) = self.tree.first_child_with(place, Production::Pbase)? else {
            return Ok(());
        };
        if !self.tree.children(pbase)?.is_empty()
            || !self
                .tree
                .children_with(place, Production::Psuffix)?
                .is_empty()
        {
            return Ok(());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(());
        };
        let Some(local) = bindings.get(&declaration) else {
            return Ok(());
        };
        if !local.live {
            return Ok(());
        }
        let referent = if local.mode != CheckedMode::Own {
            Some(local.ty)
        } else if let CheckedType::Nominal(nominal) = local.ty
            && let CheckedNominalKind::Box { referent } = self.nominal(nominal)?.kind
        {
            Some(referent)
        } else {
            None
        };
        if referent == Some(function.result) {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        Ok(())
    }
}
