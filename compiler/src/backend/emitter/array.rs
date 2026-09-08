use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_array_fill(
        &mut self,
        result: IrValueId,
        ty: IrType,
        value: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let IrType::Array { element, length } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let element_type = element.ty();
        if self.value_type(value) != Some(element_type) {
            return Err(BackendFailure::InvalidIr);
        }

        let array_type = llvm_type(self.program, ty)?;
        let llvm_element_type = llvm_type(self.program, element_type)?;
        let array_slot = self.value_place(result)?;
        let index_slot = self.entry_slot(FunctionSlot::ArrayFillIndex(result))?;
        let index = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let next_index = self.next_temporary()?;
        let operand = self.value_operand(value)?;

        writeln!(
            self.output,
            "  store i64 0, ptr {index_slot}\n  br label %{}\n{}:\n  %{index} = load i64, ptr {index_slot}\n  %{in_range} = icmp ult i64 %{index}, {length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  %{element_pointer} = getelementptr inbounds {array_type}, ptr {array_slot}, i64 0, i64 %{index}\n  store {llvm_element_type} {operand}, ptr %{element_pointer}\n  %{next_index} = add i64 %{index}, 1\n  store i64 %{next_index}, ptr {index_slot}\n  br label %{}\n{}:",
            array_fill_head_label(result),
            array_fill_head_label(result),
            array_fill_body_label(result),
            array_fill_done_label(result),
            array_fill_body_label(result),
            array_fill_head_label(result),
            array_fill_done_label(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Emits a discharged source subscript read [OP-4]: the checker derived
    /// the bounds obligation, so no compare, branch, or trap is emitted in
    /// any build mode.
    pub(super) fn emit_array_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        root: IrArrayRoot,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        if self.value_type(offset)
            != Some(IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let root_type = match root {
            IrArrayRoot::Value(value) => self
                .function
                .value_type(value)
                .ok_or(BackendFailure::InvalidIr)?,
            IrArrayRoot::Constant(id) => self
                .program
                .constant(id)
                .ok_or(BackendFailure::InvalidIr)?
                .ty(),
        };
        let IrType::Array { element, .. } = root_type else {
            return Err(BackendFailure::InvalidIr);
        };
        if element.ty() != ty {
            return Err(BackendFailure::InvalidIr);
        }

        let array_type = llvm_type(self.program, root_type)?;
        let root_pointer = match root {
            IrArrayRoot::Value(value) => self.value_place(value)?,
            IrArrayRoot::Constant(id) => constant_symbol(id),
        };
        let element_pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{element_pointer} = getelementptr inbounds {array_type}, ptr {root_pointer}, i64 0, i64 {}",
            self.value_name(offset),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.load_place_result(result, ty, &format!("%{element_pointer}"))
    }
}
