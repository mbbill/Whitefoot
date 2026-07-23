use std::collections::HashMap;

mod loops;
mod results;

use crate::CheckedProgram;
use crate::semantic::{
    BindingId, CheckedDrop, CheckedExpression, CheckedMatchArm, CheckedNominalKind,
    CheckedProgramData, CheckedProjectedDrop, CheckedStatement, CheckedValue,
};

use super::*;
use loops::LoopTarget;

pub fn lower_checked_v0_12<'classified, 'lexed, 'source>(
    checked: CheckedProgram<'classified, 'lexed, 'source>,
) -> Result<IrProgram<'classified, 'lexed, 'source>, LoweringFailure> {
    let nominals = lower_nominals(&checked.data)?;
    let functions = checked
        .data
        .functions
        .iter()
        .map(|function| lower_function(function, &nominals))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IrProgram {
        main: checked.data.main.0,
        _checked: checked,
        nominals,
        functions,
    })
}

fn lower_nominals(data: &CheckedProgramData) -> Result<Vec<IrNominal>, LoweringFailure> {
    data.nominals
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
                        .map(|field| IrField {
                            ty: lower_type(field.ty),
                        })
                        .collect(),
                },
                CheckedNominalKind::Enum { variants } => IrNominalKind::Enum {
                    variants: variants
                        .iter()
                        .map(|variant| IrVariant {
                            tag: variant.tag,
                            fields: variant
                                .fields
                                .iter()
                                .map(|field| IrField {
                                    ty: lower_type(field.ty),
                                })
                                .collect(),
                        })
                        .collect(),
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
) -> Result<IrFunction, LoweringFailure> {
    let mut builder = IrBuilder::new(nominals, lower_type(function.result))?;
    for parameter in &function.parameters {
        let ty = lower_type(parameter.ty);
        let value = builder.new_value(ty)?;
        if builder.bindings.insert(parameter.binding, value).is_some() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        builder.parameters.push((value, ty));
    }
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
        name: function.name.clone(),
        parameters: builder.parameters,
        result: lower_type(function.result),
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

struct BuildingBlock {
    parameters: Vec<(IrValueId, IrType)>,
    instructions: Vec<IrInstruction>,
    terminator: Option<IrTerminator>,
}

struct IrBuilder<'nominals> {
    nominals: &'nominals [IrNominal],
    bindings: HashMap<BindingId, IrValueId>,
    parameters: Vec<(IrValueId, IrType)>,
    values: Vec<IrType>,
    blocks: Vec<BuildingBlock>,
    current: Option<IrBlockId>,
    loops: Vec<LoopTarget>,
    result: IrType,
}

#[derive(Clone)]
struct GiveTarget {
    block: IrBlockId,
    result: IrType,
    carried_bindings: Vec<BindingId>,
}

impl<'nominals> IrBuilder<'nominals> {
    fn new(nominals: &'nominals [IrNominal], result: IrType) -> Result<Self, LoweringFailure> {
        let mut builder = Self {
            nominals,
            bindings: HashMap::new(),
            parameters: Vec::new(),
            values: Vec::new(),
            blocks: Vec::new(),
            current: None,
            loops: Vec::new(),
            result,
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
                CheckedStatement::Set { target, value } => {
                    let root = self
                        .bindings
                        .get(&target.binding)
                        .copied()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let value = self.expression(value)?;
                    if self.value_type(value)? != lower_type(target.ty) {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    let replacement = if target.fields.is_empty() {
                        if self.value_type(root)? != self.value_type(value)? {
                            return Err(LoweringFailure::InvalidCheckedProgram);
                        }
                        value
                    } else {
                        self.replace_struct_path(root, &target.fields, value)?
                    };
                    if self.bindings.insert(target.binding, replacement) != Some(root) {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                }
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
                } => self.lower_match(
                    scrutinee,
                    *enum_type,
                    arms,
                    *continues,
                    Some((*binding, lower_type(*result_type))),
                    give_target.clone(),
                )?,
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
                    lower_type(binder.ty),
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
                if self.value_type(value)? != lower_type(*ty) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(value)
            }
            CheckedExpression::Constant(value) => {
                let ty = lower_type(value.ty());
                let constant = match value {
                    CheckedValue::Unit => IrConstant::Unit,
                    CheckedValue::Bool(value) => IrConstant::Bool(*value),
                    CheckedValue::Integer { ty, bits } => IrConstant::Integer {
                        ty: lower_type(CheckedType::Integer(*ty)),
                        bits: *bits,
                    },
                };
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
                    lower_type(*result),
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
                let [left, right] = arguments.as_slice() else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                self.define(
                    lower_type(expression.ty()),
                    IrOperation::Integer {
                        operation: (*operation).into(),
                        operand_type: lower_type(CheckedType::Integer(*operand_type)),
                        arguments: [left, right],
                        trap: trap.clone().map(Into::into),
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
                        operand_type: lower_type(*operand_type),
                        arguments: [left, right],
                    },
                )
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
                let root = self
                    .bindings
                    .get(binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let mut lowered_drops = Vec::with_capacity(residual_drops.len());
                for drop in residual_drops {
                    lowered_drops.push(self.lower_projected_drop(root, drop)?);
                }
                let value = self.project_struct_path(root, fields, *consume_root)?;
                if self.value_type(value)? != lower_type(*ty) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                for drop in lowered_drops {
                    self.current_block_mut()?
                        .instructions
                        .push(IrInstruction::Drop(drop));
                }
                Ok(value)
            }
        }
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
                IrNominalKind::Enum { .. } => {
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
            IrNominalKind::Enum { .. } => {
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
        let ty = lower_type(drop.ty);
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

    fn lower_drops(&self, drops: &[CheckedDrop]) -> Result<Vec<IrDrop>, LoweringFailure> {
        drops
            .iter()
            .map(|drop| {
                let value = self
                    .bindings
                    .get(&drop.binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let ty = lower_type(drop.ty);
                if self.value_type(value)? != ty {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Ok(IrDrop { value, ty })
            })
            .collect()
    }
}
