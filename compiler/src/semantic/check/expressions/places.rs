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
        let place = self.resolve_explicit_place(node, bindings)?;
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
        if let Some(region) = place
            .borrow
            .as_ref()
            .and_then(|borrow| borrow.origin_region)
        {
            effects.add_read(region);
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
        let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
        if !self.is_copy_type(ty)? {
            if !options.explicit_move {
                return self.issue_node(
                    SemanticRule::Own1,
                    use_node,
                    SemanticIssueKind::BareAffineUse {
                        mechanical_fix: "write `move p` for the affine place",
                    },
                );
            }
            return self.issue_node(
                SemanticRule::Own5,
                use_node,
                SemanticIssueKind::BorrowConflict,
            );
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
        let mut resolved = borrow.place;
        resolved.fields.extend_from_slice(&fields);
        self.check_loan_access(
            bindings,
            Some(declaration),
            &resolved,
            AccessKind::Read,
            use_node,
        )?;
        let mut effects = EffectSet::NONE;
        if let Some(region) = borrow.origin_region {
            effects.add_read(region);
        }
        let expression = if fields.is_empty() {
            CheckedExpression::Binding {
                binding: local.binding,
                ty,
            }
        } else {
            CheckedExpression::Project {
                binding: local.binding,
                fields,
                ty,
                consume_root: false,
                residual_drops: Vec::new(),
            }
        };
        Ok(TypedExpression::owned_with_access(
            expression,
            effects,
            resolved,
            AccessKind::Read,
        ))
    }

    fn resolve_explicit_place(
        &self,
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
            let mut inner = self.resolve_explicit_place(inner, bindings)?;
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
                let CheckedNominalKind::Box { referent } = self.nominal(nominal)?.kind else {
                    return self.issue_node(
                        SemanticRule::Type7,
                        pbase,
                        SemanticIssueKind::MissingDereference {
                            mechanical_fix: "deref requires a borrow, box, or arena place",
                        },
                    );
                };
                inner.expression = CheckedExpression::BoxDeref {
                    nominal,
                    referent,
                    value: Box::new(inner.expression),
                };
                inner.ty = referent;
                inner
            }
        } else {
            if self.has_fixed(pbase, FixedTerminal::Index)?
                || !self.tree.children(pbase)?.is_empty()
            {
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
                    binding: local.binding,
                    ty: local.ty,
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
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let CheckedNominalKind::Struct { fields } = &self.nominal(nominal)?.kind else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
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
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let field_index =
                u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            place.expression = CheckedExpression::ProjectValue {
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
