use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{DeclarationId, UnsupportedSemanticFeature};

use super::super::super::super::model::{CheckedBufferRoot, CheckedType};
use super::super::super::{CheckStop, Checker, LocalBinding};
use super::{CheckedBufferPlace, CheckedIndexedPlace};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_dereferenced_buffer_place(
        &self,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let (declaration, local, borrow) =
            self.resolve_dereference_holder(node, pbase, bindings)?;
        let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
        let CheckedType::Buffer { element } = ty else {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node);
        };
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
            origin_region: borrow.origin_region,
            borrow_kind: Some(borrow.kind),
        }))
    }
}
