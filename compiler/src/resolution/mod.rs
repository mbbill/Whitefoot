//! Exact active-specification declaration collection and lexical name resolution.
//!
//! Resolution consumes canonical syntax, builds the specification-defined
//! scope and declaration inventories, and fixes every lexical target. Typed
//! owner/member relationships remain explicit dependent records for the next
//! compiler stage.

mod catalog;
mod engine;
mod kernel;
mod scopes;

#[cfg(test)]
mod tests;

use crate::{CanonicalSyntaxUnit, NodePath, SyntaxCoordinate};

pub use engine::{resolve, resolve_with_inventory};

pub use kernel::{
    CONTAINER_NOMINAL_CLASS, CONTAINER_NOMINAL_CLASSES, CONTAINER_NOMINALS, ContainerNominal,
    ContainerNominalId, ContainerShape, KERNEL_OPERATION_CLASS, KERNEL_OPERATIONS, KernelOperation,
    KernelOperationId, KernelRow, container_nominal, kernel_operation,
};

pub use catalog::{
    Inventory, OPEN_BY_NAME, SYSTEM_CONSTRUCTORS, SYSTEM_NOMINALS, SYSTEM_OPERATIONS,
    SystemConstructor, SystemEntity, SystemField, SystemIntegerResultBound, SystemNominal,
    SystemOperation, SystemParameter, SystemParameterMode, SystemRelease, SystemReleaseAction,
    SystemReleaseRow, SystemResourceBacking, SystemResourceContract, SystemResourceType,
    SystemResultPayload, SystemResultStateOrigin, SystemTypeRef, TRAVERSAL_SURFACE, TargetAction,
    TargetCompletion, TargetDispatch, TargetMilestones, operation_state_effects,
    system_constructor_declaration, system_constructor_index, system_constructors, system_entity,
    system_nominal_index, system_nominals, system_operation_index, system_operations,
    system_release_row, system_resource_contract,
};

/// Returns the exact OP-1 spelling of a resolved operation family.
#[must_use]
pub fn operation_family_spelling(id: OperationFamilyId) -> Option<&'static str> {
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

/// One scope kind from the active specification's scope-construction matrix.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScopeKind {
    /// The complete closed compilation unit.
    CompilationUnit,
    /// Type and const generics owned by one declaration.
    DeclarationGenerics,
    /// Region parameters, parameters, signature suffix, clauses, and body.
    FunctionSignature,
    /// One contract-member signature.
    ContractSignature,
    /// The erased definition and proof-clause scope of one function contract.
    ContractBlock,
    /// A concrete function body.
    FunctionBody,
    /// The statement body nested under an arm, loop, or local region.
    NestedBody,
    /// Match binders visible to one arm body.
    Arm,
    /// One loop label visible only to that loop body.
    LoopLabel,
    /// One counted label and binder visible only to that counted body.
    CountedRange,
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

    /// The entry heap's store region [PROV-1].
    ///
    /// The general store the runtime mints before `main` is named by no
    /// source REGIONID: `main` declares no region parameter [FN-7], so the
    /// region has no written spelling and every elided store brand that
    /// resolves to it reaches it by elision alone. It is one region for the
    /// whole unit, so it is one identity rather than a per-occurrence minted
    /// declaration, and it is disjoint from every resolver declaration
    /// because no unit holds `u32::MAX` of them.
    pub const ENTRY_HEAP_REGION: Self = Self(u32::MAX);

    /// Whether this identity is the entry heap's store region [PROV-1].
    #[must_use]
    pub const fn is_entry_heap_region(self) -> bool {
        self.0 == u32::MAX
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

/// Dense identity of one normative [SYS-2] system declaration record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemDeclarationId(u8);

impl SystemDeclarationId {
    pub(crate) const fn new(ordinal: u8) -> Self {
        Self(ordinal)
    }

    /// Returns the zero-based `system_declaration_ordinal` in [SYS-2] preorder.
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

/// Closed declaration-class order used by active-specification resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeclarationClass {
    /// Top-level source function.
    Function,
    /// Top-level immutable named constant.
    NamedConst,
    /// Lexical const generic.
    ConstGeneric,
    /// Parameter, let binding, match binder, or counted-range binder.
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
    /// One machine-checked invariant fact named by source.
    Invariant,
    /// One distinct OP-1 spelling.
    OperationFamily,
}

/// Closed resolver collision-domain order.
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
    /// Machine-checked invariant facts.
    Invariant,
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
            Self::Invariant => 6,
        }
    }
}

/// Source declaration roles D01 through D15.
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
    /// D15: counted-range binder.
    CountedBinder,
    /// A named invariant fact visible after its checked declaration point.
    Invariant,
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

/// Lexical-use roles retained by name resolution.
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
    /// The leading enum variant of an FN-9 selector.
    EnsuresVariant,
    /// U06: region carried by a type.
    TypeRegion,
    /// U07: region carried by a mode.
    ModeRegion,
    /// U08: explicit region type argument.
    TypeArgumentRegion,
    /// U09: formal value parameter at the root of a state-effect path.
    EffectRoot,
    /// Region whose arena allocation list an effect row extends.
    EffectAllocationRegion,
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
    /// One local integer value named by an INV-1 proof-only affine factor.
    InvariantValue,
    /// One local integer value named by a PRF-1 finite source-proof factor.
    ProofValue,
    /// One earlier named invariant selected by a local `use` step.
    InvariantFact,
}

/// Deferred member and field uses resolved by the semantic owner type.
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
    /// A statically selected field after an effect-path root.
    EffectField,
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
    /// One [SYS-2] record from the system domain present in every unit [SYS-3].
    System(SystemDeclarationId),
    /// One [TYPE-2] compiler-owned container or provider nominal.
    Container(ContainerNominalId),
    /// One [BLK-0] kernel-domain operation present in every unit [SYS-3].
    Kernel(KernelOperationId),
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
    /// One admitted [SYS-2] lookup entry ([SYS-1], [SYS-3]).
    System(SystemDeclarationId),
    /// One [TYPE-2] compiler-owned container or provider nominal, admitted at
    /// a `type` TYPEID in every unit.
    Container(ContainerNominalId),
    /// One admitted [BLK-0] kernel-domain operation, admitted at a `callee`
    /// IDENT in every unit [SYS-3].
    Kernel(KernelOperationId),
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

/// Grammar class of one private FN-9 selector record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionSelectorClass {
    Plain,
    Variant,
}

/// One FN-9 result-datum candidate. It is deliberately not a declaration or binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionCandidateRecord {
    pub(crate) spelling: String,
    pub(crate) origin: SourceOrigin,
    pub(crate) paired_field: Option<String>,
    pub(crate) live_conflicts: Vec<SourceOrigin>,
    pub(crate) later_local_collision: Option<SourceOrigin>,
}

/// One written variant-selector field and its candidate binder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionFieldRecord {
    pub(crate) spelling: String,
    pub(crate) origin: SourceOrigin,
    pub(crate) candidate: PostconditionCandidateRecord,
}

/// One in-clause pbase provisionally owned by its selector rather than TYPE-6.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionSelectorUseRecord {
    pub(crate) spelling: String,
    pub(crate) origin: SourceOrigin,
}

/// Private resolver handoff for one structurally admitted FN-9 clause.
///
/// Ordinary entry uses are linked only provisionally. The semantic FN-9
/// admission pass activates them after concrete signature substitution; an
/// invalid selector therefore wins before any deferred entry lookup issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionResolutionRecord {
    pub(crate) function: NodePath,
    pub(crate) block: NodePath,
    pub(crate) selector: NodePath,
    pub(crate) class: PostconditionSelectorClass,
    /// Every declared result ordinal's binder, in written order [GRAM-2].
    /// A declaration writing one result has exactly one entry, and every
    /// clause of either class may name any of them [CALL-4].
    pub(crate) result_binders: Vec<PostconditionCandidateRecord>,
    /// The route's written ordinal binder, `when b is V(f: r):`, when the
    /// clause writes one [CALL-4].
    pub(crate) route_ordinal: Option<String>,
    pub(crate) fields: Vec<PostconditionFieldRecord>,
    pub(crate) variant_target: Option<ResolvedTarget>,
    pub(crate) provisional_uses: Vec<LexicalUseRecord>,
    pub(crate) selector_uses: Vec<PostconditionSelectorUseRecord>,
    pub(crate) entry_inventory_issue: Option<ResolutionIssue>,
    pub(crate) entry_resolution_issue: Option<ResolutionIssue>,
}

/// One normative [SYS-2] declaration record admitted to one resolved unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemDeclarationRecord {
    id: SystemDeclarationId,
    spelling: &'static str,
    class: Option<DeclarationClass>,
}

impl SystemDeclarationRecord {
    /// Returns the [SYS-2] preorder identity.
    #[must_use]
    pub const fn id(self) -> SystemDeclarationId {
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
pub enum ResolutionRule {
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
    /// Parameter-rooted state-effect lookup.
    Eff1,
    /// Operation-family or callee lookup.
    Op1,
    /// Contract lookup.
    Fn3,
    /// Function binding lookup.
    Fn4,
    /// Requires-block structural admission.
    Fn8,
    /// Ensures-block structural and selector admission.
    Fn9,
    /// Invariant declaration names and proof-only target-value lookup.
    Inv1,
    /// Finite source-certificate relation-value lookup.
    Prf1,
}

impl ResolutionRule {
    /// Returns the exact numbered rule spelling from the active kernel specification.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Form3 => "FORM-3",
            Self::Form5 => "FORM-5",
            Self::Gram10 => "GRAM-10",
            Self::Type5 => "TYPE-5",
            Self::Type6 => "TYPE-6",
            Self::Const1 => "CONST-1",
            Self::Const2 => "CONST-2",
            Self::Own3 => "OWN-3",
            Self::Eff1 => "EFF-1",
            Self::Op1 => "OP-1",
            Self::Fn3 => "FN-3",
            Self::Fn4 => "FN-4",
            Self::Fn8 => "FN-8",
            Self::Fn9 => "FN-9",
            Self::Inv1 => "INV-1",
            Self::Prf1 => "PRF-1",
        }
    }
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
    /// Header or body-local invariant declaration.
    Invariant,
    /// Counted-range binder.
    ForBinder,
    /// Match binder.
    MatchBinder,
    /// Plain FN-9 result-selector candidate.
    PlainResultSelector,
    /// Variant-form FN-9 result-selector candidate.
    VariantResultSelector,
    /// Struct field.
    Field,
    /// Enum-variant field.
    VariantField,
    /// Region parameter, using its unsigiled interior spelling.
    RegionParameter,
    /// Local region, using its unsigiled interior spelling.
    LocalRegion,
}

/// Why an FN-8/FN-9 contract block was rejected by early admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractShapeIssue {
    /// The block contains definitions but no proof clause.
    MissingClause,
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
    /// Early FN-8/FN-9 contract structural-admission failure.
    ContractShape(ContractShapeIssue),
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
        /// The repair the colliding situation admits.
        ///
        /// The four situations differ in what a writer can do about them, and
        /// the conflict list locates the other declaration without ever saying
        /// which situation this is.
        mechanical_fix: &'static str,
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

/// The first active-specification resolver rejection in specified stage and event order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionIssue {
    rule: ResolutionRule,
    origin: SourceOrigin,
    kind: ResolutionIssueKind,
}

impl ResolutionIssue {
    /// Returns the numbered rule owning this rejection.
    #[must_use]
    pub const fn rule(&self) -> ResolutionRule {
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

/// Canonical syntax plus complete active-specification lexical resolution tables.
#[derive(Debug)]
pub struct ResolvedSyntaxUnit<'classified, 'lexed, 'source> {
    syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
    scopes: Vec<ScopeRecord>,
    prelude: Vec<PreludeDeclarationRecord>,
    system: Vec<SystemDeclarationRecord>,
    declarations: Vec<DeclarationRecord>,
    dependent_declarations: Vec<DependentDeclarationRecord>,
    lexical_uses: Vec<LexicalUseRecord>,
    deferred_uses: Vec<DeferredUseRecord>,
    postconditions: Vec<PostconditionResolutionRecord>,
    inventory: Inventory,
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

    /// Returns the complete [SYS-2] inventory in normative preorder.
    ///
    /// [SYS-3] admits this fixed declaration source into every compilation
    /// unit, independently of entry-form validity or source uses.
    #[must_use]
    pub fn system_declarations(&self) -> &[SystemDeclarationRecord] {
        &self.system
    }

    /// Returns one admitted [SYS-2] record by its normative identity.
    #[must_use]
    pub fn system_declaration(&self, id: SystemDeclarationId) -> Option<&SystemDeclarationRecord> {
        self.system.get(usize::from(id.ordinal()))
    }

    /// Which [SYS-2] inventory this unit's system records came from.
    ///
    /// Every later stage that turns a [SYS-2] declaration ordinal back into a
    /// nominal, constructor, or operation index must read the same inventory
    /// state the records were built from, because a candidate's extra nominal
    /// types shift every constructor and operation ordinal.
    #[must_use]
    pub const fn inventory(&self) -> Inventory {
        self.inventory
    }

    /// Returns all source declaration events D01 through D15.
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

    pub(crate) fn postconditions(&self) -> &[PostconditionResolutionRecord] {
        &self.postconditions
    }

    /// Consumes resolution and returns the underlying canonical syntax.
    #[must_use]
    pub fn into_syntax(self) -> CanonicalSyntaxUnit<'classified, 'lexed, 'source> {
        self.syntax
    }
}

/// Failure-atomic outcome of active-specification lexical resolution.
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
