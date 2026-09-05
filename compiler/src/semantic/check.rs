mod borrows;
mod cleanup;
mod confinement;
mod contracts;
mod control;
mod ensures;
mod entry_form;
pub(in crate::semantic::check) mod expressions;
mod floats;
mod generics;
mod linearity;
mod nominal_instances;
mod nominals;
pub(crate) mod publication;
mod requires;
mod result_state_origin;
mod support;
mod types;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, NodePath, Production, ResolvedSyntaxUnit,
    SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticOutcome,
    SemanticRule, StaticObligationDisposition, UnsupportedSemanticFeature,
};

use super::entailment::{
    CallGoalDisposition, EntailmentCallee, EntailmentContext, PostconditionSchedule,
    VerifiedPostconditionSummary, analyze_function, analyze_function_candidate,
    finalize_function_entailment, postcondition_schedule,
};
use super::goal::{
    CheckedCallRequirement, CheckedRequirement, ConcreteGoal, GoalDatum, GoalExpression,
    GoalOperation, GoalProjection, first_ephemeral_argument,
};
use super::model::{
    BindingId, CheckedConst, CheckedConstant, CheckedConstantId, CheckedContract, CheckedElement,
    CheckedExpression, CheckedFlatElement, CheckedFunction, CheckedGenericRequirement, CheckedMode,
    CheckedNominal, CheckedNominalKind, CheckedParameter, CheckedProgramData,
    CheckedResultStateOrigin, CheckedSetTarget, CheckedSliceOrigin, CheckedStateOrigins,
    CheckedStatement, CheckedType, CheckedValue, DerivedConst, DerivedConstId, FunctionId,
    NominalId, ValueInitializerKind, evaluate_const_operation,
};
use super::permission::{PermissionSignature, analyze_permission};
use super::permission_ledger::{LedgerSource, render_ledger};
use super::postcondition::CheckedPostconditionSelector;
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use borrows::{AccessKind, ResolvedPlace};
use borrows::{BorrowInfo, BorrowKind, SliceInfo, SliceLoan};
use control::{ControlCounters, ControlScope};
use generics::{GenericParameter, GenericSubstitution, PendingGenericRequirement};

/// The syntax tree, as the permission ledger's citations reach it.
struct PermissionLedgerSource<'view, 'unit, 'classified, 'lexed, 'source> {
    tree: &'view TreeView<'unit, 'classified, 'lexed, 'source>,
}

impl LedgerSource for PermissionLedgerSource<'_, '_, '_, '_, '_> {
    type Error = SemanticCompilerFailure;

    fn location(&self, path: &NodePath) -> Result<(String, u64), Self::Error> {
        self.tree.source_line(path)
    }

    fn spelling(&self, path: &NodePath) -> Result<String, Self::Error> {
        self.tree.path_spelling(path)
    }
}

#[derive(Clone)]
struct ParameterSignature {
    declaration: DeclarationId,
    node_path: crate::NodePath,
    name: String,
    mode: CheckedMode,
    ty: CheckedType,
}

/// One declared result ordinal of a callable boundary [GRAM-2, FN-1].
#[derive(Clone)]
struct ResultSignature {
    mode: CheckedMode,
    ty: CheckedType,
    /// The ordinal's complete `rtype`, for a diagnostic at the declaration.
    rtype: NodeId,
}

#[derive(Clone)]
struct FunctionSignature {
    id: FunctionId,
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    symbol: String,
    /// Every formal region of the callable: the written `region_params`
    /// first, in their written order, then the regions [FORM-8] leaves
    /// unwritten at a parameter position, in parameter order.
    region_parameters: Vec<DeclarationId>,
    /// How many leading `region_parameters` entries the declaration writes.
    written_regions: usize,
    parameters: Vec<ParameterSignature>,
    /// The callable result [FN-1]: the written result of a single-result
    /// declaration, and the compiler-owned result-list value of a declaration
    /// that writes an ordered result list [GRAM-2, CALL-4].
    result_mode: CheckedMode,
    result: CheckedType,
    /// Every declared result ordinal in written order. A single-result
    /// declaration has exactly one entry and it is the callable result above.
    results: Vec<ResultSignature>,
    /// The result-list nominal, for a declaration that writes two or more
    /// results. `None` is the single-result form, whose callable result is
    /// the written result itself.
    result_list: Option<NominalId>,
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
#[derive(Clone)]
struct CheckedFunctionInventory {
    function: CheckedFunction,
    binding_names: Vec<String>,
}

fn derive_slice_return_ceiling(
    parameters: &[ParameterSignature],
    result_mode: CheckedMode,
    result: CheckedType,
) -> Vec<CheckedSliceOrigin> {
    let (
        CheckedMode::Own,
        CheckedType::Slice {
            region,
            element,
            strength,
        },
    ) = (result_mode, result)
    else {
        return Vec::new();
    };
    let mut ceiling = vec![CheckedSliceOrigin::ImmutableConst];
    for parameter in parameters {
        if parameter.mode == CheckedMode::Own
            && parameter.ty
                == (CheckedType::Slice {
                    region,
                    element,
                    strength,
                })
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
    /// region in a written type. No caller can determine the root.
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
        // A store brand is a component of the type [PROV-1], so a run, a
        // heap, and an extent each carry their own store region here exactly
        // as a slice carries its loan region.
        CheckedType::Vector { region: store, .. }
        | CheckedType::Heap { region: store }
        | CheckedType::Extent { region: store, .. } => store == region,
        CheckedType::Generic(_) | CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => true,
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Nominal(_)
        | CheckedType::Buffer { .. }
        | CheckedType::FixedVector { .. }
        | CheckedType::Array { .. } => false,
    }
}

/// Judges a borrow-mode result's provenance from the callable boundary
/// alone. `None` for an `own` result, which roots no caller borrow.
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
    /// [S20, GRAM-2] the declaration's own `region_params`, in written order.
    ///
    /// They are components of the nominal's type name [TYPE-2], so an
    /// instance is keyed on them beside its type and const arguments and two
    /// instances at two regions are two types [PROV-1].
    region_parameters: Vec<DeclarationId>,
    /// [PROV-6] whether the declaration writes the `linear` modifier. Every
    /// instance of a marked declaration is marked.
    linear: bool,
    /// [FORM-8] one entry per constructor of this declaration — a struct has
    /// one, an enum one per variant in tag order — and empty for a
    /// declaration carrying no `region_params`.
    constructors: Vec<ConstructorShape>,
}

/// One `construct` occurrence's declaration data [FORM-8, TYPE-5]: the
/// template it names, the variant when its nominal is an enum, the
/// declaration's own generic and region parameters, and the constructor's
/// shape.
struct ConstructorSite {
    template: usize,
    variant: Option<u32>,
    generic_parameters: Vec<generics::GenericParameter>,
    region_parameters: Vec<DeclarationId>,
    shape: ConstructorShape,
}

/// What a `construct` of one nominal has to know *before* it forms the
/// instance [FORM-8].
///
/// A construct writes a region argument only for a region parameter no field
/// operand determines, and the operands are what determine the rest — so the
/// instance is what the judgment produces and cannot be what it consults.
/// This is read once, off the declaration's own symbolic instance, whose
/// region arguments are its region parameters, while the templates are
/// validated.
#[derive(Clone, Default)]
struct ConstructorShape {
    /// The declared field names of this constructor, in declared order.
    fields: Vec<String>,
    /// Parallel to the declaration's `region_parameters`: the field whose
    /// declared type names that region parameter, and `None` where no field
    /// of this constructor does, which is exactly the region argument the
    /// construct writes.
    determining_field: Vec<Option<usize>>,
}

/// A nominal instance a derived type named, awaiting interning.
#[derive(Clone)]
enum PendingNominal {
    /// [STOR-2] a box over this referent.
    Box(CheckedType),
    /// S39 a `Box<'s, T>` over this store region and referent.
    StoreBox(DeclarationId, CheckedType),
    /// The compiler-owned result-list nominal of a [BLK-0] row that declares
    /// an ordered result list [CALL-4]. A row's list is fixed by its own
    /// instance and has no written form for the interning pass to find.
    ResultList(Vec<(String, CheckedType)>),
    /// [STOR-2] an `arena<'r, T>` instance over this region and content.
    Arena(DeclarationId, CheckedType),
    /// The one compiler-owned region allocation-list nominal [STOR-3].
    ArenaStorage,
    /// A prelude instance, such as the `Result<T, E>` a checked row produces.
    Prelude(PreludeType),
    /// [S20, FN-2] one source nominal instance at a region a call determined.
    ///
    /// A callee's result names its own formal region, and the instance the
    /// caller receives is that declaration at the actual region [FORM-8]. No
    /// caller position need write that type, so the interning pass cannot
    /// have found it; the checking path records the template and the
    /// substituted instance key here and the driver interns it.
    SourceInstance {
        template: usize,
        substitution: GenericSubstitution,
    },
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
    /// Structural incoming-formal attribution for this affine value. `None`
    /// is reserved for a value with no ownership identity; a fresh owner has a
    /// present empty set.
    state_origins: Option<CheckedStateOrigins>,
    live: bool,
    loop_depth: usize,
    /// Compiler-updated counted binders are readable source bindings but are
    /// never writer-controlled storage [SET-1, OWN-11].
    compiler_updated: bool,
    borrow: Option<BorrowInfo>,
    slice: Option<SliceInfo>,
    // Region-scoped loans outlive any one slice descriptor and end only with
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
    /// One loan per (region, place, strength); a second formation of the same
    /// loan adds its own holder rather than a second entry [PROV-3].
    fn push_slice_loan(&mut self, loan: SliceLoan) {
        if let Some(existing) = self.slice_loans.iter_mut().find(|existing| {
            existing.region == loan.region
                && existing.place == loan.place
                && existing.strength == loan.strength
        }) {
            for descriptor in loan.descriptors {
                if !existing.descriptors.contains(&descriptor) {
                    existing.descriptors.push(descriptor);
                }
            }
            return;
        }
        self.slice_loans.push(loan);
    }

    /// [PROV-3] one binding takes the loans its own origin set names: a `let`
    /// that binds a formed, copied, passed or returned view is where that
    /// value's liveness — and therefore its loan's extent — begins.
    fn hold_slice_loans(&mut self, holder: DeclarationId, places: &[ResolvedPlace]) {
        for loan in &mut self.slice_loans {
            if places.contains(&loan.place) && !loan.descriptors.contains(&holder) {
                loan.descriptors.push(holder);
            }
        }
    }

    fn end_slice_region(&mut self, region: DeclarationId) {
        self.slice_loans.retain(|loan| loan.region != region);
    }

    /// Whether two joined states agree apart from facts whose finite union is
    /// the exact joined state: region-scoped loans and capability origins.
    fn same_except_region_loans(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.slice_loans.clear();
        right.slice_loans.clear();
        left.state_origins = None;
        right.state_origins = None;
        left.suspended = false;
        right.suspended = false;
        left == right
    }

    /// Union of region-scoped loans: a loan established on any joined path
    /// holds for the region remainder, matching [OWN-4]'s named-region
    /// liveness of the borrows that carry it.
    fn merge_region_loans_from(&mut self, other: &Self) {
        for loan in &other.slice_loans {
            self.push_slice_loan(loan.clone());
        }
        self.suspended |= other.suspended;
        match (&mut self.state_origins, &other.state_origins) {
            (Some(left), Some(right)) => left.union(right),
            (None, Some(right)) => self.state_origins = Some(right.clone()),
            (Some(_), None) | (None, None) => {}
        }
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

/// [EFF-2]'s only repair: the declaration must equal the exhibited row.
const EFF2_ROW_FIX: &str = "declare exactly the row the body exhibits: add every missing category and path and remove every extra one; EFF-2 admits no wider and no narrower declaration than the union of the body-syntactic and release contributions";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EffectSet {
    reads: Vec<super::model::CheckedStatePath>,
    writes: Vec<super::model::CheckedStatePath>,
    /// [S23] the declared and exhibited `allocates` paths: one formal-rooted
    /// path per store whose provider is a value.
    allocates: Vec<super::model::CheckedStatePath>,
    /// The ambient heap of `box<T>` and `buffer<T>` [STOR-1]. Its store has no
    /// provider value, so [EFF-1] gives it no `effect_path` and no written
    /// entry; the flag is derived, never declared, and never compared, and it
    /// exists because [PROG-1]'s resource closure still reads it.
    allocates_heap: bool,
    allocates_arenas: Vec<DeclarationId>,
}

impl EffectSet {
    const NONE: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates: Vec::new(),
        allocates_heap: false,
        allocates_arenas: Vec::new(),
    };
    const ALLOCATES_HEAP: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates: Vec::new(),
        allocates_heap: true,
        allocates_arenas: Vec::new(),
    };
    fn union(mut self, other: Self) -> Self {
        for path in other.reads {
            self.add_read(path);
        }
        for path in other.writes {
            self.add_write(path);
        }
        for path in other.allocates {
            self.add_allocation(path);
        }
        self.allocates_heap |= other.allocates_heap;
        for region in other.allocates_arenas {
            self.add_arena_allocation(region);
        }
        self
    }

    fn add_read(&mut self, path: super::model::CheckedStatePath) {
        if !self.reads.contains(&path) {
            self.reads.push(path);
            self.reads.sort_unstable();
        }
    }

    fn add_write(&mut self, path: super::model::CheckedStatePath) {
        if !self.writes.contains(&path) {
            self.writes.push(path);
            self.writes.sort_unstable();
        }
    }

    /// The row a writer declares. The ambient heap [STOR-1] has no
    /// `effect_path` and therefore no written entry [EFF-1, S23], so it is not
    /// part of the row [EFF-2] compares in either direction.
    fn written_row(&self) -> Self {
        Self {
            allocates_heap: false,
            ..self.clone()
        }
    }

    fn add_allocation(&mut self, path: super::model::CheckedStatePath) {
        if !self.allocates.contains(&path) {
            self.allocates.push(path);
            self.allocates.sort_unstable();
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

struct Checker<'unit, 'classified, 'lexed, 'source> {
    resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    /// Whether an undischarged obligation rejects. Always true outside the
    /// test-only observability hooks.
    reject_entailment: bool,
    /// Whether the v0.31-candidate reborrow extension is admitted; see
    /// [`REBORROW_EXTENSION_ACTIVE`].
    reborrow_extension: bool,
    tree: TreeView<'unit, 'classified, 'lexed, 'source>,
    nominals: Vec<CheckedNominal>,
    nominal_nodes: Vec<Option<NodeId>>,
    nominal_states: Vec<u8>,
    source_nominal_instances: Vec<Option<(usize, GenericSubstitution)>>,
    box_nominals: HashMap<CheckedType, NominalId>,
    /// S39 one `Box<'s, T>` nominal per (store region, referent).
    store_box_nominals: HashMap<(DeclarationId, CheckedType), NominalId>,
    /// `arena<'r, T>` instances by (region declaration, content type): the
    /// region is part of the type's identity [OWN-3, STOR-4].
    arena_nominals: HashMap<(DeclarationId, CheckedType), NominalId>,
    /// The compiler-owned result-list nominal of a `fn_decl` that declares an
    /// ordered result list [GRAM-2, CALL-4], keyed by the ordered result
    /// binder spellings and types. Two declarations whose result lists agree
    /// share one nominal; the value a multi-result callable hands back is one
    /// value of it, and a destructuring binder list is its projection.
    result_list_nominals: HashMap<Vec<(String, CheckedType)>, NominalId>,
    /// The one compiler-owned region allocation-list nominal, interned on
    /// first use [STOR-3].
    arena_storage_nominal: Option<NominalId>,
    /// Nominal instances a derived type named that were not interned yet.
    /// Written by the `&self` checking path and drained by the `&mut self`
    /// driver between attempts at one function.
    pending_nominals: RefCell<Vec<PendingNominal>>,
    /// [PROV-1] the region an elided store brand denotes at the position
    /// being parsed: the enclosing nominal.s sole region parameter while a
    /// `struct_decl` or `enum_decl` body is being read, and `None`
    /// everywhere else, where the brand resolves to the entry heap's store
    /// region.
    elided_store_brand: std::cell::Cell<Option<DeclarationId>>,
    /// [BLK-4] whether this unit's entry selects [FN-7]'s `heap` standard
    /// input, memoized because the answer is one whole-program fact and the
    /// scan that reads it is over the unit's own `input_label` nodes.
    general_store_reachable: std::cell::Cell<Option<bool>>,
    /// [FN-2, OWN-1, S37] whether the body now being checked is a *concrete
    /// instance* of a generic template whose spelling one symbolic instance
    /// has already judged.
    ///
    /// The template is the spelling authority: an `affine` or `linear` body
    /// writes `move`, a `copy` body writes bare use, and the one symbolic
    /// instance decides both once. The concrete-instance recheck therefore
    /// does not re-judge the [OWN-1]/[FORM-1] spelling, and a `move` of a
    /// template-affine value at a copy instance denotes a copy. Every other
    /// [OWN-1] judgment — consume-once, dead roots, exclusivity — is
    /// re-judged as usual, because those are properties of the concrete
    /// instance and not of the written spelling.
    template_spelling_authority: std::cell::Cell<bool>,
    /// [LIV-2] the target places of the `set` commit whose right-hand side is
    /// being checked, and whether that right-hand side has read each out.
    /// Empty everywhere else: `check_commit` installs it around exactly that
    /// one expression and removes it before any rejection leaves.
    commit_read_outs: RefCell<Vec<control::CommitReadOut>>,
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
    /// Closed-world structural state origins for the currently selected
    /// concrete or symbolic function inventory, indexed by FunctionId.
    result_state_origins: RefCell<Vec<CheckedResultStateOrigin>>,
    /// The preliminary body pass records enough checked control/data flow to
    /// derive the summaries but deliberately postpones EFF-2 equality until
    /// the summaries reach a fixed point.
    deriving_result_state_origin: Cell<bool>,
    constants: HashMap<DeclarationId, CheckedConstantId>,
    checked_constants: Vec<CheckedConstant>,
    /// Hash-consed symbolic const operations [CONST-1 candidate]. Written by
    /// the `&self` const-expression parse while a generic template or
    /// symbolic validation instance is checked; every concrete instantiation
    /// evaluates entries away, so no id reaches lowering.
    derived_consts: RefCell<Vec<DerivedConst>>,
    /// [EFF-2] the body-syntactic contribution of each generic template's
    /// written body, recorded by its symbolic validation instance and reused
    /// by every concrete instance of the same declaration; see
    /// [`Checker::written_body_effects`].
    written_body_effect_rows: RefCell<HashMap<DeclarationId, EffectSet>>,
    pending_generic_requirements: Vec<PendingGenericRequirement>,
    generic_requirements: Vec<CheckedGenericRequirement>,
    postcondition_selectors: Vec<CheckedPostconditionSelector>,
    postcondition_unavailable_declarations: Vec<DeclarationId>,
    active_postcondition: Cell<Option<PostconditionCheckContext>>,
    /// The result datums admitted in the [FN-9] clause currently being
    /// checked: each written spelling with the result ordinal it names and
    /// the type that datum has [CALL-4]. A declaration writing one result
    /// contributes one row at ordinal zero. Set and restored beside
    /// `active_postcondition`.
    active_result_datums: RefCell<Vec<(String, u32, CheckedType)>>,
    contracts: Vec<ContractInfo>,
    contracts_by_declaration: HashMap<DeclarationId, usize>,
}

/// Checks the currently implemented active-specification semantic family.
///
/// Unsupported language families remain explicit compiler capability results;
/// only a proved numbered-rule violation becomes [`SemanticOutcome::SourceIssue`].
#[must_use]
pub fn check_semantics<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true, REBORROW_EXTENSION_ACTIVE)
}

/// [`check_semantics`] with entailment rejection disabled, so unit tests can
/// observe every retained obligation disposition
/// of one function, not only the first rejecting one. This is a test-only
/// observability hook, never a compilation mode: acceptance behavior has
/// exactly one path.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_dark<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, false, REBORROW_EXTENSION_ACTIVE)
}

/// Legacy test helper selecting the one shipped semantic judgment. It remains
/// only while the arithmetic obligation tests are renamed around IntegerDomain.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_arithmetic_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true, REBORROW_EXTENSION_ACTIVE)
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
    check_semantics_with(resolved, true, true)
}

/// Legacy test helper selecting the one shipped semantic judgment. It remains
/// only while the division obligation tests are renamed around IntegerDomain.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_division_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true, REBORROW_EXTENSION_ACTIVE)
}

fn check_semantics_with<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    reject_entailment: bool,
    reborrow_extension: bool,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    let preflight = if resolved.postconditions().is_empty() {
        Ok(())
    } else {
        Checker::new(&resolved, reject_entailment, reborrow_extension).and_then(|mut checker| {
            let items = checker.item_declarations()?;
            checker.preflight_postcondition_selectors(&items)
        })
    };
    let result = preflight.and_then(|()| {
        Checker::new(&resolved, reject_entailment, reborrow_extension)
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
    const fn inventory(&self) -> crate::Inventory {
        self.resolved.inventory()
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

    /// [EFF-2] the body-syntactic contribution of one checked function,
    /// judged once on the written body.
    ///
    /// A generic declaration has one written body even though the checker
    /// validates concrete instances separately. The symbolic validation is
    /// the authority for that declaration-wide syntactic contribution, so
    /// its row is recorded once and reused by every concrete instance.
    /// Instance-specific proofs may discharge static obligations, but they do
    /// do not erase a written call's declared effects. The
    /// release contribution is not syntactic and stays per instance [STOR-3].
    /// One effect row in its exact [EFF-1] canonical spelling.
    ///
    /// The rejection compared two rows and published neither, so a writer was
    /// told their row was wrong and left to derive both sides by hand. Both
    /// are in hand here, and so is the exact difference.
    fn render_effect_row(
        &self,
        effects: &EffectSet,
        signature: &FunctionSignature,
    ) -> Result<String, CheckStop> {
        let mut categories = Vec::new();
        if !effects.reads.is_empty() {
            categories.push(format!(
                "reads({})",
                self.render_effect_paths(&effects.reads, signature)?
                    .join(", ")
            ));
        }
        if !effects.writes.is_empty() {
            categories.push(format!(
                "writes({})",
                self.render_effect_paths(&effects.writes, signature)?
                    .join(", ")
            ));
        }
        let mut allocations = Vec::new();
        for path in &effects.allocates {
            allocations.push(self.render_effect_path(path, signature)?);
        }
        for region in &effects.allocates_arenas {
            allocations.push(format!("arena {}", self.region_phrase(*region)?));
        }
        if !allocations.is_empty() {
            let separator = if effects.allocates.is_empty() {
                " "
            } else {
                ", "
            };
            categories.push(format!("allocates({})", allocations.join(separator)));
        }
        Ok(if categories.is_empty() {
            "pure".to_owned()
        } else {
            categories.join(", ")
        })
    }

    fn render_effect_paths(
        &self,
        paths: &[super::model::CheckedStatePath],
        signature: &FunctionSignature,
    ) -> Result<Vec<String>, CheckStop> {
        paths
            .iter()
            .map(|path| self.render_effect_path(path, signature))
            .collect()
    }

    /// One `effect_path`: the parameter's own spelling and its selected source
    /// struct fields, exactly as [EFF-1] admits them.
    fn render_effect_path(
        &self,
        path: &super::model::CheckedStatePath,
        signature: &FunctionSignature,
    ) -> Result<String, CheckStop> {
        let parameter = signature
            .parameters
            .iter()
            .find(|parameter| parameter.declaration == path.root);
        let (mut rendered, mut ty) = match parameter {
            Some(parameter) => (parameter.name.clone(), Some(parameter.ty)),
            None => (self.declaration_spelling(path.root)?, None),
        };
        for field in &path.fields {
            let name = match ty {
                Some(CheckedType::Nominal(nominal)) => match &self.nominal(nominal)?.kind {
                    CheckedNominalKind::Struct { fields } => fields
                        .get(*field as usize)
                        .map(|declared| (declared.name.clone(), declared.ty)),
                    _ => None,
                },
                _ => None,
            };
            match name {
                Some((name, field_type)) => {
                    rendered.push('.');
                    rendered.push_str(&name);
                    ty = Some(field_type);
                }
                None => {
                    rendered.push_str(".?");
                    ty = None;
                }
            }
        }
        Ok(rendered)
    }

    /// The exhibited categories the declaration is missing, and the declared
    /// categories the body does not exhibit, each in the spelling the writer
    /// would have to add or delete.
    fn effect_row_difference(
        &self,
        exhibited: &EffectSet,
        declared: &EffectSet,
        signature: &FunctionSignature,
    ) -> Result<(Vec<String>, Vec<String>), CheckStop> {
        let mut missing = Vec::new();
        let mut extra = Vec::new();
        for (left, right, out) in [
            (exhibited, declared, &mut missing),
            (declared, exhibited, &mut extra),
        ] {
            for path in &left.reads {
                if !right.reads.contains(path) {
                    out.push(format!(
                        "reads({})",
                        self.render_effect_path(path, signature)?
                    ));
                }
            }
            for path in &left.writes {
                if !right.writes.contains(path) {
                    out.push(format!(
                        "writes({})",
                        self.render_effect_path(path, signature)?
                    ));
                }
            }
            for path in &left.allocates {
                if !right.allocates.contains(path) {
                    out.push(format!(
                        "allocates({})",
                        self.render_effect_path(path, signature)?
                    ));
                }
            }
            for region in &left.allocates_arenas {
                if !right.allocates_arenas.contains(region) {
                    out.push(format!("allocates(arena {})", self.region_phrase(*region)?));
                }
            }
        }
        Ok((missing, extra))
    }

    /// One declaration's exact source spelling, including any sigil.
    /// The source spelling of one region, or `None` where [FORM-8] leaves it
    /// unwritten and no name exists to quote.
    pub(in crate::semantic::check) fn written_region_name(
        &self,
        region: DeclarationId,
    ) -> Result<Option<String>, CheckStop> {
        let spelling = self.declaration_spelling(region)?;
        Ok((!spelling.starts_with("'0_")).then_some(spelling))
    }

    /// One region as a diagnostic names it.
    ///
    /// A region [FORM-8] leaves unwritten has no source spelling: resolution
    /// mints it under a name no source token can form, and printing that name
    /// would name a region the writer cannot write. Diagnostics that quote a
    /// region go through this instead of the raw spelling.
    /// [OWN-1, FORM-1, FN-2, S37] whether this body is the authority on the
    /// spellings [FORM-1] keys on a value's copy/affine class.
    ///
    /// Those are `move p` versus a bare `p` [OWN-1] and `replace` versus `set`
    /// [SET-1, SET-2]: one spelling per meaning, selected by the class. A
    /// concrete instance of a generic template is not their authority: the
    /// template's one symbolic instance judged them under the parameter's
    /// written bound, so at a copy instance a `move` of a template-affine
    /// value denotes a copy and a `replace` of one denotes the same exchange,
    /// rather than reopening a judgment the template already made. Every other
    /// judgment of those rules — consume-once, dead roots, exclusivity, the
    /// region-free demand on a replacement target — is re-judged here, because
    /// each is a property of the concrete instance and not of the spelling.
    /// [PROV-3] register one new binding as a holder of every loan its value's
    /// origin set names.
    ///
    /// The origins are the value's own [PROV-3] set, so this covers the four
    /// events the rule enumerates — formation, copy, pass and return — with
    /// one judgment over the set rather than one per event. `immutable-const`
    /// and a formal origin name no loan of this function and match nothing.
    pub(in crate::semantic::check) fn hold_slice_loans_of(
        holder: DeclarationId,
        slice: Option<&borrows::SliceInfo>,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) {
        let Some(slice) = slice else {
            return;
        };
        let mut wanted: HashMap<DeclarationId, Vec<ResolvedPlace>> = HashMap::new();
        for origin in &slice.origins {
            if let crate::semantic::model::CheckedSliceOrigin::SourcePlace {
                root, fields, ..
            } = origin
            {
                wanted.entry(*root).or_default().push(ResolvedPlace {
                    root: *root,
                    fields: fields.clone(),
                });
            }
        }
        for (root, places) in wanted {
            if let Some(local) = bindings.get_mut(&root) {
                local.hold_slice_loans(holder, &places);
            }
        }
    }

    pub(in crate::semantic::check) fn judges_class_spelling(&self) -> bool {
        !self.template_spelling_authority.get()
    }

    pub(in crate::semantic::check) fn region_phrase(
        &self,
        region: DeclarationId,
    ) -> Result<String, CheckStop> {
        let spelling = self.declaration_spelling(region)?;
        Ok(if spelling.starts_with("'0_") {
            "the region this position leaves unwritten".to_owned()
        } else {
            spelling
        })
    }

    pub(in crate::semantic::check) fn declaration_spelling(
        &self,
        declaration: DeclarationId,
    ) -> Result<String, CheckStop> {
        self.resolved
            .declarations()
            .iter()
            .find(|record| record.id() == declaration)
            .map(|record| record.spelling().to_owned())
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    fn written_body_effects(
        &self,
        signature: &FunctionSignature,
        syntactic: EffectSet,
    ) -> EffectSet {
        if signature.substitution.is_symbolic() {
            self.written_body_effect_rows
                .borrow_mut()
                .insert(signature.declaration, syntactic.clone());
            return syntactic;
        }
        if signature.substitution.len() == 0 {
            return syntactic;
        }
        self.written_body_effect_rows
            .borrow()
            .get(&signature.declaration)
            .cloned()
            .unwrap_or(syntactic)
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
        if borrow_result_provenance(parameters, result_mode, result)
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
        reborrow_extension: bool,
    ) -> Result<Self, CheckStop> {
        Ok(Self {
            resolved,
            reject_entailment,
            reborrow_extension,
            tree: TreeView::new(resolved)?,
            nominals: Vec::new(),
            nominal_nodes: Vec::new(),
            nominal_states: Vec::new(),
            source_nominal_instances: Vec::new(),
            box_nominals: HashMap::new(),
            store_box_nominals: HashMap::new(),
            arena_nominals: HashMap::new(),
            result_list_nominals: HashMap::new(),
            arena_storage_nominal: None,
            pending_nominals: RefCell::new(Vec::new()),
            elided_store_brand: std::cell::Cell::new(None),
            general_store_reachable: std::cell::Cell::new(None),
            template_spelling_authority: std::cell::Cell::new(false),
            commit_read_outs: RefCell::new(Vec::new()),
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
            result_state_origins: RefCell::new(Vec::new()),
            deriving_result_state_origin: Cell::new(false),
            constants: HashMap::new(),
            checked_constants: Vec::new(),
            derived_consts: RefCell::new(Vec::new()),
            written_body_effect_rows: RefCell::new(HashMap::new()),
            pending_generic_requirements: Vec::new(),
            generic_requirements: Vec::new(),
            postcondition_selectors: Vec::new(),
            postcondition_unavailable_declarations: Vec::new(),
            active_postcondition: Cell::new(None),
            active_result_datums: RefCell::new(Vec::new()),
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
        self.validate_generic_templates()?;
        self.derive_result_state_origins()?;
        let nominal_count_before_function_checking = self.nominals.len();
        let main = self.main_id()?;

        // Phase A completes every reachable concrete function before any
        // acceptance-bearing entailment judgment runs. This makes forward,
        // recursive, mutually recursive, and concrete generic call summaries
        // independent of function traversal order.
        let mut function_inventory = Vec::with_capacity(self.signatures.len());
        for index in 0..self.signatures.len() {
            function_inventory.push(self.check_function_interning_nominals(index)?);
        }
        // Function checking discovers the instances a derived type names: a
        // purely local `box<T>` [STOR-2], the `Result<T, E>` a checked row
        // produces, and — since a call substitutes its region arguments into
        // every position of the callee's signature, results included
        // [FN-2] — a *source* instance of a declaration this unit already
        // names at another region. That last class is admitted here on one
        // condition: it is one representation with an instance the written
        // text already interned [S20], so lowering still erases it onto that
        // instance's own IR nominal and the executable prefix closes over a
        // complete set of representations. A source instance discovered here
        // that is related to no earlier one would be a representation no
        // written type produced, and is a compiler defect.
        for index in nominal_count_before_function_checking..self.nominals.len() {
            let id = NominalId(
                u32::try_from(index)
                    .map_err(|_| CheckStop::from(SemanticCompilerFailure::CounterOverflow))?,
            );
            if self.source_nominal_instance_entry(id)?.is_none() {
                continue;
            }
            let mut related = false;
            for earlier in 0..nominal_count_before_function_checking {
                let earlier = NominalId(
                    u32::try_from(earlier)
                        .map_err(|_| CheckStop::from(SemanticCompilerFailure::CounterOverflow))?,
                );
                if self.nominals_differ_only_in_region(id, earlier)? {
                    related = true;
                    break;
                }
            }
            if !related {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }

        // For an FN-9 unit, the complete existing [FN-3]/[FN-4] pass
        // remains ahead of the first acceptance-bearing optimistic query. Its
        // results are retained and reused below; no narrow duplicate prepass
        // or proof-only contract judgment exists. The no-postcondition,
        // no-marker ordinary fast path keeps its established phase order.
        let early_contracts = if self.resolved.postconditions().is_empty() {
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
        let callees = self.entailment_callees()?;
        self.install_call_requirements(&mut function_inventory)?;
        let optimistic_batch = function_inventory.iter().any(|checked| {
            !checked.function.postconditions.is_empty()
                || Self::statements_contain_value_if(&checked.function.body)
        });

        let postcondition_schedule =
            self.analyze_function_inventory(&mut function_inventory, &callees, optimistic_batch)?;
        let baseline_functions = function_inventory
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();
        if self.reject_entailment {
            let mut rejections = Vec::new();
            for function in &baseline_functions {
                match self.entailment_rejection(function) {
                    Ok(()) => {}
                    Err(CheckStop::Issue(issue)) => {
                        let path = Self::source_issue_path(&issue)?.clone();
                        rejections.push((
                            path,
                            self.concrete_instance_rank(function)?,
                            issue.rule.definition_rank(),
                            issue,
                        ));
                    }
                    Err(stop) => return Err(stop),
                }
            }
            rejections.sort_by(|left, right| {
                left.0
                    .components()
                    .cmp(right.0.components())
                    .then(left.1.cmp(&right.1))
                    .then(left.2.cmp(&right.2))
            });
            if let Some((_, _, _, issue)) = rejections.into_iter().next() {
                return Err(CheckStop::Issue(issue));
            }
        }
        drop(baseline_functions);
        let mut functions = function_inventory
            .into_iter()
            .map(|checked| checked.function)
            .collect::<Vec<_>>();
        if optimistic_batch {
            for function in &mut functions {
                finalize_function_entailment(&mut function.entailment);
            }
        }
        for function in &mut functions {
            function.body_disposition = function.entailment.body_disposition;
        }
        // Copy each accepted OP-9 site's proved numeric length ceiling onto
        // the corresponding checked allocation node. This is the sole
        // semantic-to-target handoff: lowering receives a conclusion, not the
        // proof arena, and performs no proof reconstruction.
        self.install_source_allocation_bounds(&mut functions)?;

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
        self.materialize_generic_requirements()?;
        let derived_consts = self.derived_consts.borrow().clone();
        for (index, derived) in derived_consts.iter().enumerate() {
            for operand in [derived.left, derived.right] {
                if matches!(operand, CheckedConst::Derived(id) if id.0 as usize >= index) {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
        }
        // Target execution is closed compiler metadata. Derive the complete
        // recursive summary only after all concrete bodies are final.
        super::target_action::derive_target_actions(&mut functions);
        // Permission is a read-only legality table over the completed checked
        // program. The affine-map rule consumes a successful OP-4 disposition
        // and exact value image retained on that program; no permission rule
        // repeats a local invariant or changes source acceptance.
        let permission_signatures = self
            .signatures
            .iter()
            .map(|signature| PermissionSignature {
                region_parameters: signature.region_parameters.clone(),
                reads: signature.declared_effects.reads.clone(),
                writes: signature.declared_effects.writes.clone(),
                allocates_arenas: signature.declared_effects.allocates_arenas.clone(),
            })
            .collect::<Vec<_>>();
        let permission = analyze_permission(&functions, &permission_signatures);
        // The ledger is rendered here because only the checker still holds the
        // syntax tree the citations name. It is pure presentation over the
        // table above and reaches no decision.
        let permission_ledger = if permission.functions.iter().any(|permissions| {
            !permissions.pairs.is_empty()
                || !permissions.loops.is_empty()
                || !permissions.staged.is_empty()
        }) {
            render_ledger(&permission, &PermissionLedgerSource { tree: &self.tree })?
        } else {
            Vec::new()
        };

        Ok(CheckedProgramData {
            nominals: self.nominals.clone(),
            executable_nominal_count,
            nominal_lowering_alias: self.nominal_lowering_aliases()?,
            constants: self.checked_constants.clone(),
            derived_consts,
            functions,
            postcondition_schedule,
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
            permission,
            permission_ledger,
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
            self.validate_generic_templates()?;
            self.derive_result_state_origins()?;
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
                            PendingNominal::StoreBox(region, referent) => {
                                self.intern_store_box_nominal(region, referent)?;
                            }
                            PendingNominal::ResultList(results) => {
                                self.intern_result_list_nominal(&results)?;
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
                            PendingNominal::SourceInstance {
                                template,
                                substitution,
                            } => {
                                self.ensure_source_nominal_instance(template, substitution)?;
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

    /// [FN-2, OWN-1, S37] one body, checked with the template's spelling
    /// authority recorded for the instance it is.
    ///
    /// A concrete instance of a generic template is exactly the body whose
    /// [OWN-1]/[FORM-1] spelling the template's own symbolic instance already
    /// judged under the parameter's written bound, so this instance does not
    /// re-judge it. A symbolic instance and a nongeneric body are their own
    /// authority and judge the spelling here.
    fn check_function_signature(
        &self,
        signature: &FunctionSignature,
    ) -> Result<CheckedFunctionInventory, CheckStop> {
        let previous = self
            .template_spelling_authority
            .replace(signature.substitution.len() > 0 && signature.substitution.is_concrete());
        let outcome = self.check_function_signature_body(signature);
        self.template_spelling_authority.set(previous);
        outcome
    }

    fn check_function_signature_body(
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
            let state_leaves = self.type_state_leaf_paths(parameter.ty)?;
            bindings.insert(
                parameter.declaration,
                LocalBinding {
                    binding,
                    declaration: parameter.declaration,
                    mode: parameter.mode,
                    ty: parameter.ty,
                    state_origins: (!state_leaves.is_empty()).then(|| {
                        CheckedStateOrigins::formal_leaves(parameter.declaration, state_leaves)
                    }),
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
                declaration: parameter.declaration,
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

        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
            binding_names: &mut binding_names,
        };
        let parameter_bindings = bindings.clone();
        let requirements = if let Some(node) = self
            .tree
            .first_child_with(signature.node, Production::ContractBlock)?
        {
            let mut requires_bindings = parameter_bindings.clone();
            self.check_requires(signature, node, &mut requires_bindings, &mut counters)?
                .requirements
        } else {
            Vec::new()
        };

        let postcondition_selectors = self.postcondition_selectors_for_signature(signature)?;
        let mut postcondition_relations = Vec::with_capacity(postcondition_selectors.len());
        for selector in &postcondition_selectors {
            let mut postcondition_bindings = parameter_bindings.clone();
            postcondition_relations.push(self.check_postcondition_clause(
                signature,
                selector,
                &mut postcondition_bindings,
                &mut counters,
            )?);
        }
        self.check_published_relation_consistency(
            signature,
            &postcondition_selectors,
            &postcondition_relations,
        )?;

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
        let syntactic = if self.deriving_result_state_origin.get() {
            checked.effects.clone()
        } else {
            self.written_body_effects(signature, checked.effects.clone())
        };
        let mut release_sites = Vec::new();
        self.collect_release_sites(signature, &checked.statements, &mut release_sites)?;
        let mut release = EffectSet::NONE;
        for site in &release_sites {
            release = release.union(site.effects.clone());
        }
        let exhibited = syntactic.clone().union(release.clone());
        if !self.deriving_result_state_origin.get()
            && exhibited.written_row() != signature.declared_effects.written_row()
        {
            // A state transition contributed only by a release has no offending
            // source occurrence. Keep the owner-bearing diagnostic for that
            // case even though the current system releases have empty memory
            // rows; later resource families may carry an ordinary memory row.
            let release_only = release.clone().union(syntactic.clone()).written_row()
                != syntactic.written_row()
                && release
                    .clone()
                    .union(signature.declared_effects.clone())
                    .written_row()
                    != signature.declared_effects.written_row();
            if release_only {
                let owner = release_sites
                    .iter()
                    .find(|site| site.effects != EffectSet::NONE)
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
            let (missing, extra) =
                self.effect_row_difference(&exhibited, &signature.declared_effects, signature)?;
            return self.issue_node(
                SemanticRule::Eff2,
                signature.effects_node,
                SemanticIssueKind::EffectMismatch {
                    expected_row: self.render_effect_row(&exhibited, signature)?,
                    found_row: self.render_effect_row(&signature.declared_effects, signature)?,
                    missing,
                    extra,
                    mechanical_fix: EFF2_ROW_FIX,
                },
            );
        }
        let postconditions = if signature.substitution.is_concrete() {
            postcondition_selectors
                .into_iter()
                .zip(postcondition_relations)
                .map(|(selector, relation)| {
                    self.build_checked_postcondition(
                        signature,
                        &parameters,
                        selector,
                        relation,
                        &checked.statements,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            postcondition_selectors
                .into_iter()
                .zip(postcondition_relations)
                .map(|(selector, relation)| {
                    self.build_checked_schema_postcondition(
                        signature,
                        &parameters,
                        selector,
                        relation,
                        &checked.statements,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect()
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
            parameters,
            result_mode: signature.result_mode,
            result: signature.result,
            result_state_origin: self
                .result_state_origins
                .borrow()
                .get(signature.id.0 as usize)
                .cloned()
                .unwrap_or(CheckedResultStateOrigin::Unknown),
            slice_return_ceiling: signature.slice_return_ceiling.clone(),
            reaches_ambient_heap: checked.effects.allocates_heap,
            declared_state_writes: signature.declared_effects.writes.clone(),
            target_action: crate::TargetAction::INLINE,
            requirements,
            postconditions,
            body: checked.statements,
            body_disposition: super::model::CheckedBodyDisposition::Inhabited,
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
            | CheckedStatement::DestructuringLet { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::SetList { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::Dispose { .. }
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Proof(_)
            | CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => false,
        })
    }

    /// Dense kill-relevant callee inventory shared by concrete and symbolic
    /// source-schema entailment. Function identity must be a true vector
    /// index; silently skipping a malformed identity would make masks observe
    /// a different program from the baseline.
    fn entailment_callees(&self) -> Result<Vec<EntailmentCallee>, CheckStop> {
        let mut callees = Vec::with_capacity(self.signatures.len());
        for (index, signature) in self.signatures.iter().enumerate() {
            if signature.id.0 as usize != index {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            callees.push(EntailmentCallee::from_signature(
                signature
                    .parameters
                    .iter()
                    .map(|parameter| (parameter.declaration, parameter.mode)),
                &signature.declared_effects.writes,
            ));
        }
        Ok(callees)
    }

    /// Checks the source-generic body while symbolic nominal and function
    /// identities are still alive.
    /// Checks every source-generic body once with symbolic arguments, even when
    /// no concrete instantiation is reachable from the executable program.
    /// The ordinary entailment engine is the only acceptance path: generic
    /// bodies do not receive a separate proof language or an assertion-based
    /// exception.
    fn validate_generic_body_entailment(
        &self,
        functions: &mut [CheckedFunctionInventory],
        canonical: &[(usize, DeclarationId)],
        callees: &[EntailmentCallee],
    ) -> Result<(), CheckStop> {
        let optimistic_batch = functions.iter().any(|checked| {
            !checked.function.postconditions.is_empty()
                || Self::statements_contain_value_if(&checked.function.body)
        });
        self.analyze_function_inventory(functions, callees, optimistic_batch)?;
        if optimistic_batch {
            for checked in functions.iter_mut() {
                finalize_function_entailment(&mut checked.function.entailment);
            }
        }
        if !self.reject_entailment {
            return Ok(());
        }
        for (index, declaration) in canonical {
            let checked = functions
                .get(*index)
                .filter(|checked| checked.function.declaration == *declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.entailment_rejection(&checked.function)?;
        }
        Ok(())
    }
    fn analyze_function_inventory(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
    ) -> Result<PostconditionSchedule, CheckStop> {
        // ENT is the single acceptance-bearing proof path for ordinary
        // obligations, call requirements, invariants and postconditions.
        let mut schedule =
            postcondition_schedule(functions.iter().map(|checked| &checked.function))
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if schedule.components.is_empty() {
            for checked in functions.iter_mut() {
                let context = EntailmentContext {
                    callees,
                    constants: &self.checked_constants,
                    constant_ids: &self.constants,
                    nominals: &self.nominals,
                    verified_postconditions: &[],
                    verified_postcondition_proofs: &[],
                    binding_names: &checked.binding_names,
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
                                .postconditions
                                .iter()
                                .filter(|proof| {
                                    proof.summary.as_ref().is_some_and(|summary| {
                                        summary.component < component.ordinal
                                    })
                                })
                                .filter_map(|proof| {
                                    checked
                                        .function
                                        .postconditions
                                        .get(proof.relation_ordinal as usize)
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let verified_postcondition_proofs = functions
                        .iter()
                        .map(|checked| {
                            checked
                                .function
                                .entailment
                                .postconditions
                                .iter()
                                .filter(|proof| {
                                    proof.summary.as_ref().is_some_and(|summary| {
                                        summary.component < component.ordinal
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let checked = functions
                        .get(function_index)
                        .filter(|checked| checked.function.id == *function)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let context = EntailmentContext {
                        callees,
                        constants: &self.checked_constants,
                        constant_ids: &self.constants,
                        nominals: &self.nominals,
                        verified_postconditions: &verified_postconditions,
                        verified_postcondition_proofs: &verified_postcondition_proofs,
                        binding_names: &checked.binding_names,
                    };
                    let entailment = analyze_function_candidate(&checked.function, &context);
                    drop(verified_postconditions);
                    drop(verified_postcondition_proofs);
                    functions[function_index].function.entailment = entailment;
                }

                let publish = component.functions.iter().all(|function| {
                    let checked = &functions[function.0 as usize].function;
                    checked
                        .entailment
                        .loop_invariants
                        .iter()
                        .all(|invariant| invariant.proof.discharged())
                        && (matches!(
                            checked.entailment.body_disposition,
                            super::model::CheckedBodyDisposition::Uninhabited { .. }
                        ) || checked.postconditions.is_empty()
                            || (checked.entailment.postconditions.len()
                                == checked.postconditions.len()
                                && checked
                                    .entailment
                                    .postconditions
                                    .iter()
                                    .all(|proof| proof.aggregate.discharged)))
                });
                if publish {
                    for function in &component.functions {
                        let checked = &mut functions[function.0 as usize].function;
                        if matches!(
                            checked.entailment.body_disposition,
                            super::model::CheckedBodyDisposition::Uninhabited { .. }
                        ) {
                            continue;
                        }
                        for proof in &mut checked.entailment.postconditions {
                            let summary = VerifiedPostconditionSummary {
                                function: *function,
                                block: proof.block.clone(),
                                relation_ordinal: proof.relation_ordinal,
                                component: component.ordinal,
                            };
                            proof.summary = Some(summary.clone());
                            component.summaries.push(summary);
                        }
                    }
                }
            }
        }
        Ok(schedule)
    }
    fn concrete_instance_rank(
        &self,
        function: &CheckedFunction,
    ) -> Result<u32, SemanticCompilerFailure> {
        let signature = self
            .signatures
            .get(function.id.0 as usize)
            .filter(|signature| signature.id == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if signature.substitution.len() == 0 {
            Ok(0)
        } else {
            function
                .id
                .0
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)
        }
    }

    fn install_call_requirements(
        &self,
        functions: &mut [CheckedFunctionInventory],
    ) -> Result<(), CheckStop> {
        let requirements = functions
            .iter()
            .map(|checked| checked.function.requirements.clone())
            .collect::<Vec<_>>();
        for checked in functions {
            self.install_statement_call_requirements(&mut checked.function.body, &requirements)?;
        }
        Ok(())
    }

    fn install_statement_call_requirements(
        &self,
        statements: &mut [CheckedStatement],
        requirements: &[Vec<CheckedRequirement>],
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::DestructuringLet { value, .. }
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::Dispose { value, .. }
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    self.install_expression_call_requirements(value, requirements)?;
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => {
                    self.install_expression_call_requirements(scrutinee, requirements)?;
                }
                CheckedStatement::SetList {
                    targets, values, ..
                } => {
                    for target in targets {
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
                            CheckedSetTarget::RunIndex(target) => self
                                .install_expression_call_requirements(
                                    &mut target.offset,
                                    requirements,
                                )?,
                            CheckedSetTarget::SliceIndex(target) => self
                                .install_expression_call_requirements(
                                    &mut target.offset,
                                    requirements,
                                )?,
                        }
                    }
                    for value in values.expressions_mut() {
                        self.install_expression_call_requirements(value, requirements)?;
                    }
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
                        CheckedSetTarget::RunIndex(target) => self
                            .install_expression_call_requirements(
                                &mut target.offset,
                                requirements,
                            )?,
                        CheckedSetTarget::SliceIndex(target) => self
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
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn install_expression_call_requirements(
        &self,
        expression: &mut CheckedExpression,
        requirements: &[Vec<CheckedRequirement>],
    ) -> Result<(), CheckStop> {
        match expression {
            CheckedExpression::UserCall {
                function,
                arguments,
                goal_arguments,
                goal_regions,
                requirements: call_requirements,
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
                *call_requirements = boundary
                    .iter()
                    .map(|boundary| {
                        Ok(CheckedCallRequirement {
                            requires_clause: boundary.clause.clone(),
                            goal: ConcreteGoal::new(self.instantiate_goal_expression(
                                &boundary.template.root,
                                signature,
                                goal_regions,
                                goal_arguments,
                            )?),
                        })
                    })
                    .collect::<Result<Vec<_>, CheckStop>>()?;
            }
            // A [BLK-0] row's requirement list is declaration data, so the
            // call instantiated it while it was checked and there is nothing
            // to install from the source inventory here.
            CheckedExpression::SystemCall { arguments, .. }
            | CheckedExpression::KernelCall { arguments, .. }
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
            | CheckedExpression::RunIndex { offset, .. }
            | CheckedExpression::SliceIndex { offset, .. } => {
                self.install_expression_call_requirements(offset, requirements)?;
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                self.install_expression_call_requirements(length, requirements)?;
                self.install_expression_call_requirements(value, requirements)?;
            }
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
                self.install_expression_call_requirements(length, requirements)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::PostconditionResultMeasure { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceMeasure { .. }
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

    fn install_source_allocation_bounds(
        &self,
        functions: &mut [CheckedFunction],
    ) -> Result<(), CheckStop> {
        for function in functions {
            let bounds = function
                .entailment
                .obligations
                .iter()
                .filter(|outcome| {
                    outcome.family == super::entailment::ObligationFamily::AllocationFit
                        && outcome.discharged
                })
                .map(|outcome| {
                    let upper = outcome
                        .allocation_length_upper_bound
                        .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                    Ok((outcome.node_path.clone(), upper))
                })
                .collect::<Result<HashMap<_, _>, SemanticCompilerFailure>>()?;
            Self::install_statement_allocation_bounds(&mut function.body, &bounds)?;
        }
        Ok(())
    }

    fn install_statement_allocation_bounds(
        statements: &mut [CheckedStatement],
        bounds: &HashMap<NodePath, u64>,
    ) -> Result<(), SemanticCompilerFailure> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::DestructuringLet { value, .. }
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::Dispose { value, .. }
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    Self::install_expression_allocation_bounds(value, bounds)?;
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => {
                    Self::install_expression_allocation_bounds(scrutinee, bounds)?;
                }
                CheckedStatement::SetList {
                    targets, values, ..
                } => {
                    for target in targets {
                        match target {
                            CheckedSetTarget::Place(_) => {}
                            CheckedSetTarget::ArrayIndex(target) => {
                                Self::install_expression_allocation_bounds(
                                    &mut target.offset,
                                    bounds,
                                )?;
                            }
                            CheckedSetTarget::BufferIndex(target) => {
                                Self::install_expression_allocation_bounds(
                                    &mut target.offset,
                                    bounds,
                                )?;
                            }
                            CheckedSetTarget::RunIndex(target) => {
                                Self::install_expression_allocation_bounds(
                                    &mut target.offset,
                                    bounds,
                                )?;
                            }
                            CheckedSetTarget::SliceIndex(target) => {
                                Self::install_expression_allocation_bounds(
                                    &mut target.offset,
                                    bounds,
                                )?;
                            }
                        }
                    }
                    for value in values.expressions_mut() {
                        Self::install_expression_allocation_bounds(value, bounds)?;
                    }
                }
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::RunIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::SliceIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                    }
                    Self::install_expression_allocation_bounds(value, bounds)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    Self::install_expression_allocation_bounds(scrutinee, bounds)?;
                    for arm in arms {
                        Self::install_statement_allocation_bounds(&mut arm.body, bounds)?;
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    Self::install_statement_allocation_bounds(body, bounds)?;
                }
                CheckedStatement::CountedRange {
                    lower, upper, body, ..
                } => {
                    Self::install_expression_allocation_bounds(lower, bounds)?;
                    Self::install_expression_allocation_bounds(upper, bounds)?;
                    Self::install_statement_allocation_bounds(body, bounds)?;
                }
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn install_expression_allocation_bounds(
        expression: &mut CheckedExpression,
        bounds: &HashMap<NodePath, u64>,
    ) -> Result<(), SemanticCompilerFailure> {
        match expression {
            CheckedExpression::BufferFill {
                carrier,
                length,
                value,
                target_domains,
                ..
            } => {
                // Acceptance-dark test hooks deliberately retain failed OP-9
                // sites. They keep the pending None value and never lower;
                // an accepted ordinary program has a bound for every site.
                if let Some(upper) = bounds.get(carrier).copied() {
                    target_domains.install_source_length_upper_bound(upper);
                }
                Self::install_expression_allocation_bounds(length, bounds)?;
                Self::install_expression_allocation_bounds(value, bounds)?;
            }
            CheckedExpression::BufferVacant {
                carrier,
                length,
                target_domains,
                ..
            } => {
                if let Some(upper) = bounds.get(carrier).copied() {
                    target_domains.install_source_length_upper_bound(upper);
                }
                Self::install_expression_allocation_bounds(length, bounds)?;
            }
            CheckedExpression::UserCall { arguments, .. }
            | CheckedExpression::SystemCall { arguments, .. }
            | CheckedExpression::KernelCall { arguments, .. }
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
                    Self::install_expression_allocation_bounds(argument, bounds)?;
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
                Self::install_expression_allocation_bounds(value, bounds)?;
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::RunIndex { offset, .. }
            | CheckedExpression::SliceIndex { offset, .. } => {
                Self::install_expression_allocation_bounds(offset, bounds)?;
            }
            CheckedExpression::BufferFits { length, .. } => {
                Self::install_expression_allocation_bounds(length, bounds)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::PostconditionResultMeasure { .. }
            | CheckedExpression::SliceOf { .. }
            | CheckedExpression::SliceMeasure { .. }
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
            GoalExpression::Datum(GoalDatum::Place { .. } | GoalDatum::EvaluatedValue { .. }) => {
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
            GoalOperation::ArrayMeasure {
                measure,
                element,
                length,
            } => GoalOperation::ArrayMeasure {
                measure,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            GoalOperation::ArrayIndex { element, length } => GoalOperation::ArrayIndex {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            GoalOperation::BufferMeasure { measure, element } => GoalOperation::BufferMeasure {
                measure,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            GoalOperation::BufferIndex { element } => GoalOperation::BufferIndex {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            GoalOperation::BufferFits {
                element,
                maximum_length: _,
            } => {
                let element = self.instantiate_goal_type(element, signature, regions)?;
                let maximum_length = self
                    .instantiated_layout_ceiling(element)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .stride
                    .allocation_limit();
                GoalOperation::BufferFits {
                    element,
                    maximum_length,
                }
            }
            GoalOperation::SliceMeasure {
                measure,
                region,
                element,
            } => GoalOperation::SliceMeasure {
                measure,
                region: self.instantiate_goal_region(region, signature, regions)?,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            GoalOperation::ContainerMeasure {
                measure,
                measured,
                element,
                constant,
            } => GoalOperation::ContainerMeasure {
                measure,
                measured,
                element: element
                    .map(|element| self.instantiate_goal_element(element, signature, regions))
                    .transpose()?,
                constant: constant
                    .map(|constant| self.instantiate_goal_const(constant, signature))
                    .transpose()?,
            },
            GoalOperation::RunIndex {
                measured,
                element,
                constant,
            } => GoalOperation::RunIndex {
                measured,
                element: self.instantiate_goal_element(element, signature, regions)?,
                constant: constant
                    .map(|constant| self.instantiate_goal_const(constant, signature))
                    .transpose()?,
            },
            GoalOperation::SliceIndex { region, element } => GoalOperation::SliceIndex {
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
            CheckedType::Slice {
                region,
                element,
                strength,
            } => CheckedType::Slice {
                region: self.instantiate_goal_region(region, signature, regions)?,
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
                strength,
            },
            CheckedType::Buffer { element } => CheckedType::Buffer {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            CheckedType::FixedVector { element, length } => CheckedType::FixedVector {
                element: self.instantiate_goal_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            CheckedType::Vector {
                region, element, ..
            } => {
                let region = self.instantiate_goal_region(region, signature, regions)?;
                CheckedType::Vector {
                    region,
                    element: self.instantiate_goal_element(element, signature, regions)?,
                    release: self.vector_release_class(region)?,
                }
            }
            CheckedType::Heap { region } => CheckedType::Heap {
                region: self.instantiate_goal_region(region, signature, regions)?,
            },
            CheckedType::Extent {
                region,
                bytes,
                align,
            } => CheckedType::Extent {
                region: self.instantiate_goal_region(region, signature, regions)?,
                bytes: self.instantiate_goal_const(bytes, signature)?,
                align: self.instantiate_goal_const(align, signature)?,
            },
            CheckedType::Nominal(id) => self.instantiate_goal_nominal(id, signature, regions)?,
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => ty,
        })
    }

    /// [S20] one nominal instance read at a caller: its region arguments
    /// substituted, which is the instance the caller's own value has.
    ///
    /// The substituted instance already exists wherever the caller can hold a
    /// value of it, so this is a lookup and never a minting; where it does
    /// not, the declaration's own instance stands and the ordinary type
    /// judgment decides.
    fn instantiate_goal_nominal(
        &self,
        id: NominalId,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        let Some((template, substitution)) = self.source_nominal_instance_entry(id)? else {
            return Ok(CheckedType::Nominal(id));
        };
        if substitution.region_arguments().is_empty() {
            return Ok(CheckedType::Nominal(id));
        }
        let substitution = substitution.clone();
        let mut mapped = Vec::with_capacity(substitution.region_arguments().len());
        for (formal, actual) in substitution.region_arguments() {
            mapped.push((
                *formal,
                self.instantiate_goal_region(*actual, signature, regions)?,
            ));
        }
        if mapped == substitution.region_arguments() {
            return Ok(CheckedType::Nominal(id));
        }
        let declaration = self
            .nominal_templates
            .get(template)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .declaration;
        Ok(self
            .source_nominal_instance(declaration, &substitution.with_regions(mapped))
            .map_or(CheckedType::Nominal(id), CheckedType::Nominal))
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
            | CheckedType::Buffer { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Heap { .. }
            | CheckedType::Extent { .. } => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        })
    }

    /// One run element at a caller's instance [BLK-1].
    ///
    /// The lift is one level, so an element that is itself a run instantiates
    /// through the ordinary type path and is re-lifted; anything the lift does
    /// not carry is the flat domain's own instantiation.
    fn instantiate_goal_element(
        &self,
        element: CheckedElement,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedElement, CheckStop> {
        if element.is_run() {
            let ty = self.instantiate_goal_type(element.ty(), signature, regions)?;
            return Self::run_element(ty)
                .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into());
        }
        let flat = element
            .flat()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let ty = self.instantiate_goal_type(flat.ty(), signature, regions)?;
        if let Some(lifted) = Self::run_element(ty) {
            return Ok(lifted);
        }
        Ok(CheckedElement::Flat(
            self.instantiate_goal_flat_element(flat, signature, regions)?,
        ))
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
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
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
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
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
            // [MSR-6] a const generic is fixed at [FN-2] instantiation, so a
            // concrete instance reads a mathematical constant and only the
            // one symbolic instance keeps the declaration-anchored form.
            CheckedValue::ConstGeneric { declaration, ty } => {
                match signature.substitution.const_argument(*declaration) {
                    Some(CheckedConst::Value(value)) => CheckedValue::Integer {
                        ty: *ty,
                        bits: value,
                    },
                    _ => CheckedValue::ConstGeneric {
                        declaration: *declaration,
                        ty: *ty,
                    },
                }
            }
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

    fn source_issue_path(
        issue: &SemanticIssue,
    ) -> Result<&crate::NodePath, SemanticCompilerFailure> {
        match &issue.location {
            SemanticLocation::SourceNode(path, _) => Ok(path),
            SemanticLocation::BundleRoot(_) => Err(SemanticCompilerFailure::InvalidResolution),
        }
    }

    /// The enclosing `loop_stmt` or `for_stmt` of one loop-header invariant.
    ///
    /// A header invariant is always written inside its loop statement, so the
    /// ancestor exists for every well-formed tree. The absent case keeps the
    /// caller total and simply leaves the invariant at its own position.
    fn enclosing_loop_node(&self, node: NodeId) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        let mut current = node;
        loop {
            let production = self.tree.production(current)?;
            if production == Production::LoopStmt || production == Production::ForStmt {
                return Ok(Some(current));
            }
            match self.tree.parent(current)? {
                Some(parent) => current = parent,
                None => return Ok(None),
            }
        }
    }

    fn entailment_rejection(&self, function: &CheckedFunction) -> Result<(), CheckStop> {
        /// One position in the causal order in which obligations are decided.
        ///
        /// `Child` is a syntax child ordinal, so a plain node path orders a
        /// failure exactly where the walk reaches it. `AfterSubtree` is the
        /// position immediately after everything one node encloses: it is
        /// greater than every child ordinal under that node and still less
        /// than the node's following siblings.
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum ProofPosition {
            Child(u32),
            AfterSubtree,
        }

        enum Rejection<'outcome> {
            LoopInvariant(&'outcome super::entailment::LoopInvariantOutcome),
            SourceProof(&'outcome super::entailment::SourceProofOutcome),
            Obligation(&'outcome super::entailment::ObligationOutcome),
            Call(&'outcome super::entailment::CallGoalOutcome),
        }

        impl Rejection<'_> {
            fn node_path(&self) -> &crate::NodePath {
                match self {
                    Self::LoopInvariant(outcome) => &outcome.node_path,
                    Self::SourceProof(outcome) => outcome.rejection_node_path(),
                    Self::Obligation(outcome) => &outcome.node_path,
                    Self::Call(outcome) => &outcome.node_path,
                }
            }

            const fn rule(&self) -> SemanticRule {
                match self {
                    Self::LoopInvariant(_) => SemanticRule::Inv1,
                    Self::SourceProof(outcome) => {
                        if outcome.certificate_written {
                            SemanticRule::Prf1
                        } else {
                            SemanticRule::Inv1
                        }
                    }
                    Self::Obligation(outcome) => match outcome.family {
                        super::entailment::ObligationFamily::Bounds => SemanticRule::Op4,
                        super::entailment::ObligationFamily::IntegerDomain => SemanticRule::Op2,
                        super::entailment::ObligationFamily::AllocationFit => SemanticRule::Op9,
                        super::entailment::ObligationFamily::SystemRange => SemanticRule::Sys8,
                        super::entailment::ObligationFamily::KernelRequirement => {
                            SemanticRule::Blk0
                        }
                    },
                    Self::Call(_) => SemanticRule::Fn8,
                }
            }
        }

        let loop_invariant = function
            .entailment
            .loop_invariants
            .iter()
            .filter(|outcome| !outcome.proof.discharged())
            .map(Rejection::LoopInvariant);
        let source_proof = function
            .entailment
            .source_proofs
            .iter()
            .filter(|outcome| !outcome.check.discharged())
            .map(Rejection::SourceProof);
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
        // [DIAG-1] admits exactly one rule and one location, so the single
        // reported failure is selected by the order in which the checker
        // decides obligations, not by where they are written. Every judgment
        // but one is decided where it stands. INV-1's backedge judgment is the
        // exception: it is proved only after the whole loop body has been
        // walked, and a body failure that demotes a value to a fresh full-range
        // atom is exactly what breaks it. Positioning the backedge after the
        // body it consumes therefore reports the cause rather than the effect,
        // while INV-1's base judgment stays at the header where it is decided.
        let position = |rejection: &Rejection<'_>| -> Result<Vec<ProofPosition>, CheckStop> {
            let path = rejection.node_path();
            if let Rejection::LoopInvariant(outcome) = rejection
                && outcome.proof.base
                && outcome.proof.step == Some(false)
            {
                let node = self
                    .tree
                    .node_with_path(path)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if let Some(loop_node) = self.enclosing_loop_node(node)? {
                    let mut components = self
                        .tree
                        .path(loop_node)?
                        .components()
                        .iter()
                        .copied()
                        .map(ProofPosition::Child)
                        .collect::<Vec<_>>();
                    components.push(ProofPosition::AfterSubtree);
                    return Ok(components);
                }
            }
            Ok(path
                .components()
                .iter()
                .copied()
                .map(ProofPosition::Child)
                .collect())
        };
        let mut candidates = Vec::new();
        for rejection in loop_invariant
            .chain(source_proof)
            .chain(obligation)
            .chain(call)
        {
            candidates.push((position(&rejection)?, rejection));
        }
        // `min_by` keeps the first of several equal minima, so the selection
        // depends only on this order and on collection order, never on a hash.
        let rejection = candidates
            .into_iter()
            .min_by(|left, right| {
                left.0.cmp(&right.0).then_with(|| {
                    left.1
                        .rule()
                        .definition_rank()
                        .cmp(&right.1.rule().definition_rank())
                })
            })
            .map(|(_, rejection)| rejection);
        if let Some(rejection) = rejection {
            return match rejection {
                Rejection::LoopInvariant(outcome) => {
                    let node = self
                        .tree
                        .node_with_path(&outcome.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let obligation = if !outcome.proof.base {
                        crate::LoopInvariantProofObligation::Base
                    } else if outcome.proof.step == Some(false) {
                        crate::LoopInvariantProofObligation::Backedge
                    } else {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    };
                    let mechanical_fix = match obligation {
                        crate::LoopInvariantProofObligation::Base => {
                            "weaken or correct this invariant, or establish the missing facts before the loop so the invariant holds at the first loop header"
                        }
                        crate::LoopInvariantProofObligation::Backedge => {
                            "strengthen the invariant prefix, weaken or correct this invariant, or establish the missing body facts so every reachable normal fallthrough preserves it at the next loop header"
                        }
                    };
                    let required_relation = match obligation {
                        crate::LoopInvariantProofObligation::Base => outcome.base_target.clone(),
                        crate::LoopInvariantProofObligation::Backedge => {
                            outcome.backedge_target.clone()
                        }
                    };
                    Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Inv1,
                        location: SemanticLocation::SourceNode(
                            outcome.node_path.clone(),
                            self.tree.coordinate(node)?,
                        ),
                        kind: SemanticIssueKind::UndischargedLoopInvariant {
                            name: outcome.name.clone(),
                            obligation,
                            required_relation,
                            mechanical_fix,
                        },
                    }))
                }
                Rejection::SourceProof(outcome) => {
                    let rejection_node_path = outcome.rejection_node_path();
                    let node = self
                        .tree
                        .node_with_path(rejection_node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if let Some(failure) = outcome.check.target_failure {
                        let (reason, mechanical_fix) = match failure {
                            super::entailment::SourceProofCertificateFailure::ArithmeticOverflow => (
                                "the invariant target exceeds the i128 proof domain after current value images are substituted",
                                "split or rescale the invariant so its normalized current-value coefficients and constant fit i128",
                            ),
                            super::entailment::SourceProofCertificateFailure::FormationCapacity => (
                                "the invariant target exceeds a fixed affine formation capacity after current value images are substituted",
                                "split the invariant into smaller local invariants whose normalized current-value shapes fit the fixed capacities",
                            ),
                            super::entailment::SourceProofCertificateFailure::RepeatedUse { .. }
                            | super::entailment::SourceProofCertificateFailure::UseCapacity { .. }
                            | super::entailment::SourceProofCertificateFailure::InvalidFactor { .. } => {
                                return Err(SemanticCompilerFailure::InvalidResolution.into());
                            }
                        };
                        return Err(CheckStop::source_issue(SemanticIssue {
                            rule: SemanticRule::Inv1,
                            location: SemanticLocation::SourceNode(
                                rejection_node_path.clone(),
                                self.tree.coordinate(node)?,
                            ),
                            kind: SemanticIssueKind::InvalidInvariant {
                                reason,
                                mechanical_fix,
                            },
                        }));
                    }
                    if !outcome.certificate_written {
                        if !outcome.check.premises.is_empty()
                            || outcome.check.source_failure.is_some()
                            || outcome.check.certificate_failure.is_some()
                            || outcome.check.residual_failure.is_some()
                            || outcome.check.redundant
                            || outcome.check.combination
                        {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        return Err(CheckStop::source_issue(SemanticIssue {
                            rule: SemanticRule::Inv1,
                            location: SemanticLocation::SourceNode(
                                rejection_node_path.clone(),
                                self.tree.coordinate(node)?,
                            ),
                            kind: SemanticIssueKind::UndischargedLocalInvariant {
                                name: outcome.name.clone(),
                                mechanical_fix: "weaken or correct this invariant, or establish the missing facts before this statement so AUTO proves its target in the entering context",
                            },
                        }));
                    }
                    let failure_obligation = |failure| match failure {
                        super::entailment::SourceProofCertificateFailure::RepeatedUse {
                            first,
                            repeated,
                        } => crate::SourceProofObligation::RepeatedUse { first, repeated },
                        super::entailment::SourceProofCertificateFailure::UseCapacity {
                            maximum,
                            actual,
                        } => crate::SourceProofObligation::UseCapacity { maximum, actual },
                        super::entailment::SourceProofCertificateFailure::ArithmeticOverflow => {
                            crate::SourceProofObligation::CertificateArithmeticOverflow
                        }
                        super::entailment::SourceProofCertificateFailure::FormationCapacity => {
                            crate::SourceProofObligation::CertificateFormationCapacity
                        }
                        super::entailment::SourceProofCertificateFailure::InvalidFactor {
                            use_index,
                        } => crate::SourceProofObligation::InvalidUseFactor { use_index },
                    };
                    let obligation = if let Some(failure) = outcome.check.source_failure {
                        failure_obligation(failure)
                    } else if outcome.check.redundant {
                        crate::SourceProofObligation::RedundantUseBlock
                    } else if let Some(failure) = outcome.check.certificate_failure {
                        failure_obligation(failure)
                    } else if let Some(index) = outcome.check.first_unproved_premise {
                        crate::SourceProofObligation::Premise(index)
                    } else if let Some(failure) = outcome.check.residual_failure {
                        failure_obligation(failure)
                    } else if !outcome.check.combination {
                        crate::SourceProofObligation::Combination
                    } else {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    };
                    let mechanical_fix = match obligation {
                        crate::SourceProofObligation::Premise(_) => {
                            "establish this use relation from facts already available before the invariant statement, or replace it with a relation AUTO can prove in that same entering context"
                        }
                        crate::SourceProofObligation::Combination => {
                            "rewrite the invariant target, use relations, or explicit positive factors so their source-order weighted sum leaves a residual proved by the fixed direct L0 or interval rule"
                        }
                        crate::SourceProofObligation::RedundantUseBlock => {
                            "remove the use block; AUTO already proves this invariant target from the same entering context in this specification version"
                        }
                        crate::SourceProofObligation::RepeatedUse { .. } => {
                            "replace repeated normalized use relations with one use carrying their combined explicit positive factor"
                        }
                        crate::SourceProofObligation::UseCapacity { .. } => {
                            "split this local certificate into named intermediate invariants so every written use list is within the fixed structural capacity"
                        }
                        crate::SourceProofObligation::CertificateArithmeticOverflow => {
                            "split or rescale this certificate so every source-order proof-domain coefficient and constant operation fits i128"
                        }
                        crate::SourceProofObligation::CertificateFormationCapacity => {
                            "split this certificate into smaller named intermediate invariants whose canonical affine shapes fit the fixed formation capacities"
                        }
                        crate::SourceProofObligation::InvalidUseFactor { .. } => {
                            "write a canonical positive bare-decimal factor, or omit the factor when it is one"
                        }
                    };
                    Err(CheckStop::source_issue(SemanticIssue {
                        rule: SemanticRule::Prf1,
                        location: SemanticLocation::SourceNode(
                            rejection_node_path.clone(),
                            self.tree.coordinate(node)?,
                        ),
                        kind: SemanticIssueKind::UndischargedSourceProof {
                            name: outcome.name.clone(),
                            obligation,
                            mechanical_fix,
                        },
                    }))
                }
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
                                mechanical_fix: "when the relation must hold, establish the residual with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise restructure the access",
                            },
                        },
                        super::entailment::ObligationFamily::IntegerDomain => SemanticIssue {
                            rule: SemanticRule::Op2,
                            location,
                            kind: SemanticIssueKind::UndischargedIntegerDomainObligation {
                                residual,
                                disposition: if outcome.refuted {
                                    StaticObligationDisposition::Refuted
                                } else {
                                    StaticObligationDisposition::Unproved
                                },
                                mechanical_fix: "when the relation must hold, establish the fixed `.defined` normalization with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise use an available total non-exact row or restructure the arithmetic",
                            },
                        },
                        super::entailment::ObligationFamily::AllocationFit => SemanticIssue {
                            rule: SemanticRule::Op9,
                            location,
                            kind: SemanticIssueKind::UndischargedAllocationFitObligation {
                                residual,
                                mechanical_fix: "when the allocation must fit, establish `buffer_fits::<T>(n)` with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when allocation shortage is intended program behavior; otherwise restructure the allocation",
                            },
                        },
                        super::entailment::ObligationFamily::SystemRange => SemanticIssue {
                            rule: SemanticRule::Sys8,
                            location,
                            kind: SemanticIssueKind::UndischargedSystemRangeObligation {
                                residual,
                                mechanical_fix: "when the range must be valid, establish the residual with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise restructure the system range",
                            },
                        },
                        // [BLK-0]: a diagnostic arising in this domain cites
                        // BLK-0 and names the operation in its payload,
                        // exactly as an [OP-1] diagnostic names its family.
                        // The row is a declaration record with no source
                        // node, so the payload carries the row's own ordinal
                        // and the position of the requirement in that row's
                        // declared list and never a fabricated `NodePath`.
                        super::entailment::ObligationFamily::KernelRequirement => {
                            let operation = outcome
                                .kernel_row
                                .and_then(super::kernel::kernel_signature_at)
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                            SemanticIssue {
                                rule: SemanticRule::Blk0,
                                location,
                                kind: SemanticIssueKind::UndischargedKernelRequirement(Box::new(
                                    crate::UndischargedKernelRequirementDetail {
                                        operation: operation.spelling,
                                        operation_ordinal: outcome
                                            .kernel_row
                                            .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                                        requirement: u32::from(outcome.conjunct),
                                        instantiated_goal: residual.to_owned(),
                                        disposition: if outcome.refuted {
                                            crate::CallRequirementDisposition::Refuted
                                        } else {
                                            crate::CallRequirementDisposition::Unproved
                                        },
                                        mechanical_fix: "when the operation must succeed, establish the entire instantiated row requirement with a verified requirement, a source invariant, or explicit finite proof steps before the call; use a dominating branch only when rejection is intended program behavior; otherwise restructure the call",
                                    },
                                )),
                            }
                        }
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
                        "bind that argument or referent value with one preceding ordinary let, establish the entire instantiated requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow"
                    } else {
                        "when the call is required to succeed, establish the entire instantiated callee requirement with a verified requirement, a source invariant, or explicit finite proof steps before the call; use a dominating branch only when rejection is intended program behavior; otherwise restructure the call"
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
                                requires_clause: outcome.requires_clause.clone(),
                                instantiated_goal: outcome.rendered_goal.clone(),
                                disposition,
                                mechanical_fix,
                            },
                        )),
                    }))
                }
            };
        }

        if matches!(
            function.entailment.body_disposition,
            super::model::CheckedBodyDisposition::Uninhabited { .. }
        ) {
            return Ok(());
        }
        for proof in &function.entailment.postconditions {
            if proof.exits.is_empty() {
                let node = self
                    .tree
                    .node_with_path(&proof.selector)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                return Err(CheckStop::source_issue(SemanticIssue {
                    rule: SemanticRule::Fn9,
                    location: SemanticLocation::SourceNode(
                        proof.selector.clone(),
                        self.tree.coordinate(node)?,
                    ),
                    kind: SemanticIssueKind::NoSelectedNormalExit {
                        residual: "no selected normal exit",
                    },
                }));
            }
            let Some(exit) = proof.exits.iter().find(|exit| {
                exit.disposition != super::entailment::PostconditionDisposition::Discharged
            }) else {
                continue;
            };
            let disposition = match exit.disposition {
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
            return Err(CheckStop::source_issue(SemanticIssue {
                rule: SemanticRule::Fn9,
                location: SemanticLocation::SourceNode(
                    exit.statement.clone(),
                    self.tree.coordinate(node)?,
                ),
                kind: SemanticIssueKind::UndischargedPostcondition(Box::new(
                    crate::UndischargedPostconditionDetail {
                        concrete_function: function.symbol.clone(),
                        postcondition: proof.block.clone(),
                        conjunct: proof.relation_ordinal,
                        selector: proof.selector.clone(),
                        relation: exit.residual.clone(),
                        disposition,
                    },
                )),
            }));
        }
        Ok(())
    }
}
