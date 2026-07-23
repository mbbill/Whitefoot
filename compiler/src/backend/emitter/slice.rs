use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_slice_from_array(
        &mut self,
        result: IrValueId,
        ty: IrType,
        array: IrArrayRoot,
    ) -> Result<(), BackendFailure> {
        let IrType::Slice { element } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let array_type = match array {
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
        let IrType::Array {
            element: array_element,
            length,
        } = array_type
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if array_element != element {
            return Err(BackendFailure::InvalidIr);
        }

        let pointer = match array {
            IrArrayRoot::Value(value) => {
                let slot = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{slot} = alloca {}\n  store {} {}, ptr %{slot}",
                    llvm_type(self.program, array_type)?,
                    llvm_type(self.program, array_type)?,
                    value_name(value),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                format!("%{slot}")
            }
            IrArrayRoot::Constant(id) => constant_symbol(id),
        };
        self.emit_slice_descriptor(result, ty, &pointer, length)
    }

    pub(super) fn emit_slice_from_buffer(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrType::Slice { element } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let buffer_type = IrType::Buffer { element };
        if self.function.value_type(buffer) != Some(buffer_type) {
            return Err(BackendFailure::InvalidIr);
        }
        let descriptor_type = llvm_type(self.program, ty)?;
        let pointer = self.next_temporary()?;
        let length = self.next_temporary()?;
        let partial = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{length} = extractvalue {descriptor_type} {}, 1\n  %{partial} = insertvalue {descriptor_type} zeroinitializer, ptr %{pointer}, 0\n  {} = insertvalue {descriptor_type} %{partial}, i64 %{length}, 1",
            value_name(buffer),
            value_name(buffer),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_slice_length(
        &mut self,
        result: IrValueId,
        ty: IrType,
        slice: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
            || !matches!(self.function.value_type(slice), Some(IrType::Slice { .. }))
        {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(
            self.output,
            "  {} = extractvalue {} {}, 1",
            value_name(result),
            llvm_type(
                self.program,
                self.function
                    .value_type(slice)
                    .ok_or(BackendFailure::InvalidIr)?
            )?,
            value_name(slice),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_slice_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        slice: IrValueId,
        offset: IrValueId,
        trap: &IrTrapSite,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(slice_type @ IrType::Slice { element }) = self.function.value_type(slice) else {
            return Err(BackendFailure::InvalidIr);
        };
        if element.ty() != ty
            || self.function.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let descriptor_type = llvm_type(self.program, slice_type)?;
        let element_type = llvm_type(self.program, ty)?;
        let length = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;
        writeln!(
            self.output,
            "  %{length} = extractvalue {descriptor_type} {}, 1\n  %{in_range} = icmp ult i64 {}, %{length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 {}\n  {} = load {element_type}, ptr %{element_pointer}",
            value_name(slice),
            value_name(offset),
            slice_index_continue_label(result),
            slice_index_trap_label(result),
            slice_index_trap_label(result),
            self.traps[trap_id].len(),
            slice_index_continue_label(result),
            value_name(slice),
            value_name(offset),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_slice_descriptor(
        &mut self,
        result: IrValueId,
        ty: IrType,
        pointer: &str,
        length: u64,
    ) -> Result<(), BackendFailure> {
        let descriptor_type = llvm_type(self.program, ty)?;
        let partial = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{partial} = insertvalue {descriptor_type} zeroinitializer, ptr {pointer}, 0\n  {} = insertvalue {descriptor_type} %{partial}, i64 {length}, 1",
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

fn slice_index_trap_label(value: IrValueId) -> String {
    format!("slice.index.trap.v{}", value.ordinal())
}

pub(super) fn slice_index_continue_label(value: IrValueId) -> String {
    format!("slice.index.cont.v{}", value.ordinal())
}
