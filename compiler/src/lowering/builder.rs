use std::collections::HashMap;

mod buffers;
mod loops;
mod results;
mod storage;

use crate::CheckedProgram;
use crate::semantic::CheckedSetTarget;
use crate::semantic::{
    BindingId, CheckedArrayRoot, CheckedDrop, CheckedExpression, CheckedMatchArm, CheckedMode,
    CheckedNominalKind, CheckedParameter, CheckedProgramData, CheckedProjectedDrop,
    CheckedStatement, CheckedValue,
};

use super::*;
use loops::LoopTarget;
use storage::collect_addressed_bindings;

pub fn lower_checked<'classified, 'lexed, 'source>(
    checked: CheckedProgram<'classified, 'lexed, 'source>,
) -> Result<IrProgram<'classified, 'lexed, 'source>, LoweringFailure> {
    let nominals = lower_nominals(&checked.data)?;
    let constants = lower_constants(&checked.data)?;
    let functions = checked
        .data
        .functions
        .iter()
        .map(|function| lower_function(function, &nominals, &constants))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IrProgram {
        main: checked.data.main.0,
        _checked: checked,
        nominals,
        constants,
        functions,
    })
}

fn lower_scalar_constant(value: &CheckedValue) -> Result<IrConstant, LoweringFailure> {
    match value {
        CheckedValue::Unit => Ok(IrConstant::Unit),
        CheckedValue::Bool(value) => Ok(IrConstant::Bool(*value)),
        CheckedValue::Integer { ty, bits } => Ok(IrConstant::Integer {
            ty: lower_type(crate::semantic::CheckedType::Integer(*ty))?,
            bits: *bits,
        }),
        CheckedValue::Float { ty, bits } => Ok(IrConstant::Float {
            ty: lower_type(crate::semantic::CheckedType::Float(*ty))?,
            bits: *bits,
        }),
        CheckedValue::Array { .. } => Err(LoweringFailure::InvalidCheckedProgram),
    }
}

fn lower_constants(data: &CheckedProgramData) -> Result<Vec<IrGlobalConstant>, LoweringFailure> {
    data.constants
        .iter()
        .enumerate()
        .map(|(index, constant)| {
            if constant.id.0 as usize != index || constant.value.ty() != constant.ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            let value = match &constant.value {
                CheckedValue::Array { elements, .. } => IrGlobalValue::Array(
                    elements
                        .iter()
                        .map(lower_scalar_constant)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                scalar => IrGlobalValue::Scalar(lower_scalar_constant(scalar)?),
            };
            Ok(IrGlobalConstant {
                id: IrConstantId(constant.id.0),
                name: constant.name.clone(),
                ty: lower_type(constant.ty)?,
                value,
            })
        })
        .collect()
}

fn lower_nominals(data: &CheckedProgramData) -> Result<Vec<IrNominal>, LoweringFailure> {
    data.nominals
        .get(..data.executable_nominal_count)
        .ok_or(LoweringFailure::InvalidCheckedProgram)?
        .iter()
        .enumerate()
        .map(|(index, nominal)| {
            if nominal.id.0 as usize != index {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            let kind = match &nominal.kind {
                CheckedNominalKind::Struct { fields } => IrNominalKind::Struct {
                    fields: fields
                        .iter()
                        .map(|field| {
                            Ok(IrField {
                                ty: lower_type(field.ty)?,
                            })
                        })
                        .collect::<Result<Vec<_>, LoweringFailure>>()?,
                },
                CheckedNominalKind::Enum { variants } => IrNominalKind::Enum {
                    variants: variants
                        .iter()
                        .map(|variant| {
                            Ok(IrVariant {
                                tag: variant.tag,
                                fields: variant
                                    .fields
                                    .iter()
                                    .map(|field| {
                                        Ok(IrField {
                                            ty: lower_type(field.ty)?,
                                        })
                                    })
                                    .collect::<Result<Vec<_>, LoweringFailure>>()?,
                            })
                        })
                        .collect::<Result<Vec<_>, LoweringFailure>>()?,
                },
                CheckedNominalKind::Box { referent } => IrNominalKind::Box {
                    referent: lower_type(*referent)?,
                },
            };
            Ok(IrNominal {
                id: IrNominalId(
                    u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?,
                ),
                kind,
            })
        })
        .collect()
}

fn lower_function(
    function: &crate::semantic::CheckedFunction,
    nominals: &[IrNominal],
    constants: &[IrGlobalConstant],
) -> Result<IrFunction, LoweringFailure> {
    let addressed_bindings = collect_addressed_bindings(function);
    let mut builder = IrBuilder::new(
        nominals,
        constants,
        lower_type(function.result)?,
        addressed_bindings,
    )?;
    for parameter in &function.parameters {
        let ty = lower_parameter_type(parameter, nominals)?;
        let value = builder.new_value(ty)?;
        if builder.bindings.insert(parameter.binding, value).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        builder.parameters.push((value, ty));
        builder.promote_binding_if_needed(parameter.binding)?;
    }
    builder.lower_statements(&function.requires, None)?;
    builder.lower_statements(&function.body, None)?;
    if builder.current.is_some()
        || builder
            .blocks
            .iter()
            .any(|block| block.terminator.is_none())
    {
        return Err(LoweringFailure::InvalidCheckedProgram);
    }
    Ok(IrFunction {
        name: function.symbol.clone(),
        parameters: builder.parameters,
        result: lower_type(function.result)?,
        values: builder.values,
        blocks: builder
            .blocks
            .into_iter()
            .map(|block| {
                Ok(IrBlock {
                    parameters: block.parameters,
                    instructions: block.instructions,
                    terminator: block
                        .terminator
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                })
            })
            .collect::<Result<Vec<_>, LoweringFailure>>()?,
    })
}

fn lower_parameter_type(
    parameter: &CheckedParameter,
    nominals: &[IrNominal],
) -> Result<IrType, LoweringFailure> {
    let ty = lower_type(parameter.ty)?;
    if parameter.mode == CheckedMode::Own {
        return Ok(ty);
    }
    let IrType::Nominal(nominal) = ty else {
        return Ok(ty);
    };
    let nominal_data = nominals
        .get(nominal.index())
        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
    Ok(
        if matches!(nominal_data.kind, IrNominalKind::Struct { .. }) {
            IrType::NominalAddress(nominal)
        } else {
            ty
        },
    )
}

struct BuildingBlock {
    parameters: Vec<(IrValueId, IrType)>,
    instructions: Vec<IrInstruction>,
    terminator: Option<IrTerminator>,
}

struct IrBuilder<'program> {
    nominals: &'program [IrNominal],
    constants: &'program [IrGlobalConstant],
    bindings: HashMap<BindingId, IrValueId>,
    parameters: Vec<(IrValueId, IrType)>,
    values: Vec<IrType>,
    blocks: Vec<BuildingBlock>,
    current: Option<IrBlockId>,
    loops: Vec<LoopTarget>,
    result: IrType,
    addressed_bindings: std::collections::HashSet<BindingId>,
}

#[derive(Clone)]
struct GiveTarget {
    block: IrBlockId,
    result: IrType,
    carried_bindings: Vec<BindingId>,
}

impl<'program> IrBuilder<'program> {
    fn new(
        nominals: &'program [IrNominal],
        constants: &'program [IrGlobalConstant],
        result: IrType,
        addressed_bindings: std::collections::HashSet<BindingId>,
    ) -> Result<Self, LoweringFailure> {
        let mut builder = Self {
            nominals,
            constants,
            bindings: HashMap::new(),
            parameters: Vec::new(),
            values: Vec::new(),
            blocks: Vec::new(),
            current: None,
            loops: Vec::new(),
            result,
            addressed_bindings,
        };
        let (entry, parameters) = builder.new_block(&[])?;
        if !parameters.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        builder.current = Some(entry);
        Ok(builder)
    }

    fn new_value(&mut self, ty: IrType) -> Result<IrValueId, LoweringFailure> {
        let id = IrValueId(
            u32::try_from(self.values.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        self.values.push(ty);
        Ok(id)
    }

    fn new_block(
        &mut self,
        parameter_types: &[IrType],
    ) -> Result<(IrBlockId, Vec<IrValueId>), LoweringFailure> {
        let id = IrBlockId(
            u32::try_from(self.blocks.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        let mut parameters = Vec::with_capacity(parameter_types.len());
        let mut values = Vec::with_capacity(parameter_types.len());
        for ty in parameter_types {
            let value = self.new_value(*ty)?;
            parameters.push((value, *ty));
            values.push(value);
        }
        self.blocks.push(BuildingBlock {
            parameters,
            instructions: Vec::new(),
            terminator: None,
        });
        Ok((id, values))
    }

    fn current_block_mut(&mut self) -> Result<&mut BuildingBlock, LoweringFailure> {
        let current = self.current.ok_or(LoweringFailure::InvalidCheckedProgram)?;
        self.blocks
            .get_mut(current.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn terminate(&mut self, terminator: IrTerminator) -> Result<(), LoweringFailure> {
        let block = self.current_block_mut()?;
        if block.terminator.replace(terminator).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current = None;
        Ok(())
    }

    fn define(&mut self, ty: IrType, operation: IrOperation) -> Result<IrValueId, LoweringFailure> {
        let result = self.new_value(ty)?;
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::Define {
                result,
                ty,
                operation,
            });
        Ok(result)
    }

    fn lower_statements(
        &mut self,
        statements: &[CheckedStatement],
        give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        for statement in statements {
            if self.current.is_none() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            match statement {
                CheckedStatement::Let { binding, value } => {
                    let value = self.expression(value)?;
                    if self.bindings.insert(*binding, value).is_some() {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    self.promote_binding_if_needed(*binding)?;
                }
                CheckedStatement::PropagateLet {
                    binding,
                    scrutinee,
                    result_nominal,
                    return_nominal,
                    ok_type,
                    error_type,
                    error_drops,
                    context,
                } => self.lower_propagate(
                    *binding,
                    scrutinee,
                    *result_nominal,
                    *return_nominal,
                    *ok_type,
                    *error_type,
                    error_drops,
                    context,
                )?,
                CheckedStatement::Set { target, value } => self.set(target, value)?,
                CheckedStatement::Evaluate(expression) => {
                    self.expression(expression)?;
                }
                CheckedStatement::DropExpression(expression) => {
                    let value = self.expression(expression)?;
                    let drop = IrDrop {
                        value,
                        ty: self.value_type(value)?,
                    };
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Drop(drop));
                }
                CheckedStatement::Check { condition, trap } => {
                    let condition = self.expression(condition)?;
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Check {
                            condition,
                            trap: trap.clone().into(),
                        });
                }
                CheckedStatement::Return { value, drops } => {
                    let value = self.expression(value)?;
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Return { value, drops })?;
                }
                CheckedStatement::Give { value, drops } => {
                    let target = give_target
                        .as_ref()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let value = self.expression(value)?;
                    if self.value_type(value)? != target.result {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    let mut arguments = Vec::with_capacity(1 + target.carried_bindings.len());
                    arguments.push(value);
                    arguments.extend(self.binding_values(&target.carried_bindings)?);
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Jump {
                        target: target.block,
                        arguments,
                        drops,
                    })?;
                }
                CheckedStatement::Loop {
                    id,
                    body,
                    backedge_drops,
                } => self.lower_loop(*id, body, backedge_drops, give_target.clone())?,
                CheckedStatement::Break { target, drops } => {
                    let target = self
                        .loops
                        .iter()
                        .rev()
                        .find(|candidate| candidate.id == *target)
                        .cloned()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let arguments = self.binding_values(&target.carried_bindings)?;
                    let drops = self.lower_drops(drops)?;
                    self.terminate(IrTerminator::Jump {
                        target: target.block,
                        arguments,
                        drops,
                    })?;
                }
                CheckedStatement::Region {
                    body,
                    fallthrough_drops,
                } => {
                    self.lower_statements(body, give_target.clone())?;
                    if self.current.is_some() {
                        let drops = self.lower_drops(fallthrough_drops)?;
                        for drop in drops {
                            self.current_block_mut()?
                                .instructions
                                .push(IrInstruction::Drop(drop));
                        }
                    }
                }
                CheckedStatement::Match {
                    scrutinee,
                    enum_type,
                    arms,
                    continues,
                } => self.lower_match(
                    scrutinee,
                    *enum_type,
                    arms,
                    *continues,
                    None,
                    give_target.clone(),
                )?,
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    scrutinee,
                    enum_type,
                    arms,
                    continues,
                } => {
                    self.lower_match(
                        scrutinee,
                        *enum_type,
                        arms,
                        *continues,
                        Some((*binding, lower_type(*result_type)?)),
                        give_target.clone(),
                    )?;
                    if self.current.is_some() {
                        self.promote_binding_if_needed(*binding)?;
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_match(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        arms: &[CheckedMatchArm],
        continues: bool,
        value_binding: Option<(BindingId, IrType)>,
        outer_give_target: Option<GiveTarget>,
    ) -> Result<(), LoweringFailure> {
        let scrutinee = self.expression(scrutinee)?;
        let base_bindings = self.bindings.clone();
        let mut carried_bindings = base_bindings.keys().copied().collect::<Vec<_>>();
        carried_bindings.sort_by_key(|binding| binding.0);
        let join = if continues {
            let mut parameter_types =
                Vec::with_capacity(carried_bindings.len() + usize::from(value_binding.is_some()));
            if let Some((_, ty)) = value_binding {
                parameter_types.push(ty);
            }
            for binding in &carried_bindings {
                let value = base_bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                parameter_types.push(self.value_type(value)?);
            }
            let (block, parameters) = self.new_block(&parameter_types)?;
            Some((block, parameters))
        } else {
            None
        };
        let mut arm_blocks = Vec::with_capacity(arms.len());
        for _ in arms {
            arm_blocks.push(self.new_block(&[])?.0);
        }
        self.terminate(IrTerminator::Match {
            scrutinee,
            enum_type: enum_type.into(),
            targets: arms
                .iter()
                .zip(&arm_blocks)
                .map(|(arm, block)| IrMatchTarget {
                    tag: arm.tag,
                    block: *block,
                })
                .collect(),
        })?;
        for (arm, block) in arms.iter().zip(arm_blocks) {
            self.current = Some(block);
            self.bindings = base_bindings.clone();
            for binder in &arm.binders {
                let CheckedEnumType::Nominal(nominal) = enum_type else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.define(
                    lower_type(binder.ty)?,
                    IrOperation::ProjectVariant {
                        aggregate: scrutinee,
                        nominal: IrNominalId(nominal.0),
                        variant: arm.tag,
                        field: binder.field,
                    },
                )?;
                if self.bindings.insert(binder.binding, value).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if binder.mode == CheckedMode::Own {
                    self.promote_binding_if_needed(binder.binding)?;
                }
            }
            let arm_give_target = match value_binding {
                Some((_, ty)) => join.as_ref().map(|(block, _)| GiveTarget {
                    block: *block,
                    result: ty,
                    carried_bindings: carried_bindings.clone(),
                }),
                None => outer_give_target.clone(),
            };
            self.lower_statements(&arm.body, arm_give_target)?;
            if self.current.is_some() {
                let Some((join, _)) = &join else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if value_binding.is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let drops = self.lower_drops(&arm.fallthrough_drops)?;
                let arguments = self.binding_values(&carried_bindings)?;
                self.terminate(IrTerminator::Jump {
                    target: *join,
                    arguments,
                    drops,
                })?;
            }
        }
        self.bindings = base_bindings;
        if let Some((join, parameters)) = join {
            self.current = Some(join);
            let carried_start = usize::from(value_binding.is_some());
            if parameters.len() != carried_start + carried_bindings.len() {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            for (binding, value) in carried_bindings.iter().zip(&parameters[carried_start..]) {
                if self.bindings.insert(*binding, *value).is_none() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            }
            if let Some((binding, _)) = value_binding {
                let value = *parameters
                    .first()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if self.bindings.insert(binding, value).is_some() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            }
        } else {
            self.current = None;
        }
        Ok(())
    }

    fn expression(&mut self, expression: &CheckedExpression) -> Result<IrValueId, LoweringFailure> {
        match expression {
            CheckedExpression::Binding { binding, ty } => {
                let value = self
                    .bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let expected = lower_type(*ty)?;
                let actual = self.value_type(value)?;
                let value = if self.addressed_bindings.contains(binding) {
                    self.load_storage_value(value)?
                } else {
                    value
                };
                if self.value_type(value)? != expected
                    && !matches!(
                        (actual, expected),
                        (IrType::NominalAddress(left), IrType::Nominal(right)) if left == right
                    )
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::Constant(value) => {
                let ty = lower_type(value.ty())?;
                let constant = lower_scalar_constant(value)?;
                self.define(ty, IrOperation::Constant(constant))
            }
            CheckedExpression::UserCall {
                function,
                arguments,
                result,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(*result)?,
                    IrOperation::Call {
                        function: function.0,
                        arguments,
                    },
                )
            }
            CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                trap,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::Integer {
                        operation: (*operation).into(),
                        operand_type: lower_type(*operand_type)?,
                        arguments,
                        trap: trap.clone().map(Into::into),
                    },
                )
            }
            CheckedExpression::FloatOperation {
                operation,
                operand_type,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::Float {
                        operation: (*operation).into(),
                        operand_type: lower_type(CheckedType::Float(*operand_type))?,
                        arguments,
                    },
                )
            }
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                self.define(
                    lower_type(expression.ty())?,
                    IrOperation::NumericConversion {
                        source_type: lower_numeric_type(*source),
                        destination_type: lower_numeric_type(*destination),
                        value,
                    },
                )
            }
            CheckedExpression::BooleanOperation {
                operation,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.define(
                    IrType::Bool,
                    IrOperation::Boolean {
                        operation: (*operation).into(),
                        arguments,
                    },
                )
            }
            CheckedExpression::EnumEquality {
                equal,
                operand_type,
                arguments,
            } => {
                let [left, right] = arguments.as_slice() else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.define(
                    IrType::Bool,
                    IrOperation::EnumEquality {
                        equal: *equal,
                        operand_type: lower_type(*operand_type)?,
                        arguments: [left, right],
                    },
                )
            }
            CheckedExpression::ArrayFill {
                ty,
                value,
                target_domain,
            } => {
                let IrType::Array { element, .. } = lower_type(*ty)? else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let value = self.expression(value)?;
                if self.value_type(value)? != element.ty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    lower_type(*ty)?,
                    IrOperation::ArrayFill {
                        value,
                        target_domain: (*target_domain).into(),
                    },
                )
            }
            CheckedExpression::ArrayLength { root, length } => {
                let (_, ty) = self.array_root(*root)?;
                let IrType::Array { length: actual, .. } = ty else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let length = length
                    .value()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if actual != length {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    IrType::Integer {
                        width: 64,
                        signed: false,
                    },
                    IrOperation::Constant(IrConstant::Integer {
                        ty: IrType::Integer {
                            width: 64,
                            signed: false,
                        },
                        bits: length,
                    }),
                )
            }
            CheckedExpression::ArrayIndex {
                root,
                element_type,
                length,
                offset,
                trap,
                target_domain,
            } => {
                let (root, ty) = self.array_root(*root)?;
                let IrType::Array {
                    element,
                    length: actual,
                } = ty
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let length = length
                    .value()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                if element.ty() != lower_type(*element_type)? || actual != length {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let offset = self.expression(offset)?;
                if self.value_type(offset)?
                    != (IrType::Integer {
                        width: 64,
                        signed: false,
                    })
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    element.ty(),
                    IrOperation::ArrayIndex {
                        root,
                        offset,
                        trap: trap.clone().into(),
                        target_domain: (*target_domain).into(),
                    },
                )
            }
            CheckedExpression::BufferFill {
                element,
                length,
                value,
                trap,
                target_domains,
            } => self.lower_buffer_fill(*element, length, value, trap, *target_domains),
            CheckedExpression::BufferLength { root } => self.lower_buffer_length(root),
            CheckedExpression::BufferIndex {
                root,
                offset,
                trap,
                target_domain,
            } => self.lower_buffer_index(root, offset, trap, *target_domain),
            CheckedExpression::BoxNew { nominal, value } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::BoxNew { nominal, value },
                )
            }
            CheckedExpression::BoxDeref { nominal, value, .. } => {
                let value = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                let IrNominalKind::Box { referent } = self
                    .nominals
                    .get(nominal.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .kind
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                self.define(referent, IrOperation::BoxDeref { nominal, value })
            }
            CheckedExpression::BorrowBuffer { root } => self.lower_buffer_borrow(root),
            CheckedExpression::BorrowStruct { binding, nominal } => {
                self.lower_struct_borrow(*binding, IrNominalId(nominal.0))
            }
            CheckedExpression::BorrowBox { binding, nominal } => {
                let value = self.binding_value(*binding)?;
                if self.value_type(value)? != IrType::Nominal(IrNominalId(nominal.0)) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::ReborrowStruct { binding, nominal } => {
                self.lower_struct_borrow(*binding, IrNominalId(nominal.0))
            }
            CheckedExpression::ConstructStruct { nominal, fields } => {
                let fields = fields
                    .iter()
                    .map(|field| self.expression(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::ConstructStruct { nominal, fields },
                )
            }
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
            } => {
                let fields = fields
                    .iter()
                    .map(|field| self.expression(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    IrType::Nominal(nominal),
                    IrOperation::ConstructEnum {
                        nominal,
                        variant: *variant,
                        fields,
                    },
                )
            }
            CheckedExpression::Project {
                binding,
                fields,
                ty,
                consume_root,
                residual_drops,
            } => {
                let root = self.binding_value(*binding)?;
                let mut lowered_drops = Vec::with_capacity(residual_drops.len());
                for drop in residual_drops {
                    lowered_drops.push(self.lower_projected_drop(root, drop)?);
                }
                let value = self.project_struct_path(root, fields, *consume_root)?;
                if self.value_type(value)? != lower_type(*ty)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                for drop in lowered_drops {
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Drop(drop));
                }
                Ok(value)
            }
            CheckedExpression::ProjectValue {
                value,
                nominal,
                field,
                ty,
            } => {
                let aggregate = self.expression(value)?;
                let nominal = IrNominalId(nominal.0);
                self.define(
                    lower_type(*ty)?,
                    IrOperation::ProjectStruct {
                        aggregate,
                        nominal,
                        field: *field,
                        consume_root: false,
                    },
                )
            }
        }
    }

    fn array_root(&self, root: CheckedArrayRoot) -> Result<(IrArrayRoot, IrType), LoweringFailure> {
        match root {
            CheckedArrayRoot::Binding(binding) => {
                let value = self
                    .bindings
                    .get(&binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                Ok((IrArrayRoot::Value(value), self.value_type(value)?))
            }
            CheckedArrayRoot::Constant(constant) => {
                let constant = self
                    .constants
                    .get(constant.0 as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                Ok((
                    IrArrayRoot::Constant(IrConstantId(constant.id().0)),
                    constant.ty(),
                ))
            }
        }
    }

    fn set(
        &mut self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
    ) -> Result<(), LoweringFailure> {
        let binding = target.binding();
        let storage = self
            .bindings
            .get(&binding)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let root = self.load_storage_value(storage)?;
        let replacement = match target {
            CheckedSetTarget::Place(target) => {
                let value = self.expression(value)?;
                if self.value_type(value)? != lower_type(target.ty)? {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                if target.fields.is_empty() {
                    if self.value_type(root)? != self.value_type(value)? {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    value
                } else {
                    self.replace_struct_path(root, &target.fields, value)?
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let array_type = lower_type(target.array_type)?;
                let IrType::Array { element, length } = array_type else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if self.value_type(root)? != array_type
                    || element.ty() != lower_type(target.element_type)?
                    || Some(length) != target.length.value()
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let offset = self.expression(&target.offset)?;
                if self.value_type(offset)?
                    != (IrType::Integer {
                        width: 64,
                        signed: false,
                    })
                {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let index = self.define(
                    IrType::GuardedArrayIndex { length },
                    IrOperation::ArrayBoundsCheck {
                        offset,
                        trap: target.trap.clone().into(),
                        target_domain: target.target_domain.into(),
                    },
                )?;
                let value = self.expression(value)?;
                if self.value_type(value)? != element.ty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(
                    array_type,
                    IrOperation::InsertArray {
                        aggregate: root,
                        index,
                        value,
                    },
                )?
            }
            CheckedSetTarget::BufferIndex(target) => self.lower_buffer_set(root, target, value)?,
        };
        let stored = match self.value_type(storage)? {
            IrType::NominalAddress(nominal) => {
                if self.value_type(replacement)? != IrType::Nominal(nominal) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.store_nominal(storage, replacement, nominal)?;
                storage
            }
            _ => replacement,
        };
        if self.bindings.insert(binding, stored) != Some(storage) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    fn project_struct_path(
        &mut self,
        mut value: IrValueId,
        fields: &[u32],
        consume_root: bool,
    ) -> Result<IrValueId, LoweringFailure> {
        if fields.is_empty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        for field in fields {
            let IrType::Nominal(nominal) = self.value_type(value)? else {
                return Err(LoweringFailure::InvalidCheckedProgram);
            };
            let field_ty = match &self
                .nominals
                .get(nominal.index())
                .ok_or(LoweringFailure::InvalidCheckedProgram)?
                .kind
            {
                IrNominalKind::Struct { fields } => {
                    fields
                        .get(*field as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .ty
                }
                IrNominalKind::Enum { .. } | IrNominalKind::Box { .. } => {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
            };
            value = self.define(
                field_ty,
                IrOperation::ProjectStruct {
                    aggregate: value,
                    nominal,
                    field: *field,
                    consume_root,
                },
            )?;
        }
        Ok(value)
    }

    fn replace_struct_path(
        &mut self,
        aggregate: IrValueId,
        fields: &[u32],
        replacement: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let Some((field, remaining)) = fields.split_first() else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let IrType::Nominal(nominal) = self.value_type(aggregate)? else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let field_ty = match &self
            .nominals
            .get(nominal.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .kind
        {
            IrNominalKind::Struct { fields } => {
                fields
                    .get(*field as usize)
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?
                    .ty
            }
            IrNominalKind::Enum { .. } | IrNominalKind::Box { .. } => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        let value = if remaining.is_empty() {
            if self.value_type(replacement)? != field_ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            replacement
        } else {
            let selected = self.define(
                field_ty,
                IrOperation::ProjectStruct {
                    aggregate,
                    nominal,
                    field: *field,
                    consume_root: false,
                },
            )?;
            self.replace_struct_path(selected, remaining, replacement)?
        };
        self.define(
            IrType::Nominal(nominal),
            IrOperation::InsertStruct {
                aggregate,
                nominal,
                field: *field,
                value,
            },
        )
    }

    fn lower_projected_drop(
        &mut self,
        root: IrValueId,
        drop: &CheckedProjectedDrop,
    ) -> Result<IrDrop, LoweringFailure> {
        let value = self.project_struct_path(root, &drop.fields, false)?;
        let ty = lower_type(drop.ty)?;
        if self.value_type(value)? != ty {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(IrDrop { value, ty })
    }

    fn value_type(&self, value: IrValueId) -> Result<IrType, LoweringFailure> {
        self.values
            .get(value.index())
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn binding_values(&self, bindings: &[BindingId]) -> Result<Vec<IrValueId>, LoweringFailure> {
        bindings
            .iter()
            .map(|binding| {
                self.bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
            })
            .collect()
    }

    fn lower_drops(&mut self, drops: &[CheckedDrop]) -> Result<Vec<IrDrop>, LoweringFailure> {
        let mut lowered = Vec::with_capacity(drops.len());
        for drop in drops {
            let root = self.binding_value(drop.binding)?;
            let value = if drop.fields.is_empty() {
                root
            } else {
                self.project_struct_path(root, &drop.fields, false)?
            };
            let ty = lower_type(drop.ty)?;
            if self.value_type(value)? != ty {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
            lowered.push(IrDrop { value, ty });
        }
        Ok(lowered)
    }
}
