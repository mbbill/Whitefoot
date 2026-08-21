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
//! Implemented here: S1 (through the parent's arm entry), S3 (a passed claim
//! condition), S4, S5, S6, S7, S9, and S10. Retired labels are not reused
//! [ENT-3].

use super::super::super::goal::CheckedRequirement;
use super::super::super::model::{
    BindingId, CheckedArrayRoot, CheckedEnumType, CheckedExpression, CheckedIntegerArgumentSource,
    CheckedIntegerOperation, CheckedMatchArm, CheckedNominalKind, CheckedSliceSource, CheckedType,
    CheckedValue, IntegerType,
};
use super::super::fragment_type;
use super::super::state::{
    DerivationLedger, DerivationRootKind, FactState, FlowEventId, FlowEventKind, OutcomeFact,
    OutcomeRelation, Relation, close,
};
use super::super::term::{
    CountedCaptureSide, PlaceRoot, PlaceTerm, TermId, TermKind, ZERO, integer_value, type_range,
};
use super::super::{
    ClaimComponentEvidence, ClaimImageEvidence, ClaimProofEvidence, ClaimReconstructionEvidence,
    CountedAtomicDerivation, CountedBoundDerivation, CountedDerivationSet,
    CountedEqualityDerivation, CountedProofPoint, S7Derivation, S7DerivationKind, ShiftOneIdentity,
};
use super::{Analyzer, ArmFacts};
use crate::SYSTEM_OPERATIONS;

/// The [SYS-2] operations whose outcome carries an [ENT-3] S10 absolute
/// endpoint, with the observing variant. Both endpoint actuals are found by
/// parameter name in the catalog row, never by a hardcoded position.
const BOUNDARY_ENDPOINTS: [(&str, &str); 5] = [
    ("read_once", "ReadBytes"),
    ("write_once", "Ok"),
    ("host_copy_bytes", "Ok"),
    ("host_copy_utf8", "Ok"),
    ("list_once", "ListBytes"),
];

/// The three S11 terms installed for one counted range.
pub(super) struct CountedTerms {
    pub(super) lower_source: TermId,
    pub(super) lower: TermId,
    pub(super) binder: TermId,
    pub(super) upper: TermId,
    pub(super) upper_source: TermId,
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
            },
            &mut self.derivations,
            event,
        );
        state.establish(
            &Relation::Equal {
                left: upper_capture,
                right: upper_source,
            },
            &mut self.derivations,
            event,
        );
        state.establish(
            &Relation::Equal {
                left: binder,
                right: lower_capture,
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
                relation: Relation::Equal { left, right },
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
    // S3 claim facts
    // ------------------------------------------------------------------

    /// [ENT-3] S3 establishes the canonical contribution components, never
    /// the parent first. The direct ordinary-let image is materialized only
    /// after ordinary ENT-4 reconstruction proves the expanded predicate.
    pub(super) fn establish_claim_contribution(
        &mut self,
        node_path: &crate::NodePath,
        contribution: &super::ClaimContribution,
        renderings: &[String],
        occurrence: u32,
        state: &mut FactState,
    ) -> Option<ClaimProofEvidence> {
        if contribution.components.len() != renderings.len()
            || contribution.exact_goals.is_empty()
            || contribution.exact_goals.len() > 2
        {
            return None;
        }
        let active_mask = self
            .claim_mask
            .filter(|mask| mask.function == self.function.id && mask.node_path == *node_path);
        let mut component_evidence = Vec::with_capacity(contribution.components.len());
        for (index, component) in contribution.components.iter().enumerate() {
            let ordinal = u32::try_from(index)
                .expect("claim contribution component count exceeds the u32 identity space");
            if active_mask
                .is_some_and(|mask| mask.component.is_none() || mask.component == Some(ordinal))
            {
                continue;
            }
            let event = self.derivations.claim_event(node_path.clone(), ordinal);
            let source = match component {
                super::ClaimComponentFact::Goal { goal, sign } => {
                    state.establish_goal_with_proof(*goal, *sign, &mut self.derivations, event)
                }
                super::ClaimComponentFact::Relation(Relation::Bound { left, right, bound }) => {
                    state.establish_bound_with_proof(
                        *left,
                        *right,
                        *bound,
                        &mut self.derivations,
                        event,
                    )
                }
                super::ClaimComponentFact::Relation(Relation::Distinct { left, right }) => {
                    state.establish_distinct_with_proof(*left, *right, &mut self.derivations, event)
                }
                super::ClaimComponentFact::Relation(Relation::Equal { .. }) => return None,
            };
            self.derivations.add_root(
                DerivationRootKind::ClaimComponent {
                    occurrence,
                    component: ordinal,
                },
                source,
            );
            component_evidence.push(ClaimComponentEvidence {
                ordinal,
                fact: component.clone(),
                rendering: renderings[index].clone(),
                source,
            });
        }
        if active_mask.is_some_and(|mask| mask.component.is_none()) {
            return None;
        }
        let canonical = contribution.exact_goals.last().copied()?;
        let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
        if closed.contradictory()
            || !closed.derives_goal(
                canonical,
                super::super::state::GoalSign::Positive,
                &self.goals,
            )
            || closed.derives_goal(
                canonical,
                super::super::state::GoalSign::Negative,
                &self.goals,
            )
        {
            return None;
        }
        let parent = closed.goal_proof(
            canonical,
            super::super::state::GoalSign::Positive,
            &self.goals,
            &mut self.derivations,
        )?;
        self.derivations.add_root(
            DerivationRootKind::ClaimReconstruction {
                occurrence,
                direct: false,
            },
            parent,
        );
        let direct_goal = contribution.exact_goals[0];
        let mut direct_proof = (direct_goal == canonical).then_some(parent);
        for direct in contribution
            .exact_goals
            .iter()
            .copied()
            .filter(|goal| *goal != canonical)
        {
            let event = self.proof_event(FlowEventKind::ClaimReconstruction, Some(node_path));
            let proof = state.establish_goal_from_proof(
                direct,
                super::super::state::GoalSign::Positive,
                parent,
                &mut self.derivations,
                event,
            )?;
            if direct == direct_goal {
                direct_proof = Some(proof);
            }
        }
        let direct_proof = direct_proof?;
        self.derivations.add_root(
            DerivationRootKind::ClaimReconstruction {
                occurrence,
                direct: true,
            },
            direct_proof,
        );
        Some(ClaimProofEvidence {
            images: ClaimImageEvidence {
                direct: direct_goal,
                expanded: canonical,
                complete: contribution.lifecycle_complete,
            },
            components: component_evidence,
            reconstructions: ClaimReconstructionEvidence {
                expanded: parent,
                direct: direct_proof,
            },
        })
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
        let view = state.proof_view();
        self.establish_boolean_decomposition(
            goal,
            super::super::state::GoalSign::Positive,
            state,
            event,
        );
        self.record_boolean_decomposition(goal, super::super::state::GoalSign::Positive, view);
    }

    // ------------------------------------------------------------------
    // Establishment at a `let` binding: S5, S6, S7, S9
    // ------------------------------------------------------------------

    /// Every source that establishes at an `ordinary_let_rhs` binding, in one
    /// place because they are mutually exclusive on the initializer's shape.
    pub(super) fn establish_binding_facts(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) {
        if self.establish_length_facts(node_path, binding, value, state, event) {
            return;
        }
        if self.establish_element_range(node_path, binding, value, state, event) {
            return;
        }
        if self.establish_offset_fact(node_path, binding, value, state, event) {
            return;
        }
        self.record_outcome_origin(binding, value, state);
        self.establish_copy_fact(node_path, binding, value, state, event);
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

    /// The term of a freshly bound integer place, when its type is one
    /// fragment type.
    fn bound_term(&mut self, binding: BindingId, value: &CheckedExpression) -> Option<TermId> {
        let fragment = fragment_type(value.ty())?;
        let place = PlaceTerm {
            root: PlaceRoot::Binding(binding),
            deref: false,
            fields: Vec::new(),
        };
        Some(self.terms.intern(TermKind::Place(place, fragment)))
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
    /// `let y: own Dst = cvt<Src, Dst>(p);` over a total [OP-6] pair
    /// establishes y = p, the conversion being exactly value-preserving.
    fn establish_copy_fact(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) {
        let source = match value {
            CheckedExpression::NumericConversion {
                source,
                destination,
                value: operand,
                ..
            } => {
                if !source.converts_totally_to(*destination) {
                    return;
                }
                self.read_operand(operand)
            }
            _ => self.read_operand(value),
        };
        let Some(source) = source else {
            return;
        };
        let Some(bound) = self.bound_term(binding, value) else {
            return;
        };
        let event = self.binding_event(event, FlowEventKind::S5, node_path);
        state.establish(
            &Relation::Equal {
                left: bound,
                right: source,
            },
            &mut self.derivations,
            event,
        );
    }

    /// [ENT-3] S6: `buffer_new<T>(n, v)` establishes len(b) = n;
    /// `len<T>(P)` for a tracked P establishes m = len(P); and
    /// `slice_of…(&'r P)` for a tracked P establishes len(s) = len(P).
    ///
    /// An `array<T, N>` allocation needs no clause here: its length equality
    /// is the [ENT-2] implicit fact carried by every length term over an
    /// array-typed place, registered wherever that term is interned.
    fn establish_length_facts(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        match value {
            CheckedExpression::BufferFill { length, .. }
            | CheckedExpression::BufferVacant { length, .. } => {
                let Some(allocated) = self.read_operand(length) else {
                    return true;
                };
                let place = self.bound_place(binding);
                let length_term = self.length_term(place, None);
                let event = self.binding_event(event, FlowEventKind::S6, node_path);
                state.establish(
                    &Relation::Equal {
                        left: length_term,
                        right: allocated,
                    },
                    &mut self.derivations,
                    event,
                );
                true
            }
            CheckedExpression::SliceOf { source, .. } => {
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
                let source_length = self.length_term(place, array_length);
                let slice_place = self.bound_place(binding);
                let slice_length = self.length_term(slice_place, None);
                let event = self.binding_event(event, FlowEventKind::S6, node_path);
                state.establish(
                    &Relation::Equal {
                        left: slice_length,
                        right: source_length,
                    },
                    &mut self.derivations,
                    event,
                );
                true
            }
            _ => match self.length_operand(value) {
                Some(source_length) => {
                    if let Some(bound) = self.bound_term(binding, value) {
                        let event = self.binding_event(event, FlowEventKind::S6, node_path);
                        state.establish(
                            &Relation::Equal {
                                left: bound,
                                right: source_length,
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

    /// The length term one `len<T>(P)` call reads, over the same place the
    /// obligation judgment forms for P, so both name one term [ENT-2].
    pub(super) fn length_operand(&mut self, value: &CheckedExpression) -> Option<TermId> {
        let (place, array_length) = match value {
            CheckedExpression::ArrayLength { root, length, .. } => {
                (self.array_root_place(root), Some(*length))
            }
            CheckedExpression::BufferLength { root, .. } => (
                PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                },
                None,
            ),
            CheckedExpression::SliceLength { root, .. } => (
                PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                },
                None,
            ),
            _ => return None,
        };
        Some(self.length_term(place, array_length))
    }

    /// [ENT-3] S7 constant-offset arithmetic at a `let` binding.
    ///
    /// `iadd.wrap<T>(p, k)` and `isub.wrap<T>(p, k)` with a constant k
    /// establish s = p ± k only where the closed state already proves the
    /// unwrapped result stays in T's range, so the established equality is
    /// over the mathematical value the wrap did not reach. Exact `+` and `-`
    /// establish it unconditionally on their normal continuation because
    /// their IntegerDomain obligation was proved before acceptance [OP-2].
    fn establish_offset_fact(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
        event: &mut Option<(FlowEventKind, FlowEventId)>,
    ) -> bool {
        if self.establish_bit_and_bounds(node_path, binding, value, state, event)
            || self.establish_shift_one_nonzero(node_path, binding, value, state, event)
        {
            return true;
        }
        let Some((base, delta, exact)) = self.constant_offset(value) else {
            return false;
        };
        let Some(bound) = self.bound_term(binding, value) else {
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

    fn establish_bit_and_bounds(
        &mut self,
        node_path: &crate::NodePath,
        binding: BindingId,
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
        let Some(result) = self.bound_term(binding, value) else {
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
                view: state.proof_view(),
                row: *row,
                binding,
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
        binding: BindingId,
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
        let Some(result) = self.bound_term(binding, value) else {
            return true;
        };
        let event = self.binding_event(shared_event, FlowEventKind::S7, node_path);
        let (left, right) = if result < ZERO {
            (result, ZERO)
        } else {
            (ZERO, result)
        };
        let relation = Relation::Distinct { left, right };
        let parent =
            state.establish_distinct_with_proof(result, ZERO, &mut self.derivations, event);
        self.retain_s7_derivation(S7Derivation {
            source: node_path.clone(),
            view: state.proof_view(),
            row: *row,
            binding,
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
        binding: BindingId,
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
        let (Some((low, high)), Some(bound)) = (range, self.bound_term(binding, value)) else {
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

    /// [ENT-3] S7: `iadd.checked<T>(p, k)` and `isub.checked<T>(p, k)` with a
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
pub(super) fn comparison_relation(
    operation: CheckedIntegerOperation,
    left: TermId,
    right: TermId,
) -> Option<Relation> {
    Some(match operation {
        CheckedIntegerOperation::Equal => Relation::Equal { left, right },
        CheckedIntegerOperation::NotEqual => Relation::Distinct { left, right },
        CheckedIntegerOperation::Less => Relation::Bound {
            left,
            right,
            bound: -1,
        },
        CheckedIntegerOperation::LessEqual => Relation::Bound {
            left,
            right,
            bound: 0,
        },
        CheckedIntegerOperation::Greater => Relation::Bound {
            left: right,
            right: left,
            bound: -1,
        },
        CheckedIntegerOperation::GreaterEqual => Relation::Bound {
            left: right,
            right: left,
            bound: 0,
        },
        _ => return None,
    })
}
