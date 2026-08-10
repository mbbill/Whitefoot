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
//! Implemented here: S1 (through the parent's arm entry), S2 and S3 (one
//! shared clause: a passed `check` or `claim` condition), S4, S5, S6, S7,
//! S9, and S10. The label S8 is retired, not reused [ENT-3].

use super::super::super::model::{
    BindingId, CheckedArrayRoot, CheckedEnumType, CheckedExpression, CheckedIntegerOperation,
    CheckedMatchArm, CheckedNominalKind, CheckedSliceSource, CheckedValue, IntegerType,
};
use super::super::fragment_type;
use super::super::state::{FactState, OutcomeFact, OutcomeRelation, Relation, close};
use super::super::term::{
    CountedCaptureSide, PlaceRoot, PlaceTerm, TermId, TermKind, ZERO, integer_value, type_range,
};
use super::{Analyzer, ArmFacts};
use crate::SYSTEM_OPERATIONS;

/// The [SYS-2] operations whose outcome carries an [ENT-3] S10 count bound,
/// with the name of the bounding parameter and of the observing variant. The
/// bounding actual is found by parameter name in the catalog row, never by a
/// hardcoded position.
const BOUNDARY_COUNTS: [(&str, &str, &str); 4] = [
    ("read_once", "capacity", "ReadBytes"),
    ("write_once", "count", "Ok"),
    ("host_copy_bytes", "capacity", "Ok"),
    ("host_copy_utf8", "capacity", "Ok"),
];

/// The three S11 terms installed for one counted range.
pub(super) struct CountedTerms {
    pub(super) lower: TermId,
    pub(super) binder: TermId,
    pub(super) upper: TermId,
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
        state.establish(&Relation::Equal {
            left: lower_capture,
            right: lower_source,
        });
        state.establish(&Relation::Equal {
            left: upper_capture,
            right: upper_source,
        });
        state.establish(&Relation::Equal {
            left: binder,
            right: lower_capture,
        });
        CountedTerms {
            lower: lower_capture,
            binder,
            upper: upper_capture,
        }
    }

    /// Adds exactly S11's two facts on an executed true header edge.
    pub(super) fn establish_counted_body_entry(
        &self,
        counted: &CountedTerms,
        state: &mut FactState,
    ) {
        state.establish(&Relation::Bound {
            left: counted.lower,
            right: counted.binder,
            bound: 0,
        });
        state.establish(&Relation::Bound {
            left: counted.binder,
            right: counted.upper,
            bound: -1,
        });
    }

    // ------------------------------------------------------------------
    // S2 check facts and S3 claim facts
    // ------------------------------------------------------------------

    /// [ENT-3] S2/S3: after `check e else trap "…"` or `claim n: e because
    /// "…"` whose `e` has comparison origin R, R holds on the normal
    /// continuation.
    pub(super) fn establish_passed_condition(
        &mut self,
        condition: &CheckedExpression,
        state: &mut FactState,
    ) {
        let goals = self.goal_origin_set(condition, state);
        for goal in goals {
            state.establish_goal(goal, super::super::state::GoalSign::Positive);
        }
        if let Some(relation) = self.scrutinee_relation(condition, state) {
            state.establish(&relation);
        }
    }

    // ------------------------------------------------------------------
    // S4 requires facts
    // ------------------------------------------------------------------

    /// [ENT-3] S4: the complete concrete body goal enters as a positive opaque
    /// fact, with its one exact comparison-root projection when present.
    pub(super) fn establish_requires_facts(&mut self, state: &mut FactState) {
        let Some(goal) = self.body_requirement_goal() else {
            return;
        };
        let goal = self.intern_goal_expression(goal);
        state.establish_goal(goal, super::super::state::GoalSign::Positive);
        if let Some(relation) = self.goals.projection(goal).cloned() {
            state.establish(&relation);
        }
    }

    // ------------------------------------------------------------------
    // Establishment at a `let` binding: S5, S6, S7, S9
    // ------------------------------------------------------------------

    /// Every source that establishes at an `ordinary_let_rhs` binding, in one
    /// place because they are mutually exclusive on the initializer's shape.
    pub(super) fn establish_binding_facts(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
    ) {
        if self.establish_length_facts(binding, value, state) {
            return;
        }
        if self.establish_element_range(binding, value, state) {
            return;
        }
        if self.establish_offset_fact(binding, value, state) {
            return;
        }
        self.record_outcome_origin(binding, value, state);
        self.establish_copy_fact(binding, value, state);
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
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
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
        state.establish(&Relation::Equal {
            left: bound,
            right: source,
        });
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
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
    ) -> bool {
        match value {
            CheckedExpression::BufferFill { length, .. } => {
                let Some(allocated) = self.read_operand(length) else {
                    return true;
                };
                let place = self.bound_place(binding);
                let length_term = self.length_term(place, None);
                state.establish(&Relation::Equal {
                    left: length_term,
                    right: allocated,
                });
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
                };
                let source_length = self.length_term(place, array_length);
                let slice_place = self.bound_place(binding);
                let slice_length = self.length_term(slice_place, None);
                state.establish(&Relation::Equal {
                    left: slice_length,
                    right: source_length,
                });
                true
            }
            _ => match self.length_operand(value) {
                Some(source_length) => {
                    if let Some(bound) = self.bound_term(binding, value) {
                        state.establish(&Relation::Equal {
                            left: bound,
                            right: source_length,
                        });
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
            CheckedExpression::ArrayLength { root, length } => {
                (self.array_root_place(root), Some(*length))
            }
            CheckedExpression::BufferLength { root } => (
                PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                },
                None,
            ),
            CheckedExpression::SliceLength { root } => (
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
    /// over the mathematical value the wrap did not reach. `iadd.trap` and
    /// `isub.trap` establish it unconditionally on the normal continuation:
    /// the executed contract check is the proof [OP-2].
    fn establish_offset_fact(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
    ) -> bool {
        let Some((base, delta, trapping)) = self.constant_offset(value) else {
            return false;
        };
        let Some(bound) = self.bound_term(binding, value) else {
            return true;
        };
        if !trapping {
            let Some(ty) = fragment_type(value.ty()) else {
                return true;
            };
            let (minimum, maximum) = type_range(ty);
            let closed = close(state, &self.terms, &self.goals);
            // `min(T) <= p + k` and `p + k <= max(T)`, as bounds on p through
            // Z: p - Z <= max(T) - k and Z - p <= k - min(T).
            let within = closed.derives_bound(base, ZERO, maximum.saturating_sub(delta))
                && closed.derives_bound(ZERO, base, delta.saturating_sub(minimum));
            if !within {
                return true;
            }
        }
        establish_shifted(state, bound, base, delta);
        true
    }

    /// The `(p, k, trapping)` reading of one constant-offset arithmetic call:
    /// `iadd` accepts its constant in either operand position, `isub` only as
    /// the subtrahend, since `k - p` is no offset of p.
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
        let (adding, trapping) = match operation {
            CheckedIntegerOperation::AddWrap => (true, false),
            CheckedIntegerOperation::AddTrap => (true, true),
            CheckedIntegerOperation::SubtractWrap => (false, false),
            CheckedIntegerOperation::SubtractTrap => (false, true),
            _ => return None,
        };
        let left = self.read_operand(left)?;
        let right = self.read_operand(right)?;
        let (base, delta) = self.split_offset(left, right, adding)?;
        Some((base, delta, trapping))
    }

    /// Splits one operand pair into a base term and a constant offset.
    fn split_offset(&self, left: TermId, right: TermId, adding: bool) -> Option<(TermId, i128)> {
        if let TermKind::Constant(value) = *self.terms.kind(right) {
            return Some((left, if adding { value } else { -value }));
        }
        if adding && let TermKind::Constant(value) = *self.terms.kind(left) {
            return Some((right, value));
        }
        None
    }

    /// [ENT-3] S9: `let x: own T = c[i];` where c is the bare IDENT of a
    /// named const of type `array<T, N>` and T a fragment type establishes
    /// vlo <= x and x <= vhi over its N declared element values. The index's
    /// own bounds obligation is judged separately and is unaffected. Deeper
    /// const shapes establish nothing.
    fn establish_element_range(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
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
        state.establish(&Relation::Bound {
            left: ZERO,
            right: bound,
            bound: -low,
        });
        state.establish(&Relation::Bound {
            left: bound,
            right: ZERO,
            bound: high,
        });
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
        state: &mut FactState,
    ) -> ArmFacts {
        self.expression_effects(scrutinee, state);
        if enum_type == CheckedEnumType::Bool {
            return ArmFacts {
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
    /// `Ok(value: w)` shift, or S10's count bound on the observing arm.
    fn outcome_fact(&mut self, value: &CheckedExpression) -> Option<OutcomeFact> {
        self.checked_offset_outcome(value)
            .or_else(|| self.boundary_count_outcome(value))
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
        })
    }

    /// [ENT-3] S10: a [SYS-2] transfer's observing arm binds a count that is
    /// at most the bounding actual k — `capacity` for `read_once`,
    /// `host_copy_bytes`, and `host_copy_utf8`, `count` for `write_once`.
    /// The fact carries the same trust class as S6's allocation-length
    /// equality: it is a declared operation contract, never a writer
    /// statement.
    ///
    /// The bound is admitted only where no kill event on the path to the
    /// match reaches a fact supported by k. The call's own boundary writes
    /// are on that path, so a k read through a place the call writes admits
    /// nothing — the conservative reading, which only under-derives.
    fn boundary_count_outcome(&mut self, value: &CheckedExpression) -> Option<OutcomeFact> {
        let CheckedExpression::SystemCall {
            operation,
            arguments,
            ..
        } = value
        else {
            return None;
        };
        let row = SYSTEM_OPERATIONS.get(usize::from(*operation))?;
        let (_, bounding, variant) = BOUNDARY_COUNTS
            .iter()
            .find(|(spelling, _, _)| *spelling == row.spelling)?;
        let position = row
            .parameters
            .iter()
            .position(|parameter| parameter.name == *bounding)?;
        let base = self.read_operand(arguments.get(position)?)?;
        let mut events = Vec::new();
        self.collect_expression_kills(value, &mut events);
        if events
            .iter()
            .any(|event| self.event_kills_term(base, event))
        {
            return None;
        }
        Some(OutcomeFact {
            variant,
            base,
            relation: OutcomeRelation::AtMost,
        })
    }

    /// Establishes one arm's binder fact at arm entry: the value binder of
    /// the observing variant gains the recorded relation against its base.
    pub(super) fn establish_binder_fact(
        &mut self,
        arm: &CheckedMatchArm,
        outcome: &OutcomeFact,
        state: &mut FactState,
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
                establish_shifted(state, bound, outcome.base, delta);
            }
            OutcomeRelation::AtMost => state.establish(&Relation::Bound {
                left: bound,
                right: outcome.base,
                bound: 0,
            }),
        }
    }
}

/// `bound = base + delta`, as the difference-bound pair over that term pair.
fn establish_shifted(state: &mut FactState, bound: TermId, base: TermId, delta: i128) {
    state.establish(&Relation::Bound {
        left: bound,
        right: base,
        bound: delta,
    });
    state.establish(&Relation::Bound {
        left: base,
        right: bound,
        bound: -delta,
    });
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
