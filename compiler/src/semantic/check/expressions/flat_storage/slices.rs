use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, FixedTerminal, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::super::model::{
    CheckedExpression, CheckedMode, CheckedSliceOrigin, CheckedSliceSource,
};
use super::super::super::borrows::{AccessKind, ResolvedPlace, SliceInfo, SliceLoan};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::CheckedIndexedPlace;

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(in crate::semantic::check) fn check_slice_of(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: "slice_of".to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        // [TYPE-5] `slice_of` is outside the retained-argument class, so it
        // carries no written argument: the region comes from the operand's own
        // borrow and the element from the place it views. A written argument
        // here is the rejection, not the supply.
        if self
            .tree
            .first_child_with(node, Production::Targs)?
            .is_some()
        {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let atoms = self.operation_atoms(node, 1)?;
        let borrow = self
            .tree
            .first_child_with(atoms[0], Production::BorrowExpr)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::TypeMismatch,
                )
            })?;
        if self.has_fixed(borrow, FixedTerminal::Uniq)? {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
        // [OP-2] the result region is the one the operand's borrow writes.
        let borrow_region = self.use_at(borrow, LexicalUseRole::BorrowRegion)?;
        let ResolvedTarget::Source {
            declaration: region,
            class: DeclarationClass::Region,
        } = borrow_region.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let place_node = self
            .tree
            .first_child_with(borrow, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase = self
            .tree
            .first_child_with(place_node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        let root_use = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = root_use.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let owner = match class {
            DeclarationClass::Value => Some(declaration),
            DeclarationClass::NamedConst => None,
            _ => {
                return self.issue_node(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::TypeMismatch,
                );
            }
        };
        self.check_direct_slice_borrow_lifetime(function, region, owner, borrow, loop_depth)?;
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let indexed = self.check_indexed_place(place_node, bindings, &suffixes, place_node)?;
        // [OP-2] the element is the viewed place's, and [STOR-4] still confines
        // a slice to flat elements — now judged on the derived one.
        //
        // DELIBERATELY UNTESTED, by the 2026-08-08 ruling. Once the element is
        // derived rather than written, no source appears to reach this arm:
        // `array<T, N>` and `buffer<T>` already require a flat T, so every
        // route tried — a non-copy struct element, a generic element, a nested
        // array element, an `array_new` of a struct — is rejected earlier by
        // TYPE-2 or by OP-1 on the array type itself, each confirmed with a
        // control that deletes the `slice_of` line and fails identically. That
        // is "not shown reachable", not "proven unreachable", so the rejection
        // stays. Widening what `array<T, N>` or `buffer<T>` admit re-opens the
        // question and owes this arm a test.
        let element_type = indexed.element_type();
        let Some(element) = self.flat_element(element_type)? else {
            return self.issue_node(
                SemanticRule::Op1,
                atoms[0],
                SemanticIssueKind::InvalidOperation,
            );
        };
        let (source, resolved) = match indexed {
            CheckedIndexedPlace::Array(array) => {
                let resolved = array.resolved_place().unwrap_or(ResolvedPlace {
                    root: declaration,
                    fields: Vec::new(),
                });
                (
                    CheckedSliceSource::Array {
                        root: array.root,
                        length: array.length,
                    },
                    resolved,
                )
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                (CheckedSliceSource::Buffer(buffer.root), buffer.resolved)
            }
            CheckedIndexedPlace::Slice(_) => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let origin = owner.map_or(CheckedSliceOrigin::ImmutableConst, |_| {
            CheckedSliceOrigin::SourcePlace {
                root: resolved.root,
                fields: resolved.fields.clone(),
                origin_region: None,
            }
        });
        let origins = vec![origin];
        let accesses = if let Some(owner) = owner {
            self.check_loan_access(bindings, None, &resolved, AccessKind::SharedBorrow, borrow)?;
            bindings
                .get_mut(&owner)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .push_slice_loan(SliceLoan {
                    region,
                    place: resolved.clone(),
                });
            vec![PlaceAccess {
                place: resolved,
                kind: AccessKind::SharedBorrow,
            }]
        } else {
            Vec::new()
        };
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                source,
                region,
                element,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(SliceInfo { region, origins }),
            holder: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses,
        })
    }
}
