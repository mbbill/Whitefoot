use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_integer(
        &mut self,
        block: IrBlockId,
        _index: usize,
        result: IrValueId,
        result_type: IrType,
        operation: IrIntegerOperation,
        operand_type: IrType,
        arguments: &[IrValueId],
        trap: Option<&IrTrapSite>,
    ) -> Result<(), BackendFailure> {
        let IrType::Integer { width, signed } = operand_type else {
            return Err(BackendFailure::InvalidIr);
        };
        let has_u32_amount = matches!(
            operation,
            IrIntegerOperation::ShiftLeftWrap
                | IrIntegerOperation::ShiftRightWrap
                | IrIntegerOperation::ShiftLeftTrap
                | IrIntegerOperation::ShiftRightTrap
                | IrIntegerOperation::RotateLeft
                | IrIntegerOperation::RotateRight
        );
        if arguments.iter().enumerate().any(|(index, argument)| {
            let expected = if has_u32_amount && index == 1 {
                IrType::Integer {
                    width: 32,
                    signed: false,
                }
            } else {
                operand_type
            };
            self.function.value_type(*argument) != Some(expected)
        }) {
            return Err(BackendFailure::InvalidIr);
        }
        let ty = llvm_type(self.program, operand_type)?;
        let binary = match arguments {
            [left, right] => Some((value_name(*left), value_name(*right))),
            _ => None,
        };
        let unary = match arguments {
            [argument] => Some(value_name(*argument)),
            _ => None,
        };
        match operation {
            IrIntegerOperation::AddWrap
            | IrIntegerOperation::SubtractWrap
            | IrIntegerOperation::MultiplyWrap
            | IrIntegerOperation::NegateWrap => {
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let (opcode, left, right) = match operation {
                    IrIntegerOperation::AddWrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("add", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::SubtractWrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("sub", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::MultiplyWrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("mul", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::NegateWrap if signed => (
                        "sub",
                        "0",
                        unary.as_deref().ok_or(BackendFailure::InvalidIr)?,
                    ),
                    _ => return Err(BackendFailure::InvalidIr),
                };
                writeln!(
                    self.output,
                    "  {} = {opcode} {ty} {left}, {right}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::AddTrap
            | IrIntegerOperation::SubtractTrap
            | IrIntegerOperation::MultiplyTrap
            | IrIntegerOperation::NegateTrap => {
                if result_type != operand_type {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap = trap.ok_or(BackendFailure::InvalidIr)?;
                let (stem, left, right) = match operation {
                    IrIntegerOperation::AddTrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("add", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::SubtractTrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("sub", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::MultiplyTrap => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("mul", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::NegateTrap if signed => (
                        "sub",
                        "0",
                        unary.as_deref().ok_or(BackendFailure::InvalidIr)?,
                    ),
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let sign = if signed { 's' } else { 'u' };
                let intrinsic = format!("llvm.{sign}{stem}.with.overflow.i{width}");
                self.intrinsics.insert(IntrinsicDeclaration::Overflow {
                    name: intrinsic.clone(),
                    ty: ty.clone(),
                });
                let pair = self.next_temporary()?;
                let overflow = self.next_temporary()?;
                let trap_id = self.register_trap(trap)?;
                writeln!(
                    self.output,
                    "  %{pair} = call {{ {ty}, i1 }} @{intrinsic}({ty} {left}, {ty} {right})\n  {} = extractvalue {{ {ty}, i1 }} %{pair}, 0\n  %{overflow} = extractvalue {{ {ty}, i1 }} %{pair}, 1\n  br i1 %{overflow}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
                    value_name(result),
                    overflow_trap_label(result),
                    overflow_continue_label(result),
                    overflow_trap_label(result),
                    self.traps[trap_id].len(),
                    overflow_continue_label(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::AddChecked
            | IrIntegerOperation::SubtractChecked
            | IrIntegerOperation::MultiplyChecked
            | IrIntegerOperation::NegateChecked => {
                if trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let error_type = self.checked_result_error_type(result_type, operand_type, &[0])?;

                let (stem, left, right) = match operation {
                    IrIntegerOperation::AddChecked => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("add", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::SubtractChecked => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("sub", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::MultiplyChecked => {
                        let (left, right) = binary.as_ref().ok_or(BackendFailure::InvalidIr)?;
                        ("mul", left.as_str(), right.as_str())
                    }
                    IrIntegerOperation::NegateChecked if signed => (
                        "sub",
                        "0",
                        unary.as_deref().ok_or(BackendFailure::InvalidIr)?,
                    ),
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let sign = if signed { 's' } else { 'u' };
                let intrinsic = format!("llvm.{sign}{stem}.with.overflow.i{width}");
                self.intrinsics.insert(IntrinsicDeclaration::Overflow {
                    name: intrinsic.clone(),
                    ty: ty.clone(),
                });
                let pair = self.next_temporary()?;
                let value = self.next_temporary()?;
                let overflow = self.next_temporary()?;
                let ok_tag = self.next_temporary()?;
                let ok_value = self.next_temporary()?;
                let error_tag = self.next_temporary()?;
                let error_value = self.next_temporary()?;
                let result_ty = llvm_type(self.program, result_type)?;
                let error_ty = llvm_type(self.program, error_type)?;
                writeln!(
                    self.output,
                    "  %{pair} = call {{ {ty}, i1 }} @{intrinsic}({ty} {left}, {ty} {right})\n  %{value} = extractvalue {{ {ty}, i1 }} %{pair}, 0\n  %{overflow} = extractvalue {{ {ty}, i1 }} %{pair}, 1\n  %{ok_tag} = insertvalue {result_ty} zeroinitializer, i32 0, 0\n  %{ok_value} = insertvalue {result_ty} %{ok_tag}, {ty} %{value}, 1\n  %{error_tag} = insertvalue {result_ty} zeroinitializer, i32 1, 0\n  %{error_value} = insertvalue {result_ty} %{error_tag}, {error_ty} 0, 2\n  {} = select i1 %{overflow}, {result_ty} %{error_value}, {result_ty} %{ok_value}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::DivideChecked | IrIntegerOperation::RemainderChecked => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let error_type =
                    self.checked_result_error_type(result_type, operand_type, &[0, 1])?;
                let result_ty = llvm_type(self.program, result_type)?;
                let error_ty = llvm_type(self.program, error_type)?;
                let is_zero = self.next_temporary()?;
                writeln!(self.output, "  %{is_zero} = icmp eq {ty} {right}, 0")
                    .map_err(|_| BackendFailure::TextEmission)?;

                let error_condition = if signed {
                    let is_minimum = self.next_temporary()?;
                    let is_minus_one = self.next_temporary()?;
                    let is_overflow = self.next_temporary()?;
                    let is_error = self.next_temporary()?;
                    let minimum = -(1_i128 << (width - 1));
                    writeln!(
                        self.output,
                        "  %{is_minimum} = icmp eq {ty} {left}, {minimum}\n  %{is_minus_one} = icmp eq {ty} {right}, -1\n  %{is_overflow} = and i1 %{is_minimum}, %{is_minus_one}\n  %{is_error} = or i1 %{is_zero}, %{is_overflow}"
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                    is_error
                } else {
                    is_zero.clone()
                };

                let opcode = match (operation, signed) {
                    (IrIntegerOperation::DivideChecked, true) => "sdiv",
                    (IrIntegerOperation::DivideChecked, false) => "udiv",
                    (IrIntegerOperation::RemainderChecked, true) => "srem",
                    (IrIntegerOperation::RemainderChecked, false) => "urem",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let safe_value = self.next_temporary()?;
                let ok_tag = self.next_temporary()?;
                let ok_value = self.next_temporary()?;
                let error_kind = signed.then(|| self.next_temporary()).transpose()?;
                let error_tag = self.next_temporary()?;
                let error_value = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  br i1 %{error_condition}, label %{}, label %{}\n{}:\n  %{safe_value} = {opcode} {ty} {left}, {right}\n  %{ok_tag} = insertvalue {result_ty} zeroinitializer, i32 0, 0\n  %{ok_value} = insertvalue {result_ty} %{ok_tag}, {ty} %{safe_value}, 1\n  br label %{}\n{}:",
                    integer_error_label(result),
                    integer_safe_label(result),
                    integer_safe_label(result),
                    integer_continue_label(result),
                    integer_error_label(result),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let error_operand = if let Some(error_kind) = error_kind {
                    writeln!(
                        self.output,
                        "  %{error_kind} = select i1 %{is_zero}, {error_ty} 0, {error_ty} 1"
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                    format!("%{error_kind}")
                } else {
                    "0".to_owned()
                };
                writeln!(
                    self.output,
                    "  %{error_tag} = insertvalue {result_ty} zeroinitializer, i32 1, 0\n  %{error_value} = insertvalue {result_ty} %{error_tag}, {error_ty} {error_operand}, 2\n  br label %{}\n{}:\n  {} = phi {result_ty} [ %{ok_value}, %{} ], [ %{error_value}, %{} ]",
                    integer_continue_label(result),
                    integer_continue_label(result),
                    value_name(result),
                    integer_safe_label(result),
                    integer_error_label(result),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::DivideTrap | IrIntegerOperation::RemainderTrap => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap = trap.ok_or(BackendFailure::InvalidIr)?;
                let is_zero = self.next_temporary()?;
                writeln!(self.output, "  %{is_zero} = icmp eq {ty} {right}, 0")
                    .map_err(|_| BackendFailure::TextEmission)?;
                let failure = if signed {
                    let is_minimum = self.next_temporary()?;
                    let is_minus_one = self.next_temporary()?;
                    let is_overflow = self.next_temporary()?;
                    let is_failure = self.next_temporary()?;
                    let minimum = -(1_i128 << (width - 1));
                    writeln!(
                        self.output,
                        "  %{is_minimum} = icmp eq {ty} {left}, {minimum}\n  %{is_minus_one} = icmp eq {ty} {right}, -1\n  %{is_overflow} = and i1 %{is_minimum}, %{is_minus_one}\n  %{is_failure} = or i1 %{is_zero}, %{is_overflow}"
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                    is_failure
                } else {
                    is_zero
                };
                let opcode = match (operation, signed) {
                    (IrIntegerOperation::DivideTrap, true) => "sdiv",
                    (IrIntegerOperation::DivideTrap, false) => "udiv",
                    (IrIntegerOperation::RemainderTrap, true) => "srem",
                    (IrIntegerOperation::RemainderTrap, false) => "urem",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let trap_id = self.register_trap(trap)?;
                writeln!(
                    self.output,
                    "  br i1 %{failure}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:\n  {} = {opcode} {ty} {left}, {right}",
                    overflow_trap_label(result),
                    overflow_continue_label(result),
                    overflow_trap_label(result),
                    self.traps[trap_id].len(),
                    overflow_continue_label(result),
                    value_name(result),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::AbsoluteWrap
            | IrIntegerOperation::AbsoluteTrap
            | IrIntegerOperation::AbsoluteChecked => {
                let [argument] = arguments else {
                    return Err(BackendFailure::InvalidIr);
                };
                if !signed {
                    return Err(BackendFailure::InvalidIr);
                }
                let argument = value_name(*argument);
                let intrinsic = format!("llvm.abs.i{width}");
                self.intrinsics.insert(IntrinsicDeclaration::UnaryWithFlag {
                    name: intrinsic.clone(),
                    ty: ty.clone(),
                });
                match operation {
                    IrIntegerOperation::AbsoluteWrap => {
                        if result_type != operand_type || trap.is_some() {
                            return Err(BackendFailure::InvalidIr);
                        }
                        writeln!(
                            self.output,
                            "  {} = call {ty} @{intrinsic}({ty} {argument}, i1 false)",
                            value_name(result)
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                    IrIntegerOperation::AbsoluteTrap => {
                        if result_type != operand_type {
                            return Err(BackendFailure::InvalidIr);
                        }
                        let trap = trap.ok_or(BackendFailure::InvalidIr)?;
                        let overflow = self.next_temporary()?;
                        let trap_id = self.register_trap(trap)?;
                        let minimum = -(1_i128 << (width - 1));
                        writeln!(
                            self.output,
                            "  {} = call {ty} @{intrinsic}({ty} {argument}, i1 false)\n  %{overflow} = icmp eq {ty} {argument}, {minimum}\n  br i1 %{overflow}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
                            value_name(result),
                            overflow_trap_label(result),
                            overflow_continue_label(result),
                            overflow_trap_label(result),
                            self.traps[trap_id].len(),
                            overflow_continue_label(result),
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                    IrIntegerOperation::AbsoluteChecked => {
                        if trap.is_some() {
                            return Err(BackendFailure::InvalidIr);
                        }
                        let error_type =
                            self.checked_result_error_type(result_type, operand_type, &[0])?;
                        let absolute = self.next_temporary()?;
                        let overflow = self.next_temporary()?;
                        let ok_tag = self.next_temporary()?;
                        let ok_value = self.next_temporary()?;
                        let error_tag = self.next_temporary()?;
                        let error_value = self.next_temporary()?;
                        let result_ty = llvm_type(self.program, result_type)?;
                        let error_ty = llvm_type(self.program, error_type)?;
                        let minimum = -(1_i128 << (width - 1));
                        writeln!(
                            self.output,
                            "  %{absolute} = call {ty} @{intrinsic}({ty} {argument}, i1 false)\n  %{overflow} = icmp eq {ty} {argument}, {minimum}\n  %{ok_tag} = insertvalue {result_ty} zeroinitializer, i32 0, 0\n  %{ok_value} = insertvalue {result_ty} %{ok_tag}, {ty} %{absolute}, 1\n  %{error_tag} = insertvalue {result_ty} zeroinitializer, i32 1, 0\n  %{error_value} = insertvalue {result_ty} %{error_tag}, {error_ty} 0, 2\n  {} = select i1 %{overflow}, {result_ty} %{error_value}, {result_ty} %{ok_value}",
                            value_name(result)
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                    _ => return Err(BackendFailure::InvalidIr),
                }
            }
            IrIntegerOperation::BitAnd | IrIntegerOperation::BitOr | IrIntegerOperation::BitXor => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let opcode = match operation {
                    IrIntegerOperation::BitAnd => "and",
                    IrIntegerOperation::BitOr => "or",
                    IrIntegerOperation::BitXor => "xor",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                writeln!(
                    self.output,
                    "  {} = {opcode} {ty} {left}, {right}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::BitNot => {
                let argument = unary.as_deref().ok_or(BackendFailure::InvalidIr)?;
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                writeln!(
                    self.output,
                    "  {} = xor {ty} {argument}, -1",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::ShiftLeftWrap
            | IrIntegerOperation::ShiftRightWrap
            | IrIntegerOperation::RotateLeft
            | IrIntegerOperation::RotateRight => {
                let Some((value, amount)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let amount = self.emit_integer_amount(amount, width, true)?;
                match operation {
                    IrIntegerOperation::ShiftLeftWrap | IrIntegerOperation::ShiftRightWrap => {
                        let opcode = match operation {
                            IrIntegerOperation::ShiftLeftWrap => "shl",
                            IrIntegerOperation::ShiftRightWrap if signed => "ashr",
                            IrIntegerOperation::ShiftRightWrap => "lshr",
                            _ => return Err(BackendFailure::InvalidIr),
                        };
                        writeln!(
                            self.output,
                            "  {} = {opcode} {ty} {value}, {amount}",
                            value_name(result)
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                    IrIntegerOperation::RotateLeft | IrIntegerOperation::RotateRight => {
                        let stem = if operation == IrIntegerOperation::RotateLeft {
                            "fshl"
                        } else {
                            "fshr"
                        };
                        let intrinsic = format!("llvm.{stem}.i{width}");
                        self.intrinsics.insert(IntrinsicDeclaration::Ternary {
                            name: intrinsic.clone(),
                            ty: ty.clone(),
                        });
                        writeln!(
                            self.output,
                            "  {} = call {ty} @{intrinsic}({ty} {value}, {ty} {value}, {ty} {amount})",
                            value_name(result)
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                    _ => return Err(BackendFailure::InvalidIr),
                }
            }
            IrIntegerOperation::ShiftLeftTrap | IrIntegerOperation::ShiftRightTrap => {
                let Some((value, amount)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap = trap.ok_or(BackendFailure::InvalidIr)?;
                let out_of_range = self.next_temporary()?;
                let trap_id = self.register_trap(trap)?;
                writeln!(
                    self.output,
                    "  %{out_of_range} = icmp uge i32 {amount}, {width}\n  br i1 %{out_of_range}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
                    overflow_trap_label(result),
                    overflow_continue_label(result),
                    overflow_trap_label(result),
                    self.traps[trap_id].len(),
                    overflow_continue_label(result),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let amount = self.emit_integer_amount(amount, width, false)?;
                let opcode = match operation {
                    IrIntegerOperation::ShiftLeftTrap => "shl",
                    IrIntegerOperation::ShiftRightTrap if signed => "ashr",
                    IrIntegerOperation::ShiftRightTrap => "lshr",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                writeln!(
                    self.output,
                    "  {} = {opcode} {ty} {value}, {amount}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::PopulationCount
            | IrIntegerOperation::LeadingZeros
            | IrIntegerOperation::TrailingZeros => {
                let argument = unary.as_deref().ok_or(BackendFailure::InvalidIr)?;
                if result_type
                    != (IrType::Integer {
                        width: 32,
                        signed: false,
                    })
                    || trap.is_some()
                {
                    return Err(BackendFailure::InvalidIr);
                }
                let stem = match operation {
                    IrIntegerOperation::PopulationCount => "ctpop",
                    IrIntegerOperation::LeadingZeros => "ctlz",
                    IrIntegerOperation::TrailingZeros => "cttz",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let intrinsic = format!("llvm.{stem}.i{width}");
                if operation == IrIntegerOperation::PopulationCount {
                    self.intrinsics.insert(IntrinsicDeclaration::Unary {
                        name: intrinsic.clone(),
                        ty: ty.clone(),
                    });
                } else {
                    self.intrinsics.insert(IntrinsicDeclaration::UnaryWithFlag {
                        name: intrinsic.clone(),
                        ty: ty.clone(),
                    });
                }
                let count = if width == 32 {
                    value_name(result)
                } else {
                    format!("%{}", self.next_temporary()?)
                };
                let flag = if operation == IrIntegerOperation::PopulationCount {
                    String::new()
                } else {
                    ", i1 false".to_owned()
                };
                writeln!(
                    self.output,
                    "  {count} = call {ty} @{intrinsic}({ty} {argument}{flag})"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                if width < 32 {
                    writeln!(
                        self.output,
                        "  {} = zext {ty} {count} to i32",
                        value_name(result)
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                } else if width > 32 {
                    writeln!(
                        self.output,
                        "  {} = trunc {ty} {count} to i32",
                        value_name(result)
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
            }
            IrIntegerOperation::ByteSwap => {
                let argument = unary.as_deref().ok_or(BackendFailure::InvalidIr)?;
                if result_type != operand_type || width < 16 || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let intrinsic = format!("llvm.bswap.i{width}");
                self.intrinsics.insert(IntrinsicDeclaration::Unary {
                    name: intrinsic.clone(),
                    ty: ty.clone(),
                });
                writeln!(
                    self.output,
                    "  {} = call {ty} @{intrinsic}({ty} {argument})",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::MultiplyHigh => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let wide = u16::from(width) * 2;
                let extension = if signed { "sext" } else { "zext" };
                let shift = if signed { "ashr" } else { "lshr" };
                let wide_left = self.next_temporary()?;
                let wide_right = self.next_temporary()?;
                let product = self.next_temporary()?;
                let high = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{wide_left} = {extension} {ty} {left} to i{wide}\n  %{wide_right} = {extension} {ty} {right} to i{wide}\n  %{product} = mul i{wide} %{wide_left}, %{wide_right}\n  %{high} = {shift} i{wide} %{product}, {width}\n  {} = trunc i{wide} %{high} to {ty}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::AddSaturating
            | IrIntegerOperation::SubtractSaturating
            | IrIntegerOperation::Minimum
            | IrIntegerOperation::Maximum => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let stem = match operation {
                    IrIntegerOperation::AddSaturating => "add.sat",
                    IrIntegerOperation::SubtractSaturating => "sub.sat",
                    IrIntegerOperation::Minimum => "min",
                    IrIntegerOperation::Maximum => "max",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let sign = if signed { 's' } else { 'u' };
                let intrinsic = format!("llvm.{sign}{stem}.i{width}");
                self.intrinsics.insert(IntrinsicDeclaration::Binary {
                    name: intrinsic.clone(),
                    ty: ty.clone(),
                });
                writeln!(
                    self.output,
                    "  {} = call {ty} @{intrinsic}({ty} {left}, {ty} {right})",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            IrIntegerOperation::MultiplySaturating => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                self.emit_saturating_multiply(result, &ty, width, signed, left, right)?;
            }
            IrIntegerOperation::Equal
            | IrIntegerOperation::NotEqual
            | IrIntegerOperation::Less
            | IrIntegerOperation::LessEqual
            | IrIntegerOperation::Greater
            | IrIntegerOperation::GreaterEqual => {
                let Some((left, right)) = &binary else {
                    return Err(BackendFailure::InvalidIr);
                };
                if result_type != IrType::Bool || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let predicate = match operation {
                    IrIntegerOperation::Equal => "eq",
                    IrIntegerOperation::NotEqual => "ne",
                    IrIntegerOperation::Less if signed => "slt",
                    IrIntegerOperation::Less => "ult",
                    IrIntegerOperation::LessEqual if signed => "sle",
                    IrIntegerOperation::LessEqual => "ule",
                    IrIntegerOperation::Greater if signed => "sgt",
                    IrIntegerOperation::Greater => "ugt",
                    IrIntegerOperation::GreaterEqual if signed => "sge",
                    IrIntegerOperation::GreaterEqual => "uge",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                writeln!(
                    self.output,
                    "  {} = icmp {predicate} {ty} {left}, {right}",
                    value_name(result)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
        }
        let _ = block;
        Ok(())
    }

    fn emit_integer_amount(
        &mut self,
        amount: &str,
        width: u8,
        mask: bool,
    ) -> Result<String, BackendFailure> {
        let amount = if mask {
            let masked = self.next_temporary()?;
            writeln!(self.output, "  %{masked} = and i32 {amount}, {}", width - 1)
                .map_err(|_| BackendFailure::TextEmission)?;
            format!("%{masked}")
        } else {
            amount.to_owned()
        };
        match width {
            32 => Ok(amount),
            8 | 16 => {
                let narrowed = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{narrowed} = trunc i32 {amount} to i{width}"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                Ok(format!("%{narrowed}"))
            }
            64 => {
                let widened = self.next_temporary()?;
                writeln!(self.output, "  %{widened} = zext i32 {amount} to i64")
                    .map_err(|_| BackendFailure::TextEmission)?;
                Ok(format!("%{widened}"))
            }
            _ => Err(BackendFailure::InvalidIr),
        }
    }

    fn emit_saturating_multiply(
        &mut self,
        result: IrValueId,
        ty: &str,
        width: u8,
        signed: bool,
        left: &str,
        right: &str,
    ) -> Result<(), BackendFailure> {
        let wide = u16::from(width) * 2;
        let extension = if signed { "sext" } else { "zext" };
        let wide_left = self.next_temporary()?;
        let wide_right = self.next_temporary()?;
        let product = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{wide_left} = {extension} {ty} {left} to i{wide}\n  %{wide_right} = {extension} {ty} {right} to i{wide}\n  %{product} = mul i{wide} %{wide_left}, %{wide_right}"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let clamped = if signed {
            let minimum = -(1_i128 << (width - 1));
            let maximum = (1_i128 << (width - 1)) - 1;
            let below = self.next_temporary()?;
            let above = self.next_temporary()?;
            let lower = self.next_temporary()?;
            let clamped = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{below} = icmp slt i{wide} %{product}, {minimum}\n  %{above} = icmp sgt i{wide} %{product}, {maximum}\n  %{lower} = select i1 %{below}, i{wide} {minimum}, i{wide} %{product}\n  %{clamped} = select i1 %{above}, i{wide} {maximum}, i{wide} %{lower}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            clamped
        } else {
            let maximum = (1_u128 << width) - 1;
            let above = self.next_temporary()?;
            let clamped = self.next_temporary()?;
            writeln!(
                self.output,
                "  %{above} = icmp ugt i{wide} %{product}, {maximum}\n  %{clamped} = select i1 %{above}, i{wide} {maximum}, i{wide} %{product}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            clamped
        };
        writeln!(
            self.output,
            "  {} = trunc i{wide} %{clamped} to {ty}",
            value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    fn checked_result_error_type(
        &self,
        result_type: IrType,
        operand_type: IrType,
        expected_error_tags: &[u32],
    ) -> Result<IrType, BackendFailure> {
        let IrType::Nominal(result_nominal) = result_type else {
            return Err(BackendFailure::InvalidIr);
        };
        let IrNominalKind::Enum { variants } = self.nominal(result_nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        let [ok, error] = variants.as_slice() else {
            return Err(BackendFailure::InvalidIr);
        };
        if ok.tag() != 0
            || error.tag() != 1
            || ok.fields().len() != 1
            || error.fields().len() != 1
            || ok.fields()[0].ty() != operand_type
        {
            return Err(BackendFailure::InvalidIr);
        }
        let error_type = error.fields()[0].ty();
        let IrType::Nominal(error_nominal) = error_type else {
            return Err(BackendFailure::InvalidIr);
        };
        let IrNominalKind::Enum {
            variants: error_variants,
        } = self.nominal(error_nominal)?.kind()
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if !self.nominal(error_nominal)?.is_tag_only_enum()
            || error_variants.len() != expected_error_tags.len()
            || error_variants
                .iter()
                .zip(expected_error_tags)
                .any(|(variant, expected)| variant.tag() != *expected)
        {
            return Err(BackendFailure::InvalidIr);
        }
        Ok(error_type)
    }
}
