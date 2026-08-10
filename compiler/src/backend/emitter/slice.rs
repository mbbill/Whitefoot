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
                let llvm_array_type = llvm_type(self.program, array_type)?;
                let slot = self.entry_slot(&llvm_array_type)?;
                writeln!(
                    self.output,
                    "  store {llvm_array_type} {}, ptr {slot}",
                    self.value_name(value),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                slot
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
        if self.value_type(buffer) != Some(buffer_type) {
            return Err(BackendFailure::InvalidIr);
        }
        let descriptor_type = llvm_type(self.program, ty)?;
        let pointer = self.next_temporary()?;
        let length = self.next_temporary()?;
        let partial = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{length} = extractvalue {descriptor_type} {}, 1\n  %{partial} = insertvalue {descriptor_type} zeroinitializer, ptr %{pointer}, 0\n  {} = insertvalue {descriptor_type} %{partial}, i64 %{length}, 1",
            self.value_name(buffer),
            self.value_name(buffer),
            self.value_name(result),
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
            || !matches!(self.value_type(slice), Some(IrType::Slice { .. }))
        {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(
            self.output,
            "  {} = extractvalue {} {}, 1",
            self.value_name(result),
            llvm_type(
                self.program,
                self.function
                    .value_type(slice)
                    .ok_or(BackendFailure::InvalidIr)?
            )?,
            self.value_name(slice),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Emits a discharged source subscript read [OP-4]: the checker derived
    /// the bounds obligation, so no compare, branch, or trap is emitted in
    /// any build mode.
    pub(super) fn emit_slice_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        slice: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(slice_type @ IrType::Slice { element }) = self.value_type(slice) else {
            return Err(BackendFailure::InvalidIr);
        };
        if element.ty() != ty
            || self.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let descriptor_type = llvm_type(self.program, slice_type)?;
        let element_type = llvm_type(self.program, ty)?;
        let pointer = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 {}\n  {} = load {element_type}, ptr %{element_pointer}",
            self.value_name(slice),
            self.value_name(offset),
            self.value_name(result),
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
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}
