use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{BindingId, DeclarationId, SemanticCompilerFailure, UnsupportedSemanticFeature};

use super::super::super::super::model::{
    CheckedBufferRoot, CheckedContainerRoot, CheckedPlaceStep, CheckedSliceRoot, CheckedType,
};
use super::super::super::borrows::AccessKind;
use super::super::super::{CheckStop, Checker, FunctionSignature, LocalBinding};
use super::{CheckedBufferPlace, CheckedContainerPlace, CheckedIndexedPlace, CheckedSlicePlace};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_dereferenced_buffer_place(
        &self,
        node: NodeId,
        pbase: NodeId,
        base_suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let inner = self
            .tree
            .first_child_with(pbase, crate::Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let inner = self.resolve_explicit_place(node, inner, bindings)?;
        let mut place = self.resolve_explicit_dereference(node, pbase, inner)?;
        if place.holder_pending {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let (binding, mut path) = self.explicit_container_path(&place.expression, node)?;
        let (suffix_path, ty, offsets) = self.resolve_storage_path(
            base_suffixes,
            place.ty,
            bindings,
            function,
            loop_depth,
            true,
        )?;
        place.resolved.extend_storage(&suffix_path);
        path.extend(suffix_path);
        let fields = super::field_prefix(&path);
        let holder = place.borrow.as_ref().map(|_| place.declaration);
        match ty {
            CheckedType::Buffer { element } => {
                let Some(fields) = fields else {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
                };
                Ok(CheckedIndexedPlace::Buffer(CheckedBufferPlace {
                    root: CheckedBufferRoot {
                        binding,
                        fields,
                        element,
                    },
                    declaration: place.declaration,
                    element_type: element.ty(),
                    holder,
                    resolved: place.resolved,
                    borrow_kind: place.borrow.as_ref().map(|borrow| borrow.kind),
                }))
            }
            CheckedType::Slice {
                region,
                element,
                strength,
            } if matches!(fields.as_deref(), Some([])) => {
                let borrow = place
                    .borrow
                    .clone()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let local = bindings
                    .get(&place.declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.check_holder_not_suspended(local, node)?;
                self.check_loan_access(bindings, holder, &borrow.place, AccessKind::Read, node)?;
                let slice = local
                    .slice
                    .clone()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if slice.region != region {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(CheckedIndexedPlace::Slice(CheckedSlicePlace {
                    root: CheckedSliceRoot {
                        binding,
                        element,
                        strength,
                    },
                    declaration: place.declaration,
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
            CheckedType::Array { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Extent { .. } => {
                Ok(CheckedIndexedPlace::Container(CheckedContainerPlace {
                    root: CheckedContainerRoot {
                        root: crate::semantic::CheckedPlaceRoot::Binding(binding),
                        path,
                        ty,
                    },
                    resolved: place.resolved,
                    offsets,
                    holder,
                }))
            }
            _ => self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node),
        }
    }

    pub(in crate::semantic::check) fn explicit_container_path(
        &self,
        expression: &super::super::super::super::model::CheckedExpression,
        node: NodeId,
    ) -> Result<(BindingId, Vec<CheckedPlaceStep>), CheckStop> {
        use super::super::super::super::model::CheckedExpression;

        match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. } => Ok((*binding, Vec::new())),
            CheckedExpression::BoxDeref { nominal, value, .. } => {
                let (binding, mut path) = self.explicit_container_path(value, node)?;
                path.push(CheckedPlaceStep::BoxReferent(*nominal));
                Ok((binding, path))
            }
            CheckedExpression::ProjectValue { value, field, .. } => {
                let (binding, mut path) = self.explicit_container_path(value, node)?;
                path.push(CheckedPlaceStep::Field(*field));
                Ok((binding, path))
            }
            _ => self.unsupported(UnsupportedSemanticFeature::CompositeValues, node),
        }
    }
}
