mod borrows;
mod cleanup;
mod contracts;
mod control;
mod ensures;
mod entry_form;
mod expressions;
mod floats;
mod generics;
mod nominal_instances;
mod nominals;
mod requires;
mod strict;
mod support;
mod types;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, Production, ResolvedSyntaxUnit, SemanticCompilerFailure,
    SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::entailment::{
    CallGoalDisposition, ClaimSourceIdentity, EntailmentCallee, EntailmentContext,
    PostconditionSchedule, VerifiedPostconditionSummary, analyze_function,
    analyze_function_candidate, build_claim_ledger, finalize_function_entailment,
    postcondition_schedule,
};
use super::goal::{
    CheckedCallRequirement, CheckedRequirement, ConcreteGoal, GoalDatum, GoalExpression,
    GoalOperation, GoalProjection, first_ephemeral_argument, render_goal,
};
use super::model::{
    BindingId, CheckedConst, CheckedConstant, CheckedConstantId, CheckedContract,
    CheckedExpression, CheckedFlatElement, CheckedFunction, CheckedGenericRequirement, CheckedMode,
    CheckedNominal, CheckedParameter, CheckedProgramData, CheckedSetTarget, CheckedSliceOrigin,
    CheckedStatement, CheckedType, CheckedValue, ClaimAdvisory, DerivedConst, DerivedConstId,
    FunctionId, NominalId, ValueInitializerKind, evaluate_const_operation,
};
use super::postcondition::CheckedPostconditionSelector;
use super::provenance::{
    DatumSelector, ProvenanceContext, ProvenanceDemandKind as InternalDemandKind,
    ProvenanceFailures, ProvenanceMetadata, ProvenanceTarget, analyze_program_provenance,
    analyze_program_provenance_with_frozen, freeze_program_provenance,
};
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use borrows::{AccessKind, ResolvedPlace};
use borrows::{BorrowInfo, BorrowKind, SliceInfo, SliceLoan};
use control::{ControlCounters, ControlScope};
use generics::{GenericParameter, GenericSubstitution};

#[derive(Clone)]
struct ParameterSignature {
    declaration: DeclarationId,
    node_path: crate::NodePath,
    name: String,
    mode: CheckedMode,
    ty: CheckedType,
}

#[derive(Clone)]
struct FunctionSignature {
    id: FunctionId,
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    symbol: String,
    deny_claims_marker: Option<crate::NodePath>,
    region_parameters: Vec<DeclarationId>,
    parameters: Vec<ParameterSignature>,
    result_mode: CheckedMode,
    result: CheckedType,
    slice_return_ceiling: Vec<CheckedSliceOrigin>,
    effects_node: NodeId,
    declared_effects: EffectSet,
    substitution: GenericSubstitution,
}

#[derive(Clone, Copy)]
struct PostconditionCheckContext {
    record: usize,
    result_type: CheckedType,
}

/// One fully checked concrete function awaiting program-level entailment.
///
/// Binding spellings are checker-only diagnostic data. Keeping them beside
/// the checked function lets phase A finish the complete concrete inventory
/// before phase B derives or rejects any acceptance-bearing judgment.
struct CheckedFunctionInventory {
    function: CheckedFunction,
    binding_names: Vec<String>,
}

fn derive_slice_return_ceiling(
    parameters: &[ParameterSignature],
    result_mode: CheckedMode,
    result: CheckedType,
) -> Vec<CheckedSliceOrigin> {
    let (CheckedMode::Own, CheckedType::Slice { region, element }) = (result_mode, result) else {
        return Vec::new();
    };
    let mut ceiling = vec![CheckedSliceOrigin::ImmutableConst];
    for parameter in parameters {
        if parameter.mode == CheckedMode::Own
            && parameter.ty == (CheckedType::Slice { region, element })
        {
            ceiling.push(CheckedSliceOrigin::FormalSlice {
                parameter: parameter.declaration,
                region,
            });
        }
    }
    ceiling
}

/// What a callable boundary alone says about where a borrow-mode result can
/// be rooted [FN-1, OWN-6, OWN-10].
///
/// Distinct formal regions are incomparable inside the callee [OWN-3] and
/// OWN-10 forbids rooting a result-region borrow in callee-local storage, so
/// every borrow an accepted callee can deliver in the result's formal region
/// derives from a parameter that names that region or from immutable named
/// `const` storage. Counting the parameters that could supply it is therefore
/// a complete provenance judgment over the signature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResultProvenance {
    /// The written result type blocks the judgment: a borrow of a `slice`
    /// carries descriptor and origin relations this judgment does not model
    /// [OWN-5], and an unsubstituted generic conservatively names every
    /// region. FN-1 rejects the slice shape at the boundary independently.
    Unjudgeable,
    /// Exactly one parameter can source the result: the debtor position.
    Candidate(usize),
    /// No parameter can source the result and none names its region, so
    /// permanently read-only named-const storage is the only remaining
    /// source [CONST-2, OWN-10] and provenance is unique by elimination.
    ConstStorage,
    /// Two or more parameters could source the result, or one names its
    /// region in a written type. No caller can root the claim.
    Ambiguous,
}

/// Whether a written parameter or result type carries the given formal
/// region anywhere a borrow could be rooted through it. Storage is borrow-
/// and region-free [STOR-5], so a direct `slice` type is the only written
/// type region today; an unsubstituted generic is conservatively treated as
/// carrying every region.
fn type_carries_region(ty: CheckedType, region: DeclarationId) -> bool {
    match ty {
        CheckedType::Slice { region: slice, .. } => slice == region,
        CheckedType::Generic(_) | CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => true,
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Nominal(_)
        | CheckedType::Buffer { .. }
        | CheckedType::Array { .. } => false,
    }
}

/// Judges a borrow-mode result's provenance from the callable boundary
/// alone. `None` for an `own` result, which roots no caller claim.
///
/// A parameter supplies the result when it is written as a borrow of the
/// result's kind in the result's formal region. A same-region parameter of
/// the other kind is not a supplier but still defeats the judgment: no
/// `uniq` result derives from a `shared` source, but a `shared` result can
/// derive from a `uniq` parameter through a nested borrow-returning call,
/// so the pair leaves two possible roots — reject-when-unsure [OWN-8].
fn borrow_result_provenance(
    parameters: &[ParameterSignature],
    result_mode: CheckedMode,
    result: CheckedType,
) -> Option<ResultProvenance> {
    let (result_kind, result_region) = match result_mode {
        CheckedMode::Own => return None,
        CheckedMode::Shared(region) => (BorrowKind::Shared, region),
        CheckedMode::Unique(region) => (BorrowKind::Unique, region),
    };
    if matches!(result, CheckedType::Slice { .. }) || type_carries_region(result, result_region) {
        return Some(ResultProvenance::Unjudgeable);
    }
    let mut candidate = None;
    for (index, parameter) in parameters.iter().enumerate() {
        if type_carries_region(parameter.ty, result_region) {
            return Some(ResultProvenance::Ambiguous);
        }
        let (kind, region) = match parameter.mode {
            CheckedMode::Own => continue,
            CheckedMode::Shared(region) => (BorrowKind::Shared, region),
            CheckedMode::Unique(region) => (BorrowKind::Unique, region),
        };
        if region != result_region {
            continue;
        }
        if kind != result_kind || candidate.is_some() {
            return Some(ResultProvenance::Ambiguous);
        }
        candidate = Some(index);
    }
    Some(candidate.map_or(ResultProvenance::ConstStorage, ResultProvenance::Candidate))
}

/// [FN-1]'s restructuring for a borrow-mode result whose source the callable
/// boundary does not determine, shared by the `fn_decl` and `fn_sig` sites.
const AMBIGUOUS_RESULT_PROVENANCE_RESTRUCTURING: &str = "give the source parameter its own region so exactly one parameter shares the result's \
     region and kind, or return the decision as a value and let the caller borrow from the \
     source it names";

/// [STOR-4]'s restructuring for an arena value that would leave its region's
/// block, shared by every site that establishes the escape.
const ARENA_ESCAPE_RESTRUCTURING: &str = "keep the arena value inside its region's block; \
     return or deliver its content, or a borrow OWN-10 admits, instead";

struct ContractInfo {
    checked: CheckedContract,
    members: Vec<contracts::ContractMemberInfo>,
}

#[derive(Clone)]
struct FunctionTemplate {
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    generic_parameters: Vec<GenericParameter>,
}

#[derive(Clone)]
struct NominalTemplate {
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    role: DeclarationRole,
    generic_parameters: Vec<GenericParameter>,
}

/// A nominal instance a derived type named, awaiting interning.
#[derive(Clone, Copy)]
enum PendingNominal {
    /// [STOR-2] a box over this referent.
    Box(CheckedType),
    /// [STOR-2] an `arena<'r, T>` instance over this region and content.
    Arena(DeclarationId, CheckedType),
    /// The one compiler-owned region allocation-list nominal [STOR-3].
    ArenaStorage,
    /// A prelude instance, such as the `Result<T, E>` a checked row produces.
    Prelude(PreludeType),
}

#[derive(Clone)]
struct NominalInstance {
    id: NominalId,
    substitution: GenericSubstitution,
}

#[derive(Clone, Copy)]
enum ConstructorTemplate {
    Struct { template: usize },
    Enum { template: usize, variant: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalBinding {
    binding: BindingId,
    declaration: DeclarationId,
    mode: CheckedMode,
    ty: CheckedType,
    live: bool,
    loop_depth: usize,
    /// Compiler-updated counted binders are readable source bindings but are
    /// never writer-controlled storage [SET-1, OWN-11].
    compiler_updated: bool,
    borrow: Option<BorrowInfo>,
    slice: Option<SliceInfo>,
    // Source-owned claims outlive any one slice descriptor and end only with
    // their named data region.
    slice_loans: Vec<SliceLoan>,
    // A `uniq` holder whose arm-scoped child reborrows a match created
    // [OWN-13]: its own allowance is withdrawn, and because binder borrows
    // live to the end of their derived region's block [OWN-4] the holder
    // does not resume within that region — the binding itself dies at or
    // before the window's end, so no resumption point exists [OWN-5].
    suspended: bool,
}

impl LocalBinding {
    fn push_slice_loan(&mut self, loan: SliceLoan) {
        if !self.slice_loans.contains(&loan) {
            self.slice_loans.push(loan);
        }
    }

    fn end_slice_region(&mut self, region: DeclarationId) {
        self.slice_loans.retain(|loan| loan.region != region);
    }

    /// Whether two joined states agree apart from their region-scoped claims
    /// (slice loans and [OWN-13] suspension), which join by union instead.
    fn same_except_region_claims(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.slice_loans.clear();
        right.slice_loans.clear();
        left.suspended = false;
        right.suspended = false;
        left == right
    }

    /// Union of region-scoped claims: a claim established on any joined path
    /// holds for the region remainder, matching [OWN-4]'s named-region
    /// liveness of the borrows that carry it.
    fn merge_region_claims_from(&mut self, other: &Self) {
        for loan in &other.slice_loans {
            self.push_slice_loan(loan.clone());
        }
        self.suspended |= other.suspended;
    }
}

#[derive(Clone, Copy)]
enum Constructor {
    Struct(NominalId),
    Enum { nominal: NominalId, variant: u32 },
}

struct TypedExpression {
    expression: CheckedExpression,
    mode: CheckedMode,
    borrow: Option<BorrowInfo>,
    slice: Option<SliceInfo>,
    holder: Option<DeclarationId>,
    /// Whether this expression denotes the reference itself rather than a
    /// place reached through it.
    ///
    /// A destination of borrow mode wants exactly this value; a construct
    /// that needs the referent — a `match` scrutinee under [OWN-13],
    /// `propagate` under [ERR-3] — rejects it citing [TYPE-7] with the
    /// `deref(.)` fix. A dereferenced place and an owned value are not
    /// reference values even when their mode is a borrow.
    reference_value: bool,
    effects: EffectSet,
    accesses: Vec<PlaceAccess>,
}

#[derive(Clone)]
struct PlaceAccess {
    place: ResolvedPlace,
    kind: AccessKind,
}

impl TypedExpression {
    fn owned(expression: CheckedExpression, effects: EffectSet) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            borrow: None,
            slice: None,
            holder: None,
            reference_value: false,
            effects,
            accesses: Vec::new(),
        }
    }

    fn owned_with_access(
        expression: CheckedExpression,
        effects: EffectSet,
        place: ResolvedPlace,
        kind: AccessKind,
    ) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            borrow: None,
            slice: None,
            holder: None,
            reference_value: false,
            effects,
            accesses: vec![PlaceAccess { place, kind }],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EffectSet {
    reads: Vec<DeclarationId>,
    writes: Vec<DeclarationId>,
    allocates_heap: bool,
    allocates_arenas: Vec<DeclarationId>,
    external: bool,
    blocks: bool,
    traps: bool,
}

impl EffectSet {
    const NONE: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        allocates_arenas: Vec::new(),
        external: false,
        blocks: false,
        traps: false,
    };
    const TRAPS: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        allocates_arenas: Vec::new(),
        external: false,
        blocks: false,
        traps: true,
    };
    const ALLOCATES_HEAP: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: true,
        allocates_arenas: Vec::new(),
        external: false,
        blocks: false,
        traps: false,
    };
    const ALLOCATES_HEAP_AND_TRAPS: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: true,
        allocates_arenas: Vec::new(),
        external: false,
        blocks: false,
        traps: true,
    };

    fn union(mut self, other: Self) -> Self {
        for region in other.reads {
            self.add_read(region);
        }
        for region in other.writes {
            self.add_write(region);
        }
        self.allocates_heap |= other.allocates_heap;
        for region in other.allocates_arenas {
            self.add_arena_allocation(region);
        }
        self.external |= other.external;
        self.blocks |= other.blocks;
        self.traps |= other.traps;
        self
    }

    fn add_read(&mut self, region: DeclarationId) {
        if !self.reads.contains(&region) {
            self.reads.push(region);
            self.reads.sort_unstable();
        }
    }

    fn add_write(&mut self, region: DeclarationId) {
        if !self.writes.contains(&region) {
            self.writes.push(region);
            self.writes.sort_unstable();
        }
    }

    fn add_arena_allocation(&mut self, region: DeclarationId) {
        if !self.allocates_arenas.contains(&region) {
            self.allocates_arenas.push(region);
            self.allocates_arenas.sort_unstable();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PreludeType {
    Option(CheckedType),
    Result(CheckedType, CheckedType),
    Overflow,
    DivError,
    NarrowError,
}

/// v0.31-candidate reborrow-extension switch. The candidate at
/// `spec/kernel-spec.md` admits the previously deferred forms — a reborrow
/// argument to a borrow-returning call, a bound call-result borrow holder,
/// and the grandchild chains they compose [OWN-5, OWN-6, OWN-12, OWN-14] —
/// so this is `true` and the branch implements its own candidate. The
/// test-only `check_semantics_reborrow_extension` entry now selects the same
/// judgment as the shipped path.
pub(crate) const REBORROW_EXTENSION_ACTIVE: bool = true;

/// v0.32-candidate declaration-site provenance switch [FN-1, OWN-6]. The
/// candidate moves the ambiguity rejection from the binding to the callable
/// boundary: a declaration whose borrow-mode result has no signature-
/// determined source is itself the error, because GRAM-9's flat form makes
/// every call result let-bound, so a result no caller can bind is unusable
/// by construction. `false` keeps every v0.31 disposition byte for byte;
/// the test-only `check_semantics_declaration_provenance` entry selects the
/// candidate judgment until activation flips this constant.
pub(crate) const DECLARATION_PROVENANCE: bool = true;

struct Checker<'unit, 'classified, 'lexed, 'source> {
    resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    /// Whether an undischarged obligation or refuted claim rejects [OP-4,
    /// CLM-2]. Always true outside `check_semantics_dark`, the test-only
    /// observability hook.
    reject_entailment: bool,
    /// The arithmetic-mode dissolution integration switch: whether a bare
    /// `+`/`-`/`*` with a constant operand carries an [ENT-6] overflow
    /// obligation instead of its runtime trap [OP-2]. Follows
    /// [`ARITHMETIC_OVERFLOW_OBLIGATIONS`] outside the v0.31 candidate
    /// tests.
    arithmetic_obligations: bool,
    /// Whether the v0.31-candidate reborrow extension is admitted; see
    /// [`REBORROW_EXTENSION_ACTIVE`].
    reborrow_extension: bool,
    /// Whether the v0.32-candidate declaration-site provenance judgment is
    /// live; see [`DECLARATION_PROVENANCE`].
    declaration_provenance: bool,
    /// The division dissolution integration switch: whether a bare `/` or
    /// `%` in [OP-2]'s divisor class carries an [ENT-6] division obligation
    /// instead of its runtime trap [OP-2]. Follows [`DIVISION_OBLIGATIONS`]
    /// outside the v0.32-candidate tests.
    division_obligations: bool,
    tree: TreeView<'unit, 'classified, 'lexed, 'source>,
    nominals: Vec<CheckedNominal>,
    nominal_nodes: Vec<Option<NodeId>>,
    nominal_states: Vec<u8>,
    source_nominal_instances: Vec<Option<(usize, GenericSubstitution)>>,
    box_nominals: HashMap<CheckedType, NominalId>,
    /// `arena<'r, T>` instances by (region declaration, content type): the
    /// region is part of the type's identity [OWN-3, STOR-4].
    arena_nominals: HashMap<(DeclarationId, CheckedType), NominalId>,
    /// The one compiler-owned region allocation-list nominal, interned on
    /// first use [STOR-3].
    arena_storage_nominal: Option<NominalId>,
    /// Nominal instances a derived type named that were not interned yet.
    /// Written by the `&self` checking path and drained by the `&mut self`
    /// driver between attempts at one function.
    pending_nominals: RefCell<Vec<PendingNominal>>,
    prelude_nominals: HashMap<PreludeType, NominalId>,
    system_nominals: HashMap<u8, NominalId>,
    prelude_types: Vec<Option<PreludeType>>,
    nominal_templates: Vec<NominalTemplate>,
    nominal_templates_by_declaration: HashMap<DeclarationId, usize>,
    nominals_by_declaration: HashMap<DeclarationId, Vec<NominalInstance>>,
    constructor_templates_by_declaration: HashMap<DeclarationId, ConstructorTemplate>,
    signatures: Vec<FunctionSignature>,
    function_templates: Vec<FunctionTemplate>,
    templates_by_declaration: HashMap<DeclarationId, usize>,
    functions_by_declaration: HashMap<DeclarationId, Vec<FunctionId>>,
    constants: HashMap<DeclarationId, CheckedConstantId>,
    checked_constants: Vec<CheckedConstant>,
    /// Hash-consed symbolic const operations [CONST-1 candidate]. Written by
    /// the `&self` const-expression parse while a generic template or
    /// symbolic validation instance is checked; every concrete instantiation
    /// evaluates entries away, so no id reaches lowering.
    derived_consts: RefCell<Vec<DerivedConst>>,
    generic_requirements: Vec<CheckedGenericRequirement>,
    postcondition_selectors: Vec<CheckedPostconditionSelector>,
    postcondition_unavailable_declarations: Vec<DeclarationId>,
    active_postcondition: Cell<Option<PostconditionCheckContext>>,
    contracts: Vec<ContractInfo>,
    contracts_by_declaration: HashMap<DeclarationId, usize>,
}

/// The arithmetic-mode dissolution integration switch [OP-2, ENT-6]: `true`
/// under the v0.31 candidate at `spec/kernel-spec.md`, which attaches the
/// overflow obligation family to the constant-operand class, drops those
/// sites' trap records and `traps` effect contribution, and rejects
/// undischarged class sites citing OP-2. A bare `+`/`-`/`*` with two
/// non-constant operands keeps its runtime overflow trap.
pub(crate) const ARITHMETIC_OVERFLOW_OBLIGATIONS: bool = true;

/// The division dissolution integration switch [OP-2, ENT-6]: `false` under
/// active v0.31, `true` under the v0.32 candidate, which attaches the
/// division obligation family to [OP-2]'s divisor class, drops those sites'
/// trap records and `traps` effect contribution, and rejects undischarged
/// class sites citing OP-2. A bare `/` or `%` over a signed selected type
/// with two non-constant operands stays outside the class and keeps its
/// runtime trap, because its safe condition is the disjunction
/// `dividend != iK::MIN or divisor != -1`, which the [ENT-4] conjunctive
/// fragment cannot state.
pub(crate) const DIVISION_OBLIGATIONS: bool = true;
/// Checks the currently implemented active-specification semantic family.
///
/// Unsupported language families remain explicit compiler capability results;
/// only a proved numbered-rule violation becomes [`SemanticOutcome::SourceIssue`].
#[must_use]
pub fn check_semantics<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        true,
        ARITHMETIC_OVERFLOW_OBLIGATIONS,
        REBORROW_EXTENSION_ACTIVE,
        DECLARATION_PROVENANCE,
        DIVISION_OBLIGATIONS,
    )
}

/// [`check_semantics`] with the [OP-4]/[CLM-2] entailment rejection disabled,
/// so unit tests can observe every retained obligation and claim disposition
/// of one function, not only the first rejecting one. This is a test-only
/// observability hook, never a compilation mode: acceptance behavior has
/// exactly one path.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_dark<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        false,
        ARITHMETIC_OVERFLOW_OBLIGATIONS,
        REBORROW_EXTENSION_ACTIVE,
        DECLARATION_PROVENANCE,
        DIVISION_OBLIGATIONS,
    )
}

/// [`check_semantics`] with the arithmetic-mode dissolution switch forced
/// on. [`ARITHMETIC_OVERFLOW_OBLIGATIONS`] is now `true` under the v0.31
/// candidate, so this entry selects the same judgment as the shipped path
/// and the callers naming it record which judgment they mean. Test-only; the
/// one shipped acceptance path reads that constant.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_arithmetic_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        true,
        true,
        REBORROW_EXTENSION_ACTIVE,
        DECLARATION_PROVENANCE,
        DIVISION_OBLIGATIONS,
    )
}

/// [`check_semantics`] with the v0.31-candidate reborrow extension admitted.
/// [`REBORROW_EXTENSION_ACTIVE`] is now `true`, so this entry selects the
/// same judgment as the shipped path and the callers naming it record which
/// judgment they mean. Test-only: the shipped acceptance behavior has exactly
/// one path.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_reborrow_extension<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        true,
        ARITHMETIC_OVERFLOW_OBLIGATIONS,
        true,
        DECLARATION_PROVENANCE,
        DIVISION_OBLIGATIONS,
    )
}

/// [`check_semantics`] with the v0.32-candidate declaration-site provenance
/// judgment live [FN-1, OWN-6]. Test-only until [`DECLARATION_PROVENANCE`]
/// flips at activation; the shipped acceptance behavior has exactly one
/// path, and the paired default-checker tests pin the v0.31 dispositions of
/// the same sources.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_declaration_provenance<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        true,
        ARITHMETIC_OVERFLOW_OBLIGATIONS,
        REBORROW_EXTENSION_ACTIVE,
        true,
        DIVISION_OBLIGATIONS,
    )
}

/// [`check_semantics`] with the v0.32-candidate division dissolution switch
/// forced on, so the candidate's judgment can be tested while
/// [`DIVISION_OBLIGATIONS`] stays `false` for the shipped path. Test-only;
/// the one shipped acceptance path reads that constant.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_division_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(
        resolved,
        true,
        ARITHMETIC_OVERFLOW_OBLIGATIONS,
        REBORROW_EXTENSION_ACTIVE,
        DECLARATION_PROVENANCE,
        true,
    )
}

fn check_semantics_with<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    reject_entailment: bool,
    arithmetic_obligations: bool,
    reborrow_extension: bool,
    declaration_provenance: bool,
    division_obligations: bool,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    let preflight = if resolved.postconditions().is_empty() {
        Ok(())
    } else {
        Checker::new(
            &resolved,
            reject_entailment,
            arithmetic_obligations,
            reborrow_extension,
            declaration_provenance,
            division_obligations,
        )
        .and_then(|mut checker| {
            let items = checker.item_declarations()?;
            checker.preflight_postcondition_selectors(&items)
        })
    };
    let result = preflight.and_then(|()| {
        Checker::new(
            &resolved,
            reject_entailment,
            arithmetic_obligations,
            reborrow_extension,
            declaration_provenance,
            division_obligations,
        )
        .and_then(|mut checker| checker.check_program())
    });
    match result {
        Ok(data) => SemanticOutcome::Complete(Box::new(CheckedProgram {
            _resolved: resolved,
            data,
        })),
        Err(CheckStop::Issue(issue)) => SemanticOutcome::SourceIssue { issue: *issue },
        Err(CheckStop::Resolution(issue)) => SemanticOutcome::ResolutionIssue { issue: *issue },
        Err(CheckStop::Unsupported(unsupported)) => SemanticOutcome::Unsupported { unsupported },
        Err(CheckStop::Compiler(failure)) => SemanticOutcome::CompilerFailure { failure },
        // The deferred-box signal is repaired where it is raised, one
        // function at a time, so reaching here is an internal inconsistency
        // rather than anything the source can express.
        Err(CheckStop::DeferredNominal) => SemanticOutcome::CompilerFailure {
            failure: SemanticCompilerFailure::InvalidResolution,
        },
        Err(CheckStop::PostconditionPrerequisiteUnavailable) => SemanticOutcome::CompilerFailure {
            failure: SemanticCompilerFailure::InvalidResolution,
        },
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Which [SYS-2] inventory this unit was resolved against.
    ///
    /// Every ordinal-to-index lookup must use the state the resolver built
    /// the records from; reading it from the resolved unit keeps the two
    /// stages from disagreeing about the inventory.
    const fn traversal_surface(&self) -> bool {
        self.resolved.traversal_surface()
    }

    fn mark_postcondition_unavailable(&mut self, declaration: DeclarationId) {
        if !self
            .postcondition_unavailable_declarations
            .contains(&declaration)
        {
            self.postcondition_unavailable_declarations
                .push(declaration);
        }
    }

    fn postcondition_declaration_unavailable(&self, declaration: DeclarationId) -> bool {
        self.postcondition_unavailable_declarations
            .contains(&declaration)
    }

    /// [FN-1]'s declaration-site provenance judgment under the v0.32
    /// candidate: a callable boundary whose borrow-mode result has no
    /// signature-determined source is a hard error at its complete `rtype`,
    /// whether or not the function is ever called. GRAM-9's flat form binds
    /// every call result with a `let`, so such a result is unusable by
    /// construction and the declaration, not the binding, is the error.
    /// Shared by the top-level `fn_decl` and contract-member `fn_sig`
    /// signature-formation sites, exactly as the slice-result judgments are.
    fn reject_ambiguous_result_provenance(
        &self,
        parameters: &[ParameterSignature],
        result_mode: CheckedMode,
        result: CheckedType,
        rtype: NodeId,
    ) -> Result<(), CheckStop> {
        if !self.declaration_provenance
            || borrow_result_provenance(parameters, result_mode, result)
                != Some(ResultProvenance::Ambiguous)
        {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Fn1,
            rtype,
            SemanticIssueKind::AmbiguousResultProvenance {
                mechanical_fix: AMBIGUOUS_RESULT_PROVENANCE_RESTRUCTURING,
            },
        )
    }

    fn new(
        resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
        reject_entailment: bool,
        arithmetic_obligations: bool,
        reborrow_extension: bool,
        declaration_provenance: bool,
        division_obligations: bool,
    ) -> Result<Self, CheckStop> {
        Ok(Self {
            resolved,
            reject_entailment,
            arithmetic_obligations,
            reborrow_extension,
            declaration_provenance,
            division_obligations,
            tree: TreeView::new(resolved)?,
            nominals: Vec::new(),
            nominal_nodes: Vec::new(),
            nominal_states: Vec::new(),
            source_nominal_instances: Vec::new(),
            box_nominals: HashMap::new(),
            arena_nominals: HashMap::new(),
            arena_storage_nominal: None,
            pending_nominals: RefCell::new(Vec::new()),
            prelude_nominals: HashMap::new(),
            system_nominals: HashMap::new(),
            prelude_types: Vec::new(),
            nominal_templates: Vec::new(),
            nominal_templates_by_declaration: HashMap::new(),
            nominals_by_declaration: HashMap::new(),
            constructor_templates_by_declaration: HashMap::new(),
            signatures: Vec::new(),
            function_templates: Vec::new(),
            templates_by_declaration: HashMap::new(),
            functions_by_declaration: HashMap::new(),
            constants: HashMap::new(),
            checked_constants: Vec::new(),
            derived_consts: RefCell::new(Vec::new()),
            generic_requirements: Vec::new(),
            postcondition_selectors: Vec::new(),
            postcondition_unavailable_declarations: Vec::new(),
            active_postcondition: Cell::new(None),
            contracts: Vec::new(),
            contracts_by_declaration: HashMap::new(),
        })
    }

    fn check_program(&mut self) -> Result<CheckedProgramData, CheckStop> {
        let items = self.item_declarations()?;
        // The [FN-7] entry-form and [GRAM-11] system-call-argument
        // judgments run first in DIAG-1 stage order; the former also fixes
        // which entry shape the rest of the unit is checked under. The
        // system semantic family — [SYS-2] call typing, [EFF-2] effect
        // attribution including the release contribution, and the checked
        // drop records — is implemented below, so no capability stop
        // remains at this stage; an accepted system program stops later, at
        // lowering, as an explicit unsupported capability.
        let entry = match self.check_entry_form(&items) {
            Ok(entry) => entry,
            Err(stop) => return Err(self.reject_missing_main_last(&items, stop)),
        };
        self.check_system_call_arguments()?;
        self.declare_nominals(&items)?;
        self.collect_constants(&items)?;
        self.complete_nominals()?;
        self.collect_deferred_nominal_constants(&items)?;
        self.collect_function_signatures(&items)?;
        self.admit_postcondition_selectors()?;
        let nominal_count_before_function_checking = self.nominals.len();
        let main = self.main_id()?;
        let strict_markers = self.strict_declaration_markers()?;

        // Phase A completes every reachable concrete function before any
        // acceptance-bearing entailment judgment runs. This makes forward,
        // recursive, mutually recursive, and concrete generic call summaries
        // independent of function traversal order.
        let mut function_inventory = Vec::with_capacity(self.signatures.len());
        for index in 0..self.signatures.len() {
            function_inventory.push(self.check_function_interning_nominals(index)?);
        }
        // Function checking discovers only the instances a derived type
        // names, which are box and prelude ones; a *source* nominal instance
        // is always interned from a written type before it runs.
        for instance in self
            .source_nominal_instances
            .iter()
            .skip(nominal_count_before_function_checking)
        {
            if instance.is_some() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }

        // For an FN-9 or CLM-3 unit, the complete existing [FN-3]/[FN-4] pass
        // remains ahead of the first acceptance-bearing optimistic query. Its
        // results are retained and reused below; no narrow duplicate prepass
        // or proof-only contract judgment exists. The no-postcondition,
        // no-marker ordinary fast path keeps its established phase order.
        let early_contracts =
            if self.resolved.postconditions().is_empty() && strict_markers.is_empty() {
                None
            } else {
                let executable_nominal_count = self.nominals.len();
                let functions = function_inventory
                    .iter()
                    .map(|checked| checked.function.clone())
                    .collect::<Vec<_>>();
                self.collect_contracts(&items)?;
                let (conformances, law_derivations) =
                    self.check_conformances_and_laws(&items, &functions)?;
                Some((executable_nominal_count, conformances, law_derivations))
            };

        // Phase B reads only the completed inventory. Kill-relevant [EFF-2]
        // projections are indexed by dense function identity [ENT-5]; later
        // program-level goal summaries extend this same complete context.
        let mut callees = vec![EntailmentCallee::default(); self.signatures.len()];
        for signature in &self.signatures {
            if let Some(slot) = callees.get_mut(signature.id.0 as usize) {
                *slot = EntailmentCallee::from_signature(
                    signature.parameters.iter().map(|parameter| parameter.mode),
                    &signature.declared_effects.writes,
                );
            }
        }
        self.install_call_requirements(&mut function_inventory)?;
        let optimistic_batch = function_inventory.iter().any(|checked| {
            checked.function.postcondition.is_some()
                || Self::statements_contain_value_if(&checked.function.body)
        }) || !strict_markers.is_empty();

        // [PRV-1] depends only on the phase-A checked program. Freeze it before
        // any optimistic S12/receiver facts enter the shared entailment batch;
        // PRV-2/3 below consumes this exact component inventory once.
        let frozen_provenance = if !optimistic_batch {
            None
        } else {
            let phase_a_functions = function_inventory
                .iter()
                .map(|checked| checked.function.clone())
                .collect::<Vec<_>>();
            Some(freeze_program_provenance(
                &phase_a_functions,
                &ProvenanceContext {
                    nominals: &self.nominals,
                    external_entry: matches!(
                        &entry,
                        super::model::CheckedEntryForm::Command { .. }
                    )
                    .then_some(main),
                },
            )?)
        };
        let postcondition_schedule = self.analyze_function_inventory(
            &mut function_inventory,
            &callees,
            optimistic_batch,
            main,
        )?;
        let mut functions = function_inventory
            .into_iter()
            .map(|checked| checked.function)
            .collect::<Vec<_>>();
        let provenance_context = ProvenanceContext {
            nominals: &self.nominals,
            external_entry: matches!(&entry, super::model::CheckedEntryForm::Command { .. })
                .then_some(main),
        };
        let mut provenance_analysis = match frozen_provenance {
            Some(frozen) => {
                analyze_program_provenance_with_frozen(&functions, &provenance_context, frozen)?
            }
            None => analyze_program_provenance(&functions, &provenance_context)?,
        };
        self.provenance_rejection(
            &functions,
            &provenance_analysis.metadata,
            &provenance_analysis.failures,
        )?;
        // [CLM-3] consumes only the already-successful ordinary and PRV
        // scratch. It registers successful existing-U roots before the one
        // derivation finish and never reads the observational ClaimLedger.
        let strict_partition = self.check_strict_partition(
            &mut functions,
            &postcondition_schedule,
            main,
            strict_markers,
        )?;
        if optimistic_batch {
            for function in &mut functions {
                finalize_function_entailment(&mut function.entailment);
            }
            provenance_analysis.refresh_entailment_views(&functions);
        }
        let provenance = provenance_analysis.metadata;

        let claim_ledger = if functions
            .iter()
            .any(|function| !function.entailment.claims.is_empty())
        {
            let claim_sources = functions
                .iter()
                .map(|function| {
                    function
                        .entailment
                        .claims
                        .iter()
                        .map(|claim| {
                            let (logical_path, coordinate) =
                                self.tree.source_identity(&claim.node_path)?;
                            Ok(ClaimSourceIdentity {
                                logical_path,
                                coordinate,
                                node_path: claim.node_path.clone(),
                                function: function.id,
                                function_symbol: function.symbol.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, SemanticCompilerFailure>>()
                })
                .collect::<Result<Vec<_>, SemanticCompilerFailure>>()?;
            build_claim_ledger(&functions, &provenance, claim_sources)?
        } else {
            Default::default()
        };

        // The ordinary function path is complete, so the executable prefix
        // closes here — after the derived box nominals, which executable code
        // allocates and drops, and before the contract metadata, which no
        // executable path reaches.
        let (executable_nominal_count, conformances, law_derivations) =
            if let Some(early) = early_contracts {
                early
            } else {
                let executable_nominal_count = self.nominals.len();
                self.collect_contracts(&items)?;
                let (conformances, law_derivations) =
                    self.check_conformances_and_laws(&items, &functions)?;
                (executable_nominal_count, conformances, law_derivations)
            };
        // The required non-rejecting [CLM-2] redundancy advisories, one per
        // redundant claim, in function then document order.
        let mut claim_advisories = Vec::new();
        for function in &functions {
            for claim in &function.entailment.claims {
                if claim.disposition == super::entailment::ClaimDisposition::Redundant {
                    claim_advisories.push(ClaimAdvisory {
                        function: function.name.clone(),
                        name: claim.name.clone(),
                    });
                }
            }
        }
        Ok(CheckedProgramData {
            nominals: self.nominals.clone(),
            executable_nominal_count,
            constants: self.checked_constants.clone(),
            functions,
            postcondition_schedule,
            strict_partition,
            provenance,
            generic_requirements: self.generic_requirements.clone(),
            contracts: self
                .contracts
                .iter()
                .map(|contract| contract.checked.clone())
                .collect(),
            conformances,
            law_derivations,
            main,
            entry,
            claim_advisories,
            claim_ledger,
        })
    }

    fn item_declarations(&self) -> Result<Vec<NodeId>, CheckStop> {
        let mut declarations = Vec::new();
        for item in self.tree.children(self.tree.root())? {
            if self.tree.production(*item)? != Production::Item {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            declarations.push(self.tree.only_child(*item)?);
        }
        Ok(declarations)
    }

    fn collect_function_signatures(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        self.collect_function_templates(items)?;
        self.collect_concrete_function_signatures()
    }

    /// Collects every non-nominal-typed const declaration. Runs before
    /// nominal completion because a nominal field's array length may name an
    /// earlier const; nominal-typed const declarations [CONST-2 candidate]
    /// need completed field inventories and are collected by the second pass
    /// below.
    fn collect_constants(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree
                    .production(*node)
                    .is_ok_and(|production| production == Production::ConstDecl)
            })
            .collect::<Vec<_>>();
        for node in nodes {
            if self.constant_declaration_is_deferred(node)? {
                continue;
            }
            self.collect_constant(node)?;
        }
        Ok(())
    }

    /// Collects the nominal-typed const declarations deferred by the first
    /// pass, in item order, after `complete_nominals` has filled the field
    /// inventories they are checked against. CONST-2's declaration-before-use
    /// rule is unaffected: a non-nominal const can never reference a
    /// nominal-typed one (a cvalue reference must have the exact expected
    /// type), so the two passes never reorder a legal dependency.
    fn collect_deferred_nominal_constants(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree
                    .production(*node)
                    .is_ok_and(|production| production == Production::ConstDecl)
            })
            .collect::<Vec<_>>();
        for node in nodes {
            if self.constant_declaration_is_deferred(node)? {
                self.collect_constant(node)?;
            }
        }
        Ok(())
    }

    fn constant_declaration_is_deferred(&self, node: NodeId) -> Result<bool, CheckStop> {
        if !super::V031_CANDIDATE_SEMANTICS {
            return Ok(false);
        }
        let ty = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        Ok(self
            .tree
            .direct_token_with(ty, crate::TerminalPredicate::TypeIdentifier)?
            .is_some())
    }

    pub(super) fn collect_constants_for_postconditions(
        &mut self,
        items: &[NodeId],
    ) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree
                    .production(*node)
                    .is_ok_and(|production| production == Production::ConstDecl)
            })
            .collect::<Vec<_>>();
        for node in nodes {
            let declaration = self.declaration_at(node, DeclarationRole::NamedConst)?.id();
            if !self.postcondition_constant_has_links(node)? {
                self.mark_postcondition_unavailable(declaration);
                continue;
            }
            // A nominal-typed const [CONST-2 candidate] is conservatively
            // unavailable to the FN-9 selector preflight for now; ordinary
            // checking collects it through the deferred second pass.
            if self.constant_declaration_is_deferred(node)? {
                self.mark_postcondition_unavailable(declaration);
                continue;
            }
            let ty = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let checkpoint = self.nominal_checkpoint();
            match self.ensure_nominal_type(ty, &GenericSubstitution::default()) {
                Ok(()) => {}
                Err(
                    CheckStop::Issue(_)
                    | CheckStop::Unsupported(_)
                    | CheckStop::PostconditionPrerequisiteUnavailable,
                ) => {
                    self.restore_nominal_checkpoint(checkpoint)?;
                    self.mark_postcondition_unavailable(declaration);
                    continue;
                }
                Err(stop) => return Err(stop),
            }
            match self.collect_constant(node) {
                Ok(()) => {}
                Err(CheckStop::Issue(_) | CheckStop::Unsupported(_)) => {
                    self.mark_postcondition_unavailable(declaration);
                }
                Err(stop) => return Err(stop),
            }
        }
        Ok(())
    }

    fn postcondition_constant_has_links(&self, node: NodeId) -> Result<bool, CheckStop> {
        let owner = self.tree.path(node)?.components();
        if self.resolved.lexical_uses().iter().any(|usage| {
            let path = usage.origin().node().components();
            path.len() >= owner.len()
                && path.starts_with(owner)
                && matches!(
                    usage.target(),
                    crate::ResolvedTarget::Source {
                        declaration,
                        class: crate::DeclarationClass::NamedConst,
                    } if !self.constants.contains_key(&declaration)
                )
        }) {
            return Ok(false);
        }
        for ty in self.tree.descendants_with(node, Production::Type)? {
            if self
                .tree
                .direct_token_with(ty, crate::TerminalPredicate::TypeIdentifier)?
                .is_some()
            {
                let path = self.tree.path(ty)?;
                if !self.resolved.lexical_uses().iter().any(|usage| {
                    usage.role() == crate::LexicalUseRole::Type && usage.origin().node() == path
                }) {
                    return Ok(false);
                }
            }
        }
        let value = self
            .tree
            .first_child_with(node, Production::Cvalue)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self
            .tree
            .direct_token_with(value, crate::TerminalPredicate::Identifier)?
            .is_some()
        {
            let path = self.tree.path(value)?;
            if !self.resolved.lexical_uses().iter().any(|usage| {
                usage.role() == crate::LexicalUseRole::ConstValue && usage.origin().node() == path
            }) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn collect_constant(&mut self, node: NodeId) -> Result<(), CheckStop> {
        let declaration = self.declaration_at(node, DeclarationRole::NamedConst)?;
        let declaration_id = declaration.id();
        let name = declaration.spelling().to_owned();
        let ty_node = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let ty = self.parse_const_type(ty_node)?;
        let value_node = self
            .tree
            .first_child_with(node, Production::Cvalue)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let value = self.parse_const_value(value_node, ty)?;
        let id = CheckedConstantId(
            u32::try_from(self.checked_constants.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        self.checked_constants.push(CheckedConstant {
            id,
            declaration: declaration_id,
            name,
            ty,
            value,
        });
        self.constants.insert(declaration_id, id);
        Ok(())
    }

    /// Orders the whole-unit missing-`main` rejection after per-declaration
    /// source rejections.
    ///
    /// [DIAG-1] leaves the order among rejection events at distinct nodes
    /// implementation-defined, and this compiler reports a declaration's own
    /// established rule violation before the [FN-7] `BundleRoot` whole-unit
    /// rejection: when the entry is missing, the remaining declarations are
    /// still driven through signature collection and phase-A function
    /// checking, and the first established source rejection found there is
    /// reported instead. Anything short of an established source rejection —
    /// success, an unsupported capability, an internal failure — falls back
    /// to the held missing-`main` rejection, so a capability stop never
    /// masks the definite FN-7 violation [DIAG-1].
    fn reject_missing_main_last(&mut self, items: &[NodeId], stop: CheckStop) -> CheckStop {
        let missing_main = matches!(
            &stop,
            CheckStop::Issue(issue)
                if issue.rule == SemanticRule::Fn7
                    && matches!(issue.kind, SemanticIssueKind::MissingMain)
        );
        if !missing_main {
            return stop;
        }
        let salvage = (|| -> Result<(), CheckStop> {
            self.check_system_call_arguments()?;
            self.declare_nominals(items)?;
            self.collect_constants(items)?;
            self.complete_nominals()?;
            self.collect_function_signatures(items)?;
            self.admit_postcondition_selectors()?;
            for index in 0..self.signatures.len() {
                self.check_function_interning_nominals(index)?;
            }
            Ok(())
        })();
        match salvage {
            Err(rejection @ (CheckStop::Issue(_) | CheckStop::Resolution(_))) => rejection,
            _ => stop,
        }
    }

    /// Returns the dense identity of the checked entry function.
    fn main_id(&self) -> Result<FunctionId, CheckStop> {
        self.signatures
            .iter()
            .find(|signature| signature.name == "main")
            .map(|signature| signature.id)
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    /// Checks one concrete function for the phase-A inventory, interning the
    /// nominal instances its derived types name.
    ///
    /// A derived type has no written form anywhere for the interning pass to
    /// have found — a purely local `box<T>` [STOR-2], the `Result<T, E>` a
    /// checked arithmetic row produces — and checking is `&self`, so the miss
    /// is reported as [`CheckStop::DeferredNominal`] and repaired here. Each
    /// attempt must intern at least one new nominal, which bounds the loop by
    /// the finitely many types one function can name.
    fn check_function_interning_nominals(
        &mut self,
        index: usize,
    ) -> Result<CheckedFunctionInventory, CheckStop> {
        loop {
            match self.check_function_inventory(index) {
                Err(CheckStop::DeferredNominal) => {
                    let pending = std::mem::take(&mut *self.pending_nominals.borrow_mut());
                    let before = self.nominals.len();
                    for nominal in pending {
                        match nominal {
                            PendingNominal::Box(referent) => {
                                self.intern_box_nominal(referent)?;
                            }
                            PendingNominal::Arena(region, content) => {
                                self.intern_arena_nominal(region, content)?;
                            }
                            PendingNominal::ArenaStorage => {
                                self.intern_arena_storage_nominal()?;
                            }
                            PendingNominal::Prelude(ty) => {
                                self.intern_prelude_nominal(ty)?;
                            }
                        }
                    }
                    if self.nominals.len() == before {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                }
                outcome => return outcome,
            }
        }
    }

    fn check_function_inventory(
        &self,
        index: usize,
    ) -> Result<CheckedFunctionInventory, CheckStop> {
        let signature = self
            .signatures
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.check_function_signature(signature)
    }

    fn check_function_signature(
        &self,
        signature: &FunctionSignature,
    ) -> Result<CheckedFunctionInventory, CheckStop> {
        let mut bindings = HashMap::new();
        let mut parameters = Vec::with_capacity(signature.parameters.len());
        let mut next_binding = 0_u32;
        let mut next_loop = 0_u32;
        let mut binding_names = signature
            .parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();
        for parameter in &signature.parameters {
            let binding = BindingId(next_binding);
            next_binding = next_binding
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
            bindings.insert(
                parameter.declaration,
                LocalBinding {
                    binding,
                    declaration: parameter.declaration,
                    mode: parameter.mode,
                    ty: parameter.ty,
                    live: true,
                    loop_depth: 0,
                    compiler_updated: false,
                    borrow: self.parameter_borrow(parameter),
                    slice: self.parameter_slice(parameter),
                    slice_loans: Vec::new(),
                    suspended: false,
                },
            );
            parameters.push(CheckedParameter {
                name: parameter.name.clone(),
                node_path: parameter.node_path.clone(),
                binding,
                mode: parameter.mode,
                ty: parameter.ty,
                slice_origins: self
                    .parameter_slice(parameter)
                    .map(|slice| slice.origins)
                    .unwrap_or_default(),
            });
        }

        let mut claim_names = Vec::new();
        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
            binding_names: &mut binding_names,
            claim_names: &mut claim_names,
        };
        let parameter_bindings = bindings.clone();
        let requirement = if let Some(node) = self
            .tree
            .first_child_with(signature.node, Production::RequiresBlock)?
        {
            let mut requires_bindings = parameter_bindings.clone();
            Some(
                self.check_requires(signature, node, &mut requires_bindings, &mut counters)?
                    .requirement,
            )
        } else {
            None
        };

        let postcondition_selector = self.postcondition_selector_for_signature(signature)?;
        let postcondition_relation = if let Some(selector) = &postcondition_selector {
            let mut postcondition_bindings = parameter_bindings.clone();
            Some(self.check_postcondition_clause(
                signature,
                selector,
                &mut postcondition_bindings,
                &mut counters,
            )?)
        } else {
            None
        };

        bindings = parameter_bindings;
        let statements = self.tree.children_with(signature.node, Production::Stmt)?;
        let checked = self.check_block(
            signature,
            &statements,
            &mut bindings,
            &mut counters,
            ControlScope {
                loops: &[],
                give_context: None,
            },
        )?;
        if checked.can_continue {
            return Err(CheckStop::source_issue(SemanticIssue {
                rule: SemanticRule::Fn1,
                location: SemanticLocation::SourceNode(
                    self.tree.path(signature.node)?.clone(),
                    self.tree.closing_brace_coordinate(signature.node)?,
                ),
                kind: SemanticIssueKind::FunctionFallthrough,
            }));
        }
        // The exhibited row is the union of exactly two contributions
        // [EFF-2]: the syntactic contribution of the body and the release
        // contribution of every compiler-derived release recorded on a normal
        // body edge [STOR-3]. A requirement is a signature obligation, not an
        // executed declaration occurrence.
        let syntactic = checked.effects.clone();
        let mut release_sites = Vec::new();
        self.collect_release_sites(&checked.statements, &mut release_sites)?;
        let mut release = EffectSet::NONE;
        for site in &release_sites {
            release = release.union(site.effects.clone());
        }
        let exhibited = syntactic.clone().union(release);
        if exhibited != signature.declared_effects {
            // A category contributed only by the release contribution has
            // no offending source occurrence; the diagnostic renders the
            // owner whose release contributed it, selected by the
            // deterministic traversal that collected the sites.
            let release_only =
                |exhibited_category: bool, declared_category: bool, syntactic_category: bool| {
                    exhibited_category && !declared_category && !syntactic_category
                };
            let undeclared_external = release_only(
                exhibited.external,
                signature.declared_effects.external,
                syntactic.external,
            );
            let undeclared_blocks = release_only(
                exhibited.blocks,
                signature.declared_effects.blocks,
                syntactic.blocks,
            );
            if undeclared_external || undeclared_blocks {
                let owner = release_sites
                    .iter()
                    .find(|site| {
                        (undeclared_external && site.effects.external)
                            || (undeclared_blocks && site.effects.blocks)
                    })
                    .map(|site| match &site.owner {
                        cleanup::ReleaseOwner::Binding(binding) => binding_names
                            .get(binding.0 as usize)
                            .cloned()
                            .unwrap_or_else(|| "<unnamed owner>".to_owned()),
                        cleanup::ReleaseOwner::ExpressionResult => {
                            "<discarded expression result>".to_owned()
                        }
                    })
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                return self.issue_node(
                    SemanticRule::Eff2,
                    signature.effects_node,
                    SemanticIssueKind::ReleaseEffectMismatch {
                        owner,
                        mechanical_fix: "declare the release effects of every resource this function may release, or move the owner out",
                    },
                );
            }
            return self.issue_node(
                SemanticRule::Eff2,
                signature.effects_node,
                SemanticIssueKind::EffectMismatch,
            );
        }
        let postcondition = match (postcondition_selector, postcondition_relation) {
            (Some(selector), Some(relation)) if signature.substitution.is_concrete() => {
                Some(self.build_checked_postcondition(
                    signature,
                    &parameters,
                    selector,
                    relation,
                    &checked.statements,
                )?)
            }
            (Some(_), Some(_)) => None,
            (None, None) => None,
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        // TEMPORARY capability stop, judged only after every source rejection
        // above had its chance: arena-typed parameters check under their
        // ownership and [STOR-4] confinement rules, but the region-tied
        // allocation and release lowering is not implemented yet, so a clean
        // function that would carry an arena value to execution stops as an
        // explicit unsupported capability rather than lowering wrong code.
        for parameter in &signature.parameters {
            if self.arena_instance(parameter.ty)?.is_some() {
                return self.unsupported(
                    UnsupportedSemanticFeature::ArenaRuntime,
                    self.tree
                        .node_with_path(&parameter.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                );
            }
        }
        let function = CheckedFunction {
            id: signature.id,
            declaration: signature.declaration,
            name: signature.name.clone(),
            symbol: signature.symbol.clone(),
            deny_claims_marker: signature.deny_claims_marker.clone(),
            parameters,
            result_mode: signature.result_mode,
            result: signature.result,
            slice_return_ceiling: signature.slice_return_ceiling.clone(),
            declared_traps: signature.declared_effects.traps,
            declared_allocates_heap: signature.declared_effects.allocates_heap,
            requirement,
            postcondition,
            body: checked.statements,
            entailment: super::entailment::FunctionEntailment::default(),
        };
        Ok(CheckedFunctionInventory {
            function,
            binding_names,
        })
    }

    fn statements_contain_value_if(statements: &[CheckedStatement]) -> bool {
        statements.iter().any(|statement| match statement {
            CheckedStatement::ValueMatchLet { kind, arms, .. } => {
                *kind == ValueInitializerKind::ValueIf
                    || arms
                        .iter()
                        .any(|arm| Self::statements_contain_value_if(&arm.body))
            }
            CheckedStatement::Match { arms, .. } => arms
                .iter()
                .any(|arm| Self::statements_contain_value_if(&arm.body)),
            CheckedStatement::Loop { body, .. }
            | CheckedStatement::CountedRange { body, .. }
            | CheckedStatement::Region { body, .. } => Self::statements_contain_value_if(body),
            CheckedStatement::Let { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Check { .. }
            | CheckedStatement::Claim { .. }
            | CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => false,
        })
    }

    /// Runs acceptance-bearing entailment only after phase A has completed
    /// every concrete function. All summaries are derived before deterministic
    /// dense-function-order rejection, so the analysis itself never observes an
    /// inventory truncated by an earlier diagnostic.
    fn analyze_function_inventory(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        main: FunctionId,
    ) -> Result<PostconditionSchedule, CheckStop> {
        // The [ENT] engine is acceptance-bearing [ENT-1]: it computes the
        // closed fact states, obligation and ordinary-call goal dispositions,
        // and claim lifecycle dispositions. The first offending OP-4, FN-8,
        // or CLM-2 node in document/rule order is cited; redundancy advisories
        // never reject and are collected at the program level.
        let mut schedule =
            postcondition_schedule(functions.iter().map(|checked| &checked.function))
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if schedule.components.is_empty() {
            for checked in &mut *functions {
                let context = EntailmentContext {
                    callees,
                    constants: &self.checked_constants,
                    constant_ids: &self.constants,
                    nominals: &self.nominals,
                    verified_postconditions: &[],
                    verified_postcondition_proofs: &[],
                    binding_names: &checked.binding_names,
                    marked_program_start: checked.function.id == main
                        && checked.function.deny_claims_marker.is_some()
                        && checked.function.requirement.is_some(),
                };
                checked.function.entailment = if optimistic_batch {
                    analyze_function_candidate(&checked.function, &context)
                } else {
                    analyze_function(&checked.function, &context)
                };
            }
        } else {
            for component in &mut schedule.components {
                for function in &component.functions {
                    let function_index = function.0 as usize;
                    let verified_postconditions = functions
                        .iter()
                        .map(|checked| {
                            checked
                                .function
                                .entailment
                                .postcondition
                                .as_ref()
                                .and_then(|proof| proof.summary.as_ref())
                                .filter(|summary| summary.component < component.ordinal)
                                .and(checked.function.postcondition.as_ref())
                        })
                        .collect::<Vec<_>>();
                    let verified_postcondition_proofs = functions
                        .iter()
                        .map(|checked| {
                            checked
                                .function
                                .entailment
                                .postcondition
                                .as_ref()
                                .filter(|proof| {
                                    proof.summary.as_ref().is_some_and(|summary| {
                                        summary.component < component.ordinal
                                    })
                                })
                        })
                        .collect::<Vec<_>>();
                    let checked = functions
                        .get(function_index)
                        .filter(|checked| checked.function.id == *function)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let entailment = analyze_function_candidate(
                        &checked.function,
                        &EntailmentContext {
                            callees,
                            constants: &self.checked_constants,
                            constant_ids: &self.constants,
                            nominals: &self.nominals,
                            verified_postconditions: &verified_postconditions,
                            verified_postcondition_proofs: &verified_postcondition_proofs,
                            binding_names: &checked.binding_names,
                            marked_program_start: checked.function.id == main
                                && checked.function.deny_claims_marker.is_some()
                                && checked.function.requirement.is_some(),
                        },
                    );
                    drop(verified_postconditions);
                    drop(verified_postcondition_proofs);
                    functions[function_index].function.entailment = entailment;
                }

                let publish = component.functions.iter().all(|function| {
                    let checked = &functions[function.0 as usize].function;
                    checked.postcondition.is_none()
                        || checked
                            .entailment
                            .postcondition
                            .as_ref()
                            .is_some_and(|proof| proof.complete.discharged)
                });
                if publish {
                    for function in &component.functions {
                        let checked = &mut functions[function.0 as usize].function;
                        let Some(proof) = &mut checked.entailment.postcondition else {
                            continue;
                        };
                        let summary = VerifiedPostconditionSummary {
                            function: *function,
                            block: proof.block.clone(),
                            relation_ordinal: 0,
                            component: component.ordinal,
                        };
                        proof.summary = Some(summary.clone());
                        component.summaries.push(summary);
                    }
                }
            }
        }
        if self.reject_entailment {
            for checked in functions {
                self.entailment_rejection(&checked.function)?;
            }
        }
        Ok(schedule)
    }

    /// Instantiates every retained user-call requirement from the complete
    /// phase-A inventory. The subsequent entailment step discharges these
    /// exact goals in each caller's pre-transfer state.
    fn install_call_requirements(
        &self,
        functions: &mut [CheckedFunctionInventory],
    ) -> Result<(), CheckStop> {
        let requirements = functions
            .iter()
            .map(|checked| checked.function.requirement.clone())
            .collect::<Vec<_>>();
        for checked in functions {
            self.install_statement_call_requirements(&mut checked.function.body, &requirements)?;
        }
        Ok(())
    }

    fn install_statement_call_requirements(
        &self,
        statements: &mut [CheckedStatement],
        requirements: &[Option<CheckedRequirement>],
    ) -> Result<(), CheckStop> {
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
                    self.install_expression_call_requirements(value, requirements)?;
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => {
                    self.install_expression_call_requirements(scrutinee, requirements)?;
                }
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => self
                            .install_expression_call_requirements(
                                &mut target.offset,
                                requirements,
                            )?,
                        CheckedSetTarget::BufferIndex(target) => self
                            .install_expression_call_requirements(
                                &mut target.offset,
                                requirements,
                            )?,
                    }
                    self.install_expression_call_requirements(value, requirements)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    self.install_expression_call_requirements(scrutinee, requirements)?;
                    for arm in arms {
                        self.install_statement_call_requirements(&mut arm.body, requirements)?;
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.install_statement_call_requirements(body, requirements)?;
                }
                CheckedStatement::CountedRange {
                    lower, upper, body, ..
                } => {
                    self.install_expression_call_requirements(lower, requirements)?;
                    self.install_expression_call_requirements(upper, requirements)?;
                    self.install_statement_call_requirements(body, requirements)?;
                }
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn install_expression_call_requirements(
        &self,
        expression: &mut CheckedExpression,
        requirements: &[Option<CheckedRequirement>],
    ) -> Result<(), CheckStop> {
        match expression {
            CheckedExpression::UserCall {
                function,
                arguments,
                goal_arguments,
                goal_regions,
                requirement,
                ..
            } => {
                for argument in arguments {
                    self.install_expression_call_requirements(argument, requirements)?;
                }
                let signature = self
                    .signatures
                    .get(function.0 as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let boundary = requirements
                    .get(function.0 as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                *requirement = match boundary {
                    Some(boundary) => Some(Box::new(CheckedCallRequirement {
                        final_check: boundary.trap.node_path.clone(),
                        goal: ConcreteGoal::new(self.instantiate_goal_expression(
                            &boundary.template.root,
                            signature,
                            goal_regions,
                            goal_arguments,
                        )?),
                    })),
                    None => None,
                };
            }
            CheckedExpression::SystemCall { arguments, .. }
            | CheckedExpression::IntegerOperation { arguments, .. }
            | CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. }
            | CheckedExpression::ConstructStruct {
                fields: arguments, ..
            }
            | CheckedExpression::ConstructEnum {
                fields: arguments, ..
            } => {
                for argument in arguments {
                    self.install_expression_call_requirements(argument, requirements)?;
                }
            }
            CheckedExpression::NumericConversion { value, .. }
            | CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::ArrayFill { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaNew { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.install_expression_call_requirements(value, requirements)?;
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::SliceIndex { offset, .. } => {
                self.install_expression_call_requirements(offset, requirements)?;
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                self.install_expression_call_requirements(length, requirements)?;
                self.install_expression_call_requirements(value, requirements)?;
            }
            CheckedExpression::BufferVacant { length, .. } => {
                self.install_expression_call_requirements(length, requirements)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceLength { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
            | CheckedExpression::DerefAddressed { .. }
            | CheckedExpression::Project { .. } => {}
        }
        Ok(())
    }

    fn instantiate_goal_expression(
        &self,
        expression: &GoalExpression,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
        arguments: &[GoalExpression],
    ) -> Result<GoalExpression, CheckStop> {
        match expression {
            GoalExpression::Datum(GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            }) => {
                let index = usize::try_from(*ordinal)
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                let parameter = signature
                    .parameters
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut image = arguments
                    .get(index)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let parameter_type =
                    self.instantiate_goal_type(parameter.ty, signature, regions)?;
                if image.ty() != parameter_type {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let remaining = if parameter.mode == CheckedMode::Own {
                    projections.as_slice()
                } else {
                    let Some((GoalProjection::Deref, remaining)) = projections.split_first() else {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    };
                    remaining
                };
                let final_type = self.instantiate_goal_type(*ty, signature, regions)?;
                for projection in remaining {
                    image = image
                        .with_projection(*projection, final_type)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                }
                if image.ty() != final_type {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(image)
            }
            GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) => Ok(GoalExpression::Datum(GoalDatum::NamedConst {
                declaration: *declaration,
                projections: projections.clone(),
                ty: self.instantiate_goal_type(*ty, signature, regions)?,
            })),
            GoalExpression::Datum(GoalDatum::Literal(value)) => Ok(GoalExpression::Datum(
                GoalDatum::Literal(self.instantiate_goal_value(value, signature, regions)?),
            )),
            GoalExpression::Datum(GoalDatum::Place { .. } | GoalDatum::EphemeralActual { .. }) => {
                Err(SemanticCompilerFailure::InvalidResolution.into())
            }
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments: operands,
            } => Ok(GoalExpression::Operation {
                row: self.instantiate_goal_operation(*row, signature, regions)?,
                type_arguments: type_arguments
                    .iter()
                    .map(|ty| self.instantiate_goal_type(*ty, signature, regions))
                    .collect::<Result<Vec<_>, _>>()?,
                const_arguments: const_arguments
                    .iter()
                    .map(|value| self.instantiate_goal_const(*value, signature))
                    .collect::<Result<Vec<_>, _>>()?,
                result: self.instantiate_goal_type(*result, signature, regions)?,
                arguments: operands
                    .iter()
                    .map(|operand| {
                        self.instantiate_goal_expression(operand, signature, regions, arguments)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }),
        }
    }

    fn instantiate_goal_operation(
        &self,
        operation: GoalOperation,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<GoalOperation, CheckStop> {
        Ok(match operation {
            GoalOperation::Integer {
                operation,
                operand_type,
            } => GoalOperation::Integer {
                operation,
                operand_type: self.instantiate_goal_type(operand_type, signature, regions)?,
            },
            GoalOperation::Float {
                operation,
                operand_type,
            } => GoalOperation::Float {
                operation,
                operand_type: self.instantiate_goal_type(operand_type, signature, regions)?,
            },
            GoalOperation::NumericConversion {
                source,
                destination,
            } => GoalOperation::NumericConversion {
                source,
                destination,
            },
            GoalOperation::Reinterpret {
                source,
                destination,
            } => GoalOperation::Reinterpret {
                source,
                destination,
            },
            GoalOperation::Boolean(operation) => GoalOperation::Boolean(operation),
            GoalOperation::EnumEquality {
                equal,
                operand_type,
            } => GoalOperation::EnumEquality {
                equal,
                operand_type: self.instantiate_goal_type(operand_type, signature, regions)?,
            },
            GoalOperation::ArrayFill { element, length } => GoalOperation::ArrayFill {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            GoalOperation::ArrayLength { element, length } => GoalOperation::ArrayLength {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            GoalOperation::BufferLength { element } => GoalOperation::BufferLength {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            GoalOperation::SliceLength { region, element } => GoalOperation::SliceLength {
                region: self.instantiate_goal_region(region, signature, regions)?,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
        })
    }

    fn instantiate_goal_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        Ok(match ty {
            CheckedType::Generic(declaration)
            | CheckedType::GenericInt(declaration)
            | CheckedType::GenericFloat(declaration) => signature
                .substitution
                .type_argument(declaration)
                .unwrap_or(ty),
            CheckedType::Array { element, length } => CheckedType::Array {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            CheckedType::Slice { region, element } => CheckedType::Slice {
                region: self.instantiate_goal_region(region, signature, regions)?,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            CheckedType::Buffer { element } => CheckedType::Buffer {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_)
            | CheckedType::Nominal(_) => ty,
        })
    }

    fn instantiate_goal_flat_element(
        &self,
        element: CheckedFlatElement,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedFlatElement, CheckStop> {
        let ty = self.instantiate_goal_type(element.ty(), signature, regions)?;
        Ok(match ty {
            CheckedType::Unit => CheckedFlatElement::Unit,
            CheckedType::Bool => CheckedFlatElement::Bool,
            CheckedType::Integer(ty) => CheckedFlatElement::Integer(ty),
            CheckedType::Float(ty) => CheckedFlatElement::Float(ty),
            CheckedType::GenericInt(declaration) => CheckedFlatElement::GenericInt(declaration),
            CheckedType::GenericFloat(declaration) => CheckedFlatElement::GenericFloat(declaration),
            CheckedType::Nominal(nominal) => {
                if self.nominal(nominal)?.is_copy() {
                    CheckedFlatElement::TagOnlyNominal(nominal)
                } else {
                    CheckedFlatElement::Nominal(nominal)
                }
            }
            CheckedType::Generic(_)
            | CheckedType::Array { .. }
            | CheckedType::Slice { .. }
            | CheckedType::Buffer { .. } => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        })
    }

    fn instantiate_goal_const(
        &self,
        value: CheckedConst,
        signature: &FunctionSignature,
    ) -> Result<CheckedConst, CheckStop> {
        Ok(match value {
            CheckedConst::Value(_) => value,
            CheckedConst::Parameter(declaration) => signature
                .substitution
                .const_argument(declaration)
                .unwrap_or(value),
            CheckedConst::Derived(id) => {
                let derived = self.derived_const(id)?;
                let left = self.instantiate_goal_const(derived.left, signature)?;
                let right = self.instantiate_goal_const(derived.right, signature)?;
                // The owning instance body was accepted, so the same
                // evaluation already succeeded at its source node; a failure
                // here is a trusted-invariant breach, not a source verdict.
                self.combine_const(derived.operation, left, right)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
            }
        })
    }

    /// Returns the interned symbolic const operation behind `id`.
    pub(super) fn derived_const(&self, id: DerivedConstId) -> Result<DerivedConst, CheckStop> {
        self.derived_consts
            .borrow()
            .get(id.0 as usize)
            .copied()
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    /// Combines two const operands under one const operation.
    ///
    /// Two concrete operands evaluate immediately in the u64 const-eval
    /// domain, and `None` reports the const-eval overflow policy's rejection
    /// (a result outside the domain or a zero divisor). A symbolic operand
    /// hash-conses the operation instead, so a symbolic const never fails
    /// here and always has one interned identity.
    pub(super) fn combine_const(
        &self,
        operation: super::model::ConstOperation,
        left: CheckedConst,
        right: CheckedConst,
    ) -> Option<CheckedConst> {
        if let (CheckedConst::Value(left), CheckedConst::Value(right)) = (left, right) {
            return evaluate_const_operation(operation, left, right).map(CheckedConst::Value);
        }
        let derived = DerivedConst {
            operation,
            left,
            right,
        };
        let mut table = self.derived_consts.borrow_mut();
        let index = table
            .iter()
            .position(|entry| *entry == derived)
            .unwrap_or_else(|| {
                table.push(derived);
                table.len() - 1
            });
        u32::try_from(index)
            .ok()
            .map(|index| CheckedConst::Derived(DerivedConstId(index)))
    }

    fn instantiate_goal_region(
        &self,
        region: DeclarationId,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<DeclarationId, CheckStop> {
        let Some(index) = signature
            .region_parameters
            .iter()
            .position(|formal| *formal == region)
        else {
            return Ok(region);
        };
        regions
            .get(index)
            .copied()
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    fn instantiate_goal_value(
        &self,
        value: &CheckedValue,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedValue, CheckStop> {
        Ok(match value {
            CheckedValue::Unit => CheckedValue::Unit,
            CheckedValue::Bool(value) => CheckedValue::Bool(*value),
            CheckedValue::Integer { ty, bits } => CheckedValue::Integer {
                ty: *ty,
                bits: *bits,
            },
            CheckedValue::Float { ty, bits } => CheckedValue::Float {
                ty: *ty,
                bits: *bits,
            },
            CheckedValue::NumericIdentity { ty, one } => {
                match self.instantiate_goal_type(*ty, signature, regions)? {
                    CheckedType::Integer(ty) => CheckedValue::Integer {
                        ty,
                        bits: u64::from(*one),
                    },
                    CheckedType::Float(super::model::FloatType::F32) => CheckedValue::Float {
                        ty: super::model::FloatType::F32,
                        bits: if *one { 0x3f80_0000 } else { 0 },
                    },
                    CheckedType::Float(super::model::FloatType::F64) => CheckedValue::Float {
                        ty: super::model::FloatType::F64,
                        bits: if *one { 0x3ff0_0000_0000_0000 } else { 0 },
                    },
                    ty @ (CheckedType::GenericInt(_) | CheckedType::GenericFloat(_)) => {
                        CheckedValue::NumericIdentity { ty, one: *one }
                    }
                    _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                }
            }
            CheckedValue::Array { ty, elements } => CheckedValue::Array {
                ty: self.instantiate_goal_type(*ty, signature, regions)?,
                elements: elements
                    .iter()
                    .map(|element| self.instantiate_goal_value(element, signature, regions))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            CheckedValue::Struct { ty, fields } => CheckedValue::Struct {
                ty: self.instantiate_goal_type(*ty, signature, regions)?,
                fields: fields
                    .iter()
                    .map(|field| self.instantiate_goal_value(field, signature, regions))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
    }

    fn provenance_selector(selector: DatumSelector) -> crate::ProvenanceDatumSelector {
        match selector {
            DatumSelector::Plain => crate::ProvenanceDatumSelector::Plain,
            DatumSelector::EnumPayload { variant, field } => {
                crate::ProvenanceDatumSelector::EnumPayload { variant, field }
            }
        }
    }

    fn provenance_datum(
        datum: super::provenance::ParameterDatum,
    ) -> crate::ProvenanceParameterDatumDetail {
        crate::ProvenanceParameterDatumDetail {
            ordinal: datum.ordinal,
            selector: Self::provenance_selector(datum.selector),
        }
    }

    fn provenance_carrier(
        &self,
        route: &super::provenance::CarrierRoute,
    ) -> Result<Vec<crate::ProvenanceCarrierStepDetail>, CheckStop> {
        route
            .steps()
            .iter()
            .map(|step| {
                let node = self
                    .tree
                    .node_with_path(&step.path)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let write_context = step
                    .write_context
                    .as_ref()
                    .map(|context| -> Result<_, CheckStop> {
                        let actual = self
                            .tree
                            .node_with_path(&context.actual)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        Ok(crate::ProvenanceWriteContextDetail {
                            parameter: context.parameter,
                            actual: context.actual.clone(),
                            actual_coordinate: self.tree.coordinate(actual)?,
                        })
                    })
                    .transpose()?;
                Ok(crate::ProvenanceCarrierStepDetail {
                    path: step.path.clone(),
                    selector: Self::provenance_selector(step.selector),
                    call_role: step.call_role.map(|role| match role {
                        super::provenance::CarrierCallRole::SystemResult => {
                            crate::ProvenanceCarrierCallRole::SystemResult
                        }
                        super::provenance::CarrierCallRole::SystemWrite => {
                            crate::ProvenanceCarrierCallRole::SystemWrite
                        }
                        super::provenance::CarrierCallRole::UserResult => {
                            crate::ProvenanceCarrierCallRole::UserResult
                        }
                        super::provenance::CarrierCallRole::UserWrite => {
                            crate::ProvenanceCarrierCallRole::UserWrite
                        }
                        super::provenance::CarrierCallRole::UserSubstitution => {
                            crate::ProvenanceCarrierCallRole::UserSubstitution
                        }
                    }),
                    write_context,
                    coordinate: self.tree.coordinate(node)?,
                })
            })
            .collect()
    }

    fn provenance_residual(
        provenance: &ProvenanceMetadata,
        leaf: &super::provenance::ProtectedLeaf,
    ) -> Option<String> {
        let residual = |views: &[super::entailment::FunctionEntailmentView]| {
            views
                .get(leaf.function.0 as usize)
                .and_then(|view| {
                    view.obligations
                        .iter()
                        .find(|outcome| outcome.node_path == leaf.obligation)
                })
                .and_then(|outcome| outcome.residual.clone())
        };
        residual(&provenance.s4_blinded).or_else(|| residual(&provenance.unasserted))
    }

    fn provenance_demand_state(
        functions: &[CheckedFunction],
        state: &super::provenance::DemandState,
    ) -> Result<crate::ProvenanceDemandStateDetail, CheckStop> {
        let (demand_kind, function, parameter, requirement, leaf) = match state {
            super::provenance::DemandState::Direct {
                function,
                subject,
                leaf,
            } => (
                crate::ProvenanceDemandKind::Direct,
                *function,
                *subject,
                None,
                leaf,
            ),
            super::provenance::DemandState::Bridge {
                requirement,
                subject,
                leaf,
            } => (
                crate::ProvenanceDemandKind::RequirementBridge,
                requirement.function,
                *subject,
                Some(requirement),
                leaf,
            ),
        };
        let owner = functions
            .get(function.0 as usize)
            .filter(|candidate| candidate.id == function)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let protected = functions
            .get(leaf.function.0 as usize)
            .filter(|candidate| candidate.id == leaf.function)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(crate::ProvenanceDemandStateDetail {
            demand_kind,
            function: owner.symbol.clone(),
            parameter: Self::provenance_datum(parameter),
            requirement: requirement.map(|requirement| requirement.final_check.clone()),
            requirement_conjunct: requirement.map(|requirement| requirement.conjunct),
            protected_function: protected.symbol.clone(),
            protected_leaf: leaf.obligation.clone(),
            protected_conjunct: leaf.conjunct,
        })
    }

    fn provenance_boundary(
        functions: &[CheckedFunction],
        boundary: &super::provenance::DemandBoundary,
    ) -> Result<crate::ProvenanceBoundaryDetail, CheckStop> {
        Ok(crate::ProvenanceBoundaryDetail {
            call: boundary.call.clone(),
            argument_node: boundary.argument_node.clone(),
            argument: boundary.argument,
            callee: Self::provenance_demand_state(functions, &boundary.callee)?,
            caller_continuation: boundary
                .caller_continuation
                .as_ref()
                .map(|state| Self::provenance_demand_state(functions, state))
                .transpose()?,
        })
    }

    fn provenance_target_detail(
        &self,
        functions: &[CheckedFunction],
        provenance: &ProvenanceMetadata,
        target: &ProvenanceTarget,
    ) -> Result<crate::ProvenanceTargetDetail, CheckStop> {
        let protected = functions
            .get(target.leaf.function.0 as usize)
            .filter(|candidate| candidate.id == target.leaf.function)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let residual = Self::provenance_residual(provenance, &target.leaf)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (demand_kind, target_repair) = match target.kind {
            InternalDemandKind::Direct => (
                crate::ProvenanceDemandKind::Direct,
                "add a dominating real value branch in the protected leaf's owning body and take the domain outcome on its false edge",
            ),
            InternalDemandKind::Bridge => (
                crate::ProvenanceDemandKind::RequirementBridge,
                "add a real value branch in the rejecting caller that establishes the complete bridged call goal in the unasserted state",
            ),
        };
        let requirement_function = target
            .requirement
            .as_ref()
            .map(|requirement| {
                functions
                    .get(requirement.function.0 as usize)
                    .filter(|function| function.id == requirement.function)
                    .map(|function| function.symbol.clone())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)
            })
            .transpose()?;
        let mut witness = vec![target.leaf.obligation.clone()];
        for boundary in &target.boundaries {
            if let super::provenance::DemandState::Bridge { requirement, .. } = &boundary.callee {
                witness.push(requirement.final_check.clone());
            }
            witness.push(boundary.call.clone());
            witness.push(boundary.argument_node.clone());
        }
        witness.extend(target.carrier.paths());
        let carrier = self.provenance_carrier(&target.carrier)?;
        let origin_coordinate = carrier
            .last()
            .map(|step| step.coordinate)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(crate::ProvenanceTargetDetail {
            demand_kind,
            callee_parameter: Some(Self::provenance_datum(target.callee_subject)),
            protected_function: protected.symbol.clone(),
            protected_leaf: target.leaf.obligation.clone(),
            protected_conjunct: target.leaf.conjunct,
            requirement_function,
            requirement: target
                .requirement
                .as_ref()
                .map(|requirement| requirement.final_check.clone()),
            requirement_conjunct: target
                .requirement
                .as_ref()
                .map(|requirement| requirement.conjunct),
            local_bridge_predecessor: None,
            residual,
            companion_parameter_datums: target
                .companions
                .datums
                .iter()
                .copied()
                .map(Self::provenance_datum)
                .collect(),
            boundaries: target
                .boundaries
                .iter()
                .map(|boundary| Self::provenance_boundary(functions, boundary))
                .collect::<Result<Vec<_>, _>>()?,
            carrier,
            origin_coordinate,
            witness,
            target_repair,
        })
    }

    /// Applies PRV-2/PRV-3 only after every OP-4/FN-8 base judgment has
    /// succeeded.  Events are already coalesced and deterministically ordered
    /// by the two-stratum provenance analysis.
    fn provenance_rejection(
        &self,
        functions: &[CheckedFunction],
        provenance: &ProvenanceMetadata,
        failures: &ProvenanceFailures,
    ) -> Result<(), CheckStop> {
        enum Rejection<'metadata> {
            Local(
                &'metadata super::provenance::ProtectedLeaf,
                &'metadata super::provenance::ProvenanceDependency,
                Option<&'metadata super::provenance::RequirementOccurrence>,
                &'metadata super::provenance::CarrierRoute,
            ),
            Call(&'metadata super::provenance::ProvenanceCallEvent),
        }

        impl Rejection<'_> {
            fn node_path(&self) -> &crate::NodePath {
                match self {
                    Self::Local(leaf, _, _, _) => &leaf.obligation,
                    Self::Call(event) => &event.argument_node,
                }
            }

            const fn rule(&self) -> SemanticRule {
                match self {
                    Self::Local(_, _, _, _) => SemanticRule::Prv3,
                    Self::Call(_) => SemanticRule::Prv2,
                }
            }
        }

        let local =
            failures
                .local_rejections
                .iter()
                .map(|(leaf, dependency, requirement, carrier)| {
                    Rejection::Local(leaf, dependency, requirement.as_ref(), carrier)
                });
        let calls = failures.call_events.iter().map(Rejection::Call);
        let rejection = local.chain(calls).min_by(|left, right| {
            left.node_path()
                .components()
                .cmp(right.node_path().components())
                .then_with(|| {
                    left.rule()
                        .definition_rank()
                        .cmp(&right.rule().definition_rank())
                })
        });
        let Some(rejection) = rejection else {
            return Ok(());
        };
        let node = self
            .tree
            .node_with_path(rejection.node_path())
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let location = SemanticLocation::SourceNode(
            rejection.node_path().clone(),
            self.tree.coordinate(node)?,
        );
        let restructure_alternative = "restructure the explicit dataflow so the external value no longer reaches the constrained-subject position";
        match rejection {
            Rejection::Local(leaf, dependency, requirement, carrier) => {
                let function = functions
                    .get(leaf.function.0 as usize)
                    .filter(|function| function.id == leaf.function)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let residual = Self::provenance_residual(provenance, leaf)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut witness = vec![leaf.obligation.clone()];
                if let Some(requirement) = requirement {
                    witness.push(requirement.final_check.clone());
                }
                witness.extend(carrier.paths());
                let carrier = self.provenance_carrier(carrier)?;
                let origin_coordinate = carrier
                    .last()
                    .map(|step| step.coordinate)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let requirement_function = requirement
                    .map(|requirement| {
                        functions
                            .get(requirement.function.0 as usize)
                            .filter(|function| function.id == requirement.function)
                            .map(|function| function.symbol.clone())
                            .ok_or(SemanticCompilerFailure::InvalidResolution)
                    })
                    .transpose()?;
                Err(CheckStop::source_issue(SemanticIssue {
                    rule: SemanticRule::Prv3,
                    location,
                    kind: SemanticIssueKind::ExternalProtectedSubject(Box::new(
                        crate::ProvenanceGateDetail {
                            targets: vec![crate::ProvenanceTargetDetail {
                                demand_kind: crate::ProvenanceDemandKind::LocalLeaf,
                                callee_parameter: None,
                                protected_function: function.symbol.clone(),
                                protected_leaf: leaf.obligation.clone(),
                                protected_conjunct: leaf.conjunct,
                                requirement_function,
                                requirement: requirement
                                    .map(|requirement| requirement.final_check.clone()),
                                requirement_conjunct: requirement
                                    .map(|requirement| requirement.conjunct),
                                local_bridge_predecessor: requirement
                                    .map(|_| crate::ProvenanceLocalBridgePredecessor::Local),
                                residual,
                                companion_parameter_datums: dependency
                                    .parameters
                                    .datums
                                    .iter()
                                    .copied()
                                    .map(Self::provenance_datum)
                                    .collect(),
                                boundaries: Vec::new(),
                                carrier,
                                origin_coordinate,
                                witness,
                                target_repair: "add a dominating real value branch in this body and take the domain outcome on its false edge",
                            }],
                            selected_target: 0,
                            restructure_alternative,
                        },
                    )),
                }))
            }
            Rejection::Call(event) => {
                let targets = event
                    .targets
                    .iter()
                    .map(|target| self.provenance_target_detail(functions, provenance, target))
                    .collect::<Result<Vec<_>, _>>()?;
                let selected_target = usize::try_from(event.selected_target)
                    .map_err(|_| SemanticCompilerFailure::InvalidResolution)?;
                if targets.is_empty() || selected_target >= targets.len() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Err(CheckStop::source_issue(SemanticIssue {
                    rule: SemanticRule::Prv2,
                    location,
                    kind: SemanticIssueKind::ExternalProtectedCallArgument(Box::new(
                        crate::ProvenanceGateDetail {
                            targets,
                            selected_target: event.selected_target,
                            restructure_alternative,
                        },
                    )),
                }))
            }
        }
    }

    /// Rejects a checked function whose entailment summary contains an
    /// undischarged bounds obligation [OP-4], ordinary-call goal [FN-8],
    /// refuted claim [CLM-2], or complete-view selected return [FN-9]. The
    /// ordinary judgments required to reach a return proof are selected
    /// first; only a function with no such rejection publishes its first
    /// source-ordered complete FN-9 failure.
    fn entailment_rejection(&self, function: &CheckedFunction) -> Result<(), CheckStop> {
        enum Rejection<'outcome> {
            Obligation(&'outcome super::entailment::ObligationOutcome),
            Call(&'outcome super::entailment::CallGoalOutcome),
            Claim {
                outcome: &'outcome super::entailment::ClaimOutcome,
                predicate: &'outcome str,
                negation: &'outcome str,
            },
        }

        impl Rejection<'_> {
            fn node_path(&self) -> &crate::NodePath {
                match self {
                    Self::Obligation(outcome) => &outcome.node_path,
                    Self::Call(outcome) => &outcome.node_path,
                    Self::Claim { outcome, .. } => &outcome.node_path,
                }
            }

            const fn rule(&self) -> SemanticRule {
                match self {
                    Self::Obligation(outcome) => match outcome.family {
                        super::entailment::ObligationFamily::Bounds => SemanticRule::Op4,
                        super::entailment::ObligationFamily::Overflow
                        | super::entailment::ObligationFamily::Division => SemanticRule::Op2,
                    },
                    Self::Call(_) => SemanticRule::Fn8,
                    Self::Claim { .. } => SemanticRule::Clm2,
                }
            }
        }

        let obligation = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| !outcome.discharged)
            .map(Rejection::Obligation);
        let call = function
            .entailment
            .call_goals
            .iter()
            .filter(|outcome| outcome.disposition != CallGoalDisposition::Discharged)
            .map(Rejection::Call);
        let refuted =
            function
                .entailment
                .claims
                .iter()
                .filter_map(|outcome| match &outcome.disposition {
                    super::entailment::ClaimDisposition::Refuted {
                        predicate,
                        negation,
                    } => Some(Rejection::Claim {
                        outcome,
                        predicate,
                        negation,
                    }),
                    _ => None,
                });
        let rejection = obligation.chain(call).chain(refuted).min_by(|left, right| {
            left.node_path()
                .components()
                .cmp(right.node_path().components())
                .then_with(|| {
                    left.rule()
                        .definition_rank()
                        .cmp(&right.rule().definition_rank())
                })
        });
        if let Some(rejection) = rejection {
            return match rejection {
                Rejection::Obligation(outcome) => {
                    let residual = outcome
                        .residual
                        .clone()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let node = self
                        .tree
                        .node_with_path(&outcome.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let location = SemanticLocation::SourceNode(
                        outcome.node_path.clone(),
                        self.tree.coordinate(node)?,
                    );
                    Err(CheckStop::source_issue(match outcome.family {
                        super::entailment::ObligationFamily::Bounds => SemanticIssue {
                            rule: SemanticRule::Op4,
                            location,
                            kind: SemanticIssueKind::UndischargedBoundsObligation {
                                residual,
                                mechanical_fix: "add a dominating `claim` of the residual or a dominating branch establishing it",
                            },
                        },
                        super::entailment::ObligationFamily::Overflow => SemanticIssue {
                            rule: SemanticRule::Op2,
                            location,
                            kind: SemanticIssueKind::UndischargedOverflowObligation {
                                residual,
                                mechanical_fix: "add a dominating `claim` of the residual or a dominating branch establishing it, or respell the operation `wrap`, `checked`, or `sat`",
                            },
                        },
                        super::entailment::ObligationFamily::Division => SemanticIssue {
                            rule: SemanticRule::Op2,
                            location,
                            kind: SemanticIssueKind::UndischargedDivisionObligation {
                                residual,
                                mechanical_fix: "add a dominating `claim` of the residual or a dominating branch establishing it, or respell the operation `checked`",
                            },
                        },
                    }))
                }
                Rejection::Call(outcome) => {
                    let node = self
                        .tree
                        .node_with_path(&outcome.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let signature = self
                        .signatures
                        .get(outcome.callee.0 as usize)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let disposition = match outcome.disposition {
                        CallGoalDisposition::Discharged => {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        CallGoalDisposition::Refuted => crate::CallRequirementDisposition::Refuted,
                        CallGoalDisposition::Unproved => {
                            crate::CallRequirementDisposition::Unproved
                        }
                    };
                    let mechanical_fix = if first_ephemeral_argument(&outcome.goal.root).is_some() {
                        "bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow"
                    } else {
                        "establish the complete callee requirement with one dominating branch, check, or claim before the call"
                    };
                    Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Fn8,
                        location: SemanticLocation::SourceNode(
                            outcome.node_path.clone(),
                            self.tree.coordinate(node)?,
                        ),
                        kind: SemanticIssueKind::UndischargedCallRequirement(Box::new(
                            crate::UndischargedCallRequirementDetail {
                                concrete_callee: signature.symbol.clone(),
                                final_check: outcome.final_check.clone(),
                                instantiated_goal: render_goal(&outcome.goal.root),
                                disposition,
                                mechanical_fix,
                            },
                        )),
                    }))
                }
                Rejection::Claim {
                    outcome,
                    predicate,
                    negation,
                } => {
                    let node = self
                        .tree
                        .node_with_path(&outcome.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Clm2,
                        location: SemanticLocation::SourceNode(
                            outcome.node_path.clone(),
                            self.tree.coordinate(node)?,
                        ),
                        kind: SemanticIssueKind::RefutedClaim(Box::new(
                            crate::RefutedClaimDetail {
                                name: outcome.name.clone(),
                                predicate: predicate.to_owned(),
                                negation: negation.to_owned(),
                            },
                        )),
                    }))
                }
            };
        }

        let Some(proof) = &function.entailment.postcondition else {
            return Ok(());
        };
        let Some(exit) = proof.exits.iter().find(|exit| {
            exit.complete.disposition != super::entailment::PostconditionDisposition::Discharged
        }) else {
            return Ok(());
        };
        let disposition = match exit.complete.disposition {
            super::entailment::PostconditionDisposition::Discharged => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            super::entailment::PostconditionDisposition::Refuted => {
                crate::PostconditionProofDisposition::Refuted
            }
            super::entailment::PostconditionDisposition::Unproved => {
                crate::PostconditionProofDisposition::Unproved
            }
        };
        let node = self
            .tree
            .node_with_path(&exit.statement)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Err(CheckStop::source_issue(SemanticIssue {
            rule: SemanticRule::Fn9,
            location: SemanticLocation::SourceNode(
                exit.statement.clone(),
                self.tree.coordinate(node)?,
            ),
            kind: SemanticIssueKind::UndischargedPostcondition(Box::new(
                crate::UndischargedPostconditionDetail {
                    concrete_function: function.symbol.clone(),
                    postcondition: proof.block.clone(),
                    conjunct: 0,
                    selector: proof.selector.clone(),
                    relation: exit.residual.clone(),
                    disposition,
                },
            )),
        }))
    }
}
