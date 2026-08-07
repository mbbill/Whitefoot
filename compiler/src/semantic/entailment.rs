//! The L0 entailment fragment [ENT-1..ENT-6]: a closed, deterministic,
//! search-free derivation system over difference-bound facts.
//!
//! The engine is acceptance-bearing: [`analyze_function`] computes the
//! closed fact state along the [FN-1] structural graph, the [ENT-6]
//! disposition of every bounds obligation, and the [CLM-2] lifecycle
//! disposition of every claim. The checker rejects a function whose
//! summary contains an undischarged obligation ([OP-4], with the residual
//! rendered exactly per [ENT-6]) or a refuted claim ([CLM-2]), reports a
//! non-rejecting redundancy advisory for each redundant claim, and retains
//! the summary on the checked function [DIAG-2].
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

use super::model::{
    CheckedConstant, CheckedFunction, CheckedMode, CheckedNominal, CheckedType, FunctionId,
};
use crate::NodePath;

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
    pub(crate) nominals: &'check [CheckedNominal],
    /// Binding names in dense [`super::model::BindingId`] order, for the
    /// [ENT-6] canonical residual rendering.
    pub(crate) binding_names: &'check [String],
}

impl EntailmentContext<'_> {
    pub(crate) fn callee(&self, function: FunctionId) -> Option<&EntailmentCallee> {
        self.callees.get(function.0 as usize)
    }
}

/// [ENT-6] disposition of one bounds obligation, judged at its source node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObligationOutcome {
    /// The subscript's `psuffix` node the obligation is attached to, by its
    /// trap record's path — one record per subscript in a chain [ENT-6].
    pub(crate) node_path: NodePath,
    /// The closed fact state at the node derives the normalized relation.
    pub(crate) discharged: bool,
    /// The state at the node was contradictory, discharging everything.
    pub(crate) contradictory: bool,
    /// The exact residual rendering for an undischarged obligation: the
    /// offset atom's canonical source bytes, ` < len(`, the base place's
    /// canonical source bytes, `)`.
    pub(crate) residual: Option<String>,
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

/// Retained summary of one function's entailment analysis [DIAG-2].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionEntailment {
    /// Bounds obligations in deterministic source walk order.
    pub(crate) obligations: Vec<ObligationOutcome>,
    /// Claim lifecycle outcomes in deterministic source walk order.
    pub(crate) claims: Vec<ClaimOutcome>,
}

/// Computes the L0 entailment analysis of one checked function body.
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

/// The engine's fragment-type gate: one member of the closed integer set
/// [OP-2], the only types terms may select [ENT-2].
const fn fragment_type(ty: CheckedType) -> Option<super::model::IntegerType> {
    match ty {
        CheckedType::Integer(ty) => Some(ty),
        _ => None,
    }
}
