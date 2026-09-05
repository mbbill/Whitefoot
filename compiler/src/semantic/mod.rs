//! Target-independent semantic checking for the active Whitefoot specification.
//!
//! This stage consumes complete lexical resolution and is the sole producer of
//! the private checked-program value that may later authorize lowering. A
//! language feature not implemented yet is reported as an unsupported compiler
//! capability, never as a source-language rejection.

mod check;
mod entailment;
mod goal;
mod kernel;
mod loop_permission;
mod model;
pub(crate) mod permission;
mod permission_ledger;
mod places;
mod postcondition;
mod staged_permission;
mod target_action;
mod tree;

#[cfg(test)]
mod tests;

use crate::{BundleSourceExtent, NodePath, ResolutionIssue, ResolvedSyntaxUnit, SyntaxCoordinate};

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
    BindingId, CheckedArrayRoot, CheckedArraySetTarget, CheckedBodyDisposition,
    CheckedBooleanOperation, CheckedBufferRoot, CheckedBufferSetTarget, CheckedCommitValues,
    CheckedConst, CheckedConstructor, CheckedContainerRoot, CheckedDrop, CheckedElement,
    CheckedEntryForm, CheckedEnumType, CheckedExpression, CheckedFlatElement,
    CheckedFloatOperation, CheckedFunction, CheckedIntegerOperation, CheckedKernelInstance,
    CheckedLayoutCeiling, CheckedLayoutMagnitude, CheckedLoopId, CheckedMatchArm, CheckedMeasure,
    CheckedMode, CheckedNominalKind, CheckedNumericType, CheckedParameter, CheckedPlaceStep,
    CheckedProgramData, CheckedProjectedDrop, CheckedReleaseClass, CheckedRunSetTarget,
    CheckedRuntimeTargetObligations, CheckedSetTarget, CheckedSliceRoot, CheckedSliceSetTarget,
    CheckedSliceSource, CheckedStatement, CheckedTargetDomainObligation, CheckedType, CheckedValue,
    MeasureCell, MeasuredKind, NominalId, PropagationContext,
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
    /// Canonical region spelling: which positions write a REGIONID.
    Form8,
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
    /// Join-checked liveness: every predecessor of a join agrees on a
    /// binding's live-or-dead status.
    Liv1,
    /// The one `set` commit: the read-out, the three admission conditions,
    /// and the simultaneous reinitialization of every target.
    Liv2,
    /// A store's identity is a region: brand resolution at every elided
    /// store-region position, and the one reserving occurrence per region.
    Prov1,
    /// Linearity read against the scope: the release graph, the `linear`
    /// modifier, `dispose`, the destructuring consume, the partial-consume
    /// refusal, and the linearity bound on a generic parameter.
    Prov6,
    /// The one compiler-owned kernel declaration domain: row resolution, the
    /// per-row written-argument judgment, and the per-row requirement
    /// discharge.
    Blk0,
    /// The two runs, the one window, and what a slot may hold.
    Blk1,
    /// Formation and reservation: where a reserving occurrence may stand.
    Blk2,
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
    /// Contract vocabulary, the result ordinal, the routes, and where the
    /// relations land.
    Call4,
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
    /// Counted endpoint admission to the closed term-or-constant vocabulary.
    Ent2,
    /// One denotation per operand position, keyed on the parameter's mode.
    Msr3,
    /// Publication: where a declared relation is instantiated, where it is
    /// established, and that a published relation set is consistent.
    Call6,
    /// Proof-only loop invariant formation.
    Inv1,
    /// Finite source-written affine proof formation and checking.
    Prf1,
}

impl SemanticRule {
    /// Returns the exact numbered rule spelling from the active kernel specification.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Form5 => "FORM-5",
            Self::Form7 => "FORM-7",
            Self::Form8 => "FORM-8",
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
            Self::Liv1 => "LIV-1",
            Self::Liv2 => "LIV-2",
            Self::Prov1 => "PROV-1",
            Self::Prov6 => "PROV-6",
            Self::Blk0 => "BLK-0",
            Self::Blk1 => "BLK-1",
            Self::Blk2 => "BLK-2",
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
            Self::Call4 => "CALL-4",
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
            Self::Ent2 => "ENT-2",
            Self::Msr3 => "MSR-3",
            Self::Call6 => "CALL-6",
            Self::Inv1 => "INV-1",
            Self::Prf1 => "PRF-1",
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
            Self::Form7 => Self::Form8,
            Self::Form8 => Self::Gram6,
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
            Self::Own14 => Self::Liv1,
            Self::Liv1 => Self::Liv2,
            Self::Liv2 => Self::Prov1,
            Self::Prov1 => Self::Prov6,
            Self::Prov6 => Self::Blk0,
            Self::Blk0 => Self::Blk1,
            Self::Blk1 => Self::Blk2,
            Self::Blk2 => Self::Stor1,
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
            Self::Fn9 => Self::Call4,
            Self::Call4 => Self::Eff1,
            Self::Eff1 => Self::Eff2,
            Self::Eff2 => Self::Err2,
            Self::Err2 => Self::Err3,
            Self::Err3 => Self::Sys2,
            Self::Sys2 => Self::Sys8,
            Self::Sys8 => Self::Ent2,
            Self::Ent2 => Self::Msr3,
            Self::Msr3 => Self::Call6,
            Self::Call6 => Self::Inv1,
            Self::Inv1 => Self::Prf1,
            Self::Prf1 => return None,
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
            Self::Form8 => 2,
            Self::Gram6 => 3,
            Self::Give1 => 4,
            Self::Gram8 => 5,
            Self::Gram10 => 6,
            Self::Gram11 => 7,
            Self::Type2 => 8,
            Self::Type5 => 9,
            Self::Type6 => 10,
            Self::Type7 => 11,
            Self::Set1 => 12,
            Self::Set2 => 13,
            Self::Const1 => 14,
            Self::Const2 => 15,
            Self::Own1 => 16,
            Self::Own4 => 17,
            Self::Own5 => 18,
            Self::Own6 => 19,
            Self::Own10 => 20,
            Self::Own11 => 21,
            Self::Own12 => 22,
            Self::Own14 => 23,
            Self::Liv1 => 24,
            Self::Liv2 => 25,
            Self::Prov1 => 26,
            Self::Prov6 => 27,
            Self::Blk0 => 28,
            Self::Blk1 => 29,
            Self::Blk2 => 30,
            Self::Stor1 => 31,
            Self::Stor4 => 32,
            Self::Stor5 => 33,
            Self::Op1 => 34,
            Self::Op2 => 35,
            Self::Op4 => 36,
            Self::Op5 => 37,
            Self::Op6 => 38,
            Self::Op9 => 39,
            Self::Fn1 => 40,
            Self::Fn2 => 41,
            Self::Fn3 => 42,
            Self::Fn4 => 43,
            Self::Fn6 => 44,
            Self::Fn7 => 45,
            Self::Fn8 => 46,
            Self::Fn9 => 47,
            Self::Call4 => 48,
            Self::Eff1 => 49,
            Self::Eff2 => 50,
            Self::Err2 => 51,
            Self::Err3 => 52,
            Self::Sys2 => 53,
            Self::Sys8 => 54,
            Self::Ent2 => 55,
            Self::Msr3 => 56,
            Self::Call6 => 57,
            Self::Inv1 => 58,
            Self::Prf1 => 59,
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

/// One non-discharged static source obligation disposition [ENT-6].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticObligationDisposition {
    /// The closed state derives the canonical goal or normalization false.
    Refuted,
    /// The closed state derives neither a successful nor a refuting route.
    Unproved,
}

/// Which INV-1 induction obligation a source loop invariant failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopInvariantProofObligation {
    /// The invariant did not follow from the loop preheader facts at the first
    /// loop header.
    Base,
    /// Some reachable normal body fallthrough did not preserve the invariant
    /// at the next loop header. A counted loop includes its hidden unit binder
    /// update in this transition.
    Backedge,
}

/// The first failed part of one erased local invariant certificate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceProofObligation {
    /// Zero-based `use` occurrence in source order.
    Premise(u32),
    /// The written weighted sum plus the fixed direct residual rule did not
    /// establish the invariant target.
    Combination,
    /// AUTO already established the target, so the entire written `use` block
    /// is forbidden redundant proof text in this specification version.
    RedundantUseBlock,
    /// Two normalized `use` relations are identical; one explicitly scaled
    /// use must express their combined contribution.
    RepeatedUse { first: u32, repeated: u32 },
    /// The source list exceeds the fixed structural use capacity.
    UseCapacity { maximum: u32, actual: u32 },
    /// A written proof-domain factor or source-order accumulated certificate
    /// exceeded the admitted i128 arithmetic.
    CertificateArithmeticOverflow,
    /// The accumulated certificate exceeded a fixed affine shape capacity.
    CertificateFormationCapacity,
    /// A nonpositive factor reached the certificate core. Canonical source
    /// checking normally rejects this before entailment.
    InvalidUseFactor { use_index: u32 },
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

/// The deterministic [BLK-0] kernel-row rejection payload.
///
/// A kernel-domain row is a compiler-owned declaration record and has no
/// source node, so the payload names the operation and the position of the
/// requirement in that row's own declared requirement list rather than a
/// `requires_clause` occurrence. This is the same shape an [OP-1] diagnostic
/// takes, which names its family rather than a declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndischargedKernelRequirementDetail {
    /// The row's exact IDENT spelling [BLK-0].
    pub operation: &'static str,
    /// The row's zero-based `container_declaration_ordinal` [BLK-0].
    pub operation_ordinal: u8,
    /// This requirement's position in the row's declared requirement list.
    pub requirement: u32,
    /// Stable structural rendering of the complete instantiated typed goal.
    pub instantiated_goal: String,
    /// The exact non-discharged disposition.
    pub disposition: CallRequirementDisposition,
    /// The rule-selected mechanical restructuring.
    pub mechanical_fix: &'static str,
}

/// One non-discharged [FN-9] relation disposition.
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
    /// The exact non-discharged disposition.
    pub disposition: PostconditionProofDisposition,
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
    /// never a runtime fallback and never enters EFF-2's effect relation.
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
    TypeMismatch {
        /// The exact type, mode, or written form the position requires.
        expected: String,
        /// The exact type, mode, or written form found there.
        found: String,
    },
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
    /// [PROV-6] a value linear in this scope is live on an edge leaving it,
    /// where no compiler-derived release exists to carry it.
    LinearValueNotConsumed {
        /// The binding whose value is linear here.
        binding: String,
        /// The nominal whose `linear` declaration created the obligation.
        obligation: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6] a consume of a proper sub-place of a value linear in this
    /// scope, with no commit reinitializing that sub-place.
    LinearValuePartiallyConsumed {
        /// The nominal whose `linear` declaration created the obligation.
        obligation: String,
        /// The residual the consume would abandon.
        residual: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6] the `linear` modifier on a nominal [OWN-1] classifies as copy.
    LinearModifierOnCopyNominal {
        /// The marked nominal.
        nominal: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6] a `dispose` whose operand type reaches no capability-released
    /// leaf, so the walk would reclaim nothing.
    DisposeWithoutCapabilityLeaf {
        /// The operand's exact type.
        ty: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6] a `dispose` one of whose release-graph nodes carries the
    /// `linear` modifier.
    DisposeOfLinearNode {
        /// The marked nominal reached by the walk.
        nominal: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6] a `dispose` whose operand is or reaches a view, which owns
    /// nothing.
    DisposeOfLoanBearingOperand {
        /// The operand's exact type.
        ty: String,
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [BLK-2] a reservation whose written store region is not one an
    /// enclosing `region_stmt` of this function introduced, or whose
    /// occurrence is not a statement of that region block and of no loop
    /// inside it.
    ReservationPlacement {
        /// The written store region.
        region: String,
        /// Exact restructuring required by BLK-2.
        mechanical_fix: &'static str,
    },
    /// [PROV-1] a second reserving occurrence naming a region an earlier one
    /// already named.
    SecondStoreInOneRegion {
        /// The written store region.
        region: String,
        /// Exact restructuring required by PROV-1.
        mechanical_fix: &'static str,
    },
    /// [PROV-6, S37] a region parameter written `'s: copy`, which is not one
    /// of the two classes a store has.
    InvalidRegionBound {
        /// Exact restructuring required by PROV-6.
        mechanical_fix: &'static str,
    },
    /// [PROV-6, S37] an instantiation whose argument's linearity class does not
    /// satisfy the parameter's written bound.
    LinearityBoundMismatch {
        /// The bounded parameter's written spelling.
        parameter: String,
        /// The written bound.
        bound: &'static str,
        /// The written argument.
        argument: String,
        /// The argument's actual class.
        actual: &'static str,
    },
    /// [LIV-1] two predecessors of one join disagree about whether a binding
    /// is live there.
    LivenessJoinDisagreement {
        /// The binding whose status the predecessors disagree about.
        binding: String,
        /// The predecessor that reaches the join with the binding live.
        live_predecessor: String,
        /// The predecessor that reaches the join with the binding dead.
        dead_predecessor: String,
        /// Exact restructuring required by LIV-1.
        mechanical_fix: &'static str,
    },
    /// [LIV-2] two targets of one commit overlap, so the commit order would
    /// decide the result.
    OverlappingCommitTargets {
        /// The earlier written target place.
        first: String,
        /// The later written target place, where the rejection is located.
        second: String,
        /// Exact restructuring required by LIV-2.
        mechanical_fix: &'static str,
    },
    /// [LIV-2] an affine commit target's final selected type is region-bearing,
    /// which no commit reinitializes.
    RegionBearingCommitTarget {
        /// Exact selected type.
        target_type: String,
        /// Exact restructuring required by LIV-2.
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
    /// [BLK-1] a `construct` named one of the four compiler-owned container or
    /// provider nominals. No construct produces a run, a provider, or a
    /// store: each contributes a constructor entry that exists to be refused.
    ContainerConstruction {
        /// The nominal the construct named.
        nominal: String,
        /// Exact restructuring required by BLK-1.
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
    InvalidBorrowLifetime {
        /// The region written where this borrow is created or stored, exactly
        /// as the source spells it.
        region: String,
        /// The binding whose storage the borrow views, exactly as the source
        /// spells it.
        binder: String,
        /// Where a region this borrow can name must be introduced.
        mechanical_fix: String,
    },
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
    /// [OWN-11] a loop body left a binding declared outside it in a
    /// different live-or-dead status than the entering edge did, so one
    /// iteration would start in a state the previous one did not leave.
    MoveOuterBindingInLoop {
        /// The outer binding whose status the backedge changed.
        binding: String,
        /// Exact restructuring required by OWN-11.
        mechanical_fix: &'static str,
    },
    /// A borrow created in a loop names a region introduced outside that loop.
    BorrowRegionOutsideLoop {
        /// Exact restructuring required by OWN-11.
        mechanical_fix: &'static str,
    },
    /// A region is spelled at a position [FORM-8] does not spell that way:
    /// written where the surrounding text already fixes it, or absent where
    /// nothing fixes it.
    RegionSpelling {
        /// Exact mechanical repair selected by FORM-8.
        mechanical_fix: &'static str,
    },
    /// The selected operation family has no row for the written arguments.
    InvalidOperation,
    /// A contract predicate is not exactly `own Bool`.
    InvalidPredicateCondition,
    /// A conditional was written in a form GRAM-6 does not admit for its
    /// class: a Bool-scrutinee `match`, an empty `else`, or an `else` block
    /// holding exactly one `if`.
    InvalidConditionalForm {
        /// Exact mechanical repair selected by GRAM-6.
        mechanical_fix: &'static str,
    },
    /// A subscript's bounds obligation is not derivable from the closed fact
    /// state at its node [OP-4, ENT-6].
    UndischargedBoundsObligation {
        /// The exact ENT-6 residual rendering: offset atom, ` < len_of(`, base
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
        /// The exact non-discharged disposition.
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
    /// One undischarged [BLK-0] kernel-row requirement at a call to that row.
    UndischargedKernelRequirement(Box<UndischargedKernelRequirementDetail>),
    /// A counted endpoint produced `own u64` but was not itself one preceding
    /// ENT-2 term or constant.
    InvalidCountedEndpoint {
        /// The exact restructuring required by ENT-2.
        mechanical_fix: &'static str,
    },
    /// An unlabeled break has no enclosing structural loop target [GRAM-4,
    /// FN-1].
    BreakOutsideLoop {
        /// The exact source-level restructuring required by GRAM-4.
        mechanical_fix: &'static str,
    },
    /// A header or local invariant violates INV-1 name or target formation.
    InvalidInvariant {
        reason: &'static str,
        mechanical_fix: &'static str,
    },
    /// A well-formed source loop invariant failed one of INV-1's two mandatory
    /// induction judgments in the source fact context.
    UndischargedLoopInvariant {
        /// Source spelling of the invariant name.
        name: String,
        /// The failed induction obligation, selected in proof order.
        obligation: LoopInvariantProofObligation,
        /// The exact source-language relation the failed incoming edge had to
        /// establish. A counted-loop backedge renders the hidden next binder
        /// as `i + 1_u64`; no checker-private term identity is exposed.
        required_relation: String,
        /// Exact source-level repair selected by INV-1.
        mechanical_fix: &'static str,
    },
    /// A well-formed blockless local invariant target is not established by
    /// the specification-defined AUTO family in its entering context.
    UndischargedLocalInvariant {
        /// Source spelling of the invariant name.
        name: String,
        /// Exact source-level repair selected by INV-1.
        mechanical_fix: &'static str,
    },
    /// A `proof_use` relation or certificate factor violates the closed
    /// PRF-1 source form.
    InvalidSourceProof {
        reason: &'static str,
        mechanical_fix: &'static str,
    },
    /// A well-formed local invariant failed in the source fact context.
    /// Written uses are independent entering-context premises; their explicit
    /// weighted sum may establish a target weakened by the fixed direct rule.
    UndischargedSourceProof {
        name: String,
        obligation: SourceProofObligation,
        mechanical_fix: &'static str,
    },
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
    /// [CALL-4] a route omits its ordinal binder where two or more declared
    /// result ordinals could carry it.
    AmbiguousResultRoute,
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
    /// [MSR-3] an `ensures` names a measure of a `&uniq` state parameter,
    /// which denotes no state a source-declared callee can name.
    InadmissibleStateParameterMeasure {
        /// The written `&uniq` parameter whose measure the clause names.
        parameter: String,
        /// The restructuring this clause needs.
        mechanical_fix: &'static str,
    },
    /// [CALL-6] the relations one contract publishes are contradictory at
    /// their establishment point, so every goal a caller submits would
    /// discharge from that contradiction alone.
    ContradictoryPublishedRelations {
        /// The two clause relations whose conjunction is unsatisfiable, as
        /// rendered from the declared templates.
        relations: Vec<String>,
        /// The restructuring this contract needs.
        mechanical_fix: &'static str,
    },
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

/// A written-argument count and the noun it agrees with, as `1 written type
/// argument` or `2 written type arguments`.
///
/// A diagnostic a writer reads is prose, and "1 written region arguments" is
/// the kind of sentence that makes a reader doubt the rest of it.
pub(crate) fn written_count(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} written {noun}")
    } else {
        format!("{count} written {noun}s")
    }
}

impl SemanticIssueKind {
    /// One [TYPE-5] disagreement, in the spellings the source uses.
    ///
    /// The rejection published neither side for four blind-writer rounds: a
    /// writer was told two types disagree and had to work out which two. Both
    /// are always in hand at the judgment, so both are published. Where the
    /// disagreement is about the written form rather than two types — a
    /// generic form written with no type arguments, a `move` where a place is
    /// required — each side states that form.
    #[must_use]
    pub(crate) fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }
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
    /// Container and provider values at runtime [TYPE-2, BLK-1, PROV-1]. The
    /// four compiler-owned nominals are named, branded, confined and
    /// measured by the ordinary source judgments, and the window lowering the
    /// nine [BLK-0] rows need is not implemented yet, so a checked function
    /// that would carry one of those values to execution stops here rather
    /// than lowering wrong code.
    ContainerRuntime,
    /// An exclusive view over an `array<T, N>` [VIEW-1, VIEW-2]. An array is
    /// a value with no stable address in this lowering — an element commit
    /// rebuilds the whole array and writes it back to its binding — so the
    /// descriptor a view of one carries points at a snapshot, and a write
    /// through that view would reach the snapshot and not the array. The
    /// shared view is unaffected, because a live shared loan already refuses
    /// every write to its origin [OWN-5]; only the exclusive one stops here,
    /// and it stops rather than lowering a write nobody can observe.
    ExclusiveViewOverArray,
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
