//! Lowering of the [BLK-1] runs and the [BLK-0] rows over them.
//!
//! A run's value is its complete storage: a `FixedVector<T, n>` is `n` inline
//! slots followed by the two descriptor words `len` and `head`, and a
//! `Vector<'s, T>` is the descriptor `{ pointer, cap, len, head }` over a run
//! taken from its store. Neither carries a per-slot tag: the window is the
//! complete typestate [BLK-1], so every operation here is boundary arithmetic
//! over those words plus at most one element store or load.

use crate::semantic::{
    CheckedContainerRoot, CheckedExpression, CheckedKernelInstance, CheckedMeasure,
    CheckedRunSetTarget, CheckedType, MeasureCell,
};
use crate::{IrBoundary, IrMeasure};

use super::*;

/// The two written constants of one bump extent at this instance [BLK-2].
fn extent_constants(instance: &CheckedKernelInstance) -> Result<(u64, u64), LoweringFailure> {
    let value = |constant: Option<crate::semantic::CheckedConst>| {
        constant
            .and_then(|constant| constant.value())
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    };
    Ok((value(instance.bytes)?, value(instance.align)?))
}

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
                // A bump extent carries one measure word, its cursor: its
                // `room_of` is the complement [MSR-2] relates to the byte
                // extent its type already fixes, so it is formed here rather
                // than loaded [MSR-1].
                if measure == CheckedMeasure::Room
                    && root.measured() == Some(crate::semantic::MeasuredKind::Extent)
                {
                    let bytes = root
                        .type_constant()
                        .and_then(|constant| constant.value())
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    let cursor = self.define(
                        IrType::Integer {
                            width: 64,
                            signed: false,
                        },
                        IrOperation::ContainerMeasure {
                            measure: IrMeasure::Length,
                            container,
                        },
                    )?;
                    return self.lower_measure_complement(bytes, cursor);
                }
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
            lower_element(element)?.ty(),
            IrOperation::RunIndex {
                run,
                offset,
                target_domain: target_domain.into(),
            },
        )
    }

    /// The run-element half of one [LIV-2] commit, over an ordinal value the
    /// caller has already evaluated.
    ///
    /// The offset's [OP-4] obligation was discharged at the source level, so
    /// it is consumed directly and no runtime branch remains; the value
    /// handed back is the run with that one slot replaced.
    pub(super) fn lower_run_element_commit(
        &mut self,
        root: IrValueId,
        target: &CheckedRunSetTarget,
        value: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let run = self.project_container_root(root, &target.root)?;
        let index = self.expression(&target.offset)?;
        let stored = self.run_store(run, index, value, target)?;
        self.reinsert_container_root(root, &target.root, stored)
    }

    /// [SET-2] the run-element exchange: the previous element is read out of
    /// the slot, then the replacement is stored into the same slot.
    ///
    /// The target's components are evaluated exactly once — one projected run
    /// and one offset feed both the read and the write — so the shared `set`
    /// path, which would re-lower the offset, is not reused here.
    pub(super) fn lower_run_replace(
        &mut self,
        root: IrValueId,
        target: &CheckedRunSetTarget,
        value: &CheckedExpression,
    ) -> Result<(IrValueId, IrValueId), LoweringFailure> {
        let element = target
            .root
            .element()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let run = self.project_container_root(root, &target.root)?;
        let index = self.expression(&target.offset)?;
        let previous = self.define(
            lower_element(element)?.ty(),
            IrOperation::RunIndex {
                run,
                offset: index,
                target_domain: target.target_domain.into(),
            },
        )?;
        let value = self.expression(value)?;
        let stored = self.run_store(run, index, value, target)?;
        let replacement = self.reinsert_container_root(root, &target.root, stored)?;
        Ok((previous, replacement))
    }

    /// One element store's new root value: a run reached through field
    /// selections is written back into the aggregate that holds it, because
    /// a frame-resident run's slots are part of its own value.
    fn reinsert_container_root(
        &mut self,
        root: IrValueId,
        container: &CheckedContainerRoot,
        stored: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        if container.fields.is_empty() {
            return Ok(stored);
        }
        self.replace_struct_path(root, &container.fields, stored)
    }

    fn run_store(
        &mut self,
        run: IrValueId,
        offset: IrValueId,
        value: IrValueId,
        target: &CheckedRunSetTarget,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = target
            .root
            .element()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        if self.value_type(offset)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
            || self.value_type(value)? != lower_element(element)?.ty()
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let run_type = self.value_type(run)?;
        self.define(
            run_type,
            IrOperation::RunStore {
                run,
                offset,
                value,
                target_domain: target.target_domain.into(),
            },
        )
    }

    fn project_container_root(
        &mut self,
        root: IrValueId,
        container: &CheckedContainerRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = if container.fields.is_empty() {
            root
        } else {
            self.project_struct_path(root, &container.fields, false)?
        };
        if self.value_type(value)? != lower_type(container.ty)? {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
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
            crate::KernelRow::FixedVector => {
                if !arguments.is_empty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                self.define(result_type, IrOperation::FixedVector)
            }
            crate::KernelRow::PlaceBack
            | crate::KernelRow::PlaceFront
            | crate::KernelRow::TakeBack
            | crate::KernelRow::TakeFront => {
                self.lower_boundary_row(row, instance, arguments, result)
            }
            crate::KernelRow::ArenaFrame => {
                if !arguments.is_empty() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let (bytes, align) = extent_constants(instance)?;
                self.define(result_type, IrOperation::ArenaFrame { bytes, align })
            }
            crate::KernelRow::ArenaVector
            | crate::KernelRow::ArenaVectorProved
            | crate::KernelRow::HeapVector => {
                self.lower_store_take(row, instance, arguments, result)
            }
        }
    }

    /// One of [BLK-2]'s three acquiring rows.
    ///
    /// Each takes its store's provider by `&uniq` and hands back the run the
    /// store gave it: the bump rows advance a cursor inside the extent this
    /// activation reserved, and the general-store row asks its host. A row
    /// whose refusal is a value carries the `Option` [PRE-1] declares; the
    /// proved arena row carries none, its domain requirement having been
    /// discharged at the call [MSR-4].
    fn lower_store_take(
        &mut self,
        row: crate::KernelRow,
        instance: &CheckedKernelInstance,
        arguments: &[CheckedExpression],
        result: CheckedType,
    ) -> Result<IrValueId, LoweringFailure> {
        let [store, count] = arguments else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let store = self.expression(store)?;
        if self.value_type(store)? != IrType::Address(crate::IrAddressed::Provider) {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let count = self.expression(count)?;
        if self.value_type(count)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let stride = match instance.element_ceiling.stride {
            crate::semantic::CheckedLayoutMagnitude::Finite(stride) => stride,
            crate::semantic::CheckedLayoutMagnitude::AboveU64 => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        let extent = match row {
            crate::KernelRow::HeapVector => None,
            _ => {
                let (bytes, align) = extent_constants(instance)?;
                Some(crate::IrExtentConstants { bytes, align })
            }
        };
        let refusal = match row {
            crate::KernelRow::ArenaVectorProved => None,
            _ => Some(self.refusal_of(result)?),
        };
        self.define(
            lower_type(result)?,
            IrOperation::StoreTake(crate::IrStoreTake {
                store,
                count,
                stride,
                extent,
                refusal,
            }),
        )
    }

    /// The `Option` a refusing row hands back, by the tags [PRE-1] gives its
    /// two variants: exactly one carries the run and exactly one is empty.
    fn refusal_of(&self, result: CheckedType) -> Result<crate::IrRefusal, LoweringFailure> {
        let CheckedType::Nominal(id) = result else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let nominal = IrNominalId(id.0);
        let IrNominalKind::Enum { variants } = &self
            .nominals
            .get(nominal.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .kind
        else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        let mut made = None;
        let mut refused = None;
        for variant in variants {
            match variant.fields().len() {
                0 => refused = refused.xor(Some(variant.tag())),
                1 => made = made.xor(Some(variant.tag())),
                _ => return Err(LoweringFailure::InvalidCheckedProgram),
            }
        }
        let (Some(made), Some(refused)) = (made, refused) else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
        Ok(crate::IrRefusal {
            nominal,
            made,
            refused,
        })
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
            crate::KernelRow::PlaceBack => IrBoundary::PlaceBack,
            crate::KernelRow::PlaceFront => IrBoundary::PlaceFront,
            crate::KernelRow::TakeBack => IrBoundary::TakeBack,
            crate::KernelRow::TakeFront => IrBoundary::TakeFront,
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
