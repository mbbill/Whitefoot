use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{DeclarationId, SemanticCompilerFailure, UnsupportedSemanticFeature};

use super::super::super::super::model::{
    CheckedBufferRoot, CheckedContainerRoot, CheckedPlaceStep, CheckedSliceRoot, CheckedType,
};
use super::super::super::{CheckStop, Checker, LocalBinding};
use super::{
    CarriedOperands, CheckedBufferPlace, CheckedContainerPlace, CheckedIndexedPlace,
    CheckedSlicePlace,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_dereferenced_buffer_place(
        &self,
        node: NodeId,
        pbase: NodeId,
        base_suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let (declaration, local, borrow) =
            self.resolve_dereference_holder(node, pbase, bindings)?;
        let (fields, ty) = self.resolve_struct_path(base_suffixes, local.ty)?;
        match ty {
            CheckedType::Buffer { element } => {
                let mut resolved = borrow.place.clone();
                resolved.fields.extend_from_slice(&fields);
                Ok(CheckedIndexedPlace::Buffer(CheckedBufferPlace {
                    root: CheckedBufferRoot {
                        binding: local.binding,
                        fields,
                        element,
                    },
                    declaration,
                    element_type: element.ty(),
                    holder: Some(declaration),
                    resolved,
                    borrow_kind: Some(borrow.kind),
                }))
            }
            CheckedType::Slice {
                region,
                element,
                strength,
            } if fields.is_empty() => {
                let slice = local
                    .slice
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if slice.region != region {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(CheckedIndexedPlace::Slice(CheckedSlicePlace {
                    root: CheckedSliceRoot {
                        binding: local.binding,
                        element,
                        strength,
                    },
                    declaration,
                    descriptor: Some(borrow),
                    slice,
                }))
            }
            // [BLK-1, MSR-1, OWN-5] a run or a bump extent reached through a
            // holder. A run is one measured place wherever it is reached
            // from, so this is the same container place the deref-free path
            // forms, with the holder recorded: the loan judgment reads the
            // holder's own borrow, and every measure and subscript term over
            // the place carries that holder's `deref` step. [BLK-4] refuses
            // only the `&uniq` of a run, so a holder that reaches one here is
            // a shared one or an own-mode cell.
            CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Extent { .. } => {
                let mut resolved = borrow.place.clone();
                resolved.fields.extend_from_slice(&fields);
                Ok(CheckedIndexedPlace::Container(CheckedContainerPlace {
                    root: CheckedContainerRoot {
                        binding: local.binding,
                        path: fields.into_iter().map(CheckedPlaceStep::Field).collect(),
                        ty,
                    },
                    resolved,
                    offsets: CarriedOperands::default(),
                    holder: Some(declaration),
                }))
            }
            _ => self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node),
        }
    }
}
