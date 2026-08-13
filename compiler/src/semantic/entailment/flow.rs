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

use std::collections::HashSet;

use super::super::goal::{ConcreteGoal, GoalDatum, GoalExpression, GoalOperation, GoalProjection};
use super::super::model::{
    BindingId, CheckedArrayRoot, CheckedConst, CheckedExpression, CheckedFloatOperation,
    CheckedFunction, CheckedLoopId, CheckedMatchArm, CheckedMode, CheckedNominal,
    CheckedNominalKind, CheckedSetTarget, CheckedStatement, CheckedType, CheckedValue, IntegerType,
};
use super::state::{
    CountedRootAtom, DerivationId, DerivationInventory, DerivationLedger, DerivationRootKind,
    FactState, FlowEventId, FlowEventKind, GoalId, GoalSign, GoalSupport, GoalTable, OutcomeFact,
    Relation, close, join, materialize_closure,
};
use super::term::{
    CountedCaptureSide, LengthBound, PlaceProjection, PlaceRoot, PlaceTerm, ProjectedPlaceTerm,
    TermId, TermKind, TermTable, integer_value,
};
use super::{
    BoundsRequest, CallGoalCounterfactual, CallGoalDisposition, CallGoalEvidence, CallGoalOutcome,
    ClaimDisposition, ClaimOutcome, CountedDerivationSet, EntailmentContext, FunctionEntailment,
    FunctionEntailmentRewalk, ObligationOutcome, RewalkObligationOutcome, fragment_type,
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
    Write { place: ResolvedPlace, element: bool },
    /// (c) a consuming use of a binding.
    Consume(BindingId),
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
}

/// A `loop` frame collecting break-edge states for the continuation join.
struct LoopFrame {
    id: CheckedLoopId,
    scope_depth: usize,
    /// Present only for a counted range. A break through this frame leaves
    /// the private endpoint-capture scope as well as source binding scopes.
    capture_path: Option<Vec<u32>>,
    breaks: Vec<FactState>,
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
    gives: Vec<FactState>,
}

/// The [ENT-5] loop rule's structural kill summary of one loop body.
#[derive(Default)]
struct LoopKills {
    events: Vec<KillEvent>,
    /// Every binding named as a `set` target. An ordinary-let origin is valid
    /// only while its bound value has no intervening whole, field, or element
    /// mutation; the narrower comparison/outcome origins can only inhabit
    /// nonprojectable Bool/outcome bindings, so this same set is exact there.
    set_bindings: HashSet<BindingId>,
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
    let run = run(function, context, true, true, false);
    FunctionEntailment {
        obligations: run.obligations,
        claims: run.claims,
        call_goals: run.call_goals,
        counted_derivations: run.counted_derivations,
        derivations: run.derivations,
        inventory: run.inventory,
    }
}

pub(super) fn rewalk_unasserted(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
    include_s4: bool,
) -> FunctionEntailmentRewalk {
    let run = run(function, context, false, include_s4, true);
    FunctionEntailmentRewalk {
        obligations: run
            .obligations
            .into_iter()
            .map(|outcome| RewalkObligationOutcome {
                node_path: outcome.node_path,
                discharged: outcome.discharged,
                residual: outcome.residual,
            })
            .collect(),
        call_goals: run.call_counterfactuals,
    }
}

struct AnalysisRun {
    obligations: Vec<ObligationOutcome>,
    claims: Vec<ClaimOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    call_counterfactuals: Vec<CallGoalCounterfactual>,
    counted_derivations: Vec<CountedDerivationSet>,
    derivations: DerivationLedger,
    inventory: DerivationInventory,
}

fn run(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
    include_asserted_sources: bool,
    include_s4: bool,
    counterfactual_calls: bool,
) -> AnalysisRun {
    let mut analyzer = Analyzer {
        context,
        function,
        include_asserted_sources,
        include_s4,
        counterfactual_calls,
        bindings: Vec::new(),
        terms: TermTable::new(),
        goals: GoalTable::default(),
        derivations: DerivationLedger::default(),
        obligations: Vec::new(),
        claims: Vec::new(),
        call_goals: Vec::new(),
        call_counterfactuals: Vec::new(),
        counted_derivations: Vec::new(),
        encountered_counted: 0,
        completed_counted_roots: 0,
        scopes: Vec::new(),
        loops: Vec::new(),
        gives: Vec::new(),
    };
    analyzer.collect_bindings();
    let mut state = FactState::default();
    analyzer
        .scopes
        .push(function.parameters.iter().map(|p| p.binding).collect());
    // [ENT-3] S4: the substituted `requires` relation enters the body's entry
    // fact state, the one fact that crosses into the body [ENT-2, FN-8].
    if analyzer.include_s4 {
        analyzer.establish_requires_facts(&mut state);
    }
    analyzer.walk_block(&function.body, &mut state);
    analyzer.scopes.pop();
    assert_eq!(
        analyzer.completed_counted_roots, analyzer.encountered_counted,
        "every encountered counted statement must publish one complete S11 root group"
    );
    let remap = analyzer.derivations.finish();
    for outcome in &mut analyzer.obligations {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.get(id.0 as usize).copied().flatten());
    }
    for outcome in &mut analyzer.call_goals {
        outcome.derivation = outcome
            .derivation
            .and_then(|id| remap.get(id.0 as usize).copied().flatten());
    }
    for counted in &mut analyzer.counted_derivations {
        remap_counted_derivations(counted, &remap);
    }
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
        call_counterfactuals: analyzer.call_counterfactuals,
        counted_derivations: analyzer.counted_derivations,
        derivations: analyzer.derivations,
        inventory,
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

struct Analyzer<'check, 'unit> {
    context: &'check EntailmentContext<'unit>,
    function: &'check CheckedFunction,
    /// Whether executed writer assertions S2/S3 establish facts in this run.
    include_asserted_sources: bool,
    /// Whether the body's proved requirement S4 enters at body entry.
    include_s4: bool,
    /// Whether calls retain isolated goal counterfactuals even when an actual
    /// expression obligation fails under this metadata-only state.
    counterfactual_calls: bool,
    /// Dense per-binding summaries indexed by [`BindingId`].
    bindings: Vec<BindingSummary>,
    terms: TermTable,
    goals: GoalTable,
    derivations: DerivationLedger,
    obligations: Vec<ObligationOutcome>,
    claims: Vec<ClaimOutcome>,
    call_goals: Vec<CallGoalOutcome>,
    call_counterfactuals: Vec<CallGoalCounterfactual>,
    counted_derivations: Vec<CountedDerivationSet>,
    encountered_counted: u32,
    completed_counted_roots: u32,
    /// Lexical scope stack: the bindings declared in each open block.
    scopes: Vec<Vec<BindingId>>,
    loops: Vec<LoopFrame>,
    gives: Vec<GiveFrame>,
}

impl Analyzer<'_, '_> {
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
        }
        self.collect_block_bindings(&function.body);
    }

    fn collect_block_bindings(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let holder = holder_from_value(value);
                    let implicit_deref = value_has_implicit_deref(value);
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.holder = holder;
                    summary.implicit_deref = implicit_deref;
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    self.summary_mut(*binding).ty = Some(*ok_type);
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    self.summary_mut(*binding).ty = Some(*result_type);
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
                    self.summary_mut(*binder).ty = Some(CheckedType::Integer(IntegerType::U64));
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
                KillEvent::Write {
                    place: written,
                    element: _,
                } => self.resolve(&place).overlaps(written),
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
            },
            TermKind::ProjectedPlace(place, _) => match event {
                KillEvent::Write {
                    place: written,
                    element: _,
                } => self.resolve_projected(&place).overlaps(written),
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
            },
            TermKind::Length(place) => match event {
                // An element write never kills a length fact: the length is
                // fixed at allocation or by the type [ENT-5].
                KillEvent::Write { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                } => {
                    let root = PlaceTerm {
                        root: place.root,
                        deref: place.deref,
                        fields: Vec::new(),
                    };
                    self.resolve(&root).overlaps(written)
                }
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
            },
            TermKind::ProjectedLength(place) => match event {
                KillEvent::Write { element: true, .. } => false,
                KillEvent::Write {
                    place: written,
                    element: false,
                } => self.resolve_projected(&place).overlaps(written),
                KillEvent::Consume(root) => place.root == PlaceRoot::Binding(*root),
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
                KillEvent::Consume(root) => {
                    holders.contains(root) || place.root == PlaceRoot::Binding(*root)
                }
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
            KillEvent::Consume(root) => binding == *root,
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

    fn apply_kills(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        self.promote_contradiction(state);
        state.kill(|term| {
            events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
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

    /// Applies the scope-exit kills for every scope deeper than `depth`,
    /// as the edge event ordered before any join [ENT-5].
    fn exit_scopes_to(&mut self, state: &mut FactState, depth: usize) {
        let exited: HashSet<BindingId> =
            self.scopes.iter().skip(depth).flatten().copied().collect();
        if exited.is_empty() {
            return;
        }
        self.promote_contradiction(state);
        state.kill(|term| self.scope_kills_term(term, &exited));
        state.kill_goals(|goal| self.scope_kills_goal(goal, &exited));
        state.origins.retain(|binding, _| !exited.contains(binding));
        state
            .outcomes
            .retain(|binding, _| !exited.contains(binding));
        state
            .goal_origins
            .retain(|binding, _| !exited.contains(binding));
    }

    /// Applies the private capture-scope kill of one counted construct.
    fn exit_counted_capture_scope(&mut self, state: &mut FactState, range_path: &[u32]) {
        self.promote_contradiction(state);
        state.kill(|term| {
            matches!(
                self.terms.kind(term),
                TermKind::CountedCapture { range_path: path, .. } if path == range_path
            )
        });
    }

    /// Applies capture-scope kills for every loop frame crossed by a
    /// non-local edge. Ordinary loop frames carry no private captures.
    fn exit_counted_loops_from(&mut self, state: &mut FactState, loop_depth: usize) {
        let paths: Vec<Vec<u32>> = self
            .loops
            .iter()
            .skip(loop_depth)
            .filter_map(|frame| frame.capture_path.clone())
            .collect();
        for path in paths {
            self.exit_counted_capture_scope(state, &path);
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
    fn argument_referent(&self, argument: &CheckedExpression) -> Option<(ResolvedPlace, bool)> {
        match argument {
            CheckedExpression::BorrowBuffer { root, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                Some((self.resolve(&place), true))
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: Vec::new(),
                };
                Some((self.resolve(&place), false))
            }
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                Some((self.resolve_deref(*binding, 0), false))
            }
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
            CheckedExpression::Binding { binding, ty, .. } => {
                if !self.is_copy(*ty) {
                    events.push(KillEvent::Consume(*binding));
                }
            }
            CheckedExpression::Project {
                binding,
                consume_root,
                ..
            } => {
                if *consume_root {
                    events.push(KillEvent::Consume(*binding));
                }
            }
            // These wrappers are checked reads of one place. Their nested
            // expression preserves source spelling and lowering structure;
            // it is not a second consuming evaluation of an affine holder.
            CheckedExpression::BoxDeref { .. } | CheckedExpression::ProjectValue { .. } => {}
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => {
                let callee = self.context.callee(*function);
                for (index, argument) in arguments.iter().enumerate() {
                    let written = callee.is_some_and(|callee| {
                        callee.parameter_writes.get(index).copied().unwrap_or(false)
                    });
                    if written && let Some((place, element)) = self.argument_referent(argument) {
                        events.push(KillEvent::Write { place, element });
                    }
                    self.collect_expression_kills(argument, events);
                }
            }
            CheckedExpression::SystemCall {
                operation,
                arguments,
                ..
            } => {
                let parameters = SYSTEM_OPERATIONS
                    .get(usize::from(*operation))
                    .map(|operation| operation.parameters)
                    .unwrap_or_default();
                for (index, argument) in arguments.iter().enumerate() {
                    let written = parameters.get(index).is_some_and(|parameter| {
                        matches!(parameter.mode, SystemParameterMode::UniqueBorrow(_))
                    });
                    if written && let Some((place, element)) = self.argument_referent(argument) {
                        events.push(KillEvent::Write { place, element });
                    }
                    self.collect_expression_kills(argument, events);
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
    fn judge_expression(&mut self, expression: &CheckedExpression, state: &FactState) {
        match expression {
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                requirement,
                ..
            } => {
                let obligation_start = self.obligations.len();
                for argument in arguments {
                    self.judge_expression(argument, state);
                }
                let actual_obligations_ok = !self.obligations[obligation_start..]
                    .iter()
                    .any(|outcome| !outcome.discharged);
                // FN-8 begins only after every actual-expression obligation
                // succeeds. A failed OP-4 actual therefore publishes no call
                // judgment for diagnostic selection to reorder.
                if let Some(requirement) = requirement {
                    if self.counterfactual_calls {
                        self.judge_call_goal_counterfactual(
                            *function,
                            call,
                            requirement.final_check.clone(),
                            requirement.goal.clone(),
                            actual_obligations_ok,
                            state,
                        );
                    } else if actual_obligations_ok {
                        self.judge_call_goal(
                            *function,
                            call,
                            requirement.final_check.clone(),
                            requirement.goal.clone(),
                            state,
                        );
                    }
                }
            }
            CheckedExpression::ArrayIndex {
                root,
                length,
                offset,
                trap,
                ..
            } => {
                self.judge_expression(offset, state);
                let base = self.array_root_place(root);
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, Some(*length), offset, node_path, state);
            }
            CheckedExpression::BufferIndex {
                root, offset, trap, ..
            } => {
                self.judge_expression(offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, state);
            }
            CheckedExpression::SliceIndex {
                root, offset, trap, ..
            } => {
                self.judge_expression(offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: Vec::new(),
                };
                let node_path = trap.node_path.clone();
                self.judge_obligation(base, None, offset, node_path, state);
            }
            _ => {
                for child in expression_children(expression) {
                    self.judge_expression(child, state);
                }
            }
        }
    }

    fn judge_call_goal(
        &mut self,
        callee: super::super::model::FunctionId,
        node_path: &crate::NodePath,
        final_check: crate::NodePath,
        goal: ConcreteGoal,
        state: &FactState,
    ) {
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
            disposition,
            evidence,
            derivation,
        });
    }

    fn judge_call_goal_counterfactual(
        &mut self,
        callee: super::super::model::FunctionId,
        node_path: &crate::NodePath,
        final_check: crate::NodePath,
        goal: ConcreteGoal,
        actual_obligations_ok: bool,
        state: &FactState,
    ) {
        let (goal_disposition, goal_evidence, _) = self.call_goal_disposition(&goal, state);
        self.call_counterfactuals.push(CallGoalCounterfactual {
            node_path: node_path.clone(),
            callee,
            final_check,
            goal,
            actual_obligations_ok,
            goal_disposition,
            goal_evidence,
        });
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
        state: &FactState,
    ) {
        let length_term = self.length_term(base.clone(), array_length);
        let offset_term = self.read_operand(offset);
        let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
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
            Some(format!(
                "{} < len({})",
                self.render_expression(offset),
                self.render_place(&base)
            ))
        };
        let derivation = if discharged && !self.counterfactual_calls {
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
            node_path,
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
    }

    fn judge_set_target(&mut self, target: &CheckedSetTarget, state: &FactState) {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                self.judge_expression(&target.offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, Some(target.length), &target.offset, node_path, state);
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.judge_expression(&target.offset, state);
                let base = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                let node_path = target.trap.node_path.clone();
                self.judge_obligation(base, None, &target.offset, node_path, state);
            }
        }
    }

    // ------------------------------------------------------------------
    // Statement walk
    // ------------------------------------------------------------------

    /// Walks one block in its own lexical scope. Returns the fall-through:
    /// `true` when control continues past the block with `state` holding the
    /// post-scope-exit facts.
    fn walk_block(&mut self, statements: &[CheckedStatement], state: &mut FactState) -> bool {
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

    fn expression_effects(&mut self, expression: &CheckedExpression, state: &mut FactState) {
        self.judge_expression(expression, state);
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        self.apply_kills(state, &events);
    }

    fn walk_statement(&mut self, statement: &CheckedStatement, state: &mut FactState) -> bool {
        match statement {
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                self.expression_effects(value, state);
                self.declare(*binding);
                if value.ty() == CheckedType::Bool
                    && let Some(relation) = self.direct_comparison(value)
                {
                    state.origins.insert(*binding, relation);
                }
                self.record_goal_origin(*binding, value, state);
                // Sources S5, S6, S7, and S9 establish at the binding, after
                // the initializer's own kills [ENT-3, ENT-5].
                self.establish_binding_facts(node_path, *binding, value, state);
                true
            }
            CheckedStatement::PropagateLet {
                binding, scrutinee, ..
            } => {
                // The Err edge leaves the function; the normal continuation
                // keeps the preceding state subject to the initializer
                // call's own kill events, and the binder gains no fact
                // [ENT-5].
                self.expression_effects(scrutinee, state);
                self.declare(*binding);
                true
            }
            CheckedStatement::Set { target, value, .. } => {
                // [SET-1]: the target's base and offset are evaluated before
                // the right-hand side; both are judged at this point, then
                // the commit kill applies.
                self.judge_set_target(target, state);
                self.expression_effects(value, state);
                invalidate_goal_origin_for_set(state, target);
                let mut events = Vec::new();
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
                        });
                        if place.fields.is_empty() {
                            state.origins.remove(&place.binding);
                            state.outcomes.remove(&place.binding);
                        }
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
                        });
                    }
                }
                self.apply_kills(state, &events);
                true
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                self.expression_effects(value, state);
                true
            }
            CheckedStatement::Check { condition, trap } => {
                self.expression_effects(condition, state);
                if self.include_asserted_sources {
                    self.establish_passed_condition(
                        FlowEventKind::S2,
                        &trap.node_path,
                        condition,
                        state,
                    );
                }
                true
            }
            CheckedStatement::Claim {
                name,
                condition,
                trap,
                ..
            } => {
                self.expression_effects(condition, state);
                // [CLM-2] is judged at the claim with the state before the
                // claim's own passed fact: redundancy when the closed state
                // derives the predicate (a contradictory state derives
                // everything and never refutes), refutation when the
                // non-contradictory state derives the exact negation.
                let relation = self.scrutinee_relation(condition, state);
                let closed = close(state, &self.terms, &self.goals, &mut self.derivations);
                let disposition = if let Some(relation) = relation {
                    if closed.derives(&relation) {
                        ClaimDisposition::Redundant
                    } else if !closed.contradictory() && closed.derives(&relation.negated()) {
                        ClaimDisposition::Refuted {
                            predicate: self.render_relation(&relation),
                            negation: self.render_relation(&relation.negated()),
                        }
                    } else {
                        ClaimDisposition::Retained
                    }
                } else {
                    ClaimDisposition::Retained
                };
                self.claims.push(ClaimOutcome {
                    node_path: trap.node_path.clone(),
                    name: name.clone(),
                    disposition,
                });
                // [ENT-3] S3: the passed predicate holds on the normal
                // continuation, exactly as S2 establishes a check's.
                if self.include_asserted_sources {
                    self.establish_passed_condition(
                        FlowEventKind::S3,
                        &trap.node_path,
                        condition,
                        state,
                    );
                }
                true
            }
            CheckedStatement::Return { value, .. } => {
                self.expression_effects(value, state);
                false
            }
            CheckedStatement::Give { value, .. } => {
                self.expression_effects(value, state);
                if let Some((scope_depth, loop_depth)) = self
                    .gives
                    .last()
                    .map(|frame| (frame.scope_depth, frame.loop_depth))
                {
                    let mut exit = state.clone();
                    self.exit_scopes_to(&mut exit, scope_depth);
                    self.exit_counted_loops_from(&mut exit, loop_depth);
                    if let Some(frame) = self.gives.last_mut() {
                        frame.gives.push(exit);
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
                let facts = self.arm_facts(scrutinee, *enum_type, state);
                let mut exits = Vec::new();
                for arm in arms {
                    if let Some(exit) = self.walk_arm(arm, state, &facts) {
                        exits.push(exit);
                    }
                }
                if exits.is_empty() {
                    false
                } else {
                    *state = join(&exits, &self.terms, &self.goals, &mut self.derivations);
                    true
                }
            }
            CheckedStatement::ValueMatchLet {
                binding,
                scrutinee,
                enum_type,
                arms,
                ..
            } => {
                let facts = self.arm_facts(scrutinee, *enum_type, state);
                self.gives.push(GiveFrame {
                    scope_depth: self.scopes.len(),
                    loop_depth: self.loops.len(),
                    gives: Vec::new(),
                });
                for arm in arms {
                    // Every delivering path leaves by `give`; an arm's
                    // fall-through state contributes nothing [GIVE-1].
                    let _ = self.walk_arm(arm, state, &facts);
                }
                let frame = self.gives.pop();
                let gives = frame.map(|frame| frame.gives).unwrap_or_default();
                self.declare(*binding);
                if gives.is_empty() {
                    false
                } else {
                    *state = join(&gives, &self.terms, &self.goals, &mut self.derivations);
                    true
                }
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
                self.apply_loop_kills(state, &kills);
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
                // The continuation is the join over the break edges; with no
                // break it is the contradictory all-derivable state, matching
                // an unreachable-in-truth continuation the conservative graph
                // keeps reachable [ENT-5].
                *state = join(&breaks, &self.terms, &self.goals, &mut self.derivations);
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
                self.expression_effects(lower, state);
                self.expression_effects(upper, state);
                let outer_scope_depth = self.scopes.len();
                self.scopes.push(vec![*binder]);
                let range_path = node_path.components().to_vec();
                let counted_terms = self.establish_counted_preheader(
                    node_path,
                    &range_path,
                    *binder,
                    lower,
                    upper,
                    state,
                );
                // S11 fixes the complete post-capture closure before
                // continuing kills are subtracted. This preserves sound
                // snapshot consequences without rereading a mutable endpoint
                // on later iterations.
                *state =
                    materialize_closure(state, &self.terms, &self.goals, &mut self.derivations);
                let counted = self.capture_counted_preheader(counted_terms, state);

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
                    kills.events.push(KillEvent::Write {
                        place: ResolvedPlace {
                            root: PlaceRoot::Binding(*binder),
                            fields: Vec::new(),
                        },
                        element: false,
                    });
                    kills.set_bindings.insert(*binder);
                }
                self.apply_loop_kills(state, &kills);

                let head = state.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    scope_depth: outer_scope_depth,
                    capture_path: Some(range_path.clone()),
                    breaks: Vec::new(),
                });
                let mut body_state = head.clone();
                let counted =
                    self.establish_counted_body_entry(node_path, counted, &mut body_state);
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
                *state = join(&exits, &self.terms, &self.goals, &mut self.derivations);
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
        entry: &FactState,
        facts: &ArmFacts,
    ) -> Option<FactState> {
        let mut state = entry.clone();
        let event = (!facts.goals.is_empty() || facts.comparison.is_some())
            .then(|| self.proof_event(FlowEventKind::S1, facts.node_path.as_ref()));
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
            self.establish_binder_fact(arm, outcome, &mut state);
        }
        self.scopes
            .push(arm.binders.iter().map(|b| b.binding).collect());
        let mut continues = true;
        for statement in &arm.body {
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
                    self.collect_expression_kills(value, &mut kills.events);
                }
                normal_reaches
            }
            CheckedStatement::Set { target, value, .. } => {
                if normal_reaches {
                    self.collect_set_kills(target, value, kills);
                }
                normal_reaches
            }
            CheckedStatement::Return { .. } => false,
            CheckedStatement::Give { value, .. } => {
                let reaches = reachability.gives.last().copied().unwrap_or(false);
                if reaches {
                    self.collect_expression_kills(value, &mut kills.events);
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
                    self.collect_expression_kills(scrutinee, &mut kills.events);
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
                    self.collect_expression_kills(scrutinee, &mut kills.events);
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
                    self.collect_expression_kills(lower, &mut kills.events);
                    self.collect_expression_kills(upper, &mut kills.events);
                }
                reaches
            }
            CheckedStatement::Region { body, .. } => {
                self.collect_continuing_loop_kills(body, normal_reaches, reachability, kills)
            }
        }
    }

    fn collect_set_kills(
        &self,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        kills: &mut LoopKills,
    ) {
        self.collect_expression_kills(value, &mut kills.events);
        kills.set_bindings.insert(target.binding());
        match target {
            CheckedSetTarget::Place(place) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(place.binding),
                    deref: self.is_holder(place.binding),
                    fields: place.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: false,
                });
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.binding),
                    deref: self.is_holder(target.binding),
                    fields: target.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                });
            }
            CheckedSetTarget::BufferIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: target.root.fields.clone(),
                };
                kills.events.push(KillEvent::Write {
                    place: self.resolve(&spelled),
                    element: true,
                });
            }
        }
    }

    fn apply_loop_kills(&mut self, state: &mut FactState, kills: &LoopKills) {
        self.promote_contradiction(state);
        state.kill(|term| {
            kills
                .events
                .iter()
                .any(|event| self.event_kills_term(term, event))
        });
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
fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
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
