//! Exact-v0.15 declaration collection and lexical name resolution.
//!
//! Resolution consumes canonical syntax, builds the specification-defined
//! scope and declaration inventories, and fixes every lexical target. Typed
//! owner/member relationships remain explicit dependent records for the next
//! compiler stage.

mod catalog;
mod engine;
mod scopes;

#[cfg(test)]
mod tests;

use crate::{CanonicalSyntaxUnit, NodePath, SyntaxCoordinate};

pub use engine::resolve_v0_15;

/// Returns the exact OP-1 spelling of a resolved operation family.
#[must_use]
pub fn operation_family_spelling_v0_15(id: OperationFamilyId) -> Option<&'static str> {
    catalog::operation_spelling(id)
}

/// Dense identity of one resolver scope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(u32);

impl ScopeId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// One scope kind from the v0.15 scope-construction matrix.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScopeKind {
    /// The complete closed compilation unit.
    CompilationUnit,
    /// Type and const generics owned by one declaration.
    DeclarationGenerics,
    /// Region parameters, parameters, signature suffix, requires, and body.
    FunctionSignature,
    /// One contract-member signature.
    ContractSignature,
    /// The disjoint lexical block of a function's requires clause.
    RequiresBlock,
    /// A concrete function body.
    FunctionBody,
    /// The statement body nested under an arm, loop, or local region.
    NestedBody,
    /// Match binders visible to one arm body.
    Arm,
    /// One loop label visible only to that loop body.
    LoopLabel,
    /// One local region visible only to that region body.
    LocalRegion,
}

/// One resolver scope and its lexical parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeRecord {
    id: ScopeId,
    parent: Option<ScopeId>,
    kind: ScopeKind,
    owner: NodePath,
}

impl ScopeRecord {
    /// Returns this scope's dense identity in the resolved unit.
    #[must_use]
    pub const fn id(&self) -> ScopeId {
        self.id
    }

    /// Returns the immediately enclosing scope, if any.
    #[must_use]
    pub const fn parent(&self) -> Option<ScopeId> {
        self.parent
    }

    /// Returns the specification-defined scope kind.
    #[must_use]
    pub const fn kind(&self) -> ScopeKind {
        self.kind
    }

    /// Returns the production node that creates this scope.
    #[must_use]
    pub const fn owner(&self) -> &NodePath {
        &self.owner
    }
}

/// Dense identity of one source declaration event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeclarationId(u32);

impl DeclarationId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Dense identity of one normative PRE-1 declaration record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PreludeDeclarationId(u8);

impl PreludeDeclarationId {
    pub(crate) const fn new(index: u8) -> Self {
        Self(index)
    }

    /// Returns the zero-based PRE-1 declaration ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

/// Dense identity of one distinct OP-1 operation-family spelling.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationFamilyId(u16);

impl OperationFamilyId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u16::try_from(index).ok().map(Self)
    }

    /// Returns the operation family's OP-1 first-occurrence ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u16 {
        self.0
    }
}

/// Closed declaration-class order used by v0.15 resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeclarationClass {
    /// Top-level source function.
    Function,
    /// Top-level immutable named constant.
    NamedConst,
    /// Lexical const generic.
    ConstGeneric,
    /// Parameter, let binding, or match binder.
    Value,
    /// Lexical type generic.
    GenericType,
    /// Source or prelude nominal type.
    NominalType,
    /// Constructor contributed by a source struct.
    StructConstructor,
    /// Source or prelude enum variant.
    EnumVariant,
    /// Source or prelude contract.
    Contract,
    /// Region parameter or local region.
    Region,
    /// Loop label.
    Label,
    /// One distinct OP-1 spelling.
    OperationFamily,
}

/// Closed TYPE-6 collision-domain order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeclarationDomain {
    /// Functions, constants, const generics, parameters, lets, and binders.
    LexicalIdentifier,
    /// Generic and nominal types.
    NominalType,
    /// Struct constructors and enum variants.
    Constructor,
    /// Contracts.
    Contract,
    /// Region parameters and local regions.
    Region,
    /// Loop labels.
    Label,
}

impl DeclarationDomain {
    pub(crate) const fn ordinal(self) -> u8 {
        match self {
            Self::LexicalIdentifier => 0,
            Self::NominalType => 1,
            Self::Constructor => 2,
            Self::Contract => 3,
            Self::Region => 4,
            Self::Label => 5,
        }
    }
}

/// Source declaration roles D01 through D14.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationRole {
    /// D01: top-level function.
    Function,
    /// D02: source struct nominal plus constructor.
    Struct,
    /// D03: source enum nominal.
    Enum,
    /// D04: source enum variant.
    Variant,
    /// D05: source contract.
    Contract,
    /// D06: named constant.
    NamedConst,
    /// D07: type generic.
    GenericType,
    /// D08: const generic.
    ConstGeneric,
    /// D09: region parameter.
    RegionParameter,
    /// D10: function or contract-member parameter.
    Parameter,
    /// D11: ordinary lexical let binding.
    Let,
    /// D12: loop label.
    LoopLabel,
    /// D13: local region.
    LocalRegion,
    /// D14: match binder.
    MatchBinder,
}

/// Dependent declaration roles X01 through X03.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependentDeclarationRole {
    /// X01: source struct field.
    Field,
    /// X02: source enum-variant field.
    VariantField,
    /// X03: contract member signature.
    ContractMember,
}

/// Lexical-use roles U01 through U18.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LexicalUseRole {
    /// U01: nominal or generic type.
    Type,
    /// U02: type-generic contract bound.
    GenericBound,
    /// U03: conformance contract.
    ConformanceContract,
    /// U04: struct or enum construction.
    Construct,
    /// U05: enum-variant match arm.
    ArmVariant,
    /// U06: region carried by a type.
    TypeRegion,
    /// U07: region carried by a mode.
    ModeRegion,
    /// U08: explicit region type argument.
    TypeArgumentRegion,
    /// U09: region named in an effect.
    EffectRegion,
    /// U10: region named by a borrow expression.
    BorrowRegion,
    /// U11: break target.
    BreakLabel,
    /// U12: constant-expression identifier.
    Const,
    /// U13: constant-value identifier.
    ConstValue,
    /// U14: place base.
    PlaceBase,
    /// U15: identifier callee.
    IdentifierCallee,
    /// U16: dotted operation callee.
    OperationCallee,
    /// U17: concrete function bound to a contract member.
    FunctionBinding,
    /// U18: generic suffix in `0_T` or `1_T`.
    GenericNumericSuffix,
}

/// Deferred-use roles X04 through X09.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeferredUseRole {
    /// X04: construction field or named-call argument.
    FieldInitializer,
    /// X05: match field label.
    MatchField,
    /// X06: projected field.
    ProjectedField,
    /// X07: contract member side of a conformance binding.
    ContractBinding,
    /// X08: closed law name.
    LawName,
    /// X09: complete law argument.
    LawArgument,
}

/// Exact source origin of one resolver role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceOrigin {
    node: NodePath,
    coordinate: SyntaxCoordinate,
    role_ordinal: u32,
    subtoken_ordinal: u32,
}

impl SourceOrigin {
    /// Returns the production node owning this role.
    #[must_use]
    pub const fn node(&self) -> &NodePath {
        &self.node
    }

    /// Returns the exact source coordinate of the role spelling.
    #[must_use]
    pub const fn coordinate(&self) -> SyntaxCoordinate {
        self.coordinate
    }

    /// Returns the direct-carrier ordinal within the owner production.
    #[must_use]
    pub const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }

    /// Returns zero for a complete carrier or the embedded subtoken ordinal.
    #[must_use]
    pub const fn subtoken_ordinal(&self) -> u32 {
        self.subtoken_ordinal
    }
}

/// Origin of a declaration participating in a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationOrigin {
    /// One source declaration.
    Source(SourceOrigin),
    /// One normative PRE-1 record.
    Prelude(PreludeDeclarationId),
}

/// One source declaration event and its lookup entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationRecord {
    id: DeclarationId,
    role: DeclarationRole,
    spelling: String,
    origin: SourceOrigin,
    scope: ScopeId,
    classes: Vec<DeclarationClass>,
}

impl DeclarationRecord {
    /// Returns this declaration's dense identity.
    #[must_use]
    pub const fn id(&self) -> DeclarationId {
        self.id
    }

    /// Returns its exact grammar role.
    #[must_use]
    pub const fn role(&self) -> DeclarationRole {
        self.role
    }

    /// Returns the exact source spelling, including any sigil.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns its source origin.
    #[must_use]
    pub const fn origin(&self) -> &SourceOrigin {
        &self.origin
    }

    /// Returns the scope in which it is declared.
    #[must_use]
    pub const fn scope(&self) -> ScopeId {
        self.scope
    }

    /// Returns its one or two grammar-selected lookup classes.
    #[must_use]
    pub fn classes(&self) -> &[DeclarationClass] {
        &self.classes
    }
}

/// One typed-owner-dependent declaration retained for the next stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependentDeclarationRecord {
    role: DependentDeclarationRole,
    spelling: String,
    origin: SourceOrigin,
}

impl DependentDeclarationRecord {
    /// Returns the dependent role.
    #[must_use]
    pub const fn role(&self) -> DependentDeclarationRole {
        self.role
    }

    /// Returns its exact spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns its source origin.
    #[must_use]
    pub const fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
}

/// One successful lexical target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResolvedTarget {
    /// One class contributed by a source declaration event.
    Source {
        /// Source declaration.
        declaration: DeclarationId,
        /// Selected grammar class.
        class: DeclarationClass,
    },
    /// One normative PRE-1 lookup entry.
    Prelude(PreludeDeclarationId),
    /// One exact OP-1 operation family.
    Operation(OperationFamilyId),
}

/// One lexical use and its exact target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexicalUseRecord {
    role: LexicalUseRole,
    spelling: String,
    origin: SourceOrigin,
    target: ResolvedTarget,
}

impl LexicalUseRecord {
    /// Returns the grammar-selected use role.
    #[must_use]
    pub const fn role(&self) -> LexicalUseRole {
        self.role
    }

    /// Returns the complete name spelling, or the bare generic suffix.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the source origin.
    #[must_use]
    pub const fn origin(&self) -> &SourceOrigin {
        &self.origin
    }

    /// Returns the unique resolved target.
    #[must_use]
    pub const fn target(&self) -> ResolvedTarget {
        self.target
    }
}

/// One owner/member use deliberately deferred to typed checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeferredUseRecord {
    role: DeferredUseRole,
    spelling: String,
    origin: SourceOrigin,
}

impl DeferredUseRecord {
    /// Returns the deferred role.
    #[must_use]
    pub const fn role(&self) -> DeferredUseRole {
        self.role
    }

    /// Returns the complete carrier spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the source origin.
    #[must_use]
    pub const fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
}

/// One normative PRE-1 declaration record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreludeDeclarationRecord {
    id: PreludeDeclarationId,
    spelling: &'static str,
    class: Option<DeclarationClass>,
}

impl PreludeDeclarationRecord {
    /// Returns the PRE-1 record ordinal.
    #[must_use]
    pub const fn id(self) -> PreludeDeclarationId {
        self.id
    }

    /// Returns the normative spelling.
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        self.spelling
    }

    /// Returns the source-lookup class, or `None` for owner-local records.
    #[must_use]
    pub const fn lookup_class(self) -> Option<DeclarationClass> {
        self.class
    }
}

/// Numbered rule owning one resolver rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionRuleV0_15 {
    /// Reserved declaration name.
    Form3,
    /// Generic numeric suffix.
    Form5,
    /// Match-binder freshness.
    Gram10,
    /// Type or place lookup.
    Type5,
    /// Namespace collision, constructor, or label lookup.
    Type6,
    /// Constant-expression lookup.
    Const1,
    /// Constant-value lookup.
    Const2,
    /// Region uniqueness or lookup.
    Own3,
    /// Operation-family or callee lookup.
    Op1,
    /// Contract lookup.
    Fn3,
    /// Function binding lookup.
    Fn4,
    /// Requires-block structural admission.
    Fn8,
}

/// Which closed reserved-name set owns one spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservedNameClass {
    /// A distinct dotless OP-1 family.
    DotlessOperation,
    /// One FORM-3 operation-mode suffix word.
    ModeWord,
}

/// Declaration roles covered by OP-1's reserved-lower-name inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservedDeclarationRole {
    /// Top-level function.
    Function,
    /// Top-level named constant.
    NamedConst,
    /// Function or contract-member parameter.
    Parameter,
    /// Lexical let binding.
    Let,
    /// Match binder.
    MatchBinder,
    /// Struct field.
    Field,
    /// Enum-variant field.
    VariantField,
    /// Region parameter, using its unsigiled interior spelling.
    RegionParameter,
    /// Local region, using its unsigiled interior spelling.
    LocalRegion,
}

/// Why an FN-8 direct entry or block was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequiresShapeIssue {
    /// The block is empty or contains only ordinary lets.
    MissingFinalCheck,
    /// A direct entry is not an admitted nonfinal let or final check.
    InvalidEntry,
}

/// One declaration conflict carried by a TYPE-6 issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationConflict {
    domain: DeclarationDomain,
    class: DeclarationClass,
    origin: DeclarationOrigin,
}

impl DeclarationConflict {
    /// Returns the fixed TYPE-6 collision domain.
    #[must_use]
    pub const fn domain(&self) -> DeclarationDomain {
        self.domain
    }

    /// Returns the conflicting declaration class.
    #[must_use]
    pub const fn class(&self) -> DeclarationClass {
        self.class
    }

    /// Returns the conflicting declaration origin.
    #[must_use]
    pub const fn origin(&self) -> &DeclarationOrigin {
        &self.origin
    }
}

/// Structured payload of one deterministic resolver rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionIssueKind {
    /// Early FN-8 structural-admission failure.
    RequiresShape(RequiresShapeIssue),
    /// A declaration uses a derived reserved lower name.
    ReservedName {
        /// Unsigiled spelling for a region, otherwise the declaration spelling.
        spelling: String,
        /// Exact declaration role covered by the reserved-name rule.
        declaration_role: ReservedDeclarationRole,
        /// Owning reserved set.
        class: ReservedNameClass,
        /// Ordinal inside that reserved set.
        inventory_ordinal: u16,
    },
    /// A region spelling repeats within one function or member signature.
    RepeatedRegion {
        /// Complete sigiled spelling.
        spelling: String,
        /// Earlier declaration.
        conflicting: SourceOrigin,
    },
    /// GRAM-10 freshness failed before the binder became a declaration.
    MatchBinderFreshness {
        /// Binder spelling.
        spelling: String,
        /// Paired source field label.
        paired_field: String,
        /// Earlier equal binder in this arm, if any.
        earlier_binder: Option<SourceOrigin>,
        /// Live lexical-IDENT declarations at arm entry.
        arm_entry_conflicts: Vec<SourceOrigin>,
    },
    /// A PRE-1 collision, duplicate, redeclaration, or live shadow.
    DeclarationCollision {
        /// Offending spelling.
        spelling: String,
        /// Ordered nonempty conflicts.
        conflicts: Vec<DeclarationConflict>,
    },
    /// Admissible declarations exist in the candidate universe but are hidden.
    InvisibleUse {
        /// Use spelling.
        spelling: String,
        /// Use role.
        role: LexicalUseRole,
        /// Ordered admissible classes.
        admissible: Vec<DeclarationClass>,
        /// Ordered invisible declaration origins.
        origins: Vec<DeclarationOrigin>,
    },
    /// Labels with this spelling exist in the function but do not enclose use.
    NonEnclosingLabel {
        /// Complete label spelling.
        spelling: String,
        /// The label-use role, retained explicitly in the diagnostic payload.
        role: LexicalUseRole,
        /// Ordered current-function label origins.
        origins: Vec<DeclarationOrigin>,
    },
    /// No visible declaration in the admissible classes exists.
    UnresolvedUse {
        /// Use spelling.
        spelling: String,
        /// Use role.
        role: LexicalUseRole,
        /// Ordered admissible classes.
        admissible: Vec<DeclarationClass>,
        /// Visible exact-spelling classes in the candidate universe.
        available: Vec<DeclarationClass>,
    },
}

/// The first v0.15 resolver rejection in specified stage and event order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionIssue {
    rule: ResolutionRuleV0_15,
    origin: SourceOrigin,
    kind: ResolutionIssueKind,
}

impl ResolutionIssue {
    /// Returns the numbered rule owning this rejection.
    #[must_use]
    pub const fn rule(&self) -> ResolutionRuleV0_15 {
        self.rule
    }

    /// Returns the source node and coordinate selected by DIAG-1.
    #[must_use]
    pub const fn origin(&self) -> &SourceOrigin {
        &self.origin
    }

    /// Returns the complete structured diagnostic payload.
    #[must_use]
    pub const fn kind(&self) -> &ResolutionIssueKind {
        &self.kind
    }
}

/// Trusted resolver invariant failure, never a source-language rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionCompilerFailure {
    /// Canonical topology was inconsistent with its published invariants.
    InvalidCanonicalTree,
    /// A name-shaped grammar carrier was absent or classified more than once.
    UnclassifiedNameRole,
    /// A role-bearing token had an impossible grammar shape.
    InvalidRoleShape,
    /// A source name could not be represented as its required ASCII spelling.
    InvalidNameEncoding,
    /// Scope construction did not cover the complete tree.
    InvalidScopeTree,
    /// Successful lookup produced zero or multiple targets.
    AmbiguousResolution,
    /// A dense identity, ordinal, or coordinate calculation overflowed.
    CounterOverflow,
}

/// Canonical syntax plus complete v0.15 lexical resolution tables.
#[derive(Debug)]
pub struct ResolvedSyntaxUnit<'classified, 'lexed, 'source> {
    syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
    scopes: Vec<ScopeRecord>,
    prelude: Vec<PreludeDeclarationRecord>,
    declarations: Vec<DeclarationRecord>,
    dependent_declarations: Vec<DependentDeclarationRecord>,
    lexical_uses: Vec<LexicalUseRecord>,
    deferred_uses: Vec<DeferredUseRecord>,
}

impl<'classified, 'lexed, 'source> ResolvedSyntaxUnit<'classified, 'lexed, 'source> {
    /// Returns the source-bound canonical syntax consumed by this stage.
    #[must_use]
    pub const fn syntax(&self) -> &CanonicalSyntaxUnit<'classified, 'lexed, 'source> {
        &self.syntax
    }

    /// Returns the complete scope tree.
    #[must_use]
    pub fn scopes(&self) -> &[ScopeRecord] {
        &self.scopes
    }

    /// Returns all twenty-four PRE-1 records in normative preorder.
    #[must_use]
    pub fn prelude_declarations(&self) -> &[PreludeDeclarationRecord] {
        &self.prelude
    }

    /// Returns one PRE-1 record by its normative identity.
    #[must_use]
    pub fn prelude_declaration(
        &self,
        id: PreludeDeclarationId,
    ) -> Option<&PreludeDeclarationRecord> {
        self.prelude.get(usize::from(id.ordinal()))
    }

    /// Returns all source declaration events D01 through D14.
    #[must_use]
    pub fn declarations(&self) -> &[DeclarationRecord] {
        &self.declarations
    }

    /// Returns one source declaration by its resolved identity.
    #[must_use]
    pub fn declaration(&self, id: DeclarationId) -> Option<&DeclarationRecord> {
        self.declarations.get(id.index())
    }

    /// Returns dependent declarations X01 through X03.
    #[must_use]
    pub fn dependent_declarations(&self) -> &[DependentDeclarationRecord] {
        &self.dependent_declarations
    }

    /// Returns every successful lexical use U01 through U18.
    #[must_use]
    pub fn lexical_uses(&self) -> &[LexicalUseRecord] {
        &self.lexical_uses
    }

    /// Returns deferred owner/member uses X04 through X09.
    #[must_use]
    pub fn deferred_uses(&self) -> &[DeferredUseRecord] {
        &self.deferred_uses
    }

    /// Consumes resolution and returns the underlying canonical syntax.
    #[must_use]
    pub fn into_syntax(self) -> CanonicalSyntaxUnit<'classified, 'lexed, 'source> {
        self.syntax
    }
}

/// Failure-atomic outcome of exact-v0.15 lexical resolution.
#[derive(Debug)]
pub enum ResolutionOutcome<'classified, 'lexed, 'source> {
    /// The complete scope, declaration, lexical-use, and deferred-role tables.
    Complete(ResolvedSyntaxUnit<'classified, 'lexed, 'source>),
    /// The first spec-defined FN-8, inventory, or lookup rejection.
    SourceIssue {
        /// Canonical syntax retained for diagnostics or caller policy.
        syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
        /// Deterministic resolver issue.
        issue: ResolutionIssue,
    },
    /// A trusted compiler invariant failed.
    CompilerFailure {
        /// Canonical syntax retained for debugging.
        syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
        /// Internal failure class.
        failure: ResolutionCompilerFailure,
    },
}
