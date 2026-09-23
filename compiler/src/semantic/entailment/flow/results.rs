//! Value-associated conditional Result evidence. Each context assumes only
//! its own value is Ok; no operation combines guards of different outcomes.

use super::*;

#[derive(Clone, Debug)]
pub(super) struct ResultEvidence {
    pub(super) payload: TermId,
    pub(super) facts: FactState,
    pub(super) definitely_err: bool,
}

impl Analyzer<'_, '_> {
    fn result_payload_type(&self, ty: CheckedType) -> Option<IntegerType> {
        let CheckedType::Nominal(id) = ty else {
            return None;
        };
        let CheckedNominalKind::Enum { variants } = &self.context.nominals.get(id.0 as usize)?.kind
        else {
            return None;
        };
        let ok = variants.iter().find(|variant| {
            variant.constructor == CheckedConstructor::Prelude(crate::BuiltinPreludeId::OK)
        })?;
        let [field] = ok.fields.as_slice() else {
            return None;
        };
        fragment_type(field.ty)
    }

    fn result_context(&mut self, ty: CheckedType) -> Option<ResultEvidence> {
        let ty = self.result_payload_type(ty)?;
        Some(ResultEvidence {
            payload: self.terms.intern(TermKind::ResultPayload(ty)),
            facts: FactState::new(),
            definitely_err: false,
        })
    }

    /// Merge ordinary facts into exactly one conditional context. A guard's
    /// contradiction remains local until that value's success is selected.
    pub(super) fn refresh_result(&mut self, result: &mut ResultEvidence, ordinary: &FactState) {
        let mut ordinary = ordinary.clone();
        materialize_closure_before_kill(
            &mut ordinary,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if ordinary.all_derivable {
            result
                .facts
                .promote_to_contradiction(ordinary.contradiction);
        }
        for (relation, parent) in ordinary.l0_candidates() {
            if relation
                .terms()
                .iter()
                .any(|term| matches!(self.terms.kind(*term), TermKind::ResultPayload(_)))
            {
                continue;
            }
            result
                .facts
                .establish_from_proof(&relation, parent, &self.derivations);
        }
    }

    fn substitute_result_facts(
        &mut self,
        statement: &crate::NodePath,
        source: &FactState,
        from: TermId,
        to: TermId,
    ) -> FactState {
        let mut closed = source.clone();
        materialize_closure_before_kill(
            &mut closed,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.all_derivable {
            return FactState::contradictory(closed.contradiction.expect("closed contradiction"));
        }
        let mut target = FactState::new();
        for (relation, parent) in closed.l0_candidates() {
            let substituted = Self::replace_relation_term(&relation, from, to);
            let parent = if relation.terms().contains(&from) {
                self.derivations.intern(DerivationNode::ResultTransport {
                    statement: statement.clone(),
                    from,
                    to,
                    relation: Box::new(substituted.clone()),
                    parent,
                })
            } else {
                parent
            };
            target.establish_from_proof(&substituted, parent, &self.derivations);
        }
        target
    }

    /// Read the value before its own consuming transfer. Calls supply their
    /// evidence separately, after the successful call's ordered effects.
    pub(super) fn capture_result(
        &mut self,
        statement: &crate::NodePath,
        expression: &CheckedExpression,
        state: &ProofFlowState,
    ) -> Option<ResultEvidence> {
        if matches!(expression, CheckedExpression::Binding { binding, .. } if self.is_holder(*binding))
            || matches!(
                expression,
                CheckedExpression::BorrowAddressed { .. }
                    | CheckedExpression::BorrowRangeIndex { .. }
            )
        {
            return None;
        }
        let mut result = self.result_context(expression.ty())?;
        match expression {
            CheckedExpression::Binding { binding, .. } if !self.is_holder(*binding) => {
                if let Some(held) = state.results.get(binding) {
                    result = held.clone();
                }
                self.refresh_result(&mut result, &state.facts);
            }
            CheckedExpression::ConstructEnum {
                variant, fields, ..
            } if *variant == 0 => {
                let [value] = fields.as_slice() else {
                    return Some(result);
                };
                if let Some(term) = self.read_operand(value) {
                    result.facts =
                        self.substitute_result_facts(statement, &state.facts, term, result.payload);
                    // A literal or a previously unmentioned binding needs
                    // its exact value equality as well as its consequences.
                    let event = self.proof_event(FlowEventKind::S5, Some(statement));
                    result.facts.establish(
                        &Relation::Equal {
                            left: result.payload,
                            right: term,
                            difference: 0,
                        },
                        &mut self.derivations,
                        event,
                    );
                }
            }
            CheckedExpression::ConstructEnum { variant, .. } if *variant == 1 => {
                let parent = self.derivations.intern(DerivationNode::ResultErr {
                    statement: statement.clone(),
                });
                result.facts = FactState::contradictory(parent);
                result.definitely_err = true;
            }
            _ => {}
        }
        Some(result)
    }

    pub(super) fn finish_result(
        &mut self,
        expression: &CheckedExpression,
        judgment: &ExpressionJudgment,
        result: &mut Option<ResultEvidence>,
        state: &ProofFlowState,
    ) {
        if !judgment.reached {
            *result = None;
            return;
        }
        let Some(result) = result else { return };
        let mut kills = Vec::new();
        self.collect_expression_kills(expression, &mut kills);
        self.apply_kills_one(&state.separations, &mut result.facts, &kills);
        self.refresh_result(result, &state.facts);
        let (
            Some(prepared),
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                goal_arguments,
                ..
            },
        ) = (&judgment.prepared_call, expression)
        else {
            return;
        };
        for available in &prepared.postconditions {
            if available.variant != Some(crate::BuiltinPreludeId::OK)
                || available.field != Some(crate::BuiltinPreludeId::OK_VALUE)
            {
                continue;
            }
            let Some(instantiated) = self.instantiate_call_postcondition_relation(
                *function,
                call,
                &available.relation,
                arguments,
                goal_arguments,
                &[Some(result.payload)],
                &[],
            ) else {
                continue;
            };
            if !self.s12_substitutions_survive(
                &state.separations,
                &instantiated.substitutions,
                &prepared.kills,
                true,
            ) {
                continue;
            }
            if let Some(parent) = self.retain_postcondition_call(&instantiated, available, prepared)
            {
                let occurrence = self.s12_roots;
                self.s12_roots = self
                    .s12_roots
                    .checked_add(1)
                    .expect("S12 root identity fits u32");
                self.derivations.add_root(
                    DerivationRootKind::PostconditionConditional { occurrence },
                    parent,
                );
                result.facts.establish_from_proof(
                    &instantiated.relation,
                    parent,
                    &self.derivations,
                );
            }
        }
    }

    pub(super) fn select_result(
        &mut self,
        statement: &crate::NodePath,
        result: &ResultEvidence,
        binding: BindingId,
        ty: CheckedType,
        state: &mut ProofFlowState,
    ) {
        let Some(receiver) = self.postcondition_place_term(PlaceRoot::Binding(binding), &[], ty)
        else {
            return;
        };
        let mut result = result.clone();
        self.refresh_result(&mut result, &state.facts);
        let selected =
            self.substitute_result_facts(statement, &result.facts, result.payload, receiver);
        if selected.all_derivable {
            state.facts.promote_to_contradiction(selected.contradiction);
        }
        for (relation, parent) in selected.l0_candidates() {
            if relation
                .terms()
                .iter()
                .any(|term| matches!(self.terms.kind(*term), TermKind::ResultPayload(_)))
            {
                continue;
            }
            state
                .facts
                .establish_from_proof(&relation, parent, &self.derivations);
        }
    }

    pub(super) fn kill_result_evidence(
        &mut self,
        state: &mut ProofFlowState,
        events: &[KillEvent],
    ) {
        if events.is_empty() {
            return;
        }
        let mut held = std::mem::take(&mut state.results);
        held.retain(|binding, result| {
            let place = self.bound_place(*binding);
            if events.iter().any(|event| match event {
                KillEvent::Consume {
                    binding: source, ..
                }
                | KillEvent::EntryImageHolderConsume {
                    binding: source, ..
                } => binding == source,
                KillEvent::Write { place: written, .. }
                | KillEvent::EntryImageHolderWrite { place: written, .. } => {
                    self.resolved_places_overlap(&state.separations, &place, written)
                }
            }) {
                return false;
            }
            self.refresh_result(result, &state.facts);
            self.apply_kills_one(&state.separations, &mut result.facts, events);
            true
        });
        state.results = held;
    }

    pub(super) fn exit_result_scopes(&mut self, state: &mut ProofFlowState, depth: usize) {
        let exited = self
            .scopes
            .iter()
            .skip(depth)
            .flatten()
            .copied()
            .collect::<HashSet<_>>();
        let mut held = std::mem::take(&mut state.results);
        held.retain(|binding, result| {
            if exited.contains(binding) {
                return false;
            }
            self.refresh_result(result, &state.facts);
            materialize_closure_before_kill(
                &mut result.facts,
                &self.terms,
                &self.goals,
                &mut self.derivations,
            );
            self.exit_scopes_to_one(&mut result.facts, depth);
            true
        });
        state.results = held;
    }

    pub(super) fn join_result_evidence(
        &mut self,
        states: &[ProofFlowState],
    ) -> BTreeMap<BindingId, ResultEvidence> {
        let mut bindings = BTreeMap::new();
        for state in states {
            for (&binding, result) in &state.results {
                bindings.entry(binding).or_insert(result.payload);
            }
        }
        let mut results = BTreeMap::new();
        for (binding, payload) in bindings {
            let mut images = Vec::new();
            for state in states {
                let mut result = state
                    .results
                    .get(&binding)
                    .cloned()
                    .unwrap_or(ResultEvidence {
                        payload,
                        facts: FactState::new(),
                        definitely_err: false,
                    });
                self.refresh_result(&mut result, &state.facts);
                images.push(result.facts);
            }
            let event = self.derivations.event(FlowEventKind::Join, None);
            let facts = join_at(
                &images,
                &self.terms,
                &self.goals,
                &mut self.derivations,
                event,
            );
            let definitely_err = states.iter().all(|state| {
                state
                    .results
                    .get(&binding)
                    .is_some_and(|result| result.definitely_err)
            });
            results.insert(
                binding,
                ResultEvidence {
                    payload,
                    facts,
                    definitely_err,
                },
            );
        }
        results
    }
}
