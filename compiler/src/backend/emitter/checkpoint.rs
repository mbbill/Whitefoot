//! Experimental loop checkpoints without a source suspension annotation.
//! Owned by io-model/SCHEDULER-EXPERIMENT.md; remove or supersede when the
//! measured checkpoint design is selected. This runs after all source proof.

use std::fmt::Write;
use std::num::NonZeroU32;

use super::{BackendFailure, FunctionEmitter, FunctionSlot, IrBlockId};

pub(super) const DECLARATION: &str = "declare void @wf__sched_checkpoint()";

/// LLVM inlines this helper before ordinary optimization, so its private
/// counter can become a loop phi. Keeping its control flow here lets LLVM
/// update every phi predecessor when it splits an emitted backedge.
pub(super) fn helper(interval: NonZeroU32) -> String {
    format!(
        "{DECLARATION}\n\
         define internal void @wf__checkpoint_tick(ptr %counter) alwaysinline {{\n\
         entry:\n  %old = load i32, ptr %counter, align 4\n  \
         %next = sub i32 %old, 1\n  store i32 %next, ptr %counter, align 4\n  \
         %due = icmp eq i32 %next, 0\n  br i1 %due, label %yield, label %done\n\
         yield:\n  call void @wf__sched_checkpoint()\n  \
         store i32 {}, ptr %counter, align 4\n  br label %done\n\
         done:\n  ret void\n}}\n\n",
        interval.get()
    )
}

impl FunctionEmitter<'_, '_> {
    pub(super) fn emit_checkpoint(&mut self, block: IrBlockId) -> Result<(), BackendFailure> {
        if self.checkpoint_edges.contains(&block.index()) {
            let counter = self.frame.slot(FunctionSlot::CheckpointBudget)?;
            writeln!(
                self.output,
                "  call void @wf__checkpoint_tick(ptr {counter})"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        Ok(())
    }
}
