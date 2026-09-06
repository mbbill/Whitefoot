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

mod kernel;
mod sources;

use sources::{MeasureCarry, ValueImage};
use std::collections::{HashMap, HashSet};

use super::super::goal::{
    CheckedRequirement, ConcreteGoal, EvaluatedValueOccurrence, GoalDatum, GoalExpression,
    GoalOperation, GoalProjection,
};
use super::super::model::expression_children;
use super::super::model::{
    BindingId, CheckedAffineExpression, CheckedAffineExpressionKind, CheckedAffineRelation,
    CheckedArrayRoot, CheckedBooleanOperation, CheckedCommitValues, CheckedConst,
    CheckedConstructor, CheckedContainerRoot, CheckedEnumType, CheckedExpression,
    CheckedFloatOperation, CheckedFunction, CheckedIntegerOperation, CheckedLoopId,
    CheckedLoopInvariant, CheckedMatchArm, CheckedMeasure, CheckedMode, CheckedNominal,
    CheckedNominalKind, CheckedNumericType, CheckedPlaceStep, CheckedProofMultiplicity,
    CheckedProofUseSource, CheckedSetTarget, CheckedSliceSource, CheckedStatement, CheckedType,
    CheckedValue, FloatType, IntegerType, LoanStrength, MeasureCell, MeasuredKind,
    ValueInitializerKind,
};
use super::super::places::{BindingSummary, PlaceMap, PlaceOffset, PlaceStep, ResolvedPlace};
use super::super::postcondition::{
    CheckedPostcondition, NormalizedRelation, PostconditionPlaceRoot, PostconditionReturnDatum,
    PostconditionReturnPlace, PostconditionReturnPlaceRoot, RelationDatum, RelationTemplate,
};
use super::affine::{
    AffineCheckError, AffineCheckLimit, AffineCheckState, AffineCoefficient, AffineForm,
    AffineInequality, AffineTermId, MAX_CERTIFICATE_PREMISES, ScaledAffinePremise,
    integer_tightenings, interval_maximum, interval_proves, sum_explicit_inequalities,
    sum_explicit_scaled_inequalities,
};
use super::polynomial::{CertificatePolynomial, PolynomialError};
use super::state::{
    AffinePremiseUse, ClosedState, CountedRootAtom, DerivationId, DerivationInventory,
    DerivationLedger, DerivationNode, DerivationRootKind, FactState, FlowEventId, FlowEventKind,
    GoalId, GoalNormalization, GoalSign, GoalSupport, GoalTable, JoinParent, OutcomeFact,
    PostconditionCallSubstitution, Relation, SourceAffineFactRef, SourceLoopInvariantRef, close,
    close_excluding_term, contradiction_without_proofs, join_at, materialize_closure_at,
    materialize_closure_before_kill,
};
use super::term::{
    CallDatumProjection, CountedCaptureSide, MeasureBound, MeasurePlacement, PlaceProjection,
    PlaceRoot, PlaceTerm, ProjectedPlaceTerm, TermId, TermKind, TermTable, ZERO, integer_value,
    type_range,
};
use super::{
    BoundsRequest, CallGoalDisposition, CallGoalEvidence, CallGoalOutcome, CallTransport,
    CountedDerivationSet, EntailmentContext, FunctionEntailment, FunctionPostconditionProof,
    JoinedSourceProofProvenance, LoopInvariantOutcome, LoopInvariantProof, ObligationFamily,
    ObligationOutcome, PostconditionAggregate, PostconditionDisposition, PostconditionEntryImage,
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
    /// The selected type of the operand's place, which [MSR-1]'s measure
    /// former reads to know what it is measuring.
    ty: CheckedType,
    place: ResolvedPlace,
    holders: Vec<BindingId>,
}

/// A `loop` frame collecting break-edge states for the continuation join.
struct LoopFrame {
    id: CheckedLoopId,
    invariant_declarations: Box<[crate::DeclarationId]>,
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
    /// One atom standing for the whole value of a binding whose image is not
    /// already a single atom, minted on first demand.
    ///
    /// A local's image is transparent — `let stride = width + padding;` gives
    /// `stride` the image `width + padding` — which is what an affine relation
    /// wants and what [PRF-1]'s fold cannot use: a product over `stride`
    /// distributes into its operands and no admitted multiplication matches
    /// the pieces. This map is the opposite handle on the same binding, one
    /// value the certificate can name, and the fact published beside it keeps
    /// the transparent reading available to everything else. Keyed and killed
    /// exactly as `values` is, so a write mints a fresh handle for a fresh
    /// value.
    opaque_values: HashMap<BindingId, AffineForm>,
    /// Every published affine conclusion at this control-flow point. Fact
    /// identity is only the canonical inequality over immutable value images;
    /// evidence is retained solely to explain a selected derivation.
    facts: Vec<ActiveAffineFact>,
    /// Exact immutable theorem image published by each resolved invariant
    /// declaration. Resolution owns visibility; this map carries only the
    /// canonical proposition proved at that declaration's execution point.
    published_invariants: HashMap<crate::DeclarationId, AffineInequality>,
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

/// One consumer-normalized numeric/logical proposition. A signed goal keeps
/// the finite Boolean structure written at a call boundary and lets the proof
/// entry normalize its ordering leaves under that fixed structure. An
/// ordering goal is the exact L0 relation selected for a function
/// postcondition. Either consumer may additionally provide the unique affine
/// inequality for a direct-root proposition; the proof entry never invents
/// another formula.
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
    request: Option<BoundsRequest>,
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
    /// The interval [ENT-6]'s fixed interval-product rule proved for an
    /// admitted non-constant multiplication. Carried out of the judgment so
    /// [ENT-3.S14] publishes exactly the measurement the domain decision
    /// consumed, rather than proving the same endpoints a second time.
    product_interval: Option<AffineProductInterval>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveAffineFact {
    inequality: AffineInequality,
    evidence: AffineFactEvidence,
    /// Enclosing loop assumptions on which this fact still depends. Removing
    /// any listed loop removes the fact; an empty list is path-stable.
    active_loops: Vec<CheckedLoopId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AffineFactEvidence {
    Source(SourceAffineFactRef),
    Derivation(DerivationId),
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
    /// Whether a structural join minted this atom to stand for the spread of
    /// its inputs' constants. A delta atom is an ordinary shared atom between
    /// joins, so correlations formed over it survive; a later join folds it
    /// back into the interval it stands for so that nested joins reach the
    /// image their flat equivalent reaches [ENT-6].
    join_delta: bool,
}

/// One input image prepared for a structural join [ENT-6]: the part of it no
/// earlier join minted, and the closed constant interval the folded delta
/// atoms and the written constant together contribute.
struct FoldedJoinImage {
    form: AffineForm,
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
/// The callback receives each accumulated pair sum and owns the residual
/// against the target. Candidate arithmetic is isolated: an unrepresentable
/// pair is skipped and cannot hide a later representable witness. Returning
/// after a successful callback is acceptance-order independent because every
/// earlier candidate has already failed and adding another premise cannot
/// remove an existing pair from this source-shaped finite set.
fn first_two_premise_candidate<T>(
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
            if let Some(proof) = prove(&sum, check) {
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

/// One written multiplicity, resolved where the certificate is checked.
///
/// A named multiplicity carries the value image its binding holds at the
/// entering program point, so the scaling step reads a value rather than a
/// name and a later write cannot change what was scaled.
#[derive(Clone, Debug)]
enum CertificateMultiplicity {
    Literal(i128),
    Value(AffineForm),
}

/// The accumulated [PRF-1] certificate sum.
///
/// A bare-decimal certificate stays in `Affine` for its whole accumulation and
/// reaches the residual as the inequality it always formed. The first term
/// multiplicity moves the accumulation to `Nonlinear`, where it stays until
/// the nonlinear monomials fold back to admitted products.
enum CertificateSum {
    Empty,
    Affine(AffineInequality),
    Nonlinear(CertificatePolynomial),
}

/// Why one accumulation step could not form, before it is attributed to the
/// written entry that caused it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CertificateStepFailure {
    Overflow,
    UseCapacity,
    Formation,
    InvalidFactor,
}

impl From<AffineCheckError> for CertificateStepFailure {
    fn from(error: AffineCheckError) -> Self {
        match error {
            AffineCheckError::ArithmeticOverflow => Self::Overflow,
            AffineCheckError::LimitExceeded(AffineCheckLimit::CertificatePremises) => {
                Self::UseCapacity
            }
            AffineCheckError::LimitExceeded(_) | AffineCheckError::CoefficientMismatch => {
                Self::Formation
            }
            AffineCheckError::InvalidCertificateFactor => Self::InvalidFactor,
        }
    }
}

impl From<PolynomialError> for CertificateStepFailure {
    fn from(error: PolynomialError) -> Self {
        match error {
            PolynomialError::ArithmeticOverflow => Self::Overflow,
            // A degree-three product means one written multiplicity scaled a
            // premise that already carried a nonlinear monomial; nothing in
            // [PRF-1] forms such a step, so it stops as a formation failure
            // rather than as a claim about the writer's factor.
            PolynomialError::DegreeExceeded | PolynomialError::LimitExceeded => Self::Formation,
        }
    }
}

/// What the fixed interval-product rule proved about one admitted
/// multiplication: the inclusive interval its four endpoint products bound,
/// and the affine consequences that proved the operand endpoints.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AffineProductInterval {
    minimum: i128,
    maximum: i128,
    consequences: Box<[DerivationId]>,
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
    callee: PreparedCallee,
    call: crate::NodePath,
    parents: Vec<DerivationId>,
    transfer_events: Vec<FlowEventId>,
    kills: Vec<KillEvent>,
}

/// Which callee one prepared call publishes from [CALL-6].
///
/// [ENT-3.S13]'s population is every callee whose declared relation list is
/// published data: a source `fn_decl` with a verified [FN-9] summary, and
/// every kernel-domain row [BLK-0], whose relations are declaration data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreparedCallee {
    Source(super::super::model::FunctionId),
    /// One row, by its `container_declaration_ordinal` [BLK-0].
    Kernel(u8),
}

/// Result of judging one expression in source evaluation order.
///
/// `reached` is independent of postcondition preparation: it says that every
/// acceptance-bearing judgment needed to produce this value succeeded, so the
/// expression may receive an admitted structural identity. A successfully
/// evaluated call may have no verified postcondition summary and therefore no
/// `prepared_call`.
struct ExpressionJudgment {
    prepared_call: Option<PreparedCall>,
    reached: bool,
}

#[derive(Clone, Debug)]
struct AvailablePostcondition {
    relation: RelationTemplate,
    variant: Option<crate::PreludeDeclarationId>,
    field: Option<crate::PreludeDeclarationId>,
    summary: VerifiedPostconditionSummary,
    discharged: bool,
}

/// The one payload-carrying variant of a nominal enum [MSR-3].
struct SolePayloadVariant {
    /// Its position in the declared variant list, which is what a
    /// `construct` names.
    index: u32,
    /// Its [GRAM-10] tag, which is what a `match` arm names.
    tag: u32,
    fields: Vec<super::super::model::CheckedField>,
}

/// [MSR-3] one payload placement's datums, held between the mint before the
/// `match` consumes its scrutinee and the establishment at the arm binder
/// that names the payload.
struct PayloadPlacement {
    /// The tag of the arm these datums reach; every other arm binds no
    /// payload of this enum.
    tag: u32,
    carried: Vec<(u32, MeasureCarry)>,
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
    commit_reached: bool,
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
        product_intervals: HashMap::new(),
        product_operands: HashSet::new(),
        product_atoms: HashMap::new(),
        handle_images: HashMap::new(),
        call_goals: Vec::new(),
        counted_derivations: Vec::new(),
        loop_invariants: Vec::new(),
        source_proofs: Vec::new(),
        joined_source_proofs: Vec::new(),
        invariant_targets: HashMap::new(),
        s7_derivations: Vec::new(),
        postconditions: Vec::new(),
        boolean_decompositions: Vec::new(),
        entry_images: Vec::new(),
        postcondition_entry_images: Vec::new(),
        affine_atoms: Vec::new(),
        measure_atoms: HashMap::new(),
        measure_terms_seen: Vec::new(),
        measure_terms_scanned: 0,
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
    // [MSR-3] the entry placement, before every other source: one immutable
    // datum per measure of a parameter any declared relation names, equal to
    // that measure at body entry.
    analyzer.establish_entry_datums(&mut state.facts);
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
    let (terms, measure_bounds) = analyzer.terms.into_inventory();
    let inventory = DerivationInventory {
        terms,
        measure_bounds,
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
    /// The interval [ENT-6]'s interval-product rule proved at each admitted
    /// non-constant multiplication, keyed by that operation's own node. The
    /// domain is judged while the initializer is walked and [ENT-3.S14]
    /// establishes at the binding the walk then reaches, so the measurement
    /// waits here between the two rather than being proved again.
    product_intervals: HashMap<crate::NodePath, AffineProductInterval>,
    /// Which exact multiplications discharged their [OP-2] domain over affine
    /// operand images, keyed by the operation's own node. Read once at the
    /// binding the walk then reaches, exactly as the interval above is. It
    /// records only that the domain held: which values the fold names is a
    /// separate question the binding answers.
    product_operands: HashSet<crate::NodePath>,
    /// What every admitted exact product equals, as value identities: the atom
    /// the multiplication bound, and the two operand atoms it is the product
    /// of.
    ///
    /// An `AffineTermId` names one immutable value, so this map needs no kill
    /// and no join: a write to the product or to an operand mints a new atom,
    /// which is simply absent here, while the old atoms keep denoting the old
    /// values and the recorded equality stays true. [PRF-1] reads it to fold a
    /// term-scaled premise's nonlinear monomials back to affine.
    product_atoms: HashMap<AffineTermId, (AffineTermId, AffineTermId)>,
    /// What each minted opaque handle stands for. An `AffineTermId` is one
    /// immutable value identity, so this needs no kill and no join, exactly as
    /// `product_atoms` does.
    handle_images: HashMap<AffineTermId, AffineForm>,
    call_goals: Vec<CallGoalOutcome>,
    counted_derivations: Vec<CountedDerivationSet>,
    loop_invariants: Vec<LoopInvariantOutcome>,
    source_proofs: Vec<SourceProofOutcome>,
    joined_source_proofs: Vec<JoinedSourceProofProvenance>,
    /// Canonical immutable target formed at each invariant declaration.
    ///
    /// This table is deliberately separate from flow availability. A named
    /// PRF-1 `use` must form its written certificate from the declaration's
    /// proposition even on a path where that proposition is unavailable;
    /// availability is checked later as an independent premise judgment.
    invariant_targets: HashMap<crate::DeclarationId, Result<AffineInequality, AffineCheckError>>,
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
    /// [MSR-4] one compiler-owned immutable affine atom per live measure
    /// term, minted once and never retargeted. It is not a source binding
    /// and has no written spelling; it exists so the automatic derivation of
    /// a numeric goal can range over measures.
    measure_atoms: HashMap<TermId, AffineForm>,
    /// Every measure term registered so far, and how much of the term
    /// registry the scan that found them has covered.
    measure_terms_seen: Vec<TermId>,
    measure_terms_scanned: usize,
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
        value_reached: bool,
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
            // [CALL-4] one term per declared result ordinal, in written order.
            let results = selected
                .values
                .iter()
                .map(|value| {
                    value
                        .as_ref()
                        .and_then(|value| self.postcondition_return_term(value))
                })
                .collect::<Vec<_>>();
            // [CALL-4] an ordinal whose destination is no [ENT-2] place
            // makes only the relations naming it unavailable, which is what a
            // measured result's value term is: the clause names its measure
            // and never the value.
            let Some(relation) =
                self.instantiate_postcondition_relation(postcondition, &results, &selected.values)
            else {
                continue;
            };
            // [MSR-4] the affine route over the relation's own instantiated
            // terms, which is what carries a measure operand: a measure has
            // an affine atom [MSR-4] and no result value image, so the datum
            // route below reaches it nowhere. The datum route stays for a
            // fragment result whose returned expression has a richer image
            // than its place.
            let affine_target = self
                .affine_relation_target(&relation, &states.affine)
                .or_else(|| {
                    affine_result.and_then(|result| {
                        self.postcondition_affine_target(postcondition, result, &states.affine)
                    })
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
            let unavailable = !value_reached
                || entry_images
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

    /// [MSR-4] the affine target of one already-instantiated ordering
    /// relation, read through the affine image of each of its two terms.
    ///
    /// A measure term, a measure datum and an integer binding each have an
    /// image, so this reaches every relation whose operands the affine domain
    /// carries — which is what a clause over a run's measures is.
    fn affine_relation_target(
        &mut self,
        relation: &Relation,
        state: &AffineFlowState,
    ) -> Option<AffineInequality> {
        let Relation::Bound { left, right, bound } = relation else {
            return None;
        };
        let left = self.affine_term_value(*left, state)?;
        let right = self.affine_term_value(*right, state)?;
        let mut check = AffineCheckState::new();
        let right = right.add(&AffineForm::constant(*bound), &mut check).ok()?;
        AffineInequality::from_forms(&left, &right, &mut check).ok()
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
            .map(|operand| {
                let form = self.postcondition_affine_datum(&operand.datum, result, state)?;
                form.add(
                    &AffineForm::constant(operand.displacement),
                    &mut AffineCheckState::new(),
                )
                .ok()
            })
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
                ..
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
            | RelationDatum::Measure(..) => None,
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
        &mut self,
        expression: &GoalExpression,
        state: &AffineFlowState,
    ) -> Option<AffineInequality> {
        self.affine_signed_goal_ordering_target(expression, state, GoalSign::Positive)
    }

    /// Converts either truth sign of one callable-boundary ordering leaf to
    /// its unique affine inequality. Boolean composition uses this same leaf
    /// normalization instead of adding a call-specific affine fallback.
    fn affine_signed_goal_ordering_target(
        &mut self,
        expression: &GoalExpression,
        state: &AffineFlowState,
        sign: GoalSign,
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
        let (left, right, strict) = match sign {
            GoalSign::Positive => (left, right, strict),
            // Integer order is total: not(left <= right) is right < left,
            // while not(left < right) is right <= left.
            GoalSign::Negative => (right, left, !strict),
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
        &mut self,
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
            // [MSR-4] a measure term's image in the affine domain is its own
            // compiler-owned atom, exactly as it is for a term the flow
            // already carries. Without this a goal over a measure reaches
            // only the L0 route, and every filling loop's row requirement —
            // `room_of(built) > 0_u64` under `room_of(built) + at >= n` — is
            // unproved for want of the domain the rule names.
            GoalExpression::Operation {
                row:
                    GoalOperation::ArrayMeasure { .. }
                    | GoalOperation::BufferMeasure { .. }
                    | GoalOperation::SliceMeasure { .. }
                    | GoalOperation::ContainerMeasure { .. },
                ..
            } => {
                let term = self.goal_operand(expression)?;
                Some(self.measure_atom(term))
            }
            GoalExpression::Datum(
                GoalDatum::Parameter { .. } | GoalDatum::EvaluatedValue { .. },
            )
            | GoalExpression::Datum(GoalDatum::NamedConst { .. })
            | GoalExpression::Datum(GoalDatum::Place { .. })
            | GoalExpression::Operation { .. } => None,
        }
    }

    fn instantiate_postcondition_relation(
        &mut self,
        postcondition: &CheckedPostcondition,
        results: &[Option<TermId>],
        returns: &[Option<PostconditionReturnDatum>],
    ) -> Option<Relation> {
        let operands = postcondition
            .relation
            .operands
            .iter()
            .map(|operand| {
                Some((
                    self.postcondition_relation_term(&operand.datum, results, returns)?,
                    operand.displacement,
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        let [first, second] = operands.as_slice() else {
            return None;
        };
        // [FN-9] each side's displacement folds into the one constant a
        // difference bound carries: `l + a <cmp> r + b` is
        // `l - r <cmp> b - a`.
        let gap = second.1.checked_sub(first.1)?;
        match postcondition.relation.normalized {
            NormalizedRelation::Equal => Some(Relation::Equal {
                left: first.0,
                right: second.0,
                difference: gap,
            }),
            NormalizedRelation::NotEqual => Some(if first.0 <= second.0 {
                Relation::Distinct {
                    left: first.0,
                    right: second.0,
                    difference: gap,
                }
            } else {
                Relation::Distinct {
                    left: second.0,
                    right: first.0,
                    difference: gap.checked_neg()?,
                }
            }),
            NormalizedRelation::UpperBound {
                left,
                right,
                strict,
            } => {
                let lower = *operands.get(left as usize)?;
                let upper = *operands.get(right as usize)?;
                Some(Relation::Bound {
                    left: lower.0,
                    right: upper.0,
                    bound: upper
                        .1
                        .checked_sub(lower.1)?
                        .checked_sub(i128::from(strict))?,
                })
            }
        }
    }

    fn postcondition_relation_term(
        &mut self,
        datum: &RelationDatum,
        results: &[Option<TermId>],
        returns: &[Option<PostconditionReturnDatum>],
    ) -> Option<TermId> {
        match datum {
            // [CALL-4] the datum names one declared result ordinal, and the
            // destination supplies that ordinal's term.
            RelationDatum::Result { ordinal, .. } => *results.get(*ordinal as usize)?,
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
            RelationDatum::Measure(measure, place) => match place.root {
                // [MSR-3] an `own` or shared-borrow parameter's measure in an
                // `ensures` denotes that parameter's entry datum, which the
                // entry placement minted and which nothing kills. The live
                // term is not read here: a body that writes the parameter
                // back still means the entry value.
                PostconditionPlaceRoot::Parameter { ordinal } => {
                    let kind = Self::entry_datum_kind(ordinal, &place.projections, *measure);
                    if let Some(datum) = self.terms.interned(&kind) {
                        return Some(datum);
                    }
                    let binding = self.function.parameters.get(ordinal as usize)?.binding;
                    self.postcondition_measure_term(
                        *measure,
                        PlaceRoot::Binding(binding),
                        &place.projections,
                        place.ty,
                    )
                }
                // [CALL-4] a measure over a result place is instantiated at
                // that ordinal's own destination: at an exit, the place the
                // selected return hands back.
                PostconditionPlaceRoot::Result { ordinal } => {
                    let datum = returns.get(ordinal as usize)?.as_ref()?;
                    let PostconditionReturnDatum::Place(place) = datum else {
                        return None;
                    };
                    let root = self.postcondition_return_place_root(place.root)?;
                    self.postcondition_measure_term(*measure, root, &place.projections, place.ty)
                }
            },
        }
    }

    fn postcondition_return_term(&mut self, datum: &PostconditionReturnDatum) -> Option<TermId> {
        match datum {
            PostconditionReturnDatum::Place(place) => self.postcondition_return_place_term(place),
            PostconditionReturnDatum::Literal { value, .. } => {
                self.postcondition_constant_term(value)
            }
            PostconditionReturnDatum::Measure(measure, place) => {
                let root = self.postcondition_return_place_root(place.root)?;
                self.postcondition_measure_term(*measure, root, &place.projections, place.ty)
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
        if let CheckedValue::ConstGeneric { declaration, .. } = value {
            return Some(self.terms.intern(TermKind::ConstParameter(*declaration)));
        }
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
                GoalProjection::Subscript(offset) => PlaceProjection::Subscript(*offset),
            })
            .collect::<Vec<_>>();
        let path = ProjectedPlaceTerm { root, projections };
        let kind = legacy_place(&path).map_or_else(
            || TermKind::ProjectedPlace(path, fragment),
            |place| TermKind::Place(place, fragment),
        );
        Some(self.terms.intern(kind))
    }

    fn postcondition_measure_term(
        &mut self,
        measure: CheckedMeasure,
        root: PlaceRoot,
        projections: &[GoalProjection],
        ty: CheckedType,
    ) -> Option<TermId> {
        let projections = projections
            .iter()
            .map(|projection| match projection {
                GoalProjection::Field(field) => PlaceProjection::Field(*field),
                GoalProjection::Deref => PlaceProjection::Deref,
                GoalProjection::Subscript(offset) => PlaceProjection::Subscript(*offset),
            })
            .collect::<Vec<_>>();
        let measured = measured_kind(ty)?;
        // [MSR-2] the written constant a cell the table fixes as the type's
        // own reads: an `array`'s length, a `FixedVector`'s capacity, and an
        // `Arena`'s byte extent.
        let array_length = type_constant(ty);
        Some(self.measure_term(
            measure,
            ProjectedPlaceTerm { root, projections },
            measured,
            array_length,
        ))
    }

    /// The one former of every [MSR-1] measure term.
    ///
    /// Every measure of one place is formed together, because [MSR-2]'s
    /// standing facts relate them to each other: the value the table fixes
    /// for a cell, the equality of a table cell to another measure, and the
    /// orderings `len_of(P) <= cap_of(P)` and `head_of(P) <= cap_of(P)`. A site that
    /// names only one measure still needs the others to exist for those
    /// facts to have terms to relate, and all four have empty support beyond
    /// P's own, so forming them together costs nothing a program can observe.
    fn measure_term(
        &mut self,
        measure: CheckedMeasure,
        path: ProjectedPlaceTerm,
        measured: MeasuredKind,
        array_length: Option<CheckedConst>,
    ) -> TermId {
        let extent = self.intern_measure(CheckedMeasure::Length, &path);
        // [MSR-1]'s table, read once per cell.
        for cell_measure in [
            CheckedMeasure::Length,
            CheckedMeasure::Capacity,
            CheckedMeasure::Room,
            CheckedMeasure::Head,
        ] {
            let term = self.intern_measure(cell_measure, &path);
            let bound = match cell_measure.cell(measured) {
                MeasureCell::ExactConstant(value) => {
                    Some(MeasureBound::Constant(i128::from(value)))
                }
                MeasureCell::ExactExtent => match array_length {
                    Some(CheckedConst::Value(value)) => {
                        Some(MeasureBound::Constant(i128::from(value)))
                    }
                    Some(CheckedConst::Parameter(declaration)) => Some(MeasureBound::Equal(
                        self.terms.intern(TermKind::ConstParameter(declaration)),
                    )),
                    // A symbolic derived length has no [ENT-2] term form; the
                    // concrete instance, whose length is a value, restates the
                    // constant bound.
                    Some(CheckedConst::Derived(_)) => None,
                    // A runtime extent: `cap` is equal to it, `len` is it.
                    None => (cell_measure != CheckedMeasure::Length)
                        .then_some(MeasureBound::Equal(extent)),
                },
                // [MSR-2]: a measure the table fixes as the type's own
                // written constant is a standing fact with empty support; a
                // run's `cap` is that constant and a `Vector`'s is not.
                MeasureCell::ExactTypeConstant => match array_length {
                    Some(CheckedConst::Value(value)) => {
                        Some(MeasureBound::Constant(i128::from(value)))
                    }
                    Some(CheckedConst::Parameter(declaration)) => Some(MeasureBound::Equal(
                        self.terms.intern(TermKind::ConstParameter(declaration)),
                    )),
                    Some(CheckedConst::Derived(_)) | None => None,
                },
                // An independent runtime quantity of the value's own
                // descriptor: the standing facts [MSR-2] already publishes
                // relate it to the others, and it carries no bound of its own.
                MeasureCell::ExactRuntime | MeasureCell::Bounded | MeasureCell::Absent => None,
            };
            if let Some(bound) = bound {
                self.terms.set_measure_bound(term, bound);
            }
        }
        self.intern_measure(measure, &path)
    }

    fn intern_measure(&mut self, measure: CheckedMeasure, path: &ProjectedPlaceTerm) -> TermId {
        let kind = legacy_place(path).map_or_else(
            || TermKind::ProjectedMeasure(measure, path.clone()),
            |place| TermKind::Measure(measure, place),
        );
        self.terms.intern(kind)
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
            | CheckedExpression::BufferMeasure { root, .. } => {
                self.append_holder_chain(root.binding, holders);
            }
            CheckedExpression::SliceMeasure { root, .. } => {
                self.append_holder_chain(root.binding, holders);
            }
            CheckedExpression::ArrayMeasure {
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
                    measure: None,
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
            TermKind::Place(place, _) | TermKind::Measure(_, place) => {
                if place.deref
                    && let PlaceRoot::Binding(binding) = place.root
                {
                    let _ = self.resolve_deref_with_holders(binding, 0, &mut holders);
                }
            }
            TermKind::ProjectedPlace(place, _) | TermKind::ProjectedMeasure(_, place) => {
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
                            PlaceProjection::Subscript(offset) => {
                                GoalProjection::Subscript(*offset)
                            }
                        })
                        .collect(),
                    measure: match self.terms.kind(term) {
                        TermKind::ProjectedMeasure(measure, _) => Some(*measure),
                        _ => None,
                    },
                };
                let (_, projected_holders) = self.resolve_goal_support(&support);
                holders.extend(projected_holders);
            }
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => {}
            TermKind::CountedCapture { .. }
            | TermKind::CommitValue { .. }
            | TermKind::CallDatum { .. }
            | TermKind::EntryDatum { .. }
            | TermKind::MeasureDatum { .. } => {}
        }
        holders
    }

    fn s12_transfer_event_kills_substitution(
        &self,
        substitution: &PostconditionCallSubstitution,
        event: &KillEvent,
    ) -> bool {
        // [MSR-3] a call datum contains no place and denotes the operand's
        // value at the pre-transfer point, so no event at or after the call
        // can invalidate a relation stated over it.
        if substitution.datum {
            return false;
        }
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
                    path: Vec::new(),
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
            | GoalDatum::EvaluatedValue { .. }
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
        measure: Option<CheckedMeasure>,
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
        if projections.is_empty() && measure.is_none() {
            return (actual.ty() == ty)
                .then(|| self.goal_operand(actual))
                .flatten();
        }
        let (root, projections) = self.call_parameter_place(actual, projections)?;
        if let Some(measure) = measure {
            self.postcondition_measure_term(measure, root, &projections, ty)
        } else {
            self.postcondition_place_term(root, &projections, ty)
        }
    }

    /// [MSR-3] the identity of one call datum: the call, the formal ordinal,
    /// the operand's ordered projections, and which [MSR-1] measure of the
    /// operand the datum denotes, if any.
    fn call_datum_kind(
        call: &crate::NodePath,
        formal: u32,
        projections: &[GoalProjection],
        measure: Option<CheckedMeasure>,
        ty: super::super::model::IntegerType,
    ) -> TermKind {
        TermKind::CallDatum {
            call_path: call.components().to_vec(),
            formal,
            projections: projections
                .iter()
                .map(|projection| match projection {
                    GoalProjection::Deref => CallDatumProjection::Deref,
                    GoalProjection::Field(field) => CallDatumProjection::Field(*field),
                    GoalProjection::Subscript(offset) => CallDatumProjection::Subscript(*offset),
                })
                .collect(),
            measure,
            ty,
        }
    }

    /// [ENT-3.S13, MSR-3] mints, at one call's pre-transfer point, the call
    /// datum of every `own` operand any declared relation of the resolved
    /// callee names, and establishes it equal to that operand's pre-transfer
    /// term.
    ///
    /// The equality is stated here, before the call's own consumes and
    /// kills, so [ENT-5]'s pre-kill closure carries the datum's consequences
    /// across them. The datum itself contains no place, so nothing kills it:
    /// that is exactly why a relation naming a consumed `own` operand's
    /// measure means what it reads as at the caller, and why the consume the
    /// same statement performs cannot delete it.
    fn establish_call_datums(
        &mut self,
        function: super::super::model::FunctionId,
        call: &crate::NodePath,
        goal_arguments: &[GoalExpression],
        state: &mut FactState,
    ) {
        let Some(callee) = self.context.callee(function) else {
            return;
        };
        let parameter_modes = callee.parameter_modes.clone();
        let mut operands: Vec<(
            u32,
            Vec<GoalProjection>,
            Option<CheckedMeasure>,
            CheckedType,
        )> = Vec::new();
        for available in self.available_postconditions(function) {
            for operand in &available.relation.operands {
                let datum = &operand.datum;
                match datum {
                    RelationDatum::Parameter {
                        ordinal,
                        projections,
                        ty,
                    } => operands.push((*ordinal, projections.clone(), None, *ty)),
                    // A result-rooted measure names no operand and mints no
                    // call datum [CALL-4].
                    RelationDatum::Measure(measure, place) => {
                        if let PostconditionPlaceRoot::Parameter { ordinal } = place.root {
                            operands.push((
                                ordinal,
                                place.projections.clone(),
                                Some(*measure),
                                place.ty,
                            ));
                        }
                    }
                    RelationDatum::Result { .. }
                    | RelationDatum::NamedConst { .. }
                    | RelationDatum::Literal { .. } => {}
                }
            }
        }
        let event = self.proof_event(FlowEventKind::S13, Some(call));
        for (ordinal, projections, measure, ty) in operands {
            if parameter_modes.get(ordinal as usize) != Some(&CheckedMode::Own) {
                continue;
            }
            let Some(datum_type) = (if measure.is_some() {
                Some(super::super::model::IntegerType::U64)
            } else {
                fragment_type(ty)
            }) else {
                continue;
            };
            let kind = Self::call_datum_kind(call, ordinal, &projections, measure, datum_type);
            if self.terms.interned(&kind).is_some() {
                continue;
            }
            let Some(actual) = goal_arguments.get(ordinal as usize) else {
                continue;
            };
            let Some(term) =
                self.call_parameter_term(actual, &projections, ty, measure, CheckedMode::Own)
            else {
                continue;
            };
            // A datum denotes the operand's value at this point. When the
            // pre-transfer term is already immutable and has empty support,
            // it denotes exactly that and nothing can retarget it, so the
            // datum is that term: minting a second one would add an
            // indirection to every derivation and hide the writer's own
            // constant behind it in a diagnostic.
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

    /// Whether one term is already immutable with empty support, so that no
    /// [ENT-5] event can change what it denotes [ENT-2, MSR-3].
    fn immortal_term(&self, term: TermId) -> bool {
        matches!(
            self.terms.kind(term),
            TermKind::Zero
                | TermKind::Constant(_)
                | TermKind::ConstParameter(_)
                | TermKind::CountedCapture { .. }
                | TermKind::CommitValue { .. }
                | TermKind::CallDatum { .. }
        )
    }

    /// The already-minted call datum of one `own` operand, when this call
    /// established one [MSR-3].
    fn interned_call_datum(
        &self,
        call: &crate::NodePath,
        formal: u32,
        projections: &[GoalProjection],
        measure: Option<CheckedMeasure>,
        ty: CheckedType,
    ) -> Option<TermId> {
        let datum_type = if measure.is_some() {
            super::super::model::IntegerType::U64
        } else {
            fragment_type(ty)?
        };
        self.terms.interned(&Self::call_datum_kind(
            call,
            formal,
            projections,
            measure,
            datum_type,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn instantiate_call_postcondition_relation(
        &mut self,
        function: super::super::model::FunctionId,
        call_path: &crate::NodePath,
        template: &RelationTemplate,
        checked_arguments: &[CheckedExpression],
        arguments: &[GoalExpression],
        results: &[Option<TermId>],
        result_places: &[Option<(PlaceRoot, Vec<GoalProjection>, CheckedType)>],
    ) -> Option<InstantiatedPostcondition> {
        let parameter_modes = self.context.callee(function)?.parameter_modes.clone();
        let mut substitutions = Vec::new();
        let mut operands = Vec::with_capacity(template.operands.len());
        for (operand, term_operand) in template.operands.iter().enumerate() {
            let datum = &term_operand.datum;
            let (term, formal) = match datum {
                // [CALL-4] the destination supplies one term per declared
                // result ordinal; an ordinal with none makes only this
                // relation unavailable.
                RelationDatum::Result { ordinal, .. } => {
                    ((*results.get(*ordinal as usize)?)?, None)
                }
                RelationDatum::Parameter {
                    ordinal,
                    projections,
                    ty,
                } => match self.interned_call_datum(call_path, *ordinal, projections, None, *ty) {
                    // [MSR-3] an `own` operand denotes this call's call
                    // datum, which has empty support.
                    Some(datum) => (datum, Some((*ordinal, true))),
                    None => (
                        self.call_parameter_term(
                            arguments.get(*ordinal as usize)?,
                            projections,
                            *ty,
                            None,
                            *parameter_modes.get(*ordinal as usize)?,
                        )?,
                        Some((*ordinal, false)),
                    ),
                },
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
                RelationDatum::Measure(measure, place) => match place.root {
                    PostconditionPlaceRoot::Parameter { ordinal } => {
                        match self.interned_call_datum(
                            call_path,
                            ordinal,
                            &place.projections,
                            Some(*measure),
                            place.ty,
                        ) {
                            Some(datum) => (datum, Some((ordinal, true))),
                            None => (
                                self.call_parameter_term(
                                    arguments.get(ordinal as usize)?,
                                    &place.projections,
                                    place.ty,
                                    Some(*measure),
                                    *parameter_modes.get(ordinal as usize)?,
                                )?,
                                Some((ordinal, false)),
                            ),
                        }
                    }
                    // [CALL-4] the destination supplies one place per
                    // declared result ordinal, and this operand is that
                    // place's measure rather than its value.
                    PostconditionPlaceRoot::Result { ordinal } => {
                        let (root, projections, ty) =
                            result_places.get(ordinal as usize)?.as_ref()?;
                        (
                            self.postcondition_measure_term(*measure, *root, projections, *ty)?,
                            None,
                        )
                    }
                },
            };
            if let Some((formal, datum)) = formal {
                substitutions.push(PostconditionCallSubstitution {
                    operand: u32::try_from(operand)
                        .expect("postcondition operands exceed the u32 identity space"),
                    formal,
                    term,
                    transfer_holders: self.call_argument_holder_chain(
                        checked_arguments.get(formal as usize)?,
                        arguments.get(formal as usize)?,
                    ),
                    datum,
                });
            }
            operands.push((term, term_operand.displacement));
        }
        let [first, second] = operands.as_slice() else {
            return None;
        };
        // [FN-9] each side's displacement folds into the one constant a
        // difference bound carries.
        let gap = second.1.checked_sub(first.1)?;
        let relation = match template.normalized {
            NormalizedRelation::Equal => Relation::Equal {
                left: first.0,
                right: second.0,
                difference: gap,
            },
            NormalizedRelation::NotEqual => {
                if first.0 <= second.0 {
                    Relation::Distinct {
                        left: first.0,
                        right: second.0,
                        difference: gap,
                    }
                } else {
                    Relation::Distinct {
                        left: second.0,
                        right: first.0,
                        difference: gap.checked_neg()?,
                    }
                }
            }
            NormalizedRelation::UpperBound {
                left,
                right,
                strict,
            } => {
                let lower = *operands.get(left as usize)?;
                let upper = *operands.get(right as usize)?;
                Relation::Bound {
                    left: lower.0,
                    right: upper.0,
                    bound: upper
                        .1
                        .checked_sub(lower.1)?
                        .checked_sub(i128::from(strict))?,
                }
            }
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
                summary: super::RelationProvenance::Verified(available.summary.clone()),
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
        // [CALL-6] a kernel-domain row publishes at exactly the same
        // destination, from its own declared relation list [BLK-0].
        if matches!(prepared.callee, PreparedCallee::Kernel(_)) {
            let destinations = vec![Some((binding, Vec::new(), value.ty()))];
            self.establish_kernel_relations(
                statement,
                &destinations,
                value,
                prepared,
                &mut states.facts,
            );
            return;
        }
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
        if PreparedCallee::Source(*function) != prepared.callee || *call != prepared.call {
            return;
        }
        // [CALL-4] the destination is one term when the ordinal's value is an
        // [ENT-2] term, and is always the place a measure over that ordinal is
        // taken over. A measured result has the second and not the first.
        let result_term = fragment_type(*result)
            .and_then(|_| self.postcondition_place_term(PlaceRoot::Binding(binding), &[], *result));
        let result_place = Some((PlaceRoot::Binding(binding), Vec::new(), *result));
        if result_term.is_none() && measured_kind(*result).is_none() {
            return;
        }
        for available in self.available_postconditions(*function) {
            if available.variant.is_some() {
                continue;
            }
            let Some(instantiated) = self.instantiate_call_postcondition_relation(
                *function,
                call,
                &available.relation,
                arguments,
                goal_arguments,
                &[result_term],
                std::slice::from_ref(&result_place),
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

    /// [ENT-3.S12, CALL-4] establishes, at each destination of a binder or
    /// target list, every published relation naming that destination's result
    /// ordinal.
    ///
    /// The destinations are given in written order, so destination i is
    /// result ordinal i; `extra_kills` are the events the same statement's
    /// commits contribute, which a substitution must survive exactly as it
    /// must survive the call's own.
    fn establish_result_list_destinations(
        &mut self,
        statement: &crate::NodePath,
        destinations: &[Option<(BindingId, Vec<GoalProjection>, CheckedType)>],
        value: &CheckedExpression,
        prepared: &PreparedCall,
        extra_kills: &[KillEvent],
        states: &mut ProofFlowState,
    ) {
        if matches!(prepared.callee, PreparedCallee::Kernel(_)) {
            self.establish_kernel_relations(
                statement,
                destinations,
                value,
                prepared,
                &mut states.facts,
            );
            return;
        }
        let CheckedExpression::UserCall {
            function,
            call,
            arguments,
            goal_arguments,
            ..
        } = value
        else {
            return;
        };
        if PreparedCallee::Source(*function) != prepared.callee || *call != prepared.call {
            return;
        }
        // One term per result ordinal, in written order. A subscript place is
        // no [ENT-2] term and a non-fragment ordinal carries no relation
        // datum, so either leaves its ordinal without a term and makes only
        // the relations naming it unavailable.
        let mut result_terms = Vec::with_capacity(destinations.len());
        // [CALL-4] the same destination is also the place a measure over that
        // result ordinal is taken over, which a measured ordinal has and a
        // fragment-integer value term does not.
        let mut result_places = Vec::with_capacity(destinations.len());
        let mut anchor = None;
        for destination in destinations {
            let term = destination.as_ref().and_then(|(binding, fields, ty)| {
                fragment_type(*ty)?;
                let term = self.postcondition_place_term(PlaceRoot::Binding(*binding), fields, *ty);
                if term.is_some() && anchor.is_none() {
                    anchor = Some(*binding);
                }
                term
            });
            result_places.push(destination.as_ref().map(|(binding, fields, ty)| {
                if anchor.is_none() {
                    anchor = Some(*binding);
                }
                (PlaceRoot::Binding(*binding), fields.clone(), *ty)
            }));
            result_terms.push(term);
        }
        let Some(anchor) = anchor else {
            return;
        };
        for available in self.available_postconditions(*function) {
            // A variant-routed relation is restricted to its arm [CALL-6];
            // a binder or target list enters no arm.
            if available.variant.is_some() {
                continue;
            }
            let Some(instantiated) = self.instantiate_call_postcondition_relation(
                *function,
                call,
                &available.relation,
                arguments,
                goal_arguments,
                &result_terms,
                &result_places,
            ) else {
                continue;
            };
            if !self.s12_substitutions_survive(&instantiated.substitutions, &prepared.kills)
                || !self.s12_substitutions_survive(&instantiated.substitutions, extra_kills)
            {
                continue;
            }
            self.retain_direct_result(
                statement,
                anchor,
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
            .is_some_and(|(place, _)| place.overlaps(receiver))
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
        if PreparedCallee::Source(*function) != prepared.callee
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
            path: Vec::new(),
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
                    &prepared.call,
                    &available.relation,
                    arguments,
                    goal_arguments,
                    &[Some(result_term)],
                    &[],
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
        if PreparedCallee::Source(*function) != prepared.callee
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
                call,
                &available.relation,
                arguments,
                goal_arguments,
                &[Some(result_term)],
                &[],
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
            Relation::Equal {
                left,
                right,
                difference,
            } => Relation::Equal {
                left: replace(*left),
                right: replace(*right),
                difference: *difference,
            },
            Relation::Distinct {
                left,
                right,
                difference,
            } => {
                let (left, right) = (replace(*left), replace(*right));
                // Ordering the pair reverses the difference with it.
                if left <= right {
                    Relation::Distinct {
                        left,
                        right,
                        difference: *difference,
                    }
                } else {
                    Relation::Distinct {
                        left: right,
                        right: left,
                        difference: -difference,
                    }
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
            path: Vec::new(),
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
            CheckedSetTarget::RunIndex(target) => {
                return self.container_root_place(&target.root).overlaps(place);
            }
            // A view element store writes the origin's storage and not the
            // descriptor's [PROV-3], and the origin is not this place term's
            // root, so the descriptor place is what the term names.
            CheckedSetTarget::SliceIndex(target) => PlaceTerm {
                root: PlaceRoot::Binding(target.root.binding),
                deref: self.is_holder(target.root.binding),
                fields: Vec::new(),
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
            | CheckedStatement::DestructuringLet { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Dispose { value, .. }
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
            CheckedStatement::SetList {
                targets, values, ..
            } => {
                values
                    .expressions()
                    .iter()
                    .any(|value| self.expression_writes_place(value, place))
                    || targets
                        .iter()
                        .any(|target| self.set_target_writes_place(target, place))
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
            | CheckedStatement::DestructuringLet { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::SetList { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::Dispose { .. }
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
                let datum = match &operand.datum {
                    RelationDatum::Parameter {
                        ordinal,
                        projections,
                        ty,
                    } => Some((
                        PostconditionEntryImage {
                            parameter: *ordinal,
                            projections: projections.clone(),
                            measure: None,
                        },
                        *ty,
                    )),
                    RelationDatum::Measure(measure, place) => match place.root {
                        PostconditionPlaceRoot::Parameter { ordinal } => Some((
                            PostconditionEntryImage {
                                parameter: ordinal,
                                projections: place.projections.clone(),
                                measure: Some(*measure),
                            },
                            place.ty,
                        )),
                        // A result place is not a parameter entry image.
                        PostconditionPlaceRoot::Result { .. } => None,
                    },
                    RelationDatum::Result { .. }
                    | RelationDatum::NamedConst { .. }
                    | RelationDatum::Literal { .. } => None,
                };
                if let Some(datum) = datum {
                    let index = data
                        .iter()
                        .position(|existing: &(PostconditionEntryImage, CheckedType)| {
                            existing.0 == datum.0
                        })
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
            .map(|(datum, ty)| {
                let parameter = self
                    .function
                    .parameters
                    .get(datum.parameter as usize)
                    .expect("checked postcondition parameter ordinal must resolve");
                let support = GoalSupport {
                    root: parameter.binding,
                    projections: datum.projections.clone(),
                    measure: datum.measure,
                };
                let (place, holders) = self.resolve_goal_support(&support);
                EntryImageRecord {
                    datum,
                    ty,
                    place,
                    holders,
                }
            })
            .collect();
        self.postcondition_entry_images = relation_images;
    }

    /// [MSR-3] the entry placement: at body entry, per parameter of measured
    /// type and per measure any declared relation names, one compiler-owned
    /// immutable datum established equal to that measure.
    ///
    /// The datum contains no place, so no [ENT-5] event kills it. That is
    /// what makes an `ensures` naming an `own` parameter's measure denote the
    /// entry value even where the body writes that parameter back with a
    /// [LIV-2] `set`, and it is the callee-side half of the denotation
    /// [MSR-3]'s table gives the same operand at a caller.
    fn establish_entry_datums(&mut self, state: &mut FactState) {
        if self.entry_images.is_empty() {
            return;
        }
        let event = self.proof_event(FlowEventKind::Entry, None);
        for index in 0..self.entry_images.len() {
            let image = self.entry_images[index].datum.clone();
            let ty = self.entry_images[index].ty;
            let Some(measure) = image.measure else {
                continue;
            };
            let Some(parameter) = self.function.parameters.get(image.parameter as usize) else {
                continue;
            };
            let binding = parameter.binding;
            let Some(live) = self.postcondition_measure_term(
                measure,
                PlaceRoot::Binding(binding),
                &image.projections,
                ty,
            ) else {
                continue;
            };
            let datum = self.terms.intern(Self::entry_datum_kind(
                image.parameter,
                &image.projections,
                measure,
            ));
            self.adopt_measure_atom(datum, live);
            state.establish(
                &Relation::Equal {
                    left: datum,
                    right: live,
                    difference: 0,
                },
                &mut self.derivations,
                event,
            );
        }
    }

    /// [MSR-3] the identity of one entry datum: the formal ordinal, the
    /// operand's ordered projections, and which [MSR-1] measure of it the
    /// datum denotes.
    fn entry_datum_kind(
        formal: u32,
        projections: &[GoalProjection],
        measure: CheckedMeasure,
    ) -> TermKind {
        TermKind::EntryDatum {
            formal,
            projections: projections
                .iter()
                .map(|projection| match projection {
                    GoalProjection::Deref => CallDatumProjection::Deref,
                    GoalProjection::Field(field) => CallDatumProjection::Field(*field),
                    GoalProjection::Subscript(offset) => CallDatumProjection::Subscript(*offset),
                })
                .collect(),
            measure,
        }
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
            // Counted captures and commit values are immutable. A counted
            // capture dies with its construct-scope exit, handled separately
            // from source-place write/consume events; a commit value names one
            // evaluated value that no later event can change.
            TermKind::CountedCapture { .. }
            | TermKind::CommitValue { .. }
            | TermKind::CallDatum { .. }
            | TermKind::EntryDatum { .. }
            | TermKind::MeasureDatum { .. } => false,
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
            // [MSR-2] a measure term's support is its place's DESCRIPTOR
            // storage, which is the resolved place of P itself and not of
            // P's root: a write to a sibling field of P overlaps neither.
            TermKind::Measure(_, place) => {
                let support = self.resolve(&place);
                self.event_kills_measure(&support, place.root, event)
                    || Self::event_kills_offset_support(&support, event)
            }
            TermKind::ProjectedMeasure(_, place) => {
                let support = self.resolve_projected(&place);
                self.event_kills_measure(&support, place.root, event)
                    || Self::event_kills_offset_support(&support, event)
            }
        }
    }

    /// [MSR-2] whether one event kills a measure of `support`.
    ///
    /// A write at an element position of P carries the written element's own
    /// place, `P[i]`, so it overlaps the descriptor storage of `P[i]` and
    /// none of P's own: it kills every measure of `P[i]` and no measure of P.
    /// A write of a whole value writes everything under it, so it kills the
    /// measures of every place it reaches. Two subscripts of one base are the
    /// same step of that reach unless their offsets are provably distinct
    /// [OWN-7].
    fn event_kills_measure(
        &self,
        support: &ResolvedPlace,
        root: PlaceRoot,
        event: &KillEvent,
    ) -> bool {
        match event {
            KillEvent::Write {
                place: written,
                element: true,
                ..
            }
            | KillEvent::EntryImageHolderWrite {
                place: written,
                element: true,
                ..
            } => written.is_prefix_of(support),
            KillEvent::Write {
                place: written,
                element: false,
                ..
            }
            | KillEvent::EntryImageHolderWrite {
                place: written,
                element: false,
                ..
            } => support.overlaps(written),
            KillEvent::Consume { binding, .. } => root == PlaceRoot::Binding(*binding),
            KillEvent::EntryImageHolderConsume { .. } => false,
        }
    }

    /// [ENT-5] whether one event writes or consumes a binding an offset
    /// occurring in `support` reads.
    ///
    /// The support of a measure term over P contains the support of every
    /// offset occurring in P, so a write to that offset's own binding kills
    /// the measure at every level it occurs in.
    fn event_kills_offset_support(support: &ResolvedPlace, event: &KillEvent) -> bool {
        support.path.iter().any(|step| {
            let PlaceStep::Subscript(offset) = step else {
                return false;
            };
            let Some(binding) = offset.support() else {
                return false;
            };
            let read = ResolvedPlace::binding(binding);
            match event {
                KillEvent::Write { place: written, .. }
                | KillEvent::EntryImageHolderWrite { place: written, .. } => read.overlaps(written),
                KillEvent::Consume {
                    binding: consumed, ..
                } => binding == *consumed,
                KillEvent::EntryImageHolderConsume { .. } => false,
            }
        })
    }

    /// Whether leaving the scopes of `exited` kills a fact supported by
    /// `term`: the support contains every tracked place's root binding and
    /// every holder read through, which is the spelling root here.
    fn scope_kills_term(&self, term: TermId, exited: &HashSet<BindingId>) -> bool {
        match self.terms.kind(term) {
            TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => false,
            TermKind::CountedCapture { .. }
            | TermKind::CommitValue { .. }
            | TermKind::CallDatum { .. }
            | TermKind::EntryDatum { .. }
            | TermKind::MeasureDatum { .. } => false,
            TermKind::Place(place, _) | TermKind::Measure(_, place) => match place.root {
                PlaceRoot::Binding(binding) => exited.contains(&binding),
                PlaceRoot::Constant(_) => false,
            },
            // [ENT-5] the support of every offset occurring in the place is
            // part of the term's own support, so a term dies with the scope
            // of an offset's binding exactly as it dies with its root's.
            TermKind::ProjectedPlace(place, _) | TermKind::ProjectedMeasure(_, place) => {
                let rooted = match place.root {
                    PlaceRoot::Binding(binding) => exited.contains(&binding),
                    PlaceRoot::Constant(_) => false,
                };
                rooted
                    || place.projections.iter().any(|projection| {
                        matches!(projection, PlaceProjection::Subscript(offset)
                            if offset.support().is_some_and(|binding| exited.contains(&binding)))
                    })
            }
        }
    }

    fn resolve_goal_support(&self, support: &GoalSupport) -> (ResolvedPlace, Vec<BindingId>) {
        let mut resolved = ResolvedPlace {
            root: PlaceRoot::Binding(support.root),
            path: Vec::new(),
        };
        let mut holders = Vec::new();
        for projection in &support.projections {
            match projection {
                GoalProjection::Field(field) => resolved.path.push(PlaceStep::Field(*field)),
                GoalProjection::Subscript(offset) => {
                    resolved.path.push(PlaceStep::Subscript(*offset));
                }
                GoalProjection::Deref => {
                    if resolved.path.is_empty()
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
                // [MSR-2] a write at an element position carries the written
                // element's own place, `P[i]`, so it reaches the descriptor
                // storage of `P[i]` and none of P's own. A measure goal over
                // a place the written place is a prefix of therefore dies
                // and one over P does not, which is the same sentence
                // `event_kills_measure` reads for an L0 measure term. The
                // blanket "an element write kills no measure goal" this
                // replaces was [ENT-5]'s element-position carve-out, which
                // was only ever true of a table with no measured element
                // type.
                KillEvent::Write {
                    place: written,
                    element: true,
                    ..
                }
                | KillEvent::EntryImageHolderWrite {
                    place: written,
                    element: true,
                    ..
                } if support.measure.is_some() => written.is_prefix_of(&place),
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
                    path: Vec::new(),
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

    /// Materializes the complete [ENT-4] closure before an event-kill batch.
    ///
    /// The existing event predicates remain the sole authority for which
    /// terms, goals, and origins die. Materialization only makes every
    /// survivor-to-survivor consequence independently live before one of its
    /// supporting endpoints disappears.
    fn materialize_before_event_kill(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        materialize_closure_before_kill(state, &self.terms, &self.goals, &mut self.derivations);
    }

    fn apply_kills_one(&mut self, state: &mut FactState, events: &[KillEvent]) {
        if events.is_empty() {
            return;
        }
        self.materialize_before_event_kill(state, events);
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
        self.apply_affine_kills(&mut states.affine, events);
        self.invalidate_entry_images(states, events, None);
    }

    fn event_kills_entry_image(&self, image: &EntryImageRecord, event: &KillEvent) -> bool {
        match event {
            KillEvent::Write { element: true, .. }
            | KillEvent::EntryImageHolderWrite { element: true, .. }
                if image.datum.measure.is_some() =>
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
                    // [MSR-3] a measure operand denotes the entry datum, and
                    // no [ENT-5] event kills a datum. Only a non-measure
                    // operand still reads the live place and can lose it.
                    (image.datum.measure.is_none()
                        && states.entry_images[index].is_none()
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

    fn exit_affine_scopes_to(&mut self, state: &mut AffineFlowState, depth: usize) {
        let exited = self
            .scopes
            .iter()
            .skip(depth)
            .flatten()
            .copied()
            .collect::<HashSet<_>>();
        state.values.retain(|binding, _| !exited.contains(binding));
        // A measure of a place rooted in an exited binding has no image past
        // that edge, exactly as the binding itself has none, so the next
        // occurrence of that term mints a fresh atom [MSR-2].
        let stale: Vec<TermId> = self
            .measure_atoms
            .keys()
            .copied()
            .filter(|term| {
                self.measure_term_root(*term)
                    .is_some_and(|binding| exited.contains(&binding))
            })
            .collect();
        for term in stale {
            self.measure_atoms.remove(&term);
        }
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

    fn remove_active_loop_invariants(
        state: &mut AffineFlowState,
        loop_id: CheckedLoopId,
        declarations: &[crate::DeclarationId],
    ) {
        state
            .facts
            .retain(|fact| !fact.active_loops.contains(&loop_id));
        for declaration in declarations {
            state.published_invariants.remove(declaration);
        }
    }

    /// Returns the one deterministic premise traversal used by every affine
    /// consumer: insertion order with later occurrences of the same canonical
    /// inequality removed, regardless of evidence category.
    fn canonical_affine_facts(facts: &[ActiveAffineFact]) -> Vec<&ActiveAffineFact> {
        let mut seen = HashSet::new();
        facts
            .iter()
            .filter(|fact| seen.insert(fact.inequality.clone()))
            .collect()
    }

    fn affine_facts(state: &AffineFlowState) -> Vec<ActiveAffineFact> {
        Self::canonical_affine_facts(&state.facts)
            .into_iter()
            .cloned()
            .collect()
    }

    fn affine_fact_uses_only_outer_values(
        &self,
        inequality: &AffineInequality,
        state: &AffineFlowState,
        binder: BindingId,
    ) -> bool {
        let mut live_terms = state
            .values
            .iter()
            .filter(|(binding, _)| **binding != binder)
            .flat_map(|(_, value)| value.terms().iter().map(|coefficient| coefficient.term()))
            .collect::<HashSet<_>>();
        // [MSR-1, MSR-4] a measure of a place live at the continuation is an
        // outer value exactly as an integer binding is: it has one atom, that
        // atom is retargeted only by the events that kill the term [MSR-2],
        // and a conclusion over it therefore says the same thing after the
        // loop that it said inside. Without this every filling loop's exit
        // exports nothing and the `ensures` its body was written for is
        // unproved at the return.
        live_terms.extend(
            self.measure_atoms
                .values()
                .flat_map(|value| value.terms().iter().map(|coefficient| coefficient.term())),
        );
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
            .map(|frame| {
                (
                    frame.id,
                    frame.capture_path.clone(),
                    frame.invariant_declarations.clone(),
                )
            })
            .collect::<Vec<_>>();
        for (loop_id, path, declarations) in loops {
            if let Some(path) = path {
                self.exit_counted_capture_scope(states, &path);
            }
            Self::remove_active_loop_invariants(&mut states.affine, loop_id, &declarations);
        }
    }

    // ------------------------------------------------------------------
    // Terms and relations from checked expressions
    // ------------------------------------------------------------------

    /// Reads an expression as a term or constant [ENT-2]; anything else is
    /// no operand and establishes or derives nothing.
    fn read_operand(&mut self, expression: &CheckedExpression) -> Option<TermId> {
        match expression {
            // [MSR-6] a const generic read as a value is the symbolic
            // constant term [ENT-2] clause (c) fixes; a concrete [FN-2]
            // instance has already folded it to an integer constant.
            CheckedExpression::Constant(CheckedValue::ConstGeneric { declaration, .. }) => {
                return Some(self.terms.intern(TermKind::ConstParameter(*declaration)));
            }
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
                                PlaceProjection::Deref | PlaceProjection::Subscript(_) => None,
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
        sources::comparison_relation(*operation, left, right, 0)
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
        self.goal_expression(expression, false)
    }

    /// Converts a value whose nested obligations have already been discharged
    /// to its exact stable proof expression. In addition to the pure/total
    /// direct subset, this admits an exact integer result or indexed element
    /// only after `judge_expression` has checked that nested partial operation.
    fn admitted_value_goal_expression(
        &self,
        expression: &CheckedExpression,
    ) -> Option<GoalExpression> {
        self.goal_expression(expression, true)
    }

    /// Replaces one occurrence-local FN-8 actual with the same admitted
    /// structural value used by the rest of ENT-2. The caller invokes this
    /// only after every obligation in every actual expression has succeeded.
    /// A projection that the admitted structural tree cannot represent keeps
    /// the occurrence-local value instead of inventing a different identity.
    fn admitted_call_goal_expression(
        &self,
        expression: &GoalExpression,
        call: &crate::NodePath,
        arguments: &[Option<GoalExpression>],
    ) -> GoalExpression {
        match expression {
            GoalExpression::Datum(
                original @ GoalDatum::EvaluatedValue {
                    occurrence:
                        EvaluatedValueOccurrence::CallArgument {
                            call: occurrence_call,
                            argument,
                        },
                    captured_type,
                    projections,
                    ty,
                    ..
                },
            ) if occurrence_call == call => {
                let Some(mut admitted) = usize::try_from(*argument)
                    .ok()
                    .and_then(|index| arguments.get(index))
                    .and_then(Option::as_ref)
                    .filter(|argument| argument.ty() == *captured_type)
                    .cloned()
                else {
                    return GoalExpression::Datum(original.clone());
                };
                for projection in projections {
                    let Some(projected) = admitted.with_projection(*projection, *ty) else {
                        return GoalExpression::Datum(original.clone());
                    };
                    admitted = projected;
                }
                if admitted.ty() == *ty {
                    admitted
                } else {
                    GoalExpression::Datum(original.clone())
                }
            }
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments: operands,
            } => GoalExpression::Operation {
                row: *row,
                type_arguments: type_arguments.clone(),
                const_arguments: const_arguments.clone(),
                result: *result,
                arguments: operands
                    .iter()
                    .map(|operand| self.admitted_call_goal_expression(operand, call, arguments))
                    .collect(),
            },
            GoalExpression::Datum(datum) => GoalExpression::Datum(datum.clone()),
        }
    }

    fn goal_expression(
        &self,
        expression: &CheckedExpression,
        admitted_partial: bool,
    ) -> Option<GoalExpression> {
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
                        PlaceProjection::Subscript(offset) => GoalProjection::Subscript(offset),
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
                .goal_expression(value, admitted_partial)?
                .with_projection(GoalProjection::Deref, *referent),
            CheckedExpression::ArenaDeref { content, value, .. } if self.is_copy(*content) => self
                .goal_expression(value, admitted_partial)?
                .with_projection(GoalProjection::Deref, *content),
            CheckedExpression::ProjectValue {
                value, field, ty, ..
            } if self.is_copy(*ty) => self
                .goal_expression(value, admitted_partial)?
                .with_projection(GoalProjection::Field(*field), *ty),
            CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                result,
                ..
            } if !operation.is_exact() || admitted_partial => build_operation(
                GoalOperation::Integer {
                    operation: *operation,
                    operand_type: *operand_type,
                },
                Vec::new(),
                Vec::new(),
                *result,
                arguments
                    .iter()
                    .map(|argument| self.goal_expression(argument, admitted_partial))
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
                    .map(|argument| self.goal_expression(argument, admitted_partial))
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
                vec![self.goal_expression(value, admitted_partial)?],
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
                vec![self.goal_expression(value, admitted_partial)?],
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
                    .map(|argument| self.goal_expression(argument, admitted_partial))
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
                    .map(|argument| self.goal_expression(argument, admitted_partial))
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
                    vec![self.goal_expression(value, admitted_partial)?],
                )
            }
            CheckedExpression::ArrayMeasure {
                measure,
                root,
                length,
            } => {
                let argument = self.goal_array_root(root)?;
                let CheckedType::Array { element, .. } = argument.ty() else {
                    return None;
                };
                build_operation(
                    GoalOperation::ArrayMeasure {
                        measure: *measure,
                        element,
                        length: *length,
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::ArrayIndex {
                root,
                element_type,
                length,
                offset,
                ..
            } if admitted_partial => {
                let collection = self.goal_array_root(root)?;
                let CheckedType::Array {
                    element,
                    length: root_length,
                } = collection.ty()
                else {
                    return None;
                };
                if root_length != *length || element.ty() != *element_type {
                    return None;
                }
                build_operation(
                    GoalOperation::ArrayIndex {
                        element,
                        length: *length,
                    },
                    Vec::new(),
                    Vec::new(),
                    *element_type,
                    vec![collection, self.goal_expression(offset, admitted_partial)?],
                )
            }
            // [MSR-1] a measure of a run or a bump extent, read as the same
            // quantity the reader row loads.
            CheckedExpression::ContainerMeasure { measure, root } => {
                let measured = root.measured()?;
                let argument =
                    self.goal_binding_place(root.binding, root.goal_projections(), root.ty);
                build_operation(
                    GoalOperation::ContainerMeasure {
                        measure: *measure,
                        measured,
                        element: root.element(),
                        constant: root.type_constant(),
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::RunIndex {
                root,
                element_type,
                offset,
                ..
            } if admitted_partial => {
                let measured = root.measured()?;
                let element = root.element()?;
                if element.ty() != *element_type {
                    return None;
                }
                let collection =
                    self.goal_binding_place(root.binding, root.goal_projections(), root.ty);
                build_operation(
                    GoalOperation::RunIndex {
                        measured,
                        element,
                        constant: root.type_constant(),
                    },
                    Vec::new(),
                    Vec::new(),
                    *element_type,
                    vec![collection, self.goal_expression(offset, admitted_partial)?],
                )
            }
            CheckedExpression::BufferMeasure { measure, root } => {
                let argument = self.goal_binding_place(
                    root.binding,
                    root.fields.iter().copied().map(GoalProjection::Field),
                    CheckedType::Buffer {
                        element: root.element,
                    },
                );
                build_operation(
                    GoalOperation::BufferMeasure {
                        measure: *measure,
                        element: root.element,
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::BufferIndex { root, offset, .. } if admitted_partial => {
                let collection_type = CheckedType::Buffer {
                    element: root.element,
                };
                let collection = self.goal_binding_place(
                    root.binding,
                    root.fields.iter().copied().map(GoalProjection::Field),
                    collection_type,
                );
                build_operation(
                    GoalOperation::BufferIndex {
                        element: root.element,
                    },
                    Vec::new(),
                    Vec::new(),
                    root.element.ty(),
                    vec![collection, self.goal_expression(offset, admitted_partial)?],
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
                vec![self.goal_expression(length, admitted_partial)?],
            ),
            CheckedExpression::SliceMeasure { measure, root } => {
                let ty = self.summary(root.binding)?.ty?;
                let CheckedType::Slice {
                    region, element, ..
                } = ty
                else {
                    return None;
                };
                let argument = self.goal_binding_place(root.binding, std::iter::empty(), ty);
                build_operation(
                    GoalOperation::SliceMeasure {
                        measure: *measure,
                        region,
                        element,
                    },
                    Vec::new(),
                    Vec::new(),
                    CheckedType::Integer(IntegerType::U64),
                    vec![argument],
                )
            }
            CheckedExpression::SliceIndex { root, offset, .. } if admitted_partial => {
                let ty = self.summary(root.binding)?.ty?;
                let CheckedType::Slice {
                    region, element, ..
                } = ty
                else {
                    return None;
                };
                if element != root.element {
                    return None;
                }
                let collection = self.goal_binding_place(root.binding, std::iter::empty(), ty);
                build_operation(
                    GoalOperation::SliceIndex { region, element },
                    Vec::new(),
                    Vec::new(),
                    element.ty(),
                    vec![collection, self.goal_expression(offset, admitted_partial)?],
                )
            }
            CheckedExpression::Binding { .. }
            | CheckedExpression::Project { .. }
            | CheckedExpression::DerefAddressed { .. }
            | CheckedExpression::BoxDeref { .. }
            | CheckedExpression::ProjectValue { .. }
            | CheckedExpression::UserCall { .. }
            | CheckedExpression::SystemCall { .. }
            | CheckedExpression::KernelCall { .. }
            | CheckedExpression::PostconditionResultMeasure { .. }
            | CheckedExpression::RunIndex { .. }
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

    /// Exact value identity for one operand of an already-reached proof
    /// obligation. Stable structural expressions are preferred so a prior
    /// source fact can name the same value. The occurrence-local fallback is
    /// reserved for a value that cannot be safely replayed from source.
    fn obligation_goal_operand(
        &mut self,
        site: &crate::NodePath,
        operand: usize,
        expression: &CheckedExpression,
        facts: &FactState,
    ) -> GoalExpression {
        self.admitted_value_goal_expression(expression)
            .map(|expression| self.expand_goal_expression(&expression, facts))
            .unwrap_or_else(|| {
                let operand =
                    u32::try_from(operand).expect("proof-obligation operand ordinal exceeds u32");
                GoalExpression::Datum(GoalDatum::EvaluatedValue {
                    function: self.function.id,
                    occurrence: EvaluatedValueOccurrence::ObligationOperand {
                        site: site.clone(),
                        operand,
                    },
                    captured_type: expression.ty(),
                    projections: Vec::new(),
                    ty: expression.ty(),
                })
            })
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
                        CheckedNominalKind::Box { referent, .. } => Some(referent),
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
            // [OP-4] a subscript selects the base's element type, which
            // [MSR-1] admits in a measure place and [BLK-1] gives the one
            // slot a run holds.
            GoalProjection::Subscript(_) => element_type(input),
        }
    }

    fn goal_origin_set(
        &mut self,
        expression: &CheckedExpression,
        state: &FactState,
    ) -> Vec<GoalId> {
        let Some(direct) = self.admitted_value_goal_expression(expression) else {
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
        let Some(direct) = self.admitted_value_goal_expression(value) else {
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
        self.collect_goal_support(&expression, None, &mut support);
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
        measure: Option<CheckedMeasure>,
        support: &mut Vec<GoalSupport>,
    ) {
        match expression {
            GoalExpression::Datum(GoalDatum::Place {
                root, projections, ..
            }) => support.push(GoalSupport {
                root: *root,
                projections: projections.clone(),
                measure,
            }),
            GoalExpression::Datum(
                GoalDatum::Parameter { .. }
                | GoalDatum::NamedConst { .. }
                | GoalDatum::EvaluatedValue { .. }
                | GoalDatum::Literal(_),
            ) => {}
            GoalExpression::Operation { row, arguments, .. } => {
                // [MSR-2] every measure of one place has the same support,
                // P's descriptor storage; the selected measure only says that
                // this node is a measure node rather than a place node.
                let node_measure = match row {
                    GoalOperation::ArrayMeasure { measure, .. }
                    | GoalOperation::BufferMeasure { measure, .. }
                    | GoalOperation::SliceMeasure { measure, .. }
                    | GoalOperation::ContainerMeasure { measure, .. } => Some(*measure),
                    _ => None,
                };
                for argument in arguments {
                    self.collect_goal_support(argument, node_measure, support);
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
        // [MSR-5] each side is an affine expression, so each projects to one
        // term displaced by a constant and the two displacements fold into
        // the one constant a difference bound carries.
        let (left, left_constant) = self.goal_side(left)?;
        let (right, right_constant) = self.goal_side(right)?;
        sources::comparison_relation(
            *operation,
            left,
            right,
            right_constant.checked_sub(left_constant)?,
        )
    }

    /// One clause side as a term displaced by a constant [MSR-5].
    ///
    /// A side with no term at all is one constant and keeps the constant term
    /// [ENT-2] folds it onto; a side carrying two terms, or a term with any
    /// coefficient other than one, is outside the difference-bound fragment
    /// and projects to nothing, which only under-derives [ENT-1].
    fn goal_side(&mut self, expression: &GoalExpression) -> Option<(TermId, i128)> {
        let (term, constant) = self.goal_affine_side(expression)?;
        match term {
            Some(term) => Some((term, constant)),
            None => Some((self.terms.intern(TermKind::Constant(constant)), 0)),
        }
    }

    fn goal_affine_side(&mut self, expression: &GoalExpression) -> Option<(Option<TermId>, i128)> {
        if let GoalExpression::Operation {
            row:
                GoalOperation::Integer {
                    operation:
                        operation @ (CheckedIntegerOperation::AddExact
                        | CheckedIntegerOperation::SubtractExact
                        | CheckedIntegerOperation::MultiplyExact),
                    ..
                },
            arguments,
            ..
        } = expression
        {
            let [left, right] = arguments.as_slice() else {
                return None;
            };
            let (left_term, left_value) = self.goal_affine_side(left)?;
            let (right_term, right_value) = self.goal_affine_side(right)?;
            return match operation {
                CheckedIntegerOperation::AddExact => {
                    if left_term.is_some() && right_term.is_some() {
                        return None;
                    }
                    Some((
                        left_term.or(right_term),
                        left_value.checked_add(right_value)?,
                    ))
                }
                CheckedIntegerOperation::SubtractExact => {
                    if right_term.is_some() {
                        return None;
                    }
                    Some((left_term, left_value.checked_sub(right_value)?))
                }
                _ => {
                    if left_term.is_some() || right_term.is_some() {
                        return None;
                    }
                    Some((None, left_value.checked_mul(right_value)?))
                }
            };
        }
        if let GoalExpression::Datum(GoalDatum::Literal(CheckedValue::Integer { ty, bits })) =
            expression
        {
            return Some((None, integer_value(*ty, *bits)));
        }
        Some((Some(self.goal_operand(expression)?), 0))
    }

    fn goal_operand(&mut self, expression: &GoalExpression) -> Option<TermId> {
        match expression {
            // [MSR-6] a const generic operand is the symbolic constant term.
            GoalExpression::Datum(GoalDatum::Literal(CheckedValue::ConstGeneric {
                declaration,
                ..
            })) => Some(self.terms.intern(TermKind::ConstParameter(*declaration))),
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
                                    PlaceProjection::Deref | PlaceProjection::Subscript(_) => None,
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
                    GoalOperation::ArrayMeasure { .. }
                        | GoalOperation::BufferMeasure { .. }
                        | GoalOperation::SliceMeasure { .. }
                        | GoalOperation::ContainerMeasure { .. }
                ) =>
            {
                let [place] = arguments.as_slice() else {
                    return None;
                };
                let GoalExpression::Datum(datum) = place else {
                    return None;
                };
                let path = self.goal_place_path(datum)?;
                let (measure, measured, array_length) = match row {
                    GoalOperation::ArrayMeasure {
                        measure, length, ..
                    } => (*measure, MeasuredKind::Array, Some(*length)),
                    GoalOperation::BufferMeasure { measure, .. } => {
                        (*measure, MeasuredKind::Buffer, None)
                    }
                    GoalOperation::SliceMeasure { measure, .. } => {
                        (*measure, MeasuredKind::Slice, None)
                    }
                    // [MSR-1]'s row for a run or a bump extent. The written
                    // constant is what `measure_term` reads for a cell the
                    // table fixes as the type's own constant [MSR-2].
                    GoalOperation::ContainerMeasure {
                        measure,
                        measured,
                        constant,
                        ..
                    } => (*measure, *measured, *constant),
                    _ => return None,
                };
                Some(self.measure_term(measure, path, measured, array_length))
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
            | GoalDatum::EvaluatedValue { .. }
            | GoalDatum::Literal(_) => return None,
        };
        Some(ProjectedPlaceTerm {
            root,
            projections: projections
                .iter()
                .map(|projection| match projection {
                    GoalProjection::Deref => PlaceProjection::Deref,
                    GoalProjection::Field(field) => PlaceProjection::Field(*field),
                    GoalProjection::Subscript(offset) => PlaceProjection::Subscript(*offset),
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
            GoalExpression::Datum(GoalDatum::EvaluatedValue { .. }) => None,
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

    fn argument_referent(&self, argument: &CheckedExpression) -> Option<(ResolvedPlace, bool)> {
        self.places.argument_referent(argument)
    }

    /// The [ENT-5] kills one call's write through a viewed range projects
    /// [CALL-3].
    ///
    /// The write reaches the viewed range's element storage and no measure
    /// term over the origin place itself, nor over the view: `len_of(origin)`
    /// and `len_of(view)` both survive it, and a measure of a viewed element
    /// whose type has descriptor storage of its own dies with that storage.
    /// Today a view's element domain is flat, so no measured element reaches
    /// this classification and the surviving half is its whole effect here.
    fn collect_view_write_kills(
        &self,
        argument: &CheckedExpression,
        call: &crate::NodePath,
        events: &mut Vec<KillEvent>,
    ) {
        if let Some(place) = self.places.viewed_write_referent(argument) {
            events.push(KillEvent::Write {
                place: element_write_place(place, PlaceOffset::Opaque),
                element: true,
                source: call.clone(),
            });
            return;
        }
        // [SYS-8]'s range-bearing operand class has a transitional member
        // that is a `buffer<u8>` rather than a view [VIEW-1]. The row's
        // declared extent still makes the write a viewed-range one, and the
        // descriptor it names is the buffer's own place.
        if let Some((place, entry_image_only)) = self.argument_referent(argument) {
            let place = element_write_place(place, PlaceOffset::Opaque);
            if entry_image_only {
                events.push(KillEvent::EntryImageHolderWrite {
                    place,
                    element: true,
                    source: call.clone(),
                });
            } else {
                events.push(KillEvent::Write {
                    place,
                    element: true,
                    source: call.clone(),
                });
            }
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
                } else if !self.is_copy(*ty)
                    // [VIEW-1, OWN-5] a borrow of a view is not a consume of
                    // it. A view binding occurs in a `borrow_expr` as itself,
                    // because the descriptor is what a borrow of one carries,
                    // and that occurrence carries `consume_root: false`; the
                    // exclusive view is affine, so without this the ordinary
                    // affine kill would end every fact about a view the
                    // moment it is handed to a call that only borrows it.
                    && (*consume_root || !matches!(ty, CheckedType::Slice { .. }))
                {
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
                    // [CALL-5] the transport is the declared parameter's and
                    // never the actual's spelling: a shared borrow is a kill
                    // event for nothing [CALL-1], a view confines the write to
                    // the range's element storage [CALL-3], and every other
                    // parameter kills conservatively.
                    let transport = callee
                        .and_then(|callee| callee.parameter_transports.get(index).copied())
                        .unwrap_or_default();
                    if transport == CallTransport::SharedBorrow {
                        continue;
                    }
                    let element = transport.writes_element_storage();
                    if !writes.is_empty() && element {
                        self.collect_view_write_kills(argument, call, events);
                        continue;
                    }
                    if let Some((place, entry_image_only)) = self.argument_referent(argument) {
                        for fields in writes {
                            let mut written = place.clone();
                            written.extend_fields(fields);
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
            // [BLK-0, EFF-1] a row's declared effect row is a callee effect
            // like any other: the place its `writes` names is written by the
            // call, so [ENT-5] kills every fact whose support that place
            // reaches. Without this the store's pre-call measure facts and
            // the pre-transfer call-datum equality survive beside the row's
            // own post-state relations, and the two together are a
            // contradiction the row introduces.
            CheckedExpression::KernelCall {
                row,
                call,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.collect_expression_kills(argument, events);
                }
                let signature = super::super::kernel::kernel_signature(*row);
                let Some(written) = signature.effects.writes else {
                    return;
                };
                // An `own` operand is consumed rather than written back, and
                // its consume is already collected above; only a `&uniq`
                // state operand is a place the callee writes.
                if signature
                    .parameters
                    .get(written as usize)
                    .is_none_or(|parameter| {
                        parameter.mode != super::super::kernel::KernelMode::Unique
                    })
                {
                    return;
                }
                let Some(argument) = arguments.get(written as usize) else {
                    return;
                };
                // [CALL-5] the row's declared parameter selects the transport.
                let transport = signature.parameters.get(written as usize).map_or(
                    CallTransport::Conservative,
                    CallTransport::of_kernel_parameter,
                );
                if transport == CallTransport::SharedBorrow {
                    return;
                }
                let element = transport.writes_element_storage();
                if element {
                    self.collect_view_write_kills(argument, call, events);
                    return;
                }
                if let Some((place, entry_image_only)) = self.argument_referent(argument) {
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
                    if !written {
                        continue;
                    }
                    // [CALL-5] a system operation has no body: its [SYS-2]
                    // record is the whole of its declared contract, and
                    // [SYS-8] declares that its range-bearing family's
                    // `[start, end)` extent is the complete extent it may
                    // change and is element storage, so that parameter is a
                    // viewed range [CALL-3].
                    let transport = operation_row.map_or(CallTransport::Conservative, |row| {
                        CallTransport::of_system_parameter(row, index)
                    });
                    if transport == CallTransport::SharedBorrow {
                        continue;
                    }
                    let element = transport.writes_element_storage();
                    if element {
                        self.collect_view_write_kills(argument, call, events);
                        continue;
                    }
                    if let Some((place, entry_image_only)) = self.argument_referent(argument) {
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
    ///
    /// A parent operation is reached only after every partial operation in
    /// its already-evaluated children succeeds.  The acceptance-dark test
    /// hook keeps a failed child outcome for inspection, but must not then
    /// manufacture an admitted exact/index value or a later obligation for
    /// the unreachable parent.
    fn judge_children_reach_parent<'expression>(
        &mut self,
        children: impl IntoIterator<Item = &'expression CheckedExpression>,
        states: &mut ProofFlowState,
    ) -> bool {
        let mut reached = true;
        for child in children {
            reached &= self.judge_expression(child, states).reached;
        }
        reached
    }

    fn obligations_since_discharged(&self, obligation_start: usize) -> bool {
        self.obligations[obligation_start..]
            .iter()
            .all(|outcome| outcome.discharged)
    }

    fn judge_expression(
        &mut self,
        expression: &CheckedExpression,
        states: &mut ProofFlowState,
    ) -> ExpressionJudgment {
        match expression {
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                goal_arguments,
                requirements,
                ..
            } => {
                let obligation_start = self.obligations.len();
                let mut actuals_reached = true;
                for argument in arguments {
                    actuals_reached &= self.judge_expression(argument, states).reached;
                }
                let actual_parents = self.obligations[obligation_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let mut goal_parents = Vec::with_capacity(requirements.len());
                let mut goals_ok = actuals_reached;
                let admitted_arguments = actuals_reached.then(|| {
                    arguments
                        .iter()
                        .zip(goal_arguments)
                        .map(|(argument, captured)| {
                            matches!(
                                captured,
                                GoalExpression::Datum(GoalDatum::EvaluatedValue {
                                    occurrence: EvaluatedValueOccurrence::CallArgument {
                                        call: occurrence_call,
                                        ..
                                    },
                                    ..
                                }) if occurrence_call == call
                            )
                            .then(|| self.admitted_value_goal_expression(argument))
                            .flatten()
                        })
                        .collect::<Vec<_>>()
                });
                // FN-8 begins only after every actual-expression obligation
                // succeeds. A failed OP-4 actual therefore publishes no call
                // judgment for diagnostic selection to reorder.
                for requirement in requirements {
                    if actuals_reached {
                        let goal = ConcreteGoal::new(
                            self.admitted_call_goal_expression(
                                &requirement.goal.root,
                                call,
                                admitted_arguments
                                    .as_deref()
                                    .expect("reached actuals have admitted argument slots"),
                            ),
                        );
                        let (disposition, derivation) = self.judge_call_goal(
                            *function,
                            call,
                            requirement.requires_clause.clone(),
                            goal,
                            arguments.len(),
                            ProofContext::new(&states.facts, &states.affine),
                        );
                        goals_ok &= disposition == CallGoalDisposition::Discharged;
                        if let Some(derivation) = derivation {
                            goal_parents.push(derivation);
                        }
                    }
                }
                let reached = actuals_reached && goals_ok;
                let prepared_call = (|| {
                    let mut parents = actual_parents?;
                    if !reached || goal_parents.len() != requirements.len() {
                        return None;
                    }
                    parents.extend(goal_parents);
                    // Only an earlier-component verified summary can publish
                    // an S12 carrier. Calls without one retain the exact
                    // pre-H3 kill path and create no transient postcondition
                    // events, but are still successfully evaluated.
                    if self.context.verified_postconditions(*function)?.is_empty() {
                        return None;
                    }
                    Some(PreparedCall {
                        callee: PreparedCallee::Source(*function),
                        call: call.clone(),
                        parents,
                        transfer_events: Vec::new(),
                        kills: Vec::new(),
                    })
                })();
                // [ENT-3.S13, MSR-3] the call datums are minted here, at the
                // pre-transfer point [ENT-5] fixes, and not at the later
                // establishment: instantiating at the call is what lets a
                // relation over an `own` operand outlive the consume the same
                // statement performs.
                if prepared_call.is_some() {
                    self.establish_call_datums(*function, call, goal_arguments, &mut states.facts);
                }
                ExpressionJudgment {
                    prepared_call,
                    reached,
                }
            }
            CheckedExpression::SystemCall {
                operation,
                call,
                arguments,
                ..
            } => {
                let reaches_call = self.judge_children_reach_parent(arguments, states);
                let obligation_start = self.obligations.len();
                if reaches_call {
                    self.judge_system_ranges(*operation, call, arguments, states);
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_call && self.obligations_since_discharged(obligation_start),
                }
            }
            // One [BLK-0] kernel-domain row. Its declared requirement list is
            // record data, so each clause is submitted here as an obligation
            // judged under [MSR-4] exactly as every other consumer's is.
            CheckedExpression::KernelCall {
                operation,
                row,
                call,
                instance,
                arguments,
                requirements,
                ..
            } => {
                let obligation_start = self.obligations.len();
                let mut actuals_reached = true;
                for argument in arguments {
                    actuals_reached &= self.judge_expression(argument, states).reached;
                }
                let actual_parents = self.obligations[obligation_start..]
                    .iter()
                    .map(|outcome| outcome.discharged.then_some(outcome.derivation).flatten())
                    .collect::<Option<Vec<_>>>();
                let mut goal_parents = Vec::with_capacity(requirements.len());
                let mut goals_ok = actuals_reached;
                // [BLK-0, OP-9] the acquiring rows carry the allocation-fit
                // obligation their record notation spells `fits::<T>(count)`.
                // It is not a term and therefore not a member of the row's
                // declared requirement list; it is the same object
                // `buffer_fits::<T>(n)` is, judged by [OP-9]'s own judgment
                // under [MSR-4], so an undischarged one is the ordinary
                // static OP-9 rejection.
                let fits_start = self.obligations.len();
                if actuals_reached
                    && let Some(ordinal) = crate::semantic::kernel::kernel_signature(*row).fits
                    && let Some(count) = arguments.get(ordinal as usize)
                {
                    self.judge_allocation_fit(
                        instance.element,
                        instance.element_ceiling.stride.allocation_limit(),
                        count,
                        call.clone(),
                        states,
                    );
                    goals_ok &= self.obligations_since_discharged(fits_start);
                }
                if actuals_reached {
                    for (ordinal, requirement) in requirements.iter().enumerate() {
                        let derivation = self.judge_kernel_requirement(
                            *operation,
                            u8::try_from(ordinal).unwrap_or(u8::MAX),
                            call,
                            requirement.clone(),
                            ProofContext::new(&states.facts, &states.affine),
                        );
                        match derivation {
                            Some(derivation) => goal_parents.push(derivation),
                            None => goals_ok = false,
                        }
                    }
                }
                let reached = actuals_reached && goals_ok;
                let prepared_call = (|| {
                    let mut parents = actual_parents?;
                    if !reached || goal_parents.len() != requirements.len() {
                        return None;
                    }
                    parents.extend(goal_parents);
                    Some(PreparedCall {
                        callee: PreparedCallee::Kernel(*operation),
                        call: call.clone(),
                        parents,
                        transfer_events: Vec::new(),
                        kills: Vec::new(),
                    })
                })();
                // [ENT-3.S13] a kernel-domain row is a population member of
                // the call-datum source, so its `own` operands and the
                // `at the call` measures its relations name are minted here,
                // at the same pre-transfer point.
                if prepared_call.is_some() {
                    self.establish_kernel_call_datums(expression, &mut states.facts);
                }
                ExpressionJudgment {
                    prepared_call,
                    reached,
                }
            }
            CheckedExpression::ArrayIndex {
                root,
                length,
                offset,
                obligation,
                ..
            } => {
                let reaches_index =
                    self.judge_children_reach_parent(std::iter::once(offset.as_ref()), states);
                let obligation_start = self.obligations.len();
                if reaches_index {
                    let base = self.array_root_place(root);
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Array,
                        Some(*length),
                        offset,
                        obligation.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_index && self.obligations_since_discharged(obligation_start),
                }
            }
            CheckedExpression::BufferIndex {
                root,
                offset,
                obligation,
                ..
            } => {
                let reaches_index =
                    self.judge_children_reach_parent(std::iter::once(offset.as_ref()), states);
                let obligation_start = self.obligations.len();
                if reaches_index {
                    let base = PlaceTerm {
                        root: PlaceRoot::Binding(root.binding),
                        deref: self.is_holder(root.binding),
                        fields: root.fields.clone(),
                    };
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Buffer,
                        None,
                        offset,
                        obligation.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_index && self.obligations_since_discharged(obligation_start),
                }
            }
            CheckedExpression::SliceIndex {
                root,
                offset,
                obligation,
                ..
            } => {
                let reaches_index =
                    self.judge_children_reach_parent(std::iter::once(offset.as_ref()), states);
                let obligation_start = self.obligations.len();
                if reaches_index {
                    let base = PlaceTerm {
                        root: PlaceRoot::Binding(root.binding),
                        deref: self.is_holder(root.binding),
                        fields: Vec::new(),
                    };
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Slice,
                        None,
                        offset,
                        obligation.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_index && self.obligations_since_discharged(obligation_start),
                }
            }
            // [MSR-1] a measure over a subscripted place is a term only where
            // that place's own subscripts are discharged [OP-4].
            CheckedExpression::ContainerMeasure { root, .. } => {
                let obligation_start = self.obligations.len();
                let reached = self.judge_place_subscripts(root, states);
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reached && self.obligations_since_discharged(obligation_start),
                }
            }
            // [VIEW-2] a view formed over a run submits that row's own
            // declared requirement here, judged under [MSR-4] exactly as
            // every other consumer's obligation is. The clause is
            // `head_of(vector) <= room_of(vector)`, which is what makes the
            // viewed window one contiguous range: a wrapped window is two,
            // and a view of one would reach storage the run does not own.
            // The two retiring operand types carry the same clause and
            // discharge it from their own measure-table row, whose `head`
            // and `room` cells are both the constant zero [MSR-1].
            CheckedExpression::SliceOf {
                carrier,
                source: CheckedSliceSource::Run(root),
                strength,
                ..
            } => {
                let obligation_start = self.obligations.len();
                let reached = self.judge_place_subscripts(root, states);
                if reached && let Some(goal) = self.non_wrapped_window_goal(root) {
                    self.judge_kernel_requirement(
                        crate::semantic::kernel::kernel_ordinal(match strength {
                            LoanStrength::Shared => crate::KernelRow::SliceOf,
                            LoanStrength::Exclusive => crate::KernelRow::MutSliceOf,
                        }),
                        0,
                        carrier,
                        goal,
                        ProofContext::new(&states.facts, &states.affine),
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reached && self.obligations_since_discharged(obligation_start),
                }
            }
            // [OP-4, BLK-1] a run's subscript owes `i < len_of(v)` wherever it
            // is written: the offset is a logical one and the window's length
            // bounds it, so the measured kind is the run's own and the written
            // capacity is not the bound. A read owes exactly what the
            // element-position target below owes, and is judged here.
            CheckedExpression::RunIndex {
                root,
                offset,
                obligation,
                ..
            } => {
                let reaches_base = self.judge_place_subscripts(root, states);
                let reaches_index = reaches_base
                    && self.judge_children_reach_parent(std::iter::once(offset.as_ref()), states);
                let obligation_start = self.obligations.len();
                if reaches_index && let Some(measured) = root.measured() {
                    let base = self.container_root_path(root);
                    self.judge_obligation(
                        base,
                        measured,
                        root.type_constant(),
                        offset,
                        obligation.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_index && self.obligations_since_discharged(obligation_start),
                }
            }
            CheckedExpression::BufferFill {
                carrier,
                element,
                layout_ceiling,
                length,
                value,
                ..
            } => {
                let reaches_allocation =
                    self.judge_children_reach_parent([length.as_ref(), value.as_ref()], states);
                let obligation_start = self.obligations.len();
                if reaches_allocation {
                    self.judge_allocation_fit(
                        element.ty(),
                        layout_ceiling.stride.allocation_limit(),
                        length,
                        carrier.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_allocation
                        && self.obligations_since_discharged(obligation_start),
                }
            }
            CheckedExpression::BufferVacant {
                carrier,
                element,
                layout_ceiling,
                length,
                ..
            } => {
                let reaches_allocation =
                    self.judge_children_reach_parent(std::iter::once(length.as_ref()), states);
                let obligation_start = self.obligations.len();
                if reaches_allocation {
                    self.judge_allocation_fit(
                        CheckedType::Nominal(*element),
                        layout_ceiling.stride.allocation_limit(),
                        length,
                        carrier.clone(),
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_allocation
                        && self.obligations_since_discharged(obligation_start),
                }
            }
            CheckedExpression::IntegerOperation {
                carrier,
                operation,
                operand_type,
                arguments,
                ..
            } => {
                let reaches_operation = self.judge_children_reach_parent(arguments, states);
                let obligation_start = self.obligations.len();
                if operation.is_exact() && reaches_operation {
                    self.judge_integer_domain_obligation(
                        *operation,
                        *operand_type,
                        arguments,
                        carrier,
                        states,
                    );
                }
                ExpressionJudgment {
                    prepared_call: None,
                    reached: reaches_operation
                        && self.obligations_since_discharged(obligation_start),
                }
            }
            _ => {
                let reached =
                    self.judge_children_reach_parent(expression_children(expression), states);
                ExpressionJudgment {
                    prepared_call: None,
                    reached,
                }
            }
        }
    }

    /// [BLK-0, MSR-4] one declared requirement of a kernel-domain row,
    /// judged at the call.
    ///
    /// A record has no source node, so the outcome carries the row's own
    /// ordinal and the requirement's position in the row's declared list;
    /// [DIAG-1]'s location is the call itself.
    /// [VIEW-2]'s non-wrap premise over one viewed run, as a caller-side
    /// goal.
    ///
    /// A run whose measure table has no row is no operand of this row, and a
    /// goal it cannot be stated over is simply unavailable, which leaves the
    /// obligation unsubmitted and the formation refused by the ordinary
    /// [MSR-4] disposition rather than admitted unchecked.
    fn non_wrapped_window_goal(&mut self, root: &CheckedContainerRoot) -> Option<ConcreteGoal> {
        let measured = root.measured()?;
        let measure = |measure| GoalExpression::Operation {
            row: GoalOperation::ContainerMeasure {
                measure,
                measured,
                element: root.element(),
                constant: root.type_constant(),
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Integer(IntegerType::U64),
            arguments: vec![self.goal_binding_place(
                root.binding,
                root.goal_projections(),
                root.ty,
            )],
        };
        Some(ConcreteGoal::new(GoalExpression::Operation {
            row: GoalOperation::Integer {
                operation: CheckedIntegerOperation::LessEqual,
                operand_type: CheckedType::Integer(IntegerType::U64),
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: vec![measure(CheckedMeasure::Head), measure(CheckedMeasure::Room)],
        }))
    }

    fn judge_kernel_requirement(
        &mut self,
        operation: u8,
        requirement: u8,
        call: &crate::NodePath,
        goal: ConcreteGoal,
        context: ProofContext<'_>,
    ) -> Option<DerivationId> {
        let (disposition, _, derivation) = self.call_goal_disposition(&goal, context);
        let ordinal = u32::try_from(self.obligations.len())
            .expect("ENT obligation-root ordinal exceeds the u32 identity space");
        if let Some(root) = derivation {
            self.derivations
                .add_root(DerivationRootKind::BoundsObligation(ordinal), root);
        }
        let discharged = disposition == CallGoalDisposition::Discharged;
        let rendered = self.render_concrete_goal(&goal.root);
        self.obligations.push(ObligationOutcome {
            node_path: call.clone(),
            family: ObligationFamily::KernelRequirement,
            conjunct: requirement,
            canonical_goal: Some(goal.root),
            components: Vec::new(),
            discharged,
            refuted: disposition == CallGoalDisposition::Refuted,
            contradictory: false,
            residual: (!discharged).then_some(rendered),
            derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
            kernel_row: Some(operation),
        });
        discharged.then_some(derivation).flatten()
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
                product_interval: None,
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
                product_interval: None,
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
            product_interval: None,
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
                product_interval: None,
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
                product_interval: None,
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
                product_interval: None,
            };
        }

        // The affine target is this goal's own comparison, normalized. Proving
        // it proves the goal, so the L0 projection is what the evidence names,
        // not what the route needs: a goal that carries a coefficient has no
        // two-term projection to name and is proved by the consequence alone.
        if let Some(target) = affine_target {
            let projection = self.goals.projection(goal).cloned();
            let assumptions = Self::affine_facts(context.affine);
            if let Some(proof) =
                self.affine_target_proof(target, &assumptions, context.affine, context.facts)
            {
                let consequence = self.derivations.intern(DerivationNode::AffineConsequence {
                    relation: projection.clone().map(Box::new),
                    premises: proof.premises.into_boxed_slice(),
                    parents: proof.parents,
                });
                let derivation = match projection {
                    Some(relation) => self.derivations.intern(DerivationNode::GoalProjection {
                        goal,
                        sign: GoalSign::Positive,
                        relation,
                        parent: consequence,
                    }),
                    None => consequence,
                };
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::Affine),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        let derivation = self.signed_goal_affine_proof(
            context,
            expression,
            GoalSign::Positive,
            &closed,
            &mut HashSet::new(),
        );
        ProofResult {
            disposition: if derivation.is_some() {
                ProofDisposition::Proved
            } else {
                ProofDisposition::Unknown
            },
            route: derivation.map(|_| ProofRoute::Affine),
            derivation,
            numeric_upper_bound: None,
            product_interval: None,
        }
    }

    /// Proves one signed ordering leaf through the ordinary affine entry and
    /// projects the result back to the exact interned Goal. This is the leaf
    /// case used by the fixed Boolean recursion below.
    fn affine_signed_goal_leaf_proof(
        &mut self,
        context: ProofContext<'_>,
        expression: &GoalExpression,
        goal: GoalId,
        sign: GoalSign,
    ) -> Option<DerivationId> {
        let target = self.affine_signed_goal_ordering_target(expression, context.affine, sign)?;
        let mut relation = self.goals.projection(goal)?.clone();
        if sign == GoalSign::Negative {
            relation = relation.negated();
        }
        let assumptions = Self::affine_facts(context.affine);
        let proof =
            self.affine_target_proof(&target, &assumptions, context.affine, context.facts)?;
        let consequence = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: Some(Box::new(relation.clone())),
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        Some(self.derivations.intern(DerivationNode::GoalProjection {
            goal,
            sign,
            relation,
            parent: consequence,
        }))
    }

    /// Extends the existing finite Boolean introduction rule with affine
    /// ordering leaves. The recursion follows the closed truth table exactly:
    /// conjunction requires every positive child, disjunction every negative
    /// child, the opposite signs require one witness, and `not` flips sign.
    /// It performs no premise, coefficient, or path search.
    fn signed_goal_affine_proof(
        &mut self,
        context: ProofContext<'_>,
        expression: &GoalExpression,
        sign: GoalSign,
        closed: &ClosedState,
        visiting: &mut HashSet<(GoalId, GoalSign)>,
    ) -> Option<DerivationId> {
        let goal = self.intern_goal_expression(expression.clone());
        if let Some(proof) = closed.goal_proof(goal, sign, &self.goals, &mut self.derivations) {
            return Some(proof);
        }
        if !visiting.insert((goal, sign)) {
            return None;
        }

        let proof = match expression {
            GoalExpression::Operation {
                row: GoalOperation::Boolean(operation),
                arguments,
                ..
            } => {
                let child_sign = match (operation, sign) {
                    (CheckedBooleanOperation::And, GoalSign::Positive)
                    | (CheckedBooleanOperation::Or, GoalSign::Positive) => GoalSign::Positive,
                    (CheckedBooleanOperation::And, GoalSign::Negative)
                    | (CheckedBooleanOperation::Or, GoalSign::Negative) => GoalSign::Negative,
                    (CheckedBooleanOperation::Not, GoalSign::Positive) => GoalSign::Negative,
                    (CheckedBooleanOperation::Not, GoalSign::Negative) => GoalSign::Positive,
                    (CheckedBooleanOperation::ExclusiveOr, _) => {
                        visiting.remove(&(goal, sign));
                        return None;
                    }
                };
                let requires_all = matches!(
                    (operation, sign),
                    (CheckedBooleanOperation::And, GoalSign::Positive)
                        | (CheckedBooleanOperation::Or, GoalSign::Negative)
                        | (CheckedBooleanOperation::Not, _)
                );
                let parents = if requires_all {
                    let mut parents = Vec::with_capacity(arguments.len());
                    let mut complete = true;
                    for argument in arguments {
                        let Some(parent) = self.signed_goal_affine_proof(
                            context, argument, child_sign, closed, visiting,
                        ) else {
                            complete = false;
                            break;
                        };
                        parents.push(parent);
                    }
                    complete.then_some(parents)
                } else {
                    let mut best = None;
                    for argument in arguments {
                        let Some(candidate) = self.signed_goal_affine_proof(
                            context, argument, child_sign, closed, visiting,
                        ) else {
                            continue;
                        };
                        // Existential Boolean introductions use the first
                        // successful child in source order. Later witnesses
                        // cannot change acceptance, only diagnostics.
                        if best.is_none() {
                            best = Some(candidate);
                        }
                    }
                    best.map(|parent| vec![parent])
                };
                parents.map(|parents| {
                    self.derivations
                        .intern(DerivationNode::BooleanIntroduction {
                            goal,
                            sign,
                            parents,
                        })
                })
            }
            GoalExpression::Operation { .. } => {
                self.affine_signed_goal_leaf_proof(context, expression, goal, sign)
            }
            GoalExpression::Datum(_) => None,
        };
        visiting.remove(&(goal, sign));
        proof
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
                product_interval: None,
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
                product_interval: None,
            };
        }
        if closed.derives(&relation.negated()) {
            return ProofResult {
                disposition: ProofDisposition::Refuted,
                route: Some(ProofRoute::L0),
                derivation: None,
                numeric_upper_bound: None,
                product_interval: None,
            };
        }

        let Some(target) = affine_target else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
                product_interval: None,
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
                product_interval: None,
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
            product_interval: None,
        }
    }

    fn prove_bounded_relation(
        &mut self,
        context: ProofContext<'_>,
        goal: BoundedRelationGoal<'_>,
    ) -> ProofResult {
        let canonical = goal
            .canonical
            .map(|expression| self.intern_goal_expression(expression.clone()));
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
                product_interval: None,
            };
        }
        if let Some(canonical) = canonical {
            if closed.holds_opaque(canonical, GoalSign::Positive) {
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: closed.opaque_proof(canonical, GoalSign::Positive),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
            if closed.holds_opaque(canonical, GoalSign::Negative) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: None,
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        let relation = goal.request.as_ref().and_then(request_relation);
        if let Some(relation) = relation.as_ref() {
            if closed.derives(relation) {
                let parent = closed
                    .relation_proof(relation, &mut self.derivations)
                    .expect("a proved L0 relation must retain its local derivation");
                let derivation = self.goal_numeric_derivation(canonical, Some(relation), parent);
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::L0),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
            if closed.derives(&relation.negated()) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::L0),
                    derivation: None,
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        if let Some(target) = goal.direct_affine {
            let assumptions = Self::affine_facts(context.affine);
            if let Some(proof) =
                self.affine_target_proof(target, &assumptions, context.affine, context.facts)
            {
                let parent = self.derivations.intern(DerivationNode::AffineConsequence {
                    relation: relation.clone().map(Box::new),
                    premises: proof.premises.into_boxed_slice(),
                    parents: proof.parents,
                });
                let derivation = self.goal_numeric_derivation(canonical, relation.as_ref(), parent);
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::Affine),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        if let Some(request) = goal.request
            && let Some(left) = request.left
        {
            if let Some(derivation) = goal.fixed_affine_bridge.and_then(|bridge| {
                self.fixed_affine_bound_derivation(
                    bridge,
                    left,
                    request.right,
                    request.bound,
                    context.affine,
                    context.facts,
                )
            }) {
                let derivation =
                    self.goal_numeric_derivation(canonical, relation.as_ref(), derivation);
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::Affine),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }

            if let Some(derivation) = goal.affine_left.and_then(|affine_left| {
                self.affine_bound_via_l0_right(
                    affine_left,
                    left,
                    request.right,
                    request.bound,
                    context.affine,
                    context.facts,
                )
            }) {
                let derivation =
                    self.goal_numeric_derivation(canonical, relation.as_ref(), derivation);
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::Affine),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
            product_interval: None,
        }
    }

    fn goal_numeric_derivation(
        &mut self,
        goal: Option<GoalId>,
        relation: Option<&Relation>,
        parent: DerivationId,
    ) -> DerivationId {
        let Some(goal) = goal else {
            return parent;
        };
        if let Some(relation) = relation
            && self.goals.projection(goal) == Some(relation)
        {
            return self.derivations.intern(DerivationNode::GoalProjection {
                goal,
                sign: GoalSign::Positive,
                relation: relation.clone(),
                parent,
            });
        }
        if let Some(relation) = relation
            && self.goals.normalization(goal).is_some_and(|normalization| {
                normalization.clause_is_single_relation(GoalSign::Positive, 0, relation)
            })
        {
            return self.derivations.intern(DerivationNode::GoalNormalization {
                goal,
                sign: GoalSign::Positive,
                clause: 0,
                parents: vec![parent],
            });
        }
        // A bounded obligation may have an exact complete goal whose source
        // occurrence is represented in L0 only through an evaluated alias.
        // The relation remains the obligation's direct proof root; recording
        // it as this globally interned goal's normalization would attach
        // occurrence-local data to a shared identity.
        if relation.is_some() {
            return parent;
        }
        self.derivations
            .intern(DerivationNode::GoalAffineConsequence {
                goal,
                sign: GoalSign::Positive,
                parent,
            })
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
                product_interval: None,
            };
        }
        if let Some(goal) = goal {
            if closed.holds_opaque(goal, GoalSign::Positive) {
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: closed.opaque_proof(goal, GoalSign::Positive),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
            if closed.holds_opaque(goal, GoalSign::Negative) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: None,
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }
        if let Some(relation) = relation {
            if closed.derives(relation) {
                let parent = closed
                    .relation_proof(relation, &mut self.derivations)
                    .expect("a proved L0 relation must retain its local derivation");
                let derivation = self.goal_numeric_derivation(goal, Some(relation), parent);
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::L0),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
            if closed.derives(&relation.negated()) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::L0),
                    derivation: None,
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        let Some(target) = affine_target else {
            return ProofResult {
                disposition: ProofDisposition::Unknown,
                route: None,
                derivation: None,
                numeric_upper_bound: None,
                product_interval: None,
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
                product_interval: None,
            };
        };
        let consequence = self.derivations.intern(DerivationNode::AffineConsequence {
            relation: relation.cloned().map(Box::new),
            premises: proof.premises.into_boxed_slice(),
            parents: proof.parents,
        });
        let derivation = self.goal_numeric_derivation(goal, relation, consequence);
        ProofResult {
            disposition: ProofDisposition::Proved,
            route: Some(ProofRoute::Affine),
            derivation: Some(derivation),
            numeric_upper_bound: None,
            product_interval: None,
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

    /// Interns one measure term over a place spelled in the compact
    /// [`PlaceTerm`] form, with [MSR-2]'s standing facts.
    fn place_measure_term(
        &mut self,
        measure: CheckedMeasure,
        base: ProjectedPlaceTerm,
        measured: MeasuredKind,
        array_length: Option<CheckedConst>,
    ) -> TermId {
        self.measure_term(measure, base, measured, array_length)
    }

    /// [OP-4, MSR-4, INV-1] the same obligation, over the measure places one
    /// written affine relation names.
    ///
    /// An invariant evaluates nothing and reads no storage, but
    /// `len_of(table[i])` is a term there on exactly the terms it is one at a
    /// measure former the program executes: a measure over a place whose
    /// subscripts are not all discharged is no term, so the relation names a
    /// slot the run has or it names nothing. The judgment is made once, at
    /// the point the relation is written — a `loop`'s header invariant in its
    /// entering context, a local `invariant` at its own statement.
    fn judge_affine_relation_subscripts(
        &mut self,
        relation: &CheckedAffineRelation,
        states: &mut ProofFlowState,
    ) {
        for side in [&relation.left, &relation.right] {
            self.judge_affine_expression_subscripts(side, states);
        }
    }

    fn judge_affine_expression_subscripts(
        &mut self,
        expression: &CheckedAffineExpression,
        states: &mut ProofFlowState,
    ) {
        match &expression.kind {
            CheckedAffineExpressionKind::Constant { .. }
            | CheckedAffineExpressionKind::Local { .. }
            | CheckedAffineExpressionKind::ConstGeneric { .. } => {}
            CheckedAffineExpressionKind::Measure(measure) => {
                if let CheckedExpression::ContainerMeasure { root, .. } = measure.as_ref() {
                    self.judge_place_subscripts(root, states);
                }
            }
            CheckedAffineExpressionKind::Add(left, right)
            | CheckedAffineExpressionKind::Subtract(left, right) => {
                self.judge_affine_expression_subscripts(left, states);
                self.judge_affine_expression_subscripts(right, states);
            }
            CheckedAffineExpressionKind::MultiplyByConstant { value, .. } => {
                self.judge_affine_expression_subscripts(value, states);
            }
        }
    }

    /// [OP-4, MSR-4] the obligation each subscript occurring *inside* a
    /// measured place owes, judged where the place is formed.
    ///
    /// `len_of(table[i])` names a place whose own subscript is an ordinary
    /// [OP-4] occurrence: it is discharged against `len_of(table)`, over the
    /// prefix of the path that reaches its base, and the measure term itself
    /// exists only where every one of them is discharged.
    fn judge_place_subscripts(
        &mut self,
        root: &CheckedContainerRoot,
        states: &mut ProofFlowState,
    ) -> bool {
        let mut projections = Vec::new();
        if self.is_holder(root.binding) {
            projections.push(PlaceProjection::Deref);
        }
        let mut reached = true;
        for step in &root.path {
            match step {
                CheckedPlaceStep::Field(field) => {
                    projections.push(PlaceProjection::Field(*field));
                }
                CheckedPlaceStep::Subscript(subscript) => {
                    let Some(measured) = measured_kind(subscript.base_type) else {
                        return false;
                    };
                    let base = ProjectedPlaceTerm {
                        root: PlaceRoot::Binding(root.binding),
                        projections: projections.clone(),
                    };
                    let reaches_offset = self
                        .judge_children_reach_parent(std::iter::once(&subscript.offset), states);
                    let obligation_start = self.obligations.len();
                    if reaches_offset {
                        self.judge_obligation(
                            base,
                            measured,
                            type_constant(subscript.base_type),
                            &subscript.offset,
                            subscript.obligation.clone(),
                            states,
                        );
                    }
                    reached = reached
                        && reaches_offset
                        && self.obligations_since_discharged(obligation_start);
                    projections.push(PlaceProjection::Subscript(subscript.place_offset));
                }
            }
        }
        reached
    }

    /// The exact place one measured or subscripted root names [MSR-2].
    ///
    /// A run's path may carry subscripts of its own — `len_of(table[i])` is a
    /// term [MSR-1] — so it is a source-order projection path and never a
    /// field list.
    fn container_root_path(&self, root: &CheckedContainerRoot) -> ProjectedPlaceTerm {
        let mut projections = Vec::new();
        if self.is_holder(root.binding) {
            projections.push(PlaceProjection::Deref);
        }
        projections.extend(root.path.iter().map(|step| match step {
            CheckedPlaceStep::Field(field) => PlaceProjection::Field(*field),
            CheckedPlaceStep::Subscript(subscript) => {
                PlaceProjection::Subscript(subscript.place_offset)
            }
        }));
        ProjectedPlaceTerm {
            root: PlaceRoot::Binding(root.binding),
            projections,
        }
    }

    /// The place one [LIV-2] commit writes, as every measure term over it is
    /// stated [MSR-1]: a plain place, or one element position of a run.
    ///
    /// An element position is a place only where its offset is one a place
    /// relation can name [MSR-1] — a written literal, a live `own` integer
    /// binding, or an in-scope const generic. An offset of any other form is
    /// provably distinct from nothing, itself included, so a measure over it
    /// would relate two elements as one term [OWN-7] and there is no place to
    /// carry a measure to.
    fn set_target_place(&self, target: &CheckedSetTarget) -> Option<ProjectedPlaceTerm> {
        match target {
            CheckedSetTarget::Place(place) => Some(projected_place(PlaceTerm {
                root: PlaceRoot::Binding(place.binding),
                deref: self.is_holder(place.binding),
                fields: place.fields.clone(),
            })),
            CheckedSetTarget::RunIndex(target) => {
                if matches!(target.place_offset, PlaceOffset::Opaque) {
                    return None;
                }
                let mut path = self.container_root_path(&target.root);
                path.projections
                    .push(PlaceProjection::Subscript(target.place_offset));
                Some(path)
            }
            // No flat element domain names the offset its commit wrote, so
            // none has an element place a measure could be stated over.
            CheckedSetTarget::ArrayIndex(_)
            | CheckedSetTarget::BufferIndex(_)
            | CheckedSetTarget::SliceIndex(_) => None,
        }
    }

    /// [MSR-3] the datums one [LIV-2] commit carries, minted before the
    /// statement's own kills.
    ///
    /// The right-hand side is a bare use of a measured place, which is the
    /// same shape the `let` rebind placement admits: the value keeps every
    /// measure it had and only the name it is reached by changes. Every other
    /// right-hand side mints none, and the ordinary sources establish
    /// whatever that expression publishes.
    fn mint_commit_placement(
        &mut self,
        node_path: &crate::NodePath,
        ordinal: u32,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        state: &mut FactState,
    ) -> Option<MeasureCarry> {
        let CheckedExpression::Binding { binding, ty, .. } = value else {
            return None;
        };
        self.set_target_place(target)?;
        let placement = match target {
            CheckedSetTarget::RunIndex(_) => MeasurePlacement::Element,
            _ => MeasurePlacement::Rebind,
        };
        let source = projected_place(PlaceTerm {
            root: PlaceRoot::Binding(*binding),
            deref: self.is_holder(*binding),
            fields: Vec::new(),
        });
        self.mint_measure_datums(node_path, ordinal, placement, source, *ty, state)
    }

    /// [MSR-3] the construct placement: the datums one `construct`'s field
    /// operands carry into the fields of the value they fill.
    ///
    /// A field whose operand is a bare use of a measured place carries that
    /// place's measures into the field, which is the one event at which a
    /// measured value enters a nominal it did not previously belong to. The
    /// operand shape admitted is the shape every other placement admits: the
    /// value keeps every measure it had and only the place it is reached by
    /// changes.
    fn mint_construct_placements(
        &mut self,
        node_path: &crate::NodePath,
        value: &CheckedExpression,
        state: &mut FactState,
    ) -> Vec<(u32, MeasureCarry)> {
        let fields = match value {
            CheckedExpression::ConstructStruct { fields, .. } => fields,
            // [MSR-3] an enum's payload is a place only where the nominal
            // carries one payload variant, which is what makes the field
            // path select one storage; see `sole_payload_variant`.
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
                ..
            } if self
                .sole_payload_variant(*nominal)
                .is_some_and(|sole| sole.index == *variant) =>
            {
                fields
            }
            _ => return Vec::new(),
        };
        let mut carried = Vec::new();
        for (ordinal, field) in fields.iter().enumerate() {
            let CheckedExpression::Binding { binding, ty, .. } = field else {
                continue;
            };
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            let source = projected_place(PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: self.is_holder(*binding),
                fields: Vec::new(),
            });
            if let Some(carry) = self.mint_measure_datums(
                node_path,
                ordinal,
                MeasurePlacement::Construct,
                source,
                *ty,
                state,
            ) {
                carried.push((ordinal, carry));
            }
        }
        carried
    }

    /// [MSR-3] the construct placement's second half: after the statement's
    /// own kills, field i of the constructed value has the measures its
    /// operand had.
    fn establish_construct_placements(
        &mut self,
        node_path: &crate::NodePath,
        base: &PlaceTerm,
        carried: &[(u32, MeasureCarry)],
        state: &mut FactState,
    ) {
        for (ordinal, carry) in carried {
            let mut destination = base.clone();
            destination.fields.push(*ordinal);
            let destination = projected_place(destination);
            self.establish_measure_datums(node_path, destination, carry, state);
        }
    }

    /// The one variant of a nominal enum that carries fields, where exactly
    /// one does, together with its declared order in the variant list.
    ///
    /// A tracked place's path is field selections, derefs and subscripts
    /// [ENT-2], and none of those steps names a variant: two variants'
    /// payloads are two storages one path cannot separate, so `Result`'s
    /// `Ok(value)` and `Err(error)` would be one place. Where the nominal
    /// carries a single payload variant — the prelude `Option` among them —
    /// the field path selects one storage on every execution and the payload
    /// is an ordinary [MSR-1] measure place. Everything else has no payload
    /// place in this version and carries no measure across the event
    /// [MSR-3].
    fn sole_payload_variant(
        &self,
        nominal: super::super::model::NominalId,
    ) -> Option<SolePayloadVariant> {
        let CheckedNominalKind::Enum { variants } =
            &self.context.nominals.get(nominal.0 as usize)?.kind
        else {
            return None;
        };
        let mut carrying = variants
            .iter()
            .enumerate()
            .filter(|(_, variant)| !variant.fields.is_empty());
        let (index, variant) = carrying.next()?;
        if carrying.next().is_some() {
            return None;
        }
        Some(SolePayloadVariant {
            index: u32::try_from(index).ok()?,
            tag: variant.tag,
            fields: variant.fields.clone(),
        })
    }

    /// [MSR-3] the payload placement: the datums a `match` over an own enum
    /// place carries out of that place's payload.
    ///
    /// The `match` consumes the scrutinee, so a measure of the payload dies
    /// with it; the datum minted here is the value that measure had
    /// immediately before the consume, and the arm binder that names the
    /// payload receives it on its own arm.
    fn mint_payload_placements(
        &mut self,
        scrutinee: &CheckedExpression,
        enum_type: CheckedEnumType,
        state: &mut FactState,
    ) -> Option<PayloadPlacement> {
        let CheckedExpression::Binding {
            carrier: node_path,
            binding,
            ..
        } = scrutinee
        else {
            return None;
        };
        let CheckedEnumType::Nominal(nominal) = enum_type else {
            return None;
        };
        let sole = self.sole_payload_variant(nominal)?;
        let base = PlaceTerm {
            root: PlaceRoot::Binding(*binding),
            deref: self.is_holder(*binding),
            fields: Vec::new(),
        };
        let mut carried = Vec::new();
        for (ordinal, field) in sole.fields.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            let mut source = base.clone();
            source.fields.push(ordinal);
            if let Some(carry) = self.mint_measure_datums(
                node_path,
                ordinal,
                MeasurePlacement::Payload,
                projected_place(source),
                field.ty,
                state,
            ) {
                carried.push((ordinal, carry));
            }
        }
        (!carried.is_empty()).then_some(PayloadPlacement {
            tag: sole.tag,
            carried,
        })
    }

    /// [MSR-3] the destructuring placement: the datums a destructuring
    /// consume carries out of the fields it takes apart.
    ///
    /// The operand is a bare use of a measured nominal place — `let N(f: a)
    /// = move v;` — and binder i takes the measures of `v`'s field i. A
    /// `let (a, b) = f(...)` binder list has no such operand and mints
    /// nothing; its ordinals are [CALL-4] destinations instead.
    fn mint_destructuring_placements(
        &mut self,
        node_path: &crate::NodePath,
        bindings: &[(BindingId, CheckedType)],
        value: &CheckedExpression,
        state: &mut ProofFlowState,
    ) -> Vec<(u32, MeasureCarry)> {
        let CheckedExpression::Binding { binding, .. } = value else {
            return Vec::new();
        };
        let base = PlaceTerm {
            root: PlaceRoot::Binding(*binding),
            deref: self.is_holder(*binding),
            fields: Vec::new(),
        };
        let mut carried = Vec::new();
        for (ordinal, (_, ty)) in bindings.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
            let mut source = base.clone();
            source.fields.push(ordinal);
            if let Some(carry) = self.mint_measure_datums(
                node_path,
                ordinal,
                MeasurePlacement::Destructuring,
                projected_place(source),
                *ty,
                &mut state.facts,
            ) {
                carried.push((ordinal, carry));
            }
        }
        carried
    }

    /// The [OWN-5] resolved place that root names.
    fn container_root_place(&self, root: &CheckedContainerRoot) -> ResolvedPlace {
        let path = self.container_root_path(root);
        self.resolve_projected(&path)
    }

    /// [ENT-6]: the bounds obligation `i < len_of(P)`, normalized
    /// `i - len_of(P) <= -1`, discharged exactly when the closed fact state at
    /// the node derives it.
    fn judge_obligation(
        &mut self,
        base: ProjectedPlaceTerm,
        measured: MeasuredKind,
        array_length: Option<CheckedConst>,
        offset: &CheckedExpression,
        node_path: crate::NodePath,
        states: &ProofFlowState,
    ) {
        // [OP-4] the obligation is against `len_of(p)` in logical coordinates
        // [MSR-1], never against `cap_of(p)`.
        let length_term =
            self.place_measure_term(CheckedMeasure::Length, base.clone(), measured, array_length);
        let offset_term = self.read_operand(offset);
        let affine_offset = self
            .direct_goal_expression(offset)
            .and_then(|offset| self.affine_goal_value(&offset, &states.affine));
        // [MSR-4] the subscript submits its own normalized target to the one
        // disposition, so steps 4 and 5 range over the measure's own affine
        // atom instead of being reachable only through the L0-right bridge.
        let direct_affine = affine_offset.as_ref().and_then(|offset| {
            let length = self.measure_atom(length_term);
            let mut check = AffineCheckState::new();
            AffineInequality::from_bounded_forms(offset, &length, -1, &mut check).ok()
        });
        let fixed_array_affine =
            self.affine_fixed_array_index_target(offset, array_length, &states.affine);
        let rendered_residual = format!(
            "{} < len_of({})",
            self.render_expression(offset),
            self.render_projected_place(&base)
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
                request: Some(request),
                direct_affine: direct_affine.as_ref(),
                fixed_affine_bridge,
                affine_left: affine_offset.as_ref(),
            }),
        );
        let discharged = proof.disposition == ProofDisposition::Proved;
        let refuted = proof.disposition == ProofDisposition::Refuted;
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
            refuted,
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
            kernel_row: None,
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
        &mut self,
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

    /// Judges OP-9 through either the exact total `buffer_fits::<T>(n)` goal
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
        let length_goal = self.obligation_goal_operand(&node_path, 0, length, &states.facts);
        let canonical_goal = GoalExpression::Operation {
            row: GoalOperation::BufferFits {
                element,
                maximum_length,
            },
            type_arguments: vec![element],
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: vec![length_goal],
        };
        let goal = Some(self.intern_goal_expression(canonical_goal.clone()));
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
            .admitted_value_goal_expression(length)
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
            "buffer_fits::<{:?}>({})",
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
            canonical_goal: Some(canonical_goal),
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
            kernel_row: None,
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
        // [SYS-8] the row's own range-bearing parameter: the operand
        // class this row writes or reads, whose `len_of` the second
        // obligation is stated over.
        let Some((buffer_ordinal, buffer_parameter)) =
            row.parameters.iter().enumerate().find(|(_, parameter)| {
                matches!(
                    parameter.ty,
                    crate::SystemTypeRef::DestinationU8 | crate::SystemTypeRef::SourceU8
                )
            })
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
        let start_goal =
            self.obligation_goal_operand(node_path, start_ordinal, start, &states.facts);
        let end_goal = self.obligation_goal_operand(node_path, end_ordinal, end, &states.facts);
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

        self.judge_exact_relation_obligation(
            ObligationFamily::SystemRange,
            0,
            node_path.clone(),
            comparison(start_goal, end_goal.clone()),
            start_term,
            end_term,
            format!(
                "{} <= {}",
                self.render_expression(start),
                self.render_expression(end)
            ),
            states,
        );

        // The second conjunct bounds the end against the caller's own buffer,
        // so the residual names the caller's place — `wide <= len_of(header)` —
        // the way [OP-4]'s bounds residual does. Printing the operation's
        // declared parameter name instead leaves a writer with two buffers in
        // scope unable to tell which one the bound is about. The declared name
        // remains the fallback for an argument that carries no place at all.
        let buffer_root = match buffer {
            CheckedExpression::BorrowBuffer { root, .. } => {
                Some((root.binding, root.fields.clone()))
            }
            CheckedExpression::Binding {
                binding,
                ty: CheckedType::Slice { .. },
                ..
            } => Some((*binding, Vec::new())),
            _ => None,
        };
        let buffer_spelling = match &buffer_root {
            Some((binding, fields)) => self.render_place(&PlaceTerm {
                root: PlaceRoot::Binding(*binding),
                deref: self.is_holder(*binding),
                fields: fields.clone(),
            }),
            None => buffer_parameter.name.to_owned(),
        };
        // [SYS-8] the obligation is `end <= len_of(<the range-bearing
        // operand>)`, stated over the measure-table row that operand's own
        // type has [MSR-1]: a view is measured as a view and a `buffer` as a
        // buffer, and neither reading is the other's.
        let range_type = buffer.ty();
        let (measure_row, measured_kind) = match range_type {
            CheckedType::Buffer { element } => (
                GoalOperation::BufferMeasure {
                    measure: CheckedMeasure::Length,
                    element,
                },
                MeasuredKind::Buffer,
            ),
            CheckedType::Slice {
                region, element, ..
            } => (
                GoalOperation::SliceMeasure {
                    measure: CheckedMeasure::Length,
                    region,
                    element,
                },
                MeasuredKind::Slice,
            ),
            _ => return,
        };
        let buffer_goal = match buffer_root.as_ref() {
            None => self.obligation_goal_operand(node_path, buffer_ordinal, buffer, &states.facts),
            Some((buffer_binding, buffer_fields)) => self.goal_binding_place(
                *buffer_binding,
                buffer_fields.iter().copied().map(GoalProjection::Field),
                range_type,
            ),
        };
        let length_goal = GoalExpression::Operation {
            row: measure_row,
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Integer(IntegerType::U64),
            arguments: vec![buffer_goal],
        };
        let length_term = buffer_root.map(|(buffer_binding, buffer_fields)| {
            let base = PlaceTerm {
                root: PlaceRoot::Binding(buffer_binding),
                deref: self.is_holder(buffer_binding),
                fields: buffer_fields,
            };
            self.place_measure_term(
                CheckedMeasure::Length,
                projected_place(base),
                measured_kind,
                None,
            )
        });
        self.judge_exact_relation_obligation(
            ObligationFamily::SystemRange,
            1,
            node_path.clone(),
            comparison(end_goal, length_goal),
            end_term,
            length_term,
            format!(
                "{} <= len_of({buffer_spelling})",
                self.render_expression(end)
            ),
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
        right: Option<TermId>,
        residual: String,
        states: &ProofFlowState,
    ) {
        let canonical_goal = root.clone();
        let request = left.zip(right).map(|(left, right)| BoundsRequest {
            left: Some(left),
            right,
            bound: 0,
            distinct: false,
        });
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
            components: request.into_iter().collect(),
            discharged,
            refuted,
            contradictory,
            residual: (!discharged).then(|| residual.clone()),
            derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
            kernel_row: None,
        });
    }

    fn affine_term_value(&mut self, term: TermId, state: &AffineFlowState) -> Option<AffineForm> {
        match self.terms.kind(term).clone() {
            TermKind::Zero => Some(AffineForm::constant(0)),
            TermKind::Constant(value) => Some(AffineForm::constant(value)),
            TermKind::Place(
                PlaceTerm {
                    root: PlaceRoot::Binding(binding),
                    deref: false,
                    fields,
                },
                _,
            ) if fields.is_empty() => state.values.get(&binding).cloned(),
            // [MSR-4] a measure term's image is its own compiler-owned atom,
            // and [MSR-3] a measure datum inherits the atom of the term it
            // denotes.
            TermKind::Measure(..)
            | TermKind::ProjectedMeasure(..)
            | TermKind::EntryDatum { .. }
            | TermKind::MeasureDatum { .. }
            | TermKind::CallDatum {
                measure: Some(_), ..
            } => Some(self.measure_atom(term)),
            TermKind::ConstParameter(_)
            | TermKind::Place(_, _)
            | TermKind::ProjectedPlace(_, _)
            | TermKind::CountedCapture { .. }
            | TermKind::CommitValue { .. }
            | TermKind::CallDatum { .. } => None,
        }
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
        let canonical_goal =
            self.integer_domain_goal(operation, operand_type, arguments, node_path, &states.facts);
        let goal = Some(self.intern_goal_expression(canonical_goal.clone()));
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
        // Both records below describe this walk of this operation. A loop body
        // is walked more than once and the same node then carries different
        // operand values each time, so the previous walk's measurement is
        // dropped before this one decides: a judgment that does not discharge
        // must leave nothing behind for the binding to read.
        self.product_intervals.remove(node_path);
        self.product_operands.remove(node_path);
        // [ENT-3.S14] publishes only what an admitted multiplication proved,
        // so the interval is retained exactly when this obligation discharged
        // through the interval-product route.
        if discharged && let Some(interval) = outcome.product_interval.clone() {
            self.product_intervals.insert(node_path.clone(), interval);
        }
        // That the exact multiplication's domain held, for [PRF-1] to fold a
        // term-scaled premise against. Recorded only when the domain
        // discharged through an affine route, which is what committed
        // `prepared_affine` and so fixed the images the judgment read.
        if discharged
            && outcome.route == Some(ProofRoute::Affine)
            && operation == CheckedIntegerOperation::MultiplyExact
            && affine_product.is_some()
        {
            self.product_operands.insert(node_path.clone());
        }
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
            canonical_goal: Some(canonical_goal),
            components,
            discharged,
            refuted,
            contradictory,
            residual: (!discharged).then_some(residual),
            derivation: outcome.derivation,
            allocation_length_upper_bound: None,
            allocation_length_upper_bound_derivation: None,
            affine_index_maps: Vec::new(),
            kernel_row: None,
        });
    }

    fn integer_domain_goal(
        &mut self,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: &[CheckedExpression],
        node_path: &crate::NodePath,
        facts: &FactState,
    ) -> GoalExpression {
        GoalExpression::Operation {
            row: GoalOperation::Integer {
                operation: operation
                    .defined_query()
                    .expect("every proof-required exact row has one total domain query"),
                operand_type,
            },
            type_arguments: Vec::new(),
            const_arguments: Vec::new(),
            result: CheckedType::Bool,
            arguments: arguments
                .iter()
                .enumerate()
                .map(|(ordinal, argument)| {
                    self.obligation_goal_operand(node_path, ordinal, argument, facts)
                })
                .collect(),
        }
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
                product_interval: None,
            };
        }

        if let Some((derivation, interval)) = goal.affine_product.and_then(|product| {
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
                // [ENT-3.S14] establishes this interval on whatever value the
                // multiplication binds. It travels with the judgment because
                // only this route proved it: a domain discharged by the finite
                // L0 or affine-clause route publishes no product interval.
                product_interval: Some(interval),
            };
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
            product_interval: None,
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
        if contradictory {
            let parents = closed.contradiction_proof().map(|proof| vec![proof]);
            let Some(parents) = parents else {
                unreachable!("a contradictory closure retains its proof");
            };
            let derivation = self.derivations.intern(DerivationNode::IntegerDomain {
                goal: goal.canonical,
                parents,
            });
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::Contradiction),
                derivation: Some(derivation),
                numeric_upper_bound: None,
                product_interval: None,
            };
        }

        if let Some(canonical) = goal.canonical {
            if closed.holds_opaque(canonical, GoalSign::Positive) {
                let parent = closed
                    .opaque_proof(canonical, GoalSign::Positive)
                    .expect("an opaque goal fact retains its proof");
                let derivation = self.derivations.intern(DerivationNode::IntegerDomain {
                    goal: Some(canonical),
                    parents: vec![parent],
                });
                return ProofResult {
                    disposition: ProofDisposition::Proved,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: Some(derivation),
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
            if closed.holds_opaque(canonical, GoalSign::Negative) {
                return ProofResult {
                    disposition: ProofDisposition::Refuted,
                    route: Some(ProofRoute::FiniteGoal),
                    derivation: None,
                    numeric_upper_bound: None,
                    product_interval: None,
                };
            }
        }

        let normalization_parents = if signed_division && goal.components.len() == 3 {
            component_proof(0, &mut self.derivations).and_then(|nonzero| {
                component_proof(1, &mut self.derivations)
                    .or_else(|| component_proof(2, &mut self.derivations))
                    .map(|witness| vec![nonzero, witness])
            })
        } else if !goal.components.is_empty() {
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
        if let Some(parents) = normalization_parents {
            let parents = if let Some(canonical) = goal.canonical {
                if let Some(normalization) = closed.normalization_proof(
                    canonical,
                    GoalSign::Positive,
                    &self.goals,
                    &mut self.derivations,
                ) {
                    vec![normalization]
                } else {
                    // A complete admitted Goal may expand an ordinary-let
                    // operand into an exact operation that is not an L0 term.
                    // Its source occurrence can still have fixed L0
                    // components over the already evaluated alias. Those
                    // occurrence-local parents prove this IntegerDomain
                    // judgment directly; they must not become a normalization
                    // on the globally interned complete Goal identity.
                    parents
                }
            } else {
                parents
            };
            let derivation = self.derivations.intern(DerivationNode::IntegerDomain {
                goal: goal.canonical,
                parents,
            });
            return ProofResult {
                disposition: ProofDisposition::Proved,
                route: Some(ProofRoute::L0),
                derivation: Some(derivation),
                numeric_upper_bound: None,
                product_interval: None,
            };
        }

        let component_false = |index: usize| {
            goal.components
                .get(index)
                .and_then(request_relation)
                .is_some_and(|relation| closed.derives(&relation.negated()))
        };
        let normalization_refuted = if signed_division && goal.components.len() == 3 {
            component_false(0) || (component_false(1) && component_false(2))
        } else {
            goal.components
                .iter()
                .filter_map(request_relation)
                .any(|relation| closed.derives(&relation.negated()))
        };
        if normalization_refuted {
            return ProofResult {
                disposition: ProofDisposition::Refuted,
                route: Some(ProofRoute::L0),
                derivation: None,
                numeric_upper_bound: None,
                product_interval: None,
            };
        }

        ProofResult {
            disposition: ProofDisposition::Unknown,
            route: None,
            derivation: None,
            numeric_upper_bound: None,
            product_interval: None,
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
    ) -> Option<(DerivationId, AffineProductInterval)> {
        let interval = self.affine_integer_product_interval(product, affine, facts)?;
        let derivation = self.derivations.intern(DerivationNode::IntegerDomain {
            goal,
            parents: interval.consequences.to_vec(),
        });
        Some((derivation, interval))
    }

    /// The one measurement the fixed interval-product rule performs. The four
    /// endpoint products decide [ENT-6]'s domain admission and bound
    /// [ENT-3.S14]'s published interval, so both read this result rather than
    /// proving the same endpoints twice: the admitted range and the published
    /// bound then cannot disagree by construction.
    fn affine_integer_product_interval(
        &mut self,
        product: &AffineIntegerProduct,
        affine: &AffineFlowState,
        facts: &FactState,
    ) -> Option<AffineProductInterval> {
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
        // The extrema of a product over two inclusive intervals occur among
        // exactly these four pairs, so the tightest interval the rule can
        // state is their own minimum and maximum.
        let minimum = *products.iter().min()?;
        let maximum = *products.iter().max()?;

        let consequences: Vec<DerivationId> = [
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
        Some(AffineProductInterval {
            minimum,
            maximum,
            consequences: consequences.into_boxed_slice(),
        })
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

        for assumption in Self::canonical_affine_facts(assumptions) {
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

    fn judge_set_target(&mut self, target: &CheckedSetTarget, states: &mut ProofFlowState) -> bool {
        match target {
            CheckedSetTarget::Place(_) => true,
            CheckedSetTarget::ArrayIndex(target) => {
                let reaches_target =
                    self.judge_children_reach_parent(std::iter::once(&target.offset), states);
                let obligation_start = self.obligations.len();
                if reaches_target {
                    let base = PlaceTerm {
                        root: PlaceRoot::Binding(target.binding),
                        deref: self.is_holder(target.binding),
                        fields: target.fields.clone(),
                    };
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Array,
                        Some(target.length),
                        &target.offset,
                        target.obligation.clone(),
                        states,
                    );
                }
                reaches_target && self.obligations_since_discharged(obligation_start)
            }
            CheckedSetTarget::BufferIndex(target) => {
                let reaches_target =
                    self.judge_children_reach_parent(std::iter::once(&target.offset), states);
                let obligation_start = self.obligations.len();
                if reaches_target {
                    let base = PlaceTerm {
                        root: PlaceRoot::Binding(target.root.binding),
                        deref: self.is_holder(target.root.binding),
                        fields: target.root.fields.clone(),
                    };
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Buffer,
                        None,
                        &target.offset,
                        target.obligation.clone(),
                        states,
                    );
                }
                reaches_target && self.obligations_since_discharged(obligation_start)
            }
            // [OP-4] a view element target owes the same bound its read owes:
            // `i < len_of(view)`, over the view's own measure row [MSR-1].
            CheckedSetTarget::SliceIndex(target) => {
                let reaches_target =
                    self.judge_children_reach_parent(std::iter::once(&target.offset), states);
                let obligation_start = self.obligations.len();
                if reaches_target {
                    let base = PlaceTerm {
                        root: PlaceRoot::Binding(target.root.binding),
                        deref: self.is_holder(target.root.binding),
                        fields: Vec::new(),
                    };
                    self.judge_obligation(
                        projected_place(base),
                        MeasuredKind::Slice,
                        None,
                        &target.offset,
                        target.obligation.clone(),
                        states,
                    );
                }
                reaches_target && self.obligations_since_discharged(obligation_start)
            }
            // [OP-4, BLK-1] the run's own obligation is `i < len_of(v)`: the
            // offset is a logical one and the window's length bounds it, so
            // the measured kind is the run's and the written capacity is not
            // the bound.
            CheckedSetTarget::RunIndex(target) => {
                let reaches_target =
                    self.judge_children_reach_parent(std::iter::once(&target.offset), states);
                let obligation_start = self.obligations.len();
                if reaches_target && let Some(measured) = target.root.measured() {
                    let base = self.container_root_path(&target.root);
                    self.judge_obligation(
                        base,
                        measured,
                        target.root.type_constant(),
                        &target.offset,
                        target.obligation.clone(),
                        states,
                    );
                }
                reaches_target && self.obligations_since_discharged(obligation_start)
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
        self.new_affine_atom_with_interval(ty, minimum, maximum, false)
    }

    fn new_affine_atom_with_interval(
        &mut self,
        ty: IntegerType,
        minimum: i128,
        maximum: i128,
        join_delta: bool,
    ) -> AffineForm {
        let index = u32::try_from(self.affine_atoms.len())
            .expect("affine value atoms exceed the u32 identity space");
        self.affine_atoms.push(AffineAtom {
            ty,
            minimum,
            maximum,
            join_delta,
        });
        AffineForm::term(AffineTermId::from_index(index))
    }

    /// Folds every delta atom an earlier join minted back into the constant
    /// interval it stands for [ENT-6].
    ///
    /// Without this, a delta atom counts as an ordinary nonconstant term at
    /// the next join, so two joins in sequence lose an image one join over the
    /// same branches keeps: acceptance would depend on whether the writer
    /// spelled the branch set as nested conditionals or as one flat match.
    fn fold_join_deltas(&self, value: &AffineForm) -> Option<FoldedJoinImage> {
        let mut form = value.nonconstant_part();
        let mut minimum = value.constant_value();
        let mut maximum = minimum;
        let mut check = AffineCheckState::new();
        for coefficient in value.terms() {
            let atom = self.affine_atoms.get(coefficient.term().index() as usize)?;
            if !atom.join_delta {
                continue;
            }
            let low = coefficient.coefficient().checked_mul(atom.minimum)?;
            let high = coefficient.coefficient().checked_mul(atom.maximum)?;
            minimum = minimum.checked_add(low.min(high))?;
            maximum = maximum.checked_add(low.max(high))?;
            let folded = AffineForm::term(coefficient.term())
                .scale(coefficient.coefficient(), &mut check)
                .ok()?;
            form = form.subtract(&folded, &mut check).ok()?;
        }
        Some(FoldedJoinImage {
            form,
            minimum,
            maximum,
        })
    }

    fn new_affine_binding_atom(&mut self, binding: BindingId) -> Option<AffineForm> {
        let ty = self.affine_binding_type(binding)?;
        Some(self.new_affine_atom(ty))
    }

    /// The one atom that stands for a binding's whole value, for the length of
    /// one certificate.
    ///
    /// A binding whose image is already a single atom is its own handle and
    /// mints nothing. Otherwise a fresh atom is minted and the image it stands
    /// for is remembered, so the handle can be unfolded again before anything
    /// is proved.
    ///
    /// [PRF-1]'s fold needs this because a local's image is transparent by
    /// design. `let stride = width + padding; let base = stride * row;` gives
    /// the product the operands `width + padding` and `row`, so a certificate
    /// scaling by `stride` distributes into pieces no admitted multiplication
    /// matches. Naming the binding on both sides — the product it forms and
    /// the multiplicity that scales by it — is what makes the two agree, and
    /// it is the rule [PRF-1] already states one sentence away for a named
    /// premise: resolve by the declaration the writer wrote, not by whatever
    /// that declaration currently expands to.
    ///
    /// The handle is deliberately not published as a fact and does not replace
    /// the binding's image. Both were tried and both cost more than they
    /// bought: a published equality is invisible to the residual, which is the
    /// direct L0 route by rule, and replacing the image makes every ordinary
    /// premise about the binding need that equality to prove. Keeping it a
    /// name that exists between the fold and the residual leaves the rest of
    /// the checker reading exactly what it read before.
    fn affine_opaque_handle(
        &mut self,
        binding: BindingId,
        state: &mut AffineFlowState,
    ) -> Option<AffineForm> {
        if let Some(image) = state.values.get(&binding)
            && image.unit_term().is_some()
        {
            return Some(image.clone());
        }
        if let Some(handle) = state.opaque_values.get(&binding) {
            return Some(handle.clone());
        }
        let handle = self.new_affine_binding_atom(binding)?;
        let Some(image) = state.values.get(&binding).cloned() else {
            state.values.insert(binding, handle.clone());
            state.opaque_values.insert(binding, handle.clone());
            return Some(handle);
        };
        let atom = handle.unit_term()?;
        self.handle_images.insert(atom, image);
        state.opaque_values.insert(binding, handle.clone());
        Some(handle)
    }

    /// Intersects every published affine fact by canonical numeric content,
    /// then records representative predecessor evidence for diagnostics.
    ///
    /// The first phase deliberately does not inspect evidence or the source
    /// category that published a fact: an inequality survives exactly when
    /// every predecessor contains the same relation over the same immutable
    /// value images. Loop dependencies are then unioned conservatively, so a
    /// cross-category match cannot carry an assumption beyond its loop.
    fn join_affine_facts(&mut self, states: &[ProofFlowState]) -> Vec<ActiveAffineFact> {
        let Some(first) = states.first() else {
            return Vec::new();
        };

        // This is deliberately an ordered intersection, not a set iteration:
        // the first contributing structural predecessor fixes the retained
        // order, and the first canonical occurrence in that predecessor fixes
        // each fact's position. Evidence category never participates.
        let mut common_inequalities = Vec::new();
        for candidate in &first.affine.facts {
            if common_inequalities.contains(&candidate.inequality) {
                continue;
            }
            if states.iter().skip(1).all(|state| {
                state
                    .affine
                    .facts
                    .iter()
                    .any(|fact| fact.inequality == candidate.inequality)
            }) {
                common_inequalities.push(candidate.inequality.clone());
            }
        }

        common_inequalities
            .into_iter()
            .map(|inequality| {
                let witnesses = states
                    .iter()
                    .map(|state| {
                        state
                            .affine
                            .facts
                            .iter()
                            .filter(|fact| fact.inequality == inequality)
                            // A stable witness is strictly preferable to an
                            // active-loop assumption for the same canonical
                            // fact. Remaining ties keep deterministic fact
                            // insertion order and do not affect acceptance.
                            .min_by_key(|fact| fact.active_loops.len())
                            .expect("a common affine inequality has one witness")
                    })
                    .collect::<Vec<_>>();

                let mut active_loops = witnesses
                    .iter()
                    .flat_map(|fact| fact.active_loops.iter().copied())
                    .collect::<Vec<_>>();
                active_loops.sort_unstable_by_key(|loop_id| loop_id.0);
                active_loops.dedup();

                let first_evidence = witnesses[0].evidence;
                let evidence = if witnesses.iter().all(|fact| fact.evidence == first_evidence) {
                    first_evidence
                } else if witnesses
                    .iter()
                    .all(|fact| matches!(fact.evidence, AffineFactEvidence::Source(_)))
                {
                    let predecessors = witnesses
                        .iter()
                        .map(|fact| match fact.evidence {
                            AffineFactEvidence::Source(source) => source,
                            AffineFactEvidence::Derivation(_) => unreachable!(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let join_ordinal = u32::try_from(self.joined_source_proofs.len())
                        .expect("joined affine fact count exceeds the u32 identity space");
                    self.joined_source_proofs
                        .push(JoinedSourceProofProvenance { predecessors });
                    AffineFactEvidence::Source(SourceAffineFactRef::JoinedSourceProof {
                        join_ordinal,
                    })
                } else {
                    // Derivation and source identities are explanation only.
                    // The canonical all-predecessor intersection above is the
                    // sole authority for retaining this conclusion.
                    first_evidence
                };
                ActiveAffineFact {
                    inequality,
                    evidence,
                    active_loops,
                }
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
                // [ENT-6] every input is normalized before the comparison: a
                // delta atom an earlier join minted folds back into the
                // constant interval it stands for, so what is compared is the
                // part of the image no join invented. Nested joins therefore
                // reach exactly the image one flat join over the same branches
                // reaches, and acceptance stops depending on the join shape.
                let folded = states
                    .iter()
                    .map(|state| {
                        state
                            .affine
                            .values
                            .get(&binding)
                            .and_then(|value| self.fold_join_deltas(value))
                    })
                    .collect::<Option<Vec<_>>>();
                match folded {
                    Some(folded)
                        if folded
                            .iter()
                            .skip(1)
                            .all(|image| image.form == folded[0].form) =>
                    {
                        let minimum = folded
                            .iter()
                            .map(|image| image.minimum)
                            .min()
                            .expect("one join input exists");
                        let maximum = folded
                            .iter()
                            .map(|image| image.maximum)
                            .max()
                            .expect("one join input exists");
                        let atom_start = self.affine_atoms.len();
                        let delta = self.new_affine_atom_with_interval(ty, minimum, maximum, true);
                        match folded[0].form.add(&delta, &mut AffineCheckState::new()) {
                            Ok(value) => value,
                            Err(_) => {
                                self.affine_atoms.truncate(atom_start);
                                self.new_affine_atom(ty)
                            }
                        }
                    }
                    _ => self.new_affine_atom(ty),
                }
            } else {
                continue;
            };
            values.insert(binding, value);
        }
        // An opaque handle is a convenience for one certificate, not a fact,
        // so a join keeps none: the next demand re-mints against whatever the
        // joined image is.
        let opaque_values: HashMap<BindingId, AffineForm> = HashMap::new();

        AffineFlowState {
            values,
            opaque_values,
            facts: self.join_affine_facts(states),
            published_invariants: first
                .affine
                .published_invariants
                .iter()
                .filter(|(declaration, inequality)| {
                    states.iter().skip(1).all(|state| {
                        state.affine.published_invariants.get(declaration) == Some(*inequality)
                    })
                })
                .map(|(declaration, inequality)| (*declaration, inequality.clone()))
                .collect(),
        }
    }

    fn join_flows(&mut self, states: &[ProofFlowState]) -> ProofFlowState {
        // Close L0 contradiction before any structural intersection. An
        // unreachable predecessor is neutral: it cannot erase a live affine
        // value, published invariant name, or canonical fact. Keeping all
        // promoted L0 states in `join_at` still records their contradiction
        // proofs as join parents. When every predecessor is contradictory,
        // the affine component is deliberately empty because L0 proves every
        // downstream goal.
        let mut promoted = states.to_vec();
        for state in &mut promoted {
            self.promote_flow_contradiction(state);
        }
        let contributing = promoted
            .iter()
            .filter(|state| !state.facts.all_derivable)
            .cloned()
            .collect::<Vec<_>>();
        let facts = promoted
            .iter()
            .map(|states| states.facts.clone())
            .collect::<Vec<_>>();
        let event = self.derivations.event(FlowEventKind::Join, None);
        let entry_images = (0..self.entry_images.len())
            .map(|index| {
                contributing
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
            affine: self.join_affine_states(&contributing),
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
            Relation::Equal {
                left,
                right,
                difference,
            } => Relation::Equal {
                left: replace(*left),
                right: replace(*right),
                difference: *difference,
            },
            Relation::Distinct {
                left,
                right,
                difference,
            } => {
                let (left, right) = (replace(*left), replace(*right));
                // Ordering the pair reverses the difference with it.
                if left <= right {
                    Relation::Distinct {
                        left,
                        right,
                        difference: *difference,
                    }
                } else {
                    Relation::Distinct {
                        left: right,
                        right: left,
                        difference: -difference,
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
                difference: 0,
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

    /// The dividend's affine value image of one admitted S7 unsigned
    /// division, read where the division was evaluated. A `set` commit must
    /// read it before its own target kill, since a dividend naming the target
    /// place has a different image afterwards.
    fn unsigned_division_dividend_form(
        &mut self,
        value: &CheckedExpression,
        state: &mut AffineFlowState,
    ) -> Option<AffineForm> {
        let CheckedExpression::IntegerOperation { arguments, .. } = value else {
            return None;
        };
        let [dividend, _divisor] = arguments.as_slice() else {
            return None;
        };
        self.affine_pre_domain_form(dividend, state)
    }

    /// Records what one admitted exact multiplication's bound value equals.
    ///
    /// The domain judgment already measured the operands where the product was
    /// formed; this pairs that measurement with the atom the binding took, so
    /// [PRF-1] can recognize `n*p` in a certificate sum as the value `base`
    /// already holds. A product whose result image is not one atom — a
    /// conversion, a further operation — records nothing, because there is
    /// then no single value the monomial equals.
    fn record_product_atom(
        &mut self,
        binding: BindingId,
        value: &CheckedExpression,
        state: &mut AffineFlowState,
    ) {
        let CheckedExpression::IntegerOperation {
            carrier, arguments, ..
        } = value
        else {
            return;
        };
        if !self.product_operands.contains(carrier) {
            return;
        }
        let [left, right] = arguments.as_slice() else {
            return;
        };
        // What proved the domain and what the fold names are two questions.
        // The domain judgment reads the transparent images, whose intervals are
        // what admit the multiply at all; the record names the bindings, so a
        // certificate scaling by one of them meets the same value here. Reading
        // handles at the domain site instead was tried and costs the interval:
        // an opaque operand is only bounded by its type, and the four endpoint
        // products then leave the range.
        let (Some(left), Some(right)) = (
            self.affine_operand_handle(left, state),
            self.affine_operand_handle(right, state),
        ) else {
            return;
        };
        let Some(product) = state.values.get(&binding).and_then(AffineForm::unit_term) else {
            return;
        };
        self.product_atoms
            .insert(product, (left.min(right), left.max(right)));
    }

    /// The atom a multiplication's operand contributes to the fold: the
    /// binding's opaque handle when the operand is a plain read of one, and
    /// nothing otherwise.
    fn affine_operand_handle(
        &mut self,
        operand: &CheckedExpression,
        state: &mut AffineFlowState,
    ) -> Option<AffineTermId> {
        let CheckedExpression::Binding { binding, ty, .. } = operand else {
            return None;
        };
        let CheckedType::Integer(integer) = *ty else {
            return None;
        };
        if self.affine_binding_type(*binding) != Some(integer) {
            return None;
        }
        self.affine_opaque_handle(*binding, state)?.unit_term()
    }

    /// Retains the second fixed consequence of one S7 unsigned division:
    /// for `q = a / k` with a positive written literal `k`, `k*q <= a`.
    /// Both sides are the exact affine value images computed at this program
    /// point, so later writes receive different atoms and cannot inherit it.
    fn establish_unsigned_division_image(
        &mut self,
        quotient: &AffineForm,
        dividend: &AffineForm,
        established: sources::EstablishedUnsignedDivision,
        state: &mut AffineFlowState,
    ) {
        let Ok(scaled_quotient) = quotient.scale(established.divisor, &mut AffineCheckState::new())
        else {
            return;
        };
        let Some(inequality) = Self::affine_less_equal(&scaled_quotient, dividend) else {
            return;
        };
        state.facts.push(ActiveAffineFact {
            inequality,
            evidence: AffineFactEvidence::Derivation(established.parent),
            active_loops: Vec::new(),
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
    ) -> Result<AffineForm, AffineCheckError> {
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
                            let value = self
                                .new_affine_binding_atom(*binding)
                                .ok_or(AffineCheckError::CoefficientMismatch)?;
                            state.values.insert(*binding, value.clone());
                            value
                        };
                        values.push(value);
                    }
                    // [INV-1, MSR-2] a measure factor's image is the one this
                    // program point holds for that term. It is retargeted by
                    // exactly the events that kill the term, so a relation
                    // proved before a write says nothing after it.
                    CheckedAffineExpressionKind::Measure(measure) => {
                        let term = self
                            .checked_measure_term(measure)
                            .ok_or(AffineCheckError::CoefficientMismatch)?;
                        values.push(self.measure_atom(term));
                    }
                    // [INV-1, MSR-6, ENT-2] a const generic at the symbolic
                    // instance is the declaration-anchored constant term, and
                    // no [ENT-5] event kills it, so its image is one
                    // immutable atom for the whole walk.
                    CheckedAffineExpressionKind::ConstGeneric { declaration, .. } => {
                        let term = self.terms.intern(TermKind::ConstParameter(*declaration));
                        values.push(self.measure_atom(term));
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
                    let right = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                    let left = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                    values.push(left.add(&right, check)?);
                }
                Pending::Subtract => {
                    let right = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                    let left = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                    values.push(left.subtract(&right, check)?);
                }
                Pending::Scale(constant) => {
                    let value = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
                    values.push(value.scale(constant, check)?);
                }
            }
        }
        let result = values.pop().ok_or(AffineCheckError::CoefficientMismatch)?;
        if values.is_empty() {
            Ok(result)
        } else {
            Err(AffineCheckError::CoefficientMismatch)
        }
    }

    fn checked_loop_invariant_inequality(
        &mut self,
        invariant: &CheckedLoopInvariant,
        state: &mut AffineFlowState,
        check: &mut AffineCheckState,
    ) -> Option<AffineInequality> {
        self.checked_affine_relation_inequality(&invariant.relation, state, check)
            .ok()
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
            let target = self.checked_affine_relation_inequality(
                &invariant.relation,
                state,
                &mut AffineCheckState::new(),
            );
            self.invariant_targets
                .insert(invariant.declaration, target.clone());
            if base_batch && let Ok(inequality) = target {
                state
                    .published_invariants
                    .insert(invariant.declaration, inequality.clone());
                state.facts.push(ActiveAffineFact {
                    inequality,
                    evidence: AffineFactEvidence::Source(SourceAffineFactRef::LoopInvariant(
                        SourceLoopInvariantRef {
                            loop_id,
                            source_ordinal: u32::try_from(source_ordinal)
                                .expect("loop invariant ordinal exceeds u32"),
                        },
                    )),
                    active_loops: vec![loop_id],
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
        counted_binder: Option<BindingId>,
    ) {
        for (index, invariant) in invariants.iter().enumerate() {
            self.loop_invariants.push(LoopInvariantOutcome {
                node_path: invariant.relation.node_path.clone(),
                loop_id,
                source_ordinal: u32::try_from(index).expect("loop invariant ordinal exceeds u32"),
                name: invariant.name.clone(),
                base_target: self.render_checked_invariant_relation(&invariant.relation, None),
                backedge_target: self
                    .render_checked_invariant_relation(&invariant.relation, counted_binder),
                proof: LoopInvariantProof {
                    base: base[index],
                    step: step[index],
                },
            });
        }
    }

    /// Renders one INV-1 incoming-edge target using only source spellings.
    ///
    /// The checked relation contains immutable binding identities, which are
    /// appropriate for proof but useless in a source diagnostic. For a
    /// counted backedge the only compiler-written value transition is the
    /// hidden unit update, so occurrences of that binder are rendered as the
    /// exact source expression the writer must preserve. Ordinary loops pass
    /// no binder and therefore render the header relation unchanged.
    fn render_checked_invariant_relation(
        &self,
        relation: &CheckedAffineRelation,
        counted_next_binder: Option<BindingId>,
    ) -> String {
        let left = self.render_checked_affine_expression(&relation.left, counted_next_binder);
        let right = self.render_checked_affine_expression(&relation.right, counted_next_binder);
        match relation.bound {
            0 => format!("{left} <= {right}"),
            -1 => format!("{left} < {right}"),
            // INV-1 formation currently admits only strict and non-strict
            // ordered roots. Keep a source-level fallback so an internal
            // inconsistency never leaks an affine term identity.
            bound => format!("({left} - {right}) <= {bound}_i128"),
        }
    }

    fn render_checked_affine_expression(
        &self,
        expression: &CheckedAffineExpression,
        counted_next_binder: Option<BindingId>,
    ) -> String {
        match &expression.kind {
            CheckedAffineExpressionKind::Constant { value, ty } => {
                format!("{value}_{}", integer_type_name(*ty))
            }
            CheckedAffineExpressionKind::Local { binding, .. } => {
                let name = self.binding_name(*binding);
                if counted_next_binder == Some(*binding) {
                    format!("({name} + 1_u64)")
                } else {
                    name
                }
            }
            // [INV-1] a measure factor renders as the writer wrote it: the
            // former over the place, never an internal term identity.
            CheckedAffineExpressionKind::Measure(measure) => self
                .render_affine_measure(measure)
                .unwrap_or_else(|| "?".to_owned()),
            CheckedAffineExpressionKind::ConstGeneric { name, .. } => name.clone(),
            CheckedAffineExpressionKind::Add(left, right) => format!(
                "({} + {})",
                self.render_checked_affine_expression(left, counted_next_binder),
                self.render_checked_affine_expression(right, counted_next_binder)
            ),
            CheckedAffineExpressionKind::Subtract(left, right) => format!(
                "({} - {})",
                self.render_checked_affine_expression(left, counted_next_binder),
                self.render_checked_affine_expression(right, counted_next_binder)
            ),
            CheckedAffineExpressionKind::MultiplyByConstant {
                constant,
                constant_ty,
                value,
            } => format!(
                "({constant}_{} * {})",
                integer_type_name(*constant_ty),
                self.render_checked_affine_expression(value, counted_next_binder)
            ),
        }
    }

    /// The writer's own spelling of one [INV-1] affine measure factor.
    fn render_affine_measure(&self, expression: &CheckedExpression) -> Option<String> {
        let (measure, binding, fields) = match expression {
            CheckedExpression::ArrayMeasure {
                measure,
                root: CheckedArrayRoot::Binding { binding, fields },
                ..
            } => (*measure, *binding, fields.clone()),
            CheckedExpression::BufferMeasure { measure, root } => {
                (*measure, root.binding, root.fields.clone())
            }
            CheckedExpression::SliceMeasure { measure, root } => {
                (*measure, root.binding, Vec::new())
            }
            // [MSR-1] a measured place may carry a subscript, so this one is
            // rendered from the same source-order path every other consumer
            // reads rather than from a field list.
            CheckedExpression::ContainerMeasure { measure, root } => {
                let mut path = self.container_root_path(root);
                path.projections
                    .retain(|projection| !matches!(projection, PlaceProjection::Deref));
                let place = self.render_projected_place(&ProjectedPlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    projections: path.projections,
                });
                return Some(format!("{}({place})", measure.spelling()));
            }
            _ => return None,
        };
        let place = self.render_place(&PlaceTerm {
            root: PlaceRoot::Binding(binding),
            deref: false,
            fields,
        });
        Some(format!("{}({place})", measure.spelling()))
    }

    fn checked_affine_relation_inequality(
        &mut self,
        relation: &CheckedAffineRelation,
        state: &mut AffineFlowState,
        check: &mut AffineCheckState,
    ) -> Result<AffineInequality, AffineCheckError> {
        let left = self.checked_affine_form(&relation.left, state, check)?;
        let right = self.checked_affine_form(&relation.right, state, check)?;
        AffineInequality::from_bounded_forms(&left, &right, relation.bound, check)
    }

    /// Projects the exact source relation into L0 when its normalized binding
    /// coefficients have one of the fixed difference-bound shapes. This does
    /// no discovery: it only recognizes `x - y <= c`, `x <= c`, `c <= x`, or
    /// a constant proposition after the source-written affine arithmetic has
    /// been normalized.
    fn checked_affine_relation_l0(&mut self, relation: &CheckedAffineRelation) -> Option<Relation> {
        /// One leaf of the written relation, in the order the walk reaches it.
        enum SourceLeaf {
            Local(BindingId),
            /// [INV-1] one measure factor, already interned as its [ENT-2]
            /// term by the pre-pass below.
            Measure(TermId),
        }

        fn source_form(
            expression: &CheckedAffineExpression,
            leaves: &mut Vec<SourceLeaf>,
            measures: &[TermId],
            visited: &mut usize,
            check: &mut AffineCheckState,
        ) -> Option<AffineForm> {
            match &expression.kind {
                CheckedAffineExpressionKind::Constant { value, .. } => {
                    Some(AffineForm::constant(*value))
                }
                CheckedAffineExpressionKind::Local { binding, .. } => {
                    let index = leaves
                        .iter()
                        .position(|candidate| matches!(candidate, SourceLeaf::Local(other) if other == binding))
                        .unwrap_or_else(|| {
                            leaves.push(SourceLeaf::Local(*binding));
                            leaves.len() - 1
                        });
                    let index = u32::try_from(index).ok()?;
                    Some(AffineForm::term(AffineTermId::from_index(index)))
                }
                CheckedAffineExpressionKind::Measure(_)
                | CheckedAffineExpressionKind::ConstGeneric { .. } => {
                    let term = *measures.get(*visited)?;
                    *visited = visited.checked_add(1)?;
                    let index = leaves
                        .iter()
                        .position(|candidate| matches!(candidate, SourceLeaf::Measure(other) if *other == term))
                        .unwrap_or_else(|| {
                            leaves.push(SourceLeaf::Measure(term));
                            leaves.len() - 1
                        });
                    let index = u32::try_from(index).ok()?;
                    Some(AffineForm::term(AffineTermId::from_index(index)))
                }
                CheckedAffineExpressionKind::Add(left, right) => {
                    source_form(left, leaves, measures, visited, check)?
                        .add(
                            &source_form(right, leaves, measures, visited, check)?,
                            check,
                        )
                        .ok()
                }
                CheckedAffineExpressionKind::Subtract(left, right) => {
                    source_form(left, leaves, measures, visited, check)?
                        .subtract(
                            &source_form(right, leaves, measures, visited, check)?,
                            check,
                        )
                        .ok()
                }
                CheckedAffineExpressionKind::MultiplyByConstant {
                    constant, value, ..
                } => source_form(value, leaves, measures, visited, check)?
                    .scale(*constant, check)
                    .ok(),
            }
        }

        // Interning needs `&mut self`, and the walk above does not have it, so
        // the measure terms are resolved first in exactly the order that walk
        // reaches them.
        let mut measures = Vec::new();
        self.collect_affine_measure_terms(&relation.left, &mut measures)?;
        self.collect_affine_measure_terms(&relation.right, &mut measures)?;
        let mut leaves = Vec::new();
        let mut visited = 0;
        let mut check = AffineCheckState::new();
        let left = source_form(
            &relation.left,
            &mut leaves,
            &measures,
            &mut visited,
            &mut check,
        )?;
        let right = source_form(
            &relation.right,
            &mut leaves,
            &measures,
            &mut visited,
            &mut check,
        )?;
        let inequality =
            AffineInequality::from_bounded_forms(&left, &right, relation.bound, &mut check).ok()?;
        let mut term = |coefficient: super::affine::AffineCoefficient| match leaves
            .get(coefficient.term().index() as usize)?
        {
            SourceLeaf::Measure(term) => Some(*term),
            SourceLeaf::Local(binding) => {
                let binding = *binding;
                let fragment =
                    fragment_type(CheckedType::Integer(self.affine_binding_type(binding)?))?;
                Some(self.terms.intern(TermKind::Place(
                    PlaceTerm {
                        root: PlaceRoot::Binding(binding),
                        deref: false,
                        fields: Vec::new(),
                    },
                    fragment,
                )))
            }
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

    fn source_proof_formation_failure(error: AffineCheckError) -> SourceProofCertificateFailure {
        match error {
            AffineCheckError::ArithmeticOverflow => {
                SourceProofCertificateFailure::ArithmeticOverflow
            }
            AffineCheckError::LimitExceeded(_) => SourceProofCertificateFailure::FormationCapacity,
            AffineCheckError::CoefficientMismatch | AffineCheckError::InvalidCertificateFactor => {
                unreachable!("a checked affine source has inconsistent internal structure")
            }
        }
    }

    fn source_proof_premise_results(
        &mut self,
        premises: &[Option<AffineInequality>],
        l0_premises: &[Option<Relation>],
        named_premises: &[bool],
        published_premises: &[bool],
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Vec<bool> {
        premises
            .iter()
            .zip(l0_premises)
            .zip(named_premises)
            .zip(published_premises)
            .map(|(((premise, relation), named), published)| {
                // A bare invariant name means that exact declaration's
                // published theorem, not merely any proposition with the same
                // normalized inequality. Only a relation-form use asks AUTO
                // to prove its written source from the entering context.
                if *named {
                    return *published;
                }
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
            .collect()
    }

    /// Forms the one weighted premise sum the source writer selected.
    ///
    /// The written premises are multiplied and summed exactly in source order.
    /// This phase depends only on the formed source propositions and written
    /// factors. It deliberately runs before premise availability is judged.
    fn source_proof_sum(
        &self,
        premises: &[(AffineInequality, CertificateMultiplicity)],
    ) -> Result<CertificateSum, (SourceProofCertificateFailure, u32)> {
        let actual = u32::try_from(premises.len()).unwrap_or(u32::MAX);
        if premises.len() > MAX_CERTIFICATE_PREMISES {
            let maximum =
                u32::try_from(MAX_CERTIFICATE_PREMISES).expect("certificate capacity fits u32");
            return Err((
                SourceProofCertificateFailure::UseCapacity { maximum, actual },
                maximum,
            ));
        }

        let mut first_by_premise = HashMap::new();
        for (index, (premise, multiplicity)) in premises.iter().enumerate() {
            let index = u32::try_from(index).expect("certificate capacity fits u32");
            // A term multiplicity is unsigned by [PRF-1], so only the written
            // decimal can be degenerate. A runtime zero drops its premise and
            // the sum stays sound, which is why nothing rejects it here.
            if matches!(multiplicity, CertificateMultiplicity::Literal(factor) if *factor <= 0) {
                return Err((
                    SourceProofCertificateFailure::InvalidFactor { use_index: index },
                    index,
                ));
            }
            if let Some(first) = first_by_premise.insert(premise.clone(), index) {
                return Err((
                    SourceProofCertificateFailure::RepeatedUse {
                        first,
                        repeated: index,
                    },
                    index,
                ));
            }
        }

        // Build the written sum one source entry at a time. Besides preserving
        // source order, this records the exact entry whose scale or addition
        // first exceeds the proof arithmetic or affine formation domain.
        //
        // The accumulator starts affine and becomes a degree-two polynomial at
        // the first term multiplicity, if there is one; a certificate written
        // entirely with bare decimals therefore never leaves the affine arm
        // and forms exactly the inequality it always did.
        let mut sum = CertificateSum::Empty;
        for (index, (inequality, multiplicity)) in premises.iter().enumerate() {
            let index = u32::try_from(index).expect("certificate capacity fits u32");
            sum =
                Self::extend_certificate_sum(sum, inequality, multiplicity).map_err(|failure| {
                    (
                        Self::certificate_step_failure(failure, index, actual),
                        index,
                    )
                })?;
        }
        match sum {
            CertificateSum::Empty => Err((SourceProofCertificateFailure::FormationCapacity, 0)),
            formed => Ok(formed),
        }
    }

    /// Adds one written entry to the accumulated certificate sum.
    fn extend_certificate_sum(
        sum: CertificateSum,
        inequality: &AffineInequality,
        multiplicity: &CertificateMultiplicity,
    ) -> Result<CertificateSum, CertificateStepFailure> {
        if let CertificateMultiplicity::Literal(factor) = *multiplicity {
            match sum {
                CertificateSum::Empty => {
                    let mut check = AffineCheckState::new();
                    return Ok(CertificateSum::Affine(sum_explicit_scaled_inequalities(
                        &[ScaledAffinePremise { inequality, factor }],
                        &mut check,
                    )?));
                }
                CertificateSum::Affine(previous) => {
                    let mut check = AffineCheckState::new();
                    return Ok(CertificateSum::Affine(sum_explicit_scaled_inequalities(
                        &[
                            ScaledAffinePremise {
                                inequality: &previous,
                                factor: 1,
                            },
                            ScaledAffinePremise { inequality, factor },
                        ],
                        &mut check,
                    )?));
                }
                CertificateSum::Nonlinear(previous) => {
                    let scaled =
                        CertificatePolynomial::from_inequality(inequality)?.scale(factor)?;
                    return Ok(CertificateSum::Nonlinear(previous.add(&scaled)?));
                }
            }
        }
        let CertificateMultiplicity::Value(value) = multiplicity else {
            unreachable!("the literal arm returned above");
        };
        let scaled = CertificatePolynomial::from_inequality(inequality)?
            .multiply(&CertificatePolynomial::from_form(value)?)?;
        let previous = match sum {
            CertificateSum::Empty => CertificatePolynomial::zero(),
            CertificateSum::Affine(previous) => CertificatePolynomial::from_inequality(&previous)?,
            CertificateSum::Nonlinear(previous) => previous,
        };
        Ok(CertificateSum::Nonlinear(previous.add(&scaled)?))
    }

    fn certificate_step_failure(
        failure: CertificateStepFailure,
        index: u32,
        actual: u32,
    ) -> SourceProofCertificateFailure {
        match failure {
            CertificateStepFailure::Overflow => SourceProofCertificateFailure::ArithmeticOverflow,
            CertificateStepFailure::UseCapacity => SourceProofCertificateFailure::UseCapacity {
                maximum: u32::try_from(MAX_CERTIFICATE_PREMISES)
                    .expect("certificate capacity fits u32"),
                actual,
            },
            CertificateStepFailure::Formation => SourceProofCertificateFailure::FormationCapacity,
            CertificateStepFailure::InvalidFactor => {
                SourceProofCertificateFailure::InvalidFactor { use_index: index }
            }
        }
    }

    /// Resolves one written multiplicity where the certificate is checked.
    ///
    /// A named multiplicity reads the value image its binding holds in the
    /// entering context, minting the atom if this is the first read of it, so
    /// the scaling step is over the same immutable value identity every other
    /// affine premise names.
    fn certificate_multiplicity(
        &mut self,
        multiplicity: CheckedProofMultiplicity,
        state: &mut AffineFlowState,
    ) -> Option<CertificateMultiplicity> {
        match multiplicity {
            CheckedProofMultiplicity::Literal(factor) => {
                Some(CertificateMultiplicity::Literal(factor))
            }
            CheckedProofMultiplicity::Value { binding, .. } => Some(
                CertificateMultiplicity::Value(self.affine_opaque_handle(binding, state)?),
            ),
        }
    }

    /// Brings the accumulated certificate sum back to one affine inequality
    /// and checks the writer-selected residual against it.
    ///
    /// A nonlinear accumulation folds first: each degree-two monomial must be
    /// the value image of an admitted exact product, which is the only way a
    /// term-scaled premise can meet an affine target. Once folded, the residual
    /// is the same one a bare-decimal certificate reaches, proved by the same
    /// route; a monomial with no such product is a refusal, not a weaker check.
    fn source_proof_certificate_residual(
        &mut self,
        target: &AffineInequality,
        sum: &CertificateSum,
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Result<bool, SourceProofCertificateFailure> {
        let folded;
        let sum = match sum {
            CertificateSum::Empty => {
                return Err(SourceProofCertificateFailure::FormationCapacity);
            }
            CertificateSum::Affine(sum) => sum,
            CertificateSum::Nonlinear(polynomial) => {
                folded = self.folded_certificate_sum(polynomial, target)?;
                &folded
            }
        };
        self.source_proof_residual(target, sum, values, facts)
    }

    /// Folds a nonlinear certificate sum to the affine inequality it equals.
    fn folded_certificate_sum(
        &self,
        polynomial: &CertificatePolynomial,
        target: &AffineInequality,
    ) -> Result<AffineInequality, SourceProofCertificateFailure> {
        // Several bindings can hold the same product, and they are equal
        // values, so any of them folds soundly. The target's own text picks
        // among them: a monomial folded to the value the target already names
        // cancels against it, while one folded to an equal value under
        // another name does not. Failing that, the least atom is a canonical
        // choice. Neither is a search — one pass, one winner per operand pair.
        let named_by_target = target
            .terms()
            .iter()
            .map(|coefficient| coefficient.term())
            .collect::<HashSet<_>>();
        let mut products = std::collections::BTreeMap::new();
        for (product, operands) in &self.product_atoms {
            products
                .entry(*operands)
                .and_modify(|chosen: &mut AffineTermId| {
                    let better = match (
                        named_by_target.contains(chosen),
                        named_by_target.contains(product),
                    ) {
                        (false, true) => true,
                        (true, false) => false,
                        _ => *product < *chosen,
                    };
                    if better {
                        *chosen = *product;
                    }
                })
                .or_insert(*product);
        }
        let folded = polynomial
            .fold_products(&products)
            .map_err(Self::certificate_fold_failure)?;
        let mut images = std::collections::BTreeMap::new();
        for (handle, image) in &self.handle_images {
            let mut weights = image
                .terms()
                .iter()
                .map(|coefficient| (Some(coefficient.term()), coefficient.coefficient()))
                .collect::<Vec<_>>();
            weights.push((None, image.constant_value()));
            images.insert(*handle, weights);
        }
        let folded = folded
            .unfold_handles(&images)
            .map_err(Self::certificate_fold_failure)?;
        let mut check = AffineCheckState::new();
        match folded.into_inequality(&mut check) {
            Some(formed) => formed.map_err(Self::certificate_fold_failure),
            None => Err(SourceProofCertificateFailure::NonlinearResidual),
        }
    }

    fn certificate_fold_failure(error: PolynomialError) -> SourceProofCertificateFailure {
        match error {
            PolynomialError::ArithmeticOverflow => {
                SourceProofCertificateFailure::ArithmeticOverflow
            }
            PolynomialError::DegreeExceeded | PolynomialError::LimitExceeded => {
                SourceProofCertificateFailure::FormationCapacity
            }
        }
    }

    /// Checks the final writer-selected residual after every source proposition
    /// and its scaled sum have formed.
    ///
    /// `target - sum` may be discharged only by the existing direct L0 closure
    /// or fixed interval rule at the entering program point, applied to the
    /// written sum and then to its integer tightenings. This route never
    /// selects another affine premise, derives a multiplier, or retries a
    /// subset.
    fn source_proof_residual(
        &mut self,
        target: &AffineInequality,
        sum: &AffineInequality,
        values: &AffineFlowState,
        facts: &FactState,
    ) -> Result<bool, SourceProofCertificateFailure> {
        let mut check = AffineCheckState::new();
        // The untightened residual forms first so an arithmetic or capacity
        // failure of the written sum keeps its exact PRF-1 diagnostic.
        match AffineInequality::residual_after(target, sum, &mut check) {
            Ok(_) => {}
            Err(AffineCheckError::ArithmeticOverflow) => {
                return Err(SourceProofCertificateFailure::ArithmeticOverflow);
            }
            Err(AffineCheckError::LimitExceeded(_)) => {
                return Err(SourceProofCertificateFailure::FormationCapacity);
            }
            Err(
                AffineCheckError::CoefficientMismatch | AffineCheckError::InvalidCertificateFactor,
            ) => return Ok(false),
        }
        let candidates = self.affine_l0_candidates(values);
        let closed = close(facts, &self.terms, &self.goals, &mut self.derivations);
        let l0 = self.affine_l0_index(&candidates, &closed, &mut check);
        Ok(self
            .affine_candidate_residual_proof(target, sum, &l0, values, &closed, &mut check)
            .is_some())
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

        let measure_terms_by_atom = self.measure_terms_by_atom();
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
            let mut terms = bindings
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
            if let Some(measures) = measure_terms_by_atom.get(&atom_id) {
                terms.extend(measures.iter().copied());
            }
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
    /// The image of one measure term [MSR-4].
    ///
    /// A measure whose table cell [MSR-1] fixes its value is a standing fact
    /// [MSR-2], and its image is that fact rather than a free atom: a cell
    /// with a constant value has that constant, and a cell the table equates
    /// to another term shares that term's image. Every other measure gets one
    /// compiler-owned immutable atom, minted on first use and stable for the
    /// rest of the function walk.
    /// The binding one measure term's place is rooted in, where it has one.
    fn measure_term_root(&self, term: TermId) -> Option<BindingId> {
        let root = match self.terms.kind(term) {
            TermKind::Measure(_, place) => place.root,
            TermKind::ProjectedMeasure(_, place) => place.root,
            _ => return None,
        };
        match root {
            PlaceRoot::Binding(binding) => Some(binding),
            PlaceRoot::Constant(_) => None,
        }
    }

    /// Every [INV-1] measure factor of one written affine expression, in the
    /// order a left-to-right walk reaches it, interned as its [ENT-2] term.
    fn collect_affine_measure_terms(
        &mut self,
        expression: &CheckedAffineExpression,
        out: &mut Vec<TermId>,
    ) -> Option<()> {
        match &expression.kind {
            CheckedAffineExpressionKind::Constant { .. }
            | CheckedAffineExpressionKind::Local { .. } => {}
            CheckedAffineExpressionKind::Measure(measure) => {
                out.push(self.checked_measure_term(measure)?);
            }
            CheckedAffineExpressionKind::ConstGeneric { declaration, .. } => {
                out.push(self.terms.intern(TermKind::ConstParameter(*declaration)));
            }
            CheckedAffineExpressionKind::Add(left, right)
            | CheckedAffineExpressionKind::Subtract(left, right) => {
                self.collect_affine_measure_terms(left, out)?;
                self.collect_affine_measure_terms(right, out)?;
            }
            CheckedAffineExpressionKind::MultiplyByConstant { value, .. } => {
                self.collect_affine_measure_terms(value, out)?;
            }
        }
        Some(())
    }

    /// The [ENT-2] measure term one [INV-1] affine measure factor names.
    fn checked_measure_term(&mut self, expression: &CheckedExpression) -> Option<TermId> {
        let goal = self.goal_expression(expression, false)?;
        self.goal_operand(&goal)
    }

    /// The image this program point holds for one measure term.
    ///
    /// [MSR-4]'s automatic derivation reads a measure through
    /// [`Self::measure_atom`], which is one immutable atom for the whole walk
    /// because a goal is discharged from the fact state at its own point. A
    /// written invariant is different: its conclusion is carried forward as a
    /// fact over value images, so the image has to be retargeted by the
    /// events that kill the term, exactly as a local's image is retargeted by
    /// a write to that local. A measure the table fixes has no mutable image
    /// at all, and reads as the standing fact [MSR-2] gives it.
    fn measure_atom(&mut self, term: TermId) -> AffineForm {
        let mut anchor = term;
        // The table relates a cell to a constant or to one other term, and
        // this version's rows chain at most once (`cap` to the run's own
        // extent). The bound keeps a future row from looping.
        for _ in 0..4 {
            match self.terms.measure_bound(anchor) {
                Some(MeasureBound::Constant(value)) => return AffineForm::constant(value),
                Some(MeasureBound::Equal(other)) => anchor = other,
                None => break,
            }
        }
        if let Some(atom) = self.measure_atoms.get(&anchor) {
            return atom.clone();
        }
        let atom = self.new_affine_atom(IntegerType::U64);
        self.measure_atoms.insert(anchor, atom.clone());
        atom
    }

    /// [MSR-3] one measure datum inherits the atom the term it is established
    /// equal to holds at that point.
    ///
    /// The datum denotes that value, so it is that value in the affine domain
    /// too. Because nothing kills a datum, its atom outlives the write that
    /// retargets the term's: a header conclusion published over the old atom
    /// stays anchored to a live term, which is what lets one published
    /// relation preserve an invariant across a [LIV-2] commit.
    fn adopt_measure_atom(&mut self, datum: TermId, live: TermId) {
        if self.measure_atoms.contains_key(&datum) {
            return;
        }
        let atom = self.measure_atom(live);
        if atom.terms().is_empty() {
            return;
        }
        self.measure_atoms.insert(datum, atom);
    }

    /// Every registered measure term, in term order.
    ///
    /// The registry only grows during the forward walk, so this scans just
    /// the terms interned since the last call and keeps the answer. Every
    /// numeric goal queries it, and rescanning the whole registry per query
    /// made that quadratic in the size of the function.
    fn measure_terms(&mut self) -> Vec<TermId> {
        let registered = self.terms.ids().count();
        for index in self.measure_terms_scanned..registered {
            let id = TermId(
                u32::try_from(index).expect("ENT term inventory exceeds the u32 identity space"),
            );
            // [MSR-3] a measure datum is a measure of the affine domain's
            // kind: it denotes one measure's value at a point, it is of
            // fragment type u64, and nothing kills it. It participates in
            // step 6's bridge exactly as a live measure term does, which is
            // what carries a header conclusion across the write that kills
            // the term the conclusion was published over.
            if matches!(
                self.terms.kind(id),
                TermKind::Measure(..)
                    | TermKind::ProjectedMeasure(..)
                    | TermKind::CallDatum {
                        measure: Some(_),
                        ..
                    }
                    | TermKind::EntryDatum { .. }
                    | TermKind::MeasureDatum { .. }
            ) {
                self.measure_terms_seen.push(id);
            }
        }
        self.measure_terms_scanned = registered;
        self.measure_terms_seen.clone()
    }

    /// Every live measure term, grouped by the affine atom it images.
    ///
    /// [MSR-4]'s interval step starts from each atom's *direct closed L0*
    /// interval, so it has to be able to name the term whose value the atom
    /// stands for. A local's atom is named through the binding that denotes
    /// it; a measure atom's own name is its measure term [MSR-2], and without
    /// this map a measure entered every interval substitution at its complete
    /// `u64` range however tightly the closed state had already bounded it.
    fn measure_terms_by_atom(&mut self) -> HashMap<AffineTermId, Vec<TermId>> {
        let mut grouped: HashMap<AffineTermId, Vec<TermId>> = HashMap::new();
        for term in self.measure_terms() {
            if let Some(atom) = self.measure_atom(term).unit_term() {
                grouped.entry(atom).or_default().push(term);
            }
        }
        for terms in grouped.values_mut() {
            terms.sort_unstable_by_key(|term| term.0);
            terms.dedup();
        }
        grouped
    }

    fn affine_l0_candidates(&mut self, values: &AffineFlowState) -> Vec<AffineL0Candidate> {
        let mut candidates = vec![AffineL0Candidate {
            term: ZERO,
            value: AffineForm::constant(0),
        }];
        // [MSR-4] step 6 ranges over every live measure term as well as every
        // own integer binding with an image, so a measure participates in the
        // affine domain through its own atom.
        for term in self.measure_terms() {
            let value = self.measure_atom(term);
            // A measure whose image is a constant is Z displaced by that
            // constant, and Z is already the fixed zero candidate, so its
            // index entries would duplicate Z's under one coefficient vector.
            if value.terms().is_empty() {
                continue;
            }
            candidates.push(AffineL0Candidate { term, value });
        }
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
    /// [MSR-2]'s capacity identity, appended to [ENT-6]'s automatic
    /// affine-premise sequence as two inequalities with the empty support
    /// every standing fact has.
    ///
    /// It is appended when a place's measure terms become live, never by an
    /// operation's post-state, and it is a convenience for the writer rather
    /// than a route by which an operation's own post-state is derived.
    fn capacity_identity_premises(
        &mut self,
        check: &mut AffineCheckState,
    ) -> Result<Vec<AutomaticAffinePremise>, AffineCheckError> {
        let mut premises = Vec::new();
        for capacity in self.measure_terms() {
            if !matches!(
                self.terms.kind(capacity),
                TermKind::Measure(CheckedMeasure::Capacity, _)
                    | TermKind::ProjectedMeasure(CheckedMeasure::Capacity, _)
            ) {
                continue;
            }
            let (Some(length), Some(room)) = (
                self.terms.sibling_measure(capacity, CheckedMeasure::Length),
                self.terms.sibling_measure(capacity, CheckedMeasure::Room),
            ) else {
                continue;
            };
            let capacity_atom = self.measure_atom(capacity);
            let length_atom = self.measure_atom(length);
            let room_atom = self.measure_atom(room);
            let Ok(filled) = length_atom.add(&room_atom, check) else {
                continue;
            };
            for (left, right) in [(&filled, &capacity_atom), (&capacity_atom, &filled)] {
                let Ok(inequality) = AffineInequality::from_bounded_forms(left, right, 0, check)
                else {
                    continue;
                };
                // Where the table's own cells already make the identity
                // trivial — this version's `room` is the constant zero and
                // its `cap` shares the extent's image — the two inequalities
                // carry no term and grant nothing; publishing them would only
                // make every AUTO traversal visit two empty candidates.
                if inequality.terms().is_empty() {
                    continue;
                }
                premises.push(AutomaticAffinePremise {
                    inequality,
                    source: None,
                    parent: None,
                });
            }
        }
        Ok(premises)
    }

    fn automatic_affine_premises(
        &mut self,
        facts: &[ActiveAffineFact],
        check: &mut AffineCheckState,
    ) -> Result<Vec<AutomaticAffinePremise>, AffineCheckError> {
        let mut premises = self.capacity_identity_premises(check)?;
        for fact in Self::canonical_affine_facts(facts) {
            check.charge(1)?;
            let (source, parent) = match fact.evidence {
                AffineFactEvidence::Source(source) => (Some(source), None),
                AffineFactEvidence::Derivation(parent) => (None, Some(parent)),
            };
            premises.push(AutomaticAffinePremise {
                inequality: fact.inequality.clone(),
                source,
                parent,
            });
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

        let measure_terms_by_atom = self.measure_terms_by_atom();
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
            let mut terms = bindings
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
            if let Some(measures) = measure_terms_by_atom.get(&atom_id) {
                terms.extend(measures.iter().copied());
            }
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

    /// Checks the fixed `DIRECT(T - S)` residual of one accumulated candidate
    /// `S`, and then the same residual against each integer tightening of `S`.
    ///
    /// Every affine atom denotes a mathematical integer, so an accumulated
    /// `k * v <= u` with a positive integer `k` dividing every coefficient
    /// also proves `v <= floor(u / k)`. The tightening factors are functions
    /// of the candidate and the target alone: this step selects no additional
    /// premise, guesses no multiplier, and leaves the candidate families
    /// exactly as fixed by the specification. Each tightening is formed on its
    /// own: an unrepresentable one is skipped and removes neither the other
    /// tightening nor the untightened candidate.
    fn affine_candidate_residual_proof(
        &mut self,
        target: &AffineInequality,
        candidate: &AffineInequality,
        l0: &AffineL0Index,
        values: &AffineFlowState,
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> Option<Vec<DerivationId>> {
        let tightenings = integer_tightenings(candidate, target, check);
        for accumulated in std::iter::once(candidate).chain(tightenings.iter()) {
            let Ok(residual) = AffineInequality::residual_after(target, accumulated, check) else {
                continue;
            };
            if let Ok(Some(parents)) =
                self.affine_residual_proof(&residual, l0, values, closed, check)
            {
                return Some(parents);
            }
        }
        None
    }

    /// Exhausts one coefficient-one L0 premise followed by the direct
    /// L0/interval residual rule. The L0 index contains one strongest entry
    /// per coefficient vector, so strengthening ordinary facts can only make
    /// a residual easier and never removes an earlier witness.
    fn affine_l0_then_direct_proof(
        &mut self,
        target: &AffineInequality,
        l0: &AffineL0Index,
        values: &AffineFlowState,
        closed: &ClosedState,
        check: &mut AffineCheckState,
    ) -> Option<Vec<DerivationId>> {
        for entry in &l0.entries {
            let Some(mut parents) = self.affine_candidate_residual_proof(
                target,
                &entry.inequality,
                l0,
                values,
                closed,
                check,
            ) else {
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
            .automatic_affine_premises(assumptions, &mut check)
            .ok()?;

        // Preserve the complete coefficient-one single-premise route. Every
        // premise is tried independently; an arithmetic error in one candidate
        // cannot suppress a later source or value-image fact.
        for (index, assumption) in automatic.iter().enumerate() {
            // A candidate that cannot participate in an i128 residual grants
            // no authority, but it must not hide a later independently
            // representable source fact in the same deterministic order.
            if let Some(parents) = self.affine_candidate_residual_proof(
                target,
                &assumption.inequality,
                &l0,
                values,
                &closed,
                &mut check,
            ) {
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
            first_two_premise_candidate(&automatic, &mut check, |sum, check| {
                self.affine_candidate_residual_proof(target, sum, &l0, values, &closed, check)
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
        // is the specification's final `DIRECT(T - R)` family: subtract each
        // strongest indexed L0 image once, then run the ordinary DIRECT check
        // on the residual. DIRECT may itself close an exact L0 image, but the
        // route never publishes or recursively saturates either relation.
        self.affine_l0_then_direct_proof(target, &l0, values, &closed, &mut check)
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

    fn apply_affine_kills(&mut self, state: &mut AffineFlowState, events: &[KillEvent]) {
        state.values.retain(|binding, _| {
            !events
                .iter()
                .any(|event| Self::affine_event_kills_binding(*binding, event))
        });
        // [MSR-2] a measure atom is retargeted on exactly the events that
        // kill its term, exactly as a local's image is retargeted by a write
        // to that local: the next occurrence of the term mints a fresh atom,
        // so no conclusion published over the old one reaches past the write.
        let stale: Vec<TermId> = self
            .measure_atoms
            .keys()
            .copied()
            .filter(|term| {
                events
                    .iter()
                    .any(|event| self.event_kills_term(*term, event))
            })
            .collect();
        for term in stale {
            self.measure_atoms.remove(&term);
        }
        // An opaque handle names one binding's value at the point the fold
        // took it, so it dies with a write to that binding exactly as the
        // binding's own image does [ENT-5].
        state.opaque_values.retain(|binding, _| {
            !events
                .iter()
                .any(|event| Self::affine_event_kills_binding(*binding, event))
        });
        // Published facts name immutable AffineTermId value identities, not
        // mutable bindings. Removing the map above prevents a replacement
        // value from matching an old image; retaining each theorem preserves
        // valid aliases to the old value.
    }

    fn expression_effects(
        &mut self,
        expression: &CheckedExpression,
        state: &mut ProofFlowState,
    ) -> ExpressionJudgment {
        let mut judgment = self.judge_expression(expression, state);
        let mut events = Vec::new();
        self.collect_expression_kills(expression, &mut events);
        if let Some(prepared) = &mut judgment.prepared_call {
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
                self.apply_affine_kills(&mut state.affine, std::slice::from_ref(event));
                self.invalidate_entry_images(state, std::slice::from_ref(event), Some(proof_event));
                prepared.transfer_events.push(proof_event);
            }
            prepared.kills = events;
        } else {
            self.apply_kills(state, &events);
        }
        judgment
    }

    /// The [ENT-5] commit kill of one `set` target, and the goal-origin and
    /// outcome state a whole-place commit invalidates. One target list's
    /// commits are exactly this event per target, on the same edge.
    fn collect_target_kill(
        &mut self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        state: &mut ProofFlowState,
        target_kills: &mut Vec<KillEvent>,
    ) {
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
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
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
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
                    element: true,
                    source: node_path.clone(),
                });
            }
            // [MSR-2] a view element store writes one element of the view's
            // own range, and a view's element is flat [TYPE-2], so the kill
            // is the same element write a buffer's is.
            CheckedSetTarget::SliceIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: Vec::new(),
                };
                target_kills.push(KillEvent::Write {
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
                    element: true,
                    source: node_path.clone(),
                });
            }
            // [MSR-2] an element store into a run overlaps the descriptor
            // storage of `v[i]` and none of `v`'s own, so it kills the
            // measures of the element and none of the run's.
            CheckedSetTarget::RunIndex(target) => {
                target_kills.push(KillEvent::Write {
                    place: element_write_place(
                        self.container_root_place(&target.root),
                        target.place_offset,
                    ),
                    element: true,
                    source: node_path.clone(),
                });
            }
        }
    }

    /// [GRAM-4, SET-1, CALL-4, LIV-2] one `set` target list.
    ///
    /// Every target is judged, then the whole right-hand side is judged — the
    /// one call, or every written ordinal in order — then every commit kill
    /// applies on the same edge, and only then does each target receive the
    /// published relations naming its own ordinal [ENT-3.S12]. There is no
    /// commit value: [ENT-3.S5] gives a call right-hand side no image, so no
    /// ordinal's commit establishes an equality of its own.
    fn walk_set_list(
        &mut self,
        node_path: &crate::NodePath,
        targets: &[CheckedSetTarget],
        values: &CheckedCommitValues,
        state: &mut ProofFlowState,
    ) {
        // [MSR-3] the [LIV-2] `set`-target placement, per ordinal: a written
        // value list commits value i into target i, so ordinal i carries
        // exactly what a single-target `set` carries.
        let placements = match values {
            CheckedCommitValues::Written(values) => targets
                .iter()
                .zip(values)
                .enumerate()
                .map(|(ordinal, (target, value))| {
                    let ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
                    self.mint_commit_placement(node_path, ordinal, target, value, &mut state.facts)
                })
                .collect::<Vec<_>>(),
            CheckedCommitValues::ResultList { .. } => Vec::new(),
        };
        let mut target_reached = true;
        for target in targets {
            target_reached &= self.judge_set_target(target, state);
        }
        let mut ordinal_calls = Vec::with_capacity(targets.len());
        let mut value_reached = true;
        for value in values.expressions() {
            let judgment = self.expression_effects(value, state);
            value_reached &= judgment.reached;
            ordinal_calls.push(judgment.prepared_call);
        }
        let prepared = match values {
            CheckedCommitValues::ResultList { .. } => ordinal_calls.first().cloned().flatten(),
            CheckedCommitValues::Written(_) => None,
        };
        let commit_reached = target_reached && value_reached;
        for target in targets {
            invalidate_goal_origin_for_set(&mut state.facts, target);
        }
        let mut target_kills = Vec::new();
        for target in targets {
            self.collect_target_kill(node_path, target, state, &mut target_kills);
        }
        let establishes = commit_reached && prepared.is_some();
        let target_event = establishes
            .then(|| self.proof_event(FlowEventKind::PostconditionReceiverWrite, Some(node_path)));
        if let Some(target_event) = target_event {
            if !target_kills.is_empty() {
                self.promote_flow_contradiction(state);
            }
            for event in &target_kills {
                self.apply_kills_one(&mut state.facts, std::slice::from_ref(event));
                self.apply_affine_kills(&mut state.affine, std::slice::from_ref(event));
                self.invalidate_entry_images(
                    state,
                    std::slice::from_ref(event),
                    Some(target_event),
                );
            }
        } else {
            self.apply_kills(state, &target_kills);
        }
        if !commit_reached {
            return;
        }
        for (target, carry) in targets.iter().zip(&placements) {
            if let Some(carry) = carry
                && let Some(destination) = self.set_target_place(target)
            {
                self.establish_measure_datums(node_path, destination, carry, &mut state.facts);
            }
        }
        let destination = |target: &CheckedSetTarget| match target {
            CheckedSetTarget::Place(place) => Some((
                place.binding,
                place
                    .fields
                    .iter()
                    .map(|field| GoalProjection::Field(*field))
                    .collect::<Vec<_>>(),
                place.ty,
            )),
            CheckedSetTarget::ArrayIndex(_)
            | CheckedSetTarget::BufferIndex(_)
            | CheckedSetTarget::RunIndex(_)
            | CheckedSetTarget::SliceIndex(_) => None,
        };
        match values {
            CheckedCommitValues::ResultList { value, .. } => {
                let Some(prepared) = prepared else {
                    return;
                };
                let destinations = targets.iter().map(destination).collect::<Vec<_>>();
                self.establish_result_list_destinations(
                    node_path,
                    &destinations,
                    value,
                    &prepared,
                    &target_kills,
                    state,
                );
            }
            // [LIV-2] a written value list publishes per ordinal: ordinal i's
            // own call establishes into target i and into no other, because
            // no other target receives that value.
            CheckedCommitValues::Written(values) => {
                for ((target, value), prepared) in targets.iter().zip(values).zip(&ordinal_calls) {
                    let Some(prepared) = prepared else {
                        continue;
                    };
                    self.establish_result_list_destinations(
                        node_path,
                        &[destination(target)],
                        value,
                        prepared,
                        &target_kills,
                        state,
                    );
                }
            }
        }
    }

    fn walk_set(
        &mut self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        value: &CheckedExpression,
        force_target_event: bool,
        state: &mut ProofFlowState,
    ) -> SetWalkOutcome {
        // [MSR-3] the [LIV-2] `set`-target placement is minted before the
        // statement's own kills, because the datum it forms is the value the
        // transferred place had immediately before them. The destination is
        // the place this commit writes, which is a plain place or one element
        // position of a run.
        let placement = self.mint_commit_placement(node_path, 0, target, value, &mut state.facts);
        let constructed = matches!(target, CheckedSetTarget::Place(_))
            .then(|| self.mint_construct_placements(node_path, value, &mut state.facts))
            .unwrap_or_default();
        // [SET-1]: the target's base and offset are evaluated before the
        // right-hand side; both are judged at this point, then the commit
        // kill applies.
        let target_reached = self.judge_set_target(target, state);
        let affine_value = self.affine_expression_form(value, &mut state.affine);
        let ExpressionJudgment {
            prepared_call: prepared,
            reached: value_reached,
        } = self.expression_effects(value, state);
        let commit_reached = target_reached && value_reached;
        let receiver_route = commit_reached
            .then(|| {
                prepared
                    .as_ref()
                    .and_then(|prepared| self.direct_receiver_route(target, value, prepared))
            })
            .flatten();
        invalidate_goal_origin_for_set(&mut state.facts, target);
        // [SET-1, ENT-3.S5]: the right-hand side is now evaluated, so its
        // image is established here, on this occurrence's own commit value,
        // under exactly the rules a `let` initializer uses. Establishing it
        // before the target kill is what lets [ENT-5]'s pre-kill closure
        // carry the surviving consequences of that value past the write,
        // instead of losing them with the old target value.
        //
        // Only a direct fragment place can receive that image, so only such a
        // commit forms a value term: with no destination to carry it to, the
        // image would relate nothing to the program's later state.
        let commit_carries_image = matches!(
            target,
            CheckedSetTarget::Place(place) if fragment_type(place.ty).is_some()
        );
        let mut commit_event = None;
        let commit_division = (commit_reached && commit_carries_image)
            .then(|| {
                self.establish_value_image(
                    node_path,
                    ValueImage::Commit(node_path),
                    value,
                    &mut state.facts,
                    &mut commit_event,
                )
            })
            .flatten();
        let commit_dividend = commit_division
            .as_ref()
            .and_then(|_| self.unsigned_division_dividend_form(value, &mut state.affine));
        let mut target_kills = Vec::new();
        self.collect_target_kill(node_path, target, state, &mut target_kills);
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
                self.apply_affine_kills(&mut state.affine, std::slice::from_ref(event));
                self.invalidate_entry_images(
                    state,
                    std::slice::from_ref(event),
                    Some(target_event),
                );
            }
        } else {
            self.apply_kills(state, &target_kills);
        }
        // [MSR-3] the placement's second half, after the target's own kills:
        // the committed place's measures are the datums minted before them.
        if commit_reached
            && let Some(carry) = &placement
            && let Some(destination) = self.set_target_place(target)
        {
            self.establish_measure_datums(node_path, destination, carry, &mut state.facts);
        }
        if commit_reached
            && !constructed.is_empty()
            && let CheckedSetTarget::Place(place) = target
        {
            let base = PlaceTerm {
                root: PlaceRoot::Binding(place.binding),
                deref: self.is_holder(place.binding),
                fields: place.fields.clone(),
            };
            self.establish_construct_placements(node_path, &base, &constructed, &mut state.facts);
        }
        // [CALL-4] a `set` target is an S12 destination, and [CALL-6] puts
        // the establishment after the call's own transfer, consumes and the
        // target's commit and kills — which is exactly this point. The
        // destination list is one entry long because a single-target `set`
        // takes result ordinal zero, and the route is the same one a `let`
        // binder and a `set` target list take: a kernel-domain row publishes
        // from its own declared relation list [BLK-0] and a source callee
        // from its verified summary [FN-9], with the target's own kills the
        // events every substitution must survive.
        if commit_reached
            && let Some(prepared) = prepared.as_ref()
            && let CheckedSetTarget::Place(place) = target
        {
            let destinations = vec![Some((
                place.binding,
                place
                    .fields
                    .iter()
                    .map(|field| GoalProjection::Field(*field))
                    .collect::<Vec<_>>(),
                place.ty,
            ))];
            self.establish_result_list_destinations(
                node_path,
                &destinations,
                value,
                prepared,
                &target_kills,
                state,
            );
        }
        // [ENT-3.S5, ENT-5]: the committed value exists only after the old
        // target facts have died. The equality names the commit value formed
        // above; when no source recognized the right-hand side, no commit
        // value was interned and this commit contributes no fact either.
        let mut set_image_event = None;
        if commit_reached {
            if let Some(commit) = self.interned_commit_value_term(node_path, value) {
                self.establish_commit_copy_fact(
                    node_path,
                    target,
                    commit,
                    &mut state.facts,
                    &mut set_image_event,
                );
            }
            let mut committed_affine = None;
            if let CheckedSetTarget::Place(place) = target
                && place.fields.is_empty()
                && self.affine_binding_type(place.binding).is_some()
                && let Some(value) = affine_value
            {
                committed_affine = Some(value.clone());
                state.affine.values.insert(place.binding, value);
            }
            // The scaled quotient image binds the committed value to the
            // dividend image read before the kill [ENT-3.S7].
            if let (Some(established), Some(dividend), Some(quotient)) =
                (commit_division, commit_dividend, committed_affine)
            {
                self.establish_unsigned_division_image(
                    &quotient,
                    &dividend,
                    established,
                    &mut state.affine,
                );
            }
        }
        if let (Some(prepared), Some(target_event)) = (&prepared, target_event) {
            for receiver in &receivers {
                self.establish_direct_receiver(node_path, receiver, prepared, target_event, state);
            }
        }
        SetWalkOutcome {
            target_event,
            commit_reached,
        }
    }

    fn walk_statement(&mut self, statement: &CheckedStatement, state: &mut ProofFlowState) -> bool {
        match statement {
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                let affine_value = self.affine_expression_form(value, &mut state.affine);
                // [MSR-3] the rebind placement is minted before the
                // initializer's own kills, because the datum it forms is the
                // value the transferred place had immediately before them.
                let rebind = self.mint_rebind_datums(node_path, 0, value, &mut state.facts);
                // [MSR-3] the construct placement is minted at the same
                // point and for the same reason: a field operand is consumed
                // by the construct that fills the field with it.
                let constructed =
                    self.mint_construct_placements(node_path, value, &mut state.facts);
                let judgment = self.expression_effects(value, state);
                self.declare(*binding);
                if judgment.reached
                    && self.affine_binding_type(*binding).is_some()
                    && let Some(value) = affine_value
                {
                    state.affine.values.insert(*binding, value);
                }
                if let Some(prepared) = &judgment.prepared_call {
                    self.establish_direct_result(node_path, *binding, value, prepared, state);
                }
                if judgment.reached
                    && value.ty() == CheckedType::Bool
                    && let Some(relation) = self.direct_comparison(value)
                {
                    state.facts.origins.insert(*binding, relation);
                }
                if judgment.reached {
                    self.record_goal_origin(*binding, value, &mut state.facts);
                }
                // Sources S5, S6, S7, and S9 establish at the binding, after
                // the initializer's own kills [ENT-3, ENT-5].
                let mut event = None;
                let unsigned_division = if judgment.reached {
                    self.establish_value_image(
                        node_path,
                        ValueImage::Binding(*binding),
                        value,
                        &mut state.facts,
                        &mut event,
                    )
                } else {
                    None
                };
                if judgment.reached
                    && let Some(rebind) = &rebind
                {
                    self.establish_rebind_datums(node_path, *binding, rebind, &mut state.facts);
                }
                if judgment.reached && !constructed.is_empty() {
                    let base = self.bound_place(*binding);
                    self.establish_construct_placements(
                        node_path,
                        &base,
                        &constructed,
                        &mut state.facts,
                    );
                }
                if let Some(established) = unsigned_division
                    && let Some(quotient) = state.affine.values.get(binding).cloned()
                    && let Some(dividend) =
                        self.unsigned_division_dividend_form(value, &mut state.affine)
                {
                    self.establish_unsigned_division_image(
                        &quotient,
                        &dividend,
                        established,
                        &mut state.affine,
                    );
                }
                if judgment.reached {
                    self.record_product_atom(*binding, value, &mut state.affine);
                }
                true
            }
            // [GRAM-4, CALL-4] `let (a, b) = f(...);`. The call is judged once;
            // each binder is declared and receives the published relations
            // naming its own result ordinal [ENT-3.S12].
            CheckedStatement::DestructuringLet {
                node_path,
                bindings,
                value,
                ..
            } => {
                // [MSR-3] the destructuring placement is minted before the
                // consume the statement performs, because the datums it
                // forms are the measures the taken-apart value's fields had
                // immediately before it.
                let taken = self.mint_destructuring_placements(node_path, bindings, value, state);
                let judgment = self.expression_effects(value, state);
                let mut destinations = Vec::with_capacity(bindings.len());
                for (binding, ty) in bindings {
                    self.declare(*binding);
                    destinations.push(Some((*binding, Vec::new(), *ty)));
                }
                if judgment.reached {
                    for (ordinal, carry) in &taken {
                        let Some((binding, _)) = bindings.get(*ordinal as usize) else {
                            continue;
                        };
                        let destination = projected_place(self.bound_place(*binding));
                        self.establish_measure_datums(
                            node_path,
                            destination,
                            carry,
                            &mut state.facts,
                        );
                    }
                }
                if let Some(prepared) = &judgment.prepared_call
                    && judgment.reached
                {
                    self.establish_result_list_destinations(
                        node_path,
                        &destinations,
                        value,
                        prepared,
                        &[],
                        state,
                    );
                }
                true
            }
            CheckedStatement::SetList {
                node_path,
                targets,
                values,
            } => {
                self.walk_set_list(node_path, targets, values, state);
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
                // [MSR-3] the displaced half of the placement: the value this
                // `replace` takes out of the target had the target's own
                // measures, and it is minted before the commit that
                // overwrites them. [SET-2]'s commit still establishes no
                // fact of its own — this datum carries a fact the target
                // already had across the naming event, exactly as the
                // rebind, construct, and element placements do.
                let displaced = self.set_target_place(target).and_then(|place| {
                    let placement = match target {
                        CheckedSetTarget::RunIndex(_) => MeasurePlacement::Element,
                        _ => MeasurePlacement::Rebind,
                    };
                    self.mint_measure_datums(
                        node_path,
                        1,
                        placement,
                        place,
                        target.ty(),
                        &mut state.facts,
                    )
                });
                let outcome = self.walk_set(node_path, target, value, false, state);
                self.declare(*binding);
                if outcome.commit_reached
                    && let Some(carry) = &displaced
                {
                    let destination = projected_place(self.bound_place(*binding));
                    self.establish_measure_datums(node_path, destination, carry, &mut state.facts);
                }
                if outcome.commit_reached
                    && self.affine_binding_type(*binding).is_some()
                    && let Some(previous) = previous
                {
                    state.affine.values.insert(*binding, previous);
                }
                true
            }
            CheckedStatement::Evaluate(value)
            | CheckedStatement::Dispose { value, .. }
            | CheckedStatement::DropExpression { value, .. } => {
                let _ = self.expression_effects(value, state);
                true
            }
            CheckedStatement::Proof(proof) => {
                self.judge_affine_relation_subscripts(&proof.target, state);
                for written_use in &proof.uses {
                    if let CheckedProofUseSource::Relation(relation) = &written_use.source {
                        self.judge_affine_relation_subscripts(relation, state);
                    }
                }
                let source_ordinal = u32::try_from(self.source_proofs.len())
                    .expect("local invariant count exceeds the u32 identity space");
                let l0_premises = proof
                    .uses
                    .iter()
                    .map(|written_use| match &written_use.source {
                        CheckedProofUseSource::Named(_) => None,
                        CheckedProofUseSource::Relation(relation) => {
                            self.checked_affine_relation_l0(relation)
                        }
                    })
                    .collect::<Vec<_>>();
                let target_result = self.checked_affine_relation_inequality(
                    &proof.target,
                    &mut state.affine,
                    &mut AffineCheckState::new(),
                );
                let target_failure = target_result
                    .as_ref()
                    .err()
                    .copied()
                    .map(Self::source_proof_formation_failure);
                let target = target_result.as_ref().ok().cloned();
                self.invariant_targets
                    .insert(proof.declaration, target_result);
                let formed_premises = proof
                    .uses
                    .iter()
                    .map(|written_use| match &written_use.source {
                        CheckedProofUseSource::Named(declaration) => self
                            .invariant_targets
                            .get(declaration)
                            .cloned()
                            .unwrap_or(Err(AffineCheckError::CoefficientMismatch)),
                        CheckedProofUseSource::Relation(relation) => self
                            .checked_affine_relation_inequality(
                                relation,
                                &mut state.affine,
                                &mut AffineCheckState::new(),
                            ),
                    })
                    .collect::<Vec<_>>();
                let source_failure_use_index = formed_premises
                    .iter()
                    .position(|premise| premise.is_err())
                    .map(|index| {
                        u32::try_from(index).expect("source-proof use index fits the u32 identity")
                    });
                let source_failure = source_failure_use_index.map(|index| {
                    let error = formed_premises
                        [usize::try_from(index).expect("source-proof use index fits usize")]
                    .as_ref()
                    .expect_err("source failure index names a failed source")
                    .to_owned();
                    Self::source_proof_formation_failure(error)
                });
                let premises = formed_premises
                    .iter()
                    .map(|premise| premise.as_ref().ok().cloned())
                    .collect::<Vec<_>>();
                let published_premises = proof
                    .uses
                    .iter()
                    .map(|written_use| match &written_use.source {
                        CheckedProofUseSource::Named(declaration) => self
                            .invariant_targets
                            .get(declaration)
                            .and_then(|formed| formed.as_ref().ok())
                            .zip(state.affine.published_invariants.get(declaration))
                            .is_some_and(|(declared, published)| declared == published),
                        CheckedProofUseSource::Relation(_) => false,
                    })
                    .collect::<Vec<_>>();
                let named_premises = proof
                    .uses
                    .iter()
                    .map(|written_use| {
                        matches!(&written_use.source, CheckedProofUseSource::Named(_))
                    })
                    .collect::<Vec<_>>();
                let multiplicities = proof
                    .uses
                    .iter()
                    .map(|written_use| {
                        self.certificate_multiplicity(written_use.multiplicity, &mut state.affine)
                    })
                    .collect::<Vec<_>>();
                let certificate_premises = premises
                    .iter()
                    .zip(&multiplicities)
                    .map(|(premise, multiplicity)| premise.clone().zip(multiplicity.clone()))
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
                let certificate_sum = if proof.uses.is_empty() || source_failure.is_some() {
                    None
                } else {
                    Some(certificate_premises.as_deref().map_or_else(
                        || Err((SourceProofCertificateFailure::FormationCapacity, 0)),
                        |premises| self.source_proof_sum(premises),
                    ))
                };

                // Every written `use` is proved against the same pre-proof
                // program point. No premise established by this statement can
                // help another premise in the same statement.
                let premise_results = self.source_proof_premise_results(
                    &premises,
                    &l0_premises,
                    &named_premises,
                    &published_premises,
                    &state.affine,
                    &state.facts,
                );
                let certificate_failure = certificate_sum
                    .as_ref()
                    .and_then(|sum| sum.as_ref().err().copied());
                let certificate_failure_kind = certificate_failure.map(|(failure, _)| failure);
                let certificate_failure_use_index =
                    certificate_failure.map(|(_, use_index)| use_index);
                let first_unproved_premise =
                    premise_results
                        .iter()
                        .position(|proved| !proved)
                        .map(|index| {
                            u32::try_from(index)
                                .expect("source-proof use index fits the u32 identity")
                        });
                let residual = if proof.uses.is_empty() {
                    Ok(auto_proved)
                } else if target_failure.is_some()
                    || source_failure.is_some()
                    || certificate_failure_kind.is_some()
                    || first_unproved_premise.is_some()
                {
                    Ok(false)
                } else {
                    match (
                        target.as_ref(),
                        certificate_sum.as_ref().and_then(|sum| sum.as_ref().ok()),
                    ) {
                        (Some(target), Some(sum)) => self.source_proof_certificate_residual(
                            target,
                            sum,
                            &state.affine,
                            &state.facts,
                        ),
                        _ => Err(SourceProofCertificateFailure::FormationCapacity),
                    }
                };
                let residual_failure = residual.as_ref().err().copied();
                let check = SourceProofCheck {
                    premises: premise_results,
                    first_unproved_premise,
                    combination: residual.unwrap_or(false),
                    target_failure,
                    source_failure,
                    source_failure_use_index,
                    certificate_failure: certificate_failure_kind,
                    certificate_failure_use_index,
                    residual_failure,
                    redundant,
                };

                if let Some(target) = target {
                    let fact = ActiveAffineFact {
                        inequality: target.clone(),
                        evidence: AffineFactEvidence::Source(SourceAffineFactRef::SourceProof {
                            source_ordinal,
                        }),
                        active_loops: Vec::new(),
                    };
                    if check.discharged() {
                        state.affine.facts.push(fact);
                        state
                            .affine
                            .published_invariants
                            .insert(proof.declaration, target);
                    }
                }
                self.source_proofs.push(SourceProofOutcome {
                    node_path: proof.node_path.clone(),
                    use_node_paths: proof
                        .uses
                        .iter()
                        .map(|written_use| written_use.node_path.clone())
                        .collect(),
                    source_ordinal,
                    name: proof.name.clone(),
                    certificate_written: !proof.uses.is_empty(),
                    check,
                });
                true
            }
            CheckedStatement::Return {
                node_path, value, ..
            } => {
                let affine_result = self.affine_pure_expression_form(value, &mut state.affine);
                // [FN-9] the relation is queried "immediately before return
                // transfer and edge cleanup": the returned value's own
                // consume is that transfer, so it has not happened at the
                // query point and its kills are applied after. Nothing reads
                // the state between the two, because a return has no normal
                // continuation.
                let judgment = self.judge_expression(value, state);
                let mut events = Vec::new();
                self.collect_expression_kills(value, &mut events);
                self.judge_postcondition_return(
                    node_path,
                    state,
                    affine_result.as_ref(),
                    judgment.reached,
                );
                self.apply_kills(state, &events);
                false
            }
            CheckedStatement::Give {
                node_path, value, ..
            } => {
                let judgment = self.expression_effects(value, state);
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
                    let give_goal_origin = if judgment.reached && result_type == CheckedType::Bool {
                        self.admitted_value_goal_expression(value)
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
                // [MSR-3] the payload placement is minted before the `match`
                // consumes its scrutinee, because the datums it forms are the
                // measures the payload had immediately before that consume.
                let payload = self.mint_payload_placements(scrutinee, *enum_type, &mut state.facts);
                let judgment = self.expression_effects(scrutinee, state);
                let facts = if judgment.reached {
                    self.arm_facts(scrutinee, *enum_type, &state.facts)
                } else {
                    ArmFacts::default()
                };
                let prepared = judgment.prepared_call;
                let mut exits = Vec::new();
                for arm in arms {
                    let direct_call = prepared
                        .as_ref()
                        .map(|prepared| (scrutinee, *enum_type, prepared));
                    if let Some(exit) =
                        self.walk_arm(arm, state, &facts, direct_call, payload.as_ref())
                    {
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
                // [MSR-3] the payload placement is minted before the `match`
                // consumes its scrutinee, because the datums it forms are the
                // measures the payload had immediately before that consume.
                let payload = self.mint_payload_placements(scrutinee, *enum_type, &mut state.facts);
                let judgment = self.expression_effects(scrutinee, state);
                let facts = if judgment.reached {
                    self.arm_facts(scrutinee, *enum_type, &state.facts)
                } else {
                    ArmFacts::default()
                };
                let prepared = judgment.prepared_call;
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
                    let _ = self.walk_arm(arm, state, &facts, direct_call, payload.as_ref());
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
                for invariant in invariants {
                    self.judge_affine_relation_subscripts(&invariant.relation, state);
                }
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
                let invariant_declarations = invariants
                    .iter()
                    .map(|invariant| invariant.declaration)
                    .collect::<Vec<_>>();
                let head_entry_images = state.entry_images.clone();
                self.loops.push(LoopFrame {
                    id: *id,
                    invariant_declarations: invariant_declarations.clone().into_boxed_slice(),
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
                self.record_loop_invariant_outcomes(*id, invariants, &base, &step, None);

                let frame = self.loops.pop();
                let mut breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                for break_state in &mut breaks {
                    Self::remove_active_loop_invariants(
                        &mut break_state.affine,
                        *id,
                        &invariant_declarations,
                    );
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

                for invariant in invariants {
                    self.judge_affine_relation_subscripts(&invariant.relation, state);
                }
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
                            path: Vec::new(),
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
                let invariant_declarations = invariants
                    .iter()
                    .map(|invariant| invariant.declaration)
                    .collect::<Vec<_>>();
                self.loops.push(LoopFrame {
                    id: *id,
                    invariant_declarations: invariant_declarations.clone().into_boxed_slice(),
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

                self.record_loop_invariant_outcomes(*id, invariants, &base, &step, Some(*binder));
                let step_batch = step.iter().all(|proved| proved.unwrap_or(true));
                let export = lower_le_upper && base_batch && step_batch && hidden_update;
                let frame = self.loops.pop();
                let mut breaks = frame.map(|frame| frame.breaks).unwrap_or_default();
                for break_state in &mut breaks {
                    Self::remove_active_loop_invariants(
                        &mut break_state.affine,
                        *id,
                        &invariant_declarations,
                    );
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
                        if !self.affine_fact_uses_only_outer_values(
                            &inequality,
                            &normalized,
                            *binder,
                        ) {
                            continue;
                        }
                        let fact = ActiveAffineFact {
                            inequality,
                            evidence: AffineFactEvidence::Source(
                                SourceAffineFactRef::LoopInvariant(SourceLoopInvariantRef {
                                    loop_id: *id,
                                    source_ordinal: u32::try_from(source_ordinal)
                                        .expect("loop invariant ordinal exceeds u32"),
                                }),
                            ),
                            active_loops: Vec::new(),
                        };
                        exhaustion.affine.facts.push(fact);
                    }
                }
                Self::remove_active_loop_invariants(
                    &mut exhaustion.affine,
                    *id,
                    &invariant_declarations,
                );
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
        payload: Option<&PayloadPlacement>,
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
        // [MSR-3] the payload placement's second half: on the arm whose
        // variant carries the payload, the binder that names it has the
        // measures the payload had before the consume.
        if let Some(payload) = payload.filter(|payload| payload.tag == arm.tag) {
            for (field, carry) in &payload.carried {
                let Some(binder) = arm.binders.iter().find(|binder| binder.field == *field) else {
                    continue;
                };
                let destination = projected_place(self.bound_place(binder.binding));
                self.establish_measure_datums(
                    &binder.node_path,
                    destination,
                    carry,
                    &mut state.facts,
                );
            }
        }
        // [CALL-6] a kernel-domain row publishes on the arm its route names
        // from its own declared relation list [BLK-0]; a source callee takes
        // the direct-match route below, which is what [FN-9] gives it.
        if let Some((scrutinee, enum_type, prepared)) = direct_call
            && matches!(prepared.callee, PreparedCallee::Kernel(_))
        {
            self.establish_kernel_match_relations(
                scrutinee,
                enum_type,
                arm,
                prepared,
                &mut state.facts,
            );
        }
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
            | CheckedStatement::DestructuringLet { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::SetList { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::Dispose { .. }
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
            | CheckedStatement::DestructuringLet { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::Dispose { value, .. }
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
            // [CALL-4] every target of one target list commits on the same
            // edge, so the loop-carried kill set is the union of the commits.
            CheckedStatement::SetList {
                node_path,
                targets,
                values,
                ..
            } => {
                if normal_reaches {
                    for (index, target) in targets.iter().enumerate() {
                        // [LIV-2] a result list judges the one call once, at
                        // the first target; a value list judges ordinal i
                        // beside target i.
                        match values {
                            CheckedCommitValues::ResultList { value, .. } => {
                                if index == 0 {
                                    self.collect_set_kills(node_path, target, value, kills);
                                } else {
                                    self.collect_target_kill_only(node_path, target, kills);
                                }
                            }
                            CheckedCommitValues::Written(values) => {
                                let Some(value) = values.get(index) else {
                                    continue;
                                };
                                self.collect_set_kills(node_path, target, value, kills);
                            }
                        }
                    }
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
        self.push_commit_kill(node_path, target, &mut events, kills);
    }

    /// One commit's own kill, for a target whose ordinal value another target
    /// of the same statement already judged [LIV-2].
    fn collect_target_kill_only(
        &self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        kills: &mut LoopKills,
    ) {
        self.push_commit_kill(node_path, target, &mut Vec::new(), kills);
    }

    fn push_commit_kill(
        &self,
        node_path: &crate::NodePath,
        target: &CheckedSetTarget,
        events: &mut Vec<KillEvent>,
        kills: &mut LoopKills,
    ) {
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
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
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
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
                    element: true,
                    source: node_path.clone(),
                });
            }
            CheckedSetTarget::SliceIndex(target) => {
                let spelled = PlaceTerm {
                    root: PlaceRoot::Binding(target.root.binding),
                    deref: self.is_holder(target.root.binding),
                    fields: Vec::new(),
                };
                events.push(KillEvent::Write {
                    place: element_write_place(self.resolve(&spelled), PlaceOffset::Opaque),
                    element: true,
                    source: node_path.clone(),
                });
            }
            CheckedSetTarget::RunIndex(target) => {
                events.push(KillEvent::Write {
                    place: element_write_place(
                        self.container_root_place(&target.root),
                        target.place_offset,
                    ),
                    element: true,
                    source: node_path.clone(),
                });
            }
        }
        kills.push_event_group(std::mem::take(events));
    }

    fn apply_loop_kills_one(&mut self, state: &mut FactState, kills: &LoopKills) {
        self.materialize_before_event_kill(state, &kills.events);
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
        self.apply_affine_kills(&mut states.affine, &kills.events);
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

    /// One [OP-4] subscript offset, in the spelling the source wrote it in.
    fn render_offset(&self, offset: PlaceOffset) -> String {
        match offset {
            PlaceOffset::Literal(value) => value.to_string(),
            PlaceOffset::Binding(binding) => self.binding_name(binding),
            PlaceOffset::Const(declaration) => format!("<const-parameter:{}>", declaration.index()),
            PlaceOffset::Opaque => "?".to_owned(),
        }
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
                PlaceProjection::Subscript(offset) => {
                    rendered.push_str(&format!("[{}]", self.render_offset(*offset)));
                    ty = ty.and_then(element_type);
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
            CheckedNominalKind::Box { referent, .. } => Some(referent),
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
            // An undisplaced relation reads as the writer wrote it; a
            // displaced one names its difference, exactly as a bound does.
            Relation::Equal {
                left,
                right,
                difference: 0,
            } => format!("{} = {}", self.render_term(*left), self.render_term(*right)),
            Relation::Equal {
                left,
                right,
                difference,
            } => format!(
                "{} - {} = {difference}",
                self.render_term(*left),
                self.render_term(*right)
            ),
            Relation::Distinct {
                left,
                right,
                difference: 0,
            } => format!(
                "{} != {}",
                self.render_term(*left),
                self.render_term(*right)
            ),
            Relation::Distinct {
                left,
                right,
                difference,
            } => format!(
                "{} - {} != {difference}",
                self.render_term(*left),
                self.render_term(*right)
            ),
        }
    }

    fn render_term(&self, term: TermId) -> String {
        match self.terms.kind(term) {
            TermKind::Zero => "0".to_owned(),
            TermKind::Constant(value) => value.to_string(),
            TermKind::ConstParameter(_) => "<const parameter>".to_owned(),
            TermKind::Place(place, _) => self.render_place(place),
            TermKind::ProjectedPlace(place, _) => self.render_projected_place(place),
            TermKind::Measure(measure, place) => {
                format!("{}({})", measure.spelling(), self.render_place(place))
            }
            TermKind::ProjectedMeasure(measure, place) => {
                format!(
                    "{}({})",
                    measure.spelling(),
                    self.render_projected_place(place)
                )
            }
            TermKind::CountedCapture { side, .. } => match side {
                CountedCaptureSide::Lower => "<counted lower capture>".to_owned(),
                CountedCaptureSide::Upper => "<counted upper capture>".to_owned(),
            },
            TermKind::CommitValue { .. } => "<assigned value>".to_owned(),
            TermKind::CallDatum { measure, .. } => measure.map_or_else(
                || "<argument value at the call>".to_owned(),
                |measure| format!("<argument {} at the call>", measure.spelling()),
            ),
            // [MSR-3] an entry datum is what the writer wrote: a measure of
            // the parameter, at the one state an `ensures` gives it.
            TermKind::EntryDatum {
                formal,
                projections,
                measure,
            } => {
                let mut place = self
                    .function
                    .parameters
                    .get(*formal as usize)
                    .map_or_else(|| "?".to_owned(), |parameter| parameter.name.clone());
                for projection in projections {
                    match projection {
                        CallDatumProjection::Deref => place = format!("deref({place})"),
                        CallDatumProjection::Field(field) => {
                            place = format!("{place}.{field}");
                        }
                        CallDatumProjection::Subscript(offset) => {
                            place = format!("{place}[{}]", self.render_offset(*offset));
                        }
                    }
                }
                format!("{}({place})", measure.spelling())
            }
            // A measure datum has no source spelling of its own: it is the
            // measure the carried value had at the event that renamed it.
            TermKind::MeasureDatum {
                measure, placement, ..
            } => {
                let event = match placement {
                    MeasurePlacement::Rebind => "the rebind",
                    MeasurePlacement::Construct => "the construct",
                    MeasurePlacement::Destructuring => "the destructuring",
                    MeasurePlacement::Element => "the element position",
                    MeasurePlacement::Payload => "the payload",
                };
                format!("<{} at {event}>", measure.spelling())
            }
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
            // Source cannot name this datum: render its structural source
            // role rather than inventing an expression that could reread a
            // different runtime value.
            GoalDatum::EvaluatedValue {
                occurrence,
                captured_type,
                projections,
                ..
            } => {
                let base = match occurrence {
                    EvaluatedValueOccurrence::CallArgument { argument, .. } => {
                        format!("<argument #{argument} pre-transfer value>")
                    }
                    EvaluatedValueOccurrence::ObligationOperand { operand, .. } => {
                        format!("<operand #{operand} evaluated value>")
                    }
                };
                self.render_goal_projections(base, Some(*captured_type), projections)
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
                GoalProjection::Subscript(offset) => {
                    rendered.push_str(&format!("[{}]", self.render_offset(*offset)));
                    ty = ty.and_then(element_type);
                }
            }
        }
        rendered
    }

    fn render_expression(&self, expression: &CheckedExpression) -> String {
        match expression {
            CheckedExpression::Constant(CheckedValue::ConstGeneric { .. }) => {
                "<const parameter>".to_owned()
            }
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
            difference: 0,
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

/// The type one slot of an indexable base holds [OP-4, BLK-1].
fn element_type(input: CheckedType) -> Option<CheckedType> {
    match input {
        CheckedType::Buffer { element } => Some(element.ty()),
        CheckedType::Slice { element, .. } => Some(element.ty()),
        CheckedType::FixedVector { element, .. } | CheckedType::Vector { element, .. } => {
            Some(element.ty())
        }
        _ => None,
    }
}

/// The place one element write names [MSR-2]: the base it selects through,
/// with the element it writes appended.
///
/// [MSR-2] states the granularity over storage: a write at an element
/// position of P overlaps the descriptor storage of `P[i]` and none of P's
/// own, so the event carries `P[i]` and the overlap relation reads it.
fn element_write_place(mut base: ResolvedPlace, offset: PlaceOffset) -> ResolvedPlace {
    base.path.push(PlaceStep::Subscript(offset));
    base
}

/// The same place in the exact source-order form every measure term and
/// [OP-4] obligation is stated over.
fn projected_place(base: PlaceTerm) -> ProjectedPlaceTerm {
    let mut projections = Vec::new();
    if base.deref {
        projections.push(PlaceProjection::Deref);
    }
    projections.extend(
        base.fields
            .iter()
            .map(|field| PlaceProjection::Field(*field)),
    );
    ProjectedPlaceTerm {
        root: base.root,
        projections,
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
            PlaceProjection::Deref | PlaceProjection::Subscript(_) => None,
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
            render_operation_spelling(if *equal { "eeq" } else { "ene" }, arguments)
        }
        GoalOperation::NumericConversion {
            source,
            destination,
        } => format!(
            "cvt::<{}, {}>({})",
            numeric_type_name(*source),
            numeric_type_name(*destination),
            arguments.join(", ")
        ),
        GoalOperation::Reinterpret {
            source,
            destination,
        } => format!(
            "reinterpret::<{}, {}>({})",
            numeric_type_name(*source),
            numeric_type_name(*destination),
            arguments.join(", ")
        ),
        GoalOperation::ArrayFill { .. } => render_operation_spelling("array_new", arguments),
        GoalOperation::ArrayMeasure { .. }
        | GoalOperation::BufferMeasure { .. }
        | GoalOperation::SliceMeasure { .. } => render_operation_spelling("len_of", arguments),
        // [MSR-1]: one quantity, one name, term and reader alike, so the
        // residual names the measure the row reads rather than one of them.
        GoalOperation::ContainerMeasure { measure, .. } => {
            render_operation_spelling(measure.spelling(), arguments)
        }
        GoalOperation::ArrayIndex { .. }
        | GoalOperation::BufferIndex { .. }
        | GoalOperation::RunIndex { .. }
        | GoalOperation::SliceIndex { .. } => match arguments {
            [collection, offset] => format!("{collection}[{offset}]"),
            _ => "<invalid index goal>".to_owned(),
        },
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
            declares: false,
        });

        invalidate_goal_origin_for_set(&mut state, &target);

        assert!(!state.goal_origins.contains_key(&binding));
    }
}

#[cfg(test)]
mod affine_pair_tests {
    use super::{
        AffineCheckState, AffineInequality, AffineTermId, AutomaticAffinePremise,
        first_two_premise_candidate, interval_proves,
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

    /// The pair enumeration hands each accumulated sum to its caller, so these
    /// tests take the residual against the target exactly where the semantic
    /// checker does.
    fn interval_closes_after(
        target: &AffineInequality,
    ) -> impl FnMut(&AffineInequality, &mut AffineCheckState) -> Option<()> {
        move |sum, check| {
            let residual = AffineInequality::residual_after(target, sum, check).ok()?;
            interval_closes_without_atoms(&residual, check)
        }
    }

    #[test]
    fn two_premise_enumeration_includes_one_fact_used_twice() {
        let target = inequality(&[(0, 2)], 0);
        let premises = [premise(inequality(&[(0, 1)], 0))];
        let selected = first_two_premise_candidate(
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_after(&target),
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
        let two = first_two_premise_candidate(
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_after(&inequality(&[(0, 1), (1, 1)], 0)),
        );
        assert!(two.is_some());

        let three = first_two_premise_candidate(
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_after(&inequality(&[(0, 1), (1, 1), (2, 1)], 0)),
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
        let forward_result = first_two_premise_candidate(
            &forward,
            &mut AffineCheckState::new(),
            interval_closes_after(&inequality(&[(0, 1)], 0)),
        );
        let reverse_result = first_two_premise_candidate(
            &reverse,
            &mut AffineCheckState::new(),
            interval_closes_after(&inequality(&[(1, 1)], 0)),
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
        let selected = first_two_premise_candidate(
            &premises,
            &mut AffineCheckState::new(),
            interval_closes_after(&inequality(&[(0, 1)], 0)),
        );
        assert!(matches!(selected, Some((1, 2, ()))));
    }
}

/// The [MSR-1] measured type of one checked type, if the measure table gives
/// it a row.
/// The written constant one measured type carries, when a cell of its
/// [MSR-1] row is that constant [MSR-2].
const fn type_constant(ty: CheckedType) -> Option<CheckedConst> {
    match ty {
        CheckedType::FixedVector { length, .. } => Some(length),
        CheckedType::Extent { bytes, .. } => Some(bytes),
        _ => None,
    }
}

const fn measured_kind(ty: CheckedType) -> Option<MeasuredKind> {
    match ty {
        CheckedType::Array { .. } => Some(MeasuredKind::Array),
        CheckedType::Buffer { .. } => Some(MeasuredKind::Buffer),
        CheckedType::FixedVector { .. } => Some(MeasuredKind::FixedVector),
        CheckedType::Vector { .. } => Some(MeasuredKind::Vector),
        CheckedType::Extent { .. } => Some(MeasuredKind::Extent),
        CheckedType::Slice { .. } => Some(MeasuredKind::Slice),
        _ => None,
    }
}
