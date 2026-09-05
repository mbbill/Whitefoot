use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, PostconditionCandidateRecord,
    PostconditionResolutionRecord, PostconditionSelectorClass, PreludeDeclarationId, Production,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation,
    SemanticRule, SourceOrigin,
};

use super::super::goal::{GoalOperation, GoalProjection};
use super::super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedIntegerOperation, CheckedMode,
    CheckedNominalKind, CheckedParameter, CheckedStatement, CheckedType, CheckedValue, FunctionId,
    IntegerType,
};
use super::super::postcondition::{
    CheckedPostcondition, CheckedPostconditionSelector, NormalizedRelation,
    PostconditionConstantOrigin, PostconditionFieldIdentity, PostconditionPlace,
    PostconditionPlaceRoot, PostconditionReturnDatum, PostconditionReturnPlace,
    PostconditionReturnPlaceRoot, RelationDatum, RelationTemplate, RelationTerm,
    SelectedPostconditionReturn,
};
use super::generics::GenericArgument;
use super::publication;
use super::requires::{ClauseKind, ExpandedClauseDatum, ExpandedClauseExpression};
use super::{CheckStop, Checker, ControlCounters, ControlScope, FunctionSignature, LocalBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectorAdmissionType {
    Fragment,
    ResultFragment,
    Symbolic,
    /// [CALL-4] one declared result of measured type. Its value is no [ENT-2]
    /// term, so only a measure over it is an admitted clause operand.
    Measured,
    Invalid,
}

#[derive(Clone, Copy)]
struct PostconditionBindingInfo {
    ty: CheckedType,
    implicit_deref: bool,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn with_postcondition_context<T>(
        &self,
        record: &PostconditionResolutionRecord,
        result_type: CheckedType,
        datums: Vec<(String, u32, CheckedType)>,
        check: impl FnOnce() -> Result<T, CheckStop>,
    ) -> Result<T, CheckStop> {
        let index = self
            .resolved
            .postconditions()
            .iter()
            .position(|candidate| candidate.block == record.block)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let previous = self
            .active_postcondition
            .replace(Some(super::PostconditionCheckContext {
                record: index,
                result_type,
            }));
        let previous_datums =
            std::mem::replace(&mut *self.active_result_datums.borrow_mut(), datums);
        let result = check();
        self.active_postcondition.set(previous);
        *self.active_result_datums.borrow_mut() = previous_datums;
        result
    }

    /// The result datums one [FN-9] clause admits, by written spelling
    /// [CALL-4].
    ///
    /// An unrouted clause admits every declared result ordinal's binder at
    /// that ordinal's own type. A routed clause admits its fresh payload
    /// datum for the ordinal the route names, and every other ordinal's
    /// binder unchanged; the routed ordinal's own whole-result binder stays
    /// unavailable [FN-9].
    pub(super) fn postcondition_result_datums(
        &self,
        record: &PostconditionResolutionRecord,
        signature: &FunctionSignature,
        selector: &CheckedPostconditionSelector,
    ) -> Vec<(String, u32, CheckedType)> {
        let routed = selector.variant.is_some();
        let mut datums = Vec::with_capacity(record.result_binders.len() + 1);
        for (ordinal, binder) in record.result_binders.iter().enumerate() {
            let Ok(ordinal) = u32::try_from(ordinal) else {
                continue;
            };
            if routed && ordinal == selector.ordinal {
                continue;
            }
            let Some(declared) = signature.results.get(ordinal as usize) else {
                continue;
            };
            datums.push((binder.spelling.clone(), ordinal, declared.ty));
        }
        if routed && let Some(field) = record.fields.first() {
            datums.push((
                field.candidate.spelling.clone(),
                selector.ordinal,
                selector.result_type,
            ));
        }
        datums
    }

    /// The result ordinal and datum type one written selector spelling names
    /// in the clause being checked, when it names one.
    fn active_result_datum(&self, spelling: &str) -> Option<(u32, CheckedType)> {
        self.active_result_datums
            .borrow()
            .iter()
            .find(|(candidate, _, _)| candidate == spelling)
            .map(|(_, ordinal, ty)| (*ordinal, *ty))
    }

    /// Supplies a value-only placeholder to the ordinary expression typer.
    /// The placeholder is discarded immediately after typing; relation
    /// identity is rebuilt from the resolver-owned selector-use record and
    /// never becomes a declaration, binding, or storage location.
    pub(super) fn postcondition_result_placeholder(
        &self,
        atom: NodeId,
    ) -> Result<Option<CheckedValue>, CheckStop> {
        let Some(context) = self.active_postcondition.get() else {
            return Ok(None);
        };
        let record = self
            .resolved
            .postconditions()
            .get(context.record)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let atom_path = self.tree.path(atom)?.components();
        let Some(usage) = record.selector_uses.iter().find(|usage| {
            let path = usage.origin.node().components();
            path.len() > atom_path.len() && path.starts_with(atom_path)
        }) else {
            return Ok(None);
        };
        // [CALL-4] the spelling names a result ordinal, and its datum type is
        // that ordinal's. An ordinal whose datum is not a fragment integer is
        // outside [FN-9]'s admitted operand set in this version.
        let ty = self
            .active_result_datum(&usage.spelling)
            .map_or(context.result_type, |(_, ty)| ty);
        let _ = context;
        Ok(Some(match ty {
            CheckedType::Integer(ty) => CheckedValue::Integer { ty, bits: 0 },
            CheckedType::GenericInt(_) => CheckedValue::NumericIdentity { ty, one: false },
            _ => {
                return self.issue_origin(
                    SemanticRule::Fn9,
                    &usage.origin,
                    SemanticIssueKind::InvalidPostconditionSelector,
                );
            }
        }))
    }

    /// Performs the one semantic subjudgment that DIAG-1 interleaves into
    /// resolution. This checker is throwaway: it reuses the ordinary nominal,
    /// constant, generic-cycle, signature, and FN-2 implementations, but none
    /// of the scratch identities or tables are published to the real checker.
    pub(super) fn preflight_postcondition_selectors(
        &mut self,
        items: &[NodeId],
    ) -> Result<(), CheckStop> {
        let records = self.resolved.postconditions().to_vec();
        if records.is_empty() {
            return Ok(());
        }

        let prepared = self.prepare_postcondition_selector_preflight(items);
        match prepared {
            Ok(()) => {}
            // A non-FN-9 source premise that has not succeeded establishes no
            // selector instance. The ordinary checker will publish that
            // verdict unless a resolver verdict was deliberately delayed by
            // FN-9, in which case the original resolver issue wins unchanged.
            Err(
                CheckStop::Issue(_)
                | CheckStop::Unsupported(_)
                | CheckStop::PostconditionPrerequisiteUnavailable,
            ) => return Ok(()),
            Err(stop) => return Err(stop),
        }

        let eligible = self.eligible_postcondition_functions()?;
        let mut admitted_records = Vec::new();
        for record in &records {
            let concrete = self
                .signatures
                .iter()
                .filter(|signature| {
                    eligible.contains(&signature.id)
                        && self
                            .tree
                            .path(signature.node)
                            .is_ok_and(|path| path == &record.function)
                })
                .cloned()
                .collect::<Vec<_>>();
            if concrete.is_empty() {
                let symbolic = match self.symbolic_postcondition_signature(record) {
                    Ok(symbolic) => symbolic,
                    Err(
                        CheckStop::Issue(_)
                        | CheckStop::Unsupported(_)
                        | CheckStop::PostconditionPrerequisiteUnavailable,
                    ) => {
                        continue;
                    }
                    Err(stop) => return Err(stop),
                };
                if let Some(signature) = symbolic {
                    let _ = self.admit_postcondition_selector(record, &signature, true)?;
                    admitted_records.push(record.clone());
                }
            } else {
                for signature in concrete {
                    let _ = self.admit_postcondition_selector(record, &signature, false)?;
                }
                admitted_records.push(record.clone());
            }
        }
        self.forward_delayed_postcondition_issue(&admitted_records)
    }

    fn prepare_postcondition_selector_preflight(
        &mut self,
        items: &[NodeId],
    ) -> Result<(), CheckStop> {
        self.declare_nominals_for_postconditions(items)?;
        self.collect_constants_for_postconditions(items)?;
        self.collect_function_templates_for_postconditions(items)?;
        self.collect_concrete_function_signatures_for_postconditions()
    }

    fn forward_delayed_postcondition_issue(
        &self,
        records: &[PostconditionResolutionRecord],
    ) -> Result<(), CheckStop> {
        // Inventory remains one global stage even though FN-9 delays the
        // entry-local slice. It therefore precedes every delayed entry lookup.
        if let Some(issue) = records
            .iter()
            .find_map(|record| record.entry_inventory_issue.clone())
        {
            return Err(CheckStop::Resolution(Box::new(issue)));
        }
        if let Some(issue) = records
            .iter()
            .find_map(|record| record.entry_resolution_issue.clone())
        {
            return Err(CheckStop::Resolution(Box::new(issue)));
        }
        Ok(())
    }

    fn symbolic_postcondition_signature(
        &mut self,
        record: &PostconditionResolutionRecord,
    ) -> Result<Option<FunctionSignature>, CheckStop> {
        let Some(template) = self
            .function_templates
            .iter()
            .find(|template| {
                self.tree
                    .path(template.node)
                    .is_ok_and(|path| path == &record.function)
            })
            .cloned()
        else {
            return Ok(None);
        };
        if self.postcondition_declaration_unavailable(template.declaration) {
            return Ok(None);
        }
        if template.generic_parameters.is_empty() {
            // A failed nongeneric header establishes no selector premise.
            return Ok(None);
        }
        if !self.postcondition_function_header_dependencies_available(template.node)? {
            return Ok(None);
        }
        let substitution = self.symbolic_generic_substitution(&template.generic_parameters)?;
        self.ensure_nominals_in_function_signature(template.node, &substitution)?;
        self.build_function_signature(&template, substitution, FunctionId(u32::MAX))
            .map(Some)
    }

    /// Computes the exact locally meaningful instance universe. H0 may
    /// temporarily materialize a generic signature from the type/const prefix
    /// of a call whose trailing region arguments are malformed. Such a call
    /// has not completed FN-2 and therefore contributes no selector instance.
    /// The scratch and real checkers each run this helper over their own dense
    /// identities; no FunctionId, NominalId, or CheckedType crosses between
    /// them.
    fn eligible_postcondition_functions(&self) -> Result<Vec<FunctionId>, CheckStop> {
        let mut eligible = self
            .signatures
            .iter()
            .filter(|signature| {
                self.templates_by_declaration
                    .get(&signature.declaration)
                    .and_then(|index| self.function_templates.get(*index))
                    .is_some_and(|template| template.generic_parameters.is_empty())
            })
            .map(|signature| signature.id)
            .collect::<Vec<_>>();

        let mut cursor = 0_usize;
        while cursor < eligible.len() {
            let caller = self
                .signatures
                .get(eligible[cursor].0 as usize)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for call in self.tree.descendants_with(caller.node, Production::Call)? {
                if self.call_is_inside_postcondition(call)? {
                    continue;
                }
                let Some((_, template)) = self.called_function_template(call)? else {
                    continue;
                };
                if !self.postcondition_call_arguments_have_links(call)? {
                    continue;
                }
                let target = if template.generic_parameters.is_empty() {
                    self.functions_by_declaration
                        .get(&template.declaration)
                        .into_iter()
                        .flatten()
                        .copied()
                        .next()
                } else {
                    let substitution =
                        match self.call_generic_substitution(call, &template, &caller.substitution)
                        {
                            Ok(substitution) if substitution.is_concrete() => substitution,
                            Ok(_)
                            | Err(
                                CheckStop::Issue(_)
                                | CheckStop::Unsupported(_)
                                | CheckStop::PostconditionPrerequisiteUnavailable,
                            ) => {
                                continue;
                            }
                            Err(stop) => return Err(stop),
                        };
                    self.functions_by_declaration
                        .get(&template.declaration)
                        .into_iter()
                        .flatten()
                        .copied()
                        .find(|id| {
                            self.signatures
                                .get(id.0 as usize)
                                .is_some_and(|signature| signature.substitution == substitution)
                        })
                };
                let Some(target) = target else {
                    continue;
                };
                let target_signature = self
                    .signatures
                    .get(target.0 as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                match self.call_region_arguments(call, target_signature) {
                    Ok(_) => {}
                    Err(
                        CheckStop::Issue(_)
                        | CheckStop::Unsupported(_)
                        | CheckStop::PostconditionPrerequisiteUnavailable,
                    ) => continue,
                    Err(stop) => return Err(stop),
                }
                if !eligible.contains(&target) {
                    eligible.push(target);
                }
            }
            cursor = cursor
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        Ok(eligible)
    }

    /// Builds final selector metadata after the ordinary H0 signature path.
    /// The verdict-bearing form of the same validation already ran in the
    /// throwaway preflight; any divergence here is a compiler invariant
    /// failure.
    pub(super) fn admit_postcondition_selectors(&mut self) -> Result<(), CheckStop> {
        self.admit_postcondition_selectors_including(&[])
    }

    /// Builds selectors for the ordinary locally reachable set plus concrete
    /// instances replayed from an uninstantiated generic source body. Those
    /// replayed calls were already checked in the schema pass, but their final
    /// FunctionIds are not reachable from a nongeneric concrete caller.
    pub(super) fn admit_postcondition_selectors_including(
        &mut self,
        additional: &[FunctionId],
    ) -> Result<(), CheckStop> {
        let records = self.resolved.postconditions().to_vec();
        if records.is_empty() {
            return Ok(());
        }
        let mut eligible = self.eligible_postcondition_functions()?;
        for function in additional {
            if !eligible.contains(function) {
                eligible.push(*function);
            }
        }
        for record in &records {
            let concrete = self
                .signatures
                .iter()
                .filter(|signature| {
                    eligible.contains(&signature.id)
                        && self
                            .tree
                            .path(signature.node)
                            .is_ok_and(|path| path == &record.function)
                })
                .cloned()
                .collect::<Vec<_>>();
            for signature in concrete {
                let admitted = match self.admit_postcondition_selector(record, &signature, false) {
                    Ok(admitted) => admitted,
                    Err(CheckStop::Issue(_)) => {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                    Err(stop) => return Err(stop),
                };
                self.postcondition_selectors.push(admitted);
            }
        }
        Ok(())
    }

    pub(super) fn postcondition_selectors_for_signature(
        &self,
        signature: &FunctionSignature,
    ) -> Result<Vec<CheckedPostconditionSelector>, CheckStop> {
        if signature.substitution.is_concrete() {
            return Ok(self
                .postcondition_selectors
                .iter()
                .filter(|selector| selector.function == signature.id)
                .cloned()
                .collect());
        }
        let function = self.tree.path(signature.node)?;
        let mut selectors = Vec::new();
        for record in self
            .resolved
            .postconditions()
            .iter()
            .filter(|record| &record.function == function)
        {
            let selector = self.admit_postcondition_selector(record, signature, true)?;
            // An unbounded symbolic type has no concrete FN-2 fragment
            // judgment yet. Its selector is provisionally admitted for
            // resolution order, but clause typing and selected-return
            // classification wait for a concrete instance. A declared Int
            // bound already supplies the exact symbolic integer row used by
            // ordinary generic validation.
            if !matches!(selector.result_type, CheckedType::Generic(_)) {
                selectors.push(selector);
            }
        }
        Ok(selectors)
    }

    pub(super) fn check_postcondition_clause(
        &self,
        function: &FunctionSignature,
        selector: &CheckedPostconditionSelector,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
    ) -> Result<RelationTemplate, CheckStop> {
        let record = self
            .resolved
            .postconditions()
            .iter()
            .find(|record| record.block == selector.block)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let clause = self
            .tree
            .node_with_path(&record.block)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let contract = self
            .tree
            .first_child_with(function.node, Production::ContractBlock)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let datums = self.postcondition_result_datums(record, function, selector);
        self.with_postcondition_context(record, selector.result_type, datums, || {
            let mut expanded_bindings = HashMap::<BindingId, ExpandedClauseExpression>::new();
            for (ordinal, parameter) in function.parameters.iter().enumerate() {
                let local = bindings
                    .get(&parameter.declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                expanded_bindings.insert(
                    local.binding,
                    ExpandedClauseExpression::Datum(ExpandedClauseDatum::Parameter {
                        ordinal: u32::try_from(ordinal)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                        projections: Vec::new(),
                        ty: parameter.ty,
                    }),
                );
            }
            for definition in self
                .tree
                .children_with(contract, Production::ContractDefine)?
            {
                let expression = self
                    .tree
                    .first_child_with(definition, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                if !self.validate_clause_computation(
                    ClauseKind::Postcondition(record),
                    definition,
                    expression,
                )? {
                    return self.invalid_clause(ClauseKind::Postcondition(record), definition);
                }
                let checked = self.check_statement(
                    function,
                    definition,
                    bindings,
                    counters,
                    ControlScope {
                        loops: &[],
                        give_context: None,
                    },
                )?;
                if !checked.can_continue {
                    return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
                }
                let CheckedStatement::Let { binding, value, .. } = &checked.statement else {
                    return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
                };
                self.validate_clause_copy_local(
                    ClauseKind::Postcondition(record),
                    definition,
                    *binding,
                    bindings,
                )?;
                let expanded =
                    self.build_clause_expression(expression, value, bindings, &expanded_bindings)?;
                if expanded.contains_invalid_selector_use() {
                    return self.invalid_postcondition_relation(expression);
                }
                expanded_bindings.insert(*binding, expanded);
            }
            let expression = self
                .tree
                .first_child_with(clause, Production::ClauseExpr)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.validate_clause_condition(ClauseKind::Postcondition(record), clause, expression)?;
            let condition = self.check_expression(function, expression, bindings, 0)?;
            if condition.mode != CheckedMode::Own || condition.expression.ty() != CheckedType::Bool
            {
                return self.issue_node(
                    SemanticRule::Op5,
                    expression,
                    SemanticIssueKind::InvalidPredicateCondition,
                );
            }
            let expanded = self.build_clause_expression(
                expression,
                &condition.expression,
                bindings,
                &expanded_bindings,
            )?;
            let relation = self.postcondition_relation(expression, expanded)?;
            self.reject_state_parameter_measure(function, expression, &relation)?;
            Ok(relation)
        })
    }

    /// [MSR-3] one denotation per position, keyed on the parameter's mode.
    ///
    /// A `&uniq` parameter is the one position from which a callee could
    /// leave a caller holding a measure of a value the callee replaced, so a
    /// source-declared `ensures` may not name one: the relation would be a
    /// claim about a caller's object at a point the callee cannot name. The
    /// same operand in a `requires` denotes the call datum and stays
    /// admissible, which is why this judgment is stated over the `ensures`
    /// clause alone.
    fn reject_state_parameter_measure(
        &self,
        function: &FunctionSignature,
        expression: NodeId,
        relation: &RelationTemplate,
    ) -> Result<(), CheckStop> {
        for operand in &relation.operands {
            let RelationDatum::Measure(_, place) = &operand.datum else {
                continue;
            };
            // [CALL-4] a measure over a result place names no parameter, so
            // [MSR-3]'s state-parameter inadmissibility does not reach it.
            let PostconditionPlaceRoot::Parameter { ordinal } = place.root else {
                continue;
            };
            let Some(parameter) = function.parameters.get(ordinal as usize) else {
                continue;
            };
            if matches!(parameter.mode, CheckedMode::Unique(_)) {
                return self.issue_node(
                    SemanticRule::Msr3,
                    expression,
                    SemanticIssueKind::InadmissibleStateParameterMeasure {
                        parameter: parameter.name.clone(),
                        mechanical_fix: "take the value by value and relate the result, or state the fact as a requires",
                    },
                );
            }
        }
        Ok(())
    }

    /// [CALL-6] a declaration whose published relations instantiate to a
    /// contradiction is refused at the declaration.
    ///
    /// The set is partitioned by route first, because a routed clause is
    /// established only on its own arm [CALL-6] and two clauses on two arms
    /// are never in one caller state together. Every unrouted clause is in
    /// every route's set, since an unrouted clause selects every explicit
    /// return.
    pub(super) fn check_published_relation_consistency(
        &self,
        function: &FunctionSignature,
        selectors: &[CheckedPostconditionSelector],
        relations: &[RelationTemplate],
    ) -> Result<(), CheckStop> {
        let mut routes: Vec<Option<PreludeDeclarationId>> = vec![None];
        for selector in selectors {
            if let Some(variant) = selector.variant
                && !routes.contains(&Some(variant))
            {
                routes.push(Some(variant));
            }
        }
        for route in routes {
            let published = selectors
                .iter()
                .zip(relations)
                .filter(|(selector, _)| selector.variant.is_none() || selector.variant == route)
                .map(|(_, relation)| relation)
                .collect::<Vec<_>>();
            if published.len() < 2 || !publication::relations_are_contradictory(&published) {
                continue;
            }
            let rendered = selectors
                .iter()
                .zip(relations)
                .filter(|(selector, _)| selector.variant.is_none() || selector.variant == route)
                .map(|(selector, _)| self.postcondition_clause_text(selector))
                .collect::<Result<Vec<_>, _>>()?;
            return self.issue_node(
                SemanticRule::Call6,
                function.node,
                SemanticIssueKind::ContradictoryPublishedRelations {
                    relations: rendered,
                    mechanical_fix: "state one consistent relation set: a contract whose clauses cannot hold together publishes every fact at every caller",
                },
            );
        }
        Ok(())
    }

    /// The written text of one `ensures_clause`, for the [CALL-6]
    /// diagnostic. The clause is quoted as the writer wrote it, because the
    /// judgment is over the written set and the fix is to rewrite it.
    fn postcondition_clause_text(
        &self,
        selector: &CheckedPostconditionSelector,
    ) -> Result<String, CheckStop> {
        let clause = self
            .tree
            .node_with_path(&selector.block)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(self.tree.source_spelling(clause)?.trim().to_owned())
    }

    fn postcondition_relation(
        &self,
        final_expression: NodeId,
        expanded: ExpandedClauseExpression,
    ) -> Result<RelationTemplate, CheckStop> {
        let ExpandedClauseExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation,
                    operand_type,
                },
            arguments,
            ..
        } = expanded
        else {
            return self.invalid_postcondition_relation(final_expression);
        };
        let [left, right] = arguments.as_slice() else {
            return self.invalid_postcondition_relation(final_expression);
        };
        let Some(left) = self.postcondition_relation_term(left, operand_type) else {
            return self.invalid_postcondition_relation(final_expression);
        };
        let Some(right) = self.postcondition_relation_term(right, operand_type) else {
            return self.invalid_postcondition_relation(final_expression);
        };
        if left.ty() != operand_type || right.ty() != operand_type {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if !left.contains_result() && !right.contains_result() {
            return self.invalid_postcondition_relation(final_expression);
        }
        let normalized = match operation {
            super::super::model::CheckedIntegerOperation::Equal => NormalizedRelation::Equal,
            super::super::model::CheckedIntegerOperation::NotEqual => NormalizedRelation::NotEqual,
            super::super::model::CheckedIntegerOperation::Less => NormalizedRelation::UpperBound {
                left: 0,
                right: 1,
                strict: true,
            },
            super::super::model::CheckedIntegerOperation::LessEqual => {
                NormalizedRelation::UpperBound {
                    left: 0,
                    right: 1,
                    strict: false,
                }
            }
            super::super::model::CheckedIntegerOperation::Greater => {
                NormalizedRelation::UpperBound {
                    left: 1,
                    right: 0,
                    strict: true,
                }
            }
            super::super::model::CheckedIntegerOperation::GreaterEqual => {
                NormalizedRelation::UpperBound {
                    left: 1,
                    right: 0,
                    strict: false,
                }
            }
            _ => return self.invalid_postcondition_relation(final_expression),
        };
        Ok(RelationTemplate {
            operation,
            operands: [left, right],
            normalized,
        })
    }

    /// One relation term [FN-9]: the datum one clause side names, displaced
    /// by the constant the rest of that side reduces to.
    ///
    /// A side carrying two datums, or a datum with any coefficient other than
    /// one, is outside the difference-bound fragment [ENT-4] and yields
    /// `None`, which is the ordinary FN-9 rejection at the clause.
    fn postcondition_relation_term(
        &self,
        expanded: &ExpandedClauseExpression,
        operand_type: CheckedType,
    ) -> Option<RelationTerm> {
        let (datum, displacement) = self.postcondition_relation_operand(expanded)?;
        match datum {
            Some(datum) => Some(RelationTerm {
                datum,
                displacement,
            }),
            // A side with no datum reduced to one constant; it is a literal
            // operand exactly as a written one is, and the fragment holds it
            // in the operand's own type.
            None => Some(RelationTerm::undisplaced(RelationDatum::Literal {
                value: CheckedValue::Integer {
                    ty: match operand_type {
                        CheckedType::Integer(ty) => ty,
                        _ => return None,
                    },
                    bits: representable_bits(operand_type, displacement)?,
                },
                origin: PostconditionConstantOrigin::Literal,
            })),
        }
    }

    /// One clause side's affine expression, as at most one datum with
    /// coefficient one plus a constant [MSR-5].
    fn postcondition_relation_operand(
        &self,
        expanded: &ExpandedClauseExpression,
    ) -> Option<(Option<RelationDatum>, i128)> {
        if let ExpandedClauseExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation:
                        operation @ (CheckedIntegerOperation::AddExact
                        | CheckedIntegerOperation::SubtractExact
                        | CheckedIntegerOperation::MultiplyExact),
                    ..
                },
            arguments,
            ..
        } = expanded
        {
            let [left, right] = arguments.as_slice() else {
                return None;
            };
            let (left_datum, left_value) = self.postcondition_relation_operand(left)?;
            let (right_datum, right_value) = self.postcondition_relation_operand(right)?;
            return match operation {
                CheckedIntegerOperation::AddExact => {
                    if left_datum.is_some() && right_datum.is_some() {
                        return None;
                    }
                    Some((
                        left_datum.or(right_datum),
                        left_value.checked_add(right_value)?,
                    ))
                }
                // Subtracting a datum gives it coefficient minus one, which
                // no difference-bound term carries.
                CheckedIntegerOperation::SubtractExact => {
                    if right_datum.is_some() {
                        return None;
                    }
                    Some((left_datum, left_value.checked_sub(right_value)?))
                }
                // A multiplication of two constants is one constant; any
                // other coefficient leaves the fragment.
                _ => {
                    if left_datum.is_some() || right_datum.is_some() {
                        return None;
                    }
                    Some((None, left_value.checked_mul(right_value)?))
                }
            };
        }
        let datum = self.postcondition_relation_datum(expanded)?;
        // A written integer literal is a constant of the side rather than
        // its datum; a const generic and a generic numeric identity keep
        // their own datum identity, symbolic or not.
        if let RelationDatum::Literal {
            value: CheckedValue::Integer { ty, bits },
            origin: PostconditionConstantOrigin::Literal,
        } = &datum
        {
            return Some((None, integer_value(*ty, *bits)));
        }
        Some((Some(datum), 0))
    }

    fn postcondition_relation_datum(
        &self,
        expanded: &ExpandedClauseExpression,
    ) -> Option<RelationDatum> {
        match expanded {
            ExpandedClauseExpression::Datum(ExpandedClauseDatum::Result { ordinal, ty }) => {
                Some(RelationDatum::Result {
                    ordinal: *ordinal,
                    ty: *ty,
                })
            }
            ExpandedClauseExpression::Datum(ExpandedClauseDatum::Parameter {
                ordinal,
                projections,
                ty,
            }) => Some(RelationDatum::Parameter {
                ordinal: *ordinal,
                projections: projections.clone(),
                ty: *ty,
            }),
            ExpandedClauseExpression::Datum(ExpandedClauseDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) => Some(RelationDatum::NamedConst {
                declaration: *declaration,
                projections: projections.clone(),
                ty: *ty,
            }),
            ExpandedClauseExpression::Datum(ExpandedClauseDatum::Literal { value, origin }) => {
                Some(RelationDatum::Literal {
                    value: value.clone(),
                    origin: origin.clone(),
                })
            }
            // [CALL-4] the clause operands of [FN-9] are terms, so a measure
            // over an admitted formal place is an operand with no per-family
            // admission, and so is one over an admitted result place.
            ExpandedClauseExpression::Operation {
                row:
                    GoalOperation::ArrayMeasure { measure, .. }
                    | GoalOperation::BufferMeasure { measure, .. }
                    | GoalOperation::SliceMeasure { measure, .. }
                    | GoalOperation::ContainerMeasure { measure, .. },
                arguments,
                ..
            } => {
                let [ExpandedClauseExpression::Datum(datum)] = arguments.as_slice() else {
                    return None;
                };
                let (root, projections, ty) = match datum {
                    ExpandedClauseDatum::Parameter {
                        ordinal,
                        projections,
                        ty,
                    } => (
                        PostconditionPlaceRoot::Parameter { ordinal: *ordinal },
                        projections.clone(),
                        *ty,
                    ),
                    ExpandedClauseDatum::Result { ordinal, ty } => (
                        PostconditionPlaceRoot::Result { ordinal: *ordinal },
                        Vec::new(),
                        *ty,
                    ),
                    ExpandedClauseDatum::NamedConst { .. }
                    | ExpandedClauseDatum::Literal { .. } => return None,
                };
                Some(RelationDatum::Measure(
                    *measure,
                    PostconditionPlace {
                        root,
                        projections,
                        ty,
                    },
                ))
            }
            ExpandedClauseExpression::Operation { .. }
            | ExpandedClauseExpression::InvalidSelectorUse { .. } => None,
        }
    }

    fn invalid_postcondition_relation<T>(&self, expression: NodeId) -> Result<T, CheckStop> {
        self.issue_node(
            SemanticRule::Fn9,
            expression,
            SemanticIssueKind::InvalidPostconditionRelation,
        )
    }

    pub(super) fn build_checked_postcondition(
        &self,
        function: &FunctionSignature,
        parameters: &[CheckedParameter],
        selector: CheckedPostconditionSelector,
        relation: RelationTemplate,
        body: &[CheckedStatement],
    ) -> Result<CheckedPostcondition, CheckStop> {
        if !function.substitution.is_concrete() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        self.build_checked_postcondition_inner(
            function, parameters, selector, relation, body, false,
        )
    }

    /// Builds the source-schema FN-9 handoff only when the written relation
    /// and selected returns are already in the ordinary concrete integer
    /// fragment. GenericInt remains an exact symbolic goal datum, never an L0
    /// term, so a postcondition over `T` is intentionally concrete-instance
    /// only.
    pub(super) fn build_checked_schema_postcondition(
        &self,
        function: &FunctionSignature,
        parameters: &[CheckedParameter],
        selector: CheckedPostconditionSelector,
        relation: RelationTemplate,
        body: &[CheckedStatement],
    ) -> Result<Option<CheckedPostcondition>, CheckStop> {
        if function.substitution.is_concrete() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if !matches!(selector.result_type, CheckedType::Integer(_))
            || relation
                .operands
                .iter()
                .any(|operand| !matches!(operand.ty(), CheckedType::Integer(_)))
        {
            return Ok(None);
        }
        let checked = self.build_checked_postcondition_inner(
            function, parameters, selector, relation, body, true,
        )?;
        let fragment_returns = checked.selected_returns.iter().all(|selected| {
            selected.values.iter().flatten().all(|value| match value {
                PostconditionReturnDatum::Place(place) => {
                    matches!(place.ty, CheckedType::Integer(_))
                }
                PostconditionReturnDatum::Literal { value, .. } => {
                    matches!(value.ty(), CheckedType::Integer(_))
                }
                PostconditionReturnDatum::Measure(..) => true,
            })
        });
        Ok(fragment_returns.then_some(checked))
    }

    fn build_checked_postcondition_inner(
        &self,
        function: &FunctionSignature,
        parameters: &[CheckedParameter],
        selector: CheckedPostconditionSelector,
        relation: RelationTemplate,
        body: &[CheckedStatement],
        symbolic_schema: bool,
    ) -> Result<CheckedPostcondition, CheckStop> {
        let mut type_substitutions = Vec::new();
        let mut const_substitutions = Vec::new();
        for (declaration, argument) in function.substitution.entries() {
            match argument {
                GenericArgument::Type(ty) if ty.is_concrete() => {
                    type_substitutions.push((*declaration, *ty));
                }
                GenericArgument::Const(super::super::model::CheckedConst::Value(value)) => {
                    const_substitutions.push((*declaration, *value));
                }
                _ if symbolic_schema => {}
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            }
        }

        if parameters.len() != function.parameters.len() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let mut binding_info = HashMap::new();
        for parameter in parameters {
            binding_info.insert(
                parameter.binding,
                PostconditionBindingInfo {
                    ty: parameter.ty,
                    implicit_deref: parameter.mode != CheckedMode::Own,
                },
            );
        }
        self.collect_postcondition_binding_info(body, &mut binding_info);

        // [FN-9, CALL-4] a return is selected for this clause only when every
        // result ordinal the relation names evaluates there to one admitted
        // datum; an ordinal the clause does not name imposes nothing.
        let named = relation
            .operands
            .iter()
            .filter_map(|operand| match &operand.datum {
                RelationDatum::Result { ordinal, .. } => Some(*ordinal),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut selected_returns = Vec::new();
        self.collect_postcondition_returns(
            function,
            &selector,
            &named,
            body,
            &binding_info,
            &mut selected_returns,
        )?;
        selected_returns.sort_by(|left, right| {
            left.statement
                .components()
                .cmp(right.statement.components())
        });
        Ok(CheckedPostcondition {
            selector,
            type_substitutions,
            const_substitutions,
            relation,
            selected_returns,
        })
    }

    fn collect_postcondition_binding_info(
        &self,
        statements: &[CheckedStatement],
        bindings: &mut HashMap<BindingId, PostconditionBindingInfo>,
    ) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let implicit_deref = matches!(
                        value,
                        CheckedExpression::BorrowAddressed { .. }
                            | CheckedExpression::BorrowBuffer { .. }
                            | CheckedExpression::BorrowBox { .. }
                            | CheckedExpression::BorrowSystemResource { .. }
                            | CheckedExpression::ReborrowAddressed { .. }
                    ) || match value {
                        CheckedExpression::Binding { binding, .. } => bindings
                            .get(binding)
                            .is_some_and(|source| source.implicit_deref),
                        _ => false,
                    };
                    bindings.insert(
                        *binding,
                        PostconditionBindingInfo {
                            ty: value.ty(),
                            implicit_deref,
                        },
                    );
                }
                // [CALL-4] a destructuring `let` binds one fresh own value per
                // declared result ordinal, and each is a place a return can
                // name exactly as an ordinary `let` binding is.
                CheckedStatement::DestructuringLet {
                    bindings: binders, ..
                } => {
                    for (binding, ty) in binders {
                        bindings.insert(
                            *binding,
                            PostconditionBindingInfo {
                                ty: *ty,
                                implicit_deref: false,
                            },
                        );
                    }
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    bindings.insert(
                        *binding,
                        PostconditionBindingInfo {
                            ty: *ok_type,
                            implicit_deref: false,
                        },
                    );
                }
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        for binder in &arm.binders {
                            bindings.insert(
                                binder.binding,
                                PostconditionBindingInfo {
                                    ty: binder.ty,
                                    implicit_deref: binder.mode != CheckedMode::Own,
                                },
                            );
                        }
                        self.collect_postcondition_binding_info(&arm.body, bindings);
                    }
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    bindings.insert(
                        *binding,
                        PostconditionBindingInfo {
                            ty: *result_type,
                            implicit_deref: false,
                        },
                    );
                    for arm in arms {
                        for binder in &arm.binders {
                            bindings.insert(
                                binder.binding,
                                PostconditionBindingInfo {
                                    ty: binder.ty,
                                    implicit_deref: binder.mode != CheckedMode::Own,
                                },
                            );
                        }
                        self.collect_postcondition_binding_info(&arm.body, bindings);
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.collect_postcondition_binding_info(body, bindings);
                }
                CheckedStatement::CountedRange {
                    binder,
                    lower,
                    body,
                    ..
                } => {
                    bindings.insert(
                        *binder,
                        PostconditionBindingInfo {
                            ty: lower.ty(),
                            implicit_deref: false,
                        },
                    );
                    self.collect_postcondition_binding_info(body, bindings);
                }
                _ => {}
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_postcondition_returns(
        &self,
        function: &FunctionSignature,
        selector: &CheckedPostconditionSelector,
        named: &[u32],
        statements: &[CheckedStatement],
        binding_info: &HashMap<BindingId, PostconditionBindingInfo>,
        selected: &mut Vec<SelectedPostconditionReturn>,
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Return {
                    node_path, value, ..
                } => {
                    // [GRAM-4, CALL-4] a `return e1, ..., en;` is checked as
                    // one result-list value, so the ordinals are its fields;
                    // a single-result return is the whole value.
                    let ordinals: Vec<&CheckedExpression> = match (function.result_list, value) {
                        (
                            Some(list),
                            CheckedExpression::ConstructStruct {
                                nominal, fields, ..
                            },
                        ) if *nominal == list => fields.iter().collect(),
                        (Some(_), _) => {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        (None, value) => vec![value],
                    };
                    let mut values = Vec::with_capacity(ordinals.len());
                    let mut selected_at_all = true;
                    for (ordinal, produced) in ordinals.into_iter().enumerate() {
                        let Ok(ordinal) = u32::try_from(ordinal) else {
                            return Err(SemanticCompilerFailure::CounterOverflow.into());
                        };
                        let routed = selector.variant.is_some() && ordinal == selector.ordinal;
                        let produced = if routed {
                            match self.postcondition_route_payload(
                                function, ordinal, produced, node_path,
                            )? {
                                Some(payload) => payload,
                                // A direct `Err` return is unselected for this
                                // routed clause [FN-9].
                                None => {
                                    selected_at_all = false;
                                    values.push(None);
                                    continue;
                                }
                            }
                        } else {
                            produced
                        };
                        let datum =
                            self.postcondition_return_datum(produced, node_path, binding_info)?;
                        if datum.is_none() && named.contains(&ordinal) {
                            return self.invalid_postcondition_return(node_path);
                        }
                        values.push(datum);
                    }
                    if selected_at_all {
                        selected.push(SelectedPostconditionReturn {
                            statement: node_path.clone(),
                            values,
                        });
                    }
                }
                CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } => {
                    for arm in arms {
                        self.collect_postcondition_returns(
                            function,
                            selector,
                            named,
                            &arm.body,
                            binding_info,
                            selected,
                        )?;
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => self.collect_postcondition_returns(
                    function,
                    selector,
                    named,
                    body,
                    binding_info,
                    selected,
                )?,
                _ => {}
            }
        }
        Ok(())
    }

    /// The `Ok` payload one routed ordinal produces at a return, `None` for a
    /// direct `Err`, and the [FN-9] rejection for every other Result shape.
    fn postcondition_route_payload<'value>(
        &self,
        function: &FunctionSignature,
        ordinal: u32,
        value: &'value CheckedExpression,
        node_path: &crate::NodePath,
    ) -> Result<Option<&'value CheckedExpression>, CheckStop> {
        let declared = function
            .results
            .get(ordinal as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let CheckedType::Nominal(result_nominal) = declared.ty else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if self
            .prelude_types
            .get(result_nominal.0 as usize)
            .and_then(|entry| *entry)
            .is_none_or(|ty| !matches!(ty, super::PreludeType::Result(_, _)))
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        match value {
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
                ..
            } if *nominal == result_nominal && *variant == 0 && fields.len() == 1 => {
                Ok(fields.first())
            }
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
                ..
            } if *nominal == result_nominal && *variant == 1 && fields.len() == 1 => Ok(None),
            _ => self.invalid_postcondition_return(node_path),
        }
    }

    fn postcondition_return_datum(
        &self,
        value: &CheckedExpression,
        statement: &crate::NodePath,
        binding_info: &HashMap<BindingId, PostconditionBindingInfo>,
    ) -> Result<Option<PostconditionReturnDatum>, CheckStop> {
        if let Some(place) = self.postcondition_return_place(value, statement, binding_info)? {
            return Ok(Some(PostconditionReturnDatum::Place(place)));
        }
        match value {
            CheckedExpression::Constant(value) => {
                let origin = self.postcondition_return_constant_origin(statement)?;
                Ok(Some(PostconditionReturnDatum::Literal {
                    value: value.clone(),
                    origin,
                }))
            }
            CheckedExpression::ArrayMeasure {
                measure,
                root,
                length,
            } => {
                let place = match root {
                    CheckedArrayRoot::Binding { binding, fields } => {
                        self.postcondition_binding_place(*binding, fields, statement, binding_info)?
                    }
                    CheckedArrayRoot::Constant(constant) => {
                        let constant = self.constant(*constant)?;
                        Some(PostconditionReturnPlace {
                            root: PostconditionReturnPlaceRoot::NamedConst(constant.declaration),
                            projections: Vec::new(),
                            ty: constant.ty,
                            source: statement.clone(),
                        })
                    }
                };
                let Some(
                    place @ PostconditionReturnPlace {
                        ty:
                            CheckedType::Array {
                                length: actual_length,
                                ..
                            },
                        ..
                    },
                ) = place
                else {
                    return Ok(None);
                };
                if actual_length != *length {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(Some(PostconditionReturnDatum::Measure(*measure, place)))
            }
            CheckedExpression::BufferMeasure { measure, root } => {
                let Some(
                    place @ PostconditionReturnPlace {
                        ty: CheckedType::Buffer { element },
                        ..
                    },
                ) = self.postcondition_binding_place(
                    root.binding,
                    &root.fields,
                    statement,
                    binding_info,
                )?
                else {
                    return Ok(None);
                };
                if element != root.element {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(Some(PostconditionReturnDatum::Measure(*measure, place)))
            }
            CheckedExpression::SliceMeasure { measure, root } => {
                let Some(
                    place @ PostconditionReturnPlace {
                        ty: CheckedType::Slice { element, .. },
                        ..
                    },
                ) = self.postcondition_binding_place(root.binding, &[], statement, binding_info)?
                else {
                    return Ok(None);
                };
                if element != root.element {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(Some(PostconditionReturnDatum::Measure(*measure, place)))
            }
            _ => Ok(None),
        }
    }

    fn postcondition_return_place(
        &self,
        value: &CheckedExpression,
        statement: &crate::NodePath,
        binding_info: &HashMap<BindingId, PostconditionBindingInfo>,
    ) -> Result<Option<PostconditionReturnPlace>, CheckStop> {
        let place = match value {
            CheckedExpression::Binding {
                carrier,
                binding,
                ty,
                ..
            } => {
                let Some(mut place) =
                    self.postcondition_binding_place(*binding, &[], statement, binding_info)?
                else {
                    return Ok(None);
                };
                if place.ty != *ty {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                place.source = carrier.clone();
                place
            }
            CheckedExpression::NamedConstant {
                declaration, value, ..
            } => PostconditionReturnPlace {
                root: PostconditionReturnPlaceRoot::NamedConst(*declaration),
                projections: Vec::new(),
                ty: value.ty(),
                source: statement.clone(),
            },
            CheckedExpression::Project {
                carrier,
                binding,
                fields,
                ty,
                consume_root: false,
                ..
            } => {
                let Some(mut place) =
                    self.postcondition_binding_place(*binding, fields, statement, binding_info)?
                else {
                    return Ok(None);
                };
                if place.ty != *ty {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                place.source = carrier.clone();
                place
            }
            CheckedExpression::DerefAddressed {
                carrier,
                binding,
                ty,
            } => {
                let Some(info) = binding_info.get(binding) else {
                    return Ok(None);
                };
                if info.ty != *ty {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                PostconditionReturnPlace {
                    root: PostconditionReturnPlaceRoot::Binding(*binding),
                    projections: vec![GoalProjection::Deref],
                    ty: *ty,
                    source: carrier.clone(),
                }
            }
            CheckedExpression::BoxDeref {
                carrier,
                value,
                referent,
                ..
            } => {
                let Some(mut place) =
                    self.postcondition_return_place(value, statement, binding_info)?
                else {
                    return Ok(None);
                };
                place.projections.push(GoalProjection::Deref);
                place.ty = *referent;
                place.source = carrier.clone();
                place
            }
            CheckedExpression::ProjectValue {
                carrier,
                value,
                field,
                ty,
                ..
            } => {
                let Some(mut place) =
                    self.postcondition_return_place(value, statement, binding_info)?
                else {
                    return Ok(None);
                };
                place.projections.push(GoalProjection::Field(*field));
                place.ty = *ty;
                place.source = carrier.clone();
                place
            }
            _ => return Ok(None),
        };
        Ok(Some(place))
    }

    fn postcondition_binding_place(
        &self,
        binding: BindingId,
        fields: &[u32],
        source: &crate::NodePath,
        binding_info: &HashMap<BindingId, PostconditionBindingInfo>,
    ) -> Result<Option<PostconditionReturnPlace>, CheckStop> {
        let Some(info) = binding_info.get(&binding).copied() else {
            return Ok(None);
        };
        let Some(ty) = self.postcondition_projected_type(info.ty, fields)? else {
            return Ok(None);
        };
        let projections = info
            .implicit_deref
            .then_some(GoalProjection::Deref)
            .into_iter()
            .chain(fields.iter().copied().map(GoalProjection::Field))
            .collect();
        Ok(Some(PostconditionReturnPlace {
            root: PostconditionReturnPlaceRoot::Binding(binding),
            projections,
            ty,
            source: source.clone(),
        }))
    }

    fn postcondition_projected_type(
        &self,
        mut ty: CheckedType,
        fields: &[u32],
    ) -> Result<Option<CheckedType>, CheckStop> {
        for field in fields {
            let CheckedType::Nominal(nominal) = ty else {
                return Ok(None);
            };
            let CheckedNominalKind::Struct { fields } = &self.nominal(nominal)?.kind else {
                return Ok(None);
            };
            let Some(selected) = fields.get(*field as usize) else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            ty = selected.ty;
        }
        Ok(Some(ty))
    }

    fn postcondition_return_constant_origin(
        &self,
        statement: &crate::NodePath,
    ) -> Result<PostconditionConstantOrigin, CheckStop> {
        let statement = self
            .tree
            .node_with_path(statement)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let expression = self
            .tree
            .first_child_with(statement, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let atoms = self.tree.descendants_with(expression, Production::Atom)?;
        if let [atom] = atoms.as_slice()
            && let Some(literal) = self
                .tree
                .direct_token_with(*atom, crate::TerminalPredicate::Literal)?
        {
            let bytes = self.tree.token_bytes(literal)?;
            if matches!(bytes, b"0_T" | b"1_T") {
                let usage = self.use_at(*atom, LexicalUseRole::GenericNumericSuffix)?;
                let ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::GenericType,
                } = usage.target()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                return Ok(PostconditionConstantOrigin::GenericNumericIdentity {
                    type_parameter: declaration,
                    one: bytes == b"1_T",
                });
            }
        }
        Ok(PostconditionConstantOrigin::Literal)
    }

    fn invalid_postcondition_return<T>(&self, path: &crate::NodePath) -> Result<T, CheckStop> {
        Err(self.invalid_postcondition_return_stop(path))
    }

    fn invalid_postcondition_return_stop(&self, path: &crate::NodePath) -> CheckStop {
        let node = self.tree.node_with_path(path);
        match node {
            Some(node) => self.issue_value(
                SemanticRule::Fn9,
                node,
                SemanticIssueKind::InvalidPostconditionReturn,
            ),
            None => SemanticCompilerFailure::InvalidResolution.into(),
        }
    }

    pub(super) fn postcondition_selector_use_inside(
        &self,
        node: NodeId,
    ) -> Result<bool, CheckStop> {
        let Some(context) = self.active_postcondition.get() else {
            return Ok(false);
        };
        let record = self
            .resolved
            .postconditions()
            .get(context.record)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let owner = self.tree.path(node)?.components();
        Ok(record.selector_uses.iter().any(|usage| {
            let path = usage.origin.node().components();
            path.len() > owner.len() && path.starts_with(owner)
        }))
    }

    /// The result ordinal and datum type a bare selector atom names, when the
    /// atom is exactly one written result datum [FN-9, CALL-4].
    pub(super) fn postcondition_selector_is_bare_atom(
        &self,
        atom: NodeId,
    ) -> Result<Option<(u32, CheckedType)>, CheckStop> {
        let Some(context) = self.active_postcondition.get() else {
            return Ok(None);
        };
        let record = self
            .resolved
            .postconditions()
            .get(context.record)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(place) = self.tree.first_child_with(atom, Production::Place)? else {
            return Ok(None);
        };
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase_path = self.tree.path(pbase)?;
        if !self.tree.children(pbase)?.is_empty()
            || !self
                .tree
                .children_with(place, Production::Psuffix)?
                .is_empty()
        {
            return Ok(None);
        }
        Ok(record
            .selector_uses
            .iter()
            .find(|usage| usage.origin.node() == pbase_path)
            .and_then(|usage| self.active_result_datum(&usage.spelling)))
    }

    /// The declared result ordinal one clause's result datum is anchored to
    /// [CALL-4].
    ///
    /// An unrouted clause is anchored to the first ordinal admitted as a
    /// result datum; every ordinal remains a datum of the clause, and the
    /// anchor only fixes which one the selector's admission judgment reads.
    /// A routed clause is anchored to the ordinal its written binder names,
    /// or, when the binder is omitted, to the one ordinal whose enum type can
    /// carry the route. Two such ordinals leave the route ambiguous, and the
    /// declaration is a hard error citing CALL-4 at the clause.
    fn postcondition_route_ordinal(
        &self,
        record: &PostconditionResolutionRecord,
        signature: &FunctionSignature,
        symbolic: bool,
    ) -> Result<u32, CheckStop> {
        let routed = record.class == PostconditionSelectorClass::Variant;
        if !routed {
            let anchor = record
                .result_binders
                .iter()
                .zip(&signature.results)
                .position(|(_, declared)| self.postcondition_fragment_type(declared.ty, symbolic));
            return u32::try_from(anchor.unwrap_or(0))
                .map_err(|_| SemanticCompilerFailure::CounterOverflow.into());
        }
        if let Some(spelling) = &record.route_ordinal {
            let Some(named) = record
                .result_binders
                .iter()
                .position(|binder| &binder.spelling == spelling)
            else {
                return self
                    .issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
            };
            return u32::try_from(named)
                .map_err(|_| SemanticCompilerFailure::CounterOverflow.into());
        }
        let carriers = signature
            .results
            .iter()
            .enumerate()
            .filter(|(_, declared)| self.postcondition_route_carrier(declared.ty, symbolic))
            .map(|(ordinal, _)| ordinal)
            .collect::<Vec<_>>();
        match carriers.as_slice() {
            [only] => {
                u32::try_from(*only).map_err(|_| SemanticCompilerFailure::CounterOverflow.into())
            }
            // Zero carriers is the ordinary route-admission refusal below,
            // which reports the offending result type; anchor it at ordinal
            // zero and let `validate_postcondition_selector` speak.
            [] => Ok(0),
            _ => self.issue_selector(record, SemanticIssueKind::AmbiguousResultRoute),
        }
    }

    /// Whether one declared result type can carry a route in this version:
    /// exactly `own Result<T, E>` with T a fragment integer [FN-9, CALL-4].
    fn postcondition_route_carrier(&self, ty: CheckedType, symbolic: bool) -> bool {
        let CheckedType::Nominal(nominal) = ty else {
            return false;
        };
        matches!(
            self.prelude_types.get(nominal.0 as usize).and_then(|entry| *entry),
            Some(super::PreludeType::Result(value, _))
                if self.postcondition_fragment_type(value, symbolic)
        )
    }

    fn admit_postcondition_selector(
        &self,
        record: &PostconditionResolutionRecord,
        signature: &FunctionSignature,
        symbolic: bool,
    ) -> Result<CheckedPostconditionSelector, CheckStop> {
        if signature
            .results
            .iter()
            .any(|entry| entry.mode != CheckedMode::Own)
        {
            return self.issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
        }

        // [CALL-4] a route applies to exactly one declared result ordinal:
        // the one its written binder names, or — when the binder is omitted —
        // the one ordinal whose type carries the route's variant. Two ordinals
        // that could carry it leave the route ambiguous and the declaration is
        // refused here.
        let ordinal = self.postcondition_route_ordinal(record, signature, symbolic)?;
        let declared = signature
            .results
            .get(ordinal as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;

        let (admission, result_type) = match declared.ty {
            ty if self.postcondition_fragment_type(ty, symbolic) => {
                (SelectorAdmissionType::Fragment, ty)
            }
            CheckedType::Nominal(nominal) => match self
                .prelude_types
                .get(nominal.0 as usize)
                .and_then(|entry| *entry)
            {
                Some(super::PreludeType::Result(value, _))
                    if self.postcondition_fragment_type(value, symbolic) =>
                {
                    (SelectorAdmissionType::ResultFragment, value)
                }
                _ => (SelectorAdmissionType::Invalid, declared.ty),
            },
            CheckedType::Generic(_) if symbolic => (SelectorAdmissionType::Symbolic, declared.ty),
            // [CALL-4] a result of measured type is admitted, and a measure
            // over that result place is the operand it supplies.
            ty if super::expressions::flat_storage::measured_kind_of(ty).is_some() => {
                (SelectorAdmissionType::Measured, ty)
            }
            _ => (SelectorAdmissionType::Invalid, declared.ty),
        };
        self.validate_postcondition_selector(record, admission, ordinal)?;

        let (candidate, variant, field) = match record.class {
            PostconditionSelectorClass::Plain => (
                record
                    .result_binders
                    .get(ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                None,
                None,
            ),
            PostconditionSelectorClass::Variant => {
                let ResolvedTarget::Prelude(variant) = record
                    .variant_target
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let field = record
                    .fields
                    .first()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (
                    &field.candidate,
                    Some(variant),
                    Some(PostconditionFieldIdentity {
                        declaration: PreludeDeclarationId::new(12),
                        origin: field.origin.clone(),
                    }),
                )
            }
        };

        Ok(CheckedPostconditionSelector {
            function: signature.id,
            block: record.block.clone(),
            selector: record.selector.clone(),
            candidate: candidate.origin.clone(),
            ordinal,
            variant,
            field,
            result_type,
        })
    }

    fn validate_postcondition_selector(
        &self,
        record: &PostconditionResolutionRecord,
        admission: SelectorAdmissionType,
        ordinal: u32,
    ) -> Result<(), CheckStop> {
        let candidate = match record.class {
            PostconditionSelectorClass::Plain => {
                if !matches!(
                    admission,
                    SelectorAdmissionType::Fragment
                        | SelectorAdmissionType::Symbolic
                        | SelectorAdmissionType::Measured
                ) {
                    return self
                        .issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
                }
                record
                    .result_binders
                    .get(ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
            }
            PostconditionSelectorClass::Variant => {
                if !matches!(
                    admission,
                    SelectorAdmissionType::ResultFragment | SelectorAdmissionType::Symbolic
                ) {
                    return self
                        .issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
                }
                if record.variant_target
                    != Some(ResolvedTarget::Prelude(PreludeDeclarationId::new(11)))
                {
                    return self
                        .issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
                }
                let Some(field) = record.fields.first() else {
                    return self.issue_selector(
                        record,
                        SemanticIssueKind::InvalidPostconditionFields {
                            required_fields: vec!["value".to_owned()],
                        },
                    );
                };
                if field.spelling != "value" {
                    return self.issue_origin_node(
                        &field.origin,
                        SemanticIssueKind::InvalidPostconditionFields {
                            required_fields: vec!["value".to_owned()],
                        },
                    );
                }
                if let Some(extra) = record.fields.get(1) {
                    return self.issue_origin_node(
                        &extra.origin,
                        SemanticIssueKind::InvalidPostconditionFields {
                            required_fields: vec!["value".to_owned()],
                        },
                    );
                }
                &field.candidate
            }
        };
        self.check_postcondition_candidate(candidate)
    }

    fn postcondition_fragment_type(&self, ty: CheckedType, symbolic: bool) -> bool {
        matches!(ty, CheckedType::Integer(_))
            || symbolic && matches!(ty, CheckedType::Generic(_) | CheckedType::GenericInt(_))
    }

    fn check_postcondition_candidate(
        &self,
        candidate: &PostconditionCandidateRecord,
    ) -> Result<(), CheckStop> {
        if candidate
            .paired_field
            .as_ref()
            .is_some_and(|field| field == &candidate.spelling)
            || !candidate.live_conflicts.is_empty()
        {
            return self.issue_origin(
                SemanticRule::Fn9,
                &candidate.origin,
                SemanticIssueKind::PostconditionCandidateNotFresh {
                    spelling: candidate.spelling.clone(),
                    conflicts: candidate.live_conflicts.clone(),
                },
            );
        }
        if let Some(local) = &candidate.later_local_collision {
            return self.issue_origin(
                SemanticRule::Fn9,
                local,
                SemanticIssueKind::PostconditionLocalShadowsResult {
                    spelling: candidate.spelling.clone(),
                    selector: candidate.origin.clone(),
                },
            );
        }
        Ok(())
    }

    fn issue_selector<T>(
        &self,
        record: &PostconditionResolutionRecord,
        kind: SemanticIssueKind,
    ) -> Result<T, CheckStop> {
        // [CALL-4] owns the result ordinal and the route's ambiguity; every
        // other selector rejection is [FN-9]'s admission.
        let rule = if matches!(kind, SemanticIssueKind::AmbiguousResultRoute) {
            SemanticRule::Call4
        } else {
            SemanticRule::Fn9
        };
        let node = self
            .tree
            .node_with_path(&record.selector)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.issue_node(rule, node, kind)
    }

    fn issue_origin<T>(
        &self,
        rule: SemanticRule,
        origin: &SourceOrigin,
        kind: SemanticIssueKind,
    ) -> Result<T, CheckStop> {
        Err(CheckStop::source_issue(SemanticIssue {
            rule,
            location: SemanticLocation::SourceNode(origin.node().clone(), origin.coordinate()),
            kind,
        }))
    }

    fn issue_origin_node<T>(
        &self,
        origin: &SourceOrigin,
        kind: SemanticIssueKind,
    ) -> Result<T, CheckStop> {
        let node = self
            .tree
            .node_with_path(origin.node())
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.issue_node(SemanticRule::Fn9, node, kind)
    }
}

/// The mathematical value of one checked integer constant, whose `bits` hold
/// the type-width two's-complement pattern.
const fn integer_value(ty: IntegerType, bits: u64) -> i128 {
    let value = bits as i128;
    if ty.signed() {
        let width = ty.width() as u32;
        let sign_bit = 1_u64 << (width - 1);
        if bits & sign_bit != 0 {
            return value - (1_i128 << width);
        }
    }
    value
}

/// The type-width bit pattern of one mathematical value, or `None` when the
/// value does not fit that type. A clause side reduces over the mathematical
/// integers [MSR-5], so a constant side outside its own operand type is not
/// a relation datum and the clause is refused rather than wrapped.
const fn representable_bits(ty: CheckedType, value: i128) -> Option<u64> {
    let CheckedType::Integer(ty) = ty else {
        return None;
    };
    let width = ty.width() as u32;
    let (low, high) = if ty.signed() {
        (-(1_i128 << (width - 1)), (1_i128 << (width - 1)) - 1)
    } else {
        (0, (1_i128 << width) - 1)
    };
    if value < low || value > high {
        return None;
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some((value & ((1_i128 << width) - 1)) as u64)
}
