//! Compute offer selection. Remove only already-permitted offers of
//! bounded straight-line scalar leaves; retain every original call and join
//! boundary for the remaining members. The CLI selects the provisional limit;
//! no source name selects behavior and no acceptance judgment consumes it.

use super::{IrFunction, IrInstruction, IrOperation, IrTerminator, IrType};

fn scalar(ty: IrType) -> bool {
    matches!(
        ty,
        IrType::Bool | IrType::Integer { .. } | IrType::Float { .. }
    )
}

fn small_leaf(function: &IrFunction, limit: u32) -> bool {
    if !scalar(function.result)
        || !function.parameters.iter().all(|(_, ty)| scalar(*ty))
        || function.blocks.len() != 1
    {
        return false;
    }
    let block = &function.blocks[0];
    if !matches!(&block.terminator, IrTerminator::Return { drops, .. } if drops.is_empty()) {
        return false;
    }
    let mut operations = 0_u64;
    for instruction in &block.instructions {
        let IrInstruction::Define { ty, operation, .. } = instruction else {
            return false;
        };
        if !scalar(*ty) {
            return false;
        }
        match operation {
            IrOperation::Constant(_) => {}
            IrOperation::Integer { .. }
            | IrOperation::Float { .. }
            | IrOperation::Boolean { .. }
            | IrOperation::NumericConversion { .. }
            | IrOperation::Reinterpret { .. } => operations += 1,
            _ => return false,
        }
    }
    operations <= u64::from(limit)
}

pub(super) fn prune(functions: &mut [IrFunction], limit: u32, ledger: &mut Vec<String>) {
    let leaves: Vec<_> = functions
        .iter()
        .map(|function| small_leaf(function, limit))
        .collect();
    for function in functions {
        let mut small_results = std::collections::HashSet::new();
        for block in &function.blocks {
            for instruction in &block.instructions {
                if let IrInstruction::Define {
                    result,
                    operation: IrOperation::Call { function, .. },
                    ..
                } = instruction
                    && leaves.get(*function as usize).copied().unwrap_or(false)
                {
                    small_results.insert(*result);
                }
            }
        }
        let mut removed = 0;
        for overlap in &mut function.overlaps {
            let join = overlap.join_site();
            overlap.members.retain(|member| {
                // Keep the original source-last join site even if its own
                // work is small. An omitted member becomes an ordinary call
                // at its original location; no group gains a new member.
                let keep = Some(*member) == join || !small_results.contains(member);
                removed += usize::from(!keep);
                keep
            });
        }
        function
            .overlaps
            .retain(|overlap| overlap.members.len() >= 2);
        if removed != 0 {
            ledger.push(format!(
                "PAR actualization  {}  scalar leaf limit {limit}: omitted {removed} compute offers",
                function.name
            ));
        }
    }
}
