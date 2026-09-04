use std::collections::HashSet;

use crate::semantic::{
    BindingId, CheckedDrop, CheckedExpression, CheckedLoopId, CheckedMatchArm, CheckedSetTarget,
    CheckedStatement,
};
use crate::{
    IrBlockId, IrBooleanOperation, IrCompletionPipeline, IrCompletionWindow, IrConstant,
    IrEnumType, IrIntegerOperation, IrMatchTarget, IrOperation, IrTerminator, IrType, IrValueId,
    LoweringFailure, NodePath,
};

use super::{GiveTarget, IrBuilder};

pub(super) const U64: IrType = IrType::Integer {
    width: 64,
    signed: false,
};

#[derive(Clone)]
pub(super) struct LoopTarget {
    pub(super) id: CheckedLoopId,
    pub(super) block: IrBlockId,
    pub(super) carried_bindings: Vec<BindingId>,
}

impl IrBuilder<'_> {
    pub(super) fn lower_loop(
        &mut self,
        id: CheckedLoopId,
        body: &[CheckedStatement],
        backedge_drops: &[CheckedDrop],
        give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        let pipeline_entry = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.note_staged_pipeline(id, pipeline_entry, crate::IrCompletionWindow::new(0, 0, 1));
        let base_bindings = self.bindings.clone();
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let parameter_types = carried_bindings
            .iter()
            .map(|binding| {
                base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
                    .and_then(|value| self.value_type(value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (header, header_parameters) = self.new_block(&parameter_types)?;
        let (exit, exit_parameters) = self.new_block(&parameter_types)?;
        let entry_arguments = self.binding_values(&carried_bindings)?;
        self.terminate(IrTerminator::Jump {
            target: header,
            arguments: entry_arguments,
            drops: Vec::new(),
        })?;

        self.current = Some(header);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, &header_parameters)?;
        self.loops.push(LoopTarget {
            id,
            block: exit,
            carried_bindings: carried_bindings.clone(),
        });
        if backedge_drops.is_empty() {
            self.emit_probe_skip_if_recognized(id, body, header, &carried_bindings)?;
        }
        self.lower_statements(body, give_target)?;
        if self.current.is_some() {
            let arguments = self.binding_values(&carried_bindings)?;
            let drops = self.lower_drops(backedge_drops)?;
            self.terminate(IrTerminator::Jump {
                target: header,
                arguments,
                drops,
            })?;
        }
        let Some(target) = self.loops.pop() else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        if target.id != id || target.block != exit {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }

        self.current = Some(exit);
        self.bindings = base_bindings;
        self.bind_parameters(&carried_bindings, &exit_parameters)
    }

    /// Lowers one counted `for`, either as the block graph below or — when
    /// [PAR-2] permitted this loop and this compilation asked for
    /// actualization — as a recursive split of its index range.
    ///
    /// The split is not a second lowering of the loop: its chunk *is* this
    /// block graph, built by the same code into a synthesized function, and
    /// the sequential world calls that chunk. What the split adds is a
    /// splitter beside it and a two-way rendering at this site.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_counted_range(
        &mut self,
        id: CheckedLoopId,
        node_path: &NodePath,
        binder: BindingId,
        lower: &CheckedExpression,
        upper: &CheckedExpression,
        body: &[CheckedStatement],
        backedge_drops: &[CheckedDrop],
        give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        // [FN-1] fixes endpoint evaluation order and evaluates each exactly
        // once. These SSA values are the private immutable captures, and they
        // are captured before the split decision so both renderings evaluate
        // the endpoints in the same place, exactly once.
        let lower_capture = self.expression(lower)?;
        let upper_capture = self.expression(upper)?;
        if self.value_type(lower_capture)? != U64 || self.value_type(upper_capture)? != U64 {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        if self.split_counted_range(
            id,
            node_path,
            binder,
            body,
            backedge_drops,
            lower_capture,
            upper_capture,
        )? {
            return Ok(());
        }
        if self.lower_bounded_completion_range(
            id,
            binder,
            lower,
            upper,
            body,
            backedge_drops,
            give_target.clone(),
            lower_capture,
            upper_capture,
        )? {
            return Ok(());
        }
        let pipeline_entry = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.note_staged_pipeline(
            id,
            pipeline_entry,
            crate::IrCompletionWindow::new(counted_span(lower, upper), 0, 1),
        );
        self.counted_range_graph(
            id,
            binder,
            body,
            backedge_drops,
            give_target,
            lower_capture,
            upper_capture,
        )
    }

    /// Lowers the first complete multi-slot completion schedule.
    ///
    /// The admitted IR topology is deliberately narrow: a straight-line
    /// prologue ending in the selected result match, no loop-body cleanup, and
    /// a remainder that does not read the counted binder or a prologue-local
    /// value. Those are implementation limits, not source-language
    /// rejections. A loop outside this subset continues through the ordinary
    /// graph and the complete one-slot driver below.
    #[allow(clippy::too_many_arguments)]
    fn lower_bounded_completion_range(
        &mut self,
        id: CheckedLoopId,
        binder: BindingId,
        lower: &CheckedExpression,
        upper: &CheckedExpression,
        body: &[CheckedStatement],
        backedge_drops: &[CheckedDrop],
        give_target: Option<GiveTarget>,
        lower_capture: IrValueId,
        upper_capture: IrValueId,
    ) -> Result<bool, LoweringFailure> {
        if exact_counted_span(lower, upper).is_some_and(|span| span <= 1) {
            return Ok(false);
        }
        let Some(cut) = self.unique_staged_cut(id) else {
            return Ok(false);
        };
        let Some(direct) = direct_staged_match(body, &cut) else {
            return Ok(false);
        };
        if give_target.is_some()
            || !backedge_drops.is_empty()
            || self.addressed_bindings.contains(&binder)
        {
            return Ok(false);
        }

        let mut unavailable_in_remainder = HashSet::from([binder]);
        for statement in &direct.prologue {
            match statement {
                CheckedStatement::Let { binding, .. }
                | CheckedStatement::Replace { binding, .. } => {
                    unavailable_in_remainder.insert(*binding);
                }
                CheckedStatement::Set { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Proof(_) => {}
                _ => return Ok(false),
            }
        }
        if unavailable_in_remainder
            .iter()
            .any(|binding| self.addressed_bindings.contains(binding))
            || direct.arms.iter().any(|arm| {
                arm.body
                    .iter()
                    .any(|statement| statement_uses_any(statement, &unavailable_in_remainder))
                    || drops_use_any(&arm.fallthrough_drops, &unavailable_in_remainder)
            })
        {
            return Ok(false);
        }

        let base_bindings = self.bindings.clone();
        if base_bindings.contains_key(&binder) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let carried_types = carried_bindings
            .iter()
            .map(|binding| {
                base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
                    .and_then(|value| self.value_type(value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let carried_count = carried_types.len();

        // Match edges carry no SSA arguments. Each edge therefore lands in a
        // parameterless block which performs the explicit jump into the
        // parameterized issue, drain, or exit block.
        let (empty_edge, _) = self.new_block(&[])?;
        let (window_entry, _) = self.new_block(&[])?;
        let mut issue_types = carried_types.clone();
        issue_types.extend([U64, U64, U64]); // index, upper, issued count
        let (issue, issue_parameters) = self.new_block(&issue_types)?;
        let (issue_again_edge, _) = self.new_block(&[])?;
        let (start_drain_edge, _) = self.new_block(&[])?;
        let mut drain_types = carried_types.clone();
        drain_types.extend([U64, U64, U64, U64]); // next index, upper, count, slot
        let (drain, drain_parameters) = self.new_block(&drain_types)?;
        let (exit, exit_parameters) = self.new_block(&carried_types)?;

        // The empty path never asks for a runtime window and never enters a
        // completion drain.
        let nonempty = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![lower_capture, upper_capture],
            },
        )?;
        self.terminate(IrTerminator::Match {
            scrutinee: nonempty,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: window_entry,
                },
                IrMatchTarget {
                    tag: 0,
                    block: empty_edge,
                },
            ],
        })?;
        self.current = Some(empty_edge);
        self.bindings = base_bindings.clone();
        let empty_arguments = self.binding_values(&carried_bindings)?;
        self.terminate(IrTerminator::Jump {
            target: exit,
            arguments: empty_arguments,
            drops: Vec::new(),
        })?;

        // The pipeline descriptor itself defines this value at window_entry.
        // Its runtime contract guarantees 1 <= window <= 2, while the static
        // ring has exactly two elements.
        self.current = Some(window_entry);
        self.bindings = base_bindings.clone();
        let window_value = self.new_value(U64)?;
        let zero = self.define(
            U64,
            IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 0 }),
        )?;
        let mut first_issue = self.binding_values(&carried_bindings)?;
        first_issue.extend([lower_capture, upper_capture, zero]);
        self.terminate(IrTerminator::Jump {
            target: issue,
            arguments: first_issue,
            drops: Vec::new(),
        })?;

        let issue_state = issue_parameters
            .get(..carried_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let issue_index = *issue_parameters
            .get(carried_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let issue_upper = *issue_parameters
            .get(carried_count + 1)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let issue_count = *issue_parameters
            .get(carried_count + 2)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.current = Some(issue);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, issue_state)?;
        if self.bindings.insert(binder, issue_index).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        for statement in &direct.prologue {
            let before = self.current;
            self.lower_statements(std::slice::from_ref(*statement), None)?;
            if self.current != before {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        }
        let result = self.expression(direct.scrutinee)?;
        self.note_call_result(direct.scrutinee, result)?;
        let call_is_last = self
            .blocks
            .get(issue.index())
            .and_then(|block| block.instructions.last())
            .is_some_and(|instruction| {
                matches!(
                    instruction,
                    crate::IrInstruction::Define {
                        result: defined,
                        operation: IrOperation::SystemCall { target_action, .. },
                        ..
                    } if *defined == result && target_action.may_suspend()
                )
            });
        if !call_is_last {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let one = self.define(
            U64,
            IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 1 }),
        )?;
        let next_index = self.define(
            U64,
            IrOperation::Integer {
                operation: IrIntegerOperation::AddWrap,
                operand_type: U64,
                arguments: vec![issue_index, one],
            },
        )?;
        let next_count = self.define(
            U64,
            IrOperation::Integer {
                operation: IrIntegerOperation::AddWrap,
                operand_type: U64,
                arguments: vec![issue_count, one],
            },
        )?;
        let has_next = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![next_index, issue_upper],
            },
        )?;
        let has_slot = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![next_count, window_value],
            },
        )?;
        let fill_more = self.define(
            IrType::Bool,
            IrOperation::Boolean {
                operation: IrBooleanOperation::And,
                arguments: vec![has_next, has_slot],
            },
        )?;
        self.terminate(IrTerminator::Match {
            scrutinee: fill_more,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: issue_again_edge,
                },
                IrMatchTarget {
                    tag: 0,
                    block: start_drain_edge,
                },
            ],
        })?;

        self.current = Some(issue_again_edge);
        let mut issue_again = self.binding_values(&carried_bindings)?;
        issue_again.extend([next_index, issue_upper, next_count]);
        self.terminate(IrTerminator::Jump {
            target: issue,
            arguments: issue_again,
            drops: Vec::new(),
        })?;

        self.current = Some(start_drain_edge);
        let mut start_drain = self.binding_values(&carried_bindings)?;
        start_drain.extend([next_index, issue_upper, next_count, zero]);
        self.terminate(IrTerminator::Jump {
            target: drain,
            arguments: start_drain,
            drops: Vec::new(),
        })?;

        let drain_state = drain_parameters
            .get(..carried_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let drain_next_index = *drain_parameters
            .get(carried_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let drain_upper = *drain_parameters
            .get(carried_count + 1)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let drain_count = *drain_parameters
            .get(carried_count + 2)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let drain_slot = *drain_parameters
            .get(carried_count + 3)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.current = Some(drain);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, drain_state)?;
        self.lower_match_from_value(result, direct.enum_type, direct.arms, true, None, None)?;
        if self.current.is_none() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let (drain_again_edge, _) = self.new_block(&[])?;
        let (batch_finished_edge, _) = self.new_block(&[])?;
        let next_slot = self.define(
            U64,
            IrOperation::Integer {
                operation: IrIntegerOperation::AddWrap,
                operand_type: U64,
                arguments: vec![drain_slot, one],
            },
        )?;
        let has_pending_slot = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![next_slot, drain_count],
            },
        )?;
        self.terminate(IrTerminator::Match {
            scrutinee: has_pending_slot,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: drain_again_edge,
                },
                IrMatchTarget {
                    tag: 0,
                    block: batch_finished_edge,
                },
            ],
        })?;

        self.current = Some(drain_again_edge);
        let mut drain_again = self.binding_values(&carried_bindings)?;
        drain_again.extend([drain_next_index, drain_upper, drain_count, next_slot]);
        self.terminate(IrTerminator::Jump {
            target: drain,
            arguments: drain_again,
            drops: Vec::new(),
        })?;

        self.current = Some(batch_finished_edge);
        let more_batches = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![drain_next_index, drain_upper],
            },
        )?;
        let (next_batch_edge, _) = self.new_block(&[])?;
        let (finished_edge, _) = self.new_block(&[])?;
        self.terminate(IrTerminator::Match {
            scrutinee: more_batches,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: next_batch_edge,
                },
                IrMatchTarget {
                    tag: 0,
                    block: finished_edge,
                },
            ],
        })?;

        self.current = Some(next_batch_edge);
        let batch_zero = self.define(
            U64,
            IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 0 }),
        )?;
        let mut next_batch = self.binding_values(&carried_bindings)?;
        next_batch.extend([drain_next_index, drain_upper, batch_zero]);
        self.terminate(IrTerminator::Jump {
            target: issue,
            arguments: next_batch,
            drops: Vec::new(),
        })?;

        self.current = Some(finished_edge);
        let finished = self.binding_values(&carried_bindings)?;
        self.terminate(IrTerminator::Jump {
            target: exit,
            arguments: finished,
            drops: Vec::new(),
        })?;

        self.current = Some(exit);
        self.bindings = base_bindings;
        self.bind_parameters(&carried_bindings, &exit_parameters)?;

        let mut pipeline = IrCompletionPipeline::pending(
            id,
            window_entry,
            IrCompletionWindow::new(counted_span(lower, upper), 0, 2),
        );
        pipeline.plan_bounded_batch(
            vec![issue, issue_again_edge, start_drain_edge],
            2,
            vec![(issue, issue_count), (drain, drain_slot)],
            window_value,
            issue,
            drain,
            result,
        );
        self.completion_pipeline = Some(pipeline);
        self.staged_cut = Some(cut);

        Ok(true)
    }

    /// The compiler-owned [FN-1] counted-range block graph, without
    /// desugaring it into source operations.
    ///
    /// The header owns the true/false guard, normal body fallthrough runs its
    /// drops before reaching the update block, and source exits target the
    /// continuation directly without an update.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn counted_range_graph(
        &mut self,
        id: CheckedLoopId,
        binder: BindingId,
        body: &[CheckedStatement],
        backedge_drops: &[CheckedDrop],
        give_target: Option<GiveTarget>,
        lower_capture: IrValueId,
        upper_capture: IrValueId,
    ) -> Result<(), LoweringFailure> {
        let base_bindings = self.bindings.clone();
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let mut base_parameter_types = carried_bindings
            .iter()
            .map(|binding| {
                base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
                    .and_then(|value| self.value_type(value))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // The binder may need stable storage when the checked body contains a
        // permitted body-local shared borrow. Allocate that storage once,
        // before the first header, and carry the address rather than creating
        // a new address each iteration.
        if self.bindings.insert(binder, lower_capture).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.promote_binding_if_needed(binder)?;
        let binder_storage = self
            .bindings
            .get(&binder)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let binder_storage_type = self.value_type(binder_storage)?;

        let base_count = base_parameter_types.len();
        base_parameter_types.extend([U64, U64, binder_storage_type]);
        let (header, header_parameters) = self.new_block(&base_parameter_types)?;
        let (body_block, _) = self.new_block(&[])?;
        let (exhaustion, _) = self.new_block(&[])?;
        let exit_types = &base_parameter_types[..base_count];
        let (exit, exit_parameters) = self.new_block(exit_types)?;

        let mut entry_arguments = self.binding_values(&carried_bindings)?;
        // `binding_values` above includes the newly inserted binder only when
        // it is listed; the carried list was frozen from the outer scope, so
        // append the three counted identities explicitly.
        entry_arguments.extend([lower_capture, upper_capture, binder_storage]);
        self.terminate(IrTerminator::Jump {
            target: header,
            arguments: entry_arguments,
            drops: Vec::new(),
        })?;

        let header_base = header_parameters
            .get(..base_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let header_lower = *header_parameters
            .get(base_count)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let header_upper = *header_parameters
            .get(base_count + 1)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let header_binder = *header_parameters
            .get(base_count + 2)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;

        self.current = Some(header);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, header_base)?;
        if self.bindings.insert(binder, header_binder).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let binder_value = self.binding_value(binder)?;
        let guard = self.define(
            IrType::Bool,
            IrOperation::Integer {
                operation: IrIntegerOperation::Less,
                operand_type: U64,
                arguments: vec![binder_value, header_upper],
            },
        )?;
        self.terminate(IrTerminator::Match {
            scrutinee: guard,
            enum_type: IrEnumType::Bool,
            targets: vec![
                IrMatchTarget {
                    tag: 1,
                    block: body_block,
                },
                IrMatchTarget {
                    tag: 0,
                    block: exhaustion,
                },
            ],
        })?;

        // The false header edge never enters the body. It therefore performs
        // no body cleanup and drops the binder/captures from the continuation
        // interface by forwarding only the incoming bindings.
        self.current = Some(exhaustion);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, header_base)?;
        let exhaustion_arguments = self.binding_values(&carried_bindings)?;
        self.terminate(IrTerminator::Jump {
            target: exit,
            arguments: exhaustion_arguments,
            drops: Vec::new(),
        })?;

        // The true edge enters with the same header identities. A break uses
        // the exit interface directly, so it cannot execute the update.
        self.current = Some(body_block);
        self.bindings = base_bindings.clone();
        self.bind_parameters(&carried_bindings, header_base)?;
        if self.bindings.insert(binder, header_binder).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.loops.push(LoopTarget {
            id,
            block: exit,
            carried_bindings: carried_bindings.clone(),
        });
        self.lower_statements(body, give_target)?;
        let update = if self.current.is_some() {
            let (update, parameters) = self.new_block(&base_parameter_types)?;
            let mut arguments = self.binding_values(&carried_bindings)?;
            arguments.extend([header_lower, header_upper, header_binder]);
            let drops = self.lower_drops(backedge_drops)?;
            self.terminate(IrTerminator::Jump {
                target: update,
                arguments,
                drops,
            })?;
            Some((update, parameters))
        } else {
            None
        };
        let Some(target) = self.loops.pop() else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        if target.id != id || target.block != exit {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }

        if let Some((update, update_parameters)) = update {
            // Cleanup has completed on the edge into this block. The hidden
            // update is exact because the true guard established binder <
            // upper; AddWrap is the target-independent, total modular
            // operation. An all-terminating body creates no update block at all.
            let update_base = update_parameters
                .get(..base_count)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            let update_lower = *update_parameters
                .get(base_count)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            let update_upper = *update_parameters
                .get(base_count + 1)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            let update_binder = *update_parameters
                .get(base_count + 2)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            self.current = Some(update);
            self.bindings = base_bindings.clone();
            self.bind_parameters(&carried_bindings, update_base)?;
            if self.bindings.insert(binder, update_binder).is_some() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            let old = self.binding_value(binder)?;
            let one = self.define(
                U64,
                IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 1 }),
            )?;
            let next = self.define(
                U64,
                IrOperation::Integer {
                    operation: IrIntegerOperation::AddWrap,
                    operand_type: U64,
                    arguments: vec![old, one],
                },
            )?;
            let next_storage = match binder_storage_type {
                IrType::Address(referent) => {
                    self.store_addressed(update_binder, next, referent)?;
                    update_binder
                }
                _ => {
                    if self.bindings.insert(binder, next) != Some(update_binder) {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    next
                }
            };
            let mut backedge_arguments = self.binding_values(&carried_bindings)?;
            backedge_arguments.extend([update_lower, update_upper, next_storage]);
            self.terminate(IrTerminator::Jump {
                target: header,
                arguments: backedge_arguments,
                drops: Vec::new(),
            })?;
        }

        self.current = Some(exit);
        self.bindings = base_bindings;
        self.bind_parameters(&carried_bindings, &exit_parameters)
    }

    pub(super) fn bind_parameters(
        &mut self,
        bindings: &[BindingId],
        parameters: &[IrValueId],
    ) -> Result<(), LoweringFailure> {
        if bindings.len() != parameters.len() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        for (binding, value) in bindings.iter().zip(parameters) {
            if self.bindings.insert(*binding, *value).is_none() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        }
        Ok(())
    }
}

struct DirectStagedMatch<'body> {
    prologue: Vec<&'body CheckedStatement>,
    scrutinee: &'body CheckedExpression,
    enum_type: crate::semantic::CheckedEnumType,
    arms: &'body [CheckedMatchArm],
}

/// Recognizes the target-independent topology the bounded-batch driver owns:
/// one straight-line prologue followed by the selected, continuing result
/// dispatch. This is an optimization eligibility check only; returning `None`
/// keeps the ordinary accepted program and its one-slot completion schedule.
fn direct_staged_match<'body>(
    body: &'body [CheckedStatement],
    cut: &NodePath,
) -> Option<DirectStagedMatch<'body>> {
    let mut prologue = Vec::new();
    let (scrutinee, enum_type, arms) = direct_staged_tail(body, cut, &mut prologue)?;
    Some(DirectStagedMatch {
        prologue,
        scrutinee,
        enum_type,
        arms,
    })
}

fn direct_staged_tail<'body>(
    body: &'body [CheckedStatement],
    cut: &NodePath,
    prologue: &mut Vec<&'body CheckedStatement>,
) -> Option<(
    &'body CheckedExpression,
    crate::semantic::CheckedEnumType,
    &'body [CheckedMatchArm],
)> {
    let (last, prefix) = body.split_last()?;
    prologue.extend(prefix);
    if let CheckedStatement::Region {
        arena_list: None,
        body,
        fallthrough_drops,
    } = last
        && fallthrough_drops.is_empty()
    {
        return direct_staged_tail(body, cut, prologue);
    }
    let CheckedStatement::Match {
        scrutinee,
        enum_type,
        arms,
        continues: true,
    } = last
    else {
        return None;
    };
    let CheckedExpression::SystemCall {
        call,
        target_action,
        ..
    } = scrutinee
    else {
        return None;
    };
    if call != cut || !target_action.may_suspend() {
        return None;
    }
    Some((scrutinee, *enum_type, arms))
}

fn expression_uses_any(expression: &CheckedExpression, bindings: &HashSet<BindingId>) -> bool {
    let mut found = false;
    crate::semantic::permission::visit_read_bindings(expression, &mut |binding| {
        found |= bindings.contains(&binding);
    });
    found
}

fn set_target_uses_any(target: &CheckedSetTarget, bindings: &HashSet<BindingId>) -> bool {
    if bindings.contains(&target.binding()) {
        return true;
    }
    match target {
        CheckedSetTarget::Place(_) => false,
        CheckedSetTarget::ArrayIndex(target) => expression_uses_any(&target.offset, bindings),
        CheckedSetTarget::BufferIndex(target) => expression_uses_any(&target.offset, bindings),
    }
}

fn drops_use_any(drops: &[CheckedDrop], bindings: &HashSet<BindingId>) -> bool {
    drops.iter().any(|drop| bindings.contains(&drop.binding))
}

fn statement_uses_any(statement: &CheckedStatement, bindings: &HashSet<BindingId>) -> bool {
    match statement {
        CheckedStatement::Let { value, .. }
        | CheckedStatement::DestructuringLet { value, .. }
        | CheckedStatement::Evaluate(value)
        | CheckedStatement::Dispose { value, .. }
        | CheckedStatement::DropExpression { value, .. } => expression_uses_any(value, bindings),
        CheckedStatement::PropagateLet {
            scrutinee,
            error_drops,
            ..
        } => expression_uses_any(scrutinee, bindings) || drops_use_any(error_drops, bindings),
        CheckedStatement::Set { target, value, .. }
        | CheckedStatement::Replace { target, value, .. } => {
            set_target_uses_any(target, bindings) || expression_uses_any(value, bindings)
        }
        CheckedStatement::SetList {
            targets, values, ..
        } => {
            targets
                .iter()
                .any(|target| set_target_uses_any(target, bindings))
                || values
                    .expressions()
                    .iter()
                    .any(|value| expression_uses_any(value, bindings))
        }
        CheckedStatement::Proof(_) => false,
        CheckedStatement::Return { value, drops, .. }
        | CheckedStatement::Give { value, drops, .. } => {
            expression_uses_any(value, bindings) || drops_use_any(drops, bindings)
        }
        CheckedStatement::Match {
            scrutinee, arms, ..
        }
        | CheckedStatement::ValueMatchLet {
            scrutinee, arms, ..
        } => {
            expression_uses_any(scrutinee, bindings)
                || arms.iter().any(|arm| {
                    drops_use_any(&arm.fallthrough_drops, bindings)
                        || arm
                            .body
                            .iter()
                            .any(|statement| statement_uses_any(statement, bindings))
                })
        }
        CheckedStatement::Loop {
            body,
            backedge_drops,
            ..
        } => {
            drops_use_any(backedge_drops, bindings)
                || body
                    .iter()
                    .any(|statement| statement_uses_any(statement, bindings))
        }
        CheckedStatement::CountedRange {
            lower,
            upper,
            body,
            backedge_drops,
            ..
        } => {
            expression_uses_any(lower, bindings)
                || expression_uses_any(upper, bindings)
                || drops_use_any(backedge_drops, bindings)
                || body
                    .iter()
                    .any(|statement| statement_uses_any(statement, bindings))
        }
        CheckedStatement::Break { drops, .. } => drops_use_any(drops, bindings),
        CheckedStatement::Region {
            body,
            fallthrough_drops,
            ..
        } => {
            drops_use_any(fallthrough_drops, bindings)
                || body
                    .iter()
                    .any(|statement| statement_uses_any(statement, bindings))
        }
    }
}

/// The exact static trip count where both endpoints are written constants.
///
/// A zero tells the window query that the source supplies no useful upper
/// bound. The first production driver still has a compiler ceiling of one, so
/// an unknown or empty span cannot enlarge its queue.
fn counted_span(lower: &CheckedExpression, upper: &CheckedExpression) -> u64 {
    exact_counted_span(lower, upper).unwrap_or(0)
}

fn exact_counted_span(lower: &CheckedExpression, upper: &CheckedExpression) -> Option<u64> {
    integer_literal(upper)?.checked_sub(integer_literal(lower)?)
}

fn integer_literal(expression: &CheckedExpression) -> Option<u64> {
    match expression {
        CheckedExpression::Constant(crate::semantic::CheckedValue::Integer { bits, .. })
        | CheckedExpression::NamedConstant {
            value: crate::semantic::CheckedValue::Integer { bits, .. },
            ..
        } => Some(*bits),
        _ => None,
    }
}
