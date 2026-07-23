use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_numeric_conversion(
        &mut self,
        result: IrValueId,
        result_type: IrType,
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        if source_type == destination_type || self.function.value_type(value) != Some(source_type) {
            return Err(BackendFailure::InvalidIr);
        }
        match (source_type, destination_type) {
            (IrType::Integer { .. }, IrType::Integer { .. }) => self.emit_integer_conversion(
                result,
                result_type,
                source_type,
                destination_type,
                value,
            ),
            (
                IrType::Integer { width, signed },
                IrType::Float {
                    width: destination_width,
                },
            ) if (destination_width == 32 && width <= 16)
                || (destination_width == 64 && width <= 32) =>
            {
                if result_type != destination_type {
                    return Err(BackendFailure::InvalidIr);
                }
                let opcode = if signed { "sitofp" } else { "uitofp" };
                writeln!(
                    self.output,
                    "  {} = {opcode} i{width} {} to {}",
                    value_name(result),
                    value_name(value),
                    llvm_type(self.program, destination_type)?
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            (IrType::Float { width: 32 }, IrType::Float { width: 64 }) => {
                if result_type != destination_type {
                    return Err(BackendFailure::InvalidIr);
                }
                writeln!(
                    self.output,
                    "  {} = fpext float {} to double",
                    value_name(result),
                    value_name(value)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            _ => Err(BackendFailure::InvalidIr),
        }
    }

    fn emit_integer_conversion(
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
            IrType::Integer {
                width: destination_width,
                signed: destination_signed,
            },
        ) = (source_type, destination_type)
        else {
            return Err(BackendFailure::InvalidIr);
        };
        let total = source_width < destination_width
            && (source_signed == destination_signed || (!source_signed && destination_signed));
        let converted = if total {
            if result_type != destination_type {
                return Err(BackendFailure::InvalidIr);
            }
            value_name(result)
        } else {
            format!("%{}", self.next_temporary()?)
        };
        self.emit_integer_cast(
            &converted,
            value,
            source_width,
            source_signed,
            destination_width,
            destination_signed,
        )?;
        if total {
            return Ok(());
        }

        let error_type = self.checked_result_error_type(result_type, destination_type, &[0])?;
        let valid = self.emit_integer_conversion_validity(
            value,
            source_width,
            source_signed,
            destination_width,
            destination_signed,
        )?;
        let result_ty = llvm_type(self.program, result_type)?;
        let destination_ty = llvm_type(self.program, destination_type)?;
        let error_ty = llvm_type(self.program, error_type)?;
        let ok_tag = self.next_temporary()?;
        let ok_value = self.next_temporary()?;
        let error_tag = self.next_temporary()?;
        let error_value = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{ok_tag} = insertvalue {result_ty} zeroinitializer, i32 0, 0\n  %{ok_value} = insertvalue {result_ty} %{ok_tag}, {destination_ty} {converted}, 1\n  %{error_tag} = insertvalue {result_ty} zeroinitializer, i32 1, 0\n  %{error_value} = insertvalue {result_ty} %{error_tag}, {error_ty} 0, 2\n  {} = select i1 %{valid}, {result_ty} %{ok_value}, {result_ty} %{error_value}",
            value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_integer_cast(
        &mut self,
        result: &str,
        value: IrValueId,
        source_width: u8,
        source_signed: bool,
        destination_width: u8,
        destination_signed: bool,
    ) -> Result<(), BackendFailure> {
        let source = value_name(value);
        let opcode = if source_width > destination_width {
            "trunc"
        } else if source_width < destination_width && source_signed && destination_signed {
            "sext"
        } else if source_width < destination_width {
            "zext"
        } else {
            return writeln!(self.output, "  {result} = or i{source_width} {source}, 0")
                .map_err(|_| BackendFailure::TextEmission);
        };
        writeln!(
            self.output,
            "  {result} = {opcode} i{source_width} {source} to i{destination_width}"
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_integer_conversion_validity(
        &mut self,
        value: IrValueId,
        source_width: u8,
        source_signed: bool,
        destination_width: u8,
        destination_signed: bool,
    ) -> Result<String, BackendFailure> {
        let value = value_name(value);
        if source_signed == destination_signed {
            if source_width <= destination_width {
                return Err(BackendFailure::InvalidIr);
            }
            if source_signed {
                let minimum = -(1_i128 << (destination_width - 1));
                let maximum = (1_i128 << (destination_width - 1)) - 1;
                let above_minimum = self.next_temporary()?;
                let below_maximum = self.next_temporary()?;
                let valid = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{above_minimum} = icmp sge i{source_width} {value}, {minimum}\n  %{below_maximum} = icmp sle i{source_width} {value}, {maximum}\n  %{valid} = and i1 %{above_minimum}, %{below_maximum}"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                return Ok(valid);
            }
            let maximum = (1_u128 << destination_width) - 1;
            let valid = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{valid} = icmp ule i{source_width} {value}, {maximum}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(valid);
        }
        if source_signed {
            let nonnegative = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{nonnegative} = icmp sge i{source_width} {value}, 0"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            if destination_width >= source_width {
                return Ok(nonnegative);
            }
            let maximum = (1_u128 << destination_width) - 1;
            let below_maximum = self.next_temporary()?;
            let valid = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{below_maximum} = icmp sle i{source_width} {value}, {maximum}\n  %{valid} = and i1 %{nonnegative}, %{below_maximum}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(valid);
        }
        if destination_width > source_width {
            return Err(BackendFailure::InvalidIr);
        }
        let maximum = (1_u128 << (destination_width - 1)) - 1;
        let valid = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{valid} = icmp ule i{source_width} {value}, {maximum}"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(valid)
    }
}
