use std::collections::{HashMap, HashSet};

mod loops;
mod matches;
mod results;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, ProductionV0_14, SemanticCompilerFailure, SemanticIssue,
    SemanticIssueKind, SemanticLocation, SemanticRuleV0_14, UnsupportedSemanticFeatureV0_14,
};

use super::super::model::{
    BindingId, CheckedDrop, CheckedLoopId, CheckedStatement, CheckedType, TrapSite,
};
use super::{CheckStop, Checker, FunctionSignature, LocalBinding};
use loops::{BreakState, LoopContext};

pub(super) struct BlockResult {
    pub(super) statements: Vec<CheckedStatement>,
    pub(super) can_continue: bool,
    pub(super) exhibits_traps: bool,
    all_paths_deliver: bool,
    give_states: Vec<HashMap<DeclarationId, LocalBinding>>,
    break_states: Vec<BreakState>,
}

struct StatementResult {
    statement: CheckedStatement,
    can_continue: bool,
    exhibits_traps: bool,
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
        let mut exhibits_traps = false;
        let mut all_paths_deliver = false;
        let mut direct_give = false;
        let mut give_states = Vec::new();
        let mut break_states = Vec::new();
        for wrapper in statement_wrappers {
            let statement = self.tree.only_child(*wrapper)?;
            if !can_continue {
                return self.issue_node(
                    if direct_give {
                        SemanticRuleV0_14::Give1
                    } else {
                        SemanticRuleV0_14::Fn1
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
            exhibits_traps |= checked.exhibits_traps;
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
            exhibits_traps,
            all_paths_deliver,
            give_states,
            break_states,
        })
    }

    fn check_statement(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        match self.tree.production(node)? {
            ProductionV0_14::LetStmt => self.check_let(function, node, bindings, counters, scope),
            ProductionV0_14::ExprStmt => {
                let call = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Call)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let value = self.check_call(function, call, bindings, scope.loops.len())?;
                let statement = if self.is_copy_type(value.expression.ty())? {
                    CheckedStatement::Evaluate(value.expression)
                } else {
                    CheckedStatement::DropExpression(value.expression)
                };
                Ok(Self::continuing_statement(statement, value.exhibits_traps))
            }
            ProductionV0_14::ReturnStmt => {
                let expression_node = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Expr)?
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
                        rule: SemanticRuleV0_14::Fn1,
                        location: SemanticLocation::SourceNode(
                            self.tree.path(node)?.clone(),
                            self.tree.coordinate(expression_node)?,
                        ),
                        kind: SemanticIssueKind::ReturnMismatch,
                    }));
                }
                Ok(StatementResult {
                    statement: CheckedStatement::Return {
                        value: value.expression,
                        drops: self.live_affine_drops(bindings, &HashSet::new())?,
                    },
                    can_continue: false,
                    exhibits_traps: value.exhibits_traps,
                    all_paths_deliver: true,
                    direct_give: false,
                    give_states: Vec::new(),
                    break_states: Vec::new(),
                })
            }
            ProductionV0_14::CheckStmt => {
                let expression_node = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let condition =
                    self.check_expression(function, expression_node, bindings, scope.loops.len())?;
                if condition.expression.ty() != CheckedType::Bool {
                    return Err(CheckStop::Issue(SemanticIssue {
                        rule: SemanticRuleV0_14::Op5,
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
                    true,
                ))
            }
            ProductionV0_14::MatchStmt => {
                let matched = self.check_match(function, node, bindings, counters, scope, None)?;
                Ok(StatementResult {
                    statement: CheckedStatement::Match {
                        scrutinee: matched.scrutinee,
                        enum_type: matched.enum_type,
                        arms: matched.arms,
                        continues: matched.can_continue,
                    },
                    can_continue: matched.can_continue,
                    exhibits_traps: matched.exhibits_traps,
                    all_paths_deliver: matched.all_paths_deliver,
                    direct_give: false,
                    give_states: matched.give_states,
                    break_states: matched.break_states,
                })
            }
            ProductionV0_14::GiveStmt => {
                let Some(context) = scope.give_context else {
                    return self.issue_node(
                        SemanticRuleV0_14::Give1,
                        node,
                        SemanticIssueKind::InvalidGive,
                    );
                };
                let expression_node = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Expr)?
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
                        SemanticRuleV0_14::Type5,
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
                    exhibits_traps: value.exhibits_traps,
                    all_paths_deliver: true,
                    direct_give: true,
                    give_states: vec![bindings.clone()],
                    break_states: Vec::new(),
                })
            }
            ProductionV0_14::SetStmt => {
                let target_node = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Place)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let expression_node = self
                    .tree
                    .first_child_with(node, ProductionV0_14::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;

                // SET-1 fixes this order: form and check the target first, then
                // evaluate the RHS, then re-establish target writability.
                let (declaration, target) = self.check_set_target(target_node, bindings)?;
                let value = self.check_expression_with_expected(
                    function,
                    expression_node,
                    bindings,
                    scope.loops.len(),
                    Some(target.ty),
                )?;
                if value.expression.ty() != target.ty {
                    return self.issue_node(
                        SemanticRuleV0_14::Type5,
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
                        SemanticRuleV0_14::Own1,
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
                    value.exhibits_traps,
                ))
            }
            ProductionV0_14::LoopStmt => self.check_loop(function, node, bindings, counters, scope),
            ProductionV0_14::BreakStmt => self.check_break(node, bindings, scope),
            ProductionV0_14::RegionStmt => {
                self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, node)
            }
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
            .first_child_with(node, ProductionV0_14::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.require_own_mode(mode)?;
        let ty_node = self
            .tree
            .first_child_with(node, ProductionV0_14::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expected = self.parse_type(ty_node)?;
        let declaration = self.declaration_at(node, DeclarationRole::Let)?;
        let declaration_id = declaration.id();
        let binding = Self::allocate_binding(counters.next_binding)?;

        if let Some(value_match) = self
            .tree
            .first_child_with(node, ProductionV0_14::ValueMatch)?
        {
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
                    SemanticRuleV0_14::Give1,
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
                            ty: expected,
                            live: true,
                            loop_depth: scope.loops.len(),
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
                exhibits_traps: matched.exhibits_traps,
                all_paths_deliver: !matched.can_continue,
                direct_give: false,
                give_states: Vec::new(),
                break_states: matched.break_states,
            });
        }
        if let Some(propagate) = self
            .tree
            .first_child_with(node, ProductionV0_14::PropagateLetRhs)?
        {
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
            .first_child_with(node, ProductionV0_14::OrdinaryLetRhs)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression_node = self
            .tree
            .first_child_with(rhs, ProductionV0_14::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let value = self.check_expression_with_expected(
            function,
            expression_node,
            bindings,
            scope.loops.len(),
            Some(expected),
        )?;
        if value.expression.ty() != expected {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        }
        if bindings
            .insert(
                declaration_id,
                LocalBinding {
                    binding,
                    ty: expected,
                    live: true,
                    loop_depth: scope.loops.len(),
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
            value.exhibits_traps,
        ))
    }

    fn live_affine_drops(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        preserved: &HashSet<DeclarationId>,
    ) -> Result<Vec<CheckedDrop>, CheckStop> {
        let mut drops = Vec::new();
        for (declaration, local) in bindings {
            if local.live && !preserved.contains(declaration) && !self.is_copy_type(local.ty)? {
                drops.push(CheckedDrop {
                    binding: local.binding,
                    ty: local.ty,
                });
            }
        }
        drops.sort_by(|left, right| right.binding.0.cmp(&left.binding.0));
        Ok(drops)
    }

    fn allocate_binding(next_binding: &mut u32) -> Result<BindingId, CheckStop> {
        let binding = BindingId(*next_binding);
        *next_binding = next_binding
            .checked_add(1)
            .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        Ok(binding)
    }

    fn continuing_statement(statement: CheckedStatement, exhibits_traps: bool) -> StatementResult {
        StatementResult {
            statement,
            can_continue: true,
            exhibits_traps,
            all_paths_deliver: false,
            direct_give: false,
            give_states: Vec::new(),
            break_states: Vec::new(),
        }
    }
}
