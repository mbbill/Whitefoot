use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedContainerRoot, CheckedExpression, CheckedMode, CheckedNominalKind, CheckedSetTarget,
    CheckedType,
};
use super::super::super::places::PlaceProjection;
use super::super::borrows::{AccessKind, BorrowInfo, ResolvedPlace};
use super::super::{CheckStop, Checker, EffectSet, LocalBinding, PlaceAccess, TypedExpression};
use super::{MutationAccess, MutationForm, MutationTarget, PlaceUseContext, PlaceUseOptions};

pub(super) struct ExplicitPlace {
    pub(super) declaration: DeclarationId,
    pub(super) ty: CheckedType,
    pub(super) mode: CheckedMode,
    pub(super) borrow: Option<BorrowInfo>,
    pub(super) holder_pending: bool,
    pub(super) expression: CheckedExpression,
    pub(super) resolved: ResolvedPlace,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_dereferenced_place_use(
        &self,
        use_node: NodeId,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        if self.is_direct_borrow_holder(pbase, bindings)? {
            return self.check_direct_borrowed_place_use(use_node, node, pbase, bindings, options);
        }
        let place = self.resolve_explicit_place(use_node, node, bindings)?;
        if place.holder_pending {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        self.check_commit_place_live(&place.resolved, use_node, false)?;
        let copy = self.is_copy_type(place.ty)?;
        if !copy && options.explicit_move && self.is_box_descendant_read_out(&place.resolved) {
            return self.unsupported(UnsupportedSemanticFeature::BoxReferentMove, use_node);
        }
        let read_out = !copy
            && options.explicit_move
            && !matches!(place.mode, CheckedMode::Shared(_))
            && self.take_commit_read_out(&place.resolved);
        if !copy && !read_out {
            if options.explicit_move && place.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Own5,
                    use_node,
                    SemanticIssueKind::BorrowConflict,
                );
            }
            if place.mode == CheckedMode::Own {
                return self.unsupported(UnsupportedSemanticFeature::BoxReferentMove, use_node);
            }
            if matches!(options.context, PlaceUseContext::Ordinary) {
                return self.issue_node(
                    SemanticRule::Own1,
                    use_node,
                    SemanticIssueKind::BareAffineUse {
                        mechanical_fix: "write `move p` for the affine place",
                    },
                );
            }
        }
        if copy && options.explicit_move && self.judges_class_spelling() {
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the copy place without `move`",
                },
            );
        }
        if place.borrow.is_some() {
            let local = bindings
                .get(&place.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.check_holder_not_suspended(local, use_node)?;
        }
        self.check_loan_access(
            bindings,
            place.borrow.as_ref().map(|_| place.declaration),
            &place.resolved,
            if read_out {
                AccessKind::Move
            } else {
                AccessKind::Read
            },
            use_node,
        )?;
        let mut effects = EffectSet::NONE;
        // [EFF-1] a loan-bearing parameter's effect path names the viewed
        // backing state and not the descriptor, and merely moving, returning
        // or structurally repacking that value observes none of it: a read
        // *through* the view is the subscript's own attribution. Before the
        // shared view became copy this arm was reached only by a consume,
        // which exhibited the same wrong read; the copy spelling is what made
        // an accepted program declare it.
        if !Self::checked_type_is_loan_bearing(place.ty) {
            for path in self.effect_paths_for_place(use_node, &place.resolved, bindings)? {
                effects.add_read(path);
            }
        }
        let (mode, borrow, holder) = if copy || read_out {
            (CheckedMode::Own, None, None)
        } else {
            (
                place.mode,
                place.borrow.clone().map(|borrow| BorrowInfo {
                    place: place.resolved.clone(),
                    ..borrow
                }),
                Some(place.declaration),
            )
        };
        let expression = if read_out {
            let (binding, path) = self.explicit_container_path(&place.expression, node)?;
            CheckedExpression::ReadStorage {
                carrier: self.tree.path(use_node)?.clone(),
                root: CheckedContainerRoot {
                    root: crate::semantic::CheckedPlaceRoot::Binding(binding),
                    path,
                    ty: place.ty,
                },
            }
        } else {
            place.expression
        };
        Ok(TypedExpression {
            expression,
            mode,
            borrow,
            slice: None,
            holder,
            reference_value: false,
            effects,
            accesses: vec![PlaceAccess {
                place: place.resolved,
                kind: AccessKind::Read,
            }],
        })
    }

    fn is_direct_borrow_holder(
        &self,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<bool, CheckStop> {
        let holder = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let holder_base = self
            .tree
            .first_child_with(holder, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.tree.children(holder_base)?.is_empty()
            || !self
                .tree
                .children_with(holder, Production::Psuffix)?
                .is_empty()
        {
            return Ok(false);
        }
        let usage = self.use_at(holder_base, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(false);
        };
        Ok(bindings
            .get(&declaration)
            .is_some_and(|local| local.borrow.is_some()))
    }

    /// OWN-14 admits a returned reborrow only over `deref(h)` and suffixes,
    /// where `h` itself is a holder binding. An extra owning dereference
    /// cannot be hidden inside that holder position.
    pub(in crate::semantic::check) fn check_returned_reborrow_holder_shape(
        &self,
        node: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        if self.is_direct_borrow_holder(pbase, bindings)? {
            return Ok(());
        }
        let place = self.resolve_explicit_place(node, place_node, bindings)?;
        if let Some(parent) = &place.borrow {
            let local = bindings
                .get(&place.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.check_holder_not_suspended(local, node)?;
            self.check_returned_reborrow_lifetime(place.declaration, parent, region, node)?;
            return self.issue_node(
                SemanticRule::Own14,
                node,
                SemanticIssueKind::InvalidReborrowPosition {
                    mechanical_fix: super::super::borrows::OWN14_RESTRUCTURING,
                },
            );
        }
        Ok(())
    }

    fn check_direct_borrowed_place_use(
        &self,
        use_node: NodeId,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let (declaration, local, borrow) =
            self.resolve_dereference_holder(node, pbase, bindings)?;
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        let copy = self.is_copy_type(ty)?;
        // [LIV-2] the read-out of a `deref` target of this statement's commit
        // is admitted on exactly [SET-2]'s exchange ground: the exchange
        // leaves the referent place owning one valid value at every program
        // point, so the sole [OWN-5] exception covers this move as well.
        let mut read_out_place = borrow.place.clone();
        read_out_place.extend_fields(&fields);
        self.check_commit_place_live(&read_out_place, use_node, false)?;
        let read_out = !copy && options.explicit_move && self.take_commit_read_out(&read_out_place);
        if !copy && !read_out {
            if options.explicit_move {
                return self.issue_node(
                    SemanticRule::Own5,
                    use_node,
                    SemanticIssueKind::BorrowConflict,
                );
            }
            // An affine referent may still be matched through the borrow:
            // [OWN-13] leaves the scrutinee live and derives borrowed payload
            // binders. Every other bare use is the [OWN-1] error.
            if matches!(options.context, PlaceUseContext::Ordinary) {
                return self.issue_node(
                    SemanticRule::Own1,
                    use_node,
                    SemanticIssueKind::BareAffineUse {
                        mechanical_fix: "write `move p` for the affine place",
                    },
                );
            }
        }
        if copy && options.explicit_move && self.judges_class_spelling() {
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the copy place without `move`",
                },
            );
        }
        self.check_holder_not_suspended(&local, use_node)?;
        let mut resolved = borrow.place.clone();
        resolved.extend_fields(&fields);
        self.check_loan_access(
            bindings,
            Some(declaration),
            &resolved,
            AccessKind::Read,
            use_node,
        )?;
        let mut effects = EffectSet::NONE;
        // [EFF-1] as above: the descriptor read through a holder observes the
        // viewed state no more than a direct one does.
        if !Self::checked_type_is_loan_bearing(ty) {
            for path in self.effect_paths_for_place(use_node, &resolved, bindings)? {
                effects.add_read(path);
            }
        }
        let expression = if !fields.is_empty() {
            CheckedExpression::Project {
                carrier: self.tree.path(use_node)?.clone(),
                binding: local.binding,
                state_origins: local
                    .state_origins
                    .clone()
                    .map(|origins| origins.projected(&fields)),
                fields: fields.clone(),
                ty,
                consume_root: false,
                residual_drops: Vec::new(),
            }
        } else if self.borrow_addresses_storage(ty)? {
            CheckedExpression::DerefAddressed {
                carrier: self.tree.path(use_node)?.clone(),
                binding: local.binding,
                ty,
            }
        } else {
            CheckedExpression::Binding {
                carrier: self.tree.path(use_node)?.clone(),
                binding: local.binding,
                state_origins: local.state_origins.clone(),
                ty,
                slice_origins: Vec::new(),
                consume_root: false,
            }
        };
        // A read-out delivers the referent's own value: the previous owner
        // leaves through this move and the commit reinitializes the place
        // [LIV-2], so the value is `own` and carries no borrow.
        if copy || read_out {
            return Ok(TypedExpression::owned_with_access(
                expression,
                effects,
                resolved,
                AccessKind::Read,
            ));
        }
        let mode = borrow.mode();
        let mut place = borrow.place.clone();
        place.extend_fields(&fields);
        Ok(TypedExpression {
            expression,
            mode,
            borrow: Some(BorrowInfo { place, ..borrow }),
            slice: None,
            holder: Some(declaration),
            reference_value: false,
            effects,
            accesses: vec![PlaceAccess {
                place: resolved,
                kind: AccessKind::Read,
            }],
        })
    }

    pub(super) fn resolve_explicit_place(
        &self,
        carrier: NodeId,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<ExplicitPlace, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mut place = if self.has_fixed(pbase, FixedTerminal::Deref)? {
            let inner = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let inner = self.resolve_explicit_place(carrier, inner, bindings)?;
            self.resolve_explicit_dereference(carrier, pbase, inner)?
        } else {
            if !self.tree.children(pbase)?.is_empty() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, pbase);
            }
            let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            let local = bindings
                .get(&declaration)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if !local.live {
                return self.issue_node(
                    SemanticRule::Own1,
                    node,
                    SemanticIssueKind::UseAfterMove {
                        mechanical_fix: "introduce a new `let` binding before reuse",
                    },
                );
            }
            ExplicitPlace {
                declaration,
                ty: local.ty,
                mode: local.mode,
                borrow: local.borrow.clone(),
                holder_pending: local.mode != CheckedMode::Own,
                expression: CheckedExpression::Binding {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    state_origins: local.state_origins.clone(),
                    ty: local.ty,
                    slice_origins: local
                        .slice
                        .as_ref()
                        .map(|slice| slice.origins.clone())
                        .unwrap_or_default(),
                    consume_root: false,
                },
                resolved: local.borrow.map_or_else(
                    || ResolvedPlace::fields(declaration, Vec::new()),
                    |borrow| borrow.place,
                ),
            }
        };

        for suffix in self.tree.children_with(node, Production::Psuffix)? {
            // A subscript selects a composite element value, which this
            // version does not implement for explicit deref chains.
            if self.subscript_offset(suffix)?.is_some() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, suffix);
            }
            if place.holder_pending {
                return self.issue_node(
                    SemanticRule::Type7,
                    suffix,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(holder)`",
                    },
                );
            }
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling();
            let CheckedType::Nominal(nominal) = place.ty else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        "a source struct, whose declared field this suffix selects",
                        self.checked_type_name(place.ty)?,
                    ),
                );
            };
            let CheckedNominalKind::Struct { fields } = &self.nominal(nominal)?.kind else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        "a source struct, whose declared field this suffix selects",
                        self.checked_type_name(place.ty)?,
                    ),
                );
            };
            let Some((index, field)) = fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == name)
            else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        format!("a declared field of {}", self.checked_type_name(place.ty)?),
                        format!("the field name `{name}`, which that struct does not declare"),
                    ),
                );
            };
            let field_index =
                u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            place.expression = CheckedExpression::ProjectValue {
                carrier: self.tree.path(carrier)?.clone(),
                value: Box::new(place.expression),
                nominal,
                field: field_index,
                ty: field.ty,
            };
            place.ty = field.ty;
            place.resolved.extend_fields(&[field_index]);
        }
        Ok(place)
    }

    pub(super) fn resolve_explicit_dereference(
        &self,
        carrier: NodeId,
        pbase: NodeId,
        mut inner: ExplicitPlace,
    ) -> Result<ExplicitPlace, CheckStop> {
        if inner.holder_pending {
            if self.borrow_addresses_storage(inner.ty)? {
                let CheckedExpression::Binding { binding, .. } = inner.expression else {
                    return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
                };
                // Explicit dereference chains must first read the value
                // stored behind the borrow. A later Box dereference reads
                // that value's referent, not the owner's pointer slot.
                inner.expression = CheckedExpression::DerefAddressed {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding,
                    ty: inner.ty,
                };
            }
            inner.holder_pending = false;
            return Ok(inner);
        }

        let CheckedType::Nominal(nominal) = inner.ty else {
            return self.issue_node(
                SemanticRule::Type7,
                pbase,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "deref requires a borrow, box, or arena place",
                },
            );
        };
        match self.nominal(nominal)?.kind {
            CheckedNominalKind::Box { referent, .. } => {
                inner.expression = CheckedExpression::BoxDeref {
                    carrier: self.tree.path(carrier)?.clone(),
                    nominal,
                    referent,
                    value: Box::new(inner.expression),
                };
                inner.ty = referent;
                inner.resolved.storage_path.push(PlaceProjection::Deref);
            }
            CheckedNominalKind::Arena { content, .. } => {
                inner.expression = CheckedExpression::ArenaDeref {
                    carrier: self.tree.path(carrier)?.clone(),
                    nominal,
                    content,
                    value: Box::new(inner.expression),
                };
                inner.ty = content;
                inner.resolved.storage_path.push(PlaceProjection::Deref);
            }
            _ => {
                return self.issue_node(
                    SemanticRule::Type7,
                    pbase,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "deref requires a borrow, box, or arena place",
                    },
                );
            }
        }
        Ok(inner)
    }

    /// [SET-1, SET-2] an owning Box edge is a typed storage projection, both
    /// from an own binding and after dereferencing an exclusive holder.
    pub(super) fn check_box_storage_set_target(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        form: MutationForm,
    ) -> Result<Option<MutationTarget>, CheckStop> {
        let place = self.resolve_explicit_place(node, node, bindings)?;
        if !place
            .resolved
            .storage_path
            .contains(&PlaceProjection::Deref)
        {
            return Ok(None);
        }
        if matches!(place.mode, CheckedMode::Shared(_)) {
            return self.issue_node(SemanticRule::Own5, node, SemanticIssueKind::BorrowConflict);
        }
        let holder = place.borrow.as_ref().map(|_| place.declaration);
        if holder.is_some() {
            let local = bindings
                .get(&place.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.check_holder_not_suspended(local, node)?;
        }
        self.check_loan_access(bindings, holder, &place.resolved, AccessKind::Write, node)?;
        self.check_mutation_target_class(node, place.ty, form)?;
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(node, &place.resolved, bindings)? {
            effects.add_write(path.clone());
            if form.is_replace() {
                effects.add_read(path);
            }
        }
        let (binding, path) = self.explicit_container_path(&place.expression, node)?;
        Ok(Some(MutationTarget {
            declaration: place.declaration,
            access: MutationAccess::Place {
                holder,
                place: place.resolved.clone(),
            },
            place: place.resolved,
            element: false,
            target: CheckedSetTarget::Storage(CheckedContainerRoot {
                root: crate::semantic::CheckedPlaceRoot::Binding(binding),
                path,
                ty: place.ty,
            }),
            effects,
            unsupported: None,
        }))
    }
}
