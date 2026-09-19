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
        // implementation types.
        let mut normalized = formal.clone();
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
        // [FN-4] the actual's `requires` must be WEAKER than the formal's and
        // its `ensures` STRONGER, each decided by a fixed finite check inside
        // the affine entailment fragment: the discharging set is the only
        // premise set, and both sets are finite, so the check terminates.
        //
        // The refinement query itself is not yet wired to the declaration-level
        // proof context, so this check admits only the case in which the goal
        // is literally one of its own premises: a formal requirement the actual
        // also states, and a formal ensures relation the actual also states.
        // That is deliberately fail-closed [FN-4 has no permissive default]:
        // it refuses refinements the language admits and admits none it
        // refuses, and it is strictly weaker than the structural set equality
        // it replaces only in that the two sets may now differ in size.
        let formal_contracts = self.behavior_contracts(&normalized)?;
        let actual_contracts = self.behavior_contracts(actual)?;
        // Weaker precondition: every requirement the actual states must
        // already be one the formal's callers establish.
        if actual_contracts
            .requires
            .iter()
            .any(|goal| !formal_contracts.requires.contains(goal))
        {
            return self.behavior_mismatch(
                SemanticRule::Fn4,
                node,
                "a requirement no caller of the formal interface establishes",
            );
        }
        // Stronger postcondition: every relation the formal promises must
        // already be one the actual publishes.
        if formal_contracts.ensures.iter().any(|promised| {
            !actual_contracts.ensures.iter().any(|published| {
                published.variant == promised.variant
                    && published.field == promised.field
                    && published.expression == promised.expression
            })
        }) {
            return self.behavior_mismatch(
                SemanticRule::Fn4,
                node,
                "a promised relation the supplied function does not publish",
            );
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
