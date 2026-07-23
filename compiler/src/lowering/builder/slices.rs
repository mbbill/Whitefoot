use crate::semantic::{
    CheckedExpression, CheckedFlatElement, CheckedSliceRoot, CheckedSliceSource,
    CheckedTargetDomainObligation, TrapSite,
};

use super::*;

impl IrBuilder<'_> {
    pub(super) fn lower_slice_of(
        &mut self,
        source: &CheckedSliceSource,
        expected_element: CheckedFlatElement,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(expected_element)?;
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
            IrOperation::SliceLength { slice },
        )
    }

    pub(super) fn lower_slice_index(
        &mut self,
        root: &CheckedSliceRoot,
        offset: &CheckedExpression,
        trap: &TrapSite,
        target_domain: CheckedTargetDomainObligation,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.slice_root(root)?;
        let element = lower_flat_element(root.element)?;
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
                trap: trap.clone().into(),
                target_domain: target_domain.into(),
            },
        )
    }

    fn slice_root(&mut self, root: &CheckedSliceRoot) -> Result<IrValueId, LoweringFailure> {
        let slice = self.binding_value(root.binding)?;
        if self.value_type(slice)?
            != (IrType::Slice {
                element: lower_flat_element(root.element)?,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(slice)
    }
}
