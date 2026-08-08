//! Target-independent semantic checking for the active Whitefoot specification.
//!
//! This stage consumes complete lexical resolution and is the sole producer of
//! the private checked-program value that may later authorize lowering. A
//! language feature not implemented yet is reported as an unsupported compiler
//! capability, never as a source-language rejection.

mod check;
mod entailment;
mod model;
mod tree;

#[cfg(test)]
mod tests;

use crate::{BundleSourceExtent, NodePath, ResolvedSyntaxUnit, SyntaxCoordinate};

pub use check::check_semantics;

pub(crate) use model::{
    BindingId, CheckedArrayRoot, CheckedBooleanOperation, CheckedBufferRoot,
    CheckedBufferSetTarget, CheckedDrop, CheckedEntryForm, CheckedEnumType, CheckedExpression,
    CheckedFlatElement, CheckedFloatOperation, CheckedFunction, CheckedIntegerOperation,
    CheckedLoopId, CheckedMatchArm, CheckedMode, CheckedNominalKind, CheckedNumericType,
    CheckedParameter, CheckedProgramData, CheckedProjectedDrop, CheckedRuntimeTargetObligations,
    CheckedSetTarget, CheckedSliceRoot, CheckedSliceSource, CheckedStatement,
    CheckedTargetDomainObligation, CheckedType, CheckedValue, NominalId, PropagationContext,
    TrapSite,
};

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
    /// Borrow-free and region-free stored-content formation.
    Stor5,
    /// Operation-table row selection.
    Op1,
    /// Subscript bounds-obligation discharge and offset typing.
    Op4,
    /// Exact conversion-pair result classification.
    Op6,
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
    /// Closed-program `main` contract.
    Fn7,
    /// Restricted executable function-entry requirement prologue.
    Fn8,
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
    /// Named runtime claim formation and per-function name uniqueness.
    Clm1,
    /// Claim lifecycle: refutation rejection under the entailment fragment.
    Clm2,
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
            Self::Stor5 => "STOR-5",
            Self::Op1 => "OP-1",
            Self::Op4 => "OP-4",
            Self::Op6 => "OP-6",
            Self::Op5 => "OP-5",
            Self::Fn1 => "FN-1",
            Self::Fn2 => "FN-2",
            Self::Fn3 => "FN-3",
            Self::Fn4 => "FN-4",
            Self::Fn7 => "FN-7",
            Self::Fn8 => "FN-8",
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
            Self::Clm1 => "CLM-1",
            Self::Clm2 => "CLM-2",
        }
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
            Self::Const1 => 12,
            Self::Const2 => 13,
            Self::Own1 => 14,
            Self::Own4 => 15,
            Self::Own5 => 16,
            Self::Own6 => 17,
            Self::Own10 => 18,
            Self::Own11 => 19,
            Self::Own12 => 20,
            Self::Own14 => 21,
            Self::Stor1 => 22,
            Self::Stor5 => 23,
            Self::Op1 => 24,
            Self::Op4 => 25,
            Self::Op5 => 26,
            Self::Op6 => 27,
            Self::Fn1 => 28,
            Self::Fn2 => 29,
            Self::Fn3 => 30,
            Self::Fn4 => 31,
            Self::Fn7 => 32,
            Self::Fn8 => 33,
            Self::Eff1 => 34,
            Self::Eff2 => 35,
            Self::Err2 => 36,
            Self::Err3 => 37,
            Self::Clm1 => 38,
            Self::Clm2 => 39,
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
    InvalidChildReborrow,
    /// A written reborrow form occurred outside OWN-14's admitted positions,
    /// or a return-position reborrow failed OWN-14's admission.
    InvalidReborrowPosition {
        /// Exact restructuring required by OWN-14.
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
    /// An explicit check condition is not exactly `own Bool`.
    InvalidCheckCondition,
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
    /// The fact state at a claim derives the exact negation of its predicate
    /// [CLM-2].
    RefutedClaim(Box<RefutedClaimDetail>),
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
    /// A stored-content position contains a region-bearing value.
    RegionBearingStorage {
        /// Required STOR-5 restructuring.
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
    /// A requirement entry uses a construct outside the FN-8 prologue subset.
    InvalidRequires,
    /// The unique source `main` declaration has a header shape FN-7 admits in
    /// neither entry form.
    InvalidMain,
    /// No source `main` declaration exists.
    MissingMain,
    /// A `program_kind` IDENT equals no row of FN-7's closed kind table.
    InvalidProgramKind {
        /// Written kind IDENT.
        kind: String,
        /// Kinds for which FN-7 defines an entry form in this version.
        admitted_kinds: Vec<String>,
    },
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
    InvalidEffectRow,
    /// The written effect row differs from syntactically exhibited effects.
    EffectMismatch,
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
    #[must_use]
    #[cfg(test)]
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

    /// Returns the [FN-7] entry form the checker admitted for this unit.
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn entry_form(&self) -> &CheckedEntryForm {
        &self.data.entry
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
    Issue(SemanticIssue),
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
}

impl From<SemanticCompilerFailure> for CheckStop {
    fn from(value: SemanticCompilerFailure) -> Self {
        Self::Compiler(value)
    }
}
