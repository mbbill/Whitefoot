//! [ENT-3] flow of facts over the conservative structural graph [FN-1], with
//! [ENT-5] kills, joins, and the no-induction loop rule, and [ENT-6]
//! obligation discharge with residual rendering.
//!
//! The walker carries the live fact state forward through the checked
//! statement tree, which is the structural graph: statements sequence, match
//! arms fork and join, loops iterate through their break edges, and
//! `return`/`give`/`break`/`propagate` leave scopes on edges. Scope-exit
//! kills always apply on the edge, before any join at the edge's target.
//!
//! The [ENT-3] fact sources themselves — which checked shape establishes
//! which relation — live in [`sources`]; this module owns the graph, the
//! kills, the joins, and the obligation judgment, and calls into the sources
//! at each establishment point.

mod sources;

use std::collections::{HashMap, HashSet};

use super::super::goal::{
    CheckedRequirement, ConcreteGoal, GoalDatum, GoalExpression, GoalOperation, GoalProjection,
};
use super::super::model::expression_children;
use super::super::model::{
    BindingId, CheckedAffineExpression, CheckedAffineExpressionKind, CheckedAffineRelation,
    CheckedArrayRoot, CheckedBooleanOperation, CheckedConst, CheckedConstructor, CheckedEnumType,
    CheckedExpression, CheckedFloatOperation, CheckedFunction, CheckedIntegerOperation,
    CheckedLoopId, CheckedLoopInvariant, CheckedMatchArm, CheckedMode, CheckedNominal,
    CheckedNominalKind, CheckedNumericType, CheckedSetTarget, CheckedStatement, CheckedType,
    CheckedValue, FloatType, IntegerType, ValueInitializerKind,
};
use super::super::places::{BindingSummary, PlaceMap, ResolvedPlace};
use super::super::postcondition::{
    CheckedPostcondition, NormalizedRelation, PostconditionPlaceRoot, PostconditionReturnDatum,
    PostconditionReturnPlace, PostconditionReturnPlaceRoot, RelationDatum, RelationTemplate,
};
use super::affine::{
    AffineCheckError, AffineCheckLimit, AffineCheckState, AffineCoefficient, AffineForm,
    AffineInequality, AffineTermId, MAX_CERTIFICATE_PREMISES, ScaledAffinePremise,
    interval_maximum, interval_proves, sum_explicit_inequalities, sum_explicit_scaled_inequalities,
};
use super::state::{
    AffinePremiseUse, ClosedState, CountedRootAtom, DerivationId, DerivationInventory,
    DerivationLedger, DerivationNode, DerivationRootKind, FactState, FlowEventId, FlowEventKind,
    GoalId, GoalNormalization, GoalSign, GoalSupport, GoalTable, JoinParent, OutcomeFact,
    PostconditionCallSubstitution, Relation, SourceAffineFactRef, SourceLoopInvariantRef, close,
    close_excluding_term, contradiction_without_proofs, join_at, materialize_closure_at,
};
use super::term::{
    CountedCaptureSide, LengthBound, PlaceProjection, PlaceRoot, PlaceTerm, ProjectedPlaceTerm,
    TermId, TermKind, TermTable, ZERO, integer_value, type_range,
};
use super::{
    BoundsRequest, CallGoalDisposition, CallGoalEvidence, CallGoalOutcome, CountedDerivationSet,
    EntailmentContext, FunctionEntailment, FunctionPostconditionProof, JoinedSourceProofProvenance,
    LoopInvariantOutcome, LoopInvariantProof, ObligationFamily, ObligationOutcome,
    PostconditionAggregate, PostconditionDisposition, PostconditionEntryImage,
    PostconditionEntryImageOutcome, PostconditionExit, S7Derivation, SourceProofCertificateFailure,
    SourceProofCheck, SourceProofOutcome, VerifiedPostconditionSummary,
    VerifiedPostconditionSummaryRef, fragment_type, overflow_conjuncts_for_values,
};
use crate::SYSTEM_OPERATIONS;

/// One [ENT-5] kill event gathered from a statement or expression.
#[derive(Clone, Debug)]
enum KillEvent {
    /// (a) a `set` commit or (b) a boundary-projected callee write. An
    /// element write targets indexed element storage, which never kills a
    /// length fact [ENT-5].
    Write {
        place: ResolvedPlace,
        element: bool,
        source: crate::NodePath,
    },
    /// (c) a consuming use of a binding.
    Consume {
        binding: BindingId,
        source: crate::NodePath,
    },
    /// An affine borrow-holder occurrence whose checked move identity is not
    /// represented by its referent value type. The pre-v0.28 L0 flow did not
    /// apply this event to ordinary facts; FN-9 consumes it only for the new
    /// view-independent entry-image lifetime, preserving no-ensures behavior.
    EntryImageHolderConsume {
        binding: BindingId,
        source: crate::NodePath,
    },
    /// A callee write projected through a directly transferred holder. The
    /// old fact flow did not recognize that checked argument shape; retaining
    /// it as entry-image-only keeps that path unchanged while FN-9 observes
    /// the required effect kill.
    EntryImageHolderWrite {
        place: ResolvedPlace,
        element: bool,
        source: crate::NodePath,
    },
}

impl KillEvent {
    fn source(&self) -> &crate::NodePath {
        match self {
            Self::Write { source, .. }
            | Self::Consume { source, .. }
            | Self::EntryImageHolderConsume { source, .. }
            | Self::EntryImageHolderWrite { source, .. } => source,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EntryImageRecord {
    datum: PostconditionEntryImage,
    place: ResolvedPlace,
    holders: Vec<BindingId>,
}

/// A `loop` frame collecting break-edge states for the continuation join.
struct LoopFrame {
    id: CheckedLoopId,
    scope_depth: usize,
    /// The compiler-owned counted binder while this is a `for` frame. An
    /// ordinary `loop` has no binder and contributes no affine index image.
    counted_binder: Option<BindingId>,
    /// Present only for a counted range. A break through this frame leaves
    /// the private endpoint-capture scope as well as source binding scopes.
    capture_path: Option<Vec<u32>>,
    breaks: Vec<ProofFlowState>,
}

/// The [ENT-3] facts one `match` scrutinee admits at its arms' entries: the
/// S1 comparison relation, taken positively on `True()` and exactly negated
/// on `False()`, and the S7/S10 fact one named arm's value binder gains,
/// carried with that arm's tag. Every other arm establishes nothing.
#[derive(Default)]
struct ArmFacts {
    node_path: Option<crate::NodePath>,
    comparison: Option<Relation>,
    goals: Vec<GoalId>,
    outcome: Option<(u32, OutcomeFact)>,
}

/// A `value_match` frame collecting give-edge states for the continuation.
struct GiveFrame {
    scope_depth: usize,
    loop_depth: usize,
    kind: ValueInitializerKind,
    node_path: crate::NodePath,
    binding: BindingId,
    result_type: CheckedType,
    gives: Vec<ProofFlowState>,
    give_goal_origins: Vec<Option<GoalId>>,
    delivery_images: Vec<ProofFlowState>,
    delivery_edges: Vec<crate::NodePath>,
}

/// Stable source and substitution identity for one [GIVE-1] edge.
struct DeliveryEdgeContext<'a> {
    statement: &'a crate::NodePath,
    carrier_binding: BindingId,
    receiver_binding: BindingId,
    carrier: TermId,
    receiver: TermId,
    event: FlowEventId,
}

/// The value-initializer receiver and lexical boundary shared by its gives.
struct DeliveryImageContext<'a> {
    statement: &'a crate::NodePath,
    receiver_binding: BindingId,
    receiver_type: CheckedType,
    scope_depth: usize,
    loop_depth: usize,
}

/// Stable receiver identity shared by all delivery edges at one join.
struct DeliveryJoinContext<'a> {
    statement: &'a crate::NodePath,
    receiver_binding: BindingId,
    receiver: TermId,
    event: FlowEventId,
}

/// The source facts carried through one structural walk.
#[derive(Clone, Debug, Default)]
struct ProofFlowState {
    facts: FactState,
    /// First invalidating event for each relation-template entry image. This
    /// state branches with the same structural flow; `None` means the image
    /// is still live.
    entry_images: Vec<Option<FlowEventId>>,
    /// Exact integer value images and active source-proved loop invariants.
    /// Executing a statement computes the runtime value represented here.
    affine: AffineFlowState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AffineFlowState {
    values: HashMap<BindingId, AffineForm>,
    assumptions: Vec<ActiveAffineFact>,
    exports: Vec<ActiveAffineFact>,
    proofs: Vec<ActiveAffineFact>,
    /// Fixed unsigned literal-division consequences over exact runtime value
    /// atoms. A binding kill removes the binding-to-value map, not a theorem
    /// about the old value: a later value receives another atom, while a live
    /// alias may still use the old image. An image with no live alias is only
    /// unreachable, bounded compiler state that may be reclaimed later.
    division_images: Vec<AutomaticAffineFact>,
}

/// The numeric/logical proof state at one exact control-flow point.
#[derive(Clone, Copy)]
struct ProofContext<'a> {
    facts: &'a FactState,
    affine: &'a AffineFlowState,
}

impl<'a> ProofContext<'a> {
    fn new(facts: &'a FactState, affine: &'a AffineFlowState) -> Self {
        Self { facts, affine }
    }
}

/// One consumer-normalized numeric/logical proposition.  A signed goal keeps
/// the finite Boolean structure written at a call boundary.  An ordering goal
/// is the exact L0 relation selected for a function postcondition.  Either
/// consumer may additionally provide the unique affine inequality for the
/// same proposition; the proof entry never invents another formula.
enum ProofGoal<'a> {
    /// One source-written canonical affine inequality. PRF-1 submits each
    /// `use` through the same proof entry as partial operations and callable
    /// boundaries; it does not call the affine checker as a private fallback.
    Affine { inequality: &'a AffineInequality },
    Signed {
        expression: &'a GoalExpression,
        affine: Option<&'a AffineInequality>,
    },
    Ordering {
        relation: &'a Relation,
        affine: Option<&'a AffineInequality>,
    },
    /// One OP-2 exact-integer domain proposition. The dispatcher first checks
    /// its finite goal/L0 normalization, then the fixed affine clauses, then
    /// the one fixed two-operand interval-product rule. The consumer chooses
    /// none of those routes and performs no second query.
    IntegerDomain(IntegerDomainGoal<'a>),
    /// One exact relation `left - right <= bound`. OP-4 and SYS-8 submit the
    /// same proposition through this entry; the finite signed form and the
    /// explicitly prepared affine forms are alternate representations of
    /// that proposition, not additional queries by either consumer.
    BoundedRelation(BoundedRelationGoal<'a>),
    /// One fixed ordering relation exposed through an optional finite goal
    /// normalization. OP-9 supplies both identities when its source operand
    /// belongs to the goal fragment, and only the relation otherwise.
    NormalizedOrdering {
        goal: Option<GoalId>,
        relation: Option<&'a Relation>,
        affine: Option<&'a AffineInequality>,
        upper_bound: Option<NumericUpperBoundRequest<'a>>,
    },
}

#[derive(Clone, Copy)]
struct BoundedRelationGoal<'a> {
    canonical: Option<&'a GoalExpression>,
    request: BoundsRequest,
    direct_affine: Option<&'a AffineInequality>,
    fixed_affine_bridge: Option<FixedAffineBoundBridge<'a>>,
    affine_left: Option<&'a AffineForm>,
}

/// One fixed affine first step followed by ordinary difference closure.  The
/// consumer supplies the exact first-step inequality and middle term; the
/// checker derives the remaining middle-to-right bound from the submitted
/// relation, so no route or coefficient search is introduced.
#[derive(Clone, Copy)]
struct FixedAffineBoundBridge<'a> {
    target: &'a AffineInequality,
    middle: TermId,
    left_to_middle_bound: i128,
}

struct IntegerDomainGoal<'a> {
    canonical: Option<GoalId>,
    operation: CheckedIntegerOperation,
    operand_type: CheckedType,
    components: &'a [BoundsRequest],
    affine_clauses: Option<&'a [Vec<AffineInequality>]>,
    affine_product: Option<&'a AffineIntegerProduct>,
}

/// Optional numeric projection requested by a consumer of one proved
/// ordering. `admitted` is the ceiling stated by that exact ordering; `term`
/// and `affine` are its two current-context value images, when available.
/// Projection can tighten the admitted ceiling, but never decides whether the
/// ordering itself is proved.
#[derive(Clone, Copy)]
struct NumericUpperBoundRequest<'a> {
    term: Option<TermId>,
    affine: Option<&'a AffineForm>,
    admitted: i128,
}

#[derive(Clone, Copy)]
struct ProvedNumericUpperBound {
    value: i128,
    derivation: DerivationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProofDisposition {
    Proved,
    Refuted,
    Unknown,
}

/// Complete route selected by one [`Analyzer::prove`] call.  The signed
/// ordinary route retains every simultaneously available finite ground so
/// FN-8 can preserve its existing evidence payload without running a second
/// query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProofRoute {
    Contradiction,
    SignedOrdinary {
        opaque: bool,
        projection: bool,
        normalization: bool,
        introduction: bool,
    },
    FiniteGoal,
    L0,
    Affine,
}

struct ProofResult {
    disposition: ProofDisposition,
    route: Option<ProofRoute>,
    derivation: Option<DerivationId>,
    numeric_upper_bound: Option<ProvedNumericUpperBound>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveAffineFact {
    source: SourceAffineFactRef,
    inequality: AffineInequality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AutomaticAffineFact {
    inequality: AffineInequality,
    parent: DerivationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AffineAtom {
    /// Source integer class used only when a live binding maps directly to
    /// this atom and ordinary L0 facts can therefore tighten its interval.
    ty: IntegerType,
    /// Intrinsic mathematical interval of this atom. Ordinary unknown values
    /// use their full source-type range. A structural affine join may instead
    /// allocate a delta atom whose bounds are the exact minimum and maximum
    /// incoming constants, including negative constants.
    minimum: i128,
    maximum: i128,
}

struct PostconditionExitProof {
    disposition: PostconditionDisposition,
    derivation: Option<DerivationId>,
}

struct AffineConsequenceProof {
    premises: Vec<AffinePremiseUse>,
    parents: Vec<DerivationId>,
}

struct AffineL0Candidate {
    term: TermId,
    value: AffineForm,
}

struct AffineL0Entry {
    inequality: AffineInequality,
    left: TermId,
    right: TermId,
    bound: i128,
}

#[derive(Default)]
struct AffineL0Index {
    entries: Vec<AffineL0Entry>,
    by_terms: HashMap<Box<[AffineCoefficient]>, usize>,
}

impl AffineL0Index {
    fn entry(&self, terms: &[AffineCoefficient]) -> Option<&AffineL0Entry> {
        self.by_terms.get(terms).map(|index| &self.entries[*index])
    }
}

struct AutomaticAffinePremise {
    inequality: AffineInequality,
    source: Option<SourceAffineFactRef>,
    parent: Option<DerivationId>,
}

/// Exhausts the unordered coefficient-one premise pairs, including `(p, p)`.
///
/// Candidate arithmetic is isolated: an unrepresentable pair is skipped and
/// cannot hide a later representable witness. Returning after a successful
/// callback is acceptance-order independent because every earlier candidate
/// has already failed and adding another premise cannot remove an existing
/// pair from this source-shaped finite set.
fn first_two_premise_residual<T>(
    target: &AffineInequality,
    premises: &[AutomaticAffinePremise],
    check: &mut AffineCheckState,
    mut prove: impl FnMut(&AffineInequality, &mut AffineCheckState) -> Option<T>,
) -> Option<(usize, usize, T)> {
    for first in 0..premises.len() {
        for second in first..premises.len() {
            let pair = [
                premises[first].inequality.clone(),
                premises[second].inequality.clone(),
            ];
            let Ok(sum) = sum_explicit_inequalities(&pair, check) else {
                continue;
            };
            let Ok(residual) = AffineInequality::residual_after(target, &sum, check) else {
                continue;
            };
            if let Some(proof) = prove(&residual, check) {
                return Some((first, second, proof));
            }
        }
    }
    None
}

struct AffineIntervalEndpointProof {
    value: i128,
    consequence: AffineConsequenceProof,
}

struct AffineClosedIntervalProof {
    minimum: AffineIntervalEndpointProof,
    maximum: AffineIntervalEndpointProof,
}

struct AffineIntegerProduct {
    left: AffineForm,
    right: AffineForm,
    ty: IntegerType,
}

#[derive(Clone, Copy)]
enum IntegerDomainPlanKind {
    Conjunction,
    SignedDivision,
}

struct IntegerDomainPlan {
    components: Vec<BoundsRequest>,
    kind: IntegerDomainPlanKind,
}

impl IntegerDomainPlan {
    fn normalization(&self) -> GoalNormalization {
        let components = self.components.iter().map(request_relation).collect();
        match self.kind {
            IntegerDomainPlanKind::Conjunction => GoalNormalization::conjunction(components),
            IntegerDomainPlanKind::SignedDivision => GoalNormalization::signed_division(components),
        }
    }
}

#[derive(Clone, Copy)]
struct IntegerDomainOperand {
    term: Option<TermId>,
    constant: Option<i128>,
}

/// Transient A0 evidence for an exact root user call. It never enters the
/// checked expression tree, so named, nested, or stored outcomes acquire no
/// pending publication token.
#[derive(Clone, Debug)]
struct PreparedCall {
    function: super::super::model::FunctionId,
    call: crate::NodePath,
    parents: Vec<DerivationId>,
    transfer_events: Vec<FlowEventId>,
    kills: Vec<KillEvent>,
}

#[derive(Clone, Debug)]
struct AvailablePostcondition {
    relation: RelationTemplate,
    variant: Option<crate::PreludeDeclarationId>,
    field: Option<crate::PreludeDeclarationId>,
    summary: VerifiedPostconditionSummary,
    discharged: bool,
}

#[derive(Clone)]
struct InstantiatedPostcondition {
    relation: Relation,
    substitutions: Vec<PostconditionCallSubstitution>,
}

#[derive(Clone, Copy)]
struct DirectMatchRoute {
    variant: crate::PreludeDeclarationId,
    field: crate::PreludeDeclarationId,
    tag: u32,
    binding: BindingId,
    ty: CheckedType,
}

struct EstablishedDirectMatch {
    route: DirectMatchRoute,
    instantiated: InstantiatedPostcondition,
    parent: Option<DerivationId>,
}

#[derive(Clone, Copy)]
struct SelectedReceiverRoute {
    payload: BindingId,
    binding: BindingId,
}

struct SelectedReceiverCandidate {
    route: SelectedReceiverRoute,
    relation: Relation,
    parent: Option<DerivationId>,
}

#[derive(Clone, Copy)]
struct DirectReceiverRoute {
    binding: BindingId,
    formal: u32,
    ty: CheckedType,
}

struct DirectReceiverCandidate {
    route: DirectReceiverRoute,
    available: AvailablePostcondition,
    instantiated: InstantiatedPostcondition,
}

struct SetWalkOutcome {
    target_event: Option<FlowEventId>,
}

/// The [ENT-5] loop rule's structural kill summary of one loop body.
#[derive(Default)]
struct LoopKills {
    events: Vec<KillEvent>,
    /// Statement/expression event groups retain semantic evaluation order
    /// within one carrier while the reachability walk scans statements in
    /// reverse. Entry-image invalidation reorders only these groups by source,
    /// never the argument-consume/callee-write events inside a group.
    entry_image_groups: Vec<LoopKillEventGroup>,
    /// Every binding named as a `set` target. An ordinary-let origin is valid
    /// only while its bound value has no intervening whole, field, or element
    /// mutation; the narrower comparison/outcome origins can only inhabit
    /// nonprojectable Bool/outcome bindings, so this same set is exact there.
    set_bindings: HashSet<BindingId>,
}

struct LoopKillEventGroup {
    owner: crate::NodePath,
    range: std::ops::Range<usize>,
}

impl LoopKills {
    fn push_event_group(&mut self, events: Vec<KillEvent>) {
        let Some(owner) = events
            .iter()
            .map(KillEvent::source)
            .min_by(|left, right| left.components().cmp(right.components()))
            .cloned()
        else {
            return;
        };
        let start = self.events.len();
        self.events.extend(events);
        self.entry_image_groups.push(LoopKillEventGroup {
            owner,
            range: start..self.events.len(),
        });
    }
}

// A continuing scope-exit edge can close only scopes opened inside the target
// loop body. No binding from such a scope can support a fact in the pre-loop
// state this summary filters. An edge that closes a pre-loop binding's scope
// necessarily leaves the target body and is non-continuing, so kill event (d)
// needs no payload in `LoopKills`.

/// Non-local successors visible while asking whether an edge inside one loop
/// body can reach that loop's next iteration head without leaving the body.
/// Targets outside the body are deliberately absent and therefore do not
/// reach the head.
#[derive(Default)]
struct LoopReachability {
    breaks: Vec<(CheckedLoopId, bool)>,
    gives: Vec<bool>,
}

impl LoopReachability {
    fn break_reaches(&self, target: CheckedLoopId) -> bool {
        self.breaks
            .iter()
            .rev()
            .find_map(|(id, reaches)| (*id == target).then_some(*reaches))
            .unwrap_or(false)
    }
}

pub(super) fn analyze(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    let mut entailment = analyze_candidate_inner(function, context);
    finish(&mut entailment);
    entailment
}

/// Builds one optimistic per-function FN-9 proof batch without pruning or
/// remapping its derivation ledger. The checker calls [`finish`] after the
/// component publication decision.
pub(super) fn analyze_candidate(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    let mut entailment = analyze_candidate_inner(function, context);
    entailment.derivations.settle();
    entailment
}

fn analyze_candidate_inner(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    let run = run(function, context);
    FunctionEntailment {
        body_disposition: run.body_disposition,
        obligations: run.obligations,
        call_goals: run.call_goals,
        counted_derivations: run.counted_derivations,
        loop_invariants: run.loop_invariants,
        source_proofs: run.source_proofs,
        joined_source_proofs: run.joined_source_proofs,
        s7_derivations: run.s7_derivations,
        postconditions: run.postconditions,
        boolean_decompositions: run.boolean_decompositions,
        derivations: run.derivations,
        inventory: run.inventory,
    }
}

struct AnalysisRun {
    body_disposition: super::super::model::CheckedBodyDisposition,
    obligations: Vec<ObligationOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    counted_derivations: Vec<CountedDerivationSet>,
    loop_invariants: Vec<LoopInvariantOutcome>,
    source_proofs: Vec<SourceProofOutcome>,
    joined_source_proofs: Vec<JoinedSourceProofProvenance>,
    s7_derivations: Vec<S7Derivation>,
    postconditions: Vec<super::FunctionPostconditionProof>,
    boolean_decompositions: Vec<super::BooleanGoalDecomposition>,
    derivations: DerivationLedger,
    inventory: DerivationInventory,
}

fn run(function: &CheckedFunction, context: &EntailmentContext<'_>) -> AnalysisRun {
    let mut analyzer = Analyzer {
        context,
        function,
        places: PlaceMap::default(),
        terms: TermTable::new(),
        goals: GoalTable::default(),
        derivations: DerivationLedger::default(),
        obligations: Vec::new(),
        call_goals: Vec::new(),
        counted_derivations: Vec::new(),
        loop_invariants: Vec::new(),
        source_proofs: Vec::new(),
        joined_source_proofs: Vec::new(),
        s7_derivations: Vec::new(),
        postconditions: Vec::new(),
        boolean_decompositions: Vec::new(),
        entry_images: Vec::new(),
        postcondition_entry_images: Vec::new(),
        affine_atoms: Vec::new(),
        encountered_counted: 0,
        completed_counted_roots: 0,
        s12_roots: 0,
        delivery_give_roots: 0,
        delivery_join_roots: 0,
        scopes: Vec::new(),
        loops: Vec::new(),
        gives: Vec::new(),
    };
    analyzer.collect_bindings();
    analyzer.collect_postcondition_entry_images();
    let mut state = ProofFlowState {
        entry_images: vec![None; analyzer.entry_images.len()],
        ..ProofFlowState::default()
    };
    analyzer.initialize_affine_parameters(&mut state.affine);
    analyzer
        .scopes
        .push(function.parameters.iter().map(|p| p.binding).collect());
    // [ENT-3] S4: every substituted `requires` goal independently enters the
    // body state in source order. No clause derives another clause.
    for requirement in &function.requirements {
        let event = analyzer.proof_event(FlowEventKind::S4, Some(&requirement.clause));
        analyzer.establish_requires_facts(requirement, &mut state.facts, event);
    }
    let body_disposition = {
        let closed = close(
            &state.facts,
            &analyzer.terms,
            &analyzer.goals,
            &mut analyzer.derivations,
        );
        match closed.contradiction_proof() {
            Some(contradiction) => {
                analyzer
                    .derivations
                    .add_root(DerivationRootKind::BodyEntryContradiction, contradiction);
                super::super::model::CheckedBodyDisposition::Uninhabited { contradiction }
            }
            None => super::super::model::CheckedBodyDisposition::Inhabited,
        }
    };
    if matches!(
        body_disposition,
        super::super::model::CheckedBodyDisposition::Inhabited
    ) {
        analyzer.initialize_postcondition_proofs();
    }
    analyzer.walk_block(&function.body, &mut state);
    analyzer.scopes.pop();
    analyzer.finalize_postcondition_aggregates();
    assert_eq!(
        analyzer.completed_counted_roots, analyzer.encountered_counted,
        "every encountered counted statement must publish one complete S11 root group"
    );
    let (terms, length_bounds) = analyzer.terms.into_inventory();
    let inventory = DerivationInventory {
        terms,
        length_bounds,
        goals: analyzer.goals.into_inventory(),
    };
    AnalysisRun {
        body_disposition,
        obligations: analyzer.obligations,
        call_goals: analyzer.call_goals,
        counted_derivations: analyzer.counted_derivations,
        loop_invariants: analyzer.loop_invariants,
        source_proofs: analyzer.source_proofs,
        joined_source_proofs: analyzer.joined_source_proofs,
        s7_derivations: analyzer.s7_derivations,
        postconditions: analyzer.postconditions,
        boolean_decompositions: analyzer.boolean_decompositions,
        derivations: analyzer.derivations,
        inventory,
    }
}

/// Finalizes the sole function-local derivation ledger after the optimistic
/// FN-9 component batch has been accepted.
pub(super) fn finish(entailment: &mut FunctionEntailment) {
    let event_roots = entailment
        .postconditions
        .iter()
        .flat_map(|proof| &proof.exits)
        .flat_map(|exit| &exit.entry_images)
        .filter_map(|image| image.invalidation)
        .collect::<Vec<_>>();
    let remap = entailment.derivations.finish_with_event_roots(&event_roots);
    if let super::super::model::CheckedBodyDisposition::Uninhabited { contradiction } =
        &mut entailment.body_disposition
    {
        *contradiction = remap
            .nodes
            .get(contradiction.0 as usize)
            .copied()
            .flatten()
            .expect("body-entry contradiction root retained by finish");
    }
    for outcome in &mut entailment.obligations {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
        outcome.allocation_length_upper_bound_derivation = outcome
            .allocation_length_upper_bound_derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for outcome in &mut entailment.call_goals {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for counted in &mut entailment.counted_derivations {
        remap_counted_derivations(counted, &remap.nodes);
    }
    for source in &mut entailment.s7_derivations {
        source.parent = remap
            .nodes
            .get(source.parent.0 as usize)
            .copied()
            .flatten()
            .expect("required S7 source root retained by the sole ledger root channel");
        source.event = entailment
            .derivations
            .node_event(source.parent)
            .expect("S7 source parent retains its shared structural event");
    }
    for postcondition in &mut entailment.postconditions {
        remap_postcondition(postcondition, &remap.nodes, &remap.events);
    }
}

fn remap_counted_derivations(counted: &mut CountedDerivationSet, remap: &[Option<DerivationId>]) {
    let remap_parent = |parent: &mut DerivationId| {
        *parent = remap
            .get(parent.0 as usize)
            .copied()
            .flatten()
            .expect("counted S11 root parent retained by the sole ledger root channel");
    };
    for parent in [
        &mut counted.lower_capture_eq_endpoint.forward.parent,
        &mut counted.lower_capture_eq_endpoint.reverse.parent,
        &mut counted.upper_capture_eq_endpoint.forward.parent,
        &mut counted.upper_capture_eq_endpoint.reverse.parent,
        &mut counted.binder_eq_lower_capture.forward.parent,
        &mut counted.binder_eq_lower_capture.reverse.parent,
        &mut counted.lower_capture_le_binder.atomic.parent,
        &mut counted.binder_lt_upper_capture.atomic.parent,
    ] {
        remap_parent(parent);
    }
}

fn remap_postcondition(
    proof: &mut FunctionPostconditionProof,
    nodes: &[Option<DerivationId>],
    events: &[Option<FlowEventId>],
) {
    for exit in &mut proof.exits {
        for image in &mut exit.entry_images {
            if let Some(old) = image.invalidation {
                image.invalidation = Some(
                    events
                        .get(old.0 as usize)
                        .copied()
                        .flatten()
                        .expect("required entry-image invalidation event retained by finish"),
                );
            }
        }
        match exit.disposition {
            PostconditionDisposition::Discharged => {
                let old = exit
                    .derivation
                    .expect("every discharged postcondition exit has a required root");
                exit.derivation = Some(
                    nodes
                        .get(old.0 as usize)
                        .copied()
                        .flatten()
                        .expect("required postcondition exit root retained by finish"),
                );
            }
            PostconditionDisposition::Refuted | PostconditionDisposition::Unproved => {
                assert!(exit.derivation.is_none());
            }
        }
    }
    let remap_aggregate = |aggregate: &mut PostconditionAggregate| {
        if aggregate.discharged {
            let old = aggregate
                .derivation
                .expect("every discharged postcondition aggregate has a required root");
            aggregate.derivation = Some(
                nodes
                    .get(old.0 as usize)
                    .copied()
                    .flatten()
                    .expect("required postcondition aggregate root retained by finish"),
            );
        } else {
            assert!(aggregate.derivation.is_none());
        }
    };
    remap_aggregate(&mut proof.aggregate);
}

struct Analyzer<'check, 'unit> {
    context: &'check EntailmentContext<'unit>,
    function: &'check CheckedFunction,
    /// Structural [OWN-5] place resolution for this function.
    places: PlaceMap,
    terms: TermTable,
    goals: GoalTable,
    derivations: DerivationLedger,
    obligations: Vec<ObligationOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    counted_derivations: Vec<CountedDerivationSet>,
    loop_invariants: Vec<LoopInvariantOutcome>,
    source_proofs: Vec<SourceProofOutcome>,
    joined_source_proofs: Vec<JoinedSourceProofProvenance>,
    s7_derivations: Vec<S7Derivation>,
    postconditions: Vec<super::FunctionPostconditionProof>,
    /// O11 candidate decomposition sets, recorded at
    /// signed-goal establishments and never established as facts.
    boolean_decompositions: Vec<super::BooleanGoalDecomposition>,
    entry_images: Vec<EntryImageRecord>,
    /// Global entry-image indices used by each source-ordered relation. The
    /// flow state tracks invalidation once per structural image, while each
    /// FN-9 proof consults only the images its own relation references.
    postcondition_entry_images: Vec<Vec<usize>>,
    /// Function-local mathematical atoms allocated in structural execution
    /// order. They are ordinary checker state and are discarded with the
    /// analysis.
    affine_atoms: Vec<AffineAtom>,
    encountered_counted: u32,
    completed_counted_roots: u32,
    s12_roots: u32,
    delivery_give_roots: u32,
    delivery_join_roots: u32,
    /// Lexical scope stack: the bindings declared in each open block.
    scopes: Vec<Vec<BindingId>>,
    loops: Vec<LoopFrame>,
    gives: Vec<GiveFrame>,
}

impl Analyzer<'_, '_> {
    fn initialize_affine_parameters(&mut self, state: &mut AffineFlowState) {
        let parameters = self
            .function
            .parameters
            .iter()
            .filter_map(|parameter| match (parameter.mode, parameter.ty) {
                (CheckedMode::Own, CheckedType::Integer(ty)) => Some((parameter.binding, ty)),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (binding, ty) in parameters {
            let value = self.new_affine_atom(ty);
            state.values.insert(binding, value);
        }
    }

    fn initialize_postcondition_proofs(&mut self) {
        let aggregate = || PostconditionAggregate {
            discharged: false,
            derivation: None,
        };
        self.postconditions = self
            .function
            .postconditions
            .iter()
            .enumerate()
            .map(|(ordinal, postcondition)| FunctionPostconditionProof {
                block: postcondition.selector.block.clone(),
                selector: postcondition.selector.selector.clone(),
                relation_ordinal: u32::try_from(ordinal)
                    .expect("postcondition relation ordinal exceeds u32"),
                summary: None,
                exits: Vec::new(),
                aggregate: aggregate(),
            })
            .collect();
    }

    fn finalize_postcondition_aggregates(&mut self) {
        for index in 0..self.postconditions.len() {
            let block = self.postconditions[index].block.clone();
            let relation_ordinal = self.postconditions[index].relation_ordinal;
            let parents = self.postconditions[index]
                .exits
                .iter()
                .map(|exit| {
                    (exit.disposition == PostconditionDisposition::Discharged)
                        .then_some(exit.derivation)
                        .flatten()
                })
                .collect::<Option<Vec<_>>>();
            self.postconditions[index].aggregate =
                self.retain_postcondition_aggregate(&block, relation_ordinal, parents);
        }
    }

    fn retain_postcondition_aggregate(
        &mut self,
        block: &crate::NodePath,
        relation_ordinal: u32,
        parents: Option<Vec<DerivationId>>,
    ) -> PostconditionAggregate {
        let Some(parents) = parents.filter(|parents| !parents.is_empty()) else {
            return PostconditionAggregate {
                discharged: false,
                derivation: None,
            };
        };
        let node = self
            .derivations
            .intern(super::state::DerivationNode::PostconditionAggregate {
                block: block.clone(),
                relation_ordinal,
                parents,
            });
        self.derivations.add_root(
            DerivationRootKind::PostconditionAggregate { relation_ordinal },
            node,
        );
        PostconditionAggregate {
            discharged: true,
            derivation: Some(node),
        }
    }

    fn judge_postcondition_return(
        &mut self,
        statement: &crate::NodePath,
        states: &ProofFlowState,
        affine_result: Option<&AffineForm>,
    ) {
        if self.postconditions.is_empty() {
            return;
        }
        for index in 0..self.function.postconditions.len() {
            let postcondition = &self.function.postconditions[index];
            let Some(selected) = postcondition
                .selected_returns
                .iter()
                .find(|selected| selected.statement == *statement)
                .cloned()
            else {
                continue;
            };
            let result = self
                .postcondition_return_term(&selected.value)
                .expect("H1 selected-return datum must remain in the ENT-2 term fragment");
            let relation = self
                .instantiate_postcondition_relation(postcondition, result)
                .expect("H1 relation template must remain in the ENT-2 term fragment");
            let affine_target = affine_result.and_then(|result| {
                self.postcondition_affine_target(postcondition, result, &states.affine)
            });
            let residual = self.render_relation(&relation);
            let entry_images = self.postcondition_entry_images[index]
                .iter()
                .map(|entry_index| PostconditionEntryImageOutcome {
                    datum: self.entry_images[*entry_index].datum.clone(),
                    invalidation: states.entry_images[*entry_index],
                })
                .collect::<Vec<_>>();
            let occurrence = self.postconditions[index].exits.len();
            let relation_ordinal = self.postconditions[index].relation_ordinal;
            let unavailable = entry_images
                .iter()
                .any(|image| image.invalidation.is_some());
            let complete = self.judge_postcondition(
                relation_ordinal,
                occurrence,
                statement,
                &relation,
                &states.facts,
                affine_target.as_ref(),
                &states.affine,
                unavailable,
            );
            self.postconditions[index].exits.push(PostconditionExit {
                statement: statement.clone(),
                relation,
                residual,
                entry_images,
                disposition: complete.disposition,
                derivation: complete.derivation,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn judge_postcondition(
        &mut self,
        relation_ordinal: u32,
        occurrence: usize,
        statement: &crate::NodePath,
        relation: &Relation,
        state: &FactState,
        affine_target: Option<&AffineInequality>,
        affine_state: &AffineFlowState,
        unavailable: bool,
    ) -> PostconditionExitProof {
        let context = ProofContext::new(state, affine_state);
        if unavailable {
            return PostconditionExitProof {
                disposition: PostconditionDisposition::Unproved,
                derivation: None,
            };
        }
        let proof = self.prove(
            context,
            ProofGoal::Ordering {
                relation,
                affine: affine_target,
            },
        );
        if proof.disposition == ProofDisposition::Proved {
            let parent = proof
                .derivation
                .expect("a proved postcondition relation must retain its local derivation");
            let node = self
                .derivations
                .intern(super::state::DerivationNode::PostconditionExit {
                    statement: statement.clone(),
                    relation_ordinal,
                    relation: Box::new(relation.clone()),
                    parent,
                });
            self.derivations.add_root(
                DerivationRootKind::PostconditionExit {
                    relation_ordinal,
                    occurrence: u32::try_from(occurrence)
                        .expect("postcondition exits exceed the u32 identity space"),
                },
                node,
            );
            PostconditionExitProof {
                disposition: PostconditionDisposition::Discharged,
                derivation: Some(node),
            }
        } else {
            PostconditionExitProof {
                disposition: if proof.disposition == ProofDisposition::Refuted {
                    PostconditionDisposition::Refuted
                } else {
                    PostconditionDisposition::Unproved
                },
                derivation: None,
            }
        }
    }

    fn postcondition_affine_target(
        &self,
        postcondition: &CheckedPostcondition,
        result: &AffineForm,
        state: &AffineFlowState,
    ) -> Option<AffineInequality> {
        let operands = postcondition
            .relation
            .operands
            .iter()
            .map(|datum| self.postcondition_affine_datum(datum, result, state))
            .collect::<Option<Vec<_>>>()?;
        let NormalizedRelation::UpperBound {
            left,
            right,
            strict,
        } = postcondition.relation.normalized
        else {
            return None;
        };
        let left = operands.get(left as usize)?;
        let right = operands.get(right as usize)?;
        let right = if strict {
            right
                .subtract(&AffineForm::constant(1), &mut AffineCheckState::new())
                .ok()?
        } else {
            right.clone()
        };
        AffineInequality::from_forms(left, &right, &mut AffineCheckState::new()).ok()
    }

    fn postcondition_affine_datum(
        &self,
        datum: &RelationDatum,
        result: &AffineForm,
        state: &AffineFlowState,
    ) -> Option<AffineForm> {
        match datum {
            RelationDatum::Result {
                ty: CheckedType::Integer(_),
            } => Some(result.clone()),
            RelationDatum::Parameter {
                ordinal,
                projections,
                ty: CheckedType::Integer(_),
            } if projections.is_empty() => {
                let binding = self.function.parameters.get(*ordinal as usize)?.binding;
                state.values.get(&binding).cloned()
            }
            RelationDatum::NamedConst {
                declaration,
                projections,
                ty: CheckedType::Integer(_),
            } if projections.is_empty() => self
                .context
                .constant(*declaration)
                .and_then(|constant| Self::postcondition_affine_constant(&constant.value)),
            RelationDatum::Literal { value, .. } => Self::postcondition_affine_constant(value),
            RelationDatum::Result { .. }
            | RelationDatum::Parameter { .. }
            | RelationDatum::NamedConst { .. }
            | RelationDatum::Length(_) => None,
        }
    }

    fn postcondition_affine_constant(value: &CheckedValue) -> Option<AffineForm> {
        let value = match value {
            CheckedValue::Integer { ty, bits } => integer_value(*ty, *bits),
            CheckedValue::NumericIdentity {
                ty: CheckedType::Integer(_),
                one,
            } => i128::from(*one),
            _ => return None,
        };
        Some(AffineForm::constant(value))
    }

    /// Converts one already-substituted callable-boundary ordering predicate
    /// to its unique affine inequality. Unsupported goal shapes simply retain
    /// the ordinary L0 result; no alternate formula is guessed.
    fn affine_goal_ordering_target(
        &self,
        expression: &GoalExpression,
        state: &AffineFlowState,
    ) -> Option<AffineInequality> {
        let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation,
                    operand_type: CheckedType::Integer(_),
                },
            arguments,
            result: CheckedType::Bool,
            ..
        } = expression
        else {
            return None;
        };
        let [written_left, written_right] = arguments.as_slice() else {
            return None;
        };
        let left = self.affine_goal_value(written_left, state)?;
        let right = self.affine_goal_value(written_right, state)?;
        let mut check = AffineCheckState::new();
        let (left, right, strict) = match operation {
            CheckedIntegerOperation::LessEqual => (left, right, false),
            CheckedIntegerOperation::Less => (left, right, true),
            CheckedIntegerOperation::GreaterEqual => (right, left, false),
            CheckedIntegerOperation::Greater => (right, left, true),
            _ => return None,
        };
        let right = if strict {
            right.subtract(&AffineForm::constant(1), &mut check).ok()?
        } else {
            right
        };
        AffineInequality::from_forms(&left, &right, &mut check).ok()
    }

    /// Reads the mathematical value of the fixed affine subset admitted in a
    /// concrete call goal. Every place must be an unprojected current integer
    /// binding, and multiplication must have a literal/constant side.
    fn affine_goal_value(
        &self,
        expression: &GoalExpression,
        state: &AffineFlowState,
    ) -> Option<AffineForm> {
        match expression {
            GoalExpression::Datum(GoalDatum::Literal(value)) => {
                Self::postcondition_affine_constant(value)
            }
            GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty: CheckedType::Integer(_),
            }) if projections.is_empty() => self
                .context
                .constant(*declaration)
                .and_then(|constant| Self::postcondition_affine_constant(&constant.value)),
            GoalExpression::Datum(GoalDatum::Place {
                root,
                projections,
                ty: CheckedType::Integer(_),
            }) if projections.is_empty() => state.values.get(root).cloned(),
            GoalExpression::Operation {
                row:
                    GoalOperation::NumericConversion {
                        source: CheckedNumericType::Integer(source),
                        destination: CheckedNumericType::Integer(destination),
                    },
                arguments,
                result: CheckedType::Integer(_),
                ..
            } if source == destination || source.converts_totally_to(*destination) => {
                let [value] = arguments.as_slice() else {
                    return None;
                };
                self.affine_goal_value(value, state)
            }
            GoalExpression::Operation {
                row: GoalOperation::Integer { operation, .. },
                arguments,
                result: CheckedType::Integer(_),
                ..
            } => {
                let [left, right] = arguments.as_slice() else {
                    return None;
                };
                let left = self.affine_goal_value(left, state)?;
                let right = self.affine_goal_value(right, state)?;
                let mut check = AffineCheckState::new();
                match operation {
                    CheckedIntegerOperation::AddExact => left.add(&right, &mut check).ok(),
                    CheckedIntegerOperation::SubtractExact => {
                        left.subtract(&right, &mut check).ok()
                    }
                    CheckedIntegerOperation::MultiplyExact => {
                        if left.terms().is_empty() {
                            right.scale(left.constant_value(), &mut check).ok()
                        } else if right.terms().is_empty() {
                            left.scale(right.constant_value(), &mut check).ok()
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            GoalExpression::Datum(
                GoalDatum::Parameter { .. } | GoalDatum::EphemeralActual { .. },
            )
            | GoalExpression::Datum(GoalDatum::NamedConst { .. })
            | GoalExpression::Datum(GoalDatum::Place { .. })
            | GoalExpression::Operation { .. } => None,
        }
    }

    fn instantiate_postcondition_relation(
        &mut self,
        postcondition: &CheckedPostcondition,
        result: TermId,
    ) -> Option<Relation> {
        let operands = postcondition
            .relation
            .operands
            .iter()
            .map(|operand| self.postcondition_relation_term(operand, result))
            .collect::<Option<Vec<_>>>()?;
        let [first, second] = operands.as_slice() else {
            return None;
        };
        match postcondition.relation.normalized {
            NormalizedRelation::Equal => Some(Relation::Equal {
                left: *first,
                right: *second,
            }),
            NormalizedRelation::NotEqual => Some(Relation::Distinct {
                left: (*first).min(*second),
                right: (*first).max(*second),
            }),
            NormalizedRelation::UpperBound {
                left,
                right,
                strict,
            } => Some(Relation::Bound {
                left: *operands.get(left as usize)?,
                right: *operands.get(right as usize)?,
                bound: if strict { -1 } else { 0 },
            }),
        }
    }

    fn postcondition_relation_term(
        &mut self,
        datum: &RelationDatum,
        result: TermId,
    ) -> Option<TermId> {
        match datum {
            RelationDatum::Result { .. } => Some(result),
            RelationDatum::Parameter {
                ordinal,
                projections,
                ty,
            } => {
                let binding = self.function.parameters.get(*ordinal as usize)?.binding;
                self.postcondition_place_term(PlaceRoot::Binding(binding), projections, *ty)
            }
            RelationDatum::NamedConst {
                declaration,
                projections,
                ty,
            } => self.postcondition_named_const_term(*declaration, projections, *ty),
            RelationDatum::Literal { value, .. } => self.postcondition_constant_term(value),
            RelationDatum::Length(place) => {
                let PostconditionPlaceRoot::Parameter { ordinal } = place.root;
                let binding = self.function.parameters.get(ordinal as usize)?.binding;
                self.postcondition_length_term(
                    PlaceRoot::Binding(binding),
                    &place.projections,
                    place.ty,
                )
            }
        }
    }

    fn postcondition_return_term(&mut self, datum: &PostconditionReturnDatum) -> Option<TermId> {
        match datum {
            PostconditionReturnDatum::Place(place) => self.postcondition_return_place_term(place),
            PostconditionReturnDatum::Literal { value, .. } => {
                self.postcondition_constant_term(value)
            }
            PostconditionReturnDatum::Length(place) => {
                let root = self.postcondition_return_place_root(place.root)?;
                self.postcondition_length_term(root, &place.projections, place.ty)
            }
        }
    }

    fn postcondition_return_place_term(
        &mut self,
        place: &PostconditionReturnPlace,
    ) -> Option<TermId> {
        if let PostconditionReturnPlaceRoot::NamedConst(declaration) = place.root {
            return self.postcondition_named_const_term(declaration, &place.projections, place.ty);
        }
        let root = self.postcondition_return_place_root(place.root)?;
        self.postcondition_place_term(root, &place.projections, place.ty)
    }

    fn postcondition_return_place_root(
        &self,
        root: PostconditionReturnPlaceRoot,
    ) -> Option<PlaceRoot> {
        match root {
            PostconditionReturnPlaceRoot::Binding(binding) => Some(PlaceRoot::Binding(binding)),
            PostconditionReturnPlaceRoot::NamedConst(declaration) => Some(PlaceRoot::Constant(
                *self.context.constant_ids.get(&declaration)?,
            )),
        }
    }

    fn postcondition_named_const_term(
        &mut self,
        declaration: crate::DeclarationId,
        projections: &[GoalProjection],
        ty: CheckedType,
    ) -> Option<TermId> {
        if projections.is_empty()
            && let Some(term) = self
                .context
                .constant(declaration)
                .and_then(|constant| self.postcondition_constant_term(&constant.value))
        {
            return Some(term);
        }
        let root = PlaceRoot::Constant(*self.context.constant_ids.get(&declaration)?);
        self.postcondition_place_term(root, projections, ty)
    }

    fn postcondition_constant_term(&mut self, value: &CheckedValue) -> Option<TermId> {
        let value = match value {
            CheckedValue::Integer { ty, bits } => integer_value(*ty, *bits),
            CheckedValue::NumericIdentity {
                ty: CheckedType::Integer(_),
                one,
            } => i128::from(*one),
            _ => return None,
        };
        Some(self.terms.intern(TermKind::Constant(value)))
    }

    fn postcondition_place_term(
        &mut self,
        root: PlaceRoot,
        projections: &[GoalProjection],
        ty: CheckedType,
    ) -> Option<TermId> {
        let fragment = fragment_type(ty)?;
        let projections = projections
            .iter()
            .map(|projection| match projection {
                GoalProjection::Field(field) => PlaceProjection::Field(*field),
                GoalProjection::Deref => PlaceProjection::Deref,
            })
            .collect::<Vec<_>>();
        let path = ProjectedPlaceTerm { root, projections };
        let kind = legacy_place(&path).map_or_else(
            || TermKind::ProjectedPlace(path, fragment),
            |place| TermKind::Place(place, fragment),
        );
        Some(self.terms.intern(kind))
    }

    fn postcondition_length_term(
        &mut self,
        root: PlaceRoot,
        projections: &[GoalProjection],
        ty: CheckedType,
    ) -> Option<TermId> {
        let projections = projections
            .iter()
            .map(|projection| match projection {
                GoalProjection::Field(field) => PlaceProjection::Field(*field),
                GoalProjection::Deref => PlaceProjection::Deref,
            })
            .collect::<Vec<_>>();
        let path = ProjectedPlaceTerm { root, projections };
        let term = if let Some(place) = legacy_place(&path) {
            self.terms.intern(TermKind::Length(place))
        } else {
            self.terms.intern(TermKind::ProjectedLength(path))
        };
        if let CheckedType::Array { length, .. } = ty {
            let bound = match length {
                CheckedConst::Value(value) => Some(LengthBound::Constant(i128::from(value))),
                CheckedConst::Parameter(declaration) => Some(LengthBound::Equal(
                    self.terms.intern(TermKind::ConstParameter(declaration)),
                )),
                // A symbolic derived length has no [ENT-2] term form; the
                // template states no bound and the concrete instance, whose
                // length is a value, restates the constant bound.
                CheckedConst::Derived(_) => None,
            };
            if let Some(bound) = bound {
                self.terms.set_length_bound(term, bound);
            }
        } else if !matches!(ty, CheckedType::Buffer { .. } | CheckedType::Slice { .. }) {
            return None;
        }
        Some(term)
    }

    fn available_postconditions(
        &self,
        function: super::super::model::FunctionId,
    ) -> Vec<AvailablePostcondition> {
        self.context
            .verified_postconditions(function)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(postcondition, proof)| {
                Some(AvailablePostcondition {
                    relation: postcondition.relation.clone(),
                    variant: postcondition.selector.variant,
                    field: postcondition
                        .selector
                        .field
                        .as_ref()
                        .map(|field| field.declaration),
                    summary: proof.summary.clone()?,
                    discharged: proof.aggregate.discharged,
                })
            })
            .collect()
    }

    fn append_holder_chain(&self, binding: BindingId, holders: &mut Vec<BindingId>) {
        if !self.is_holder(binding) {
            return;
        }
        let mut chain = Vec::new();
        let _ = self.resolve_deref_with_holders(binding, 0, &mut chain);
        for holder in chain {
            if !holders.contains(&holder) {
                holders.push(holder);
            }
        }
    }

    fn collect_checked_argument_holders(
        &self,
        argument: &CheckedExpression,
        holders: &mut Vec<BindingId>,
    ) {
        match argument {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::Project { binding, .. }
            | CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => {
                self.append_holder_chain(*binding, holders);
            }
            CheckedExpression::BorrowBuffer { root, .. }
            | CheckedExpression::BufferLength { root } => {
                self.append_holder_chain(root.binding, holders);
            }
            CheckedExpression::SliceLength { root } => {
                self.append_holder_chain(root.binding, holders);
            }
            CheckedExpression::ArrayLength {
                root: CheckedArrayRoot::Binding { binding, .. },
                ..
            } => self.append_holder_chain(*binding, holders),
            // These checked wrappers are one read of their nested place. They
            // do not create a second consume, but M must retain the holder on
            // which the resulting caller image depends.
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.collect_checked_argument_holders(value, holders);
            }
            _ => {}
        }
    }

    fn collect_goal_image_holders(&self, argument: &GoalExpression, holders: &mut Vec<BindingId>) {
        match argument {
            GoalExpression::Datum(GoalDatum::Place {
                root, projections, ..
            }) => {
                let support = GoalSupport {
                    root: *root,
                    projections: projections.clone(),
                    length: false,
                };
                let (_, image_holders) = self.resolve_goal_support(&support);
                for holder in image_holders {
                    if !holders.contains(&holder) {
                        holders.push(holder);
                    }
                }
            }
            GoalExpression::Operation { arguments, .. } => {
                for argument in arguments {
                    self.collect_goal_image_holders(argument, holders);
                }
            }
            GoalExpression::Datum(_) => {}
        }
    }

    fn call_argument_holder_chain(
        &self,
        argument: &CheckedExpression,
        goal_argument: &GoalExpression,
    ) -> Vec<BindingId> {
        let mut holders = Vec::new();
        self.collect_checked_argument_holders(argument, &mut holders);
        self.collect_goal_image_holders(goal_argument, &mut holders);
        holders
    }

    fn postcondition_term_live_holders(&self, term: TermId) -> Vec<BindingId> {
        let mut holders = Vec::new();
        match self.terms.kind(term) {
            TermKind::Place(place, _) | TermKind::Length(place) => {
                if place.deref
                    && let PlaceRoot::Binding(binding) = place.root
                {
                    let _ = self.resolve_deref_with_holders(binding, 0, &mut holders);
                }
            }
            TermKind::ProjectedPlace(place, _) | TermKind::ProjectedLength(place) => {
                let PlaceRoot::Binding(root) = place.root else {
                    return holders;
                };
                let support = GoalSupport {
                    root,
                    projections: place
                        .projections
                        .iter()
                        .map(|projection| match projection {
                            PlaceProjection::Deref => GoalProjection::Deref,
                            PlaceProjection::Field(field) => GoalProjection::Field(*field),
                        })
                        .collect(),
                    length: matches!(self.terms.kind(term), TermKind::ProjectedLength(_)),
                };
                let (_, projected_holders) = self.resolve_goal_support(&support);
                holders.extend(projected_holders);
            }
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => {}
            TermKind::CountedCapture { .. } => {}
        }
        holders
    }

    fn s12_transfer_event_kills_substitution(
        &self,
        substitution: &PostconditionCallSubstitution,
        event: &KillEvent,
    ) -> bool {
        let holder_consumed = match event {
            KillEvent::Consume { binding, .. }
            | KillEvent::EntryImageHolderConsume { binding, .. } => {
                substitution.transfer_holders.contains(binding)
            }
            _ => false,
        };
        if holder_consumed {
            return true;
        }
        match event {
            KillEvent::EntryImageHolderWrite {
                place,
                element,
                source,
            } => self.event_kills_term(
                substitution.term,
                &KillEvent::Write {
                    place: place.clone(),
                    element: *element,
                    source: source.clone(),
                },
            ),
            _ => self.event_kills_term(substitution.term, event),
        }
    }

    fn s12_candidate_term_killed(&self, term: TermId, event: &KillEvent) -> bool {
        let live_holders = self.postcondition_term_live_holders(term);
        let live_holder_killed = match event {
            KillEvent::Consume { binding, .. }
            | KillEvent::EntryImageHolderConsume { binding, .. } => live_holders.contains(binding),
            KillEvent::Write {
                place,
                element: false,
                ..
            }
            | KillEvent::EntryImageHolderWrite {
                place,
                element: false,
                ..
            } => live_holders.iter().any(|holder| {
                ResolvedPlace {
                    root: PlaceRoot::Binding(*holder),
                    fields: Vec::new(),
                }
                .overlaps(place)
            }),
            KillEvent::Write { element: true, .. }
            | KillEvent::EntryImageHolderWrite { element: true, .. } => false,
        };
        if live_holder_killed {
            return true;
        }
        match event {
            KillEvent::EntryImageHolderConsume { binding, source } => self.event_kills_term(
                term,
                &KillEvent::Consume {
                    binding: *binding,
                    source: source.clone(),
                },
            ),
            KillEvent::EntryImageHolderWrite {
                place,
                element,
                source,
            } => self.event_kills_term(
                term,
                &KillEvent::Write {
                    place: place.clone(),
                    element: *element,
                    source: source.clone(),
                },
            ),
            _ => self.event_kills_term(term, event),
        }
    }

    fn s12_candidate_scope_kills_term(&self, term: TermId, exited: &HashSet<BindingId>) -> bool {
        self.scope_kills_term(term, exited)
            || self
                .postcondition_term_live_holders(term)
                .iter()
                .any(|holder| exited.contains(holder))
    }

    fn s12_substitutions_survive(
        &self,
        substitutions: &[PostconditionCallSubstitution],
        events: &[KillEvent],
    ) -> bool {
        substitutions.iter().all(|substitution| {
            events
                .iter()
                .all(|event| !self.s12_transfer_event_kills_substitution(substitution, event))
        })
    }

    fn kill_s12_candidates_for_event(&self, state: &mut FactState, event: &KillEvent) {
        state.kill_proof_candidates(&self.derivations, |left, right, proof| {
            self.derivations.depends_on_postcondition_call(proof)
                && (self.s12_candidate_term_killed(left, event)
                    || self.s12_candidate_term_killed(right, event))
        });
    }

    fn kill_s12_candidates_for_scope(&self, state: &mut FactState, exited: &HashSet<BindingId>) {
        state.kill_proof_candidates(&self.derivations, |left, right, proof| {
            self.derivations.depends_on_postcondition_call(proof)
                && (self.s12_candidate_scope_kills_term(left, exited)
                    || self.s12_candidate_scope_kills_term(right, exited))
        });
    }

    fn call_parameter_place(
        &self,
        actual: &GoalExpression,
        projections: &[GoalProjection],
    ) -> Option<(PlaceRoot, Vec<GoalProjection>)> {
        let GoalExpression::Datum(datum) = actual else {
            return None;
        };
        let (root, actual_projections) = match datum {
            GoalDatum::Place {
                root, projections, ..
            } => (PlaceRoot::Binding(*root), projections),
            GoalDatum::NamedConst {
                declaration,
                projections,
                ..
            } => (
                PlaceRoot::Constant(*self.context.constant_ids.get(declaration)?),
                projections,
            ),
            GoalDatum::Parameter { .. }
            | GoalDatum::EphemeralActual { .. }
            | GoalDatum::Literal(_) => return None,
        };
        Some((
            root,
            actual_projections
                .iter()
                .chain(projections)
                .copied()
                .collect(),
        ))
    }

    fn call_parameter_term(
        &mut self,
        actual: &GoalExpression,
        projections: &[GoalProjection],
        ty: CheckedType,
        length: bool,
        mode: CheckedMode,
    ) -> Option<TermId> {
        let projections = if mode == CheckedMode::Own {
            projections
        } else {
            let (GoalProjection::Deref, remaining) = projections.split_first()? else {
                return None;
            };
            remaining
        };
        if projections.is_empty() && !length {
            return (actual.ty() == ty)
                .then(|| self.goal_operand(actual))
                .flatten();
        }
        let (root, projections) = self.call_parameter_place(actual, projections)?;
        if length {
            self.postcondition_length_term(root, &projections, ty)
        } else {
            self.postcondition_place_term(root, &projections, ty)
        }
    }

    fn instantiate_call_postcondition_relation(
        &mut self,
        function: super::super::model::FunctionId,
        template: &RelationTemplate,
        checked_arguments: &[CheckedExpression],
        arguments: &[GoalExpression],
        result: TermId,
    ) -> Option<InstantiatedPostcondition> {
        let parameter_modes = &self.context.callee(function)?.parameter_modes;
        let mut substitutions = Vec::new();
        let mut operands = Vec::with_capacity(template.operands.len());
        for (operand, datum) in template.operands.iter().enumerate() {
            let (term, formal) = match datum {
                RelationDatum::Result { .. } => (result, None),
                RelationDatum::Parameter {
                    ordinal,
                    projections,
                    ty,
                } => (
                    self.call_parameter_term(
                        arguments.get(*ordinal as usize)?,
                        projections,
                        *ty,
                        false,
                        *parameter_modes.get(*ordinal as usize)?,
                    )?,
                    Some(*ordinal),
                ),
                RelationDatum::NamedConst {
                    declaration,
                    projections,
                    ty,
                } => (
                    self.postcondition_named_const_term(*declaration, projections, *ty)?,
                    None,
                ),
                RelationDatum::Literal { value, .. } => {
                    (self.postcondition_constant_term(value)?, None)
                }
                RelationDatum::Length(place) => {
                    let PostconditionPlaceRoot::Parameter { ordinal } = place.root;
                    (
                        self.call_parameter_term(
                            arguments.get(ordinal as usize)?,
                            &place.projections,
                            place.ty,
                            true,
                            *parameter_modes.get(ordinal as usize)?,
                        )?,
                        Some(ordinal),
                    )
                }
            };
            if let Some(formal) = formal {
                substitutions.push(PostconditionCallSubstitution {
                    operand: u32::try_from(operand)
                        .expect("postcondition operands exceed the u32 identity space"),
                    formal,
                    term,
                    transfer_holders: self.call_argument_holder_chain(
                        checked_arguments.get(formal as usize)?,
                        arguments.get(formal as usize)?,
                    ),
                });
            }
            operands.push(term);
        }
        let [first, second] = operands.as_slice() else {
            return None;
        };
        let relation = match template.normalized {
            NormalizedRelation::Equal => Relation::Equal {
                left: *first,
                right: *second,
            },
            NormalizedRelation::NotEqual => Relation::Distinct {
                left: (*first).min(*second),
                right: (*first).max(*second),
            },
            NormalizedRelation::UpperBound {
                left,
                right,
                strict,
            } => Relation::Bound {
                left: *operands.get(left as usize)?,
                right: *operands.get(right as usize)?,
                bound: if strict { -1 } else { 0 },
            },
        };
        Some(InstantiatedPostcondition {
            relation,
            substitutions,
        })
    }

    fn selected_call_summary(
        &self,
        available: &AvailablePostcondition,
    ) -> Option<VerifiedPostconditionSummaryRef> {
        available
            .discharged
            .then(|| VerifiedPostconditionSummaryRef {
                summary: available.summary.clone(),
            })
    }

    fn retain_postcondition_call(
        &mut self,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
    ) -> Option<DerivationId> {
        let summary = self.selected_call_summary(available)?;
        Some(
            self.derivations
                .intern(super::state::DerivationNode::PostconditionCall {
                    detail: Box::new(super::state::PostconditionCallDetail {
                        call: prepared.call.clone(),
                        relation: instantiated.relation.clone(),
                        summary,
                        substitutions: instantiated.substitutions.clone(),
                        transfer_events: prepared.transfer_events.clone(),
                        parents: prepared.parents.clone(),
                    }),
                }),
        )
    }

    fn retain_direct_result(
        &mut self,
        statement: &crate::NodePath,
        binding: BindingId,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let Some(call) = self.retain_postcondition_call(instantiated, available, prepared) else {
            return;
        };
        let route =
            self.derivations
                .intern(super::state::DerivationNode::PostconditionDirectResult {
                    statement: statement.clone(),
                    binding,
                    relation: Box::new(instantiated.relation.clone()),
                    parent: call,
                });
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectResult { occurrence },
            route,
        );
        state.establish_from_proof(&instantiated.relation, route, &self.derivations);
    }

    fn establish_direct_result(
        &mut self,
        statement: &crate::NodePath,
        binding: BindingId,
        value: &CheckedExpression,
        prepared: &PreparedCall,
        states: &mut ProofFlowState,
    ) {
        let CheckedExpression::UserCall {
            function,
            call,
            arguments,
            goal_arguments,
            result,
            ..
        } = value
        else {
            return;
        };
        if *function != prepared.function
            || *call != prepared.call
            || fragment_type(*result).is_none()
        {
            return;
        }
        let Some(result_term) =
            self.postcondition_place_term(PlaceRoot::Binding(binding), &[], *result)
        else {
            return;
        };
        for available in self.available_postconditions(*function) {
            if available.variant.is_some() {
                continue;
            }
            let Some(instantiated) = self.instantiate_call_postcondition_relation(
                *function,
                &available.relation,
                arguments,
                goal_arguments,
                result_term,
            ) else {
                continue;
            };
            if !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills) {
                continue;
            }
            self.retain_direct_result(
                statement,
                binding,
                &instantiated,
                &available,
                prepared,
                &mut states.facts,
            );
        }
    }

    fn receiver_argument_overlaps(
        &self,
        expression: &CheckedExpression,
        receiver: &ResolvedPlace,
    ) -> bool {
        if self
            .read_place_path(expression)
            .is_some_and(|place| self.resolve_projected(&place).overlaps(receiver))
        {
            return true;
        }
        if self
            .argument_referent(expression)
            .is_some_and(|(place, _, _)| place.overlaps(receiver))
        {
            return true;
        }
        expression_children(expression)
            .into_iter()
            .any(|child| self.receiver_argument_overlaps(child, receiver))
    }

    fn direct_receiver_route(
        &self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        prepared: &PreparedCall,
    ) -> Option<DirectReceiverRoute> {
        let CheckedSetTarget::Place(target) = target else {
            return None;
        };
        let CheckedExpression::UserCall {
            function,
            call,
            arguments,
            result,
            ..
        } = value
        else {
            return None;
        };
        if *function != prepared.function
            || *call != prepared.call
            || !target.fields.is_empty()
            || self.is_holder(target.binding)
            || *result != target.ty
            || fragment_type(target.ty).is_none()
        {
            return None;
        }
        let receiver = ResolvedPlace {
            root: PlaceRoot::Binding(target.binding),
            fields: Vec::new(),
        };
        let mut selected = None;
        for (formal, argument) in arguments.iter().enumerate() {
            let exact = matches!(
                argument,
                CheckedExpression::Binding {
                    binding,
                    ty,
                    consume_root: false,
                    ..
                } if *binding == target.binding && *ty == target.ty
            );
            if exact {
                if selected.is_some() {
                    return None;
                }
                selected = Some(
                    u32::try_from(formal)
                        .expect("call argument ordinal exceeds the u32 identity space"),
                );
            } else if self.receiver_argument_overlaps(argument, &receiver) {
                return None;
            }
        }
        Some(DirectReceiverRoute {
            binding: target.binding,
            formal: selected?,
            ty: target.ty,
        })
    }

    fn retain_direct_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &DirectReceiverCandidate,
        target_event: FlowEventId,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let Some(call) =
            self.retain_postcondition_call(&candidate.instantiated, &candidate.available, prepared)
        else {
            return;
        };
        let proof =
            self.derivations
                .intern(super::state::DerivationNode::PostconditionDirectReceiver {
                    statement: statement.clone(),
                    binding: candidate.route.binding,
                    receiver_formal: candidate.route.formal,
                    relation: Box::new(candidate.instantiated.relation.clone()),
                    target_event,
                    parent: call,
                });
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectReceiver { occurrence },
            proof,
        );
        state.establish_from_proof(&candidate.instantiated.relation, proof, &self.derivations);
    }

    fn prepare_direct_receiver(
        &mut self,
        route: DirectReceiverRoute,
        value: &CheckedExpression,
        prepared: &PreparedCall,
        target_events: &[KillEvent],
    ) -> Vec<DirectReceiverCandidate> {
        let CheckedExpression::UserCall {
            function,
            arguments,
            goal_arguments,
            ..
        } = value
        else {
            return Vec::new();
        };
        let Some(result_term) =
            self.postcondition_place_term(PlaceRoot::Binding(route.binding), &[], route.ty)
        else {
            return Vec::new();
        };
        self.available_postconditions(*function)
            .into_iter()
            .filter_map(|available| {
                if available.variant.is_some() {
                    return None;
                }
                let instantiated = self.instantiate_call_postcondition_relation(
                    *function,
                    &available.relation,
                    arguments,
                    goal_arguments,
                    result_term,
                )?;
                if instantiated
                    .substitutions
                    .iter()
                    .any(|substitution| substitution.formal == route.formal)
                    || !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills)
                    || !self.s12_substitutions_survive(&instantiated.substitutions, target_events)
                {
                    return None;
                }
                Some(DirectReceiverCandidate {
                    route,
                    available,
                    instantiated,
                })
            })
            .collect()
    }

    fn establish_direct_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &DirectReceiverCandidate,
        prepared: &PreparedCall,
        target_event: FlowEventId,
        states: &mut ProofFlowState,
    ) {
        self.retain_direct_receiver(
            statement,
            candidate,
            target_event,
            prepared,
            &mut states.facts,
        );
    }

    fn retain_direct_match(
        &mut self,
        route: DirectMatchRoute,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) -> Option<DerivationId> {
        let call = self.retain_postcondition_call(instantiated, available, prepared)?;
        let route =
            self.derivations
                .intern(super::state::DerivationNode::PostconditionDirectMatch {
                    call: prepared.call.clone(),
                    variant: route.variant,
                    field: route.field,
                    tag: route.tag,
                    binding: route.binding,
                    relation: Box::new(instantiated.relation.clone()),
                    parent: call,
                });
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectMatch { occurrence },
            route,
        );
        state.establish_from_proof(&instantiated.relation, route, &self.derivations);
        Some(route)
    }

    fn establish_direct_match(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        arm: &CheckedMatchArm,
        prepared: &PreparedCall,
        states: &mut ProofFlowState,
    ) -> Vec<EstablishedDirectMatch> {
        let CheckedExpression::UserCall {
            function,
            call,
            arguments,
            goal_arguments,
            result: CheckedType::Nominal(result_nominal),
            ..
        } = scrutinee
        else {
            return Vec::new();
        };
        let CheckedEnumType::Nominal(match_nominal) = enum_type else {
            return Vec::new();
        };
        if *function != prepared.function
            || *call != prepared.call
            || *result_nominal != match_nominal
        {
            return Vec::new();
        }
        let Some(nominal) = self.context.nominals.get(result_nominal.0 as usize) else {
            return Vec::new();
        };
        let CheckedNominalKind::Enum { variants } = &nominal.kind else {
            return Vec::new();
        };
        let Some(binder) = arm.binders.iter().find(|binder| binder.field == 0) else {
            return Vec::new();
        };
        let Some(result_term) =
            self.postcondition_place_term(PlaceRoot::Binding(binder.binding), &[], binder.ty)
        else {
            return Vec::new();
        };
        let mut established = Vec::new();
        for available in self.available_postconditions(*function) {
            let (Some(selector_variant), Some(selector_field)) =
                (available.variant, available.field)
            else {
                continue;
            };
            let Some(variant) = variants.iter().find(|variant| {
                variant.tag == arm.tag
                    && variant.constructor == CheckedConstructor::Prelude(selector_variant)
            }) else {
                continue;
            };
            let [selected_field] = variant.fields.as_slice() else {
                continue;
            };
            if binder.mode != CheckedMode::Own
                || binder.ty != selected_field.ty
                || fragment_type(binder.ty).is_none()
            {
                continue;
            }
            let Some(instantiated) = self.instantiate_call_postcondition_relation(
                *function,
                &available.relation,
                arguments,
                goal_arguments,
                result_term,
            ) else {
                continue;
            };
            if !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills) {
                continue;
            }
            let route = DirectMatchRoute {
                variant: selector_variant,
                field: selector_field,
                tag: arm.tag,
                binding: binder.binding,
                ty: binder.ty,
            };
            let parent = self.retain_direct_match(
                route,
                &instantiated,
                &available,
                prepared,
                &mut states.facts,
            );
            established.push(EstablishedDirectMatch {
                route,
                instantiated,
                parent,
            });
        }
        established
    }

    fn replace_relation_term(relation: &Relation, from: TermId, to: TermId) -> Relation {
        let replace = |term| if term == from { to } else { term };
        match relation {
            Relation::Bound { left, right, bound } => Relation::Bound {
                left: replace(*left),
                right: replace(*right),
                bound: *bound,
            },
            Relation::Equal { left, right } => Relation::Equal {
                left: replace(*left),
                right: replace(*right),
            },
            Relation::Distinct { left, right } => {
                let left = replace(*left);
                let right = replace(*right);
                Relation::Distinct {
                    left: left.min(right),
                    right: left.max(right),
                }
            }
        }
    }

    fn prepare_selected_receiver(
        &mut self,
        arm: &CheckedMatchArm,
        statement: &CheckedStatement,
        scrutinee: &CheckedExpression,
        direct_match: &EstablishedDirectMatch,
    ) -> Option<SelectedReceiverCandidate> {
        let CheckedStatement::Set {
            node_path,
            target: CheckedSetTarget::Place(target),
            value:
                CheckedExpression::Binding {
                    binding: payload,
                    ty,
                    consume_root: false,
                    ..
                },
        } = statement
        else {
            return None;
        };
        if *payload != direct_match.route.binding
            || *ty != direct_match.route.ty
            || !target.fields.is_empty()
            || target.ty != direct_match.route.ty
            || self.is_holder(target.binding)
            || fragment_type(target.ty).is_none()
            || arm
                .binders
                .iter()
                .any(|binder| binder.binding == target.binding)
        {
            return None;
        }
        let CheckedExpression::UserCall { arguments, .. } = scrutinee else {
            return None;
        };
        let receiver = ResolvedPlace {
            root: PlaceRoot::Binding(target.binding),
            fields: Vec::new(),
        };
        if arguments
            .iter()
            .any(|argument| self.receiver_argument_overlaps(argument, &receiver))
            || self.block_has_reaching_write(&arm.body[1..], &receiver)
        {
            return None;
        }
        let target_kill = KillEvent::Write {
            place: receiver,
            element: false,
            source: node_path.clone(),
        };
        if !self.s12_substitutions_survive(
            &direct_match.instantiated.substitutions,
            std::slice::from_ref(&target_kill),
        ) {
            return None;
        }
        let payload_term = self.postcondition_place_term(
            PlaceRoot::Binding(direct_match.route.binding),
            &[],
            direct_match.route.ty,
        )?;
        let receiver_term =
            self.postcondition_place_term(PlaceRoot::Binding(target.binding), &[], target.ty)?;
        let relation = Self::replace_relation_term(
            &direct_match.instantiated.relation,
            payload_term,
            receiver_term,
        );
        (relation != direct_match.instantiated.relation).then_some(SelectedReceiverCandidate {
            route: SelectedReceiverRoute {
                payload: *payload,
                binding: target.binding,
            },
            relation,
            parent: direct_match.parent,
        })
    }

    fn kill_writes_place(event: &KillEvent, place: &ResolvedPlace) -> bool {
        match event {
            KillEvent::Write { place: written, .. }
            | KillEvent::EntryImageHolderWrite { place: written, .. } => written.overlaps(place),
            KillEvent::Consume { .. } | KillEvent::EntryImageHolderConsume { .. } => false,
        }
    }

    fn expression_writes_place(
        &self,
        expression: &CheckedExpression,
        place: &ResolvedPlace,
    ) -> bool {
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        events
            .iter()
            .any(|event| Self::kill_writes_place(event, place))
    }

    fn set_target_writes_place(&self, target: &CheckedSetTarget, place: &ResolvedPlace) -> bool {
        let target = match target {
            CheckedSetTarget::Place(target) => PlaceTerm {
                root: PlaceRoot::Binding(target.binding),
                deref: self.is_holder(target.binding),
                fields: target.fields.clone(),
            },
            CheckedSetTarget::ArrayIndex(target) => PlaceTerm {
                root: PlaceRoot::Binding(target.binding),
                deref: self.is_holder(target.binding),
                fields: target.fields.clone(),
            },
            CheckedSetTarget::BufferIndex(target) => PlaceTerm {
                root: PlaceRoot::Binding(target.root.binding),
                deref: self.is_holder(target.root.binding),
                fields: target.root.fields.clone(),
            },
        };
        self.resolve(&target).overlaps(place)
    }

    /// Whether a structurally reachable statement in this block writes the
    /// selected receiver. A terminating statement stops later siblings, while
    /// every reachable nested arm or body is inspected.
    fn block_has_reaching_write(
        &self,
        statements: &[CheckedStatement],
        place: &ResolvedPlace,
    ) -> bool {
        for statement in statements {
            if self.statement_has_reaching_write(statement, place) {
                return true;
            }
            if !self.statement_falls_through(statement) {
                break;
            }
        }
        false
    }

    fn statement_has_reaching_write(
        &self,
        statement: &CheckedStatement,
        place: &ResolvedPlace,
    ) -> bool {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. }
            | CheckedStatement::PropagateLet {
                scrutinee: value, ..
            } => self.expression_writes_place(value, place),
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                self.expression_writes_place(value, place)
                    || self.set_target_writes_place(target, place)
            }
            CheckedStatement::Break { .. } | CheckedStatement::Proof(_) => false,
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                self.expression_writes_place(scrutinee, place)
                    || arms
                        .iter()
                        .any(|arm| self.block_has_reaching_write(&arm.body, place))
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                self.block_has_reaching_write(body, place)
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                self.expression_writes_place(lower, place)
                    || self.expression_writes_place(upper, place)
                    || self.block_has_reaching_write(body, place)
            }
        }
    }

    fn statement_falls_through(&self, statement: &CheckedStatement) -> bool {
        match statement {
            CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => false,
            CheckedStatement::Match { continues, .. }
            | CheckedStatement::ValueMatchLet { continues, .. } => *continues,
            CheckedStatement::Region { body, .. } => body
                .iter()
                .all(|statement| self.statement_falls_through(statement)),
            CheckedStatement::Let { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Proof(_)
            | CheckedStatement::Loop { .. }
            | CheckedStatement::CountedRange { .. } => true,
        }
    }

    fn retain_selected_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &SelectedReceiverCandidate,
        target_event: FlowEventId,
        parent: Option<DerivationId>,
        state: &mut FactState,
    ) {
        let Some(parent) = parent else {
            return;
        };
        let proof = self.derivations.intern(
            super::state::DerivationNode::PostconditionSelectedReceiver {
                statement: statement.clone(),
                payload: candidate.route.payload,
                binding: candidate.route.binding,
                relation: Box::new(candidate.relation.clone()),
                target_event,
                parent,
            },
        );
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionSelectedReceiver { occurrence },
            proof,
        );
        state.establish_from_proof(&candidate.relation, proof, &self.derivations);
    }

    fn establish_selected_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &SelectedReceiverCandidate,
        target_event: FlowEventId,
        states: &mut ProofFlowState,
    ) {
        self.retain_selected_receiver(
            statement,
            candidate,
            target_event,
            candidate.parent,
            &mut states.facts,
        );
    }

    fn retain_s7_derivation(&mut self, source: S7Derivation) {
        let occurrence = u32::try_from(self.s7_derivations.len())
            .expect("S7 source roots exceed the u32 identity space");
        let kind = match &source.kind {
            super::S7DerivationKind::BitAndBound { .. } => {
                DerivationRootKind::BitAndBound(occurrence)
            }
            super::S7DerivationKind::ShiftOneNonzero { .. } => {
                DerivationRootKind::ShiftOneNonzero(occurrence)
            }
            super::S7DerivationKind::UnsignedDivisionBound { .. } => {
                DerivationRootKind::UnsignedDivisionBound(occurrence)
            }
            super::S7DerivationKind::UnsignedRemainderBound { .. } => {
                DerivationRootKind::UnsignedRemainderBound(occurrence)
            }
            super::S7DerivationKind::SignedRemainderBound { .. } => {
                DerivationRootKind::SignedRemainderBound(occurrence)
            }
        };
        self.derivations.add_root(kind, source.parent);
        self.s7_derivations.push(source);
    }

    fn retain_counted_derivations(&mut self, occurrence: u32, counted: CountedDerivationSet) {
        assert_eq!(
            occurrence, self.completed_counted_roots,
            "counted S11 groups must complete in statement-walk order"
        );
        let atoms = [
            (
                CountedRootAtom::LowerCaptureToEndpoint,
                counted.lower_capture_eq_endpoint.forward.parent,
            ),
            (
                CountedRootAtom::LowerEndpointToCapture,
                counted.lower_capture_eq_endpoint.reverse.parent,
            ),
            (
                CountedRootAtom::UpperCaptureToEndpoint,
                counted.upper_capture_eq_endpoint.forward.parent,
            ),
            (
                CountedRootAtom::UpperEndpointToCapture,
                counted.upper_capture_eq_endpoint.reverse.parent,
            ),
            (
                CountedRootAtom::BinderToLowerCapture,
                counted.binder_eq_lower_capture.forward.parent,
            ),
            (
                CountedRootAtom::LowerCaptureToBinder,
                counted.binder_eq_lower_capture.reverse.parent,
            ),
            (
                CountedRootAtom::LowerCaptureLeBinder,
                counted.lower_capture_le_binder.atomic.parent,
            ),
            (
                CountedRootAtom::BinderLtUpperCapture,
                counted.binder_lt_upper_capture.atomic.parent,
            ),
        ];
        for (atom, parent) in atoms {
            self.derivations
                .add_root(DerivationRootKind::CountedS11 { occurrence, atom }, parent);
        }
        self.counted_derivations.push(counted);
        self.completed_counted_roots = self
            .completed_counted_roots
            .checked_add(1)
            .expect("counted S11 root groups exceed the u32 identity space");
    }

    fn proof_event(
        &mut self,
        kind: FlowEventKind,
        node_path: Option<&crate::NodePath>,
    ) -> FlowEventId {
        self.derivations.event(kind, node_path.cloned())
    }

    fn expression_node_path(expression: &CheckedExpression) -> Option<&crate::NodePath> {
        expression.carrier()
    }

    // ------------------------------------------------------------------
    // Binding prepass
    // ------------------------------------------------------------------

    fn summary(&self, binding: BindingId) -> Option<&BindingSummary> {
        self.places.summary(binding)
    }

    fn collect_bindings(&mut self) {
        self.places = PlaceMap::for_function(self.function);
    }

    fn collect_postcondition_entry_images(&mut self) {
        let mut data = Vec::new();
        let mut relation_images = Vec::with_capacity(self.function.postconditions.len());
        for postcondition in &self.function.postconditions {
            let mut indices = Vec::new();
            for operand in &postcondition.relation.operands {
                let datum = match operand {
                    RelationDatum::Parameter {
                        ordinal,
                        projections,
                        ..
                    } => Some(PostconditionEntryImage {
                        parameter: *ordinal,
                        projections: projections.clone(),
                        length: false,
                    }),
                    RelationDatum::Length(place) => match place.root {
                        PostconditionPlaceRoot::Parameter { ordinal } => {
                            Some(PostconditionEntryImage {
                                parameter: ordinal,
                                projections: place.projections.clone(),
                                length: true,
                            })
                        }
                    },
                    RelationDatum::Result { .. }
                    | RelationDatum::NamedConst { .. }
                    | RelationDatum::Literal { .. } => None,
                };
                if let Some(datum) = datum {
                    let index = data
                        .iter()
                        .position(|existing| existing == &datum)
                        .unwrap_or_else(|| {
                            let index = data.len();
                            data.push(datum);
                            index
                        });
                    if !indices.contains(&index) {
                        indices.push(index);
                    }
                }
            }
            relation_images.push(indices);
        }
        self.entry_images = data
            .into_iter()
            .map(|datum| {
                let parameter = self
                    .function
                    .parameters
                    .get(datum.parameter as usize)
                    .expect("checked postcondition parameter ordinal must resolve");
                let support = GoalSupport {
                    root: parameter.binding,
                    projections: datum.projections.clone(),
                    length: datum.length,
                };
                let (place, holders) = self.resolve_goal_support(&support);
                EntryImageRecord {
                    datum,
                    place,
                    holders,
                }
            })
            .collect();
        self.postcondition_entry_images = relation_images;
    }

    fn is_holder(&self, binding: BindingId) -> bool {
        self.places.is_holder(binding)
    }

    fn needs_implicit_deref(&self, binding: BindingId) -> bool {
        self.places
            .summary(binding)
            .is_some_and(|summary| summary.implicit_deref)
    }

    // ------------------------------------------------------------------
    // Place resolution and support
    // ------------------------------------------------------------------

    fn resolve(&self, place: &PlaceTerm) -> ResolvedPlace {
        self.places.resolve(place)
    }

    fn resolve_projected(&self, place: &ProjectedPlaceTerm) -> ResolvedPlace {
        self.places.resolve_projected(place)
    }

    /// Whether a kill event kills a fact supported by `term` [ENT-5].
    fn event_kills_term(&self, term: TermId, event: &KillEvent) -> bool {
        match self.terms.kind(term).clone() {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            // Counted captures are immutable. Their construct-scope exit is
            // handled separately from source-place write/consume events.
            TermKind::CountedCapture { .. } => false,
            TermKind::Place(place, _) => match event {
                KillEvent::Write { place: written, .. }
                | KillEvent::EntryImageHolderWrite { place: written, .. } => {
                    self.resolve(&place).overlaps(written)
                }
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. } => false,
            },
            TermKind::ProjectedPlace(place, _) => match event {
                KillEvent::Write { place: written, .. }
                | KillEvent::EntryImageHolderWrite { place: written, .. } => {
                    self.resolve_projected(&place).overlaps(written)
                }
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. } => false,
            },
            TermKind::Length(place) => match event {
                // An element write never kills a length fact: the length is
                // fixed at allocation or by the type [ENT-5].
                KillEvent::Write { element: true, .. }
                | KillEvent::EntryImageHolderWrite { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                    ..
                }
                | KillEvent::EntryImageHolderWrite {
                    place: written,
                    element: false,
                    ..
                } => {
                    let root = PlaceTerm {
                        root: place.root,
                        deref: place.deref,
                        fields: Vec::new(),
                    };
                    self.resolve(&root).overlaps(written)
                }
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. } => false,
            },
            TermKind::ProjectedLength(place) => match event {
                KillEvent::Write { element: true, .. }
                | KillEvent::EntryImageHolderWrite { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                    ..
                }
                | KillEvent::EntryImageHolderWrite {
                    place: written,
                    element: false,
                    ..
                } => self.resolve_projected(&place).overlaps(written),
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. } => false,
            },
        }
    }

    /// Whether leaving the scopes of `exited` kills a fact supported by
    /// `term`: the support contains every tracked place's root binding and
    /// every holder read through, which is the spelling root here.
    fn scope_kills_term(&self, term: TermId, exited: &HashSet<BindingId>) -> bool {
        match self.terms.kind(term) {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            TermKind::CountedCapture { .. } => false,
            TermKind::Place(place, _) | TermKind::Length(place) => match place.root {
                PlaceRoot::Binding(binding) => exited.contains(&binding),
                PlaceRoot::Constant(_) => false,
            },
            TermKind::ProjectedPlace(place, _) | TermKind::ProjectedLength(place) => {
                match place.root {
                    PlaceRoot::Binding(binding) => exited.contains(&binding),
                    PlaceRoot::Constant(_) => false,
                }
            }
        }
    }

    fn resolve_goal_support(&self, support: &GoalSupport) -> (ResolvedPlace, Vec<BindingId>) {
        let mut resolved = ResolvedPlace {
            root: PlaceRoot::Binding(support.root),
            fields: Vec::new(),
        };
        let mut holders = Vec::new();
        for projection in &support.projections {
            match projection {
                GoalProjection::Field(field) => resolved.fields.push(*field),
                GoalProjection::Deref => {
                    if resolved.fields.is_empty()
                        && let PlaceRoot::Binding(binding) = resolved.root
                    {
                        resolved = self.resolve_deref_with_holders(binding, 0, &mut holders);
                    } else if let PlaceRoot::Binding(binding) = resolved.root {
                        holders.push(binding);
                    }
                }
            }
        }
        (resolved, holders)
    }

    fn resolve_deref_with_holders(
        &self,
        holder: BindingId,
        depth: usize,
        holders: &mut Vec<BindingId>,
    ) -> ResolvedPlace {
        self.places
            .resolve_deref_with_holders(holder, depth, holders)
    }

    fn event_kills_goal(&self, goal: GoalId, event: &KillEvent) -> bool {
        self.goals.support(goal).iter().any(|support| {
            let (place, holders) = self.resolve_goal_support(support);
            match event {
                KillEvent::Write { element: true, .. }
                | KillEvent::EntryImageHolderWrite { element: true, .. }
                    if support.length =>
                {
                    false
                }
                KillEvent::Write { place: written, .. }
                | KillEvent::EntryImageHolderWrite { place: written, .. } => {
                    place.overlaps(written)
                }
                KillEvent::Consume { binding, .. } => {
                    holders.contains(binding) || place.root == PlaceRoot::Binding(*binding)
                }
                KillEvent::EntryImageHolderConsume { .. } => false,
            }
        })
    }

    /// An ordinary-let origin is available only while the binding whose
    /// initializer it describes has not itself been written or consumed.
    /// This key guard is separate from the goal's value support: invalidating
    /// it stops future alias expansion without erasing a signed snapshot fact
    /// that an earlier branch already established.
    fn event_kills_goal_origin_binding(&self, binding: BindingId, event: &KillEvent) -> bool {
        match event {
            KillEvent::Write { place, .. } | KillEvent::EntryImageHolderWrite { place, .. } => {
                ResolvedPlace {
                    root: PlaceRoot::Binding(binding),
                    fields: Vec::new(),
                }
                .overlaps(place)
            }
            KillEvent::Consume {
                binding: consumed, ..
            } => binding == *consumed,
            KillEvent::EntryImageHolderConsume { .. } => false,
        }
    }

    fn scope_kills_goal(&self, goal: GoalId, exited: &HashSet<BindingId>) -> bool {
        self.goals.support(goal).iter().any(|support| {
            let (place, holders) = self.resolve_goal_support(support);
            holders.iter().any(|holder| exited.contains(holder))
                || matches!(place.root, PlaceRoot::Binding(binding) if exited.contains(&binding))
        })
    }

    /// Contradiction is absorbing. Promote the complete combined closure
    /// before every kill entry so a write cannot erase one premise and make
    /// an unreachable point reachable again.
    fn promote_contradiction(&mut self, state: &mut FactState) {
        if !state.all_derivable && contradiction_without_proofs(state, &self.terms, &self.goals) {
            let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
            if closed.contradictory() {
                state.all_derivable = true;
                state.contradiction = closed.contradiction_proof();
            }
        }
    }

    fn promote_flow_contradiction(&mut self, states: &mut ProofFlowState) {
        self.promote_contradiction(&mut states.facts);
    }

    /// Preserves only survivor-to-survivor difference bounds whose internal
    /// graph vertices this event batch is about to invalidate. The event
    /// overlap logic remains the single authority for place, holder, and alias
    /// kills; projection changes neither that set nor any goal support.
    fn project_event_killed_bounds(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        let killed = state
            .bound_terms()
            .into_iter()
            .filter(|term| {
                events
                    .iter()
                    .any(|event| self.event_kills_term(*term, event))
            })
            .collect::<Vec<_>>();
        if killed.is_empty() {
            return;
        }
        let term_count = u32::try_from(self.terms.ids().count())
            .expect("ENT term inventory exceeds the u32 identity space");
        state.project_bounds_through_killed(&killed, term_count, &mut self.derivations);
    }

    fn apply_kills_one(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        self.project_event_killed_bounds(state, events);
        state.kill(|term| {
            events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
        for event in events {
            self.kill_s12_candidates_for_event(state, event);
        }
        state.kill_goals(|goal| {
            events
                .iter()
                .any(|event| self.event_kills_goal(goal, event))
        });
        state.goal_origins.retain(|binding, _| {
            !events
                .iter()
                .any(|event| self.event_kills_goal_origin_binding(*binding, event))
        });
        state.ambiguous_goal_origins.retain(|binding| {
            !events
                .iter()
                .any(|event| self.event_kills_goal_origin_binding(*binding, event))
        });
    }

    fn apply_kills(&mut self, states: &mut ProofFlowState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        self.promote_flow_contradiction(states);
        self.apply_kills_one(&mut states.facts, events);
        Self::apply_affine_kills(&mut states.affine, events);
        self.invalidate_entry_images(states, events, None);
    }

    fn event_kills_entry_image(&self, image: &EntryImageRecord, event: &KillEvent) -> bool {
        match event {
            KillEvent::Write { element: true, .. }
            | KillEvent::EntryImageHolderWrite { element: true, .. }
                if image.datum.length =>
            {
                false
            }
            KillEvent::Write { place, .. } | KillEvent::EntryImageHolderWrite { place, .. } => {
                image.place.overlaps(place)
            }
            KillEvent::Consume { binding, .. }
            | KillEvent::EntryImageHolderConsume { binding, .. } => {
                image.holders.contains(binding) || image.place.root == PlaceRoot::Binding(*binding)
            }
        }
    }

    fn invalidate_entry_images(
        &mut self,
        states: &mut ProofFlowState,
        events: &[KillEvent],
        shared_event: Option<FlowEventId>,
    ) {
        if self.entry_images.is_empty() {
            return;
        }
        for event in events {
            let killed = self
                .entry_images
                .iter()
                .enumerate()
                .filter_map(|(index, image)| {
                    (states.entry_images[index].is_none()
                        && self.event_kills_entry_image(image, event))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            if killed.is_empty() {
                continue;
            }
            let invalidation = shared_event.unwrap_or_else(|| {
                self.proof_event(
                    FlowEventKind::PostconditionEntryImageInvalidation,
                    Some(event.source()),
                )
            });
            for index in killed {
                states.entry_images[index] = Some(invalidation);
            }
        }
    }

    /// Applies the scope-exit kills for every scope deeper than `depth`,
    /// as the edge event ordered before any join [ENT-5].
    fn exit_scopes_to_one(&mut self, state: &mut FactState, depth: usize) {
        let exited: HashSet<BindingId> =
            self.scopes.iter().skip(depth).flatten().copied().collect();
        if exited.is_empty() {
            return;
        }
        state.kill(|term| self.scope_kills_term(term, &exited));
        self.kill_s12_candidates_for_scope(state, &exited);
        state.kill_goals(|goal| self.scope_kills_goal(goal, &exited));
        state.origins.retain(|binding, _| !exited.contains(binding));
        state
            .outcomes
            .retain(|binding, _| !exited.contains(binding));
        state
            .goal_origins
            .retain(|binding, _| !exited.contains(binding));
        state
            .ambiguous_goal_origins
            .retain(|binding| !exited.contains(binding));
    }

    /// Applies only the lexical support kills. Delivery images call this
    /// directly because they were already constructed from a closed source
    /// state and must retain only their explicit PostconditionGive roots.
    fn kill_scopes_to(&mut self, states: &mut ProofFlowState, depth: usize) {
        self.promote_flow_contradiction(states);
        self.exit_scopes_to_one(&mut states.facts, depth);
        self.exit_affine_scopes_to(&mut states.affine, depth);
    }

    fn exit_affine_scopes_to(&self, state: &mut AffineFlowState, depth: usize) {
        let exited = self
            .scopes
            .iter()
            .skip(depth)
            .flatten()
            .copied()
            .collect::<HashSet<_>>();
        state.values.retain(|binding, _| !exited.contains(binding));
    }

    fn exit_scopes_to(&mut self, states: &mut ProofFlowState, depth: usize) {
        let has_exited_bindings = self
            .scopes
            .iter()
            .skip(depth)
            .any(|scope| !scope.is_empty());
        if !has_exited_bindings {
            return;
        }
        // [ENT-4, ENT-5]: a local term may be the middle vertex of a proof
        // whose conclusion names only values that remain live. Fix the least
        // closure while that vertex still exists, then let the ordinary scope
        // kill remove every materialized fact whose own support still names
        // the exiting scope.
        let snapshot = self.derivations.event(FlowEventKind::Snapshot, None);
        states.facts = materialize_closure_at(
            &states.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
            snapshot,
        );
        // Materialization has already promoted any relation or goal
        // contradiction. Apply only the endpoint projection here.
        self.exit_scopes_to_one(&mut states.facts, depth);
        self.exit_affine_scopes_to(&mut states.affine, depth);
    }

    /// Applies the private capture-scope kill of one counted construct.
    fn exit_counted_capture_scope_one(&mut self, state: &mut FactState, range_path: &[u32]) {
        state.kill(|term| {
            matches!(
                self.terms.kind(term),
                TermKind::CountedCapture { range_path: path, .. } if path == range_path
            )
        });
    }

    fn exit_counted_capture_scope(&mut self, states: &mut ProofFlowState, range_path: &[u32]) {
        self.promote_flow_contradiction(states);
        self.exit_counted_capture_scope_one(&mut states.facts, range_path);
    }

    fn remove_active_loop_invariants(state: &mut AffineFlowState, loop_id: CheckedLoopId) {
        let retain = |assumption: &ActiveAffineFact| {
            !matches!(
                assumption.source,
                SourceAffineFactRef::LoopInvariant(SourceLoopInvariantRef {
                    loop_id: source_loop,
                    ..
                }) if source_loop == loop_id
            )
        };
        state.assumptions.retain(retain);
    }

    fn affine_facts(state: &AffineFlowState) -> Vec<ActiveAffineFact> {
        let (exports, active, proofs) = (&state.exports, &state.assumptions, &state.proofs);
        let mut facts = Vec::with_capacity(exports.len() + active.len() + proofs.len());
        facts.extend(exports.iter().cloned());
        facts.extend(active.iter().cloned());
        facts.extend(proofs.iter().cloned());
        facts
    }

    fn affine_fact_uses_only_outer_values(
        inequality: &AffineInequality,
        state: &AffineFlowState,
        binder: BindingId,
    ) -> bool {
        let live_terms = state
            .values
            .iter()
            .filter(|(binding, _)| **binding != binder)
            .flat_map(|(_, value)| value.terms().iter().map(|coefficient| coefficient.term()))
            .collect::<HashSet<_>>();
        inequality
            .terms()
            .iter()
            .all(|coefficient| live_terms.contains(&coefficient.term()))
    }

    /// Applies capture-scope kills for every loop frame crossed by a
    /// non-local edge. Ordinary loop frames carry no private captures.
    fn exit_counted_loops_from(&mut self, states: &mut ProofFlowState, loop_depth: usize) {
        let loops = self
            .loops
            .iter()
            .skip(loop_depth)
            .map(|frame| (frame.id, frame.capture_path.clone()))
            .collect::<Vec<_>>();
        for (loop_id, path) in loops {
            if let Some(path) = path {
                self.exit_counted_capture_scope(states, &path);
            }
            Self::remove_active_loop_invariants(&mut states.affine, loop_id);
        }
    }

    // ------------------------------------------------------------------
    // Terms and relations from checked expressions
    // ------------------------------------------------------------------

    /// Reads an expression as a term or constant [ENT-2]; anything else is
    /// no operand and establishes or derives nothing.
    fn read_operand(&mut self, expression: &CheckedExpression) -> Option<TermId> {
        match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
            | CheckedExpression::NamedConstant {
                value: CheckedValue::Integer { ty, bits },
                ..
            } => {
                return Some(
                    self.terms
                        .intern(TermKind::Constant(integer_value(*ty, *bits))),
                );
            }
            _ => {}
        }
        let fragment = fragment_type(expression.ty())?;
        let path = self.read_place_path(expression)?;
        let kind = match path.projections.as_slice() {
            projections
                if projections
                    .iter()
                    .all(|projection| matches!(projection, PlaceProjection::Field(_))) =>
            {
                TermKind::Place(
                    PlaceTerm {
                        root: path.root,
                        deref: false,
                        fields: projections
                            .iter()
                            .filter_map(|projection| match projection {
                                PlaceProjection::Field(field) => Some(*field),
                                PlaceProjection::Deref => None,
                            })
                            .collect(),
                    },
                    fragment,
                )
            }
            _ => TermKind::ProjectedPlace(path, fragment),
        };
        Some(self.terms.intern(kind))
    }

    /// Reconstructs the exact source-order place path retained by the checked
    /// expression. This is deliberately recursive: field selection may occur
    /// before or after a deref, and nested boxes may introduce more than one
    /// deref. [ENT-2] distinguishes those canonical spellings.
    fn read_place_path(&self, expression: &CheckedExpression) -> Option<ProjectedPlaceTerm> {
        match expression {
            CheckedExpression::Binding { binding, .. } => Some(ProjectedPlaceTerm {
                root: PlaceRoot::Binding(*binding),
                projections: self
                    .needs_implicit_deref(*binding)
                    .then_some(PlaceProjection::Deref)
                    .into_iter()
                    .collect(),
            }),
            CheckedExpression::Project {
                binding,
                fields,
                consume_root: false,
                ..
            } => Some(ProjectedPlaceTerm {
                root: PlaceRoot::Binding(*binding),
                projections: self
                    .needs_implicit_deref(*binding)
                    .then_some(PlaceProjection::Deref)
                    .into_iter()
                    .chain(fields.iter().copied().map(PlaceProjection::Field))
                    .collect(),
            }),
            CheckedExpression::DerefAddressed { binding, .. } => Some(ProjectedPlaceTerm {
                root: PlaceRoot::Binding(*binding),
                projections: vec![PlaceProjection::Deref],
            }),
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. } => {
                let mut path = self.read_place_path(value)?;
                path.projections.push(PlaceProjection::Deref);
                Some(path)
            }
            CheckedExpression::ProjectValue { value, field, .. } => {
                let mut path = self.read_place_path(value)?;
                path.projections.push(PlaceProjection::Field(*field));
                Some(path)
            }
            _ => None,
        }
    }

    /// [ENT-3] comparison-origin shape (a): a direct comparison call whose
    /// operands are each a term or constant.
    fn direct_comparison(&mut self, expression: &CheckedExpression) -> Option<Relation> {
        let CheckedExpression::IntegerOperation {
            operation,
            operand_type,
            arguments,
            ..
        } = expression
        else {
            return None;
        };
        fragment_type(*operand_type)?;
        let [left_expression, right_expression] = arguments.as_slice() else {
            return None;
        };
        let left = self.read_operand(left_expression)?;
        let right = self.read_operand(right_expression)?;
        sources::comparison_relation(*operation, left, right)
    }

    /// [ENT-3] comparison origin of a match scrutinee: shape (a) directly, or
    /// shape (b), a bare `own Bool` binding whose initializer comparison is
    /// still valid on every path to this use.
    fn scrutinee_relation(
        &mut self,
        expression: &CheckedExpression,
        state: &FactState,
    ) -> Option<Relation> {
        if let Some(relation) = self.direct_comparison(expression) {
            return Some(relation);
        }
        if let CheckedExpression::Binding { binding, ty, .. } = expression
            && *ty == CheckedType::Bool
        {
            return state.origins.get(binding).cloned();
        }
        None
    }

    // ------------------------------------------------------------------
    // Finite exact opaque goals [ENT-2..ENT-4]
    // ------------------------------------------------------------------

    /// Converts one source expression to ENT-3's exact direct pure/total
    /// origin. Any excluded child excludes the whole expression.
    fn direct_goal_expression(&self, expression: &CheckedExpression) -> Option<GoalExpression> {
        // A non-consuming place read is admitted by its final copy value, not
        // by the mode of every holder traversed on the way there. In
        // particular, reading through an owning box must retain the box's
        // explicit Deref projection even though the box binding itself is
        // affine and cannot be a standalone goal datum.
        if self.is_copy(expression.ty())
            && let Some(path) = self.read_place_path(expression)
            && let PlaceRoot::Binding(root) = path.root
        {
            return Some(GoalExpression::Datum(GoalDatum::Place {
                root,
                projections: path
                    .projections
                    .into_iter()
                    .map(|projection| match projection {
                        PlaceProjection::Field(field) => GoalProjection::Field(field),
                        PlaceProjection::Deref => GoalProjection::Deref,
                    })
                    .collect(),
                ty: expression.ty(),
            }));
        }
        let build_operation = |row, type_arguments, const_arguments, result, arguments: Vec<_>| {
            Some(GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            })
        };
        match expression {
            CheckedExpression::Constant(value) => {
                Some(GoalExpression::Datum(GoalDatum::Literal(value.clone())))
            }
            CheckedExpression::NamedConstant { declaration, value } => {
                Some(GoalExpression::Datum(GoalDatum::NamedConst {
                    declaration: *declaration,
                    projections: Vec::new(),
                    ty: value.ty(),
                }))
            }
            CheckedExpression::Binding { binding, ty, .. } if self.is_copy(*ty) => {
                Some(GoalExpression::Datum(GoalDatum::Place {
                    root: *binding,
                    projections: self
                        .needs_implicit_deref(*binding)
                        .then_some(GoalProjection::Deref)
                        .into_iter()
                        .collect(),
                    ty: *ty,
                }))
            }
            CheckedExpression::Project {
                binding,
                fields,
                ty,
                consume_root: false,
                ..
            } if self.is_copy(*ty) => Some(GoalExpression::Datum(GoalDatum::Place {
                root: *binding,
                projections: self
                    .needs_implicit_deref(*binding)
                    .then_some(GoalProjection::Deref)
                    .into_iter()
                    .chain(fields.iter().copied().map(GoalProjection::Field))
                    .collect(),
                ty: *ty,
            })),
            CheckedExpression::DerefAddressed { binding, ty, .. } if self.is_copy(*ty) => {
                Some(GoalExpression::Datum(GoalDatum::Place {
                    root: *binding,
                    projections: vec![GoalProjection::Deref],
                    ty: *ty,
                }))
            }
            CheckedExpression::BoxDeref {
                referent, value, ..
            } if self.is_copy(*referent) => self
                .direct_goal_expression(value)?
                .with_projection(GoalProjection::Deref, *referent),
            CheckedExpression::ArenaDeref { content, value, .. } if self.is_copy(*content) => self
                .direct_goal_expression(value)?
                .with_projection(GoalProjection::Deref, *content),
            CheckedExpression::ProjectValue {
                value, field, ty, ..
            } if self.is_copy(*ty) => self
                .direct_goal_expression(value)?
                .with_projection(GoalProjection::Field(*field), *ty),
            CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                result,
                ..
            } if !operation.is_exact() => build_operation(
                GoalOperation::Integer {
                    operation: *operation,
                    operand_type: *operand_type,
                },
                Vec::new(),
                Vec::new(),
                *result,
                arguments
                    .iter()
                    .map(|argument| self.direct_goal_expression(argument))
                    .collect::<Option<Vec<_>>>()?,
            ),
            CheckedExpression::IntegerOperation { .. } => None,
            CheckedExpression::FloatOperation {
                operation: row,
                operand_type,
                arguments,
                ..
            } => build_operation(
                GoalOperation::Float {
                    operation: *row,
                    operand_type: *operand_type,
                },
                if matches!(
                    row,
                    CheckedFloatOperation::Infinity | CheckedFloatOperation::Nan
                ) {
                    vec![*operand_type]
                } else {
                    Vec::new()
                },
                Vec::new(),
                row.result_type(*operand_type),
                arguments
                    .iter()
                    .map(|argument| self.direct_goal_expression(argument))
                    .collect::<Option<Vec<_>>>()?,
            ),
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                result,
                ..
            } => build_operation(
                GoalOperation::NumericConversion {
                    source: *source,
                    destination: *destination,
                },
                vec![source.ty(), destination.ty()],
                Vec::new(),
                *result,
                vec![self.direct_goal_expression(value)?],
            ),
            CheckedExpression::Reinterpret {
                source,
                destination,
                value,
                ..
            } => build_operation(
                GoalOperation::Reinterpret {
                    source: *source,
                    destination: *destination,
                },
                vec![source.ty(), destination.ty()],
                Vec::new(),
                destination.ty(),
                vec![self.direct_goal_expression(value)?],
            ),
            CheckedExpression::BooleanOperation {
                operation: row,
                arguments,
                ..
            } => build_operation(
                GoalOperation::Boolean(*row),
                Vec::new(),
                Vec::new(),
                CheckedType::Bool,
                arguments
                    .iter()
                    .map(|argument| self.direct_goal_expression(argument))
                    .collect::<Option<Vec<_>>>()?,
            ),
            CheckedExpression::EnumEquality {
                equal,
                operand_type,
                arguments,
                ..
            } => build_operation(
                GoalOperation::EnumEquality {
                    equal: *equal,
                    operand_type: *operand_type,
                },
                Vec::new(),
                Vec::new(),
                CheckedType::Bool,
                arguments
                    .iter()
                    .map(|argument| self.direct_goal_expression(argument))
                    .collect::<Option<Vec<_>>>()?,
            ),
            CheckedExpression::ArrayFill { ty, value, .. } => {
                let CheckedType::Array { element, length } = ty else {
                    return None;
                };
                build_operation(
                    GoalOperation::ArrayFill {
                        element: *element,
                        length: *length,
                    },
                    vec![element.ty()],
                    vec![*length],
                    *ty,
                    vec![self.direct_goal_expression(value)?],
                )
            }
            CheckedExpression::ArrayLength { root, length, .. } => {
                let argument = self.goal_array_root(root)?;
                let CheckedType::Array { element, .. } = argument.ty() else {
                    return None;
                };
                build_operation(
                    GoalOperation::ArrayLength {
                        element,
                        length: *length,
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::BufferLength { root, .. } => {
                let argument = self.goal_binding_place(
                    root.binding,
                    root.fields.iter().copied().map(GoalProjection::Field),
                    CheckedType::Buffer {
                        element: root.element,
                    },
                );
                build_operation(
                    GoalOperation::BufferLength {
                        element: root.element,
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::BufferFits {
                element,
                layout_ceiling,
                length,
                ..
            } => build_operation(
                GoalOperation::BufferFits {
                    element: *element,
                    maximum_length: layout_ceiling.stride.allocation_limit(),
                },
                vec![*element],
                Vec::new(),
                CheckedType::Bool,
                vec![self.direct_goal_expression(length)?],
            ),
            CheckedExpression::SliceLength { root, .. } => {
                let ty = self.summary(root.binding)?.ty?;
                let CheckedType::Slice { region, element } = ty else {
                    return None;
                };
                let argument = self.goal_binding_place(root.binding, std::iter::empty(), ty);
                build_operation(
                    GoalOperation::SliceLength { region, element },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::Binding { .. }
            | CheckedExpression::Project { .. }
            | CheckedExpression::DerefAddressed { .. }
            | CheckedExpression::BoxDeref { .. }
            | CheckedExpression::ProjectValue { .. }
            | CheckedExpression::UserCall { .. }
            | CheckedExpression::SystemCall { .. }
            | CheckedExpression::ArrayIndex { .. }
            | CheckedExpression::BufferFill { .. }
            | CheckedExpression::BufferVacant { .. }
            | CheckedExpression::BufferIndex { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceIndex { .. }
            | CheckedExpression::BoxNew { .. }
            | CheckedExpression::ArenaNew { .. }
            | CheckedExpression::ArenaDeref { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
            | CheckedExpression::ConstructStruct { .. }
            | CheckedExpression::ConstructEnum { .. } => None,
        }
    }

    fn goal_binding_place(
        &self,
        binding: BindingId,
        projections: impl IntoIterator<Item = GoalProjection>,
        ty: CheckedType,
    ) -> GoalExpression {
        GoalExpression::Datum(GoalDatum::Place {
            root: binding,
            projections: self
                .needs_implicit_deref(binding)
                .then_some(GoalProjection::Deref)
                .into_iter()
                .chain(projections)
                .collect(),
            ty,
        })
    }

    fn goal_array_root(&self, root: &CheckedArrayRoot) -> Option<GoalExpression> {
        match root {
            CheckedArrayRoot::Binding { binding, fields } => {
                let ty = self.projected_binding_type(*binding, fields)?;
                Some(self.goal_binding_place(
                    *binding,
                    fields.iter().copied().map(GoalProjection::Field),
                    ty,
                ))
            }
            CheckedArrayRoot::Constant(id) => {
                let declaration = self.context.constant_declaration(*id)?;
                let ty = self.context.constants.get(id.0 as usize)?.ty;
                Some(GoalExpression::Datum(GoalDatum::NamedConst {
                    declaration,
                    projections: Vec::new(),
                    ty,
                }))
            }
        }
    }

    fn projected_binding_type(&self, binding: BindingId, fields: &[u32]) -> Option<CheckedType> {
        let mut ty = self.summary(binding)?.ty?;
        for field in fields {
            let CheckedType::Nominal(nominal) = ty else {
                return None;
            };
            let CheckedNominalKind::Struct { fields } =
                &self.context.nominals.get(nominal.0 as usize)?.kind
            else {
                return None;
            };
            ty = fields.get(*field as usize)?.ty;
        }
        Some(ty)
    }

    /// Replaces every still-valid ordinary-let leaf by its one complete
    /// origin. Leaves without a valid origin remain direct, so expansion is
    /// all-or-nothing over exactly the eligible leaves.
    fn expand_goal_expression(
        &mut self,
        expression: &GoalExpression,
        state: &FactState,
    ) -> GoalExpression {
        self.expand_goal_expression_inner(expression, state, &mut HashSet::new(), false)
    }

    fn expand_goal_expression_inner(
        &mut self,
        expression: &GoalExpression,
        state: &FactState,
        expanding: &mut HashSet<BindingId>,
        preserve_normalized_leaf: bool,
    ) -> GoalExpression {
        match expression {
            GoalExpression::Datum(GoalDatum::Place {
                root,
                projections,
                ty,
            }) => {
                let Some(origin) = state.goal_origins.get(root).copied() else {
                    return expression.clone();
                };
                if !expanding.insert(*root) {
                    return expression.clone();
                }
                let origin = self.goals.expression(origin).clone();
                let mut expanded = self.expand_goal_expression_inner(
                    &origin,
                    state,
                    expanding,
                    preserve_normalized_leaf,
                );
                expanding.remove(root);
                for projection in projections {
                    let Some(result) = self.goal_projection_type(expanded.ty(), *projection) else {
                        return expression.clone();
                    };
                    let Some(next) = expanded.with_projection(*projection, result) else {
                        return expression.clone();
                    };
                    expanded = next;
                }
                if expanded.ty() == *ty {
                    expanded
                } else {
                    expression.clone()
                }
            }
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            } => {
                // Once an operation already has an exact L0 projection or
                // domain normalization, expanding one of its place operands
                // into a non-fragment expression would erase the checker
                // fact that Contrib(P) must classify. Boolean parents still
                // expand their children, so their normalized leaf predicates
                // remain visible without sacrificing those leaf identities.
                if preserve_normalized_leaf
                    && (self.goal_projection(expression).is_some()
                        || self.goal_normalization(expression).is_some())
                {
                    return expression.clone();
                }
                GoalExpression::Operation {
                    row: *row,
                    type_arguments: type_arguments.clone(),
                    const_arguments: const_arguments.clone(),
                    result: *result,
                    arguments: arguments
                        .iter()
                        .map(|argument| {
                            self.expand_goal_expression_inner(
                                argument,
                                state,
                                expanding,
                                preserve_normalized_leaf,
                            )
                        })
                        .collect(),
                }
            }
            GoalExpression::Datum(_) => expression.clone(),
        }
    }

    fn goal_projection_type(
        &self,
        input: CheckedType,
        projection: GoalProjection,
    ) -> Option<CheckedType> {
        match projection {
            GoalProjection::Deref => match input {
                CheckedType::Nominal(nominal) => {
                    match self.context.nominals.get(nominal.0 as usize)?.kind {
                        CheckedNominalKind::Box { referent } => Some(referent),
                        _ => Some(input),
                    }
                }
                // Borrow holders retain the referent type in checked form.
                _ => Some(input),
            },
            GoalProjection::Field(field) => {
                let CheckedType::Nominal(nominal) = input else {
                    return None;
                };
                let CheckedNominalKind::Struct { fields } =
                    &self.context.nominals.get(nominal.0 as usize)?.kind
                else {
                    return None;
                };
                fields.get(field as usize).map(|field| field.ty)
            }
        }
    }

    fn goal_origin_set(
        &mut self,
        expression: &CheckedExpression,
        state: &FactState,
    ) -> Vec<GoalId> {
        let Some(direct) = self.direct_goal_expression(expression) else {
            return Vec::new();
        };
        if direct.ty() != CheckedType::Bool {
            return Vec::new();
        }
        let complete = self.expand_goal_expression(&direct, state);
        let direct = self.intern_goal_expression(direct);
        let complete = self.intern_goal_expression(complete);
        if direct == complete {
            vec![direct]
        } else {
            vec![direct, complete]
        }
    }

    fn record_goal_origin(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut FactState,
    ) {
        let Some(direct) = self.direct_goal_expression(value) else {
            return;
        };
        let origin = self.intern_goal_expression(direct);
        state.goal_origins.insert(binding, origin);
        state.ambiguous_goal_origins.remove(&binding);
    }

    fn record_value_initializer_origin(&self, frame: &GiveFrame, state: &mut FactState) {
        let mut origins =
            frame
                .gives
                .iter()
                .zip(&frame.give_goal_origins)
                .filter_map(|(edge, origin)| {
                    let edge = &edge.facts;
                    (!edge.all_derivable).then_some(*origin)
                });
        let Some(first) = origins.next() else {
            return;
        };
        if origins.any(|origin| origin != first) {
            state.ambiguous_goal_origins.insert(frame.binding);
        }
    }

    fn intern_goal_expression(&mut self, expression: GoalExpression) -> GoalId {
        if let GoalExpression::Operation {
            row: GoalOperation::Boolean(_),
            arguments,
            ..
        } = &expression
        {
            for argument in arguments {
                if argument.ty() == CheckedType::Bool {
                    self.intern_goal_expression(argument.clone());
                }
            }
        }
        let projection = self.goal_projection(&expression);
        let normalization = self.goal_normalization(&expression);
        let mut support = Vec::new();
        self.collect_goal_support(&expression, false, &mut support);
        self.goals
            .intern(expression, projection, normalization, support)
    }

    /// [O11 candidate] The signed Boolean decomposition set of one
    /// established goal: `+band` and `-bor` decompose into their signed
    /// children recursively, `bnot` flips the sign, and every other root —
    /// in particular `-band` and `+bor`, whose content is genuinely
    /// disjunctive, and `bxor` on either sign — contributes nothing.
    ///
    /// Members are interned so their exact identities, projections, and
    /// supports are retained in the inventory, but nothing establishes them
    /// as facts in this version: v0.30 acceptance is untouched. Design:
    /// `research/investigations/o11-composition/DESIGN.md`.
    fn signed_boolean_decomposition(
        &mut self,
        parent: GoalId,
        sign: GoalSign,
        state: &FactState,
    ) -> Vec<(GoalId, GoalSign)> {
        let expression = self.goals.expression(parent).clone();
        let mut members = Vec::new();
        self.collect_decomposition_members(
            &expression,
            sign,
            state,
            &mut members,
            &mut HashSet::new(),
        );
        members
    }

    fn collect_decomposition_members(
        &mut self,
        expression: &GoalExpression,
        sign: GoalSign,
        state: &FactState,
        members: &mut Vec<(GoalId, GoalSign)>,
        following: &mut HashSet<BindingId>,
    ) {
        // An unprojected `own Bool` leaf carrying a still-valid ordinary-let
        // origin stands for that origin under either sign, so the Boolean root
        // is read through the leaf and the leaf contributes no member of its
        // own. Reading the origin here is what keeps a conjunct in the operand
        // form its own binding recorded: the members of a `band` written over
        // comparison bindings are those bindings, whose relations
        // [`Self::establish_boolean_decomposition`] then takes from
        // `state.origins`, so both source spellings use the same relations.
        if let GoalExpression::Datum(GoalDatum::Place {
            root,
            projections,
            ty: CheckedType::Bool,
        }) = expression
            && projections.is_empty()
        {
            let Some(origin) = state.goal_origins.get(root).copied() else {
                return;
            };
            // Only a Boolean root has anything to decompose, and the ordinary
            // guard binding holds a comparison, so this settles the common case
            // without retaining the origin.
            if !matches!(
                self.goals.expression(origin),
                GoalExpression::Operation {
                    row: GoalOperation::Boolean(_),
                    ..
                }
            ) {
                return;
            }
            if !following.insert(*root) {
                return;
            }
            let origin = self.goals.expression(origin).clone();
            self.collect_decomposition_members(&origin, sign, state, members, following);
            following.remove(root);
            return;
        }
        let GoalExpression::Operation {
            row: GoalOperation::Boolean(operation),
            arguments,
            ..
        } = expression
        else {
            return;
        };
        let child_sign = match (operation, sign) {
            (CheckedBooleanOperation::And, GoalSign::Positive)
            | (CheckedBooleanOperation::Or, GoalSign::Negative) => sign,
            (CheckedBooleanOperation::Not, GoalSign::Positive) => GoalSign::Negative,
            (CheckedBooleanOperation::Not, GoalSign::Negative) => GoalSign::Positive,
            _ => return,
        };
        for argument in arguments {
            let member = self.intern_goal_expression(argument.clone());
            if !members.contains(&(member, child_sign)) {
                members.push((member, child_sign));
            }
            self.collect_decomposition_members(argument, child_sign, state, members, following);
        }
    }

    /// [ENT-3] Establishes the signed Boolean decomposition set of one
    /// just-established signed goal, at that same point and in whatever proof
    /// view the state carries.
    ///
    /// Each member enters as its own concrete opaque goal under [FN-8]
    /// structural identity, and a member whose complete root is one admitted
    /// comparison additionally delivers its exact L0 projection under `+` and
    /// that projection's exact negation under `-`. A member that is a bare
    /// comparison binding carries no projection of its own and delivers the
    /// relation that binding recorded instead, so a conjunct proves exactly
    /// what the same comparison proves at a direct branch.
    /// Decomposition never runs upward: this establishes children of an
    /// established parent only, so no child ever establishes or derives a
    /// parent.
    pub(super) fn establish_boolean_decomposition(
        &mut self,
        parent: GoalId,
        sign: GoalSign,
        state: &mut FactState,
        event: FlowEventId,
    ) {
        for (member, member_sign) in self.signed_boolean_decomposition(parent, sign, state) {
            state.establish_goal(member, member_sign, &mut self.derivations, event);
            let Some(relation) = self
                .goals
                .projection(member)
                .cloned()
                .or_else(|| self.member_binding_relation(member, state))
            else {
                continue;
            };
            let relation = match member_sign {
                GoalSign::Positive => relation,
                GoalSign::Negative => relation.negated(),
            };
            state.establish(&relation, &mut self.derivations, event);
        }
    }

    /// The comparison one decomposition member's own binding recorded, for a
    /// member that is an unprojected `own Bool` place. This is `state.origins`,
    /// the [ENT-3] comparison-origin map [`Self::scrutinee_relation`] reads, so
    /// a conjunct and a direct comparison on the same binding deliver the
    /// same relation over the same terms.
    fn member_binding_relation(&self, member: GoalId, state: &FactState) -> Option<Relation> {
        let GoalExpression::Datum(GoalDatum::Place {
            root,
            projections,
            ty: CheckedType::Bool,
        }) = self.goals.expression(member)
        else {
            return None;
        };
        projections
            .is_empty()
            .then(|| state.origins.get(root).cloned())
            .flatten()
    }

    /// Records the O11 decomposition inventory entry for one signed-goal
    /// establishment. Entries deduplicate by parent and sign; this is
    /// retained metadata beside the facts
    /// [`Self::establish_boolean_decomposition`] establishes.
    pub(super) fn record_boolean_decomposition(
        &mut self,
        parent: GoalId,
        sign: GoalSign,
        state: &FactState,
    ) {
        if self
            .boolean_decompositions
            .iter()
            .any(|candidate| candidate.parent == parent && candidate.sign == sign)
        {
            return;
        }
        let members = self.signed_boolean_decomposition(parent, sign, state);
        if members.is_empty() {
            return;
        }
        self.boolean_decompositions
            .push(super::BooleanGoalDecomposition {
                parent,
                sign,
                members,
            });
    }

    fn collect_goal_support(
        &self,
        expression: &GoalExpression,
        length: bool,
        support: &mut Vec<GoalSupport>,
    ) {
        match expression {
            GoalExpression::Datum(GoalDatum::Place {
                root, projections, ..
            }) => support.push(GoalSupport {
                root: *root,
                projections: projections.clone(),
                length,
            }),
            GoalExpression::Datum(
                GoalDatum::Parameter { .. }
                | GoalDatum::NamedConst { .. }
                | GoalDatum::EphemeralActual { .. }
                | GoalDatum::Literal(_),
            ) => {}
            GoalExpression::Operation { row, arguments, .. } => {
                let is_length = matches!(
                    row,
                    GoalOperation::ArrayLength { .. }
                        | GoalOperation::BufferLength { .. }
                        | GoalOperation::SliceLength { .. }
                );
                for argument in arguments {
                    self.collect_goal_support(argument, is_length, support);
                }
            }
        }
    }

    fn goal_projection(&mut self, expression: &GoalExpression) -> Option<Relation> {
        let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation,
                    operand_type,
                },
            arguments,
            ..
        } = expression
        else {
            return None;
        };
        fragment_type(*operand_type)?;
        let [left, right] = arguments.as_slice() else {
            return None;
        };
        let left = self.goal_operand(left)?;
        let right = self.goal_operand(right)?;
        sources::comparison_relation(*operation, left, right)
    }

    fn goal_operand(&mut self, expression: &GoalExpression) -> Option<TermId> {
        match expression {
            GoalExpression::Datum(GoalDatum::Literal(CheckedValue::Integer { ty, bits })) => Some(
                self.terms
                    .intern(TermKind::Constant(integer_value(*ty, *bits))),
            ),
            GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) if projections.is_empty() => {
                let CheckedValue::Integer {
                    ty: value_type,
                    bits,
                } = &self.context.constant(*declaration)?.value
                else {
                    return None;
                };
                (*ty == CheckedType::Integer(*value_type)).then(|| {
                    self.terms
                        .intern(TermKind::Constant(integer_value(*value_type, *bits)))
                })
            }
            GoalExpression::Datum(datum) => {
                let fragment = fragment_type(datum.ty())?;
                let path = self.goal_place_path(datum)?;
                let kind = if path
                    .projections
                    .iter()
                    .all(|projection| matches!(projection, PlaceProjection::Field(_)))
                {
                    TermKind::Place(
                        PlaceTerm {
                            root: path.root,
                            deref: false,
                            fields: path
                                .projections
                                .iter()
                                .filter_map(|projection| match projection {
                                    PlaceProjection::Field(field) => Some(*field),
                                    PlaceProjection::Deref => None,
                                })
                                .collect(),
                        },
                        fragment,
                    )
                } else {
                    TermKind::ProjectedPlace(path, fragment)
                };
                Some(self.terms.intern(kind))
            }
            GoalExpression::Operation { row, arguments, .. }
                if matches!(
                    row,
                    GoalOperation::ArrayLength { .. }
                        | GoalOperation::BufferLength { .. }
                        | GoalOperation::SliceLength { .. }
                ) =>
            {
                let [place] = arguments.as_slice() else {
                    return None;
                };
                let GoalExpression::Datum(datum) = place else {
                    return None;
                };
                let path = self.goal_place_path(datum)?;
                let term = if let Some(place) = legacy_place(&path) {
                    self.terms.intern(TermKind::Length(place))
                } else {
                    self.terms.intern(TermKind::ProjectedLength(path))
                };
                if let GoalOperation::ArrayLength { length, .. } = row {
                    let bound = match length {
                        CheckedConst::Value(value) => {
                            Some(LengthBound::Constant(i128::from(*value)))
                        }
                        CheckedConst::Parameter(declaration) => Some(LengthBound::Equal(
                            self.terms.intern(TermKind::ConstParameter(*declaration)),
                        )),
                        // A symbolic derived length has no [ENT-2] term form.
                        CheckedConst::Derived(_) => None,
                    };
                    if let Some(bound) = bound {
                        self.terms.set_length_bound(term, bound);
                    }
                }
                Some(term)
            }
            GoalExpression::Operation { .. } => None,
        }
    }

    fn goal_place_path(&self, datum: &GoalDatum) -> Option<ProjectedPlaceTerm> {
        let (root, projections) = match datum {
            GoalDatum::Place {
                root, projections, ..
            } => (PlaceRoot::Binding(*root), projections),
            GoalDatum::NamedConst {
                declaration,
                projections,
                ..
            } => (
                PlaceRoot::Constant(*self.context.constant_ids.get(declaration)?),
                projections,
            ),
            GoalDatum::Parameter { .. }
            | GoalDatum::EphemeralActual { .. }
            | GoalDatum::Literal(_) => return None,
        };
        Some(ProjectedPlaceTerm {
            root,
            projections: projections
                .iter()
                .map(|projection| match projection {
                    GoalProjection::Deref => PlaceProjection::Deref,
                    GoalProjection::Field(field) => PlaceProjection::Field(*field),
                })
                .collect(),
        })
    }

    fn body_requirement_goal(&self, requirement: &CheckedRequirement) -> Option<GoalExpression> {
        self.body_goal_expression(&requirement.template.root)
    }

    fn body_goal_expression(&self, expression: &GoalExpression) -> Option<GoalExpression> {
        match expression {
            GoalExpression::Datum(GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            }) => {
                let parameter = self.function.parameters.get(*ordinal as usize)?;
                Some(GoalExpression::Datum(GoalDatum::Place {
                    root: parameter.binding,
                    projections: projections.clone(),
                    ty: *ty,
                }))
            }
            GoalExpression::Datum(GoalDatum::EphemeralActual { .. }) => None,
            GoalExpression::Datum(datum) => Some(GoalExpression::Datum(datum.clone())),
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            } => Some(GoalExpression::Operation {
                row: *row,
                type_arguments: type_arguments.clone(),
                const_arguments: const_arguments.clone(),
                result: *result,
                arguments: arguments
                    .iter()
                    .map(|argument| self.body_goal_expression(argument))
                    .collect::<Option<Vec<_>>>()?,
            }),
        }
    }

    // ------------------------------------------------------------------
    // Kill collection from expressions
    // ------------------------------------------------------------------

    fn is_copy(&self, ty: CheckedType) -> bool {
        match ty {
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_) => true,
            CheckedType::Nominal(id) => self
                .context
                .nominals
                .get(id.0 as usize)
                .is_some_and(CheckedNominal::is_copy),
            _ => false,
        }
    }

    fn argument_referent(
        &self,
        argument: &CheckedExpression,
    ) -> Option<(ResolvedPlace, bool, bool)> {
        self.places.argument_referent(argument)
    }

    /// Collects [ENT-5] kill events (b) and (c) from one expression tree.
    fn collect_expression_kills(
        &self,
        expression: &CheckedExpression,
        events: &mut Vec<KillEvent>,
    ) {
        match expression {
            CheckedExpression::Binding {
                carrier,
                binding,
                consume_root,
                ty,
                ..
            } => {
                if self.is_holder(*binding) {
                    if *consume_root {
                        events.push(KillEvent::EntryImageHolderConsume {
                            binding: *binding,
                            source: carrier.clone(),
                        });
                    }
                } else if !self.is_copy(*ty) {
                    events.push(KillEvent::Consume {
                        binding: *binding,
                        source: carrier.clone(),
                    });
                }
            }
            CheckedExpression::Project {
                carrier,
                binding,
                consume_root,
                ..
            } => {
                if *consume_root {
                    events.push(KillEvent::Consume {
                        binding: *binding,
                        source: carrier.clone(),
                    });
                }
            }
            // These wrappers are checked reads of one place. Their nested
            // expression preserves source spelling and lowering structure;
            // it is not a second consuming evaluation of an affine holder.
            CheckedExpression::BoxDeref { .. }
            | CheckedExpression::ArenaDeref { .. }
            | CheckedExpression::ProjectValue { .. } => {}
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                ..
            } => {
                let callee = self.context.callee(*function);
                for argument in arguments {
                    self.collect_expression_kills(argument, events);
                }
                for (index, argument) in arguments.iter().enumerate() {
                    let Some(writes) = callee.and_then(|callee| callee.parameter_writes.get(index))
                    else {
                        continue;
                    };
                    if let Some((place, element, entry_image_only)) =
                        self.argument_referent(argument)
                    {
                        for fields in writes {
                            let mut written = place.clone();
                            written.fields.extend_from_slice(fields);
                            if entry_image_only {
                                events.push(KillEvent::EntryImageHolderWrite {
                                    place: written,
                                    element,
                                    source: call.clone(),
                                });
                            } else {
                                events.push(KillEvent::Write {
                                    place: written,
                                    element,
                                    source: call.clone(),
                                });
                            }
                        }
                    }
                }
            }
            CheckedExpression::SystemCall {
                operation,
                call,
                arguments,
                ..
            } => {
                let operation_row = SYSTEM_OPERATIONS.get(usize::from(*operation));
                let writes = operation_row
                    .map(|operation| crate::operation_state_effects(operation).1)
                    .unwrap_or_default();
                for argument in arguments {
                    self.collect_expression_kills(argument, events);
                }
                for (index, argument) in arguments.iter().enumerate() {
                    let written =
                        u8::try_from(index).is_ok_and(|ordinal| writes.contains(&ordinal));
                    if written
                        && let Some((place, element, entry_image_only)) =
                            self.argument_referent(argument)
                    {
                        if entry_image_only {
                            events.push(KillEvent::EntryImageHolderWrite {
                                place,
                                element,
                                source: call.clone(),
                            });
                        } else {
                            events.push(KillEvent::Write {
                                place,
                                element,
                                source: call.clone(),
                            });
                        }
                    }
                }
            }
            _ => {
                for child in expression_children(expression) {
                    self.collect_expression_kills(child, events);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Obligations [ENT-6]
    // ------------------------------------------------------------------

    /// Judges every bounds obligation inside one expression against the
    /// state at this point, inner offsets before the sites they feed.
    fn judge_expression(
        &mut self,
        expression: &CheckedExpression,
        states: &mut ProofFlowState,
    ) -> Option<PreparedCall> {
        match expression {
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                requirements,
                ..
            } => {
                let obligation_start = self.obligations.len();
                for argument in arguments {
                    let _ = self.judge_expression(argument, states);
                }
                let actual_parents = self.obligations[obligation_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let mut goal_parents = Vec::with_capacity(requirements.len());
                let mut goals_ok = true;
                // FN-8 begins only after every actual-expression obligation
                // succeeds. A failed OP-4 actual therefore publishes no call
                // judgment for diagnostic selection to reorder.
                for requirement in requirements {
                    if actual_parents.is_some() {
                        let (disposition, derivation) = self.judge_call_goal(
                            *function,
                            call,
                            requirement.requires_clause.clone(),
                            requirement.goal.clone(),
                            arguments.len(),
                            ProofContext::new(&states.facts, &states.affine),
                        );
                        goals_ok &= disposition == CallGoalDisposition::Discharged;
                        if let Some(derivation) = derivation {
                            goal_parents.push(derivation);
                        }
                    }
                }
                let mut parents = actual_parents?;
                if !goals_ok || goal_parents.len() != requirements.len() {
                    return None;
                }
                parents.extend(goal_parents);
                // Only an earlier-component verified summary can publish an
                // S12 carrier. Calls without one retain the exact pre-H3 kill
                // path and create no transient postcondition events.
                if self.context.verified_postconditions(*function)?.is_empty() {
                    return None;
                }
                Some(PreparedCall {
                    function: *function,
                    call: call.clone(),
                    parents,
                    transfer_events: Vec::new(),
                    kills: Vec::new(),
                })
            }
            CheckedExpression::SystemCall {
                operation,
                call,
                arguments,
                ..
            } => {
                for argument in arguments {
                    let _ = self.judge_expression(argument, states);
                }
                self.judge_system_ranges(*operation, call, arguments, states);
                None
            }
            CheckedExpression::ArrayIndex {
                root,
                length,
                offset,
                obligation,
                ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = self.array_root_place(root);
                self.judge_obligation(base, Some(*length), offset, obligation.clone(), states);
                None
            }
            CheckedExpression::BufferIndex {
                root,
                offset,
                obligation,
                ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                self.judge_obligation(base, None, offset, obligation.clone(), states);
                None
            }
            CheckedExpression::SliceIndex {
                root,
                offset,
                obligation,
                ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                self.judge_obligation(base, None, offset, obligation.clone(), states);
                None
            }
            CheckedExpression::BufferFill {
                carrier,
                element,
                layout_ceiling,
                length,
                value,
                ..
            } => {
                let _ = self.judge_expression(length, states);
                let _ = self.judge_expression(value, states);
                self.judge_allocation_fit(
                    element.ty(),
                    layout_ceiling.stride.allocation_limit(),
                    length,
                    carrier.clone(),
                    states,
                );
                None
            }
            CheckedExpression::BufferVacant {
                carrier,
                element,
                layout_ceiling,
                length,
                ..
            } => {
                let _ = self.judge_expression(length, states);
                self.judge_allocation_fit(
                    CheckedType::Nominal(*element),
                    layout_ceiling.stride.allocation_limit(),
                    length,
                    carrier.clone(),
                    states,
                );
                None
            }
            CheckedExpression::IntegerOperation {
                carrier,
                operation,
                operand_type,
                arguments,
                ..
            } => {
                for argument in arguments {
                    let _ = self.judge_expression(argument, states);
                }
                if operation.is_exact() {
                    self.judge_integer_domain_obligation(
                        *operation,
                        *operand_type,
                        arguments,
                        carrier,
                        states,
                    );
                }
                None
            }
            _ => {
                for child in expression_children(expression) {
                    let _ = self.judge_expression(child, states);
                }
                None
            }
        }
    }

    fn judge_call_goal(
        &mut self,
        callee: super::super::model::FunctionId,
        node_path: &crate::NodePath,
        requires_clause: crate::NodePath,
        goal: ConcreteGoal,
        argument_count: usize,
        context: ProofContext<'_>,
    ) -> (CallGoalDisposition, Option<DerivationId>) {
        let (disposition, evidence, derivation) = self.call_goal_disposition(&goal, context);
        let ordinal = u32::try_from(self.call_goals.len())
            .expect("ENT call-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::CallGoal(ordinal), root);
        }
        let rendered_goal = self.render_concrete_goal(&goal.root);
        self.call_goals.push(CallGoalOutcome {
            node_path: node_path.clone(),
            callee,
            requires_clause,
            goal,
            rendered_goal,
            argument_count: u32::try_from(argument_count)
                .expect("ENT call argument count exceeds the u32 identity space"),
            disposition,
            evidence,
            derivation,
        });
        (disposition, derivation)
    }

    fn call_goal_disposition(
        &mut self,
        goal: &ConcreteGoal,
        context: ProofContext<'_>,
    ) -> (
        CallGoalDisposition,
        Vec<CallGoalEvidence>,
        Option<DerivationId>,
    ) {
        let affine_target = self.affine_goal_ordering_target(&goal.root, context.affine);
        let result = self.prove(
            context,
            ProofGoal::Signed {
                expression: &goal.root,
                affine: affine_target.as_ref(),
            },
        );
        let disposition = match result.disposition {
            ProofDisposition::Proved => CallGoalDisposition::Discharged,
            ProofDisposition::Refuted => CallGoalDisposition::Refuted,
            ProofDisposition::Unknown => CallGoalDisposition::Unproved,
        };
        let evidence = match (result.disposition, result.route) {
            (ProofDisposition::Proved, Some(ProofRoute::Contradiction)) => {
                vec![CallGoalEvidence::AllDerivable]
            }
            (
                sign @ (ProofDisposition::Proved | ProofDisposition::Refuted),
                Some(ProofRoute::SignedOrdinary {
                    opaque,
                    projection,
                    normalization,
                    introduction,
                }),
            ) => {
                let mut evidence = Vec::with_capacity(4);
                match sign {
                    ProofDisposition::Proved => {
                        if opaque {
                            evidence.push(CallGoalEvidence::OpaquePositive);
                        }
                        if projection {
                            evidence.push(CallGoalEvidence::ExactL0Projection);
                        }
                        if normalization {
                            evidence.push(CallGoalEvidence::NormalizationPositive);
                        }
                        if introduction {
                            evidence.push(CallGoalEvidence::BooleanIntroductionPositive);
                        }
                    }
                    ProofDisposition::Refuted => {
                        if opaque {
                            evidence.push(CallGoalEvidence::OpaqueNegative);
                        }
                        if projection {
                            evidence.push(CallGoalEvidence::NegatedL0Projection);
                        }
                        if normalization {
                            evidence.push(CallGoalEvidence::NormalizationNegative);
                        }
                        if introduction {
                            evidence.push(CallGoalEvidence::BooleanIntroductionNegative);
                        }
                    }
                    ProofDisposition::Unknown => unreachable!(),
                }
                evidence
            }
            (ProofDisposition::Proved, Some(ProofRoute::Affine)) => {
                vec![CallGoalEvidence::AffinePositive]
            }
            (ProofDisposition::Unknown, None) => Vec::new(),
            _ => unreachable!("a signed proof returned an incompatible route"),
        };
        (disposition, evidence, result.derivation)
    }

    /// Unified deterministic entry for numeric/logical entailment.  The
    /// consumer supplies one normalized proposition; this function tries the
    /// fixed ordinary closure before the fixed affine rule and constructs the
    /// selected derivation during that same query.
    fn prove(&mut self, context: ProofContext<'_>, goal: ProofGoal<'_>) -> ProofResult {
        match goal {
            ProofGoal::Affine { inequality } => self.prove_affine(context, inequality),
            ProofGoal::Signed { expression, affine } => {
                self.prove_signed(context, expression, affine)
            }
            ProofGoal::Ordering { relation, affine } => {
                self.prove_ordering(context, relation, affine)
            }
            ProofGoal::IntegerDomain(goal) => self.prove_integer_domain(context, goal),
            ProofGoal::BoundedRelation(goal) => self.prove_bounded_relation(context, goal),
            ProofGoal::NormalizedOrdering {
                goal,
                relation,
                affine,
                upper_bound,
            } => {
                let proof = self.prove_normalized_ordering(&context, goal, relation, affine);
                self.project_numeric_upper_bound(&context, proof, upper_bound)
            }
        }
    }

    fn prove_affine(
        &mut self,
        context: ProofContext<'_>,
        inequality: &AffineInequality,
    ) -> ProofResult {
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.contradictory() {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: closed.contradiction_proof(),
                numeric_upper_bound: None,
            };
        }
        let facts = Self::affine_facts(context.affine);
        let Some(proof) =
            self.affine_target_proof(inequality, &facts, context.affine, context.facts)
        else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let derivation = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: None,
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        ProofResult {
            disposition: ProofDisposition::Proved,
            route: Some(ProofRoute::Affine),
            derivation: Some(derivation),
            numeric_upper_bound: None,
        }
    }

    fn prove_signed(
        &mut self,
        context: ProofContext<'_>,
        expression: &GoalExpression,
        affine_target: Option<&AffineInequality>,
    ) -> ProofResult {
        let goal = self.intern_goal_expression(expression.clone());
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.contradictory() {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: closed.contradiction_proof(),
                numeric_upper_bound: None,
            };
        }

        let positive_opaque = closed.holds_opaque(goal, GoalSign::Positive);
        let positive_projection = self
            .goals
            .projection(goal)
            .is_some_and(|relation| closed.derives(relation));
        let positive_normalization =
            closed.derives_normalized_goal(goal, GoalSign::Positive, &self.goals);
        let positive_introduction = !positive_opaque
            && !positive_projection
            && !positive_normalization
            && closed.derives_goal(goal, GoalSign::Positive, &self.goals);
        if positive_opaque || positive_projection || positive_normalization || positive_introduction
        {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::SignedOrdinary {
                    opaque: positive_opaque,
                    projection: positive_projection,
                    normalization: positive_normalization,
                    introduction: positive_introduction,
                }),
                derivation: closed.goal_proof(
                    goal,
                    GoalSign::Positive,
                    &self.goals,
                    &mut self.derivations,
                ),
                numeric_upper_bound: None,
            };
        }

        let negative_opaque = closed.holds_opaque(goal, GoalSign::Negative);
        let negative_projection = self
            .goals
            .projection(goal)
            .is_some_and(|relation| closed.derives(&relation.negated()));
        let negative_normalization =
            closed.derives_normalized_goal(goal, GoalSign::Negative, &self.goals);
        let negative_introduction = !negative_opaque
            && !negative_projection
            && !negative_normalization
            && closed.derives_goal(goal, GoalSign::Negative, &self.goals);
        if negative_opaque || negative_projection || negative_normalization || negative_introduction
        {
            return ProofResult {
                disposition: ProofDisposition::Refuted,
                route: Some(ProofRoute::SignedOrdinary {
                    opaque: negative_opaque,
                    projection: negative_projection,
                    normalization: negative_normalization,
                    introduction: negative_introduction,
                }),
                derivation: None,
                numeric_upper_bound: None,
            };
        }

        let Some(target) = affine_target else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let Some(relation) = self.goals.projection(goal).cloned() else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let assumptions = Self::affine_facts(context.affine);
        let Some(proof) =
            self.affine_target_proof(target, &assumptions, context.affine, context.facts)
        else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let consequence = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: Some(Box::new(relation.clone())),
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        let derivation = self.derivations.intern(DerivationNode::GoalProjection {
            goal,
            sign: GoalSign::Positive,
            relation,
            parent: consequence,
        });
        ProofResult {
            disposition: ProofDisposition::Proved,
            route: Some(ProofRoute::Affine),
            derivation: Some(derivation),
            numeric_upper_bound: None,
        }
    }

    fn prove_ordering(
        &mut self,
        context: ProofContext<'_>,
        relation: &Relation,
        affine_target: Option<&AffineInequality>,
    ) -> ProofResult {
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.contradictory() {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: closed.contradiction_proof(),
                numeric_upper_bound: None,
            };
        }
        if closed.derives(relation) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::L0),
                derivation: Some(
                    closed
                        .relation_proof(relation, &mut self.derivations)
                        .expect("a proved L0 relation must retain its local derivation"),
                ),
                numeric_upper_bound: None,
            };
        }
        if closed.derives(&relation.negated()) {
            return ProofResult {
                disposition: ProofDisposition::Refuted,
                route: Some(ProofRoute::L0),
                derivation: None,
                numeric_upper_bound: None,
            };
        }

        let Some(target) = affine_target else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let assumptions = Self::affine_facts(context.affine);
        let Some(proof) =
            self.affine_target_proof(target, &assumptions, context.affine, context.facts)
        else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let derivation = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: None,
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        ProofResult {
            disposition: ProofDisposition::Proved,
            route: Some(ProofRoute::Affine),
            derivation: Some(derivation),
            numeric_upper_bound: None,
        }
    }

    fn prove_bounded_relation(
        &mut self,
        context: ProofContext<'_>,
        goal: BoundedRelationGoal<'_>,
    ) -> ProofResult {
        if let Some(canonical) = goal.canonical {
            let direct = self.prove_signed(context, canonical, goal.direct_affine);
            if direct.disposition != ProofDisposition::Unknown {
                return direct;
            }
        }
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.contradictory() {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: closed.contradiction_proof(),
                numeric_upper_bound: None,
            };
        }
        let Some(left) = goal.request.left else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        if let Some(derivation) = closed.bound_proof(
            left,
            goal.request.right,
            goal.request.bound,
            &mut self.derivations,
        ) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::L0),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        if let (None, Some(target)) = (goal.canonical, goal.direct_affine) {
            let direct = self.prove_affine(context, target);
            if direct.disposition == ProofDisposition::Proved {
                return direct;
            }
        }

        if let Some(derivation) = goal.fixed_affine_bridge.and_then(|bridge| {
            self.fixed_affine_bound_derivation(
                bridge,
                left,
                goal.request.right,
                goal.request.bound,
                context.affine,
                context.facts,
            )
        }) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Affine),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        if let Some(derivation) = goal.affine_left.and_then(|affine_left| {
            self.affine_bound_via_l0_right(
                affine_left,
                left,
                goal.request.right,
                goal.request.bound,
                context.affine,
                context.facts,
            )
        }) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Affine),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
        }
    }

    /// Proves one canonical ordering that may also be represented by a finite
    /// normalized goal. The goal identity has ordinary priority when present;
    /// the bare L0 relation is the fallback only for source operands outside
    /// that goal fragment. The affine route proves the same written relation
    /// and, when needed, concludes the exact goal normalization in the same
    /// call.
    fn prove_normalized_ordering(
        &mut self,
        context: &ProofContext<'_>,
        goal: Option<GoalId>,
        relation: Option<&Relation>,
        affine_target: Option<&AffineInequality>,
    ) -> ProofResult {
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        if closed.contradictory() {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: closed.contradiction_proof(),
                numeric_upper_bound: None,
            };
        }
        if let Some(goal) = goal {
            if closed.derives_goal(goal, GoalSign::Positive, &self.goals) {
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: closed.goal_proof(
                        goal,
                        GoalSign::Positive,
                        &self.goals,
                        &mut self.derivations,
                    ),
                    numeric_upper_bound: None,
                };
            }
            if closed.derives_goal(goal, GoalSign::Negative, &self.goals) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: None,
                    numeric_upper_bound: None,
                };
            }
        } else if let Some(relation) = relation {
            if closed.derives(relation) {
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::L0),
                    derivation: Some(
                        closed
                            .relation_proof(relation, &mut self.derivations)
                            .expect("a proved L0 relation must retain its local derivation"),
                    ),
                    numeric_upper_bound: None,
                };
            }
            if closed.derives(&relation.negated()) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::L0),
                    derivation: None,
                    numeric_upper_bound: None,
                };
            }
        }

        let (Some(target), Some(relation)) = (affine_target, relation) else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let assumptions = Self::affine_facts(context.affine);
        let Some(proof) =
            self.affine_target_proof(target, &assumptions, context.affine, context.facts)
        else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
            };
        };
        let consequence = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: Some(Box::new(relation.clone())),
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        let derivation = goal.map_or(consequence, |goal| {
            self.derivations.intern(DerivationNode::GoalNormalization {
                goal,
                sign: GoalSign::Positive,
                clause: 0,
                parents: vec![consequence],
            })
        });
        ProofResult {
            disposition: ProofDisposition::Proved,
            route: Some(ProofRoute::Affine),
            derivation: Some(derivation),
            numeric_upper_bound: None,
        }
    }

    /// Projects the tightest numeric ceiling available from the same proof
    /// context after, and only after, the normalized ordering was proved.
    /// This is not another admission query: the selected `ProofResult` remains
    /// the sole acceptance authority. The projection merely chooses between
    /// the ordering's own admitted ceiling, the ordinary closure, and the
    /// fixed affine interval rule, retaining the derivation for the chosen
    /// number.
    fn project_numeric_upper_bound(
        &mut self,
        context: &ProofContext<'_>,
        mut proof: ProofResult,
        request: Option<NumericUpperBoundRequest<'_>>,
    ) -> ProofResult {
        let Some(request) = request else {
            return proof;
        };
        if proof.disposition != ProofDisposition::Proved {
            return proof;
        }
        let Some(admission_derivation) = proof.derivation else {
            return proof;
        };
        if proof.route == Some(ProofRoute::Contradiction) {
            proof.numeric_upper_bound = Some(ProvedNumericUpperBound {
                value: 0,
                derivation: admission_derivation,
            });
            return proof;
        }

        let mut selected = ProvedNumericUpperBound {
            value: request.admitted,
            derivation: admission_derivation,
        };

        if let Some(term) = request.term {
            let closed = close(
                context.facts,
                &self.terms,
                &self.goals,
                &mut self.derivations,
            );
            if let Some(candidate) = closed.tight_bound(term, ZERO)
                && candidate < selected.value
                && let Some(derivation) =
                    closed.bound_proof(term, ZERO, candidate, &mut self.derivations)
            {
                selected = ProvedNumericUpperBound {
                    value: candidate,
                    derivation,
                };
            }
        }

        if let Some(form) = request.affine {
            let assumptions = Self::affine_facts(context.affine);
            if let Some(endpoint) = self
                .affine_closed_interval_proof(form, &assumptions, context.affine, context.facts)
                .map(|interval| interval.maximum)
                && endpoint.value < selected.value
            {
                let relation = request.term.map(|left| {
                    Box::new(Relation::Bound {
                        left,
                        right: ZERO,
                        bound: endpoint.value,
                    })
                });
                let derivation = self.derivations.intern(DerivationNode::AffineConsequence {
                    relation,
                    premises: endpoint.consequence.premises.into_boxed_slice(),
                    parents: endpoint.consequence.parents,
                });
                selected = ProvedNumericUpperBound {
                    value: endpoint.value,
                    derivation,
                };
            }
        }

        proof.numeric_upper_bound = Some(selected);
        proof
    }

    fn array_root_place(&self, root: &CheckedArrayRoot) -> PlaceTerm {
        match root {
            CheckedArrayRoot::Binding { binding, fields } => PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: self.is_holder(*binding),
                fields: fields.clone(),
            },
            CheckedArrayRoot::Constant(id) => PlaceTerm {
                root: PlaceRoot::Constant(*id),
                deref: false,
                fields: Vec::new(),
            },
        }
    }

    /// Interns the length term `len(P)` and, for an `array<T, N>` place,
    /// registers the [ENT-2] implicit length equality `len(P) = N` that holds
    /// at every program point. Every length term is created here, so the
    /// implicit equality is never missed at a site that only reads a length.
    fn length_term(&mut self, base: PlaceTerm, array_length: Option<CheckedConst>) -> TermId {
        let length_term = self.terms.intern(TermKind::Length(base));
        if let Some(length) = array_length {
            let bound = match length {
                CheckedConst::Value(value) => Some(LengthBound::Constant(i128::from(value))),
                CheckedConst::Parameter(declaration) => Some(LengthBound::Equal(
                    self.terms.intern(TermKind::ConstParameter(declaration)),
                )),
                // A symbolic derived length has no [ENT-2] term form; the
                // template registers no implicit equality and each concrete
                // instance, whose length is a value, registers the constant.
                CheckedConst::Derived(_) => None,
            };
            if let Some(bound) = bound {
                self.terms.set_length_bound(length_term, bound);
            }
        }
        length_term
    }

    /// [ENT-6]: the bounds obligation `i < len(P)`, normalized
    /// `i - len(P) <= -1`, discharged exactly when the closed fact state at
    /// the node derives it.
    fn judge_obligation(
        &mut self,
        base: PlaceTerm,
        array_length: Option<CheckedConst>,
        offset: &CheckedExpression,
        node_path: crate::NodePath,
        states: &ProofFlowState,
    ) {
        let length_term = self.length_term(base.clone(), array_length);
        let offset_term = self.read_operand(offset);
        let affine_offset = self
            .direct_goal_expression(offset)
            .and_then(|offset| self.affine_goal_value(&offset, &states.affine));
        let fixed_array_affine =
            self.affine_fixed_array_index_target(offset, array_length, &states.affine);
        let rendered_residual = format!(
            "{} < len({})",
            self.render_expression(offset),
            self.render_place(&base)
        );
        let request = BoundsRequest {
            left: offset_term,
            right: length_term,
            bound: -1,
            distinct: false,
        };
        let fixed_array_middle = match array_length {
            Some(CheckedConst::Value(length)) => {
                Some(self.terms.intern(TermKind::Constant(i128::from(length))))
            }
            Some(CheckedConst::Parameter(_) | CheckedConst::Derived(_)) | None => None,
        };
        let fixed_affine_bridge =
            fixed_array_affine
                .as_ref()
                .zip(fixed_array_middle)
                .map(|(target, middle)| FixedAffineBoundBridge {
                    target,
                    middle,
                    left_to_middle_bound: -1,
                });
        let proof = self.prove(
            ProofContext::new(&states.facts, &states.affine),
            ProofGoal::BoundedRelation(BoundedRelationGoal {
                canonical: None,
                request,
                direct_affine: None,
                fixed_affine_bridge,
                affine_left: affine_offset.as_ref(),
            }),
        );
        let discharged = proof.disposition == ProofDisposition::Proved;
        let contradictory = proof.route == Some(ProofRoute::Contradiction);
        let derivation = proof.derivation;
        let residual = (!discharged).then(|| rendered_residual.clone());
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::BoundsObligation(ordinal), root);
        }
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            family: ObligationFamily::Bounds,
            conjunct: 0,
            canonical_goal: None,
            components: vec![request],
            discharged,
            refuted: false,
            contradictory,
            residual,
            derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: if discharged {
                self.proved_affine_index_maps(affine_offset.as_ref(), states)
            } else {
                Vec::new()
            },
        });
    }

    /// Projects one already-computed exact offset value onto each active
    /// counted binder it depends on alone. The affine form is canonical, so
    /// the single-term test is complete for the deliberately small rule and
    /// its coefficient is nonzero by construction.
    fn proved_affine_index_maps(
        &self,
        offset: Option<&AffineForm>,
        states: &ProofFlowState,
    ) -> Vec<super::ProvedAffineIndexMap> {
        let Some(offset) = offset else {
            return Vec::new();
        };
        let [coefficient] = offset.terms() else {
            return Vec::new();
        };
        self.loops
            .iter()
            .filter_map(|frame| {
                let binder = frame.counted_binder?;
                let binder_term = states.affine.values.get(&binder)?.unit_term()?;
                (coefficient.term() == binder_term).then_some(super::ProvedAffineIndexMap {
                    loop_id: frame.id,
                    coefficient: coefficient.coefficient(),
                    constant: offset.constant_value(),
                })
            })
            .collect()
    }

    /// Forms the exact `offset <= N - 1` target for a fixed-size array.
    /// Dynamic buffer and slice lengths remain on the ordinary L0 route until
    /// their length term is connected to an affine value by a fixed rule.
    fn affine_fixed_array_index_target(
        &self,
        offset: &CheckedExpression,
        array_length: Option<CheckedConst>,
        state: &AffineFlowState,
    ) -> Option<AffineInequality> {
        let CheckedConst::Value(length) = array_length? else {
            return None;
        };
        let offset = self.direct_goal_expression(offset)?;
        let left = self.affine_goal_value(&offset, state)?;
        let right = AffineForm::constant(i128::from(length).checked_sub(1)?);
        AffineInequality::from_forms(&left, &right, &mut AffineCheckState::new()).ok()
    }

    fn affine_consequence_derivation(
        &mut self,
        target: &AffineInequality,
        relation: Option<Relation>,
        affine: &AffineFlowState,
        facts: &FactState,
    ) -> Option<DerivationId> {
        let assumptions = Self::affine_facts(affine);
        let proof = self.affine_target_proof(target, &assumptions, affine, facts)?;
        Some(self.derivations.intern(DerivationNode::AffineConsequence {
            relation: relation.map(Box::new),
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    fn fixed_affine_bound_derivation(
        &mut self,
        bridge: FixedAffineBoundBridge<'_>,
        left: TermId,
        right: TermId,
        requested: i128,
        affine: &AffineFlowState,
        facts: &FactState,
    ) -> Option<DerivationId> {
        let remaining = requested.checked_sub(bridge.left_to_middle_bound)?;
        let first_relation = Relation::Bound {
            left,
            right: bridge.middle,
            bound: bridge.left_to_middle_bound,
        };
        let first =
            self.affine_consequence_derivation(bridge.target, Some(first_relation), affine, facts)?;
        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        let second = closed.bound_proof(bridge.middle, right, remaining, &mut self.derivations)?;
        Some(self.derivations.intern(DerivationNode::TransitiveBound {
            left,
            middle: bridge.middle,
            right,
            bound: requested,
            first,
            second,
        }))
    }

    /// Combines one affine left-hand value with one already-live L0 bridge to
    /// the requested right-hand term. For a candidate middle term `m`, L0
    /// fixes `m - right <= c`; the single affine target is therefore
    /// `left - m <= requested - c`. Candidates are visited once in BindingId
    /// order, so work and selection are deterministic functions of the
    /// current checker state.
    #[allow(clippy::too_many_arguments)]
    fn affine_bound_via_l0_right(
        &mut self,
        left: &AffineForm,
        left_term: TermId,
        right_term: TermId,
        requested: i128,
        affine: &AffineFlowState,
        facts: &FactState,
    ) -> Option<DerivationId> {
        let mut bindings = affine.values.keys().copied().collect::<Vec<_>>();
        bindings.sort_by_key(|binding| binding.0);
        let candidates = bindings
            .into_iter()
            .filter_map(|binding| {
                let ty = self.affine_binding_type(binding)?;
                let value = affine.values.get(&binding)?.clone();
                let term = self.terms.intern(TermKind::Place(
                    PlaceTerm {
                        root: PlaceRoot::Binding(binding),
                        deref: false,
                        fields: Vec::new(),
                    },
                    ty,
                ));
                Some((term, value))
            })
            .collect::<Vec<_>>();
        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        if closed.contradictory() {
            return closed.contradiction_proof();
        }
        for (middle, middle_value) in candidates {
            let Some(bridge) = closed.tight_bound(middle, right_term) else {
                continue;
            };
            let Some(affine_bound) = requested.checked_sub(bridge) else {
                continue;
            };
            let Ok(right) = middle_value.add(
                &AffineForm::constant(affine_bound),
                &mut AffineCheckState::new(),
            ) else {
                continue;
            };
            let Ok(target) =
                AffineInequality::from_forms(left, &right, &mut AffineCheckState::new())
            else {
                continue;
            };
            let relation = Relation::Bound {
                left: left_term,
                right: middle,
                bound: affine_bound,
            };
            let Some(first) =
                self.affine_consequence_derivation(&target, Some(relation), affine, facts)
            else {
                continue;
            };
            let Some(second) =
                closed.bound_proof(middle, right_term, bridge, &mut self.derivations)
            else {
                continue;
            };
            return Some(self.derivations.intern(DerivationNode::TransitiveBound {
                left: left_term,
                middle,
                right: right_term,
                bound: requested,
                first,
                second,
            }));
        }
        None
    }

    /// Judges OP-9 through either the exact total `buffer_fits<T>(n)` goal
    /// or its one canonical L0 component. The component is used only in this
    /// direction: proving the comparison authorizes the allocation, while a
    /// predicate fact does not publish an ambient comparison fact.
    fn judge_allocation_fit(
        &mut self,
        element: CheckedType,
        maximum_length: u64,
        length: &CheckedExpression,
        node_path: crate::NodePath,
        states: &ProofFlowState,
    ) {
        let length_goal = self.direct_goal_expression(length);
        let canonical_goal = length_goal.map(|argument| GoalExpression::Operation {
            row: GoalOperation::BufferFits {
                element,
                maximum_length,
            },
            type_arguments: vec![element],
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: vec![argument],
        });
        let goal = canonical_goal
            .as_ref()
            .cloned()
            .map(|goal| self.intern_goal_expression(goal));
        let length_term = self.read_operand(length);
        let threshold_term = self
            .terms
            .intern(TermKind::Constant(i128::from(maximum_length)));
        let ordering_relation = length_term.map(|length| Relation::Bound {
            left: length,
            right: threshold_term,
            bound: 0,
        });
        let affine_length = self
            .direct_goal_expression(length)
            .and_then(|length| self.affine_goal_value(&length, &states.affine));
        let affine_target = affine_length.as_ref().and_then(|length| {
            AffineInequality::from_forms(
                length,
                &AffineForm::constant(i128::from(maximum_length)),
                &mut AffineCheckState::new(),
            )
            .ok()
        });
        let rendered = format!(
            "buffer_fits<{:?}>({})",
            element,
            self.render_expression(length)
        );

        let proof = self.prove(
            ProofContext::new(&states.facts, &states.affine),
            ProofGoal::NormalizedOrdering {
                goal,
                relation: ordering_relation.as_ref(),
                affine: affine_target.as_ref(),
                upper_bound: Some(NumericUpperBoundRequest {
                    term: length_term,
                    affine: affine_length.as_ref(),
                    admitted: i128::from(maximum_length),
                }),
            },
        );
        let discharged = proof.disposition == ProofDisposition::Proved;
        let refuted = proof.disposition == ProofDisposition::Refuted;
        let contradictory = proof.route == Some(ProofRoute::Contradiction);
        let derivation = proof.derivation;
        let allocation_length_upper_bound = proof
            .numeric_upper_bound
            .and_then(|bound| u64::try_from(bound.value).ok());
        let allocation_length_upper_bound_derivation =
            proof.numeric_upper_bound.map(|bound| bound.derivation);
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::BoundsObligation(ordinal), root);
        }
        if let Some(root) = allocation_length_upper_bound_derivation
            && Some(root) != derivation
        {
            self.derivations
                .add_root(DerivationRootKind::AllocationUpperBound(ordinal), root);
        }
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            family: ObligationFamily::AllocationFit,
            conjunct: 0,
            canonical_goal,
            components: vec![BoundsRequest {
                left: length_term,
                right: threshold_term,
                bound: 0,
                distinct: false,
            }],
            discharged,
            refuted,
            contradictory,
            residual: (!discharged).then(|| rendered.clone()),
            derivation,
            allocation_length_upper_bound,
            allocation_length_upper_bound_derivation,
            affine_index_maps: Vec::new(),
        });
    }

    fn judge_system_ranges(
        &mut self,
        operation: u8,
        node_path: &crate::NodePath,
        arguments: &[CheckedExpression],
        states: &ProofFlowState,
    ) {
        let Some(row) = SYSTEM_OPERATIONS.get(usize::from(operation)) else {
            return;
        };
        let Some(start_ordinal) = row
            .parameters
            .iter()
            .position(|parameter| parameter.name == "start")
        else {
            return;
        };
        let Some(end_ordinal) = row
            .parameters
            .iter()
            .position(|parameter| parameter.name == "end")
        else {
            return;
        };
        let Some((buffer_ordinal, buffer_parameter)) = row
            .parameters
            .iter()
            .enumerate()
            .find(|(_, parameter)| parameter.ty == crate::SystemTypeRef::BufferU8)
        else {
            return;
        };
        let (Some(start), Some(end), Some(buffer)) = (
            arguments.get(start_ordinal),
            arguments.get(end_ordinal),
            arguments.get(buffer_ordinal),
        ) else {
            return;
        };
        let start_goal = self.direct_goal_expression(start);
        let end_goal = self.direct_goal_expression(end);
        let start_term = self.read_operand(start);
        let end_term = self.read_operand(end);
        let comparison = |left, right| GoalExpression::Operation {
            row: GoalOperation::Integer {
                operation: super::super::model::CheckedIntegerOperation::LessEqual,
                operand_type: CheckedType::Integer(IntegerType::U64),
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: vec![left, right],
        };

        if let (Some(start_goal), Some(end_goal)) = (start_goal, end_goal.clone()) {
            self.judge_exact_relation_obligation(
                ObligationFamily::SystemRange,
                0,
                node_path.clone(),
                comparison(start_goal, end_goal),
                start_term,
                end_term.unwrap_or(ZERO),
                format!(
                    "{} <= {}",
                    self.render_expression(start),
                    self.render_expression(end)
                ),
                states,
            );
        } else {
            self.judge_unrepresentable_system_range(
                node_path,
                0,
                format!(
                    "{} <= {}",
                    self.render_expression(start),
                    self.render_expression(end)
                ),
                states,
            );
        }

        // The second conjunct bounds the end against the caller's own buffer,
        // so the residual names the caller's place — `wide <= len(header)` —
        // the way [OP-4]'s bounds residual does. Printing the operation's
        // declared parameter name instead leaves a writer with two buffers in
        // scope unable to tell which one the bound is about. The declared name
        // remains the fallback for an argument that carries no place at all.
        let buffer_root = match buffer {
            CheckedExpression::BorrowBuffer { root, .. } => {
                Some((root.binding, root.fields.clone(), root.element))
            }
            CheckedExpression::Binding {
                binding,
                ty: CheckedType::Buffer { element },
                ..
            } => Some((*binding, Vec::new(), *element)),
            _ => None,
        };
        let buffer_spelling = match &buffer_root {
            Some((binding, fields, _)) => self.render_place(&PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: self.is_holder(*binding),
                fields: fields.clone(),
            }),
            None => buffer_parameter.name.to_owned(),
        };
        let Some(end_goal) = end_goal else {
            self.judge_unrepresentable_system_range(
                node_path,
                1,
                format!("{} <= len({buffer_spelling})", self.render_expression(end)),
                states,
            );
            return;
        };
        let Some((buffer_binding, buffer_fields, buffer_element)) = buffer_root else {
            self.judge_unrepresentable_system_range(
                node_path,
                1,
                format!("{} <= len({buffer_spelling})", self.render_expression(end)),
                states,
            );
            return;
        };
        let buffer_goal = self.goal_binding_place(
            buffer_binding,
            buffer_fields.iter().copied().map(GoalProjection::Field),
            CheckedType::Buffer {
                element: buffer_element,
            },
        );
        let length_goal = GoalExpression::Operation {
            row: GoalOperation::BufferLength {
                element: buffer_element,
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Integer(IntegerType::U64),
            arguments: vec![buffer_goal],
        };
        let base = PlaceTerm {
            root: PlaceRoot::Binding(buffer_binding),
            deref: self.is_holder(buffer_binding),
            fields: buffer_fields,
        };
        let length_term = self.length_term(base, None);
        self.judge_exact_relation_obligation(
            ObligationFamily::SystemRange,
            1,
            node_path.clone(),
            comparison(end_goal, length_goal),
            end_term,
            length_term,
            format!("{} <= len({buffer_spelling})", self.render_expression(end)),
            states,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn judge_exact_relation_obligation(
        &mut self,
        family: ObligationFamily,
        conjunct: u8,
        node_path: crate::NodePath,
        root: GoalExpression,
        left: Option<TermId>,
        right: TermId,
        residual: String,
        states: &ProofFlowState,
    ) {
        let canonical_goal = root.clone();
        let request = BoundsRequest {
            left,
            right,
            bound: 0,
            distinct: false,
        };
        let direct_affine = self.affine_goal_ordering_target(&root, &states.affine);
        let affine_left = left.and_then(|term| self.affine_term_value(term, &states.affine));
        let proof = self.prove(
            ProofContext::new(&states.facts, &states.affine),
            ProofGoal::BoundedRelation(BoundedRelationGoal {
                canonical: Some(&root),
                request,
                direct_affine: direct_affine.as_ref(),
                fixed_affine_bridge: None,
                affine_left: affine_left.as_ref(),
            }),
        );
        let discharged = proof.disposition == ProofDisposition::Proved;
        let refuted = proof.disposition == ProofDisposition::Refuted;
        let contradictory = proof.route == Some(ProofRoute::Contradiction);
        let derivation = proof.derivation;
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::BoundsObligation(ordinal), root);
        }
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            family,
            conjunct,
            canonical_goal: Some(canonical_goal),
            components: vec![request],
            discharged,
            refuted,
            contradictory,
            residual: (!discharged).then(|| residual.clone()),
            derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
        });
    }

    fn affine_term_value(&self, term: TermId, state: &AffineFlowState) -> Option<AffineForm> {
        match self.terms.kind(term) {
            TermKind::Zero => Some(AffineForm::constant(0)),
            TermKind::Constant(value) => Some(AffineForm::constant(*value)),
            TermKind::Place(
                PlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    deref: false,
                    fields,
                },
                _,
            ) if fields.is_empty() => state.values.get(binding).cloned(),
            TermKind::ConstParameter(_)
            | TermKind::Place(_, _)
            | TermKind::ProjectedPlace(_, _)
            | TermKind::Length(_)
            | TermKind::ProjectedLength(_)
            | TermKind::CountedCapture { .. } => None,
        }
    }

    fn judge_unrepresentable_system_range(
        &mut self,
        node_path: &crate::NodePath,
        conjunct: u8,
        residual: String,
        states: &ProofFlowState,
    ) {
        let complete = close(
            &states.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        let contradictory = complete.contradictory();
        let derivation = contradictory.then(|| {
            complete
                .contradiction_proof()
                .expect("contradictory state has a proof")
        });
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            family: ObligationFamily::SystemRange,
            conjunct,
            canonical_goal: None,
            components: vec![BoundsRequest {
                left: None,
                right: ZERO,
                bound: 0,
                distinct: false,
            }],
            discharged: contradictory,
            refuted: false,
            contradictory,
            residual: (!contradictory).then(|| residual.clone()),
            derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
        });
    }

    /// [ENT-6] judges one proof-required exact integer operation. The source
    /// occurrence owns one canonical `.defined` goal and one obligation
    /// identity. Fixed L0 components are alternate derivations of that goal;
    /// they are never independent source obligations.
    fn judge_integer_domain_obligation(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
        node_path: &crate::NodePath,
        states: &mut ProofFlowState,
    ) {
        let canonical_goal = self.integer_domain_goal(operation, operand_type, arguments);
        let goal = canonical_goal
            .as_ref()
            .cloned()
            .map(|goal| self.intern_goal_expression(goal));
        let components = self.integer_domain_components(operation, operand_type, arguments);
        let residual = self.render_integer_domain_goal(operation, arguments);
        // Goal preparation may need to install a missing binding value image
        // before it can spell the fixed affine alternatives. Keep that work
        // local until the one proof query selects an affine route (or leaves
        // the goal unknown, matching the prior attempted-route state change).
        let candidate_atom_start = self.affine_atoms.len();
        let mut prepared_affine = states.affine.clone();
        let affine_clauses = self.affine_integer_domain_clauses(
            operation,
            operand_type,
            arguments,
            &mut prepared_affine,
        );
        let affine_product =
            self.affine_integer_product(operation, operand_type, arguments, &mut prepared_affine);

        let outcome = self.prove(
            ProofContext::new(&states.facts, &prepared_affine),
            ProofGoal::IntegerDomain(IntegerDomainGoal {
                canonical: goal,
                operation,
                operand_type,
                components: &components,
                affine_clauses: affine_clauses.as_deref(),
                affine_product: affine_product.as_ref(),
            }),
        );
        if outcome.route == Some(ProofRoute::Affine) || outcome.route.is_none() {
            states.affine = prepared_affine;
        } else {
            self.affine_atoms.truncate(candidate_atom_start);
        }
        let discharged = outcome.disposition == ProofDisposition::Proved;
        let refuted = outcome.disposition == ProofDisposition::Refuted;
        let contradictory = outcome.route == Some(ProofRoute::Contradiction);
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = outcome.derivation {
            self.derivations
                .add_root(DerivationRootKind::IntegerDomainObligation(ordinal), root);
        }
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            family: ObligationFamily::IntegerDomain,
            conjunct: 0,
            canonical_goal,
            components,
            discharged,
            refuted,
            contradictory,
            residual: (!discharged).then_some(residual),
            derivation: outcome.derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
        });
    }

    fn integer_domain_goal(
        &self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
    ) -> Option<GoalExpression> {
        Some(GoalExpression::Operation {
            row: GoalOperation::Integer {
                operation: operation.defined_query()?,
                operand_type,
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: arguments
                .iter()
                .map(|argument| self.direct_goal_expression(argument))
                .collect::<Option<Vec<_>>>()?,
        })
    }

    fn integer_domain_components(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
    ) -> Vec<BoundsRequest> {
        let operands = arguments
            .iter()
            .map(|argument| IntegerDomainOperand {
                term: self.read_operand(argument),
                constant: checked_integer_constant(argument),
            })
            .collect::<Vec<_>>();
        self.integer_domain_plan(operation, operand_type, &operands)
            .map_or_else(Vec::new, |plan| plan.components)
    }

    /// The one normalization authority attached to any goal family that has
    /// a fixed L0 interpretation. Integer domains may use a small DNF;
    /// AllocationFit is one conjunction containing its ceiling comparison.
    fn goal_normalization(&mut self, expression: &GoalExpression) -> Option<GoalNormalization> {
        if let Some(plan) = self.goal_integer_domain_plan(expression) {
            return Some(plan.normalization());
        }
        let GoalExpression::Operation {
            row: GoalOperation::BufferFits { maximum_length, .. },
            arguments,
            result: CheckedType::Bool,
            ..
        } = expression
        else {
            return None;
        };
        let [length] = arguments.as_slice() else {
            return None;
        };
        let threshold = self
            .terms
            .intern(TermKind::Constant(i128::from(*maximum_length)));
        Some(GoalNormalization::conjunction(vec![
            self.goal_operand(length).map(|length| Relation::Bound {
                left: length,
                right: threshold,
                bound: 0,
            }),
        ]))
    }

    fn goal_integer_domain_plan(
        &mut self,
        expression: &GoalExpression,
    ) -> Option<IntegerDomainPlan> {
        let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation,
                    operand_type,
                },
            arguments,
            result: CheckedType::Bool,
            ..
        } = expression
        else {
            return None;
        };
        if !operation.is_defined_query() {
            return None;
        }
        let mut operands = Vec::with_capacity(arguments.len());
        for argument in arguments {
            operands.push(IntegerDomainOperand {
                term: self.goal_operand(argument),
                constant: self.goal_integer_constant(argument),
            });
        }
        self.integer_domain_plan(*operation, *operand_type, &operands)
    }

    fn goal_integer_constant(&self, expression: &GoalExpression) -> Option<i128> {
        match expression {
            GoalExpression::Datum(GoalDatum::Literal(CheckedValue::Integer { ty, bits })) => {
                Some(integer_value(*ty, *bits))
            }
            GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) if projections.is_empty() => {
                let CheckedValue::Integer {
                    ty: value_type,
                    bits,
                } = &self.context.constant(*declaration)?.value
                else {
                    return None;
                };
                (*ty == CheckedType::Integer(*value_type))
                    .then(|| integer_value(*value_type, *bits))
            }
            _ => None,
        }
    }

    fn integer_domain_plan(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        operands: &[IntegerDomainOperand],
    ) -> Option<IntegerDomainPlan> {
        let fragment = fragment_type(operand_type)?;
        if matches!(
            operation,
            CheckedIntegerOperation::AddExact
                | CheckedIntegerOperation::AddDefined
                | CheckedIntegerOperation::SubtractExact
                | CheckedIntegerOperation::SubtractDefined
                | CheckedIntegerOperation::MultiplyExact
                | CheckedIntegerOperation::MultiplyDefined
        ) {
            let [left, right] = operands else {
                return None;
            };
            let conjuncts =
                overflow_conjuncts_for_values(operation, left.constant, right.constant, fragment)?;
            let operand = if conjuncts.ground {
                Some(ZERO)
            } else if left.constant.is_some() {
                right.term
            } else {
                left.term
            };
            return Some(IntegerDomainPlan {
                components: vec![
                    BoundsRequest {
                        left: conjuncts.ground.then_some(ZERO).or(operand),
                        right: ZERO,
                        bound: conjuncts.upper,
                        distinct: false,
                    },
                    BoundsRequest {
                        left: (conjuncts.ground || operand.is_some()).then_some(ZERO),
                        right: if conjuncts.ground {
                            ZERO
                        } else {
                            operand.unwrap_or(ZERO)
                        },
                        bound: conjuncts.lower,
                        distinct: false,
                    },
                ],
                kind: IntegerDomainPlanKind::Conjunction,
            });
        }

        if matches!(
            operation,
            CheckedIntegerOperation::DivideExact
                | CheckedIntegerOperation::DivideDefined
                | CheckedIntegerOperation::RemainderExact
                | CheckedIntegerOperation::RemainderDefined
        ) {
            let [dividend, divisor] = operands else {
                return None;
            };
            let mut components = vec![BoundsRequest {
                left: divisor.term,
                right: ZERO,
                bound: 0,
                distinct: true,
            }];
            let kind = if fragment.signed() {
                components.push(BoundsRequest {
                    left: dividend.term,
                    right: self
                        .terms
                        .intern(TermKind::Constant(type_range(fragment).0)),
                    bound: 0,
                    distinct: true,
                });
                components.push(BoundsRequest {
                    left: divisor.term,
                    right: self.terms.intern(TermKind::Constant(-1)),
                    bound: 0,
                    distinct: true,
                });
                IntegerDomainPlanKind::SignedDivision
            } else {
                components.push(BoundsRequest {
                    left: Some(ZERO),
                    right: ZERO,
                    bound: 0,
                    distinct: false,
                });
                IntegerDomainPlanKind::Conjunction
            };
            normalize_distinct_requests(&mut components);
            return Some(IntegerDomainPlan { components, kind });
        }

        if matches!(
            operation,
            CheckedIntegerOperation::AbsoluteExact
                | CheckedIntegerOperation::AbsoluteDefined
                | CheckedIntegerOperation::NegateExact
                | CheckedIntegerOperation::NegateDefined
        ) {
            let [operand] = operands else {
                return None;
            };
            let mut components = vec![BoundsRequest {
                left: operand.term,
                right: self
                    .terms
                    .intern(TermKind::Constant(type_range(fragment).0)),
                bound: 0,
                distinct: true,
            }];
            normalize_distinct_requests(&mut components);
            return Some(IntegerDomainPlan {
                components,
                kind: IntegerDomainPlanKind::Conjunction,
            });
        }

        if matches!(
            operation,
            CheckedIntegerOperation::ShiftLeftExact
                | CheckedIntegerOperation::ShiftLeftDefined
                | CheckedIntegerOperation::ShiftRightExact
                | CheckedIntegerOperation::ShiftRightDefined
        ) {
            let [_, amount] = operands else {
                return None;
            };
            return Some(IntegerDomainPlan {
                components: vec![BoundsRequest {
                    left: amount.term,
                    right: ZERO,
                    bound: i128::from(fragment.width()) - 1,
                    distinct: false,
                }],
                kind: IntegerDomainPlanKind::Conjunction,
            });
        }

        None
    }

    fn prove_integer_domain(
        &mut self,
        context: ProofContext<'_>,
        goal: IntegerDomainGoal<'_>,
    ) -> ProofResult {
        let finite = self.prove_integer_domain_finite(context, &goal);
        if finite.disposition != ProofDisposition::Unknown {
            return finite;
        }

        if let Some(derivation) = goal.affine_clauses.and_then(|clauses| {
            self.affine_integer_domain_derivation(
                clauses,
                context.affine,
                context.facts,
                goal.canonical,
            )
        }) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Affine),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        if let Some(derivation) = goal.affine_product.and_then(|product| {
            self.affine_integer_product_derivation(
                product,
                context.affine,
                context.facts,
                goal.canonical,
            )
        }) {
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Affine),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
        }
    }

    fn prove_integer_domain_finite(
        &mut self,
        context: ProofContext<'_>,
        goal: &IntegerDomainGoal<'_>,
    ) -> ProofResult {
        let closed = close(
            context.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        let contradictory = closed.contradictory();
        let signed_division = matches!(
            goal.operation,
            CheckedIntegerOperation::DivideExact | CheckedIntegerOperation::RemainderExact
        ) && fragment_type(goal.operand_type)
            .is_some_and(IntegerType::signed);
        let component_proof =
            |index: usize, derivations: &mut DerivationLedger| -> Option<DerivationId> {
                request_relation(goal.components.get(index)?)
                    .and_then(|relation| closed.relation_proof(&relation, derivations))
            };
        let parents = if contradictory {
            closed.contradiction_proof().map(|proof| vec![proof])
        } else if let Some(canonical) = goal
            .canonical
            .filter(|canonical| closed.derives_goal(*canonical, GoalSign::Positive, &self.goals))
        {
            closed
                .goal_proof(
                    canonical,
                    GoalSign::Positive,
                    &self.goals,
                    &mut self.derivations,
                )
                .map(|proof| vec![proof])
        } else if goal.canonical.is_none() && signed_division && goal.components.len() == 3 {
            component_proof(0, &mut self.derivations).and_then(|nonzero| {
                component_proof(1, &mut self.derivations)
                    .or_else(|| component_proof(2, &mut self.derivations))
                    .map(|witness| vec![nonzero, witness])
            })
        } else if goal.canonical.is_none() && !goal.components.is_empty() {
            goal.components
                .iter()
                .map(|request| {
                    request_relation(request).and_then(|relation| {
                        closed.relation_proof(&relation, &mut self.derivations)
                    })
                })
                .collect::<Option<Vec<_>>>()
        } else {
            None
        };
        if let Some(parents) = parents {
            let derivation = self.derivations.intern(DerivationNode::IntegerDomain {
                goal: goal.canonical,
                parents,
            });
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(if contradictory {
                    ProofRoute::Contradiction
                } else if goal.canonical.is_some() {
                    ProofRoute::FiniteGoal
                } else {
                    ProofRoute::L0
                }),
                derivation: Some(derivation),
                numeric_upper_bound: None,
            };
        }

        let component_false = |index: usize| {
            goal.components
                .get(index)
                .and_then(request_relation)
                .is_some_and(|relation| closed.derives(&relation.negated()))
        };
        let normalization_refuted = if goal.canonical.is_some() {
            false
        } else if signed_division && goal.components.len() == 3 {
            component_false(0) || (component_false(1) && component_false(2))
        } else {
            goal.components
                .iter()
                .filter_map(request_relation)
                .any(|relation| closed.derives(&relation.negated()))
        };
        let refuted = goal.canonical.is_some_and(|canonical| {
            closed.derives_goal(canonical, GoalSign::Negative, &self.goals)
        }) || normalization_refuted;
        if refuted {
            return ProofResult {
                disposition: ProofDisposition::Refuted,
                route: Some(if goal.canonical.is_some() {
                    ProofRoute::FiniteGoal
                } else {
                    ProofRoute::L0
                }),
                derivation: None,
                numeric_upper_bound: None,
            };
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
        }
    }

    /// Builds and proves the fixed affine range normalization of one exact
    /// integer operation from its already-evaluated operands.  This function
    /// never asks for the current operation's result image: using that image's
    /// result type here would circularly assume the domain being checked.
    fn affine_integer_domain_derivation(
        &mut self,
        clauses: &[Vec<AffineInequality>],
        affine: &AffineFlowState,
        facts: &FactState,
        goal: Option<GoalId>,
    ) -> Option<DerivationId> {
        let assumptions = Self::affine_facts(affine);
        for clause in clauses {
            let mut consequences = Vec::with_capacity(clause.len());
            let mut proved = true;
            for target in clause {
                let Some(proof) = self.affine_target_proof(target, &assumptions, affine, facts)
                else {
                    proved = false;
                    break;
                };
                consequences.push(self.derivations.intern(DerivationNode::AffineConsequence {
                    relation: None,
                    premises: proof.premises.into_boxed_slice(),
                    parents: proof.parents,
                }));
            }
            if proved {
                return Some(self.derivations.intern(DerivationNode::IntegerDomain {
                    goal,
                    parents: consequences,
                }));
            }
        }
        None
    }

    /// Selects the one nonlinear integer-domain rule. Both operands must be
    /// genuine affine values: constant multiplication stays on the ordinary
    /// affine path above, while every other nonlinear expression remains
    /// unavailable to this checker.
    fn affine_integer_product(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
        state: &mut AffineFlowState,
    ) -> Option<AffineIntegerProduct> {
        if !matches!(
            operation,
            CheckedIntegerOperation::MultiplyExact | CheckedIntegerOperation::MultiplyDefined
        ) {
            return None;
        }
        let CheckedType::Integer(ty) = operand_type else {
            return None;
        };
        let [left, right] = arguments else {
            return None;
        };
        let left = self.affine_pre_domain_form(left, state)?;
        let right = self.affine_pre_domain_form(right, state)?;
        if left.terms().is_empty() || right.terms().is_empty() {
            return None;
        }
        Some(AffineIntegerProduct { left, right, ty })
    }

    /// Applies the fixed interval-product rule. Once independent inclusive
    /// intervals are proved for the two affine operands, the product's extrema
    /// occur among exactly four endpoint pairs. All four products are formed
    /// with checked `i128` arithmetic before any range decision is made.
    fn affine_integer_product_derivation(
        &mut self,
        product: &AffineIntegerProduct,
        affine: &AffineFlowState,
        facts: &FactState,
        goal: Option<GoalId>,
    ) -> Option<DerivationId> {
        let assumptions = Self::affine_facts(affine);
        let left = self.affine_closed_interval_proof(&product.left, &assumptions, affine, facts)?;
        let right =
            self.affine_closed_interval_proof(&product.right, &assumptions, affine, facts)?;

        let products = [
            left.minimum.value.checked_mul(right.minimum.value)?,
            left.minimum.value.checked_mul(right.maximum.value)?,
            left.maximum.value.checked_mul(right.minimum.value)?,
            left.maximum.value.checked_mul(right.maximum.value)?,
        ];
        let (type_minimum, type_maximum) = type_range(product.ty);
        if products
            .iter()
            .any(|value| *value < type_minimum || *value > type_maximum)
        {
            return None;
        }

        let consequences = [
            left.minimum.consequence,
            left.maximum.consequence,
            right.minimum.consequence,
            right.maximum.consequence,
        ]
        .into_iter()
        .map(|proof| {
            self.derivations.intern(DerivationNode::AffineConsequence {
                relation: None,
                premises: proof.premises.into_boxed_slice(),
                parents: proof.parents,
            })
        })
        .collect();
        Some(self.derivations.intern(DerivationNode::IntegerDomain {
            goal,
            parents: consequences,
        }))
    }

    /// Computes the tightest endpoint found by the existing coefficient-one
    /// rule: first the L0/type interval alone, then each source invariant once
    /// in deterministic order. The final endpoint is reproved through
    /// `affine_target_proof`, so the retained consequence names the actual
    /// invariant premise and every selected L0 endpoint.
    fn affine_closed_interval_proof(
        &mut self,
        form: &AffineForm,
        assumptions: &[ActiveAffineFact],
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Option<AffineClosedIntervalProof> {
        let zero = AffineForm::constant(0);
        let upper_zero = Self::affine_less_equal(form, &zero)?;
        let lower_zero = Self::affine_less_equal(&zero, form)?;
        let constant = form.constant_value();

        let mut maximum = self
            .affine_lhs_maximum(&upper_zero, values, facts, &mut AffineCheckState::new())
            .ok()
            .flatten()
            .and_then(|terms| constant.checked_add(terms));
        let mut minimum = self
            .affine_lhs_maximum(&lower_zero, values, facts, &mut AffineCheckState::new())
            .ok()
            .flatten()
            .and_then(|terms| constant.checked_sub(terms));

        for assumption in assumptions {
            if let Ok(residual) = AffineInequality::residual_after(
                &upper_zero,
                &assumption.inequality,
                &mut AffineCheckState::new(),
            ) && let Some(candidate) = self
                .affine_lhs_maximum(&residual, values, facts, &mut AffineCheckState::new())
                .ok()
                .flatten()
                .and_then(|residual_maximum| {
                    constant
                        .checked_add(assumption.inequality.upper())?
                        .checked_add(residual_maximum)
                })
                && maximum.is_none_or(|current| candidate < current)
            {
                maximum = Some(candidate);
            }

            if let Ok(residual) = AffineInequality::residual_after(
                &lower_zero,
                &assumption.inequality,
                &mut AffineCheckState::new(),
            ) && let Some(candidate) = self
                .affine_lhs_maximum(&residual, values, facts, &mut AffineCheckState::new())
                .ok()
                .flatten()
                .and_then(|residual_maximum| {
                    constant
                        .checked_sub(assumption.inequality.upper())?
                        .checked_sub(residual_maximum)
                })
                && minimum.is_none_or(|current| candidate > current)
            {
                minimum = Some(candidate);
            }
        }

        let minimum = minimum?;
        let maximum = maximum?;
        if minimum > maximum {
            return None;
        }
        let minimum_target = Self::affine_less_equal(&AffineForm::constant(minimum), form)?;
        let maximum_target = Self::affine_less_equal(form, &AffineForm::constant(maximum))?;
        let minimum_proof =
            self.affine_target_proof(&minimum_target, assumptions, values, facts)?;
        let maximum_proof =
            self.affine_target_proof(&maximum_target, assumptions, values, facts)?;
        Some(AffineClosedIntervalProof {
            minimum: AffineIntervalEndpointProof {
                value: minimum,
                consequence: minimum_proof,
            },
            maximum: AffineIntervalEndpointProof {
                value: maximum,
                consequence: maximum_proof,
            },
        })
    }

    fn affine_integer_domain_clauses(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
        state: &mut AffineFlowState,
    ) -> Option<Vec<Vec<AffineInequality>>> {
        let CheckedType::Integer(ty) = operand_type else {
            return None;
        };
        if matches!(
            operation,
            CheckedIntegerOperation::ShiftLeftExact
                | CheckedIntegerOperation::ShiftLeftDefined
                | CheckedIntegerOperation::ShiftRightExact
                | CheckedIntegerOperation::ShiftRightDefined
        ) {
            let [_, amount] = arguments else {
                return None;
            };
            let amount = self.affine_pre_domain_form(amount, state)?;
            let target = Self::affine_less_equal(
                &amount,
                &AffineForm::constant(i128::from(ty.width()) - 1),
            )?;
            return Some(vec![vec![target]]);
        }
        if matches!(
            operation,
            CheckedIntegerOperation::AbsoluteExact | CheckedIntegerOperation::AbsoluteDefined
        ) {
            let [value] = arguments else {
                return None;
            };
            let value = self.affine_pre_domain_form(value, state)?;
            let minimum = type_range(ty).0;
            let target =
                Self::affine_less_equal(&AffineForm::constant(minimum.checked_add(1)?), &value)?;
            return Some(vec![vec![target]]);
        }
        if matches!(
            operation,
            CheckedIntegerOperation::DivideExact
                | CheckedIntegerOperation::DivideDefined
                | CheckedIntegerOperation::RemainderExact
                | CheckedIntegerOperation::RemainderDefined
        ) {
            let [dividend, divisor] = arguments else {
                return None;
            };
            let dividend = self.affine_pre_domain_form(dividend, state)?;
            let divisor = self.affine_pre_domain_form(divisor, state)?;
            let positive = Self::affine_less_equal(&AffineForm::constant(1), &divisor)?;
            if !ty.signed() {
                return Some(vec![vec![positive]]);
            }
            let negative = Self::affine_less_equal(&divisor, &AffineForm::constant(-1))?;
            let dividend_not_min = Self::affine_less_equal(
                &AffineForm::constant(type_range(ty).0.checked_add(1)?),
                &dividend,
            )?;
            let divisor_below_minus_one =
                Self::affine_less_equal(&divisor, &AffineForm::constant(-2))?;
            let divisor_above_minus_one =
                Self::affine_less_equal(&AffineForm::constant(0), &divisor)?;
            let nonzero = [negative, positive];
            let overflow_safe = [
                dividend_not_min,
                divisor_below_minus_one,
                divisor_above_minus_one,
            ];
            let mut clauses = Vec::with_capacity(nonzero.len() * overflow_safe.len());
            for nonzero in &nonzero {
                for overflow_safe in &overflow_safe {
                    clauses.push(vec![nonzero.clone(), overflow_safe.clone()]);
                }
            }
            return Some(clauses);
        }
        let result = match operation {
            CheckedIntegerOperation::AddExact | CheckedIntegerOperation::AddDefined => {
                let [left, right] = arguments else {
                    return None;
                };
                let left = self.affine_pre_domain_form(left, state)?;
                let right = self.affine_pre_domain_form(right, state)?;
                left.add(&right, &mut AffineCheckState::new()).ok()?
            }
            CheckedIntegerOperation::SubtractExact | CheckedIntegerOperation::SubtractDefined => {
                let [left, right] = arguments else {
                    return None;
                };
                let left = self.affine_pre_domain_form(left, state)?;
                let right = self.affine_pre_domain_form(right, state)?;
                left.subtract(&right, &mut AffineCheckState::new()).ok()?
            }
            CheckedIntegerOperation::MultiplyExact | CheckedIntegerOperation::MultiplyDefined => {
                let [left, right] = arguments else {
                    return None;
                };
                let left = self.affine_pre_domain_form(left, state)?;
                let right = self.affine_pre_domain_form(right, state)?;
                if left.terms().is_empty() {
                    right
                        .scale(left.constant_value(), &mut AffineCheckState::new())
                        .ok()?
                } else if right.terms().is_empty() {
                    left.scale(right.constant_value(), &mut AffineCheckState::new())
                        .ok()?
                } else {
                    return None;
                }
            }
            CheckedIntegerOperation::NegateExact | CheckedIntegerOperation::NegateDefined => {
                let [value] = arguments else {
                    return None;
                };
                self.affine_pre_domain_form(value, state)?
                    .scale(-1, &mut AffineCheckState::new())
                    .ok()?
            }
            _ => return None,
        };
        let (minimum, maximum) = type_range(ty);
        let mut check = AffineCheckState::new();
        Some(vec![vec![
            AffineInequality::from_forms(&result, &AffineForm::constant(maximum), &mut check)
                .ok()?,
            AffineInequality::from_forms(&AffineForm::constant(minimum), &result, &mut check)
                .ok()?,
        ]])
    }

    fn affine_less_equal(left: &AffineForm, right: &AffineForm) -> Option<AffineInequality> {
        AffineInequality::from_forms(left, right, &mut AffineCheckState::new()).ok()
    }

    /// Exact pre-domain value construction.  Unlike ordinary value flow it
    /// has no fresh-result fallback: failure to reconstruct the mathematical
    /// operands simply makes the affine proof route unavailable.
    fn affine_pre_domain_form(
        &mut self,
        expression: &CheckedExpression,
        state: &mut AffineFlowState,
    ) -> Option<AffineForm> {
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        if !events.is_empty() {
            return None;
        }
        match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
            | CheckedExpression::NamedConstant {
                value: CheckedValue::Integer { ty, bits },
                ..
            } => Some(AffineForm::constant(integer_value(*ty, *bits))),
            CheckedExpression::Binding { binding, ty, .. } => {
                let CheckedType::Integer(integer) = *ty else {
                    return None;
                };
                if self.affine_binding_type(*binding) != Some(integer) {
                    return None;
                }
                if let Some(value) = state.values.get(binding) {
                    Some(value.clone())
                } else {
                    let value = self.new_affine_atom(integer);
                    state.values.insert(*binding, value.clone());
                    Some(value)
                }
            }
            CheckedExpression::NumericConversion {
                source: CheckedNumericType::Integer(source),
                destination: CheckedNumericType::Integer(destination),
                value,
                ..
            } if source == destination || source.converts_totally_to(*destination) => {
                self.affine_pre_domain_form(value, state)
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                result: CheckedType::Integer(_),
                ..
            } => match operation {
                CheckedIntegerOperation::AddExact | CheckedIntegerOperation::AddDefined => {
                    let [left, right] = arguments.as_slice() else {
                        return None;
                    };
                    self.affine_pre_domain_form(left, state)?
                        .add(
                            &self.affine_pre_domain_form(right, state)?,
                            &mut AffineCheckState::new(),
                        )
                        .ok()
                }
                CheckedIntegerOperation::SubtractExact
                | CheckedIntegerOperation::SubtractDefined => {
                    let [left, right] = arguments.as_slice() else {
                        return None;
                    };
                    self.affine_pre_domain_form(left, state)?
                        .subtract(
                            &self.affine_pre_domain_form(right, state)?,
                            &mut AffineCheckState::new(),
                        )
                        .ok()
                }
                CheckedIntegerOperation::MultiplyExact
                | CheckedIntegerOperation::MultiplyDefined => {
                    let [left, right] = arguments.as_slice() else {
                        return None;
                    };
                    let left = self.affine_pre_domain_form(left, state)?;
                    let right = self.affine_pre_domain_form(right, state)?;
                    if left.terms().is_empty() {
                        right
                            .scale(left.constant_value(), &mut AffineCheckState::new())
                            .ok()
                    } else if right.terms().is_empty() {
                        left.scale(right.constant_value(), &mut AffineCheckState::new())
                            .ok()
                    } else {
                        None
                    }
                }
                CheckedIntegerOperation::NegateExact | CheckedIntegerOperation::NegateDefined => {
                    let [value] = arguments.as_slice() else {
                        return None;
                    };
                    self.affine_pre_domain_form(value, state)?
                        .scale(-1, &mut AffineCheckState::new())
                        .ok()
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn render_integer_domain_goal(
        &self,
        operation: CheckedIntegerOperation,
        arguments: &[CheckedExpression],
    ) -> String {
        let rendered = arguments
            .iter()
            .map(|argument| self.render_expression(argument))
            .collect::<Vec<_>>();
        match (operation, rendered.as_slice()) {
            (CheckedIntegerOperation::AddExact, [left, right]) => {
                format!("{left} +defined {right}")
            }
            (CheckedIntegerOperation::SubtractExact, [left, right]) => {
                format!("{left} -defined {right}")
            }
            (CheckedIntegerOperation::MultiplyExact, [left, right]) => {
                format!("{left} *defined {right}")
            }
            (CheckedIntegerOperation::DivideExact, [left, right]) => {
                format!("{left} /defined {right}")
            }
            (CheckedIntegerOperation::RemainderExact, [left, right]) => {
                format!("{left} %defined {right}")
            }
            (CheckedIntegerOperation::AbsoluteExact, [value]) => {
                format!("iabs.defined({value})")
            }
            (CheckedIntegerOperation::NegateExact, [value]) => {
                format!("ineg.defined({value})")
            }
            (CheckedIntegerOperation::ShiftLeftExact, [value, amount]) => {
                format!("ishl.defined({value}, {amount})")
            }
            (CheckedIntegerOperation::ShiftRightExact, [value, amount]) => {
                format!("ishr.defined({value}, {amount})")
            }
            _ => "<invalid integer-domain goal>".to_owned(),
        }
    }

    fn judge_set_target(&mut self, target: &CheckedSetTarget, states: &mut ProofFlowState) {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                let _ = self.judge_expression(&target.offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                self.judge_obligation(
                    base,
                    Some(target.length),
                    &target.offset,
                    target.obligation.clone(),
                    states,
                );
            }
            CheckedSetTarget::BufferIndex(target) => {
                let _ = self.judge_expression(&target.offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                self.judge_obligation(
                    base,
                    None,
                    &target.offset,
                    target.obligation.clone(),
                    states,
                );
            }
        }
    }

    // ------------------------------------------------------------------
    // Statement walk
    // ------------------------------------------------------------------

    /// Walks one block in its own lexical scope. Returns the fall-through:
    /// `true` when control continues past the block with `state` holding the
    /// post-scope-exit facts.
    fn affine_binding_type(&self, binding: BindingId) -> Option<IntegerType> {
        match self.summary(binding)?.ty? {
            CheckedType::Integer(ty) if !self.is_holder(binding) => Some(ty),
            _ => None,
        }
    }

    fn new_affine_atom(&mut self, ty: IntegerType) -> AffineForm {
        let (minimum, maximum) = type_range(ty);
        self.new_affine_atom_with_interval(ty, minimum, maximum)
    }

    fn new_affine_atom_with_interval(
        &mut self,
        ty: IntegerType,
        minimum: i128,
        maximum: i128,
    ) -> AffineForm {
        let index = u32::try_from(self.affine_atoms.len())
            .expect("affine value atoms exceed the u32 identity space");
        self.affine_atoms.push(AffineAtom {
            ty,
            minimum,
            maximum,
        });
        AffineForm::term(AffineTermId::from_index(index))
    }

    fn new_affine_binding_atom(&mut self, binding: BindingId) -> Option<AffineForm> {
        let ty = self.affine_binding_type(binding)?;
        Some(self.new_affine_atom(ty))
    }

    fn intersect_affine_facts(
        states: &[ProofFlowState],
        select: impl Fn(&AffineFlowState) -> &[ActiveAffineFact],
    ) -> Vec<ActiveAffineFact> {
        let Some(first) = states.first() else {
            return Vec::new();
        };
        select(&first.affine)
            .iter()
            .filter(|candidate| {
                states
                    .iter()
                    .skip(1)
                    .all(|state| select(&state.affine).contains(candidate))
            })
            .cloned()
            .collect()
    }

    /// Intersects source-proof facts by canonical numeric content, then
    /// records their predecessor sources for diagnostics.
    ///
    /// The first phase deliberately does not inspect `source`: an inequality
    /// survives exactly when every predecessor contains the same canonical
    /// value relation. Only after that decision does the second phase either
    /// reuse one common source reference or append a diagnostic join node.
    fn join_source_proof_facts(&mut self, states: &[ProofFlowState]) -> Vec<ActiveAffineFact> {
        let Some(first) = states.first() else {
            return Vec::new();
        };

        let mut common_inequalities = Vec::new();
        for candidate in &first.affine.proofs {
            if common_inequalities.contains(&candidate.inequality) {
                continue;
            }
            if states.iter().skip(1).all(|state| {
                state
                    .affine
                    .proofs
                    .iter()
                    .any(|fact| fact.inequality == candidate.inequality)
            }) {
                common_inequalities.push(candidate.inequality.clone());
            }
        }

        common_inequalities
            .into_iter()
            .map(|inequality| {
                let sources = states
                    .iter()
                    .map(|state| {
                        state
                            .affine
                            .proofs
                            .iter()
                            .filter(|fact| fact.inequality == inequality)
                            .map(|fact| fact.source)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                debug_assert!(sources.iter().all(|matches| !matches.is_empty()));

                let common_source = sources[0].iter().copied().find(|candidate| {
                    sources
                        .iter()
                        .skip(1)
                        .all(|matches| matches.contains(candidate))
                });
                let source = common_source.unwrap_or_else(|| {
                    let predecessors = sources
                        .iter()
                        .map(|matches| matches[0])
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let join_ordinal = u32::try_from(self.joined_source_proofs.len())
                        .expect("joined local invariant count exceeds the u32 identity space");
                    self.joined_source_proofs
                        .push(JoinedSourceProofProvenance { predecessors });
                    SourceAffineFactRef::JoinedSourceProof { join_ordinal }
                });
                ActiveAffineFact { source, inequality }
            })
            .collect()
    }

    fn join_affine_states(&mut self, states: &[ProofFlowState]) -> AffineFlowState {
        let Some(first) = states.first() else {
            return AffineFlowState::default();
        };
        let mut bindings = first.affine.values.keys().copied().collect::<Vec<_>>();
        bindings.sort_by_key(|binding| binding.0);
        let mut values = HashMap::new();
        for binding in bindings {
            let Some(first_value) = first.affine.values.get(&binding) else {
                continue;
            };
            if !states
                .iter()
                .skip(1)
                .all(|state| state.affine.values.contains_key(&binding))
            {
                continue;
            }
            let value = if states.iter().skip(1).all(|state| {
                state
                    .affine
                    .values
                    .get(&binding)
                    .is_some_and(|value| value == first_value)
            }) {
                first_value.clone()
            } else if let Some(ty) = self.affine_binding_type(binding) {
                let same_nonconstant_part = states.iter().skip(1).all(|state| {
                    state
                        .affine
                        .values
                        .get(&binding)
                        .is_some_and(|value| value.terms() == first_value.terms())
                });
                if same_nonconstant_part {
                    let mut minimum = first_value.constant_value();
                    let mut maximum = minimum;
                    for state in states.iter().skip(1) {
                        let constant = state
                            .affine
                            .values
                            .get(&binding)
                            .expect("one join binding exists on every input")
                            .constant_value();
                        minimum = minimum.min(constant);
                        maximum = maximum.max(constant);
                    }
                    let atom_start = self.affine_atoms.len();
                    let delta = self.new_affine_atom_with_interval(ty, minimum, maximum);
                    match first_value
                        .nonconstant_part()
                        .add(&delta, &mut AffineCheckState::new())
                    {
                        Ok(value) => value,
                        Err(_) => {
                            self.affine_atoms.truncate(atom_start);
                            self.new_affine_atom(ty)
                        }
                    }
                } else {
                    self.new_affine_atom(ty)
                }
            } else {
                continue;
            };
            values.insert(binding, value);
        }
        AffineFlowState {
            values,
            assumptions: Self::intersect_affine_facts(states, |state| &state.assumptions),
            exports: Self::intersect_affine_facts(states, |state| &state.exports),
            proofs: self.join_source_proof_facts(states),
            // Equality includes the exact value atoms and originating proof.
            // Therefore this only preserves the same incoming theorem. Two
            // branches that independently compute equal-looking divisions
            // need a separate value-delivery transfer before their facts can
            // be merged; silently equating their atoms would be unsound.
            division_images: first
                .affine
                .division_images
                .iter()
                .filter(|candidate| {
                    states
                        .iter()
                        .skip(1)
                        .all(|state| state.affine.division_images.contains(candidate))
                })
                .cloned()
                .collect(),
        }
    }

    fn join_flows(&mut self, states: &[ProofFlowState]) -> ProofFlowState {
        let facts = states
            .iter()
            .map(|states| states.facts.clone())
            .collect::<Vec<_>>();
        let event = self.derivations.event(FlowEventKind::Join, None);
        let entry_images = (0..self.entry_images.len())
            .map(|index| {
                states
                    .iter()
                    .filter_map(|state| state.entry_images[index])
                    .min()
            })
            .collect();
        ProofFlowState {
            facts: join_at(
                &facts,
                &self.terms,
                &self.goals,
                &mut self.derivations,
                event,
            ),
            entry_images,
            affine: self.join_affine_states(states),
        }
    }

    fn eligible_delivery_terms(
        &mut self,
        value: &CheckedExpression,
        receiver_type: CheckedType,
    ) -> Option<(BindingId, TermId, IntegerType)> {
        let CheckedExpression::Binding {
            binding,
            ty,
            consume_root: false,
            ..
        } = value
        else {
            return None;
        };
        let summary = self.summary(*binding)?;
        if !summary.delivery_carrier
            || summary.holder.is_some()
            || summary.implicit_deref
            || *ty != receiver_type
        {
            return None;
        }
        let fragment = fragment_type(*ty)?;
        let carrier = self.terms.intern(TermKind::Place(
            PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: false,
                fields: Vec::new(),
            },
            fragment,
        ));
        Some((*binding, carrier, fragment))
    }

    fn substitute_delivery_relation(
        relation: &Relation,
        carrier: TermId,
        receiver: TermId,
    ) -> Relation {
        let replace = |term| if term == carrier { receiver } else { term };
        match relation {
            Relation::Bound { left, right, bound } => Relation::Bound {
                left: replace(*left),
                right: replace(*right),
                bound: *bound,
            },
            Relation::Equal { left, right } => Relation::Equal {
                left: replace(*left),
                right: replace(*right),
            },
            Relation::Distinct { left, right } => {
                let (left, right) = (replace(*left), replace(*right));
                if left <= right {
                    Relation::Distinct { left, right }
                } else {
                    Relation::Distinct {
                        left: right,
                        right: left,
                    }
                }
            }
        }
    }

    fn delivery_edge_state(
        &mut self,
        closed: ClosedState,
        context: &DeliveryEdgeContext<'_>,
    ) -> FactState {
        if closed.contradictory() {
            return FactState::contradictory(
                closed
                    .contradiction_proof()
                    .expect("contradictory delivery edge has one exact proof"),
            );
        }
        let mut image = FactState::new();
        let mut explicit = HashMap::new();
        for (source_relation, parent) in closed.delivery_relations() {
            if !source_relation.terms().contains(&context.carrier)
                || !self
                    .derivations
                    .depends_on_explicit_relation(parent, &mut explicit)
            {
                continue;
            }
            let relation = Self::substitute_delivery_relation(
                &source_relation,
                context.carrier,
                context.receiver,
            );
            let proof = self.derivations.intern(DerivationNode::PostconditionGive {
                statement: context.statement.clone(),
                carrier: context.carrier_binding,
                receiver: context.receiver_binding,
                relation: Box::new(relation.clone()),
                event: context.event,
                parent,
            });
            image.establish_from_proof(&relation, proof, &self.derivations);
        }
        image
    }

    fn retain_delivery_give_parents(&mut self, parents: &[JoinParent]) {
        for parent in parents {
            if !matches!(
                self.derivations.nodes[parent.parent.0 as usize],
                DerivationNode::PostconditionGive { .. }
            ) {
                continue;
            }
            let occurrence = self.delivery_give_roots;
            self.delivery_give_roots = self
                .delivery_give_roots
                .checked_add(1)
                .expect("value-if give roots exceed the u32 identity space");
            self.derivations.add_root(
                DerivationRootKind::PostconditionGive { occurrence },
                parent.parent,
            );
        }
    }

    fn value_if_delivery_image(
        &mut self,
        value: &CheckedExpression,
        source: &ProofFlowState,
        context: DeliveryImageContext<'_>,
    ) -> ProofFlowState {
        let Some((carrier_binding, carrier, fragment)) =
            self.eligible_delivery_terms(value, context.receiver_type)
        else {
            return ProofFlowState::default();
        };
        let receiver = self.terms.intern(TermKind::Place(
            PlaceTerm {
                root: PlaceRoot::Binding(context.receiver_binding),
                deref: false,
                fields: Vec::new(),
            },
            fragment,
        ));
        // Every edge explicitly withholds the fresh receiver, including
        // edges visited after an earlier give interned the same stable term.
        // No implicit fact on x may participate in selecting d -> x.
        let facts = close_excluding_term(
            &source.facts,
            &self.terms,
            &self.goals,
            &mut self.derivations,
            receiver,
        );
        let event = self.proof_event(FlowEventKind::PostconditionGive, Some(context.statement));
        let edge = DeliveryEdgeContext {
            statement: context.statement,
            carrier_binding,
            receiver_binding: context.receiver_binding,
            carrier,
            receiver,
            event,
        };
        let mut image = ProofFlowState {
            facts: self.delivery_edge_state(facts, &edge),
            entry_images: Vec::new(),
            // Delivery-image construction currently exists only to retain
            // postcondition relations.  Withholding an affine image is
            // conservative; the normal value-initializer join installs the
            // receiver's value separately.
            affine: AffineFlowState::default(),
        };
        // The forward substitution happens above before the ordinary edge
        // kills, so the carrier's own branch scope cannot delete the image.
        self.kill_scopes_to(&mut image, context.scope_depth);
        self.exit_counted_loops_from(&mut image, context.loop_depth);
        image
    }

    fn establish_delivery_join(
        &mut self,
        images: &[FactState],
        context: &DeliveryJoinContext<'_>,
        target: &mut FactState,
    ) {
        assert!(images.iter().all(|image| {
            image.all_derivable
                || image.live_l0_relations().iter().all(|(_, proof)| {
                    matches!(
                        self.derivations.nodes[proof.0 as usize],
                        DerivationNode::PostconditionGive { .. }
                    )
                })
        }));
        let contributing = images
            .iter()
            .enumerate()
            .filter_map(|(index, image)| (!image.all_derivable).then_some(index))
            .collect::<Vec<_>>();
        let Some((&first_index, rest)) = contributing.split_first() else {
            return;
        };
        let first = &images[first_index];
        let mut bound_pairs = first.bounds.keys().copied().collect::<Vec<_>>();
        bound_pairs.sort_unstable();
        for pair in bound_pairs {
            if pair.0 != context.receiver && pair.1 != context.receiver {
                continue;
            }
            let mut weakest = first.bounds[&pair];
            if !rest.iter().all(|index| {
                images[*index].bounds.get(&pair).is_some_and(|bound| {
                    weakest = weakest.max(*bound);
                    true
                })
            }) {
                continue;
            }
            let parents = images
                .iter()
                .enumerate()
                .map(|(ordinal, image)| JoinParent {
                    ordinal: u32::try_from(ordinal)
                        .expect("delivery predecessor ordinal exceeds the u32 identity space"),
                    parent: if image.all_derivable {
                        image
                            .contradiction
                            .expect("contradictory delivery image has one proof")
                    } else {
                        image.bound_proofs[&pair]
                    },
                })
                .collect::<Vec<_>>();
            let relation = Relation::Bound {
                left: pair.0,
                right: pair.1,
                bound: weakest,
            };
            let proof = self
                .derivations
                .intern(DerivationNode::PostconditionDeliveryJoin {
                    detail: Box::new(super::state::PostconditionDeliveryJoinDetail {
                        statement: context.statement.clone(),
                        receiver: context.receiver_binding,
                        relation: relation.clone(),
                        event: context.event,
                        parents,
                    }),
                });
            let DerivationNode::PostconditionDeliveryJoin { detail } =
                &self.derivations.nodes[proof.0 as usize]
            else {
                unreachable!("just interned one delivery join")
            };
            let parents = detail.parents.clone();
            self.retain_delivery_give_parents(&parents);
            let occurrence = self.delivery_join_roots;
            self.delivery_join_roots = self
                .delivery_join_roots
                .checked_add(1)
                .expect("value-if delivery join roots exceed the u32 identity space");
            self.derivations.add_root(
                DerivationRootKind::PostconditionDeliveryJoin { occurrence },
                proof,
            );
            target.establish_from_proof(&relation, proof, &self.derivations);
        }

        let mut distinct = first.distinct.iter().copied().collect::<Vec<_>>();
        distinct.sort_unstable();
        for pair in distinct {
            if (pair.0 != context.receiver && pair.1 != context.receiver)
                || !rest
                    .iter()
                    .all(|index| images[*index].distinct.contains(&pair))
            {
                continue;
            }
            let parents = images
                .iter()
                .enumerate()
                .map(|(ordinal, image)| JoinParent {
                    ordinal: u32::try_from(ordinal)
                        .expect("delivery predecessor ordinal exceeds the u32 identity space"),
                    parent: if image.all_derivable {
                        image
                            .contradiction
                            .expect("contradictory delivery image has one proof")
                    } else {
                        image.distinct_proofs[&pair]
                    },
                })
                .collect::<Vec<_>>();
            let relation = Relation::Distinct {
                left: pair.0,
                right: pair.1,
            };
            let proof = self
                .derivations
                .intern(DerivationNode::PostconditionDeliveryJoin {
                    detail: Box::new(super::state::PostconditionDeliveryJoinDetail {
                        statement: context.statement.clone(),
                        receiver: context.receiver_binding,
                        relation: relation.clone(),
                        event: context.event,
                        parents,
                    }),
                });
            let DerivationNode::PostconditionDeliveryJoin { detail } =
                &self.derivations.nodes[proof.0 as usize]
            else {
                unreachable!("just interned one delivery join")
            };
            let parents = detail.parents.clone();
            self.retain_delivery_give_parents(&parents);
            let occurrence = self.delivery_join_roots;
            self.delivery_join_roots = self
                .delivery_join_roots
                .checked_add(1)
                .expect("value-if delivery join roots exceed the u32 identity space");
            self.derivations.add_root(
                DerivationRootKind::PostconditionDeliveryJoin { occurrence },
                proof,
            );
            target.establish_from_proof(&relation, proof, &self.derivations);
        }
    }

    fn establish_value_if_delivery_join(&mut self, frame: &GiveFrame, target: &mut ProofFlowState) {
        assert_eq!(frame.delivery_images.len(), frame.gives.len());
        assert_eq!(frame.delivery_edges.len(), frame.delivery_images.len());
        assert!(
            frame
                .delivery_edges
                .windows(2)
                .all(|pair| { pair[0].components().cmp(pair[1].components()).is_lt() })
        );
        let Some(fragment) = fragment_type(frame.result_type) else {
            return;
        };
        let receiver = self.terms.intern(TermKind::Place(
            PlaceTerm {
                root: PlaceRoot::Binding(frame.binding),
                deref: false,
                fields: Vec::new(),
            },
            fragment,
        ));
        let event = self.proof_event(
            FlowEventKind::PostconditionDeliveryJoin,
            Some(&frame.node_path),
        );
        let context = DeliveryJoinContext {
            statement: &frame.node_path,
            receiver_binding: frame.binding,
            receiver,
            event,
        };
        let facts = frame
            .delivery_images
            .iter()
            .map(|image| image.facts.clone())
            .collect::<Vec<_>>();
        self.establish_delivery_join(&facts, &context, &mut target.facts);
    }

    fn walk_block(&mut self, statements: &[CheckedStatement], state: &mut ProofFlowState) -> bool {
        self.scopes.push(Vec::new());
        let mut continues = true;
        for statement in statements {
            if !continues {
                break;
            }
            continues = self.walk_statement(statement, state);
        }
        if continues {
            let depth = self.scopes.len() - 1;
            self.exit_scopes_to(state, depth);
        }
        self.scopes.pop();
        continues
    }

    fn declare(&mut self, binding: BindingId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(binding);
        }
    }

    fn affine_unknown_integer(&mut self, ty: CheckedType) -> Option<AffineForm> {
        let CheckedType::Integer(ty) = ty else {
            return None;
        };
        Some(self.new_affine_atom(ty))
    }

    fn affine_expression_form(
        &mut self,
        expression: &CheckedExpression,
        state: &mut AffineFlowState,
    ) -> Option<AffineForm> {
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        if !events.is_empty() {
            return self.affine_unknown_integer(expression.ty());
        }
        self.affine_pure_expression_form(expression, state)
    }

    /// Retains the second fixed consequence of one S7 unsigned division:
    /// for `q = a / k` with a positive written literal `k`, `k*q <= a`.
    /// Both sides are the exact affine value images computed at this program
    /// point, so later writes receive different atoms and cannot inherit it.
    fn establish_unsigned_division_image(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        established: sources::EstablishedUnsignedDivision,
        state: &mut AffineFlowState,
    ) {
        let CheckedExpression::IntegerOperation { arguments, .. } = value else {
            return;
        };
        let [dividend, _divisor] = arguments.as_slice() else {
            return;
        };
        let Some(quotient) = state.values.get(&binding).cloned() else {
            return;
        };
        let Some(dividend) = self.affine_pre_domain_form(dividend, state) else {
            return;
        };
        let Ok(scaled_quotient) = quotient.scale(established.divisor, &mut AffineCheckState::new())
        else {
            return;
        };
        let Some(inequality) = Self::affine_less_equal(&scaled_quotient, &dividend) else {
            return;
        };
        state.division_images.push(AutomaticAffineFact {
            inequality,
            parent: established.parent,
        });
    }

    fn affine_pure_expression_form(
        &mut self,
        expression: &CheckedExpression,
        state: &mut AffineFlowState,
    ) -> Option<AffineForm> {
        let formed = match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
            | CheckedExpression::NamedConstant {
                value: CheckedValue::Integer { ty, bits },
                ..
            } => Some(AffineForm::constant(integer_value(*ty, *bits))),
            CheckedExpression::Binding { binding, ty, .. } => {
                let CheckedType::Integer(integer) = *ty else {
                    return None;
                };
                if self.affine_binding_type(*binding) != Some(integer) {
                    Some(self.new_affine_atom(integer))
                } else if let Some(value) = state.values.get(binding) {
                    Some(value.clone())
                } else {
                    let value = self.new_affine_atom(integer);
                    state.values.insert(*binding, value.clone());
                    Some(value)
                }
            }
            CheckedExpression::NumericConversion {
                source: CheckedNumericType::Integer(source),
                destination: CheckedNumericType::Integer(destination),
                value,
                ..
            } if source == destination || source.converts_totally_to(*destination) => {
                self.affine_pure_expression_form(value, state)
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                result: CheckedType::Integer(_),
                ..
            } => {
                let [left, right] = arguments.as_slice() else {
                    return self.affine_unknown_integer(expression.ty());
                };
                let left = self.affine_pure_expression_form(left, state)?;
                let right = self.affine_pure_expression_form(right, state)?;
                let mut check = AffineCheckState::new();
                match operation {
                    CheckedIntegerOperation::AddExact | CheckedIntegerOperation::AddDefined => {
                        left.add(&right, &mut check).ok()
                    }
                    CheckedIntegerOperation::SubtractExact
                    | CheckedIntegerOperation::SubtractDefined => {
                        left.subtract(&right, &mut check).ok()
                    }
                    CheckedIntegerOperation::MultiplyExact
                    | CheckedIntegerOperation::MultiplyDefined => {
                        if left.terms().is_empty() {
                            right.scale(left.constant_value(), &mut check).ok()
                        } else if right.terms().is_empty() {
                            left.scale(right.constant_value(), &mut check).ok()
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        formed.or_else(|| self.affine_unknown_integer(expression.ty()))
    }

    fn checked_affine_form(
        &mut self,
        expression: &CheckedAffineExpression,
        state: &mut AffineFlowState,
        check: &mut AffineCheckState,
    ) -> Option<AffineForm> {
        enum Pending<'expression> {
            Visit(&'expression CheckedAffineExpression),
            Add,
            Subtract,
            Scale(i128),
        }

        let mut pending = vec![Pending::Visit(expression)];
        let mut values = Vec::new();
        while let Some(next) = pending.pop() {
            match next {
                Pending::Visit(expression) => match &expression.kind {
                    CheckedAffineExpressionKind::Constant { value, .. } => {
                        values.push(AffineForm::constant(*value));
                    }
                    CheckedAffineExpressionKind::Local { binding, .. } => {
                        let value = if let Some(value) = state.values.get(binding) {
                            value.clone()
                        } else {
                            let value = self.new_affine_binding_atom(*binding)?;
                            state.values.insert(*binding, value.clone());
                            value
                        };
                        values.push(value);
                    }
                    CheckedAffineExpressionKind::Add(left, right) => {
                        pending.push(Pending::Add);
                        pending.push(Pending::Visit(right));
                        pending.push(Pending::Visit(left));
                    }
                    CheckedAffineExpressionKind::Subtract(left, right) => {
                        pending.push(Pending::Subtract);
                        pending.push(Pending::Visit(right));
                        pending.push(Pending::Visit(left));
                    }
                    CheckedAffineExpressionKind::MultiplyByConstant {
                        constant, value, ..
                    } => {
                        pending.push(Pending::Scale(*constant));
                        pending.push(Pending::Visit(value));
                    }
                },
                Pending::Add => {
                    let right = values.pop()?;
                    let left = values.pop()?;
                    values.push(left.add(&right, check).ok()?);
                }
                Pending::Subtract => {
                    let right = values.pop()?;
                    let left = values.pop()?;
                    values.push(left.subtract(&right, check).ok()?);
                }
                Pending::Scale(constant) => {
                    let value = values.pop()?;
                    values.push(value.scale(constant, check).ok()?);
                }
            }
        }
        let result = values.pop()?;
        values.is_empty().then_some(result)
    }

    fn checked_loop_invariant_inequality(
        &mut self,
        invariant: &CheckedLoopInvariant,
        state: &mut AffineFlowState,
        check: &mut AffineCheckState,
    ) -> Option<AffineInequality> {
        self.checked_affine_relation_inequality(&invariant.relation, state, check)
    }

    /// INV-1 base is a simultaneous batch: every target is checked against
    /// the same preheader state before any invariant from the batch becomes an
    /// assumption.
    fn prove_loop_invariant_bases(
        &mut self,
        invariants: &[CheckedLoopInvariant],
        state: &mut ProofFlowState,
    ) -> Vec<bool> {
        invariants
            .iter()
            .map(|invariant| {
                let target = self.checked_loop_invariant_inequality(
                    invariant,
                    &mut state.affine,
                    &mut AffineCheckState::new(),
                );
                target.as_ref().is_some_and(|target| {
                    self.prove(
                        ProofContext::new(&state.facts, &state.affine),
                        ProofGoal::Affine { inequality: target },
                    )
                    .disposition
                        == ProofDisposition::Proved
                })
            })
            .collect()
    }

    /// Installs the complete invariant batch at a generic loop header only
    /// after every base judgment succeeded. No source-order prefix can lend
    /// authority to a later base case.
    fn activate_loop_invariant_batch(
        &mut self,
        loop_id: CheckedLoopId,
        invariants: &[CheckedLoopInvariant],
        base_batch: bool,
        state: &mut AffineFlowState,
    ) {
        for (source_ordinal, invariant) in invariants.iter().enumerate() {
            let target = self.checked_loop_invariant_inequality(
                invariant,
                state,
                &mut AffineCheckState::new(),
            );
            if base_batch && let Some(inequality) = target {
                state.assumptions.push(ActiveAffineFact {
                    source: SourceAffineFactRef::LoopInvariant(SourceLoopInvariantRef {
                        loop_id,
                        source_ordinal: u32::try_from(source_ordinal)
                            .expect("loop invariant ordinal exceeds u32"),
                    }),
                    inequality,
                });
            }
        }
    }

    fn record_loop_invariant_outcomes(
        &mut self,
        loop_id: CheckedLoopId,
        invariants: &[CheckedLoopInvariant],
        base: &[bool],
        step: &[Option<bool>],
    ) {
        for (index, invariant) in invariants.iter().enumerate() {
            self.loop_invariants.push(LoopInvariantOutcome {
                node_path: invariant.relation.node_path.clone(),
                loop_id,
                source_ordinal: u32::try_from(index).expect("loop invariant ordinal exceeds u32"),
                name: invariant.name.clone(),
                proof: LoopInvariantProof {
                    base: base[index],
                    step: step[index],
                },
            });
        }
    }

    fn checked_affine_relation_inequality(
        &mut self,
        relation: &CheckedAffineRelation,
        state: &mut AffineFlowState,
        check: &mut AffineCheckState,
    ) -> Option<AffineInequality> {
        let left = self.checked_affine_form(&relation.left, state, check)?;
        let right = self.checked_affine_form(&relation.right, state, check)?;
        AffineInequality::from_bounded_forms(&left, &right, relation.bound, check).ok()
    }

    /// Projects the exact source relation into L0 when its normalized binding
    /// coefficients have one of the fixed difference-bound shapes. This does
    /// no discovery: it only recognizes `x - y <= c`, `x <= c`, `c <= x`, or
    /// a constant proposition after the source-written affine arithmetic has
    /// been normalized.
    fn checked_affine_relation_l0(&mut self, relation: &CheckedAffineRelation) -> Option<Relation> {
        fn source_form(
            expression: &CheckedAffineExpression,
            bindings: &mut Vec<BindingId>,
            check: &mut AffineCheckState,
        ) -> Option<AffineForm> {
            match &expression.kind {
                CheckedAffineExpressionKind::Constant { value, .. } => {
                    Some(AffineForm::constant(*value))
                }
                CheckedAffineExpressionKind::Local { binding, .. } => {
                    let index = bindings
                        .iter()
                        .position(|candidate| candidate == binding)
                        .unwrap_or_else(|| {
                            bindings.push(*binding);
                            bindings.len() - 1
                        });
                    let index = u32::try_from(index).ok()?;
                    Some(AffineForm::term(AffineTermId::from_index(index)))
                }
                CheckedAffineExpressionKind::Add(left, right) => {
                    source_form(left, bindings, check)?
                        .add(&source_form(right, bindings, check)?, check)
                        .ok()
                }
                CheckedAffineExpressionKind::Subtract(left, right) => {
                    source_form(left, bindings, check)?
                        .subtract(&source_form(right, bindings, check)?, check)
                        .ok()
                }
                CheckedAffineExpressionKind::MultiplyByConstant {
                    constant, value, ..
                } => source_form(value, bindings, check)?
                    .scale(*constant, check)
                    .ok(),
            }
        }

        let mut bindings = Vec::new();
        let mut check = AffineCheckState::new();
        let left = source_form(&relation.left, &mut bindings, &mut check)?;
        let right = source_form(&relation.right, &mut bindings, &mut check)?;
        let inequality =
            AffineInequality::from_bounded_forms(&left, &right, relation.bound, &mut check).ok()?;
        let mut term = |coefficient: super::affine::AffineCoefficient| {
            let binding = *bindings.get(coefficient.term().index() as usize)?;
            let fragment = fragment_type(CheckedType::Integer(self.affine_binding_type(binding)?))?;
            Some(self.terms.intern(TermKind::Place(
                PlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    deref: false,
                    fields: Vec::new(),
                },
                fragment,
            )))
        };
        let (left, right) = match inequality.terms() {
            [] => (ZERO, ZERO),
            [coefficient] if coefficient.coefficient() == 1 => (term(*coefficient)?, ZERO),
            [coefficient] if coefficient.coefficient() == -1 => (ZERO, term(*coefficient)?),
            [first, second] => match (first.coefficient(), second.coefficient()) {
                (1, -1) => (term(*first)?, term(*second)?),
                (-1, 1) => (term(*second)?, term(*first)?),
                _ => return None,
            },
            _ => return None,
        };
        Some(Relation::Bound {
            left,
            right,
            bound: inequality.upper(),
        })
    }

    fn source_proof_check(
        &mut self,
        premises: &[Option<AffineInequality>],
        l0_premises: &[Option<Relation>],
        combination: Result<bool, SourceProofCertificateFailure>,
        redundant: bool,
        values: &AffineFlowState,
        facts: &FactState,
    ) -> SourceProofCheck {
        let premises = premises
            .iter()
            .zip(l0_premises)
            .map(|(premise, relation)| {
                let Some(premise) = premise.as_ref() else {
                    return false;
                };
                let goal = relation.as_ref().map_or(
                    ProofGoal::Affine {
                        inequality: premise,
                    },
                    |relation| ProofGoal::Ordering {
                        relation,
                        affine: Some(premise),
                    },
                );
                self.prove(ProofContext::new(facts, values), goal)
                    .disposition
                    == ProofDisposition::Proved
            })
            .collect();
        let certificate_failure = combination.as_ref().err().copied();
        SourceProofCheck {
            premises,
            combination: combination.unwrap_or(false),
            certificate_failure,
            redundant,
        }
    }

    /// Checks the one weighted combination the source writer selected.
    ///
    /// The written premises are multiplied and summed exactly in source order.
    /// The remaining proposition `target - sum` may be discharged only by the
    /// existing direct L0 closure or fixed interval rule at the entering
    /// program point. This route never selects another affine premise, derives
    /// a multiplier, or retries a subset.
    fn source_proof_combination(
        &mut self,
        target: &AffineInequality,
        premises: &[(AffineInequality, i128)],
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Result<bool, SourceProofCertificateFailure> {
        let actual = u32::try_from(premises.len()).unwrap_or(u32::MAX);
        if premises.len() > MAX_CERTIFICATE_PREMISES {
            return Err(SourceProofCertificateFailure::UseCapacity {
                maximum: u32::try_from(MAX_CERTIFICATE_PREMISES)
                    .expect("certificate capacity fits u32"),
                actual,
            });
        }

        let mut first_by_premise = HashMap::new();
        for (index, (premise, factor)) in premises.iter().enumerate() {
            let index = u32::try_from(index).expect("certificate capacity fits u32");
            if *factor <= 0 {
                return Err(SourceProofCertificateFailure::InvalidFactor { use_index: index });
            }
            if let Some(first) = first_by_premise.insert(premise.clone(), index) {
                return Err(SourceProofCertificateFailure::RepeatedUse {
                    first,
                    repeated: index,
                });
            }
        }

        let scaled = premises
            .iter()
            .map(|(inequality, factor)| ScaledAffinePremise {
                inequality,
                factor: *factor,
            })
            .collect::<Vec<_>>();
        let mut check = AffineCheckState::new();
        let sum = match sum_explicit_scaled_inequalities(&scaled, &mut check) {
            Ok(sum) => sum,
            Err(AffineCheckError::ArithmeticOverflow) => {
                return Err(SourceProofCertificateFailure::ArithmeticOverflow);
            }
            Err(AffineCheckError::LimitExceeded(AffineCheckLimit::CertificatePremises)) => {
                return Err(SourceProofCertificateFailure::UseCapacity {
                    maximum: u32::try_from(MAX_CERTIFICATE_PREMISES)
                        .expect("certificate capacity fits u32"),
                    actual,
                });
            }
            Err(AffineCheckError::LimitExceeded(_)) => {
                return Err(SourceProofCertificateFailure::FormationCapacity);
            }
            Err(AffineCheckError::InvalidCertificateFactor) => {
                let use_index = premises
                    .iter()
                    .position(|(_, factor)| *factor <= 0)
                    .and_then(|index| u32::try_from(index).ok())
                    .unwrap_or(0);
                return Err(SourceProofCertificateFailure::InvalidFactor { use_index });
            }
            Err(AffineCheckError::CoefficientMismatch) => return Ok(false),
        };
        let residual = match AffineInequality::residual_after(target, &sum, &mut check) {
            Ok(residual) => residual,
            Err(AffineCheckError::ArithmeticOverflow) => {
                return Err(SourceProofCertificateFailure::ArithmeticOverflow);
            }
            Err(AffineCheckError::LimitExceeded(_)) => {
                return Err(SourceProofCertificateFailure::FormationCapacity);
            }
            Err(
                AffineCheckError::CoefficientMismatch | AffineCheckError::InvalidCertificateFactor,
            ) => return Ok(false),
        };
        let candidates = self.affine_l0_candidates(values);
        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        let l0 = self.affine_l0_index(&candidates, &closed, &mut check);
        Ok(matches!(
            self.affine_residual_proof(&residual, &l0, values, &closed, &mut check),
            Ok(Some(_))
        ))
    }

    /// Exact maximum of one affine left-hand side under the independently
    /// known L0/type interval of each atom. This is numeric discovery only;
    /// callers must subsequently prove any selected endpoint with
    /// `affine_target_proof` before it can discharge a source obligation.
    fn affine_lhs_maximum(
        &mut self,
        inequality: &AffineInequality,
        values: &AffineFlowState,
        facts: &FactState,
        check: &mut AffineCheckState,
    ) -> Result<Option<i128>, AffineCheckError> {
        let mut requested = inequality
            .terms()
            .iter()
            .map(|coefficient| coefficient.term())
            .collect::<Vec<_>>();
        requested.sort_unstable();
        requested.dedup();

        let mut term_intervals = HashMap::new();
        for atom_id in requested {
            let atom = *self
                .affine_atoms
                .get(atom_id.index() as usize)
                .ok_or(AffineCheckError::CoefficientMismatch)?;
            let (minimum, maximum) = (atom.minimum, atom.maximum);
            let mut bindings = values
                .values
                .iter()
                .filter_map(|(binding, value)| {
                    (value.unit_term() == Some(atom_id)).then_some(*binding)
                })
                .collect::<Vec<_>>();
            bindings.sort_by_key(|binding| binding.0);
            let terms = bindings
                .into_iter()
                .filter_map(|binding| {
                    if self.affine_binding_type(binding) != Some(atom.ty) {
                        return None;
                    }
                    Some(self.terms.intern(TermKind::Place(
                        PlaceTerm {
                            root: PlaceRoot::Binding(binding),
                            deref: false,
                            fields: Vec::new(),
                        },
                        atom.ty,
                    )))
                })
                .collect::<Vec<_>>();
            term_intervals.insert(atom_id, (minimum, maximum, terms));
        }

        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        if closed.contradictory() {
            return Ok(None);
        }
        let intervals = term_intervals
            .into_iter()
            .map(|(atom, (mut minimum, mut maximum, terms))| {
                for term in terms {
                    if let Some(upper) = closed.tight_bound(term, ZERO) {
                        maximum = maximum.min(upper);
                    }
                    if let Some(negative_lower) = closed.tight_bound(ZERO, term)
                        && let Some(lower) = negative_lower.checked_neg()
                    {
                        minimum = minimum.max(lower);
                    }
                }
                (atom, (minimum, maximum))
            })
            .collect::<HashMap<_, _>>();
        interval_maximum(
            inequality.terms(),
            |term| intervals.get(&term).copied(),
            check,
        )
    }

    /// Builds the fixed L0 vocabulary before closure. Each live integer
    /// binding contributes its ordinary term and its exact current affine
    /// value; Z is the fixed zero candidate. Later matching never invents a
    /// term after the closed state was formed.
    fn affine_l0_candidates(&mut self, values: &AffineFlowState) -> Vec<AffineL0Candidate> {
        let mut candidates = vec![AffineL0Candidate {
            term: ZERO,
            value: AffineForm::constant(0),
        }];
        let mut bindings = values.values.keys().copied().collect::<Vec<_>>();
        bindings.sort_by_key(|binding| binding.0);
        for binding in bindings {
            let Some(ty) = self.affine_binding_type(binding) else {
                continue;
            };
            let term = self.terms.intern(TermKind::Place(
                PlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    deref: false,
                    fields: Vec::new(),
                },
                ty,
            ));
            candidates.push(AffineL0Candidate {
                term,
                value: values.values[&binding].clone(),
            });
        }
        candidates
    }

    /// Builds the goal-query index for ordinary difference bounds.
    ///
    /// This is an ephemeral view over the already-closed L0 state, not a copy
    /// of `FactState::bounds` in the affine premise set. For each canonical
    /// affine coefficient vector it retains the strongest live L0 image. A
    /// target or residual can therefore query exactly its own vector without
    /// making every L0 edge participate in affine premise enumeration.
    fn affine_l0_index(
        &self,
        candidates: &[AffineL0Candidate],
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> AffineL0Index {
        let mut index = AffineL0Index::default();
        for left in candidates {
            for right in candidates {
                let Some(bound) = closed.tight_bound(left.term, right.term) else {
                    continue;
                };
                let Ok(inequality) =
                    AffineInequality::from_bounded_forms(&left.value, &right.value, bound, check)
                else {
                    // This L0 image is outside the affine i128 vocabulary.
                    // It cannot suppress another representable image.
                    continue;
                };
                let key: Box<[AffineCoefficient]> = inequality.terms().into();
                if let Some(existing) = index.by_terms.get(&key).copied() {
                    if inequality.upper() < index.entries[existing].inequality.upper() {
                        index.entries[existing] = AffineL0Entry {
                            inequality,
                            left: left.term,
                            right: right.term,
                            bound,
                        };
                    }
                    continue;
                }
                let entry = index.entries.len();
                index.by_terms.insert(key, entry);
                index.entries.push(AffineL0Entry {
                    inequality,
                    left: left.term,
                    right: right.term,
                    bound,
                });
            }
        }
        index
    }

    /// Collects only explicit source-affine facts and automatic value images.
    /// Ordinary difference bounds remain in L0 and are queried through
    /// [`Self::affine_l0_index`] for the concrete target or residual.
    fn automatic_affine_premises(
        &self,
        assumptions: &[ActiveAffineFact],
        images: &[AutomaticAffineFact],
        check: &mut AffineCheckState,
    ) -> Result<Vec<AutomaticAffinePremise>, AffineCheckError> {
        let mut premises = Vec::new();
        let mut seen = HashSet::new();
        for assumption in assumptions {
            check.charge(1)?;
            if seen.insert(assumption.inequality.clone()) {
                premises.push(AutomaticAffinePremise {
                    inequality: assumption.inequality.clone(),
                    source: Some(assumption.source),
                    parent: None,
                });
            }
        }
        for image in images {
            check.charge(1)?;
            if seen.insert(image.inequality.clone()) {
                premises.push(AutomaticAffinePremise {
                    inequality: image.inequality.clone(),
                    source: None,
                    parent: Some(image.parent),
                });
            }
        }
        Ok(premises)
    }

    fn affine_consequence_from_residual(
        selected: &[(usize, i128)],
        automatic: &[AutomaticAffinePremise],
        mut parents: Vec<DerivationId>,
    ) -> AffineConsequenceProof {
        let mut premises = Vec::new();
        for &(index, factor) in selected {
            let premise = &automatic[index];
            if let Some(source) = premise.source {
                premises.push(AffinePremiseUse { source, factor });
            }
            if let Some(parent) = premise.parent {
                parents.push(parent);
            }
        }
        parents.sort_unstable_by_key(|parent| parent.0);
        parents.dedup();
        AffineConsequenceProof { premises, parents }
    }

    /// Queries the strongest closed L0 image with exactly this affine vector.
    fn affine_l0_proof(
        &mut self,
        inequality: &AffineInequality,
        index: &AffineL0Index,
        closed: &ClosedState,
    ) -> Result<Option<Vec<DerivationId>>, AffineCheckError> {
        let Some(entry) = index.entry(inequality.terms()) else {
            return Ok(None);
        };
        if entry.inequality.upper() > inequality.upper() {
            return Ok(None);
        }
        let parent = closed
            .bound_proof(entry.left, entry.right, entry.bound, &mut self.derivations)
            .ok_or(AffineCheckError::CoefficientMismatch)?;
        Ok(Some(vec![parent]))
    }

    fn affine_interval_proof(
        &mut self,
        inequality: &AffineInequality,
        values: &AffineFlowState,
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> Result<Option<Vec<DerivationId>>, AffineCheckError> {
        let mut requested = inequality
            .terms()
            .iter()
            .map(|coefficient| coefficient.term())
            .collect::<Vec<_>>();
        requested.sort_unstable();
        requested.dedup();

        let mut term_intervals = HashMap::new();
        for atom_id in requested {
            let atom = *self
                .affine_atoms
                .get(atom_id.index() as usize)
                .ok_or(AffineCheckError::CoefficientMismatch)?;
            let (minimum, maximum) = (atom.minimum, atom.maximum);
            let mut bindings = values
                .values
                .iter()
                .filter_map(|(binding, value)| {
                    (value.unit_term() == Some(atom_id)).then_some(*binding)
                })
                .collect::<Vec<_>>();
            bindings.sort_by_key(|binding| binding.0);
            let terms = bindings
                .into_iter()
                .filter_map(|binding| {
                    if self.affine_binding_type(binding) != Some(atom.ty) {
                        return None;
                    }
                    Some(self.terms.intern(TermKind::Place(
                        PlaceTerm {
                            root: PlaceRoot::Binding(binding),
                            deref: false,
                            fields: Vec::new(),
                        },
                        atom.ty,
                    )))
                })
                .collect::<Vec<_>>();
            term_intervals.insert(atom_id, (minimum, maximum, terms));
        }

        if closed.contradictory() {
            return Ok(closed.contradiction_proof().map(|proof| vec![proof]));
        }
        let intervals = term_intervals
            .into_iter()
            .map(|(atom, (mut minimum, mut maximum, terms))| {
                let mut minimum_parent = None;
                let mut maximum_parent = None;
                for term in terms {
                    if let Some(upper) = closed.tight_bound(term, ZERO)
                        && upper < maximum
                    {
                        maximum = upper;
                        maximum_parent = Some((term, ZERO, upper));
                    }
                    if let Some(negative_lower) = closed.tight_bound(ZERO, term)
                        && let Some(lower) = negative_lower.checked_neg()
                        && lower > minimum
                    {
                        minimum = lower;
                        minimum_parent = Some((ZERO, term, negative_lower));
                    }
                }
                (atom, (minimum, maximum, minimum_parent, maximum_parent))
            })
            .collect::<HashMap<_, _>>();
        let proved = interval_proves(
            inequality,
            |term| {
                intervals
                    .get(&term)
                    .map(|(minimum, maximum, _, _)| (*minimum, *maximum))
            },
            check,
        )?;
        if !proved {
            return Ok(None);
        }
        let mut parents = Vec::new();
        for coefficient in inequality.terms() {
            let (_, _, minimum_parent, maximum_parent) = intervals
                .get(&coefficient.term())
                .ok_or(AffineCheckError::CoefficientMismatch)?;
            let selected = if coefficient.coefficient() > 0 {
                maximum_parent
            } else {
                minimum_parent
            };
            if let Some((left, right, bound)) = *selected {
                let parent = closed
                    .bound_proof(left, right, bound, &mut self.derivations)
                    .ok_or(AffineCheckError::CoefficientMismatch)?;
                parents.push(parent);
            }
        }
        parents.sort_unstable_by_key(|parent| parent.0);
        parents.dedup();
        Ok(Some(parents))
    }

    fn affine_residual_proof(
        &mut self,
        inequality: &AffineInequality,
        l0: &AffineL0Index,
        values: &AffineFlowState,
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> Result<Option<Vec<DerivationId>>, AffineCheckError> {
        if closed.contradictory() {
            return Ok(closed.contradiction_proof().map(|proof| vec![proof]));
        }
        if let Some(parents) = self.affine_l0_proof(inequality, l0, closed)? {
            return Ok(Some(parents));
        }
        self.affine_interval_proof(inequality, values, closed, check)
    }

    /// Exhausts one coefficient-one L0 premise followed by the direct
    /// L0/interval residual rule. The L0 index contains one strongest entry
    /// per coefficient vector, so strengthening ordinary facts can only make
    /// a residual easier and never removes an earlier witness.
    fn affine_two_l0_proof(
        &mut self,
        target: &AffineInequality,
        l0: &AffineL0Index,
        values: &AffineFlowState,
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> Option<Vec<DerivationId>> {
        for entry in &l0.entries {
            let Ok(residual) = AffineInequality::residual_after(target, &entry.inequality, check)
            else {
                continue;
            };
            let Ok(Some(mut parents)) =
                self.affine_residual_proof(&residual, l0, values, closed, check)
            else {
                continue;
            };
            let Some(parent) =
                closed.bound_proof(entry.left, entry.right, entry.bound, &mut self.derivations)
            else {
                continue;
            };
            parents.push(parent);
            parents.sort_unstable_by_key(|parent| parent.0);
            parents.dedup();
            return Some(parents);
        }
        None
    }

    fn affine_target_proof(
        &mut self,
        target: &AffineInequality,
        assumptions: &[ActiveAffineFact],
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Option<AffineConsequenceProof> {
        let mut check = AffineCheckState::new();
        let candidates = self.affine_l0_candidates(values);
        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        let l0 = self.affine_l0_index(&candidates, &closed, &mut check);
        if let Ok(Some(parents)) =
            self.affine_residual_proof(target, &l0, values, &closed, &mut check)
        {
            return Some(AffineConsequenceProof {
                premises: Vec::new(),
                parents,
            });
        }
        let automatic = self
            .automatic_affine_premises(assumptions, &values.division_images, &mut check)
            .ok()?;

        // Preserve the complete coefficient-one single-premise route. Every
        // premise is tried independently; an arithmetic error in one candidate
        // cannot suppress a later source or value-image fact.
        for (index, assumption) in automatic.iter().enumerate() {
            let Ok(residual) =
                AffineInequality::residual_after(target, &assumption.inequality, &mut check)
            else {
                // This fixed candidate cannot participate in an i128
                // coefficient-one residual. It grants no authority, but it
                // must not hide a later independently representable source
                // fact in the same deterministic candidate order.
                continue;
            };
            let Ok(proof) = self.affine_residual_proof(&residual, &l0, values, &closed, &mut check)
            else {
                continue;
            };
            if let Some(parents) = proof {
                return Some(Self::affine_consequence_from_residual(
                    &[(index, 1)],
                    &automatic,
                    parents,
                ));
            }
        }

        // R2 exhausts the source-shaped set of unordered coefficient-one
        // pairs, including one premise used twice. There is no greedy state,
        // backtracking cutoff, or cumulative work budget: fact order changes
        // only which successful derivation is retained, never acceptance.
        if let Some((first, second, parents)) =
            first_two_premise_residual(target, &automatic, &mut check, |residual, check| {
                self.affine_residual_proof(residual, &l0, values, &closed, check)
                    .ok()
                    .flatten()
            })
        {
            let selected = if first == second {
                vec![(first, 2)]
            } else {
                vec![(first, 1), (second, 1)]
            };
            return Some(Self::affine_consequence_from_residual(
                &selected, &automatic, parents,
            ));
        }

        // Ordinary L0 relations remain outside the affine premise set. This
        // final goal-directed route combines at most two indexed L0 images
        // without making them compete with an explicit source-affine fact.
        self.affine_two_l0_proof(target, &l0, values, &closed, &mut check)
            .map(|parents| AffineConsequenceProof {
                premises: Vec::new(),
                parents,
            })
    }

    fn affine_event_kills_binding(binding: BindingId, event: &KillEvent) -> bool {
        match event {
            KillEvent::Write { place, .. } | KillEvent::EntryImageHolderWrite { place, .. } => {
                ResolvedPlace::binding(binding).overlaps(place)
            }
            KillEvent::Consume {
                binding: consumed, ..
            }
            | KillEvent::EntryImageHolderConsume {
                binding: consumed, ..
            } => binding == *consumed,
        }
    }

    fn apply_affine_kills(state: &mut AffineFlowState, events: &[KillEvent]) {
        state.values.retain(|binding, _| {
            !events
                .iter()
                .any(|event| Self::affine_event_kills_binding(*binding, event))
        });
        // `division_images` name AffineTermId value identities, not mutable
        // bindings. Removing the map above prevents a replacement value from
        // matching an old image; retaining the image preserves valid aliases.
    }

    fn expression_effects(
        &mut self,
        expression: &CheckedExpression,
        state: &mut ProofFlowState,
    ) -> Option<PreparedCall> {
        let mut prepared = self.judge_expression(expression, state);
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        if let Some(prepared) = &mut prepared {
            if !events.is_empty() {
                self.promote_flow_contradiction(state);
            }
            for event in &events {
                let kind = match event {
                    KillEvent::Consume { .. } | KillEvent::EntryImageHolderConsume { .. } => {
                        FlowEventKind::PostconditionCallConsume
                    }
                    KillEvent::Write { .. } | KillEvent::EntryImageHolderWrite { .. } => {
                        FlowEventKind::PostconditionCallWrite
                    }
                };
                let proof_event = self.proof_event(kind, Some(event.source()));
                self.apply_kills_one(&mut state.facts, std::slice::from_ref(event));
                Self::apply_affine_kills(&mut state.affine, std::slice::from_ref(event));
                self.invalidate_entry_images(state, std::slice::from_ref(event), Some(proof_event));
                prepared.transfer_events.push(proof_event);
            }
            prepared.kills = events;
        } else {
            self.apply_kills(state, &events);
        }
        prepared
    }

    fn walk_set(
        &mut self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        force_target_event: bool,
        state: &mut ProofFlowState,
    ) -> SetWalkOutcome {
        // [SET-1]: the target's base and offset are evaluated before the
        // right-hand side; both are judged at this point, then the commit
        // kill applies.
        self.judge_set_target(target, state);
        let affine_value = self.affine_expression_form(value, &mut state.affine);
        let prepared = self.expression_effects(value, state);
        let receiver_route = prepared
            .as_ref()
            .and_then(|prepared| self.direct_receiver_route(target, value, prepared));
        invalidate_goal_origin_for_set(&mut state.facts, target);
        let mut target_kills = Vec::new();
        match target {
            CheckedSetTarget::Place(place) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(place.binding),
                    deref: self.is_holder(place.binding),
                    fields: place.fields.clone(),
                };
                target_kills.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: false,
                    source: node_path.clone(),
                });
                if place.fields.is_empty() {
                    state.facts.origins.remove(&place.binding);
                    state.facts.outcomes.remove(&place.binding);
                }
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                target_kills.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                    source: node_path.clone(),
                });
            }
            CheckedSetTarget::BufferIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                target_kills.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                    source: node_path.clone(),
                });
            }
        }
        let receivers =
            if let (Some(prepared), Some(receiver_route)) = (prepared.as_ref(), receiver_route) {
                self.prepare_direct_receiver(receiver_route, value, prepared, &target_kills)
            } else {
                Vec::new()
            };
        let target_event = (force_target_event || !receivers.is_empty())
            .then(|| self.proof_event(FlowEventKind::PostconditionReceiverWrite, Some(node_path)));
        if let Some(target_event) = target_event {
            if !target_kills.is_empty() {
                self.promote_flow_contradiction(state);
            }
            for event in &target_kills {
                self.apply_kills_one(&mut state.facts, std::slice::from_ref(event));
                Self::apply_affine_kills(&mut state.affine, std::slice::from_ref(event));
                self.invalidate_entry_images(
                    state,
                    std::slice::from_ref(event),
                    Some(target_event),
                );
            }
        } else {
            self.apply_kills(state, &target_kills);
        }
        // [ENT-3.S5, ENT-5]: the committed value exists only after the old
        // target facts have died. An unsupported RHS shape contributes no fact.
        let mut set_image_event = None;
        self.establish_set_copy_fact(
            node_path,
            target,
            value,
            &mut state.facts,
            &mut set_image_event,
        );
        if let CheckedSetTarget::Place(place) = target
            && place.fields.is_empty()
            && self.affine_binding_type(place.binding).is_some()
            && let Some(value) = affine_value
        {
            state.affine.values.insert(place.binding, value);
        }
        if let (Some(prepared), Some(target_event)) = (&prepared, target_event) {
            for receiver in &receivers {
                self.establish_direct_receiver(node_path, receiver, prepared, target_event, state);
            }
        }
        SetWalkOutcome { target_event }
    }

    fn walk_statement(&mut self, statement: &CheckedStatement, state: &mut ProofFlowState) -> bool {
        match statement {
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                let affine_value = self.affine_expression_form(value, &mut state.affine);
                let prepared = self.expression_effects(value, state);
                self.declare(*binding);
                if self.affine_binding_type(*binding).is_some()
                    && let Some(value) = affine_value
                {
                    state.affine.values.insert(*binding, value);
                }
                if let Some(prepared) = &prepared {
                    self.establish_direct_result(node_path, *binding, value, prepared, state);
                }
                if value.ty() == CheckedType::Bool
                    && let Some(relation) = self.direct_comparison(value)
                {
                    state.facts.origins.insert(*binding, relation);
                }
                self.record_goal_origin(*binding, value, &mut state.facts);
                // Sources S5, S6, S7, and S9 establish at the binding, after
                // the initializer's own kills [ENT-3, ENT-5].
                let mut event = None;
                let unsigned_division = self.establish_binding_facts(
                    node_path,
                    *binding,
                    value,
                    &mut state.facts,
                    &mut event,
                );
                if let Some(established) = unsigned_division {
                    self.establish_unsigned_division_image(
                        *binding,
                        value,
                        established,
                        &mut state.affine,
                    );
                }
                true
            }
            CheckedStatement::PropagateLet {
                binding,
                scrutinee,
                ok_type,
                ..
            } => {
                // The Err edge leaves the function; the normal continuation
                // keeps the preceding state subject to the initializer
                // call's own kill events, and the binder gains no fact
                // [ENT-5].
                let _ = self.expression_effects(scrutinee, state);
                self.declare(*binding);
                if self.affine_binding_type(*binding).is_some()
                    && let Some(value) = self.affine_unknown_integer(*ok_type)
                {
                    state.affine.values.insert(*binding, value);
                }
                true
            }
            CheckedStatement::Set {
                node_path,
                target,
                value,
            } => {
                let _ = self.walk_set(node_path, target, value, false, state);
                true
            }
            CheckedStatement::Replace {
                node_path,
                binding,
                target,
                value,
            } => {
                // [SET-2, ENT-5]: the commit's kill events are exactly a Set
                // commit's on the same resolved target — a whole-place
                // replace kills the covered length facts and an
                // element-position replace spares them — and the commit
                // establishes nothing. The fresh old-value binding is
                // declared and carries no fact.
                let previous = match target {
                    CheckedSetTarget::Place(place)
                        if place.fields.is_empty()
                            && self.affine_binding_type(place.binding).is_some() =>
                    {
                        state.affine.values.get(&place.binding).cloned()
                    }
                    _ => None,
                };
                let _ = self.walk_set(node_path, target, value, false, state);
                self.declare(*binding);
                if self.affine_binding_type(*binding).is_some()
                    && let Some(previous) = previous
                {
                    state.affine.values.insert(*binding, previous);
                }
                true
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                let _ = self.expression_effects(value, state);
                true
            }
            CheckedStatement::Proof(proof) => {
                let source_ordinal = u32::try_from(self.source_proofs.len())
                    .expect("local invariant count exceeds the u32 identity space");
                let l0_premises = proof
                    .uses
                    .iter()
                    .map(|written_use| self.checked_affine_relation_l0(&written_use.relation))
                    .collect::<Vec<_>>();
                let target = self.checked_affine_relation_inequality(
                    &proof.target,
                    &mut state.affine,
                    &mut AffineCheckState::new(),
                );
                let premises = proof
                    .uses
                    .iter()
                    .map(|written_use| {
                        self.checked_affine_relation_inequality(
                            &written_use.relation,
                            &mut state.affine,
                            &mut AffineCheckState::new(),
                        )
                    })
                    .collect::<Vec<_>>();
                let certificate_premises = proof
                    .uses
                    .iter()
                    .zip(&premises)
                    .map(|(written_use, premise)| {
                        premise.clone().map(|premise| (premise, written_use.factor))
                    })
                    .collect::<Option<Vec<_>>>();

                // AUTO is exactly the unified zero-, one-, and exhaustive
                // unordered two-premise route for this specification version.
                // A written block is redundant when that route already proves
                // its target from this same entering context. A blockless local
                // invariant uses AUTO itself as its complete check.
                let auto_proved = target.as_ref().is_some_and(|target| {
                    self.prove(
                        ProofContext::new(&state.facts, &state.affine),
                        ProofGoal::Affine { inequality: target },
                    )
                    .disposition
                        == ProofDisposition::Proved
                });
                let redundant = !proof.uses.is_empty() && auto_proved;
                let combination = if proof.uses.len() > MAX_CERTIFICATE_PREMISES {
                    Err(SourceProofCertificateFailure::UseCapacity {
                        maximum: u32::try_from(MAX_CERTIFICATE_PREMISES)
                            .expect("certificate capacity fits u32"),
                        actual: u32::try_from(proof.uses.len()).unwrap_or(u32::MAX),
                    })
                } else if proof.uses.is_empty() {
                    Ok(auto_proved)
                } else {
                    match (target.as_ref(), certificate_premises.as_deref()) {
                        (Some(target), Some(premises)) => self.source_proof_combination(
                            target,
                            premises,
                            &state.affine,
                            &state.facts,
                        ),
                        _ => Ok(false),
                    }
                };

                // Every written `use` is proved against the same pre-proof
                // program point. No premise established by this statement can
                // help another premise in the same statement.
                let check = self.source_proof_check(
                    &premises,
                    &l0_premises,
                    combination,
                    redundant,
                    &state.affine,
                    &state.facts,
                );

                if let Some(target) = target {
                    let fact = ActiveAffineFact {
                        source: SourceAffineFactRef::SourceProof { source_ordinal },
                        inequality: target.clone(),
                    };
                    if check.discharged() {
                        state.affine.proofs.push(fact);
                    }
                }
                self.source_proofs.push(SourceProofOutcome {
                    node_path: proof.node_path.clone(),
                    source_ordinal,
                    name: proof.name.clone(),
                    check,
                });
                true
            }
            CheckedStatement::Return {
                node_path, value, ..
            } => {
                let affine_result = self.affine_pure_expression_form(value, &mut state.affine);
                let _ = self.expression_effects(value, state);
                self.judge_postcondition_return(node_path, state, affine_result.as_ref());
                false
            }
            CheckedStatement::Give {
                node_path, value, ..
            } => {
                let _ = self.expression_effects(value, state);
                if let Some((scope_depth, loop_depth, kind, binding, result_type)) =
                    self.gives.last().map(|frame| {
                        (
                            frame.scope_depth,
                            frame.loop_depth,
                            frame.kind,
                            frame.binding,
                            frame.result_type,
                        )
                    })
                {
                    let give_goal_origin = if result_type == CheckedType::Bool {
                        self.direct_goal_expression(value)
                            .map(|origin| self.intern_goal_expression(origin))
                    } else {
                        None
                    };
                    let delivery = (kind == ValueInitializerKind::ValueIf).then(|| {
                        self.value_if_delivery_image(
                            value,
                            state,
                            DeliveryImageContext {
                                statement: node_path,
                                receiver_binding: binding,
                                receiver_type: result_type,
                                scope_depth,
                                loop_depth,
                            },
                        )
                    });
                    let mut exit = state.clone();
                    self.exit_scopes_to(&mut exit, scope_depth);
                    self.exit_counted_loops_from(&mut exit, loop_depth);
                    if let Some(frame) = self.gives.last_mut() {
                        frame.gives.push(exit);
                        frame.give_goal_origins.push(give_goal_origin);
                        if let Some(delivery) = delivery {
                            frame.delivery_images.push(delivery);
                            frame.delivery_edges.push(node_path.clone());
                        }
                    }
                }
                false
            }
            CheckedStatement::Break { target, .. } => {
                if let Some(position) = self.loops.iter().rposition(|frame| frame.id == *target) {
                    let depth = self.loops[position].scope_depth;
                    let mut exit = state.clone();
                    self.exit_scopes_to(&mut exit, depth);
                    self.exit_counted_loops_from(&mut exit, position);
                    self.loops[position].breaks.push(exit);
                }
                false
            }
            CheckedStatement::Match {
                scrutinee,
                enum_type,
                arms,
                ..
            } => {
                let prepared = self.expression_effects(scrutinee, state);
                let facts = self.arm_facts(scrutinee, *enum_type, &state.facts);
                let mut exits = Vec::new();
                for arm in arms {
                    let direct_call = prepared
                        .as_ref()
                        .map(|prepared| (scrutinee, *enum_type, prepared));
                    if let Some(exit) = self.walk_arm(arm, state, &facts, direct_call) {
                        exits.push(exit);
                    }
                }
                if exits.is_empty() {
                    false
                } else {
                    *state = self.join_flows(&exits);
                    true
                }
            }
            CheckedStatement::ValueMatchLet {
                node_path,
                kind,
                binding,
                result_type,
                scrutinee,
                enum_type,
                arms,
                ..
            } => {
                let prepared = self.expression_effects(scrutinee, state);
                let facts = self.arm_facts(scrutinee, *enum_type, &state.facts);
                self.gives.push(GiveFrame {
                    scope_depth: self.scopes.len(),
                    loop_depth: self.loops.len(),
                    kind: *kind,
                    node_path: node_path.clone(),
                    binding: *binding,
                    result_type: *result_type,
                    gives: Vec::new(),
                    give_goal_origins: Vec::new(),
                    delivery_images: Vec::new(),
                    delivery_edges: Vec::new(),
                });
                for arm in arms {
                    // Every delivering path leaves by `give`; an arm's
                    // fall-through state contributes nothing [GIVE-1].
                    let direct_call = prepared
                        .as_ref()
                        .map(|prepared| (scrutinee, *enum_type, prepared));
                    let _ = self.walk_arm(arm, state, &facts, direct_call);
                }
                let frame = self
                    .gives
                    .pop()
                    .expect("checked value initializer has one active give frame");
                self.declare(*binding);
                if frame.gives.is_empty() {
                    return false;
                }
                *state = self.join_flows(&frame.gives);
                if self.affine_binding_type(*binding).is_some()
                    && let Some(value) = self.affine_unknown_integer(*result_type)
                {
                    state.affine.values.insert(*binding, value);
                }
                self.record_value_initializer_origin(&frame, &mut state.facts);
                if frame.kind == ValueInitializerKind::ValueIf {
                    self.establish_value_if_delivery_join(&frame, state);
                }
                true
            }
            CheckedStatement::Loop {
                id,
                invariants,
                body,
                ..
            } => {
                let base = self.prove_loop_invariant_bases(invariants, state);
                let base_batch = base.iter().all(|proved| *proved);

                // The generic header starts from the preheader minus every
                // fact a continuing kill may invalidate. Invariants then add
                // precisely the author-written induction hypotheses whose
                // complete base batch succeeded.
                let mut kills = LoopKills::default();
                self.collect_continuing_loop_kills(
                    body,
                    true,
                    &mut LoopReachability::default(),
                    &mut kills,
                );
                self.apply_loop_kills(state, &kills, None);
                self.activate_loop_invariant_batch(*id, invariants, base_batch, &mut state.affine);
                let head_entry_images = state.entry_images.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: self.scopes.len(),
                    counted_binder: None,
                    capture_path: None,
                    breaks: Vec::new(),
                });
                let mut body_state = state.clone();
                let body_falls_through = self.walk_block(body, &mut body_state);

                let mut step = vec![None; invariants.len()];
                if body_falls_through {
                    for (index, invariant) in invariants.iter().enumerate() {
                        let target = self.checked_loop_invariant_inequality(
                            invariant,
                            &mut body_state.affine,
                            &mut AffineCheckState::new(),
                        );
                        step[index] = Some(target.as_ref().is_some_and(|target| {
                            self.prove(
                                ProofContext::new(&body_state.facts, &body_state.affine),
                                ProofGoal::Affine { inequality: target },
                            )
                            .disposition
                                == ProofDisposition::Proved
                        }));
                    }
                }
                self.record_loop_invariant_outcomes(*id, invariants, &base, &step);

                let frame = self.loops.pop();
                let mut breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                for break_state in &mut breaks {
                    Self::remove_active_loop_invariants(&mut break_state.affine, *id);
                }
                let has_breaks = !breaks.is_empty();
                // The continuation is the join over the break edges; with no
                // break it is the contradictory all-derivable state, matching
                // an unreachable-in-truth continuation the conservative graph
                // keeps reachable [ENT-5].
                *state = self.join_flows(&breaks);
                if !has_breaks {
                    state.entry_images = head_entry_images;
                }
                true
            }
            CheckedStatement::CountedRange {
                id,
                node_path,
                binder,
                lower,
                upper,
                invariants,
                body,
                ..
            } => {
                let occurrence = self.encountered_counted;
                self.encountered_counted = self
                    .encountered_counted
                    .checked_add(1)
                    .expect("counted statements exceed the u32 identity space");
                // [FN-1, ENT-3 S11]: evaluate each endpoint exactly once,
                // left to right, then install the private captures and the
                // compiler-updated binder in a construct-owned fact scope.
                let lower_affine = self.affine_expression_form(lower, &mut state.affine);
                let _ = self.expression_effects(lower, state);
                // Capture the upper endpoint after lower-endpoint effects even
                // when no current proof consumes its affine image.  This
                // preserves FN-1 evaluation order and therefore deterministic
                // atom identities for every later program-point value.
                let upper_affine = self.affine_expression_form(upper, &mut state.affine);
                let _ = self.expression_effects(upper, state);
                let outer_scope_depth = self.scopes.len();
                self.scopes.push(vec![*binder]);
                let range_path = node_path.components().to_vec();
                let preheader_event = self.proof_event(FlowEventKind::S11, Some(node_path));
                let counted_terms = self.establish_counted_preheader(
                    &range_path,
                    *binder,
                    lower,
                    upper,
                    &mut state.facts,
                    preheader_event,
                );
                // S11 fixes the complete post-capture closure before
                // continuing kills are subtracted. This preserves sound
                // snapshot consequences without rereading a mutable endpoint
                // on later iterations.
                let snapshot = self.derivations.event(FlowEventKind::Snapshot, None);
                state.facts = materialize_closure_at(
                    &state.facts,
                    &self.terms,
                    &self.goals,
                    &mut self.derivations,
                    snapshot,
                );
                let counted = self.capture_counted_preheader(counted_terms, &state.facts);
                let binder_affine = lower_affine
                    .clone()
                    .or_else(|| self.new_affine_binding_atom(*binder))
                    .expect("a checked counted binder has one u64 affine value");
                state.affine.values.insert(*binder, binder_affine);

                let lower_le_upper = lower_affine.as_ref().and_then(|lower| {
                    upper_affine.as_ref().and_then(|upper| {
                        AffineInequality::from_forms(lower, upper, &mut AffineCheckState::new())
                            .ok()
                    })
                });
                let lower_le_upper = lower_le_upper.as_ref().is_some_and(|target| {
                    self.prove(
                        ProofContext::new(&state.facts, &state.affine),
                        ProofGoal::Affine { inequality: target },
                    )
                    .disposition
                        == ProofDisposition::Proved
                });

                let base = self.prove_loop_invariant_bases(invariants, state);
                let base_batch = base.iter().all(|proved| *proved);

                let mut kills = LoopKills::default();
                let body_reaches_head = self.collect_continuing_loop_kills(
                    body,
                    true,
                    &mut LoopReachability::default(),
                    &mut kills,
                );
                if body_reaches_head {
                    // The hidden update is a continuing write exactly when
                    // normal body fallthrough can reach it.
                    kills.push_event_group(vec![KillEvent::Write {
                        place: ResolvedPlace {
                            root: PlaceRoot::Binding(*binder),
                            fields: Vec::new(),
                        },
                        element: false,
                        source: node_path.clone(),
                    }]);
                    kills.set_bindings.insert(*binder);
                }
                self.apply_loop_kills(state, &kills, Some(snapshot));

                // Values killed by a possible continuing iteration now read
                // as fresh header atoms.  The written expressions themselves
                // are unchanged; only their program-point value images differ
                // from the base targets above.
                let header_binder = self
                    .new_affine_binding_atom(*binder)
                    .expect("a checked counted binder has one u64 affine value");
                state.affine.values.insert(*binder, header_binder);
                self.activate_loop_invariant_batch(*id, invariants, base_batch, &mut state.affine);

                let head = state.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: outer_scope_depth,
                    counted_binder: Some(*binder),
                    capture_path: Some(range_path.clone()),
                    breaks: Vec::new(),
                });
                let mut body_state = head.clone();
                let body_event = self.proof_event(FlowEventKind::S11, Some(node_path));
                let counted = self.establish_counted_body_entry(
                    node_path,
                    counted,
                    &mut body_state.facts,
                    body_event,
                );
                self.retain_counted_derivations(occurrence, counted);
                let body_falls_through = self.walk_block(body, &mut body_state);

                let mut step = vec![None; invariants.len()];
                let mut hidden_update = !body_falls_through;
                if body_falls_through {
                    let current_binder = body_state
                        .affine
                        .values
                        .get(binder)
                        .cloned()
                        .expect("a counted body retains its header binder affine value");
                    let next_binder = current_binder
                        .add(&AffineForm::constant(1), &mut AffineCheckState::new())
                        .ok();
                    let hidden_target = next_binder.as_ref().and_then(|next| {
                        AffineInequality::from_forms(
                            next,
                            &AffineForm::constant(u64::MAX as i128),
                            &mut AffineCheckState::new(),
                        )
                        .ok()
                    });
                    hidden_update = hidden_target.as_ref().is_some_and(|target| {
                        self.prove(
                            ProofContext::new(&body_state.facts, &body_state.affine),
                            ProofGoal::Affine { inequality: target },
                        )
                        .disposition
                            == ProofDisposition::Proved
                    });
                    // Normalize the next-header target with `binder :=
                    // binder_head + 1`, but retain the old header binding in
                    // the proof state.  The true-header S11 relation constrains
                    // `binder_head`; replacing the live binding first would
                    // make that exact old value unreachable while proving the
                    // backedge target.
                    let mut next_affine = body_state.affine.clone();
                    if let Some(next_binder) = next_binder {
                        next_affine.values.insert(*binder, next_binder);
                    }

                    for (index, invariant) in invariants.iter().enumerate() {
                        let next_target = self.checked_loop_invariant_inequality(
                            invariant,
                            &mut next_affine,
                            &mut AffineCheckState::new(),
                        );
                        step[index] = Some(
                            hidden_update
                                && next_target.as_ref().is_some_and(|target| {
                                    self.prove(
                                        ProofContext::new(&body_state.facts, &body_state.affine),
                                        ProofGoal::Affine { inequality: target },
                                    )
                                    .disposition
                                        == ProofDisposition::Proved
                                }),
                        );
                    }
                }

                self.record_loop_invariant_outcomes(*id, invariants, &base, &step);
                let step_batch = step.iter().all(|proved| proved.unwrap_or(true));
                let export = lower_le_upper && base_batch && step_batch && hidden_update;
                let frame = self.loops.pop();
                let mut breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                for break_state in &mut breaks {
                    Self::remove_active_loop_invariants(&mut break_state.affine, *id);
                }

                // Unlike an ordinary loop, the real false-header edge always
                // contributes. Binder and captures leave scope before it or
                // a matching break reaches the continuation.
                let mut exhaustion = head;
                if let Some(upper_affine) = &upper_affine
                    && export
                {
                    let mut normalized = exhaustion.affine.clone();
                    normalized.values.insert(*binder, upper_affine.clone());
                    for (source_ordinal, invariant) in invariants.iter().enumerate() {
                        let Some(inequality) = self.checked_loop_invariant_inequality(
                            invariant,
                            &mut normalized,
                            &mut AffineCheckState::new(),
                        ) else {
                            continue;
                        };
                        if !Self::affine_fact_uses_only_outer_values(
                            &inequality,
                            &normalized,
                            *binder,
                        ) {
                            continue;
                        }
                        let fact = ActiveAffineFact {
                            source: SourceAffineFactRef::LoopInvariant(SourceLoopInvariantRef {
                                loop_id: *id,
                                source_ordinal: u32::try_from(source_ordinal)
                                    .expect("loop invariant ordinal exceeds u32"),
                            }),
                            inequality,
                        };
                        exhaustion.affine.exports.push(fact);
                    }
                }
                Self::remove_active_loop_invariants(&mut exhaustion.affine, *id);
                self.exit_scopes_to(&mut exhaustion, outer_scope_depth);
                self.exit_counted_capture_scope(&mut exhaustion, &range_path);
                let mut exits = Vec::with_capacity(1 + breaks.len());
                exits.push(exhaustion);
                exits.extend(breaks);
                self.scopes.pop();
                *state = self.join_flows(&exits);
                true
            }
            CheckedStatement::Region { body, .. } => self.walk_block(body, state),
        }
    }

    /// Walks one match arm from `entry`; establishes the arm-entry facts the
    /// scrutinee admits, applies the arm's scope-exit kills on fall-through,
    /// and returns the arm-exit state when the arm reaches the continuation.
    fn walk_arm(
        &mut self,
        arm: &CheckedMatchArm,
        entry: &ProofFlowState,
        facts: &ArmFacts,
        direct_call: Option<(&CheckedExpression, CheckedEnumType, &PreparedCall)>,
    ) -> Option<ProofFlowState> {
        let mut state = entry.clone();
        let s1_event = (!facts.goals.is_empty() || facts.comparison.is_some())
            .then(|| self.proof_event(FlowEventKind::S1, facts.node_path.as_ref()));
        let outcome_event = facts
            .outcome
            .as_ref()
            .map(|(_, outcome)| outcome.event_kind)
            .and_then(|kind| {
                arm.binders
                    .iter()
                    .find(|binder| binder.field == 0)
                    .map(|binder| self.proof_event(kind, Some(&binder.node_path)))
            });
        self.establish_arm_entry(arm, facts, &mut state.facts, s1_event, outcome_event);
        let direct_matches =
            direct_call.map_or_else(Vec::new, |(scrutinee, enum_type, prepared)| {
                self.establish_direct_match(scrutinee, enum_type, arm, prepared, &mut state)
            });
        self.scopes
            .push(arm.binders.iter().map(|b| b.binding).collect());
        for binder in &arm.binders {
            if let CheckedType::Integer(ty) = binder.ty
                && matches!(binder.mode, CheckedMode::Own)
            {
                let value = self.new_affine_atom(ty);
                state.affine.values.insert(binder.binding, value);
            }
        }
        let mut continues = true;
        let mut first = 0usize;
        if let (Some((scrutinee, _, _)), Some(statement)) = (direct_call, arm.body.first()) {
            let candidates = direct_matches
                .iter()
                .filter_map(|direct_match| {
                    self.prepare_selected_receiver(arm, statement, scrutinee, direct_match)
                })
                .collect::<Vec<_>>();
            if !candidates.is_empty() {
                let CheckedStatement::Set {
                    node_path,
                    target,
                    value,
                } = statement
                else {
                    unreachable!("selected receiver preparation admits only a set statement");
                };
                let outcome = self.walk_set(node_path, target, value, true, &mut state);
                let target_event = outcome
                    .target_event
                    .expect("an admitted selected receiver retains its target event");
                for candidate in &candidates {
                    self.establish_selected_receiver(
                        node_path,
                        candidate,
                        target_event,
                        &mut state,
                    );
                }
                continues = true;
                first = 1;
            }
        }
        for statement in &arm.body[first..] {
            if !continues {
                break;
            }
            continues = self.walk_statement(statement, &mut state);
        }
        if continues {
            let depth = self.scopes.len() - 1;
            self.exit_scopes_to(&mut state, depth);
        }
        self.scopes.pop();
        continues.then_some(state)
    }

    fn establish_arm_entry(
        &mut self,
        arm: &CheckedMatchArm,
        facts: &ArmFacts,
        state: &mut FactState,
        event: Option<FlowEventId>,
        outcome_event: Option<FlowEventId>,
    ) {
        if let Some(relation) = &facts.comparison {
            // Bool arms: tag 1 is `True()`, tag 0 is `False()`; the False
            // arm takes the exact negation [ENT-3].
            if arm.tag == 1 {
                state.establish(
                    relation,
                    &mut self.derivations,
                    event.expect("comparison arm has an S1 proof event"),
                );
            } else if arm.tag == 0 {
                state.establish(
                    &relation.negated(),
                    &mut self.derivations,
                    event.expect("comparison arm has an S1 proof event"),
                );
            }
        }
        for goal in &facts.goals {
            if arm.tag == 1 {
                state.establish_goal(
                    *goal,
                    GoalSign::Positive,
                    &mut self.derivations,
                    event.expect("goal arm has an S1 proof event"),
                );
                // [ENT-3] Signed Boolean decomposition of the established goal.
                self.establish_boolean_decomposition(
                    *goal,
                    GoalSign::Positive,
                    state,
                    event.expect("goal arm has an S1 proof event"),
                );
                self.record_boolean_decomposition(*goal, GoalSign::Positive, state);
            } else if arm.tag == 0 {
                state.establish_goal(
                    *goal,
                    GoalSign::Negative,
                    &mut self.derivations,
                    event.expect("goal arm has an S1 proof event"),
                );
                // [ENT-3] Signed Boolean decomposition of the established goal.
                self.establish_boolean_decomposition(
                    *goal,
                    GoalSign::Negative,
                    state,
                    event.expect("goal arm has an S1 proof event"),
                );
                self.record_boolean_decomposition(*goal, GoalSign::Negative, state);
            }
        }
        if let Some((tag, outcome)) = &facts.outcome
            && arm.tag == *tag
        {
            self.establish_binder_fact(
                arm,
                outcome,
                state,
                outcome_event.expect("outcome arm has a shared proof event"),
            );
        }
    }

    // ------------------------------------------------------------------
    // Loop kill summary
    // ------------------------------------------------------------------

    /// Returns whether a block entry can reach the loop head whose summary is
    /// being built. `normal_reaches` describes the containing block's normal
    /// exit. This is structural reachability over [FN-1], not an executable
    /// constant-folding judgment.
    fn loop_block_reaches(
        &self,
        statements: &[CheckedStatement],
        normal_reaches: bool,
        reachability: &mut LoopReachability,
    ) -> bool {
        let mut reaches = normal_reaches;
        for statement in statements.iter().rev() {
            reaches = self.loop_statement_reaches(statement, reaches, reachability);
        }
        reaches
    }

    fn loop_statement_reaches(
        &self,
        statement: &CheckedStatement,
        normal_reaches: bool,
        reachability: &mut LoopReachability,
    ) -> bool {
        match statement {
            CheckedStatement::Let { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Proof(_) => normal_reaches,
            CheckedStatement::Return { .. } => false,
            CheckedStatement::Give { .. } => reachability.gives.last().copied().unwrap_or(false),
            CheckedStatement::Break { target, .. } => reachability.break_reaches(*target),
            CheckedStatement::Match { arms, .. } => {
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.loop_block_reaches(&arm.body, normal_reaches, reachability);
                }
                reaches
            }
            CheckedStatement::ValueMatchLet { arms, .. } => {
                // Arm fallthrough never reaches a value initializer's
                // continuation. Its `give` edges do, and nested value
                // initializers shadow this target while they are inspected.
                reachability.gives.push(normal_reaches);
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.loop_block_reaches(&arm.body, false, reachability);
                }
                reachability.gives.pop();
                reaches
            }
            CheckedStatement::Loop { id, body, .. } => {
                // A nested loop body reaches its successor through its own
                // break edges, or can escape through another visible target.
                // A backedge alone cannot create reachability, so evaluating
                // the body with a false normal exit computes the least fixed
                // point. Once the body entry reaches the target, its normal
                // exit can take another iteration and eventually use that
                // same route.
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches = self.loop_block_reaches(body, false, reachability);
                reachability.breaks.pop();
                // [FN-1] also keeps a conservative direct edge from the
                // nested loop statement to its normal successor. That edge
                // carries no event from inside the body.
                normal_reaches || body_reaches
            }
            CheckedStatement::CountedRange { id, body, .. } => {
                // The false-header edge reaches the normal successor, while
                // body fallthrough updates and returns to a header that may
                // then take that same edge. A matching break also reaches the
                // successor; enclosing exits retain their visible targets.
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches = self.loop_block_reaches(body, normal_reaches, reachability);
                reachability.breaks.pop();
                normal_reaches || body_reaches
            }
            CheckedStatement::Region { body, .. } => {
                self.loop_block_reaches(body, normal_reaches, reachability)
            }
        }
    }

    /// Collects exactly the kill events whose carrying edge can reach this
    /// loop's next head. The return value is the same structural entry
    /// reachability computed by [`Self::loop_block_reaches`].
    fn collect_continuing_loop_kills(
        &self,
        statements: &[CheckedStatement],
        normal_reaches: bool,
        reachability: &mut LoopReachability,
        kills: &mut LoopKills,
    ) -> bool {
        let mut reaches = normal_reaches;
        for statement in statements.iter().rev() {
            reaches =
                self.collect_continuing_statement_kills(statement, reaches, reachability, kills);
        }
        reaches
    }

    fn collect_continuing_statement_kills(
        &self,
        statement: &CheckedStatement,
        normal_reaches: bool,
        reachability: &mut LoopReachability,
        kills: &mut LoopKills,
    ) -> bool {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::PropagateLet {
                scrutinee: value, ..
            } => {
                if normal_reaches {
                    self.collect_loop_expression_kills(value, kills);
                }
                normal_reaches
            }
            CheckedStatement::Set {
                node_path,
                target,
                value,
            }
            | CheckedStatement::Replace {
                node_path,
                target,
                value,
                ..
            } => {
                if normal_reaches {
                    self.collect_set_kills(node_path, target, value, kills);
                }
                normal_reaches
            }
            CheckedStatement::Return { .. } => false,
            CheckedStatement::Give { value, .. } => {
                let reaches = reachability.gives.last().copied().unwrap_or(false);
                if reaches {
                    self.collect_loop_expression_kills(value, kills);
                }
                reaches
            }
            CheckedStatement::Break { target, .. } => reachability.break_reaches(*target),
            CheckedStatement::Proof(_) => normal_reaches,
            CheckedStatement::Match {
                scrutinee, arms, ..
            } => {
                let mut reaches = false;
                for arm in arms {
                    reaches |= self.collect_continuing_loop_kills(
                        &arm.body,
                        normal_reaches,
                        reachability,
                        kills,
                    );
                }
                if reaches {
                    self.collect_loop_expression_kills(scrutinee, kills);
                }
                reaches
            }
            CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                reachability.gives.push(normal_reaches);
                let mut reaches = false;
                for arm in arms {
                    reaches |=
                        self.collect_continuing_loop_kills(&arm.body, false, reachability, kills);
                }
                reachability.gives.pop();
                if reaches {
                    self.collect_loop_expression_kills(scrutinee, kills);
                }
                reaches
            }
            CheckedStatement::Loop { id, body, .. } => {
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches = self.loop_block_reaches(body, false, reachability);
                self.collect_continuing_loop_kills(body, body_reaches, reachability, kills);
                reachability.breaks.pop();
                normal_reaches || body_reaches
            }
            CheckedStatement::CountedRange {
                id,
                lower,
                upper,
                body,
                ..
            } => {
                reachability.breaks.push((*id, normal_reaches));
                let body_reaches =
                    self.collect_continuing_loop_kills(body, normal_reaches, reachability, kills);
                reachability.breaks.pop();
                // Both endpoint atoms execute before either the real false
                // edge or a body path. Their own effects are continuing for
                // the enclosing target exactly when this statement can reach
                // that target through one of those successors.
                let reaches = normal_reaches || body_reaches;
                if reaches {
                    let mut events = Vec::new();
                    self.collect_expression_kills(lower, &mut events);
                    self.collect_expression_kills(upper, &mut events);
                    kills.push_event_group(events);
                }
                reaches
            }
            CheckedStatement::Region { body, .. } => {
                self.collect_continuing_loop_kills(body, normal_reaches, reachability, kills)
            }
        }
    }

    fn collect_loop_expression_kills(&self, expression: &CheckedExpression, kills: &mut LoopKills) {
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        kills.push_event_group(events);
    }

    fn collect_set_kills(
        &self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        kills: &mut LoopKills,
    ) {
        let mut events = Vec::new();
        self.collect_expression_kills(value, &mut events);
        kills.set_bindings.insert(target.binding());
        match target {
            CheckedSetTarget::Place(place) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(place.binding),
                    deref: self.is_holder(place.binding),
                    fields: place.fields.clone(),
                };
                events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: false,
                    source: node_path.clone(),
                });
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                    source: node_path.clone(),
                });
            }
            CheckedSetTarget::BufferIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                    source: node_path.clone(),
                });
            }
        }
        kills.push_event_group(events);
    }

    fn apply_loop_kills_one(&mut self, state: &mut FactState, kills: &LoopKills) {
        self.project_event_killed_bounds(state, &kills.events);
        state.kill(|term| {
            kills
                .events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
        for event in &kills.events {
            self.kill_s12_candidates_for_event(state, event);
        }
        state.kill_goals(|goal| {
            kills
                .events
                .iter()
                .any(|event| self.event_kills_goal(goal, event))
        });
        state.goal_origins.retain(|binding, _| {
            !kills
                .events
                .iter()
                .any(|event| self.event_kills_goal_origin_binding(*binding, event))
        });
        state
            .origins
            .retain(|binding, _| !kills.set_bindings.contains(binding));
        state
            .outcomes
            .retain(|binding, _| !kills.set_bindings.contains(binding));
        state
            .goal_origins
            .retain(|binding, _| !kills.set_bindings.contains(binding));
        state
            .ambiguous_goal_origins
            .retain(|binding| !kills.set_bindings.contains(binding));
    }

    fn apply_loop_kills(
        &mut self,
        states: &mut ProofFlowState,
        kills: &LoopKills,
        event: Option<FlowEventId>,
    ) {
        self.promote_flow_contradiction(states);
        self.apply_loop_kills_one(&mut states.facts, kills);
        Self::apply_affine_kills(&mut states.affine, &kills.events);
        let mut groups = kills.entry_image_groups.iter().collect::<Vec<_>>();
        groups.sort_by(|left, right| left.owner.components().cmp(right.owner.components()));
        for group in groups {
            self.invalidate_entry_images(states, &kills.events[group.range.clone()], event);
        }
    }

    // ------------------------------------------------------------------
    // Canonical rendering [ENT-6]
    // ------------------------------------------------------------------

    fn binding_name(&self, binding: BindingId) -> String {
        self.context
            .binding_names
            .get(binding.0 as usize)
            .cloned()
            .unwrap_or_else(|| "?".to_owned())
    }

    fn render_place(&self, place: &PlaceTerm) -> String {
        let (mut rendered, mut ty) = match place.root {
            PlaceRoot::Binding(binding) => {
                let base = if place.deref {
                    format!("deref({})", self.binding_name(binding))
                } else {
                    self.binding_name(binding)
                };
                (base, self.summary(binding).and_then(|summary| summary.ty))
            }
            PlaceRoot::Constant(id) => (
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.name.clone())
                    .unwrap_or_else(|| "?".to_owned()),
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.ty),
            ),
        };
        for field in &place.fields {
            let name = ty
                .and_then(|current| self.field_name(current, *field))
                .unwrap_or(None);
            match name {
                Some((field_name, field_ty)) => {
                    rendered.push('.');
                    rendered.push_str(&field_name);
                    ty = Some(field_ty);
                }
                None => {
                    rendered.push_str(".?");
                    ty = None;
                }
            }
        }
        rendered
    }

    fn render_projected_place(&self, place: &ProjectedPlaceTerm) -> String {
        let (mut rendered, mut ty) = match place.root {
            PlaceRoot::Binding(binding) => (
                self.binding_name(binding),
                self.summary(binding).and_then(|summary| summary.ty),
            ),
            PlaceRoot::Constant(id) => (
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.name.clone())
                    .unwrap_or_else(|| "?".to_owned()),
                self.context
                    .constants
                    .get(id.0 as usize)
                    .map(|constant| constant.ty),
            ),
        };
        for projection in &place.projections {
            match projection {
                PlaceProjection::Field(field) => {
                    let name = ty
                        .and_then(|current| self.field_name(current, *field))
                        .unwrap_or(None);
                    match name {
                        Some((field_name, field_ty)) => {
                            rendered.push('.');
                            rendered.push_str(&field_name);
                            ty = Some(field_ty);
                        }
                        None => {
                            rendered.push_str(".?");
                            ty = None;
                        }
                    }
                }
                PlaceProjection::Deref => {
                    rendered = format!("deref({rendered})");
                    ty = ty.and_then(|current| self.deref_type(current));
                }
            }
        }
        rendered
    }

    fn deref_type(&self, ty: CheckedType) -> Option<CheckedType> {
        let CheckedType::Nominal(id) = ty else {
            // Borrow bindings retain the referent type in checked form.
            return Some(ty);
        };
        let nominal = self.context.nominals.get(id.0 as usize)?;
        match nominal.kind {
            CheckedNominalKind::Box { referent } => Some(referent),
            _ => Some(ty),
        }
    }

    #[allow(clippy::type_complexity)]
    fn field_name(&self, ty: CheckedType, field: u32) -> Option<Option<(String, CheckedType)>> {
        let CheckedType::Nominal(id) = ty else {
            return Some(None);
        };
        let nominal = self.context.nominals.get(id.0 as usize)?;
        let CheckedNominalKind::Struct { fields } = &nominal.kind else {
            return Some(None);
        };
        let field = fields.get(field as usize)?;
        Some(Some((field.name.clone(), field.ty)))
    }

    /// Renders one normalized relation for diagnostics.
    fn render_relation(&self, relation: &Relation) -> String {
        match relation {
            Relation::Bound { left, right, bound } => format!(
                "{} - {} <= {bound}",
                self.render_term(*left),
                self.render_term(*right)
            ),
            Relation::Equal { left, right } => {
                format!("{} = {}", self.render_term(*left), self.render_term(*right))
            }
            Relation::Distinct { left, right } => {
                format!(
                    "{} != {}",
                    self.render_term(*left),
                    self.render_term(*right)
                )
            }
        }
    }

    fn render_term(&self, term: TermId) -> String {
        match self.terms.kind(term) {
            TermKind::Zero => "0".to_owned(),
            TermKind::Constant(value) => value.to_string(),
            TermKind::ConstParameter(_) => "<const parameter>".to_owned(),
            TermKind::Place(place, _) => self.render_place(place),
            TermKind::ProjectedPlace(place, _) => self.render_projected_place(place),
            TermKind::Length(place) => format!("len({})", self.render_place(place)),
            TermKind::ProjectedLength(place) => {
                format!("len({})", self.render_projected_place(place))
            }
            TermKind::CountedCapture { side, .. } => match side {
                CountedCaptureSide::Lower => "<counted lower capture>".to_owned(),
                CountedCaptureSide::Upper => "<counted upper capture>".to_owned(),
            },
        }
    }

    /// One concrete [FN-8] call goal in the terms the source wrote it in.
    ///
    /// The structural dump this replaced published `Integer { operation:
    /// LessEqual, .. }(Place { root: BindingId(6), .. })`: a writer cannot find
    /// either half in their own program, and the blind-writer trial of
    /// 2026-08-28 recorded four rounds of readers failing to. [OP-4] and
    /// [SYS-8] already publish their residual as source terms from the
    /// renderers below, so FN-8 publishes its goal from the same ones. The
    /// operation spellings come from the compiler's own exhaustive maps, which
    /// `semantic::tests::operation_table` locks against the specification.
    pub(super) fn render_concrete_goal(&self, expression: &GoalExpression) -> String {
        match expression {
            GoalExpression::Datum(datum) => self.render_goal_datum(datum),
            GoalExpression::Operation { row, arguments, .. } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.render_concrete_goal(argument))
                    .collect::<Vec<_>>();
                render_goal_row(row, &arguments)
            }
        }
    }

    fn render_goal_datum(&self, datum: &GoalDatum) -> String {
        match datum {
            // A concrete goal has no formal left in it, but a template
            // rendered through this path names the position the formal holds.
            GoalDatum::Parameter {
                ordinal,
                projections,
                ..
            } => self.render_goal_projections(format!("parameter #{ordinal}"), None, projections),
            GoalDatum::NamedConst {
                declaration,
                projections,
                ..
            } => {
                let (name, ty) = self.context.constant(*declaration).map_or_else(
                    || ("?".to_owned(), None),
                    |constant| (constant.name.clone(), Some(constant.ty)),
                );
                self.render_goal_projections(name, ty, projections)
            }
            GoalDatum::Place {
                root, projections, ..
            } => {
                let base = self.binding_name(*root);
                let ty = self.summary(*root).and_then(|summary| summary.ty);
                // A holder's own name selects the holder; the source spells
                // the referent `deref(h)`, and the goal carries that deref as
                // a projection only where the source wrote one.
                let implicit = self.is_holder(*root)
                    && !matches!(projections.first(), Some(GoalProjection::Deref));
                let base = if implicit {
                    format!("deref({base})")
                } else {
                    base
                };
                self.render_goal_projections(base, ty, projections)
            }
            // Source cannot name this datum: it is the already-evaluated
            // pre-transfer value of one direct-subscript actual, so the
            // rendering says which argument it is instead of inventing a
            // spelling. FN-8's stronger repair for exactly this shape is to
            // bind it first, which then gives it a name.
            GoalDatum::EphemeralActual { argument, .. } => {
                format!("<argument #{argument} pre-transfer value>")
            }
            GoalDatum::Literal(value) => match value {
                CheckedValue::Integer { ty, bits } => {
                    format!("{}_{}", integer_value(*ty, *bits), integer_type_name(*ty))
                }
                other => format!("{other:?}"),
            },
        }
    }

    fn render_goal_projections(
        &self,
        base: String,
        root_type: Option<CheckedType>,
        projections: &[GoalProjection],
    ) -> String {
        let mut rendered = base;
        let mut ty = root_type;
        for projection in projections {
            match projection {
                GoalProjection::Deref => {
                    rendered = format!("deref({rendered})");
                    ty = ty.and_then(|current| self.deref_type(current));
                }
                GoalProjection::Field(field) => {
                    match ty
                        .and_then(|current| self.field_name(current, *field))
                        .unwrap_or(None)
                    {
                        Some((name, field_type)) => {
                            rendered.push('.');
                            rendered.push_str(&name);
                            ty = Some(field_type);
                        }
                        None => {
                            rendered.push_str(".?");
                            ty = None;
                        }
                    }
                }
            }
        }
        rendered
    }

    fn render_expression(&self, expression: &CheckedExpression) -> String {
        match expression {
            CheckedExpression::Constant(CheckedValue::Integer { ty, bits }) => {
                format!("{}_{}", integer_value(*ty, *bits), integer_type_name(*ty))
            }
            CheckedExpression::NamedConstant {
                value: CheckedValue::Integer { ty, bits },
                ..
            } => format!("{}_{}", integer_value(*ty, *bits), integer_type_name(*ty)),
            CheckedExpression::Binding { binding, .. } => self.binding_name(*binding),
            CheckedExpression::Project {
                binding, fields, ..
            } => self.render_place(&PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: false,
                fields: fields.clone(),
            }),
            CheckedExpression::DerefAddressed { binding, .. } => {
                format!("deref({})", self.binding_name(*binding))
            }
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. } => {
                format!("deref({})", self.render_expression(value))
            }
            CheckedExpression::ProjectValue {
                value,
                nominal,
                field,
                ..
            } => {
                let field_name = self
                    .context
                    .nominals
                    .get(nominal.0 as usize)
                    .and_then(|nominal| match &nominal.kind {
                        CheckedNominalKind::Struct { fields } => {
                            fields.get(*field as usize).map(|field| field.name.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| "?".to_owned());
                format!("{}.{field_name}", self.render_expression(value))
            }
            CheckedExpression::ArrayIndex { root, offset, .. } => {
                let base = self.array_root_place(root);
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                format!(
                    "{}[{}]",
                    self.render_place(&base),
                    self.render_expression(offset)
                )
            }
            _ => "?".to_owned(),
        }
    }
}

fn normalize_distinct_requests(requests: &mut [BoundsRequest]) {
    for request in requests {
        if request.distinct
            && let Some(left) = request.left
            && request.right < left
        {
            request.left = Some(request.right);
            request.right = left;
        }
    }
}

fn request_relation(request: &BoundsRequest) -> Option<Relation> {
    let left = request.left?;
    Some(if request.distinct {
        Relation::Distinct {
            left,
            right: request.right,
        }
    } else {
        Relation::Bound {
            left,
            right: request.right,
            bound: request.bound,
        }
    })
}

fn checked_integer_constant(expression: &CheckedExpression) -> Option<i128> {
    match expression {
        CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
        | CheckedExpression::NamedConstant {
            value: CheckedValue::Integer { ty, bits },
            ..
        } => Some(integer_value(*ty, *bits)),
        _ => None,
    }
}

/// A let-origin expansion is valid only while the bound value has no `set`
/// target on the path to its use. The target's projection does not narrow
/// this invalidation: changing one field or element invalidates the aggregate
/// value identity even when a separately established length fact survives.
fn invalidate_goal_origin_for_set(state: &mut FactState, target: &CheckedSetTarget) {
    state.goal_origins.remove(&target.binding());
    state.ambiguous_goal_origins.remove(&target.binding());
}

/// Uses the compact legacy term shape exactly when the complete projection
/// order is zero-or-one leading deref followed only by fields.
fn legacy_place(path: &ProjectedPlaceTerm) -> Option<PlaceTerm> {
    let mut projections = path.projections.iter();
    let deref = matches!(projections.clone().next(), Some(PlaceProjection::Deref));
    if deref {
        projections.next();
    }
    let fields = projections
        .map(|projection| match projection {
            PlaceProjection::Field(field) => Some(*field),
            PlaceProjection::Deref => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(PlaceTerm {
        root: path.root,
        deref,
        fields,
    })
}

/// One goal operation applied to already-rendered operands, in the spelling
/// the source uses for that row.
///
/// An operation whose [OP-1] spelling is a call name renders as a call; the
/// arithmetic rows, whose only spelling is the infix operator [GRAM-6], render
/// as the infix expression a writer would have to write.
fn render_goal_row(row: &GoalOperation, arguments: &[String]) -> String {
    match row {
        GoalOperation::Integer { operation, .. } => {
            render_operation_spelling(operation.spelling(), arguments)
        }
        GoalOperation::Float { operation, .. } => {
            render_operation_spelling(operation.spelling(), arguments)
        }
        GoalOperation::Boolean(operation) => {
            render_operation_spelling(operation.spelling(), arguments)
        }
        GoalOperation::EnumEquality { equal, .. } => {
            render_operation_spelling(if *equal { "ieq" } else { "ine" }, arguments)
        }
        GoalOperation::NumericConversion {
            source,
            destination,
        } => format!(
            "cvt<{}, {}>({})",
            numeric_type_name(*source),
            numeric_type_name(*destination),
            arguments.join(", ")
        ),
        GoalOperation::Reinterpret {
            source,
            destination,
        } => format!(
            "reinterpret<{}, {}>({})",
            numeric_type_name(*source),
            numeric_type_name(*destination),
            arguments.join(", ")
        ),
        GoalOperation::ArrayFill { .. } => render_operation_spelling("array_new", arguments),
        GoalOperation::ArrayLength { .. }
        | GoalOperation::BufferLength { .. }
        | GoalOperation::SliceLength { .. } => render_operation_spelling("len", arguments),
        GoalOperation::BufferFits { .. } => render_operation_spelling("buffer_fits", arguments),
    }
}

/// A call spelling renders `name(a, b)`; an operator spelling renders the
/// binary infix form, which is the only form [GRAM-6] admits for those rows.
fn render_operation_spelling(spelling: &str, arguments: &[String]) -> String {
    let infix = !spelling.starts_with(|first: char| first.is_ascii_alphabetic());
    match (infix, arguments) {
        (true, [left, right]) => format!("{left} {spelling} {right}"),
        _ => format!("{spelling}({})", arguments.join(", ")),
    }
}

const fn numeric_type_name(ty: CheckedNumericType) -> &'static str {
    match ty {
        CheckedNumericType::Integer(integer) => integer_type_name(integer),
        CheckedNumericType::Float(FloatType::F32) => "f32",
        CheckedNumericType::Float(FloatType::F64) => "f64",
    }
}

const fn integer_type_name(ty: IntegerType) -> &'static str {
    match ty {
        IntegerType::I8 => "i8",
        IntegerType::I16 => "i16",
        IntegerType::I32 => "i32",
        IntegerType::I64 => "i64",
        IntegerType::U8 => "u8",
        IntegerType::U16 => "u16",
        IntegerType::U32 => "u32",
        IntegerType::U64 => "u64",
    }
}

#[cfg(test)]
mod goal_origin_kill_tests {
    use super::super::state::{FactState, GoalId};
    use super::invalidate_goal_origin_for_set;
    use crate::semantic::model::{BindingId, CheckedSetTarget, CheckedType, CheckedWritablePlace};

    #[test]
    fn a_projected_set_invalidates_the_aggregate_ordinary_let_origin() {
        let binding = BindingId(0);
        let mut state = FactState::default();
        state.goal_origins.insert(binding, GoalId(0));
        let target = CheckedSetTarget::Place(CheckedWritablePlace {
            binding,
            fields: vec![1],
            ty: CheckedType::Bool,
        });

        invalidate_goal_origin_for_set(&mut state, &target);

        assert!(!state.goal_origins.contains_key(&binding));
    }
}

#[cfg(test)]
mod affine_pair_tests {
    use super::{
        AffineCheckState, AffineInequality, AffineTermId, AutomaticAffinePremise,
        first_two_premise_residual, interval_proves,
    };

    fn inequality(terms: &[(u32, i128)], upper: i128) -> AffineInequality {
        let terms = terms
            .iter()
            .map(|&(term, coefficient)| (AffineTermId::from_index(term), coefficient))
            .collect::<Vec<_>>();
        AffineInequality::from_terms(&terms, upper, &mut AffineCheckState::new())
            .expect("test inequality is representable")
    }

    fn premise(inequality: AffineInequality) -> AutomaticAffinePremise {
        AutomaticAffinePremise {
            inequality,
            source: None,
            parent: None,
        }
    }

    fn interval_closes_without_atoms(
        residual: &AffineInequality,
        check: &mut AffineCheckState,
    ) -> Option<()> {
        interval_proves(residual, |_| None, check)
            .ok()
            .filter(|proved| *proved)
            .map(|_| ())
    }

    #[test]
    fn two_premise_enumeration_includes_one_fact_used_twice() {
        let target = inequality(&[(0, 2)], 0);
        let premises = [premise(inequality(&[(0, 1)], 0))];
        let selected = first_two_premise_residual(
            &target,
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        assert!(matches!(selected, Some((0, 0, ()))));
    }

    #[test]
    fn two_independent_facts_close_while_three_remain_outside_the_pair_rule() {
        let premises = [
            premise(inequality(&[(0, 1)], 0)),
            premise(inequality(&[(1, 1)], 0)),
            premise(inequality(&[(2, 1)], 0)),
        ];
        let two = first_two_premise_residual(
            &inequality(&[(0, 1), (1, 1)], 0),
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        assert!(two.is_some());

        let three = first_two_premise_residual(
            &inequality(&[(0, 1), (1, 1), (2, 1)], 0),
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        assert!(three.is_none());
    }

    #[test]
    fn premise_and_term_order_do_not_change_pair_acceptance() {
        let forward = [
            premise(inequality(&[(0, 1)], 1)),
            premise(inequality(&[(0, 2), (1, -2)], 0)),
            premise(inequality(&[(0, -1), (1, 2)], 0)),
        ];
        let reverse = [
            premise(inequality(&[(0, 2), (1, -1)], 0)),
            premise(inequality(&[(0, -2), (1, 2)], 0)),
            premise(inequality(&[(1, 1)], 1)),
        ];
        let forward_result = first_two_premise_residual(
            &inequality(&[(0, 1)], 0),
            &forward,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        let reverse_result = first_two_premise_residual(
            &inequality(&[(1, 1)], 0),
            &reverse,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        assert!(forward_result.is_some());
        assert!(reverse_result.is_some());
    }

    #[test]
    fn one_unrepresentable_pair_does_not_hide_a_later_pair() {
        let premises = [
            premise(inequality(&[(0, i128::MAX)], 0)),
            premise(inequality(&[(0, 2), (1, -2)], 0)),
            premise(inequality(&[(0, -1), (1, 2)], 0)),
        ];
        let selected = first_two_premise_residual(
            &inequality(&[(0, 1)], 0),
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_without_atoms,
        );
        assert!(matches!(selected, Some((1, 2, ()))));
    }
}
