//! Shared CFG facts for loop cost estimation and cooperative checkpoints.
//! These inspect checked IR and do not participate in source acceptance.

use super::{IrBlock, IrTerminator};

/// Sources of natural-loop jump backedges. A destination must dominate the
/// source; block ordering alone mistakes a loop break for a backedge because
/// the builder creates the exit block before the body.
pub(crate) fn backedge_sources(blocks: &[IrBlock]) -> Vec<usize> {
    blocks
        .iter()
        .enumerate()
        .filter_map(|(index, block)| {
            let IrTerminator::Jump { target, .. } = block.terminator() else {
                return None;
            };
            (!reachable_without(blocks, target.index(), index)).then_some(index)
        })
        .collect()
}

/// Whether `goal` is reachable from entry without visiting `removed`.
fn reachable_without(blocks: &[IrBlock], removed: usize, goal: usize) -> bool {
    if removed == 0 || goal == removed {
        return false;
    }
    let mut seen = vec![false; blocks.len()];
    let mut pending = vec![0_usize];
    if let Some(first) = seen.first_mut() {
        *first = true;
    }
    while let Some(index) = pending.pop() {
        if index == goal {
            return true;
        }
        let Some(block) = blocks.get(index) else {
            continue;
        };
        let successors: Vec<usize> = match block.terminator() {
            IrTerminator::Jump { target, .. } => vec![target.index()],
            IrTerminator::Match { targets, .. } => targets
                .iter()
                .map(|target| target.block().index())
                .collect(),
            IrTerminator::Return { .. } | IrTerminator::Unreachable => Vec::new(),
        };
        for successor in successors {
            if successor != removed
                && let Some(visited) = seen.get_mut(successor)
                && !*visited
            {
                *visited = true;
                pending.push(successor);
            }
        }
    }
    false
}
