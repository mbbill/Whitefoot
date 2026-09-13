//! Structural equality of the existing FN-8/FN-9 clause trees after binding.
//! Ordinary clause checking owns formation and define expansion here too.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{Production, SemanticCompilerFailure, SemanticRule};

use super::super::super::goal::GoalTemplate;
use super::super::super::model::{BindingId, CheckedType};
use super::super::super::postcondition::PostconditionConstantOrigin;
use super::super::requires::{ExpandedClauseDatum, ExpandedClauseExpression};
use super::super::{CheckStop, Checker, ControlCounters, FunctionSignature};

#[derive(Eq, PartialEq)]
struct InterfaceContracts {
    requires: Vec<GoalTemplate>,
    ensures: Vec<InterfaceEnsures>,
}

#[derive(Eq, PartialEq)]
struct InterfaceEnsures {
    ordinal: u32,
    variant: Option<crate::BuiltinPreludeId>,
    field: Option<crate::BuiltinPreludeId>,
    expression: ExpandedClauseExpression,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_formal_contract_formation(
        &self,
        signature: &FunctionSignature,
    ) -> Result<(), CheckStop> {
        self.check_entry_formers(signature)?;
        self.behavior_contracts(signature)?;
        Ok(())
    }

    pub(super) fn check_behavior_contracts(
        &self,
        node: NodeId,
        formal: &FunctionSignature,
        actual: &FunctionSignature,
    ) -> Result<(), CheckStop> {
        // Retain formal source identities for resolution, with the exact
        // implementation types and alpha-renamed signature regions.
        let mut normalized = formal.clone();
        let mut regions = formal.substitution.region_arguments().to_vec();
        regions.extend(
            formal
                .region_parameters
                .iter()
                .copied()
                .zip(actual.region_parameters.iter().copied()),
        );
        normalized.substitution = normalized.substitution.with_regions(regions);
        normalized
            .region_parameters
            .clone_from(&actual.region_parameters);
        for (left, right) in normalized.parameters.iter_mut().zip(&actual.parameters) {
            left.mode = right.mode;
            left.ty = right.ty;
        }
        for (left, right) in normalized.results.iter_mut().zip(&actual.results) {
            left.mode = right.mode;
            left.ty = right.ty;
        }
        normalized.result = actual.result;
        normalized.result_mode = actual.result_mode;
        normalized.result_list = actual.result_list;
        if self.behavior_contracts(&normalized)? != self.behavior_contracts(actual)? {
            return self.behavior_mismatch(SemanticRule::Fn4, node, "ordered requirements and ensures match structurally after substitution and define expansion");
        }
        Ok(())
    }

    fn behavior_contracts(
        &self,
        signature: &FunctionSignature,
    ) -> Result<InterfaceContracts, CheckStop> {
        let Some(block) = self
            .tree
            .first_child_with(signature.node, Production::ContractBlock)?
        else {
            return Ok(InterfaceContracts {
                requires: Vec::new(),
                ensures: Vec::new(),
            });
        };
        let mut bindings = HashMap::new();
        for (ordinal, parameter) in signature.parameters.iter().enumerate() {
            let binding = BindingId(
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            bindings.insert(
                parameter.declaration,
                self.parameter_local(parameter, binding)?,
            );
        }
        let mut next_binding =
            u32::try_from(bindings.len()).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
        let mut next_loop = 0;
        let mut names = signature
            .parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect();
        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
            binding_names: &mut names,
        };
        let requires = self
            .check_requires(signature, block, &mut bindings.clone(), &mut counters)?
            .requirements
            .into_iter()
            .map(|requirement| requirement.template)
            .collect();
        let mut ensures = Vec::new();
        let selectors = self.postcondition_selectors_from_source(signature)?;
        let mut relations = Vec::with_capacity(selectors.len());
        for selector in &selectors {
            relations.push(self.check_postcondition_clause(
                signature,
                selector,
                &mut bindings.clone(),
                &mut counters,
            )?);
            let mut expression = self.expand_postcondition_clause(
                signature,
                selector,
                &mut bindings.clone(),
                &mut counters,
            )?;
            normalize_substituted_constants(&mut expression);
            ensures.push(InterfaceEnsures {
                ordinal: selector.ordinal,
                variant: selector.variant,
                field: selector.field.as_ref().map(|field| field.declaration),
                expression,
            });
        }
        self.check_published_relation_consistency(signature, &selectors, &relations)?;
        Ok(InterfaceContracts { requires, ensures })
    }
}

fn normalize_substituted_constants(expression: &mut ExpandedClauseExpression) {
    match expression {
        ExpandedClauseExpression::Datum(ExpandedClauseDatum::Literal { value, origin }) => {
            if matches!(value.ty(), CheckedType::Integer(_))
                && matches!(
                    origin,
                    PostconditionConstantOrigin::ConstGeneric { .. }
                        | PostconditionConstantOrigin::GenericNumericIdentity { .. }
                )
            {
                *origin = PostconditionConstantOrigin::Literal;
            }
        }
        ExpandedClauseExpression::Operation { arguments, .. } => {
            for argument in arguments {
                normalize_substituted_constants(argument);
            }
        }
        ExpandedClauseExpression::Datum(_)
        | ExpandedClauseExpression::InvalidSelectorUse { .. } => {}
    }
}
