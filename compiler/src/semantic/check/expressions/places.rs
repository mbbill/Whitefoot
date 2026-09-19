//! Places written with an explicit `deref` [GRAM-5, TYPE-7].
//!
//! `pbase := IDENT | "deref" "(" place ")" | "entry" "(" IDENT ")"` and the
//! `deref` alternative takes any `place` whose selected kind is a reference,
//! `&T` or `&[T]`, and spells its referent. A `Box`'s content is *not* reached
//! that way: it is the field `inner`, an ordinary field step [TYPE-9], and a
//! `deref` of anything that is not a reference is [TYPE-7]'s rejection.
//!
//! Resolving a place whose `deref` step names a reference variable replaces
//! that step with the path the reference names, recursively, and every
//! [OWN-7] judgment reads the resolved place [REF-1].

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedContainerRoot, CheckedExpression, CheckedMeasure, CheckedMode, CheckedNominalKind,
    CheckedType,
};
use super::super::super::places::{PlaceRoot, PlaceStep, ResolvedPlace};
use super::super::references::{AccessKind, OWN1_ROOTED_CONSUME, WIN3_NO_TAKE};
use super::super::{CheckStop, Checker, EffectSet, LocalBinding, PlaceAccess, TypedExpression};
use super::{PlaceUseContext, PlaceUseOptions};

/// One place resolved through an explicit `deref` chain.
///
/// The v0.59 `borrow` and `holder_pending` fields are gone with the loans: a
/// reference is not storage of its own, so a resolved place never carries one
/// — the `deref` step has already been replaced by the path the reference
/// names [REF-1].
pub(super) struct ExplicitPlace {
    /// The source declaration the *written* base names. For a `deref` chain
    /// that is the reference binding, whose [REF-2] validity the caller
    /// rechecks; `resolved` below is rooted at what that reference names.
    pub(super) declaration: DeclarationId,
    pub(super) ty: CheckedType,
    pub(super) mode: CheckedMode,
    pub(super) expression: CheckedExpression,
    pub(super) resolved: ResolvedPlace,
    /// [OP-15, MSR-1] the measure a trailing `.len`, `.cap`, `.room` or
    /// `.head` reads. A measure selects no storage below itself, so it is
    /// always the last written suffix and the place it is read over is the
    /// one this record otherwise describes.
    pub(super) measure: Option<super::super::super::model::CheckedMeasure>,
    /// Whether the written base was `deref` of a `&[T]` range reference
    /// [REF-4]. [MSR-1] gives `&[T]` its own row, whose one cell is the
    /// range's element count, and the referent type the deref selects is the
    /// element type, so the row cannot be recovered from that type.
    pub(super) range_referent: bool,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// A read of a place written through an explicit `deref` [TYPE-7].
    pub(super) fn check_dereferenced_place_use(
        &self,
        use_node: NodeId,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let place = self.resolve_explicit_place(use_node, node, bindings)?;
        self.check_commit_place_live(&place.resolved, use_node, false)?;
        // [OP-15, MSR-1] a measure is a read-only `own u64` member of the
        // measured place: it reads that place's descriptor storage and
        // nothing below it, so it is neither a copy nor an affine use of the
        // value itself.
        if let Some(measure) = place.measure {
            let mut effects = EffectSet::NONE;
            for path in
                self.effect_paths_for_descriptor(use_node, &place.resolved, bindings, measure)?
            {
                effects.add_read(path);
            }
            let (binding, path) = self.explicit_container_path(&place.expression, node)?;
            // [REF-4, MSR-1] a range reference's one measure is its element
            // count, and the row it belongs to is fixed by the written base
            // rather than by the type the dereference selects [TYPE-7].
            if place.range_referent {
                if !path.is_empty() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let Some(element) = self.flat_element(place.ty)? else {
                    return self
                        .unsupported(UnsupportedSemanticFeature::CompositeValues, use_node);
                };
                return Ok(TypedExpression {
                    expression: CheckedExpression::RangeMeasure {
                        measure,
                        root: super::super::super::model::CheckedRangeRoot { binding, element },
                    },
                    mode: CheckedMode::Own,
                    reference: None,
                    reference_value: false,
                    effects,
                    accesses: vec![PlaceAccess {
                        place: place.resolved,
                        kind: AccessKind::Read,
                    }],
                });
            }
            return Ok(TypedExpression {
                expression: CheckedExpression::ContainerMeasure {
                    measure,
                    root: CheckedContainerRoot {
                        root: PlaceRoot::Binding(binding),
                        path,
                        ty: place.ty,
                    },
                },
                mode: CheckedMode::Own,
                reference: None,
                reference_value: false,
                effects,
                accesses: vec![PlaceAccess {
                    place: place.resolved,
                    kind: AccessKind::Read,
                }],
            });
        }
        // [TYPE-8] `&[T]` is a reference kind and not a type, so `deref(p)`
        // of a range reference denotes the run it names and never a value of
        // its own: the admitted readers are its one measure above and one
        // subscript [MSR-1, OP-4], both of which resolve before this point.
        if place.range_referent {
            return self.issue_node(
                SemanticRule::Type5,
                use_node,
                SemanticIssueKind::type_mismatch(
                    "a value place",
                    "the run a range reference names, which is read by `deref(p)[i]` or \
                     `deref(p).len` [REF-4, MSR-1]",
                ),
            );
        }
        let copy = self.is_copy_type(place.ty)?;
        let read_out = !copy && options.explicit_move && self.take_commit_read_out(&place.resolved);
        if !copy && !read_out {
            if options.explicit_move {
                // [OWN-1] a consume is admitted only for a place rooted in a
                // live own-mode binding. A place written under a `deref` is
                // rooted at the storage the reference names, which this
                // function does not own, so the `move` is refused there and
                // not by [WIN-3], whose subject is a window slot or an array
                // element.
                if self.has_fixed(pbase, FixedTerminal::Deref)? {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::MoveThroughReference {
                            mechanical_fix: OWN1_ROOTED_CONSUME,
                        },
                    );
                }
                // [WIN-3] there is no take operation and no hole: a move out
                // of a place a reference names would leave storage no owner
                // can account for.
                return self.issue_node(
                    SemanticRule::Win3,
                    use_node,
                    SemanticIssueKind::InvalidElementMove {
                        mechanical_fix: WIN3_NO_TAKE,
                    },
                );
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
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(use_node, &place.resolved, bindings)? {
            effects.add_read(path);
        }
        let expression = if read_out {
            let (binding, path) = self.explicit_container_path(&place.expression, node)?;
            CheckedExpression::ReadStorage {
                carrier: self.tree.path(use_node)?.clone(),
                root: CheckedContainerRoot {
                    root: PlaceRoot::Binding(binding),
                    path,
                    ty: place.ty,
                },
            }
        } else {
            place.expression
        };
        Ok(TypedExpression {
            expression,
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects,
            accesses: vec![PlaceAccess {
                place: place.resolved,
                kind: AccessKind::Read,
            }],
        })
    }

    /// [GRAM-5, REF-1] the complete written `place`, with every `deref` step
    /// already replaced by the path the reference it names names.
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
            self.resolve_explicit_dereference(carrier, pbase, inner, bindings)?
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
                expression: CheckedExpression::Binding {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    ty: local.ty,
                    consume_root: false,
                },
                resolved: ResolvedPlace::binding(local.binding),
                measure: None,
                range_referent: false,
            }
        };

        for suffix in self.tree.children_with(node, Production::Psuffix)? {
            // [OP-15] a measure is read as a member of the measured place,
            // and [MSR-1] gives it no storage below itself, so it ends the
            // written path. [TYPE-10] refuses a write of one and a read of a
            // window part; a read of a measure is exactly this form.
            if place.measure.is_some() {
                return self.issue_node(
                    SemanticRule::Type10,
                    suffix,
                    SemanticIssueKind::ReservedPseudoField {
                        spelling: self
                            .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                            .spelling()
                            .to_owned(),
                        mechanical_fix: "a measure selects no member of its own [MSR-1]",
                    },
                );
            }
            // A subscript selects a composite element value, which this
            // version does not implement for explicit deref chains; the
            // indexed path in `flat_storage` owns [OP-4].
            if self.subscript_offset(suffix)?.is_some() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, suffix);
            }
            // [TYPE-7] a reference binding used where a value of its referent
            // type is expected needs `deref(.)`; a suffix on one is exactly
            // that use.
            if place.mode.is_reference() {
                return self.issue_node(
                    SemanticRule::Type7,
                    suffix,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(p)`",
                    },
                );
            }
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling()
                .to_owned();
            // [TYPE-10] the eight measure and window-part spellings are
            // reserved from every field name [FORM-3], so one of them here is
            // a read of a pseudo-field rather than a struct field. A measure
            // read is [OP-15]'s place form and is resolved by the measure
            // path before this walker sees it, so a spelling arriving here is
            // one of the positions TYPE-10 refuses.
            if let Some(measure) = super::super::types::measure_named(&name) {
                // [OP-14, PRE-1] at a boxed argument a compiler-owned row's
                // measure place instantiates as `window.inner`: the shape
                // parameter W admits `Box<Slots<T>>` and `Box<Ring<T>>`, and
                // the clause then reads the content's measure, exactly as any
                // `Box` content is reached [TYPE-9]. Only a row's own clause
                // takes this step; source code writes the `inner` field
                // itself.
                if !place.range_referent
                    && self.tree.is_prelude_node(suffix)?
                    && let CheckedType::Nominal(nominal) = place.ty
                    && let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind
                    && super::super::expressions::flat_storage::measured_kind_of(place.ty)
                        .is_none()
                    && super::super::expressions::flat_storage::measured_kind_of(referent)
                        .is_some()
                {
                    place.expression = CheckedExpression::BoxDeref {
                        carrier: self.tree.path(carrier)?.clone(),
                        nominal,
                        referent,
                        value: Box::new(place.expression),
                    };
                    place.ty = referent;
                    place.resolved.path.push(PlaceStep::Deref);
                }
                let measured = if place.range_referent {
                    Some(super::super::super::model::MeasuredKind::Range)
                } else {
                    super::super::expressions::flat_storage::measured_kind_of(place.ty)
                };
                let Some(measured) = measured else {
                    return self.issue_node(
                        SemanticRule::Type5,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "a measured place [MSR-1]",
                            self.checked_type_name(place.ty)?,
                        ),
                    );
                };
                if matches!(
                    measure.cell(measured),
                    super::super::super::model::MeasureCell::Absent
                ) {
                    return self.issue_node(
                        SemanticRule::Type5,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "a measured place whose measure table has this row",
                            self.checked_type_name(place.ty)?,
                        ),
                    );
                }
                place.measure = Some(measure);
                continue;
            }
            self.reject_reserved_pseudo_field(suffix, &name)?;
            // [TYPE-9] a `Box`'s content is its field `inner`, reached by the
            // ordinary field step and never by `deref`.
            if let CheckedType::Nominal(nominal) = place.ty
                && let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind
            {
                if name != "inner" {
                    return self.issue_node(
                        SemanticRule::Type9,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "the Box content field `inner`",
                            format!("the field name `{name}`, which a Box does not declare"),
                        ),
                    );
                }
                place.expression = CheckedExpression::BoxDeref {
                    carrier: self.tree.path(carrier)?.clone(),
                    nominal,
                    referent,
                    value: Box::new(place.expression),
                };
                place.ty = referent;
                place.resolved.path.push(PlaceStep::Deref);
                continue;
            }
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
            let field_type = field.ty;
            place.expression = CheckedExpression::ProjectValue {
                carrier: self.tree.path(carrier)?.clone(),
                value: Box::new(place.expression),
                nominal,
                field: field_index,
                ty: field_type,
            };
            place.ty = field_type;
            place.resolved.extend_fields(&[field_index]);
        }
        Ok(place)
    }

    /// [TYPE-7] `deref(place)` denotes the referent of a reference, and
    /// nothing else.
    ///
    /// [REF-1] resolving a place whose `deref` step names a reference
    /// variable replaces that step with the path that reference names, so the
    /// resolved place below this step is rooted wherever that path is rooted.
    pub(super) fn resolve_explicit_dereference(
        &self,
        carrier: NodeId,
        pbase: NodeId,
        mut inner: ExplicitPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<ExplicitPlace, CheckStop> {
        if !inner.mode.is_reference() {
            return self.issue_node(
                SemanticRule::Type7,
                pbase,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix:
                        "a Box's content is its field inner [TYPE-9]; an owned place is named \
                         as itself",
                },
            );
        }
        let local = bindings
            .get(&inner.declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.check_reference_valid(local, pbase)?;
        let binding = local.binding;
        let named = self.resolve_reference_root(inner.declaration, bindings)?;
        let [path] = named.as_slice() else {
            // [REF-1] a reference whose incoming edges name different paths
            // carries the union of their sets, and every check on it must
            // hold for every member. One written place selects one storage,
            // so lowering a `deref` of such a union needs a representation
            // this compiler does not have. That is a capability limit and
            // never a source rejection.
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, pbase);
        };
        // The referent is read through the reference; the reference itself
        // stays a distinct expression so lowering never has to guess whether
        // a reference binding is passed on or read through [TYPE-7].
        inner.expression = CheckedExpression::DerefAddressed {
            carrier: self.tree.path(carrier)?.clone(),
            binding,
            ty: inner.ty,
        };
        inner.range_referent = inner.mode == CheckedMode::Range;
        inner.mode = CheckedMode::Own;
        inner.resolved = path.clone();
        Ok(inner)
    }

    /// The measure a written `psuffix` run ends with [OP-15, MSR-1], if any.
    ///
    /// [MSR-1] gives a measure no storage below itself, so only the last
    /// suffix can name one and a subscript never does. The spelling decides
    /// this without a type because [FORM-3] reserves the four names from
    /// every field, parameter, binder and result binding.
    pub(in crate::semantic::check) fn trailing_measure_member(
        &self,
        suffixes: &[NodeId],
    ) -> Result<Option<CheckedMeasure>, CheckStop> {
        let Some(&last) = suffixes.last() else {
            return Ok(None);
        };
        if self.subscript_offset(last)?.is_some() {
            return Ok(None);
        }
        let name = self
            .deferred_use_at(last, DeferredUseRole::ProjectedField)?
            .spelling()
            .to_owned();
        Ok(super::super::types::measure_named(&name))
    }

    /// Whether a written `psuffix` run reaches a [TYPE-9] `Box`'s content.
    ///
    /// A `Box`'s one member `inner` *is* that content, so the place below it
    /// is the dereference the content already is and not a field selection.
    /// The ordinary field walk has no step for that and the explicit-place
    /// walker does, so this decides which of the two resolves the place. A
    /// path this walk cannot follow reaches no box content: the ordinary walk
    /// then reports its own rejection at the suffix that failed, which is the
    /// diagnostic the writer needs.
    pub(in crate::semantic::check) fn place_path_reaches_box_content(
        &self,
        suffixes: &[NodeId],
        mut ty: CheckedType,
    ) -> Result<bool, CheckStop> {
        for (position, &suffix) in suffixes.iter().enumerate() {
            if self.subscript_offset(suffix)?.is_some() {
                return Ok(false);
            }
            if let CheckedType::Nominal(nominal) = ty
                && matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Box { .. })
            {
                return Ok(true);
            }
            let Ok((_, selected)) =
                self.resolve_struct_path(&suffixes[position..=position], ty)
            else {
                return Ok(false);
            };
            ty = selected;
        }
        Ok(false)
    }

    /// [TYPE-10] the eight measure and window-part spellings are names, not
    /// declarations, and occupy no field domain.
    pub(in crate::semantic::check) fn reject_reserved_pseudo_field(
        &self,
        suffix: NodeId,
        name: &str,
    ) -> Result<(), CheckStop> {
        if super::super::types::measure_named(name).is_none()
            && super::super::types::window_part_named(name).is_none()
        {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Type10,
            suffix,
            SemanticIssueKind::ReservedPseudoField {
                spelling: name.to_owned(),
                mechanical_fix: "use the operation that moves the window boundary [OP-10]",
            },
        )
    }
}
