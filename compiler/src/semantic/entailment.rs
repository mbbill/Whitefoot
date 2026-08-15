//! The combined entailment fragment [ENT-1..ENT-6]: a closed, deterministic,
//! search-free derivation system over L0 difference bounds and finite exact
//! signed goals.
//!
//! The engine is acceptance-bearing: [`analyze_function`] computes the
//! closed fact state along the [FN-1] structural graph, the [ENT-6]
//! disposition of every bounds obligation, the [FN-8] disposition of every
//! ordinary call requirement, and the [CLM-2] lifecycle disposition of every
//! claim. The checker rejects a function whose summary contains an
//! undischarged obligation or call goal, or a refuted claim, and retains the
//! complete summary on the checked function [DIAG-2].
//!
//! Judgments are per function body [ENT-2]; the [ENT-3] S4 `requires`
//! relation is the one fact that enters from outside the body, and no fact
//! crosses a call boundary.
//!
//! Implemented fact sources: S1 branch and match facts with both
//! comparison-origin shapes, S2 check facts, S3 claim facts, S4 requires
//! facts, S5 copy and conversion equalities, S6 length facts, S7
//! constant-offset arithmetic, S9 const-array element ranges, and S10
//! boundary count facts; the label S8 is retired, not reused [ENT-3]. An
//! absent source only under-derives, which is the version-monotone
//! direction [ENT-1].

mod flow;
mod state;
mod term;

#[cfg(not(test))]
use state::DerivationId;
pub(crate) use state::ProofView;
use state::{DerivationInventory, DerivationLedger};
#[cfg(not(test))]
use term::TermId;

#[cfg(test)]
pub(crate) use state::{
    CountedRootAtom, DerivationId, DerivationNode, DerivationRootKind, FlowEvent, FlowEventId,
    FlowEventKind, GoalId, GoalSign, ImplicitBoundKind, JoinParent, Relation,
};
#[cfg(test)]
pub(crate) use term::{
    CountedCaptureSide, LengthBound, PlaceProjection, PlaceRoot, TermId, TermKind, ZERO, type_range,
};

use std::collections::{BTreeSet, HashMap};

use super::goal::ConcreteGoal;
use super::model::{
    BindingId, CheckedConstant, CheckedConstantId, CheckedExpression, CheckedFunction, CheckedMode,
    CheckedNominal, CheckedSetTarget, CheckedStatement, CheckedType, FunctionId, IntegerType,
};
use super::postcondition::CheckedPostcondition;
use crate::{DeclarationId, NodePath};

/// Kill-relevant [EFF-2] projection of one callee signature: for each
/// parameter, whether the callee's declared effect row writes the region that
/// parameter carries, so a call kills exactly the facts whose support
/// overlaps that actual's resolved place [ENT-5](b).
#[derive(Clone, Debug, Default)]
pub(crate) struct EntailmentCallee {
    pub(crate) parameter_modes: Vec<CheckedMode>,
    pub(crate) parameter_writes: Vec<bool>,
}

impl EntailmentCallee {
    /// Derives the projection from one callee's parameter modes and declared
    /// `writes` regions. A row with no `writes` kills nothing; a written
    /// region reached only through a `&uniq` actual kills through exactly
    /// that actual. Slice element writes have no [SET-1] target form in the
    /// current compiler, so an owned slice parameter never projects a write.
    pub(crate) fn from_signature(
        parameters: impl Iterator<Item = CheckedMode>,
        writes: &[crate::DeclarationId],
    ) -> Self {
        let parameter_modes = parameters.collect::<Vec<_>>();
        Self {
            parameter_writes: parameter_modes
                .iter()
                .copied()
                .map(|mode| match mode {
                    CheckedMode::Unique(region) => writes.contains(&region),
                    CheckedMode::Own | CheckedMode::Shared(_) => false,
                })
                .collect(),
            parameter_modes,
        }
    }
}

/// Program-level context the per-function analysis reads.
pub(crate) struct EntailmentContext<'check> {
    /// Callee projections indexed by [`FunctionId`].
    pub(crate) callees: &'check [EntailmentCallee],
    pub(crate) constants: &'check [CheckedConstant],
    /// Source declaration identity to dense checked-constant identity. Goal
    /// equality keeps the former while L0 projection reads the latter's
    /// mathematical value.
    pub(crate) constant_ids: &'check HashMap<DeclarationId, CheckedConstantId>,
    pub(crate) nominals: &'check [CheckedNominal],
    /// Published earlier-component FN-9 declarations and proofs, indexed by
    /// concrete [`FunctionId`]. Same-component entries remain absent until
    /// the component's atomic publication boundary.
    pub(crate) verified_postconditions: &'check [Option<&'check CheckedPostcondition>],
    pub(crate) verified_postcondition_proofs:
        &'check [Option<&'check FunctionPostconditionProof>],
    /// Binding names in dense [`super::model::BindingId`] order, for the
    /// [ENT-6] canonical residual rendering.
    pub(crate) binding_names: &'check [String],
}

impl EntailmentContext<'_> {
    pub(crate) fn callee(&self, function: FunctionId) -> Option<&EntailmentCallee> {
        self.callees.get(function.0 as usize)
    }

    pub(crate) fn constant(&self, declaration: DeclarationId) -> Option<&CheckedConstant> {
        let id = self.constant_ids.get(&declaration)?;
        self.constants.get(id.0 as usize)
    }

    pub(crate) fn constant_declaration(
        &self,
        constant: CheckedConstantId,
    ) -> Option<DeclarationId> {
        self.constant_ids
            .iter()
            .find_map(|(declaration, id)| (*id == constant).then_some(*declaration))
    }

    pub(crate) fn verified_postcondition(
        &self,
        function: FunctionId,
    ) -> Option<(&CheckedPostcondition, &FunctionPostconditionProof)> {
        let postcondition = self
            .verified_postconditions
            .get(function.0 as usize)?
            .as_ref()
            .copied()?;
        let proof = self
            .verified_postcondition_proofs
            .get(function.0 as usize)?
            .as_ref()
            .copied()?;
        (proof.summary.as_ref()?.function == function).then_some((postcondition, proof))
    }
}

/// [ENT-6] disposition of one bounds obligation, judged at its source node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObligationOutcome {
    /// The subscript's `psuffix` node the obligation is attached to, by its
    /// trap record's path — one record per subscript in a chain [ENT-6].
    pub(crate) node_path: NodePath,
    /// The current bounds obligation has one upper-bound conjunct, numbered
    /// zero in the same source-subscript query namespace later tasks extend.
    pub(crate) conjunct: u8,
    /// Normalized `offset - len(base) <= -1`. `left` is absent only when the
    /// checked offset is outside ENT-2's term vocabulary; the exact checked
    /// expression remains recoverable from `node_path` in the same function.
    pub(crate) requested: BoundsRequest,
    /// The closed fact state at the node derives the normalized relation.
    pub(crate) discharged: bool,
    /// The state at the node was contradictory, discharging everything.
    pub(crate) contradictory: bool,
    /// The exact residual rendering for an undischarged obligation: the
    /// offset atom's canonical source bytes, ` < len(`, the base place's
    /// canonical source bytes, `)`.
    pub(crate) residual: Option<String>,
    /// Exact ENT-4 derivation for an accepted obligation. Failed judgments
    /// deliberately carry no positive root.
    pub(crate) derivation: Option<DerivationId>,
}

/// Exact normalized identity of one bounds query in the function-local term
/// inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoundsRequest {
    pub(crate) left: Option<TermId>,
    pub(crate) right: TermId,
    pub(crate) bound: i128,
}

/// The two exact S11 proof points retained for one counted statement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountedProofPoint {
    /// The complete post-capture closure, before continuing kills.
    PreheaderSnapshot,
    /// The executed true-header edge entering the counted body.
    BodyEntry,
}

/// One directed normalized bound and its exact parent in the sole
/// function-local derivation ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountedAtomicDerivation {
    pub(crate) relation: state::Relation,
    pub(crate) proof_point: CountedProofPoint,
    pub(crate) parent: DerivationId,
}

/// One normative S11 equality and both of its directed atomic bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountedEqualityDerivation {
    pub(crate) relation: state::Relation,
    pub(crate) forward: CountedAtomicDerivation,
    pub(crate) reverse: CountedAtomicDerivation,
}

/// One normative S11 ordering relation and its directed atomic bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountedBoundDerivation {
    pub(crate) relation: state::Relation,
    pub(crate) atomic: CountedAtomicDerivation,
}

/// The complete fixed S11 root group for one concrete counted statement.
///
/// Field order is normative S11 order: the two endpoint captures, binder
/// initialization, and the two true-header bounds. The three equalities each
/// retain both directions, for exactly eight atomic ledger roots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountedDerivationSet {
    pub(crate) counted_node_path: NodePath,
    pub(crate) lower_capture_eq_endpoint: CountedEqualityDerivation,
    pub(crate) upper_capture_eq_endpoint: CountedEqualityDerivation,
    pub(crate) binder_eq_lower_capture: CountedEqualityDerivation,
    pub(crate) lower_capture_le_binder: CountedBoundDerivation,
    pub(crate) binder_lt_upper_capture: CountedBoundDerivation,
}

/// The exact written mathematical-one identity admitted by S7. Generic
/// numeric identities and const-generic values deliberately have no member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShiftOneIdentity {
    TypedLiteral { source: NodePath },
    NamedConstant { declaration: DeclarationId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum S7DerivationKind {
    BitAndBound {
        operand: u8,
        admitted: TermId,
    },
    ShiftOneNonzero {
        count_atom: NodePath,
        one: ShiftOneIdentity,
    },
}

/// One required unused-or-consumed S7 source root in one proof view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct S7Derivation {
    pub(crate) source: NodePath,
    pub(crate) view: ProofView,
    pub(crate) row: IntegerType,
    pub(crate) binding: BindingId,
    pub(crate) kind: S7DerivationKind,
    pub(crate) relation: state::Relation,
    pub(crate) event: state::FlowEventId,
    pub(crate) parent: DerivationId,
}

/// The complete and exclusive FN-9 relation-query disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionDisposition {
    Discharged,
    Refuted,
    Unproved,
}

/// One exact entry-image datum referenced by the concrete relation template.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionEntryImage {
    pub(crate) parameter: u32,
    pub(crate) projections: Vec<super::goal::GoalProjection>,
    pub(crate) length: bool,
}

/// View-independent stability retained at one selected return. `None` is the
/// successful absence of an invalidating event, never a positive proof node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionEntryImageOutcome {
    pub(crate) datum: PostconditionEntryImage,
    pub(crate) invalidation: Option<state::FlowEventId>,
}

/// One view's judgment of one instantiated selected-return relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionViewExit {
    pub(crate) view: ProofView,
    pub(crate) disposition: PostconditionDisposition,
    pub(crate) derivation: Option<DerivationId>,
}

/// One source-ordered selected return and its fixed C/U/B judgments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionExit {
    pub(crate) statement: NodePath,
    pub(crate) relation: state::Relation,
    pub(crate) residual: String,
    pub(crate) entry_images: Vec<PostconditionEntryImageOutcome>,
    pub(crate) complete: PostconditionViewExit,
    pub(crate) unasserted: PostconditionViewExit,
    pub(crate) s4_blinded: PostconditionViewExit,
}

/// One view's nonempty all-exit aggregation. A failed view has no derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionAggregate {
    pub(crate) view: ProofView,
    pub(crate) discharged: bool,
    pub(crate) derivation: Option<DerivationId>,
}

/// The checked local FN-9 proof retained with one concrete function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionPostconditionProof {
    pub(crate) block: NodePath,
    pub(crate) selector: NodePath,
    /// Present only after the concrete-call SCC scheduler publishes every
    /// independently verified summary in this component atomically. This is
    /// checked-program-private identity; a caller never imports this proof's
    /// function-local derivation IDs.
    pub(crate) summary: Option<VerifiedPostconditionSummary>,
    pub(crate) exits: Vec<PostconditionExit>,
    pub(crate) complete: PostconditionAggregate,
    pub(crate) unasserted: PostconditionAggregate,
    pub(crate) s4_blinded: PostconditionAggregate,
}

/// One verified concrete FN-9 summary identity made referenceable by the SCC
/// schedule. The single relation ordinal is retained explicitly because the
/// occurrence identity is `(function, ensures block, 0)`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct VerifiedPostconditionSummary {
    pub(crate) function: FunctionId,
    pub(crate) block: NodePath,
    pub(crate) relation_ordinal: u32,
    pub(crate) component: u32,
}

/// Caller-local reference to one view of an earlier-component verified
/// summary. It intentionally carries no callee-local [`DerivationId`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct VerifiedPostconditionSummaryRef {
    pub(crate) summary: VerifiedPostconditionSummary,
    pub(crate) view: ProofView,
}

/// One concrete ordinary-call SCC in deterministic callee-before-caller
/// order. Function and summary identities are both dense-function ordered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionComponent {
    pub(crate) ordinal: u32,
    pub(crate) functions: Vec<FunctionId>,
    pub(crate) summaries: Vec<VerifiedPostconditionSummary>,
}

/// Program-private SCC schedule retained for the later caller-publication
/// handoff. An empty schedule is the no-postcondition fast path.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PostconditionSchedule {
    pub(crate) components: Vec<PostconditionComponent>,
}

/// [CLM-2] lifecycle disposition of one claim, judged at its statement node
/// with the fact state before the claim's own passed fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClaimDisposition {
    /// The predicate has no comparison origin, or the state derives neither
    /// it nor its negation: an ordinary retained runtime check.
    Retained,
    /// The closed state derives the predicate: accepted, still executed,
    /// reported through the required non-rejecting advisory.
    Redundant,
    /// The non-contradictory closed state derives the exact negation: a
    /// compile-time rejection citing CLM-2.
    Refuted {
        /// The predicate as a normalized relation.
        predicate: String,
        /// The derived negation.
        negation: String,
    },
}

/// [CLM-2] outcome of one claim statement, judged at its node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimOutcome {
    /// The `claim_stmt` node, by its trap record's path.
    pub(crate) node_path: NodePath,
    /// The claim's written name.
    pub(crate) name: String,
    pub(crate) disposition: ClaimDisposition,
}

/// Complete [FN-8] disposition of one ordinary call requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallGoalDisposition {
    Discharged,
    Refuted,
    Unproved,
}

/// Every direct derivation ground retained for one call judgment, in the
/// fixed order documented on [`CallGoalOutcome::evidence`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallGoalEvidence {
    AllDerivable,
    OpaquePositive,
    ExactL0Projection,
    OpaqueNegative,
    NegatedL0Projection,
}

/// Retained checked metadata for one ordinary call carrying a requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallGoalOutcome {
    /// Exact source `call` occurrence.
    pub(crate) node_path: NodePath,
    pub(crate) callee: FunctionId,
    /// Exact final-check occurrence in the concrete callee.
    pub(crate) final_check: NodePath,
    pub(crate) goal: ConcreteGoal,
    pub(crate) disposition: CallGoalDisposition,
    /// Deterministic complete evidence. Contradictory states retain only
    /// `AllDerivable`; positive opaque and projection grounds follow in that
    /// order, as do negative opaque and negated-projection grounds.
    pub(crate) evidence: Vec<CallGoalEvidence>,
    /// One exact positive or contradiction root for a discharged call.
    /// Refuted and unproved calls carry none.
    pub(crate) derivation: Option<DerivationId>,
}

/// One metadata-only rejudgment of an ordinary call's complete goal.
///
/// This is deliberately not a [`CallGoalOutcome`]: the caller's actual
/// expressions may contain an obligation that the counterfactual state does
/// not discharge.  `goal_disposition` answers only the isolated FN-8 goal
/// question after those actuals have been walked.  Full-state analysis remains
/// the sole source-acceptance judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallGoalCounterfactual {
    pub(crate) node_path: NodePath,
    pub(crate) callee: FunctionId,
    pub(crate) final_check: NodePath,
    pub(crate) goal: ConcreteGoal,
    pub(crate) actual_obligations_ok: bool,
    pub(crate) goal_disposition: CallGoalDisposition,
    pub(crate) goal_evidence: Vec<CallGoalEvidence>,
    /// Exact same-view positive or contradiction proof, retained only when a
    /// caller-local S12 root reaches it.
    pub(crate) derivation: Option<DerivationId>,
}

/// One bounds result retained from a non-complete ENT proof view [ENT-6].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ViewObligationOutcome {
    /// The exact protected subscript occurrence consumed by provenance.
    pub(crate) node_path: NodePath,
    /// Whether the selected counterfactual fact sources discharge it.
    pub(crate) discharged: bool,
    /// The ordinary canonical residual when it remains undischarged.
    pub(crate) residual: Option<String>,
    /// Exact same-view proof, retained only through a required caller-local
    /// postcondition root.
    pub(crate) derivation: Option<DerivationId>,
}

/// One non-complete proof view produced by the shared structural ENT flow.
///
/// This metadata deliberately strips dense term and derivation IDs until a
/// required root retains them. The shared structural walk produces it beside
/// the complete view.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionEntailmentView {
    /// Every protected leaf under the selected counterfactual fact sources.
    pub(crate) obligations: Vec<ViewObligationOutcome>,
    /// Isolated call-goal results, explicitly separated from actual validity.
    pub(crate) call_goals: Vec<CallGoalCounterfactual>,
}

/// Retained summary of one function's entailment analysis [DIAG-2].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionEntailment {
    /// Bounds obligations in deterministic source walk order.
    pub(crate) obligations: Vec<ObligationOutcome>,
    /// Claim lifecycle outcomes in deterministic source walk order.
    pub(crate) claims: Vec<ClaimOutcome>,
    /// Ordinary call-goal judgments in deterministic checked-tree walk order.
    pub(crate) call_goals: Vec<CallGoalOutcome>,
    /// S2/S3-disabled judgments with body-entry S4 retained.
    pub(crate) unasserted: FunctionEntailmentView,
    /// The unasserted view with S4 and its exact projection omitted.
    pub(crate) s4_blinded: FunctionEntailmentView,
    /// One complete five-relation/eight-atomic S11 group per counted
    /// statement, in deterministic statement-walk order.
    pub(crate) counted_derivations: Vec<CountedDerivationSet>,
    /// Every admitted S7 relation, in structural source / C-U-B view /
    /// operand order. Each entry owns one required source root.
    pub(crate) s7_derivations: Vec<S7Derivation>,
    /// Present exactly for a concrete function carrying an FN-9 declaration.
    pub(crate) postcondition: Option<FunctionPostconditionProof>,
    /// Function-local, lifetime-bound derivations for mandatory DIAG-2 roots.
    pub(crate) derivations: DerivationLedger,
    /// Canonical term and goal identities moved from the analyzer so every
    /// retained dense ID remains exact and interpretable after analysis.
    pub(crate) inventory: DerivationInventory,
}

/// Computes the combined entailment analysis of one checked function body.
///
/// The analysis is total: it never rejects, never reports unsupported, and
/// never fails compilation. A body shape outside the engine's current
/// vocabulary contributes no facts, which only under-derives.
pub(crate) fn analyze_function(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    flow::analyze(function, context)
}

/// Computes one optimistic FN-9 function batch without pruning its shared
/// derivation ledger. Program provenance decides whether this candidate batch
/// is discarded or finalized unchanged.
pub(crate) fn analyze_function_candidate(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    flow::analyze_candidate(function, context)
}

/// Performs the sole root retention and dense-ID remap for one accepted
/// optimistic function batch.
pub(crate) fn finalize_function_entailment(entailment: &mut FunctionEntailment) {
    flow::finish(entailment);
}

/// Builds the concrete ordinary-call SCC schedule used to make verified FN-9
/// summaries referenceable. The schedule is absent when the unit declares no
/// postcondition, preserving the established no-postcondition fast path.
/// `None` reports a broken dense [`FunctionId`] invariant.
pub(crate) fn postcondition_schedule<'function>(
    functions: impl IntoIterator<Item = &'function CheckedFunction>,
) -> Option<PostconditionSchedule> {
    let functions = functions.into_iter().collect::<Vec<_>>();
    if !functions
        .iter()
        .any(|function| function.postcondition.is_some())
    {
        return Some(PostconditionSchedule::default());
    }
    let mut graph = vec![Vec::new(); functions.len()];
    for (index, function) in functions.iter().enumerate() {
        if function.id.0 as usize != index {
            return None;
        }
        collect_statement_calls(&function.body, &mut graph[index]);
        if graph[index]
            .iter()
            .any(|callee| callee.0 as usize >= functions.len())
        {
            return None;
        }
        graph[index].sort_unstable_by_key(|function| function.0);
        graph[index].dedup();
    }

    let graph = graph
        .into_iter()
        .map(|callees| {
            callees
                .into_iter()
                .map(|function| function.0 as usize)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut components = strongly_connected_components(&graph);
    for component in &mut components {
        component.sort_unstable();
    }
    let mut component_of = vec![usize::MAX; functions.len()];
    for (component, members) in components.iter().enumerate() {
        for member in members {
            component_of[*member] = component;
        }
    }

    // The source graph points caller -> callee. Reverse each inter-component
    // edge so Kahn's order is callee-before-caller, selecting the component
    // with the smallest dense FunctionId whenever multiple are ready.
    let mut callers = vec![BTreeSet::new(); components.len()];
    let mut incoming = vec![0usize; components.len()];
    for (caller, callees) in graph.iter().enumerate() {
        let caller_component = component_of[caller];
        for callee in callees {
            let callee_component = component_of[*callee];
            if caller_component != callee_component
                && callers[callee_component].insert(caller_component)
            {
                incoming[caller_component] += 1;
            }
        }
    }
    let mut ready = components
        .iter()
        .enumerate()
        .filter(|(component, _)| incoming[*component] == 0)
        .map(|(component, members)| (members[0], component))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(components.len());
    while let Some(next) = ready.iter().next().copied() {
        ready.remove(&next);
        let (_, component) = next;
        order.push(component);
        for caller in &callers[component] {
            incoming[*caller] -= 1;
            if incoming[*caller] == 0 {
                ready.insert((components[*caller][0], *caller));
            }
        }
    }
    if order.len() != components.len() {
        return None;
    }
    let mut ordered_component_of = vec![usize::MAX; components.len()];
    for (ordinal, component) in order.iter().enumerate() {
        ordered_component_of[*component] = ordinal;
    }
    for (caller, callees) in graph.iter().enumerate() {
        for callee in callees {
            let caller_component = ordered_component_of[component_of[caller]];
            let callee_component = ordered_component_of[component_of[*callee]];
            if caller_component != callee_component && callee_component >= caller_component {
                return None;
            }
        }
    }

    Some(PostconditionSchedule {
        components: order
            .into_iter()
            .enumerate()
            .map(|(ordinal, component)| PostconditionComponent {
                ordinal: u32::try_from(ordinal)
                    .expect("postcondition SCC count exceeds the u32 identity space"),
                functions: components[component]
                    .iter()
                    .map(|function| {
                        FunctionId(
                            u32::try_from(*function)
                                .expect("concrete function count exceeds the u32 identity space"),
                        )
                    })
                    .collect(),
                summaries: Vec::new(),
            })
            .collect(),
    })
}

fn collect_statement_calls(statements: &[CheckedStatement], calls: &mut Vec<FunctionId>) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Check {
                condition: value, ..
            }
            | CheckedStatement::Claim {
                condition: value, ..
            }
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. } => collect_expression_calls(value, calls),
            CheckedStatement::PropagateLet { scrutinee, .. } => {
                collect_expression_calls(scrutinee, calls);
            }
            CheckedStatement::Set { target, value, .. } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        collect_expression_calls(&target.offset, calls);
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression_calls(&target.offset, calls);
                    }
                }
                collect_expression_calls(value, calls);
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_expression_calls(scrutinee, calls);
                for arm in arms {
                    collect_statement_calls(&arm.body, calls);
                }
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                collect_statement_calls(body, calls);
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                collect_expression_calls(lower, calls);
                collect_expression_calls(upper, calls);
                collect_statement_calls(body, calls);
            }
            CheckedStatement::Break { .. } => {}
        }
    }
}

fn collect_expression_calls(expression: &CheckedExpression, calls: &mut Vec<FunctionId>) {
    if let CheckedExpression::UserCall { function, .. } = expression {
        calls.push(*function);
    }
    for child in flow::expression_children(expression) {
        collect_expression_calls(child, calls);
    }
}

fn strongly_connected_components(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct Tarjan<'graph> {
        graph: &'graph [Vec<usize>],
        next_index: usize,
        indices: Vec<Option<usize>>,
        lowlinks: Vec<usize>,
        stack: Vec<usize>,
        on_stack: Vec<bool>,
        components: Vec<Vec<usize>>,
    }

    impl Tarjan<'_> {
        fn visit(&mut self, node: usize) {
            let index = self.next_index;
            self.next_index += 1;
            self.indices[node] = Some(index);
            self.lowlinks[node] = index;
            self.stack.push(node);
            self.on_stack[node] = true;

            for successor in &self.graph[node] {
                if self.indices[*successor].is_none() {
                    self.visit(*successor);
                    self.lowlinks[node] = self.lowlinks[node].min(self.lowlinks[*successor]);
                } else if self.on_stack[*successor] {
                    self.lowlinks[node] = self.lowlinks[node].min(
                        self.indices[*successor]
                            .expect("on-stack node has a Tarjan discovery index"),
                    );
                }
            }

            if self.lowlinks[node] == index {
                let mut component = Vec::new();
                loop {
                    let member = self
                        .stack
                        .pop()
                        .expect("Tarjan root retains its stack member");
                    self.on_stack[member] = false;
                    component.push(member);
                    if member == node {
                        break;
                    }
                }
                self.components.push(component);
            }
        }
    }

    let mut tarjan = Tarjan {
        graph,
        next_index: 0,
        indices: vec![None; graph.len()],
        lowlinks: vec![0; graph.len()],
        stack: Vec::new(),
        on_stack: vec![false; graph.len()],
        components: Vec::new(),
    };
    for node in 0..graph.len() {
        if tarjan.indices[node].is_none() {
            tarjan.visit(node);
        }
    }
    tarjan.components
}

/// The engine's fragment-type gate: one member of the closed integer set
/// [OP-2], the only types terms may select [ENT-2].
const fn fragment_type(ty: CheckedType) -> Option<super::model::IntegerType> {
    match ty {
        CheckedType::Integer(ty) => Some(ty),
        _ => None,
    }
}
