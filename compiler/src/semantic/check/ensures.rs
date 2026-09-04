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
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedMode, CheckedNominalKind,
    CheckedParameter, CheckedStatement, CheckedType, CheckedValue, FunctionId,
};
use super::super::postcondition::{
    CheckedPostcondition, CheckedPostconditionSelector, NormalizedRelation,
    PostconditionConstantOrigin, PostconditionFieldIdentity, PostconditionPlace,
    PostconditionPlaceRoot, PostconditionReturnDatum, PostconditionReturnPlace,
    PostconditionReturnPlaceRoot, RelationDatum, RelationTemplate, SelectedPostconditionReturn,
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
        let result = check();
        self.active_postcondition.set(previous);
        result
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
        let selector_inside = record.selector_uses.iter().any(|usage| {
            let path = usage.origin.node().components();
            path.len() > atom_path.len() && path.starts_with(atom_path)
        });
        if !selector_inside {
            return Ok(None);
        }
        Ok(Some(match context.result_type {
            CheckedType::Integer(ty) => CheckedValue::Integer { ty, bits: 0 },
            CheckedType::GenericInt(_) => CheckedValue::NumericIdentity {
                ty: context.result_type,
                one: false,
            },
            _ => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
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
        self.with_postcondition_context(record, selector.result_type, || {
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
            let RelationDatum::Length(place) = operand else {
                continue;
            };
            let PostconditionPlaceRoot::Parameter { ordinal } = place.root;
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
        let Some(left) = self.postcondition_relation_datum(left) else {
            return self.invalid_postcondition_relation(final_expression);
        };
        let Some(right) = self.postcondition_relation_datum(right) else {
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

    fn postcondition_relation_datum(
        &self,
        expanded: &ExpandedClauseExpression,
    ) -> Option<RelationDatum> {
        match expanded {
            ExpandedClauseExpression::Datum(ExpandedClauseDatum::Result { ty }) => {
                Some(RelationDatum::Result { ty: *ty })
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
            ExpandedClauseExpression::Operation {
                row:
                    GoalOperation::ArrayLength { .. }
                    | GoalOperation::BufferLength { .. }
                    | GoalOperation::SliceLength { .. },
                arguments,
                ..
            } => {
                let [
                    ExpandedClauseExpression::Datum(ExpandedClauseDatum::Parameter {
                        ordinal,
                        projections,
                        ty,
                    }),
                ] = arguments.as_slice()
                else {
                    return None;
                };
                Some(RelationDatum::Length(PostconditionPlace {
                    root: PostconditionPlaceRoot::Parameter { ordinal: *ordinal },
                    projections: projections.clone(),
                    ty: *ty,
                }))
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
        let fragment_returns =
            checked
                .selected_returns
                .iter()
                .all(|selected| match &selected.value {
                    PostconditionReturnDatum::Place(place) => {
                        matches!(place.ty, CheckedType::Integer(_))
                    }
                    PostconditionReturnDatum::Literal { value, .. } => {
                        matches!(value.ty(), CheckedType::Integer(_))
                    }
                    PostconditionReturnDatum::Length(_) => true,
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

        let mut selected_returns = Vec::new();
        self.collect_postcondition_returns(
            function,
            &selector,
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

    fn collect_postcondition_returns(
        &self,
        function: &FunctionSignature,
        selector: &CheckedPostconditionSelector,
        statements: &[CheckedStatement],
        binding_info: &HashMap<BindingId, PostconditionBindingInfo>,
        selected: &mut Vec<SelectedPostconditionReturn>,
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Return {
                    node_path, value, ..
                } => {
                    let selected_value = if selector.variant.is_none() {
                        Some(value)
                    } else {
                        let CheckedType::Nominal(result_nominal) = function.result else {
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
                            } if *nominal == result_nominal
                                && *variant == 0
                                && fields.len() == 1 =>
                            {
                                fields.first()
                            }
                            CheckedExpression::ConstructEnum {
                                nominal,
                                variant,
                                fields,
                                ..
                            } if *nominal == result_nominal
                                && *variant == 1
                                && fields.len() == 1 =>
                            {
                                None
                            }
                            _ => return self.invalid_postcondition_return(node_path),
                        }
                    };
                    if let Some(value) = selected_value {
                        let datum = self
                            .postcondition_return_datum(value, node_path, binding_info)?
                            .ok_or_else(|| self.invalid_postcondition_return_stop(node_path))?;
                        selected.push(SelectedPostconditionReturn {
                            statement: node_path.clone(),
                            value: datum,
                        });
                    }
                }
                CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } => {
                    for arm in arms {
                        self.collect_postcondition_returns(
                            function,
                            selector,
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
                    body,
                    binding_info,
                    selected,
                )?,
                _ => {}
            }
        }
        Ok(())
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
            CheckedExpression::ArrayLength { root, length } => {
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
                Ok(Some(PostconditionReturnDatum::Length(place)))
            }
            CheckedExpression::BufferLength { root } => {
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
                Ok(Some(PostconditionReturnDatum::Length(place)))
            }
            CheckedExpression::SliceLength { root } => {
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
                Ok(Some(PostconditionReturnDatum::Length(place)))
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

    pub(super) fn postcondition_selector_is_bare_atom(
        &self,
        atom: NodeId,
    ) -> Result<bool, CheckStop> {
        let Some(context) = self.active_postcondition.get() else {
            return Ok(false);
        };
        let record = self
            .resolved
            .postconditions()
            .get(context.record)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(place) = self.tree.first_child_with(atom, Production::Place)? else {
            return Ok(false);
        };
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase_path = self.tree.path(pbase)?;
        Ok(self.tree.children(pbase)?.is_empty()
            && self
                .tree
                .children_with(place, Production::Psuffix)?
                .is_empty()
            && record
                .selector_uses
                .iter()
                .any(|usage| usage.origin.node() == pbase_path))
    }

    fn admit_postcondition_selector(
        &self,
        record: &PostconditionResolutionRecord,
        signature: &FunctionSignature,
        symbolic: bool,
    ) -> Result<CheckedPostconditionSelector, CheckStop> {
        if signature.result_mode != CheckedMode::Own {
            return self.issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
        }

        let (admission, result_type) = match signature.result {
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
                _ => (SelectorAdmissionType::Invalid, signature.result),
            },
            CheckedType::Generic(_) if symbolic => {
                (SelectorAdmissionType::Symbolic, signature.result)
            }
            _ => (SelectorAdmissionType::Invalid, signature.result),
        };
        self.validate_postcondition_selector(record, admission)?;

        let (candidate, variant, field) = match record.class {
            PostconditionSelectorClass::Plain => (
                record
                    .plain_candidate
                    .as_ref()
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
            variant,
            field,
            result_type,
        })
    }

    fn validate_postcondition_selector(
        &self,
        record: &PostconditionResolutionRecord,
        admission: SelectorAdmissionType,
    ) -> Result<(), CheckStop> {
        let candidate = match record.class {
            PostconditionSelectorClass::Plain => {
                if !matches!(
                    admission,
                    SelectorAdmissionType::Fragment | SelectorAdmissionType::Symbolic
                ) {
                    return self
                        .issue_selector(record, SemanticIssueKind::InvalidPostconditionSelector);
                }
                record
                    .plain_candidate
                    .as_ref()
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
        let node = self
            .tree
            .node_with_path(&record.selector)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.issue_node(SemanticRule::Fn9, node, kind)
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
