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

use super::super::goal::{ConcreteGoal, GoalDatum, GoalExpression, GoalOperation, GoalProjection};
use super::super::model::{
    BindingId, CheckedArrayRoot, CheckedConst, CheckedConstructor, CheckedEnumType,
    CheckedExpression, CheckedFloatOperation, CheckedFunction, CheckedLoopId, CheckedMatchArm,
    CheckedMode, CheckedNominal, CheckedNominalKind, CheckedSetTarget, CheckedStatement,
    CheckedType, CheckedValue, IntegerType, ValueInitializerKind,
};
use super::super::postcondition::{
    NormalizedRelation, PostconditionPlaceRoot, PostconditionReturnDatum, PostconditionReturnPlace,
    PostconditionReturnPlaceRoot, RelationDatum, RelationTemplate,
};
use super::state::{
    ClaimLifecycleKind, ClosedState, CountedRootAtom, DerivationId, DerivationInventory,
    DerivationLedger, DerivationNode, DerivationRootKind, FactState, FlowEventId, FlowEventKind,
    GoalId, GoalSign, GoalSupport, GoalTable, JoinParent, OutcomeFact,
    PostconditionCallSubstitution, ProofView, Relation, close, close_excluding_term, join_at,
    materialize_closure_at,
};
use super::term::{
    CountedCaptureSide, LengthBound, PlaceProjection, PlaceRoot, PlaceTerm, ProjectedPlaceTerm,
    TermId, TermKind, TermTable, integer_value,
};
use super::{
    BoundsRequest, CallGoalCounterfactual, CallGoalDisposition, CallGoalEvidence, CallGoalOutcome,
    ClaimDisposition, ClaimOutcome, CountedDerivationSet, EntailmentContext, FunctionEntailment,
    FunctionEntailmentView, FunctionPostconditionProof, ObligationOutcome, PostconditionAggregate,
    PostconditionDisposition, PostconditionEntryImage, PostconditionEntryImageOutcome,
    PostconditionExit, PostconditionViewExit, S7Derivation, VerifiedPostconditionSummary,
    VerifiedPostconditionSummaryRef, ViewObligationOutcome, fragment_type,
};
use crate::{SYSTEM_OPERATIONS, SystemParameterMode};

/// One [OWN-5] resolved place, for the [OWN-7] overlap relation kills use.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedPlace {
    root: PlaceRoot,
    fields: Vec<u32>,
}

impl ResolvedPlace {
    /// [OWN-7]: places overlap when one's path is a prefix of the other's.
    fn overlaps(&self, other: &Self) -> bool {
        self.root == other.root
            && (self.fields.starts_with(&other.fields) || other.fields.starts_with(&self.fields))
    }
}

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

/// What one binding reads through by `deref`, for place resolution.
#[derive(Clone, Debug)]
enum HolderReferent {
    /// A borrow of a known local place.
    Place {
        binding: BindingId,
        fields: Vec<u32>,
    },
    /// A reborrow: reads through another holder.
    Holder(BindingId),
    /// A parameter or match-binder borrow, or an owning box: the referent has
    /// no caller-visible local place, so the binding itself anchors identity.
    Opaque,
}

#[derive(Clone, Debug, Default)]
struct BindingSummary {
    ty: Option<CheckedType>,
    holder: Option<HolderReferent>,
    /// The checked expression for a read through a borrow holder may retain
    /// only the referent value. Reconstructing its [ENT-2] source spelling
    /// must restore that explicit `deref`; an owning box already retains a
    /// `BoxDeref` node and therefore does not use this flag.
    implicit_deref: bool,
    /// Exact [GIVE-1] source class admitted as a bounded-delivery carrier.
    delivery_carrier: bool,
}

/// A `loop` frame collecting break-edge states for the continuation join.
struct LoopFrame {
    id: CheckedLoopId,
    scope_depth: usize,
    /// Present only for a counted range. A break through this frame leaves
    /// the private endpoint-capture scope as well as source binding scopes.
    capture_path: Option<Vec<u32>>,
    breaks: Vec<ViewStates>,
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
    gives: Vec<ViewStates>,
    delivery_images: Vec<ViewStates>,
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

/// Stable receiver identity shared by the three delivery-join views.
struct DeliveryJoinContext<'a> {
    statement: &'a crate::NodePath,
    receiver_binding: BindingId,
    receiver: TermId,
    event: FlowEventId,
}

/// The three FN-9/PRV proof views carried through one structural walk.
#[derive(Clone, Debug)]
struct ViewStates {
    complete: FactState,
    unasserted: FactState,
    s4_blinded: FactState,
    /// First invalidating event for each relation-template entry image. This
    /// state is shared by the three proof views and branches with the same
    /// structural flow; `None` means the image is still live.
    entry_images: Vec<Option<FlowEventId>>,
}

/// Same-view caller premises captured at the pre-transfer call point. The
/// vector is present exactly when every actual obligation and the optional
/// FN-8 goal discharged in this view.
#[derive(Clone, Debug)]
struct PreparedCallView {
    parents: Vec<DerivationId>,
}

/// Transient A0 evidence for an exact root user call. It never enters the
/// checked expression tree, so named, nested, or stored outcomes acquire no
/// pending publication token.
#[derive(Clone, Debug)]
struct PreparedCall {
    function: super::super::model::FunctionId,
    call: crate::NodePath,
    a0_parents: Vec<DerivationId>,
    unasserted: Option<PreparedCallView>,
    s4_blinded: Option<PreparedCallView>,
    transfer_events: Vec<FlowEventId>,
    kills: Vec<KillEvent>,
}

#[derive(Clone, Debug)]
struct AvailablePostcondition {
    relation: RelationTemplate,
    variant: Option<crate::PreludeDeclarationId>,
    field: Option<crate::PreludeDeclarationId>,
    summary: VerifiedPostconditionSummary,
    complete: bool,
    unasserted: bool,
    s4_blinded: bool,
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
    parents: [Option<DerivationId>; 3],
}

#[derive(Clone, Copy)]
struct SelectedReceiverRoute {
    payload: BindingId,
    binding: BindingId,
}

struct SelectedReceiverCandidate {
    route: SelectedReceiverRoute,
    relation: Relation,
    parents: [Option<DerivationId>; 3],
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

impl Default for ViewStates {
    fn default() -> Self {
        Self {
            complete: FactState::for_view(ProofView::Complete),
            unasserted: FactState::for_view(ProofView::Unasserted),
            s4_blinded: FactState::for_view(ProofView::S4Blinded),
            entry_images: Vec::new(),
        }
    }
}

impl ViewStates {
    fn for_each_mut(&mut self, mut visit: impl FnMut(&mut FactState)) {
        visit(&mut self.complete);
        visit(&mut self.unasserted);
        visit(&mut self.s4_blinded);
    }
}

#[derive(Default)]
struct ViewArmFacts {
    complete: ArmFacts,
    unasserted: ArmFacts,
    s4_blinded: ArmFacts,
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
    let mut entailment = analyze_candidate(function, context);
    finish(&mut entailment);
    entailment
}

/// Builds the one optimistic per-function proof batch without pruning or
/// remapping its derivation ledger. The checker uses this for shared FN-9 and
/// CLM-3 units, then calls [`finish`] after the program-level PRV and CLM-3
/// batches have no event.
pub(super) fn analyze_candidate(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
) -> FunctionEntailment {
    let run = run(function, context);
    FunctionEntailment {
        obligations: run.obligations,
        claims: run.claims,
        call_goals: run.call_goals,
        unasserted: run.unasserted,
        s4_blinded: run.s4_blinded,
        program_start: run.program_start,
        strict_roots: Vec::new(),
        counted_derivations: run.counted_derivations,
        s7_derivations: run.s7_derivations,
        postcondition: run.postcondition,
        derivations: run.derivations,
        inventory: run.inventory,
    }
}

struct AnalysisRun {
    obligations: Vec<ObligationOutcome>,
    claims: Vec<ClaimOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    unasserted: FunctionEntailmentView,
    s4_blinded: FunctionEntailmentView,
    program_start: Option<super::ProgramStartGoalOutcome>,
    counted_derivations: Vec<CountedDerivationSet>,
    s7_derivations: Vec<S7Derivation>,
    postcondition: Option<super::FunctionPostconditionProof>,
    derivations: DerivationLedger,
    inventory: DerivationInventory,
}

fn run(function: &CheckedFunction, context: &EntailmentContext<'_>) -> AnalysisRun {
    let mut analyzer = Analyzer {
        context,
        function,
        bindings: Vec::new(),
        terms: TermTable::new(),
        goals: GoalTable::default(),
        derivations: DerivationLedger::default(),
        obligations: Vec::new(),
        claims: Vec::new(),
        call_goals: Vec::new(),
        unasserted_obligations: Vec::new(),
        s4_blinded_obligations: Vec::new(),
        unasserted_call_goals: Vec::new(),
        s4_blinded_call_goals: Vec::new(),
        counted_derivations: Vec::new(),
        s7_derivations: Vec::new(),
        postcondition: None,
        entry_images: Vec::new(),
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
    analyzer.initialize_postcondition_proof();
    let mut state = ViewStates {
        entry_images: vec![None; analyzer.entry_images.len()],
        ..ViewStates::default()
    };
    analyzer
        .scopes
        .push(function.parameters.iter().map(|p| p.binding).collect());
    // [CLM-3, PROG-3] The marked entry query observes the existing U proof
    // state after parameter setup but before the retained wrapper check or S4
    // can authorize the body. It is computed by the same analyzer and DAG.
    let program_start = if context.marked_program_start {
        let requirement = function
            .requirement
            .as_ref()
            .expect("a marked program-start query exists only with a requirement");
        let goal = ConcreteGoal::new(
            analyzer
                .body_requirement_goal()
                .expect("a checked concrete requirement has a body image"),
        );
        let (disposition, evidence, derivation) =
            analyzer.call_goal_disposition(&goal, &state.unasserted);
        Some(super::ProgramStartGoalOutcome {
            final_check: requirement.trap.node_path.clone(),
            goal,
            disposition,
            evidence,
            derivation,
        })
    } else {
        None
    };
    // [ENT-3] S4: the substituted `requires` relation enters the body's entry
    // fact state, the one fact that crosses into the body [ENT-2, FN-8].
    if let Some(requirement) = &function.requirement {
        let event = analyzer.proof_event(FlowEventKind::S4, Some(&requirement.trap.node_path));
        analyzer.establish_requires_facts(&mut state.complete, event);
        analyzer.establish_requires_facts(&mut state.unasserted, event);
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
        obligations: analyzer.obligations,
        claims: analyzer.claims,
        call_goals: analyzer.call_goals,
        unasserted: FunctionEntailmentView {
            obligations: analyzer.unasserted_obligations,
            call_goals: analyzer.unasserted_call_goals,
        },
        s4_blinded: FunctionEntailmentView {
            obligations: analyzer.s4_blinded_obligations,
            call_goals: analyzer.s4_blinded_call_goals,
        },
        program_start,
        counted_derivations: analyzer.counted_derivations,
        s7_derivations: analyzer.s7_derivations,
        postcondition: analyzer.postcondition,
        derivations: analyzer.derivations,
        inventory,
    }
}

/// Finalizes the sole function-local derivation ledger after the optimistic
/// program batch has passed PRV-2/PRV-3 and CLM-3. A rejecting batch drops the
/// candidate function inventory without ever calling this boundary.
pub(super) fn finish(entailment: &mut FunctionEntailment) {
    let event_roots = entailment
        .postcondition
        .iter()
        .flat_map(|proof| &proof.exits)
        .flat_map(|exit| &exit.entry_images)
        .filter_map(|image| image.invalidation)
        .collect::<Vec<_>>();
    let remap = entailment.derivations.finish_with_event_roots(&event_roots);
    for outcome in &mut entailment.obligations {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for outcome in &mut entailment.call_goals {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for outcome in &mut entailment.claims {
        outcome.lifecycle_derivation = outcome
            .lifecycle_derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for outcome in entailment
        .unasserted
        .obligations
        .iter_mut()
        .chain(&mut entailment.s4_blinded.obligations)
    {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    if let Some(outcome) = &mut entailment.program_start {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.nodes.get(id.0 as usize).copied().flatten());
    }
    for root in &mut entailment.strict_roots {
        root.derivation = remap
            .nodes
            .get(root.derivation.0 as usize)
            .copied()
            .flatten()
            .expect("registered strict U root retained by the sole finish boundary");
    }
    for outcome in entailment
        .unasserted
        .call_goals
        .iter_mut()
        .chain(&mut entailment.s4_blinded.call_goals)
    {
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
    if let Some(postcondition) = &mut entailment.postcondition {
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
    let remap_view = |outcome: &mut PostconditionViewExit| match outcome.disposition {
        PostconditionDisposition::Discharged => {
            let old = outcome
                .derivation
                .expect("every discharged postcondition exit has a required root");
            outcome.derivation = Some(
                nodes
                    .get(old.0 as usize)
                    .copied()
                    .flatten()
                    .expect("required postcondition exit root retained by finish"),
            );
        }
        PostconditionDisposition::Refuted | PostconditionDisposition::Unproved => {
            assert!(outcome.derivation.is_none());
        }
    };
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
        remap_view(&mut exit.complete);
        remap_view(&mut exit.unasserted);
        remap_view(&mut exit.s4_blinded);
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
    remap_aggregate(&mut proof.complete);
    remap_aggregate(&mut proof.unasserted);
    remap_aggregate(&mut proof.s4_blinded);
}

struct Analyzer<'check, 'unit> {
    context: &'check EntailmentContext<'unit>,
    function: &'check CheckedFunction,
    /// Dense per-binding summaries indexed by [`BindingId`].
    bindings: Vec<BindingSummary>,
    terms: TermTable,
    goals: GoalTable,
    derivations: DerivationLedger,
    obligations: Vec<ObligationOutcome>,
    claims: Vec<ClaimOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    unasserted_obligations: Vec<ViewObligationOutcome>,
    s4_blinded_obligations: Vec<ViewObligationOutcome>,
    unasserted_call_goals: Vec<CallGoalCounterfactual>,
    s4_blinded_call_goals: Vec<CallGoalCounterfactual>,
    counted_derivations: Vec<CountedDerivationSet>,
    s7_derivations: Vec<S7Derivation>,
    postcondition: Option<super::FunctionPostconditionProof>,
    entry_images: Vec<EntryImageRecord>,
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
    fn initialize_postcondition_proof(&mut self) {
        let Some(postcondition) = &self.function.postcondition else {
            return;
        };
        let aggregate = |view| PostconditionAggregate {
            view,
            discharged: false,
            derivation: None,
        };
        self.postcondition = Some(FunctionPostconditionProof {
            block: postcondition.selector.block.clone(),
            selector: postcondition.selector.selector.clone(),
            summary: None,
            exits: Vec::new(),
            complete: aggregate(ProofView::Complete),
            unasserted: aggregate(ProofView::Unasserted),
            s4_blinded: aggregate(ProofView::S4Blinded),
        });
    }

    fn finalize_postcondition_aggregates(&mut self) {
        let Some(proof) = &self.postcondition else {
            return;
        };
        let block = proof.block.clone();
        let collect = |view: ProofView| {
            proof
                .exits
                .iter()
                .map(|exit| match view {
                    ProofView::Complete => &exit.complete,
                    ProofView::Unasserted => &exit.unasserted,
                    ProofView::S4Blinded => &exit.s4_blinded,
                })
                .map(|outcome| {
                    (outcome.disposition == PostconditionDisposition::Discharged)
                        .then_some(outcome.derivation)
                        .flatten()
                })
                .collect::<Option<Vec<_>>>()
        };
        let complete = collect(ProofView::Complete);
        let unasserted = collect(ProofView::Unasserted);
        let s4_blinded = collect(ProofView::S4Blinded);
        let retain = |this: &mut Self, view, parents: Option<Vec<DerivationId>>| {
            let Some(parents) = parents.filter(|parents| !parents.is_empty()) else {
                return PostconditionAggregate {
                    view,
                    discharged: false,
                    derivation: None,
                };
            };
            let node = this.derivations.intern_for(
                view,
                super::state::DerivationNode::PostconditionAggregate {
                    block: block.clone(),
                    parents,
                },
            );
            this.derivations
                .add_root(DerivationRootKind::PostconditionAggregate { view }, node);
            PostconditionAggregate {
                view,
                discharged: true,
                derivation: Some(node),
            }
        };
        let complete = retain(self, ProofView::Complete, complete);
        let unasserted = retain(self, ProofView::Unasserted, unasserted);
        let s4_blinded = retain(self, ProofView::S4Blinded, s4_blinded);
        let proof = self
            .postcondition
            .as_mut()
            .expect("postcondition proof initialized above");
        proof.complete = complete;
        proof.unasserted = unasserted;
        proof.s4_blinded = s4_blinded;
    }

    fn judge_postcondition_return(&mut self, statement: &crate::NodePath, states: &ViewStates) {
        let Some(postcondition) = &self.function.postcondition else {
            return;
        };
        let Some(selected) = postcondition
            .selected_returns
            .iter()
            .find(|selected| selected.statement == *statement)
            .cloned()
        else {
            return;
        };
        let result = self
            .postcondition_return_term(&selected.value)
            .expect("H1 selected-return datum must remain in the ENT-2 term fragment");
        let relation = self
            .instantiate_postcondition_relation(result)
            .expect("H1 relation template must remain in the ENT-2 term fragment");
        let residual = self.render_relation(&relation);
        let entry_images = self
            .entry_images
            .iter()
            .zip(&states.entry_images)
            .map(|(image, invalidation)| PostconditionEntryImageOutcome {
                datum: image.datum.clone(),
                invalidation: *invalidation,
            })
            .collect::<Vec<_>>();
        let occurrence = self
            .postcondition
            .as_ref()
            .map_or(0, |proof| proof.exits.len());
        let unavailable = entry_images
            .iter()
            .any(|image| image.invalidation.is_some());
        let complete = self.judge_postcondition_view(
            ProofView::Complete,
            occurrence,
            statement,
            &relation,
            &states.complete,
            unavailable,
        );
        let unasserted = self.judge_postcondition_view(
            ProofView::Unasserted,
            occurrence,
            statement,
            &relation,
            &states.unasserted,
            unavailable,
        );
        let s4_blinded = self.judge_postcondition_view(
            ProofView::S4Blinded,
            occurrence,
            statement,
            &relation,
            &states.s4_blinded,
            unavailable,
        );
        self.postcondition
            .as_mut()
            .expect("postcondition proof initialized")
            .exits
            .push(PostconditionExit {
                statement: statement.clone(),
                relation,
                residual,
                entry_images,
                complete,
                unasserted,
                s4_blinded,
            });
    }

    fn judge_postcondition_view(
        &mut self,
        view: ProofView,
        occurrence: usize,
        statement: &crate::NodePath,
        relation: &Relation,
        state: &FactState,
        unavailable: bool,
    ) -> PostconditionViewExit {
        if unavailable {
            return PostconditionViewExit {
                view,
                disposition: PostconditionDisposition::Unproved,
                derivation: None,
            };
        }
        let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
        if closed.derives(relation) {
            let parent = closed
                .relation_proof(relation, &mut self.derivations)
                .expect("a discharged relation must retain its local proof");
            let node = self.derivations.intern_for(
                view,
                super::state::DerivationNode::PostconditionExit {
                    statement: statement.clone(),
                    relation: relation.clone(),
                    parent,
                },
            );
            self.derivations.add_root(
                DerivationRootKind::PostconditionExit {
                    occurrence: u32::try_from(occurrence)
                        .expect("postcondition exits exceed the u32 identity space"),
                    view,
                },
                node,
            );
            PostconditionViewExit {
                view,
                disposition: PostconditionDisposition::Discharged,
                derivation: Some(node),
            }
        } else {
            PostconditionViewExit {
                view,
                disposition: if !closed.contradictory() && closed.derives(&relation.negated()) {
                    PostconditionDisposition::Refuted
                } else {
                    PostconditionDisposition::Unproved
                },
                derivation: None,
            }
        }
    }

    fn instantiate_postcondition_relation(&mut self, result: TermId) -> Option<Relation> {
        let postcondition = self.function.postcondition.as_ref()?;
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
                CheckedConst::Value(value) => LengthBound::Constant(i128::from(value)),
                CheckedConst::Parameter(declaration) => {
                    LengthBound::Equal(self.terms.intern(TermKind::ConstParameter(declaration)))
                }
            };
            self.terms.set_length_bound(term, bound);
        } else if !matches!(ty, CheckedType::Buffer { .. } | CheckedType::Slice { .. }) {
            return None;
        }
        Some(term)
    }

    fn available_postcondition(
        &self,
        function: super::super::model::FunctionId,
    ) -> Option<AvailablePostcondition> {
        let (postcondition, proof) = self.context.verified_postcondition(function)?;
        let summary = proof.summary.clone()?;
        Some(AvailablePostcondition {
            relation: postcondition.relation.clone(),
            variant: postcondition.selector.variant,
            field: postcondition
                .selector
                .field
                .as_ref()
                .map(|field| field.declaration),
            summary,
            complete: proof.complete.discharged,
            unasserted: proof.unasserted.discharged,
            s4_blinded: proof.s4_blinded.discharged,
        })
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
        let mut memo = HashMap::new();
        state.kill_proof_candidates(&self.derivations, |left, right, proof| {
            self.derivations
                .depends_on_postcondition_call(proof, &mut memo)
                && (self.s12_candidate_term_killed(left, event)
                    || self.s12_candidate_term_killed(right, event))
        });
    }

    fn kill_s12_candidates_for_scope(&self, state: &mut FactState, exited: &HashSet<BindingId>) {
        let mut memo = HashMap::new();
        state.kill_proof_candidates(&self.derivations, |left, right, proof| {
            self.derivations
                .depends_on_postcondition_call(proof, &mut memo)
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
        prepared: &PreparedCall,
        view: ProofView,
    ) -> Option<(VerifiedPostconditionSummaryRef, Vec<DerivationId>)> {
        let (summary_view, view_parents) = match view {
            ProofView::Complete if available.complete => (ProofView::Complete, Vec::new()),
            ProofView::Unasserted | ProofView::S4Blinded if available.s4_blinded => {
                (ProofView::S4Blinded, Vec::new())
            }
            ProofView::Unasserted if available.unasserted => (
                ProofView::Unasserted,
                prepared.unasserted.as_ref()?.parents.clone(),
            ),
            ProofView::S4Blinded if available.unasserted => (
                ProofView::Unasserted,
                prepared.s4_blinded.as_ref()?.parents.clone(),
            ),
            _ => return None,
        };
        Some((
            VerifiedPostconditionSummaryRef {
                summary: available.summary.clone(),
                view: summary_view,
            },
            view_parents,
        ))
    }

    fn retain_postcondition_call(
        &mut self,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) -> Option<DerivationId> {
        let view = state.proof_view();
        let (summary, view_parents) = self.selected_call_summary(available, prepared, view)?;
        Some(self.derivations.intern_for(
            view,
            super::state::DerivationNode::PostconditionCall {
                call: prepared.call.clone(),
                relation: instantiated.relation.clone(),
                summary,
                substitutions: instantiated.substitutions.clone(),
                transfer_events: prepared.transfer_events.clone(),
                a0_parents: prepared.a0_parents.clone(),
                view_parents,
            },
        ))
    }

    fn retain_direct_result_view(
        &mut self,
        statement: &crate::NodePath,
        binding: BindingId,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let view = state.proof_view();
        let Some(call) = self.retain_postcondition_call(instantiated, available, prepared, state)
        else {
            return;
        };
        let route = self.derivations.intern_for(
            view,
            super::state::DerivationNode::PostconditionDirectResult {
                statement: statement.clone(),
                binding,
                relation: instantiated.relation.clone(),
                parent: call,
            },
        );
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectResult { occurrence, view },
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
        states: &mut ViewStates,
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
        let Some(available) = self.available_postcondition(*function) else {
            return;
        };
        if available.variant.is_some() {
            return;
        }
        let Some(result_term) =
            self.postcondition_place_term(PlaceRoot::Binding(binding), &[], *result)
        else {
            return;
        };
        let Some(instantiated) = self.instantiate_call_postcondition_relation(
            *function,
            &available.relation,
            arguments,
            goal_arguments,
            result_term,
        ) else {
            return;
        };
        if !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills) {
            return;
        }
        self.retain_direct_result_view(
            statement,
            binding,
            &instantiated,
            &available,
            prepared,
            &mut states.complete,
        );
        self.retain_direct_result_view(
            statement,
            binding,
            &instantiated,
            &available,
            prepared,
            &mut states.unasserted,
        );
        self.retain_direct_result_view(
            statement,
            binding,
            &instantiated,
            &available,
            prepared,
            &mut states.s4_blinded,
        );
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

    fn retain_direct_receiver_view(
        &mut self,
        statement: &crate::NodePath,
        candidate: &DirectReceiverCandidate,
        target_event: FlowEventId,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) {
        let view = state.proof_view();
        let Some(call) = self.retain_postcondition_call(
            &candidate.instantiated,
            &candidate.available,
            prepared,
            state,
        ) else {
            return;
        };
        let proof = self.derivations.intern_for(
            view,
            super::state::DerivationNode::PostconditionDirectReceiver {
                statement: statement.clone(),
                binding: candidate.route.binding,
                receiver_formal: candidate.route.formal,
                relation: candidate.instantiated.relation.clone(),
                target_event,
                parent: call,
            },
        );
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectReceiver { occurrence, view },
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
    ) -> Option<DirectReceiverCandidate> {
        let CheckedExpression::UserCall {
            function,
            arguments,
            goal_arguments,
            ..
        } = value
        else {
            return None;
        };
        let available = self.available_postcondition(*function)?;
        if available.variant.is_some() {
            return None;
        }
        let result_term =
            self.postcondition_place_term(PlaceRoot::Binding(route.binding), &[], route.ty)?;
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
    }

    fn establish_direct_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &DirectReceiverCandidate,
        prepared: &PreparedCall,
        target_event: FlowEventId,
        states: &mut ViewStates,
    ) {
        self.retain_direct_receiver_view(
            statement,
            candidate,
            target_event,
            prepared,
            &mut states.complete,
        );
        self.retain_direct_receiver_view(
            statement,
            candidate,
            target_event,
            prepared,
            &mut states.unasserted,
        );
        self.retain_direct_receiver_view(
            statement,
            candidate,
            target_event,
            prepared,
            &mut states.s4_blinded,
        );
    }

    fn retain_direct_match_view(
        &mut self,
        route: DirectMatchRoute,
        instantiated: &InstantiatedPostcondition,
        available: &AvailablePostcondition,
        prepared: &PreparedCall,
        state: &mut FactState,
    ) -> Option<DerivationId> {
        let view = state.proof_view();
        let call = self.retain_postcondition_call(instantiated, available, prepared, state)?;
        let route = self.derivations.intern_for(
            view,
            super::state::DerivationNode::PostconditionDirectMatch {
                call: prepared.call.clone(),
                variant: route.variant,
                field: route.field,
                tag: route.tag,
                binding: route.binding,
                relation: instantiated.relation.clone(),
                parent: call,
            },
        );
        let occurrence = self.s12_roots;
        self.s12_roots = self
            .s12_roots
            .checked_add(1)
            .expect("S12 roots exceed the u32 identity space");
        self.derivations.add_root(
            DerivationRootKind::PostconditionDirectMatch { occurrence, view },
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
        states: &mut ViewStates,
    ) -> Option<EstablishedDirectMatch> {
        let CheckedExpression::UserCall {
            function,
            call,
            arguments,
            goal_arguments,
            result: CheckedType::Nominal(result_nominal),
            ..
        } = scrutinee
        else {
            return None;
        };
        let CheckedEnumType::Nominal(match_nominal) = enum_type else {
            return None;
        };
        if *function != prepared.function
            || *call != prepared.call
            || *result_nominal != match_nominal
        {
            return None;
        }
        let available = self.available_postcondition(*function)?;
        let (Some(selector_variant), Some(selector_field)) = (available.variant, available.field)
        else {
            return None;
        };
        let nominal = self.context.nominals.get(result_nominal.0 as usize)?;
        let CheckedNominalKind::Enum { variants } = &nominal.kind else {
            return None;
        };
        let variant = variants.iter().find(|variant| {
            variant.tag == arm.tag
                && variant.constructor == CheckedConstructor::Prelude(selector_variant)
        })?;
        let binder = arm.binders.iter().find(|binder| binder.field == 0)?;
        let [selected_field] = variant.fields.as_slice() else {
            return None;
        };
        if binder.mode != CheckedMode::Own
            || binder.ty != selected_field.ty
            || fragment_type(binder.ty).is_none()
        {
            return None;
        }
        let result_term =
            self.postcondition_place_term(PlaceRoot::Binding(binder.binding), &[], binder.ty)?;
        let instantiated = self.instantiate_call_postcondition_relation(
            *function,
            &available.relation,
            arguments,
            goal_arguments,
            result_term,
        )?;
        if !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills) {
            return None;
        }
        let route = DirectMatchRoute {
            variant: selector_variant,
            field: selector_field,
            tag: arm.tag,
            binding: binder.binding,
            ty: binder.ty,
        };
        let complete = self.retain_direct_match_view(
            route,
            &instantiated,
            &available,
            prepared,
            &mut states.complete,
        );
        let unasserted = self.retain_direct_match_view(
            route,
            &instantiated,
            &available,
            prepared,
            &mut states.unasserted,
        );
        let s4_blinded = self.retain_direct_match_view(
            route,
            &instantiated,
            &available,
            prepared,
            &mut states.s4_blinded,
        );
        Some(EstablishedDirectMatch {
            route,
            instantiated,
            parents: [complete, unasserted, s4_blinded],
        })
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
            parents: direct_match.parents,
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
            | CheckedStatement::Check {
                condition: value, ..
            }
            | CheckedStatement::Claim {
                condition: value, ..
            }
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
            CheckedStatement::Break { .. } => false,
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
            | CheckedStatement::Check { .. }
            | CheckedStatement::Claim { .. }
            | CheckedStatement::Loop { .. }
            | CheckedStatement::CountedRange { .. } => true,
        }
    }

    fn retain_selected_receiver_view(
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
        let view = state.proof_view();
        let proof = self.derivations.intern_for(
            view,
            super::state::DerivationNode::PostconditionSelectedReceiver {
                statement: statement.clone(),
                payload: candidate.route.payload,
                binding: candidate.route.binding,
                relation: candidate.relation.clone(),
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
            DerivationRootKind::PostconditionSelectedReceiver { occurrence, view },
            proof,
        );
        state.establish_from_proof(&candidate.relation, proof, &self.derivations);
    }

    fn establish_selected_receiver(
        &mut self,
        statement: &crate::NodePath,
        candidate: &SelectedReceiverCandidate,
        target_event: FlowEventId,
        states: &mut ViewStates,
    ) {
        self.retain_selected_receiver_view(
            statement,
            candidate,
            target_event,
            candidate.parents[0],
            &mut states.complete,
        );
        self.retain_selected_receiver_view(
            statement,
            candidate,
            target_event,
            candidate.parents[1],
            &mut states.unasserted,
        );
        self.retain_selected_receiver_view(
            statement,
            candidate,
            target_event,
            candidate.parents[2],
            &mut states.s4_blinded,
        );
    }

    fn retain_s7_derivation(&mut self, source: S7Derivation) {
        assert_eq!(
            self.derivations.node_views[source.parent.0 as usize], source.view,
            "S7 source metadata and proof node must share one view"
        );
        let occurrence = u32::try_from(self.s7_derivations.len())
            .expect("S7 source roots exceed the u32 identity space");
        let kind = match &source.kind {
            super::S7DerivationKind::BitAndBound { .. } => {
                DerivationRootKind::BitAndBound(occurrence)
            }
            super::S7DerivationKind::ShiftOneNonzero { .. } => {
                DerivationRootKind::ShiftOneNonzero(occurrence)
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

    fn summary_mut(&mut self, binding: BindingId) -> &mut BindingSummary {
        let index = binding.0 as usize;
        if self.bindings.len() <= index {
            self.bindings.resize(index + 1, BindingSummary::default());
        }
        &mut self.bindings[index]
    }

    fn summary(&self, binding: BindingId) -> Option<&BindingSummary> {
        self.bindings.get(binding.0 as usize)
    }

    fn collect_bindings(&mut self) {
        let function = self.function;
        for parameter in &function.parameters {
            let (holder, implicit_deref) = match parameter.mode {
                CheckedMode::Own => (None, false),
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => {
                    (Some(HolderReferent::Opaque), true)
                }
            };
            let summary = self.summary_mut(parameter.binding);
            summary.ty = Some(parameter.ty);
            summary.holder = holder;
            summary.implicit_deref = implicit_deref;
            summary.delivery_carrier = matches!(parameter.mode, CheckedMode::Own);
        }
        self.collect_block_bindings(&function.body);
    }

    fn collect_postcondition_entry_images(&mut self) {
        let Some(postcondition) = &self.function.postcondition else {
            return;
        };
        let mut data = Vec::new();
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
            if let Some(datum) = datum
                && !data.contains(&datum)
            {
                data.push(datum);
            }
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
    }

    fn collect_block_bindings(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let (holder, implicit_deref) = match value {
                        CheckedExpression::Binding {
                            binding: source, ..
                        } if self.is_holder(*source) => {
                            (Some(HolderReferent::Holder(*source)), true)
                        }
                        _ => (holder_from_value(value), value_has_implicit_deref(value)),
                    };
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.holder = holder;
                    summary.implicit_deref = implicit_deref;
                    summary.delivery_carrier = summary.holder.is_none();
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(*ok_type);
                    summary.delivery_carrier = true;
                }
                CheckedStatement::Replace {
                    binding, target, ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(target.ty());
                    summary.delivery_carrier = true;
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(*result_type);
                    summary.delivery_carrier = true;
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                }
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.collect_block_bindings(body);
                }
                CheckedStatement::CountedRange { binder, body, .. } => {
                    let summary = self.summary_mut(*binder);
                    summary.ty = Some(CheckedType::Integer(IntegerType::U64));
                    summary.delivery_carrier = true;
                    self.collect_block_bindings(body);
                }
                _ => {}
            }
        }
    }

    fn collect_arm_bindings(&mut self, arm: &CheckedMatchArm) {
        for binder in &arm.binders {
            let (holder, implicit_deref) = match binder.mode {
                CheckedMode::Own => (None, false),
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => {
                    (Some(HolderReferent::Opaque), true)
                }
            };
            let summary = self.summary_mut(binder.binding);
            summary.ty = Some(binder.ty);
            summary.holder = holder;
            summary.implicit_deref = implicit_deref;
            summary.delivery_carrier = matches!(binder.mode, CheckedMode::Own);
        }
        self.collect_block_bindings(&arm.body);
    }

    fn is_holder(&self, binding: BindingId) -> bool {
        self.summary(binding)
            .is_some_and(|summary| summary.holder.is_some())
    }

    fn needs_implicit_deref(&self, binding: BindingId) -> bool {
        self.summary(binding)
            .is_some_and(|summary| summary.implicit_deref)
    }

    // ------------------------------------------------------------------
    // Place resolution and support
    // ------------------------------------------------------------------

    /// Resolves a spelled place to its [OWN-5] resolved place, reading
    /// through let-bound borrows; opaque holders anchor at themselves.
    fn resolve(&self, place: &PlaceTerm) -> ResolvedPlace {
        match place.root {
            PlaceRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(id),
                fields: place.fields.clone(),
            },
            PlaceRoot::Binding(binding) => {
                let mut resolved = if place.deref {
                    self.resolve_deref(binding, 0)
                } else {
                    ResolvedPlace {
                        root: PlaceRoot::Binding(binding),
                        fields: Vec::new(),
                    }
                };
                resolved.fields.extend_from_slice(&place.fields);
                resolved
            }
        }
    }

    /// Resolves an exact interleaved field/deref spelling for [ENT-5] kills.
    /// A deref of a direct holder follows the existing holder summary. A
    /// deref after a field remains anchored at that selected storage path;
    /// replacing any prefix therefore conservatively kills the fact.
    fn resolve_projected(&self, place: &ProjectedPlaceTerm) -> ResolvedPlace {
        let mut resolved = ResolvedPlace {
            root: place.root,
            fields: Vec::new(),
        };
        for projection in &place.projections {
            match projection {
                PlaceProjection::Field(field) => resolved.fields.push(*field),
                PlaceProjection::Deref => {
                    if resolved.fields.is_empty()
                        && let PlaceRoot::Binding(binding) = resolved.root
                    {
                        resolved = self.resolve_deref(binding, 0);
                    }
                }
            }
        }
        resolved
    }

    fn resolve_deref(&self, holder: BindingId, depth: usize) -> ResolvedPlace {
        let anchored = ResolvedPlace {
            root: PlaceRoot::Binding(holder),
            fields: Vec::new(),
        };
        if depth > 32 {
            return anchored;
        }
        match self
            .summary(holder)
            .and_then(|summary| summary.holder.as_ref())
        {
            Some(HolderReferent::Place { binding, fields }) => {
                let mut resolved = if self.is_holder(*binding) {
                    self.resolve_deref(*binding, depth + 1)
                } else {
                    ResolvedPlace {
                        root: PlaceRoot::Binding(*binding),
                        fields: Vec::new(),
                    }
                };
                resolved.fields.extend_from_slice(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => self.resolve_deref(*next, depth + 1),
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    /// Whether a kill event kills a fact supported by `term` [ENT-5].
    fn event_kills_term(&self, term: TermId, event: &KillEvent) -> bool {
        match self.terms.kind(term).clone() {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            // Counted captures are immutable. Their construct-scope exit is
            // handled separately from source-place write/consume events.
            TermKind::CountedCapture { .. } => false,
            TermKind::Place(place, _) => match event {
                KillEvent::Write { place: written, .. } => self.resolve(&place).overlaps(written),
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. }
                | KillEvent::EntryImageHolderWrite { .. } => false,
            },
            TermKind::ProjectedPlace(place, _) => match event {
                KillEvent::Write { place: written, .. } => {
                    self.resolve_projected(&place).overlaps(written)
                }
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. }
                | KillEvent::EntryImageHolderWrite { .. } => false,
            },
            TermKind::Length(place) => match event {
                // An element write never kills a length fact: the length is
                // fixed at allocation or by the type [ENT-5].
                KillEvent::Write { element: true, .. } => false,
                KillEvent::Write {
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
                KillEvent::EntryImageHolderConsume { .. }
                | KillEvent::EntryImageHolderWrite { .. } => false,
            },
            TermKind::ProjectedLength(place) => match event {
                KillEvent::Write { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                    ..
                } => self.resolve_projected(&place).overlaps(written),
                KillEvent::Consume { binding, .. } => place.root == PlaceRoot::Binding(*binding),
                KillEvent::EntryImageHolderConsume { .. }
                | KillEvent::EntryImageHolderWrite { .. } => false,
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
        holders.push(holder);
        let anchored = ResolvedPlace {
            root: PlaceRoot::Binding(holder),
            fields: Vec::new(),
        };
        if depth > 32 {
            return anchored;
        }
        match self
            .summary(holder)
            .and_then(|summary| summary.holder.as_ref())
        {
            Some(HolderReferent::Place { binding, fields }) => {
                let mut resolved = if self.is_holder(*binding) {
                    self.resolve_deref_with_holders(*binding, depth + 1, holders)
                } else {
                    ResolvedPlace {
                        root: PlaceRoot::Binding(*binding),
                        fields: Vec::new(),
                    }
                };
                resolved.fields.extend_from_slice(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => {
                self.resolve_deref_with_holders(*next, depth + 1, holders)
            }
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    fn event_kills_goal(&self, goal: GoalId, event: &KillEvent) -> bool {
        self.goals.support(goal).iter().any(|support| {
            let (place, holders) = self.resolve_goal_support(support);
            match event {
                KillEvent::Write { element: true, .. } if support.length => false,
                KillEvent::Write { place: written, .. } => place.overlaps(written),
                KillEvent::Consume { binding, .. } => {
                    holders.contains(binding) || place.root == PlaceRoot::Binding(*binding)
                }
                KillEvent::EntryImageHolderConsume { .. }
                | KillEvent::EntryImageHolderWrite { .. } => false,
            }
        })
    }

    /// An ordinary-let origin is available only while the binding whose
    /// initializer it describes has not itself been written or consumed.
    /// This key guard is separate from the goal's value support: invalidating
    /// it stops future alias expansion without erasing a signed snapshot fact
    /// that an earlier branch, check, or claim already established.
    fn event_kills_goal_origin_binding(&self, binding: BindingId, event: &KillEvent) -> bool {
        match event {
            KillEvent::Write { place, .. } => ResolvedPlace {
                root: PlaceRoot::Binding(binding),
                fields: Vec::new(),
            }
            .overlaps(place),
            KillEvent::Consume {
                binding: consumed, ..
            } => binding == *consumed,
            KillEvent::EntryImageHolderConsume { .. } | KillEvent::EntryImageHolderWrite { .. } => {
                false
            }
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
        if !state.all_derivable {
            let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
            if closed.contradictory() {
                state.all_derivable = true;
                state.contradiction = closed.contradiction_proof();
            }
        }
    }

    fn apply_kills_one(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        self.promote_contradiction(state);
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
    }

    fn apply_kills(&mut self, states: &mut ViewStates, events: &[KillEvent]) {
        self.apply_kills_one(&mut states.complete, events);
        self.apply_kills_one(&mut states.unasserted, events);
        self.apply_kills_one(&mut states.s4_blinded, events);
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
        states: &mut ViewStates,
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
        self.promote_contradiction(state);
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
    }

    fn exit_scopes_to(&mut self, states: &mut ViewStates, depth: usize) {
        self.exit_scopes_to_one(&mut states.complete, depth);
        self.exit_scopes_to_one(&mut states.unasserted, depth);
        self.exit_scopes_to_one(&mut states.s4_blinded, depth);
    }

    /// Applies the private capture-scope kill of one counted construct.
    fn exit_counted_capture_scope_one(&mut self, state: &mut FactState, range_path: &[u32]) {
        self.promote_contradiction(state);
        state.kill(|term| {
            matches!(
                self.terms.kind(term),
                TermKind::CountedCapture { range_path: path, .. } if path == range_path
            )
        });
    }

    fn exit_counted_capture_scope(&mut self, states: &mut ViewStates, range_path: &[u32]) {
        self.exit_counted_capture_scope_one(&mut states.complete, range_path);
        self.exit_counted_capture_scope_one(&mut states.unasserted, range_path);
        self.exit_counted_capture_scope_one(&mut states.s4_blinded, range_path);
    }

    /// Applies capture-scope kills for every loop frame crossed by a
    /// non-local edge. Ordinary loop frames carry no private captures.
    fn exit_counted_loops_from(&mut self, states: &mut ViewStates, loop_depth: usize) {
        let paths: Vec<Vec<u32>> = self
            .loops
            .iter()
            .skip(loop_depth)
            .filter_map(|frame| frame.capture_path.clone())
            .collect();
        for path in paths {
            self.exit_counted_capture_scope(states, &path);
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
            CheckedExpression::BoxDeref { value, .. } => {
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
                trap,
                ..
            } if trap.is_none() => build_operation(
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
            | CheckedExpression::IntegerOperation { .. }
            | CheckedExpression::UserCall { .. }
            | CheckedExpression::SystemCall { .. }
            | CheckedExpression::ArrayIndex { .. }
            | CheckedExpression::BufferFill { .. }
            | CheckedExpression::BufferIndex { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceIndex { .. }
            | CheckedExpression::BoxNew { .. }
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
        &self,
        expression: &GoalExpression,
        state: &FactState,
    ) -> GoalExpression {
        self.expand_goal_expression_inner(expression, state, &mut HashSet::new())
    }

    fn expand_goal_expression_inner(
        &self,
        expression: &GoalExpression,
        state: &FactState,
        expanding: &mut HashSet<BindingId>,
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
                let mut expanded = self.expand_goal_expression_inner(&origin, state, expanding);
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
            } => GoalExpression::Operation {
                row: *row,
                type_arguments: type_arguments.clone(),
                const_arguments: const_arguments.clone(),
                result: *result,
                arguments: arguments
                    .iter()
                    .map(|argument| self.expand_goal_expression_inner(argument, state, expanding))
                    .collect(),
            },
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
        let expanded = self.expand_goal_expression(&direct, state);
        let direct = self.intern_goal_expression(direct);
        let expanded = self.intern_goal_expression(expanded);
        if direct == expanded {
            vec![direct]
        } else {
            vec![direct, expanded]
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
    }

    fn intern_goal_expression(&mut self, expression: GoalExpression) -> GoalId {
        let projection = self.goal_projection(&expression);
        let mut support = Vec::new();
        self.collect_goal_support(&expression, false, &mut support);
        self.goals.intern(expression, projection, support)
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
                        CheckedConst::Value(value) => LengthBound::Constant(i128::from(*value)),
                        CheckedConst::Parameter(declaration) => LengthBound::Equal(
                            self.terms.intern(TermKind::ConstParameter(*declaration)),
                        ),
                    };
                    self.terms.set_length_bound(term, bound);
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

    fn body_requirement_goal(&self) -> Option<GoalExpression> {
        let requirement = self.function.requirement.as_ref()?;
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
            | CheckedType::Float(_) => true,
            CheckedType::Nominal(id) => self
                .context
                .nominals
                .get(id.0 as usize)
                .is_some_and(CheckedNominal::is_copy),
            _ => false,
        }
    }

    /// The resolved place a borrow-shaped call argument reads through, and
    /// whether a callee write through it is an element write.
    fn argument_referent(
        &self,
        argument: &CheckedExpression,
    ) -> Option<(ResolvedPlace, bool, bool)> {
        match argument {
            CheckedExpression::BorrowBuffer { root, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                Some((self.resolve(&place), true, false))
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: Vec::new(),
                };
                Some((self.resolve(&place), false, false))
            }
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                Some((self.resolve_deref(*binding, 0), false, false))
            }
            CheckedExpression::Binding { binding, ty, .. } if self.is_holder(*binding) => Some((
                self.resolve_deref(*binding, 0),
                matches!(ty, CheckedType::Buffer { .. } | CheckedType::Slice { .. }),
                true,
            )),
            _ => None,
        }
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
            CheckedExpression::BoxDeref { .. } | CheckedExpression::ProjectValue { .. } => {}
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
                    let written = callee.is_some_and(|callee| {
                        callee.parameter_writes.get(index).copied().unwrap_or(false)
                    });
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
            CheckedExpression::SystemCall {
                operation,
                call,
                arguments,
                ..
            } => {
                let parameters = SYSTEM_OPERATIONS
                    .get(usize::from(*operation))
                    .map(|operation| operation.parameters)
                    .unwrap_or_default();
                for argument in arguments {
                    self.collect_expression_kills(argument, events);
                }
                for (index, argument) in arguments.iter().enumerate() {
                    let written = parameters.get(index).is_some_and(|parameter| {
                        matches!(parameter.mode, SystemParameterMode::UniqueBorrow(_))
                    });
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
        states: &ViewStates,
    ) -> Option<PreparedCall> {
        match expression {
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                requirement,
                ..
            } => {
                let complete_start = self.obligations.len();
                let unasserted_start = self.unasserted_obligations.len();
                let blinded_start = self.s4_blinded_obligations.len();
                for argument in arguments {
                    let _ = self.judge_expression(argument, states);
                }
                let complete_actual_parents = self.obligations[complete_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let unasserted_actual_parents = self.unasserted_obligations[unasserted_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let blinded_actual_parents = self.s4_blinded_obligations[blinded_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let mut complete_goal_parent = None;
                let mut unasserted_goal_parent = None;
                let mut blinded_goal_parent = None;
                let mut complete_goal_ok = requirement.is_none();
                // FN-8 begins only after every actual-expression obligation
                // succeeds. A failed OP-4 actual therefore publishes no call
                // judgment for diagnostic selection to reorder.
                if let Some(requirement) = requirement {
                    if complete_actual_parents.is_some() {
                        let (disposition, derivation) = self.judge_call_goal(
                            *function,
                            call,
                            requirement.final_check.clone(),
                            requirement.goal.clone(),
                            arguments.len(),
                            &states.complete,
                        );
                        complete_goal_ok = disposition == CallGoalDisposition::Discharged;
                        complete_goal_parent = derivation;
                    }
                    let unasserted = self.call_goal_counterfactual(
                        *function,
                        call,
                        requirement.final_check.clone(),
                        requirement.goal.clone(),
                        unasserted_actual_parents.is_some(),
                        &states.unasserted,
                    );
                    if unasserted.actual_obligations_ok
                        && unasserted.goal_disposition == CallGoalDisposition::Discharged
                    {
                        unasserted_goal_parent = unasserted.derivation;
                    }
                    self.unasserted_call_goals.push(unasserted);
                    let blinded = self.call_goal_counterfactual(
                        *function,
                        call,
                        requirement.final_check.clone(),
                        requirement.goal.clone(),
                        blinded_actual_parents.is_some(),
                        &states.s4_blinded,
                    );
                    if blinded.actual_obligations_ok
                        && blinded.goal_disposition == CallGoalDisposition::Discharged
                    {
                        blinded_goal_parent = blinded.derivation;
                    }
                    self.s4_blinded_call_goals.push(blinded);
                }
                let mut a0_parents = complete_actual_parents?;
                if requirement.is_some() {
                    if !complete_goal_ok {
                        return None;
                    }
                    a0_parents.push(complete_goal_parent?);
                }
                let unasserted = unasserted_actual_parents.and_then(|mut parents| {
                    if requirement.is_some() {
                        parents.push(unasserted_goal_parent?);
                    }
                    Some(PreparedCallView { parents })
                });
                let s4_blinded = blinded_actual_parents.and_then(|mut parents| {
                    if requirement.is_some() {
                        parents.push(blinded_goal_parent?);
                    }
                    Some(PreparedCallView { parents })
                });
                // Only an earlier-component verified summary can publish an
                // S12 carrier. Calls without one retain the exact pre-H3 kill
                // path and create no transient postcondition events.
                self.context.verified_postcondition(*function)?;
                Some(PreparedCall {
                    function: *function,
                    call: call.clone(),
                    a0_parents,
                    unasserted,
                    s4_blinded,
                    transfer_events: Vec::new(),
                    kills: Vec::new(),
                })
            }
            CheckedExpression::ArrayIndex {
                root,
                length,
                offset,
                trap,
                ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = self.array_root_place(root);
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, Some(*length), offset, node_path, states);
                None
            }
            CheckedExpression::BufferIndex {
                root, offset, trap, ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, states);
                None
            }
            CheckedExpression::SliceIndex {
                root, offset, trap, ..
            } => {
                let _ = self.judge_expression(offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, states);
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
        final_check: crate::NodePath,
        goal: ConcreteGoal,
        argument_count: usize,
        state: &FactState,
    ) -> (CallGoalDisposition, Option<DerivationId>) {
        let (disposition, evidence, derivation) = self.call_goal_disposition(&goal, state);
        let ordinal = u32::try_from(self.call_goals.len())
            .expect("ENT call-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::CallGoal(ordinal), root);
        }
        self.call_goals.push(CallGoalOutcome {
            node_path: node_path.clone(),
            callee,
            final_check,
            goal,
            argument_count: u32::try_from(argument_count)
                .expect("ENT call argument count exceeds the u32 identity space"),
            disposition,
            evidence,
            derivation,
        });
        (disposition, derivation)
    }

    fn call_goal_counterfactual(
        &mut self,
        callee: super::super::model::FunctionId,
        node_path: &crate::NodePath,
        final_check: crate::NodePath,
        goal: ConcreteGoal,
        actual_obligations_ok: bool,
        state: &FactState,
    ) -> CallGoalCounterfactual {
        let (goal_disposition, goal_evidence, derivation) =
            self.call_goal_disposition(&goal, state);
        CallGoalCounterfactual {
            node_path: node_path.clone(),
            callee,
            final_check,
            goal,
            actual_obligations_ok,
            goal_disposition,
            goal_evidence,
            derivation,
        }
    }

    fn call_goal_disposition(
        &mut self,
        goal: &ConcreteGoal,
        state: &FactState,
    ) -> (
        CallGoalDisposition,
        Vec<CallGoalEvidence>,
        Option<DerivationId>,
    ) {
        let id = self.intern_goal_expression(goal.root.clone());
        let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
        if closed.contradictory() {
            (
                CallGoalDisposition::Discharged,
                vec![CallGoalEvidence::AllDerivable],
                closed.contradiction_proof(),
            )
        } else {
            let positive_opaque = closed.holds_opaque(id, GoalSign::Positive);
            let positive_projection = self
                .goals
                .projection(id)
                .is_some_and(|relation| closed.derives(relation));
            let negative_opaque = closed.holds_opaque(id, GoalSign::Negative);
            let negative_projection = self
                .goals
                .projection(id)
                .is_some_and(|relation| closed.derives(&relation.negated()));
            if positive_opaque || positive_projection {
                let mut evidence = Vec::with_capacity(2);
                if positive_opaque {
                    evidence.push(CallGoalEvidence::OpaquePositive);
                }
                if positive_projection {
                    evidence.push(CallGoalEvidence::ExactL0Projection);
                }
                let derivation = closed.opaque_proof(id, GoalSign::Positive).or_else(|| {
                    closed.goal_projection_proof(
                        id,
                        GoalSign::Positive,
                        &self.goals,
                        &mut self.derivations,
                    )
                });
                (CallGoalDisposition::Discharged, evidence, derivation)
            } else if negative_opaque || negative_projection {
                let mut evidence = Vec::with_capacity(2);
                if negative_opaque {
                    evidence.push(CallGoalEvidence::OpaqueNegative);
                }
                if negative_projection {
                    evidence.push(CallGoalEvidence::NegatedL0Projection);
                }
                (CallGoalDisposition::Refuted, evidence, None)
            } else {
                (CallGoalDisposition::Unproved, Vec::new(), None)
            }
        }
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
                CheckedConst::Value(value) => LengthBound::Constant(i128::from(value)),
                CheckedConst::Parameter(declaration) => {
                    LengthBound::Equal(self.terms.intern(TermKind::ConstParameter(declaration)))
                }
            };
            self.terms.set_length_bound(length_term, bound);
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
        states: &ViewStates,
    ) {
        let length_term = self.length_term(base.clone(), array_length);
        let offset_term = self.read_operand(offset);
        let rendered_residual = format!(
            "{} < len({})",
            self.render_expression(offset),
            self.render_place(&base)
        );
        let closed = close(
            &states.complete,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        let discharged = match offset_term {
            Some(offset_term) => closed.derives_bound(offset_term, length_term, -1),
            // An operand that is not a term or constant leaves the relation
            // underivable, never ill-formed [ENT-6] — unless the state is
            // contradictory, where every obligation is discharged [ENT-4].
            None => closed.contradictory(),
        };
        let residual = if discharged {
            None
        } else {
            Some(rendered_residual.clone())
        };
        let derivation = if discharged {
            match offset_term {
                Some(offset_term) => {
                    closed.bound_proof(offset_term, length_term, -1, &mut self.derivations)
                }
                None => closed.contradiction_proof(),
            }
        } else {
            None
        };
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::BoundsObligation(ordinal), root);
        }
        self.obligations.push(ObligationOutcome {
            node_path: node_path.clone(),
            conjunct: 0,
            requested: BoundsRequest {
                left: offset_term,
                right: length_term,
                bound: -1,
            },
            discharged,
            contradictory: closed.contradictory(),
            residual,
            derivation,
        });
        let unasserted = close(
            &states.unasserted,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        let unasserted_discharged = offset_term.map_or_else(
            || unasserted.contradictory(),
            |offset| unasserted.derives_bound(offset, length_term, -1),
        );
        let unasserted_derivation = if unasserted_discharged {
            match offset_term {
                Some(offset) => {
                    unasserted.bound_proof(offset, length_term, -1, &mut self.derivations)
                }
                None => unasserted.contradiction_proof(),
            }
        } else {
            None
        };
        self.unasserted_obligations.push(ViewObligationOutcome {
            node_path: node_path.clone(),
            discharged: unasserted_discharged,
            residual: (!unasserted_discharged).then(|| rendered_residual.clone()),
            derivation: unasserted_derivation,
        });
        let blinded = close(
            &states.s4_blinded,
            &self.terms,
            &self.goals,
            &mut self.derivations,
        );
        let blinded_discharged = offset_term.map_or_else(
            || blinded.contradictory(),
            |offset| blinded.derives_bound(offset, length_term, -1),
        );
        let blinded_derivation = if blinded_discharged {
            match offset_term {
                Some(offset) => blinded.bound_proof(offset, length_term, -1, &mut self.derivations),
                None => blinded.contradiction_proof(),
            }
        } else {
            None
        };
        self.s4_blinded_obligations.push(ViewObligationOutcome {
            node_path,
            discharged: blinded_discharged,
            residual: (!blinded_discharged).then_some(rendered_residual),
            derivation: blinded_derivation,
        });
    }

    fn judge_set_target(&mut self, target: &CheckedSetTarget, states: &ViewStates) {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                let _ = self.judge_expression(&target.offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, Some(target.length), &target.offset, node_path, states);
            }
            CheckedSetTarget::BufferIndex(target) => {
                let _ = self.judge_expression(&target.offset, states);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, None, &target.offset, node_path, states);
            }
        }
    }

    // ------------------------------------------------------------------
    // Statement walk
    // ------------------------------------------------------------------

    /// Walks one block in its own lexical scope. Returns the fall-through:
    /// `true` when control continues past the block with `state` holding the
    /// post-scope-exit facts.
    fn join_views(&mut self, states: &[ViewStates]) -> ViewStates {
        let complete = states
            .iter()
            .map(|states| states.complete.clone())
            .collect::<Vec<_>>();
        let unasserted = states
            .iter()
            .map(|states| states.unasserted.clone())
            .collect::<Vec<_>>();
        let s4_blinded = states
            .iter()
            .map(|states| states.s4_blinded.clone())
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
        ViewStates {
            complete: join_at(
                &complete,
                &self.terms,
                &self.goals,
                &mut self.derivations,
                ProofView::Complete,
                event,
            ),
            unasserted: join_at(
                &unasserted,
                &self.terms,
                &self.goals,
                &mut self.derivations,
                ProofView::Unasserted,
                event,
            ),
            s4_blinded: join_at(
                &s4_blinded,
                &self.terms,
                &self.goals,
                &mut self.derivations,
                ProofView::S4Blinded,
                event,
            ),
            entry_images,
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
        view: ProofView,
        closed: ClosedState,
        context: &DeliveryEdgeContext<'_>,
    ) -> FactState {
        if closed.contradictory() {
            return FactState::contradictory_for_view(
                view,
                closed
                    .contradiction_proof()
                    .expect("contradictory delivery edge has one exact proof"),
            );
        }
        let mut image = FactState::for_view(view);
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
            let proof = self.derivations.intern_for(
                view,
                DerivationNode::PostconditionGive {
                    statement: context.statement.clone(),
                    carrier: context.carrier_binding,
                    receiver: context.receiver_binding,
                    relation: relation.clone(),
                    event: context.event,
                    parent,
                },
            );
            image.establish_from_proof(&relation, proof, &self.derivations);
        }
        image
    }

    fn retain_delivery_give_parents(&mut self, parents: &[JoinParent], view: ProofView) {
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
                DerivationRootKind::PostconditionGive { occurrence, view },
                parent.parent,
            );
        }
    }

    fn value_if_delivery_image(
        &mut self,
        value: &CheckedExpression,
        source: &ViewStates,
        context: DeliveryImageContext<'_>,
    ) -> ViewStates {
        let Some((carrier_binding, carrier, fragment)) =
            self.eligible_delivery_terms(value, context.receiver_type)
        else {
            return ViewStates::default();
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
        let complete = close_excluding_term(
            &source.complete,
            &self.terms,
            &self.goals,
            &mut self.derivations,
            receiver,
        );
        let unasserted = close_excluding_term(
            &source.unasserted,
            &self.terms,
            &self.goals,
            &mut self.derivations,
            receiver,
        );
        let s4_blinded = close_excluding_term(
            &source.s4_blinded,
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
        let mut image = ViewStates {
            complete: self.delivery_edge_state(ProofView::Complete, complete, &edge),
            unasserted: self.delivery_edge_state(ProofView::Unasserted, unasserted, &edge),
            s4_blinded: self.delivery_edge_state(ProofView::S4Blinded, s4_blinded, &edge),
            entry_images: Vec::new(),
        };
        // The forward substitution happens above before the ordinary edge
        // kills, so the carrier's own branch scope cannot delete the image.
        self.exit_scopes_to(&mut image, context.scope_depth);
        self.exit_counted_loops_from(&mut image, context.loop_depth);
        image
    }

    fn establish_delivery_join_view(
        &mut self,
        images: &[FactState],
        view: ProofView,
        context: &DeliveryJoinContext<'_>,
        target: &mut FactState,
    ) {
        assert!(images.iter().all(|image| {
            image.all_derivable
                || image.live_l0_relations().iter().all(|(_, proof)| {
                    self.derivations.node_views[proof.0 as usize] == view
                        && matches!(
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
            let proof = self.derivations.intern_for(
                view,
                DerivationNode::PostconditionDeliveryJoin {
                    statement: context.statement.clone(),
                    receiver: context.receiver_binding,
                    relation: relation.clone(),
                    event: context.event,
                    parents,
                },
            );
            let DerivationNode::PostconditionDeliveryJoin { parents, .. } =
                &self.derivations.nodes[proof.0 as usize]
            else {
                unreachable!("just interned one delivery join")
            };
            let parents = parents.clone();
            self.retain_delivery_give_parents(&parents, view);
            let occurrence = self.delivery_join_roots;
            self.delivery_join_roots = self
                .delivery_join_roots
                .checked_add(1)
                .expect("value-if delivery join roots exceed the u32 identity space");
            self.derivations.add_root(
                DerivationRootKind::PostconditionDeliveryJoin { occurrence, view },
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
            let proof = self.derivations.intern_for(
                view,
                DerivationNode::PostconditionDeliveryJoin {
                    statement: context.statement.clone(),
                    receiver: context.receiver_binding,
                    relation: relation.clone(),
                    event: context.event,
                    parents,
                },
            );
            let DerivationNode::PostconditionDeliveryJoin { parents, .. } =
                &self.derivations.nodes[proof.0 as usize]
            else {
                unreachable!("just interned one delivery join")
            };
            let parents = parents.clone();
            self.retain_delivery_give_parents(&parents, view);
            let occurrence = self.delivery_join_roots;
            self.delivery_join_roots = self
                .delivery_join_roots
                .checked_add(1)
                .expect("value-if delivery join roots exceed the u32 identity space");
            self.derivations.add_root(
                DerivationRootKind::PostconditionDeliveryJoin { occurrence, view },
                proof,
            );
            target.establish_from_proof(&relation, proof, &self.derivations);
        }
    }

    fn establish_value_if_delivery_join(&mut self, frame: &GiveFrame, target: &mut ViewStates) {
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
        let complete = frame
            .delivery_images
            .iter()
            .map(|image| image.complete.clone())
            .collect::<Vec<_>>();
        let unasserted = frame
            .delivery_images
            .iter()
            .map(|image| image.unasserted.clone())
            .collect::<Vec<_>>();
        let s4_blinded = frame
            .delivery_images
            .iter()
            .map(|image| image.s4_blinded.clone())
            .collect::<Vec<_>>();
        self.establish_delivery_join_view(
            &complete,
            ProofView::Complete,
            &context,
            &mut target.complete,
        );
        self.establish_delivery_join_view(
            &unasserted,
            ProofView::Unasserted,
            &context,
            &mut target.unasserted,
        );
        self.establish_delivery_join_view(
            &s4_blinded,
            ProofView::S4Blinded,
            &context,
            &mut target.s4_blinded,
        );
    }

    fn walk_block(&mut self, statements: &[CheckedStatement], state: &mut ViewStates) -> bool {
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

    fn expression_effects(
        &mut self,
        expression: &CheckedExpression,
        state: &mut ViewStates,
    ) -> Option<PreparedCall> {
        let mut prepared = self.judge_expression(expression, state);
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        if let Some(prepared) = &mut prepared {
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
                self.apply_kills_one(&mut state.complete, std::slice::from_ref(event));
                self.apply_kills_one(&mut state.unasserted, std::slice::from_ref(event));
                self.apply_kills_one(&mut state.s4_blinded, std::slice::from_ref(event));
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
        state: &mut ViewStates,
    ) -> SetWalkOutcome {
        // [SET-1]: the target's base and offset are evaluated before the
        // right-hand side; both are judged at this point, then the commit
        // kill applies.
        self.judge_set_target(target, state);
        let prepared = self.expression_effects(value, state);
        let receiver_route = prepared
            .as_ref()
            .and_then(|prepared| self.direct_receiver_route(target, value, prepared));
        invalidate_goal_origin_for_set(&mut state.complete, target);
        invalidate_goal_origin_for_set(&mut state.unasserted, target);
        invalidate_goal_origin_for_set(&mut state.s4_blinded, target);
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
                    state.for_each_mut(|view| {
                        view.origins.remove(&place.binding);
                        view.outcomes.remove(&place.binding);
                    });
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
        let receiver = prepared.as_ref().and_then(|prepared| {
            self.prepare_direct_receiver(receiver_route?, value, prepared, &target_kills)
        });
        let target_event = (force_target_event || receiver.is_some())
            .then(|| self.proof_event(FlowEventKind::PostconditionReceiverWrite, Some(node_path)));
        if let Some(target_event) = target_event {
            for event in &target_kills {
                self.apply_kills_one(&mut state.complete, std::slice::from_ref(event));
                self.apply_kills_one(&mut state.unasserted, std::slice::from_ref(event));
                self.apply_kills_one(&mut state.s4_blinded, std::slice::from_ref(event));
                self.invalidate_entry_images(
                    state,
                    std::slice::from_ref(event),
                    Some(target_event),
                );
            }
        } else {
            self.apply_kills(state, &target_kills);
        }
        if let (Some(prepared), Some(receiver), Some(target_event)) =
            (&prepared, receiver, target_event)
        {
            self.establish_direct_receiver(node_path, &receiver, prepared, target_event, state);
        }
        SetWalkOutcome { target_event }
    }

    fn walk_statement(&mut self, statement: &CheckedStatement, state: &mut ViewStates) -> bool {
        match statement {
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                let prepared = self.expression_effects(value, state);
                self.declare(*binding);
                if let Some(prepared) = &prepared {
                    self.establish_direct_result(node_path, *binding, value, prepared, state);
                }
                if value.ty() == CheckedType::Bool
                    && let Some(relation) = self.direct_comparison(value)
                {
                    state.for_each_mut(|view| {
                        view.origins.insert(*binding, relation.clone());
                    });
                }
                self.record_goal_origin(*binding, value, &mut state.complete);
                self.record_goal_origin(*binding, value, &mut state.unasserted);
                self.record_goal_origin(*binding, value, &mut state.s4_blinded);
                // Sources S5, S6, S7, and S9 establish at the binding, after
                // the initializer's own kills [ENT-3, ENT-5].
                let mut event = None;
                self.establish_binding_facts(
                    node_path,
                    *binding,
                    value,
                    &mut state.complete,
                    &mut event,
                );
                self.establish_binding_facts(
                    node_path,
                    *binding,
                    value,
                    &mut state.unasserted,
                    &mut event,
                );
                self.establish_binding_facts(
                    node_path,
                    *binding,
                    value,
                    &mut state.s4_blinded,
                    &mut event,
                );
                true
            }
            CheckedStatement::PropagateLet {
                binding, scrutinee, ..
            } => {
                // The Err edge leaves the function; the normal continuation
                // keeps the preceding state subject to the initializer
                // call's own kill events, and the binder gains no fact
                // [ENT-5].
                let _ = self.expression_effects(scrutinee, state);
                self.declare(*binding);
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
                let _ = self.walk_set(node_path, target, value, false, state);
                self.declare(*binding);
                true
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                let _ = self.expression_effects(value, state);
                true
            }
            CheckedStatement::Check { condition, trap } => {
                let _ = self.expression_effects(condition, state);
                self.establish_passed_condition(
                    FlowEventKind::S2,
                    &trap.node_path,
                    condition,
                    &mut state.complete,
                );
                true
            }
            CheckedStatement::Claim {
                name,
                predicate,
                justification,
                condition,
                trap,
                ..
            } => {
                let _ = self.expression_effects(condition, state);
                // [CLM-2] is judged at the claim with the state before the
                // claim's own passed fact: redundancy when the closed state
                // derives the predicate (a contradictory state derives
                // everything and never refutes), refutation when the
                // non-contradictory state derives the exact negation.
                let relation = self.scrutinee_relation(condition, &state.complete);
                let closed = close(
                    &state.complete,
                    &self.terms,
                    &self.goals,
                    &mut self.derivations,
                );
                let occurrence = u32::try_from(self.claims.len())
                    .expect("ENT claim-root occurrence exceeds the u32 identity space");
                let (disposition, lifecycle) = if let Some(relation) = relation {
                    if closed.derives(&relation) {
                        let proof = closed
                            .relation_proof(&relation, &mut self.derivations)
                            .expect("a derivable claim relation must retain its canonical proof");
                        (
                            ClaimDisposition::Redundant,
                            Some((ClaimLifecycleKind::Redundant, proof)),
                        )
                    } else {
                        let negation = relation.negated();
                        if !closed.contradictory() && closed.derives(&negation) {
                            let proof = closed
                                .relation_proof(&negation, &mut self.derivations)
                                .expect("a derived claim negation must retain its canonical proof");
                            (
                                ClaimDisposition::Refuted {
                                    predicate: self.render_relation(&relation),
                                    negation: self.render_relation(&negation),
                                },
                                Some((ClaimLifecycleKind::Refuted, proof)),
                            )
                        } else {
                            (ClaimDisposition::Retained, None)
                        }
                    }
                } else {
                    (ClaimDisposition::Retained, None)
                };
                let lifecycle_derivation = lifecycle.map(|(kind, proof)| {
                    self.derivations.add_root(
                        DerivationRootKind::ClaimLifecycle { occurrence, kind },
                        proof,
                    );
                    proof
                });
                self.claims.push(ClaimOutcome {
                    node_path: trap.node_path.clone(),
                    name: name.clone(),
                    predicate: predicate.clone(),
                    justification: justification.clone(),
                    disposition,
                    lifecycle_derivation,
                });
                // [ENT-3] S3: the passed predicate holds on the normal
                // continuation, exactly as S2 establishes a check's.
                self.establish_passed_condition(
                    FlowEventKind::S3,
                    &trap.node_path,
                    condition,
                    &mut state.complete,
                );
                true
            }
            CheckedStatement::Return {
                node_path, value, ..
            } => {
                let _ = self.expression_effects(value, state);
                self.judge_postcondition_return(node_path, state);
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
                let facts = ViewArmFacts {
                    complete: self.arm_facts(scrutinee, *enum_type, &state.complete),
                    unasserted: self.arm_facts(scrutinee, *enum_type, &state.unasserted),
                    s4_blinded: self.arm_facts(scrutinee, *enum_type, &state.s4_blinded),
                };
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
                    *state = self.join_views(&exits);
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
                let facts = ViewArmFacts {
                    complete: self.arm_facts(scrutinee, *enum_type, &state.complete),
                    unasserted: self.arm_facts(scrutinee, *enum_type, &state.unasserted),
                    s4_blinded: self.arm_facts(scrutinee, *enum_type, &state.s4_blinded),
                };
                self.gives.push(GiveFrame {
                    scope_depth: self.scopes.len(),
                    loop_depth: self.loops.len(),
                    kind: *kind,
                    node_path: node_path.clone(),
                    binding: *binding,
                    result_type: *result_type,
                    gives: Vec::new(),
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
                *state = self.join_views(&frame.gives);
                if frame.kind == ValueInitializerKind::ValueIf {
                    self.establish_value_if_delivery_join(&frame, state);
                }
                true
            }
            CheckedStatement::Loop { id, body, .. } => {
                // [ENT-5] no-induction loop rule: the head state is the state
                // before the loop minus every fact a continuing kill event in
                // the body may kill. The body's normal exit is this loop's
                // backedge; exits from the body are not.
                let mut kills = LoopKills::default();
                self.collect_continuing_loop_kills(
                    body,
                    true,
                    &mut LoopReachability::default(),
                    &mut kills,
                );
                self.apply_loop_kills(state, &kills, None);
                let head_entry_images = state.entry_images.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: self.scopes.len(),
                    capture_path: None,
                    breaks: Vec::new(),
                });
                let mut head = state.clone();
                let _ = self.walk_block(body, &mut head);
                let frame = self.loops.pop();
                let breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                let has_breaks = !breaks.is_empty();
                // The continuation is the join over the break edges; with no
                // break it is the contradictory all-derivable state, matching
                // an unreachable-in-truth continuation the conservative graph
                // keeps reachable [ENT-5].
                *state = self.join_views(&breaks);
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
                let _ = self.expression_effects(lower, state);
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
                    &mut state.complete,
                    preheader_event,
                );
                let unasserted_terms = self.establish_counted_preheader(
                    &range_path,
                    *binder,
                    lower,
                    upper,
                    &mut state.unasserted,
                    preheader_event,
                );
                let blinded_terms = self.establish_counted_preheader(
                    &range_path,
                    *binder,
                    lower,
                    upper,
                    &mut state.s4_blinded,
                    preheader_event,
                );
                // S11 fixes the complete post-capture closure before
                // continuing kills are subtracted. This preserves sound
                // snapshot consequences without rereading a mutable endpoint
                // on later iterations.
                let snapshot = self.derivations.event(FlowEventKind::Snapshot, None);
                state.complete = materialize_closure_at(
                    &state.complete,
                    &self.terms,
                    &self.goals,
                    &mut self.derivations,
                    snapshot,
                );
                state.unasserted = materialize_closure_at(
                    &state.unasserted,
                    &self.terms,
                    &self.goals,
                    &mut self.derivations,
                    snapshot,
                );
                state.s4_blinded = materialize_closure_at(
                    &state.s4_blinded,
                    &self.terms,
                    &self.goals,
                    &mut self.derivations,
                    snapshot,
                );
                let counted = self.capture_counted_preheader(counted_terms, &state.complete);
                let unasserted_counted =
                    self.capture_counted_preheader(unasserted_terms, &state.unasserted);
                let blinded_counted =
                    self.capture_counted_preheader(blinded_terms, &state.s4_blinded);

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

                let head = state.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: outer_scope_depth,
                    capture_path: Some(range_path.clone()),
                    breaks: Vec::new(),
                });
                let mut body_state = head.clone();
                let body_event = self.proof_event(FlowEventKind::S11, Some(node_path));
                let counted = self.establish_counted_body_entry(
                    node_path,
                    counted,
                    &mut body_state.complete,
                    body_event,
                );
                let _ = self.establish_counted_body_entry(
                    node_path,
                    unasserted_counted,
                    &mut body_state.unasserted,
                    body_event,
                );
                let _ = self.establish_counted_body_entry(
                    node_path,
                    blinded_counted,
                    &mut body_state.s4_blinded,
                    body_event,
                );
                self.retain_counted_derivations(occurrence, counted);
                let _ = self.walk_block(body, &mut body_state);
                let frame = self.loops.pop();
                let breaks = frame.map(|frame| frame.breaks).unwrap_or_default();

                // Unlike an ordinary loop, the real false-header edge always
                // contributes. Binder and captures leave scope before it or
                // a matching break reaches the continuation.
                let mut exhaustion = head;
                self.exit_scopes_to(&mut exhaustion, outer_scope_depth);
                self.exit_counted_capture_scope(&mut exhaustion, &range_path);
                let mut exits = Vec::with_capacity(1 + breaks.len());
                exits.push(exhaustion);
                exits.extend(breaks);
                self.scopes.pop();
                *state = self.join_views(&exits);
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
        entry: &ViewStates,
        facts: &ViewArmFacts,
        direct_call: Option<(&CheckedExpression, CheckedEnumType, &PreparedCall)>,
    ) -> Option<ViewStates> {
        let mut state = entry.clone();
        let arm_facts = [&facts.complete, &facts.unasserted, &facts.s4_blinded];
        let s1_event = arm_facts
            .iter()
            .find(|facts| !facts.goals.is_empty() || facts.comparison.is_some())
            .map(|facts| self.proof_event(FlowEventKind::S1, facts.node_path.as_ref()));
        let outcome_event = arm_facts
            .iter()
            .find_map(|facts| {
                facts
                    .outcome
                    .as_ref()
                    .map(|(_, outcome)| outcome.event_kind)
            })
            .and_then(|kind| {
                arm.binders
                    .iter()
                    .find(|binder| binder.field == 0)
                    .map(|binder| self.proof_event(kind, Some(&binder.node_path)))
            });
        self.establish_arm_entry(
            arm,
            &facts.complete,
            &mut state.complete,
            s1_event,
            outcome_event,
        );
        self.establish_arm_entry(
            arm,
            &facts.unasserted,
            &mut state.unasserted,
            s1_event,
            outcome_event,
        );
        self.establish_arm_entry(
            arm,
            &facts.s4_blinded,
            &mut state.s4_blinded,
            s1_event,
            outcome_event,
        );
        let direct_match = direct_call.and_then(|(scrutinee, enum_type, prepared)| {
            self.establish_direct_match(scrutinee, enum_type, arm, prepared, &mut state)
        });
        self.scopes
            .push(arm.binders.iter().map(|b| b.binding).collect());
        let mut continues = true;
        let mut first = 0usize;
        if let (Some((scrutinee, _, _)), Some(direct_match), Some(statement)) =
            (direct_call, direct_match.as_ref(), arm.body.first())
            && let Some(candidate) =
                self.prepare_selected_receiver(arm, statement, scrutinee, direct_match)
        {
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
            self.establish_selected_receiver(node_path, &candidate, target_event, &mut state);
            continues = true;
            first = 1;
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
            } else if arm.tag == 0 {
                state.establish_goal(
                    *goal,
                    GoalSign::Negative,
                    &mut self.derivations,
                    event.expect("goal arm has an S1 proof event"),
                );
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
            | CheckedStatement::Check { .. }
            | CheckedStatement::Claim { .. } => normal_reaches,
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
            | CheckedStatement::Check {
                condition: value, ..
            }
            | CheckedStatement::Claim {
                condition: value, ..
            }
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
        self.promote_contradiction(state);
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
    }

    fn apply_loop_kills(
        &mut self,
        states: &mut ViewStates,
        kills: &LoopKills,
        event: Option<FlowEventId>,
    ) {
        self.apply_loop_kills_one(&mut states.complete, kills);
        self.apply_loop_kills_one(&mut states.unasserted, kills);
        self.apply_loop_kills_one(&mut states.s4_blinded, kills);
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

    /// Renders one normalized relation for the [CLM-2] refutation diagnostic.
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
            CheckedExpression::BoxDeref { value, .. } => {
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

/// A let-origin expansion is valid only while the bound value has no `set`
/// target on the path to its use. The target's projection does not narrow
/// this invalidation: changing one field or element invalidates the aggregate
/// value identity even when a separately established length fact survives.
fn invalidate_goal_origin_for_set(state: &mut FactState, target: &CheckedSetTarget) {
    state.goal_origins.remove(&target.binding());
}

fn holder_from_value(value: &CheckedExpression) -> Option<HolderReferent> {
    match value {
        CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::BorrowSystemResource { binding, .. } => Some(HolderReferent::Place {
            binding: *binding,
            fields: Vec::new(),
        }),
        CheckedExpression::BorrowBuffer { root, .. } => Some(HolderReferent::Place {
            binding: root.binding,
            fields: root.fields.clone(),
        }),
        CheckedExpression::ReborrowAddressed { binding, .. } => {
            Some(HolderReferent::Holder(*binding))
        }
        CheckedExpression::BoxNew { .. } => Some(HolderReferent::Opaque),
        _ => None,
    }
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

const fn value_has_implicit_deref(value: &CheckedExpression) -> bool {
    matches!(
        value,
        CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
    )
}

/// Every direct subexpression, for uniform recursion.
pub(super) fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
    match expression {
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayLength { .. }
        | CheckedExpression::BufferLength { .. }
        | CheckedExpression::SliceLength { .. }
        | CheckedExpression::SliceOf { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowAddressed { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => Vec::new(),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::SystemCall { arguments, .. }
        | CheckedExpression::IntegerOperation { arguments, .. }
        | CheckedExpression::FloatOperation { arguments, .. }
        | CheckedExpression::BooleanOperation { arguments, .. }
        | CheckedExpression::EnumEquality { arguments, .. } => arguments.iter().collect(),
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value.as_ref()],
        CheckedExpression::ArrayIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::BufferFill { length, value, .. } => {
            vec![length.as_ref(), value.as_ref()]
        }
        CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::ConstructStruct { fields, .. }
        | CheckedExpression::ConstructEnum { fields, .. } => fields.iter().collect(),
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
