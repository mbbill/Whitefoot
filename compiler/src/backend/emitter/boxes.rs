use std::fmt::Write;

use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_box_new(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Nominal(nominal) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Box { referent } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.value_type(value) != Some(*referent) {
            return Err(BackendFailure::InvalidIr);
        }
        let referent_type = llvm_type(self.program, *referent)?;
        let nonnull = self.next_temporary()?;
        let oom = format!("box.new.oom.v{}", result.ordinal());
        // The shared label helper keeps this block split visible to
        // `block_exit_label`, so a phi in a successor names the right
        // predecessor.
        let ready = box_new_ready_label(result);
        writeln!(
            self.output,
            "  {} = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({referent_type}, ptr null, i64 1) to i64))\n  %{nonnull} = icmp ne ptr {}, null\n  br i1 %{nonnull}, label %{ready}, label %{oom}\n{oom}:\n  call void @wf_resource_abort()\n  unreachable\n{ready}:\n  store {referent_type} {}, ptr {}",
            self.value_name(result),
            self.value_name(result),
            self.value_name(value),
            self.value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_box_deref(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrNominalKind::Box { referent } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        if ty != *referent || self.value_type(value) != Some(IrType::Nominal(nominal)) {
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
