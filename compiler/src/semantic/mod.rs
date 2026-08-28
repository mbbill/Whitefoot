//! Target-independent semantic checking for the active Whitefoot specification.
//!
//! This stage consumes complete lexical resolution and is the sole producer of
//! the private checked-program value that may later authorize lowering. A
//! language feature not implemented yet is reported as an unsupported compiler
//! capability, never as a source-language rejection.

mod check;
mod claim_locality;
mod entailment;
mod goal;
mod loop_permission;
mod model;
pub(crate) mod permission;
mod permission_ledger;
mod places;
mod postcondition;
mod provenance;
mod staged_permission;
mod target_action;
mod tree;

#[cfg(test)]
mod tests;

use crate::{
    BundleSourceExtent, DeclarationId, NodePath, ResolutionIssue, ResolvedSyntaxUnit,
    SyntaxCoordinate,
};

pub use check::check_semantics;
#[cfg(test)]
pub(crate) use check::check_semantics_arithmetic_obligations;
#[cfg(test)]
pub(crate) use check::check_semantics_division_obligations;
#[cfg(test)]
pub(crate) use check::check_semantics_reborrow_extension;

/// The permission table the overlap lowering reads. It is the same table the
/// ledger renders; nothing derives a second judgment from it.
pub(crate) use permission::FunctionPermissions;

/// One counted loop's [PAR-2] verdict and, where the loop is permitted and
/// eligible, the two identities actualizing it needs. Lowering reads these; it
/// never derives a verdict of its own from them.
pub(crate) use loop_permission::{LoopActualization, LoopCombine, LoopPermission};

pub(crate) use model::{
    BindingId, CheckedArrayRoot, CheckedBodyDisposition, CheckedBooleanOperation,
    CheckedBufferRoot, CheckedBufferSetTarget, CheckedConstructor, CheckedDrop, CheckedEntryForm,
    CheckedEnumType, CheckedExpression, CheckedFlatElement, CheckedFloatOperation, CheckedFunction,
    CheckedIntegerOperation, CheckedLayoutCeiling, CheckedLayoutMagnitude, CheckedLoopId,
    CheckedMatchArm, CheckedMode, CheckedNominalKind, CheckedNumericType, CheckedParameter,
    CheckedProgramData, CheckedProjectedDrop, CheckedRuntimeTargetObligations, CheckedSetTarget,
    CheckedSliceRoot, CheckedSliceSource, CheckedStatement, CheckedTargetDomainObligation,
    CheckedType, CheckedValue, ClaimSite, NominalId, PropagationContext,
};

/// Master switch for the v0.31 candidate's gated semantic surface:
/// struct-typed named consts [CONST-2 candidate] and the clause-conditional
/// OWN-1 bare-affine repair [#35].
///
/// `true` selects the candidate semantics, matched in the same change by the
/// v0.31 candidate specification bytes and the grammar tables regenerated
/// from them — the const-arithmetic and construction-cvalue grammar shapes
/// are additionally gated by those tables and need no switch of their own.
pub(crate) const V031_CANDIDATE_SEMANTICS: bool = true;

/// Numbered rule owning one post-resolution semantic rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRule {
    /// Generic numeric identity literal eligibility.
    Form5,
    /// Numeric literal range or canonicality.
    Form7,
    /// Composite-type formation and element eligibility.
    Type2,
    /// Constant-expression formation and evaluation.
    Const1,
    /// Named-constant type and value formation.
    Const2,
    /// Exact mode/type agreement.
    Type5,
    /// Copy-place assignment target formation and writability.
    Set1,
    /// Affine-place replacement target class and commit.
    Set2,
    /// Copy-versus-affine use spelling.
    Own1,
    /// Borrow liveness and region ordering.
    Own4,
    /// Live-loan access and exclusivity.
    Own5,
    /// Statement-scoped child-reborrow formation and suspension.
    Own6,
    /// Borrow storage duration.
    Own10,
    /// Loop-local region and move restrictions.
    Own11,
    /// Region substitution and call-boundary loan checks.
    Own12,
    /// Non-argument reborrow disposition and the returned reborrow.
    Own14,
    /// Explicit dereference of a borrow holder.
    Type7,
    /// Storage-class and affine replacement restrictions.
    Stor1,
    /// Arena confinement to its region's block.
    Stor4,
    /// Borrow-free and region-free stored-content formation.
    Stor5,
    /// Operation-table row selection.
    Op1,
    /// Exact integer arithmetic semantics and the constant-operand-class
    /// overflow-obligation discharge.
    Op2,
    /// Subscript bounds-obligation discharge and offset typing.
    Op4,
    /// Exact conversion-pair result classification.
    Op6,
    /// Runtime-sized buffer allocation-domain discharge.
    Op9,
    /// Exact `own Bool` explicit-check condition.
    Op5,
    /// Function result, reachability, or completion.
    Fn1,
    /// Explicit generic-instantiation argument presence.
    Fn2,
    /// Generic bounds and source-contract conformance.
    Fn3,
    /// Closed source-law declaration and discharge.
    Fn4,
    /// Polymorphic recursion in a call cycle among generic functions.
    Fn6,
    /// Closed-program `main` contract.
    Fn7,
    /// Finite atomic function requirement goal.
    Fn8,
    /// Verified narrow normal-return relation.
    Fn9,
    /// Type-driven conditional form, and the `else` spellings it forbids.
    Gram6,
    /// Exact declared-order named user-call arguments.
    Gram11,
    /// Exact declared-order construction fields.
    Gram8,
    /// Exact declared-order match binders.
    Gram10,
    /// Constructor/variant owner agreement.
    Type6,
    /// Exhaustive enum matching.
    Err2,
    /// Exact Result propagation and same-error forwarding.
    Err3,
    /// Value-match delivery.
    Give1,
    /// Effect-row canonicality.
    Eff1,
    /// Exact exhibited-versus-declared effect row.
    Eff2,
    /// The system inventory, and the region arguments a system operation's
    /// call site must state. [TYPE-5] assigns the written arguments by callee
    /// class — "region arguments for system operations [SYS-2]" — so this rule
    /// owns that argument list exactly as FN-2 owns a user generic's.
    Sys2,
    /// Half-open system buffer-range discharge.
    Sys8,
    /// Named runtime claim formation and per-function name uniqueness.
    Clm1,
    /// Claim lifecycle: refutation rejection under the entailment fragment.
    Clm2,
    /// Opt-in strict no-claim partition and its imported-claim boundary.
    Clm3,
    /// Counted endpoint admission to the closed term-or-constant vocabulary.
    Ent2,
    /// External actual protecting one downstream constrained subject.
    Prv2,
    /// External local constrained subject authorized only by assertion state.
    Prv3,
}

impl SemanticRule {
    /// Returns the exact numbered rule spelling from the active kernel specification.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Form5 => "FORM-5",
            Self::Form7 => "FORM-7",
            Self::Type2 => "TYPE-2",
            Self::Const1 => "CONST-1",
            Self::Const2 => "CONST-2",
            Self::Type5 => "TYPE-5",
            Self::Set1 => "SET-1",
            Self::Set2 => "SET-2",
            Self::Own1 => "OWN-1",
            Self::Own4 => "OWN-4",
            Self::Own5 => "OWN-5",
            Self::Own6 => "OWN-6",
            Self::Own10 => "OWN-10",
            Self::Own11 => "OWN-11",
            Self::Own12 => "OWN-12",
            Self::Own14 => "OWN-14",
            Self::Type7 => "TYPE-7",
            Self::Stor1 => "STOR-1",
            Self::Stor4 => "STOR-4",
            Self::Stor5 => "STOR-5",
            Self::Op1 => "OP-1",
            Self::Op2 => "OP-2",
            Self::Op4 => "OP-4",
            Self::Op6 => "OP-6",
            Self::Op9 => "OP-9",
            Self::Op5 => "OP-5",
            Self::Fn1 => "FN-1",
            Self::Fn2 => "FN-2",
            Self::Fn3 => "FN-3",
            Self::Fn4 => "FN-4",
            Self::Fn6 => "FN-6",
            Self::Fn7 => "FN-7",
            Self::Fn8 => "FN-8",
            Self::Fn9 => "FN-9",
            Self::Gram6 => "GRAM-6",
            Self::Gram11 => "GRAM-11",
            Self::Gram8 => "GRAM-8",
            Self::Gram10 => "GRAM-10",
            Self::Type6 => "TYPE-6",
            Self::Err2 => "ERR-2",
            Self::Err3 => "ERR-3",
            Self::Give1 => "GIVE-1",
            Self::Eff1 => "EFF-1",
            Self::Eff2 => "EFF-2",
            Self::Sys2 => "SYS-2",
            Self::Sys8 => "SYS-8",
            Self::Clm1 => "CLM-1",
            Self::Clm2 => "CLM-2",
            Self::Clm3 => "CLM-3",
            Self::Ent2 => "ENT-2",
            Self::Prv2 => "PRV-2",
            Self::Prv3 => "PRV-3",
        }
    }

    /// The first rule in the active specification's definition order.
    #[cfg(test)]
    pub(crate) const FIRST: Self = Self::Form5;

    /// The rule defined immediately after this one, or `None` for the last.
    ///
    /// This is the enumeration of the semantic rules, and it exists so that a
    /// checked set over them cannot silently omit one. The match is exhaustive,
    /// so a new variant does not compile until it is given a position here and
    /// a rank in [`Self::definition_rank`] — two matches that
    /// `definition_rank_matches_the_active_specification` then checks against
    /// each other, since walking this chain must yield the ranks 0, 1, 2, … in
    /// order. `SemanticRule::Gram6` was omitted from that check's
    /// hand-maintained list until 2026-08-08, which is the omission this makes
    /// impossible rather than merely unlikely.
    #[cfg(test)]
    pub(crate) const fn next_in_definition_order(self) -> Option<Self> {
        Some(match self {
            Self::Form5 => Self::Form7,
            Self::Form7 => Self::Gram6,
            Self::Gram6 => Self::Give1,
            Self::Give1 => Self::Gram8,
            Self::Gram8 => Self::Gram10,
            Self::Gram10 => Self::Gram11,
            Self::Gram11 => Self::Type2,
            Self::Type2 => Self::Type5,
            Self::Type5 => Self::Type6,
            Self::Type6 => Self::Type7,
            Self::Type7 => Self::Set1,
            Self::Set1 => Self::Set2,
            Self::Set2 => Self::Const1,
            Self::Const1 => Self::Const2,
            Self::Const2 => Self::Own1,
            Self::Own1 => Self::Own4,
            Self::Own4 => Self::Own5,
            Self::Own5 => Self::Own6,
            Self::Own6 => Self::Own10,
            Self::Own10 => Self::Own11,
            Self::Own11 => Self::Own12,
            Self::Own12 => Self::Own14,
            Self::Own14 => Self::Stor1,
            Self::Stor1 => Self::Stor4,
            Self::Stor4 => Self::Stor5,
            Self::Stor5 => Self::Op1,
            Self::Op1 => Self::Op2,
            Self::Op2 => Self::Op4,
            Self::Op4 => Self::Op5,
            Self::Op5 => Self::Op6,
            Self::Op6 => Self::Op9,
            Self::Op9 => Self::Fn1,
            Self::Fn1 => Self::Fn2,
            Self::Fn2 => Self::Fn3,
            Self::Fn3 => Self::Fn4,
            Self::Fn4 => Self::Fn6,
            Self::Fn6 => Self::Fn7,
            Self::Fn7 => Self::Fn8,
            Self::Fn8 => Self::Fn9,
            Self::Fn9 => Self::Eff1,
            Self::Eff1 => Self::Eff2,
            Self::Eff2 => Self::Err2,
            Self::Err2 => Self::Err3,
            Self::Err3 => Self::Sys2,
            Self::Sys2 => Self::Sys8,
            Self::Sys8 => Self::Clm1,
            Self::Clm1 => Self::Clm2,
            Self::Clm2 => Self::Clm3,
            Self::Clm3 => Self::Ent2,
            Self::Ent2 => Self::Prv2,
            Self::Prv2 => Self::Prv3,
            Self::Prv3 => return None,
        })
    }

    /// [DIAG-1] same-node citation rank: this rule's definition position in
    /// the active kernel specification. Simultaneously established
    /// post-resolution rejections whose offending premise is the same use of
    /// the same canonical node are one rejection event citing the established
    /// rule whose rank is least, so a site with a known simultaneity asks its
    /// judgments in ascending rank order. The order is machine-checked
    /// against the active specification text by
    /// `definition_rank_matches_the_active_specification`.
    #[must_use]
    pub const fn definition_rank(self) -> usize {
        match self {
            Self::Form5 => 0,
            Self::Form7 => 1,
            Self::Gram6 => 2,
            Self::Give1 => 3,
            Self::Gram8 => 4,
            Self::Gram10 => 5,
            Self::Gram11 => 6,
            Self::Type2 => 7,
            Self::Type5 => 8,
            Self::Type6 => 9,
            Self::Type7 => 10,
            Self::Set1 => 11,
            Self::Set2 => 12,
            Self::Const1 => 13,
            Self::Const2 => 14,
            Self::Own1 => 15,
            Self::Own4 => 16,
            Self::Own5 => 17,
            Self::Own6 => 18,
            Self::Own10 => 19,
            Self::Own11 => 20,
            Self::Own12 => 21,
            Self::Own14 => 22,
            Self::Stor1 => 23,
            Self::Stor4 => 24,
            Self::Stor5 => 25,
            Self::Op1 => 26,
            Self::Op2 => 27,
            Self::Op4 => 28,
            Self::Op5 => 29,
            Self::Op6 => 30,
            Self::Op9 => 31,
            Self::Fn1 => 32,
            Self::Fn2 => 33,
            Self::Fn3 => 34,
            Self::Fn4 => 35,
            Self::Fn6 => 36,
            Self::Fn7 => 37,
            Self::Fn8 => 38,
            Self::Fn9 => 39,
            Self::Eff1 => 40,
            Self::Eff2 => 41,
            Self::Err2 => 42,
            Self::Err3 => 43,
            Self::Sys2 => 44,
            Self::Sys8 => 45,
            Self::Clm1 => 46,
            Self::Clm2 => 47,
            Self::Clm3 => 48,
            Self::Ent2 => 49,
            Self::Prv2 => 50,
            Self::Prv3 => 51,
        }
    }
}

/// Exact checked location selected for a semantic rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticLocation {
    /// One source-backed production node and its rule-selected coordinate.
    SourceNode(NodePath, SyntaxCoordinate),
    /// The closed compilation-unit root when no source declaration exists.
    BundleRoot(Vec<BundleSourceExtent>),
}

/// The [CLM-2] refutation payload: the rejection carries the claim name, the
/// predicate, and the derived negation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefutedClaimDetail {
    /// The claim's written name.
    pub name: String,
    /// The claim predicate as a normalized relation.
    pub predicate: String,
    /// The derived negation.
    pub negation: String,
    /// Stable concrete generic instance spelling, or `None` for a source
    /// schema or nongeneric occurrence.
    pub instance: Option<String>,
}

/// One CLM-1 canonical-formation or CLM-2 lifecycle/residual-canonicality
/// rejection other than an exact refutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidClaimDetail {
    pub name: String,
    pub predicate: String,
    pub classification: &'static str,
    pub component: Option<u32>,
    pub reason: &'static str,
    /// Stable concrete generic instance spelling, or `None` for a source
    /// schema or nongeneric occurrence.
    pub instance: Option<String>,
}

/// The call boundary whose result reached a non-local claim component
/// [CLM-1].  Every published identity is source-stable: a user call names its
/// declaration, while a system call names its catalog spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimBoundaryResultDetail {
    /// An ordinary user-call result.
    UserCall {
        /// Source declaration identity of the called function.
        declaration: DeclarationId,
        /// Source function spelling, never a concrete-instance symbol.
        callee: String,
    },
    /// A system-call result.
    SystemCall {
        /// Zero-based [SYS-2] system declaration ordinal.
        declaration_ordinal: u8,
        /// Stable system-operation spelling.
        operation: String,
    },
}

/// A CLM-1 claim-locality rejection.  The component is the least canonical
/// contribution component that reads a value descended from a call result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonLocalClaimDetail {
    /// The claim's written name.
    pub name: String,
    /// Least non-local canonical contribution component ordinal.
    pub component: u32,
    /// Source rendering of the first canonical support that observes the
    /// selected earliest boundary witness.
    pub carrier: String,
    /// Earliest source call occurrence that introduced the boundary result.
    pub boundary_call: NodePath,
    /// Stable kind and callee identity of that boundary.
    pub boundary: ClaimBoundaryResultDetail,
    /// Exact mechanical repair selected by CLM-1.
    pub mechanical_fix: &'static str,
}

/// One non-discharged static source obligation disposition [ENT-6].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticObligationDisposition {
    /// The closed state derives the canonical goal or normalization false.
    Refuted,
    /// The closed state derives neither a successful nor a refuting route.
    Unproved,
}

/// One non-discharged [FN-8] ordinary-call goal disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallRequirementDisposition {
    /// The entering state derives the goal's exact negative sign.
    Refuted,
    /// The entering state derives neither exact sign.
    Unproved,
}

/// The deterministic [FN-8] ordinary-call rejection payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndischargedCallRequirementDetail {
    /// The resolved concrete, possibly generic, callee instance.
    pub concrete_callee: String,
    /// The callee requirement occurrence's `requires_clause` path.
    pub requires_clause: NodePath,
    /// Stable structural rendering of the complete instantiated typed goal.
    pub instantiated_goal: String,
    /// The exact non-discharged disposition.
    pub disposition: CallRequirementDisposition,
    /// The rule-selected mechanical restructuring.
    pub mechanical_fix: &'static str,
}

/// The fixed proof view used by every [CLM-3] non-claim query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrictProofView {
    /// Existing S3-disabled U view, with independently proved S4 retained.
    Unasserted,
}

/// Public lifecycle spelling retained in a direct [CLM-3] claim diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrictClaimLifecycleDisposition {
    /// A CLM-1/CLM-2-validated proof residual retained as a runtime check.
    Retained,
}

/// The least downstream direct-claim identity carried by an import event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictClaimIdentityDetail {
    pub concrete_function: String,
    pub claim: NodePath,
    pub name: String,
}

/// A direct claim in the marked root's own concrete SCC [CLM-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictDirectClaimDetail {
    pub strict_root: String,
    pub concrete_claim_owner: String,
    pub claim: NodePath,
    pub name: String,
    pub predicate: String,
    pub justification: String,
    pub lifecycle: StrictClaimLifecycleDisposition,
}

/// A root-SCC call importing a nonempty downstream `MayClaims` set [CLM-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictImportedClaimDetail {
    pub strict_root: String,
    pub concrete_caller: String,
    pub call: NodePath,
    pub concrete_callee: String,
    pub least_downstream_claim: StrictClaimIdentityDetail,
}

/// One demanded or marked-boundary U-view call-goal failure [FN-8].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictUndischargedCallRequirementDetail {
    pub strict_root: String,
    pub concrete_caller: String,
    pub concrete_callee: String,
    pub requires_clause: NodePath,
    pub instantiated_goal: String,
    pub disposition: CallRequirementDisposition,
    pub view: StrictProofView,
    pub mechanical_fix: &'static str,
}

/// One non-discharged complete-view [FN-9] relation disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostconditionProofDisposition {
    Refuted,
    Unproved,
}

/// The deterministic [FN-9] selected-return rejection payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndischargedPostconditionDetail {
    /// The concrete, possibly generic, function instance.
    pub concrete_function: String,
    /// The unique postcondition occurrence's block path.
    pub postcondition: NodePath,
    /// The fixed relation occurrence ordinal (zero in this version).
    pub conjunct: u32,
    /// Exact admitted selector identity.
    pub selector: NodePath,
    /// The instantiated normalized relation at the selected exit.
    pub relation: String,
    /// The exact non-discharged complete-view disposition.
    pub disposition: PostconditionProofDisposition,
}

/// Public finite spelling of one PRV-1 parameter component selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceDatumSelector {
    /// A non-payload value's sole component.
    Plain,
    /// One direct enum payload projection, never a recursive payload path.
    EnumPayload {
        /// Zero-based variant declaration ordinal.
        variant: u32,
        /// Zero-based payload-field declaration ordinal.
        field: u32,
    },
}

/// One exact finite parameter datum rendered in a provenance diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceParameterDatumDetail {
    /// Zero-based value-parameter ordinal.
    pub ordinal: u32,
    /// Exact plain or direct-payload selector.
    pub selector: ProvenanceDatumSelector,
}

/// The identity class of one ordered provenance target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceDemandKind {
    /// The protected leaf is local to the rejected function [PRV-3].
    LocalLeaf,
    /// A direct caller-visible parameter demand [PRV-2].
    Direct,
    /// An exact S4 requirement occurrence bridges to the leaf [PRV-2].
    RequirementBridge,
}

/// One complete direct or requirement-bridge demand state retained at a
/// diagnostic call boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceDemandStateDetail {
    pub demand_kind: ProvenanceDemandKind,
    /// Concrete function that owns this boundary state.
    pub function: String,
    pub parameter: ProvenanceParameterDatumDetail,
    /// Exact source-ordered occurrence identity for a requirement-bridge
    /// state. Empty for a direct state.
    pub requirements: Vec<NodePath>,
    pub protected_function: String,
    pub protected_leaf: NodePath,
    pub protected_conjunct: u32,
}

/// One ordered call boundary in a post-convergence provenance witness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceBoundaryDetail {
    pub call: NodePath,
    pub argument_node: NodePath,
    pub argument: u32,
    pub callee: ProvenanceDemandStateDetail,
    /// Absent only at the real true-bit terminal boundary.
    pub caller_continuation: Option<ProvenanceDemandStateDetail>,
}

/// One exact PRV-1 predecessor edge and its complete checked source extent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceWriteContextDetail {
    /// Exact writable formal ordinal at the system/user call boundary.
    pub parameter: u32,
    /// Exact caller actual atom paired with the writable formal.
    pub actual: NodePath,
    pub actual_coordinate: SyntaxCoordinate,
}

/// The positive transfer represented by a call carrier.  A parameter-backed
/// user result/write has a distinct receiving edge and substitution edge even
/// though both use the same source call NodePath.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceCarrierCallRole {
    SystemResult,
    SystemWrite,
    UserResult,
    UserWrite,
    UserSubstitution,
}

/// One exact PRV-1 predecessor edge and its complete checked source extent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceCarrierStepDetail {
    pub path: NodePath,
    pub selector: ProvenanceDatumSelector,
    pub call_role: Option<ProvenanceCarrierCallRole>,
    /// Explanation-only write identity attached to this one call edge.
    pub write_context: Option<ProvenanceWriteContextDetail>,
    pub coordinate: SyntaxCoordinate,
}

/// The entry-local S4 bridge is rooted directly at the retained requirement;
/// it has no source call boundary or caller continuation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceLocalBridgePredecessor {
    Local,
}

/// One complete ordered target retained by a PRV-2/PRV-3 diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceTargetDetail {
    /// Local, direct, or exact requirement-bridge identity.
    pub demand_kind: ProvenanceDemandKind,
    /// Exact selected callee datum for PRV-2; absent only for a PRV-3 leaf.
    pub callee_parameter: Option<ProvenanceParameterDatumDetail>,
    /// Concrete function instance that owns the protected leaf.
    pub protected_function: String,
    /// Exact protected obligation occurrence.
    pub protected_leaf: NodePath,
    /// Normalized conjunct ordinal (zero for the current bounds family).
    pub protected_conjunct: u32,
    /// Concrete function owning an exact requirement occurrence, if bridged.
    pub requirement_function: Option<String>,
    /// Exact source-ordered clause set for a requirement bridge.
    pub requirements: Vec<NodePath>,
    /// Present only for an entry-local PRV-3 leaf whose U success came from
    /// its own S4 requirement while B failed.
    pub local_bridge_predecessor: Option<ProvenanceLocalBridgePredecessor>,
    /// Exact ENT-6 residual at the downstream protected leaf.
    pub residual: String,
    /// Ordered parameter-only explanations beside a terminating true bit.
    pub companion_parameter_datums: Vec<ProvenanceParameterDatumDetail>,
    /// Complete ordered call boundaries; their full state identities are also
    /// the deterministic cycle-cut and tie-break key.
    pub boundaries: Vec<ProvenanceBoundaryDetail>,
    /// Selector-preserving PRV-1 suffix used for deterministic tie-breaking
    /// and source rendering.
    pub carrier: Vec<ProvenanceCarrierStepDetail>,
    /// Coordinate of the labelled-entry or system origin (the final carrier).
    pub origin_coordinate: SyntaxCoordinate,
    /// Deterministic carrier chain ending at a command-param or SYS-2 call.
    pub witness: Vec<NodePath>,
    /// Repair specific to this target's direct/bridge/local identity.
    pub target_repair: &'static str,
}

/// Complete coalesced target set for one provenance rejection event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceGateDetail {
    /// Ordered nonempty target set. One call argument can carry both kinds.
    pub targets: Vec<ProvenanceTargetDetail>,
    /// Target whose deterministic witness and target-specific repair render.
    pub selected_target: u32,
    /// Alternative common to every target: remove the external subject route.
    pub restructure_alternative: &'static str,
}

/// Structured reason for one semantic rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticIssueKind {
    /// A literal is not the unique in-range FORM-7 spelling.
    InvalidIntegerLiteral,
    /// A float literal is not FORM-5's unique finite canonical spelling.
    InvalidFloatLiteral,
    /// A named constant value does not exactly inhabit its written type.
    InvalidConstValue,
    /// A const-expression's compile-time evaluation has no u64 result: the
    /// mathematical result lies outside the domain or the divisor is zero.
    /// This is the const-eval overflow policy's rejection [CONST-1]; it is
    /// never a runtime trap and never enters EFF-2's exhibits-traps relation.
    ConstEvalOverflow {
        /// Bare spelling of the rejected const operation.
        operation: &'static str,
    },
    /// A const-expression names a runtime arithmetic mode; const evaluation
    /// has exactly the five bare spellings under the const-eval overflow
    /// policy [CONST-1].
    ConstRuntimeArithmeticMode {
        /// Exact mechanical repair selected by CONST-1.
        mechanical_fix: &'static str,
    },
    /// Two exact written modes or types disagree.
    TypeMismatch,
    /// A constant was selected as an assignment target.
    ImmutableSetTarget,
    /// SET-1's closed writability relation did not admit the target root.
    InvalidSetTarget {
        /// Resolved target-root class.
        root_class: String,
        /// Closed set of classes required by SET-1.
        required_classes: &'static str,
    },
    /// An affine final place cannot be replaced by `set`.
    AffineSetTarget {
        /// Exact selected affine type.
        target_type: String,
        /// Required STOR-1 restructuring.
        mechanical_fix: &'static str,
    },
    /// A `replace` target's final selected type is not an admitted
    /// region-free affine type [SET-2].
    InvalidReplaceTarget {
        /// Exact selected type.
        target_type: String,
        /// Required SET-2 restructuring.
        mechanical_fix: &'static str,
    },
    /// `move` was written for a copy value.
    MoveOfCopy {
        /// Exact mechanical repair required by OWN-1.
        mechanical_fix: &'static str,
    },
    /// An affine value was used without its required consuming spelling.
    BareAffineUse {
        /// Exact mechanical repair required by OWN-1.
        mechanical_fix: &'static str,
    },
    /// An affine buffer element was moved out of its slot; elements leave
    /// and enter their slots only through [SET-2] replacement [TYPE-2].
    AffineElementMove {
        /// Exact restructuring required by TYPE-2.
        mechanical_fix: &'static str,
    },
    /// A binding was used after ownership had already been consumed.
    UseAfterMove {
        /// Exact restructuring required by OWN-1.
        mechanical_fix: &'static str,
    },
    /// A borrow was stored or passed into a region it cannot outlive.
    InvalidBorrowLifetime,
    /// A read, write, move, or new borrow conflicts with a live loan.
    BorrowConflict,
    /// A written child reborrow does not satisfy OWN-6's closed form.
    InvalidChildReborrow {
        /// Exact restructuring required by OWN-6 at this site.
        mechanical_fix: &'static str,
    },
    /// A written reborrow form occurred outside OWN-14's admitted positions,
    /// or a return-position reborrow failed OWN-14's admission.
    InvalidReborrowPosition {
        /// Exact restructuring required by OWN-14.
        mechanical_fix: &'static str,
    },
    /// A declared callable boundary returns a borrow whose source the
    /// signature does not determine, so no caller can bind its result.
    AmbiguousResultProvenance {
        /// Exact restructuring required by FN-1.
        mechanical_fix: &'static str,
    },
    /// A borrow holder was used without the required explicit dereference.
    MissingDereference {
        /// Exact mechanical repair selected by TYPE-7.
        mechanical_fix: &'static str,
    },
    /// A loop attempted to consume an affine binding declared outside it.
    MoveOuterBindingInLoop {
        /// Exact restructuring required by OWN-11.
        mechanical_fix: &'static str,
    },
    /// A borrow created in a loop names a region introduced outside that loop.
    BorrowRegionOutsideLoop {
        /// Exact restructuring required by OWN-11.
        mechanical_fix: &'static str,
    },
    /// The selected operation family has no row for the written arguments.
    InvalidOperation,
    /// A contract or claim predicate is not exactly `own Bool`.
    InvalidPredicateCondition,
    /// A claim predicate contains computation outside CLM-1's total,
    /// observational, non-consuming proof-expression subset.
    InvalidClaimProofPredicate { reason: &'static str },
    /// A decoded claim justification is not the exact five-field CLM-1
    /// review record.
    InvalidClaimJustification { expected: &'static str },
    /// A canonical claim component reads a user-call or system-call result,
    /// or a value transitively derived from one [CLM-1].
    NonLocalClaim(Box<NonLocalClaimDetail>),
    /// A conditional was written in a form GRAM-6 does not admit for its
    /// class: a Bool-scrutinee `match`, an empty `else`, or an `else` block
    /// holding exactly one `if`.
    InvalidConditionalForm {
        /// Exact mechanical repair selected by GRAM-6.
        mechanical_fix: &'static str,
    },
    /// A later claim in the same function repeats a claim-name spelling.
    DuplicateClaimName {
        /// Repeated claim name.
        name: String,
    },
    /// A subscript's bounds obligation is not derivable from the closed fact
    /// state at its node [OP-4, ENT-6].
    UndischargedBoundsObligation {
        /// The exact ENT-6 residual rendering: offset atom, ` < len(`, base
        /// place, `)`.
        residual: String,
        /// The mechanical fix ENT-6 names.
        mechanical_fix: &'static str,
    },
    /// One proof-required exact integer operation's canonical `.defined`
    /// goal is not derivable from the closed fact state [OP-2, ENT-6].
    UndischargedIntegerDomainObligation {
        /// The exact canonical `.defined` predicate for this occurrence.
        residual: String,
        /// The exact non-discharged complete-view disposition.
        disposition: StaticObligationDisposition,
        /// The mechanical fix OP-2 names.
        mechanical_fix: &'static str,
    },
    /// A runtime-sized buffer allocation lacks an OP-9 fit proof.
    UndischargedAllocationFitObligation {
        residual: String,
        mechanical_fix: &'static str,
    },
    /// One half-open system buffer-range conjunct lacks a SYS-8 proof.
    UndischargedSystemRangeObligation {
        residual: String,
        mechanical_fix: &'static str,
    },
    /// The complete instantiated requirement at an ordinary call is refuted
    /// or unproved in the caller's pre-transfer state [FN-8].
    UndischargedCallRequirement(Box<UndischargedCallRequirementDetail>),
    /// A demanded call or outside caller-to-marked-root boundary fails the
    /// existing unasserted U goal judgment [FN-8, CLM-3].
    StrictUndischargedCallRequirement(Box<StrictUndischargedCallRequirementDetail>),
    /// A full-state-accepted call passes an unconditionally external actual
    /// into one or more protected downstream subjects [PRV-2].
    ExternalProtectedCallArgument(Box<ProvenanceGateDetail>),
    /// A full-state-discharged local leaf relies on assertion state for an
    /// unconditionally external constrained subject [PRV-3].
    ExternalProtectedSubject(Box<ProvenanceGateDetail>),
    /// A counted endpoint produced `own u64` but was not itself one preceding
    /// ENT-2 term or constant.
    InvalidCountedEndpoint {
        /// The exact restructuring required by ENT-2.
        mechanical_fix: &'static str,
    },
    /// The fact state at a claim derives the exact negation of its predicate
    /// [CLM-2].
    RefutedClaim(Box<RefutedClaimDetail>),
    /// A claim is vacuous, redundant, overlapping, inconsistent,
    /// unreconstructable, unsupported, or not individually load-bearing.
    InvalidClaim(Box<InvalidClaimDetail>),
    /// A direct claim belongs to the marked root's own SCC [CLM-3].
    StrictDirectClaim(Box<StrictDirectClaimDetail>),
    /// A root-SCC call imports a nonempty downstream `MayClaims` set [CLM-3].
    StrictImportedClaim(Box<StrictImportedClaimDetail>),
    /// A return expression disagrees with the written function result.
    ReturnMismatch,
    /// A returned direct slice may originate outside its signature ceiling.
    InvalidSliceReturnOrigin {
        /// Required FN-1 restructuring.
        mechanical_fix: &'static str,
    },
    /// A borrow-mode result cannot directly refer to a slice descriptor.
    BorrowedSliceResult {
        /// Required FN-1 restructuring.
        mechanical_fix: &'static str,
    },
    /// A generic type argument contains a region-bearing value.
    RegionBearingGenericArgument {
        /// Required FN-2 restructuring.
        mechanical_fix: &'static str,
    },
    /// A call on a cycle among generic functions instantiates its callee at
    /// something other than exactly the caller's own type parameters [FN-6].
    PolymorphicRecursion {
        /// The cycle FN-6 requires the diagnostic to name: the function
        /// spellings along the shortest cycle through this call, in call
        /// order, joined by ` -> ` and closed on the caller.
        cycle: String,
        /// Required FN-6 restructuring.
        mechanical_fix: &'static str,
    },
    /// A stored-content position contains a region-bearing value.
    RegionBearingStorage {
        /// Required STOR-5 restructuring.
        mechanical_fix: &'static str,
    },
    /// An arena value would leave its region's block [STOR-4]: it may not be
    /// returned, stored into a field, or moved to an outside destination.
    ArenaEscape {
        /// Required STOR-4 restructuring.
        mechanical_fix: &'static str,
    },
    /// A slice-valued value match would require an unselected origin join.
    SliceValueMatch {
        /// Required OWN-5 restructuring.
        mechanical_fix: &'static str,
    },
    /// A statement follows a structurally terminating statement.
    UnreachableStatement,
    /// The function body can reach its closing brace.
    FunctionFallthrough,
    /// A requirement entry uses a construct outside the admitted FN-8 goal subset.
    InvalidRequires,
    /// An ensures selector does not match the concrete result class FN-9 admits.
    InvalidPostconditionSelector,
    /// A variant selector does not spell exact `Ok(value: result)`.
    InvalidPostconditionFields {
        /// Exact closed field list required by the admitted selector.
        required_fields: Vec<String>,
    },
    /// The symbolic result candidate conflicts with a live declaration.
    PostconditionCandidateNotFresh {
        /// Written candidate spelling.
        spelling: String,
        /// Ordered live declaration origins that conflict with the candidate.
        conflicts: Vec<crate::SourceOrigin>,
    },
    /// A later ensures-local declaration attempts to shadow the symbolic result.
    PostconditionLocalShadowsResult {
        /// Written candidate spelling.
        spelling: String,
        /// The admitted selector candidate's exact origin.
        selector: crate::SourceOrigin,
    },
    /// An ensures entry uses a construct outside FN-9's proof-only ANF subset.
    InvalidPostconditionClause,
    /// The alpha-expanded final condition is not one output-bearing L0 relation.
    InvalidPostconditionRelation,
    /// A selected Result exit is not a direct canonical `Ok(value: atom)` or `Err(error: atom)`.
    InvalidPostconditionReturn,
    /// One concrete postcondition has no selected normal exit.
    NoSelectedNormalExit {
        /// The exact fixed residual required by FN-9.
        residual: &'static str,
    },
    /// A selected normal return's complete instantiated FN-9 relation is
    /// refuted or unproved after entry-image stability and ordinary kills.
    UndischargedPostcondition(Box<UndischargedPostconditionDetail>),
    /// The unique source `main` declaration has a header shape FN-7 admits in
    /// neither entry form.
    InvalidMain,
    /// No source `main` declaration exists.
    MissingMain,
    /// A declaration other than the unit's entry carries a `program_kind`.
    NonEntryProgramKind {
        /// Function that declared the program kind.
        function: String,
    },
    /// A standard-input label is unknown, repeated, out of table-ordinal
    /// order, or carries a foreign kind prefix.
    InvalidStandardInputLabel {
        /// Complete written label spelling.
        label: String,
        /// The kind's closed standard-input labels in table-ordinal order.
        declared_labels: Vec<String>,
    },
    /// An `input_label` was written outside a kind-declaring entry's own
    /// parameters, including in a `fn_sig`.
    StandardInputLabelOutsideEntry {
        /// Complete written label spelling.
        label: String,
    },
    /// A selected standard input's written mode and type differ from its row.
    InvalidStandardInput {
        /// Complete written label spelling.
        label: String,
        /// The row's exact written mode and type.
        declared: &'static str,
    },
    /// A kind-declaring entry declared a value parameter with no
    /// `input_label`.
    UnlabelledEntryParameter {
        /// Binder spelling of the unlabelled parameter.
        parameter: String,
    },
    /// The entry's written result differs from its form's fixed result.
    InvalidEntryResult {
        /// The form's exact written result.
        required: &'static str,
    },
    /// The entry's written effect row is inadmissible for its form.
    InvalidEntryEffects {
        /// The rows or categories the entry's form admits.
        admitted: &'static str,
    },
    /// A source `call` named the kind-declaring entry, which only program
    /// start invokes.
    CallToKindDeclaringEntry {
        /// Entry spelling written at the call site.
        entry: String,
    },
    /// Named user-call arguments differ from the parameter list.
    InvalidNamedArguments {
        /// Callee spelling at the call site.
        callee: String,
        /// Exact declared parameter names in their required order.
        declared_parameters: Vec<String>,
    },
    /// Two fields in one owner-local table have the same label.
    DuplicateFieldLabel {
        /// Repeated field label.
        label: String,
    },
    /// Construction fields differ from the constructor's declared table.
    InvalidConstructionFields {
        /// Constructor named at the failing site.
        constructor: String,
        /// Exact declared field labels in their required order.
        declared_fields: Vec<String>,
    },
    /// Match binders differ from the variant's declared field table.
    InvalidMatchFields {
        /// Variant named by the arm.
        variant: String,
        /// Exact declared field labels in their required order.
        declared_fields: Vec<String>,
    },
    /// A match arm names a variant belonging to a different enum.
    ForeignMatchVariant,
    /// A match omits one or more declared variants.
    NonExhaustiveMatch {
        /// Declared variants with no arm, in declaration order.
        missing_variants: Vec<String>,
    },
    /// A propagation operand or enclosing result has the wrong Result shape.
    InvalidPropagation,
    /// `give` is absent, misplaced, duplicated, or followed by a statement.
    InvalidGive,
    /// The effect row is not a valid exact EFF-1 row.
    InvalidEffectRow {
        /// Which EFF-1 condition this row failed.
        reason: &'static str,
        /// Exact repair required by EFF-1 for that condition.
        mechanical_fix: &'static str,
    },
    /// The written effect row differs from syntactically exhibited effects.
    EffectMismatch {
        /// The row the body exhibits, in EFF-1 canonical spelling. This is
        /// exactly what the declaration must say.
        expected_row: String,
        /// The row the declaration writes, in the same spelling.
        found_row: String,
        /// Exhibited categories and paths the declaration does not carry.
        missing: Vec<String>,
        /// Declared categories and paths the body does not exhibit.
        extra: Vec<String>,
        /// Exact restructuring required by EFF-2.
        mechanical_fix: &'static str,
    },
    /// The written effect row omits a category contributed only by a
    /// compiler-derived release, which has no source occurrence [EFF-2].
    ReleaseEffectMismatch {
        /// The parameter or binding whose release contributed the category.
        owner: String,
        /// Exact restructuring required by EFF-2.
        mechanical_fix: &'static str,
    },
    /// A source contract carried the syntactically admitted generic list.
    GenericContract,
    /// Two members of one source contract have the same name.
    DuplicateContractMember {
        /// Repeated member name.
        member: String,
    },
    /// A conformance subject is not one concrete type.
    NonConcreteConformanceSubject,
    /// A conformance named a prelude marker instead of a source contract.
    InvalidConformanceContract,
    /// A conformance supplied contract arguments.
    ConformanceContractArguments,
    /// A later conformance repeated an exact `(type, contract)` key.
    DuplicateConformance,
    /// A conformance binding did not exactly match the next contract member.
    InvalidConformanceBinding {
        /// Member required at this source position, if one remains.
        expected_member: Option<String>,
    },
    /// A conformance ended before binding every contract member.
    MissingConformanceBinding {
        /// First member with no binding.
        member: String,
    },
    /// A bound function was generic, had requirements, or had a different signature.
    IncompatibleConformanceFunction,
    /// A generic type parameter named a source contract as its bound.
    SourceContractGenericBound,
    /// A law declaration does not match FN-4's closed declaration table.
    InvalidContractLaw,
    /// A valid law declaration cannot be discharged for one conformance.
    UndischargedContractLaw,
}

/// One deterministic post-resolution source-language rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticIssue {
    rule: SemanticRule,
    location: SemanticLocation,
    kind: SemanticIssueKind,
}

impl SemanticIssue {
    /// Returns the exact numbered rule established by this rejection.
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        self.rule.id()
    }

    /// Returns the exact numbered rule established by this issue.
    #[must_use]
    #[cfg(test)]
    pub const fn rule(&self) -> SemanticRule {
        self.rule
    }

    /// Returns the exact DIAG-1 semantic location.
    ///
    /// The driver reads it to quote the offending source line, so a semantic
    /// rejection names a file and a line rather than a `SourceId` and a byte.
    #[must_use]
    pub const fn location(&self) -> &SemanticLocation {
        &self.location
    }

    /// Returns the structured rejection premise.
    #[must_use]
    #[cfg(test)]
    pub const fn kind(&self) -> &SemanticIssueKind {
        &self.kind
    }
}

/// A language family that the current compiler has not implemented yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedSemanticFeature {
    /// Type, const, or region polymorphism.
    Generics,
    /// Nongeneric PRE-1 enum types and constructors outside Bool.
    PreludeNominalValues,
    /// A borrow form outside the implemented lexical buffer-borrow family.
    RegionsAndBorrows,
    /// Composite types or values outside the implemented nominal-data family.
    CompositeValues,
    /// A loop with no structurally reachable break exit for current SSA lowering.
    StructuredControlFlow,
    /// A recursive nominal layout whose finite representation is not selected.
    RecursiveNominalLayout,
    /// Moving an affine referent out of owning indirection has no selected cleanup semantics.
    BoxReferentMove,
    /// An ownership-state join not yet covered by the selected finite rule.
    OwnershipJoin,
    /// Repeated match arms, whose meaning the active specification does not select.
    DuplicateMatchArm,
    /// An OP-1 family outside the implemented scalar and nominal-tag families.
    OperationFamily,
    /// Arena values at runtime: the region-tied allocation and release
    /// lowering [STOR-2, STOR-3] is not implemented yet, so a checked
    /// function that would carry an arena value to execution stops here.
    ArenaRuntime,
}

/// Exact source node at which an unimplemented compiler family was required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticUnsupported {
    feature: UnsupportedSemanticFeature,
    node: NodePath,
}

impl SemanticUnsupported {
    /// Returns the unimplemented semantic family.
    #[must_use]
    #[cfg(test)]
    pub const fn feature(&self) -> UnsupportedSemanticFeature {
        self.feature
    }
}

/// Trusted semantic-checker invariant failure, never a source verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticCompilerFailure {
    /// Canonical production topology had an impossible local shape.
    InvalidCanonicalTree,
    /// A resolved declaration or use record was missing or inconsistent.
    InvalidResolution,
    /// Exact source bytes were not representable by the required semantic form.
    InvalidSourceEncoding,
    /// A dense identity or source-coordinate calculation overflowed.
    CounterOverflow,
}

/// Whole-unit semantic success and its only lowering authority.
#[derive(Debug)]
pub struct CheckedProgram<'classified, 'lexed, 'source> {
    pub(crate) _resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    pub(crate) data: CheckedProgramData,
}

impl CheckedProgram<'_, '_, '_> {
    /// Returns the number of checked source functions.
    #[must_use]
    #[cfg(test)]
    pub fn function_count(&self) -> usize {
        self.data.functions.len()
    }

    /// Returns the exact source name of the checked entry function.
    #[must_use]
    #[cfg(test)]
    pub fn entry_function_name(&self) -> &str {
        self.data
            .functions
            .get(self.data.main.0 as usize)
            .map_or("", |function| function.name.as_str())
    }
}

/// Failure-atomic result of target-independent semantic checking.
#[derive(Debug)]
pub enum SemanticOutcome<'classified, 'lexed, 'source> {
    /// Every applicable whole-unit judgment succeeded.
    Complete(Box<CheckedProgram<'classified, 'lexed, 'source>>),
    /// A numbered language rule was violated.
    SourceIssue {
        /// Deterministically selected semantic issue.
        issue: SemanticIssue,
    },
    /// A delayed ensures-entry resolver issue selected only after FN-9 selector admission.
    ResolutionIssue {
        /// The original resolution issue, unchanged in rule, location, and payload.
        issue: ResolutionIssue,
    },
    /// Valid source requires a language family the compiler has not implemented.
    Unsupported {
        /// Exact unimplemented family and source node.
        unsupported: SemanticUnsupported,
    },
    /// Trusted compiler invariants failed.
    CompilerFailure {
        /// Internal failure class.
        failure: SemanticCompilerFailure,
    },
}

enum CheckStop {
    Issue(Box<SemanticIssue>),
    Resolution(Box<ResolutionIssue>),
    Unsupported(SemanticUnsupported),
    Compiler(SemanticCompilerFailure),
    /// A derived type named a nominal instance that is not interned yet.
    ///
    /// Function checking is `&self`, and every interning site reads a
    /// *written* type — a `box<T>` for [STOR-2], a `Result<T, E>` for the
    /// checked arithmetic rows. A derived type has no written form anywhere,
    /// so once the annotation is gone nothing interns it. This is the
    /// recoverable signal that closes that gap: the driver interns what is
    /// pending and checks the function again. It is private to the checker
    /// and never reaches a diagnostic.
    DeferredNominal,
    /// A throwaway FN-9 selector dependency whose ordinary source premise did
    /// not succeed. It must be consumed inside preflight and never becomes a
    /// source or compiler diagnostic of its own.
    PostconditionPrerequisiteUnavailable,
}

impl CheckStop {
    fn source_issue(issue: SemanticIssue) -> Self {
        Self::Issue(Box::new(issue))
    }
}

impl From<SemanticCompilerFailure> for CheckStop {
    fn from(value: SemanticCompilerFailure) -> Self {
        Self::Compiler(value)
    }
}
