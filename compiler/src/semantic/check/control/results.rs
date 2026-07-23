use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationId, ProductionV0_15, SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_15,
};

use super::super::super::model::{
    BindingId, CheckedMode, CheckedStatement, CheckedType, PropagationContext,
};
use super::super::{CheckStop, Checker, FunctionSignature, LocalBinding, PreludeType};
use super::{ControlScope, StatementResult};

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
            .first_child_with(propagate, ProductionV0_15::Expr)?
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
            SemanticRuleV0_15::Err3,
            node,
            SemanticIssueKind::InvalidPropagation,
        )
    }
}
