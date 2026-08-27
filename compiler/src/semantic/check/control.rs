use std::cell::Cell;
use std::collections::{HashMap, HashSet};

mod loops;
mod matches;
mod results;

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::model::{
    BindingId, CheckedBooleanOperation, CheckedDrop, CheckedExpression, CheckedLoopId, CheckedMode,
    CheckedSetTarget, CheckedStatement, CheckedType, ClaimJustification, ClaimSite,
    ValueInitializerKind,
};
use super::borrows::ReborrowPosition;
use super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use loops::{BreakState, LoopContext};

pub(super) struct BlockResult {
    pub(super) statements: Vec<CheckedStatement>,
    pub(super) can_continue: bool,
    pub(super) effects: EffectSet,
    all_paths_deliver: bool,
    give_states: Vec<HashMap<DeclarationId, LocalBinding>>,
    break_states: Vec<BreakState>,
}

pub(super) struct StatementResult {
    pub(super) statement: CheckedStatement,
    pub(super) can_continue: bool,
    pub(super) effects: EffectSet,
    all_paths_deliver: bool,
    direct_give: bool,
    give_states: Vec<HashMap<DeclarationId, LocalBinding>>,
    break_states: Vec<BreakState>,
}

pub(super) struct GiveContext {
    /// [GIVE-1] the binding's mode and type are derived from the delivery
    /// set, not written: the first delivering `give` of this initializer
    /// produces them and every later one must agree exactly. `None` until
    /// that first `give` is checked; still `None` afterwards exactly when
    /// the delivery set is empty.
    delivered: Cell<Option<(CheckedMode, CheckedType)>>,
    preserved: HashSet<DeclarationId>,
    enclosing_loops: HashSet<CheckedLoopId>,
}

impl GiveContext {
    pub(super) fn empty(preserved: &HashSet<DeclarationId>, scope: ControlScope<'_>) -> Self {
        Self {
            delivered: Cell::new(None),
            preserved: preserved.clone(),
            enclosing_loops: scope.loops.iter().map(|context| context.id).collect(),
        }
    }

    pub(super) fn delivered(&self) -> Option<(CheckedMode, CheckedType)> {
        self.delivered.get()
    }
}

pub(super) struct ControlCounters<'state> {
    pub(super) next_binding: &'state mut u32,
    pub(super) next_loop: &'state mut u32,
    /// Source spelling of every allocated binding, indexed by [`BindingId`];
    /// kept only to render the owner in a release-attributed EFF-2
    /// diagnostic. Every allocation site pushes exactly one name.
    pub(super) binding_names: &'state mut Vec<String>,
    /// Claim names already written in this function, for CLM-1's
    /// per-function uniqueness judgment.
    pub(super) claim_names: &'state mut Vec<String>,
}

#[derive(Clone, Copy)]
pub(super) struct ControlScope<'state> {
    pub(super) loops: &'state [LoopContext],
    pub(super) give_context: Option<&'state GiveContext>,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_block(
        &self,
        function: &FunctionSignature,
        statement_wrappers: &[NodeId],
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<BlockResult, CheckStop> {
        let mut statements = Vec::with_capacity(statement_wrappers.len());
        let mut can_continue = true;
        let mut effects = EffectSet::NONE;
        let mut all_paths_deliver = false;
        let mut direct_give = false;
        let mut give_states = Vec::new();
        let mut break_states = Vec::new();
        for wrapper in statement_wrappers {
            let statement = self.tree.only_child(*wrapper)?;
            if !can_continue {
                return self.issue_node(
                    if direct_give {
                        SemanticRule::Give1
                    } else {
                        SemanticRule::Fn1
                    },
                    statement,
                    if direct_give {
                        SemanticIssueKind::InvalidGive
                    } else {
                        SemanticIssueKind::UnreachableStatement
                    },
                );
            }
            let checked = self.check_statement(function, statement, bindings, counters, scope)?;
            can_continue = checked.can_continue;
            effects = effects.union(checked.effects);
            all_paths_deliver = checked.all_paths_deliver;
            direct_give = checked.direct_give;
            give_states.extend(checked.give_states);
            break_states.extend(checked.break_states);
            statements.push(checked.statement);
        }
        if can_continue {
            all_paths_deliver = false;
        }
        Ok(BlockResult {
            statements,
            can_continue,
            effects,
            all_paths_deliver,
            give_states,
            break_states,
        })
    }

    pub(super) fn check_statement(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        match self.tree.production(node)? {
            Production::LetStmt | Production::ContractDefine => {
                self.check_let(function, node, bindings, counters, scope)
            }
            Production::ExprStmt => {
                let call = self
                    .tree
                    .first_child_with(node, Production::Call)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let value = self.check_call(function, call, bindings, scope.loops.len())?;
                // A discarded borrow-mode result is a reference, never the
                // owner of its referent: no drop or release may run for it
                // [OWN-2, STOR-3]. Only an own-mode affine result is dropped.
                let statement = if value.mode != CheckedMode::Own
                    || self.is_copy_type(value.expression.ty())?
                {
                    CheckedStatement::Evaluate(value.expression)
                } else {
                    let release = self.release_of_type(value.expression.ty())?;
                    let state_origins = self.state_origins_of_value(&value, bindings)?;
                    CheckedStatement::DropExpression {
                        value: value.expression,
                        state_origins,
                        release,
                    }
                };
                Ok(Self::continuing_statement(statement, value.effects))
            }
            Production::ReturnStmt => {
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                self.check_return_implicit_read(function, expression_node, bindings)?;
                // A borrow_expr as the complete return expression is the sole
                // non-argument position that admits a written reborrow form:
                // the returned reborrow [OWN-14]. Control leaves the function
                // before the creating statement ends, so the suspended holder
                // never resumes and no point observes both usable [OWN-5].
                let value = if let Some(borrow) =
                    self.complete_borrow_expression(expression_node)?
                {
                    self.check_borrow(
                        borrow,
                        function,
                        bindings,
                        scope.loops.len(),
                        ReborrowPosition::ReturnExpression,
                    )?
                } else {
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?
                };
                if value.expression.ty() != function.result {
                    return Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Fn1,
                        location: SemanticLocation::SourceNode(
                            self.tree.path(node)?.clone(),
                            self.tree.coordinate(expression_node)?,
                        ),
                        kind: SemanticIssueKind::ReturnMismatch,
                    }));
                }
                // [FN-1] owns the result mode; [OWN-4] owns the region
                // relation between the returned borrow and the written
                // `rtype` region, so the two are judged separately.
                let modes_agree = matches!(
                    (value.mode, function.result_mode),
                    (CheckedMode::Own, CheckedMode::Own)
                        | (CheckedMode::Shared(_), CheckedMode::Shared(_))
                        | (CheckedMode::Unique(_), CheckedMode::Unique(_))
                );
                if !modes_agree {
                    if value.mode != CheckedMode::Own && function.result_mode == CheckedMode::Own {
                        return self.issue_node(
                            SemanticRule::Type7,
                            expression_node,
                            SemanticIssueKind::MissingDereference {
                                mechanical_fix: "write `deref(holder)`",
                            },
                        );
                    }
                    return self.issue_node(
                        SemanticRule::Fn1,
                        node,
                        SemanticIssueKind::ReturnMismatch,
                    );
                }
                self.borrow_for_destination(function.result_mode, &value, expression_node)?;
                if matches!(function.result, CheckedType::Slice { .. }) {
                    let origins = value
                        .slice
                        .as_ref()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if !origins
                        .origins
                        .iter()
                        .all(|origin| function.slice_return_ceiling.contains(origin))
                    {
                        return Err(CheckStop::source_issue(SemanticIssue {
                            rule: SemanticRule::Fn1,
                            location: SemanticLocation::SourceNode(
                                self.tree.path(node)?.clone(),
                                self.tree.coordinate(expression_node)?,
                            ),
                            kind: SemanticIssueKind::InvalidSliceReturnOrigin {
                                mechanical_fix: "accept an exact direct input slice in the result region or keep the newly formed view in its caller; do not return a view of raw callee storage",
                            },
                        }));
                    }
                }
                Ok(StatementResult {
                    statement: CheckedStatement::Return {
                        node_path: self.tree.path(node)?.clone(),
                        value: value.expression,
                        drops: self.live_affine_drops(bindings, &HashSet::new())?,
                    },
                    can_continue: false,
                    effects: value.effects,
                    all_paths_deliver: true,
                    direct_give: false,
                    give_states: Vec::new(),
                    break_states: Vec::new(),
                })
            }
            // [GRAM-6] the Bool conditional checks into the same two-armed
            // Bool match the `match` spelling produced, so everything below
            // the checker sees one statement kind for both.
            Production::IfStmt => {
                let matched = self.check_if(function, node, bindings, counters, scope, false)?;
                Ok(StatementResult {
                    statement: CheckedStatement::Match {
                        scrutinee: matched.scrutinee,
                        enum_type: matched.enum_type,
                        arms: matched.arms,
                        continues: matched.can_continue,
                    },
                    can_continue: matched.can_continue,
                    effects: matched.effects,
                    all_paths_deliver: matched.all_paths_deliver,
                    direct_give: false,
                    give_states: matched.give_states,
                    break_states: matched.break_states,
                })
            }
            Production::MatchStmt => {
                let matched = self.check_match(function, node, bindings, counters, scope, false)?;
                Ok(StatementResult {
                    statement: CheckedStatement::Match {
                        scrutinee: matched.scrutinee,
                        enum_type: matched.enum_type,
                        arms: matched.arms,
                        continues: matched.can_continue,
                    },
                    can_continue: matched.can_continue,
                    effects: matched.effects,
                    all_paths_deliver: matched.all_paths_deliver,
                    direct_give: false,
                    give_states: matched.give_states,
                    break_states: matched.break_states,
                })
            }
            Production::GiveStmt => {
                let Some(context) = scope.give_context else {
                    return self.issue_node(
                        SemanticRule::Give1,
                        node,
                        SemanticIssueKind::InvalidGive,
                    );
                };
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let value =
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?;
                // [GIVE-1] derivation is agreement over the closed delivery
                // set: the first delivering `give` produces the binding's
                // exact mode and type, and every later one must match them.
                let delivered = (value.mode, value.expression.ty());
                match context.delivered.get() {
                    None => context.delivered.set(Some(delivered)),
                    Some(earlier) if earlier == delivered => {}
                    Some(_) => {
                        return self.issue_node(
                            SemanticRule::Give1,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                }
                Ok(StatementResult {
                    statement: CheckedStatement::Give {
                        node_path: self.tree.path(node)?.clone(),
                        value: value.expression,
                        drops: self.live_affine_drops(bindings, &context.preserved)?,
                    },
                    can_continue: false,
                    effects: value.effects,
                    all_paths_deliver: true,
                    direct_give: true,
                    give_states: vec![bindings.clone()],
                    break_states: Vec::new(),
                })
            }
            Production::SetStmt => {
                let target_node = self
                    .tree
                    .first_child_with(node, Production::Place)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;

                // SET-1 fixes this order: form and check the target first, then
                // evaluate the RHS, then re-establish target writability.
                let (declaration, target, target_effects) =
                    self.check_set_target(function, target_node, bindings, scope.loops.len())?;
                let value =
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?;
                if value.expression.ty() != target.ty() {
                    return self.issue_node(
                        SemanticRule::Type5,
                        expression_node,
                        SemanticIssueKind::TypeMismatch,
                    );
                }
                if !bindings
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .live
                {
                    return self.issue_node(
                        SemanticRule::Own1,
                        target_node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                Ok(Self::continuing_statement(
                    CheckedStatement::Set {
                        node_path: self.tree.path(node)?.clone(),
                        target,
                        value: value.expression,
                    },
                    value.effects.union(target_effects),
                ))
            }
            Production::ClaimStmt => {
                let name_token = self
                    .tree
                    .direct_token_with(node, TerminalPredicate::Identifier)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let name = String::from_utf8(self.tree.token_bytes(name_token)?.to_vec())
                    .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?;
                // CLM-1: within one `fn_decl` every claim name is unique;
                // the later `claim_stmt` node carries the rejection.
                if counters.claim_names.contains(&name) {
                    return self.issue_node(
                        SemanticRule::Clm1,
                        node,
                        SemanticIssueKind::DuplicateClaimName { name },
                    );
                }
                counters.claim_names.push(name.clone());
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let condition =
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?;
                // The condition judgment is exactly [OP-5]'s, cited as CLM-1
                // at the selected expression node.
                if condition.expression.ty() != CheckedType::Bool
                    || condition.mode != CheckedMode::Own
                {
                    return Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Clm1,
                        location: SemanticLocation::SourceNode(
                            self.tree.path(node)?.clone(),
                            self.tree.coordinate(expression_node)?,
                        ),
                        kind: SemanticIssueKind::InvalidPredicateCondition,
                    }));
                }
                self.check_claim_proof_predicate(
                    expression_node,
                    &condition.expression,
                    &condition.effects,
                )?;
                let justification = self.check_claim_justification(node)?;
                let predicate = self.tree.source_spelling(expression_node)?;
                Ok(Self::continuing_statement(
                    CheckedStatement::Claim {
                        site: ClaimSite {
                            rule_id: "CLM-1",
                            message: name.clone(),
                            function: function.name.clone(),
                            node_path: self.tree.path(node)?.clone(),
                        },
                        name,
                        predicate,
                        justification,
                        condition: condition.expression,
                    },
                    condition.effects.union(EffectSet::TRAPS),
                ))
            }
            Production::LoopStmt => self.check_loop(function, node, bindings, counters, scope),
            Production::ForStmt => {
                self.check_counted_range(function, node, bindings, counters, scope)
            }
            Production::BreakStmt => self.check_break(node, bindings, scope),
            Production::RegionStmt => self.check_region(function, node, bindings, counters, scope),
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    fn check_let(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        // [TYPE-5] a `let` binder's mode and type are derived, never written:
        // exactly what its selected right-hand side produces. Each arm below
        // therefore checks that right-hand side first and reads the binding's
        // mode and type off the result.
        let declaration = self.declaration_at(node, DeclarationRole::Let)?;
        let declaration_id = declaration.id();
        let binding = Self::allocate_binding(counters.next_binding)?;
        counters
            .binding_names
            .push(declaration.spelling().to_owned());

        // [GIVE-1] a value initializer is a `match` or an `if`. Both derive the
        // binder from their delivery set and share every judgment below, so
        // only the checker that produces the delivery set differs.
        let value_match = self.tree.first_child_with(node, Production::ValueMatch)?;
        let value_if = self.tree.first_child_with(node, Production::ValueIf)?;
        if let Some(initializer) = value_match.or(value_if) {
            let matched = if value_if.is_some() {
                self.check_if(function, initializer, bindings, counters, scope, true)?
            } else {
                self.check_match(function, initializer, bindings, counters, scope, true)?
            };
            if !matched.all_paths_deliver {
                return self.issue_node(
                    SemanticRule::Give1,
                    initializer,
                    SemanticIssueKind::InvalidGive,
                );
            }
            // [GIVE-1] an empty delivery set — every arm leaves by `return`
            // or by `break` — rejects at the `let_stmt` node, because the
            // mechanical fix is the statement form with the binding dropped.
            let Some((mode, expected)) = matched.delivered else {
                return self.issue_node(SemanticRule::Give1, node, SemanticIssueKind::InvalidGive);
            };
            if mode != CheckedMode::Own {
                return self
                    .unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, initializer);
            }
            // [STOR-4] the delivered value lands in this binding, so a
            // delivered arena value whose region block does not enclose the
            // binding has been moved to a destination outside its region.
            if let Some((region, _)) = self.arena_instance(expected)?
                && !self.declaration_is_within_region_block(declaration_id, region)?
            {
                return self.issue_node(
                    SemanticRule::Stor4,
                    initializer,
                    SemanticIssueKind::ArenaEscape {
                        mechanical_fix: super::ARENA_ESCAPE_RESTRUCTURING,
                    },
                );
            }
            // [OWN-5]'s slice-valued-delivery prohibition used to be judged
            // here, one step too late: the branch-state join runs inside the
            // checkers above and stopped with a capability limit before this
            // rejection could be reached. It now lives at the delivery site,
            // in `reject_slice_valued_delivery`, so the rule has one home and
            // no capability stop stands in front of it.
            let state_origins = self
                .type_carries_identity(expected)?
                .then(|| self.give_state_origins(&matched.arms, expected))
                .transpose()?
                .flatten();
            if matched.can_continue
                && bindings
                    .insert(
                        declaration_id,
                        LocalBinding {
                            binding,
                            declaration: declaration_id,
                            mode,
                            ty: expected,
                            state_origins,
                            live: true,
                            loop_depth: scope.loops.len(),
                            compiler_updated: false,
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
            return Ok(StatementResult {
                statement: CheckedStatement::ValueMatchLet {
                    node_path: self.tree.path(node)?.clone(),
                    kind: if value_if.is_some() {
                        ValueInitializerKind::ValueIf
                    } else {
                        ValueInitializerKind::ValueMatch
                    },
                    binding,
                    result_type: expected,
                    scrutinee: matched.scrutinee,
                    enum_type: matched.enum_type,
                    arms: matched.arms,
                    continues: matched.can_continue,
                },
                can_continue: matched.can_continue,
                effects: matched.effects,
                all_paths_deliver: !matched.can_continue,
                direct_give: false,
                give_states: Vec::new(),
                break_states: matched.break_states,
            });
        }
        if let Some(propagate) = self
            .tree
            .first_child_with(node, Production::PropagateLetRhs)?
        {
            return self.check_propagate_let(
                function,
                node,
                propagate,
                declaration_id,
                binding,
                bindings,
                scope,
            );
        }
        if let Some(replace) = self
            .tree
            .first_child_with(node, Production::ReplaceLetRhs)?
        {
            return self.check_replace_let(
                function,
                node,
                replace,
                declaration_id,
                binding,
                bindings,
                scope,
            );
        }
        let expression_owner = if self.tree.production(node)? == Production::ContractDefine {
            node
        } else {
            self.tree
                .first_child_with(node, Production::OrdinaryLetRhs)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
        };
        let expression_node = self
            .tree
            .first_child_with(expression_owner, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // An `ordinary_let_rhs` is always self-typed [TYPE-5], so it is
        // checked with no expectation and the binder takes what it produces.
        let value =
            self.check_expression(function, expression_node, bindings, scope.loops.len())?;
        let mode = value.mode;
        let expected = value.expression.ty();
        if matches!(mode, CheckedMode::Unique(_)) && value.holder.is_some() {
            return self.unsupported(
                UnsupportedSemanticFeature::RegionsAndBorrows,
                expression_node,
            );
        }
        // Binding a borrow-mode call result requires the callee signature to
        // determine its one provenance-candidate parameter. [FN-1] rejects
        // every boundary whose borrow result has no signature-determined
        // source at its own `rtype`, so a bound result is either usable or
        // its declaration is already gone — bindable iff usable. What
        // reaches here is the const-storage disposition, whose claim needs a
        // const-rooted holder the checker does not represent: an explicit
        // capability stop, never an invalid-source verdict [OWN-6, OWN-8].
        if self.reborrow_extension
            && mode != CheckedMode::Own
            && value.borrow.is_none()
            && matches!(value.expression, CheckedExpression::UserCall { .. })
        {
            return self.unsupported(
                UnsupportedSemanticFeature::RegionsAndBorrows,
                expression_node,
            );
        }
        if !self.borrow_holder_scope_supported(declaration_id, mode)? {
            return self.unsupported(
                UnsupportedSemanticFeature::RegionsAndBorrows,
                expression_node,
            );
        }
        let borrow = self.borrow_for_destination(mode, &value, node)?;
        let state_origins = self.state_origins_of_value(&value, bindings)?;
        if bindings
            .insert(
                declaration_id,
                LocalBinding {
                    binding,
                    declaration: declaration_id,
                    mode,
                    ty: expected,
                    state_origins,
                    live: true,
                    loop_depth: scope.loops.len(),
                    compiler_updated: false,
                    borrow,
                    slice: value.slice,
                    slice_loans: Vec::new(),
                    suspended: false,
                },
            )
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(Self::continuing_statement(
            CheckedStatement::Let {
                node_path: self.tree.path(node)?.clone(),
                binding,
                value: value.expression,
            },
            value.effects,
        ))
    }

    /// [SET-2] `let x = replace p = e;`: SET-1's target order with the
    /// affine class judgment, then the fresh old-value binding.
    #[allow(clippy::too_many_arguments)]
    fn check_replace_let(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        replace: NodeId,
        declaration_id: DeclarationId,
        binding: crate::BindingId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let target_node = self
            .tree
            .first_child_with(replace, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression_node = self
            .tree
            .first_child_with(replace, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;

        // [SET-2] fixes SET-1's order: form and check the target first
        // (its affine, region-free class judged inside), then evaluate the
        // right-hand side, then re-establish target-root liveness.
        let (target_declaration, target, target_effects) =
            self.check_replace_target(function, target_node, bindings, scope.loops.len())?;
        let value =
            self.check_expression(function, expression_node, bindings, scope.loops.len())?;
        // [TYPE-5]: the right-hand side must produce exactly `own T`.
        if value.expression.ty() != target.ty() || value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                expression_node,
                SemanticIssueKind::TypeMismatch,
            );
        }
        if !bindings
            .get(&target_declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .live
        {
            return self.issue_node(
                SemanticRule::Own1,
                target_node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        let replacement_origins = self.state_origins_of_value(&value, bindings)?;
        let previous_whole_origins = bindings
            .get(&target_declaration)
            .and_then(|binding| binding.state_origins.clone());
        let target_fields = match &target {
            CheckedSetTarget::Place(place) => Some(place.fields.as_slice()),
            CheckedSetTarget::ArrayIndex(_) | CheckedSetTarget::BufferIndex(_) => None,
        };
        let previous_origins = match (previous_whole_origins.clone(), target_fields) {
            (Some(origins), Some(fields)) => Some(origins.projected(fields)),
            (origins, None) => origins,
            (None, Some(_)) => None,
        };
        let target_carries_identity = self.type_carries_identity(target.ty())?;
        // The moved-out value's sole owner is the fresh ordinary binding;
        // the target root stays live [SET-2, OWN-1].
        if bindings
            .insert(
                declaration_id,
                LocalBinding {
                    binding,
                    declaration: declaration_id,
                    mode: CheckedMode::Own,
                    ty: target.ty(),
                    state_origins: target_carries_identity
                        .then_some(previous_origins)
                        .flatten(),
                    live: true,
                    loop_depth: scope.loops.len(),
                    compiler_updated: false,
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
        if target_carries_identity {
            let updated = match (previous_whole_origins, target_fields) {
                (Some(origins), Some(fields)) => {
                    Some(origins.replace_path(fields, replacement_origins))
                }
                (_, Some(_)) => replacement_origins,
                (origins, None) => origins,
            };
            bindings
                .get_mut(&target_declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .state_origins = updated;
        }
        Ok(Self::continuing_statement(
            CheckedStatement::Replace {
                node_path: self.tree.path(node)?.clone(),
                binding,
                target,
                value: value.expression,
            },
            value.effects.union(target_effects),
        ))
    }

    fn check_region(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let declaration = self.declaration_at(node, DeclarationRole::LocalRegion)?;
        let region = declaration.id();
        let base_keys = bindings.keys().copied().collect::<HashSet<_>>();
        // A region block with arena allocations carries the compiler-owned
        // allocation list [STOR-3]: an ordinary hidden own binding keyed by
        // the region declaration, so `arena_new` sites find it by region and
        // every existing exit-edge drop derivation releases it exactly once
        // per normal edge leaving the block, after the block's own bindings.
        let arena_list = if self.region_allocates_arenas(node, region)? {
            let storage = self.arena_storage_nominal_or_defer()?;
            let list = Self::allocate_binding(counters.next_binding)?;
            counters
                .binding_names
                .push(format!("<arena {}>", declaration.spelling()));
            if bindings
                .insert(
                    region,
                    LocalBinding {
                        binding: list,
                        declaration: region,
                        mode: CheckedMode::Own,
                        ty: CheckedType::Nominal(storage),
                        state_origins: None,
                        live: true,
                        loop_depth: scope.loops.len(),
                        compiler_updated: false,
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
            Some(list)
        } else {
            None
        };
        let statements = self.tree.children_with(node, Production::Stmt)?;
        let mut checked = self.check_block(function, &statements, bindings, counters, scope)?;
        let fallthrough_drops = if checked.can_continue {
            self.live_affine_drops(bindings, &base_keys)?
        } else {
            Vec::new()
        };
        if checked.can_continue {
            bindings.retain(|declaration, _| base_keys.contains(declaration));
        }
        for local in bindings.values_mut() {
            local.end_slice_region(region);
        }
        for state in &mut checked.give_states {
            state.retain(|declaration, _| base_keys.contains(declaration));
            for local in state.values_mut() {
                local.end_slice_region(region);
            }
        }
        for state in &mut checked.break_states {
            state.retain_bindings(&base_keys);
            state.end_slice_region(region);
        }
        Ok(StatementResult {
            statement: CheckedStatement::Region {
                arena_list,
                body: checked.statements,
                fallthrough_drops,
            },
            can_continue: checked.can_continue,
            effects: checked.effects,
            all_paths_deliver: checked.all_paths_deliver,
            direct_give: false,
            give_states: checked.give_states,
            break_states: checked.break_states,
        })
    }

    /// Whether any `call` in this region block resolves to the `arena_new`
    /// operation naming this region [STOR-2]. The judgment reads resolved
    /// operation identity and the resolved region argument — never a source
    /// spelling — so shadowing cannot select it, and an inner region's
    /// allocations register on the inner region's own list.
    fn region_allocates_arenas(
        &self,
        node: NodeId,
        region: DeclarationId,
    ) -> Result<bool, CheckStop> {
        for call in self.tree.descendants_with(node, Production::Call)? {
            let Some(callee) = self.tree.first_child_with(call, Production::Callee)? else {
                continue;
            };
            let usage = self.use_at_roles(
                callee,
                &[
                    LexicalUseRole::IdentifierCallee,
                    LexicalUseRole::OperationCallee,
                ],
            )?;
            let ResolvedTarget::Operation(operation) = usage.target() else {
                continue;
            };
            if crate::operation_family_spelling(operation) != Some("arena_new") {
                continue;
            }
            let Some(targs) = self.tree.first_child_with(call, Production::Targs)? else {
                continue;
            };
            let Some(first) = self
                .tree
                .children_with(targs, Production::Targ)?
                .first()
                .copied()
            else {
                continue;
            };
            let Ok(region_use) = self.use_at(first, LexicalUseRole::TypeArgumentRegion) else {
                continue;
            };
            if region_use.target()
                == (ResolvedTarget::Source {
                    declaration: region,
                    class: DeclarationClass::Region,
                })
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn live_affine_drops(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        preserved: &HashSet<DeclarationId>,
    ) -> Result<Vec<CheckedDrop>, CheckStop> {
        let mut live = bindings
            .iter()
            .filter_map(|(declaration, local)| {
                (local.live && local.mode == CheckedMode::Own && !preserved.contains(declaration))
                    .then_some((*declaration, local.clone()))
            })
            .collect::<Vec<_>>();
        live.sort_by_key(|entry| std::cmp::Reverse(entry.1.binding.0));
        let mut drops = Vec::new();
        for (_, local) in live {
            if !self.is_copy_type(local.ty)? {
                let paths = self.drop_paths(local.ty, Vec::new())?;
                for (fields, ty, release) in self.released_paths(paths)? {
                    drops.push(CheckedDrop {
                        binding: local.binding,
                        state_origins: local
                            .state_origins
                            .clone()
                            .map(|origins| origins.projected(&fields)),
                        fields,
                        ty,
                        release,
                    });
                }
            }
        }
        Ok(drops)
    }

    fn allocate_binding(next_binding: &mut u32) -> Result<BindingId, CheckStop> {
        let binding = BindingId(*next_binding);
        *next_binding = next_binding
            .checked_add(1)
            .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        Ok(binding)
    }

    fn check_claim_justification(&self, node: NodeId) -> Result<ClaimJustification, CheckStop> {
        const LABELS: [&str; 5] = [
            "premises: ",
            "derivation: ",
            "conclusion: ",
            "checker gap: ",
            "consumers: ",
        ];
        const EXPECTED: &str = "exactly five LF-separated nonempty fields: premises, derivation, conclusion, checker gap, consumers";
        let raw = self.check_message(node)?;
        let lines = raw.split('\n').collect::<Vec<_>>();
        if lines.len() != LABELS.len() {
            return self.issue_node(
                SemanticRule::Clm1,
                node,
                SemanticIssueKind::InvalidClaimJustification { expected: EXPECTED },
            );
        }
        let mut values = Vec::with_capacity(LABELS.len());
        for (line, label) in lines.into_iter().zip(LABELS) {
            let Some(value) = line.strip_prefix(label) else {
                return self.issue_node(
                    SemanticRule::Clm1,
                    node,
                    SemanticIssueKind::InvalidClaimJustification { expected: EXPECTED },
                );
            };
            let value = value.trim_matches(' ');
            if value.is_empty() {
                return self.issue_node(
                    SemanticRule::Clm1,
                    node,
                    SemanticIssueKind::InvalidClaimJustification { expected: EXPECTED },
                );
            }
            values.push(value.to_owned());
        }
        let [premises, derivation, conclusion, checker_gap, consumers]: [String; 5] = values
            .try_into()
            .map_err(|_| SemanticCompilerFailure::InvalidResolution)?;
        Ok(ClaimJustification {
            raw,
            premises,
            derivation,
            conclusion,
            checker_gap,
            consumers,
        })
    }

    fn check_claim_proof_predicate(
        &self,
        expression_node: NodeId,
        expression: &CheckedExpression,
        effects: &EffectSet,
    ) -> Result<(), CheckStop> {
        let effectful = !effects.writes.is_empty()
            || effects.allocates_heap
            || !effects.allocates_arenas.is_empty()
            || effects.traps;
        let invalid = if effectful {
            Some("the predicate has a forbidden effect")
        } else {
            self.invalid_claim_expression(expression, false)?
        };
        if let Some(reason) = invalid {
            return self.issue_node(
                SemanticRule::Clm1,
                expression_node,
                SemanticIssueKind::InvalidClaimProofPredicate { reason },
            );
        }
        Ok(())
    }

    /// Returns the first reason an expression falls outside CLM-1's proof
    /// subset. `holder` admits an affine owner only while an explicit deref
    /// reads copy content; it never admits the owner as a proof datum.
    fn invalid_claim_expression(
        &self,
        expression: &CheckedExpression,
        holder: bool,
    ) -> Result<Option<&'static str>, CheckStop> {
        let recurse = |this: &Self,
                       arguments: &[CheckedExpression]|
         -> Result<Option<&'static str>, CheckStop> {
            for argument in arguments {
                if let Some(reason) = this.invalid_claim_expression(argument, false)? {
                    return Ok(Some(reason));
                }
            }
            Ok(None)
        };
        Ok(match expression {
            CheckedExpression::Constant(_) | CheckedExpression::NamedConstant { .. } => None,
            CheckedExpression::Binding {
                ty, consume_root, ..
            } => {
                if *consume_root {
                    Some("the predicate consumes a binding")
                } else if holder || self.is_copy_type(*ty)? {
                    None
                } else {
                    Some("the predicate reads an affine value rather than copy data")
                }
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                ..
            } => {
                if operation.is_exact() {
                    Some("the predicate contains a proof-required exact operation")
                } else if operation.checked_error().is_some() {
                    Some("the predicate contains a checked-result operation")
                } else {
                    recurse(self, arguments)?
                }
            }
            CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. } => recurse(self, arguments)?,
            CheckedExpression::BooleanOperation {
                operation,
                arguments,
                ..
            } => {
                if *operation == CheckedBooleanOperation::ExclusiveOr {
                    // Evaluation is a valid proof expression; CLM-1 later
                    // rejects its unsupported canonical contribution form.
                }
                recurse(self, arguments)?
            }
            CheckedExpression::NumericConversion { value, .. }
            | CheckedExpression::Reinterpret { value, .. } => {
                self.invalid_claim_expression(value, false)?
            }
            CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceLength { .. }
            | CheckedExpression::DerefAddressed { .. } => None,
            CheckedExpression::BufferFits { length, .. } => {
                self.invalid_claim_expression(length, false)?
            }
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. } => {
                self.invalid_claim_expression(value, true)?
            }
            CheckedExpression::Project {
                ty,
                consume_root,
                residual_drops,
                ..
            } => {
                if *consume_root || !residual_drops.is_empty() {
                    Some("the predicate contains a consuming projection or residual cleanup")
                } else if holder || self.is_copy_type(*ty)? {
                    None
                } else {
                    Some("the predicate projects affine data")
                }
            }
            CheckedExpression::ProjectValue { value, ty, .. } => {
                if holder || self.is_copy_type(*ty)? {
                    self.invalid_claim_expression(value, true)?
                } else {
                    Some("the predicate projects affine data")
                }
            }
            CheckedExpression::UserCall { .. } => {
                Some("the predicate contains a user call that may not terminate")
            }
            CheckedExpression::SystemCall { .. } => Some("the predicate contains a system call"),
            CheckedExpression::ArrayIndex { .. }
            | CheckedExpression::BufferIndex { .. }
            | CheckedExpression::SliceIndex { .. } => Some("the predicate contains a subscript"),
            CheckedExpression::ArrayFill { .. }
            | CheckedExpression::BufferFill { .. }
            | CheckedExpression::BufferVacant { .. }
            | CheckedExpression::BoxNew { .. }
            | CheckedExpression::ArenaNew { .. }
            | CheckedExpression::ConstructStruct { .. }
            | CheckedExpression::ConstructEnum { .. } => {
                Some("the predicate contains construction or allocation")
            }
            CheckedExpression::SliceOf { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. } => {
                Some("the predicate changes borrowing or ownership")
            }
        })
    }

    fn continuing_statement(statement: CheckedStatement, effects: EffectSet) -> StatementResult {
        StatementResult {
            statement,
            can_continue: true,
            effects,
            all_paths_deliver: false,
            direct_give: false,
            give_states: Vec::new(),
            break_states: Vec::new(),
        }
    }
}
