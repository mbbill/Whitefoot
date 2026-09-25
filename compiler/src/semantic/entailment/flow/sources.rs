//! The [ENT-3] fact sources: which checked shape establishes which relation,
//! at which point of the flow.
//!
//! Each source below is one clause of [ENT-3], recognized on the checked tree
//! and expressed in the normalized difference-bound vocabulary [ENT-2]. The
//! walk in the parent module owns where a source fires; this module owns what
//! it establishes. Sources are additive and never reject: a shape outside a
//! source's stated form contributes nothing, which only under-derives, the
//! version-monotone direction [ENT-1].
//!
//! Implemented here: S1 (through the parent's arm entry), S4, S5, S6, S7,
//! and S9. Retired labels are not reused [ENT-3].

use super::super::super::goal::CheckedRequirement;
use super::super::super::model::{
    BindingId, CheckedArrayRoot, CheckedConst, CheckedConversionMode, CheckedEnumType,
    CheckedExpression, CheckedIntegerOperation, CheckedMeasure, CheckedNominalKind,
    CheckedNumericType, CheckedPlaceStep, CheckedSetTarget, CheckedType, CheckedValue, IntegerType,
    MeasuredKind, NominalId,
};
use super::super::super::places::CapturedTerm;
use super::super::fragment_type;
use super::super::state::{
    DerivationId, DerivationNode, FactState, FlowEventId, FlowEventKind, Relation, close,
};
use super::super::term::{
    CountedCaptureSide, MeasurePlacement, PlaceRoot, PlaceStep, ResolvedPlace, TermId, TermKind,
    ZERO, integer_value,
};
use super::super::{
    CountedAtomicDerivation, CountedBoundDerivation, CountedDerivationSet,
    CountedEqualityDerivation, CountedProofPoint,
};
use super::operation_facts::{self, Interval, Row, Span};
use super::{Analyzer, ArmFacts, ProofFlowState};
use std::rc::Rc;
/// Which term one evaluated value's [ENT-3] image is established on: the
/// place a `let` binder introduces, the compiler-owned commit value of one
/// `set` occurrence, or a checked integer conversion's private success
/// payload [ENT-2, ENT-5].
/// These destinations use the same admitted source image; a conditional
/// payload interprets it only inside its own success context.
#[derive(Clone, Copy)]
pub(super) enum ValueImage<'a> {
    Binding(BindingId),
    Commit(&'a crate::NodePath),
    ResultPayload(TermId),
}

/// The three S11 terms installed for one counted range.
pub(super) struct CountedTerms {
    pub(super) lower_source: TermId,
    pub(super) lower: TermId,
    pub(super) binder: TermId,
    pub(super) upper: TermId,
    pub(super) upper_source: TermId,
}

/// The affine half of one admitted unsigned exact-division transfer.
/// The S7 fact bounding the quotient by its dividend supplies the retained
/// source proof; the parent flow binds the scaled quotient image to the exact
/// current operand values.
pub(super) struct EstablishedUnsignedDivision {
    pub(super) literal_divisor: Option<i128>,
    pub(super) parent: DerivationId,
}

/// One right-hand side read as a row of the [ENT-3.S7] table: its row, its
/// selected and result types, and its operands in order.
struct OperationShape<'a> {
    row: Row,
    operand: IntegerType,
    result: IntegerType,
    operands: &'a [CheckedExpression],
    carrier: &'a crate::NodePath,
}

/// The three preheader relations after their complete S11 snapshot has been
/// materialized, but before continuing kills are applied.
pub(super) struct CountedPreheader {
    terms: CountedTerms,
    lower_capture_eq_endpoint: CountedEqualityDerivation,
    upper_capture_eq_endpoint: CountedEqualityDerivation,
    binder_eq_lower_capture: CountedEqualityDerivation,
}

impl Analyzer<'_, '_> {
    // ------------------------------------------------------------------
    // S11 counted-range structural facts
    // ------------------------------------------------------------------

    /// Establishes the once-only endpoint snapshots and binder
    /// initialization, before the caller materializes [ENT-4] closure and
    /// applies the counted continuing-kill summary.
    pub(super) fn establish_counted_preheader(
        &mut self,
        range_path: &[u32],
        binder: BindingId,
        lower: &CheckedExpression,
        upper: &CheckedExpression,
        state: &mut FactState,
        event: FlowEventId,
    ) -> CountedTerms {
        // [FN-1] each capture copies its endpoint's value, so an admitted
        // endpoint reads exactly as an ordinary `let` of the same atom: a
        // measure term through its [MSR-1] former, otherwise the tracked
        // place or constant [ENT-2].
        let lower_source = self
            .copy_source(lower)
            .expect("checked counted lower endpoint must be an ENT-2 term or constant");
        let upper_source = self
            .copy_source(upper)
            .expect("checked counted upper endpoint must be an ENT-2 term or constant");
        let lower_capture = self.terms.intern(TermKind::CountedCapture {
            range_path: range_path.to_vec(),
            side: CountedCaptureSide::Lower,
        });
        let upper_capture = self.terms.intern(TermKind::CountedCapture {
            range_path: range_path.to_vec(),
            side: CountedCaptureSide::Upper,
        });
        let binder = self.terms.intern(TermKind::Place(
            ResolvedPlace::spelled(PlaceRoot::Binding(binder), false, Vec::new()),
            IntegerType::U64,
        ));
        state.establish(
            &Relation::Equal {
                left: lower_capture,
                right: lower_source,
                difference: 0,
            },
            &mut self.derivations,
            event,
        );
        state.establish(
            &Relation::Equal {
                left: upper_capture,
                right: upper_source,
                difference: 0,
            },
            &mut self.derivations,
            event,
        );
        state.establish(
            &Relation::Equal {
                left: binder,
                right: lower_capture,
                difference: 0,
            },
            &mut self.derivations,
            event,
        );
        CountedTerms {
            lower_source,
            lower: lower_capture,
            binder,
            upper: upper_capture,
            upper_source,
        }
    }

    /// Captures the three once-only S11 equality roots from the already
    /// materialized post-capture state. This does not close or walk again.
    pub(super) fn capture_counted_preheader(
        &self,
        terms: CountedTerms,
        state: &FactState,
    ) -> CountedPreheader {
        let equality = |left: TermId, right: TermId| {
            let forward = Relation::Bound {
                left,
                right,
                bound: 0,
            };
            let reverse = Relation::Bound {
                left: right,
                right: left,
                bound: 0,
            };
            CountedEqualityDerivation {
                relation: Relation::Equal {
                    left,
                    right,
                    difference: 0,
                },
                forward: CountedAtomicDerivation {
                    relation: forward,
                    proof_point: CountedProofPoint::PreheaderSnapshot,
                    parent: state
                        .bound_parent(left, right, 0)
                        .expect("materialized S11 equality must retain its forward parent"),
                },
                reverse: CountedAtomicDerivation {
                    relation: reverse,
                    proof_point: CountedProofPoint::PreheaderSnapshot,
                    parent: state
                        .bound_parent(right, left, 0)
                        .expect("materialized S11 equality must retain its reverse parent"),
                },
            }
        };
        CountedPreheader {
            lower_capture_eq_endpoint: equality(terms.lower, terms.lower_source),
            upper_capture_eq_endpoint: equality(terms.upper, terms.upper_source),
            binder_eq_lower_capture: equality(terms.binder, terms.lower),
            terms,
        }
    }

    /// Adds exactly S11's two facts on an executed true header edge.
    pub(super) fn establish_counted_body_entry(
        &mut self,
        node_path: &crate::NodePath,
        counted: CountedPreheader,
        state: &mut FactState,
        event: FlowEventId,
    ) -> CountedDerivationSet {
        let lower_relation = Relation::Bound {
            left: counted.terms.lower,
            right: counted.terms.binder,
            bound: 0,
        };
        let lower_parent = state.establish_bound_with_proof(
            counted.terms.lower,
            counted.terms.binder,
            0,
            &mut self.derivations,
            event,
        );
        let upper_relation = Relation::Bound {
            left: counted.terms.binder,
            right: counted.terms.upper,
            bound: -1,
        };
        let upper_parent = state.establish_bound_with_proof(
            counted.terms.binder,
            counted.terms.upper,
            -1,
            &mut self.derivations,
            event,
        );
        CountedDerivationSet {
            counted_node_path: node_path.clone(),
            lower_capture_eq_endpoint: counted.lower_capture_eq_endpoint,
            upper_capture_eq_endpoint: counted.upper_capture_eq_endpoint,
            binder_eq_lower_capture: counted.binder_eq_lower_capture,
            lower_capture_le_binder: CountedBoundDerivation {
                relation: lower_relation.clone(),
                atomic: CountedAtomicDerivation {
                    relation: lower_relation,
                    proof_point: CountedProofPoint::BodyEntry,
                    parent: lower_parent,
                },
            },
            binder_lt_upper_capture: CountedBoundDerivation {
                relation: upper_relation.clone(),
                atomic: CountedAtomicDerivation {
                    relation: upper_relation,
                    proof_point: CountedProofPoint::BodyEntry,
                    parent: upper_parent,
                },
            },
        }
    }

    // ------------------------------------------------------------------
    // S4 requires facts
    // ------------------------------------------------------------------

    /// [ENT-3] S4: the complete concrete body goal enters as a positive opaque
    /// fact, with its one exact comparison-root projection when present.
    pub(super) fn establish_requires_facts(
        &mut self,
        requirement: &CheckedRequirement,
        state: &mut FactState,
        event: FlowEventId,
    ) {
        let Some(goal) = self.body_requirement_goal(requirement) else {
            return;
        };
        let goal = self.intern_goal_expression(goal);
        state.establish_goal(
            goal,
            super::super::state::GoalSign::Positive,
            &mut self.derivations,
            event,
        );
        if let Some(relation) = self.goals.projection(goal).cloned() {
            state.establish(&relation, &mut self.derivations, event);
        }
        // [ENT-3] Signed Boolean decomposition of the established body goal.
        self.establish_boolean_decomposition(
            goal,
            super::super::state::GoalSign::Positive,
            state,
            event,
        );
        self.record_boolean_decomposition(goal, super::super::state::GoalSign::Positive, state);
    }

    // ------------------------------------------------------------------
    // Establishment of one evaluated value's image: S5, S6, S7, S9
    // ------------------------------------------------------------------

    /// Every source that establishes at an `ordinary_let_rhs` binding, in one
    /// place because they are mutually exclusive on the initializer's shape.
    /// A [SET-1] commit evaluates its right-hand side under exactly these
    /// rules before its target kill, naming the value by the occurrence's own
    /// commit-value term instead of a binder [ENT-2, ENT-3.S5].
    pub(super) fn establish_value_image(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> Option<EstablishedUnsignedDivision> {
        if self.establish_length_facts(node_path, destination, value, state, event) {
            return None;
        }
        if self.establish_element_range(node_path, destination, value, state, event) {
            return None;
        }
        if let Some(shape) = Self::operation_shape(value) {
            return self.establish_operation_facts(
                node_path,
                destination,
                value,
                &shape,
                state,
                event,
            );
        }
        self.establish_copy_fact(node_path, destination, value, state, event);
        None
    }

    fn binding_event(
        &mut self,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
        kind: FlowEventKind,
        node_path: &crate::NodePath,
    ) -> FlowEventId {
        if let Some((existing_kind, id)) = event {
            debug_assert_eq!(*existing_kind, kind);
            return *id;
        }
        let id = self.proof_event(kind, Some(node_path));
        *event = Some((kind, id));
        id
    }

    /// The exact term named by one writable fragment place [ENT-2].
    fn writable_place_term(
        &mut self,
        binding: BindingId,
        fields: &[u32],
        ty: CheckedType,
    ) -> Option<TermId> {
        let fragment = fragment_type(ty)?;
        // [REF-1] a reference root is resolved to the path it names rather
        // than spelled with a synthesized `deref`, so the term carries the
        // written path and nothing else.
        let kind = TermKind::Place(
            ResolvedPlace::spelled(PlaceRoot::Binding(binding), false, fields.to_vec()),
            fragment,
        );
        Some(self.terms.intern(kind))
    }

    /// [ENT-3.S5] admits direct scalar places, including fields and Box
    /// content reached through a reference. A subscript anywhere in the
    /// path excludes a commit image, even if its offset is a literal.
    pub(super) fn commit_target_term(&mut self, target: &CheckedSetTarget) -> Option<TermId> {
        match target {
            CheckedSetTarget::Place(target) => {
                self.writable_place_term(target.binding, &target.fields, target.ty)
            }
            CheckedSetTarget::Storage(target)
                if !target
                    .path
                    .iter()
                    .any(|step| matches!(step, CheckedPlaceStep::Subscript(_))) =>
            {
                let fragment = fragment_type(target.ty)?;
                let place = self.container_root_path(target);
                Some(self.terms.intern(TermKind::Place(place, fragment)))
            }
            _ => None,
        }
    }

    /// The term one evaluated value's image is established on: a freshly
    /// bound integer place, a commit value, or a private success payload.
    fn bound_term(
        &mut self,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
    ) -> Option<TermId> {
        match destination {
            ValueImage::Binding(binding) => self.writable_place_term(binding, &[], value.ty()),
            ValueImage::Commit(node_path) => self.commit_value_term(node_path, value),
            ValueImage::ResultPayload(payload) => Some(payload),
        }
    }

    /// The commit-value term of one `set` occurrence, interned on first use.
    /// Its identity is the statement's NodePath and the value's fragment
    /// type, so every source establishing at that one occurrence names one
    /// term [ENT-2].
    fn commit_value_term(
        &mut self,
        node_path: &crate::NodePath,
        value: &CheckedExpression,
    ) -> Option<TermId> {
        let kind = Self::commit_value_kind(node_path, value)?;
        Some(self.terms.intern(kind))
    }

    /// The same term when a source above already formed it, and nothing when
    /// the right-hand side matched no source: an image-free value needs no
    /// term and no post-write equality [ENT-3.S5].
    pub(super) fn interned_commit_value_term(
        &self,
        node_path: &crate::NodePath,
        value: &CheckedExpression,
    ) -> Option<TermId> {
        self.terms
            .interned(&Self::commit_value_kind(node_path, value)?)
    }

    fn commit_value_kind(
        node_path: &crate::NodePath,
        value: &CheckedExpression,
    ) -> Option<TermKind> {
        let ty = fragment_type(value.ty())?;
        Some(TermKind::CommitValue {
            commit_path: node_path.components().to_vec(),
            ty,
        })
    }

    /// The value image shared by an ordinary let, a direct-place SET-1
    /// commit and a counted endpoint capture [FN-1]. Every admitted exact
    /// integer conversion preserves its input's mathematical value; checked
    /// and defined rows have another result type.
    fn copy_source(&mut self, value: &CheckedExpression) -> Option<TermId> {
        match value {
            CheckedExpression::NumericConversion {
                mode: CheckedConversionMode::Exact,
                source: CheckedNumericType::Integer(_),
                destination: CheckedNumericType::Integer(_),
                value: operand,
                ..
            } => self.copy_source(operand),
            _ => self.copy_operand(value),
        }
    }

    /// One atom read as an admitted [ENT-2] term or constant: a measure term
    /// through its [MSR-1] former, otherwise a tracked place or constant. This
    /// is the one complete reader every value image and S7 operand uses.
    fn copy_operand(&mut self, value: &CheckedExpression) -> Option<TermId> {
        self.measure_operand(value)
            .or_else(|| self.read_operand(value))
    }

    fn establish_copy_equality(
        &mut self,
        node_path: &crate::NodePath,
        destination: TermId,
        source: TermId,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) {
        let event = self.binding_event(event, FlowEventKind::S5, node_path);
        state.establish(
            &Relation::Equal {
                left: destination,
                right: source,
                difference: 0,
            },
            &mut self.derivations,
            event,
        );
    }

    /// The place term a binding names directly, for length facts over an
    /// allocated or borrowed collection.
    pub(super) fn bound_place(&self, binding: BindingId) -> ResolvedPlace {
        ResolvedPlace::binding(binding)
    }

    /// [ENT-3] S5: `let x: T = lit;` establishes x = value(lit);
    /// `let x: T = p;` with p a term establishes x = p; and
    /// `let y: Dst = cvt::<Src, Dst>(p);` after its [OP-6] domain proof
    /// establishes y = p, the conversion being exactly value-preserving.
    /// [MSR-3] the rebind placement, first half: at the pre-transfer point of
    /// one `let` or one [LIV-2] `set` whose right-hand side is a measured
    /// place, one immutable datum per [MSR-1] measure, established equal to
    /// that measure.
    ///
    /// The datum contains no place, so the consume the same statement
    /// performs cannot kill it. `let built = move spare;` is what needs it:
    /// without the datum a fresh name for a measured value starts with no
    /// measures at all, because the equality to the source dies with the
    /// source.
    pub(super) fn mint_rebind_datums(
        &mut self,
        node_path: &crate::NodePath,
        ordinal: u32,
        value: &CheckedExpression,
        state: &mut ProofFlowState,
    ) -> Option<MeasureCarry> {
        let source = self.placement_source_place(value)?;
        self.mint_measure_datums(
            node_path,
            ordinal,
            MeasurePlacement::Rebind,
            source,
            value.ty(),
            state,
        )
    }

    /// A placement reads the exact checked source place, including a moved
    /// field or Box content. Computed values and calls have their own fact
    /// sources and are not placements [MSR-3].
    pub(super) fn placement_source_place(
        &self,
        value: &CheckedExpression,
    ) -> Option<ResolvedPlace> {
        let source = match value {
            CheckedExpression::Project {
                binding, fields, ..
            } => Some(ResolvedPlace {
                root: PlaceRoot::Binding(*binding),
                path: fields.iter().copied().map(PlaceStep::Field).collect(),
            }),
            CheckedExpression::BoxTake { binding, path, .. } => Some(ResolvedPlace {
                root: PlaceRoot::Binding(*binding),
                path: path.iter().map(CheckedPlaceStep::place_step).collect(),
            }),
            _ => self.read_place_path(value),
        }?;
        // A source subscript must name an MSR-1 offset. A computed offset
        // and a possible-descendant cover have no exact placement identity.
        (!source.path.iter().any(|step| {
            matches!(step, PlaceStep::Descendant(_))
                || matches!(step, PlaceStep::Index(offset) if offset.term == CapturedTerm::Opaque)
        }))
        .then_some(source)
    }

    /// [MSR-3] the first half of every placement below the entry and the
    /// call: at the point before the statement's own kills, one immutable
    /// datum per [MSR-1] measure of the source place, established equal to
    /// that measure.
    ///
    /// The datum contains no place, so neither the consume the statement
    /// performs nor the write it commits can kill it. That is the whole
    /// content of a placement: without a term with empty support standing
    /// between the two names, a measured value would arrive at its new name
    /// with no measures at all, because the equality to the source dies with
    /// the source.
    ///
    /// `ordinal` separates the placements one statement carries — a
    /// construct's field, a destructuring's binder, a target list's target,
    /// the displaced and the stored halves of one `replace`.
    pub(super) fn mint_measure_datums(
        &mut self,
        node_path: &crate::NodePath,
        ordinal: u32,
        placement: MeasurePlacement,
        source: ResolvedPlace,
        ty: CheckedType,
        state: &mut ProofFlowState,
    ) -> Option<MeasureCarry> {
        let mut carried = Vec::new();
        for (path, measured_type) in self.measured_paths(&source, ty) {
            let Some(measured) = super::measured_kind(measured_type) else {
                continue;
            };
            let constant = super::type_constant(measured_type);
            let mut place = source.clone();
            place.path.extend(path.iter().copied());
            let event = self.proof_event(FlowEventKind::S5, Some(node_path));
            let mut datums = Vec::with_capacity(4);
            for measure in MEASURES {
                let live = self.place_measure_term(measure, place.clone(), measured, constant);
                let datum = self.terms.intern(TermKind::MeasureDatum {
                    statement: node_path.components().to_vec(),
                    ordinal,
                    path: path.clone(),
                    placement,
                    measure,
                });
                self.adopt_measure_atom(datum, live, &state.affine);
                state.facts.establish(
                    &Relation::Equal {
                        left: datum,
                        right: live,
                        difference: 0,
                    },
                    &mut self.derivations,
                    event,
                );
                datums.push(datum);
            }
            carried.push(CarriedMeasures {
                path,
                measured,
                constant,
                datums,
            });
        }
        (!carried.is_empty()).then_some(MeasureCarry { carried })
    }

    /// [MSR-1, MSR-3] measured descendants reached through owned fields,
    /// payloads and Box content. A structural walk covers acyclic type paths;
    /// already registered exact source terms supply deeper recursive paths.
    /// Both inventories are finite, without a depth limit on a written path.
    ///
    /// A term's presence does not establish its old facts: the datum is
    /// equated to the source in the current state, after all earlier kills.
    /// Unregistered descendants have no numeric facts beyond their standing
    /// type facts, which are available at the destination without transport.
    /// Payload paths stay variant-specific and add no refinement fact.
    fn measured_paths(
        &self,
        source: &ResolvedPlace,
        ty: CheckedType,
    ) -> Vec<(Vec<PlaceStep>, CheckedType)> {
        let mut found = Vec::new();
        if !self.collect_measured_paths(ty, &mut Vec::new(), &mut Vec::new(), &mut found) {
            return found;
        }
        let source = source.clone().term_identity();
        for term in self.terms.ids() {
            let TermKind::Measure(CheckedMeasure::Length, place) = self.terms.kind(term) else {
                continue;
            };
            if place.root != source.root {
                continue;
            }
            let Some(path) = place.path.strip_prefix(source.path.as_slice()) else {
                continue;
            };
            // No alias resolution or overlap test turns a possible target
            // into this source. The suffix must select concrete owned
            // fields; an unknown descendant cover is never such a step.
            let Some(selected) = self.measured_path_type(ty, path) else {
                continue;
            };
            if !found.iter().any(|(existing, _)| existing == path) {
                found.push((path.to_vec(), selected));
            }
        }
        found
    }

    /// Returns whether a nominal cycle left paths for the finite source-term
    /// inventory to supply. Acyclic operands need no registry scan.
    fn collect_measured_paths(
        &self,
        ty: CheckedType,
        path: &mut Vec<PlaceStep>,
        ancestors: &mut Vec<NominalId>,
        found: &mut Vec<(Vec<PlaceStep>, CheckedType)>,
    ) -> bool {
        if super::measured_kind(ty).is_some() {
            found.push((path.clone(), ty));
            return false;
        }
        let CheckedType::Nominal(nominal) = ty else {
            return false;
        };
        let Some(kind) = self
            .context
            .nominals
            .get(nominal.0 as usize)
            .map(|record| &record.kind)
        else {
            return false;
        };
        if ancestors.contains(&nominal) {
            return true;
        }
        ancestors.push(nominal);
        let mut recursive = false;
        match kind {
            CheckedNominalKind::Struct { fields } => {
                for (ordinal, field) in fields.iter().enumerate() {
                    let Ok(ordinal) = u32::try_from(ordinal) else {
                        continue;
                    };
                    path.push(PlaceStep::Field(ordinal));
                    recursive |= self.collect_measured_paths(field.ty, path, ancestors, found);
                    path.pop();
                }
            }
            CheckedNominalKind::Enum { variants } => {
                for variant in variants {
                    for (ordinal, field) in variant.fields.iter().enumerate() {
                        let Ok(field_ordinal) = u32::try_from(ordinal) else {
                            continue;
                        };
                        path.push(PlaceStep::Payload {
                            variant: variant.tag,
                            field: field_ordinal,
                        });
                        recursive |= self.collect_measured_paths(field.ty, path, ancestors, found);
                        path.pop();
                    }
                }
            }
            CheckedNominalKind::Box { referent, .. } => {
                path.push(PlaceStep::Deref);
                recursive |= self.collect_measured_paths(*referent, path, ancestors, found);
                path.pop();
            }
            CheckedNominalKind::Opaque => {}
        }
        ancestors.pop();
        recursive
    }

    /// Replays a finite owned projection against its actual operand type.
    /// Element and range storage are not aggregate ownership projections.
    fn measured_path_type(&self, mut ty: CheckedType, path: &[PlaceStep]) -> Option<CheckedType> {
        for step in path {
            let CheckedType::Nominal(nominal) = ty else {
                return None;
            };
            ty = match (&self.context.nominals.get(nominal.0 as usize)?.kind, step) {
                (CheckedNominalKind::Struct { fields }, PlaceStep::Field(field)) => {
                    fields.get(*field as usize)?.ty
                }
                (CheckedNominalKind::Box { referent, .. }, PlaceStep::Deref) => *referent,
                (CheckedNominalKind::Enum { variants }, PlaceStep::Payload { variant, field }) => {
                    variants
                        .iter()
                        .find(|candidate| candidate.tag == *variant)?
                        .fields
                        .get(*field as usize)?
                        .ty
                }
                _ => return None,
            };
        }
        super::measured_kind(ty).map(|_| ty)
    }

    /// [MSR-3] the rebind placement, second half: after the transfer, the
    /// destination binding's own measures equal the datums minted before it.
    pub(super) fn establish_rebind_datums(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
        rebind: &MeasureCarry,
        state: &mut FactState,
    ) {
        let target = self.bound_place(binding);
        self.establish_measure_datums(node_path, target, rebind, state);
    }

    /// [MSR-3] the second half of every placement: after the statement's own
    /// kills, the destination place's measures equal the datums minted before
    /// them.
    pub(super) fn establish_measure_datums(
        &mut self,
        node_path: &crate::NodePath,
        destination: ResolvedPlace,
        carry: &MeasureCarry,
        state: &mut FactState,
    ) {
        let event = self.proof_event(FlowEventKind::S5, Some(node_path));
        for carried in &carry.carried {
            let mut place = destination.clone();
            place.path.extend(carried.path.iter().copied());
            for (measure, datum) in MEASURES.into_iter().zip(&carried.datums) {
                let left = self.place_measure_term(
                    measure,
                    place.clone(),
                    carried.measured,
                    carried.constant,
                );
                state.establish(
                    &Relation::Equal {
                        left,
                        right: *datum,
                        difference: 0,
                    },
                    &mut self.derivations,
                    event,
                );
            }
        }
    }

    fn establish_copy_fact(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) {
        let Some(source) = self.copy_source(value) else {
            return;
        };
        let Some(bound) = self.bound_term(destination, value) else {
            return;
        };
        self.establish_copy_equality(node_path, bound, source, state, event);
    }

    /// [ENT-3] S5 at a SET-1 value commit. The caller has already evaluated
    /// the right-hand side to the `commit` term above and killed every fact
    /// about the old target value; this equality names that evaluated value,
    /// so an arithmetic or other computed right-hand side carries its own
    /// image across the write exactly as an intervening `let` would. Only a
    /// direct fragment place receives it; indexed storage establishes
    /// nothing, since one element write is no image of the whole collection.
    pub(super) fn establish_commit_copy_fact(
        &mut self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        commit: TermId,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) {
        let Some(destination) = self.commit_target_term(target) else {
            return;
        };
        self.establish_copy_equality(node_path, destination, commit, state, event);
    }

    /// [ENT-3] S6: `len::<T>(P)` for a tracked P establishes m = len_of(P); and
    /// `slice_of…(&'r P)` for a tracked P establishes len_of(s) = len_of(P).
    ///
    /// An `array<T, N>` allocation needs no clause here: its length equality
    /// is the [ENT-2] implicit fact carried by every length term over an
    /// array-typed place, registered wherever that term is interned.
    fn establish_length_facts(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        match self.measure_operand(value) {
            Some(source_length) => {
                if let Some(bound) = self.bound_term(destination, value) {
                    let event = self.binding_event(event, FlowEventKind::S6, node_path);
                    state.establish(
                        &Relation::Equal {
                            left: bound,
                            right: source_length,
                            difference: 0,
                        },
                        &mut self.derivations,
                        event,
                    );
                }
                true
            }
            None => false,
        }
    }

    /// The measure term one [MSR-1] measure former reads, over the same
    /// place the obligation judgment forms for P, so both name one term
    /// [ENT-2].
    pub(super) fn measure_operand(&mut self, value: &CheckedExpression) -> Option<TermId> {
        let (measure, place, measured, array_length) = match value {
            CheckedExpression::ArrayMeasure {
                measure,
                root,
                length,
            } => (
                *measure,
                self.array_root_place(root),
                MeasuredKind::ConstantArray,
                Some(*length),
            ),
            CheckedExpression::BufferMeasure { measure, root } => (
                *measure,
                ResolvedPlace::from_path(root.binding, root.place_path()),
                MeasuredKind::RuntimeArray,
                None,
            ),
            // [MSR-1, REF-4] a range reference's one measure, over the place
            // the reference names [REF-1].
            CheckedExpression::RangeMeasure { measure, root } => (
                *measure,
                ResolvedPlace::spelled(
                    PlaceRoot::Binding(root.binding),
                    self.is_holder(root.binding),
                    Vec::new(),
                ),
                MeasuredKind::Range,
                None,
            ),
            // [MSR-1] a storage shape's measure reader names the
            // same [ENT-2] term the clause and the invariant name, so a `let`
            // over one is the ordinary [ENT-3.S6] equality a buffer's is.
            CheckedExpression::ContainerMeasure { measure, root } => {
                return Some(self.place_measure_term(
                    *measure,
                    self.container_root_path(root),
                    root.measured()?,
                    root.type_constant(),
                ));
            }
            CheckedExpression::RangeElementMeasure { measure, place, .. } => {
                return Some(self.place_measure_term(
                    *measure,
                    ResolvedPlace::from_path(place.root.binding, place.place_path()),
                    place.measured()?,
                    place.type_constant(),
                ));
            }
            _ => return None,
        };
        Some(self.place_measure_term(measure, place, measured, array_length))
    }

    // ------------------------------------------------------------------
    // S7 operation facts
    // ------------------------------------------------------------------

    /// The [ENT-3.S7] row one right-hand side reads as: an integer-valued
    /// operation of the table or an integer `reinterpret`. Defined, checked
    /// and comparison rows produce no integer, and a type-parameter-typed
    /// operation has no fragment type, so neither is a row here.
    fn operation_shape(value: &CheckedExpression) -> Option<OperationShape<'_>> {
        match value {
            CheckedExpression::IntegerOperation {
                carrier,
                operation,
                operand_type: CheckedType::Integer(operand),
                arguments,
                result: CheckedType::Integer(result),
                ..
            } => Some(OperationShape {
                row: Row::of(*operation)?,
                operand: *operand,
                result: *result,
                operands: arguments,
                carrier,
            }),
            CheckedExpression::Reinterpret {
                carrier,
                source: CheckedNumericType::Integer(operand),
                destination: CheckedNumericType::Integer(result),
                value,
            } => Some(OperationShape {
                row: Row::Reinterpret,
                operand: *operand,
                result: *result,
                operands: std::slice::from_ref(value.as_ref()),
                carrier,
            }),
            _ => None,
        }
    }

    /// The exact row a checked row's success payload is read as [ENT-5].
    fn checked_operation_shape(value: &CheckedExpression) -> Option<OperationShape<'_>> {
        let CheckedExpression::IntegerOperation {
            carrier,
            operation,
            operand_type: CheckedType::Integer(operand),
            arguments,
            ..
        } = value
        else {
            return None;
        };
        Some(OperationShape {
            row: Row::of_checked(*operation)?,
            operand: *operand,
            result: *operand,
            operands: arguments,
            carrier,
        })
    }

    /// [ENT-5] a checked integer row's private success payload denotes the
    /// exact row's mathematical result and receives that row's [ENT-3.S7]
    /// facts, read in the ordinary closed state `facts` already holds.
    pub(super) fn establish_checked_payload(
        &mut self,
        statement: &crate::NodePath,
        payload: TermId,
        value: &CheckedExpression,
        facts: &mut FactState,
    ) {
        if let Some(shape) = Self::checked_operation_shape(value) {
            let _ = self.establish_operation_facts(
                statement,
                ValueImage::ResultPayload(payload),
                value,
                &shape,
                facts,
                &mut None,
            );
        }
    }

    /// The mathematical value of a constant term. Z is the interned form of
    /// the written constant zero, so it reads as one here.
    fn constant_term_value(&self, term: TermId) -> Option<i128> {
        match *self.terms.kind(term) {
            TermKind::Zero => Some(0),
            TermKind::Constant(value) => Some(value),
            _ => None,
        }
    }

    /// [ENT-3.S7] the result bounds and operand relations one table row
    /// establishes on the value it binds.
    ///
    /// Each operand reads one interval in the closed state where the
    /// right-hand side is evaluated: a literal or named-const value is its
    /// value, any other admitted term its strongest closed bounds through Z,
    /// and every other operand its type's interval. The closed bounds read are
    /// the parents of every fact the row establishes, so the retained
    /// derivation shows what each fact stood on [DIAG-2]. Operands are read
    /// with the one complete term reader, so a measure operand is a term here
    /// exactly as it is in a copy [ENT-2].
    ///
    /// An unsigned exact division bound at a binding or commit also returns
    /// the fact bounding the quotient by its dividend, which the caller's
    /// affine division images cite.
    fn establish_operation_facts(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        shape: &OperationShape<'_>,
        state: &mut FactState,
        shared_event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> Option<EstablishedUnsignedDivision> {
        let terms = shape
            .operands
            .iter()
            .map(|operand| self.copy_operand(operand))
            .collect::<Vec<_>>();
        let mut closed = None;
        let mut intervals = Vec::with_capacity(terms.len());
        let mut related = Vec::with_capacity(terms.len());
        let mut parents = Vec::new();
        for (operand, term) in shape.operands.iter().zip(&terms) {
            let ty = fragment_type(operand.ty())?;
            let span = Span::of(ty).interval();
            let Some(term) = *term else {
                intervals.push(span);
                related.push(None);
                continue;
            };
            if let Some(value) = self.constant_term_value(term) {
                intervals.push(Interval::value(value));
                related.push(None);
                continue;
            }
            let view = match &closed {
                Some(view) => Rc::clone(view),
                None => {
                    let view = close(state, &self.terms, &self.goals, &mut self.derivations);
                    closed = Some(Rc::clone(&view));
                    view
                }
            };
            // At a contradictory point every relation is already derivable.
            if view.contradictory() {
                return None;
            }
            let high = view.tight_bound(term, ZERO).unwrap_or(span.high);
            let low = view
                .tight_bound(ZERO, term)
                .and_then(i128::checked_neg)
                .unwrap_or(span.low);
            parents.extend(view.bound_proof(term, ZERO, high, &mut self.derivations));
            parents.extend(view.bound_proof(ZERO, term, -low, &mut self.derivations));
            intervals.push(Interval::new(low, high));
            related.push(Some(term));
        }
        let single = intervals
            .iter()
            .all(|interval| interval.low == interval.high);
        let product = matches!(shape.row, Row::Multiply { wrap: false })
            .then(|| self.product_intervals.get(shape.carrier).cloned())
            .flatten();
        if let (Some((_, Some(domain))), false) = (&product, single) {
            // The interval-product rule's four products are the measurement
            // the domain decision consumed; that decision is the parent.
            parents = vec![*domain];
        }
        let facts = operation_facts::facts(
            shape.row,
            Span::of(shape.operand),
            Span::of(shape.result),
            &intervals,
            product.map(|(interval, _)| Interval::new(interval.minimum, interval.maximum)),
        );
        if facts.bounds.is_none() && facts.relations.is_empty() {
            return None;
        }
        let result = self.bound_term(destination, value)?;
        let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
        let parents = parents.into_boxed_slice();
        let mut upper = None;
        if let Some(bounds) = facts.bounds {
            if let Some(low) = bounds.low.checked_neg() {
                self.establish_operation_fact(state, ZERO, result, low, event, &parents);
            }
            upper = Some(self.establish_operation_fact(
                state,
                result,
                ZERO,
                bounds.high,
                event,
                &parents,
            ));
        }
        let mut dividend_order = None;
        for relation in &facts.relations {
            let Some(term) = related.get(relation.operand).copied().flatten() else {
                continue;
            };
            if let Some(high) = relation.high {
                let proof =
                    self.establish_operation_fact(state, result, term, high, event, &parents);
                if relation.operand == 0 && high == 0 {
                    dividend_order = Some(proof);
                }
            }
            if let Some(low) = relation.low.and_then(i128::checked_neg) {
                self.establish_operation_fact(state, term, result, low, event, &parents);
            }
        }
        // [ENT-3.S7] an unsigned exact division at a binding or commit also
        // captures its value images; a conditional payload captures none.
        if shape.row != Row::Divide
            || shape.operand.signed()
            || matches!(destination, ValueImage::ResultPayload(_))
            || terms.iter().any(Option::is_none)
        {
            return None;
        }
        let literal_divisor = match &shape.operands[1] {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
                if *ty == shape.operand =>
            {
                let value = integer_value(*ty, *bits);
                (value > 0).then_some(value)
            }
            _ => None,
        };
        let parent = if related[0].is_some() {
            dividend_order
        } else {
            upper
        }?;
        Some(EstablishedUnsignedDivision {
            literal_divisor,
            parent,
        })
    }

    /// Establishes `left - right <= bound` as one [ENT-3.S7] fact whose
    /// parents are the closed operand bounds its row read [DIAG-2].
    fn establish_operation_fact(
        &mut self,
        state: &mut FactState,
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
        parents: &[DerivationId],
    ) -> DerivationId {
        let relation = Relation::Bound { left, right, bound };
        let proof = self.derivations.intern(DerivationNode::OperationFact {
            relation: relation.clone(),
            event,
            parents: parents.into(),
        });
        state.establish_from_proof(&relation, proof, &self.derivations);
        proof
    }

    /// [ENT-3] S9: `let x: T = c[i];` where c is the bare IDENT of a
    /// named const of type `array<T, N>` and T a fragment type establishes
    /// vlo <= x and x <= vhi over its N declared element values. The index's
    /// own bounds obligation is judged separately and is unaffected. Deeper
    /// const shapes establish nothing.
    fn establish_element_range(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        let constant = match value {
            CheckedExpression::ArrayIndex {
                root: CheckedArrayRoot::Constant(constant),
                ..
            } => *constant,
            CheckedExpression::ReadStorage { root, .. } => {
                let PlaceRoot::Constant(constant) = root.root else {
                    return false;
                };
                // Typed storage preserves the same bare-constant, single
                // subscript source shape. Fields or deeper subscripts do
                // not gain an element-range fact through this adapter.
                if !matches!(root.path.as_slice(), [CheckedPlaceStep::Subscript(_)]) {
                    return false;
                }
                constant
            }
            _ => return false,
        };
        let Some(constant) = self.context.constants.get(constant.0 as usize) else {
            return true;
        };
        let CheckedValue::Array { elements, .. } = &constant.value else {
            return true;
        };
        let mut range: Option<(i128, i128)> = None;
        for element in elements {
            let CheckedValue::Integer { ty, bits } = element else {
                return true;
            };
            let element = integer_value(*ty, *bits);
            range = Some(match range {
                Some((low, high)) => (low.min(element), high.max(element)),
                None => (element, element),
            });
        }
        let (Some((low, high)), Some(bound)) = (range, self.bound_term(destination, value)) else {
            return true;
        };
        let event = self.binding_event(event, FlowEventKind::S9, node_path);
        state.establish(
            &Relation::Bound {
                left: ZERO,
                right: bound,
                bound: -low,
            },
            &mut self.derivations,
            event,
        );
        state.establish(
            &Relation::Bound {
                left: bound,
                right: ZERO,
                bound: high,
            },
            &mut self.derivations,
            event,
        );
        true
    }

    // ------------------------------------------------------------------
    // Arm entry facts
    // ------------------------------------------------------------------

    /// The [ENT-3] arm facts one match scrutinee admits: the S1 comparison
    /// relation and goal origins of a `Bool` match. A checked row's success
    /// facts reach its `Ok` arm through the scrutinee's conditional Result
    /// context instead [ENT-5].
    pub(super) fn arm_facts(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        state: &FactState,
    ) -> ArmFacts {
        let node_path = Self::expression_node_path(scrutinee).cloned();
        if enum_type != CheckedEnumType::Bool {
            return ArmFacts {
                node_path,
                comparison: None,
                goals: Vec::new(),
            };
        }
        ArmFacts {
            node_path,
            comparison: self.scrutinee_relation(scrutinee, state),
            goals: self.goal_origin_set(scrutinee, state),
        }
    }
}

/// The normalized relation of one comparison operation over two read
/// operands, shared by the comparison-origin shape and S4's substitution.
///
/// `gap` is the displacement the two sides carry, `right`'s constant minus
/// `left`'s: a clause side is an affine expression [MSR-5], and
/// `at + 2_u64 <= len_of(run)` is the ordinary difference bound
/// `at - len_of(run) <= -2`.
pub(super) fn comparison_relation(
    operation: CheckedIntegerOperation,
    left: TermId,
    right: TermId,
    gap: i128,
) -> Option<Relation> {
    Some(match operation {
        CheckedIntegerOperation::Equal => Relation::Equal {
            left,
            right,
            difference: gap,
        },
        CheckedIntegerOperation::NotEqual => Relation::Distinct {
            left,
            right,
            difference: gap,
        },
        CheckedIntegerOperation::Less => Relation::Bound {
            left,
            right,
            bound: gap.checked_sub(1)?,
        },
        CheckedIntegerOperation::LessEqual => Relation::Bound {
            left,
            right,
            bound: gap,
        },
        CheckedIntegerOperation::Greater => Relation::Bound {
            left: right,
            right: left,
            bound: gap.checked_neg()?.checked_sub(1)?,
        },
        CheckedIntegerOperation::GreaterEqual => Relation::Bound {
            left: right,
            right: left,
            bound: gap.checked_neg()?,
        },
        _ => return None,
    })
}

/// The three [MSR-1] measures, in the order every former reads them.
const MEASURES: [CheckedMeasure; 3] = [
    CheckedMeasure::Length,
    CheckedMeasure::Capacity,
    CheckedMeasure::Head,
];

/// [MSR-3] one placement's datums, held between the mint before the
/// statement's kills and the establishment after them.
///
/// One placement carries one entry per measured place its operand reaches by
/// field or payload selection, so an aggregate operand carries the measures
/// of every run beneath it and not only its own.
pub(super) struct MeasureCarry {
    carried: Vec<CarriedMeasures>,
}

/// The datums of one measured place under one placement, with the aggregate
/// projection path that reaches it from the placement's source and destination.
struct CarriedMeasures {
    path: Vec<PlaceStep>,
    measured: MeasuredKind,
    constant: Option<CheckedConst>,
    datums: Vec<TermId>,
}
