use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{DeclarationId, SemanticCompilerFailure, UnsupportedSemanticFeature};

use super::super::super::super::model::{CheckedBufferRoot, CheckedSliceRoot, CheckedType};
use super::super::super::{CheckStop, Checker, LocalBinding};
use super::{CheckedBufferPlace, CheckedIndexedPlace, CheckedSlicePlace};

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
            _ => self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node),
        }
    }
}
