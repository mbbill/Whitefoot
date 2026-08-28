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
mod result_state_origin;
mod strict;
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

use super::claim_locality::{BoundaryResultKind, ClaimAuthorityAnalysis};
use super::entailment::{
    CallGoalDisposition, CheckedGenericClaimSchema, ClaimCounterfactualWitness,
    ClaimFormationFailure, ClaimLocalityFailure, ClaimMask, ClaimMaskedDisposition,
    ClaimSchemaProofEvidence, ClaimSourceIdentity, ClaimTerminalOwner, ClaimTerminalRoot,
    DerivationRootKind, EntailmentCallee, EntailmentContext, FunctionPostconditionProof,
    PostconditionSchedule, VerifiedPostconditionSummary, analyze_function,
    analyze_function_candidate, analyze_function_candidate_masked, analyze_function_masked,
    build_claim_ledger, finalize_function_entailment, postcondition_schedule,
};
use super::goal::{
    CheckedCallRequirement, CheckedRequirement, ConcreteGoal, GoalDatum, GoalExpression,
    GoalOperation, GoalProjection, first_ephemeral_argument, render_goal,
};
use super::model::{
    BindingId, CheckedConst, CheckedConstant, CheckedConstantId, CheckedContract,
    CheckedExpression, CheckedFlatElement, CheckedFunction, CheckedGenericRequirement, CheckedMode,
    CheckedNominal, CheckedParameter, CheckedProgramData, CheckedResultStateOrigin,
    CheckedSetTarget, CheckedSliceOrigin, CheckedStateOrigins, CheckedStatement, CheckedType,
    CheckedValue, DerivedConst, DerivedConstId, FunctionId, NominalId, ValueInitializerKind,
    evaluate_const_operation,
};
use super::permission::{PermissionSignature, analyze_permission};
use super::permission_ledger::{LedgerSource, render_ledger};
use super::postcondition::{CheckedPostcondition, CheckedPostconditionSelector};
use super::provenance::{
    DatumSelector, FrozenProvenanceDependencies, ProvenanceContext,
    ProvenanceDemandKind as InternalDemandKind, ProvenanceFailures, ProvenanceMetadata,
    ProvenanceTarget, analyze_program_provenance_with_frozen, freeze_program_provenance,
};
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use borrows::{AccessKind, ResolvedPlace};
use borrows::{BorrowInfo, BorrowKind, SliceInfo, SliceLoan};
use control::{ControlCounters, ControlScope};
use generics::{GenericArgument, GenericParameter, GenericSubstitution, PendingGenericRequirement};

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
#[derive(Clone)]
struct CheckedFunctionInventory {
    function: CheckedFunction,
    binding_names: Vec<String>,
    claim_authority: ClaimAuthorityAnalysis,
}

struct ResidualProvenanceContext<'a> {
    failures: &'a ProvenanceFailures,
    frozen: &'a FrozenProvenanceDependencies,
    main: FunctionId,
}

/// One [CLM-2] counterfactual entailment reuse cache.
///
/// `claim_residuality_outcome` reruns the whole concrete inventory once per
/// claim component, and each rerun analyzes every function again. The
/// entailment walk reads its claim mask at exactly five points; three consult
/// only whether a mask is present, and the two that read the mask's identity
/// (`flow::Analyzer::walk_statement` and
/// `flow::Analyzer::establish_claim_contribution`) both additionally require
/// `mask.function == self.function.id`. The analysis of a function no mask
/// names is therefore one value shared by every masked rerun, and repeating it
/// is pure recomputation: on `tests/programs/wfgrep.wf` 82 of the 102
/// whole-function analyses are bit-identical repeats.
///
/// One entry holds that value beside the exact published-postcondition context
/// it was computed under. A reuse is admitted only when the analyzed function
/// is untargeted and that context still compares equal, so the reused
/// entailment is the same value the rerun would have produced. The cache is
/// scoped to one `claim_residuality_outcome` invocation, whose remaining
/// analysis inputs — the checked function itself, its binding names, its claim
/// authority, the callee, constant and nominal tables, and `optimistic_batch`
/// — are the same for every rerun. The analyzer never reads the function's own
/// `entailment` field, which is its output slot.
///
/// The entry lends its value rather than copying it. Each rerun's inventory is
/// the only place the entailment lives while that rerun runs, and
/// [`CounterfactualReuse::reclaim`] takes it back before the inventory is
/// dropped. A held copy would double the largest live structure in the check:
/// on `tests/programs/wfgrep.wf` one inventory of derivation arenas is half a
/// gigabyte.
///
/// What the entry holds is the analysis, not the inventory slot it was lent
/// to. The slot picks up one thing the analysis never produces: the SCC
/// scheduler stamps a [`VerifiedPostconditionSummary`] onto every postcondition
/// proof of a component it publishes, and whether a component publishes is a
/// decision each rerun takes again under its own mask. `reclaim` therefore
/// clears that stamp on the way back, so no rerun can read a summary a
/// different rerun published.
#[derive(Default)]
struct CounterfactualReuse {
    entries: Vec<Option<CounterfactualReuseEntry>>,
}

struct CounterfactualReuseEntry {
    verified_postconditions: Vec<Vec<CheckedPostcondition>>,
    verified_postcondition_proofs: Vec<Vec<FunctionPostconditionProof>>,
    /// Absent exactly while the rerun's own inventory holds this value.
    entailment: Option<super::entailment::FunctionEntailment>,
}

impl CounterfactualReuse {
    /// Lends the untargeted analysis of `index` when it was computed under
    /// exactly this published-postcondition context.
    fn take(
        &mut self,
        index: usize,
        verified_postconditions: &[Vec<&CheckedPostcondition>],
        verified_postcondition_proofs: &[Vec<&FunctionPostconditionProof>],
    ) -> Option<super::entailment::FunctionEntailment> {
        let entry = self.entries.get_mut(index)?.as_mut()?;
        if !Self::same(&entry.verified_postconditions, verified_postconditions)
            || !Self::same(
                &entry.verified_postcondition_proofs,
                verified_postcondition_proofs,
            )
        {
            return None;
        }
        entry.entailment.take()
    }

    /// Records the context of a value this rerun's inventory now holds.
    fn lend(
        &mut self,
        index: usize,
        verified_postconditions: &[Vec<&CheckedPostcondition>],
        verified_postcondition_proofs: &[Vec<&FunctionPostconditionProof>],
    ) {
        if self.entries.len() <= index {
            self.entries.resize_with(index + 1, || None);
        }
        self.entries[index] = Some(CounterfactualReuseEntry {
            verified_postconditions: Self::own(verified_postconditions),
            verified_postcondition_proofs: Self::own(verified_postcondition_proofs),
            entailment: None,
        });
    }

    /// Takes every lent value back out of one finished rerun's inventory.
    fn reclaim(&mut self, functions: &mut [CheckedFunctionInventory]) {
        for (index, checked) in functions.iter_mut().enumerate() {
            self.reclaim_one(index, &mut checked.function.entailment);
        }
    }

    /// Takes one lent value back, as the analysis produced it.
    ///
    /// An entry is lent exactly when its `entailment` is absent, so a function
    /// this rerun analyzed under a mask that names it — whose inventory slot
    /// therefore holds a masked value — keeps the untargeted value an earlier
    /// rerun recorded.
    ///
    /// The published FN-9 summaries are cleared because they belong to the
    /// rerun that published them rather than to the analysis: the analyzer
    /// emits `summary: None` for every postcondition proof, and the scheduler
    /// fills them in afterwards only for a component whose functions all
    /// discharged under *that* rerun's mask. Carrying them back would let a
    /// rerun whose component does not publish still read an earlier rerun's
    /// summaries as visible postconditions, which is a different proof search
    /// for every function scheduled after it.
    fn reclaim_one(
        &mut self,
        index: usize,
        entailment: &mut super::entailment::FunctionEntailment,
    ) {
        let Some(Some(entry)) = self.entries.get_mut(index) else {
            return;
        };
        if entry.entailment.is_some() {
            return;
        }
        let mut analyzed = std::mem::take(entailment);
        for proof in &mut analyzed.postconditions {
            proof.summary = None;
        }
        entry.entailment = Some(analyzed);
    }

    fn own<T: Clone>(borrowed: &[Vec<&T>]) -> Vec<Vec<T>> {
        borrowed
            .iter()
            .map(|entries| entries.iter().map(|entry| (*entry).clone()).collect())
            .collect()
    }

    fn same<T: PartialEq>(owned: &[Vec<T>], borrowed: &[Vec<&T>]) -> bool {
        owned.len() == borrowed.len()
            && owned.iter().zip(borrowed).all(|(owned, borrowed)| {
                owned.len() == borrowed.len()
                    && owned
                        .iter()
                        .zip(borrowed)
                        .all(|(owned, borrowed)| owned == *borrowed)
            })
    }
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

    /// Whether two joined states agree apart from facts whose finite union is
    /// the exact joined state: region-scoped claims and capability origins.
    fn same_except_region_claims(&self, other: &Self) -> bool {
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

    /// Union of region-scoped claims: a claim established on any joined path
    /// holds for the region remainder, matching [OWN-4]'s named-region
    /// liveness of the borrows that carry it.
    fn merge_region_claims_from(&mut self, other: &Self) {
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EffectSet {
    reads: Vec<super::model::CheckedStatePath>,
    writes: Vec<super::model::CheckedStatePath>,
    allocates_heap: bool,
    allocates_arenas: Vec<DeclarationId>,
    traps: bool,
}

impl EffectSet {
    const NONE: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        allocates_arenas: Vec::new(),
        traps: false,
    };
    const TRAPS: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        allocates_arenas: Vec::new(),
        traps: true,
    };
    const ALLOCATES_HEAP: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: true,
        allocates_arenas: Vec::new(),
        traps: false,
    };
    fn union(mut self, other: Self) -> Self {
        for path in other.reads {
            self.add_read(path);
        }
        for path in other.writes {
            self.add_write(path);
        }
        self.allocates_heap |= other.allocates_heap;
        for region in other.allocates_arenas {
            self.add_arena_allocation(region);
        }
        self.traps |= other.traps;
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
    /// Whether an undischarged obligation or refuted claim rejects [OP-4,
    /// CLM-2]. Always true outside `check_semantics_dark`, the test-only
    /// observability hook.
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
    generic_claim_schemas: Vec<CheckedGenericClaimSchema>,
    /// Owned CLM-1 canonical-formation failure from the symbolic generic
    /// source-schema batch. Formation precedes locality globally.
    generic_claim_schema_formation_issue: Option<SemanticIssue>,
    /// Owned CLM-1 failure from the symbolic generic source-schema batch.
    /// Claim-locality is judged before lifecycle, ordinary entailment, PRV,
    /// and residuality, so the checkpoint retains this source-stable issue
    /// independently of the later schema judgments.
    generic_claim_schema_locality_issue: Option<SemanticIssue>,
    /// First ordinary OP/FN source issue owned by a canonical symbolic
    /// generic body. It survives the symbolic checkpoint as source-stable
    /// diagnostic data and competes with concrete ordinary issues only after
    /// every claim lifecycle has succeeded.
    generic_claim_schema_entailment_issue: Option<SemanticIssue>,
    /// Owned PRV-2/3 failure from the symbolic generic source-schema batch.
    /// It is selected against the concrete batch only after ordinary
    /// admission and claim-lifecycle judgments have succeeded.
    generic_claim_schema_provenance_issue: Option<SemanticIssue>,
    postcondition_selectors: Vec<CheckedPostconditionSelector>,
    postcondition_unavailable_declarations: Vec<DeclarationId>,
    active_postcondition: Cell<Option<PostconditionCheckContext>>,
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
    /// not erase a written claim or a written call's declared effects. The
    /// release contribution is not syntactic and stays per instance [STOR-3].
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
            result_state_origins: RefCell::new(Vec::new()),
            deriving_result_state_origin: Cell::new(false),
            constants: HashMap::new(),
            checked_constants: Vec::new(),
            derived_consts: RefCell::new(Vec::new()),
            written_body_effect_rows: RefCell::new(HashMap::new()),
            pending_generic_requirements: Vec::new(),
            generic_requirements: Vec::new(),
            generic_claim_schemas: Vec::new(),
            generic_claim_schema_formation_issue: None,
            generic_claim_schema_locality_issue: None,
            generic_claim_schema_entailment_issue: None,
            generic_claim_schema_provenance_issue: None,
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
        self.validate_generic_templates()?;
        self.derive_result_state_origins()?;
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
        let callees = self.entailment_callees()?;
        self.install_call_requirements(&mut function_inventory)?;
        // CLM-2 counterfactuals always restart from these completed phase-A
        // functions. No baseline entailment or materialized parent may leak
        // into a masked rewalk.
        let claim_counterfactual_inventory = function_inventory.clone();
        let optimistic_batch = function_inventory.iter().any(|checked| {
            !checked.function.postconditions.is_empty()
                || Self::statements_contain_value_if(&checked.function.body)
        }) || !strict_markers.is_empty();

        let postcondition_schedule = self.analyze_function_inventory_with_mask(
            &mut function_inventory,
            &callees,
            optimistic_batch,
            None,
        )?;
        let baseline_functions = function_inventory
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();
        let mut formation_rejections = Vec::new();
        if let Some(issue) = self.generic_claim_schema_formation_issue.take() {
            let path = Self::source_issue_path(&issue)?.clone();
            formation_rejections.push((path, 0, issue));
        }
        for function in &baseline_functions {
            let rank = self.concrete_claim_instance_rank(function)?;
            let instance = self.concrete_claim_instance_name(function)?;
            for failure in &function.entailment.claim_formation_failures {
                formation_rejections.push((
                    failure.node_path.clone(),
                    rank,
                    self.claim_formation_issue(failure, instance.clone())?,
                ));
            }
        }
        formation_rejections.sort_by(|left, right| {
            left.0
                .components()
                .cmp(right.0.components())
                .then(left.1.cmp(&right.1))
        });
        if let Some((_, _, issue)) = formation_rejections.into_iter().next() {
            return Err(CheckStop::source_issue(issue));
        }
        let mut locality_rejections = Vec::new();
        if let Some(issue) = self.generic_claim_schema_locality_issue.take() {
            let path = Self::source_issue_path(&issue)?.clone();
            let SemanticIssueKind::NonLocalClaim(detail) = &issue.kind else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            locality_rejections.push((
                path,
                0,
                detail.component,
                detail.boundary_call.clone(),
                issue,
            ));
        }
        for function in &baseline_functions {
            let rank = self.concrete_claim_instance_rank(function)?;
            for failure in &function.entailment.claim_locality_failures {
                locality_rejections.push((
                    failure.node_path.clone(),
                    rank,
                    failure.component,
                    failure.boundary.call.clone(),
                    self.claim_locality_issue(failure)?,
                ));
            }
        }
        locality_rejections.sort_by(|left, right| {
            left.0
                .components()
                .cmp(right.0.components())
                .then(left.1.cmp(&right.1))
                .then(left.2.cmp(&right.2))
                .then_with(|| left.3.components().cmp(right.3.components()))
        });
        if let Some((_, _, _, _, issue)) = locality_rejections.into_iter().next() {
            return Err(CheckStop::source_issue(issue));
        }
        self.claim_lifecycle_rejection_global(&baseline_functions)?;
        if self.reject_entailment {
            let mut rejections = Vec::new();
            if let Some(issue) = self.generic_claim_schema_entailment_issue.take() {
                let path = Self::source_issue_path(&issue)?.clone();
                rejections.push((path, 0, issue.rule.definition_rank(), Box::new(issue)));
            }
            for function in &baseline_functions {
                match self.entailment_rejection(function) {
                    Ok(()) => {}
                    Err(CheckStop::Issue(issue)) => {
                        let path = Self::source_issue_path(&issue)?.clone();
                        rejections.push((
                            path,
                            self.concrete_claim_instance_rank(function)?,
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
        // Every baseline claim judgment above has been made, and the
        // inventory these name is moved into `functions` below.
        drop(baseline_functions);
        // [PRV-1] depends only on the immutable phase-A value/storage flow,
        // never on S3. Delay its one fixed point until the earlier CLM/ENT
        // gates pass, then freeze from the saved phase-A inventory and reuse
        // it for the baseline and every Full-minus mask. Each run still
        // recomputes the complete PRV-2/3 gate over its fresh proof views.
        let phase_a_functions = claim_counterfactual_inventory
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();
        let frozen_provenance = freeze_program_provenance(
            &phase_a_functions,
            &ProvenanceContext {
                nominals: &self.nominals,
                external_entry: Some(main),
            },
        )?;
        let mut functions = function_inventory
            .into_iter()
            .map(|checked| checked.function)
            .collect::<Vec<_>>();
        let provenance_context = ProvenanceContext {
            nominals: &self.nominals,
            external_entry: Some(main),
        };
        let mut provenance_analysis = analyze_program_provenance_with_frozen(
            &functions.iter().collect::<Vec<_>>(),
            &provenance_context,
            &frozen_provenance,
        )?;
        let concrete_provenance_issue = self.provenance_rejection(
            &functions,
            &provenance_analysis.metadata,
            &provenance_analysis.failures,
            None,
        )?;
        let schema_provenance_issue = self.generic_claim_schema_provenance_issue.take();
        let selected_provenance_issue = match (schema_provenance_issue, concrete_provenance_issue) {
            (Some(schema), Some(concrete)) => {
                let ordering = Self::source_issue_path(&schema)?
                    .components()
                    .cmp(Self::source_issue_path(&concrete)?.components())
                    .then_with(|| {
                        schema
                            .rule
                            .definition_rank()
                            .cmp(&concrete.rule.definition_rank())
                    });
                Some(if ordering.is_le() { schema } else { concrete })
            }
            (Some(schema), None) => Some(schema),
            (None, Some(concrete)) => Some(concrete),
            (None, None) => None,
        };
        if let Some(issue) = selected_provenance_issue {
            return Err(CheckStop::source_issue(issue));
        }
        let residual_provenance = ResidualProvenanceContext {
            failures: &provenance_analysis.failures,
            frozen: &frozen_provenance,
            main,
        };
        let concrete_residual = self.claim_residuality_outcome(
            &mut functions,
            &claim_counterfactual_inventory,
            &callees,
            optimistic_batch,
            &residual_provenance,
        )?;
        self.reject_first_residual_outcome(&functions, concrete_residual.as_ref())?;
        self.remove_uninhabited_generic_claim_reports(&mut functions)?;
        self.link_generic_claim_concrete_reports(&functions)?;
        // [CLM-3] consumes only the already-successful ordinary and PRV
        // scratch. It registers successful existing-U roots before the one
        // derivation finish and never reads the observational ClaimLedger.
        let strict_partition =
            self.check_strict_partition(&mut functions, &postcondition_schedule, strict_markers)?;
        if optimistic_batch {
            for function in &mut functions {
                finalize_function_entailment(&mut function.entailment);
            }
            provenance_analysis.refresh_entailment_views(&functions.iter().collect::<Vec<_>>());
        }
        for function in &mut functions {
            function.body_disposition = function.entailment.body_disposition;
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
                            let (display_path, coordinate) =
                                self.tree.source_identity(&claim.node_path)?;
                            Ok(ClaimSourceIdentity {
                                display_path,
                                coordinate,
                                node_path: claim.node_path.clone(),
                                declaration: function.declaration,
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
        // [PAR-1 candidate] permission is a read-only legality table over the
        // completed checked program: callable-boundary rows, resolved places,
        // statement exit edges, and the concrete call graph. It reads no
        // entailment fact state, so it is identical facts-on and facts-off.
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
            constants: self.checked_constants.clone(),
            derived_consts,
            functions,
            postcondition_schedule,
            strict_partition,
            provenance,
            generic_requirements: self.generic_requirements.clone(),
            generic_claim_schemas: self.generic_claim_schemas.clone(),
            contracts: self
                .contracts
                .iter()
                .map(|contract| contract.checked.clone())
                .collect(),
            conformances,
            law_derivations,
            main,
            entry,
            claim_ledger,
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

        let mut claim_names = Vec::new();
        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
            binding_names: &mut binding_names,
            claim_names: &mut claim_names,
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
        self.collect_release_sites(&checked.statements, &mut release_sites)?;
        let mut release = EffectSet::NONE;
        for site in &release_sites {
            release = release.union(site.effects.clone());
        }
        let exhibited = syntactic.clone().union(release.clone());
        if !self.deriving_result_state_origin.get() && exhibited != signature.declared_effects {
            // A state transition contributed only by a release has no offending
            // source occurrence. Keep the owner-bearing diagnostic for that
            // case even though the current system releases have empty memory
            // rows; later resource families may carry an ordinary memory row.
            let release_only = release.clone().union(syntactic.clone()) != syntactic
                && release.clone().union(signature.declared_effects.clone())
                    != signature.declared_effects;
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
            return self.issue_node(
                SemanticRule::Eff2,
                signature.effects_node,
                SemanticIssueKind::EffectMismatch,
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
            deny_claims_marker: signature.deny_claims_marker.clone(),
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
            declared_traps: signature.declared_effects.traps,
            declared_allocates_heap: signature.declared_effects.allocates_heap,
            declared_state_writes: signature.declared_effects.writes.clone(),
            target_action: crate::TargetAction::INLINE,
            requirements,
            postconditions,
            body: checked.statements,
            body_disposition: super::model::CheckedBodyDisposition::Inhabited,
            entailment: super::entailment::FunctionEntailment::default(),
        };
        let claim_authority = ClaimAuthorityAnalysis::analyze(&function, &self.nominals)?;
        Ok(CheckedFunctionInventory {
            function,
            binding_names,
            claim_authority,
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
            | CheckedStatement::Claim { .. }
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

    /// Runs CLM-2's source-schema lifecycle and Full-minus judgments while
    /// symbolic nominal and function identities are still alive. Only stable
    /// source paths, renderings, dispositions, and terminal-root keys survive
    /// the enclosing generic-validation checkpoint.
    fn evaluate_generic_claim_schemas(
        &mut self,
        phase_a: &[CheckedFunctionInventory],
        canonical: &[(usize, DeclarationId)],
        callees: &[EntailmentCallee],
    ) -> Result<(), CheckStop> {
        let optimistic_batch = phase_a.iter().any(|checked| {
            !checked.function.postconditions.is_empty()
                || Self::statements_contain_value_if(&checked.function.body)
        });
        let mut full = phase_a.to_vec();
        self.analyze_function_inventory_with_mask(&mut full, callees, optimistic_batch, None)?;
        let mut formation_failures = canonical
            .iter()
            .flat_map(|(index, declaration)| {
                full.get(*index)
                    .filter(|checked| checked.function.declaration == *declaration)
                    .into_iter()
                    .flat_map(|checked| &checked.function.entailment.claim_formation_failures)
            })
            .collect::<Vec<_>>();
        formation_failures.sort_by(|left, right| {
            left.node_path
                .components()
                .cmp(right.node_path.components())
        });
        self.generic_claim_schema_formation_issue = formation_failures
            .first()
            .map(|failure| self.claim_formation_issue(failure, None))
            .transpose()?;
        if self.generic_claim_schema_formation_issue.is_some() {
            self.generic_claim_schema_locality_issue = None;
            self.generic_claim_schema_entailment_issue = None;
            self.generic_claim_schema_provenance_issue = None;
            self.generic_claim_schemas.clear();
            return Ok(());
        }
        let mut locality_failures = canonical
            .iter()
            .flat_map(|(index, declaration)| {
                full.get(*index)
                    .filter(|checked| checked.function.declaration == *declaration)
                    .into_iter()
                    .flat_map(|checked| &checked.function.entailment.claim_locality_failures)
            })
            .collect::<Vec<_>>();
        locality_failures.sort_by(|left, right| {
            left.node_path
                .components()
                .cmp(right.node_path.components())
                .then(left.component.cmp(&right.component))
                .then_with(|| {
                    left.boundary
                        .call
                        .components()
                        .cmp(right.boundary.call.components())
                })
        });
        self.generic_claim_schema_locality_issue = locality_failures
            .first()
            .map(|failure| self.claim_locality_issue(failure))
            .transpose()?;
        if self.generic_claim_schema_locality_issue.is_some() {
            self.generic_claim_schema_entailment_issue = None;
            self.generic_claim_schema_provenance_issue = None;
            self.generic_claim_schemas.clear();
            return Ok(());
        }
        let full_schema_functions = full
            .iter()
            .map(|checked| checked.function.clone())
            .collect::<Vec<_>>();
        let full_schema_refs = full_schema_functions.iter().collect::<Vec<_>>();
        let schema_provenance_context = ProvenanceContext {
            nominals: &self.nominals,
            external_entry: None,
        };
        let frozen_schema_provenance =
            freeze_program_provenance(&full_schema_refs, &schema_provenance_context)?;
        let full_schema_provenance = analyze_program_provenance_with_frozen(
            &full_schema_refs,
            &schema_provenance_context,
            &frozen_schema_provenance,
        )?;

        let has_lifecycle_failure =
            canonical
                .iter()
                .try_fold(false, |found, (index, declaration)| {
                    let checked = full
                        .get(*index)
                        .filter(|checked| checked.function.declaration == *declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    Ok::<_, CheckStop>(
                        found
                            || checked.function.entailment.claims.iter().any(|claim| {
                                claim.disposition != super::entailment::ClaimDisposition::Retained
                            }),
                    )
                })?;

        self.generic_claim_schema_entailment_issue = None;
        if !has_lifecycle_failure && self.reject_entailment {
            let mut rejections = Vec::new();
            for (index, declaration) in canonical {
                let checked = full
                    .get(*index)
                    .filter(|checked| checked.function.declaration == *declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut function = checked.function.clone();
                function.symbol.clone_from(&function.name);
                match self.entailment_rejection(&function) {
                    Ok(()) => {}
                    Err(CheckStop::Issue(mut issue)) => {
                        let path = Self::source_issue_path(&issue)?.clone();
                        if let SemanticIssueKind::UndischargedCallRequirement(detail) =
                            &mut issue.kind
                        {
                            let call = checked
                                .function
                                .entailment
                                .call_goals
                                .iter()
                                .find(|call| {
                                    call.node_path == path
                                        && call.requires_clause == detail.requires_clause
                                })
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                            let signature = self
                                .signatures
                                .get(call.callee.0 as usize)
                                .filter(|signature| signature.id == call.callee)
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                            detail.concrete_callee.clone_from(&signature.name);
                            detail.instantiated_goal =
                                self.render_stable_schema_goal(&call.goal.root)?;
                        }
                        rejections.push((path, issue.rule.definition_rank(), *issue));
                    }
                    Err(stop) => return Err(stop),
                }
            }
            rejections.sort_by(|left, right| {
                left.0
                    .components()
                    .cmp(right.0.components())
                    .then(left.1.cmp(&right.1))
            });
            self.generic_claim_schema_entailment_issue =
                rejections.into_iter().next().map(|(_, _, issue)| issue);
        }

        self.generic_claim_schema_provenance_issue =
            if has_lifecycle_failure || self.generic_claim_schema_entailment_issue.is_some() {
                None
            } else {
                let mut diagnostic_functions = full_schema_functions.clone();
                for function in &mut diagnostic_functions {
                    function.symbol.clone_from(&function.name);
                }
                let schema_owners = canonical
                    .iter()
                    .map(|(index, _)| {
                        full_schema_functions
                            .get(*index)
                            .map(|function| function.id)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.provenance_rejection(
                    &diagnostic_functions,
                    &full_schema_provenance.metadata,
                    &full_schema_provenance.failures,
                    Some(&schema_owners),
                )?
            };

        if !has_lifecycle_failure
            && self.generic_claim_schema_entailment_issue.is_none()
            && self.generic_claim_schema_provenance_issue.is_none()
        {
            let mut reuse = CounterfactualReuse::default();
            'claims: for (function_index, declaration) in canonical {
                let claim_count = full[*function_index].function.entailment.claims.len();
                for claim_index in 0..claim_count {
                    let node_path = full[*function_index].function.entailment.claims[claim_index]
                        .node_path
                        .clone();
                    let component_count = full[*function_index].function.entailment.claims
                        [claim_index]
                        .components
                        .len();
                    for component_index in 0..component_count {
                        let component = u32::try_from(component_index)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                        let mask = ClaimMask {
                            function: full[*function_index].function.id,
                            node_path: node_path.clone(),
                            component: Some(component),
                        };
                        let mut scratch = phase_a.to_vec();
                        self.analyze_function_inventory_masked(
                            &mut scratch,
                            callees,
                            optimistic_batch,
                            &mask,
                            &mut reuse,
                        )?;
                        let witness = self.counterfactual_witness(
                            &full_schema_refs,
                            &scratch,
                            &mask,
                            &full_schema_provenance.failures,
                            &frozen_schema_provenance,
                            None,
                        )?;
                        reuse.reclaim(&mut scratch);
                        let Some(witness) = witness else {
                            full[*function_index].function.entailment.claims[claim_index]
                                .disposition = super::entailment::ClaimDisposition::NonResidual {
                                component: Some(component),
                            };
                            break 'claims;
                        };
                        full[*function_index].function.entailment.claims[claim_index]
                            .residual_witnesses
                            .push(witness);
                    }
                    if full[*function_index].function.entailment.claims[claim_index].disposition
                        != super::entailment::ClaimDisposition::Retained
                    {
                        break 'claims;
                    }
                    if component_count == 1 {
                        let component_witnesses = &full[*function_index].function.entailment.claims
                            [claim_index]
                            .residual_witnesses;
                        if component_witnesses.len() != 1
                            || component_witnesses[0].component != Some(0)
                        {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        let mut whole = component_witnesses[0].clone();
                        whole.component = None;
                        full[*function_index].function.entailment.claims[claim_index]
                            .residual_witnesses
                            .push(whole);
                        continue;
                    }
                    let mask = ClaimMask {
                        function: full[*function_index].function.id,
                        node_path,
                        component: None,
                    };
                    let mut scratch = phase_a.to_vec();
                    self.analyze_function_inventory_masked(
                        &mut scratch,
                        callees,
                        optimistic_batch,
                        &mask,
                        &mut reuse,
                    )?;
                    let witness = self.counterfactual_witness(
                        &full_schema_refs,
                        &scratch,
                        &mask,
                        &full_schema_provenance.failures,
                        &frozen_schema_provenance,
                        None,
                    )?;
                    reuse.reclaim(&mut scratch);
                    let Some(witness) = witness else {
                        full[*function_index].function.entailment.claims[claim_index].disposition =
                            super::entailment::ClaimDisposition::NonResidual { component: None };
                        break 'claims;
                    };
                    full[*function_index].function.entailment.claims[claim_index]
                        .residual_witnesses
                        .push(witness);
                }
                if full[*function_index]
                    .function
                    .entailment
                    .claims
                    .iter()
                    .any(|claim| claim.disposition != super::entailment::ClaimDisposition::Retained)
                {
                    break;
                }
                if full[*function_index].function.declaration != *declaration {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
        }

        let mut reports = Vec::with_capacity(canonical.len());
        for (index, declaration) in canonical {
            let signature = self
                .signatures
                .get(*index)
                .filter(|signature| signature.declaration == *declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let checked = full
                .get(*index)
                .filter(|checked| checked.function.declaration == *declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let mut claims = checked.function.entailment.claims.clone();
            for (occurrence, claim) in claims.iter_mut().enumerate() {
                claim.lifecycle_derivation = None;
                if let Some(proof) = claim.proof.take() {
                    let (schema_proof, components) = self.stabilize_schema_proof(
                        &checked.function.entailment,
                        u32::try_from(occurrence)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                        proof,
                    )?;
                    claim.schema_proof = Some(schema_proof);
                    claim.components = components;
                }
                if claim.disposition == super::entailment::ClaimDisposition::Retained
                    && claim.schema_proof.is_none()
                {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                for witness in &mut claim.residual_witnesses {
                    Self::stabilize_schema_terminal(&mut witness.terminal, &full_schema_refs)?;
                    let owner = match &witness.terminal {
                        ClaimTerminalRoot::Obligation { owner, .. }
                        | ClaimTerminalRoot::Call { owner, .. }
                        | ClaimTerminalRoot::Postcondition { owner, .. } => *owner,
                    };
                    if owner != ClaimTerminalOwner::Schema(*declaration) {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                }
            }
            reports.push(CheckedGenericClaimSchema {
                declaration: *declaration,
                function_path: self.tree.path(signature.node)?.clone(),
                display_symbol: checked.function.name.clone(),
                claims,
                concrete_reports: Vec::new(),
            });
        }
        self.generic_claim_schemas = reports;
        Ok(())
    }

    fn stabilize_schema_proof(
        &self,
        entailment: &super::entailment::FunctionEntailment,
        occurrence: u32,
        proof: super::entailment::ClaimProofEvidence,
    ) -> Result<(ClaimSchemaProofEvidence, Vec<String>), CheckStop> {
        let render_image = |goal: super::entailment::GoalId| {
            let index =
                usize::try_from(goal.0).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let expression = entailment
                .inventory
                .goals
                .get(index)
                .map(|goal| &goal.expression)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.render_stable_schema_goal(expression)
        };
        let expanded_root = entailment.derivations.roots.iter().any(|root| {
            root.kind
                == (DerivationRootKind::ClaimReconstruction {
                    occurrence,
                    direct: false,
                })
                && root.node == proof.reconstructions.expanded
        });
        let direct_root = entailment.derivations.roots.iter().any(|root| {
            root.kind
                == (DerivationRootKind::ClaimReconstruction {
                    occurrence,
                    direct: true,
                })
                && root.node == proof.reconstructions.direct
        });
        if !expanded_root || !direct_root {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let mut components = Vec::with_capacity(proof.components.len());
        for component in &proof.components {
            components.push(match component.fact {
                super::entailment::ClaimComponentFact::Goal { goal, sign } => {
                    let index = usize::try_from(goal.0)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                    let expression = entailment
                        .inventory
                        .goals
                        .get(index)
                        .map(|goal| &goal.expression)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    format!("{sign:?} {}", self.render_stable_schema_goal(expression)?)
                }
                super::entailment::ClaimComponentFact::Relation(_) => component.rendering.clone(),
            });
        }
        Ok((
            ClaimSchemaProofEvidence {
                direct_image: render_image(proof.images.direct)?,
                expanded_image: render_image(proof.images.expanded)?,
                complete_image: render_image(proof.images.complete)?,
                reconstruction_succeeded: true,
            },
            components,
        ))
    }

    fn stabilize_schema_terminal(
        terminal: &mut ClaimTerminalRoot,
        functions: &[&CheckedFunction],
    ) -> Result<(), CheckStop> {
        fn schema_owner(
            owner: ClaimTerminalOwner,
            symbol: &str,
            functions: &[&CheckedFunction],
        ) -> Result<(ClaimTerminalOwner, String), SemanticCompilerFailure> {
            match owner {
                ClaimTerminalOwner::Concrete(function) => functions
                    .get(function.0 as usize)
                    .filter(|checked| checked.id == function)
                    .filter(|checked| checked.symbol == symbol)
                    .map(|checked| {
                        (
                            ClaimTerminalOwner::Schema(checked.declaration),
                            checked.name.clone(),
                        )
                    })
                    .ok_or(SemanticCompilerFailure::InvalidResolution),
                ClaimTerminalOwner::Schema(_) => Err(SemanticCompilerFailure::InvalidResolution),
            }
        }

        match terminal {
            ClaimTerminalRoot::Obligation {
                owner,
                function_symbol,
                ..
            }
            | ClaimTerminalRoot::Postcondition {
                owner,
                function_symbol,
                ..
            } => {
                let (stable_owner, stable_symbol) =
                    schema_owner(*owner, function_symbol, functions)?;
                *owner = stable_owner;
                *function_symbol = stable_symbol;
            }
            ClaimTerminalRoot::Call {
                owner,
                function_symbol,
                callee,
                callee_symbol,
                ..
            } => {
                let (stable_owner, stable_function_symbol) =
                    schema_owner(*owner, function_symbol, functions)?;
                let (stable_callee, stable_callee_symbol) =
                    schema_owner(*callee, callee_symbol, functions)?;
                *owner = stable_owner;
                *function_symbol = stable_function_symbol;
                *callee = stable_callee;
                *callee_symbol = stable_callee_symbol;
            }
        }
        Ok(())
    }

    fn analyze_function_inventory_masked(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        mask: &ClaimMask,
        reuse: &mut CounterfactualReuse,
    ) -> Result<PostconditionSchedule, CheckStop> {
        self.analyze_function_inventory_reusing(
            functions,
            callees,
            optimistic_batch,
            Some(mask),
            Some(reuse),
        )
    }

    fn analyze_function_inventory_with_mask(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        mask: Option<&ClaimMask>,
    ) -> Result<PostconditionSchedule, CheckStop> {
        self.analyze_function_inventory_reusing(functions, callees, optimistic_batch, mask, None)
    }

    fn analyze_function_inventory_reusing(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        mask: Option<&ClaimMask>,
        mut reuse: Option<&mut CounterfactualReuse>,
    ) -> Result<PostconditionSchedule, CheckStop> {
        // The [ENT] engine is acceptance-bearing [ENT-1]: it computes the
        // closed fact states, obligation and ordinary-call goal dispositions,
        // and claim lifecycle dispositions. The first offending OP-4, FN-8,
        // or CLM-2 node in document/rule order is cited; every invalid claim
        // classification is a source rejection rather than an advisory.
        let mut schedule =
            postcondition_schedule(functions.iter().map(|checked| &checked.function))
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if schedule.components.is_empty() {
            for (index, checked) in functions.iter_mut().enumerate() {
                let context = EntailmentContext {
                    callees,
                    constants: &self.checked_constants,
                    constant_ids: &self.constants,
                    nominals: &self.nominals,
                    verified_postconditions: &[],
                    verified_postcondition_proofs: &[],
                    binding_names: &checked.binding_names,
                    claim_authority: &checked.claim_authority,
                };
                let untargeted = mask.is_some_and(|mask| mask.function != checked.function.id);
                if untargeted
                    && let Some(reuse) = reuse.as_deref_mut()
                    && let Some(lent) = reuse.take(index, &[], &[])
                {
                    checked.function.entailment = lent;
                    continue;
                }
                let entailment = match (optimistic_batch, mask) {
                    (true, Some(mask)) => {
                        analyze_function_candidate_masked(&checked.function, &context, mask)
                    }
                    (false, Some(mask)) => {
                        analyze_function_masked(&checked.function, &context, mask)
                    }
                    (true, None) => analyze_function_candidate(&checked.function, &context),
                    (false, None) => analyze_function(&checked.function, &context),
                };
                if untargeted && let Some(reuse) = reuse.as_deref_mut() {
                    reuse.lend(index, &[], &[]);
                }
                checked.function.entailment = entailment;
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
                        claim_authority: &checked.claim_authority,
                    };
                    let untargeted = mask.is_some_and(|mask| mask.function != checked.function.id);
                    let lent = if untargeted {
                        reuse.as_deref_mut().and_then(|reuse| {
                            reuse.take(
                                function_index,
                                &verified_postconditions,
                                &verified_postcondition_proofs,
                            )
                        })
                    } else {
                        None
                    };
                    let reanalyzed = lent.is_none();
                    let entailment = match (lent, mask) {
                        (Some(lent), _) => lent,
                        (None, Some(mask)) => {
                            analyze_function_candidate_masked(&checked.function, &context, mask)
                        }
                        (None, None) => analyze_function_candidate(&checked.function, &context),
                    };
                    if untargeted
                        && reanalyzed
                        && let Some(reuse) = reuse.as_deref_mut()
                    {
                        reuse.lend(
                            function_index,
                            &verified_postconditions,
                            &verified_postcondition_proofs,
                        );
                    }
                    drop(verified_postconditions);
                    drop(verified_postcondition_proofs);
                    functions[function_index].function.entailment = entailment;
                }

                let publish = component.functions.iter().all(|function| {
                    let checked = &functions[function.0 as usize].function;
                    matches!(
                        checked.entailment.body_disposition,
                        super::model::CheckedBodyDisposition::Uninhabited { .. }
                    ) || checked.postconditions.is_empty()
                        || (checked.entailment.postconditions.len() == checked.postconditions.len()
                            && checked
                                .entailment
                                .postconditions
                                .iter()
                                .all(|proof| proof.complete.discharged))
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

    fn concrete_claim_instance_rank(
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

    fn stable_claim_type_name(&self, ty: CheckedType) -> Result<String, CheckStop> {
        Ok(match ty {
            CheckedType::Nominal(id) => {
                if let Some((template_index, substitution)) = self
                    .source_nominal_instances
                    .get(id.0 as usize)
                    .and_then(Clone::clone)
                {
                    let template = self
                        .nominal_templates
                        .get(template_index)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if substitution.len() == 0 {
                        template.name.clone()
                    } else {
                        let arguments = substitution
                            .entries()
                            .iter()
                            .map(|(_, argument)| self.stable_claim_argument_name(*argument))
                            .collect::<Result<Vec<_>, _>>()?;
                        format!("{}<{}>", template.name, arguments.join(", "))
                    }
                } else if let Some((referent, _)) = self
                    .box_nominals
                    .iter()
                    .find(|(_, candidate)| **candidate == id)
                {
                    format!("box<{}>", self.stable_claim_type_name(*referent)?)
                } else if let Some(((region, content), _)) = self
                    .arena_nominals
                    .iter()
                    .find(|(_, candidate)| **candidate == id)
                {
                    format!(
                        "arena<'region#{}, {}>",
                        region.index(),
                        self.stable_claim_type_name(*content)?
                    )
                } else {
                    match self.prelude_types.get(id.0 as usize).copied().flatten() {
                        Some(PreludeType::Option(value)) => {
                            format!("Option<{}>", self.stable_claim_type_name(value)?)
                        }
                        Some(PreludeType::Result(ok, error)) => format!(
                            "Result<{}, {}>",
                            self.stable_claim_type_name(ok)?,
                            self.stable_claim_type_name(error)?
                        ),
                        Some(PreludeType::Overflow) => "Overflow".to_owned(),
                        Some(PreludeType::DivError) => "DivError".to_owned(),
                        Some(PreludeType::NarrowError) => "NarrowError".to_owned(),
                        None => self.nominal(id)?.name.clone(),
                    }
                }
            }
            CheckedType::Array { element, length } => format!(
                "array<{}, {}>",
                self.stable_claim_type_name(element.ty())?,
                self.checked_const_name(length)?
            ),
            CheckedType::Slice { region, element } => format!(
                "slice<'region#{}, {}>",
                region.index(),
                self.stable_claim_type_name(element.ty())?
            ),
            CheckedType::Buffer { element } => {
                format!("buffer<{}>", self.stable_claim_type_name(element.ty())?)
            }
            primitive => self.checked_type_name(primitive)?,
        })
    }

    fn render_stable_schema_goal(&self, expression: &GoalExpression) -> Result<String, CheckStop> {
        match expression {
            GoalExpression::Datum(datum) => self.render_stable_schema_datum(datum),
            GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            } => {
                let type_arguments = type_arguments
                    .iter()
                    .map(|ty| self.stable_claim_type_name(*ty))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                let const_arguments = const_arguments
                    .iter()
                    .map(|value| self.checked_const_name(*value))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                let arguments = arguments
                    .iter()
                    .map(|argument| self.render_stable_schema_goal(argument))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!(
                    "{}<types=[{}], consts=[{}]>({arguments}):{}",
                    self.render_stable_schema_operation(row)?,
                    type_arguments,
                    const_arguments,
                    self.stable_claim_type_name(*result)?
                ))
            }
        }
    }

    fn render_stable_schema_datum(&self, datum: &GoalDatum) -> Result<String, CheckStop> {
        Ok(match datum {
            GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            } => format!(
                "parameter#{ordinal}{}:{}",
                Self::render_stable_schema_projections(projections),
                self.stable_claim_type_name(*ty)?
            ),
            GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            } => format!(
                "const#{}{}:{}",
                declaration.index(),
                Self::render_stable_schema_projections(projections),
                self.stable_claim_type_name(*ty)?
            ),
            GoalDatum::Place {
                root,
                projections,
                ty,
            } => format!(
                "place#{}{}:{}",
                root.0,
                Self::render_stable_schema_projections(projections),
                self.stable_claim_type_name(*ty)?
            ),
            GoalDatum::EphemeralActual {
                caller,
                call,
                argument,
                captured_type,
                projections,
                ty,
            } => {
                let caller = self
                    .signatures
                    .get(caller.0 as usize)
                    .filter(|signature| signature.id == *caller)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                format!(
                    "argument#{argument}(caller={}#decl{}, call={call:?}, captured={}, projections={}, type={})",
                    caller.name,
                    caller.declaration.index(),
                    self.stable_claim_type_name(*captured_type)?,
                    Self::render_stable_schema_projections(projections),
                    self.stable_claim_type_name(*ty)?
                )
            }
            GoalDatum::Literal(value) => self.render_stable_schema_value(value)?,
        })
    }

    fn render_stable_schema_operation(&self, row: &GoalOperation) -> Result<String, CheckStop> {
        Ok(match row {
            GoalOperation::Integer {
                operation,
                operand_type,
            } => format!(
                "Integer({operation:?}, operand={})",
                self.stable_claim_type_name(*operand_type)?
            ),
            GoalOperation::Float {
                operation,
                operand_type,
            } => format!(
                "Float({operation:?}, operand={})",
                self.stable_claim_type_name(*operand_type)?
            ),
            GoalOperation::NumericConversion {
                source,
                destination,
            } => format!("NumericConversion({source:?}->{destination:?})"),
            GoalOperation::Reinterpret {
                source,
                destination,
            } => format!("Reinterpret({source:?}->{destination:?})"),
            GoalOperation::Boolean(operation) => format!("Boolean({operation:?})"),
            GoalOperation::EnumEquality {
                equal,
                operand_type,
            } => format!(
                "EnumEquality(equal={equal}, operand={})",
                self.stable_claim_type_name(*operand_type)?
            ),
            GoalOperation::ArrayFill { element, length } => format!(
                "ArrayFill(element={}, length={})",
                self.stable_claim_type_name(element.ty())?,
                self.checked_const_name(*length)?
            ),
            GoalOperation::ArrayLength { element, length } => format!(
                "ArrayLength(element={}, length={})",
                self.stable_claim_type_name(element.ty())?,
                self.checked_const_name(*length)?
            ),
            GoalOperation::BufferLength { element } => format!(
                "BufferLength(element={})",
                self.stable_claim_type_name(element.ty())?
            ),
            GoalOperation::BufferFits {
                element,
                maximum_length,
            } => format!(
                "BufferFits(element={}, maximum_length={maximum_length})",
                self.stable_claim_type_name(*element)?
            ),
            GoalOperation::SliceLength { region, element } => format!(
                "SliceLength(region={}, element={})",
                region.index(),
                self.stable_claim_type_name(element.ty())?
            ),
        })
    }

    fn render_stable_schema_value(&self, value: &CheckedValue) -> Result<String, CheckStop> {
        Ok(match value {
            CheckedValue::Unit => "unit".to_owned(),
            CheckedValue::Bool(value) => value.to_string(),
            CheckedValue::Integer { ty, bits } => format!("Integer({ty:?}, bits={bits})"),
            CheckedValue::Float { ty, bits } => format!("Float({ty:?}, bits={bits})"),
            CheckedValue::NumericIdentity { ty, one } => format!(
                "NumericIdentity(type={}, one={one})",
                self.stable_claim_type_name(*ty)?
            ),
            CheckedValue::Array { ty, elements } => {
                let elements = elements
                    .iter()
                    .map(|element| self.render_stable_schema_value(element))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!(
                    "Array(type={}, elements=[{elements}])",
                    self.stable_claim_type_name(*ty)?
                )
            }
            CheckedValue::Struct { ty, fields } => {
                let fields = fields
                    .iter()
                    .map(|field| self.render_stable_schema_value(field))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!(
                    "Struct(type={}, fields=[{fields}])",
                    self.stable_claim_type_name(*ty)?
                )
            }
        })
    }

    fn render_stable_schema_projections(projections: &[GoalProjection]) -> String {
        let projections = projections
            .iter()
            .map(|projection| match projection {
                GoalProjection::Deref => "deref".to_owned(),
                GoalProjection::Field(field) => format!("field#{field}"),
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("[{projections}]")
    }

    fn stable_claim_argument_name(&self, argument: GenericArgument) -> Result<String, CheckStop> {
        match argument {
            GenericArgument::Type(ty) => self.stable_claim_type_name(ty),
            GenericArgument::Const(value) => self.checked_const_name(value),
        }
    }

    fn concrete_claim_instance_name(
        &self,
        function: &CheckedFunction,
    ) -> Result<Option<String>, CheckStop> {
        let signature = self
            .signatures
            .get(function.id.0 as usize)
            .filter(|signature| signature.id == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if signature.substitution.len() == 0 {
            return Ok(None);
        }
        let arguments = signature
            .substitution
            .entries()
            .iter()
            .map(|(_, argument)| self.stable_claim_argument_name(*argument))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(format!("{}<{}>", function.name, arguments.join(", "))))
    }

    fn uninhabited_concrete_generic(
        &self,
        function: &CheckedFunction,
    ) -> Result<bool, SemanticCompilerFailure> {
        let signature = self
            .signatures
            .get(function.id.0 as usize)
            .filter(|signature| signature.id == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(signature.substitution.len() != 0
            && signature.substitution.is_concrete()
            && matches!(
                function.entailment.body_disposition,
                super::model::CheckedBodyDisposition::Uninhabited { .. }
            ))
    }

    fn first_schema_invalid_claim(&self) -> Option<&super::entailment::ClaimOutcome> {
        self.generic_claim_schemas
            .iter()
            .flat_map(|schema| &schema.claims)
            .filter(|claim| claim.disposition != super::entailment::ClaimDisposition::Retained)
            .min_by(|left, right| {
                left.node_path
                    .components()
                    .cmp(right.node_path.components())
            })
    }

    fn claim_lifecycle_rejection_global(
        &self,
        functions: &[&CheckedFunction],
    ) -> Result<(), CheckStop> {
        let mut invalid = Vec::new();
        for function in functions {
            if self.uninhabited_concrete_generic(function)? {
                continue;
            }
            let rank = self.concrete_claim_instance_rank(function)?;
            let instance = self.concrete_claim_instance_name(function)?;
            for claim in &function.entailment.claims {
                if claim.disposition != super::entailment::ClaimDisposition::Retained {
                    invalid.push((claim.node_path.clone(), rank, instance.clone(), claim));
                }
            }
        }
        for schema in &self.generic_claim_schemas {
            for claim in &schema.claims {
                if claim.disposition != super::entailment::ClaimDisposition::Retained
                    && !matches!(
                        claim.disposition,
                        super::entailment::ClaimDisposition::NonResidual { .. }
                    )
                {
                    invalid.push((claim.node_path.clone(), 0, None, claim));
                }
            }
        }
        invalid.sort_by(|left, right| {
            left.0
                .components()
                .cmp(right.0.components())
                .then(left.1.cmp(&right.1))
        });
        match invalid.first() {
            Some((_, _, instance, claim)) => self.claim_outcome_rejection(claim, instance.clone()),
            None => Ok(()),
        }
    }

    fn reject_first_residual_outcome(
        &self,
        functions: &[CheckedFunction],
        concrete: Option<&(FunctionId, super::entailment::ClaimOutcome)>,
    ) -> Result<(), CheckStop> {
        let schema = self.first_schema_invalid_claim().filter(|claim| {
            matches!(
                claim.disposition,
                super::entailment::ClaimDisposition::NonResidual { .. }
            )
        });
        let selected = match (schema, concrete) {
            (Some(schema), Some((function, concrete))) => {
                if schema.node_path.components() <= concrete.node_path.components() {
                    Some((schema, None))
                } else {
                    Some((
                        concrete,
                        Some(
                            functions
                                .get(function.0 as usize)
                                .filter(|checked| checked.id == *function)
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                        ),
                    ))
                }
            }
            (Some(schema), None) => Some((schema, None)),
            (None, Some((function, concrete))) => Some((
                concrete,
                Some(
                    functions
                        .get(function.0 as usize)
                        .filter(|checked| checked.id == *function)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                ),
            )),
            (None, None) => None,
        };
        match selected {
            Some((claim, function)) => self.claim_outcome_rejection(
                claim,
                function
                    .map(|function| self.concrete_claim_instance_name(function))
                    .transpose()?
                    .flatten(),
            ),
            None => Ok(()),
        }
    }

    fn remove_uninhabited_generic_claim_reports(
        &self,
        functions: &mut [CheckedFunction],
    ) -> Result<(), SemanticCompilerFailure> {
        for function in functions {
            if self.uninhabited_concrete_generic(function)? {
                function.entailment.claims.clear();
            }
        }
        Ok(())
    }

    fn link_generic_claim_concrete_reports(
        &mut self,
        functions: &[CheckedFunction],
    ) -> Result<(), CheckStop> {
        for schema in &mut self.generic_claim_schemas {
            schema.concrete_reports.clear();
            for function in functions
                .iter()
                .filter(|function| function.declaration == schema.declaration)
            {
                for claim in &function.entailment.claims {
                    if !schema.claims.iter().any(|schema_claim| {
                        schema_claim.node_path == claim.node_path && schema_claim.name == claim.name
                    }) {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                    schema.concrete_reports.push(
                        super::entailment::CheckedGenericClaimConcreteReport {
                            function: function.id,
                            claim: claim.node_path.clone(),
                            name: claim.name.clone(),
                        },
                    );
                }
            }
            schema.concrete_reports.sort_by(|left, right| {
                left.function
                    .0
                    .cmp(&right.function.0)
                    .then_with(|| left.claim.components().cmp(right.claim.components()))
                    .then_with(|| left.name.cmp(&right.name))
            });
            if schema.concrete_reports.windows(2).any(|pair| {
                pair[0].function == pair[1].function
                    && pair[0].claim == pair[1].claim
                    && pair[0].name == pair[1].name
            }) {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }
        Ok(())
    }

    fn claim_residuality_outcome(
        &self,
        functions: &mut [CheckedFunction],
        phase_a: &[CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        provenance: &ResidualProvenanceContext<'_>,
    ) -> Result<Option<(FunctionId, super::entailment::ClaimOutcome)>, CheckStop> {
        let mut candidates = Vec::new();
        for (function_index, function) in functions.iter().enumerate() {
            if self.uninhabited_concrete_generic(function)? {
                continue;
            }
            let rank = self.concrete_claim_instance_rank(function)?;
            for (claim_index, claim) in function.entailment.claims.iter().enumerate() {
                candidates.push((claim.node_path.clone(), rank, function_index, claim_index));
            }
        }
        candidates.sort_by(|left, right| {
            left.0
                .components()
                .cmp(right.0.components())
                .then(left.1.cmp(&right.1))
        });

        let mut reuse = CounterfactualReuse::default();
        for (_, _, function_index, claim_index) in candidates {
            let node_path = functions[function_index].entailment.claims[claim_index]
                .node_path
                .clone();
            let component_count = functions[function_index].entailment.claims[claim_index]
                .components
                .len();
            let mut witnesses = Vec::with_capacity(component_count + 1);
            for component_index in 0..component_count {
                let component = u32::try_from(component_index)
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                let mask = ClaimMask {
                    function: functions[function_index].id,
                    node_path: node_path.clone(),
                    component: Some(component),
                };
                let mut scratch = phase_a.to_vec();
                self.analyze_function_inventory_masked(
                    &mut scratch,
                    callees,
                    optimistic_batch,
                    &mask,
                    &mut reuse,
                )?;
                let witness = self.counterfactual_witness(
                    &functions.iter().collect::<Vec<_>>(),
                    &scratch,
                    &mask,
                    provenance.failures,
                    provenance.frozen,
                    Some(provenance.main),
                )?;
                reuse.reclaim(&mut scratch);
                if let Some(witness) = witness {
                    witnesses.push(witness);
                } else {
                    functions[function_index].entailment.claims[claim_index].disposition =
                        super::entailment::ClaimDisposition::NonResidual {
                            component: Some(component),
                        };
                    return Ok(Some((
                        functions[function_index].id,
                        functions[function_index].entailment.claims[claim_index].clone(),
                    )));
                }
            }

            if component_count == 1 {
                if witnesses.len() != 1 || witnesses[0].component != Some(0) {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let mut whole = witnesses[0].clone();
                whole.component = None;
                witnesses.push(whole);
                functions[function_index].entailment.claims[claim_index].residual_witnesses =
                    witnesses;
                continue;
            }

            let mask = ClaimMask {
                function: functions[function_index].id,
                node_path,
                component: None,
            };
            let mut scratch = phase_a.to_vec();
            self.analyze_function_inventory_masked(
                &mut scratch,
                callees,
                optimistic_batch,
                &mask,
                &mut reuse,
            )?;
            let witness = self.counterfactual_witness(
                &functions.iter().collect::<Vec<_>>(),
                &scratch,
                &mask,
                provenance.failures,
                provenance.frozen,
                Some(provenance.main),
            )?;
            reuse.reclaim(&mut scratch);
            if let Some(witness) = witness {
                witnesses.push(witness);
            } else {
                functions[function_index].entailment.claims[claim_index].disposition =
                    super::entailment::ClaimDisposition::NonResidual { component: None };
                return Ok(Some((
                    functions[function_index].id,
                    functions[function_index].entailment.claims[claim_index].clone(),
                )));
            }
            functions[function_index].entailment.claims[claim_index].residual_witnesses = witnesses;
        }
        Ok(None)
    }

    fn counterfactual_witness(
        &self,
        full: &[&CheckedFunction],
        masked: &[CheckedFunctionInventory],
        mask: &ClaimMask,
        full_provenance_failures: &ProvenanceFailures,
        frozen_provenance: &FrozenProvenanceDependencies,
        external_entry: Option<FunctionId>,
    ) -> Result<Option<ClaimCounterfactualWitness>, CheckStop> {
        let terminal_witness = Self::masked_terminal_witness(full, masked, mask)?;
        let masked_functions = masked
            .iter()
            .map(|checked| &checked.function)
            .collect::<Vec<_>>();
        let masked_provenance = analyze_program_provenance_with_frozen(
            &masked_functions,
            &ProvenanceContext {
                nominals: &self.nominals,
                external_entry,
            },
            frozen_provenance,
        )?;
        // S3 is absent from U by definition, B only removes S4 from U, and
        // claims do not alter PRV-1 data flow. If no admission root changed,
        // masking one S3 contribution therefore cannot create a new PRV event.
        // Treat any such delta as broken view isolation, never as evidence that
        // a claim is residual.
        if masked_provenance.failures != *full_provenance_failures {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(terminal_witness)
    }

    fn masked_terminal_witness(
        full: &[&CheckedFunction],
        masked: &[CheckedFunctionInventory],
        mask: &ClaimMask,
    ) -> Result<Option<ClaimCounterfactualWitness>, SemanticCompilerFailure> {
        if full.len() != masked.len() {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let mut witness = None;
        for (full_function, masked_function) in full.iter().zip(masked) {
            if full_function.id != masked_function.function.id {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
            let masked_function = &masked_function.function;
            for full_root in &full_function.entailment.obligations {
                let Some(derivation) = full_root.derivation else {
                    continue;
                };
                if !full_root.discharged
                    || full_root.contradictory
                    || !full_function
                        .entailment
                        .derivations
                        .is_non_explosive(derivation)
                    || !full_function
                        .entailment
                        .derivations
                        .reaches_claim_component(derivation, &mask.node_path, mask.component)
                {
                    continue;
                }
                let masked_root = masked_function
                    .entailment
                    .obligations
                    .iter()
                    .find(|candidate| {
                        candidate.node_path == full_root.node_path
                            && candidate.family == full_root.family
                            && candidate.conjunct == full_root.conjunct
                    });
                if let Some(masked_root) = masked_root {
                    if masked_root.contradictory {
                        return Err(SemanticCompilerFailure::InvalidResolution);
                    }
                    if masked_root.discharged {
                        let masked_derivation = masked_root
                            .derivation
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        if !masked_function
                            .entailment
                            .derivations
                            .is_non_explosive(masked_derivation)
                        {
                            return Err(SemanticCompilerFailure::InvalidResolution);
                        }
                        continue;
                    }
                }
                witness.get_or_insert(ClaimCounterfactualWitness {
                    component: mask.component,
                    terminal: ClaimTerminalRoot::Obligation {
                        owner: ClaimTerminalOwner::Concrete(full_function.id),
                        function_symbol: full_function.symbol.clone(),
                        node_path: full_root.node_path.clone(),
                        family: full_root.family,
                        conjunct: full_root.conjunct,
                    },
                    masked: masked_root.map_or(ClaimMaskedDisposition::Missing, |root| {
                        ClaimMaskedDisposition::Obligation {
                            refuted: root.refuted,
                        }
                    }),
                });
            }

            for full_root in &full_function.entailment.call_goals {
                let Some(derivation) = full_root.derivation else {
                    continue;
                };
                if full_root.disposition != CallGoalDisposition::Discharged
                    || full_root
                        .evidence
                        .contains(&super::entailment::CallGoalEvidence::AllDerivable)
                    || !full_function
                        .entailment
                        .derivations
                        .is_non_explosive(derivation)
                    || !full_function
                        .entailment
                        .derivations
                        .reaches_claim_component(derivation, &mask.node_path, mask.component)
                {
                    continue;
                }
                let masked_root = masked_function
                    .entailment
                    .call_goals
                    .iter()
                    .find(|candidate| {
                        candidate.node_path == full_root.node_path
                            && candidate.callee == full_root.callee
                            && candidate.requires_clause == full_root.requires_clause
                    });
                if let Some(masked_root) = masked_root
                    && masked_root.disposition == CallGoalDisposition::Discharged
                {
                    if masked_root
                        .evidence
                        .contains(&super::entailment::CallGoalEvidence::AllDerivable)
                    {
                        return Err(SemanticCompilerFailure::InvalidResolution);
                    }
                    let masked_derivation = masked_root
                        .derivation
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if !masked_function
                        .entailment
                        .derivations
                        .is_non_explosive(masked_derivation)
                    {
                        return Err(SemanticCompilerFailure::InvalidResolution);
                    }
                    continue;
                }
                let callee_symbol = full
                    .get(full_root.callee.0 as usize)
                    .filter(|callee| callee.id == full_root.callee)
                    .map(|callee| callee.symbol.clone())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                witness.get_or_insert(ClaimCounterfactualWitness {
                    component: mask.component,
                    terminal: ClaimTerminalRoot::Call {
                        owner: ClaimTerminalOwner::Concrete(full_function.id),
                        function_symbol: full_function.symbol.clone(),
                        node_path: full_root.node_path.clone(),
                        callee: ClaimTerminalOwner::Concrete(full_root.callee),
                        callee_symbol,
                        requires_clause: full_root.requires_clause.clone(),
                    },
                    masked: masked_root.map_or(ClaimMaskedDisposition::Missing, |root| {
                        ClaimMaskedDisposition::Call(root.disposition)
                    }),
                });
            }

            for full_proof in &full_function.entailment.postconditions {
                let Some(derivation) = full_proof.complete.derivation else {
                    continue;
                };
                if !full_proof.complete.discharged
                    || !full_function
                        .entailment
                        .derivations
                        .is_non_explosive(derivation)
                    || !full_function
                        .entailment
                        .derivations
                        .reaches_claim_component(derivation, &mask.node_path, mask.component)
                {
                    continue;
                }
                let masked_proof =
                    masked_function
                        .entailment
                        .postconditions
                        .iter()
                        .find(|candidate| {
                            candidate.block == full_proof.block
                                && candidate.relation_ordinal == full_proof.relation_ordinal
                        });
                if let Some(masked_proof) = masked_proof
                    && masked_proof.complete.discharged
                {
                    let masked_derivation = masked_proof
                        .complete
                        .derivation
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if !masked_function
                        .entailment
                        .derivations
                        .is_non_explosive(masked_derivation)
                    {
                        return Err(SemanticCompilerFailure::InvalidResolution);
                    }
                    continue;
                }
                witness.get_or_insert(ClaimCounterfactualWitness {
                    component: mask.component,
                    terminal: ClaimTerminalRoot::Postcondition {
                        owner: ClaimTerminalOwner::Concrete(full_function.id),
                        function_symbol: full_function.symbol.clone(),
                        block: full_proof.block.clone(),
                        relation_ordinal: full_proof.relation_ordinal,
                    },
                    masked: masked_proof.map_or(ClaimMaskedDisposition::Missing, |_| {
                        ClaimMaskedDisposition::PostconditionFailed
                    }),
                });
            }
        }
        Ok(witness)
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
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. }
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
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
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
            requirements: requirement
                .map(|requirement| requirement.clauses.clone())
                .unwrap_or_default(),
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
                witness.extend(requirement.clauses.iter().cloned());
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
            requirements: target
                .requirement
                .as_ref()
                .map(|requirement| requirement.clauses.clone())
                .unwrap_or_default(),
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

    fn source_issue_path(
        issue: &SemanticIssue,
    ) -> Result<&crate::NodePath, SemanticCompilerFailure> {
        match &issue.location {
            SemanticLocation::SourceNode(path, _) => Ok(path),
            SemanticLocation::BundleRoot(_) => Err(SemanticCompilerFailure::InvalidResolution),
        }
    }

    fn claim_formation_issue(
        &self,
        failure: &ClaimFormationFailure,
        instance: Option<String>,
    ) -> Result<SemanticIssue, CheckStop> {
        let node = self
            .tree
            .node_with_path(&failure.node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(SemanticIssue {
            rule: SemanticRule::Clm1,
            location: SemanticLocation::SourceNode(
                failure.node_path.clone(),
                self.tree.coordinate(node)?,
            ),
            kind: SemanticIssueKind::InvalidClaim(Box::new(crate::InvalidClaimDetail {
                name: failure.name.clone(),
                predicate: failure.predicate.clone(),
                classification: "unsupported canonical formation",
                component: None,
                reason: "the predicate has no unique supported contribution normal form",
                instance,
            })),
        })
    }

    fn claim_locality_issue(
        &self,
        failure: &ClaimLocalityFailure,
    ) -> Result<SemanticIssue, CheckStop> {
        let node = self
            .tree
            .node_with_path(&failure.node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (boundary, mechanical_fix) = match failure.boundary.kind {
            BoundaryResultKind::UserCall(function) => {
                let signature = self
                    .signatures
                    .get(function.0 as usize)
                    .filter(|signature| signature.id == function)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (
                    crate::ClaimBoundaryResultDetail::UserCall {
                        declaration: signature.declaration,
                        callee: signature.name.clone(),
                    },
                    "publish the required cross-function relation as an exact verified ensures clause on the callee and remove this caller claim",
                )
            }
            BoundaryResultKind::SystemCall(declaration_ordinal) => {
                let operation = crate::SYSTEM_OPERATIONS
                    .get(usize::from(declaration_ordinal))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (
                    crate::ClaimBoundaryResultDetail::SystemCall {
                        declaration_ordinal,
                        operation: operation.spelling.to_owned(),
                    },
                    "use the system operation's specified fact or typed outcome, or branch on the returned value; do not claim an unstated system-result property",
                )
            }
        };
        Ok(SemanticIssue {
            rule: SemanticRule::Clm1,
            location: SemanticLocation::SourceNode(
                failure.node_path.clone(),
                self.tree.coordinate(node)?,
            ),
            kind: SemanticIssueKind::NonLocalClaim(Box::new(crate::NonLocalClaimDetail {
                name: failure.name.clone(),
                component: failure.component,
                carrier: failure.carrier.clone(),
                boundary_call: failure.boundary.call.clone(),
                boundary,
                mechanical_fix,
            })),
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
        owner_filter: Option<&[FunctionId]>,
    ) -> Result<Option<SemanticIssue>, CheckStop> {
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

        let local = failures
            .local_rejections
            .iter()
            .filter(|(leaf, _, _, _)| {
                owner_filter.is_none_or(|owners| owners.contains(&leaf.function))
            })
            .map(|(leaf, dependency, requirement, carrier)| {
                Rejection::Local(leaf, dependency, requirement.as_ref(), carrier)
            });
        let calls = failures
            .call_events
            .iter()
            .filter(|event| owner_filter.is_none_or(|owners| owners.contains(&event.caller)))
            .map(Rejection::Call);
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
            return Ok(None);
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
                    witness.extend(requirement.clauses.iter().cloned());
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
                Ok(Some(SemanticIssue {
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
                                requirements: requirement
                                    .map(|requirement| requirement.clauses.clone())
                                    .unwrap_or_default(),
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
                Ok(Some(SemanticIssue {
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

    fn claim_outcome_rejection(
        &self,
        outcome: &super::entailment::ClaimOutcome,
        instance: Option<String>,
    ) -> Result<(), CheckStop> {
        let node = self
            .tree
            .node_with_path(&outcome.node_path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let location =
            SemanticLocation::SourceNode(outcome.node_path.clone(), self.tree.coordinate(node)?);
        if let super::entailment::ClaimDisposition::Refuted {
            predicate,
            negation,
        } = &outcome.disposition
        {
            return Err(CheckStop::source_issue(SemanticIssue {
                rule: SemanticRule::Clm2,
                location,
                kind: SemanticIssueKind::RefutedClaim(Box::new(crate::RefutedClaimDetail {
                    name: outcome.name.clone(),
                    predicate: predicate.clone(),
                    negation: negation.clone(),
                    instance,
                })),
            }));
        }
        if outcome.disposition == super::entailment::ClaimDisposition::BothSigns {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let (classification, component, reason) = match outcome.disposition {
            super::entailment::ClaimDisposition::Redundant => (
                "redundant",
                None,
                "the checker already derives the exact predicate",
            ),
            super::entailment::ClaimDisposition::Vacuous { cause } => match cause {
                super::entailment::ClaimVacuity::PreStateContradiction => {
                    ("vacuous", None, "the pre-claim state is contradictory")
                }
                super::entailment::ClaimVacuity::ExactImageConflict => (
                    "vacuous",
                    None,
                    "equivalent exact predicate images have opposite signs",
                ),
                super::entailment::ClaimVacuity::ComponentManifestationConflict { component } => (
                    "vacuous",
                    Some(component),
                    "equivalent manifestations of this contribution component have opposite signs",
                ),
            },
            super::entailment::ClaimDisposition::ComponentRedundant { component } => (
                "component overlap",
                Some(component),
                "the checker already derives this contribution component",
            ),
            super::entailment::ClaimDisposition::ComponentRefuted { component } => (
                "component refuted",
                Some(component),
                "the checker derives this contribution component's negation",
            ),
            super::entailment::ClaimDisposition::InconsistentContribution => (
                "inconsistent contribution",
                None,
                "the contribution is inconsistent or cannot reconstruct the predicate",
            ),
            super::entailment::ClaimDisposition::NonResidual { component } => (
                "non-residual",
                component,
                "withholding this claim authority changes no eligible admission root",
            ),
            super::entailment::ClaimDisposition::Retained
            | super::entailment::ClaimDisposition::Refuted { .. }
            | super::entailment::ClaimDisposition::BothSigns => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        };
        Err(CheckStop::source_issue(SemanticIssue {
            rule: SemanticRule::Clm2,
            location,
            kind: SemanticIssueKind::InvalidClaim(Box::new(crate::InvalidClaimDetail {
                name: outcome.name.clone(),
                predicate: outcome.predicate.clone(),
                classification,
                component,
                reason,
                instance,
            })),
        }))
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
        }

        impl Rejection<'_> {
            fn node_path(&self) -> &crate::NodePath {
                match self {
                    Self::Obligation(outcome) => &outcome.node_path,
                    Self::Call(outcome) => &outcome.node_path,
                }
            }

            const fn rule(&self) -> SemanticRule {
                match self {
                    Self::Obligation(outcome) => match outcome.family {
                        super::entailment::ObligationFamily::Bounds => SemanticRule::Op4,
                        super::entailment::ObligationFamily::IntegerDomain => SemanticRule::Op2,
                        super::entailment::ObligationFamily::AllocationFit => SemanticRule::Op9,
                        super::entailment::ObligationFamily::SystemRange => SemanticRule::Sys8,
                    },
                    Self::Call(_) => SemanticRule::Fn8,
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
        let rejection = obligation.chain(call).min_by(|left, right| {
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
                                mechanical_fix: "establish the residual with a dominating branch, or, only when it is an independently true theorem outside checker rules, add a CLM-2-admissible residual `claim` with a complete exact `because` record",
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
                                mechanical_fix: "establish the fixed `.defined` normalization with a dominating branch, use an available total non-exact row, or, only when the predicate is an independently true theorem outside checker rules, add a CLM-2-admissible residual `claim` with a complete exact `because` record",
                            },
                        },
                        super::entailment::ObligationFamily::AllocationFit => SemanticIssue {
                            rule: SemanticRule::Op9,
                            location,
                            kind: SemanticIssueKind::UndischargedAllocationFitObligation {
                                residual,
                                mechanical_fix: "establish `buffer_fits<T>(n)` with a branch or requirement, or, only when it is an independently true theorem outside checker rules, add a CLM-2-admissible residual `claim` with a complete exact `because` record",
                            },
                        },
                        super::entailment::ObligationFamily::SystemRange => SemanticIssue {
                            rule: SemanticRule::Sys8,
                            location,
                            kind: SemanticIssueKind::UndischargedSystemRangeObligation {
                                residual,
                                mechanical_fix: "establish the residual with a branch or requirement, or, only when it is an independently true theorem outside checker rules, add a CLM-2-admissible residual `claim` with a complete exact `because` record",
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
                        "establish the complete callee requirement with one dominating branch before the call, or, only when it is an independently true theorem outside checker rules, add a CLM-2-admissible residual claim with a complete exact `because` record"
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
                                instantiated_goal: render_goal(&outcome.goal.root),
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
                exit.complete.disposition != super::entailment::PostconditionDisposition::Discharged
            }) else {
                continue;
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

#[cfg(test)]
mod counterfactual_reuse_tests {
    use super::super::entailment::FlowEventKind;
    use super::{CounterfactualReuse, FunctionPostconditionProof};
    use crate::NodePath;
    use crate::semantic::entailment::{
        FunctionEntailment, PostconditionAggregate, ProofView, VerifiedPostconditionSummary,
    };
    use crate::semantic::model::FunctionId;

    fn aggregate(view: ProofView) -> PostconditionAggregate {
        PostconditionAggregate {
            view,
            discharged: true,
            derivation: None,
        }
    }

    fn proof(relation_ordinal: u32) -> FunctionPostconditionProof {
        let block = NodePath {
            components: vec![0],
        };
        FunctionPostconditionProof {
            block: block.clone(),
            selector: block.clone(),
            relation_ordinal,
            summary: Some(VerifiedPostconditionSummary {
                function: FunctionId(0),
                block,
                relation_ordinal,
                component: 0,
            }),
            exits: Vec::new(),
            complete: aggregate(ProofView::Complete),
            unasserted: aggregate(ProofView::Unasserted),
            s4_blinded: aggregate(ProofView::S4Blinded),
        }
    }

    /// The reuse of a [CLM-2] counterfactual analysis is admitted only when
    /// the published FN-9 context the entry was computed under is unchanged,
    /// and only for the function the entry belongs to. A key that ignored
    /// either would hand a later rerun an entailment derived from
    /// postconditions that rerun no longer publishes.
    #[test]
    fn a_changed_published_context_is_not_reused() {
        let mut entailment = FunctionEntailment::default();
        let stored = proof(0);
        let other = proof(1);
        let before = vec![vec![&stored]];
        let mut reuse = CounterfactualReuse::default();
        reuse.lend(1, &[], &before);
        reuse.reclaim_one(1, &mut entailment);

        assert!(reuse.take(1, &[], &[vec![&other]]).is_none());
        assert!(reuse.take(1, &[], &[Vec::new()]).is_none());
        assert!(reuse.take(1, &[], &[]).is_none());
        assert!(reuse.take(1, &[vec![]], &before).is_none());
        assert!(reuse.take(0, &[], &before).is_none());
        assert!(reuse.take(2, &[], &before).is_none());
        assert!(reuse.take(1, &[], &before).is_some());
    }

    /// The [CLM-2] reuse entry lends its analysis to one rerun's inventory
    /// instead of copying it, so exactly one copy of a function's derivation
    /// arena exists at any moment. A second take before the reclaim would mean
    /// two live copies, which is what the entry exists to avoid.
    #[test]
    fn a_lent_counterfactual_entry_is_not_a_second_copy() {
        let stored = proof(0);
        let context = vec![vec![&stored]];
        let mut reuse = CounterfactualReuse::default();
        let mut entailment = FunctionEntailment::default();
        entailment.derivations.event(FlowEventKind::Snapshot, None);

        reuse.lend(0, &[], &context);
        assert!(
            reuse.take(0, &[], &context).is_none(),
            "a recorded context holds no value until the rerun gives one back"
        );
        reuse.reclaim_one(0, &mut entailment);
        assert!(
            entailment.derivations.events.is_empty(),
            "the arena moved out of the inventory"
        );

        let lent = reuse.take(0, &[], &context).expect("the value comes back");
        assert_eq!(lent.derivations.events.len(), 1);
        assert!(
            reuse.take(0, &[], &context).is_none(),
            "the entry holds nothing while the inventory holds the value"
        );
    }

    /// The entry holds the analysis, not the inventory slot it was lent to.
    /// The slot picks up the published FN-9 summaries of every component the
    /// SCC scheduler publishes, and each [CLM-2] rerun takes that publication
    /// decision again under its own mask. An entry that carried the stamp back
    /// would hand the next rerun a postcondition proof marked verified by a
    /// component that rerun never published, which every function scheduled
    /// after it then reads as one more visible postcondition.
    #[test]
    fn a_published_summary_is_not_lent_to_the_next_rerun() {
        let mut reuse = CounterfactualReuse::default();
        // `proof` builds the proofs as the publish loop leaves them.
        let mut entailment = FunctionEntailment {
            postconditions: vec![proof(0), proof(1)],
            ..FunctionEntailment::default()
        };

        reuse.lend(0, &[], &[]);
        reuse.reclaim_one(0, &mut entailment);

        let lent = reuse.take(0, &[], &[]).expect("the value comes back");
        assert_eq!(lent.postconditions.len(), 2);
        assert!(
            lent.postconditions
                .iter()
                .all(|proof| proof.summary.is_none()),
            "a summary published by one rerun must not reach the next"
        );
    }
}
