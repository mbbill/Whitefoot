//! Lowering of the [BLK-1] runs and the [BLK-0] rows over them.
//!
//! A run's value is its complete storage: a `FixedVector<T, n>` is `n` inline
//! slots followed by the two descriptor words `len` and `head`, and a
//! `Vector<'s, T>` is the descriptor `{ pointer, cap, len, head }` over a run
//! taken from its store. Neither carries a per-slot tag: the window is the
//! complete typestate [BLK-1], so every operation here is boundary arithmetic
//! over those words plus at most one element store or load.

use crate::semantic::{
    CheckedContainerRoot, CheckedExpression, CheckedKernelInstance, CheckedMeasure, CheckedType,
    MeasureCell,
};
use crate::{IrBoundary, IrMeasure};

use super::*;

/// The IR spelling of one [MSR-1] measure.
const fn lower_measure(measure: CheckedMeasure) -> IrMeasure {
    match measure {
        CheckedMeasure::Length => IrMeasure::Length,
        CheckedMeasure::Capacity => IrMeasure::Capacity,
        CheckedMeasure::Room => IrMeasure::Room,
        CheckedMeasure::Head => IrMeasure::Head,
    }
}

impl IrBuilder<'_> {
    /// One [MSR-1] measure of a run or a bump extent.
    ///
    /// A cell the table fixes as a compile-time constant is that constant and
    /// loads nothing; every other cell is a descriptor word or, for `room`,
    /// the complement [MSR-2] relates it to.
    pub(super) fn lower_container_measure(
        &mut self,
        measure: CheckedMeasure,
        root: &CheckedContainerRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let measured = root
            .measured()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        match measure.cell(measured) {
            MeasureCell::ExactConstant(value) => self.lower_fixed_measure(value),
            // A `FixedVector`'s capacity and a bump extent's byte extent are
            // the type's own written constant and are stored nowhere.
            MeasureCell::ExactTypeConstant => {
                let constant = root
                    .type_constant()
                    .and_then(|constant| match constant {
                        crate::semantic::CheckedConst::Value(value) => Some(value),
                        _ => None,
                    })
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                self.lower_fixed_measure(constant)
            }
            MeasureCell::ExactExtent | MeasureCell::ExactRuntime | MeasureCell::Bounded => {
                let container = self.container_root_value(root)?;
                self.define(
                    IrType::Integer {
                        width: 64,
                        signed: false,
                    },
                    IrOperation::ContainerMeasure {
                        measure: lower_measure(measure),
                        container,
                    },
                )
            }
            MeasureCell::Absent => Err(LoweringFailure::InvalidCheckedProgram),
        }
    }

    /// One discharged source subscript read of a run [OP-4, BLK-1].
    pub(super) fn lower_run_index(
        &mut self,
        root: &CheckedContainerRoot,
        offset: &CheckedExpression,
        target_domain: CheckedTargetDomainObligation,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = root
            .element()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let run = self.container_root_value(root)?;
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
            lower_flat_element(element)?.ty(),
            IrOperation::RunIndex {
                run,
                offset,
                target_domain: target_domain.into(),
            },
        )
    }

    /// One call to a [BLK-0] kernel-domain row.
    ///
    /// The row is selected from the record's own discriminant and never from
    /// a spelling, and the lowering of each is exactly what [BLK-2] and
    /// [BLK-3] say the row does: a formation writes an empty window, and a
    /// boundary operation is one store plus boundary arithmetic.
    pub(super) fn lower_kernel_call(
        &mut self,
        row: crate::KernelRow,
        instance: &CheckedKernelInstance,
        arguments: &[CheckedExpression],
        result: CheckedType,
    ) -> Result<IrValueId, LoweringFailure> {
        let result_type = lower_type(result)?;
        match row {
            crate::KernelRow::SeqFixed => {
                if !arguments.is_empty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(result_type, IrOperation::SeqFixed)
            }
            crate::KernelRow::SeqPlace
            | crate::KernelRow::SeqPlaceFront
            | crate::KernelRow::SeqTake
            | crate::KernelRow::SeqTakeFront => {
                self.lower_boundary_row(row, instance, arguments, result)
            }
            // [BLK-2]'s store-backed rows and the reservation row are the
            // capability this version does not lower; the checker stops a
            // call to one before it reaches here.
            crate::KernelRow::SeqArena
            | crate::KernelRow::SeqArenaProved
            | crate::KernelRow::SeqHeap
            | crate::KernelRow::ArenaFrame => Err(LoweringFailure::InvalidCheckedProgram),
        }
    }

    /// One of [BLK-3]'s four boundary operations.
    ///
    /// Each takes its run by value and hands it back, so the lowering is one
    /// new run value; a removal row additionally hands back the element,
    /// which makes its value the ordered result list [CALL-4] the record
    /// declares.
    fn lower_boundary_row(
        &mut self,
        row: crate::KernelRow,
        instance: &CheckedKernelInstance,
        arguments: &[CheckedExpression],
        result: CheckedType,
    ) -> Result<IrValueId, LoweringFailure> {
        let boundary = match row {
            crate::KernelRow::SeqPlace => IrBoundary::PlaceBack,
            crate::KernelRow::SeqPlaceFront => IrBoundary::PlaceFront,
            crate::KernelRow::SeqTake => IrBoundary::TakeBack,
            crate::KernelRow::SeqTakeFront => IrBoundary::TakeFront,
            _ => return Err(LoweringFailure::InvalidCheckedProgram),
        };
        let run_type = lower_type(instance.run.ok_or(LoweringFailure::InvalidCheckedProgram)?)?;
        let result_type = lower_type(result)?;
        let element_type = lower_type(instance.element)?;
        let [run, rest @ ..] = arguments else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let run = self.expression(run)?;
        if self.value_type(run)? != run_type {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let value = match (boundary.places(), rest) {
            (true, [value]) => {
                let value = self.expression(value)?;
                if self.value_type(value)? != element_type {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                Some(value)
            }
            (false, []) => None,
            _ => return Err(LoweringFailure::InvalidCheckedProgram),
        };
        // The element is read out before the boundary moves, so the removal
        // rows evaluate their taken value first and the run second.
        let taken = (!boundary.places())
            .then(|| self.define(element_type, IrOperation::RunTaken { row: boundary, run }))
            .transpose()?;
        let handed_back = self.define(
            run_type,
            IrOperation::RunBoundary {
                row: boundary,
                run,
                value,
            },
        )?;
        let Some(taken) = taken else {
            if result_type == run_type {
                return Ok(handed_back);
            }
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        // The ordered result list [CALL-4] is one value of the row's
        // compiler-owned result-list nominal, whose fields are `rest` then
        // `value` in declared order.
        let CheckedType::Nominal(nominal) = result else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let nominal = IrNominalId(nominal.0);
        self.define(
            IrType::Nominal(nominal),
            IrOperation::ConstructStruct {
                nominal,
                fields: vec![handed_back, taken],
            },
        )
    }

    /// The run or extent value one measured place reads, projected out of its
    /// root binding by the field selections that reach it [MSR-2].
    fn container_root_value(
        &mut self,
        root: &CheckedContainerRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = self.binding_value(root.binding)?;
        let value = if root.fields.is_empty() {
            value
        } else {
            self.project_struct_path(value, &root.fields, false)?
        };
        if self.value_type(value)? != lower_type(root.ty)? {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }
}
