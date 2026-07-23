use crate::IrFlatElement;

use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_buffer_fill(
        &mut self,
        result: IrValueId,
        ty: IrType,
        length: IrValueId,
        value: IrValueId,
        trap: &IrTrapSite,
        target_domains: IrRuntimeTargetObligations,
    ) -> Result<(), BackendFailure> {
        if !target_domains.is_complete() {
            return Err(BackendFailure::InvalidIr);
        }
        let IrType::Buffer { element } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let u64_type = IrType::Integer {
            width: 64,
            signed: false,
        };
        if self.function.value_type(length) != Some(u64_type)
            || self.function.value_type(value) != Some(element.ty())
        {
            return Err(BackendFailure::InvalidIr);
        }

        let intrinsic = "llvm.umul.with.overflow.i64";
        self.intrinsics.insert(IntrinsicDeclaration::Overflow {
            name: intrinsic.to_owned(),
            ty: "i64".to_owned(),
        });
        let buffer_type = llvm_type(self.program, ty)?;
        let element_type = llvm_type(self.program, element.ty())?;
        let element_size = self.buffer_element_size(element)?;
        let product = self.next_temporary()?;
        let bytes = self.next_temporary()?;
        let overflow = self.next_temporary()?;
        let target_in_range = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let zero_size = self.next_temporary()?;
        let nonnull = self.next_temporary()?;
        let usable = self.next_temporary()?;
        let index = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let next_index = self.next_temporary()?;
        let descriptor = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;

        writeln!(
            self.output,
            "  %{product} = call {{ i64, i1 }} @{intrinsic}(i64 {}, i64 {element_size})\n  %{bytes} = extractvalue {{ i64, i1 }} %{product}, 0\n  %{overflow} = extractvalue {{ i64, i1 }} %{product}, 1\n  br i1 %{overflow}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  %{target_in_range} = icmp ule i64 %{bytes}, {}\n  br i1 %{target_in_range}, label %{}, label %{}\n{}:\n  call void @abort()\n  unreachable\n{}:\n  %{pointer} = call ptr @malloc(i64 %{bytes})\n  %{zero_size} = icmp eq i64 %{bytes}, 0\n  %{nonnull} = icmp ne ptr %{pointer}, null\n  %{usable} = or i1 %{zero_size}, %{nonnull}\n  br i1 %{usable}, label %{}, label %{}\n{}:\n  call void @abort()\n  unreachable\n{}:\n  %{index} = phi i64 [ 0, %{} ], [ %{next_index}, %{} ]\n  %{in_range} = icmp ult i64 %{index}, {}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 %{index}\n  store {element_type} {}, ptr %{element_pointer}\n  %{next_index} = add i64 %{index}, 1\n  br label %{}\n{}:\n  %{descriptor} = insertvalue {buffer_type} zeroinitializer, ptr %{pointer}, 0\n  {} = insertvalue {buffer_type} %{descriptor}, i64 {}, 1",
            value_name(length),
            buffer_fill_overflow_label(result),
            buffer_fill_target_check_label(result),
            buffer_fill_overflow_label(result),
            self.traps[trap_id].len(),
            buffer_fill_target_check_label(result),
            self.target.runtime_allocation_max(),
            buffer_fill_allocate_label(result),
            buffer_fill_target_failure_label(result),
            buffer_fill_target_failure_label(result),
            buffer_fill_allocate_label(result),
            buffer_fill_head_label(result),
            buffer_fill_oom_label(result),
            buffer_fill_oom_label(result),
            buffer_fill_head_label(result),
            buffer_fill_allocate_label(result),
            buffer_fill_body_label(result),
            value_name(length),
            buffer_fill_body_label(result),
            buffer_fill_done_label(result),
            buffer_fill_body_label(result),
            value_name(value),
            buffer_fill_head_label(result),
            buffer_fill_done_label(result),
            value_name(result),
            value_name(length),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_buffer_length(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
            || !matches!(
                self.function.value_type(buffer),
                Some(IrType::Buffer { .. })
            )
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
                    .value_type(buffer)
                    .ok_or(BackendFailure::InvalidIr)?
            )?,
            value_name(buffer),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_buffer_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
        offset: IrValueId,
        trap: &IrTrapSite,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(buffer_type @ IrType::Buffer { element }) = self.function.value_type(buffer)
        else {
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
        let descriptor_type = llvm_type(self.program, buffer_type)?;
        let element_type = llvm_type(self.program, ty)?;
        let length = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;
        writeln!(
            self.output,
            "  %{length} = extractvalue {descriptor_type} {}, 1\n  %{in_range} = icmp ult i64 {}, %{length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 {}\n  {} = load {element_type}, ptr %{element_pointer}",
            value_name(buffer),
            value_name(offset),
            buffer_index_continue_label(result),
            buffer_index_trap_label(result),
            buffer_index_trap_label(result),
            self.traps[trap_id].len(),
            buffer_index_continue_label(result),
            value_name(buffer),
            value_name(offset),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_buffer_bounds_check(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
        offset: IrValueId,
        trap: &IrTrapSite,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let IrType::GuardedBufferIndex { element } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let buffer_type = IrType::Buffer { element };
        if self.function.value_type(buffer) != Some(buffer_type)
            || self.function.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let length = self.next_temporary()?;
        let in_range = self.next_temporary()?;
        let trap_id = self.register_trap(trap)?;
        writeln!(
            self.output,
            "  %{length} = extractvalue {} {}, 1\n  %{in_range} = icmp ult i64 {}, %{length}\n  br i1 %{in_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  {} = add i64 {}, 0",
            llvm_type(self.program, buffer_type)?,
            value_name(buffer),
            value_name(offset),
            buffer_bounds_continue_label(result),
            buffer_bounds_trap_label(result),
            buffer_bounds_trap_label(result),
            self.traps[trap_id].len(),
            buffer_bounds_continue_label(result),
            value_name(result),
            value_name(offset),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_buffer_store(
        &mut self,
        buffer: IrValueId,
        index: IrValueId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let Some(buffer_type @ IrType::Buffer { element }) = self.function.value_type(buffer)
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.function.value_type(index) != Some(IrType::GuardedBufferIndex { element })
            || self.function.value_type(value) != Some(element.ty())
        {
            return Err(BackendFailure::InvalidIr);
        }
        let pointer = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        let element_type = llvm_type(self.program, element.ty())?;
        writeln!(
            self.output,
            "  %{pointer} = extractvalue {} {}, 0\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 {}\n  store {element_type} {}, ptr %{element_pointer}",
            llvm_type(self.program, buffer_type)?,
            value_name(buffer),
            value_name(index),
            value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    fn buffer_element_size(&self, element: IrFlatElement) -> Result<u64, BackendFailure> {
        match element {
            IrFlatElement::Unit | IrFlatElement::Bool => Ok(1),
            IrFlatElement::Integer { width, .. } if matches!(width, 8 | 16 | 32 | 64) => {
                Ok(u64::from(width / 8))
            }
            IrFlatElement::Integer { .. } => Err(BackendFailure::InvalidIr),
            IrFlatElement::TagOnlyNominal(id) => {
                let nominal = self.nominal(id)?;
                if !nominal.is_tag_only_enum() {
                    return Err(BackendFailure::InvalidIr);
                }
                let IrNominalKind::Enum { variants } = nominal.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                Ok(if variants.len() <= 2 { 1 } else { 4 })
            }
        }
    }
}

pub(super) fn buffer_fill_overflow_label(value: IrValueId) -> String {
    format!("buffer.fill.overflow.v{}", value.ordinal())
}

pub(super) fn buffer_fill_allocate_label(value: IrValueId) -> String {
    format!("buffer.fill.allocate.v{}", value.ordinal())
}

pub(super) fn buffer_fill_target_check_label(value: IrValueId) -> String {
    format!("buffer.fill.target.check.v{}", value.ordinal())
}

pub(super) fn buffer_fill_target_failure_label(value: IrValueId) -> String {
    format!("buffer.fill.target.failure.v{}", value.ordinal())
}

pub(super) fn buffer_fill_oom_label(value: IrValueId) -> String {
    format!("buffer.fill.oom.v{}", value.ordinal())
}

pub(super) fn buffer_fill_head_label(value: IrValueId) -> String {
    format!("buffer.fill.head.v{}", value.ordinal())
}

pub(super) fn buffer_fill_body_label(value: IrValueId) -> String {
    format!("buffer.fill.body.v{}", value.ordinal())
}

pub(super) fn buffer_fill_done_label(value: IrValueId) -> String {
    format!("buffer.fill.done.v{}", value.ordinal())
}

pub(super) fn buffer_index_trap_label(value: IrValueId) -> String {
    format!("buffer.index.trap.v{}", value.ordinal())
}

pub(super) fn buffer_index_continue_label(value: IrValueId) -> String {
    format!("buffer.index.cont.v{}", value.ordinal())
}

pub(super) fn buffer_bounds_trap_label(value: IrValueId) -> String {
    format!("buffer.bounds.trap.v{}", value.ordinal())
}

pub(super) fn buffer_bounds_continue_label(value: IrValueId) -> String {
    format!("buffer.bounds.cont.v{}", value.ordinal())
}
