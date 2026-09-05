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
    KernelOffset, KernelOperand, KernelPlace, KernelSignature, kernel_signature,
};
use super::super::super::model::CheckedConst;
use super::super::super::model::{
    BindingId, CheckedExpression, CheckedKernelInstance, CheckedMeasure, CheckedType,
};
use super::super::RelationProvenance;
use super::super::state::{FactState, PostconditionCallDetail, Relation};
use super::super::term::{PlaceRoot, TermId, TermKind};
use super::super::{DerivationRootKind, VerifiedPostconditionSummaryRef};
use super::{Analyzer, GoalProjection, PreparedCall, PreparedCallee};

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
    pub(super) fn establish_kernel_relations(
        &mut self,
        statement: &crate::NodePath,
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
        value: &CheckedExpression,
        prepared: &PreparedCall,
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
        let destinations = destinations.to_vec();
        let mut anchor = None;
        for (binding, _, _) in destinations.iter().flatten() {
            anchor.get_or_insert(*binding);
        }
        let Some(anchor) = anchor else {
            return;
        };
        for (ordinal, relation) in signature.ensures.iter().enumerate() {
            // A routed relation is restricted to its own arm [CALL-6]; the
            // destinations here enter no arm.
            if relation.route.is_some() {
                continue;
            }
            let displacement = |offset: KernelOffset| match offset {
                KernelOffset::Constant(value) => Some(i128::from(value)),
                KernelOffset::Advance(ordinal) => {
                    super::super::super::kernel::kernel_advance(&instance, &goal_arguments, ordinal)
                }
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
    fn kernel_relation_term(
        &mut self,
        operand: KernelOperand,
        instance: &CheckedKernelInstance,
        signature: &KernelSignature,
        call: &crate::NodePath,
        goal_arguments: &[super::super::super::goal::GoalExpression],
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
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
                self.kernel_call_operand_term(call, ordinal, None, actual, signature)
            }
            KernelOperand::Measure(measure, place) => match place {
                KernelPlace::Parameter(ordinal) | KernelPlace::ParameterAtCall(ordinal) => {
                    let actual = goal_arguments.get(ordinal as usize)?;
                    self.kernel_call_operand_term(call, ordinal, Some(measure), actual, signature)
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
                // A routed payload has no unrouted destination.
                KernelPlace::Payload => None,
            },
        }
    }

    /// One operand's term at the caller: this call's call datum where
    /// [ENT-3.S13] minted one, and the operand's live term otherwise.
    fn kernel_call_operand_term(
        &mut self,
        call: &crate::NodePath,
        ordinal: u32,
        measure: Option<CheckedMeasure>,
        actual: &super::super::super::goal::GoalExpression,
        signature: &KernelSignature,
    ) -> Option<TermId> {
        let ty = actual.ty();
        if let Some(datum) = self.interned_call_datum(call, ordinal, &[], measure, ty) {
            return Some(datum);
        }
        let _ = signature;
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
