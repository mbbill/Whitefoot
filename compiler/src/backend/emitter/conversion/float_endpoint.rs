use super::*;

impl FunctionEmitter<'_, '_> {
    pub(super) fn emit_float_endpoint_conversion(
        &mut self,
        result: IrValueId,
        result_type: IrType,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        match (source_type, destination_type) {
            (IrType::Integer { .. }, IrType::Float { .. }) => self.emit_integer_to_float(
                result,
                result_type,
                source_type,
                destination_type,
                value,
            ),
            (IrType::Float { .. }, IrType::Integer { .. }) => self.emit_float_to_integer(
                result,
                result_type,
                source_type,
                destination_type,
                value,
            ),
            (IrType::Float { .. }, IrType::Float { .. }) => {
                self.emit_float_to_float(result, result_type, source_type, destination_type, value)
            }
            _ => Err(BackendFailure::InvalidIr),
        }
    }

    fn emit_integer_to_float(
        &mut self,
        result: IrValueId,
        result_type: IrType,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let (
            IrType::Integer {
                width: source_width,
                signed: source_signed,
            },
            IrType::Float {
                width: destination_width,
            },
        ) = (source_type, destination_type)
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if !matches!(source_width, 8 | 16 | 32 | 64) || !matches!(destination_width, 32 | 64) {
            return Err(BackendFailure::InvalidIr);
        }
        let total = (destination_width == 32 && source_width <= 16)
            || (destination_width == 64 && source_width <= 32);
        let converted = if total {
            if result_type != destination_type {
                return Err(BackendFailure::InvalidIr);
            }
            value_name(result)
        } else {
            format!("%{}", self.next_temporary()?)
        };
        let source_ty = llvm_type(self.program, source_type)?;
        let destination_ty = llvm_type(self.program, destination_type)?;
        let opcode = if source_signed { "sitofp" } else { "uitofp" };
        writeln!(
            self.output,
            "  {converted} = {opcode} {source_ty} {} to {destination_ty}",
            value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if total {
            return Ok(());
        }

        let recovered = self.next_temporary()?;
        let equal = self.next_temporary()?;
        let below_maximum = self.next_temporary()?;
        let valid = self.next_temporary()?;
        let reverse = if source_signed { "fptosi" } else { "fptoui" };
        let intrinsic = format!("llvm.{reverse}.sat.i{source_width}.f{destination_width}");
        self.intrinsics.insert(IntrinsicDeclaration::UnaryCast {
            name: intrinsic.clone(),
            result_ty: source_ty.clone(),
            argument_ty: destination_ty.clone(),
        });
        let maximum = if source_signed {
            (1_u128 << (source_width - 1)) - 1
        } else {
            (1_u128 << source_width) - 1
        };
        // A saturating reverse cast normally proves exactness. The one collision
        // is an unrepresentable integer maximum whose rounded float lies above
        // the source range and therefore saturates back to that same maximum.
        writeln!(
            self.output,
            "  %{recovered} = call {source_ty} @{intrinsic}({destination_ty} {converted})\n  %{equal} = icmp eq {source_ty} {}, %{recovered}\n  %{below_maximum} = icmp ne {source_ty} {}, {maximum}\n  %{valid} = and i1 %{equal}, %{below_maximum}",
            value_name(value),
            value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.emit_narrow_result(
            result,
            result_type,
            destination_type,
            &converted,
            &format!("%{valid}"),
        )
    }

    fn emit_float_to_integer(
        &mut self,
        result: IrValueId,
        result_type: IrType,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let (
            IrType::Float {
                width: source_width,
            },
            IrType::Integer {
                width: destination_width,
                signed: destination_signed,
            },
        ) = (source_type, destination_type)
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if !matches!(source_width, 32 | 64) || !matches!(destination_width, 8 | 16 | 32 | 64) {
            return Err(BackendFailure::InvalidIr);
        }
        let source_ty = llvm_type(self.program, source_type)?;
        let destination_ty = llvm_type(self.program, destination_type)?;
        let converted = self.next_temporary()?;
        let reverse = self.next_temporary()?;
        let equal = self.next_temporary()?;
        let opcode = if destination_signed {
            "fptosi"
        } else {
            "fptoui"
        };
        let intrinsic = format!("llvm.{opcode}.sat.i{destination_width}.f{source_width}");
        self.intrinsics.insert(IntrinsicDeclaration::UnaryCast {
            name: intrinsic.clone(),
            result_ty: destination_ty.clone(),
            argument_ty: source_ty.clone(),
        });
        let return_opcode = if destination_signed {
            "sitofp"
        } else {
            "uitofp"
        };
        writeln!(
            self.output,
            "  %{converted} = call {destination_ty} @{intrinsic}({source_ty} {})\n  %{reverse} = {return_opcode} {destination_ty} %{converted} to {source_ty}\n  %{equal} = fcmp oeq {source_ty} {}, %{reverse}",
            value_name(value),
            value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;

        let precision = if source_width == 32 { 24 } else { 53 };
        let maximum_bits = if destination_signed {
            destination_width - 1
        } else {
            destination_width
        };
        let valid = if maximum_bits <= precision {
            format!("%{equal}")
        } else {
            let below_maximum = self.next_temporary()?;
            let valid = self.next_temporary()?;
            let maximum = if destination_signed {
                (1_u128 << (destination_width - 1)) - 1
            } else {
                (1_u128 << destination_width) - 1
            };
            // When this maximum needs more significand bits than the source
            // float has, the first out-of-range power of two can saturate to it
            // and round back to the original float. Exclude that collision.
            writeln!(
                self.output,
                "  %{below_maximum} = icmp ne {destination_ty} %{converted}, {maximum}\n  %{valid} = and i1 %{equal}, %{below_maximum}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            format!("%{valid}")
        };
        self.emit_narrow_result(
            result,
            result_type,
            destination_type,
            &format!("%{converted}"),
            &valid,
        )
    }

    fn emit_float_to_float(
        &mut self,
        result: IrValueId,
        result_type: IrType,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let (
            IrType::Float {
                width: source_width,
            },
            IrType::Float {
                width: destination_width,
            },
        ) = (source_type, destination_type)
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if (source_width, destination_width) == (32, 64) {
            if result_type != destination_type {
                return Err(BackendFailure::InvalidIr);
            }
            return self.emit_widened_float(result, source_type, destination_type, value);
        }
        if (source_width, destination_width) != (64, 32) {
            return Err(BackendFailure::InvalidIr);
        }

        let source_ty = llvm_type(self.program, source_type)?;
        let destination_ty = llvm_type(self.program, destination_type)?;
        let converted = self.next_temporary()?;
        let widened = self.next_temporary()?;
        let exact = self.next_temporary()?;
        let nan = self.next_temporary()?;
        let valid = self.next_temporary()?;
        let selected = self.next_temporary()?;
        let canonical_nan = canonical_nan_operand(destination_type)?;
        writeln!(
            self.output,
            "  %{converted} = fptrunc {source_ty} {} to {destination_ty}\n  %{widened} = fpext {destination_ty} %{converted} to {source_ty}\n  %{exact} = fcmp oeq {source_ty} {}, %{widened}\n  %{nan} = fcmp uno {source_ty} {}, {}\n  %{valid} = or i1 %{nan}, %{exact}\n  %{selected} = select i1 %{nan}, {destination_ty} {canonical_nan}, {destination_ty} %{converted}",
            value_name(value),
            value_name(value),
            value_name(value),
            value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.emit_narrow_result(
            result,
            result_type,
            destination_type,
            &format!("%{selected}"),
            &format!("%{valid}"),
        )
    }

    fn emit_widened_float(
        &mut self,
        result: IrValueId,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        let source_ty = llvm_type(self.program, source_type)?;
        let destination_ty = llvm_type(self.program, destination_type)?;
        let widened = self.next_temporary()?;
        let nan = self.next_temporary()?;
        let canonical_nan = canonical_nan_operand(destination_type)?;
        writeln!(
            self.output,
            "  %{widened} = fpext {source_ty} {} to {destination_ty}\n  %{nan} = fcmp uno {source_ty} {}, {}\n  {} = select i1 %{nan}, {destination_ty} {canonical_nan}, {destination_ty} %{widened}",
            value_name(value),
            value_name(value),
            value_name(value),
            value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

fn canonical_nan_operand(ty: IrType) -> Result<String, BackendFailure> {
    let bits = match ty {
        IrType::Float { width: 32 } => 0x7fc0_0000,
        IrType::Float { width: 64 } => 0x7ff8_0000_0000_0000,
        _ => return Err(BackendFailure::InvalidIr),
    };
    constant_operand(IrConstant::Float { ty, bits }, ty)
}
