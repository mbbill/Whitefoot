use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{DeclarationId, Production, SemanticCompilerFailure, SemanticIssueKind, SemanticRule};

use super::super::super::super::model::{CheckedExpression, CheckedMode, CheckedType};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PreludeType, TypedExpression,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_conversion(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: "cvt".to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        let [source, destination] = self.numeric_type_arguments(node, function)?;
        if source == destination {
            return self.issue_node(SemanticRule::Op6, node, SemanticIssueKind::InvalidOperation);
        }
        let result = if source.converts_totally_to(destination) {
            destination.ty()
        } else {
            let error = CheckedType::Nominal(self.prelude_nominal(PreludeType::NarrowError)?);
            CheckedType::Nominal(
                self.prelude_nominal(PreludeType::Result(destination.ty(), error))?,
            )
        };
        let atom = self
            .operation_atoms(node, 1)?
            .into_iter()
            .next()
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let argument = self.check_atom(function, atom, bindings, loop_depth)?;
        if argument.expression.ty() != source.ty() || argument.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atom,
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(source.ty())?),
                    self.checked_value_name(argument.mode, argument.expression.ty())?,
                ),
            );
        }
        Ok(TypedExpression::owned(
            CheckedExpression::NumericConversion {
                carrier: self.tree.path(node)?.clone(),
                source,
                destination,
                value: Box::new(argument.expression),
                result,
            },
            EffectSet::NONE.union(argument.effects),
        ))
    }
}
