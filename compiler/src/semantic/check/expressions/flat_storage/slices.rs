use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, FixedTerminal, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::super::model::{CheckedExpression, CheckedMode, CheckedSliceSource};
use super::super::super::borrows::{AccessKind, ResolvedPlace, SliceInfo};
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
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(SemanticRule::Fn2, node, SemanticIssueKind::InvalidOperation)
            })?;
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [region_argument, element_argument] = arguments.as_slice() else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let region_use = self.use_at(*region_argument, LexicalUseRole::TypeArgumentRegion)?;
        let ResolvedTarget::Source {
            declaration: region,
            class: DeclarationClass::Region,
        } = region_use.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let element_node = self
            .tree
            .first_child_with(*element_argument, Production::Type)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Op1,
                    *element_argument,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let element_type = self.parse_type_with(element_node, &function.substitution)?;
        let Some(element) = self.flat_element(element_type)? else {
            return self.issue_node(
                SemanticRule::Op1,
                element_node,
                SemanticIssueKind::InvalidOperation,
            );
        };
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
        let borrow_region = self.use_at(borrow, LexicalUseRole::BorrowRegion)?;
        let ResolvedTarget::Source {
            declaration: written_region,
            class: DeclarationClass::Region,
        } = borrow_region.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if written_region != region {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
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
        let indexed = self.check_indexed_place(place_node, bindings)?;
        if indexed.element_type() != element_type {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
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
        self.check_loan_access(bindings, None, &resolved, AccessKind::SharedBorrow, borrow)?;
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                source,
                region,
                element,
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(SliceInfo {
                region,
                place: resolved.clone(),
                origin_region: None,
            }),
            holder: None,
            effects: EffectSet::NONE,
            accesses: vec![PlaceAccess {
                place: resolved,
                kind: AccessKind::SharedBorrow,
            }],
        })
    }
}
