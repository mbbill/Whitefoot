//! Runtime value uses shared by capture selection, storage and emission.

use super::{IrArrayRoot, IrInstruction, IrOperation, IrTerminator, IrValueId};

impl IrInstruction {
    pub(crate) fn operands(&self) -> Vec<IrValueId> {
        match self {
            Self::Define { operation, .. } => operation.operands(),
            Self::StoreBuffer {
                buffer,
                index,
                value,
            } => vec![*buffer, *index, *value],
            Self::StoreSlice {
                slice,
                index,
                value,
            } => vec![*slice, *index, *value],
            Self::Store { address, value, .. } => vec![*address, *value],
            Self::Drops(drops) => drops.iter().map(|drop| drop.operand()).collect(),
        }
    }
}

impl IrTerminator {
    pub(crate) fn operands(&self) -> Vec<IrValueId> {
        match self {
            Self::Unreachable => Vec::new(),
            Self::Jump {
                arguments, drops, ..
            } => arguments
                .iter()
                .copied()
                .chain(drops.iter().map(|drop| drop.operand()))
                .collect(),
            Self::Match { scrutinee, .. } => vec![*scrutinee],
            Self::Return { value, drops } => std::iter::once(*value)
                .chain(drops.iter().map(|drop| drop.operand()))
                .collect(),
        }
    }
}

impl IrOperation {
    /// Every runtime value read, including allocation inputs and loop captures.
    /// New operations must specify their uses before any consumer compiles.
    pub(crate) fn operands(&self) -> Vec<IrValueId> {
        match self {
            Self::Constant(_) | Self::ConstantAddress { .. } | Self::Window => Vec::new(),
            Self::Call { arguments, .. }
            | Self::Integer { arguments, .. }
            | Self::Float { arguments, .. }
            | Self::Boolean { arguments, .. } => arguments.clone(),
            Self::EnumEquality { arguments, .. } => arguments.to_vec(),
            Self::NumericConversion { value, .. }
            | Self::Reinterpret { value, .. }
            | Self::ArrayFill { value, .. }
            | Self::FullArrayConversion { value }
            | Self::BoxNew { value, .. }
            | Self::BoxTake { value, .. }
            | Self::BoxDeref { value, .. }
            | Self::RuntimeBoxPayload { owner: value, .. }
            | Self::RuntimeBoxOwner { payload: value, .. }
            | Self::AddressOf { value, .. } => vec![*value],
            Self::ArrayIndex { root, offset, .. } => match root {
                IrArrayRoot::Value(value) => vec![*value, *offset],
                IrArrayRoot::Constant(_) => vec![*offset],
            },
            Self::BufferFill { length, value, .. } => vec![*length, *value],
            Self::BufferMeasure { buffer } | Self::SliceFromBuffer { buffer } => vec![*buffer],
            Self::ContainerMeasure { container, .. } => vec![*container],
            Self::RunIndex { run, offset, .. } => vec![*run, *offset],
            Self::RunBoundary { run, value, .. } => {
                std::iter::once(*run).chain(value.iter().copied()).collect()
            }
            Self::RunTaken { run, .. } | Self::SliceFromRun { run } => vec![*run],
            Self::RunShift { run, index, .. } => vec![*run, *index],
            Self::RunInsert { run, index, value } => vec![*run, *index, *value],
            Self::RunTransfer {
                destination,
                source,
                index,
            } => vec![*destination, *source, *index],
            Self::WindowBlockNew { capacity, .. } => vec![*capacity],
            Self::WindowGrow { cell, capacity, .. } => vec![*cell, *capacity],
            Self::CellFree { value, .. } => vec![*value],
            Self::SliceRange { slice, start, end } => vec![*slice, *start, *end],
            Self::BufferIndex { buffer, offset, .. } => vec![*buffer, *offset],
            Self::BufferProbeSkip {
                buffer,
                index,
                limit,
                needles,
            } => [*buffer, *index, *limit]
                .into_iter()
                .chain(needles.iter().copied())
                .collect(),
            Self::SliceMeasure { slice } => vec![*slice],
            Self::SliceIndex { slice, offset, .. } => vec![*slice, *offset],
            Self::SliceAddress { slice, offset, .. } => vec![*slice, *offset],
            Self::ConstructStruct { fields, .. } | Self::ConstructEnum { fields, .. } => {
                fields.clone()
            }
            Self::ProjectStruct { aggregate, .. } | Self::ProjectVariant { aggregate, .. } => {
                vec![*aggregate]
            }
            Self::InsertStruct {
                aggregate, value, ..
            } => vec![*aggregate, *value],
            Self::Load { address, .. } => vec![*address],
            Self::ProjectAddress {
                address,
                projection,
            } => match projection {
                super::IrPlaceStep::Field { .. }
                | super::IrPlaceStep::BoxReferent { .. }
                | super::IrPlaceStep::EnumVariant { .. } => vec![*address],
                super::IrPlaceStep::RunElement { offset, .. }
                | super::IrPlaceStep::ArrayElement { offset, .. }
                | super::IrPlaceStep::BufferElement { offset, .. } => vec![*address, *offset],
            },
            Self::LoopSplit {
                seed,
                lower,
                upper,
                captures,
                ..
            } => [*seed, *lower, *upper]
                .into_iter()
                .chain(captures.iter().copied())
                .collect(),
        }
    }
}
