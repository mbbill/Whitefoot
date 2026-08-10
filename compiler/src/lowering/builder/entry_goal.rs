use crate::semantic::{
    CheckedBooleanOperation, CheckedConst, CheckedFunction, CheckedMode, CheckedNominalKind,
    CheckedProgramData, CheckedType, GoalDatum, GoalExpression, GoalOperation, GoalProjection,
    IntegerType,
};

use super::super::{
    IrBooleanOperation, IrConstant, IrEntryGoal, IrEntryGoalDefinition, IrIntegerOperation,
    IrNominal, IrNominalId, IrNominalKind, IrOperation, IrType, IrValueId, LoweringFailure,
    lower_numeric_type, lower_type,
};
use super::{lower_parameter_type, lower_scalar_constant};

pub(super) fn lower_entry_goal(
    data: &CheckedProgramData,
    nominals: &[IrNominal],
) -> Result<Option<IrEntryGoal>, LoweringFailure> {
    let main = data
        .functions
        .get(data.main.0 as usize)
        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
    if main.id != data.main {
        return Err(LoweringFailure::InvalidCheckedProgram);
    }
    let Some(requirement) = &main.requirement else {
        return Ok(None);
    };

    EntryGoalBuilder::new(data, main, nominals)?
        .lower(&requirement.template.root, requirement.trap.clone().into())
}

struct EntryGoalBuilder<'program> {
    data: &'program CheckedProgramData,
    function: &'program CheckedFunction,
    nominals: &'program [IrNominal],
    inputs: Vec<(IrValueId, IrType)>,
    values: Vec<IrType>,
    definitions: Vec<IrEntryGoalDefinition>,
}

impl<'program> EntryGoalBuilder<'program> {
    fn new(
        data: &'program CheckedProgramData,
        function: &'program CheckedFunction,
        nominals: &'program [IrNominal],
    ) -> Result<Self, LoweringFailure> {
        let mut builder = Self {
            data,
            function,
            nominals,
            inputs: Vec::with_capacity(function.parameters.len()),
            values: Vec::with_capacity(function.parameters.len()),
            definitions: Vec::new(),
        };
        for parameter in &function.parameters {
            let ty = lower_parameter_type(parameter, nominals)?;
            let value = builder.new_value(ty)?;
            builder.inputs.push((value, ty));
        }
        Ok(builder)
    }

    fn lower(
        mut self,
        root: &GoalExpression,
        trap: super::super::IrTrapSite,
    ) -> Result<Option<IrEntryGoal>, LoweringFailure> {
        if root.ty() != CheckedType::Bool {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let condition = self.expression(root)?;
        if self.value_type(condition)? != IrType::Bool {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(Some(IrEntryGoal {
            inputs: self.inputs,
            values: self.values,
            definitions: self.definitions,
            condition,
            trap,
        }))
    }

    fn expression(&mut self, expression: &GoalExpression) -> Result<IrValueId, LoweringFailure> {
        match expression {
            GoalExpression::Datum(datum) => self.datum(datum),
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            } => {
                if type_arguments.iter().any(|ty| !ty.is_concrete())
                    || const_arguments.iter().any(|value| !value.is_concrete())
                    || !result.is_concrete()
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.operation(*row, type_arguments, const_arguments, *result, arguments)
            }
        }
    }

    fn operation(
        &mut self,
        row: GoalOperation,
        type_arguments: &[CheckedType],
        const_arguments: &[CheckedConst],
        result: CheckedType,
        arguments: &[GoalExpression],
    ) -> Result<IrValueId, LoweringFailure> {
        match row {
            GoalOperation::Integer {
                operation,
                operand_type,
            } => {
                if operation.traps()
                    || !operation.accepts_operand_type(operand_type)
                    || !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || operation.scalar_result_type(operand_type) != Some(result)
                    || operation.operand_count() != arguments.len()
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let lowered = self.arguments(arguments)?;
                for (index, value) in lowered.iter().copied().enumerate() {
                    let expected = operation
                        .argument_type(operand_type, index)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    self.expect_type(value, expected)?;
                }
                self.define(
                    lower_type(result)?,
                    IrOperation::Integer {
                        operation: IrIntegerOperation::from(operation),
                        operand_type: lower_type(operand_type)?,
                        arguments: lowered,
                        trap: None,
                    },
                )
            }
            GoalOperation::Float {
                operation,
                operand_type,
            } => {
                let type_arguments_match = if matches!(
                    operation,
                    crate::semantic::CheckedFloatOperation::Infinity
                        | crate::semantic::CheckedFloatOperation::Nan
                ) {
                    type_arguments == [operand_type]
                } else {
                    type_arguments.is_empty()
                };
                if !type_arguments_match
                    || !const_arguments.is_empty()
                    || operation.result_type(operand_type) != result
                    || operation.operand_count() != arguments.len()
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let lowered = self.arguments(arguments)?;
                for value in &lowered {
                    self.expect_type(*value, operand_type)?;
                }
                self.define(
                    lower_type(result)?,
                    IrOperation::Float {
                        operation: operation.into(),
                        operand_type: lower_type(operand_type)?,
                        arguments: lowered,
                    },
                )
            }
            GoalOperation::NumericConversion {
                source,
                destination,
            } => {
                if type_arguments != [source.ty(), destination.ty()]
                    || !const_arguments.is_empty()
                    || result != destination.ty()
                    || source == destination
                    || !source.converts_totally_to(destination)
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [argument] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.expression(argument)?;
                self.expect_type(value, source.ty())?;
                self.define(
                    lower_numeric_type(destination),
                    IrOperation::NumericConversion {
                        source_type: lower_numeric_type(source),
                        destination_type: lower_numeric_type(destination),
                        value,
                    },
                )
            }
            GoalOperation::Reinterpret {
                source,
                destination,
            } => {
                if type_arguments != [source.ty(), destination.ty()]
                    || !const_arguments.is_empty()
                    || result != destination.ty()
                    || !source.reinterprets_to(destination)
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [argument] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.expression(argument)?;
                self.expect_type(value, source.ty())?;
                self.define(
                    lower_numeric_type(destination),
                    IrOperation::Reinterpret {
                        source_type: lower_numeric_type(source),
                        destination_type: lower_numeric_type(destination),
                        value,
                    },
                )
            }
            GoalOperation::Boolean(operation) => {
                let expected = match operation {
                    CheckedBooleanOperation::Not => 1,
                    CheckedBooleanOperation::And
                    | CheckedBooleanOperation::Or
                    | CheckedBooleanOperation::ExclusiveOr => 2,
                };
                if !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || result != CheckedType::Bool
                    || arguments.len() != expected
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let lowered = self.arguments(arguments)?;
                for value in &lowered {
                    self.expect_type(*value, CheckedType::Bool)?;
                }
                self.define(
                    IrType::Bool,
                    IrOperation::Boolean {
                        operation: IrBooleanOperation::from(operation),
                        arguments: lowered,
                    },
                )
            }
            GoalOperation::EnumEquality {
                equal,
                operand_type,
            } => {
                if !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || result != CheckedType::Bool
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [left, right] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.expect_type(left, operand_type)?;
                self.expect_type(right, operand_type)?;
                self.define(
                    IrType::Bool,
                    IrOperation::EnumEquality {
                        equal,
                        operand_type: lower_type(operand_type)?,
                        arguments: [left, right],
                    },
                )
            }
            // `array_new` may appear in a body-origin expansion but never in
            // a declaration GoalTemplate: FN-8 requires clause locals to be
            // copy values. Keep that provenance boundary explicit here.
            GoalOperation::ArrayFill { .. } => Err(LoweringFailure::InvalidCheckedProgram),
            GoalOperation::ArrayLength { element, length } => {
                if !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || result != CheckedType::Integer(IntegerType::U64)
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [argument] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let expected = CheckedType::Array { element, length };
                if argument.ty() != expected {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.validate_static_datum(argument)?;
                let length = length
                    .value()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let ty = IrType::Integer {
                    width: 64,
                    signed: false,
                };
                self.define(
                    ty,
                    IrOperation::Constant(IrConstant::Integer { ty, bits: length }),
                )
            }
            GoalOperation::BufferLength { element } => {
                if !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || result != CheckedType::Integer(IntegerType::U64)
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [argument] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if argument.ty() != (CheckedType::Buffer { element }) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let buffer = self.expression(argument)?;
                self.define(lower_type(result)?, IrOperation::BufferLength { buffer })
            }
            GoalOperation::SliceLength { region, element } => {
                if !type_arguments.is_empty()
                    || !const_arguments.is_empty()
                    || result != CheckedType::Integer(IntegerType::U64)
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let [argument] = arguments else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if argument.ty() != (CheckedType::Slice { region, element }) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let slice = self.expression(argument)?;
                self.define(lower_type(result)?, IrOperation::SliceLength { slice })
            }
        }
    }

    fn arguments(
        &mut self,
        arguments: &[GoalExpression],
    ) -> Result<Vec<IrValueId>, LoweringFailure> {
        arguments
            .iter()
            .map(|argument| self.expression(argument))
            .collect()
    }

    fn datum(&mut self, datum: &GoalDatum) -> Result<IrValueId, LoweringFailure> {
        match datum {
            GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            } => {
                let parameter = self
                    .function
                    .parameters
                    .get(*ordinal as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let (value, input_ty) = self
                    .inputs
                    .get(*ordinal as usize)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if input_ty != lower_parameter_type(parameter, self.nominals)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.project(
                    value,
                    parameter.ty,
                    parameter.mode != CheckedMode::Own,
                    projections,
                    *ty,
                )
            }
            GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            } => {
                let mut matches = self
                    .data
                    .constants
                    .iter()
                    .filter(|constant| constant.declaration == *declaration);
                let constant = matches
                    .next()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if matches.next().is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if projections.is_empty() {
                    if constant.ty != *ty || constant.value.ty() != *ty {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    let lowered = lower_scalar_constant(&constant.value)?;
                    return self.define(lower_type(*ty)?, IrOperation::Constant(lowered));
                }
                let lowered = lower_scalar_constant(&constant.value)?;
                let value =
                    self.define(lower_type(constant.ty)?, IrOperation::Constant(lowered))?;
                self.project(value, constant.ty, false, projections, *ty)
            }
            GoalDatum::Literal(value) => {
                let ty = lower_type(value.ty())?;
                self.define(ty, IrOperation::Constant(lower_scalar_constant(value)?))
            }
            GoalDatum::Place { .. } | GoalDatum::EphemeralActual { .. } => {
                Err(LoweringFailure::InvalidCheckedProgram)
            }
        }
    }

    fn validate_static_datum(&self, expression: &GoalExpression) -> Result<(), LoweringFailure> {
        match expression {
            GoalExpression::Datum(GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            }) => {
                let parameter = self
                    .function
                    .parameters
                    .get(*ordinal as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if parameter.mode != CheckedMode::Own
                    || !projections.is_empty()
                    || parameter.ty != *ty
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(())
            }
            GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) => {
                let mut matches = self
                    .data
                    .constants
                    .iter()
                    .filter(|constant| constant.declaration == *declaration);
                let constant = matches
                    .next()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if matches.next().is_some()
                    || !projections.is_empty()
                    || constant.ty != *ty
                    || constant.value.ty() != *ty
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(())
            }
            _ => Err(LoweringFailure::InvalidCheckedProgram),
        }
    }

    fn project(
        &mut self,
        mut value: IrValueId,
        mut checked_ty: CheckedType,
        mut holder_pending: bool,
        projections: &[GoalProjection],
        final_ty: CheckedType,
    ) -> Result<IrValueId, LoweringFailure> {
        for projection in projections {
            match projection {
                GoalProjection::Deref if holder_pending => {
                    if let IrType::Address(referent) = self.value_type(value)? {
                        value = self.define(
                            referent.ty(),
                            IrOperation::Load {
                                address: value,
                                referent,
                            },
                        )?;
                    }
                    holder_pending = false;
                }
                GoalProjection::Deref => {
                    let CheckedType::Nominal(id) = checked_ty else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let nominal = self
                        .data
                        .nominals
                        .get(id.0 as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let CheckedNominalKind::Box { referent } = nominal.kind else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let ir_nominal = IrNominalId(id.0);
                    if self.value_type(value)? != IrType::Nominal(ir_nominal) {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    value = self.define(
                        lower_type(referent)?,
                        IrOperation::BoxDeref {
                            nominal: ir_nominal,
                            value,
                        },
                    )?;
                    checked_ty = referent;
                }
                GoalProjection::Field(field) => {
                    if holder_pending {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    let CheckedType::Nominal(id) = checked_ty else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let nominal = self
                        .data
                        .nominals
                        .get(id.0 as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let CheckedNominalKind::Struct { fields } = &nominal.kind else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    let field_ty = fields
                        .get(*field as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .ty;
                    let ir_nominal = IrNominalId(id.0);
                    let Some(IrNominal {
                        kind: IrNominalKind::Struct { fields: ir_fields },
                        ..
                    }) = self.nominals.get(id.0 as usize)
                    else {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    };
                    if ir_fields.get(*field as usize).map(|field| field.ty())
                        != Some(lower_type(field_ty)?)
                    {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    value = self.define(
                        lower_type(field_ty)?,
                        IrOperation::ProjectStruct {
                            aggregate: value,
                            nominal: ir_nominal,
                            field: *field,
                            consume_root: false,
                        },
                    )?;
                    checked_ty = field_ty;
                }
            }
        }
        if holder_pending
            || checked_ty != final_ty
            || self.value_type(value)? != lower_type(final_ty)?
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }

    fn expect_type(&self, value: IrValueId, expected: CheckedType) -> Result<(), LoweringFailure> {
        if self.value_type(value)? != lower_type(expected)? {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    fn value_type(&self, value: IrValueId) -> Result<IrType, LoweringFailure> {
        self.values
            .get(value.0 as usize)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn new_value(&mut self, ty: IrType) -> Result<IrValueId, LoweringFailure> {
        let ordinal =
            u32::try_from(self.values.len()).map_err(|_| LoweringFailure::CounterOverflow)?;
        self.values.push(ty);
        Ok(IrValueId(ordinal))
    }

    fn define(&mut self, ty: IrType, operation: IrOperation) -> Result<IrValueId, LoweringFailure> {
        let result = self.new_value(ty)?;
        self.definitions.push(IrEntryGoalDefinition {
            result,
            ty,
            operation,
        });
        Ok(result)
    }
}
