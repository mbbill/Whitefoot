use std::collections::HashSet;

use crate::semantic::{
    BindingId, CheckedDrop, CheckedEnumType, CheckedExpression, CheckedLoopId, CheckedMatchArm,
    CheckedMode, CheckedSetTarget, CheckedStatement,
};
use crate::{
    IrBlockId, IrBooleanOperation, IrCompletionPipeline, IrCompletionWindow, IrConstant,
    IrEnumType, IrIntegerOperation, IrMatchTarget, IrOperation, IrTerminator, IrType, IrValueId,
    LoweringFailure, NodePath, SystemRelease, SystemReleaseAction,
};

use super::{GiveTarget, IrBuilder};
use crate::lowering::lower_type;

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
        let Some(direct) = direct_staged_match(body, &cut, id) else {
            return Ok(false);
        };
        if give_target.is_some()
            || !backedge_drops.is_empty()
            || self.addressed_bindings.contains(&binder)
        {
            return Ok(false);
        }

        let mut unavailable_in_remainder = HashSet::from([binder]);
        for item in &direct.prologue {
            match item {
                PrologueItem::Statement(CheckedStatement::Let { binding, .. })
                | PrologueItem::Statement(CheckedStatement::Replace { binding, .. }) => {
                    unavailable_in_remainder.insert(*binding);
                }
                PrologueItem::Statement(
                    CheckedStatement::Set { .. }
                    | CheckedStatement::Evaluate(_)
                    | CheckedStatement::DropExpression { .. }
                    | CheckedStatement::Proof(_),
                ) => {}
                PrologueItem::Statement(_) => return Ok(false),
                PrologueItem::Gate(gate) => {
                    for arm in gate.arms {
                        for binder in &arm.binders {
                            unavailable_in_remainder.insert(binder.binding);
                        }
                    }
                }
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
        let gates = direct
            .prologue
            .iter()
            .filter_map(|item| match item {
                PrologueItem::Gate(gate) => Some(gate),
                PrologueItem::Statement(_) => None,
            })
            .collect::<Vec<_>>();
        let mut gate_plans = Vec::with_capacity(gates.len());
        let mut exit_count = 0_u64;
        for gate in &gates {
            let mut arm_blocks = Vec::with_capacity(gate.arms.len());
            let mut exits = Vec::new();
            for index in 0..gate.arms.len() {
                arm_blocks.push(self.new_block(&[])?.0);
                if index != gate.continuing {
                    exit_count += 1;
                    exits.push((index, exit_count));
                }
            }
            gate_plans.push(GatePlan { arm_blocks, exits });
        }
        let (issue_again_edge, _) = self.new_block(&[])?;
        let (start_drain_edge, _) = self.new_block(&[])?;
        let mut drain_types = carried_types.clone();
        drain_types.extend([U64, U64, U64, U64, U64]); // next index, upper, count, slot, leaving
        let (drain, drain_parameters) = self.new_block(&drain_types)?;
        let (exit, exit_parameters) = self.new_block(&carried_types)?;
        // One block per exiting arm, entered on the carried bindings after the
        // batch in flight has drained (or at once when nothing was in flight).
        let mut exit_targets = Vec::new();
        let mut exit_arms = Vec::new();
        for (gate, plan) in gates.iter().zip(&gate_plans) {
            for (arm_index, leaving) in &plan.exits {
                let (block, parameters) = self.new_block(&carried_types)?;
                exit_targets.push((*leaving, block));
                let arm = gate
                    .arms
                    .get(*arm_index)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                exit_arms.push((arm, block, parameters));
            }
        }

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
        // Defined at the head of the issue stage, which dominates every block
        // the constant reaches: the drain is entered from the issue tail and
        // from a gate's exiting arm alike.
        let one = self.define(
            U64,
            IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 1 }),
        )?;
        let mut gate_cursor = 0;
        let mut submission_blocks = Vec::new();
        let mut pending_exit_edges = Vec::new();
        for item in &direct.prologue {
            match item {
                PrologueItem::Statement(statement) => {
                    let before = self.current;
                    self.lower_statements(std::slice::from_ref(*statement), None)?;
                    if self.current != before {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                }
                PrologueItem::Gate(gate) => {
                    let plan = gate_plans
                        .get(gate_cursor)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    gate_cursor += 1;
                    let edges = self.lower_prologue_gate_staged(
                        gate,
                        plan,
                        &GateContext {
                            issue_index,
                            issue_upper,
                            issue_count,
                            zero,
                            drain,
                            carried_bindings: &carried_bindings,
                            exit_targets: &exit_targets,
                        },
                    )?;
                    pending_exit_edges.extend(edges);
                    submission_blocks
                        .push(self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?);
                }
            }
        }
        let issue_tail = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let result = self.expression(direct.scrutinee)?;
        self.note_call_result(direct.scrutinee, result)?;
        let call_is_last = self
            .blocks
            .get(issue_tail.index())
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
        start_drain.extend([next_index, issue_upper, next_count, zero, zero]);
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
        let drain_leaving = *drain_parameters
            .get(carried_count + 4)
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
        drain_again.extend([
            drain_next_index,
            drain_upper,
            drain_count,
            next_slot,
            drain_leaving,
        ]);
        self.terminate(IrTerminator::Jump {
            target: drain,
            arguments: drain_again,
            drops: Vec::new(),
        })?;

        self.current = Some(batch_finished_edge);
        // A gate exit taken while this batch had operations in flight arrives
        // here with `leaving` naming its arm. The batch is drained, so every
        // earlier iteration's remainder has run, and the exit now leaves in
        // source order. `leaving` is dispatched by an ascending chain of
        // strict comparisons: below one is no exit, below two is arm one, and
        // so on.
        if !exit_targets.is_empty() {
            let one_bound = self.define(
                U64,
                IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 1 }),
            )?;
            let no_exit = self.define(
                IrType::Bool,
                IrOperation::Integer {
                    operation: IrIntegerOperation::Less,
                    operand_type: U64,
                    arguments: vec![drain_leaving, one_bound],
                },
            )?;
            let (normal_edge, _) = self.new_block(&[])?;
            let (mut rest, _) = self.new_block(&[])?;
            self.terminate(IrTerminator::Match {
                scrutinee: no_exit,
                enum_type: IrEnumType::Bool,
                targets: vec![
                    IrMatchTarget {
                        tag: 1,
                        block: normal_edge,
                    },
                    IrMatchTarget {
                        tag: 0,
                        block: rest,
                    },
                ],
            })?;
            for (leaving, target) in &exit_targets {
                self.current = Some(rest);
                let bound = self.define(
                    U64,
                    IrOperation::Constant(IrConstant::Integer {
                        ty: U64,
                        bits: leaving.wrapping_add(1),
                    }),
                )?;
                let is_this = self.define(
                    IrType::Bool,
                    IrOperation::Integer {
                        operation: IrIntegerOperation::Less,
                        operand_type: U64,
                        arguments: vec![drain_leaving, bound],
                    },
                )?;
                let (leave, _) = self.new_block(&[])?;
                let (next_rest, _) = self.new_block(&[])?;
                self.terminate(IrTerminator::Match {
                    scrutinee: is_this,
                    enum_type: IrEnumType::Bool,
                    targets: vec![
                        IrMatchTarget {
                            tag: 1,
                            block: leave,
                        },
                        IrMatchTarget {
                            tag: 0,
                            block: next_rest,
                        },
                    ],
                })?;
                self.current = Some(leave);
                let arguments = self.binding_values(&carried_bindings)?;
                self.terminate(IrTerminator::Jump {
                    target: *target,
                    arguments,
                    drops: Vec::new(),
                })?;
                rest = next_rest;
            }
            self.current = Some(rest);
            self.terminate(IrTerminator::Unreachable)?;
            self.current = Some(normal_edge);
        }
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

        // The exiting arms themselves, on the carried bindings alone (the
        // recognizer refused an arm that reads its own binders). A `break`
        // leaves through this driver's exit block like any other.
        for (arm, block, parameters) in &exit_arms {
            self.current = Some(*block);
            self.bindings = base_bindings.clone();
            self.bind_parameters(&carried_bindings, parameters)?;
            let binders = arm_binders(arm);
            let Some((last, prefix)) = arm.body.split_last() else {
                return Err(LoweringFailure::InvalidCheckedProgram);
            };
            let before = self.current;
            self.lower_statements(prefix, None)?;
            if self.current != before {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            // The arm's own binders were released in the issue stage; the
            // remaining releases and the exit itself run here, after the
            // drain, in source order.
            match last {
                CheckedStatement::Return { value, drops, .. } => {
                    let value = self.expression(value)?;
                    let remaining = drops
                        .iter()
                        .filter(|drop| !binders.contains(&drop.binding))
                        .cloned()
                        .collect::<Vec<_>>();
                    let drops = self.lower_drops(&remaining)?;
                    self.terminate(IrTerminator::Return { value, drops })?;
                }
                CheckedStatement::Break { target, drops } if *target == id => {
                    let remaining = drops
                        .iter()
                        .filter(|drop| !binders.contains(&drop.binding))
                        .cloned()
                        .collect::<Vec<_>>();
                    let arguments = self.binding_values(&carried_bindings)?;
                    let drops = self.lower_drops(&remaining)?;
                    self.terminate(IrTerminator::Jump {
                        target: exit,
                        arguments,
                        drops,
                    })?;
                }
                _ => return Err(LoweringFailure::InvalidCheckedProgram),
            }
        }

        self.current = Some(exit);
        self.bindings = base_bindings;
        self.bind_parameters(&carried_bindings, &exit_parameters)?;

        let mut carrying = vec![issue, issue_again_edge, start_drain_edge];
        let mut slot_index = vec![(issue, issue_count)];
        for plan in &gate_plans {
            carrying.extend(plan.arm_blocks.iter().copied());
        }
        for block in submission_blocks {
            slot_index.push((block, issue_count));
        }
        slot_index.push((drain, drain_slot));
        let mut pipeline = IrCompletionPipeline::pending(
            id,
            window_entry,
            IrCompletionWindow::new(counted_span(lower, upper), 0, 2),
        );
        pipeline.plan_bounded_batch(
            carrying,
            2,
            slot_index,
            window_value,
            issue,
            drain,
            result,
            pending_exit_edges,
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

/// One gate in the prologue: a `match` on a `never-suspends` operation whose
/// one continuing arm holds the staged submission and whose every other arm
/// leaves the loop before any submission, which is an exit written in the
/// prologue and admitted by [PAR-3]'s second condition. `reserve_handle` is the
/// instance the backed permit introduces [SYS-10]: its `Err` arm exits, its
/// `Ok` arm carries the permit into the open.
struct PrologueGate<'body> {
    scrutinee: &'body CheckedExpression,
    enum_type: crate::semantic::CheckedEnumType,
    arms: &'body [CheckedMatchArm],
    continuing: usize,
}

enum PrologueItem<'body> {
    Statement(&'body CheckedStatement),
    Gate(PrologueGate<'body>),
}

/// The blocks one gate dispatches into, allocated before the drain so that
/// the block holding the submission precedes the drain in emission order,
/// which is the order the emitter retires a driven result in. Each exiting
/// arm is numbered from one; zero means no exit is pending.
struct GatePlan {
    arm_blocks: Vec<IrBlockId>,
    exits: Vec<(usize, u64)>,
}

/// What an exiting arm needs from the issue stage it leaves.
struct GateContext<'a> {
    issue_index: IrValueId,
    issue_upper: IrValueId,
    issue_count: IrValueId,
    zero: IrValueId,
    drain: IrBlockId,
    carried_bindings: &'a [BindingId],
    exit_targets: &'a [(u64, IrBlockId)],
}

struct DirectStagedMatch<'body> {
    prologue: Vec<PrologueItem<'body>>,
    scrutinee: &'body CheckedExpression,
    enum_type: crate::semantic::CheckedEnumType,
    arms: &'body [CheckedMatchArm],
}

/// Recognizes the target-independent topology the bounded-batch driver owns:
/// a prologue of straight-line statements and exiting gates, followed by the
/// selected, continuing result dispatch. This is an optimization eligibility
/// check only; returning `None` keeps the ordinary accepted program and its
/// one-slot completion schedule.
fn direct_staged_match<'body>(
    body: &'body [CheckedStatement],
    cut: &NodePath,
    loop_id: CheckedLoopId,
) -> Option<DirectStagedMatch<'body>> {
    let mut prologue = Vec::new();
    let (scrutinee, enum_type, arms) = direct_staged_tail(body, cut, loop_id, &mut prologue)?;
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
    loop_id: CheckedLoopId,
    prologue: &mut Vec<PrologueItem<'body>>,
) -> Option<(
    &'body CheckedExpression,
    crate::semantic::CheckedEnumType,
    &'body [CheckedMatchArm],
)> {
    let (last, prefix) = body.split_last()?;
    prologue.extend(prefix.iter().map(PrologueItem::Statement));
    if let CheckedStatement::Region {
        arena_list: None,
        body,
        fallthrough_drops,
    } = last
        && fallthrough_drops.is_empty()
    {
        return direct_staged_tail(body, cut, loop_id, prologue);
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
    if call == cut && target_action.may_suspend() {
        return Some((scrutinee, *enum_type, arms));
    }
    if target_action.may_suspend() {
        return None;
    }
    // A gate: exactly one arm continues into the cut, and every other arm's
    // last statement leaves the loop, so no submission of this iteration has
    // happened when the exit is taken.
    let mut continuing = None;
    for (index, arm) in arms.iter().enumerate() {
        let mut inner = Vec::new();
        if let Some(tail) = direct_staged_tail(&arm.body, cut, loop_id, &mut inner) {
            if continuing.is_some() || !arm.fallthrough_drops.is_empty() {
                return None;
            }
            continuing = Some((index, inner, tail));
            continue;
        }
        if !exit_arm_leaves_on_carried_bindings(arm, loop_id) {
            return None;
        }
    }
    let (continuing, inner, tail) = continuing?;
    prologue.push(PrologueItem::Gate(PrologueGate {
        scrutinee,
        enum_type: *enum_type,
        arms,
        continuing,
    }));
    prologue.extend(inner);
    Some(tail)
}

impl IrBuilder<'_> {
    /// Lowers one prologue gate inside the issue stage: the `never-suspends`
    /// scrutinee, the dispatch into the pre-allocated arm blocks, the
    /// continuing arm's binders into the block the issue stage continues on,
    /// and for each exiting arm the choice between leaving at once (nothing of
    /// this batch is in flight) and draining the batch first with `leaving`
    /// naming the arm.
    fn lower_prologue_gate_staged(
        &mut self,
        gate: &PrologueGate<'_>,
        plan: &GatePlan,
        context: &GateContext<'_>,
    ) -> Result<Vec<IrBlockId>, LoweringFailure> {
        let mut pending_exit_edges = Vec::new();
        let before = self.current;
        let scrutinee = self.expression(gate.scrutinee)?;
        if self.current != before {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let CheckedEnumType::Nominal(nominal) = gate.enum_type else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let base_bindings = self.bindings.clone();
        self.terminate(IrTerminator::Match {
            scrutinee,
            enum_type: gate.enum_type.into(),
            targets: gate
                .arms
                .iter()
                .zip(&plan.arm_blocks)
                .map(|(arm, block)| IrMatchTarget {
                    tag: arm.tag,
                    block: *block,
                })
                .collect(),
        })?;
        let mut continuing = None;
        for (index, (arm, block)) in gate.arms.iter().zip(&plan.arm_blocks).enumerate() {
            self.current = Some(*block);
            self.bindings = base_bindings.clone();
            for binder in &arm.binders {
                let value = self.define(
                    lower_type(binder.ty)?,
                    IrOperation::ProjectVariant {
                        aggregate: scrutinee,
                        nominal: crate::lowering::IrNominalId(nominal.0),
                        variant: arm.tag,
                        field: binder.field,
                    },
                )?;
                if self.bindings.insert(binder.binding, value).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if binder.mode == CheckedMode::Own {
                    self.promote_binding_if_needed(binder.binding)?;
                }
            }
            if index == gate.continuing {
                continuing = Some((*block, self.bindings.clone()));
                continue;
            }
            // The exit's releases of this arm's own binders run here, where the
            // values are; the recognizer admitted only releases that emit
            // nothing, so running them before the drain moves nothing
            // observable.
            let binders = arm_binders(arm);
            let binder_drops = match arm.body.last() {
                Some(CheckedStatement::Return { drops, .. })
                | Some(CheckedStatement::Break { drops, .. }) => drops
                    .iter()
                    .filter(|drop| binders.contains(&drop.binding))
                    .cloned()
                    .collect::<Vec<_>>(),
                _ => return Err(LoweringFailure::InvalidCheckedProgram),
            };
            let bindings_with_binders = self.bindings.clone();
            let leaving = plan
                .exits
                .iter()
                .find(|(arm_index, _)| *arm_index == index)
                .map(|(_, leaving)| *leaving)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            let target = context
                .exit_targets
                .iter()
                .find(|(candidate, _)| *candidate == leaving)
                .map(|(_, block)| *block)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            let one = self.define(
                U64,
                IrOperation::Constant(IrConstant::Integer { ty: U64, bits: 1 }),
            )?;
            let nothing_pending = self.define(
                IrType::Bool,
                IrOperation::Integer {
                    operation: IrIntegerOperation::Less,
                    operand_type: U64,
                    arguments: vec![context.issue_count, one],
                },
            )?;
            let (leave_now, _) = self.new_block(&[])?;
            let (drain_first, _) = self.new_block(&[])?;
            pending_exit_edges.push(drain_first);
            self.terminate(IrTerminator::Match {
                scrutinee: nothing_pending,
                enum_type: IrEnumType::Bool,
                targets: vec![
                    IrMatchTarget {
                        tag: 1,
                        block: leave_now,
                    },
                    IrMatchTarget {
                        tag: 0,
                        block: drain_first,
                    },
                ],
            })?;
            self.current = Some(leave_now);
            self.bindings = bindings_with_binders.clone();
            let arguments = self.binding_values(context.carried_bindings)?;
            let drops = self.lower_drops(&binder_drops)?;
            self.terminate(IrTerminator::Jump {
                target,
                arguments,
                drops,
            })?;
            self.current = Some(drain_first);
            self.bindings = bindings_with_binders;
            let drops = self.lower_drops(&binder_drops)?;
            let leaving_value = self.define(
                U64,
                IrOperation::Constant(IrConstant::Integer {
                    ty: U64,
                    bits: leaving,
                }),
            )?;
            let mut arguments = self.binding_values(context.carried_bindings)?;
            arguments.extend([
                context.issue_index,
                context.issue_upper,
                context.issue_count,
                context.zero,
                leaving_value,
            ]);
            self.terminate(IrTerminator::Jump {
                target: context.drain,
                arguments,
                drops,
            })?;
        }
        let (block, bindings) = continuing.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.current = Some(block);
        self.bindings = bindings;
        Ok(pending_exit_edges)
    }
}

/// The binders one gate arm introduces.
fn arm_binders(arm: &CheckedMatchArm) -> HashSet<BindingId> {
    arm.binders.iter().map(|binder| binder.binding).collect()
}

/// A release that performs no system action: nothing observable moves when it
/// runs early, so an exiting arm's release of its own binders can run in the
/// issue stage while the arm's exit itself waits for the batch to drain.
fn release_emits_nothing(release: &SystemRelease) -> bool {
    !matches!(
        release.action,
        Some(SystemReleaseAction::NativeCloseAttempt)
    )
}

/// Whether an exiting gate arm can be lowered as the driver requires: its
/// last statement leaves the loop, everything before it and the exit's own
/// value read only carried bindings, and the arm's binders appear only in
/// that last statement's releases, each of which emits nothing.
fn exit_arm_leaves_on_carried_bindings(arm: &CheckedMatchArm, loop_id: CheckedLoopId) -> bool {
    let binders = arm_binders(arm);
    let Some((last, prefix)) = arm.body.split_last() else {
        return false;
    };
    if prefix
        .iter()
        .any(|statement| statement_uses_any(statement, &binders))
    {
        return false;
    }
    let (drops, value_uses_binders, leaves) = match last {
        CheckedStatement::Return { value, drops, .. } => {
            (drops, expression_uses_any(value, &binders), true)
        }
        CheckedStatement::Break { target, drops } => (drops, false, *target == loop_id),
        _ => return false,
    };
    leaves
        && !value_uses_binders
        && drops
            .iter()
            .filter(|drop| binders.contains(&drop.binding))
            .all(|drop| release_emits_nothing(&drop.release))
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
        | CheckedStatement::Evaluate(value)
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
