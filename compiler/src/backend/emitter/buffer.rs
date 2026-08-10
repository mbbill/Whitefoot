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
        if self.value_type(length) != Some(u64_type) || self.value_type(value) != Some(element.ty())
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
            self.value_name(length),
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
            self.value_name(length),
            buffer_fill_body_label(result),
            buffer_fill_done_label(result),
            buffer_fill_body_label(result),
            self.value_name(value),
            buffer_fill_head_label(result),
            buffer_fill_done_label(result),
            self.value_name(result),
            self.value_name(length),
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
            || !matches!(self.value_type(buffer), Some(IrType::Buffer { .. }))
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
                    .value_type(buffer)
                    .ok_or(BackendFailure::InvalidIr)?
            )?,
            self.value_name(buffer),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Emits a discharged source subscript read [OP-4]: the checker derived
    /// the bounds obligation, so no compare, branch, or trap is emitted in
    /// any build mode.
    pub(super) fn emit_buffer_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(buffer_type @ IrType::Buffer { element }) = self.value_type(buffer) else {
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
        let descriptor_type = llvm_type(self.program, buffer_type)?;
        let element_type = llvm_type(self.program, ty)?;
        let pointer = self.next_temporary()?;
        let element_pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{element_pointer} = getelementptr inbounds {element_type}, ptr %{pointer}, i64 {}\n  {} = load {element_type}, ptr %{element_pointer}",
            self.value_name(buffer),
            self.value_name(offset),
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Emits a discharged source subscript write [OP-4]: the index is the
    /// plain `u64` offset, already proven in bounds by the checker.
    pub(super) fn emit_buffer_store(
        &mut self,
        buffer: IrValueId,
        index: IrValueId,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let Some(buffer_type @ IrType::Buffer { element }) = self.value_type(buffer) else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.value_type(index)
            != Some(IrType::Integer {
                width: 64,
                signed: false,
            })
            || self.value_type(value) != Some(element.ty())
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
            self.value_name(buffer),
            self.value_name(index),
            self.value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Emits the check-aware wide probe: how many upcoming byte-walk
    /// iterations are provably no-ops.
    ///
    /// The window guard `index + 16 <= min(limit, length)` is internal, so
    /// the probe reads only bytes it proves in bounds and below the walk's
    /// exit bound, and it carries no trap: every observable byte — a needle
    /// hit, the exit bound, or any trap — stays with the unchanged scalar
    /// body. The lane-to-bit `bitcast` places lane 0 at the least
    /// significant bit on every supported (little-endian) target, so
    /// `cttz` yields the count of leading clean bytes.
    pub(super) fn emit_buffer_probe_skip(
        &mut self,
        result: IrValueId,
        ty: IrType,
        buffer: IrValueId,
        index: IrValueId,
        limit: IrValueId,
        needles: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let u64_type = IrType::Integer {
            width: 64,
            signed: false,
        };
        let u8_type = IrType::Integer {
            width: 8,
            signed: false,
        };
        let Some(buffer_type @ IrType::Buffer { element }) = self.value_type(buffer) else {
            return Err(BackendFailure::InvalidIr);
        };
        if ty != u64_type
            || element.ty() != u8_type
            || self.value_type(index) != Some(u64_type)
            || self.value_type(limit) != Some(u64_type)
            || needles.is_empty()
            || needles.len() > 4
            || needles
                .iter()
                .any(|needle| self.value_type(*needle) != Some(u8_type))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let descriptor_type = llvm_type(self.program, buffer_type)?;
        self.intrinsics.insert(IntrinsicDeclaration::UnaryWithFlag {
            name: "llvm.cttz.i16".to_owned(),
            ty: "i16".to_owned(),
        });
        let length = self.next_temporary()?;
        let tighter = self.next_temporary()?;
        let window = self.next_temporary()?;
        let room = self.next_temporary()?;
        let edge = self.next_temporary()?;
        let fits = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let address = self.next_temporary()?;
        let vector = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{length} = extractvalue {descriptor_type} {}, 1\n  %{tighter} = icmp ult i64 {}, %{length}\n  %{window} = select i1 %{tighter}, i64 {}, i64 %{length}\n  %{room} = icmp uge i64 %{window}, 16\n  br i1 %{room}, label %{}, label %{}\n{}:\n  %{edge} = sub i64 %{window}, 16\n  %{fits} = icmp ule i64 {}, %{edge}\n  br i1 %{fits}, label %{}, label %{}\n{}:\n  %{pointer} = extractvalue {descriptor_type} {}, 0\n  %{address} = getelementptr inbounds i8, ptr %{pointer}, i64 {}\n  %{vector} = load <16 x i8>, ptr %{address}, align 1",
            self.value_name(buffer),
            self.value_name(limit),
            self.value_name(limit),
            buffer_probe_room_label(result),
            buffer_probe_zero_label(result),
            buffer_probe_room_label(result),
            self.value_name(index),
            buffer_probe_load_label(result),
            buffer_probe_zero_label(result),
            buffer_probe_load_label(result),
            self.value_name(buffer),
            self.value_name(index),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let mut accumulated: Option<String> = None;
        for needle in needles {
            let inserted = self.next_temporary()?;
            let splat = self.next_temporary()?;
            let equal = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{inserted} = insertelement <16 x i8> poison, i8 {}, i64 0\n  %{splat} = shufflevector <16 x i8> %{inserted}, <16 x i8> poison, <16 x i32> zeroinitializer\n  %{equal} = icmp eq <16 x i8> %{vector}, %{splat}",
                self.value_name(*needle),
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            accumulated = Some(match accumulated {
                None => equal,
                Some(previous) => {
                    let combined = self.next_temporary()?;
                    writeln!(
                        self.output,
                        "  %{combined} = or <16 x i1> %{previous}, %{equal}"
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                    combined
                }
            });
        }
        let mask_lanes = accumulated.ok_or(BackendFailure::InvalidIr)?;
        let mask = self.next_temporary()?;
        let none = self.next_temporary()?;
        let zeros = self.next_temporary()?;
        let extended = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{mask} = bitcast <16 x i1> %{mask_lanes} to i16\n  %{none} = icmp eq i16 %{mask}, 0\n  br i1 %{none}, label %{}, label %{}\n{}:\n  br label %{}\n{}:\n  %{zeros} = call i16 @llvm.cttz.i16(i16 %{mask}, i1 true)\n  %{extended} = zext i16 %{zeros} to i64\n  br label %{}\n{}:\n  br label %{}\n{}:\n  {} = phi i64 [ 16, %{} ], [ %{extended}, %{} ], [ 0, %{} ]",
            buffer_probe_clean_label(result),
            buffer_probe_found_label(result),
            buffer_probe_clean_label(result),
            buffer_probe_join_label(result),
            buffer_probe_found_label(result),
            buffer_probe_join_label(result),
            buffer_probe_zero_label(result),
            buffer_probe_join_label(result),
            buffer_probe_join_label(result),
            self.value_name(result),
            buffer_probe_clean_label(result),
            buffer_probe_found_label(result),
            buffer_probe_zero_label(result),
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
            IrFlatElement::Float { width } if matches!(width, 32 | 64) => Ok(u64::from(width / 8)),
            IrFlatElement::Float { .. } => Err(BackendFailure::InvalidIr),
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

pub(super) fn buffer_probe_room_label(value: IrValueId) -> String {
    format!("buffer.probe.room.v{}", value.ordinal())
}

pub(super) fn buffer_probe_load_label(value: IrValueId) -> String {
    format!("buffer.probe.load.v{}", value.ordinal())
}

pub(super) fn buffer_probe_clean_label(value: IrValueId) -> String {
    format!("buffer.probe.clean.v{}", value.ordinal())
}

pub(super) fn buffer_probe_found_label(value: IrValueId) -> String {
    format!("buffer.probe.found.v{}", value.ordinal())
}

pub(super) fn buffer_probe_zero_label(value: IrValueId) -> String {
    format!("buffer.probe.zero.v{}", value.ordinal())
}

pub(super) fn buffer_probe_join_label(value: IrValueId) -> String {
    format!("buffer.probe.join.v{}", value.ordinal())
}
