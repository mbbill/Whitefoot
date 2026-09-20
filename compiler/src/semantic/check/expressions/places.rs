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
use super::super::references::{OWN1_ROOTED_CONSUME, WIN3_NO_TAKE};

/// [TYPE-9] the restructuring a `move` of a runtime-capacity content names.
const TYPE9_NO_CONTENT_MOVE: &str =
    "let the Box release it at scope exit, or empty it and call free_empty(move b) [OP-14]";
use super::super::{CheckStop, Checker, EffectSet, LocalBinding, PlaceAccess, TypedExpression};
use super::{PlaceUseContext, PlaceUseOptions, ResolvedPlaceSet};

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
    pub(super) resolved: ResolvedPlaceSet,
    /// [OP-15, MSR-1] the measure a trailing `.len`, `.cap` or
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
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let place = self.resolve_explicit_place(use_node, node, bindings)?;
        for member in &place.resolved.members {
            self.check_commit_place_live(member, use_node, false)?;
        }
        // [OP-15, MSR-1] a measure is a read-only `own u64` member of the
        // measured place: it reads that place's descriptor storage and
        // nothing below it, so it is neither a copy nor an affine use of the
        // value itself.
        if let Some(measure) = place.measure {
            let mut effects = EffectSet::NONE;
            for member in &place.resolved.members {
                for path in self.effect_paths_for_descriptor(use_node, member, bindings, measure)? {
                    effects.add_read(path);
                }
            }
            let (binding, path) = self.explicit_container_path(&place.expression, node)?;
            // [REF-4, MSR-1] a range reference's one measure is its element
            // count, and the row it belongs to is fixed by the written base
            // rather than by the type the dereference selects [TYPE-7].
            if place.range_referent {
                if !path.is_empty() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let element = self.intern_element(place.ty)?;
                return Ok(TypedExpression {
                    expression: CheckedExpression::RangeMeasure {
                        measure,
                        root: super::super::super::model::CheckedRangeRoot {
                            binding,
                            element,
                            element_type: place.ty,
                        },
                    },
                    mode: CheckedMode::Own,
                    reference: None,
                    reference_value: false,
                    effects,
                    accesses: place
                        .resolved
                        .members
                        .into_iter()
                        .map(|place| PlaceAccess {
                            place,
                            selected: true,
                        })
                        .collect(),
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
                accesses: place
                    .resolved
                    .members
                    .into_iter()
                    .map(|place| PlaceAccess {
                        place,
                        selected: true,
                    })
                    .collect(),
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
        let read_out =
            !copy && options.explicit_move && self.take_commit_read_out(&place.resolved.identity);
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
                // [TYPE-9, WIN-3] `let n = move b.inner;` consumes the
                // `Box`, yields its content, and frees the cell. It is the
                // ordinary [WIN-3] consume of a field out of its owner: the
                // owner ceases to exist at this use, which for a cell leaves
                // no other part to release.
                if let CheckedExpression::BoxDeref {
                    nominal,
                    referent,
                    value,
                    ..
                } = &place.expression
                    && place.resolved.identity.path.as_slice() == [PlaceStep::Deref]
                {
                    return self.check_box_unbox(
                        use_node,
                        place.declaration,
                        *nominal,
                        *referent,
                        value.as_ref().clone(),
                        bindings,
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
        for member in &place.resolved.members {
            for path in self.effect_paths_for_place(use_node, member, bindings)? {
                effects.add_read(path);
            }
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
            accesses: place
                .resolved
                .members
                .into_iter()
                .map(|place| PlaceAccess {
                    place,
                    selected: true,
                })
                .collect(),
        })
    }

    /// [TYPE-9, WIN-3] `let n = move b.inner;`.
    ///
    /// [TYPE-9]: "`let n = move b.inner;` consumes the `Box`, yields its
    /// content, and frees the cell [WIN-3]." [WIN-3] makes the move out of a
    /// field a consume of the whole owner, so the cell binding dies here and
    /// the content is this expression's value. A cell has exactly one field,
    /// so no other part of the owner survives to take a derived release.
    ///
    /// [TYPE-9] refuses the same spelling at a runtime-capacity content: its
    /// block is the cell's heap object, and taking the window out of the cell
    /// would leave a `Slots<T>` value in a position the rule admits nowhere.
    #[allow(clippy::too_many_arguments)]
    fn check_box_unbox(
        &self,
        use_node: NodeId,
        declaration: DeclarationId,
        nominal: super::super::super::model::NominalId,
        referent: CheckedType,
        value: CheckedExpression,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) -> Result<TypedExpression, CheckStop> {
        if matches!(
            referent,
            CheckedType::Buffer { .. } | CheckedType::Window { capacity: None, .. }
        ) {
            return self.issue_node(
                SemanticRule::Type9,
                use_node,
                SemanticIssueKind::InlineRuntimeCapacityShape {
                    spelling: self.checked_type_name(referent)?,
                    mechanical_fix: TYPE9_NO_CONTENT_MOVE,
                },
            );
        }
        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        // [REF-2] a consume is one of the three invalidating actions: every
        // reference into the cell names storage this move has carried away.
        Self::invalidate_references(
            bindings,
            &ResolvedPlace::binding(local.binding),
            &super::super::references::InvalidationEvent::PrefixMoved,
        );
        bindings
            .get_mut(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .live = false;
        Ok(TypedExpression {
            expression: CheckedExpression::BoxTake {
                carrier: self.tree.path(use_node)?.clone(),
                nominal,
                referent,
                value: Box::new(value),
            },
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses: vec![PlaceAccess {
                place: ResolvedPlace::binding(local.binding),
                selected: true,
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
                resolved: ResolvedPlaceSet::one(ResolvedPlace::binding(local.binding)),
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
                    && super::super::expressions::flat_storage::measured_kind_of(place.ty).is_none()
                    && super::super::expressions::flat_storage::measured_kind_of(referent).is_some()
                {
                    place.expression = CheckedExpression::BoxDeref {
                        carrier: self.tree.path(carrier)?.clone(),
                        nominal,
                        referent,
                        value: Box::new(place.expression),
                    };
                    place.ty = referent;
                    place.resolved.append_step(PlaceStep::Deref);
                }
                let measured = if place.range_referent {
                    Some(super::super::super::model::MeasuredKind::Range)
                } else {
                    super::super::expressions::flat_storage::measured_kind_of(place.ty)
                };
                // x1 [TYPE-10]: the measure spellings reserve nothing, so
                // `len` selects a measure only where the type of the place it
                // follows has a row for it. Everywhere else the suffix falls
                // through to the ordinary field walk below, which is how a
                // source struct declares its own field named `len`.
                if let Some(measured) = measured {
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
            }
            self.reject_window_part(suffix, &name, place.ty, place.range_referent)?;
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
                place.resolved.append_step(PlaceStep::Deref);
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
        if named.is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        // [REF-1, OP-12] one resolved member is an exact static place even
        // when two different reference holders name it. A true union has no
        // selected member until run time, so its exact identity remains the
        // holder and no overlapping candidate is chosen on its behalf.
        let identity = match named.as_slice() {
            [unique] => unique.clone(),
            _ => ResolvedPlace::binding(binding),
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
        inner.resolved = ResolvedPlaceSet {
            identity,
            members: named,
        };
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
            let Ok((_, selected)) = self.resolve_struct_path(&suffixes[position..=position], ty)
            else {
                return Ok(false);
            };
            ty = selected;
        }
        Ok(false)
    }

    /// [CONST-2, OWN-11, TYPE-2] a callee's declared write requires writable
    /// actual storage. A reference cannot grant write access to a named const,
    /// a compiler-updated counted binder, or a readonly field.
    ///
    /// The checked reference carries the resolved origin path through local
    /// aliases, reborrows and reference-root projections [REF-1]. Walking
    /// that path, rather than the immediate argument syntax, therefore keeps
    /// the readonly provenance that the reference spelling itself no longer
    /// exposes.
    pub(in crate::semantic::check) fn check_written_reference_argument(
        &self,
        atom: NodeId,
        argument: &TypedExpression,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let Some(reference) = &argument.reference else {
            return Ok(());
        };
        for path in &reference.paths {
            let immutable = match path.root {
                PlaceRoot::Constant(id) => {
                    Some((SemanticRule::Const2, self.constant(id)?.name.clone()))
                }
                PlaceRoot::Binding(binding) => bindings
                    .values()
                    .find(|local| local.binding == binding && local.compiler_updated)
                    .map(|local| {
                        self.declaration_spelling(local.declaration)
                            .map(|name| (SemanticRule::Own11, name))
                    })
                    .transpose()?,
            };
            if let Some((rule, binding)) = immutable {
                return self.issue_node(
                    rule,
                    atom,
                    SemanticIssueKind::ImmutableWrittenArgument {
                        binding,
                        mechanical_fix: "copy the value into a local binding and pass a reference \
                                         to that writable copy",
                    },
                );
            }
            self.reject_readonly_resolved_write(atom, path, bindings)?;
        }
        Ok(())
    }

    /// [TYPE-2] applies the readonly provenance carried by one resolved path
    /// to either a direct write through a reference or a written-reference
    /// call argument.
    pub(in crate::semantic::check) fn reject_readonly_resolved_write(
        &self,
        target: NodeId,
        path: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let Some(spelling) = self.readonly_member_on_resolved_path(path, bindings)? else {
            return Ok(());
        };
        self.issue_node(
            SemanticRule::Type2,
            target,
            SemanticIssueKind::ReadonlyWriteTarget {
                spelling,
                mechanical_fix: Self::READONLY_WRITE_TARGET_FIX,
            },
        )
    }

    /// The first readonly declaration member crossed by one resolved origin
    /// path. Every reference alias retains this path, while its referent type
    /// alone cannot say which declaration field supplied it.
    fn readonly_member_on_resolved_path(
        &self,
        path: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Option<String>, CheckStop> {
        #[derive(Clone, Copy)]
        enum PathType {
            Value(CheckedType),
            Range(CheckedType),
        }

        let mut ty = match path.root {
            PlaceRoot::Binding(binding) => {
                let local = bindings
                    .values()
                    .find(|local| local.binding == binding)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if local.mode == CheckedMode::Range {
                    PathType::Range(local.ty)
                } else {
                    PathType::Value(local.ty)
                }
            }
            PlaceRoot::Constant(constant) => {
                PathType::Value(self.constant(constant)?.declared_type)
            }
        };
        for step in &path.path {
            match *step {
                PlaceStep::Field(index) => {
                    let PathType::Value(current) = ty else {
                        return Ok(None);
                    };
                    let CheckedType::Nominal(nominal) = current else {
                        return Ok(None);
                    };
                    let CheckedNominalKind::Struct { fields } = &self.nominal(nominal)?.kind else {
                        return Ok(None);
                    };
                    let field = fields
                        .get(index as usize)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if field.readonly {
                        return Ok(Some(field.name.clone()));
                    }
                    ty = PathType::Value(field.ty);
                }
                PlaceStep::Deref => {
                    let PathType::Value(current) = ty else {
                        return Ok(None);
                    };
                    let CheckedType::Nominal(nominal) = current else {
                        return Ok(None);
                    };
                    let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind
                    else {
                        return Ok(None);
                    };
                    ty = PathType::Value(referent);
                }
                PlaceStep::Payload { variant, field } => {
                    let PathType::Value(current) = ty else {
                        return Ok(None);
                    };
                    let CheckedType::Nominal(nominal) = current else {
                        return Ok(None);
                    };
                    let CheckedNominalKind::Enum { variants } = &self.nominal(nominal)?.kind else {
                        return Ok(None);
                    };
                    let field = variants
                        .get(variant as usize)
                        .and_then(|variant| variant.fields.get(field as usize))
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if field.readonly {
                        return Ok(Some(field.name.clone()));
                    }
                    ty = PathType::Value(field.ty);
                }
                PlaceStep::Index(_) => {
                    ty = PathType::Value(match ty {
                        PathType::Range(element) => element,
                        PathType::Value(current) => match current {
                            CheckedType::Array { element, .. }
                            | CheckedType::Window { element, .. } => self.element_type(element)?,
                            CheckedType::Buffer { element } => element.ty(),
                            _ => return Ok(None),
                        },
                    });
                }
                PlaceStep::Range(_) => {
                    ty = PathType::Range(match ty {
                        PathType::Range(element) => element,
                        PathType::Value(current) => match current {
                            CheckedType::Array { element, .. }
                            | CheckedType::Window { element, .. } => self.element_type(element)?,
                            CheckedType::Buffer { element } => element.ty(),
                            _ => return Ok(None),
                        },
                    });
                }
                PlaceStep::Measure(measure) => return Ok(Some(measure.spelling().to_owned())),
                PlaceStep::Part(_) => return Ok(None),
            }
        }
        Ok(None)
    }

    /// [TYPE-10] a window part is effect-row vocabulary and never a place.
    ///
    /// x1 makes the four spellings reserve nothing: they are "selected by the
    /// window type of the place they follow", so `node.next` on a source
    /// struct is that struct's declared field and only `r.next` on a measured
    /// place is the refusal. The measure spellings left this rule with the
    /// reservation: they are the readonly fields [PRE-1] declares, refused as
    /// write targets by [TYPE-2] and read as ordinary fields everywhere else.
    pub(in crate::semantic::check) fn reject_window_part(
        &self,
        suffix: NodeId,
        name: &str,
        followed: CheckedType,
        range_referent: bool,
    ) -> Result<(), CheckStop> {
        if super::super::types::window_part_named(name).is_none() {
            return Ok(());
        }
        if !range_referent
            && super::super::expressions::flat_storage::measured_kind_of(followed).is_none()
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

    /// The restructuring [TYPE-2] states for a readonly write target.
    pub(in crate::semantic::check) const READONLY_WRITE_TARGET_FIX: &'static str =
        "use the operation that changes it, or replace the whole value";

    /// [TYPE-2, TYPE-10] refuses a write target whose path names a readonly
    /// field or a window part of the place it follows.
    ///
    /// The walk is type-directed because x1 reserves neither vocabulary from a
    /// declaration: a suffix names a measure or a part only where the type of
    /// the place it follows has one, and it is an ordinary field everywhere
    /// else. A path whose step this walk cannot follow is left to the
    /// ordinary resolver, which reports the step that failed.
    pub(in crate::semantic::check) fn reject_reserved_write_members(
        &self,
        target: NodeId,
        suffixes: &[NodeId],
        mut ty: CheckedType,
    ) -> Result<(), CheckStop> {
        for &suffix in suffixes {
            if self.subscript_offset(suffix)?.is_some() {
                // [MSR-2] a write at an element position reaches that
                // element's own storage; the offset carries no member name.
                ty = match ty {
                    CheckedType::Array { element, .. } | CheckedType::Window { element, .. } => {
                        self.element_type(element)?
                    }
                    CheckedType::Buffer { element } => element.ty(),
                    _ => return Ok(()),
                };
                continue;
            }
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling()
                .to_owned();
            self.reject_window_part(suffix, &name, ty, false)?;
            // A measure of a measured place is a readonly field of that
            // shape's [PRE-1] declaration, so a `set` on it is [TYPE-2]'s
            // refusal and never a struct-field rejection.
            if super::super::types::measure_named(&name).is_some()
                && super::super::expressions::flat_storage::measured_kind_of(ty).is_some()
            {
                return self.issue_node(
                    SemanticRule::Type2,
                    target,
                    SemanticIssueKind::ReadonlyWriteTarget {
                        spelling: name,
                        mechanical_fix: Self::READONLY_WRITE_TARGET_FIX,
                    },
                );
            }
            let CheckedType::Nominal(nominal) = ty else {
                return Ok(());
            };
            let kind = &self.nominal(nominal)?.kind;
            if let CheckedNominalKind::Box { referent, .. } = kind {
                // [TYPE-9] `b.inner` is the cell's content, a dereference
                // step rather than a field of the cell.
                ty = *referent;
                continue;
            }
            let CheckedNominalKind::Struct { fields } = kind else {
                return Ok(());
            };
            let Some(field) = fields.iter().find(|field| field.name == name) else {
                return Ok(());
            };
            if field.readonly {
                return self.issue_node(
                    SemanticRule::Type2,
                    target,
                    SemanticIssueKind::ReadonlyWriteTarget {
                        spelling: name,
                        mechanical_fix: Self::READONLY_WRITE_TARGET_FIX,
                    },
                );
            }
            ty = field.ty;
        }
        Ok(())
    }
}
