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
use state::{DerivationInventory, DerivationLedger};
#[cfg(not(test))]
use term::TermId;

#[cfg(test)]
pub(crate) use state::{
    DerivationId, DerivationNode, DerivationRootKind, FlowEvent, FlowEventId, FlowEventKind,
    GoalId, GoalSign, ImplicitBoundKind, JoinParent, Relation,
};
#[cfg(test)]
pub(crate) use term::{LengthBound, TermId, TermKind, ZERO, type_range};

use std::collections::HashMap;

use super::goal::ConcreteGoal;
use super::model::{
    CheckedConstant, CheckedConstantId, CheckedFunction, CheckedMode, CheckedNominal, CheckedType,
    FunctionId,
};
use crate::{DeclarationId, NodePath};

/// Kill-relevant [EFF-2] projection of one callee signature: for each
/// parameter, whether the callee's declared effect row writes the region that
/// parameter carries, so a call kills exactly the facts whose support
/// overlaps that actual's resolved place [ENT-5](b).
#[derive(Clone, Debug, Default)]
pub(crate) struct EntailmentCallee {
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
        Self {
            parameter_writes: parameters
                .map(|mode| match mode {
                    CheckedMode::Unique(region) => writes.contains(&region),
                    CheckedMode::Own | CheckedMode::Shared(_) => false,
                })
                .collect(),
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
}

/// One bounds result retained from a counterfactual ENT rewalk [ENT-6].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RewalkObligationOutcome {
    /// The exact protected subscript occurrence consumed by provenance.
    pub(crate) node_path: NodePath,
    /// Whether the selected counterfactual fact sources discharge it.
    pub(crate) discharged: bool,
    /// The ordinary canonical residual when it remains undischarged.
    pub(crate) residual: Option<String>,
}

/// Counterfactual ENT rewalk consumed by the PRV bridge and gate [ENT-6].
///
/// This metadata deliberately strips normal-analysis term and derivation IDs:
/// the rewalk discards its private inventories and is not a DIAG-2 authority.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionEntailmentRewalk {
    /// Every protected leaf under the selected counterfactual fact sources.
    pub(crate) obligations: Vec<RewalkObligationOutcome>,
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

/// Recomputes ENT flow without S2/S3, optionally retaining body-entry S4.
///
/// PRV-2/PRV-3 source acceptance reads this counterfactual result after the
/// complete base judgment succeeds. Lowering and optimization do not read it.
pub(crate) fn rewalk_function_unasserted(
    function: &CheckedFunction,
    context: &EntailmentContext<'_>,
    include_s4: bool,
) -> FunctionEntailmentRewalk {
    flow::rewalk_unasserted(function, context, include_s4)
}

/// The engine's fragment-type gate: one member of the closed integer set
/// [OP-2], the only types terms may select [ENT-2].
const fn fragment_type(ty: CheckedType) -> Option<super::model::IntegerType> {
    match ty {
        CheckedType::Integer(ty) => Some(ty),
        _ => None,
    }
}
