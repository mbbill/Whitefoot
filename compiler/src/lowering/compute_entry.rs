//! A bounded, side-effect-free entry probe for experimental compute offload.
//!
//! The probe repeats an unchanged scalar prefix of an already checked call.
//! It answers true only on a cheap, cleanup-free return. The real call still
//! executes exactly once, with its original arguments, moves, and cleanup.
//! This is scheduling policy, never proof search or a source rejection rule.
//! Owned by io-model experiment 66; supersede with the selected cost policy.

use super::*;
use std::collections::VecDeque;

const PREFIX_COST: usize = 32;

pub(crate) fn probe(function: &IrFunction, ordinal: usize) -> Option<IrFunction> {
    let mut result = function.clone();
    result.name = format!("_compute_probe_{ordinal}");
    result.result = IrType::Bool;
    result.overlaps.clear();
    result.completion_steps.clear();
    result.completion_pipeline = None;
    result.synthesis = None;
    result.compute_weight = 0;
    // Cut natural backedges, then independently require the remaining graph
    // to be acyclic. Block numbering is not a loop test.
    for source in control_flow::backedge_sources(&result.blocks) {
        answer(&mut result, source, false)?;
    }
    let order = acyclic_reachable(&result.blocks)?;
    let mut costs = vec![0_usize; result.blocks.len()];
    for index in order {
        let mut cost = costs[index].saturating_add(1);
        let cut = result.blocks[index]
            .instructions
            .iter()
            .position(|instruction| {
                cost = cost.saturating_add(1);
                cost > PREFIX_COST || !repeatable(instruction)
            });
        if let Some(cut) = cut {
            result.blocks[index].instructions.truncate(cut);
            answer(&mut result, index, false)?;
            continue;
        }
        let terminal = match &result.blocks[index].terminator {
            IrTerminator::Return { drops, .. } => Some(drops.is_empty()),
            IrTerminator::Jump { drops, .. } if !drops.is_empty() => Some(false),
            IrTerminator::Match { enum_type, .. } if *enum_type != IrEnumType::Bool => Some(false),
            IrTerminator::Unreachable => Some(false),
            _ if cost >= PREFIX_COST => Some(false),
            _ => None,
        };
        if let Some(cheap) = terminal {
            // Backedge cuts already return false: do not turn those into cheap
            // source returns merely because their generated cleanup is empty.
            let original_return = matches!(
                function.blocks[index].terminator,
                IrTerminator::Return { .. }
            );
            answer(&mut result, index, cheap && original_return)?;
        } else {
            for next in successors(&result.blocks[index]) {
                costs[next] = costs[next].max(cost);
            }
        }
    }
    // Cutting a prefix removes predecessor edges. Clear unreachable blocks
    // entirely so the ordinary emitter reconstructs only live phi inputs.
    let live = reachable(&result.blocks);
    for (index, block) in result.blocks.iter_mut().enumerate() {
        if !live[index] {
            block.parameters.clear();
            block.instructions.clear();
            block.terminator = IrTerminator::Unreachable;
        }
    }
    Some(result)
}

fn repeatable(instruction: &IrInstruction) -> bool {
    matches!(
        instruction,
        IrInstruction::Define {
            ty: IrType::Unit | IrType::Bool | IrType::Integer { .. },
            operation: IrOperation::Constant(
                IrConstant::Unit | IrConstant::Bool(_) | IrConstant::Integer { .. }
            ) | IrOperation::Integer { .. }
                | IrOperation::Boolean { .. },
            ..
        }
    )
}

fn answer(function: &mut IrFunction, block: usize, cheap: bool) -> Option<()> {
    let value = IrValueId(u32::try_from(function.values.len()).ok()?);
    function.values.push(IrType::Bool);
    function.blocks[block]
        .instructions
        .push(IrInstruction::Define {
            result: value,
            ty: IrType::Bool,
            operation: IrOperation::Constant(IrConstant::Bool(cheap)),
        });
    function.blocks[block].terminator = IrTerminator::Return {
        value,
        drops: Vec::new(),
    };
    Some(())
}

fn successors(block: &IrBlock) -> Vec<usize> {
    match &block.terminator {
        IrTerminator::Jump { target, .. } => vec![target.index()],
        IrTerminator::Match { targets, .. } => {
            targets.iter().map(|target| target.block.index()).collect()
        }
        IrTerminator::Return { .. } | IrTerminator::Unreachable => Vec::new(),
    }
}

fn reachable(blocks: &[IrBlock]) -> Vec<bool> {
    let mut live = vec![false; blocks.len()];
    let mut pending = vec![0];
    while let Some(index) = pending.pop() {
        if index >= blocks.len() || live[index] {
            continue;
        }
        live[index] = true;
        pending.extend(successors(&blocks[index]));
    }
    live
}

fn acyclic_reachable(blocks: &[IrBlock]) -> Option<Vec<usize>> {
    let live = reachable(blocks);
    let mut incoming = vec![0_usize; blocks.len()];
    for (index, block) in blocks.iter().enumerate() {
        if live[index] {
            for next in successors(block) {
                *incoming.get_mut(next)? += 1;
            }
        }
    }
    let mut ready: VecDeque<_> = (0..blocks.len())
        .filter(|&index| live[index] && incoming[index] == 0)
        .collect();
    let mut order = Vec::new();
    while let Some(index) = ready.pop_front() {
        order.push(index);
        for next in successors(&blocks[index]) {
            incoming[next] -= 1;
            if incoming[next] == 0 {
                ready.push_back(next);
            }
        }
    }
    (order.len() == live.iter().filter(|&&live| live).count()).then_some(order)
}
