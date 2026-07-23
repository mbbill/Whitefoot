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
