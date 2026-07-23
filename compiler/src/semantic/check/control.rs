use std::collections::{HashMap, HashSet};

mod loops;
mod matches;
mod results;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, Production, SemanticCompilerFailure, SemanticIssue,
    SemanticIssueKind, SemanticLocation, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{
    BindingId, CheckedDrop, CheckedLoopId, CheckedMode, CheckedStatement, CheckedType, TrapSite,
};
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
    expected: CheckedType,
    preserved: HashSet<DeclarationId>,
    enclosing_loops: HashSet<CheckedLoopId>,
}

pub(super) struct ControlCounters<'state> {
    pub(super) next_binding: &'state mut u32,
    pub(super) next_loop: &'state mut u32,
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
            Production::LetStmt => self.check_let(function, node, bindings, counters, scope),
            Production::ExprStmt => {
                let call = self
                    .tree
                    .first_child_with(node, Production::Call)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let value = self.check_call(function, call, bindings, scope.loops.len())?;
                let statement = if self.is_copy_type(value.expression.ty())? {
                    CheckedStatement::Evaluate(value.expression)
                } else {
                    CheckedStatement::DropExpression(value.expression)
                };
                Ok(Self::continuing_statement(statement, value.effects))
            }
            Production::ReturnStmt => {
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let value = self.check_expression_with_expected(
                    function,
                    expression_node,
                    bindings,
                    scope.loops.len(),
                    Some(function.result),
                )?;
                if value.expression.ty() != function.result {
                    return Err(CheckStop::Issue(SemanticIssue {
                        rule: SemanticRule::Fn1,
                        location: SemanticLocation::SourceNode(
                            self.tree.path(node)?.clone(),
                            self.tree.coordinate(expression_node)?,
                        ),
                        kind: SemanticIssueKind::ReturnMismatch,
                    }));
                }
                if value.mode != function.result_mode {
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
                Ok(StatementResult {
                    statement: CheckedStatement::Return {
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
            Production::CheckStmt => {
                let expression_node = self
                    .tree
                    .first_child_with(node, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let condition =
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?;
                if condition.expression.ty() != CheckedType::Bool
                    || condition.mode != CheckedMode::Own
                {
                    return Err(CheckStop::Issue(SemanticIssue {
                        rule: SemanticRule::Op5,
                        location: SemanticLocation::SourceNode(
                            self.tree.path(node)?.clone(),
                            self.tree.coordinate(expression_node)?,
                        ),
                        kind: SemanticIssueKind::InvalidCheckCondition,
                    }));
                }
                Ok(Self::continuing_statement(
                    CheckedStatement::Check {
                        condition: condition.expression,
                        trap: TrapSite {
                            rule_id: "OP-5",
                            message: self.check_message(node)?,
                            function: function.name.clone(),
                            node_path: self.tree.path(node)?.clone(),
                        },
                    },
                    condition.effects.union(EffectSet::TRAPS),
                ))
            }
            Production::MatchStmt => {
                let matched = self.check_match(function, node, bindings, counters, scope, None)?;
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
                let value = self.check_expression_with_expected(
                    function,
                    expression_node,
                    bindings,
                    scope.loops.len(),
                    Some(context.expected),
                )?;
                if value.expression.ty() != context.expected {
                    return self.issue_node(
                        SemanticRule::Type5,
                        node,
                        SemanticIssueKind::TypeMismatch,
                    );
                }
                Ok(StatementResult {
                    statement: CheckedStatement::Give {
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
                let value = self.check_expression_with_expected(
                    function,
                    expression_node,
                    bindings,
                    scope.loops.len(),
                    Some(target.ty()),
                )?;
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
                        target,
                        value: value.expression,
                    },
                    value.effects.union(target_effects),
                ))
            }
            Production::LoopStmt => self.check_loop(function, node, bindings, counters, scope),
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
        let mode = self
            .tree
            .first_child_with(node, Production::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self.parse_mode(mode)?;
        let ty_node = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expected = self.parse_type_with(ty_node, &function.substitution)?;
        let declaration = self.declaration_at(node, DeclarationRole::Let)?;
        let declaration_id = declaration.id();
        let binding = Self::allocate_binding(counters.next_binding)?;

        if let Some(value_match) = self.tree.first_child_with(node, Production::ValueMatch)? {
            if mode != CheckedMode::Own {
                return self
                    .unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, value_match);
            }
            let matched = self.check_match(
                function,
                value_match,
                bindings,
                counters,
                scope,
                Some(expected),
            )?;
            if !matched.all_paths_deliver {
                return self.issue_node(
                    SemanticRule::Give1,
                    value_match,
                    SemanticIssueKind::InvalidGive,
                );
            }
            if matched.can_continue
                && bindings
                    .insert(
                        declaration_id,
                        LocalBinding {
                            binding,
                            declaration: declaration_id,
                            mode,
                            ty: expected,
                            live: true,
                            loop_depth: scope.loops.len(),
                            borrow: None,
                        },
                    )
                    .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            return Ok(StatementResult {
                statement: CheckedStatement::ValueMatchLet {
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
            if mode != CheckedMode::Own {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, propagate);
            }
            return self.check_propagate_let(
                function,
                propagate,
                expected,
                declaration_id,
                binding,
                bindings,
                scope,
            );
        }
        let rhs = self
            .tree
            .first_child_with(node, Production::OrdinaryLetRhs)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression_node = self
            .tree
            .first_child_with(rhs, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let value = self.check_expression_with_expected(
            function,
            expression_node,
            bindings,
            scope.loops.len(),
            Some(expected),
        )?;
        if value.expression.ty() != expected {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        }
        if matches!(mode, CheckedMode::Unique(_)) && value.holder.is_some() {
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
        if bindings
            .insert(
                declaration_id,
                LocalBinding {
                    binding,
                    declaration: declaration_id,
                    mode,
                    ty: expected,
                    live: true,
                    loop_depth: scope.loops.len(),
                    borrow,
                },
            )
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(Self::continuing_statement(
            CheckedStatement::Let {
                binding,
                value: value.expression,
            },
            value.effects,
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
        let _region = self.declaration_at(node, DeclarationRole::LocalRegion)?;
        let base_keys = bindings.keys().copied().collect::<HashSet<_>>();
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
        for state in &mut checked.give_states {
            state.retain(|declaration, _| base_keys.contains(declaration));
        }
        for state in &mut checked.break_states {
            state.retain_bindings(&base_keys);
        }
        Ok(StatementResult {
            statement: CheckedStatement::Region {
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
        live.sort_by(|left, right| right.1.binding.0.cmp(&left.1.binding.0));
        let mut drops = Vec::new();
        for (_, local) in live {
            if !self.is_copy_type(local.ty)? {
                for (fields, ty) in self.drop_paths(local.ty, Vec::new())? {
                    drops.push(CheckedDrop {
                        binding: local.binding,
                        fields,
                        ty,
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
