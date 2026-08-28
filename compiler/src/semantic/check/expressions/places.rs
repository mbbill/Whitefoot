use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{CheckedExpression, CheckedMode, CheckedNominalKind, CheckedType};
use super::super::borrows::{AccessKind, BorrowInfo, ResolvedPlace};
use super::super::{CheckStop, Checker, EffectSet, LocalBinding, PlaceAccess, TypedExpression};
use super::{PlaceUseContext, PlaceUseOptions};

struct ExplicitPlace {
    declaration: DeclarationId,
    ty: CheckedType,
    mode: CheckedMode,
    borrow: Option<BorrowInfo>,
    holder_pending: bool,
    expression: CheckedExpression,
    resolved: ResolvedPlace,
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
        let copy = self.is_copy_type(place.ty)?;
        if !copy {
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
        if options.explicit_move {
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the copy place without `move`",
                },
            );
        }
        if place.borrow.is_some() {
            self.check_loan_access(
                bindings,
                Some(place.declaration),
                &place.resolved,
                AccessKind::Read,
                use_node,
            )?;
        }
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(&place.resolved, bindings)? {
            effects.add_read(path);
        }
        let (mode, borrow, holder) = if copy {
            (CheckedMode::Own, None, None)
        } else {
            (place.mode, place.borrow.clone(), Some(place.declaration))
        };
        Ok(TypedExpression {
            expression: place.expression,
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
        if !copy {
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
        if copy && options.explicit_move {
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
        resolved.fields.extend_from_slice(&fields);
        self.check_loan_access(
            bindings,
            Some(declaration),
            &resolved,
            AccessKind::Read,
            use_node,
        )?;
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(&resolved, bindings)? {
            effects.add_read(path);
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
        if copy {
            return Ok(TypedExpression::owned_with_access(
                expression,
                effects,
                resolved,
                AccessKind::Read,
            ));
        }
        let mode = borrow.mode();
        let mut place = borrow.place.clone();
        place.fields.extend_from_slice(&fields);
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

    fn resolve_explicit_place(
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
            let mut inner = self.resolve_explicit_place(carrier, inner, bindings)?;
            if inner.holder_pending {
                inner.holder_pending = false;
                inner
            } else {
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
                    CheckedNominalKind::Box { referent } => {
                        inner.expression = CheckedExpression::BoxDeref {
                            carrier: self.tree.path(carrier)?.clone(),
                            nominal,
                            referent,
                            value: Box::new(inner.expression),
                        };
                        inner.ty = referent;
                    }
                    CheckedNominalKind::Arena { content, .. } => {
                        inner.expression = CheckedExpression::ArenaDeref {
                            carrier: self.tree.path(carrier)?.clone(),
                            nominal,
                            content,
                            value: Box::new(inner.expression),
                        };
                        inner.ty = content;
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
                inner
            }
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
                    || ResolvedPlace {
                        root: declaration,
                        fields: Vec::new(),
                    },
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
            place.resolved.fields.push(field_index);
        }
        Ok(place)
    }
}
