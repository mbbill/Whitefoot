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

pub(crate) use state::{DerivationId, DerivationRootKind, ProofView, StrictDerivationRootKind};
use state::{DerivationInventory, DerivationLedger};
#[cfg(not(test))]
use term::TermId;

#[cfg(test)]
pub(crate) use state::{
    ClaimLifecycleKind, CountedRootAtom, DerivationNode, FlowEvent, FlowEventId, FlowEventKind,
    GoalId, GoalSign, ImplicitBoundKind, JoinParent, Relation,
};
#[cfg(test)]
pub(crate) use term::{
    CountedCaptureSide, LengthBound, PlaceProjection, PlaceRoot, TermId, TermKind, ZERO, type_range,
};

use std::collections::{BTreeSet, HashMap};

use super::goal::ConcreteGoal;
use super::model::{
    BindingId, CheckedConstant, CheckedConstantId, CheckedExpression, CheckedFunction,
    CheckedIntegerOperation, CheckedMode, CheckedNominal, CheckedSetTarget, CheckedStatement,
    CheckedType, CheckedValue, FunctionId, IntegerType,
};
use super::postcondition::CheckedPostcondition;
use crate::{DeclarationId, NodePath, SemanticCompilerFailure, SyntaxCoordinate};

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
    /// True only for the marked concrete program-entry instance. The flow
    /// uses this bit solely to retain the post-setup, pre-S4 U judgment that
    /// CLM-3 later consumes.
    pub(crate) marked_program_start: bool,
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

/// The [ENT-6] obligation family one outcome belongs to. The bounds family
/// rejects citing OP-4 at its `psuffix` node; the overflow family rejects
/// citing OP-2 at its `infix` node. Both share one outcome inventory,
/// derivation-root namespace, and strict U re-judgment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObligationFamily {
    /// A subscript bounds obligation `i < len(P)` [OP-4].
    Bounds,
    /// A constant-operand-class bare `+`/`-`/`*` overflow obligation [OP-2].
    Overflow,
}

/// [ENT-6] disposition of one obligation conjunct, judged at its source node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObligationOutcome {
    /// The subscript's `psuffix` node or the class call's `infix` node the
    /// obligation is attached to — one record per subscript in a chain and
    /// one per overflow conjunct [ENT-6].
    pub(crate) node_path: NodePath,
    /// The obligation family this conjunct belongs to.
    pub(crate) family: ObligationFamily,
    /// The bounds obligation has one upper-bound conjunct at ordinal zero;
    /// the overflow obligation has its upper conjunct at ordinal zero and
    /// its lower conjunct at ordinal one [ENT-6].
    pub(crate) conjunct: u8,
    /// The normalized conjunct as `left - right <= bound`: the bounds family
    /// requests `offset - len(base) <= -1`; the overflow family requests
    /// `operand - Z <= c` at ordinal zero and `Z - operand <= c` at ordinal
    /// one, with both sides Z for a ground conjunct. `left` is absent only
    /// when the checked operand is outside ENT-2's term vocabulary; the
    /// exact checked expression remains recoverable from `node_path` in the
    /// same function.
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

/// The bare trapping operation of one [OP-2] constant-operand-class call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverflowClassOperation {
    Add,
    Subtract,
    Multiply,
}

/// [OP-2] constant-operand-class decomposition of one bare trapping `+`,
/// `-`, or `*` call: the overflow obligation is expressible exactly when at
/// least one operand atom reads as an [ENT-2] constant. One shared
/// classifier serves the checker's trap/effect classification and the
/// flow's obligation judgment, so the two views cannot drift.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OverflowOperandClass<'expression> {
    /// One constant operand; the obligation folds onto the other operand.
    Folded {
        operation: OverflowClassOperation,
        /// The constant operand's exact mathematical value.
        constant: i128,
        /// The constant occupies the left operand position (`c - t`); the
        /// distinction matters only for subtraction.
        constant_is_left: bool,
        /// The non-constant operand the folded conjuncts bound.
        operand: &'expression CheckedExpression,
    },
    /// Two constant operands; the obligation is ground.
    Ground {
        operation: OverflowClassOperation,
        left: i128,
        right: i128,
    },
}

/// Classifies one integer operation for the [ENT-6] overflow obligation
/// family. Returns `None` for every operation outside the bare trapping
/// add/subtract/multiply family and for every such call with two
/// non-constant operands — the retained trapping class.
pub(crate) fn overflow_obligation_class(
    operation: CheckedIntegerOperation,
    arguments: &[CheckedExpression],
) -> Option<OverflowOperandClass<'_>> {
    let operation = match operation {
        CheckedIntegerOperation::AddTrap => OverflowClassOperation::Add,
        CheckedIntegerOperation::SubtractTrap => OverflowClassOperation::Subtract,
        CheckedIntegerOperation::MultiplyTrap => OverflowClassOperation::Multiply,
        _ => return None,
    };
    let [left, right] = arguments else {
        return None;
    };
    let constant = |expression: &CheckedExpression| match expression {
        CheckedExpression::Constant(CheckedValue::Integer { ty, bits })
        | CheckedExpression::NamedConstant {
            value: CheckedValue::Integer { ty, bits },
            ..
        } => Some(term::integer_value(*ty, *bits)),
        _ => None,
    };
    match (constant(left), constant(right)) {
        (Some(left), Some(right)) => Some(OverflowOperandClass::Ground {
            operation,
            left,
            right,
        }),
        (Some(constant), None) => Some(OverflowOperandClass::Folded {
            operation,
            constant,
            constant_is_left: true,
            operand: right,
        }),
        (None, Some(constant)) => Some(OverflowOperandClass::Folded {
            operation,
            constant,
            constant_is_left: false,
            operand: left,
        }),
        (None, None) => None,
    }
}

/// The two normalized [ENT-6] overflow conjuncts of one class call over one
/// selected fragment type: ordinal zero the upper bound `operand - Z <=
/// upper`, ordinal one the lower bound `Z - operand <= lower`. A ground
/// obligation relates Z to Z on both sides with bound 0 (in range) or -1
/// (inevitable overflow).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OverflowConjuncts {
    pub(crate) upper: i128,
    pub(crate) lower: i128,
    /// Both conjuncts relate Z to Z; no operand term participates.
    pub(crate) ground: bool,
    /// The exact decimal mathematical result of a ground obligation, for
    /// the `z outside T` residual rendering.
    pub(crate) ground_result: Option<GroundResult>,
}

/// Exact mathematical result of a two-constant class call. A multiply of
/// two 64-bit magnitudes can exceed `i128`, so magnitude and sign are kept
/// separately; every value is renderable in decimal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GroundResult {
    pub(crate) negative: bool,
    pub(crate) magnitude: u128,
}

impl GroundResult {
    fn from_value(value: i128) -> Self {
        Self {
            negative: value < 0,
            magnitude: value.unsigned_abs(),
        }
    }

    fn in_range(self, low: i128, high: i128) -> bool {
        let Ok(magnitude) = i128::try_from(self.magnitude) else {
            return false;
        };
        let value = if self.negative { -magnitude } else { magnitude };
        low <= value && value <= high
    }

    pub(crate) fn render(self) -> String {
        if self.negative && self.magnitude != 0 {
            format!("-{}", self.magnitude)
        } else {
            format!("{}", self.magnitude)
        }
    }
}

/// Exact-quotient division rounding toward negative infinity.
const fn floor_div(dividend: i128, divisor: i128) -> i128 {
    let quotient = dividend / divisor;
    if dividend % divisor != 0 && ((dividend < 0) != (divisor < 0)) {
        quotient - 1
    } else {
        quotient
    }
}

/// Exact-quotient division rounding toward positive infinity.
const fn ceil_div(dividend: i128, divisor: i128) -> i128 {
    let quotient = dividend / divisor;
    if dividend % divisor != 0 && ((dividend < 0) == (divisor < 0)) {
        quotient + 1
    } else {
        quotient
    }
}

/// Folds one class call's overflow obligation into its two checker-computed
/// conjunct constants over the selected fragment type, exactly as [ENT-6]
/// tabulates: the fold is an equivalence over mathematical integers, never
/// an approximation. Every intermediate fits `i128` because type extrema
/// and operand constants are below `2^64` in magnitude, except a ground
/// multiply, whose exact result keeps magnitude and sign separately.
pub(crate) fn overflow_conjuncts(
    class: &OverflowOperandClass<'_>,
    ty: IntegerType,
) -> OverflowConjuncts {
    let (low, high) = term::type_range(ty);
    let folded = |upper: i128, lower: i128| OverflowConjuncts {
        upper,
        lower,
        ground: false,
        ground_result: None,
    };
    match *class {
        OverflowOperandClass::Folded {
            operation: OverflowClassOperation::Add,
            constant,
            ..
        } => folded(high - constant, constant - low),
        OverflowOperandClass::Folded {
            operation: OverflowClassOperation::Subtract,
            constant,
            constant_is_left: false,
            ..
        } => folded(high + constant, -low - constant),
        OverflowOperandClass::Folded {
            operation: OverflowClassOperation::Subtract,
            constant,
            constant_is_left: true,
            ..
        } => folded(constant - low, high - constant),
        OverflowOperandClass::Folded {
            operation: OverflowClassOperation::Multiply,
            constant,
            ..
        } => {
            if constant == 0 {
                // Zero times anything is zero, in range for every type.
                OverflowConjuncts {
                    upper: 0,
                    lower: 0,
                    ground: true,
                    ground_result: Some(GroundResult::from_value(0)),
                }
            } else if constant > 0 {
                folded(floor_div(high, constant), -ceil_div(low, constant))
            } else {
                folded(floor_div(low, constant), -ceil_div(high, constant))
            }
        }
        OverflowOperandClass::Ground {
            operation,
            left,
            right,
        } => {
            let result = match operation {
                OverflowClassOperation::Add => {
                    left.checked_add(right).map(GroundResult::from_value)
                }
                OverflowClassOperation::Subtract => {
                    left.checked_sub(right).map(GroundResult::from_value)
                }
                OverflowClassOperation::Multiply => Some(GroundResult {
                    negative: (left < 0) != (right < 0) && left != 0 && right != 0,
                    magnitude: left.unsigned_abs() * right.unsigned_abs(),
                }),
            }
            .expect("class constants are below 2^64 in magnitude, so add and subtract fit i128");
            let bound = if result.in_range(low, high) { 0 } else { -1 };
            OverflowConjuncts {
                upper: bound,
                lower: bound,
                ground: true,
                ground_result: Some(result),
            }
        }
    }
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
    /// Strictly outgoing callee components in callee-before-caller order.
    pub(crate) outgoing: Vec<u32>,
    pub(crate) summaries: Vec<VerifiedPostconditionSummary>,
}

/// One checked concrete ordinary-user-call occurrence collected by the same
/// structural walk that builds the FN-9/CLM-3 SCC graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConcreteCallOccurrence {
    pub(crate) caller: FunctionId,
    pub(crate) node_path: NodePath,
    pub(crate) callee: FunctionId,
}

/// Program-private SCC schedule retained for the later caller-publication
/// handoff. An empty schedule is the no-postcondition, no-strict-root fast
/// path.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PostconditionSchedule {
    pub(crate) components: Vec<PostconditionComponent>,
    /// Dense function-to-component map, using ordered component ordinals.
    pub(crate) function_components: Vec<u32>,
    /// Stable caller-instance then call-NodePath order.
    pub(crate) calls: Vec<ConcreteCallOccurrence>,
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
    /// Exact canonical spelling of the predicate expression.
    pub(crate) predicate: String,
    /// The writer's compile-time review justification.
    pub(crate) justification: String,
    pub(crate) disposition: ClaimDisposition,
    /// The already-selected proof of a redundant predicate or refuted
    /// negation. Retained claims deliberately have no lifecycle proof.
    pub(crate) lifecycle_derivation: Option<DerivationId>,
}

/// Bundle-local source identity of one concrete checked claim occurrence.
/// This value deliberately has no hash or meaning outside its checked program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimSourceIdentity {
    pub(crate) logical_path: String,
    pub(crate) coordinate: SyntaxCoordinate,
    pub(crate) node_path: NodePath,
    pub(crate) function: FunctionId,
    pub(crate) function_symbol: String,
}

/// Existing provenance disposition attached to one claim-supported root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClaimUseProvenance {
    /// The retained query is not a protected leaf governed by PRV-1/2/3.
    NotApplicable,
    /// Exact success-side PRV disposition for the protected leaf.
    ProtectedLeaf {
        disposition: super::provenance::LocalLeafDisposition,
        direct_demands: Vec<super::provenance::DirectDemand>,
        structural_bridges: Vec<super::provenance::StructuralBridge>,
        subject_bridges: Vec<super::provenance::SubjectBridge>,
        calls: Vec<super::provenance::BridgeCallLink>,
    },
    /// Exact success-side provenance inventory for a claim-discharged call
    /// requirement. Argument ordinals cover the call densely; bridge links
    /// may be empty when the call reaches no protected leaf.
    Call {
        arguments: Vec<super::provenance::CallArgumentDisposition>,
        bridges: Vec<super::provenance::BridgeCallLink>,
    },
}

/// One canonical retained query whose proof actually reaches this claim's S3
/// event. Dense derivation IDs are function-local and valid only in the
/// enclosing checked program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimLedgerUse {
    pub(crate) root: state::DerivationRootKind,
    pub(crate) root_derivation: DerivationId,
    pub(crate) premise_derivations: Vec<DerivationId>,
    pub(crate) provenance: ClaimUseProvenance,
}

/// Complete observational record for one named claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimLedgerEntry {
    pub(crate) source: ClaimSourceIdentity,
    pub(crate) name: String,
    pub(crate) predicate: String,
    pub(crate) justification: String,
    pub(crate) disposition: ClaimDisposition,
    pub(crate) lifecycle_derivation: Option<DerivationId>,
    pub(crate) uses: Vec<ClaimLedgerUse>,
}

/// Deterministic checked-program claim inventory in dense function and source
/// order. It is neither serialized nor consulted by semantic acceptance.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClaimLedger {
    pub(crate) entries: Vec<ClaimLedgerEntry>,
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
    /// Exact declared-order actual count at this concrete call occurrence.
    /// This remains zero for a legal zero-argument call with a requirement.
    pub(crate) argument_count: u32,
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
    /// caller-local S12 or strict U root reaches it.
    pub(crate) derivation: Option<DerivationId>,
}

/// One bounds result retained from a non-complete ENT proof view [ENT-6].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ViewObligationOutcome {
    /// The exact protected subscript occurrence consumed by provenance, or
    /// the overflow conjunct occurrence consumed by the strict U judgment.
    pub(crate) node_path: NodePath,
    /// The obligation family this view outcome belongs to.
    pub(crate) family: ObligationFamily,
    /// Whether the selected counterfactual fact sources discharge it.
    pub(crate) discharged: bool,
    /// The ordinary canonical residual when it remains undischarged.
    pub(crate) residual: Option<String>,
    /// Exact same-view proof, retained only through a required caller-local
    /// postcondition or strict U root.
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

/// The marked program entry's pre-wrapper, pre-S4 U requirement judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramStartGoalOutcome {
    /// Exact final requirement check occurrence.
    pub(crate) final_check: NodePath,
    /// Concrete requirement over the entry parameter images.
    pub(crate) goal: ConcreteGoal,
    /// Existing U-view goal disposition.
    pub(crate) disposition: CallGoalDisposition,
    /// Deterministic complete U-view evidence.
    pub(crate) evidence: Vec<CallGoalEvidence>,
    /// Existing positive or contradiction proof on success.
    pub(crate) derivation: Option<DerivationId>,
}

/// One successful CLM-3 query rooted in the owning function's existing U
/// derivation arena. Registration precedes the sole finish/remap boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StrictEntailmentRoot {
    /// Exact protected leaf, call, or program-start final-check occurrence.
    pub(crate) node_path: NodePath,
    /// Query class sharing this occurrence namespace.
    pub(crate) kind: StrictDerivationRootKind,
    /// Owning-function derivation, remapped by the sole finish boundary.
    pub(crate) derivation: DerivationId,
}

/// One O11 signed-Boolean-decomposition candidate, recorded at a
/// complete-view signed-goal establishment point.
///
/// This is acceptance-dark candidate metadata for the v0.31 O11 boolean
/// composition item: the members and their retained projections are exactly
/// what the candidate rule would establish (`+band` and `-bor` decompose
/// into signed children, `bnot` flips, `-band`/`+bor`/`bxor` decompose on
/// no sign), but nothing establishes them as facts in this version, so
/// v0.30 acceptance is untouched. Activation establishes precisely these
/// members at the same establishment sites. Design and delta:
/// `research/investigations/o11-composition/`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BooleanGoalDecomposition {
    /// The established parent goal.
    pub(crate) parent: state::GoalId,
    /// The parent's established sign at the recording point.
    pub(crate) sign: state::GoalSign,
    /// The signed decomposition set in deterministic structural walk order.
    pub(crate) members: Vec<(state::GoalId, state::GoalSign)>,
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
    /// Present only for a marked program entry carrying a requirement.
    pub(crate) program_start: Option<ProgramStartGoalOutcome>,
    /// Exact existing-U roots demanded by successful CLM-3 validation.
    pub(crate) strict_roots: Vec<StrictEntailmentRoot>,
    /// One complete five-relation/eight-atomic S11 group per counted
    /// statement, in deterministic statement-walk order.
    pub(crate) counted_derivations: Vec<CountedDerivationSet>,
    /// Every admitted S7 relation, in structural source / C-U-B view /
    /// operand order. Each entry owns one required source root.
    pub(crate) s7_derivations: Vec<S7Derivation>,
    /// Present exactly for a concrete function carrying an FN-9 declaration.
    pub(crate) postcondition: Option<FunctionPostconditionProof>,
    /// O11 candidate decomposition sets recorded at complete-view signed-goal
    /// establishments; never an acceptance input in this version.
    pub(crate) boolean_decompositions: Vec<BooleanGoalDecomposition>,
    /// Function-local, lifetime-bound derivations for mandatory DIAG-2 roots.
    pub(crate) derivations: DerivationLedger,
    /// Canonical term and goal identities moved from the analyzer so every
    /// retained dense ID remains exact and interpretable after analysis.
    pub(crate) inventory: DerivationInventory,
}

impl FunctionEntailment {
    fn register_strict_root(
        &mut self,
        node_path: &NodePath,
        kind: StrictDerivationRootKind,
        derivation: DerivationId,
    ) -> Result<(), SemanticCompilerFailure> {
        if let Some(existing) = self
            .strict_roots
            .iter()
            .find(|root| root.node_path == *node_path && root.kind == kind)
        {
            return (existing.derivation == derivation)
                .then_some(())
                .ok_or(SemanticCompilerFailure::InvalidResolution);
        }
        let occurrence = u32::try_from(self.strict_roots.len())
            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
        self.derivations
            .add_root(DerivationRootKind::Strict { occurrence, kind }, derivation);
        self.strict_roots.push(StrictEntailmentRoot {
            node_path: node_path.clone(),
            kind,
            derivation,
        });
        Ok(())
    }

    /// Retains one already-successful protected-obligation U proof.
    ///
    /// An overflow obligation [OP-2] holds two conjunct outcomes at one
    /// `infix` node path; registration runs only after the strict closure
    /// reported no failure, so every same-path conjunct is discharged, and
    /// the retained root carries the least conjunct's proof. The other
    /// conjunct's U proof stays on its `ViewObligationOutcome`.
    pub(crate) fn register_strict_obligation(
        &mut self,
        node_path: &NodePath,
    ) -> Result<(), SemanticCompilerFailure> {
        let outcome = self
            .unasserted
            .obligations
            .iter()
            .find(|outcome| outcome.node_path == *node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !outcome.discharged {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let derivation = outcome
            .derivation
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.register_strict_root(
            node_path,
            StrictDerivationRootKind::BoundsObligation,
            derivation,
        )
    }

    /// Retains one already-successful ordinary-call requirement U proof.
    pub(crate) fn register_strict_call(
        &mut self,
        node_path: &NodePath,
    ) -> Result<(), SemanticCompilerFailure> {
        let outcome = self
            .unasserted
            .call_goals
            .iter()
            .find(|outcome| outcome.node_path == *node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !outcome.actual_obligations_ok
            || outcome.goal_disposition != CallGoalDisposition::Discharged
        {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let derivation = outcome
            .derivation
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.register_strict_root(node_path, StrictDerivationRootKind::CallGoal, derivation)
    }

    /// Retains one successful outside-caller-to-marked-root U goal. The
    /// caller's actual-expression obligations are not part of this additional
    /// boundary judgment: ordinary complete-state checking already accepted
    /// them, and CLM-3 does not demand the outside caller's component.
    pub(crate) fn register_strict_boundary_call(
        &mut self,
        node_path: &NodePath,
    ) -> Result<(), SemanticCompilerFailure> {
        let outcome = self
            .unasserted
            .call_goals
            .iter()
            .find(|outcome| outcome.node_path == *node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if outcome.goal_disposition != CallGoalDisposition::Discharged {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let derivation = outcome
            .derivation
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.register_strict_root(node_path, StrictDerivationRootKind::CallGoal, derivation)
    }

    /// Retains the marked entry's already-successful pre-S4 U proof.
    pub(crate) fn register_strict_program_start(&mut self) -> Result<(), SemanticCompilerFailure> {
        let outcome = self
            .program_start
            .as_ref()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if outcome.disposition != CallGoalDisposition::Discharged {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let node_path = outcome.final_check.clone();
        let derivation = outcome
            .derivation
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.register_strict_root(
            &node_path,
            StrictDerivationRootKind::ProgramStart,
            derivation,
        )
    }
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

/// Computes one optimistic FN-9/CLM-3 function batch without pruning its
/// shared derivation ledger. Program provenance and strict validation decide
/// whether this candidate batch is discarded or finalized unchanged.
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

/// Builds the read-only Stage 9a claim report from finalized function-local
/// derivation ledgers and the already-accepted provenance table.
///
/// This routine never closes a fact state or rewalks checked syntax. A link is
/// present exactly when the canonical retained root reaches a proof node whose
/// own structural event is the claim's S3 occurrence.
pub(crate) fn build_claim_ledger(
    functions: &[CheckedFunction],
    provenance: &super::provenance::ProvenanceMetadata,
    sources: Vec<Vec<ClaimSourceIdentity>>,
) -> Result<ClaimLedger, SemanticCompilerFailure> {
    if functions.len() != sources.len() {
        return Err(SemanticCompilerFailure::InvalidResolution);
    }
    let mut ledger = ClaimLedger::default();
    for (function, sources) in functions.iter().zip(sources) {
        if function.entailment.claims.len() != sources.len() {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let entry_start = ledger.entries.len();
        let mut claim_by_path = HashMap::with_capacity(function.entailment.claims.len());
        for (occurrence, (claim, source)) in
            function.entailment.claims.iter().zip(sources).enumerate()
        {
            if source.function != function.id
                || source.function_symbol != function.symbol
                || source.node_path != claim.node_path
                || claim_by_path
                    .insert(claim.node_path.clone(), occurrence)
                    .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
            ledger.entries.push(ClaimLedgerEntry {
                source,
                name: claim.name.clone(),
                predicate: claim.predicate.clone(),
                justification: claim.justification.clone(),
                disposition: claim.disposition.clone(),
                lifecycle_derivation: claim.lifecycle_derivation,
                uses: Vec::new(),
            });
        }
        if function.entailment.claims.is_empty() {
            continue;
        }

        for root in &function.entailment.derivations.roots {
            if matches!(root.kind, state::DerivationRootKind::ClaimLifecycle { .. }) {
                continue;
            }
            let mut premise_derivations = vec![Vec::new(); function.entailment.claims.len()];
            for (node_path, premise) in function
                .entailment
                .derivations
                .event_premises(root.node, state::FlowEventKind::S3)
            {
                let occurrence = claim_by_path
                    .get(&node_path)
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                premise_derivations[occurrence].push(premise);
            }
            for (occurrence, premises) in premise_derivations.into_iter().enumerate() {
                if premises.is_empty() {
                    continue;
                }
                let use_provenance = match root.kind {
                    state::DerivationRootKind::BoundsObligation(ordinal)
                        if function
                            .entailment
                            .obligations
                            .get(ordinal as usize)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?
                            .family
                            == ObligationFamily::Overflow =>
                    {
                        // The overflow family attaches base discharge only:
                        // no provenance judgment exists for its occurrences
                        // [OP-2, ENT-6], so a supporting claim records the
                        // use with no leaf disposition.
                        ClaimUseProvenance::NotApplicable
                    }
                    state::DerivationRootKind::BoundsObligation(ordinal) => {
                        let outcome = function
                            .entailment
                            .obligations
                            .get(ordinal as usize)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        let leaf = super::provenance::ProtectedLeaf {
                            function: function.id,
                            obligation: outcome.node_path.clone(),
                            conjunct: u32::from(outcome.conjunct),
                        };
                        let mut matches = provenance
                            .local_leaf_dispositions
                            .iter()
                            .filter(|disposition| disposition.leaf == leaf);
                        let disposition = matches
                            .next()
                            .cloned()
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        if matches.next().is_some() {
                            return Err(SemanticCompilerFailure::InvalidResolution);
                        }
                        let direct_demands = provenance
                            .direct_demands
                            .iter()
                            .filter(|demand| demand.leaf == leaf)
                            .cloned()
                            .collect::<Vec<_>>();
                        let structural_bridges = provenance
                            .structural_bridges
                            .iter()
                            .filter(|bridge| bridge.leaf == leaf)
                            .cloned()
                            .collect::<Vec<_>>();
                        let subject_bridges = provenance
                            .subject_bridges
                            .iter()
                            .filter(|bridge| bridge.leaf == leaf)
                            .cloned()
                            .collect::<Vec<_>>();
                        let calls = provenance
                            .calls
                            .iter()
                            .filter(|call| call.leaf == leaf)
                            .cloned()
                            .collect::<Vec<_>>();
                        match disposition.disposition {
                            super::provenance::LocalLeafProvenanceDisposition::DirectDemand
                                if direct_demands.is_empty() =>
                            {
                                return Err(SemanticCompilerFailure::InvalidResolution);
                            }
                            super::provenance::LocalLeafProvenanceDisposition::RequirementBridge
                                if structural_bridges.is_empty() || subject_bridges.is_empty() =>
                            {
                                return Err(SemanticCompilerFailure::InvalidResolution);
                            }
                            _ => {}
                        }
                        ClaimUseProvenance::ProtectedLeaf {
                            disposition,
                            direct_demands,
                            structural_bridges,
                            subject_bridges,
                            calls,
                        }
                    }
                    state::DerivationRootKind::CallGoal(ordinal) => {
                        let outcome = function
                            .entailment
                            .call_goals
                            .get(ordinal as usize)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        let arguments = provenance
                            .call_argument_dispositions
                            .iter()
                            .filter(|argument| {
                                argument.caller == function.id && argument.call == outcome.node_path
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        if arguments.len() != outcome.argument_count as usize
                            || arguments.iter().enumerate().any(|(ordinal, argument)| {
                                argument.argument as usize != ordinal
                                    || argument.caller != function.id
                                    || argument.call != outcome.node_path
                            })
                        {
                            return Err(SemanticCompilerFailure::InvalidResolution);
                        }
                        ClaimUseProvenance::Call {
                            arguments,
                            bridges: provenance
                                .calls
                                .iter()
                                .filter(|call| {
                                    call.caller == function.id && call.call == outcome.node_path
                                })
                                .cloned()
                                .collect(),
                        }
                    }
                    _ => ClaimUseProvenance::NotApplicable,
                };
                ledger.entries[entry_start + occurrence]
                    .uses
                    .push(ClaimLedgerUse {
                        root: root.kind,
                        root_derivation: root.node,
                        premise_derivations: premises,
                        provenance: use_provenance,
                    });
            }
        }
    }
    Ok(ledger)
}

/// Builds the one concrete ordinary-call SCC schedule shared by verified
/// FN-9 summaries and the CLM-3 strict partition. The schedule is absent when
/// the unit declares neither a postcondition nor a strict root, preserving the
/// established fast path.
/// `None` reports a broken dense [`FunctionId`] invariant.
pub(crate) fn postcondition_schedule<'function>(
    functions: impl IntoIterator<Item = &'function CheckedFunction>,
) -> Option<PostconditionSchedule> {
    let functions = functions.into_iter().collect::<Vec<_>>();
    if !functions
        .iter()
        .any(|function| function.postcondition.is_some() || function.deny_claims_marker.is_some())
    {
        return Some(PostconditionSchedule::default());
    }
    let mut graph = vec![Vec::new(); functions.len()];
    let mut calls = Vec::new();
    for (index, function) in functions.iter().enumerate() {
        if function.id.0 as usize != index {
            return None;
        }
        let start = calls.len();
        collect_statement_calls(function.id, &function.body, &mut calls);
        calls[start..].sort_by(|left, right| {
            left.node_path
                .components()
                .cmp(right.node_path.components())
                .then_with(|| left.callee.0.cmp(&right.callee.0))
        });
        graph[index].extend(calls[start..].iter().map(|call| call.callee));
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

    let function_components = component_of
        .iter()
        .map(|component| {
            u32::try_from(ordered_component_of[*component])
                .expect("concrete call component count exceeds the u32 identity space")
        })
        .collect::<Vec<_>>();
    let mut outgoing = vec![Vec::new(); components.len()];
    for (caller, callees) in graph.iter().enumerate() {
        let caller_component = function_components[caller];
        for callee in callees {
            let callee_component = function_components[*callee];
            if caller_component != callee_component {
                outgoing[caller_component as usize].push(callee_component);
            }
        }
    }
    for callees in &mut outgoing {
        callees.sort_unstable();
        callees.dedup();
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
                outgoing: outgoing[ordinal].clone(),
                summaries: Vec::new(),
            })
            .collect(),
        function_components,
        calls,
    })
}

fn collect_statement_calls(
    caller: FunctionId,
    statements: &[CheckedStatement],
    calls: &mut Vec<ConcreteCallOccurrence>,
) {
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
            | CheckedStatement::Give { value, .. } => {
                collect_expression_calls(caller, value, calls);
            }
            CheckedStatement::PropagateLet { scrutinee, .. } => {
                collect_expression_calls(caller, scrutinee, calls);
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        collect_expression_calls(caller, &target.offset, calls);
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression_calls(caller, &target.offset, calls);
                    }
                }
                collect_expression_calls(caller, value, calls);
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            }
            | CheckedStatement::ValueMatchLet {
                scrutinee, arms, ..
            } => {
                collect_expression_calls(caller, scrutinee, calls);
                for arm in arms {
                    collect_statement_calls(caller, &arm.body, calls);
                }
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                collect_statement_calls(caller, body, calls);
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                collect_expression_calls(caller, lower, calls);
                collect_expression_calls(caller, upper, calls);
                collect_statement_calls(caller, body, calls);
            }
            CheckedStatement::Break { .. } => {}
        }
    }
}

fn collect_expression_calls(
    caller: FunctionId,
    expression: &CheckedExpression,
    calls: &mut Vec<ConcreteCallOccurrence>,
) {
    if let CheckedExpression::UserCall { function, call, .. } = expression {
        calls.push(ConcreteCallOccurrence {
            caller,
            node_path: call.clone(),
            callee: *function,
        });
    }
    for child in flow::expression_children(expression) {
        collect_expression_calls(caller, child, calls);
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
