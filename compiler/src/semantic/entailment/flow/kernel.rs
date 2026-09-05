//! [CALL-6] publication over a [BLK-0] kernel-domain row.
//!
//! A row's declared relations are published exactly as a source [FN-9]
//! relation set is: instantiated at the call and established on its
//! continuation, with each operand substituted at the denotation [MSR-3]'s
//! table gives its parameter's mode. What differs is only the source of the
//! relation list — a compiler-owned record rather than a verified body — and
//! [ENT-3.S13] states that difference: a row's relations are declaration data
//! and therefore need no earlier-component verification premise.
//!
//! Every relation of the inventory is one ordinary difference bound between
//! two terms [BLK-0], so the establishment here is `Relation::Bound` and
//! nothing wider. A relation whose displacement does not resolve to a
//! constant at this instance is simply unavailable, which only under-derives
//! [ENT-1].

use super::super::super::kernel::{
    KernelOffset, KernelOperand, KernelPlace, KernelRoute, KernelSignature, kernel_signature,
};
use super::super::super::model::CheckedConst;
use super::super::super::model::{
    BindingId, CheckedConstructor, CheckedEnumType, CheckedExpression, CheckedKernelInstance,
    CheckedMatchArm, CheckedMeasure, CheckedNominalKind, CheckedType,
};
use super::super::RelationProvenance;
use super::super::state::{FactState, PostconditionCallDetail, Relation};
use super::super::term::{PlaceRoot, TermId, TermKind};
use super::super::{DerivationRootKind, VerifiedPostconditionSummaryRef};
use super::{Analyzer, GoalProjection, PreparedCall, PreparedCallee};

/// The prelude ordinal of `Option`'s `None` variant.
const NONE_VARIANT: u8 = 5;

/// The prelude ordinal of `Option`'s `Some` variant.
const SOME_VARIANT: u8 = 6;

/// One declared exit of a row, as the caller reaches it [CALL-6].
///
/// An exit is either the destination list a binder or target list gives an
/// unrouted result, or one arm of a `match` over a routed one. The two carry
/// the same relation list filtered two ways, which is why they share one
/// establishment path: an unrouted relation is a member of every exit's set,
/// and a routed one only of the arm its route names.
struct KernelExit<'a> {
    /// The route this exit selects, absent where the exit enters no arm.
    route: Option<KernelRoute>,
    /// The payload binder the route's variant carries, and its type.
    payload: Option<(BindingId, CheckedType)>,
    /// Destination i takes declared result ordinal i [CALL-4].
    destinations: &'a [Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
}

/// One kernel-domain call, read off the checked tree.
struct KernelCallSite<'a> {
    operation: u8,
    signature: &'static KernelSignature,
    instance: &'a CheckedKernelInstance,
    call: &'a crate::NodePath,
    goal_arguments: &'a [super::super::super::goal::GoalExpression],
}

/// The kernel-domain call one checked expression is, when it is one.
fn kernel_call_site(expression: &CheckedExpression) -> Option<KernelCallSite<'_>> {
    let CheckedExpression::KernelCall {
        operation,
        row,
        call,
        instance,
        goal_arguments,
        ..
    } = expression
    else {
        return None;
    };
    Some(KernelCallSite {
        operation: *operation,
        signature: kernel_signature(*row),
        instance,
        call,
        goal_arguments,
    })
}

impl Analyzer<'_, '_> {
    /// [ENT-3.S13, MSR-3] mints, at one kernel-domain call's pre-transfer
    /// point, the call datum of every operand any declared relation of the
    /// row names, and establishes it equal to that operand's pre-transfer
    /// term.
    ///
    /// The row additionally mints one datum for each `&uniq` state operand
    /// its relations name in the `at the call` form, on exactly the same
    /// terms and at the same point.
    pub(super) fn establish_kernel_call_datums(
        &mut self,
        expression: &CheckedExpression,
        state: &mut FactState,
    ) {
        let Some(site) = kernel_call_site(expression) else {
            return;
        };
        let mut operands: Vec<(u32, Option<CheckedMeasure>)> = Vec::new();
        for relation in site.signature.ensures.iter().chain(site.signature.requires) {
            for term in [relation.left, relation.right] {
                match term.operand {
                    KernelOperand::Value(ordinal) => operands.push((ordinal, None)),
                    KernelOperand::Measure(measure, KernelPlace::Parameter(ordinal)) => {
                        // An `own` operand denotes this call's call datum; a
                        // `&uniq` state operand's plain occurrence is the
                        // post-state and mints none [MSR-3].
                        if site.signature.parameters.get(ordinal as usize).is_some_and(
                            |parameter| {
                                parameter.mode == super::super::super::kernel::KernelMode::Own
                            },
                        ) {
                            operands.push((ordinal, Some(measure)));
                        }
                    }
                    KernelOperand::Measure(measure, KernelPlace::ParameterAtCall(ordinal)) => {
                        operands.push((ordinal, Some(measure)));
                    }
                    KernelOperand::Measure(..)
                    | KernelOperand::Const(_)
                    | KernelOperand::AlignCeiling
                    | KernelOperand::Zero => {}
                }
                if let KernelOffset::Advance(ordinal) = term.offset {
                    operands.push((ordinal, None));
                }
            }
        }
        let event = self.proof_event(super::FlowEventKind::S13, Some(site.call));
        let call = site.call.clone();
        for (ordinal, measure) in operands {
            let Some(parameter) = site.signature.parameters.get(ordinal as usize) else {
                continue;
            };
            let Some(actual) = site.goal_arguments.get(ordinal as usize) else {
                continue;
            };
            let ty = actual.ty();
            let Some(datum_type) = (if measure.is_some() {
                Some(super::super::super::model::IntegerType::U64)
            } else {
                super::fragment_type(ty)
            }) else {
                continue;
            };
            let kind = Self::call_datum_kind(&call, ordinal, &[], measure, datum_type);
            if self.terms.interned(&kind).is_some() {
                continue;
            }
            let _ = parameter;
            let Some(term) = self.kernel_operand_term(actual, ty, measure) else {
                continue;
            };
            if self.immortal_term(term) {
                continue;
            }
            let datum = self.terms.intern(kind);
            self.adopt_measure_atom(datum, term);
            state.establish(
                &Relation::Equal {
                    left: datum,
                    right: term,
                    difference: 0,
                },
                &mut self.derivations,
                event,
            );
        }
    }

    /// [CALL-6] establishes one row's declared relations at the destinations
    /// [CALL-4] fixes: ordinal i lands at destination i.
    ///
    /// A binder or target list enters no arm, so this exit carries the row's
    /// unrouted relations and none of its routed ones.
    pub(super) fn establish_kernel_relations(
        &mut self,
        statement: &crate::NodePath,
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
        value: &CheckedExpression,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        self.establish_kernel_exit(
            statement,
            value,
            prepared,
            &KernelExit {
                route: None,
                payload: None,
                destinations,
            },
            state,
        );
    }

    /// [CALL-6, ENT-3.S13] establishes, on one arm of a `match` over a
    /// kernel-domain call, the relations that arm's route selects.
    ///
    /// A row whose declared result is an enum states part of its relation
    /// list per variant [BLK-0], and publication restricts each routed
    /// relation to the arm its route names while an unrouted one is a member
    /// of every arm's set. The payload binder is what the route's own binder
    /// spelling denotes — `when made is Some(value: r)` names `r`, and the
    /// arm binds it — so a measure of the payload is a measure of that
    /// binding's place.
    pub(super) fn establish_kernel_match_relations(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        arm: &CheckedMatchArm,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let Some(site) = kernel_call_site(scrutinee) else {
            return;
        };
        let Some(route) = self.kernel_arm_route(enum_type, arm.tag) else {
            return;
        };
        let payload = arm
            .binders
            .iter()
            .find(|binder| binder.field == 0)
            .map(|binder| (binder.binding, binder.ty));
        let call = site.call.clone();
        self.establish_kernel_exit(
            &call,
            scrutinee,
            prepared,
            &KernelExit {
                route: Some(route),
                payload,
                destinations: &[],
            },
            state,
        );
    }

    /// The route one arm of a `match` over an `Option`-shaped kernel result
    /// selects, read off the arm's own variant rather than off its tag.
    fn kernel_arm_route(&self, enum_type: CheckedEnumType, tag: u32) -> Option<KernelRoute> {
        let CheckedEnumType::Nominal(nominal) = enum_type else {
            return None;
        };
        let CheckedNominalKind::Enum { variants } =
            &self.context.nominals.get(nominal.0 as usize)?.kind
        else {
            return None;
        };
        let variant = variants.iter().find(|variant| variant.tag == tag)?;
        let CheckedConstructor::Prelude(declaration) = variant.constructor else {
            return None;
        };
        // The two variants of the prelude `Option` a routed row writes its
        // clauses over [BLK-0].
        if declaration == crate::PreludeDeclarationId::new(SOME_VARIANT) {
            Some(KernelRoute::Some)
        } else if declaration == crate::PreludeDeclarationId::new(NONE_VARIANT) {
            Some(KernelRoute::None)
        } else {
            None
        }
    }

    /// One row's relations at one declared exit.
    ///
    /// The exit's relations are established in two passes because [BLK-0]
    /// judges them in two. The **unrouted** relations are a member of every
    /// exit's set, so they hold wherever the call's continuation is reached
    /// at all: a caller state that turns contradictory across them turned so
    /// on the row's own relations, and that is the judgment [BLK-0] carries
    /// from [CALL-6] onto a compiler-owned row — the set a row publishes may
    /// narrow what its caller derives and may never make everything
    /// derivable. Every requirement of the row is discharged before this
    /// point, a call whose requirement is undischarged preparing and
    /// publishing nothing, so nothing else can be the cause.
    ///
    /// The **routed** relations are the arm's own, and a contradiction across
    /// them is not a defect: it is the ordinary [ENT-3] statement that this
    /// arm is not reached, which a written `if` guard the caller can refute
    /// produces in exactly the same way. `arena_vector` asked for more bytes
    /// than the extent holds publishes `len_of(store) = len_of(store at the
    /// call) + advance<T>(count)` on its `Some` arm against a `cap_of(store)`
    /// that cannot hold it, and the arm it makes underivable is the arm that
    /// never runs. The exits of one call partition its outcomes, so at most
    /// one of them can be refuted this way.
    ///
    /// What is checked on every exit instead is the denotation [MSR-3] fixes,
    /// asserted below over the terms this instantiation actually formed.
    fn establish_kernel_exit(
        &mut self,
        statement: &crate::NodePath,
        value: &CheckedExpression,
        prepared: &PreparedCall,
        exit: &KernelExit<'_>,
        state: &mut FactState,
    ) {
        let Some(site) = kernel_call_site(value) else {
            return;
        };
        if PreparedCallee::Kernel(site.operation) != prepared.callee || *site.call != prepared.call
        {
            return;
        }
        let operation = site.operation;
        let signature = site.signature;
        let instance = site.instance.clone();
        let call = site.call.clone();
        let goal_arguments = site.goal_arguments.to_vec();
        let destinations = exit.destinations.to_vec();
        let Some(anchor) = self.kernel_exit_anchor(exit, &destinations, &goal_arguments) else {
            return;
        };
        self.assert_kernel_denotations(signature, exit, &call, &goal_arguments);
        let already_contradictory = self.state_is_contradictory(state);
        for shared in [true, false] {
            for (ordinal, relation) in signature.ensures.iter().enumerate() {
                // A routed relation is restricted to its own arm and an
                // unrouted one is a member of every arm's set [CALL-6].
                if relation.route.is_none() != shared
                    || (relation.route.is_some() && relation.route != exit.route)
                {
                    continue;
                }
                let displacement = |offset: KernelOffset| match offset {
                    KernelOffset::Constant(value) => Some(i128::from(value)),
                    KernelOffset::Advance(ordinal) => super::super::super::kernel::kernel_advance(
                        &instance,
                        &goal_arguments,
                        ordinal,
                    ),
                };
                let Some(bounds) = relation.bounds(displacement) else {
                    continue;
                };
                for bound in bounds {
                    let Some(left) = self.kernel_relation_term(
                        bound.left,
                        &instance,
                        signature,
                        &call,
                        &goal_arguments,
                        &destinations,
                        exit.payload,
                    ) else {
                        continue;
                    };
                    let Some(right) = self.kernel_relation_term(
                        bound.right,
                        &instance,
                        signature,
                        &call,
                        &goal_arguments,
                        &destinations,
                        exit.payload,
                    ) else {
                        continue;
                    };
                    let established = Relation::Bound {
                        left,
                        right,
                        bound: bound.bound,
                    };
                    self.retain_kernel_relation(
                        statement,
                        anchor,
                        operation,
                        u32::try_from(ordinal).unwrap_or(u32::MAX),
                        &call,
                        &established,
                        prepared,
                        state,
                    );
                }
            }
            if shared {
                assert!(
                    already_contradictory || !self.state_is_contradictory(state),
                    "[BLK-0, CALL-6] the relations kernel row {operation} publishes on every \
                     exit at {call:?} make the caller's fact state contradictory, so every \
                     relation and both signs of every goal become derivable there and every \
                     obligation the caller submits after it is discharged; a row's declared \
                     relation set may narrow what a caller derives and may never introduce a \
                     contradiction"
                );
            }
        }
    }

    /// [MSR-3, BLK-0] a measure's `at the call` form and its post-state
    /// occurrence are two terms wherever both occur.
    ///
    /// This is asserted over the terms the instantiation formed rather than
    /// over the record, because the record already keys the two apart and the
    /// defect this closes was the caller reading one term for both: the row's
    /// own `len_of(store) = len_of(store at the call) + advance<T>(count)`
    /// then becomes `t = t + advance<T>(count)` over one term, which is the
    /// bound pair `advance<T>(count) <= 0` and `advance<T>(count) >= 0` and,
    /// at any nonzero take, a contradiction the row introduces into every
    /// caller.
    fn assert_kernel_denotations(
        &mut self,
        signature: &KernelSignature,
        exit: &KernelExit<'_>,
        call: &crate::NodePath,
        goal_arguments: &[super::super::super::goal::GoalExpression],
    ) {
        let carried = |relation: &&super::super::super::kernel::KernelRelation| {
            relation.route.is_none() || relation.route == exit.route
        };
        let mut pairs: Vec<(CheckedMeasure, u32)> = Vec::new();
        for relation in signature.ensures.iter().filter(carried) {
            for term in [relation.left, relation.right] {
                if let KernelOperand::Measure(measure, KernelPlace::ParameterAtCall(ordinal)) =
                    term.operand
                {
                    pairs.push((measure, ordinal));
                }
            }
        }
        pairs.dedup();
        for (measure, ordinal) in pairs {
            let Some(actual) = goal_arguments.get(ordinal as usize) else {
                continue;
            };
            let ty = actual.ty();
            let Some(datum) = self.interned_call_datum(call, ordinal, &[], Some(measure), ty)
            else {
                continue;
            };
            let Some(live) = self.kernel_operand_term(actual, ty, Some(measure)) else {
                continue;
            };
            assert_ne!(
                datum, live,
                "[MSR-3, BLK-0] kernel row {:?} names {measure:?} of formal {ordinal} at {call:?} \
                 both at the call and in its post-state, and the two denote one term",
                signature.row
            );
        }
    }

    /// The binding one exit's retained relations are anchored at.
    ///
    /// The anchor is derivation identity and nothing else: a retained
    /// relation records where in the caller it landed [DIAG-2]. A destination
    /// list lands at its first destination; an arm lands at its payload
    /// binder where the route carries one, and otherwise at the root of the
    /// first operand place the call names, which is the store whose measures
    /// a payload-free arm relates.
    fn kernel_exit_anchor(
        &self,
        exit: &KernelExit<'_>,
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
        goal_arguments: &[super::super::super::goal::GoalExpression],
    ) -> Option<BindingId> {
        if let Some((binding, _)) = exit.payload {
            return Some(binding);
        }
        if let Some((binding, _, _)) = destinations.iter().flatten().next() {
            return Some(*binding);
        }
        exit.route?;
        goal_arguments.iter().find_map(|argument| {
            let super::super::super::goal::GoalExpression::Datum(
                super::super::super::goal::GoalDatum::Place { root, .. },
            ) = argument
            else {
                return None;
            };
            Some(*root)
        })
    }

    /// Whether the caller's fact state discharges everything at this point,
    /// under [ENT-4]'s least closure and without building a derivation.
    fn state_is_contradictory(&self, state: &FactState) -> bool {
        state.all_derivable
            || super::super::state::contradiction_without_proofs(state, &self.terms, &self.goals)
    }

    /// One published relation, retained with its own provenance and
    /// established on the call's normal continuation.
    #[allow(clippy::too_many_arguments)]
    fn retain_kernel_relation(
        &mut self,
        statement: &crate::NodePath,
        binding: BindingId,
        operation: u8,
        relation_ordinal: u32,
        call: &crate::NodePath,
        relation: &Relation,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let source =
            self.derivations
                .intern(super::super::state::DerivationNode::PostconditionCall {
                    detail: Box::new(PostconditionCallDetail {
                        call: call.clone(),
                        relation: relation.clone(),
                        summary: VerifiedPostconditionSummaryRef {
                            summary: RelationProvenance::Kernel {
                                operation,
                                relation_ordinal,
                            },
                        },
                        substitutions: Vec::new(),
                        transfer_events: prepared.transfer_events.clone(),
                        parents: prepared.parents.clone(),
                    }),
                });
        let route = self.derivations.intern(
            super::super::state::DerivationNode::PostconditionDirectResult {
                statement: statement.clone(),
                binding,
                relation: Box::new(relation.clone()),
                parent: source,
            },
        );
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectResult { occurrence },
            route,
        );
        state.establish_from_proof(relation, route, &self.derivations);
    }

    /// One record operand as an [ENT-2] term at this call.
    #[allow(clippy::too_many_arguments)]
    fn kernel_relation_term(
        &mut self,
        operand: KernelOperand,
        instance: &CheckedKernelInstance,
        signature: &KernelSignature,
        call: &crate::NodePath,
        goal_arguments: &[super::super::super::goal::GoalExpression],
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
        payload: Option<(BindingId, CheckedType)>,
    ) -> Option<TermId> {
        match operand {
            KernelOperand::Zero => Some(super::super::term::ZERO),
            KernelOperand::AlignCeiling => Some(self.terms.intern(TermKind::Constant(i128::from(
                instance.element_ceiling.align,
            )))),
            KernelOperand::Const(which) => match kernel_constant(instance, which)? {
                CheckedConst::Value(value) => {
                    Some(self.terms.intern(TermKind::Constant(i128::from(value))))
                }
                CheckedConst::Parameter(declaration) => {
                    Some(self.terms.intern(TermKind::ConstParameter(declaration)))
                }
                CheckedConst::Derived(_) => None,
            },
            KernelOperand::Value(ordinal) => {
                let actual = goal_arguments.get(ordinal as usize)?;
                self.kernel_call_operand_term(call, ordinal, None, false, actual, signature)
            }
            KernelOperand::Measure(measure, place) => match place {
                KernelPlace::Parameter(ordinal) | KernelPlace::ParameterAtCall(ordinal) => {
                    let actual = goal_arguments.get(ordinal as usize)?;
                    let at_call = matches!(place, KernelPlace::ParameterAtCall(_));
                    self.kernel_call_operand_term(
                        call,
                        ordinal,
                        Some(measure),
                        at_call,
                        actual,
                        signature,
                    )
                }
                KernelPlace::Result(ordinal) => {
                    let (binding, projections, ty) =
                        destinations.get(ordinal as usize)?.as_ref()?;
                    self.postcondition_measure_term(
                        measure,
                        PlaceRoot::Binding(*binding),
                        projections,
                        *ty,
                    )
                }
                // The payload binder the route's variant carries; a
                // destination list enters no arm and binds none.
                KernelPlace::Payload => {
                    let (binding, ty) = payload?;
                    self.postcondition_measure_term(measure, PlaceRoot::Binding(binding), &[], ty)
                }
            },
        }
    }

    /// One operand's term at the caller, at the denotation [MSR-3]'s table
    /// gives the operand position.
    ///
    /// A call datum and a post-state occurrence of one measure of one place
    /// are two different terms, and [ENT-2] clause (h) keys a call datum on
    /// the call, the ordinal, the projections and the measure and on nothing
    /// else — so the position decides which of the two this operand is
    /// before the datum table is consulted, and never after. An `own`
    /// operand and the `at the call` form both denote this call's call datum
    /// [BLK-0]; the plain occurrence of a `&uniq` state operand denotes the
    /// live term after the call's own kills, which is a different term even
    /// though its place, its measure and its ordinal are the same. Reading
    /// the datum for that position would make a row's own `len_of(store) =
    /// len_of(store at the call) + advance<T>(count)` the bound
    /// `0 <= -advance<T>(count)` over one term, which is a contradiction the
    /// row would introduce into the caller's state.
    fn kernel_call_operand_term(
        &mut self,
        call: &crate::NodePath,
        ordinal: u32,
        measure: Option<CheckedMeasure>,
        at_call: bool,
        actual: &super::super::super::goal::GoalExpression,
        signature: &KernelSignature,
    ) -> Option<TermId> {
        let ty = actual.ty();
        let denotes_datum = at_call
            || signature
                .parameters
                .get(ordinal as usize)
                .is_some_and(|parameter| {
                    parameter.mode == super::super::super::kernel::KernelMode::Own
                });
        if denotes_datum
            && let Some(datum) = self.interned_call_datum(call, ordinal, &[], measure, ty)
        {
            return Some(datum);
        }
        self.kernel_operand_term(actual, ty, measure)
    }

    /// The term one already-captured caller image denotes, as a value or as
    /// one of its [MSR-1] measures.
    fn kernel_operand_term(
        &mut self,
        actual: &super::super::super::goal::GoalExpression,
        ty: CheckedType,
        measure: Option<CheckedMeasure>,
    ) -> Option<TermId> {
        match measure {
            None => self.goal_operand(actual),
            Some(measure) => {
                let super::super::super::goal::GoalExpression::Datum(datum) = actual else {
                    return None;
                };
                let path = self.goal_place_path(datum)?;
                let measured = super::measured_kind(ty)?;
                let constant = match ty {
                    CheckedType::FixedVector { length, .. } => Some(length),
                    CheckedType::Extent { bytes, .. } => Some(bytes),
                    _ => None,
                };
                Some(self.measure_term(measure, path, measured, constant))
            }
        }
    }
}

/// The constant one written const parameter of the row takes at this
/// instance.
const fn kernel_constant(
    instance: &CheckedKernelInstance,
    which: super::super::super::kernel::KernelConst,
) -> Option<CheckedConst> {
    match which {
        super::super::super::kernel::KernelConst::Capacity => instance.capacity,
        super::super::super::kernel::KernelConst::Bytes => instance.bytes,
        super::super::super::kernel::KernelConst::Align => instance.align,
    }
}
