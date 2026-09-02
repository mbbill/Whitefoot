use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedLoopId, CheckedLoopInvariant, CheckedMode, CheckedStatement, CheckedType, IntegerType,
};
use super::super::borrows::RequiredReferent;
use super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};
use super::proofs::AffineProofOwner;
use super::{ControlCounters, ControlScope, StatementResult};

#[derive(Clone)]
pub(in crate::semantic::check) struct LoopContext {
    pub(super) id: CheckedLoopId,
    label_declaration: Option<DeclarationId>,
    preserved: HashSet<DeclarationId>,
}

pub(in crate::semantic::check) struct BreakState {
    target: CheckedLoopId,
    bindings: HashMap<DeclarationId, LocalBinding>,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    fn form_loop_invariants(
        &self,
        nodes: Vec<NodeId>,
        loop_id: CheckedLoopId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
    ) -> Result<Vec<CheckedLoopInvariant>, CheckStop> {
        let mut names = HashSet::new();
        let mut invariants = Vec::with_capacity(nodes.len());
        for node in nodes {
            invariants.push(self.check_loop_invariant(
                node,
                loop_id,
                bindings,
                allowed_values,
                &mut names,
            )?);
        }
        Ok(invariants)
    }

    pub(super) fn check_counted_range(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let binding_node = self
            .tree
            .first_child_with(node, Production::ForBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let endpoints = self.tree.children_with(binding_node, Production::Atom)?;
        let [lower_node, upper_node] = endpoints.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };

        // [FN-1] endpoint evaluation is source ordered and happens before the
        // counted binder exists. Each check can therefore observe only the
        // preceding endpoint's ordinary ownership/effect consequences.
        let lower =
            self.check_counted_endpoint(function, *lower_node, bindings, scope.loops.len())?;
        let upper =
            self.check_counted_endpoint(function, *upper_node, bindings, scope.loops.len())?;
        let effects = lower.effects.union(upper.effects);

        let label = self
            .optional_declaration_at(node, DeclarationRole::LoopLabel)?
            .map(crate::DeclarationRecord::id);
        let binder_declaration =
            self.declaration_at(binding_node, DeclarationRole::CountedBinder)?;
        let binder_declaration_id = binder_declaration.id();
        let id = Self::allocate_loop(counters.next_loop)?;
        let binder = Self::allocate_binding(counters.next_binding)?;
        counters
            .binding_names
            .push(binder_declaration.spelling().to_owned());

        // The structural false-header edge always carries this exact state to
        // the continuation. The binder and body locals exist only in the
        // separate header/body state below.
        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let preserved = base_keys.iter().copied().collect::<HashSet<_>>();
        let mut body_bindings = base_bindings.clone();
        if body_bindings
            .insert(
                binder_declaration_id,
                LocalBinding {
                    binding: binder,
                    declaration: binder_declaration_id,
                    mode: CheckedMode::Own,
                    ty: CheckedType::Integer(IntegerType::U64),
                    state_origins: None,
                    live: true,
                    loop_depth: scope.loops.len() + 1,
                    compiler_updated: true,
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
        let header_bindings = body_bindings.clone();
        let header_keys = body_bindings.keys().copied().collect::<Vec<_>>();
        let header_preserved = header_keys.iter().copied().collect::<HashSet<_>>();
        let mut nested_loops = scope.loops.to_vec();
        nested_loops.push(LoopContext {
            id,
            label_declaration: label,
            preserved: preserved.clone(),
        });

        let invariant_nodes = self.tree.children_with(node, Production::HeaderInvariant)?;
        let executable_statements = self.tree.children_with(node, Production::Stmt)?;
        let allowed_invariant_values = header_keys.iter().copied().collect::<HashSet<_>>();
        let invariants = self.form_loop_invariants(
            invariant_nodes,
            id,
            &body_bindings,
            &allowed_invariant_values,
        )?;
        let checked = self.check_block(
            function,
            &executable_statements,
            &mut body_bindings,
            counters,
            ControlScope {
                loops: &nested_loops,
                give_context: scope.give_context,
            },
        )?;
        if checked.can_continue
            && header_keys
                .iter()
                .any(|key| body_bindings.get(key) != header_bindings.get(key))
        {
            return self.unsupported(UnsupportedSemanticFeature::OwnershipJoin, node);
        }

        let backedge_drops = if checked.can_continue {
            self.live_affine_drops(&body_bindings, &header_preserved)?
        } else {
            Vec::new()
        };

        // Unlike an ordinary loop, exhaustion is an executable continuation
        // input even when no local break is written. Local breaks join it;
        // breaks targeting an enclosing loop keep escaping normally.
        let mut continuation_states = vec![base_bindings];
        let mut escaping_break_states = Vec::new();
        for state in checked.break_states {
            if state.target == id {
                continuation_states.push(state.bindings);
            } else {
                escaping_break_states.push(state);
            }
        }
        self.join_states(&base_keys, &continuation_states, node, bindings)?;

        Ok(StatementResult {
            statement: CheckedStatement::CountedRange {
                id,
                node_path: self.tree.path(node)?.clone(),
                binder,
                lower: lower.expression,
                upper: upper.expression,
                invariants,
                body: checked.statements,
                backedge_drops,
            },
            can_continue: true,
            effects: effects.union(checked.effects),
            all_paths_deliver: false,
            direct_give: false,
            give_states: checked.give_states,
            break_states: escaping_break_states,
        })
    }

    fn check_loop_invariant(
        &self,
        node: NodeId,
        loop_id: CheckedLoopId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        names: &mut HashSet<String>,
    ) -> Result<CheckedLoopInvariant, CheckStop> {
        let declaration = self.declaration_at(node, DeclarationRole::Invariant)?;
        let identifiers = self.tree.direct_identifiers(node)?;
        let [name_token, relation_token] = identifiers.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let name = std::str::from_utf8(self.tree.token_bytes(*name_token)?)
            .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?
            .to_owned();
        if !names.insert(name.clone()) {
            return self.invalid_loop_invariant(
                node,
                "a loop contains two invariants with the same name",
                "give every invariant in this loop a distinct name",
            );
        }
        let relation = self.check_ordered_affine_relation(
            node,
            *relation_token,
            bindings,
            allowed_values,
            AffineProofOwner::LoopInvariant,
        )?;

        Ok(CheckedLoopInvariant {
            loop_id,
            declaration: declaration.id(),
            name,
            relation,
        })
    }

    pub(super) fn invalid_loop_invariant<ResultValue>(
        &self,
        node: NodeId,
        reason: &'static str,
        mechanical_fix: &'static str,
    ) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Inv1,
            node,
            SemanticIssueKind::InvalidLoopInvariant {
                reason,
                mechanical_fix,
            },
        )
    }

    fn check_counted_endpoint(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let required = CheckedType::Integer(IntegerType::U64);
        if self.direct_counted_endpoint_holder_requires_deref(node, bindings, required)? {
            return self.issue_node(
                SemanticRule::Type7,
                node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        // TYPE-7 precedes the endpoint's TYPE-5 exact-value judgment. Use the
        // consuming-position atom path so a box/arena or borrow holder reaches
        // that exclusive judgment instead of stopping first at OWN-1's bare
        // affine spelling rule.
        let endpoint = self.check_consuming_atom(function, node, bindings, loop_depth)?;
        if self.reads_implicitly_through_holder(
            endpoint.reference_value,
            endpoint.expression.ty(),
            RequiredReferent::Exact(required),
        )? {
            return self.issue_node(
                SemanticRule::Type7,
                node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        if endpoint.mode != CheckedMode::Own || endpoint.expression.ty() != required {
            return self.issue_node(
                SemanticRule::Type5,
                node,
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(required)?),
                    self.checked_value_name(endpoint.mode, endpoint.expression.ty())?,
                ),
            );
        }
        if !self.counted_endpoint_is_term_or_constant(node)? {
            return self.issue_node(
                SemanticRule::Ent2,
                node,
                SemanticIssueKind::InvalidCountedEndpoint {
                    mechanical_fix: "bind the computed u64 value with one preceding ordinary let and use that term as the endpoint",
                },
            );
        }
        Ok(endpoint)
    }

    /// After TYPE-5, the only atom shapes still capable of producing `own
    /// u64` are a literal or a place. ENT-2 admits the literal and exactly a
    /// tracked place with field/deref wrappers but no subscript at any depth.
    fn counted_endpoint_is_term_or_constant(&self, node: NodeId) -> Result<bool, CheckStop> {
        let Some(place) = self.tree.first_child_with(node, Production::Place)? else {
            // TYPE-5 has already excluded a borrow expression, so the
            // remaining non-place atom is an integer literal constant.
            return Ok(true);
        };
        self.counted_endpoint_place_is_term(place)
    }

    fn counted_endpoint_place_is_term(&self, place: NodeId) -> Result<bool, CheckStop> {
        for suffix in self.tree.children_with(place, Production::Psuffix)? {
            if self.subscript_offset(suffix)?.is_some() {
                return Ok(false);
            }
        }
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let Some(inner) = self.tree.first_child_with(pbase, Production::Place)? else {
            return Ok(true);
        };
        self.counted_endpoint_place_is_term(inner)
    }

    /// TYPE-7 is definitionally earlier than both OWN-1's holder spelling and
    /// OWN-11's outer-affine move check. Inspect a live direct holder before
    /// those generic place checks so an endpoint that plainly needs `deref`
    /// keeps the rule's exclusive attribution even inside another loop.
    fn direct_counted_endpoint_holder_requires_deref(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        required: CheckedType,
    ) -> Result<bool, CheckStop> {
        let Some(place) = self.tree.first_child_with(node, Production::Place)? else {
            return Ok(false);
        };
        let Some(pbase) = self.tree.first_child_with(place, Production::Pbase)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        if !self.tree.children(pbase)?.is_empty()
            || !self
                .tree
                .children_with(place, Production::Psuffix)?
                .is_empty()
        {
            return Ok(false);
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(false);
        };
        let Some(local) = bindings.get(&declaration) else {
            return Ok(false);
        };
        if !local.live {
            return Ok(false);
        }
        self.reads_implicitly_through_holder(
            local.mode != CheckedMode::Own,
            local.ty,
            RequiredReferent::Exact(required),
        )
    }

    pub(super) fn check_loop(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let declaration = self
            .optional_declaration_at(node, DeclarationRole::LoopLabel)?
            .map(crate::DeclarationRecord::id);
        let id = Self::allocate_loop(counters.next_loop)?;
        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let preserved = base_keys.iter().copied().collect::<HashSet<_>>();
        let mut nested_loops = scope.loops.to_vec();
        nested_loops.push(LoopContext {
            id,
            label_declaration: declaration,
            preserved: preserved.clone(),
        });

        let mut body_bindings = base_bindings.clone();
        let invariant_nodes = self.tree.children_with(node, Production::HeaderInvariant)?;
        let executable_statements = self.tree.children_with(node, Production::Stmt)?;
        let allowed_invariant_values = base_keys.iter().copied().collect::<HashSet<_>>();
        let invariants = self.form_loop_invariants(
            invariant_nodes,
            id,
            &body_bindings,
            &allowed_invariant_values,
        )?;
        let checked = self.check_block(
            function,
            &executable_statements,
            &mut body_bindings,
            counters,
            ControlScope {
                loops: &nested_loops,
                give_context: scope.give_context,
            },
        )?;
        if checked.can_continue
            && base_keys
                .iter()
                .any(|key| body_bindings.get(key) != base_bindings.get(key))
        {
            return self.unsupported(UnsupportedSemanticFeature::OwnershipJoin, node);
        }

        let mut own_break_states = Vec::new();
        let mut escaping_break_states = Vec::new();
        for state in checked.break_states {
            if state.target == id {
                own_break_states.push(state.bindings);
            } else {
                escaping_break_states.push(state);
            }
        }
        if own_break_states.is_empty() {
            return self.unsupported(UnsupportedSemanticFeature::StructuredControlFlow, node);
        }
        self.join_states(&base_keys, &own_break_states, node, bindings)?;
        let backedge_drops = if checked.can_continue {
            self.live_affine_drops(&body_bindings, &preserved)?
        } else {
            Vec::new()
        };

        Ok(StatementResult {
            statement: CheckedStatement::Loop {
                id,
                invariants,
                body: checked.statements,
                backedge_drops,
            },
            // FN-1 conservatively gives every loop a normal successor; the
            // executable path reaches it only through a checked break edge.
            can_continue: true,
            effects: checked.effects,
            all_paths_deliver: false,
            direct_give: false,
            give_states: checked.give_states,
            break_states: escaping_break_states,
        })
    }

    pub(super) fn check_break(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let uses = self.uses_at_ordered(node, LexicalUseRole::BreakLabel)?;
        let target = match uses.as_slice() {
            [] => scope.loops.last().ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Fn1,
                    node,
                    SemanticIssueKind::BreakOutsideLoop {
                        mechanical_fix: "move `break;` inside a loop or remove it",
                    },
                )
            })?,
            [usage] => {
                let ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Label,
                } = usage.target()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                scope
                    .loops
                    .iter()
                    .rev()
                    .find(|context| context.label_declaration == Some(declaration))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let all_paths_deliver = scope
            .give_context
            .is_some_and(|context| context.enclosing_loops.contains(&target.id));
        Ok(StatementResult {
            statement: CheckedStatement::Break {
                target: target.id,
                drops: self.live_affine_drops(bindings, &target.preserved)?,
            },
            can_continue: false,
            effects: EffectSet::NONE,
            all_paths_deliver,
            direct_give: false,
            give_states: Vec::new(),
            break_states: vec![BreakState {
                target: target.id,
                bindings: bindings.clone(),
            }],
        })
    }

    fn allocate_loop(next_loop: &mut u32) -> Result<CheckedLoopId, CheckStop> {
        let id = CheckedLoopId(*next_loop);
        *next_loop = next_loop
            .checked_add(1)
            .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        Ok(id)
    }
}

impl BreakState {
    pub(super) fn retain_bindings(&mut self, preserved: &HashSet<DeclarationId>) {
        self.bindings
            .retain(|declaration, _| preserved.contains(declaration));
    }

    pub(super) fn end_slice_region(&mut self, region: DeclarationId) {
        for local in self.bindings.values_mut() {
            local.end_slice_region(region);
        }
    }
}
