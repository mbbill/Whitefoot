use std::fmt::Write;

use super::*;

/// The one module-level release helper the region allocation list uses: it
/// walks the list, frees every registered allocation, and leaves the cell
/// empty [STOR-3]. Emitted once per module that carries an allocation list.
pub(super) const ARENA_RELEASE_HELPER: &str = "\
define private void @wf_arena_release(ptr %cell) {
entry:
  %first = load ptr, ptr %cell
  store ptr null, ptr %cell
  br label %walk
walk:
  %node = phi ptr [ %first, %entry ], [ %next, %release ]
  %done = icmp eq ptr %node, null
  br i1 %done, label %exit, label %release
release:
  %next = load ptr, ptr %node
  call void @free(ptr %node)
  br label %walk
exit:
  ret void
}

";

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    /// One region block's allocation-list cell [STOR-3]: a stack cell whose
    /// address is the operation's value, reset to empty on every region
    /// entry so a re-entered region starts with no registered allocation.
    pub(super) fn emit_arena_list_new(
        &mut self,
        result: IrValueId,
        ty: IrType,
    ) -> Result<(), BackendFailure> {
        let IrType::Nominal(nominal) = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        if !matches!(self.nominal(nominal)?.kind(), IrNominalKind::ArenaStorage) {
            return Err(BackendFailure::InvalidIr);
        }
        let name = self.value_name(result);
        self.declare_entry_slot(&name, "ptr")?;
        writeln!(self.output, "  store ptr null, ptr {name}")
            .map_err(|_| BackendFailure::TextEmission)
    }

    /// One `arena_new` allocation [STOR-2]: one heap node `{ next, content }`
    /// pushed onto the owning region's list, so the region's exit release
    /// frees it [STOR-3, STOR-4]. The operation's value is the content
    /// address; a hosted allocation failure aborts like a box allocation.
    pub(super) fn emit_arena_new(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        list: IrValueId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Nominal(nominal) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Arena { content } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.value_type(value) != Some(*content) {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(IrType::Nominal(list_nominal)) = self.value_type(list) else {
            return Err(BackendFailure::InvalidIr);
        };
        if !matches!(
            self.nominal(list_nominal)?.kind(),
            IrNominalKind::ArenaStorage
        ) {
            return Err(BackendFailure::InvalidIr);
        }
        let content_type = llvm_type(self.program, *content)?;
        let node_type = format!("{{ ptr, {content_type} }}");
        let node = self.next_temporary()?;
        let nonnull = self.next_temporary()?;
        let head = self.next_temporary()?;
        let oom = format!("arena.new.oom.v{}", result.ordinal());
        // The shared label helper keeps this block split visible to
        // `block_exit_label`, so a phi in a successor names the right
        // predecessor.
        let ready = arena_new_ready_label(result);
        writeln!(
            self.output,
            "  %{node} = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({node_type}, ptr null, i64 1) to i64))\n  %{nonnull} = icmp ne ptr %{node}, null\n  br i1 %{nonnull}, label %{ready}, label %{oom}\n{oom}:\n  call void @wf_resource_abort()\n  unreachable\n{ready}:\n  %{head} = load ptr, ptr {list}\n  store ptr %{head}, ptr %{node}\n  store ptr %{node}, ptr {list}\n  {result} = getelementptr inbounds {node_type}, ptr %{node}, i64 0, i32 1\n  store {content_type} {value}, ptr {result}",
            list = self.value_name(list),
            result = self.value_name(result),
            value = self.value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Arena content read through explicit `deref` [STOR-2]: one load from
    /// the content address the allocation produced.
    pub(super) fn emit_arena_deref(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrNominalKind::Arena { content } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        if ty != *content || self.value_type(value) != Some(IrType::Nominal(nominal)) {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(
            self.output,
            "  {} = load {}, ptr {}",
            self.value_name(result),
            llvm_type(self.program, ty)?,
            self.value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}
