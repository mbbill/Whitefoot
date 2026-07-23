//! Target-independent semantic checking for exact Whitefoot v0.12.
//!
//! This stage consumes complete lexical resolution and is the sole producer of
//! the private checked-program value that may later authorize lowering. A
//! language feature not implemented yet is reported as an unsupported compiler
//! capability, never as a source-language rejection.

mod check;
mod model;
mod tree;

#[cfg(test)]
mod tests;

use crate::{BundleSourceExtent, NodePath, ResolvedSyntaxUnit, SyntaxCoordinate};

pub use check::check_semantics_v0_12;

pub(crate) use model::{
    BindingId, CheckedBooleanOperation, CheckedDrop, CheckedEnumType, CheckedExpression,
    CheckedFunction, CheckedIntegerOperation, CheckedLoopId, CheckedMatchArm, CheckedNominalKind,
    CheckedProgramData, CheckedProjectedDrop, CheckedStatement, CheckedType, CheckedValue,
    NominalId, PropagationContext, TrapSite,
};

/// Numbered rule owning one post-resolution semantic rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRuleV0_12 {
    /// Numeric literal range or canonicality.
    Form7,
    /// Named-constant type and value formation.
    Const2,
    /// Exact mode/type agreement.
    Type5,
    /// Copy-place assignment target formation and writability.
    Set1,
    /// Copy-versus-affine use spelling.
    Own1,
    /// Loop-local region and move restrictions.
    Own11,
    /// Storage-class and affine replacement restrictions.
    Stor1,
    /// Operation-table row selection.
    Op1,
    /// Exact `own Bool` explicit-check condition.
    Op5,
    /// Function result, reachability, or completion.
    Fn1,
    /// Explicit generic-instantiation argument presence.
    Fn2,
    /// Closed-program `main` contract.
    Fn7,
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
}

impl SemanticRuleV0_12 {
    /// Returns the exact numbered rule spelling from kernel specification v0.12.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Form7 => "FORM-7",
            Self::Const2 => "CONST-2",
            Self::Type5 => "TYPE-5",
            Self::Set1 => "SET-1",
            Self::Own1 => "OWN-1",
            Self::Own11 => "OWN-11",
            Self::Stor1 => "STOR-1",
            Self::Op1 => "OP-1",
            Self::Op5 => "OP-5",
            Self::Fn1 => "FN-1",
            Self::Fn2 => "FN-2",
            Self::Fn7 => "FN-7",
            Self::Gram11 => "GRAM-11",
            Self::Gram8 => "GRAM-8",
            Self::Gram10 => "GRAM-10",
            Self::Type6 => "TYPE-6",
            Self::Err2 => "ERR-2",
            Self::Err3 => "ERR-3",
            Self::Give1 => "GIVE-1",
            Self::Eff1 => "EFF-1",
            Self::Eff2 => "EFF-2",
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

/// Structured reason for one semantic rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticIssueKind {
    /// A literal is not the unique in-range FORM-7 spelling.
    InvalidIntegerLiteral,
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
    /// A loop attempted to consume an affine binding declared outside it.
    MoveOuterBindingInLoop {
        /// Exact restructuring required by OWN-11.
        mechanical_fix: &'static str,
    },
    /// The selected operation family has no row for the written arguments.
    InvalidOperation,
    /// An explicit check condition is not exactly `own Bool`.
    InvalidCheckCondition,
    /// A return expression disagrees with the written function result.
    ReturnMismatch,
    /// A statement follows a structurally terminating statement.
    UnreachableStatement,
    /// The function body can reach its closing brace.
    FunctionFallthrough,
    /// The unique source `main` declaration has the wrong header or effect row.
    InvalidMain,
    /// No source `main` declaration exists.
    MissingMain,
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
}

/// One deterministic post-resolution source-language rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticIssue {
    rule: SemanticRuleV0_12,
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
    pub const fn rule(&self) -> SemanticRuleV0_12 {
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
pub enum UnsupportedSemanticFeatureV0_12 {
    /// Contracts or conformances.
    ContractsAndConformances,
    /// Type, const, or region polymorphism.
    Generics,
    /// Nongeneric PRE-1 enum types and constructors outside Bool.
    PreludeNominalValues,
    /// Borrow modes, region parameters, or local regions.
    RegionsAndBorrows,
    /// Composite types or values outside the implemented nominal-data family.
    CompositeValues,
    /// Float types, literals, or operations.
    FloatingPoint,
    /// Requires blocks.
    RequiresBlocks,
    /// A loop with no structurally reachable break exit for current SSA lowering.
    StructuredControlFlow,
    /// A recursive nominal layout whose finite representation is not selected.
    RecursiveNominalLayout,
    /// An ownership-state join not yet covered by the selected finite rule.
    OwnershipJoin,
    /// Repeated match arms, whose dispatch meaning v0.12 does not select.
    DuplicateMatchArm,
    /// An OP-1 family outside the implemented scalar and nominal-tag families.
    OperationFamily,
    /// An effect other than `pure` or `traps`.
    EffectFamily,
}

/// Exact source node at which an unimplemented compiler family was required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticUnsupported {
    feature: UnsupportedSemanticFeatureV0_12,
    node: NodePath,
}

impl SemanticUnsupported {
    /// Returns the unimplemented semantic family.
    #[must_use]
    #[cfg(test)]
    pub const fn feature(&self) -> UnsupportedSemanticFeatureV0_12 {
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
}

impl From<SemanticCompilerFailure> for CheckStop {
    fn from(value: SemanticCompilerFailure) -> Self {
        Self::Compiler(value)
    }
}
