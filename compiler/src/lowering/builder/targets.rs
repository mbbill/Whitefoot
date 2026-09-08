//! Mutation targets captured before right-hand-side evaluation [SET-1, SET-2].
//!
//! A prepared target carries only evaluated addresses, descriptors and offsets.
//! Its commit never evaluates a source expression again. Plain SSA field paths
//! need no runtime target evaluation; rebuilding one at commit uses the current
//! root so writes performed by the right-hand side to sibling fields survive.

use crate::semantic::{CheckedContainerRoot, CheckedPlaceStep, CheckedWritablePlace};

use super::*;

pub(super) struct PreparedTarget<'target> {
    ty: IrType,
    kind: TargetStorage<'target>,
}

enum TargetStorage<'target> {
    Place(&'target CheckedWritablePlace),
    Address {
        address: IrValueId,
        referent: IrAddressed,
    },
    Buffer {
        buffer: IrValueId,
        index: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    Slice {
        slice: IrValueId,
        index: IrValueId,
    },
}

impl IrBuilder<'_> {
    pub(super) fn prepare_target<'target>(
        &mut self,
        target: &'target CheckedSetTarget,
    ) -> Result<PreparedTarget<'target>, LoweringFailure> {
        let ty = lower_type(self.erasure, target.ty())?;
        let address_kind = |address, referent| TargetStorage::Address { address, referent };
        let kind = match target {
            CheckedSetTarget::Storage(root) => {
                let address = self.lower_place_address(root)?;
                let IrType::Address(referent) = self.value_type(address)? else {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                };
                address_kind(address, referent)
            }
            CheckedSetTarget::Place(place) => {
                if place.declares {
                    if !place.fields.is_empty() {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    TargetStorage::Place(place)
                } else {
                    let storage = self
                        .bindings
                        .get(&place.binding)
                        .copied()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                    if let Some(address) = self.addressed_target(target, storage)? {
                        let IrType::Address(referent) = self.value_type(address)? else {
                            return Err(LoweringFailure::InvalidCheckedProgram);
                        };
                        address_kind(address, referent)
                    } else {
                        TargetStorage::Place(place)
                    }
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let root = CheckedContainerRoot {
                    binding: target.binding,
                    path: target
                        .fields
                        .iter()
                        .copied()
                        .map(CheckedPlaceStep::Field)
                        .collect(),
                    ty: target.array_type,
                };
                let array = self.lower_place_address(&root)?;
                let offset = self.expression(&target.offset)?;
                self.check_target_offset(offset, target.target_domain.into())?;
                let referent = IrAddressed::of(ty).ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let address = self.define(
                    IrType::Address(referent),
                    IrOperation::ProjectAddress {
                        address: array,
                        projection: IrPlaceProjection::ArrayElement {
                            offset,
                            target_domain: target.target_domain.into(),
                        },
                    },
                )?;
                address_kind(address, referent)
            }
            CheckedSetTarget::BufferIndex(target) => {
                let buffer = self.lower_buffer_borrow(&target.root)?;
                let index = self.expression(&target.offset)?;
                let target_domain = target.target_domain.into();
                self.check_target_offset(index, target_domain)?;
                TargetStorage::Buffer {
                    buffer,
                    index,
                    target_domain,
                }
            }
            CheckedSetTarget::SliceIndex(target) => {
                let slice = self.slice_root(&target.root)?;
                let index = self.expression(&target.offset)?;
                self.check_target_offset(index, target.target_domain.into())?;
                TargetStorage::Slice { slice, index }
            }
        };
        Ok(PreparedTarget { ty, kind })
    }

    fn check_target_offset(
        &self,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), LoweringFailure> {
        if self.value_type(offset)?
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
            || target_domain != IrTargetDomainObligation::ElementAddress
        {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(())
    }

    /// The read half of replacement occurs at commit, after the RHS's effects.
    pub(super) fn read_target(
        &mut self,
        target: &PreparedTarget<'_>,
    ) -> Result<IrValueId, LoweringFailure> {
        let value = match &target.kind {
            TargetStorage::Address { address, .. } => self.load_storage_value(*address)?,
            TargetStorage::Place(place) if !place.declares => {
                let root = self.binding_value(place.binding)?;
                if place.fields.is_empty() {
                    root
                } else {
                    self.project_struct_path(root, &place.fields, false)?
                }
            }
            TargetStorage::Buffer {
                buffer,
                index,
                target_domain,
            } => self.define(
                target.ty,
                IrOperation::BufferIndex {
                    buffer: *buffer,
                    offset: *index,
                    target_domain: *target_domain,
                },
            )?,
            TargetStorage::Place(_) | TargetStorage::Slice { .. } => {
                return Err(LoweringFailure::InvalidCheckedProgram);
            }
        };
        if self.value_type(value)? != target.ty {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        Ok(value)
    }

    pub(super) fn write_target(
        &mut self,
        target: &PreparedTarget<'_>,
        value: IrValueId,
    ) -> Result<(), LoweringFailure> {
        if self.value_type(value)? != target.ty {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        match &target.kind {
            TargetStorage::Address { address, referent } => {
                self.store_addressed(*address, value, *referent)
            }
            TargetStorage::Buffer { buffer, index, .. } => {
                self.current_block_mut()?
                    .instructions
                    .push(IrInstruction::StoreBuffer {
                        buffer: *buffer,
                        index: *index,
                        value,
                    });
                Ok(())
            }
            TargetStorage::Slice { slice, index } => {
                self.current_block_mut()?
                    .instructions
                    .push(IrInstruction::StoreSlice {
                        slice: *slice,
                        index: *index,
                        value,
                    });
                Ok(())
            }
            TargetStorage::Place(place) => {
                if place.declares {
                    if self.bindings.insert(place.binding, value).is_some() {
                        return Err(LoweringFailure::InvalidCheckedProgram);
                    }
                    return self.promote_binding_if_needed(place.binding);
                }
                let storage = self
                    .bindings
                    .get(&place.binding)
                    .copied()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?;
                let replacement = if place.fields.is_empty() {
                    value
                } else {
                    let root = self.load_storage_value(storage)?;
                    self.replace_struct_path(root, &place.fields, value)?
                };
                self.commit_root_storage(place.binding, storage, replacement)
            }
        }
    }
}
