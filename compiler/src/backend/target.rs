use std::collections::{HashMap, HashSet};

use crate::{
    IrArrayRoot, IrFlatElement, IrFunction, IrInstruction, IrNominalId, IrNominalKind, IrOperation,
    IrProgram, IrTargetDomainObligation, IrTrapSite, IrType,
};

use super::emitter::trap_record;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetObject {
    Representation,
    Static,
    FunctionAbi,
    StackFrame,
    TrapRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetLayoutFailure {
    UnsupportedHost,
    InvalidIr,
    Unrepresentable(TargetObject),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetLayout {
    triple: &'static str,
    data_layout: &'static str,
    address_index_max: u64,
    allocator_parameter_max: u64,
}

impl TargetLayout {
    pub(super) fn host() -> Result<Self, TargetLayoutFailure> {
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        {
            return Ok(Self {
                triple: "aarch64-apple-darwin",
                data_layout: "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
            });
        }
        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        {
            return Ok(Self {
                triple: "x86_64-apple-darwin",
                data_layout: "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
            });
        }
        #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
        {
            return Ok(Self {
                triple: "aarch64-unknown-linux-gnu",
                data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
            });
        }
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            return Ok(Self {
                triple: "x86_64-unknown-linux-gnu",
                data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
            });
        }
        #[allow(unreachable_code)]
        Err(TargetLayoutFailure::UnsupportedHost)
    }

    pub(super) const fn triple(self) -> &'static str {
        self.triple
    }

    pub(super) const fn data_layout(self) -> &'static str {
        self.data_layout
    }

    pub(super) const fn address_index_max(self) -> u64 {
        self.address_index_max
    }

    pub(super) const fn runtime_allocation_max(self) -> u64 {
        if self.address_index_max < self.allocator_parameter_max {
            self.address_index_max
        } else {
            self.allocator_parameter_max
        }
    }
}

#[derive(Clone, Copy)]
struct Layout {
    size: u64,
    align: u64,
}

pub(super) fn validate_program(
    target: TargetLayout,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };

    for nominal in program.nominals() {
        layouts.layout(IrType::Nominal(nominal.id()))?;
    }
    for constant in program.constants() {
        layouts
            .layout(constant.ty())
            .map_err(|failure| as_object(failure, TargetObject::Static))?;
    }
    for function in program.functions() {
        validate_function(&mut layouts, function)?;
    }
    Ok(())
}

fn validate_function(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    function: &IrFunction,
) -> Result<(), TargetLayoutFailure> {
    layouts
        .layout(function.result())
        .map_err(|failure| as_object(failure, TargetObject::FunctionAbi))?;
    for (_, ty) in function.parameters() {
        layouts
            .layout(*ty)
            .map_err(|failure| as_object(failure, TargetObject::FunctionAbi))?;
    }

    let mut frame_size = 0_u64;
    for block in function.blocks() {
        for (_, ty) in block.parameters() {
            layouts.layout(*ty)?;
        }
        for instruction in block.instructions() {
            if let Some(trap) = instruction_trap(instruction) {
                validate_trap_record(layouts.target, trap)?;
            }
            let IrInstruction::Define { ty, operation, .. } = instruction else {
                continue;
            };
            layouts.layout(*ty)?;
            validate_target_obligation(layouts, function, operation)?;
            for slot in emitted_stack_slots(function, *ty, operation)? {
                let layout = layouts
                    .layout(slot)
                    .map_err(|failure| as_object(failure, TargetObject::StackFrame))?;
                frame_size = add_frame_slot(layouts.target, frame_size, layout)?;
            }
        }
    }
    if frame_size > layouts.target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(
            TargetObject::StackFrame,
        ));
    }
    Ok(())
}

fn validate_target_obligation(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    function: &IrFunction,
    operation: &IrOperation,
) -> Result<(), TargetLayoutFailure> {
    match operation {
        IrOperation::ArrayFill { target_domain, .. }
            if *target_domain == IrTargetDomainObligation::ElementAddress => {}
        IrOperation::BufferFill { target_domains, .. } if target_domains.is_complete() => {}
        IrOperation::ArrayIndex {
            root,
            target_domain,
            ..
        } if *target_domain == IrTargetDomainObligation::ElementAddress => {
            let root_type = match root {
                IrArrayRoot::Value(value) => function
                    .value_type(*value)
                    .ok_or(TargetLayoutFailure::InvalidIr)?,
                IrArrayRoot::Constant(id) => layouts
                    .program
                    .constant(*id)
                    .ok_or(TargetLayoutFailure::InvalidIr)?
                    .ty(),
            };
            layouts.layout(root_type)?;
        }
        IrOperation::BufferIndex { target_domain, .. }
        | IrOperation::ArrayBoundsCheck { target_domain, .. }
        | IrOperation::BufferBoundsCheck { target_domain, .. }
            if *target_domain == IrTargetDomainObligation::ElementAddress => {}
        IrOperation::ArrayFill { .. }
        | IrOperation::BufferFill { .. }
        | IrOperation::ArrayIndex { .. }
        | IrOperation::BufferIndex { .. }
        | IrOperation::ArrayBoundsCheck { .. }
        | IrOperation::BufferBoundsCheck { .. } => {
            return Err(TargetLayoutFailure::InvalidIr);
        }
        _ => {}
    }
    Ok(())
}

fn emitted_stack_slots(
    function: &IrFunction,
    result_type: IrType,
    operation: &IrOperation,
) -> Result<Vec<IrType>, TargetLayoutFailure> {
    let slots = match operation {
        IrOperation::ArrayFill { .. } => vec![
            result_type,
            IrType::Integer {
                width: 64,
                signed: false,
            },
        ],
        IrOperation::ArrayIndex {
            root: IrArrayRoot::Value(value),
            ..
        } => vec![
            function
                .value_type(*value)
                .ok_or(TargetLayoutFailure::InvalidIr)?,
        ],
        IrOperation::InsertArray { .. } => vec![result_type],
        IrOperation::AddressOfNominal { nominal, .. } => vec![IrType::Nominal(*nominal)],
        _ => Vec::new(),
    };
    Ok(slots)
}

fn add_frame_slot(
    target: TargetLayout,
    frame_size: u64,
    slot: Layout,
) -> Result<u64, TargetLayoutFailure> {
    let start = align_up(target, frame_size, slot.align, TargetObject::StackFrame)?;
    checked_add(start, slot.size, target, TargetObject::StackFrame)
}

fn validate_trap_record(
    target: TargetLayout,
    trap: &IrTrapSite,
) -> Result<(), TargetLayoutFailure> {
    let length = u64::try_from(trap_record(trap).len())
        .map_err(|_| TargetLayoutFailure::Unrepresentable(TargetObject::TrapRecord))?;
    if length > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(
            TargetObject::TrapRecord,
        ));
    }
    Ok(())
}

fn instruction_trap(instruction: &IrInstruction) -> Option<&IrTrapSite> {
    match instruction {
        IrInstruction::Check { trap, .. } => Some(trap),
        IrInstruction::Define { operation, .. } => match operation {
            IrOperation::Integer {
                trap: Some(trap), ..
            }
            | IrOperation::ArrayIndex { trap, .. }
            | IrOperation::ArrayBoundsCheck { trap, .. }
            | IrOperation::BufferFill { trap, .. }
            | IrOperation::BufferIndex { trap, .. }
            | IrOperation::BufferBoundsCheck { trap, .. } => Some(trap),
            _ => None,
        },
        IrInstruction::StoreBuffer { .. }
        | IrInstruction::StoreNominal { .. }
        | IrInstruction::Drop(_) => None,
    }
}

struct LayoutComputer<'program, 'classified, 'lexed, 'source> {
    target: TargetLayout,
    program: &'program IrProgram<'classified, 'lexed, 'source>,
    nominal: HashMap<IrNominalId, Layout>,
    visiting: HashSet<IrNominalId>,
}

impl LayoutComputer<'_, '_, '_, '_> {
    fn layout(&mut self, ty: IrType) -> Result<Layout, TargetLayoutFailure> {
        match ty {
            IrType::Unit | IrType::Bool => Ok(Layout { size: 1, align: 1 }),
            IrType::Integer { width, .. } if matches!(width, 8 | 16 | 32 | 64) => {
                let bytes = u64::from(width / 8);
                Ok(Layout {
                    size: bytes,
                    align: bytes,
                })
            }
            IrType::Integer { .. } => Err(TargetLayoutFailure::InvalidIr),
            IrType::Nominal(id) => self.nominal_layout(id),
            IrType::NominalAddress(_) => Ok(Layout { size: 8, align: 8 }),
            IrType::Array { element, length } => {
                let element = self.layout(element.ty())?;
                let stride = align_up(
                    self.target,
                    element.size,
                    element.align,
                    TargetObject::Representation,
                )?;
                let size = checked_mul(stride, length, self.target, TargetObject::Representation)?;
                Ok(Layout {
                    size,
                    align: element.align,
                })
            }
            IrType::Buffer { element } => {
                self.flat_element(element)?;
                Ok(Layout { size: 16, align: 8 })
            }
            IrType::GuardedArrayIndex { .. } | IrType::GuardedBufferIndex { .. } => {
                Ok(Layout { size: 8, align: 8 })
            }
        }
    }

    fn flat_element(&mut self, element: IrFlatElement) -> Result<Layout, TargetLayoutFailure> {
        self.layout(element.ty())
    }

    fn nominal_layout(&mut self, id: IrNominalId) -> Result<Layout, TargetLayoutFailure> {
        if let Some(layout) = self.nominal.get(&id) {
            return Ok(*layout);
        }
        if !self.visiting.insert(id) {
            return Err(TargetLayoutFailure::InvalidIr);
        }
        let nominal = self
            .program
            .nominal(id)
            .ok_or(TargetLayoutFailure::InvalidIr)?;
        let layout = if nominal.is_tag_only_enum() {
            let IrNominalKind::Enum { variants } = nominal.kind() else {
                return Err(TargetLayoutFailure::InvalidIr);
            };
            if variants.len() <= 2 {
                Layout { size: 1, align: 1 }
            } else {
                Layout { size: 4, align: 4 }
            }
        } else {
            let mut fields = Vec::new();
            match nominal.kind() {
                IrNominalKind::Struct {
                    fields: declarations,
                } => fields.extend(declarations.iter().map(|field| field.ty())),
                IrNominalKind::Enum { variants } => {
                    fields.push(IrType::Integer {
                        width: 32,
                        signed: false,
                    });
                    fields.extend(
                        variants
                            .iter()
                            .flat_map(|variant| variant.fields())
                            .map(|field| field.ty()),
                    );
                }
            }
            self.struct_layout(fields)?
        };
        self.visiting.remove(&id);
        self.nominal.insert(id, layout);
        Ok(layout)
    }

    fn struct_layout(&mut self, fields: Vec<IrType>) -> Result<Layout, TargetLayoutFailure> {
        let mut size = 0_u64;
        let mut alignment = 1_u64;
        for field in fields {
            let field = self.layout(field)?;
            size = align_up(self.target, size, field.align, TargetObject::Representation)?;
            size = checked_add(size, field.size, self.target, TargetObject::Representation)?;
            alignment = alignment.max(field.align);
        }
        size = align_up(self.target, size, alignment, TargetObject::Representation)?;
        Ok(Layout {
            size,
            align: alignment,
        })
    }
}

fn checked_add(
    left: u64,
    right: u64,
    target: TargetLayout,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let value = left
        .checked_add(right)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if value > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(value)
}

fn checked_mul(
    left: u64,
    right: u64,
    target: TargetLayout,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let value = left
        .checked_mul(right)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if value > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(value)
}

fn align_up(
    target: TargetLayout,
    value: u64,
    alignment: u64,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let mask = alignment
        .checked_sub(1)
        .ok_or(TargetLayoutFailure::InvalidIr)?;
    let aligned = value
        .checked_add(mask)
        .map(|sum| sum & !mask)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if aligned > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(aligned)
}

fn as_object(failure: TargetLayoutFailure, object: TargetObject) -> TargetLayoutFailure {
    match failure {
        TargetLayoutFailure::Unrepresentable(_) => TargetLayoutFailure::Unrepresentable(object),
        other => other,
    }
}
