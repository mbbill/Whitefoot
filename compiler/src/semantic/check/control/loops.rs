use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, UnsupportedSemanticFeature,
};

use super::super::super::model::{CheckedLoopId, CheckedStatement};
use super::super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use super::{ControlCounters, ControlScope, StatementResult};

#[derive(Clone)]
pub(in crate::semantic::check) struct LoopContext {
    pub(super) id: CheckedLoopId,
    declaration: DeclarationId,
    preserved: HashSet<DeclarationId>,
}

pub(in crate::semantic::check) struct BreakState {
    target: CheckedLoopId,
    bindings: HashMap<DeclarationId, LocalBinding>,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_loop(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let declaration = self.declaration_at(node, DeclarationRole::LoopLabel)?.id();
        let id = Self::allocate_loop(counters.next_loop)?;
        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let preserved = base_keys.iter().copied().collect::<HashSet<_>>();
        let mut nested_loops = scope.loops.to_vec();
        nested_loops.push(LoopContext {
            id,
            declaration,
            preserved: preserved.clone(),
        });

        let mut body_bindings = base_bindings.clone();
        let statements = self.tree.children_with(node, Production::Stmt)?;
        let checked = self.check_block(
            function,
            &statements,
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
        let usage = self.use_at(node, LexicalUseRole::BreakLabel)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Label,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let target = scope
            .loops
            .iter()
            .rev()
            .find(|context| context.declaration == declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
}
