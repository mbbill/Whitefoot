use crate::semantic::{
    CheckedBufferRoot, CheckedBufferSetTarget, CheckedExpression, CheckedFlatElement,
    CheckedRuntimeTargetObligations, CheckedTargetDomainObligation, TrapSite,
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
        trap: &TrapSite,
        target_domains: CheckedRuntimeTargetObligations,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(element)?;
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
                trap: trap.clone().into(),
                target_domains: target_domains.into(),
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
            IrOperation::BufferLength { buffer },
        )
    }

    pub(super) fn lower_buffer_index(
        &mut self,
        root: &CheckedBufferRoot,
        offset: &CheckedExpression,
        trap: &TrapSite,
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
                trap: trap.clone().into(),
                target_domain: target_domain.into(),
            },
        )
    }

    pub(super) fn lower_buffer_set(
        &mut self,
        root: IrValueId,
        target: &CheckedBufferSetTarget,
        value: &CheckedExpression,
    ) -> Result<IrValueId, LoweringFailure> {
        let element = lower_flat_element(target.root.element)?;
        let buffer = self.project_buffer_root(root, &target.root)?;
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
            IrType::GuardedBufferIndex { element },
            IrOperation::BufferBoundsCheck {
                buffer,
                offset,
                trap: target.trap.clone().into(),
                target_domain: target.target_domain.into(),
            },
        )?;
        let value = self.expression(value)?;
        if self.value_type(value)? != element.ty() {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        self.current_block_mut()?
            .instructions
            .push(IrInstruction::StoreBuffer {
                buffer,
                index,
                value,
            });
        Ok(root)
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
                element: lower_flat_element(root.element)?,
            })
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }
}
