use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, ProductionV0_15, SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_15,
    UnsupportedSemanticFeatureV0_15,
};

use super::super::super::super::model::{CheckedExpression, CheckedMode, CheckedType};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PreludeType, TypedExpression,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_integer_conversion(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, ProductionV0_15::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRuleV0_15::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: "cvt".to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        let targs = self
            .tree
            .first_child_with(node, ProductionV0_15::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_15::Fn2,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let targs = self.tree.children_with(targs, ProductionV0_15::Targ)?;
        if targs.len() != 2 {
            return self.issue_node(
                SemanticRuleV0_15::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let mut types = Vec::with_capacity(2);
        for targ in targs {
            let type_node = self
                .tree
                .first_child_with(targ, ProductionV0_15::Type)?
                .ok_or_else(|| {
                    self.issue_value(
                        SemanticRuleV0_15::Op1,
                        node,
                        SemanticIssueKind::InvalidOperation,
                    )
                })?;
            let integer = match self.parse_type_with(type_node, &function.substitution)? {
                CheckedType::Integer(integer) => integer,
                CheckedType::GenericInt(_) => {
                    return self.unsupported(UnsupportedSemanticFeatureV0_15::Generics, type_node);
                }
                _ => {
                    return self.issue_node(
                        SemanticRuleV0_15::Op1,
                        node,
                        SemanticIssueKind::InvalidOperation,
                    );
                }
            };
            types.push(integer);
        }
        let [source, destination] = types.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        if source == destination {
            return self.issue_node(
                SemanticRuleV0_15::Op6,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let atom = self
            .operation_atoms(node, 1)?
            .into_iter()
            .next()
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let argument = self.check_atom(function, atom, bindings, loop_depth)?;
        if argument.expression.ty() != CheckedType::Integer(*source)
            || argument.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRuleV0_15::Type5,
                atom,
                SemanticIssueKind::TypeMismatch,
            );
        }
        let result = if source.converts_totally_to(*destination) {
            CheckedType::Integer(*destination)
        } else {
            let error = CheckedType::Nominal(self.prelude_nominal(PreludeType::NarrowError)?);
            CheckedType::Nominal(self.prelude_nominal(PreludeType::Result(
                CheckedType::Integer(*destination),
                error,
            ))?)
        };
        Ok(TypedExpression::owned(
            CheckedExpression::IntegerConversion {
                source: *source,
                destination: *destination,
                value: Box::new(argument.expression),
                result,
            },
            EffectSet::NONE.union(argument.effects),
        ))
    }
}
