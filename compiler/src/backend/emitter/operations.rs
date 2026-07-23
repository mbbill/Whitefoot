use super::*;

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    pub(super) fn emit_constant(
        &mut self,
        result: IrValueId,
        ty: IrType,
        constant: IrConstant,
    ) -> Result<(), BackendFailure> {
        let rendered = constant_operand(constant, ty)?;
        let llvm_ty = llvm_type(self.program, ty)?;
        writeln!(
            self.output,
            "  {} = select i1 true, {llvm_ty} {rendered}, {llvm_ty} {rendered}",
            value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        function: u32,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let target = self
            .program
            .functions()
            .get(function as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        if target.result() != ty || target.parameters().len() != arguments.len() {
            return Err(BackendFailure::InvalidIr);
        }
        let mut rendered = Vec::with_capacity(arguments.len());
        for (argument, (_, parameter_type)) in arguments.iter().zip(target.parameters()) {
            if self.function.value_type(*argument) != Some(*parameter_type) {
                return Err(BackendFailure::InvalidIr);
            }
            rendered.push(format!(
                "{} {}",
                llvm_type(self.program, *parameter_type)?,
                value_name(*argument)
            ));
        }
        writeln!(
            self.output,
            "  {} = call {} @{}({})",
            value_name(result),
            llvm_type(self.program, ty)?,
            source_symbol(target.name()),
            rendered.join(", ")
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_integer(
        &mut self,
        block: IrBlockId,
        _index: usize,
        result: IrValueId,
        result_type: IrType,
        operation: IrIntegerOperation,
        operand_type: IrType,
        arguments: [IrValueId; 2],
        trap: Option<&IrTrapSite>,
    ) -> Result<(), BackendFailure> {
        let IrType::Integer { width, signed } = operand_type else {
            return Err(BackendFailure::InvalidIr);
        };
        if arguments
            .iter()
            .any(|argument| self.function.value_type(*argument) != Some(operand_type))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let ty = llvm_type(self.program, operand_type)?;
        let left = value_name(arguments[0]);
        let right = value_name(arguments[1]);
        match operation {
            IrIntegerOperation::AddWrap
            | IrIntegerOperation::SubtractWrap
            | IrIntegerOperation::MultiplyWrap => {
                if result_type != operand_type || trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let opcode = match operation {
                    IrIntegerOperation::AddWrap => "add",
                    IrIntegerOperation::SubtractWrap => "sub",
                    IrIntegerOperation::MultiplyWrap => "mul",
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
            | IrIntegerOperation::MultiplyTrap => {
                if result_type != operand_type {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap = trap.ok_or(BackendFailure::InvalidIr)?;
                let stem = match operation {
                    IrIntegerOperation::AddTrap => "add",
                    IrIntegerOperation::SubtractTrap => "sub",
                    IrIntegerOperation::MultiplyTrap => "mul",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let sign = if signed { 's' } else { 'u' };
                let intrinsic = format!("llvm.{sign}{stem}.with.overflow.i{width}");
                self.intrinsics.insert(format!("{intrinsic}|{ty}"));
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
            | IrIntegerOperation::MultiplyChecked => {
                let IrType::Nominal(result_nominal) = result_type else {
                    return Err(BackendFailure::InvalidIr);
                };
                if trap.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
                let error_type = {
                    let IrNominalKind::Enum { variants } = self.nominal(result_nominal)?.kind()
                    else {
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
                    error.fields()[0].ty()
                };
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
                    || !matches!(error_variants.as_slice(), [variant] if variant.tag() == 0)
                {
                    return Err(BackendFailure::InvalidIr);
                }

                let stem = match operation {
                    IrIntegerOperation::AddChecked => "add",
                    IrIntegerOperation::SubtractChecked => "sub",
                    IrIntegerOperation::MultiplyChecked => "mul",
                    _ => return Err(BackendFailure::InvalidIr),
                };
                let sign = if signed { 's' } else { 'u' };
                let intrinsic = format!("llvm.{sign}{stem}.with.overflow.i{width}");
                self.intrinsics.insert(format!("{intrinsic}|{ty}"));
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
            IrIntegerOperation::Equal
            | IrIntegerOperation::NotEqual
            | IrIntegerOperation::Less
            | IrIntegerOperation::LessEqual
            | IrIntegerOperation::Greater
            | IrIntegerOperation::GreaterEqual => {
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

    pub(super) fn emit_boolean(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: IrBooleanOperation,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Bool
            || arguments
                .iter()
                .any(|argument| self.function.value_type(*argument) != Some(IrType::Bool))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let (opcode, left, right) = match (operation, arguments) {
            (IrBooleanOperation::And, [left, right]) => ("and", *left, value_name(*right)),
            (IrBooleanOperation::Or, [left, right]) => ("or", *left, value_name(*right)),
            (IrBooleanOperation::ExclusiveOr, [left, right]) => ("xor", *left, value_name(*right)),
            (IrBooleanOperation::Not, [value]) => ("xor", *value, "true".to_owned()),
            _ => return Err(BackendFailure::InvalidIr),
        };
        writeln!(
            self.output,
            "  {} = {opcode} i1 {}, {right}",
            value_name(result),
            value_name(left)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_enum_equality(
        &mut self,
        result: IrValueId,
        ty: IrType,
        equal: bool,
        operand_type: IrType,
        arguments: [IrValueId; 2],
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Bool
            || !is_tag_only_type(self.program, operand_type)?
            || arguments
                .iter()
                .any(|argument| self.function.value_type(*argument) != Some(operand_type))
        {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(
            self.output,
            "  {} = icmp {} {} {}, {}",
            value_name(result),
            if equal { "eq" } else { "ne" },
            llvm_type(self.program, operand_type)?,
            value_name(arguments[0]),
            value_name(arguments[1])
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_struct(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        fields: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Nominal(nominal) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Struct {
            fields: declared_fields,
        } = self.nominal(nominal)?.kind()
        else {
            return Err(BackendFailure::InvalidIr);
        };
        if fields.len() != declared_fields.len() {
            return Err(BackendFailure::InvalidIr);
        }
        for (value, field) in fields.iter().zip(declared_fields) {
            if self.function.value_type(*value) != Some(field.ty()) {
                return Err(BackendFailure::InvalidIr);
            }
        }
        self.emit_insert_sequence(
            result,
            ty,
            fields
                .iter()
                .enumerate()
                .map(|(index, value)| (index, *value))
                .collect(),
        )
    }

    pub(super) fn emit_enum(
        &mut self,
        result: IrValueId,
        ty: IrType,
        nominal: IrNominalId,
        variant: u32,
        fields: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Nominal(nominal) {
            return Err(BackendFailure::InvalidIr);
        }
        let nominal_data = self.nominal(nominal)?;
        let IrNominalKind::Enum { variants } = nominal_data.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        let selected = variants
            .iter()
            .find(|candidate| candidate.tag() == variant)
            .ok_or(BackendFailure::InvalidIr)?;
        if fields.len() != selected.fields().len() {
            return Err(BackendFailure::InvalidIr);
        }
        for (value, field) in fields.iter().zip(selected.fields()) {
            if self.function.value_type(*value) != Some(field.ty()) {
                return Err(BackendFailure::InvalidIr);
            }
        }
        if nominal_data.is_tag_only_enum() {
            let llvm_ty = llvm_type(self.program, ty)?;
            writeln!(
                self.output,
                "  {} = or {llvm_ty} 0, {variant}",
                value_name(result)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }
        let mut inserts = vec![(0_usize, None)];
        let base = variant_field_base(variants, variant)?;
        inserts.extend(
            fields
                .iter()
                .enumerate()
                .map(|(index, value)| (base + index, Some(*value))),
        );
        self.emit_enum_insert_sequence(result, ty, variant, inserts)
    }

    pub(super) fn emit_insert_sequence(
        &mut self,
        result: IrValueId,
        ty: IrType,
        fields: Vec<(usize, IrValueId)>,
    ) -> Result<(), BackendFailure> {
        let aggregate_ty = llvm_type(self.program, ty)?;
        if fields.is_empty() {
            writeln!(
                self.output,
                "  {} = select i1 true, {aggregate_ty} zeroinitializer, {aggregate_ty} zeroinitializer",
                value_name(result)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }
        let mut base = "zeroinitializer".to_owned();
        let total = fields.len();
        for (ordinal, (index, value)) in fields.into_iter().enumerate() {
            let field_ty = self
                .function
                .value_type(value)
                .ok_or(BackendFailure::InvalidIr)?;
            let output = if ordinal + 1 == total {
                value_name(result)
            } else {
                format!("%{}", self.next_temporary()?)
            };
            writeln!(
                self.output,
                "  {output} = insertvalue {aggregate_ty} {base}, {} {}, {index}",
                llvm_type(self.program, field_ty)?,
                value_name(value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            base = output;
        }
        Ok(())
    }

    pub(super) fn emit_enum_insert_sequence(
        &mut self,
        result: IrValueId,
        ty: IrType,
        tag: u32,
        inserts: Vec<(usize, Option<IrValueId>)>,
    ) -> Result<(), BackendFailure> {
        let aggregate_ty = llvm_type(self.program, ty)?;
        let mut base = "zeroinitializer".to_owned();
        let total = inserts.len();
        for (ordinal, (index, value)) in inserts.into_iter().enumerate() {
            let output = if ordinal + 1 == total {
                value_name(result)
            } else {
                format!("%{}", self.next_temporary()?)
            };
            match value {
                Some(value) => {
                    let field_ty = self
                        .function
                        .value_type(value)
                        .ok_or(BackendFailure::InvalidIr)?;
                    writeln!(
                        self.output,
                        "  {output} = insertvalue {aggregate_ty} {base}, {} {}, {index}",
                        llvm_type(self.program, field_ty)?,
                        value_name(value)
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
                None => {
                    writeln!(
                        self.output,
                        "  {output} = insertvalue {aggregate_ty} {base}, i32 {tag}, {index}"
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
            }
            base = output;
        }
        Ok(())
    }

    pub(super) fn emit_struct_projection(
        &mut self,
        result: IrValueId,
        ty: IrType,
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        consume_root: bool,
    ) -> Result<(), BackendFailure> {
        if self.function.value_type(aggregate) != Some(IrType::Nominal(nominal)) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Struct { fields } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        if fields.get(field as usize).map(|field| field.ty()) != Some(ty) {
            return Err(BackendFailure::InvalidIr);
        }
        if consume_root {
            writeln!(self.output, "  ; ownership-consuming projection")
                .map_err(|_| BackendFailure::TextEmission)?;
        }
        writeln!(
            self.output,
            "  {} = extractvalue {} {}, {field}",
            value_name(result),
            llvm_type(self.program, IrType::Nominal(nominal))?,
            value_name(aggregate)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_struct_insertion(
        &mut self,
        result: IrValueId,
        ty: IrType,
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        value: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Nominal(nominal) || self.function.value_type(aggregate) != Some(ty) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Struct { fields } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        let field_ty = fields
            .get(field as usize)
            .map(|field| field.ty())
            .ok_or(BackendFailure::InvalidIr)?;
        if self.function.value_type(value) != Some(field_ty) {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(
            self.output,
            "  {} = insertvalue {} {}, {} {}, {field}",
            value_name(result),
            llvm_type(self.program, ty)?,
            value_name(aggregate),
            llvm_type(self.program, field_ty)?,
            value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn emit_variant_projection(
        &mut self,
        result: IrValueId,
        ty: IrType,
        aggregate: IrValueId,
        nominal: IrNominalId,
        variant: u32,
        field: u32,
    ) -> Result<(), BackendFailure> {
        if self.function.value_type(aggregate) != Some(IrType::Nominal(nominal)) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrNominalKind::Enum { variants } = self.nominal(nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        let selected = variants
            .iter()
            .find(|candidate| candidate.tag() == variant)
            .ok_or(BackendFailure::InvalidIr)?;
        if selected
            .fields()
            .get(field as usize)
            .map(|field| field.ty())
            != Some(ty)
        {
            return Err(BackendFailure::InvalidIr);
        }
        let index = variant_field_base(variants, variant)? + field as usize;
        writeln!(
            self.output,
            "  {} = extractvalue {} {}, {index}",
            value_name(result),
            llvm_type(self.program, IrType::Nominal(nominal))?,
            value_name(aggregate)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}
