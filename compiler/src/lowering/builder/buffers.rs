use crate::semantic::{
    CheckedBufferRoot, CheckedExpression, CheckedFlatElement, CheckedLayoutCeiling,
    CheckedRuntimeTargetObligations, CheckedTargetDomainObligation,
};

use super::*;

impl IrBuilder<'_> {
    pub(super) fn lower_buffer_borrow(
        &mut self,
        root: &CheckedBufferRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        self.buffer_root(root)
    }

    pub(super) fn lower_buffer_fill(
        &mut self,
        element: CheckedFlatElement,
        length: &CheckedExpression,
        value: &CheckedExpression,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(self.erasure, element)?;
        let length = self.expression(length)?;
        let value = self.expression(value)?;
        if self.value_type(length)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
            || self.value_type(value)? != element.ty()
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.define(
            IrType::Buffer { element },
            IrOperation::BufferFill {
                length,
                value,
                layout_ceiling: layout_ceiling.into(),
                target_domains: target_domains.try_into()?,
            },
        )
    }

    /// One `buffer_vacant::<T>(n)` allocation [OP-1, OP-9]: the element is the
    /// interned `Option<T>` instance and every element starts as its
    /// compiler-minted `None()`.
    pub(super) fn lower_buffer_vacant(
        &mut self,
        element: crate::semantic::NominalId,
        length: &CheckedExpression,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = IrFlatElement::Nominal(self.erased(element));
        let length = self.expression(length)?;
        if self.value_type(length)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.define(
            IrType::Buffer { element },
            IrOperation::BufferVacant {
                length,
                layout_ceiling: layout_ceiling.into(),
                target_domains: target_domains.try_into()?,
            },
        )
    }

    pub(super) fn lower_buffer_length(
        &mut self,
        root: &CheckedBufferRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let buffer = self.buffer_root(root)?;
        self.define(
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrOperation::BufferMeasure { buffer },
        )
    }

    pub(super) fn lower_buffer_index(
        &mut self,
        root: &CheckedBufferRoot,
        offset: &CheckedExpression,
        target_domain: CheckedTargetDomainObligation,
    ) -> Result<IrValueId, LoweringFailure> {
        let buffer = self.buffer_root(root)?;
        let IrType::Buffer { element } = self.value_type(buffer)? else {
            return Err(LoweringFailure::InvalidCheckedProgram);
        };
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
            IrOperation::BufferIndex {
                buffer,
                offset,
                target_domain: target_domain.into(),
            },
        )
    }

    fn buffer_root(&mut self, root: &CheckedBufferRoot) -> Result<IrValueId, LoweringFailure> {
        let value = self.binding_value(root.binding)?;
        self.project_buffer_root(value, root)
    }

    fn project_buffer_root(
        &mut self,
        root_value: IrValueId,
        root: &CheckedBufferRoot,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = if root.fields.is_empty() {
            root_value
        } else {
            self.project_struct_path(root_value, &root.fields, false)?
        };
        if self.value_type(value)?
            != (IrType::Buffer {
                element: lower_flat_element(self.erasure, root.element)?,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }
}
