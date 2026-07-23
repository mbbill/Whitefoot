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
        if self.function.value_type(value) != Some(element_type) {
            return Err(BackendFailure::InvalidIr);
        }

        let array_type = llvm_type(self.program, ty)?;
        let llvm_element_type = llvm_type(self.program, element_type)?;
        let array_slot = self.next_temporary()?;
        let index_slot = self.next_temporary()?;
        let index = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let next_index = self.next_temporary()?;

        writeln!(
            self.output,
            "  %{array_slot} = alloca {array_type}\n  %{index_slot} = alloca i64\n  store i64 0, ptr %{index_slot}\n  br label %{}\n{}:\n  %{index} = load i64, ptr %{index_slot}\n  %{in_range} = icmp ult i64 %{index}, {length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  %{element_pointer} = getelementptr inbounds {array_type}, ptr %{array_slot}, i64 0, i64 %{index}\n  store {llvm_element_type} {}, ptr %{element_pointer}\n  %{next_index} = add i64 %{index}, 1\n  store i64 %{next_index}, ptr %{index_slot}\n  br label %{}\n{}:\n  {} = load {array_type}, ptr %{array_slot}",
            array_fill_head_label(result),
            array_fill_head_label(result),
            array_fill_body_label(result),
            array_fill_done_label(result),
            array_fill_body_label(result),
            value_name(value),
            array_fill_head_label(result),
            array_fill_done_label(result),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_array_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        root: IrArrayRoot,
        offset: IrValueId,
        trap: &IrTrapSite,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        if self.function.value_type(offset)
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
        let IrType::Array { element, length } = root_type else {
            return Err(BackendFailure::InvalidIr);
        };
        if element.ty() != ty {
            return Err(BackendFailure::InvalidIr);
        }

        let array_type = llvm_type(self.program, root_type)?;
        let element_type = llvm_type(self.program, ty)?;
        let in_range = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;
        writeln!(
            self.output,
            "  %{in_range} = icmp ult i64 {}, {length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
            value_name(offset),
            array_index_continue_label(result),
            array_index_trap_label(result),
            array_index_trap_label(result),
            self.traps[trap_id].len(),
            array_index_continue_label(result),
        )
        .map_err(|_| BackendFailure::TextEmission)?;

        let root_pointer = match root {
            IrArrayRoot::Value(value) => {
                let slot = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{slot} = alloca {array_type}\n  store {array_type} {}, ptr %{slot}",
                    value_name(value)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                format!("%{slot}")
            }
            IrArrayRoot::Constant(id) => constant_symbol(id),
        };
        let element_pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{element_pointer} = getelementptr inbounds {array_type}, ptr {root_pointer}, i64 0, i64 {}\n  {} = load {element_type}, ptr %{element_pointer}",
            value_name(offset),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_array_bounds_check(
        &mut self,
        result: IrValueId,
        ty: IrType,
        offset: IrValueId,
        trap: &IrTrapSite,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let IrType::GuardedArrayIndex { length } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.function.value_type(offset)
            != Some(IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let in_range = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;
        writeln!(
            self.output,
            "  %{in_range} = icmp ult i64 {}, {length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  {} = add i64 {}, 0",
            value_name(offset),
            array_bounds_continue_label(result),
            array_bounds_trap_label(result),
            array_bounds_trap_label(result),
            self.traps[trap_id].len(),
            array_bounds_continue_label(result),
            value_name(result),
            value_name(offset),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_array_insertion(
        &mut self,
        result: IrValueId,
        ty: IrType,
        aggregate: IrValueId,
        index: IrValueId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrType::Array { element, length } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let element_type = element.ty();
        if self.function.value_type(aggregate) != Some(ty)
            || self.function.value_type(index) != Some(IrType::GuardedArrayIndex { length })
            || self.function.value_type(value) != Some(element_type)
        {
            return Err(BackendFailure::InvalidIr);
        }
        let array_type = llvm_type(self.program, ty)?;
        let llvm_element_type = llvm_type(self.program, element_type)?;
        let array_slot = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{array_slot} = alloca {array_type}\n  store {array_type} {}, ptr %{array_slot}\n  %{element_pointer} = getelementptr inbounds {array_type}, ptr %{array_slot}, i64 0, i64 {}\n  store {llvm_element_type} {}, ptr %{element_pointer}\n  {} = load {array_type}, ptr %{array_slot}",
            value_name(aggregate),
            value_name(index),
            value_name(value),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}
