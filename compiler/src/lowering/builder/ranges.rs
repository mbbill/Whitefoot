//! Lowering of the [REF-4] range reference `&[T]`.
//!
//! compiler/storage-representation (range reference) lowers `&[T]` as a
//! pointer to the range's first element together with the element count,
//! which is the pair `IrType::Range` already names. Forming one addresses the
//! source place and narrows the descriptor to the written endpoints; the two
//! readers are the element count and one discharged element address.

use crate::semantic::{
    CheckedExpression, CheckedRangeRoot, CheckedRangeSource, CheckedTargetDomainObligation,
};

use super::*;

impl IrBuilder<'_> {
    /// [REF-4] `&x[lo..hi]`: the source descriptor, narrowed by the endpoints
    /// the checker already discharged `lo <= hi <= x.len` for.
    pub(super) fn lower_range_of(
        &mut self,
        source: &CheckedRangeSource,
        start: &CheckedExpression,
        end: &CheckedExpression,
        element: crate::semantic::CheckedFlatElement,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(self.erasure, element)?;
        let ty = IrType::Range { element };
        let slice = match source {
            CheckedRangeSource::Storage(root) => {
                let address = self.lower_place_address(root)?;
                // A runtime-capacity `Array<T>` [TYPE-9] is its own
                // pointer-and-count descriptor, so the range is formed from
                // that value rather than from the storage address.
                if matches!(
                    lower_type(self.erasure, root.ty)?,
                    IrType::Buffer { .. }
                ) {
                    let buffer = self.load_storage_value(address)?;
                    self.define(ty, IrOperation::SliceFromBuffer { buffer })?
                } else {
                    self.define(ty, IrOperation::SliceFromRun { run: address })?
                }
            }
            CheckedRangeSource::Range(root) => self.range_root(root)?,
        };
        if self.value_type(slice)? != ty {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        let start = self.expression(start)?;
        let end = self.expression(end)?;
        let index = IrType::Integer {
            width: 64,
            signed: false,
        };
        if self.value_type(start)? != index || self.value_type(end)? != index {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.define(ty, IrOperation::SliceRange { slice, start, end })
    }

    /// [MSR-1] the one measure a range reference has.
    pub(super) fn lower_range_measure(
        &mut self,
        root: &CheckedRangeRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.range_root(root)?;
        self.define(
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrOperation::SliceMeasure { slice },
        )
    }

    /// [OP-4] one discharged element read of the run a range names.
    pub(super) fn lower_range_index(
        &mut self,
        root: &CheckedRangeRoot,
        offset: &CheckedExpression,
        target_domain: CheckedTargetDomainObligation,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.range_root(root)?;
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

    /// The descriptor value one range reference binding holds [REF-4].
    pub(super) fn range_root(
        &mut self,
        root: &CheckedRangeRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let slice = self.binding_value(root.binding)?;
        if self.value_type(slice)?
            != (IrType::Range {
                element: lower_flat_element(self.erasure, root.element)?,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(slice)
    }
}
