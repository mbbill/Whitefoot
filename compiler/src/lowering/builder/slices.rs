use crate::semantic::{
    CheckedExpression, CheckedFlatElement, CheckedSliceRoot, CheckedSliceSetTarget,
    CheckedSliceSource, CheckedTargetDomainObligation,
};

use super::*;

impl IrBuilder<'_> {
    pub(super) fn lower_slice_of(
        &mut self,
        source: &CheckedSliceSource,
        expected_element: CheckedFlatElement,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(self.erasure, expected_element)?;
        let operation = match source {
            CheckedSliceSource::Array { root, length } => {
                let (array, ty) = self.array_root(root)?;
                let IrType::Array {
                    element: actual,
                    length: actual_length,
                } = ty
                else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                if actual != element || Some(actual_length) != length.value() {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                IrOperation::SliceFromArray { array }
            }
            CheckedSliceSource::Buffer(root) => {
                let buffer = self.lower_buffer_borrow(root)?;
                if self.value_type(buffer)? != (IrType::Buffer { element }) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                IrOperation::SliceFromBuffer { buffer }
            }
            // [VIEW-2, OWN-6] the shared child of a view reached through its
            // holder. A view value is already a descriptor and the child
            // carries the parent's range, so the child *is* the parent's
            // descriptor value: nothing is computed and nothing is narrowed.
            CheckedSliceSource::ViewHolder { binding, .. } => {
                let parent = self.binding_value(*binding)?;
                if self.value_type(parent)? != (IrType::Slice { element }) {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                return Ok(parent);
            }
            // [VIEW-2] a run's window: its own slots, from `head` onward.
            CheckedSliceSource::Run(root) => {
                let run = self.container_root_value(root)?;
                IrOperation::SliceFromRun { run }
            }
            // The arena runtime lowering is not implemented. Two semantic
            // capability stops together keep this source out of every
            // published checked program: a view over a local arena stops
            // where it is formed, and a view over an arena parameter stops
            // at the arena-parameter gate that ends the whole function. So
            // reaching it is an invariant failure, not a silent miscompile.
            CheckedSliceSource::ArenaContent { .. } => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        self.define(IrType::Slice { element }, operation)
    }

    pub(super) fn lower_slice_length(
        &mut self,
        root: &CheckedSliceRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.slice_root(root)?;
        self.define(
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrOperation::SliceMeasure { slice },
        )
    }

    pub(super) fn lower_slice_index(
        &mut self,
        root: &CheckedSliceRoot,
        offset: &CheckedExpression,
        target_domain: CheckedTargetDomainObligation,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.slice_root(root)?;
        let element = lower_flat_element(self.erasure, root.element)?;
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
            IrOperation::SliceIndex {
                slice,
                offset,
                target_domain: target_domain.into(),
            },
        )
    }

    /// [SET-1] one element-position store through an exclusive view.
    ///
    /// The descriptor is not the storage: the store reaches the origin
    /// through the view's own data pointer, so the value handed back is the
    /// descriptor unchanged, exactly as a buffer element commit hands back
    /// the buffer it wrote through.
    pub(super) fn lower_slice_element_commit(
        &mut self,
        target: &CheckedSliceSetTarget,
        value: IrValueId,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.slice_root(&target.root)?;
        let element = lower_flat_element(self.erasure, target.root.element)?;
        // The subscript's bounds obligation is discharged at the source level
        // [OP-4]; the offset is consumed directly with no runtime branch.
        let index = self.expression(&target.offset)?;
        if self.value_type(index)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        if self.value_type(value)? != element.ty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::StoreSlice {
                slice,
                index,
                value,
            });
        Ok(slice)
    }

    fn slice_root(&mut self, root: &CheckedSliceRoot) -> Result<IrValueId, LoweringFailure> {
        let slice = self.binding_value(root.binding)?;
        if self.value_type(slice)?
            != (IrType::Slice {
                element: lower_flat_element(self.erasure, root.element)?,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(slice)
    }
}
