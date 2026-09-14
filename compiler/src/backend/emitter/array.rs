use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    /// Transfer all owners in logical order. Array and fixed-run storage have
    /// different types and cannot share a storage-plan group; retaining the
    /// source operand keeps its storage live through both split copies.
    pub(super) fn emit_full_array_conversion(
        &mut self,
        result: IrValueId,
        ty: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let source_type = self.value_type(value).ok_or(BackendFailure::InvalidIr)?;
        let (element, length, to_array) = match (source_type, ty) {
            (
                IrType::FixedVector {
                    element: source,
                    length: source_length,
                },
                IrType::Array { element, length },
            ) if source == element && source_length == length => (element, length, true),
            (
                IrType::Array {
                    element: source,
                    length: source_length,
                },
                IrType::FixedVector { element, length },
            ) if source == element && source_length == length => (element, length, false),
            _ => return Err(BackendFailure::InvalidIr),
        };
        let source = self.value_place(value)?;
        let destination = self.value_place(result)?;
        let fixed_type = if to_array { source_type } else { ty };
        let fixed_llvm = llvm_type(self.program, fixed_type)?;
        if !to_array {
            if length != 0 {
                // The fixed run's first field is the dense element storage.
                self.copy_storage(source_type, &source, &destination)?;
            }
            for (field, contents) in [(1, length), (2, 0)] {
                let pointer = self.next_temporary()?;
                writeln!(self.output, "  %{pointer} = getelementptr inbounds {fixed_llvm}, ptr {destination}, i64 0, i32 {field}\n  store i64 {contents}, ptr %{pointer}")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            return Ok(());
        }
        // No element access or head arithmetic is needed for the empty case.
        if length == 0 {
            return Ok(());
        }
        let element_type = self
            .program
            .element(element)
            .ok_or(BackendFailure::InvalidIr)?;
        let element_llvm = llvm_type(self.program, element_type)?;
        let head_pointer = self.next_temporary()?;
        let head = self.next_temporary()?;
        let tail = self.next_temporary()?;
        let tail_source = self.next_temporary()?;
        let prefix_destination = self.next_temporary()?;
        writeln!(self.output,
            "  %{head_pointer} = getelementptr inbounds {fixed_llvm}, ptr {source}, i64 0, i32 2\n  %{head} = load i64, ptr %{head_pointer}\n  %{tail} = sub i64 {length}, %{head}\n  %{tail_source} = getelementptr inbounds {element_llvm}, ptr {source}, i64 %{head}\n  %{prefix_destination} = getelementptr inbounds {element_llvm}, ptr {destination}, i64 %{tail}")
            .map_err(|_| BackendFailure::TextEmission)?;
        self.copy_element_range(
            &element_llvm,
            &format!("%{tail_source}"),
            &destination,
            &format!("%{tail}"),
        )?;
        self.copy_element_range(
            &element_llvm,
            &source,
            &format!("%{prefix_destination}"),
            &format!("%{head}"),
        )
    }

    /// A bounded count of complete stride-spaced representations. Target
    /// layout supplies the stride; zero-byte elements transfer no bytes while
    /// ownership still moves with the whole source value.
    fn copy_element_range(
        &mut self,
        element_llvm: &str,
        source: &str,
        destination: &str,
        count: &str,
    ) -> Result<(), BackendFailure> {
        let end = self.next_temporary()?;
        let bytes = self.next_temporary()?;
        writeln!(self.output, "  %{end} = getelementptr {element_llvm}, ptr null, i64 {count}\n  %{bytes} = ptrtoint ptr %{end} to i64\n  call void @llvm.memmove.p0.p0.i64(ptr {destination}, ptr {source}, i64 %{bytes}, i1 false)")
            .map_err(|_| BackendFailure::TextEmission)
    }

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
        let element_type = self
            .program
            .element(element)
            .ok_or(BackendFailure::InvalidIr)?;
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
        if self.program.element(element) != Some(ty) {
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
