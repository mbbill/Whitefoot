use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, Production, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::super::super::model::{
    CheckedExpression, CheckedMode, CheckedNumericType, CheckedType,
};
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
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(SemanticRule::Fn2, node, SemanticIssueKind::InvalidOperation)
            })?;
        let targs = self.tree.children_with(targs, Production::Targ)?;
        if targs.len() != 2 {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let mut types = Vec::with_capacity(2);
        for targ in targs {
            let type_node = self
                .tree
                .first_child_with(targ, Production::Type)?
                .ok_or_else(|| {
                    self.issue_value(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation)
                })?;
            let ty = match self.parse_type_with(type_node, &function.substitution)? {
                CheckedType::Integer(ty) => CheckedNumericType::Integer(ty),
                CheckedType::Float(ty) => CheckedNumericType::Float(ty),
                CheckedType::GenericInt(_) => {
                    return self.unsupported(UnsupportedSemanticFeature::Generics, type_node);
                }
                _ => {
                    return self.issue_node(
                        SemanticRule::Op1,
                        node,
                        SemanticIssueKind::InvalidOperation,
                    );
                }
            };
            types.push(ty);
        }
        let [source, destination] = types.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        if source == destination {
            return self.issue_node(SemanticRule::Op6, node, SemanticIssueKind::InvalidOperation);
        }
        let result = if source.converts_totally_to(*destination) {
            destination.ty()
        } else if let (CheckedNumericType::Integer(_), CheckedNumericType::Integer(destination)) =
            (*source, *destination)
        {
            let error = CheckedType::Nominal(self.prelude_nominal(PreludeType::NarrowError)?);
            CheckedType::Nominal(self.prelude_nominal(PreludeType::Result(
                CheckedType::Integer(destination),
                error,
            ))?)
        } else {
            return self.unsupported(UnsupportedSemanticFeature::FloatingPointConversion, node);
        };
        let atom = self
            .operation_atoms(node, 1)?
            .into_iter()
            .next()
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let argument = self.check_atom(function, atom, bindings, loop_depth)?;
        if argument.expression.ty() != source.ty() || argument.mode != CheckedMode::Own {
            return self.issue_node(SemanticRule::Type5, atom, SemanticIssueKind::TypeMismatch);
        }
        Ok(TypedExpression::owned(
            CheckedExpression::NumericConversion {
                source: *source,
                destination: *destination,
                value: Box::new(argument.expression),
                result,
            },
            EffectSet::NONE.union(argument.effects),
        ))
    }
}
