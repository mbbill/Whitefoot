//! [ENT-4] fact states and the least-fixed-point difference-bound closure.
//!
//! A state carries the *flow* of [ENT-3]: the live established facts (plus
//! facts a join admitted, since [ENT-5] closes each joined state first). The
//! closed state at a point is computed on demand as the least set containing
//! the live and implicit facts, closed under transitivity, disequality
//! strengthening, and subsumption; every derivability answer equals that
//! least-closure answer.

use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use super::super::goal::{GoalExpression, GoalOperation, GoalProjection};
use super::super::model::{BindingId, CheckedBooleanOperation, CheckedValue, IntegerType};
use super::VerifiedPostconditionSummaryRef;
use super::term::{LengthBound, TermId, TermKind, TermTable, ZERO, type_range};
use crate::{NodePath, PreludeDeclarationId};

/// One normalized source relation over interned terms.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Relation {
    /// `left - right <= bound`.
    Bound {
        left: TermId,
        right: TermId,
        bound: i128,
    },
    /// `left = right`, the bound pair in both directions.
    Equal { left: TermId, right: TermId },
    /// `left != right`, one disequality.
    Distinct { left: TermId, right: TermId },
}

/// Dense identity of one finite typed expression in a concrete function's
/// [ENT-2] goal universe. Only Bool-typed members may carry signed facts;
/// non-Bool members are retained solely as ordinary-let origin expansions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GoalId(pub(crate) u32);

/// The two exact opaque facts [ENT-2] admits for one complete goal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum GoalSign {
    Positive,
    Negative,
}

/// One literal in a fixed goal-normalization clause.
///
/// The component index preserves [ENT-6]'s normative order; `negated`
/// selects the exact L0 negation of that component for a negative-goal proof.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct GoalNormalizationLiteral {
    component: u32,
    negated: bool,
}

/// The complete fixed [ENT-4] normalization of one goal.
///
/// Clauses are a small deterministic DNF. `None` components retain an
/// operand outside L0's term vocabulary: a clause using one is unavailable,
/// while another complete clause may still prove the goal or its negation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoalNormalization {
    components: Vec<Option<Relation>>,
    positive_clauses: Vec<Vec<GoalNormalizationLiteral>>,
    negative_clauses: Vec<Vec<GoalNormalizationLiteral>>,
}

impl GoalNormalization {
    /// A fixed conjunction and its exact De Morgan negation.
    pub(crate) fn conjunction(components: Vec<Option<Relation>>) -> Self {
        let count = u32::try_from(components.len())
            .expect("goal-normalization component count exceeds u32");
        let positive_clauses = vec![
            (0..count)
                .map(|component| GoalNormalizationLiteral {
                    component,
                    negated: false,
                })
                .collect(),
        ];
        let negative_clauses = (0..count)
            .map(|component| {
                vec![GoalNormalizationLiteral {
                    component,
                    negated: true,
                }]
            })
            .collect();
        Self {
            components,
            positive_clauses,
            negative_clauses,
        }
    }

    /// Signed division/remainder: `c0 && (c1 || c2)`, with its exact
    /// negation `!c0 || (!c1 && !c2)`.
    pub(crate) fn signed_division(components: Vec<Option<Relation>>) -> Self {
        debug_assert_eq!(components.len(), 3);
        let literal = |component, negated| GoalNormalizationLiteral { component, negated };
        Self {
            components,
            positive_clauses: vec![
                vec![literal(0, false), literal(1, false)],
                vec![literal(0, false), literal(2, false)],
            ],
            negative_clauses: vec![
                vec![literal(0, true)],
                vec![literal(1, true), literal(2, true)],
            ],
        }
    }

    fn clauses(&self, sign: GoalSign) -> &[Vec<GoalNormalizationLiteral>] {
        match sign {
            GoalSign::Positive => &self.positive_clauses,
            GoalSign::Negative => &self.negative_clauses,
        }
    }

    /// Exact selected clause relations, for retained-proof verification.
    pub(crate) fn clause_relations(&self, sign: GoalSign, clause: u32) -> Option<Vec<Relation>> {
        self.clauses(sign)
            .get(usize::try_from(clause).ok()?)?
            .iter()
            .map(|literal| {
                let relation = self
                    .components
                    .get(usize::try_from(literal.component).ok()?)?
                    .clone()?;
                Some(if literal.negated {
                    relation.negated()
                } else {
                    relation
                })
            })
            .collect()
    }

    /// The unique conjunction basis for a signed goal, when that sign has
    /// exactly one complete clause. Alternative clauses are deliberately not
    /// collapsed into one claim component.
    pub(crate) fn conjunctive_relations(&self, sign: GoalSign) -> Option<Vec<Relation>> {
        (self.clauses(sign).len() == 1)
            .then(|| self.clause_relations(sign, 0))
            .flatten()
    }
}

/// Dense function-local identity of one retained ENT-4 derivation node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct DerivationId(pub(crate) u32);

/// Dense function-local identity of one proof-producing flow event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FlowEventId(pub(crate) u32);

/// The proof view whose fact state produced one derivation node. The three
/// views share the structural event stream, but their proof nodes remain
/// deliberately distinct even when the same relation has the same parents.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProofView {
    #[default]
    Complete,
    Unasserted,
    S4Blinded,
}

/// Proof-producing phase of one event in the existing ENT flow.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FlowEventKind {
    S1,
    S3,
    S4,
    S5,
    S6,
    S7,
    S9,
    S10,
    S11,
    ClaimReconstruction,
    Join,
    Snapshot,
    PostconditionEntryImageInvalidation,
    PostconditionCallConsume,
    PostconditionCallWrite,
    PostconditionReceiverWrite,
    PostconditionGive,
    PostconditionDeliveryJoin,
}

/// One deterministic flow event. A source path is retained exactly when the
/// checked tree already carries one; synthetic joins and loop edges need only
/// their dense event identity and predecessor ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlowEvent {
    pub(crate) kind: FlowEventKind,
    pub(crate) node_path: Option<NodePath>,
    /// Present only for one component-specific S3 source event.
    pub(crate) claim_component: Option<u32>,
}

/// Canonical retained identity for one dense goal ID after analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedGoal {
    pub(crate) expression: GoalExpression,
    pub(crate) projection: Option<Relation>,
    pub(crate) normalization: Option<GoalNormalization>,
}

/// The exact canonical identities referenced by retained dense term and goal
/// IDs. The analyzer moves these inventories into the checked function; it
/// does not duplicate them or invent a portable encoding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationInventory {
    pub(crate) terms: Vec<TermKind>,
    pub(crate) length_bounds: Vec<Option<LengthBound>>,
    pub(crate) goals: Vec<RetainedGoal>,
}

/// Why an implicit bound exists independently of writer facts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ImplicitBoundKind {
    Reflexive,
    Constant,
    TypeMinimum,
    TypeMaximum,
    ArrayLength,
}

/// One reaching predecessor named by a join derivation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct JoinParent {
    pub(crate) ordinal: u32,
    pub(crate) parent: DerivationId,
}

/// One caller-side image substituted for a callee postcondition formal.
/// The holder chain is retained because the checked goal image already names
/// the ultimate referent, while M must still observe any holder consumed by
/// the call transfer before S12 publishes the relation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PostconditionCallSubstitution {
    pub(crate) operand: u32,
    pub(crate) formal: u32,
    pub(crate) term: TermId,
    /// Exact checked holder chain used only by M at this call transfer. The
    /// published term's later support is reconstructed from `term` itself;
    /// an ultimate-referent image must not keep a temporary borrow holder
    /// alive after establishment.
    pub(crate) transfer_holders: Vec<BindingId>,
}

/// The closed set of proof steps emitted by the existing entailment flow.
/// Parent IDs always precede their child in the arena.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DerivationNode {
    SourceBound {
        relation: Relation,
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
    },
    SourceDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
    },
    SourceGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
    },
    /// The canonical truth sign of a Bool literal in the finite goal
    /// universe. This is an ordinary ENT-4 ground, not a source event.
    BooleanLiteral {
        goal: GoalId,
        sign: GoalSign,
    },
    ImplicitBound {
        left: TermId,
        right: TermId,
        bound: i128,
        kind: ImplicitBoundKind,
    },
    TransitiveBound {
        left: TermId,
        middle: TermId,
        right: TermId,
        bound: i128,
        first: DerivationId,
        second: DerivationId,
    },
    StrengthenedBound {
        left: TermId,
        right: TermId,
        bound: i128,
        weak: DerivationId,
        distinct: DerivationId,
    },
    SubsumedBound {
        left: TermId,
        right: TermId,
        held: i128,
        requested: i128,
        parent: DerivationId,
    },
    Equality {
        left: TermId,
        right: TermId,
        forward: DerivationId,
        reverse: DerivationId,
    },
    DisequalityFromStrictBound {
        left: TermId,
        right: TermId,
        parent: DerivationId,
    },
    GoalProjection {
        goal: GoalId,
        sign: GoalSign,
        relation: Relation,
        parent: DerivationId,
    },
    L0Contradiction {
        term: TermId,
        parent: DerivationId,
    },
    GoalContradiction {
        goal: GoalId,
        positive: DerivationId,
        negative: DerivationId,
    },
    /// One successful [ENT-6] integer-domain judgment. `parents` is either
    /// the contradiction/direct-goal proof or the fixed normalization's
    /// component proofs in ordinal order.
    IntegerDomain {
        goal: Option<GoalId>,
        parents: Vec<DerivationId>,
    },
    /// One signed goal derived from a fixed normalization clause.
    /// The retained goal inventory makes the clause and every ordered parent
    /// relation independently verifiable.
    GoalNormalization {
        goal: GoalId,
        sign: GoalSign,
        clause: u32,
        parents: Vec<DerivationId>,
    },
    /// One finite truth-table introduction for an already-interned Boolean
    /// parent (`band`, `bor`, or `bnot`).
    BooleanIntroduction {
        goal: GoalId,
        sign: GoalSign,
        parents: Vec<DerivationId>,
    },
    JoinBound {
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinContradiction {
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    MaterializedBound {
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedContradiction {
        event: FlowEventId,
        parent: DerivationId,
    },
    PostconditionExit {
        statement: NodePath,
        relation_ordinal: u32,
        relation: Relation,
        parent: DerivationId,
    },
    PostconditionAggregate {
        block: NodePath,
        relation_ordinal: u32,
        parents: Vec<DerivationId>,
    },
    /// Caller-local S12 evidence for one instantiated earlier-component
    /// summary. `a0_parents` deliberately name complete-view nodes even when
    /// this node belongs to U or B; they are retained checked references, not
    /// cross-view derivation edges. `view_parents` are the exact same-view Gv
    /// premises used only by the Uq branch.
    PostconditionCall {
        call: NodePath,
        relation: Relation,
        summary: VerifiedPostconditionSummaryRef,
        /// Instantiated ENT-2 terms and their exact caller holder support, in
        /// relation operand order.
        substitutions: Vec<PostconditionCallSubstitution>,
        transfer_events: Vec<FlowEventId>,
        a0_parents: Vec<DerivationId>,
        view_parents: Vec<DerivationId>,
    },
    PostconditionDirectResult {
        statement: NodePath,
        binding: BindingId,
        relation: Relation,
        parent: DerivationId,
    },
    PostconditionDirectMatch {
        call: NodePath,
        variant: PreludeDeclarationId,
        field: PreludeDeclarationId,
        tag: u32,
        binding: BindingId,
        relation: Relation,
        parent: DerivationId,
    },
    PostconditionDirectReceiver {
        statement: NodePath,
        binding: BindingId,
        receiver_formal: u32,
        relation: Relation,
        target_event: FlowEventId,
        parent: DerivationId,
    },
    PostconditionSelectedReceiver {
        statement: NodePath,
        payload: BindingId,
        binding: BindingId,
        relation: Relation,
        target_event: FlowEventId,
        parent: DerivationId,
    },
    /// One eligible `value_if` edge after forward carrier-to-receiver
    /// substitution and the edge's ordinary kills.
    PostconditionGive {
        statement: NodePath,
        carrier: BindingId,
        receiver: BindingId,
        relation: Relation,
        event: FlowEventId,
        parent: DerivationId,
    },
    /// The ordinary weakest-bound L0 join of every reaching delivery image.
    PostconditionDeliveryJoin {
        statement: NodePath,
        receiver: BindingId,
        relation: Relation,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
}

impl DerivationNode {
    fn for_each_parent(&self, mut visit: impl FnMut(DerivationId)) {
        match self {
            Self::TransitiveBound { first, second, .. } => {
                visit(*first);
                visit(*second);
            }
            Self::StrengthenedBound { weak, distinct, .. } => {
                visit(*weak);
                visit(*distinct);
            }
            Self::SubsumedBound { parent, .. }
            | Self::DisequalityFromStrictBound { parent, .. }
            | Self::GoalProjection { parent, .. }
            | Self::L0Contradiction { parent, .. }
            | Self::MaterializedBound { parent, .. }
            | Self::MaterializedDistinct { parent, .. }
            | Self::MaterializedGoal { parent, .. }
            | Self::MaterializedContradiction { parent, .. }
            | Self::PostconditionExit { parent, .. }
            | Self::PostconditionDirectResult { parent, .. }
            | Self::PostconditionDirectMatch { parent, .. }
            | Self::PostconditionDirectReceiver { parent, .. }
            | Self::PostconditionSelectedReceiver { parent, .. }
            | Self::PostconditionGive { parent, .. } => visit(*parent),
            Self::Equality {
                forward, reverse, ..
            } => {
                visit(*forward);
                visit(*reverse);
            }
            Self::GoalContradiction {
                positive, negative, ..
            } => {
                visit(*positive);
                visit(*negative);
            }
            Self::JoinBound { parents, .. }
            | Self::JoinDistinct { parents, .. }
            | Self::JoinGoal { parents, .. }
            | Self::JoinContradiction { parents, .. }
            | Self::PostconditionDeliveryJoin { parents, .. } => {
                for parent in parents {
                    visit(parent.parent);
                }
            }
            Self::PostconditionAggregate { parents, .. }
            | Self::IntegerDomain { parents, .. }
            | Self::GoalNormalization { parents, .. }
            | Self::BooleanIntroduction { parents, .. }
            | Self::PostconditionCall {
                view_parents: parents,
                ..
            } => {
                for parent in parents {
                    visit(*parent);
                }
            }
            Self::SourceBound { .. }
            | Self::SourceDistinct { .. }
            | Self::SourceGoal { .. }
            | Self::BooleanLiteral { .. }
            | Self::ImplicitBound { .. } => {}
        }
    }

    /// Every retained node reference, including S12's complete-view A0
    /// evidence carried beside a U/B node without laundering its proof view.
    fn for_each_retained_reference(&self, mut visit: impl FnMut(DerivationId)) {
        self.for_each_parent(&mut visit);
        if let Self::PostconditionCall { a0_parents, .. } = self {
            for parent in a0_parents {
                visit(*parent);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn parent_ids(&self) -> Vec<DerivationId> {
        let mut parents = Vec::with_capacity(self.parent_count());
        self.for_each_retained_reference(|parent| parents.push(parent));
        parents
    }

    fn parent_count(&self) -> usize {
        match self {
            Self::TransitiveBound { .. }
            | Self::StrengthenedBound { .. }
            | Self::Equality { .. }
            | Self::GoalContradiction { .. } => 2,
            Self::SubsumedBound { .. }
            | Self::DisequalityFromStrictBound { .. }
            | Self::GoalProjection { .. }
            | Self::L0Contradiction { .. }
            | Self::MaterializedBound { .. }
            | Self::MaterializedDistinct { .. }
            | Self::MaterializedGoal { .. }
            | Self::MaterializedContradiction { .. }
            | Self::PostconditionExit { .. }
            | Self::PostconditionDirectResult { .. }
            | Self::PostconditionDirectMatch { .. }
            | Self::PostconditionDirectReceiver { .. }
            | Self::PostconditionSelectedReceiver { .. }
            | Self::PostconditionGive { .. } => 1,
            Self::JoinBound { parents, .. }
            | Self::JoinDistinct { parents, .. }
            | Self::JoinGoal { parents, .. }
            | Self::JoinContradiction { parents, .. }
            | Self::PostconditionDeliveryJoin { parents, .. } => parents.len(),
            Self::PostconditionAggregate { parents, .. } => parents.len(),
            Self::IntegerDomain { parents, .. }
            | Self::GoalNormalization { parents, .. }
            | Self::BooleanIntroduction { parents, .. } => parents.len(),
            Self::PostconditionCall {
                a0_parents,
                view_parents,
                ..
            } => a0_parents.len() + view_parents.len(),
            Self::SourceBound { .. }
            | Self::SourceDistinct { .. }
            | Self::SourceGoal { .. }
            | Self::BooleanLiteral { .. }
            | Self::ImplicitBound { .. } => 0,
        }
    }

    fn maximum_parent_depth(&self, depths: &[u32]) -> Option<u32> {
        let mut maximum = None;
        self.for_each_retained_reference(|parent| {
            let depth = depths[parent.0 as usize];
            maximum = Some(maximum.map_or(depth, |current: u32| current.max(depth)));
        });
        maximum
    }

    fn rank(&self) -> u8 {
        match self {
            Self::SourceBound { .. } => 0,
            Self::SourceDistinct { .. } => 1,
            Self::SourceGoal { .. } => 2,
            Self::BooleanLiteral { .. } => 3,
            Self::ImplicitBound { .. } => 4,
            Self::TransitiveBound { .. } => 5,
            Self::StrengthenedBound { .. } => 6,
            Self::SubsumedBound { .. } => 7,
            Self::Equality { .. } => 8,
            Self::DisequalityFromStrictBound { .. } => 9,
            Self::GoalProjection { .. } => 10,
            Self::GoalNormalization { .. } => 11,
            Self::BooleanIntroduction { .. } => 12,
            Self::L0Contradiction { .. } => 13,
            Self::GoalContradiction { .. } => 14,
            Self::JoinBound { .. } => 15,
            Self::JoinDistinct { .. } => 16,
            Self::JoinGoal { .. } => 17,
            Self::JoinContradiction { .. } => 18,
            Self::MaterializedBound { .. } => 19,
            Self::MaterializedDistinct { .. } => 20,
            Self::MaterializedGoal { .. } => 21,
            Self::MaterializedContradiction { .. } => 22,
            Self::PostconditionExit { .. } => 23,
            Self::PostconditionAggregate { .. } => 24,
            Self::PostconditionCall { .. } => 25,
            Self::PostconditionDirectResult { .. } => 26,
            Self::PostconditionDirectMatch { .. } => 27,
            Self::PostconditionDirectReceiver { .. } => 28,
            Self::PostconditionSelectedReceiver { .. } => 29,
            Self::PostconditionGive { .. } => 30,
            Self::PostconditionDeliveryJoin { .. } => 31,
            Self::IntegerDomain { .. } => 32,
        }
    }
}

/// Root class counts are kept explicit so task 0056 can measure the frozen
/// corpus without parsing debug output or inventing a persistent format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationMetrics {
    pub(crate) bounds_roots: u32,
    pub(crate) opaque_goal_roots: u32,
    pub(crate) projected_goal_roots: u32,
    pub(crate) contradiction_roots: u32,
    pub(crate) claim_lifecycle_roots: u32,
    pub(crate) claim_evidence_roots: u32,
    pub(crate) unique_nodes: u32,
    pub(crate) parent_edges: u32,
    pub(crate) maximum_depth: u32,
    pub(crate) retained_bytes: usize,
}

/// Which non-retained [CLM-2] lifecycle judgment owns a derivation root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClaimLifecycleKind {
    Redundant,
    Refuted,
}

/// Which successful CLM-3 U query retains one existing derivation root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StrictDerivationRootKind {
    Obligation,
    CallGoal,
}

/// Which mandatory checked-program query owns a retained root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DerivationRootKind {
    BodyEntryContradiction,
    BoundsObligation(u32),
    IntegerDomainObligation(u32),
    CallGoal(u32),
    BitAndBound(u32),
    ShiftOneNonzero(u32),
    CountedS11 {
        occurrence: u32,
        atom: CountedRootAtom,
    },
    PostconditionExit {
        relation_ordinal: u32,
        occurrence: u32,
        view: ProofView,
    },
    PostconditionAggregate {
        relation_ordinal: u32,
        view: ProofView,
    },
    PostconditionDirectResult {
        occurrence: u32,
        view: ProofView,
    },
    PostconditionDirectMatch {
        occurrence: u32,
        view: ProofView,
    },
    PostconditionDirectReceiver {
        occurrence: u32,
        view: ProofView,
    },
    PostconditionSelectedReceiver {
        occurrence: u32,
        view: ProofView,
    },
    PostconditionGive {
        occurrence: u32,
        view: ProofView,
    },
    PostconditionDeliveryJoin {
        occurrence: u32,
        view: ProofView,
    },
    ClaimLifecycle {
        occurrence: u32,
        kind: ClaimLifecycleKind,
    },
    ClaimComponent {
        occurrence: u32,
        component: u32,
    },
    ClaimReconstruction {
        occurrence: u32,
        direct: bool,
    },
    Strict {
        occurrence: u32,
        kind: StrictDerivationRootKind,
    },
}

pub(crate) struct FinishRemap {
    pub(crate) nodes: Vec<Option<DerivationId>>,
    pub(crate) events: Vec<Option<FlowEventId>>,
}

/// The fixed eight directed atomic bounds in one normative S11 group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountedRootAtom {
    LowerCaptureToEndpoint,
    LowerEndpointToCapture,
    UpperCaptureToEndpoint,
    UpperEndpointToCapture,
    BinderToLowerCapture,
    LowerCaptureToBinder,
    LowerCaptureLeBinder,
    BinderLtUpperCapture,
}

/// One mandatory query root into the function-local arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DerivationRoot {
    pub(crate) kind: DerivationRootKind,
    pub(crate) node: DerivationId,
}

/// Private, lifetime-bound derivation storage for one concrete checked
/// function. It is intentionally neither serializable nor independently
/// verifiable; the checked program remains the only authority.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationLedger {
    pub(crate) events: Vec<FlowEvent>,
    pub(crate) nodes: Vec<DerivationNode>,
    /// Parallel to `nodes`: the explicit C/U/B identity of every retained
    /// node. Keeping the tag beside the node leaves the established proof-step
    /// representation compact while making view separation inspectable.
    pub(crate) node_views: Vec<ProofView>,
    pub(crate) roots: Vec<DerivationRoot>,
    depths: Vec<u32>,
    /// Parallel to `nodes`: whether the same-view parent ancestry reaches an
    /// S12 call relation. Parent IDs precede children, so this is computed
    /// once at interning rather than rediscovered by every kill/join query.
    postcondition_call_ancestry: Vec<bool>,
    interned: InternIndex,
    pub(crate) metrics: DerivationMetrics,
}

/// Content-addressed accelerator for [`DerivationLedger::intern_for`].
///
/// The index is derived from the ledger's own nodes rather than ledger state:
/// it holds one entry per interned node and answers exactly what a linear
/// search of `nodes` and `node_views` would. Two ledgers with equal nodes are
/// therefore equal whatever their indices hold, and a clone can leave the
/// entries behind and rebuild them the first time the copy interns anything.
/// That matters because [CLM-2] clones the whole concrete inventory once per
/// counterfactual rerun and almost never interns into the copy: on
/// `tests/programs/wfgrep.wf` copying this table was the single largest cost
/// in the check.
///
/// `stale` separates the two ways the entries can be absent. A clone sets it,
/// so the copy rebuilds before its first lookup. [`DerivationLedger::
/// finish_with_event_roots`] instead clears the index deliberately, after
/// remapping every node identity, and leaves it empty and fresh.
#[derive(Debug, Default)]
struct InternIndex {
    entries: HashMap<(ProofView, DerivationNode), DerivationId, InternHashBuilder>,
    stale: bool,
}

impl Clone for InternIndex {
    fn clone(&self) -> Self {
        Self {
            entries: HashMap::default(),
            stale: !self.entries.is_empty(),
        }
    }
}

/// Hash builder for [`InternIndex`].
///
/// The index is private to this module, is never iterated, and is rebuilt from
/// the ledger's nodes whenever it is absent, so its hash function reaches no
/// compiler output and no traversal order. The [ENT-4] closure interns one
/// candidate proof step for every accepted matrix cell, which made hashing
/// whole [`DerivationNode`] values with the default `SipHash` the largest
/// single remaining cost of checking `tests/programs/wfgrep.wf`.
#[derive(Clone, Copy, Debug, Default)]
struct InternHashBuilder;

impl std::hash::BuildHasher for InternHashBuilder {
    type Hasher = InternHasher;

    fn build_hasher(&self) -> Self::Hasher {
        InternHasher::default()
    }
}

/// Deterministic multiply-rotate word hasher, seeded by the fractional bits of
/// the golden ratio.
#[derive(Clone, Copy, Debug, Default)]
struct InternHasher {
    state: u64,
}

impl InternHasher {
    const SEED: u64 = 0x517c_c1b7_2722_0a95;

    fn mix(&mut self, word: u64) {
        self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(Self::SEED);
    }
}

impl std::hash::Hasher for InternHasher {
    fn write(&mut self, bytes: &[u8]) {
        let (words, tail) = bytes.as_chunks::<8>();
        for word in words {
            self.mix(u64::from_le_bytes(*word));
        }
        if !tail.is_empty() {
            let mut word = [0_u8; 8];
            word[..tail.len()].copy_from_slice(tail);
            self.mix(u64::from_le_bytes(word));
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.mix(u64::from(value));
    }

    fn write_u16(&mut self, value: u16) {
        self.mix(u64::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.mix(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.mix(value);
    }

    fn write_usize(&mut self, value: usize) {
        self.mix(value as u64);
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

impl PartialEq for InternIndex {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for InternIndex {}

impl DerivationLedger {
    pub(crate) fn event(
        &mut self,
        kind: FlowEventKind,
        node_path: Option<NodePath>,
    ) -> FlowEventId {
        let id = FlowEventId(
            u32::try_from(self.events.len())
                .expect("ENT flow event inventory exceeds the u32 identity space"),
        );
        self.events.push(FlowEvent {
            kind,
            node_path,
            claim_component: None,
        });
        id
    }

    pub(crate) fn claim_event(&mut self, node_path: NodePath, component: u32) -> FlowEventId {
        let id = FlowEventId(
            u32::try_from(self.events.len())
                .expect("ENT flow event inventory exceeds the u32 identity space"),
        );
        self.events.push(FlowEvent {
            kind: FlowEventKind::S3,
            node_path: Some(node_path),
            claim_component: Some(component),
        });
        id
    }

    pub(crate) fn intern_for(&mut self, view: ProofView, node: DerivationNode) -> DerivationId {
        if self.interned.stale {
            self.rebuild_intern_index();
        }
        let key = (view, node);
        if let Some(id) = self.interned.entries.get(&key).copied() {
            return id;
        }
        let (view, node) = key;
        let id = DerivationId(
            u32::try_from(self.nodes.len())
                .expect("ENT derivation inventory exceeds the u32 identity space"),
        );
        let mut parents_precede = true;
        let mut parents_share_view = true;
        node.for_each_parent(|parent| {
            parents_precede &= parent.0 < id.0;
            parents_share_view &= self.node_views[parent.0 as usize] == view;
        });
        node.for_each_retained_reference(|parent| {
            parents_precede &= parent.0 < id.0;
        });
        assert!(
            parents_precede,
            "ENT derivation parents must precede their child"
        );
        assert!(
            parents_share_view,
            "ENT derivation parents must belong to the child's proof view"
        );
        if let DerivationNode::PostconditionCall { a0_parents, .. } = &node {
            assert!(
                a0_parents
                    .iter()
                    .all(|parent| { self.node_views[parent.0 as usize] == ProofView::Complete }),
                "S12 A0 references must name complete-view proof nodes"
            );
        }
        let depth = node
            .maximum_parent_depth(&self.depths)
            .map_or(0, |maximum| maximum.saturating_add(1));
        let mut postcondition_call_ancestry =
            matches!(node, DerivationNode::PostconditionCall { .. });
        if !postcondition_call_ancestry {
            node.for_each_parent(|parent| {
                postcondition_call_ancestry |= self.postcondition_call_ancestry[parent.0 as usize];
            });
        }
        self.nodes.push(node.clone());
        self.node_views.push(view);
        self.depths.push(depth);
        self.postcondition_call_ancestry
            .push(postcondition_call_ancestry);
        self.interned.entries.insert((view, node), id);
        id
    }

    /// Restores the index a clone left behind.
    ///
    /// Replaying the ledger's own nodes in identity order reproduces the same
    /// map the interning sequence built: every node was inserted under its own
    /// identity, and a node identity repeated after an index reset kept the
    /// later entry.
    fn rebuild_intern_index(&mut self) {
        self.interned.entries.clear();
        self.interned.entries.reserve(self.nodes.len());
        for (index, node) in self.nodes.iter().enumerate() {
            let id = DerivationId(
                u32::try_from(index)
                    .expect("ENT derivation inventory exceeds the u32 identity space"),
            );
            self.interned
                .entries
                .insert((self.node_views[index], node.clone()), id);
        }
        self.interned.stale = false;
    }

    pub(crate) fn depth(&self, id: DerivationId) -> u32 {
        self.depths[id.0 as usize]
    }

    pub(crate) fn node_event(&self, id: DerivationId) -> Option<FlowEventId> {
        node_event(&self.nodes[id.0 as usize])
    }

    /// Returns the exact retained proof nodes under `root` whose own event has
    /// the requested kind. The traversal follows the already-retained proof
    /// references and performs no closure or semantic reconstruction.
    #[cfg(test)]
    pub(crate) fn event_premises(
        &self,
        root: DerivationId,
        kind: FlowEventKind,
    ) -> Vec<(NodePath, DerivationId)> {
        let mut seen = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        let mut premises = Vec::new();
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let node = &self.nodes[index];
            if let Some(event) =
                node_event(node).and_then(|event| self.events.get(event.0 as usize))
                && event.kind == kind
                && let Some(node_path) = &event.node_path
            {
                premises.push((node_path.clone(), id));
            }
            node.for_each_retained_reference(|parent| stack.push(parent));
        }
        premises.sort_by_key(|(_, id)| *id);
        premises
    }

    /// Returns the exact component-specific S3 premises below one retained
    /// terminal root. Claim evidence and counterfactual reports use the
    /// component ordinal as authority; a path-only S3 inventory is not
    /// precise enough when one occurrence contributes multiple facts.
    pub(crate) fn claim_component_premises(
        &self,
        root: DerivationId,
    ) -> Option<Vec<(NodePath, u32, DerivationId)>> {
        let mut seen = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        let mut premises = Vec::new();
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let node = &self.nodes[index];
            if let Some(event) =
                node_event(node).and_then(|event| self.events.get(event.0 as usize))
                && event.kind == FlowEventKind::S3
            {
                let (Some(node_path), Some(component)) = (&event.node_path, event.claim_component)
                else {
                    return None;
                };
                premises.push((node_path.clone(), component, id));
            }
            node.for_each_retained_reference(|parent| stack.push(parent));
        }
        premises.sort_by_key(|(_, component, id)| (*component, *id));
        Some(premises)
    }

    /// Whether one retained proof reaches the selected component-specific S3
    /// authority. `component == None` matches any component of the occurrence.
    pub(crate) fn reaches_claim_component(
        &self,
        root: DerivationId,
        node_path: &NodePath,
        component: Option<u32>,
    ) -> bool {
        let mut seen = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let node = &self.nodes[index];
            if let Some(event) = node_event(node).and_then(|id| self.events.get(id.0 as usize))
                && event.kind == FlowEventKind::S3
                && event.node_path.as_ref() == Some(node_path)
                && event.claim_component.is_some()
                && component.is_none_or(|component| event.claim_component == Some(component))
            {
                return true;
            }
            node.for_each_retained_reference(|parent| stack.push(parent));
        }
        false
    }

    /// CLM-2 consumers must not succeed through ex-falso, including a
    /// contradictory predecessor hidden below a non-contradictory join.
    pub(crate) fn is_non_explosive(&self, root: DerivationId) -> bool {
        let mut seen = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let node = &self.nodes[index];
            if matches!(
                node,
                DerivationNode::L0Contradiction { .. }
                    | DerivationNode::GoalContradiction { .. }
                    | DerivationNode::JoinContradiction { .. }
                    | DerivationNode::MaterializedContradiction { .. }
            ) {
                return false;
            }
            node.for_each_retained_reference(|parent| stack.push(parent));
        }
        true
    }

    /// Whether this proof's same-view derivation ancestry consumes an S12
    /// call relation. Complete-view A0 references are execution premises, not
    /// live value support, and therefore are deliberately not traversed.
    pub(crate) fn depends_on_postcondition_call(&self, id: DerivationId) -> bool {
        self.postcondition_call_ancestry[id.0 as usize]
    }

    /// Whether a closed L0 proof depends on one established relation rather
    /// than only on the universal implicit type/reflexive inventory. Delivery
    /// transports writer-visible facts and their closure; it does not create
    /// roots merely because every fresh integer term has implicit bounds.
    pub(crate) fn depends_on_explicit_relation(
        &self,
        id: DerivationId,
        memo: &mut HashMap<DerivationId, bool>,
    ) -> bool {
        if let Some(depends) = memo.get(&id) {
            return *depends;
        }
        let node = &self.nodes[id.0 as usize];
        let mut depends = matches!(
            node,
            DerivationNode::SourceBound { .. }
                | DerivationNode::SourceDistinct { .. }
                | DerivationNode::PostconditionCall { .. }
                | DerivationNode::PostconditionDirectResult { .. }
                | DerivationNode::PostconditionDirectMatch { .. }
                | DerivationNode::PostconditionDirectReceiver { .. }
                | DerivationNode::PostconditionSelectedReceiver { .. }
                | DerivationNode::PostconditionGive { .. }
                | DerivationNode::PostconditionDeliveryJoin { .. }
        );
        if !depends
            && !matches!(
                node,
                DerivationNode::ImplicitBound { .. } | DerivationNode::SourceGoal { .. }
            )
        {
            node.for_each_parent(|parent| {
                depends |= self.depends_on_explicit_relation(parent, memo);
            });
        }
        memo.insert(id, depends);
        depends
    }

    fn better(&self, candidate: DerivationId, current: DerivationId) -> bool {
        let candidate_depth = self.depth(candidate);
        let current_depth = self.depth(current);
        candidate_depth < current_depth
            || (candidate_depth == current_depth
                && compare_node_ties(
                    &self.nodes[candidate.0 as usize],
                    &self.nodes[current.0 as usize],
                )
                .is_lt())
    }

    fn candidate_better(&self, candidate: &DerivationNode, current: DerivationId) -> bool {
        let candidate_depth = candidate
            .maximum_parent_depth(&self.depths)
            .map_or(0, |maximum| maximum.saturating_add(1));
        let current_depth = self.depth(current);
        candidate_depth < current_depth
            || (candidate_depth == current_depth
                && compare_node_ties(candidate, &self.nodes[current.0 as usize]).is_lt())
    }

    pub(crate) fn add_root(&mut self, kind: DerivationRootKind, node: DerivationId) {
        self.roots.push(DerivationRoot { kind, node });
    }

    #[cfg(test)]
    pub(crate) fn finish(&mut self) -> Vec<Option<DerivationId>> {
        self.finish_with_event_roots(&[]).nodes
    }

    pub(crate) fn finish_with_event_roots(&mut self, event_roots: &[FlowEventId]) -> FinishRemap {
        let old_len = self.nodes.len();
        let mut keep = vec![false; old_len];
        let mut stack: Vec<DerivationId> = self.roots.iter().map(|root| root.node).collect();
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if keep[index] {
                continue;
            }
            keep[index] = true;
            self.nodes[index].for_each_retained_reference(|parent| stack.push(parent));
        }
        let mut remap = vec![None; old_len];
        let retained = keep.iter().filter(|kept| **kept).count();
        let mut nodes = Vec::with_capacity(retained);
        let mut node_views = Vec::with_capacity(retained);
        for (index, node) in self.nodes.iter().enumerate() {
            if keep[index] {
                let id = DerivationId(
                    u32::try_from(nodes.len())
                        .expect("retained ENT derivations exceed the u32 identity space"),
                );
                remap[index] = Some(id);
                nodes.push(node.clone());
                node_views.push(self.node_views[index]);
            }
        }
        for node in &mut nodes {
            remap_node(node, &remap);
        }
        for root in &mut self.roots {
            root.node = remap[root.node.0 as usize].expect("root retained");
        }
        self.nodes = nodes;
        self.node_views = node_views;
        let mut depths = Vec::with_capacity(self.nodes.len());
        let mut postcondition_call_ancestry = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let depth = node
                .maximum_parent_depth(&depths)
                .map_or(0, |maximum| maximum.saturating_add(1));
            depths.push(depth);
            let mut depends = matches!(node, DerivationNode::PostconditionCall { .. });
            if !depends {
                node.for_each_parent(|parent| {
                    depends |= postcondition_call_ancestry[parent.0 as usize];
                });
            }
            postcondition_call_ancestry.push(depends);
        }
        self.depths = depths;
        self.postcondition_call_ancestry = postcondition_call_ancestry;
        self.interned.entries.clear();
        self.interned.entries.shrink_to_fit();
        self.interned.stale = false;
        let event_remap = self.prune_events(event_roots);
        self.validate_view_integrity();
        self.metrics = self.measure();
        FinishRemap {
            nodes: remap,
            events: event_remap,
        }
    }

    fn validate_view_integrity(&self) {
        assert_eq!(self.node_views.len(), self.nodes.len());
        assert_eq!(self.postcondition_call_ancestry.len(), self.nodes.len());
        for (index, node) in self.nodes.iter().enumerate() {
            let view = self.node_views[index];
            node.for_each_parent(|parent| {
                assert!(parent.0 < index as u32);
                assert_eq!(self.node_views[parent.0 as usize], view);
            });
            if let DerivationNode::PostconditionCall { a0_parents, .. } = node {
                for parent in a0_parents {
                    assert!(parent.0 < index as u32);
                    assert_eq!(self.node_views[parent.0 as usize], ProofView::Complete);
                }
            }
        }
    }

    fn prune_events(&mut self, event_roots: &[FlowEventId]) -> Vec<Option<FlowEventId>> {
        let mut keep = vec![false; self.events.len()];
        for node in &self.nodes {
            for_each_node_event(node, |event| {
                keep[event.0 as usize] = true;
            });
        }
        for event in event_roots {
            keep[event.0 as usize] = true;
        }
        let mut remap = vec![None; self.events.len()];
        let mut events = Vec::with_capacity(keep.iter().filter(|kept| **kept).count());
        for (index, event) in self.events.iter().enumerate() {
            if keep[index] {
                let id = FlowEventId(
                    u32::try_from(events.len())
                        .expect("retained ENT events exceed the u32 identity space"),
                );
                remap[index] = Some(id);
                events.push(event.clone());
            }
        }
        for node in &mut self.nodes {
            remap_node_events(node, &remap);
        }
        self.events = events;
        remap
    }

    fn measure(&self) -> DerivationMetrics {
        let mut metrics = DerivationMetrics::default();
        for root in &self.roots {
            match root.kind {
                DerivationRootKind::ClaimLifecycle { .. } => {
                    metrics.claim_lifecycle_roots += 1;
                    continue;
                }
                DerivationRootKind::ClaimComponent { .. }
                | DerivationRootKind::ClaimReconstruction { .. } => {
                    metrics.claim_evidence_roots += 1;
                    continue;
                }
                _ => {}
            }
            match &self.nodes[root.node.0 as usize] {
                DerivationNode::SourceGoal { .. }
                | DerivationNode::JoinGoal { .. }
                | DerivationNode::MaterializedGoal { .. } => metrics.opaque_goal_roots += 1,
                DerivationNode::GoalProjection { .. } => metrics.projected_goal_roots += 1,
                DerivationNode::L0Contradiction { .. }
                | DerivationNode::GoalContradiction { .. }
                | DerivationNode::JoinContradiction { .. }
                | DerivationNode::MaterializedContradiction { .. } => {
                    metrics.contradiction_roots += 1;
                }
                _ => metrics.bounds_roots += 1,
            }
        }
        metrics.unique_nodes = u32::try_from(self.nodes.len())
            .expect("retained ENT derivations exceed the u32 metric space");
        let parent_edges: usize = self.nodes.iter().map(DerivationNode::parent_count).sum();
        metrics.parent_edges = u32::try_from(parent_edges)
            .expect("retained ENT parent edges exceed the u32 metric space");
        metrics.maximum_depth = self.depths.iter().copied().max().unwrap_or(0);
        metrics.retained_bytes = self.nodes.capacity() * size_of::<DerivationNode>()
            + self.node_views.capacity() * size_of::<ProofView>()
            + self.events.capacity() * size_of::<FlowEvent>()
            + self.roots.capacity() * size_of::<DerivationRoot>()
            + self.depths.capacity() * size_of::<u32>()
            + self.postcondition_call_ancestry.capacity() * size_of::<bool>()
            + self
                .nodes
                .iter()
                .map(|node| match node {
                    DerivationNode::JoinBound { parents, .. }
                    | DerivationNode::JoinDistinct { parents, .. }
                    | DerivationNode::JoinGoal { parents, .. }
                    | DerivationNode::JoinContradiction { parents, .. } => {
                        parents.capacity() * size_of::<JoinParent>()
                    }
                    DerivationNode::PostconditionAggregate { parents, .. } => {
                        parents.capacity() * size_of::<DerivationId>()
                    }
                    DerivationNode::IntegerDomain { parents, .. }
                    | DerivationNode::GoalNormalization { parents, .. } => {
                        parents.capacity() * size_of::<DerivationId>()
                    }
                    DerivationNode::PostconditionDeliveryJoin { parents, .. } => {
                        parents.capacity() * size_of::<JoinParent>()
                    }
                    DerivationNode::PostconditionCall {
                        substitutions,
                        transfer_events,
                        a0_parents,
                        view_parents,
                        ..
                    } => {
                        substitutions.capacity() * size_of::<PostconditionCallSubstitution>()
                            + substitutions
                                .iter()
                                .map(|substitution| {
                                    substitution.transfer_holders.capacity()
                                        * size_of::<BindingId>()
                                })
                                .sum::<usize>()
                            + transfer_events.capacity() * size_of::<FlowEventId>()
                            + a0_parents.capacity() * size_of::<DerivationId>()
                            + view_parents.capacity() * size_of::<DerivationId>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
            + self
                .nodes
                .iter()
                .filter_map(|node| match node {
                    DerivationNode::PostconditionExit { statement, .. } => Some(statement),
                    DerivationNode::PostconditionAggregate { block, .. } => Some(block),
                    DerivationNode::PostconditionCall { call, .. }
                    | DerivationNode::PostconditionDirectMatch { call, .. } => Some(call),
                    DerivationNode::PostconditionDirectResult { statement, .. }
                    | DerivationNode::PostconditionDirectReceiver { statement, .. }
                    | DerivationNode::PostconditionSelectedReceiver { statement, .. }
                    | DerivationNode::PostconditionGive { statement, .. }
                    | DerivationNode::PostconditionDeliveryJoin { statement, .. } => {
                        Some(statement)
                    }
                    _ => None,
                })
                .map(|path| path.components.capacity() * size_of::<u32>())
                .sum::<usize>()
            + self
                .events
                .iter()
                .filter_map(|event| event.node_path.as_ref())
                .map(|path| path.components.capacity() * size_of::<u32>())
                .sum::<usize>();
        metrics
    }
}

fn compare_node_ties(left: &DerivationNode, right: &DerivationNode) -> std::cmp::Ordering {
    let rank = left.rank().cmp(&right.rank());
    if !rank.is_eq() {
        return rank;
    }
    let mut index = 0;
    loop {
        match (tie_component(left, index), tie_component(right, index)) {
            (Some(left), Some(right)) => {
                let ordering = left.cmp(&right);
                if !ordering.is_eq() {
                    return ordering;
                }
            }
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
        }
        index += 1;
    }
}

fn tie_component(node: &DerivationNode, index: usize) -> Option<u32> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. } => (index == 0).then_some(event.0),
        DerivationNode::BooleanLiteral { goal, sign } => [
            goal.0,
            match sign {
                GoalSign::Positive => 0,
                GoalSign::Negative => 1,
            },
        ]
        .get(index)
        .copied(),
        DerivationNode::ImplicitBound { kind, .. } => (index == 0).then_some(match kind {
            ImplicitBoundKind::Reflexive => 0,
            ImplicitBoundKind::Constant => 1,
            ImplicitBoundKind::TypeMinimum => 2,
            ImplicitBoundKind::TypeMaximum => 3,
            ImplicitBoundKind::ArrayLength => 4,
        }),
        DerivationNode::TransitiveBound { first, second, .. } => {
            [first.0, second.0].get(index).copied()
        }
        DerivationNode::StrengthenedBound { weak, distinct, .. } => {
            [weak.0, distinct.0].get(index).copied()
        }
        DerivationNode::Equality {
            forward, reverse, ..
        } => [forward.0, reverse.0].get(index).copied(),
        DerivationNode::GoalContradiction {
            positive, negative, ..
        } => [positive.0, negative.0].get(index).copied(),
        DerivationNode::IntegerDomain { goal, parents } => {
            goal.map(|goal| goal.0).filter(|_| index == 0).or_else(|| {
                parents
                    .get(index - usize::from(goal.is_some()))
                    .map(|parent| parent.0)
            })
        }
        DerivationNode::GoalNormalization {
            goal,
            sign,
            clause,
            parents,
        } => [
            goal.0,
            match sign {
                GoalSign::Positive => 0,
                GoalSign::Negative => 1,
            },
            *clause,
        ]
        .get(index)
        .copied()
        .or_else(|| parents.get(index.checked_sub(3)?).map(|parent| parent.0)),
        DerivationNode::BooleanIntroduction {
            goal,
            sign,
            parents,
        } => [
            goal.0,
            match sign {
                GoalSign::Positive => 0,
                GoalSign::Negative => 1,
            },
        ]
        .get(index)
        .copied()
        .or_else(|| parents.get(index.checked_sub(2)?).map(|parent| parent.0)),
        DerivationNode::SubsumedBound { parent, .. }
        | DerivationNode::DisequalityFromStrictBound { parent, .. }
        | DerivationNode::GoalProjection { parent, .. }
        | DerivationNode::L0Contradiction { parent, .. } => (index == 0).then_some(parent.0),
        DerivationNode::JoinBound { event, parents, .. }
        | DerivationNode::JoinDistinct { event, parents, .. }
        | DerivationNode::JoinGoal { event, parents, .. }
        | DerivationNode::JoinContradiction { event, parents, .. } => {
            if index == 0 {
                Some(event.0)
            } else {
                let parent_index = (index - 1) / 2;
                let parent = parents.get(parent_index)?;
                if (index - 1).is_multiple_of(2) {
                    Some(parent.ordinal)
                } else {
                    Some(parent.parent.0)
                }
            }
        }
        DerivationNode::MaterializedBound { event, parent, .. }
        | DerivationNode::MaterializedDistinct { event, parent, .. }
        | DerivationNode::MaterializedGoal { event, parent, .. }
        | DerivationNode::MaterializedContradiction { event, parent, .. } => {
            [parent.0, event.0].get(index).copied()
        }
        DerivationNode::PostconditionExit { parent, .. } => (index == 0).then_some(parent.0),
        DerivationNode::PostconditionAggregate { parents, .. } => {
            parents.get(index).map(|parent| parent.0)
        }
        DerivationNode::PostconditionCall {
            summary,
            a0_parents,
            view_parents,
            transfer_events,
            ..
        } => {
            let fixed = [
                summary.summary.function.0,
                summary.summary.component,
                summary.view as u32,
            ];
            fixed
                .get(index)
                .copied()
                .or_else(|| {
                    let index = index.checked_sub(fixed.len())?;
                    a0_parents.get(index).map(|parent| parent.0)
                })
                .or_else(|| {
                    let index = index.checked_sub(fixed.len() + a0_parents.len())?;
                    view_parents.get(index).map(|parent| parent.0)
                })
                .or_else(|| {
                    let index =
                        index.checked_sub(fixed.len() + a0_parents.len() + view_parents.len())?;
                    transfer_events.get(index).map(|event| event.0)
                })
        }
        DerivationNode::PostconditionDirectResult {
            binding, parent, ..
        } => [binding.0, parent.0].get(index).copied(),
        DerivationNode::PostconditionDirectMatch {
            tag,
            binding,
            parent,
            ..
        } => [*tag, binding.0, parent.0].get(index).copied(),
        DerivationNode::PostconditionDirectReceiver {
            binding,
            receiver_formal,
            target_event,
            parent,
            ..
        } => [binding.0, *receiver_formal, target_event.0, parent.0]
            .get(index)
            .copied(),
        DerivationNode::PostconditionSelectedReceiver {
            payload,
            binding,
            target_event,
            parent,
            ..
        } => [payload.0, binding.0, target_event.0, parent.0]
            .get(index)
            .copied(),
        DerivationNode::PostconditionGive {
            carrier,
            receiver,
            event,
            parent,
            ..
        } => [carrier.0, receiver.0, event.0, parent.0]
            .get(index)
            .copied(),
        DerivationNode::PostconditionDeliveryJoin {
            receiver,
            event,
            parents,
            ..
        } => [Some(receiver.0), Some(event.0)]
            .get(index)
            .copied()
            .flatten()
            .or_else(|| {
                let index = index.checked_sub(2)?;
                let parent = parents.get(index / 2)?;
                if index.is_multiple_of(2) {
                    Some(parent.ordinal)
                } else {
                    Some(parent.parent.0)
                }
            }),
    }
}

fn node_event(node: &DerivationNode) -> Option<FlowEventId> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. }
        | DerivationNode::JoinBound { event, .. }
        | DerivationNode::JoinDistinct { event, .. }
        | DerivationNode::JoinGoal { event, .. }
        | DerivationNode::JoinContradiction { event, .. }
        | DerivationNode::MaterializedBound { event, .. }
        | DerivationNode::MaterializedDistinct { event, .. }
        | DerivationNode::MaterializedGoal { event, .. }
        | DerivationNode::MaterializedContradiction { event, .. }
        | DerivationNode::PostconditionDirectReceiver {
            target_event: event,
            ..
        }
        | DerivationNode::PostconditionSelectedReceiver {
            target_event: event,
            ..
        }
        | DerivationNode::PostconditionGive { event, .. }
        | DerivationNode::PostconditionDeliveryJoin { event, .. } => Some(*event),
        _ => None,
    }
}

fn node_event_mut(node: &mut DerivationNode) -> Option<&mut FlowEventId> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. }
        | DerivationNode::JoinBound { event, .. }
        | DerivationNode::JoinDistinct { event, .. }
        | DerivationNode::JoinGoal { event, .. }
        | DerivationNode::JoinContradiction { event, .. }
        | DerivationNode::MaterializedBound { event, .. }
        | DerivationNode::MaterializedDistinct { event, .. }
        | DerivationNode::MaterializedGoal { event, .. }
        | DerivationNode::MaterializedContradiction { event, .. }
        | DerivationNode::PostconditionDirectReceiver {
            target_event: event,
            ..
        }
        | DerivationNode::PostconditionSelectedReceiver {
            target_event: event,
            ..
        }
        | DerivationNode::PostconditionGive { event, .. }
        | DerivationNode::PostconditionDeliveryJoin { event, .. } => Some(event),
        _ => None,
    }
}

fn for_each_node_event(node: &DerivationNode, mut visit: impl FnMut(FlowEventId)) {
    if let Some(event) = node_event(node) {
        visit(event);
    }
    if let DerivationNode::PostconditionCall {
        transfer_events, ..
    } = node
    {
        for event in transfer_events {
            visit(*event);
        }
    }
}

fn remap_node_events(node: &mut DerivationNode, remap: &[Option<FlowEventId>]) {
    if let Some(event) = node_event_mut(node) {
        *event = remap[event.0 as usize].expect("retained node event retained");
    }
    if let DerivationNode::PostconditionCall {
        transfer_events, ..
    } = node
    {
        for event in transfer_events {
            *event = remap[event.0 as usize].expect("retained S12 transfer event retained");
        }
    }
}

fn remap_id(id: &mut DerivationId, remap: &[Option<DerivationId>]) {
    *id = remap[id.0 as usize].expect("retained node parent retained");
}

fn remap_node(node: &mut DerivationNode, remap: &[Option<DerivationId>]) {
    match node {
        DerivationNode::TransitiveBound { first, second, .. } => {
            remap_id(first, remap);
            remap_id(second, remap);
        }
        DerivationNode::StrengthenedBound { weak, distinct, .. } => {
            remap_id(weak, remap);
            remap_id(distinct, remap);
        }
        DerivationNode::Equality {
            forward, reverse, ..
        } => {
            remap_id(forward, remap);
            remap_id(reverse, remap);
        }
        DerivationNode::GoalContradiction {
            positive, negative, ..
        } => {
            remap_id(positive, remap);
            remap_id(negative, remap);
        }
        DerivationNode::IntegerDomain { parents, .. }
        | DerivationNode::GoalNormalization { parents, .. }
        | DerivationNode::BooleanIntroduction { parents, .. } => {
            for parent in parents {
                remap_id(parent, remap);
            }
        }
        DerivationNode::JoinBound { parents, .. }
        | DerivationNode::JoinDistinct { parents, .. }
        | DerivationNode::JoinGoal { parents, .. }
        | DerivationNode::JoinContradiction { parents, .. } => {
            for parent in parents {
                remap_id(&mut parent.parent, remap);
            }
        }
        DerivationNode::SubsumedBound { parent, .. }
        | DerivationNode::DisequalityFromStrictBound { parent, .. }
        | DerivationNode::GoalProjection { parent, .. }
        | DerivationNode::L0Contradiction { parent, .. }
        | DerivationNode::MaterializedBound { parent, .. }
        | DerivationNode::MaterializedDistinct { parent, .. }
        | DerivationNode::MaterializedGoal { parent, .. }
        | DerivationNode::MaterializedContradiction { parent, .. }
        | DerivationNode::PostconditionExit { parent, .. }
        | DerivationNode::PostconditionDirectResult { parent, .. }
        | DerivationNode::PostconditionDirectMatch { parent, .. }
        | DerivationNode::PostconditionDirectReceiver { parent, .. }
        | DerivationNode::PostconditionSelectedReceiver { parent, .. }
        | DerivationNode::PostconditionGive { parent, .. } => remap_id(parent, remap),
        DerivationNode::PostconditionDeliveryJoin { parents, .. } => {
            for parent in parents {
                remap_id(&mut parent.parent, remap);
            }
        }
        DerivationNode::PostconditionAggregate { parents, .. } => {
            for parent in parents {
                remap_id(parent, remap);
            }
        }
        DerivationNode::PostconditionCall {
            a0_parents,
            view_parents,
            ..
        } => {
            for parent in a0_parents.iter_mut().chain(view_parents) {
                remap_id(parent, remap);
            }
        }
        DerivationNode::SourceBound { .. }
        | DerivationNode::SourceDistinct { .. }
        | DerivationNode::SourceGoal { .. }
        | DerivationNode::BooleanLiteral { .. }
        | DerivationNode::ImplicitBound { .. } => {}
    }
}

/// One place read by a complete goal. `length` records ENT-5's fixed-length
/// boundary: an element write does not invalidate a `len(P)` observation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct GoalSupport {
    pub(crate) root: BindingId,
    pub(crate) projections: Vec<GoalProjection>,
    pub(crate) length: bool,
}

/// Derived data attached to one exact typed expression.
#[derive(Clone, Debug)]
struct GoalRecord {
    expression: GoalExpression,
    projection: Option<Relation>,
    normalization: Option<GoalNormalization>,
    support: Vec<GoalSupport>,
}

/// Per-function interning table for [ENT-2]'s finite goal universe.
#[derive(Default)]
pub(crate) struct GoalTable {
    ids: HashMap<GoalExpression, GoalId>,
    records: Vec<GoalRecord>,
}

impl GoalTable {
    pub(crate) fn intern(
        &mut self,
        expression: GoalExpression,
        projection: Option<Relation>,
        normalization: Option<GoalNormalization>,
        support: Vec<GoalSupport>,
    ) -> GoalId {
        if let Some(id) = self.ids.get(&expression).copied() {
            let record = &mut self.records[id.0 as usize];
            debug_assert_eq!(record.support, support);
            if record.projection.is_none() {
                record.projection = projection;
            } else {
                debug_assert_eq!(record.projection, projection);
            }
            if record.normalization.is_none() {
                record.normalization = normalization;
            } else {
                debug_assert_eq!(record.normalization, normalization);
            }
            return id;
        }
        let id = GoalId(
            u32::try_from(self.records.len())
                .expect("ENT goal inventory exceeds the u32 identity space"),
        );
        self.ids.insert(expression.clone(), id);
        self.records.push(GoalRecord {
            expression,
            projection,
            normalization,
            support,
        });
        id
    }

    pub(crate) fn expression(&self, id: GoalId) -> &GoalExpression {
        &self.records[id.0 as usize].expression
    }

    pub(crate) fn id(&self, expression: &GoalExpression) -> Option<GoalId> {
        self.ids.get(expression).copied()
    }

    pub(crate) fn projection(&self, id: GoalId) -> Option<&Relation> {
        self.records[id.0 as usize].projection.as_ref()
    }

    pub(crate) fn normalization(&self, id: GoalId) -> Option<&GoalNormalization> {
        self.records[id.0 as usize].normalization.as_ref()
    }

    pub(crate) fn support(&self, id: GoalId) -> &[GoalSupport] {
        &self.records[id.0 as usize].support
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = GoalId> + '_ {
        (0..self.records.len()).map(|index| {
            GoalId(u32::try_from(index).expect("ENT goal inventory exceeds the u32 identity space"))
        })
    }

    pub(crate) fn into_inventory(self) -> Vec<RetainedGoal> {
        self.records
            .into_iter()
            .map(|record| RetainedGoal {
                expression: record.expression,
                projection: record.projection,
                normalization: record.normalization,
            })
            .collect()
    }
}

impl Relation {
    /// [ENT-3] S1 exact negation over mathematical integers.
    pub(crate) fn negated(&self) -> Self {
        match self {
            Self::Bound { left, right, bound } => Self::Bound {
                left: *right,
                right: *left,
                bound: -bound - 1,
            },
            Self::Equal { left, right } => Self::Distinct {
                left: *left,
                right: *right,
            },
            Self::Distinct { left, right } => Self::Equal {
                left: *left,
                right: *right,
            },
        }
    }

    /// Every term occurring in the relation, for kill support tests.
    pub(crate) fn terms(&self) -> [TermId; 2] {
        match self {
            Self::Bound { left, right, .. }
            | Self::Equal { left, right }
            | Self::Distinct { left, right } => [*left, *right],
        }
    }
}

/// What one match arm's value binder gains when the scrutinee is an
/// outcome-carrying call [ENT-3] S7, S10.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutcomeRelation {
    /// S7 checked arithmetic: the binder equals the base term shifted by this
    /// constant.
    Shifted(i128),
    /// S10 absolute endpoint: `base <= binder <= upper`.
    Between { upper: TermId },
}

/// One pending arm fact: the relation the observing arm's value binder gains,
/// against a base term whose support must survive the path to the match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OutcomeFact {
    /// The variant whose arm observes it — `Ok` for checked arithmetic and
    /// for the `Result`-shaped boundary calls, `ReadBytes` for `read_at`.
    /// Every other arm establishes nothing [ENT-3].
    pub(crate) variant: &'static str,
    /// The term the binder is related to: p for S7, k for S10.
    pub(crate) base: TermId,
    pub(crate) relation: OutcomeRelation,
    pub(crate) event_kind: FlowEventKind,
}

/// One live fact state on the structural flow [ENT-3].
#[derive(Clone, Debug)]
pub(crate) struct FactState {
    view: ProofView,
    /// Term-universe size for which `bounds` and `distinct` already contain
    /// the complete ENT-4 relation closure. S11 materialization and ENT-5
    /// joins produce such states. Endpoint kills preserve closure; adding a
    /// source relation, removing a proof candidate, or interning a new term
    /// invalidates the marker.
    closed_term_count: Option<u32>,
    /// The loop rule's empty join: the contradictory all-derivable state, in
    /// which every relation is derivable and every fact is present. Z has
    /// empty support, so `Z - Z <= -1` never dies and the flag is absorbing
    /// under kills [ENT-4, ENT-5].
    pub(crate) all_derivable: bool,
    /// Exact reason the state is all-derivable. Every transition that sets
    /// the flag sets this handle at the same time.
    pub(crate) contradiction: Option<DerivationId>,
    /// Live difference bounds `left - right <= bound`, smallest bound kept.
    pub(crate) bounds: HashMap<(TermId, TermId), i128>,
    pub(crate) bound_proofs: HashMap<(TermId, TermId), DerivationId>,
    /// Every independently live proof of a directed bound. `bounds` and
    /// `bound_proofs` remain the canonical query cache; retaining the other
    /// candidates lets an S12-only holder invalidation expose an ordinary
    /// fallback instead of deleting the relation wholesale.
    bound_candidates: HashMap<(TermId, TermId), Vec<(i128, DerivationId)>>,
    /// Live disequalities, stored with ordered term pair.
    pub(crate) distinct: HashSet<(TermId, TermId)>,
    pub(crate) distinct_proofs: HashMap<(TermId, TermId), DerivationId>,
    /// Independently live disequality proofs, parallel to `bound_candidates`.
    distinct_candidates: HashMap<(TermId, TermId), Vec<DerivationId>>,
    /// [ENT-3] comparison origins (b): `own Bool` bindings whose initializer
    /// comparison is still valid on every path from initializer to here.
    pub(crate) origins: HashMap<BindingId, Relation>,
    /// [ENT-3] S7/S10 outcome origins: bindings holding the outcome of a
    /// checked-arithmetic or bounded boundary call, under the same no-kill,
    /// no-`set` path discipline the comparison origins carry.
    pub(crate) outcomes: HashMap<BindingId, OutcomeFact>,
    /// Live exact signed whole-goal facts [ENT-2..ENT-4].
    pub(crate) opaque: HashSet<(GoalId, GoalSign)>,
    pub(crate) opaque_proofs: HashMap<(GoalId, GoalSign), DerivationId>,
    /// Complete still-valid pure/total origin expansion of an ordinary let.
    /// The binding's own direct value goal is intentionally separate.
    pub(crate) goal_origins: HashMap<BindingId, GoalId>,
    /// Bool value initializers with two or more distinct live source goals.
    /// Such a receiver has no unique claim contribution expansion [CLM-2].
    pub(crate) ambiguous_goal_origins: HashSet<BindingId>,
}

impl Default for FactState {
    fn default() -> Self {
        Self::for_view(ProofView::Complete)
    }
}

impl FactState {
    pub(crate) fn for_view(view: ProofView) -> Self {
        Self {
            view,
            closed_term_count: None,
            all_derivable: false,
            contradiction: None,
            bounds: HashMap::new(),
            bound_proofs: HashMap::new(),
            bound_candidates: HashMap::new(),
            distinct: HashSet::new(),
            distinct_proofs: HashMap::new(),
            distinct_candidates: HashMap::new(),
            origins: HashMap::new(),
            outcomes: HashMap::new(),
            opaque: HashSet::new(),
            opaque_proofs: HashMap::new(),
            goal_origins: HashMap::new(),
            ambiguous_goal_origins: HashSet::new(),
        }
    }

    pub(crate) fn contradictory_for_view(view: ProofView, contradiction: DerivationId) -> Self {
        Self {
            view,
            all_derivable: true,
            contradiction: Some(contradiction),
            ..Self::for_view(view)
        }
    }

    /// Establishes one normalized source bound and returns that exact source
    /// parent even when a stronger fact was already live. S11 uses this at a
    /// true body entry so the retained root names the executed header proof
    /// point instead of whichever equivalent bound won fact-state
    /// canonicalization.
    pub(crate) fn establish_bound_with_proof(
        &mut self,
        left: TermId,
        right: TermId,
        bound: i128,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) -> DerivationId {
        let relation = Relation::Bound { left, right, bound };
        let proof = ledger.intern_for(
            self.view,
            DerivationNode::SourceBound {
                relation,
                left,
                right,
                bound,
                event,
            },
        );
        if !self.all_derivable {
            self.add_bound(left, right, bound, proof, ledger);
        }
        proof
    }

    /// Returns the live parent proving this directed bound without rerunning
    /// closure. Immediately after S11 materializes its preheader snapshot,
    /// this is either the exact materialized bound or the materialized
    /// contradiction that proves every requested relation.
    pub(crate) fn bound_parent(
        &self,
        left: TermId,
        right: TermId,
        requested: i128,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        self.bounds
            .get(&(left, right))
            .is_some_and(|held| *held <= requested)
            .then(|| self.bound_proofs[&(left, right)])
    }

    pub(crate) fn establish(
        &mut self,
        relation: &Relation,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) {
        if self.all_derivable {
            return;
        }
        match relation {
            Relation::Bound { left, right, bound } => {
                self.establish_bound_with_proof(*left, *right, *bound, ledger, event);
            }
            Relation::Equal { left, right } => {
                let forward = ledger.intern_for(
                    self.view,
                    DerivationNode::SourceBound {
                        relation: relation.clone(),
                        left: *left,
                        right: *right,
                        bound: 0,
                        event,
                    },
                );
                self.add_bound(*left, *right, 0, forward, ledger);
                let reverse = ledger.intern_for(
                    self.view,
                    DerivationNode::SourceBound {
                        relation: relation.clone(),
                        left: *right,
                        right: *left,
                        bound: 0,
                        event,
                    },
                );
                self.add_bound(*right, *left, 0, reverse, ledger);
            }
            Relation::Distinct { left, right } => {
                self.establish_distinct_with_proof(*left, *right, ledger, event);
            }
        }
    }

    /// Installs an already-validated caller-local S12 proof as an ordinary
    /// live L0 fact. The specialized route node is itself the proof of every
    /// normalized component; no second source event or proof authority is
    /// manufactured here.
    pub(crate) fn establish_from_proof(
        &mut self,
        relation: &Relation,
        proof: DerivationId,
        ledger: &DerivationLedger,
    ) {
        assert_eq!(ledger.node_views[proof.0 as usize], self.view);
        if self.all_derivable {
            return;
        }
        match relation {
            Relation::Bound { left, right, bound } => {
                self.add_bound(*left, *right, *bound, proof, ledger);
            }
            Relation::Equal { left, right } => {
                self.add_bound(*left, *right, 0, proof, ledger);
                self.add_bound(*right, *left, 0, proof, ledger);
            }
            Relation::Distinct { left, right } => {
                let pair = ordered(*left, *right);
                self.add_distinct_candidate(pair, proof, ledger);
            }
        }
    }

    /// Deterministic normalized live L0 facts and their canonical proofs.
    /// This excludes opaque goals and origin metadata.
    pub(crate) fn live_l0_relations(&self) -> Vec<(Relation, DerivationId)> {
        if self.all_derivable {
            return Vec::new();
        }
        let mut bounds = self.bounds.keys().copied().collect::<Vec<_>>();
        bounds.sort_unstable();
        let mut relations = bounds
            .into_iter()
            .map(|(left, right)| {
                (
                    Relation::Bound {
                        left,
                        right,
                        bound: self.bounds[&(left, right)],
                    },
                    self.bound_proofs[&(left, right)],
                )
            })
            .collect::<Vec<_>>();
        let mut distinct = self.distinct.iter().copied().collect::<Vec<_>>();
        distinct.sort_unstable();
        relations.extend(distinct.into_iter().map(|(left, right)| {
            (
                Relation::Distinct { left, right },
                self.distinct_proofs[&(left, right)],
            )
        }));
        relations
    }

    pub(crate) fn establish_goal(
        &mut self,
        goal: GoalId,
        sign: GoalSign,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) {
        let _ = self.establish_goal_with_proof(goal, sign, ledger, event);
    }

    pub(crate) fn establish_goal_with_proof(
        &mut self,
        goal: GoalId,
        sign: GoalSign,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) -> DerivationId {
        let fact = (goal, sign);
        let proof = ledger.intern_for(self.view, DerivationNode::SourceGoal { goal, sign, event });
        if !self.all_derivable
            && (self.opaque.insert(fact) || ledger.better(proof, self.opaque_proofs[&fact]))
        {
            self.opaque_proofs.insert(fact, proof);
        }
        proof
    }

    pub(crate) fn establish_goal_from_proof(
        &mut self,
        goal: GoalId,
        sign: GoalSign,
        parent: DerivationId,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return None;
        }
        let fact = (goal, sign);
        let proof = ledger.intern_for(
            self.view,
            DerivationNode::MaterializedGoal {
                goal,
                sign,
                event,
                parent,
            },
        );
        if self.opaque.insert(fact) || ledger.better(proof, self.opaque_proofs[&fact]) {
            self.opaque_proofs.insert(fact, proof);
        }
        Some(proof)
    }

    pub(crate) fn establish_distinct_with_proof(
        &mut self,
        left: TermId,
        right: TermId,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) -> DerivationId {
        let pair = ordered(left, right);
        let proof = ledger.intern_for(
            self.view,
            DerivationNode::SourceDistinct {
                left: pair.0,
                right: pair.1,
                event,
            },
        );
        if !self.all_derivable {
            self.add_distinct_candidate(pair, proof, ledger);
        }
        proof
    }

    pub(crate) const fn proof_view(&self) -> ProofView {
        self.view
    }

    fn selected_relations_depend_on_postcondition_call(&self, ledger: &DerivationLedger) -> bool {
        self.bound_proofs
            .values()
            .chain(self.distinct_proofs.values())
            .any(|proof| ledger.depends_on_postcondition_call(*proof))
    }

    fn add_bound(
        &mut self,
        left: TermId,
        right: TermId,
        bound: i128,
        proof: DerivationId,
        ledger: &DerivationLedger,
    ) {
        self.closed_term_count = None;
        let pair = (left, right);
        let candidates = self.bound_candidates.entry(pair).or_default();
        if !candidates.contains(&(bound, proof)) {
            candidates.push((bound, proof));
        }
        self.select_bound_candidate(pair, ledger);
    }

    fn add_distinct_candidate(
        &mut self,
        pair: (TermId, TermId),
        proof: DerivationId,
        ledger: &DerivationLedger,
    ) {
        self.closed_term_count = None;
        let candidates = self.distinct_candidates.entry(pair).or_default();
        if !candidates.contains(&proof) {
            candidates.push(proof);
        }
        self.select_distinct_candidate(pair, ledger);
    }

    fn select_bound_candidate(&mut self, pair: (TermId, TermId), ledger: &DerivationLedger) {
        let selected = self.bound_candidates.get(&pair).and_then(|candidates| {
            candidates.iter().copied().reduce(|current, candidate| {
                if candidate.0 < current.0
                    || (candidate.0 == current.0 && ledger.better(candidate.1, current.1))
                {
                    candidate
                } else {
                    current
                }
            })
        });
        if let Some((bound, proof)) = selected {
            self.bounds.insert(pair, bound);
            self.bound_proofs.insert(pair, proof);
        } else {
            self.bounds.remove(&pair);
            self.bound_proofs.remove(&pair);
            self.bound_candidates.remove(&pair);
        }
    }

    fn select_distinct_candidate(&mut self, pair: (TermId, TermId), ledger: &DerivationLedger) {
        let selected = self.distinct_candidates.get(&pair).and_then(|candidates| {
            candidates.iter().copied().reduce(|current, candidate| {
                if ledger.better(candidate, current) {
                    candidate
                } else {
                    current
                }
            })
        });
        if let Some(proof) = selected {
            self.distinct.insert(pair);
            self.distinct_proofs.insert(pair, proof);
        } else {
            self.distinct.remove(&pair);
            self.distinct_proofs.remove(&pair);
            self.distinct_candidates.remove(&pair);
        }
    }

    /// Removes every live fact and origin with a support member the kill
    /// predicate reaches. Closure never resurrects a killed fact: derived
    /// facts normally live in [`ClosedState`] views or join results, while
    /// S11 deliberately materializes its post-capture closure before kills.
    pub(crate) fn kill(&mut self, mut killed: impl FnMut(TermId) -> bool) {
        if self.all_derivable {
            return;
        }
        let dead: Vec<(TermId, TermId)> = self
            .bounds
            .keys()
            .filter(|(left, right)| killed(*left) || killed(*right))
            .copied()
            .collect();
        for pair in dead {
            self.bounds.remove(&pair);
            self.bound_proofs.remove(&pair);
            self.bound_candidates.remove(&pair);
        }
        self.distinct
            .retain(|(left, right)| !killed(*left) && !killed(*right));
        self.distinct_proofs
            .retain(|(left, right), _| !killed(*left) && !killed(*right));
        self.distinct_candidates
            .retain(|(left, right), _| !killed(*left) && !killed(*right));
        self.origins.retain(|_, relation| {
            let [left, right] = relation.terms();
            !killed(left) && !killed(right)
        });
        self.outcomes.retain(|_, outcome| !killed(outcome.base));
    }

    /// Removes only proof candidates invalidated by an S12-private holder
    /// event, then deterministically exposes the best surviving ordinary or
    /// S12 proof. Ordinary ENT-5 term kills continue to delete whole pairs.
    pub(crate) fn kill_proof_candidates(
        &mut self,
        ledger: &DerivationLedger,
        mut killed: impl FnMut(TermId, TermId, DerivationId) -> bool,
    ) -> bool {
        if self.all_derivable {
            return false;
        }
        let mut changed = false;
        let mut bound_pairs = self.bound_candidates.keys().copied().collect::<Vec<_>>();
        bound_pairs.sort_unstable();
        for pair in bound_pairs {
            let candidates = self
                .bound_candidates
                .get_mut(&pair)
                .expect("candidate key came from the same map");
            let before = candidates.len();
            candidates.retain(|(_, proof)| !killed(pair.0, pair.1, *proof));
            changed |= candidates.len() != before;
            self.select_bound_candidate(pair, ledger);
        }
        let mut distinct_pairs = self.distinct_candidates.keys().copied().collect::<Vec<_>>();
        distinct_pairs.sort_unstable();
        for pair in distinct_pairs {
            let candidates = self
                .distinct_candidates
                .get_mut(&pair)
                .expect("candidate key came from the same map");
            let before = candidates.len();
            candidates.retain(|proof| !killed(pair.0, pair.1, *proof));
            changed |= candidates.len() != before;
            self.select_distinct_candidate(pair, ledger);
        }
        if changed {
            // Candidate removal is not an endpoint projection. A surviving
            // ordinary and S12 candidate can rederive a relation whose one
            // retained materialized proof was just removed, so the next
            // query must close the surviving set again.
            self.closed_term_count = None;
        }
        changed
    }

    fn retain_non_postcondition_candidates(&mut self, ledger: &DerivationLedger) -> bool {
        self.kill_proof_candidates(ledger, |_, _, proof| {
            ledger.depends_on_postcondition_call(proof)
        })
    }

    fn merge_relation_candidates_from(&mut self, other: &Self, ledger: &DerivationLedger) {
        let mut bound_pairs = other.bound_candidates.keys().copied().collect::<Vec<_>>();
        bound_pairs.sort_unstable();
        for pair in bound_pairs {
            for (bound, proof) in &other.bound_candidates[&pair] {
                self.add_bound(pair.0, pair.1, *bound, *proof, ledger);
            }
        }
        let mut distinct_pairs = other
            .distinct_candidates
            .keys()
            .copied()
            .collect::<Vec<_>>();
        distinct_pairs.sort_unstable();
        for pair in distinct_pairs {
            for proof in &other.distinct_candidates[&pair] {
                self.add_distinct_candidate(pair, *proof, ledger);
            }
        }
    }

    /// Removes signed facts and ordinary-let origin expansions whose exact
    /// goal support is invalidated by one ENT-5 event.
    pub(crate) fn kill_goals(&mut self, mut killed: impl FnMut(GoalId) -> bool) {
        if self.all_derivable {
            return;
        }
        self.opaque.retain(|(goal, _)| !killed(*goal));
        self.opaque_proofs.retain(|(goal, _), _| !killed(*goal));
        self.goal_origins.retain(|_, goal| !killed(*goal));
    }
}

fn ordered(left: TermId, right: TermId) -> (TermId, TermId) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

/// The closed fact state at one point: the [ENT-4] least fixed point over the
/// live facts and the implicit facts of every registered term.
pub(crate) struct ClosedState {
    view: ProofView,
    all_derivable: bool,
    contradiction: Option<DerivationId>,
    bounds: HashMap<(TermId, TermId), i128>,
    bound_proofs: HashMap<(TermId, TermId), DerivationId>,
    distinct: HashSet<(TermId, TermId)>,
    distinct_proofs: HashMap<(TermId, TermId), DerivationId>,
    opaque: HashSet<(GoalId, GoalSign)>,
    opaque_proofs: HashMap<(GoalId, GoalSign), DerivationId>,
}

impl ClosedState {
    fn selected_relations_depend_on_postcondition_call(&self, ledger: &DerivationLedger) -> bool {
        self.bound_proofs
            .values()
            .chain(self.distinct_proofs.values())
            .any(|proof| ledger.depends_on_postcondition_call(*proof))
    }

    /// `left - right <= bound` is derivable.
    pub(crate) fn derives_bound(&self, left: TermId, right: TermId, bound: i128) -> bool {
        if self.all_derivable {
            return true;
        }
        self.bounds
            .get(&(left, right))
            .is_some_and(|held| *held <= bound)
    }

    /// A state is contradictory when `t - t <= -1` is derivable for any term;
    /// there every relation is derivable and every obligation is discharged.
    pub(crate) const fn contradictory(&self) -> bool {
        self.all_derivable
    }

    pub(crate) fn contradiction_proof(&self) -> Option<DerivationId> {
        self.contradiction
    }

    /// The finite normalized L0 inventory used by bounded `value_if`
    /// delivery. Opaque signed goals are deliberately absent.
    pub(crate) fn delivery_relations(&self) -> Vec<(Relation, DerivationId)> {
        if self.all_derivable {
            return Vec::new();
        }
        let mut relations = self
            .bounds
            .iter()
            .map(|(&(left, right), &bound)| {
                (
                    Relation::Bound { left, right, bound },
                    self.bound_proofs[&(left, right)],
                )
            })
            .collect::<Vec<_>>();
        relations.sort_by_key(|(relation, _)| match relation {
            Relation::Bound { left, right, bound } => (0, left.0, right.0, *bound),
            Relation::Distinct { left, right } => (1, left.0, right.0, 0),
            Relation::Equal { .. } => unreachable!("closed delivery inventory is normalized"),
        });
        let mut distinct = self.distinct.iter().copied().collect::<Vec<_>>();
        distinct.sort_unstable();
        relations.extend(distinct.into_iter().map(|(left, right)| {
            (
                Relation::Distinct { left, right },
                self.distinct_proofs[&(left, right)],
            )
        }));
        relations
    }

    /// [ENT-4] exact derivability of one normalized relation: a bound by the
    /// held smaller-or-equal constant, an equality by both zero bounds, a
    /// disequality by presence or by either strict bound.
    pub(crate) fn derives(&self, relation: &Relation) -> bool {
        if self.all_derivable {
            return true;
        }
        match relation {
            Relation::Bound { left, right, bound } => self.derives_bound(*left, *right, *bound),
            Relation::Equal { left, right } => {
                self.derives_bound(*left, *right, 0) && self.derives_bound(*right, *left, 0)
            }
            Relation::Distinct { left, right } => {
                self.distinct.contains(&ordered(*left, *right))
                    || self.derives_bound(*left, *right, -1)
                    || self.derives_bound(*right, *left, -1)
            }
        }
    }

    fn derives_normalization(&self, normalization: &GoalNormalization, sign: GoalSign) -> bool {
        normalization.clauses(sign).iter().any(|clause| {
            clause.iter().all(|literal| {
                let Some(relation) = normalization
                    .components
                    .get(usize::try_from(literal.component).expect("component index fits usize"))
                    .and_then(Option::as_ref)
                else {
                    return false;
                };
                if literal.negated {
                    self.derives(&relation.negated())
                } else {
                    self.derives(relation)
                }
            })
        })
    }

    pub(crate) fn derives_normalized_goal(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
    ) -> bool {
        goals
            .normalization(goal)
            .is_some_and(|normalization| self.derives_normalization(normalization, sign))
    }

    /// Exact signed-goal derivability: a retained opaque sign, its one
    /// comparison-root projection, its family's fixed normalization, a Bool
    /// literal, or finite truth-table introduction for an interned parent.
    pub(crate) fn derives_goal(&self, goal: GoalId, sign: GoalSign, goals: &GoalTable) -> bool {
        self.derives_goal_inner(goal, sign, goals, &mut HashSet::new())
    }

    fn derives_goal_inner(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        visiting: &mut HashSet<(GoalId, GoalSign)>,
    ) -> bool {
        if self.all_derivable || self.opaque.contains(&(goal, sign)) {
            return true;
        }
        let projected = goals.projection(goal).is_some_and(|relation| match sign {
            GoalSign::Positive => self.derives(relation),
            GoalSign::Negative => self.derives(&relation.negated()),
        });
        if projected || self.derives_normalized_goal(goal, sign, goals) {
            return true;
        }
        if literal_goal_sign(goals.expression(goal)).is_some_and(|truth| truth == sign) {
            return true;
        }
        if !visiting.insert((goal, sign)) {
            return false;
        }
        let result = match goals.expression(goal) {
            GoalExpression::Operation {
                row: GoalOperation::Boolean(operation),
                arguments,
                ..
            } => {
                let child =
                    |argument: &GoalExpression,
                     child_sign: GoalSign,
                     visiting: &mut HashSet<(GoalId, GoalSign)>| {
                        goals.id(argument).is_some_and(|child| {
                            self.derives_goal_inner(child, child_sign, goals, visiting)
                        })
                    };
                match (operation, sign) {
                    (CheckedBooleanOperation::And, GoalSign::Positive) => arguments
                        .iter()
                        .all(|argument| child(argument, GoalSign::Positive, visiting)),
                    (CheckedBooleanOperation::And, GoalSign::Negative) => arguments
                        .iter()
                        .any(|argument| child(argument, GoalSign::Negative, visiting)),
                    (CheckedBooleanOperation::Or, GoalSign::Positive) => arguments
                        .iter()
                        .any(|argument| child(argument, GoalSign::Positive, visiting)),
                    (CheckedBooleanOperation::Or, GoalSign::Negative) => arguments
                        .iter()
                        .all(|argument| child(argument, GoalSign::Negative, visiting)),
                    (CheckedBooleanOperation::Not, GoalSign::Positive) => arguments
                        .first()
                        .is_some_and(|argument| child(argument, GoalSign::Negative, visiting)),
                    (CheckedBooleanOperation::Not, GoalSign::Negative) => arguments
                        .first()
                        .is_some_and(|argument| child(argument, GoalSign::Positive, visiting)),
                    (CheckedBooleanOperation::ExclusiveOr, _) => false,
                }
            }
            GoalExpression::Datum(_) | GoalExpression::Operation { .. } => false,
        };
        visiting.remove(&(goal, sign));
        result
    }

    pub(crate) fn holds_opaque(&self, goal: GoalId, sign: GoalSign) -> bool {
        !self.all_derivable && self.opaque.contains(&(goal, sign))
    }

    pub(crate) fn opaque_proof(&self, goal: GoalId, sign: GoalSign) -> Option<DerivationId> {
        (!self.all_derivable)
            .then(|| self.opaque_proofs.get(&(goal, sign)).copied())
            .flatten()
    }

    pub(crate) fn bound_proof(
        &self,
        left: TermId,
        right: TermId,
        requested: i128,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let held = *self.bounds.get(&(left, right))?;
        if held > requested {
            return None;
        }
        let parent = self.bound_proofs[&(left, right)];
        if held == requested {
            Some(parent)
        } else {
            Some(ledger.intern_for(
                self.view,
                DerivationNode::SubsumedBound {
                    left,
                    right,
                    held,
                    requested,
                    parent,
                },
            ))
        }
    }

    pub(crate) fn relation_proof(
        &self,
        relation: &Relation,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        match relation {
            Relation::Bound { left, right, bound } => {
                self.bound_proof(*left, *right, *bound, ledger)
            }
            Relation::Equal { left, right } => {
                let forward = self.bound_proof(*left, *right, 0, ledger)?;
                let reverse = self.bound_proof(*right, *left, 0, ledger)?;
                Some(ledger.intern_for(
                    self.view,
                    DerivationNode::Equality {
                        left: *left,
                        right: *right,
                        forward,
                        reverse,
                    },
                ))
            }
            Relation::Distinct { left, right } => {
                let pair = ordered(*left, *right);
                let mut best = self.distinct_proofs.get(&pair).copied();
                for (from, to) in [(*left, *right), (*right, *left)] {
                    if let Some(parent) = self.bound_proof(from, to, -1, ledger) {
                        let candidate = ledger.intern_for(
                            self.view,
                            DerivationNode::DisequalityFromStrictBound {
                                left: pair.0,
                                right: pair.1,
                                parent,
                            },
                        );
                        if best.is_none_or(|current| ledger.better(candidate, current)) {
                            best = Some(candidate);
                        }
                    }
                }
                best
            }
        }
    }

    pub(crate) fn goal_projection_proof(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let mut relation = goals.projection(goal)?.clone();
        if sign == GoalSign::Negative {
            relation = relation.negated();
        }
        let parent = self.relation_proof(&relation, ledger)?;
        Some(ledger.intern_for(
            self.view,
            DerivationNode::GoalProjection {
                goal,
                sign,
                relation,
                parent,
            },
        ))
    }

    pub(crate) fn normalization_proof(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let normalization = goals.normalization(goal)?;
        for (clause, literals) in normalization.clauses(sign).iter().enumerate() {
            let parents = literals
                .iter()
                .map(|literal| {
                    let relation = normalization
                        .components
                        .get(usize::try_from(literal.component).ok()?)?
                        .clone()?;
                    self.relation_proof(
                        &if literal.negated {
                            relation.negated()
                        } else {
                            relation
                        },
                        ledger,
                    )
                })
                .collect::<Option<Vec<_>>>();
            if let Some(parents) = parents {
                return Some(
                    ledger.intern_for(
                        self.view,
                        DerivationNode::GoalNormalization {
                            goal,
                            sign,
                            clause: u32::try_from(clause)
                                .expect("goal-normalization clause count exceeds u32"),
                            parents,
                        },
                    ),
                );
            }
        }
        None
    }

    pub(crate) fn goal_proof(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        self.goal_proof_inner(goal, sign, goals, ledger, &mut HashSet::new())
    }

    fn goal_proof_inner(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
        visiting: &mut HashSet<(GoalId, GoalSign)>,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let direct = self
            .opaque_proof(goal, sign)
            .or_else(|| self.goal_projection_proof(goal, sign, goals, ledger))
            .or_else(|| self.normalization_proof(goal, sign, goals, ledger));
        if direct.is_some() {
            return direct;
        }
        if literal_goal_sign(goals.expression(goal)).is_some_and(|truth| truth == sign) {
            return Some(
                ledger.intern_for(self.view, DerivationNode::BooleanLiteral { goal, sign }),
            );
        }
        if !visiting.insert((goal, sign)) {
            return None;
        }
        let proof = match goals.expression(goal) {
            GoalExpression::Operation {
                row: GoalOperation::Boolean(operation),
                arguments,
                ..
            } => {
                let child_proof =
                    |argument: &GoalExpression,
                     child_sign: GoalSign,
                     visiting: &mut HashSet<(GoalId, GoalSign)>,
                     ledger: &mut DerivationLedger| {
                        let child = goals.id(argument)?;
                        self.goal_proof_inner(child, child_sign, goals, ledger, visiting)
                    };
                let all = |child_sign: GoalSign,
                           visiting: &mut HashSet<(GoalId, GoalSign)>,
                           ledger: &mut DerivationLedger| {
                    arguments
                        .iter()
                        .map(|argument| child_proof(argument, child_sign, visiting, ledger))
                        .collect::<Option<Vec<_>>>()
                };
                let any = |child_sign: GoalSign,
                           visiting: &mut HashSet<(GoalId, GoalSign)>,
                           ledger: &mut DerivationLedger| {
                    let mut best = None;
                    for argument in arguments {
                        let Some(candidate) = child_proof(argument, child_sign, visiting, ledger)
                        else {
                            continue;
                        };
                        if best.is_none_or(|current| ledger.better(candidate, current)) {
                            best = Some(candidate);
                        }
                    }
                    best.map(|parent| vec![parent])
                };
                let parents = match (operation, sign) {
                    (CheckedBooleanOperation::And, GoalSign::Positive) => {
                        all(GoalSign::Positive, visiting, ledger)
                    }
                    (CheckedBooleanOperation::And, GoalSign::Negative) => {
                        any(GoalSign::Negative, visiting, ledger)
                    }
                    (CheckedBooleanOperation::Or, GoalSign::Positive) => {
                        any(GoalSign::Positive, visiting, ledger)
                    }
                    (CheckedBooleanOperation::Or, GoalSign::Negative) => {
                        all(GoalSign::Negative, visiting, ledger)
                    }
                    (CheckedBooleanOperation::Not, GoalSign::Positive) => arguments
                        .first()
                        .and_then(|argument| {
                            child_proof(argument, GoalSign::Negative, visiting, ledger)
                        })
                        .map(|parent| vec![parent]),
                    (CheckedBooleanOperation::Not, GoalSign::Negative) => arguments
                        .first()
                        .and_then(|argument| {
                            child_proof(argument, GoalSign::Positive, visiting, ledger)
                        })
                        .map(|parent| vec![parent]),
                    (CheckedBooleanOperation::ExclusiveOr, _) => None,
                };
                parents.map(|parents| {
                    ledger.intern_for(
                        self.view,
                        DerivationNode::BooleanIntroduction {
                            goal,
                            sign,
                            parents,
                        },
                    )
                })
            }
            GoalExpression::Datum(_) | GoalExpression::Operation { .. } => None,
        };
        visiting.remove(&(goal, sign));
        proof
    }
}

fn literal_goal_sign(expression: &GoalExpression) -> Option<GoalSign> {
    let GoalExpression::Datum(super::super::goal::GoalDatum::Literal(CheckedValue::Bool(value))) =
        expression
    else {
        return None;
    };
    Some(if *value {
        GoalSign::Positive
    } else {
        GoalSign::Negative
    })
}

/// Computes the [ENT-4] closure of `state` over the registered terms.
pub(crate) fn close(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
) -> ClosedState {
    close_with_excluded_term(state, terms, goals, ledger, None)
}

/// Exact contradiction query without constructing a derivation DAG.
///
/// Kill handling needs to know whether contradiction must become absorbing,
/// but an accepted path almost always answers no.  Building every canonical
/// transitive proof merely to obtain that Boolean answer dominated CLM-2's
/// fresh whole-program reruns.  This computes the same finite ENT-4 value
/// closure in a dense matrix, including disequality strengthening and finite
/// goal introduction; callers construct the ordinary proof closure only for
/// the exceptional contradictory result.
pub(crate) fn contradiction_without_proofs(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
) -> bool {
    if state.all_derivable {
        return true;
    }
    let dimension = terms.ids().count();
    let ids = terms.ids().collect::<Vec<_>>();
    let active_middles = closure_middle_terms(state, terms, goals, &ids);
    let cells = dimension
        .checked_mul(dimension)
        .expect("ENT contradiction matrix exceeds the address space");
    let mut bounds = vec![None; cells];
    let index = |left: TermId, right: TermId| {
        let left = left.0 as usize;
        let right = right.0 as usize;
        assert!(left < dimension && right < dimension);
        left * dimension + right
    };
    let insert = |bounds: &mut [Option<i128>], left: TermId, right: TermId, bound: i128| {
        let cell = &mut bounds[index(left, right)];
        if cell.is_none_or(|current| bound < current) {
            *cell = Some(bound);
        }
    };
    for (&(left, right), &bound) in &state.bounds {
        insert(&mut bounds, left, right, bound);
    }
    for id in terms.ids() {
        insert(&mut bounds, id, id, 0);
        match terms.kind(id) {
            TermKind::Zero | TermKind::ConstParameter(_) => {}
            TermKind::Constant(value) => {
                insert(&mut bounds, id, ZERO, *value);
                insert(&mut bounds, ZERO, id, -value);
            }
            TermKind::Place(_, ty) | TermKind::ProjectedPlace(_, ty) => {
                let (minimum, maximum) = type_range(*ty);
                insert(&mut bounds, id, ZERO, maximum);
                insert(&mut bounds, ZERO, id, -minimum);
            }
            TermKind::Length(_) | TermKind::ProjectedLength(_) => {
                let (minimum, maximum) = type_range(IntegerType::U64);
                insert(&mut bounds, id, ZERO, maximum);
                insert(&mut bounds, ZERO, id, -minimum);
                match terms.length_bound(id) {
                    Some(LengthBound::Constant(length)) => {
                        insert(&mut bounds, id, ZERO, length);
                        insert(&mut bounds, ZERO, id, -length);
                    }
                    Some(LengthBound::Equal(parameter)) => {
                        insert(&mut bounds, id, parameter, 0);
                        insert(&mut bounds, parameter, id, 0);
                    }
                    None => {}
                }
            }
            TermKind::CountedCapture { .. } => {
                let (minimum, maximum) = type_range(IntegerType::U64);
                insert(&mut bounds, id, ZERO, maximum);
                insert(&mut bounds, ZERO, id, -minimum);
            }
        }
    }

    let mut distinct = state.distinct.clone();
    loop {
        // Floyd-Warshall over the exact same saturating difference bounds.
        // One pass closes the current edge set; a second is needed only when
        // a newly derived disequality strengthens a weak bound below.
        for middle in ids.iter().filter(|id| active_middles.contains(**id)) {
            let middle = middle.0 as usize;
            let middle_row = middle * dimension;
            for left in 0..dimension {
                let left_row = left * dimension;
                let Some(first) = bounds[left_row + middle] else {
                    continue;
                };
                for right in 0..dimension {
                    let Some(second) = bounds[middle_row + right] else {
                        continue;
                    };
                    let via = first.saturating_add(second);
                    let cell = &mut bounds[left_row + right];
                    if cell.is_none_or(|current| via < current) {
                        *cell = Some(via);
                    }
                }
            }
        }
        if (0..dimension).any(|id| bounds[id * dimension + id].is_some_and(|bound| bound < 0)) {
            return true;
        }

        for left in 0..dimension {
            for right in (left + 1)..dimension {
                let forward = bounds[left * dimension + right];
                let reverse = bounds[right * dimension + left];
                if forward.is_some_and(|bound| bound <= -1)
                    || reverse.is_some_and(|bound| bound <= -1)
                {
                    distinct.insert((
                        TermId(u32::try_from(left).expect("term index fits u32")),
                        TermId(u32::try_from(right).expect("term index fits u32")),
                    ));
                }
            }
        }
        let mut strengthened = false;
        for &(left, right) in &distinct {
            for (from, to) in [(left, right), (right, left)] {
                let cell = &mut bounds[index(from, to)];
                if *cell == Some(0) {
                    *cell = Some(-1);
                    strengthened = true;
                }
            }
        }
        if !strengthened {
            break;
        }
    }

    // Reuse the ordinary goal truth table over proof-free relation maps.
    // `derives_goal` consults no proof identity.
    let mut closed_bounds = HashMap::with_capacity(bounds.iter().flatten().count());
    for left in 0..dimension {
        for right in 0..dimension {
            if let Some(bound) = bounds[left * dimension + right] {
                closed_bounds.insert(
                    (
                        TermId(u32::try_from(left).expect("term index fits u32")),
                        TermId(u32::try_from(right).expect("term index fits u32")),
                    ),
                    bound,
                );
            }
        }
    }
    let closed = ClosedState {
        view: state.view,
        all_derivable: false,
        contradiction: None,
        bounds: closed_bounds,
        bound_proofs: HashMap::new(),
        distinct,
        distinct_proofs: HashMap::new(),
        opaque: state.opaque.clone(),
        opaque_proofs: HashMap::new(),
    };
    goals.ids().any(|goal| {
        closed.derives_goal(goal, GoalSign::Positive, goals)
            && closed.derives_goal(goal, GoalSign::Negative, goals)
    })
}

/// The ordinary closure with one fresh, not-yet-in-scope receiver withheld
/// from the implicit term universe. Every `value_if` edge uses this same
/// narrow boundary, including edges visited after an earlier edge interned
/// the receiver's stable identity.
pub(crate) fn close_excluding_term(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
    excluded: TermId,
) -> ClosedState {
    close_with_excluded_term(state, terms, goals, ledger, Some(excluded))
}

fn close_with_excluded_term(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
    excluded: Option<TermId>,
) -> ClosedState {
    if state.all_derivable {
        return ClosedState {
            view: state.view,
            all_derivable: true,
            contradiction: state.contradiction,
            bounds: HashMap::new(),
            bound_proofs: HashMap::new(),
            distinct: HashSet::new(),
            distinct_proofs: HashMap::new(),
            opaque: HashSet::new(),
            opaque_proofs: HashMap::new(),
        };
    }
    let term_count = terms.ids().count();
    let term_count_id =
        u32::try_from(term_count).expect("ENT term inventory exceeds the u32 identity space");
    if excluded.is_none() && state.closed_term_count == Some(term_count_id) {
        return close_goal_contradictions(
            ClosedState {
                view: state.view,
                all_derivable: false,
                contradiction: None,
                bounds: state.bounds.clone(),
                bound_proofs: state.bound_proofs.clone(),
                distinct: state.distinct.clone(),
                distinct_proofs: state.distinct_proofs.clone(),
                opaque: state.opaque.clone(),
                opaque_proofs: state.opaque_proofs.clone(),
            },
            goals,
            ledger,
        );
    }
    let mut distinct = state.distinct.clone();
    let mut distinct_proofs = state.distinct_proofs.clone();
    // The dense matrix is the only live bound index while the fixed point
    // runs; every rule reads it and nothing reads the maps. Rebuilding the
    // maps once from the settled matrix keeps their exact content while
    // dropping one hashed pair insert per accepted candidate, of which
    // `tests/programs/wfgrep.wf` accepts eighteen million.
    let mut dense_bounds =
        DenseClosureBounds::from_maps(term_count, &state.bounds, &state.bound_proofs);
    let ids = terms
        .ids()
        .filter(|id| Some(*id) != excluded)
        .collect::<Vec<_>>();
    let active_middles = closure_middle_terms(state, terms, goals, &ids);
    {
        let mut add = |left: TermId,
                       right: TermId,
                       bound: i128,
                       kind: ImplicitBoundKind,
                       ledger: &mut DerivationLedger| {
            let node = DerivationNode::ImplicitBound {
                left,
                right,
                bound,
                kind,
            };
            insert_closed_candidate(
                state.view,
                &mut dense_bounds,
                ClosedBoundCandidate {
                    left,
                    right,
                    bound,
                    node,
                },
                ledger,
            );
        };
        // Implicit facts [ENT-2]: reflexive bounds, fragment-type ranges, the
        // constant fold through Z, and array length equalities registered on the
        // length term itself by the flow.
        for id in ids.iter().copied() {
            add(id, id, 0, ImplicitBoundKind::Reflexive, ledger);
            match terms.kind(id) {
                TermKind::Zero | TermKind::ConstParameter(_) => {}
                TermKind::Constant(value) => {
                    add(id, ZERO, *value, ImplicitBoundKind::Constant, ledger);
                    add(ZERO, id, -value, ImplicitBoundKind::Constant, ledger);
                }
                TermKind::Place(_, ty) | TermKind::ProjectedPlace(_, ty) => {
                    let (minimum, maximum) = type_range(*ty);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
                }
                TermKind::Length(_) | TermKind::ProjectedLength(_) => {
                    let (minimum, maximum) = type_range(IntegerType::U64);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
                    match terms.length_bound(id) {
                        Some(LengthBound::Constant(length)) => {
                            add(id, ZERO, length, ImplicitBoundKind::ArrayLength, ledger);
                            add(ZERO, id, -length, ImplicitBoundKind::ArrayLength, ledger);
                        }
                        Some(LengthBound::Equal(parameter)) => {
                            add(id, parameter, 0, ImplicitBoundKind::ArrayLength, ledger);
                            add(parameter, id, 0, ImplicitBoundKind::ArrayLength, ledger);
                        }
                        None => {}
                    }
                }
                TermKind::CountedCapture { .. } => {
                    let (minimum, maximum) = type_range(IntegerType::U64);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
                }
            }
        }
    }
    // Least fixed point of transitivity (1), disequality strengthening (2),
    // and subsumption (3). The rules are monotone over finitely many ordered
    // pairs, so iteration terminates; strengthening only lowers a bound to
    // -1, so the outer loop runs at most a handful of rounds.
    loop {
        dense_bounds.begin_round();
        let mut changed = false;
        for middle in ids.iter().filter(|id| active_middles.contains(**id)) {
            // Preserve the former dense TermId order while skipping absent
            // matrix cells.  Transitivity cannot add a new incoming or
            // outgoing key for `middle` while processing this middle: the
            // only candidate with `left == middle` or `right == middle`
            // already requires that same key plus the reflexive diagonal.
            // Proofs for existing keys may improve, so fetch their current
            // values inside the product rather than snapshotting them here.
            let incoming = ids
                .iter()
                .copied()
                .filter(|left| dense_bounds.get(*left, *middle).is_some())
                .collect::<Vec<_>>();
            let outgoing = ids
                .iter()
                .copied()
                .filter(|right| dense_bounds.get(*middle, *right).is_some())
                .collect::<Vec<_>>();
            // Semi-naive round: a triple whose two premise cells both still
            // hold the values they held when this same traversal last offered
            // it builds the identical candidate node, and the conclusion cell
            // has only improved since, so `insert_closed_candidate` would
            // reject it without touching the ledger. Skipping such a triple
            // removes work and nothing else; the surviving triples keep their
            // former order, so the accepted sequence is unchanged. When no
            // incident cell of `middle` is fresh, none of its triples can
            // become fresh either, because the block changes nothing.
            if !incoming
                .iter()
                .any(|left| dense_bounds.fresh(*left, *middle))
                && !outgoing
                    .iter()
                    .any(|right| dense_bounds.fresh(*middle, *right))
            {
                continue;
            }
            for left in incoming {
                let (first, first_proof) = dense_bounds
                    .get(left, *middle)
                    .expect("incoming closure key collected above");
                for right in &outgoing {
                    if !dense_bounds.fresh(left, *middle) && !dense_bounds.fresh(*middle, *right) {
                        continue;
                    }
                    let (second, second_proof) = dense_bounds
                        .get(*middle, *right)
                        .expect("outgoing closure key collected above");
                    let via = first.saturating_add(second);
                    if let Some((current_bound, current_proof)) = dense_bounds.get(left, *right) {
                        if via > current_bound {
                            continue;
                        }
                        if via == current_bound {
                            let candidate_depth = ledger
                                .depth(first_proof)
                                .max(ledger.depth(second_proof))
                                .saturating_add(1);
                            if candidate_depth > ledger.depth(current_proof) {
                                continue;
                            }
                        }
                    }
                    let node = DerivationNode::TransitiveBound {
                        left,
                        middle: *middle,
                        right: *right,
                        bound: via,
                        first: first_proof,
                        second: second_proof,
                    };
                    changed |= insert_closed_candidate(
                        state.view,
                        &mut dense_bounds,
                        ClosedBoundCandidate {
                            left,
                            right: *right,
                            bound: via,
                            node,
                        },
                        ledger,
                    );
                }
            }
        }
        // ENT-4 makes every strict bound a disequality in either
        // orientation. Retain that derived fact in this same fixed point so
        // it can strengthen an available weak bound and so ENT-5 joins can
        // intersect the complete closed disequality set.
        for left in &ids {
            for right in &ids {
                if left == right
                    || dense_bounds
                        .get(*left, *right)
                        .is_none_or(|(bound, _)| bound > -1)
                {
                    continue;
                }
                let pair = ordered(*left, *right);
                let (_, parent) = dense_bounds
                    .get(*left, *right)
                    .expect("strict bound checked above");
                let node = DerivationNode::DisequalityFromStrictBound {
                    left: pair.0,
                    right: pair.1,
                    parent,
                };
                let accepted = distinct_proofs
                    .get(&pair)
                    .is_none_or(|current| ledger.candidate_better(&node, *current));
                if accepted {
                    let proof = ledger.intern_for(state.view, node);
                    distinct.insert(pair);
                    distinct_proofs.insert(pair, proof);
                    changed = true;
                }
            }
        }
        let mut distinct_pairs: Vec<_> = distinct.iter().copied().collect();
        distinct_pairs.sort_unstable();
        for (left, right) in distinct_pairs {
            for (from, to) in [(left, right), (right, left)] {
                if let Some((0, weak)) = dense_bounds.get(from, to) {
                    let node = DerivationNode::StrengthenedBound {
                        left: from,
                        right: to,
                        bound: -1,
                        weak,
                        distinct: distinct_proofs[&ordered(left, right)],
                    };
                    changed |= insert_closed_candidate(
                        state.view,
                        &mut dense_bounds,
                        ClosedBoundCandidate {
                            left: from,
                            right: to,
                            bound: -1,
                            node,
                        },
                        ledger,
                    );
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut contradiction = None;
    for id in &ids {
        if let Some((bound, parent)) = dense_bounds.get(*id, *id) {
            if bound >= 0 {
                continue;
            }
            let candidate = ledger.intern_for(
                state.view,
                DerivationNode::L0Contradiction { term: *id, parent },
            );
            if contradiction.is_none_or(|current| ledger.better(candidate, current)) {
                contradiction = Some(candidate);
            }
        }
    }
    let (bounds, bound_proofs) = dense_bounds.into_maps();
    let closed = ClosedState {
        view: state.view,
        all_derivable: contradiction.is_some(),
        contradiction,
        bounds,
        bound_proofs,
        distinct,
        distinct_proofs,
        opaque: state.opaque.clone(),
        opaque_proofs: state.opaque_proofs.clone(),
    };
    close_goal_contradictions(closed, goals, ledger)
}

fn close_goal_contradictions(
    mut closed: ClosedState,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
) -> ClosedState {
    if !closed.all_derivable {
        for goal in goals.ids() {
            if !closed.derives_goal(goal, GoalSign::Positive, goals)
                || !closed.derives_goal(goal, GoalSign::Negative, goals)
            {
                continue;
            }
            let positive = closed.goal_proof(goal, GoalSign::Positive, goals, ledger);
            let negative = closed.goal_proof(goal, GoalSign::Negative, goals, ledger);
            if let (Some(positive), Some(negative)) = (positive, negative) {
                let candidate = ledger.intern_for(
                    closed.view,
                    DerivationNode::GoalContradiction {
                        goal,
                        positive,
                        negative,
                    },
                );
                if closed
                    .contradiction
                    .is_none_or(|current| ledger.better(candidate, current))
                {
                    closed.contradiction = Some(candidate);
                }
            }
        }
        closed.all_derivable = closed.contradiction.is_some();
    }
    closed
}

/// Terms that can improve a transitive path beyond the direct implicit path
/// through zero.
///
/// Every term remains an endpoint in the closed matrix. A term carrying only
/// its reflexive and integer-range edges cannot be a useful middle: entering
/// and leaving it adds a nonnegative range cycle to the path already available
/// through `ZERO` (and an equal path has greater proof depth). Live relation
/// endpoints can be useful, as can an exact array-length alias connected to
/// one, so those are the only additional middle vertices the fixed point
/// needs. Opaque goals are included because their opposite sign can be proved
/// from a projected or normalized relation and form a contradiction.
fn closure_middle_terms(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ids: &[TermId],
) -> ActiveMiddles {
    // `TermId` is the dense function-local term identity, so membership is a
    // direct index rather than a hashed probe. The old sets answered one probe
    // per live relation endpoint of every closure.
    let width = terms.ids().count();
    let mut available = vec![false; width];
    for id in ids {
        available[id.0 as usize] = true;
    }
    let mut active = ActiveMiddles(vec![false; width]);
    let admit = |term: TermId, active: &mut ActiveMiddles| {
        if available[term.0 as usize] {
            active.0[term.0 as usize] = true;
        }
    };
    admit(ZERO, &mut active);
    for &(left, right) in state.bounds.keys() {
        admit(left, &mut active);
        admit(right, &mut active);
    }
    for &(left, right) in &state.distinct {
        admit(left, &mut active);
        admit(right, &mut active);
    }

    let mut pending_goals = state
        .opaque
        .iter()
        .map(|(goal, _)| *goal)
        .collect::<Vec<_>>();
    let mut visited_goals = HashSet::new();
    while let Some(goal) = pending_goals.pop() {
        if !visited_goals.insert(goal) {
            continue;
        }
        if let Some(relation) = goals.projection(goal) {
            for term in relation.terms() {
                admit(term, &mut active);
            }
        }
        if let Some(normalization) = goals.normalization(goal) {
            for relation in normalization.components.iter().flatten() {
                for term in relation.terms() {
                    admit(term, &mut active);
                }
            }
        }
        if let GoalExpression::Operation { arguments, .. } = goals.expression(goal) {
            pending_goals.extend(arguments.iter().filter_map(|argument| goals.id(argument)));
        }
    }

    // Array/slice length aliases are the only implicit non-zero-to-non-zero
    // edges. Every such edge needs both endpoints as middles even without a
    // live source: the length range transfers to an otherwise unbounded const
    // parameter, and two lengths equal to that parameter become equal to one
    // another. `available` keeps an excluded receiver out of this universe.
    for id in ids {
        let Some(LengthBound::Equal(parameter)) = terms.length_bound(*id) else {
            continue;
        };
        active.0[id.0 as usize] = true;
        admit(parameter, &mut active);
    }
    active
}

/// Dense membership of the terms the [ENT-4] fixed point uses as middles.
struct ActiveMiddles(Vec<bool>);

impl ActiveMiddles {
    fn contains(&self, term: TermId) -> bool {
        self.0[term.0 as usize]
    }
}

struct ClosedBoundCandidate {
    left: TermId,
    right: TermId,
    bound: i128,
    node: DerivationNode,
}

/// Dense scratch index for ENT-4's fixed point.
///
/// `TermId` is a dense function-local identity.  The closed result remains in
/// the long-lived maps above, but using tuple-key hash tables for every probe
/// in the transitivity cube made each fresh CLM-2 counterfactual spend most of
/// its time hashing the same two integers.  This index preserves the existing
/// TermId traversal and proof-selection order; it is updated only alongside
/// the authoritative maps and is discarded when this one closure finishes.
struct DenseClosureBounds {
    dimension: usize,
    cells: Vec<Option<ClosedCell>>,
    live: usize,
    round: u32,
}

/// The closed state's live bound and proof maps, in that order.
type ClosedBoundMaps = (
    HashMap<(TermId, TermId), i128>,
    HashMap<(TermId, TermId), DerivationId>,
);

/// One live cell of the closure matrix. Bound, proof and freshness stamp sit
/// together because the transitivity cube reads all three of a cell at once.
#[derive(Clone, Copy)]
struct ClosedCell {
    bound: i128,
    proof: DerivationId,
    /// Fixed-point round in which the cell last changed. Zero covers both a
    /// cell carried in from the state and one an implicit fact established
    /// before the first round, which is what makes round 1 complete.
    changed_in: u32,
}

impl DenseClosureBounds {
    fn from_maps(
        dimension: usize,
        bounds: &HashMap<(TermId, TermId), i128>,
        proofs: &HashMap<(TermId, TermId), DerivationId>,
    ) -> Self {
        let count = dimension
            .checked_mul(dimension)
            .expect("ENT closure matrix exceeds the address space");
        let mut dense = Self {
            dimension,
            cells: vec![None; count],
            live: 0,
            round: 0,
        };
        for (&(left, right), &bound) in bounds {
            let proof = proofs
                .get(&(left, right))
                .copied()
                .expect("every live ENT bound has a proof");
            dense.set(left, right, bound, proof);
        }
        dense
    }

    fn begin_round(&mut self) {
        self.round += 1;
    }

    /// Whether this cell changed during the previous fixed-point round or
    /// later. Every cell reports fresh in round 1, so the first pass over the
    /// transitivity cube is the complete one.
    fn fresh(&self, left: TermId, right: TermId) -> bool {
        self.cells[self.index(left, right)].is_none_or(|cell| cell.changed_in + 1 >= self.round)
    }

    fn index(&self, left: TermId, right: TermId) -> usize {
        let left = left.0 as usize;
        let right = right.0 as usize;
        assert!(left < self.dimension && right < self.dimension);
        left * self.dimension + right
    }

    fn get(&self, left: TermId, right: TermId) -> Option<(i128, DerivationId)> {
        self.cells[self.index(left, right)].map(|cell| (cell.bound, cell.proof))
    }

    fn set(&mut self, left: TermId, right: TermId, bound: i128, proof: DerivationId) {
        let index = self.index(left, right);
        if self.cells[index].is_none() {
            self.live += 1;
        }
        self.cells[index] = Some(ClosedCell {
            bound,
            proof,
            changed_in: self.round,
        });
    }

    /// The settled matrix as the closed state's live bound and proof maps.
    ///
    /// Every pair the state carried in reached the matrix through
    /// [`Self::from_maps`] and nothing removes a cell, so this returns exactly
    /// the incoming pairs together with the ones the fixed point derived.
    fn into_maps(self) -> ClosedBoundMaps {
        let mut bounds = HashMap::with_capacity(self.live);
        let mut proofs = HashMap::with_capacity(self.live);
        for (index, cell) in self.cells.iter().enumerate() {
            let Some(cell) = cell else {
                continue;
            };
            let left = TermId(
                u32::try_from(index / self.dimension).expect("term index fits the u32 identity"),
            );
            let right = TermId(
                u32::try_from(index % self.dimension).expect("term index fits the u32 identity"),
            );
            bounds.insert((left, right), cell.bound);
            proofs.insert((left, right), cell.proof);
        }
        (bounds, proofs)
    }
}

fn insert_closed_candidate(
    view: ProofView,
    dense: &mut DenseClosureBounds,
    candidate: ClosedBoundCandidate,
    ledger: &mut DerivationLedger,
) -> bool {
    let ClosedBoundCandidate {
        left,
        right,
        bound,
        node,
    } = candidate;
    let accepted = match dense.get(left, right) {
        None => true,
        Some((current, _)) if bound < current => true,
        Some((current, proof)) if bound == current => ledger.candidate_better(&node, proof),
        Some(_) => false,
    };
    if !accepted {
        return false;
    }
    let proof = ledger.intern_for(view, node);
    dense.set(left, right, bound, proof);
    true
}

/// Materializes the [ENT-4] least closure as a live flow state.
///
/// Ordinary queries can keep closure as an ephemeral view. S11 instead fixes
/// the complete post-capture closure *before* the counted loop's continuing
/// kill subtraction, so consequences whose support no longer includes a
/// mutable endpoint source must become independently live facts first.
/// Materializes one view at a structural snapshot event shared by all views.
pub(crate) fn materialize_closure_at(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
    event: FlowEventId,
) -> FactState {
    let closed = close(state, terms, goals, ledger);
    if closed.all_derivable {
        let parent = closed.contradiction.expect("contradictory closure proof");
        let proof = ledger.intern_for(
            state.view,
            DerivationNode::MaterializedContradiction { event, parent },
        );
        return FactState {
            view: state.view,
            all_derivable: true,
            contradiction: Some(proof),
            ..FactState::for_view(state.view)
        };
    }
    let needs_ordinary_fallback = closed.selected_relations_depend_on_postcondition_call(ledger);
    let mut bound_proofs = HashMap::new();
    let mut bound_keys: Vec<_> = closed.bounds.keys().copied().collect();
    bound_keys.sort_unstable();
    for (left, right) in bound_keys {
        let bound = closed.bounds[&(left, right)];
        let proof = ledger.intern_for(
            state.view,
            DerivationNode::MaterializedBound {
                left,
                right,
                bound,
                event,
                parent: closed.bound_proofs[&(left, right)],
            },
        );
        bound_proofs.insert((left, right), proof);
    }
    let mut distinct_proofs = HashMap::new();
    let mut distinct_keys: Vec<_> = closed.distinct.iter().copied().collect();
    distinct_keys.sort_unstable();
    for (left, right) in distinct_keys {
        let proof = ledger.intern_for(
            state.view,
            DerivationNode::MaterializedDistinct {
                left,
                right,
                event,
                parent: closed.distinct_proofs[&(left, right)],
            },
        );
        distinct_proofs.insert((left, right), proof);
    }
    let mut opaque_proofs = HashMap::new();
    let mut opaque_keys: Vec<_> = closed.opaque.iter().copied().collect();
    opaque_keys.sort_unstable();
    for (goal, sign) in opaque_keys {
        let proof = ledger.intern_for(
            state.view,
            DerivationNode::MaterializedGoal {
                goal,
                sign,
                event,
                parent: closed.opaque_proofs[&(goal, sign)],
            },
        );
        opaque_proofs.insert((goal, sign), proof);
    }
    let bound_candidates = bound_proofs
        .iter()
        .map(|(pair, proof)| (*pair, vec![(closed.bounds[pair], *proof)]))
        .collect();
    let distinct_candidates = distinct_proofs
        .iter()
        .map(|(pair, proof)| (*pair, vec![*proof]))
        .collect();
    let mut materialized = FactState {
        view: state.view,
        closed_term_count: Some(
            u32::try_from(terms.ids().count())
                .expect("ENT term inventory exceeds the u32 identity space"),
        ),
        all_derivable: false,
        contradiction: None,
        bounds: closed.bounds,
        bound_proofs,
        bound_candidates,
        distinct: closed.distinct,
        distinct_proofs,
        distinct_candidates,
        origins: state.origins.clone(),
        outcomes: state.outcomes.clone(),
        opaque: closed.opaque,
        opaque_proofs,
        goal_origins: state.goal_origins.clone(),
        ambiguous_goal_origins: state.ambiguous_goal_origins.clone(),
    };

    if !needs_ordinary_fallback {
        return materialized;
    }

    // Preserve the ordinary fallback when the canonical closure happened to
    // select an S12 proof. The second closure is not another flow walk: it is
    // the same snapshot over the same live state with S12 candidates removed,
    // and its proof candidates share the one structural snapshot event.
    let mut ordinary = state.clone();
    ordinary.retain_non_postcondition_candidates(ledger);
    let ordinary_closed = close(&ordinary, terms, goals, ledger);
    if !ordinary_closed.all_derivable {
        let mut keys = ordinary_closed.bounds.keys().copied().collect::<Vec<_>>();
        keys.sort_unstable();
        for (left, right) in keys {
            let bound = ordinary_closed.bounds[&(left, right)];
            let proof = ledger.intern_for(
                state.view,
                DerivationNode::MaterializedBound {
                    left,
                    right,
                    bound,
                    event,
                    parent: ordinary_closed.bound_proofs[&(left, right)],
                },
            );
            materialized.add_bound(left, right, bound, proof, ledger);
        }
        let mut keys = ordinary_closed.distinct.iter().copied().collect::<Vec<_>>();
        keys.sort_unstable();
        for (left, right) in keys {
            let proof = ledger.intern_for(
                state.view,
                DerivationNode::MaterializedDistinct {
                    left,
                    right,
                    event,
                    parent: ordinary_closed.distinct_proofs[&(left, right)],
                },
            );
            materialized.add_distinct_candidate((left, right), proof, ledger);
        }
    }
    materialized.closed_term_count = Some(
        u32::try_from(terms.ids().count())
            .expect("ENT term inventory exceeds the u32 identity space"),
    );
    materialized
}

/// [ENT-5] join of arm-exit states, each already taken after its scope-exit
/// kills. Each input is closed first; the join keeps, per ordered term pair,
/// the weakest bound held by all, and each disequality held by all. The empty
/// join is the contradictory all-derivable state.
#[cfg(test)]
pub(crate) fn join(
    states: &[FactState],
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
) -> FactState {
    let event = ledger.event(FlowEventKind::Join, None);
    let view = states
        .first()
        .map_or(ProofView::Complete, |state| state.view);
    join_at(states, terms, goals, ledger, view, event)
}

/// Joins one proof view at a structural event shared by all three views.
pub(crate) fn join_at(
    states: &[FactState],
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
    view: ProofView,
    event: FlowEventId,
) -> FactState {
    let mut joined = join_at_once(states, terms, goals, ledger, view, event);
    if joined.all_derivable {
        return joined;
    }
    if !joined.selected_relations_depend_on_postcondition_call(ledger) {
        return joined;
    }
    let mut ordinary_states = states.to_vec();
    let mut removed_postcondition_candidate = false;
    for state in &mut ordinary_states {
        removed_postcondition_candidate |= state.retain_non_postcondition_candidates(ledger);
    }
    if !removed_postcondition_candidate {
        return joined;
    }
    let ordinary = join_at_once(&ordinary_states, terms, goals, ledger, view, event);
    if !ordinary.all_derivable {
        joined.merge_relation_candidates_from(&ordinary, ledger);
    }
    joined.closed_term_count = Some(
        u32::try_from(terms.ids().count())
            .expect("ENT term inventory exceeds the u32 identity space"),
    );
    joined
}

fn join_at_once(
    states: &[FactState],
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
    view: ProofView,
    event: FlowEventId,
) -> FactState {
    assert!(
        states.iter().all(|state| state.view == view),
        "ENT join inputs must belong to the selected proof view"
    );
    // Close before filtering: a contradiction established immediately before
    // an edge is already the absorbing all-derivable state even when no kill
    // had occasion to materialize its flag.
    let closed: Vec<ClosedState> = states
        .iter()
        .map(|state| close(state, terms, goals, ledger))
        .collect();
    let contributing: Vec<usize> = closed
        .iter()
        .enumerate()
        .filter_map(|(index, state)| (!state.contradictory()).then_some(index))
        .collect();
    let Some((&first_index, rest_indices)) = contributing.split_first() else {
        let parents = closed
            .iter()
            .enumerate()
            .map(|(ordinal, state)| JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent: state
                    .contradiction
                    .expect("every noncontributing join edge is contradictory"),
            })
            .collect();
        let proof = ledger.intern_for(view, DerivationNode::JoinContradiction { event, parents });
        return FactState {
            view,
            all_derivable: true,
            contradiction: Some(proof),
            ..FactState::for_view(view)
        };
    };
    let first = &closed[first_index];
    let mut bounds = HashMap::new();
    let mut bound_proofs = HashMap::new();
    let mut first_bound_keys: Vec<_> = first.bounds.keys().copied().collect();
    first_bound_keys.sort_unstable();
    for pair in first_bound_keys {
        let bound = first.bounds[&pair];
        let mut weakest = bound;
        let held = rest_indices.iter().all(|index| {
            closed[*index].bounds.get(&pair).is_some_and(|other| {
                if *other > weakest {
                    weakest = *other;
                }
                true
            })
        });
        if held {
            let mut parents = Vec::with_capacity(states.len());
            for (ordinal, state) in closed.iter().enumerate() {
                let parent = if state.contradictory() {
                    state
                        .contradiction
                        .expect("contradictory predecessor proof")
                } else {
                    state
                        .bound_proof(pair.0, pair.1, weakest, ledger)
                        .expect("contributing predecessor proves joined bound")
                };
                parents.push(JoinParent {
                    ordinal: u32::try_from(ordinal)
                        .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                    parent,
                });
            }
            let proof = ledger.intern_for(
                view,
                DerivationNode::JoinBound {
                    left: pair.0,
                    right: pair.1,
                    bound: weakest,
                    event,
                    parents,
                },
            );
            bounds.insert(pair, weakest);
            bound_proofs.insert(pair, proof);
        }
    }
    let mut distinct = first.distinct.clone();
    for index in rest_indices {
        distinct.retain(|pair| closed[*index].distinct.contains(pair));
    }
    let mut distinct_proofs = HashMap::new();
    let mut distinct_keys: Vec<_> = distinct.iter().copied().collect();
    distinct_keys.sort_unstable();
    for pair in distinct_keys {
        let mut parents = Vec::with_capacity(states.len());
        for (ordinal, state) in closed.iter().enumerate() {
            let parent = if state.contradictory() {
                state
                    .contradiction
                    .expect("contradictory predecessor proof")
            } else {
                state.distinct_proofs[&pair]
            };
            parents.push(JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent,
            });
        }
        let proof = ledger.intern_for(
            view,
            DerivationNode::JoinDistinct {
                left: pair.0,
                right: pair.1,
                event,
                parents,
            },
        );
        distinct_proofs.insert(pair, proof);
    }
    // Comparison and outcome origins are path conditions, not facts; one
    // survives a join only when every contributing path carries the same one.
    let mut opaque = first.opaque.clone();
    for index in rest_indices {
        opaque.retain(|fact| closed[*index].opaque.contains(fact));
    }
    let mut opaque_proofs = HashMap::new();
    let mut opaque_keys: Vec<_> = opaque.iter().copied().collect();
    opaque_keys.sort_unstable();
    for (goal, sign) in opaque_keys {
        let mut parents = Vec::with_capacity(states.len());
        for (ordinal, state) in closed.iter().enumerate() {
            let parent = if state.contradictory() {
                state
                    .contradiction
                    .expect("contradictory predecessor proof")
            } else {
                state.opaque_proofs[&(goal, sign)]
            };
            parents.push(JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent,
            });
        }
        let proof = ledger.intern_for(
            view,
            DerivationNode::JoinGoal {
                goal,
                sign,
                event,
                parents,
            },
        );
        opaque_proofs.insert((goal, sign), proof);
    }
    let contributing_states: Vec<&FactState> =
        contributing.iter().map(|index| &states[*index]).collect();
    let mut origins = contributing_states[0].origins.clone();
    let mut outcomes = contributing_states[0].outcomes.clone();
    let mut goal_origins = contributing_states[0].goal_origins.clone();
    let mut ambiguous_goal_origins = contributing_states[0].ambiguous_goal_origins.clone();
    for state in contributing_states.iter().skip(1) {
        origins.retain(|binding, relation| {
            state
                .origins
                .get(binding)
                .is_some_and(|other| other == relation)
        });
        outcomes.retain(|binding, outcome| {
            state
                .outcomes
                .get(binding)
                .is_some_and(|other| other == outcome)
        });
        goal_origins.retain(|binding, goal| {
            state
                .goal_origins
                .get(binding)
                .is_some_and(|other| other == goal)
        });
        ambiguous_goal_origins.retain(|binding| state.ambiguous_goal_origins.contains(binding));
    }
    let bound_candidates = bound_proofs
        .iter()
        .map(|(pair, proof)| (*pair, vec![(bounds[pair], *proof)]))
        .collect();
    let distinct_candidates = distinct_proofs
        .iter()
        .map(|(pair, proof)| (*pair, vec![*proof]))
        .collect();
    FactState {
        view,
        closed_term_count: Some(
            u32::try_from(terms.ids().count())
                .expect("ENT term inventory exceeds the u32 identity space"),
        ),
        all_derivable: false,
        contradiction: None,
        bounds,
        bound_proofs,
        bound_candidates,
        distinct,
        distinct_proofs,
        distinct_candidates,
        origins,
        outcomes,
        opaque,
        opaque_proofs,
        goal_origins,
        ambiguous_goal_origins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeclarationId;
    use crate::semantic::entailment::VerifiedPostconditionSummary;
    use crate::semantic::model::FunctionId;

    #[test]
    fn dormant_const_length_aliases_still_transfer_ranges_and_equality() {
        let mut terms = TermTable::new();
        let parameter = terms.intern(TermKind::ConstParameter(
            DeclarationId::from_index(0).expect("zero declaration identity exists"),
        ));
        let length = |terms: &mut TermTable, binding| {
            let term = terms.intern(TermKind::Length(super::super::term::PlaceTerm {
                root: super::super::term::PlaceRoot::Binding(BindingId(binding)),
                deref: false,
                fields: Vec::new(),
            }));
            terms.set_length_bound(term, LengthBound::Equal(parameter));
            term
        };
        let first = length(&mut terms, 0);
        let second = length(&mut terms, 1);
        let mut ledger = DerivationLedger::default();
        let closed = close(
            &FactState::for_view(ProofView::Complete),
            &terms,
            &GoalTable::default(),
            &mut ledger,
        );

        assert!(closed.derives_bound(first, second, 0));
        assert!(closed.derives_bound(second, first, 0));
        assert!(closed.derives_bound(parameter, ZERO, u64::MAX.into()));
        assert!(closed.derives_bound(ZERO, parameter, 0));
    }

    #[test]
    fn an_empty_join_remains_the_contradictory_all_derivable_state() {
        let mut ledger = DerivationLedger::default();
        let joined = join(&[], &TermTable::new(), &GoalTable::default(), &mut ledger);
        assert!(joined.all_derivable);
        let proof = joined
            .contradiction
            .expect("the empty join has one exact contradiction proof");
        assert!(matches!(
            ledger.nodes[proof.0 as usize],
            DerivationNode::JoinContradiction { ref parents, event }
                if parents.is_empty()
                    && ledger.events[event.0 as usize].kind == FlowEventKind::Join
        ));
        ledger.add_root(DerivationRootKind::BoundsObligation(0), proof);
        let remap = ledger.finish();
        assert_eq!(ledger.nodes.len(), 1);
        assert_eq!(ledger.roots.len(), 1);
        assert_eq!(remap[proof.0 as usize], Some(DerivationId(0)));
    }

    #[test]
    fn one_structural_event_produces_distinct_view_tagged_nodes() {
        let mut ledger = DerivationLedger::default();
        let event = ledger.event(FlowEventKind::S5, None);
        let mut roots = Vec::new();
        for (ordinal, view) in [
            ProofView::Complete,
            ProofView::Unasserted,
            ProofView::S4Blinded,
        ]
        .into_iter()
        .enumerate()
        {
            let mut state = FactState::for_view(view);
            let proof = state.establish_bound_with_proof(ZERO, ZERO, 0, &mut ledger, event);
            ledger.add_root(DerivationRootKind::BoundsObligation(ordinal as u32), proof);
            roots.push(proof);
        }

        assert_eq!(ledger.events.len(), 1);
        assert_eq!(ledger.nodes.len(), 3);
        assert_eq!(ledger.nodes[0], ledger.nodes[1]);
        assert_eq!(ledger.nodes[1], ledger.nodes[2]);
        assert_eq!(
            ledger.node_views,
            vec![
                ProofView::Complete,
                ProofView::Unasserted,
                ProofView::S4Blinded,
            ]
        );

        let remap = ledger.finish();
        assert_eq!(ledger.events.len(), 1);
        assert_eq!(ledger.nodes.len(), 3);
        assert_eq!(
            ledger.node_views,
            vec![
                ProofView::Complete,
                ProofView::Unasserted,
                ProofView::S4Blinded,
            ]
        );
        assert!(
            ledger
                .nodes
                .iter()
                .all(|node| node_event(node) == Some(FlowEventId(0)))
        );
        assert!(
            roots
                .into_iter()
                .all(|root| remap[root.0 as usize].is_some())
        );
    }

    #[test]
    #[should_panic(expected = "ENT derivation parents must belong to the child's proof view")]
    fn a_derivation_cannot_import_a_parent_from_another_view() {
        let mut ledger = DerivationLedger::default();
        let event = ledger.event(FlowEventKind::S5, None);
        let mut complete = FactState::for_view(ProofView::Complete);
        let parent = complete.establish_bound_with_proof(ZERO, ZERO, 0, &mut ledger, event);
        let _ = ledger.intern_for(
            ProofView::Unasserted,
            DerivationNode::SubsumedBound {
                left: ZERO,
                right: ZERO,
                held: 0,
                requested: 1,
                parent,
            },
        );
    }

    #[test]
    fn ordinary_fallback_candidates_survive_join_and_materialization() {
        let mut terms = TermTable::new();
        let left = terms.intern(TermKind::Place(
            super::super::term::PlaceTerm {
                root: super::super::term::PlaceRoot::Binding(BindingId(0)),
                deref: false,
                fields: Vec::new(),
            },
            IntegerType::I32,
        ));
        let right = terms.intern(TermKind::Place(
            super::super::term::PlaceTerm {
                root: super::super::term::PlaceRoot::Binding(BindingId(1)),
                deref: false,
                fields: Vec::new(),
            },
            IntegerType::I32,
        ));
        let pair = (left, right);
        let ordinary = Relation::Bound {
            left: pair.0,
            right: pair.1,
            bound: 0,
        };
        let s12 = Relation::Bound {
            left: pair.0,
            right: pair.1,
            bound: -1,
        };
        let mut ledger = DerivationLedger::default();
        let source_event = ledger.event(FlowEventKind::S3, None);
        let mut state = FactState::for_view(ProofView::Complete);
        state.establish(&ordinary, &mut ledger, source_event);
        let ordinary_proof = state.bound_proofs[&pair];
        assert!(!ledger.depends_on_postcondition_call(ordinary_proof));
        let call = ledger.intern_for(
            ProofView::Complete,
            DerivationNode::PostconditionCall {
                call: NodePath {
                    components: vec![0],
                },
                relation: s12.clone(),
                summary: VerifiedPostconditionSummaryRef {
                    summary: VerifiedPostconditionSummary {
                        function: FunctionId(0),
                        block: NodePath {
                            components: vec![0, 0],
                        },
                        relation_ordinal: 0,
                        component: 0,
                    },
                    view: ProofView::Complete,
                },
                substitutions: Vec::new(),
                transfer_events: Vec::new(),
                a0_parents: Vec::new(),
                view_parents: Vec::new(),
            },
        );
        assert!(ledger.depends_on_postcondition_call(call));
        state.establish_from_proof(&s12, call, &ledger);
        assert_eq!(state.bounds[&pair], -1);

        let snapshot_event = ledger.event(FlowEventKind::Snapshot, None);
        let mut materialized = materialize_closure_at(
            &state,
            &terms,
            &GoalTable::default(),
            &mut ledger,
            snapshot_event,
        );
        materialized.retain_non_postcondition_candidates(&ledger);
        assert_eq!(materialized.bounds[&pair], 0);

        let join_event = ledger.event(FlowEventKind::Join, None);
        let mut joined = join_at(
            &[state.clone(), state],
            &terms,
            &GoalTable::default(),
            &mut ledger,
            ProofView::Complete,
            join_event,
        );
        joined.retain_non_postcondition_candidates(&ledger);
        assert_eq!(joined.bounds[&pair], 0);

        ledger.add_root(DerivationRootKind::BoundsObligation(0), ordinary_proof);
        ledger.add_root(DerivationRootKind::BoundsObligation(1), call);
        let remap = ledger.finish();
        let ordinary_proof = remap[ordinary_proof.0 as usize].expect("ordinary proof retained");
        let call = remap[call.0 as usize].expect("S12 proof retained");
        assert!(!ledger.depends_on_postcondition_call(ordinary_proof));
        assert!(ledger.depends_on_postcondition_call(call));
    }

    /// Cloning a ledger deliberately leaves its interning index behind, which
    /// is sound only because the copy rebuilds the index from its own nodes
    /// before answering. Without the rebuild a clone silently re-interns
    /// nodes it already holds, giving one proof step two identities.
    #[test]
    fn interning_into_a_cloned_ledger_keeps_the_original_identity() {
        let mut ledger = DerivationLedger::default();
        let event = ledger.event(FlowEventKind::S3, None);
        let relation = Relation::Bound {
            left: ZERO,
            right: ZERO,
            bound: 0,
        };
        let node = DerivationNode::SourceBound {
            relation,
            left: ZERO,
            right: ZERO,
            bound: 0,
            event,
        };
        let original = ledger.intern_for(ProofView::Complete, node.clone());
        let nodes = ledger.nodes.len();

        let mut copy = ledger.clone();
        assert_eq!(copy.nodes.len(), nodes);
        assert_eq!(copy.intern_for(ProofView::Complete, node.clone()), original);
        assert_eq!(copy.nodes.len(), nodes);
        // The rebuilt index must also keep separating the proof views.
        assert_ne!(copy.intern_for(ProofView::Unasserted, node), original);
        assert_eq!(copy.nodes.len(), nodes + 1);
    }

    /// The ENT-4 fixed point revisits a transitivity triple only when one of
    /// its premise cells changed since that triple was last offered. A
    /// disequality strengthens `a - b <= 0` to `-1` only after the first
    /// transitivity pass, so the strengthened edge must still reach `c`
    /// through the later rounds. A freshness rule that stopped one round too
    /// early would leave `a - c <= -1` underived.
    #[test]
    fn a_strengthened_bound_still_propagates_in_a_later_closure_round() {
        let mut terms = TermTable::new();
        let mut place = |binding| {
            terms.intern(TermKind::Place(
                super::super::term::PlaceTerm {
                    root: super::super::term::PlaceRoot::Binding(BindingId(binding)),
                    deref: false,
                    fields: Vec::new(),
                },
                IntegerType::I32,
            ))
        };
        let a = place(0);
        let b = place(1);
        let c = place(2);
        let mut ledger = DerivationLedger::default();
        let event = ledger.event(FlowEventKind::S3, None);
        let mut state = FactState::for_view(ProofView::Complete);
        state.establish(
            &Relation::Bound {
                left: a,
                right: b,
                bound: 0,
            },
            &mut ledger,
            event,
        );
        state.establish(
            &Relation::Bound {
                left: b,
                right: c,
                bound: 0,
            },
            &mut ledger,
            event,
        );
        state.establish_distinct_with_proof(a, b, &mut ledger, event);

        let closed = close(&state, &terms, &GoalTable::default(), &mut ledger);
        assert!(!closed.all_derivable);
        assert!(closed.derives_bound(a, b, -1));
        assert!(closed.derives_bound(a, c, -1));
    }
}
