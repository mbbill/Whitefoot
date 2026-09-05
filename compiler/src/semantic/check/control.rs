use std::cell::Cell;
use std::collections::{HashMap, HashSet};

mod commit;
mod loops;
mod matches;
mod proofs;
mod results;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::model::{
    BindingId, CheckedDrop, CheckedExpression, CheckedLoopId, CheckedMode, CheckedProjectedDrop,
    CheckedSetTarget, CheckedStatement, CheckedType, ValueInitializerKind,
};
use super::borrows::ReborrowPosition;
use super::expressions::MutationTarget;
use super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
pub(super) use commit::CommitReadOut;
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
            Production::InvariantStmt => {
                self.check_local_invariant(node, bindings, function, scope.loops.len())
            }
            // [FN-1, GRAM-4] a `return` writes exactly as many expressions as
            // the enclosing declaration writes results, and expression i
            // produces result ordinal i. A count mismatch is the ordinary
            // FN-1 return-shape rejection at the `return_stmt`.
            Production::ReturnStmt
                if self.tree.children_with(node, Production::Expr)?.len()
                    != function.results.len() =>
            {
                self.issue_node(SemanticRule::Fn1, node, SemanticIssueKind::ReturnMismatch)
            }
            Production::ReturnStmt if function.results.len() > 1 => {
                self.check_result_list_return(function, node, bindings, scope)
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
                        drops: self.live_affine_drops(bindings, &HashSet::new(), node)?,
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
                    Some((mode, ty)) => {
                        return self.issue_node(
                            SemanticRule::Give1,
                            node,
                            SemanticIssueKind::type_mismatch(
                                self.checked_value_name(mode, ty)?,
                                self.checked_value_name(value.mode, value.expression.ty())?,
                            ),
                        );
                    }
                }
                Ok(StatementResult {
                    statement: CheckedStatement::Give {
                        node_path: self.tree.path(node)?.clone(),
                        value: value.expression,
                        drops: self.live_affine_drops(bindings, &context.preserved, node)?,
                    },
                    can_continue: false,
                    effects: value.effects,
                    all_paths_deliver: true,
                    direct_give: true,
                    give_states: vec![bindings.clone()],
                    break_states: Vec::new(),
                })
            }
            // [GRAM-4, SET-1, LIV-2] every written `set` is one commit: the
            // targets are resolved and judged first, then the whole
            // right-hand side, then the three admission conditions.
            Production::SetStmt => self.check_commit(function, node, bindings, counters, scope),
            Production::LoopStmt => self.check_loop(function, node, bindings, counters, scope),
            Production::ForStmt => {
                self.check_counted_range(function, node, bindings, counters, scope)
            }
            Production::BreakStmt => self.check_break(node, bindings, scope),
            // [PROV-6, GRAM-4] `dispose p;` runs at this point exactly the
            // release walk the scope exit would have run for `p`.
            Production::DisposeStmt => self.check_dispose(function, node, bindings, scope),
            Production::RegionStmt => self.check_region(function, node, bindings, counters, scope),
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// [PROV-6, GRAM-4] `dispose p;`.
    ///
    /// The admission is judged over `p`'s release graph before the operand's
    /// own ownership consume, so a value the walk could never reclaim is
    /// refused at the statement rather than after it has killed a binding.
    fn check_dispose(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let place = self
            .tree
            .first_child_with(node, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // The statement is one consuming use of `p`'s root [OWN-1], and every
        // ownership rejection here — a shared-borrow root, a dead root, a live
        // loan, a partial consume of a linear value — is that judgment's,
        // asked first because [OWN-1] is defined before [PROV-6] [DIAG-1].
        let value =
            self.check_consumed_place(function, node, place, bindings, scope.loops.len(), false)?;
        if value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "dispose an own-mode value; a borrow owns nothing to release",
                },
            );
        }
        let ty = value.expression.ty();
        self.dispose_admission(ty, node)?;
        let whole_origins = self.state_origins_of_value(&value, bindings)?;
        let paths = self.drop_paths(ty, Vec::new())?;
        // [PROV-6, EFF-2] the statement writes `p`'s ultimate storage origin
        // exactly as a commit writes its target's, and each release the walk
        // runs contributes its own [SYS-5] row here rather than through the
        // release contribution, because `dispose` is a written statement.
        let mut effects = value.effects;
        for access in &value.accesses {
            for path in self.effect_paths_for_place(&access.place, bindings)? {
                effects.add_write(path);
            }
        }
        let mut drops = Vec::new();
        for (fields, ty, release) in self.released_paths(paths)? {
            let state_origins = whole_origins
                .clone()
                .map(|origins| origins.projected(&fields));
            effects = effects.union(self.effects_of_row(release.row, state_origins.as_ref())?);
            drops.push(CheckedProjectedDrop {
                state_origins,
                fields,
                ty,
                release,
            });
        }
        Ok(Self::continuing_statement(
            CheckedStatement::Dispose {
                node_path: self.tree.path(node)?.clone(),
                value: value.expression,
                drops,
            },
            effects,
        ))
    }

    fn check_let(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        // [PROV-6, GRAM-4] the destructuring consume is the one `let`
        // alternative whose operand place is a direct child of the statement.
        if let Some(place) = self.tree.first_child_with(node, Production::Place)? {
            return self
                .check_destructuring_consume(function, node, place, bindings, counters, scope);
        }
        // [GRAM-4, CALL-4] a binder list takes its right-hand side's result
        // ordinals; a `call` directly under the `let_stmt` is that form and
        // no other selects it.
        if let Some(call) = self.tree.first_child_with(node, Production::Call)? {
            return self.check_destructuring_let(function, node, call, bindings, counters, scope);
        }
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
        // reaches here is the const-storage disposition, whose validity needs a
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
        // [PROV-3] a loan's extent is its holding value's own liveness, and
        // this `let` is where that value becomes a binding with uses. Every
        // origin place this value reaches — formed here, copied, passed
        // through a call, or returned — names the loans this binding now
        // holds.
        Self::hold_slice_loans_of(declaration_id, value.slice.as_ref(), bindings);
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
        let MutationTarget {
            declaration: target_declaration,
            target,
            effects: target_effects,
            unsupported: target_unsupported,
            ..
        } = self.check_replace_target(function, target_node, bindings, scope.loops.len())?;
        let value =
            self.check_expression(function, expression_node, bindings, scope.loops.len())?;
        // [TYPE-5]: the right-hand side must produce exactly `own T`.
        if value.expression.ty() != target.ty() || value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                expression_node,
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(target.ty())?),
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
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
        // Every source rejection of this statement is judged above; a target
        // this compiler cannot lower stops here and nowhere earlier [DIAG-1].
        if let Some(feature) = target_unsupported {
            return self.unsupported(feature, target_node);
        }
        let replacement_origins = self.state_origins_of_value(&value, bindings)?;
        let previous_whole_origins = bindings
            .get(&target_declaration)
            .and_then(|binding| binding.state_origins.clone());
        let target_fields = match &target {
            CheckedSetTarget::Place(place) => Some(place.fields.as_slice()),
            CheckedSetTarget::ArrayIndex(_)
            | CheckedSetTarget::BufferIndex(_)
            | CheckedSetTarget::RunIndex(_)
            | CheckedSetTarget::SliceIndex(_) => None,
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
        // [FORM-8] a loop body is itself a region block [OWN-11], so a block
        // that is the body's only statement has exactly the body's own block
        // and is a second spelling of its region. The exception is a block
        // some `targ` inside it must write the name at, because an implicit
        // region has no name to put there.
        if self.region_block_is_the_loop_body(node)?
            && !(self.writes_region(node)?
                && self.region_is_type_argument_below(node, declaration.spelling())?)
        {
            return self.issue_node(
                SemanticRule::Form8,
                node,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "the loop body is its own region; remove the region block, \
keep its statements where they stand, and drop every region name it carried",
                },
            );
        }
        // [FORM-8] the block writes its name exactly when its body still
        // references it after elision.
        if self.writes_region(node)?
            && !self.region_is_referenced_below(node, declaration.spelling())?
        {
            return self.issue_node(
                SemanticRule::Form8,
                node,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "drop the region name: nothing inside this block names it, \
so the block is written `region { ... }`",
                },
            );
        }
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
            self.live_affine_drops(bindings, &base_keys, node)?
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

    /// The compiler-derived releases one edge leaving a scope carries
    /// [STOR-3, LIV-1], and the [PROV-6] refusal of a value that is linear in
    /// this scope and has no derived release to carry it there.
    fn live_affine_drops(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        preserved: &HashSet<DeclarationId>,
        edge: NodeId,
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
        for (_, local) in &live {
            let name = self
                .resolved
                .declarations()
                .iter()
                .find(|declaration| declaration.id() == local.declaration)
                .map_or_else(String::new, |declaration| declaration.spelling().to_owned());
            self.reject_linear_value_not_consumed(local.ty, &name, edge)?;
        }
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
