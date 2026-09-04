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
//! S9, and S10. Retired labels are not reused [ENT-3].

use super::super::super::goal::CheckedRequirement;
use super::super::super::model::{
    BindingId, CheckedArrayRoot, CheckedEnumType, CheckedExpression, CheckedIntegerArgumentSource,
    CheckedIntegerOperation, CheckedMatchArm, CheckedMeasure, CheckedNominalKind, CheckedSetTarget,
    CheckedSliceSource, CheckedType, CheckedValue, IntegerType, MeasuredKind,
};
use super::super::fragment_type;
use super::super::state::{
    DerivationId, DerivationLedger, FactState, FlowEventId, FlowEventKind, OutcomeFact,
    OutcomeRelation, Relation, close,
};
use super::super::term::{
    CountedCaptureSide, PlaceProjection, PlaceRoot, PlaceTerm, ProjectedPlaceTerm, TermId,
    TermKind, ZERO, integer_value, type_range,
};
use super::super::{
    CountedAtomicDerivation, CountedBoundDerivation, CountedDerivationSet,
    CountedEqualityDerivation, CountedProofPoint, RemainderEndpoint, S7Derivation,
    S7DerivationKind, S7Subject, ShiftOneIdentity,
};
use super::{Analyzer, ArmFacts};
use crate::SYSTEM_OPERATIONS;

/// The [SYS-2] operations whose outcome carries an [ENT-3] S10 absolute
/// endpoint, with the observing variant. Both endpoint actuals are found by
/// parameter name in the catalog row, never by a hardcoded position.
const BOUNDARY_ENDPOINTS: [(&str, &str); 5] = [
    ("read_at", "ReadBytes"),
    ("write_once", "Ok"),
    ("host_copy_bytes", "Ok"),
    ("host_copy_utf8", "Ok"),
    ("directory_next", "ListBytes"),
];

/// Which term one evaluated value's [ENT-3] image is established on: the
/// place a `let` binder introduces, or the compiler-owned commit value of one
/// `set` occurrence, named by that statement's NodePath [ENT-2].
/// The sources below are written once against this destination so that a
/// commit's right-hand side receives exactly the image the same initializer
/// receives at a `let`.
#[derive(Clone, Copy)]
pub(super) enum ValueImage<'a> {
    Binding(BindingId),
    Commit(&'a crate::NodePath),
}

/// The three S11 terms installed for one counted range.
pub(super) struct CountedTerms {
    pub(super) lower_source: TermId,
    pub(super) lower: TermId,
    pub(super) binder: TermId,
    pub(super) upper: TermId,
    pub(super) upper_source: TermId,
}

/// The affine half of one admitted unsigned literal-division transfer.
/// The ordinary S7 relation supplies the retained source proof; the parent
/// flow binds the scaled quotient image to the exact current operand values.
pub(super) struct EstablishedUnsignedDivision {
    pub(super) divisor: i128,
    pub(super) parent: DerivationId,
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
        let lower_source = self
            .read_operand(lower)
            .expect("checked counted lower endpoint must be an ENT-2 term or constant");
        let upper_source = self
            .read_operand(upper)
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
            PlaceTerm {
                root: PlaceRoot::Binding(binder),
                deref: false,
                fields: Vec::new(),
            },
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
        if let Some(image) =
            self.establish_unsigned_division_bound(node_path, destination, value, state, event)
        {
            return Some(image);
        }
        if self.establish_offset_fact(node_path, destination, value, state, event) {
            return None;
        }
        if let ValueImage::Binding(binding) = destination {
            // Outcome origins are keyed by the binding a `match` can name;
            // a commit value is unnameable and carries none.
            self.record_outcome_origin(binding, value, state);
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

    /// The exact term named by one writable fragment place. A borrow holder
    /// keeps its canonical deref projection so the post-write destination is
    /// identical to a later read of that same place [ENT-2].
    fn writable_place_term(
        &mut self,
        binding: BindingId,
        fields: &[u32],
        ty: CheckedType,
    ) -> Option<TermId> {
        let fragment = fragment_type(ty)?;
        let kind = if self.needs_implicit_deref(binding) {
            TermKind::ProjectedPlace(
                ProjectedPlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    projections: std::iter::once(PlaceProjection::Deref)
                        .chain(fields.iter().copied().map(PlaceProjection::Field))
                        .collect(),
                },
                fragment,
            )
        } else {
            TermKind::Place(
                PlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    deref: false,
                    fields: fields.to_vec(),
                },
                fragment,
            )
        };
        Some(self.terms.intern(kind))
    }

    /// The term one evaluated value's image is established on, when its type
    /// is one fragment type: a freshly bound integer place, or the commit
    /// value of one `set` occurrence.
    fn bound_term(
        &mut self,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
    ) -> Option<TermId> {
        match destination {
            ValueImage::Binding(binding) => self.writable_place_term(binding, &[], value.ty()),
            ValueImage::Commit(node_path) => self.commit_value_term(node_path, value),
        }
    }

    /// The retained identity of the value one S7 image was established on.
    fn s7_subject(destination: ValueImage<'_>) -> S7Subject {
        match destination {
            ValueImage::Binding(binding) => S7Subject::Binding(binding),
            ValueImage::Commit(node_path) => S7Subject::Commit(node_path.clone()),
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

    /// The value image shared by an ordinary let and a direct-place SET-1
    /// commit. A narrowing conversion and every computed expression outside
    /// this finite S5 table have no image.
    fn copy_source(&mut self, value: &CheckedExpression) -> Option<TermId> {
        match value {
            CheckedExpression::NumericConversion {
                source,
                destination,
                value: operand,
                ..
            } => source
                .converts_totally_to(*destination)
                .then(|| self.read_operand(operand))
                .flatten(),
            _ => self.read_operand(value),
        }
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
    fn bound_place(&self, binding: BindingId) -> PlaceTerm {
        PlaceTerm {
            root: PlaceRoot::Binding(binding),
            deref: false,
            fields: Vec::new(),
        }
    }

    /// [ENT-3] S5: `let x: own T = lit;` establishes x = value(lit);
    /// `let x: own T = p;` with p a term establishes x = p; and
    /// `let y: own Dst = cvt::<Src, Dst>(p);` over a total [OP-6] pair
    /// establishes y = p, the conversion being exactly value-preserving.
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
        let CheckedSetTarget::Place(target) = target else {
            return;
        };
        let Some(destination) = self.writable_place_term(target.binding, &target.fields, target.ty)
        else {
            return;
        };
        self.establish_copy_equality(node_path, destination, commit, state, event);
    }

    /// [ENT-3] S6: `buffer_new::<T>(n, v)` establishes len_of(b) = n;
    /// `len::<T>(P)` for a tracked P establishes m = len_of(P); and
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
        // A length equality is stated over the destination place. One commit
        // value is no place, so an allocation or slice-formation right-hand
        // side has no commit image; the length operand row below is an
        // ordinary fragment value and applies to both destinations.
        match value {
            CheckedExpression::BufferFill { length, .. }
            | CheckedExpression::BufferVacant { length, .. } => {
                let ValueImage::Binding(binding) = destination else {
                    return true;
                };
                let Some(allocated) = self.read_operand(length) else {
                    return true;
                };
                let place = self.bound_place(binding);
                let length_term = self.place_measure_term(
                    CheckedMeasure::Length,
                    place,
                    MeasuredKind::Buffer,
                    None,
                );
                let event = self.binding_event(event, FlowEventKind::S6, node_path);
                state.establish(
                    &Relation::Equal {
                        left: length_term,
                        right: allocated,
                        difference: 0,
                    },
                    &mut self.derivations,
                    event,
                );
                true
            }
            CheckedExpression::SliceOf { source, .. } => {
                let ValueImage::Binding(binding) = destination else {
                    return true;
                };
                let (place, array_length) = match source {
                    CheckedSliceSource::Array { root, length } => {
                        (self.array_root_place(root), Some(*length))
                    }
                    CheckedSliceSource::Buffer(root) => (
                        PlaceTerm {
                            root: PlaceRoot::Binding(root.binding),
                            deref: self.is_holder(root.binding),
                            fields: root.fields.clone(),
                        },
                        None,
                    ),
                    // Content reached in an arena through one explicit deref
                    // [OWN-5]; the viewed array's constant length still
                    // equates to the formed slice's length.
                    CheckedSliceSource::ArenaContent {
                        binding,
                        fields,
                        length,
                    } => (
                        PlaceTerm {
                            root: PlaceRoot::Binding(*binding),
                            deref: true,
                            fields: fields.clone(),
                        },
                        Some(*length),
                    ),
                };
                let source_length = self.place_measure_term(
                    CheckedMeasure::Length,
                    place,
                    if array_length.is_some() {
                        MeasuredKind::Array
                    } else {
                        MeasuredKind::Buffer
                    },
                    array_length,
                );
                let slice_place = self.bound_place(binding);
                let slice_length = self.place_measure_term(
                    CheckedMeasure::Length,
                    slice_place,
                    MeasuredKind::Slice,
                    None,
                );
                let event = self.binding_event(event, FlowEventKind::S6, node_path);
                state.establish(
                    &Relation::Equal {
                        left: slice_length,
                        right: source_length,
                        difference: 0,
                    },
                    &mut self.derivations,
                    event,
                );
                true
            }
            _ => match self.measure_operand(value) {
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
            },
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
                MeasuredKind::Array,
                Some(*length),
            ),
            CheckedExpression::BufferMeasure { measure, root } => (
                *measure,
                PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                },
                MeasuredKind::Buffer,
                None,
            ),
            CheckedExpression::SliceMeasure { measure, root } => (
                *measure,
                PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                },
                MeasuredKind::Slice,
                None,
            ),
            _ => return None,
        };
        Some(self.place_measure_term(measure, place, measured, array_length))
    }

    /// [ENT-3] S7 constant-offset arithmetic at a `let` binding.
    ///
    /// `iadd.wrap::<T>(p, k)` and `isub.wrap::<T>(p, k)` with a constant k
    /// establish s = p ± k only where the closed state already proves the
    /// unwrapped result stays in T's range, so the established equality is
    /// over the mathematical value the wrap did not reach. Exact `+` and `-`
    /// establish it unconditionally on their normal continuation because
    /// their IntegerDomain obligation was proved before acceptance [OP-2].
    fn establish_offset_fact(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        if self.establish_remainder_bounds(node_path, destination, value, state, event)
            || self.establish_bit_and_bounds(node_path, destination, value, state, event)
            || self.establish_shift_one_nonzero(node_path, destination, value, state, event)
        {
            return true;
        }
        let Some((base, delta, exact)) = self.constant_offset(value) else {
            return false;
        };
        let Some(bound) = self.bound_term(destination, value) else {
            return true;
        };
        if !exact {
            let Some(ty) = fragment_type(value.ty()) else {
                return true;
            };
            let (minimum, maximum) = type_range(ty);
            let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
            // `min(T) <= p + k` and `p + k <= max(T)`, as bounds on p through
            // Z: p - Z <= max(T) - k and Z - p <= k - min(T).
            let within = closed.derives_bound(base, ZERO, maximum.saturating_sub(delta))
                && closed.derives_bound(ZERO, base, delta.saturating_sub(minimum));
            if !within {
                return true;
            }
        }
        let event = self.binding_event(event, FlowEventKind::S7, node_path);
        establish_shifted(state, bound, base, delta, &mut self.derivations, event);
        true
    }

    /// [ENT-3] S7: an admitted unsigned exact division by a positive written
    /// integer literal publishes `quotient <= dividend`. The returned source
    /// proof lets the parent flow also retain the exact affine image
    /// `literal * quotient <= dividend` over the same runtime value atoms.
    /// Signed division deliberately has no member of this rule.
    fn establish_unsigned_division_bound(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        shared_event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> Option<EstablishedUnsignedDivision> {
        let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::DivideExact,
            operand_type: CheckedType::Integer(row),
            arguments,
            ..
        } = value
        else {
            return None;
        };
        if row.signed() {
            return None;
        }
        let [dividend, divisor] = arguments.as_slice() else {
            return None;
        };
        let CheckedExpression::Constant(CheckedValue::Integer { ty, bits }) = divisor else {
            return None;
        };
        if ty != row {
            return None;
        }
        let divisor = integer_value(*ty, *bits);
        if divisor <= 0 {
            return None;
        }
        let result = self.bound_term(destination, value)?;
        let dividend = self.read_operand(dividend)?;
        let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
        let relation = Relation::Bound {
            left: result,
            right: dividend,
            bound: 0,
        };
        let parent =
            state.establish_bound_with_proof(result, dividend, 0, &mut self.derivations, event);
        self.retain_s7_derivation(S7Derivation {
            source: node_path.clone(),
            row: *row,
            subject: Self::s7_subject(destination),
            kind: S7DerivationKind::UnsignedDivisionBound { dividend, divisor },
            relation,
            event,
            parent,
        });
        Some(EstablishedUnsignedDivision { divisor, parent })
    }

    /// [ENT-3] S7: an admitted unsigned exact remainder publishes
    /// `result < divisor`. A signed remainder by a written constant `d`
    /// publishes the closed interval `-(|d| - 1) <= result <= |d| - 1`.
    /// The operation's ordinary IntegerDomain judgment has already proved its
    /// domain before acceptance; this transfer performs no proof search.
    fn establish_remainder_bounds(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        shared_event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::RemainderExact,
            operand_type: CheckedType::Integer(row),
            arguments,
            ..
        } = value
        else {
            return false;
        };
        let [_dividend, divisor] = arguments.as_slice() else {
            return true;
        };
        let Some(result) = self.bound_term(destination, value) else {
            return true;
        };
        let Some(divisor) = self.read_operand(divisor) else {
            return true;
        };
        let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
        if !row.signed() {
            let relation = Relation::Bound {
                left: result,
                right: divisor,
                bound: -1,
            };
            let parent =
                state.establish_bound_with_proof(result, divisor, -1, &mut self.derivations, event);
            self.retain_s7_derivation(S7Derivation {
                source: node_path.clone(),
                row: *row,
                subject: Self::s7_subject(destination),
                kind: S7DerivationKind::UnsignedRemainderBound { divisor },
                relation,
                event,
                parent,
            });
            return true;
        }

        let Some(divisor_value) = self
            .constant_term_value(divisor)
            .filter(|value| *value != 0)
        else {
            return true;
        };
        let Some(limit) = divisor_value
            .checked_abs()
            .and_then(|value| value.checked_sub(1))
        else {
            return true;
        };
        for (endpoint, left, right) in [
            (RemainderEndpoint::Minimum, ZERO, result),
            (RemainderEndpoint::Maximum, result, ZERO),
        ] {
            let relation = Relation::Bound {
                left,
                right,
                bound: limit,
            };
            let parent =
                state.establish_bound_with_proof(left, right, limit, &mut self.derivations, event);
            self.retain_s7_derivation(S7Derivation {
                source: node_path.clone(),
                row: *row,
                subject: Self::s7_subject(destination),
                kind: S7DerivationKind::SignedRemainderBound {
                    divisor: divisor_value,
                    endpoint,
                },
                relation,
                event,
                parent,
            });
        }
        true
    }

    fn establish_bit_and_bounds(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        shared_event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::BitAnd,
            operand_type: CheckedType::Integer(row),
            arguments,
            ..
        } = value
        else {
            return false;
        };
        if row.signed() {
            return true;
        }
        let Some(result) = self.bound_term(destination, value) else {
            return true;
        };
        for (operand, argument) in arguments.iter().enumerate() {
            let Some(admitted) = self.read_operand(argument) else {
                continue;
            };
            let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
            let relation = Relation::Bound {
                left: result,
                right: admitted,
                bound: 0,
            };
            let parent =
                state.establish_bound_with_proof(result, admitted, 0, &mut self.derivations, event);
            self.retain_s7_derivation(S7Derivation {
                source: node_path.clone(),
                row: *row,
                subject: Self::s7_subject(destination),
                kind: S7DerivationKind::BitAndBound {
                    operand: u8::try_from(operand)
                        .expect("integer-operation operand ordinal exceeds u8"),
                    admitted,
                },
                relation,
                event,
                parent,
            });
        }
        true
    }

    fn establish_shift_one_nonzero(
        &mut self,
        node_path: &crate::NodePath,
        destination: ValueImage<'_>,
        value: &CheckedExpression,
        state: &mut FactState,
        shared_event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        let CheckedExpression::IntegerOperation {
            operation: CheckedIntegerOperation::ShiftLeftWrap,
            operand_type: CheckedType::Integer(row),
            argument_metadata,
            arguments,
            ..
        } = value
        else {
            return false;
        };
        if row.signed() {
            return true;
        }
        let [one, _count] = arguments.as_slice() else {
            return true;
        };
        let [one_metadata, count_metadata] = argument_metadata.as_slice() else {
            return true;
        };
        let one_value = match one {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits }) => {
                (integer_value(*ty, *bits) == 1).then_some(())
            }
            CheckedExpression::NamedConstant {
                value: CheckedValue::Integer { ty, bits },
                ..
            } => (integer_value(*ty, *bits) == 1).then_some(()),
            _ => None,
        };
        if one_value.is_none() {
            return true;
        }
        let one = match one_metadata.source {
            CheckedIntegerArgumentSource::TypedLiteral => ShiftOneIdentity::TypedLiteral {
                source: one_metadata.node_path.clone(),
            },
            CheckedIntegerArgumentSource::NamedConstant { declaration } => {
                ShiftOneIdentity::NamedConstant { declaration }
            }
            CheckedIntegerArgumentSource::GenericNumericIdentity
            | CheckedIntegerArgumentSource::Other => return true,
        };
        let Some(result) = self.bound_term(destination, value) else {
            return true;
        };
        let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
        let (left, right) = if result < ZERO {
            (result, ZERO)
        } else {
            (ZERO, result)
        };
        let relation = Relation::Distinct {
            left,
            right,
            difference: 0,
        };
        let parent =
            state.establish_distinct_with_proof(result, ZERO, &mut self.derivations, event);
        self.retain_s7_derivation(S7Derivation {
            source: node_path.clone(),
            row: *row,
            subject: Self::s7_subject(destination),
            kind: S7DerivationKind::ShiftOneNonzero {
                count_atom: count_metadata.node_path.clone(),
                one,
            },
            relation,
            event,
            parent,
        });
        true
    }

    /// The `(p, k, exact)` reading of one constant-offset arithmetic call.
    /// Exact rows have already discharged their static IntegerDomain
    /// obligation; wrapping rows need the additional range proof above.
    /// Addition accepts its constant in either operand position, subtraction
    /// only as the subtrahend, since `k - p` is no offset of p.
    fn constant_offset(&mut self, value: &CheckedExpression) -> Option<(TermId, i128, bool)> {
        let CheckedExpression::IntegerOperation {
            operation,
            operand_type,
            arguments,
            ..
        } = value
        else {
            return None;
        };
        fragment_type(*operand_type)?;
        let [left, right] = arguments.as_slice() else {
            return None;
        };
        let (adding, exact) = match operation {
            CheckedIntegerOperation::AddWrap => (true, false),
            CheckedIntegerOperation::AddExact => (true, true),
            CheckedIntegerOperation::SubtractWrap => (false, false),
            CheckedIntegerOperation::SubtractExact => (false, true),
            _ => return None,
        };
        let left = self.read_operand(left)?;
        let right = self.read_operand(right)?;
        let (base, delta) = self.split_offset(left, right, adding)?;
        Some((base, delta, exact))
    }

    /// Splits one operand pair into a base term and a constant offset.
    fn split_offset(&self, left: TermId, right: TermId, adding: bool) -> Option<(TermId, i128)> {
        if let Some(value) = self.constant_term_value(right) {
            return Some((left, if adding { value } else { -value }));
        }
        if adding && let Some(value) = self.constant_term_value(left) {
            return Some((right, value));
        }
        None
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

    /// [ENT-3] S9: `let x: own T = c[i];` where c is the bare IDENT of a
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
        let CheckedExpression::ArrayIndex {
            root: CheckedArrayRoot::Constant(constant),
            ..
        } = value
        else {
            return false;
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
    // S7 checked arithmetic and S10 boundary counts, observed at a match
    // ------------------------------------------------------------------

    /// Records the outcome origin of a `let` whose initializer is a checked
    /// arithmetic call or a bounded [SYS-2] boundary call, so a later match
    /// over the bare IDENT observes the same fact the direct scrutinee does.
    /// The recorded origin dies with any kill on its base term and with a
    /// `set` naming the binding, the discipline [ENT-3] states for it.
    fn record_outcome_origin(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
    ) {
        if let Some(outcome) = self.outcome_fact(value) {
            state.outcomes.insert(binding, outcome);
        }
    }

    /// The [ENT-3] arm facts one match scrutinee admits: the S1 comparison
    /// relation for a `Bool` match, and the S7/S10 fact carried by an
    /// outcome-typed scrutinee — the call directly, or a bare IDENT naming a
    /// binding of its outcome whose origin survived the path here.
    pub(super) fn arm_facts(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        state: &FactState,
    ) -> ArmFacts {
        let node_path = Self::expression_node_path(scrutinee).cloned();
        if enum_type == CheckedEnumType::Bool {
            return ArmFacts {
                node_path,
                comparison: self.scrutinee_relation(scrutinee, state),
                goals: self.goal_origin_set(scrutinee, state),
                outcome: None,
            };
        }
        let outcome = match scrutinee {
            CheckedExpression::Binding { binding, .. } => state.outcomes.get(binding).cloned(),
            _ => self.outcome_fact(scrutinee),
        };
        let outcome = outcome.and_then(|outcome| {
            self.variant_tag(enum_type, outcome.variant)
                .map(|tag| (tag, outcome))
        });
        ArmFacts {
            node_path,
            comparison: None,
            goals: Vec::new(),
            outcome,
        }
    }

    /// The tag of one named variant of a checked enum type.
    fn variant_tag(&self, enum_type: CheckedEnumType, variant: &str) -> Option<u32> {
        let CheckedEnumType::Nominal(nominal) = enum_type else {
            return None;
        };
        let nominal = self.context.nominals.get(nominal.0 as usize)?;
        let CheckedNominalKind::Enum { variants } = &nominal.kind else {
            return None;
        };
        variants
            .iter()
            .find(|candidate| candidate.name == variant)
            .map(|candidate| candidate.tag)
    }

    /// The outcome fact one call expression carries, if any: S7's checked
    /// `Ok(value: w)` shift, or S10's absolute endpoint on the observing arm.
    fn outcome_fact(&mut self, value: &CheckedExpression) -> Option<OutcomeFact> {
        self.checked_offset_outcome(value)
            .or_else(|| self.boundary_endpoint_outcome(value))
    }

    /// [ENT-3] S7: `iadd.checked::<T>(p, k)` and `isub.checked::<T>(p, k)` with a
    /// constant k give the `Ok(value: w)` arm w = p ± k; the `Err` arm
    /// establishes nothing.
    fn checked_offset_outcome(&mut self, value: &CheckedExpression) -> Option<OutcomeFact> {
        let CheckedExpression::IntegerOperation {
            operation,
            operand_type,
            arguments,
            ..
        } = value
        else {
            return None;
        };
        fragment_type(*operand_type)?;
        let adding = match operation {
            CheckedIntegerOperation::AddChecked => true,
            CheckedIntegerOperation::SubtractChecked => false,
            _ => return None,
        };
        let [left, right] = arguments.as_slice() else {
            return None;
        };
        let left = self.read_operand(left)?;
        let right = self.read_operand(right)?;
        let (base, delta) = self.split_offset(left, right, adding)?;
        Some(OutcomeFact {
            variant: "Ok",
            base,
            relation: OutcomeRelation::Shifted(delta),
            event_kind: FlowEventKind::S7,
        })
    }

    /// [ENT-3] S10: a [SYS-2] transfer's observing arm binds the absolute
    /// endpoint `next`, establishing `start <= next <= end`.
    /// The fact carries the same trust class as S6's allocation-length
    /// equality: it is a declared operation contract, never a writer
    /// statement.
    ///
    /// The bounds are admitted only where no kill event on the path to the
    /// match reaches either endpoint support. The call's own boundary writes
    /// are on that path, so an endpoint read through a place the call writes
    /// admits nothing — the conservative reading, which only under-derives.
    fn boundary_endpoint_outcome(&mut self, value: &CheckedExpression) -> Option<OutcomeFact> {
        let CheckedExpression::SystemCall {
            operation,
            arguments,
            ..
        } = value
        else {
            return None;
        };
        let row = SYSTEM_OPERATIONS.get(usize::from(*operation))?;
        let (_, variant) = BOUNDARY_ENDPOINTS
            .iter()
            .find(|(spelling, _)| *spelling == row.spelling)?;
        let start_position = row
            .parameters
            .iter()
            .position(|parameter| parameter.name == "start")?;
        let end_position = row
            .parameters
            .iter()
            .position(|parameter| parameter.name == "end")?;
        let base = self.read_operand(arguments.get(start_position)?)?;
        let upper = self.read_operand(arguments.get(end_position)?)?;
        let mut events = Vec::new();
        self.collect_expression_kills(value, &mut events);
        if events
            .iter()
            .any(|event| self.event_kills_term(base, event) || self.event_kills_term(upper, event))
        {
            return None;
        }
        Some(OutcomeFact {
            variant,
            base,
            relation: OutcomeRelation::Between { upper },
            event_kind: FlowEventKind::S10,
        })
    }

    /// Establishes one arm's binder fact at arm entry: the value binder of
    /// the observing variant gains the recorded relation against its base.
    pub(super) fn establish_binder_fact(
        &mut self,
        arm: &CheckedMatchArm,
        outcome: &OutcomeFact,
        state: &mut FactState,
        event: FlowEventId,
    ) {
        let Some(binder) = arm.binders.iter().find(|binder| binder.field == 0) else {
            return;
        };
        let Some(fragment) = fragment_type(binder.ty) else {
            return;
        };
        let place = PlaceTerm {
            root: PlaceRoot::Binding(binder.binding),
            deref: false,
            fields: Vec::new(),
        };
        let bound = self.terms.intern(TermKind::Place(place, fragment));
        match outcome.relation {
            OutcomeRelation::Shifted(delta) => {
                establish_shifted(
                    state,
                    bound,
                    outcome.base,
                    delta,
                    &mut self.derivations,
                    event,
                );
            }
            OutcomeRelation::Between { upper } => {
                state.establish(
                    &Relation::Bound {
                        left: outcome.base,
                        right: bound,
                        bound: 0,
                    },
                    &mut self.derivations,
                    event,
                );
                state.establish(
                    &Relation::Bound {
                        left: bound,
                        right: upper,
                        bound: 0,
                    },
                    &mut self.derivations,
                    event,
                );
            }
        }
    }
}

/// `bound = base + delta`, as the difference-bound pair over that term pair.
fn establish_shifted(
    state: &mut FactState,
    bound: TermId,
    base: TermId,
    delta: i128,
    ledger: &mut DerivationLedger,
    event: FlowEventId,
) {
    state.establish(
        &Relation::Bound {
            left: bound,
            right: base,
            bound: delta,
        },
        ledger,
        event,
    );
    state.establish(
        &Relation::Bound {
            left: base,
            right: bound,
            bound: -delta,
        },
        ledger,
        event,
    );
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
