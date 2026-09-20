mod behavior;
mod cleanup;
mod control;
mod ensures;
pub(in crate::semantic::check) mod expressions;
mod floats;
mod generics;
mod linearity;
mod nominal_instances;
mod nominals;
pub(crate) mod publication;
mod references;
mod requires;
mod support;
mod type_regions;
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
    collect_statement_calls, finalize_function_entailment, postcondition_schedule,
};
use super::goal::{
    CheckedCallRequirement, CheckedRequirement, ConcreteGoal, GoalDatum, GoalExpression,
    GoalOperation, GoalProjection, first_ephemeral_argument,
};
use super::model::{
    BindingId, CheckedConst, CheckedConstant, CheckedConstantId, CheckedElement, CheckedExpression,
    CheckedFlatElement, CheckedFunction, CheckedGenericRequirement, CheckedMode, CheckedNominal,
    CheckedNominalKind, CheckedParameter, CheckedProgramData, CheckedSetTarget, CheckedStatement,
    CheckedType, CheckedValue, DerivedConst, DerivedConstId, FunctionId, NominalId,
    ValueInitializerKind, evaluate_const_operation,
};
use super::permission::{PermissionSignature, analyze_permission, plan_permission_separations};
use super::permission_ledger::{LedgerSource, render_ledger};
use super::places::ResolvedPlace;
use super::postcondition::CheckedPostconditionSelector;
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use control::{ControlCounters, ControlScope};
use generics::{GenericParameter, GenericSubstitution, PendingGenericRequirement};
use references::ReferenceInfo;

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
    /// Every formal region of the callable.
    region_parameters: Vec<DeclarationId>,
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
    effects_node: NodeId,
    declared_effects: EffectSet,
    /// A callable hypothesis used only while checking generic source spelling.
    /// Concrete calls always select a verified source function instead.
    formal_parameter: Option<generics::GenericParameterKey>,
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
    /// [PROV-6] whether the declaration writes the `nodrop` modifier. Every
    /// instance of a marked declaration is marked.
    linear: bool,
    /// [OWN-1, GRAM-2] whether the declaration writes the `nocopy` modifier.
    /// Every instance of a marked declaration is marked.
    nocopy: bool,
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
    /// The compiler-owned result-list nominal of a [BLK-0] row that declares
    /// an ordered result list [CALL-4]. A row's list is fixed by its own
    /// instance and has no written form for the interning pass to find.
    ResultList(Vec<(String, CheckedType)>),
    /// [STOR-2] an `arena<'r, T>` instance over this region and content.
    Arena(DeclarationId, CheckedType),
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
    /// Whether the binding still owns a usable value, or names a path
    /// [OWN-1, REF-1].
    live: bool,
    loop_depth: usize,
    /// Compiler-updated counted binders are readable source bindings but are
    /// never writer-controlled storage [SET-1, OWN-11].
    compiler_updated: bool,
    /// [REF-1] what this binding names when it is a reference variable: its
    /// path set, its kind, and its [REF-2] validity. `None` is storage of its
    /// own.
    ///
    /// This replaces v0.59's `borrow`, `slice`, `slice_loans` and `suspended`
    /// fields. None of the four has a subject: a reference carries no region,
    /// no strength, no parent holder and no loan, and exclusivity suspension
    /// was the loan apparatus's way of stating what [REF-2] now states
    /// directly as a validity fact.
    reference: Option<ReferenceInfo>,
    /// [REF-2, ENT-3.S15] the borrowed-match refinement occurrences this
    /// binding witnesses. The occurrence identity is immutable even when a
    /// reference binding is rebound; only its flow-sensitive validity meets
    /// at joins.
    refinement_witnesses: Vec<RefinementWitness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RefinementWitness {
    origin: NodePath,
    place: ResolvedPlace,
    variant: u32,
    valid: bool,
}

impl RefinementWitness {
    fn same_occurrence(&self, other: &Self) -> bool {
        self.origin == other.origin && self.place == other.place && self.variant == other.variant
    }
}

impl LocalBinding {
    /// [REF-1] the join of one binding's state over two incoming edges: the
    /// union of the reference path sets, and the meet of the validity facts.
    ///
    /// Liveness is *not* joined here. [LIV-1] makes a disagreement at a join
    /// a hard error naming the binding and the two disagreeing predecessors,
    /// so the caller compares liveness and rejects; this method runs after
    /// that comparison has agreed.
    fn join_from(&mut self, other: &Self) {
        if let (Some(left), Some(right)) = (&mut self.reference, &other.reference) {
            left.join(right);
        }
        for witness in &mut self.refinement_witnesses {
            witness.valid &= other
                .refinement_witnesses
                .iter()
                .find(|candidate| witness.same_occurrence(candidate))
                .is_some_and(|candidate| candidate.valid);
        }
        for witness in &other.refinement_witnesses {
            if !self
                .refinement_witnesses
                .iter()
                .any(|candidate| candidate.same_occurrence(witness))
            {
                let mut absent_on_left = witness.clone();
                absent_on_left.valid = false;
                self.refinement_witnesses.push(absent_on_left);
            }
        }
    }

    /// Whether two joined states agree on everything [LIV-1] compares.
    ///
    /// A reference's path set is deliberately excluded: [REF-1] states that
    /// at a join a reference variable's target is the *union* of the incoming
    /// sets, so two different sets are the joined state rather than a
    /// disagreement. Validity is compared, because a reference valid on one
    /// edge and invalid on the other is invalid after the join and every
    /// check on it must hold for every member of the set.
    fn agrees_with(&self, other: &Self) -> bool {
        self.binding == other.binding
            && self.declaration == other.declaration
            && self.mode == other.mode
            && self.ty == other.ty
            && self.live == other.live
            && self.loop_depth == other.loop_depth
            && self.compiler_updated == other.compiler_updated
            && match (&self.reference, &other.reference) {
                (Some(left), Some(right)) => left.kind == right.kind,
                (None, None) => true,
                _ => false,
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
    /// [REF-1] the path set this expression names when it produces a
    /// reference: a `borrow_expr`, a read of a reference variable, or a value
    /// initializer every arm of which delivers one.
    ///
    /// This replaces v0.59's `borrow`, `slice` and `holder` fields. A
    /// reference has no region, no strength and no parent holder, so the
    /// three collapse into the one thing [REF-1] gives it.
    reference: Option<ReferenceInfo>,
    /// Whether this expression denotes the reference itself rather than a
    /// place reached through it.
    ///
    /// A destination of reference kind wants exactly this value; a construct
    /// that needs an owned value — a `match` scrutinee of own mode under
    /// [OWN-13], `propagate` under [ERR-3] — reads the path instead. A
    /// reference is read bare [TYPE-7], so this flag distinguishes the two
    /// readings and never selects a read-through operation.
    reference_value: bool,
    effects: EffectSet,
    accesses: Vec<PlaceAccess>,
}

#[derive(Clone)]
struct PlaceAccess {
    place: ResolvedPlace,
    /// This place is the storage selected by the enclosing place expression,
    /// rather than storage read while evaluating one of its offsets or other
    /// operands. Effects retain both; borrowed-match path recovery needs only
    /// the selected members [REF-1, OWN-13].
    selected: bool,
}

impl PlaceAccess {
    fn operand(mut self) -> Self {
        self.selected = false;
        self
    }
}

impl TypedExpression {
    fn owned(expression: CheckedExpression, effects: EffectSet) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects,
            accesses: Vec::new(),
        }
    }

    fn owned_with_access(
        expression: CheckedExpression,
        effects: EffectSet,
        place: ResolvedPlace,
    ) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects,
            accesses: vec![PlaceAccess {
                place,
                selected: true,
            }],
        }
    }
}

/// [EFF-2]'s only repair: the declaration must equal the exhibited row.
const EFF2_ROW_FIX: &str = "declare exactly the row the body exhibits: add every missing category and path and remove every extra one; EFF-2 admits no wider and no narrower declaration than the union of the body-syntactic and release contributions";

/// One ordinary resolved-place contribution to the enclosing effect row.
#[derive(Clone, Debug)]
struct EffectPath {
    path: super::model::CheckedStatePath,
}

impl From<super::model::CheckedStatePath> for EffectPath {
    fn from(path: super::model::CheckedStatePath) -> Self {
        Self { path }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EffectSet {
    reads: Vec<super::model::CheckedStatePath>,
    writes: Vec<super::model::CheckedStatePath>,
    /// [EFF-3] whether the boundary this set describes allocates. It is not
    /// a row category [EFF-1, STOR-8]; it is the checked-program metadata
    /// [EFF-3]'s deduplication and reordering licence reads.
    allocates: bool,
}

impl EffectSet {
    const NONE: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates: false,
    };
    fn union(mut self, other: Self) -> Self {
        for path in other.reads {
            self.add_read(path);
        }
        for path in other.writes {
            self.add_write(path);
        }
        self.allocates |= other.allocates;
        self
    }

    fn add_read(&mut self, path: impl Into<EffectPath>) {
        Self::add_path(&mut self.reads, path.into());
    }

    fn add_write(&mut self, path: impl Into<EffectPath>) {
        Self::add_path(&mut self.writes, path.into());
    }

    fn add_path(paths: &mut Vec<super::model::CheckedStatePath>, contribution: EffectPath) {
        if !paths.contains(&contribution.path) {
            paths.push(contribution.path);
            paths.sort_unstable();
        }
    }

    /// [EFF-3] records that this boundary allocates.
    fn add_allocation(&mut self) {
        self.allocates = true;
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

struct Checker<'unit, 'classified, 'lexed, 'source> {
    resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    /// [STOR-8, GRAM-2] whether this compilation unit wrote `program
    /// no_heap;`.
    ///
    /// The declaration's placement rule — at most once, and only as the first
    /// `item` of the first source record — is resolution's [GRAM-2]
    /// admission, so a `heap_decl` reaching the checker at all is the
    /// admitted one and the fact is a unit-level flag. What the checker owns
    /// is [STOR-8]'s consequence: such a unit cannot name `Box` or a
    /// runtime-capacity shape and cannot call an allocating prelude row.
    no_heap: bool,
    /// Whether an undischarged obligation rejects. Always true outside the
    /// test-only observability hooks.
    reject_entailment: bool,
    tree: TreeView<'unit, 'classified, 'lexed, 'source>,
    nominals: Vec<CheckedNominal>,
    elements: RefCell<Vec<CheckedType>>,
    element_ids: RefCell<HashMap<CheckedType, CheckedElement>>,
    nominal_nodes: Vec<Option<NodeId>>,
    nominal_states: Vec<u8>,
    source_nominal_instances: Vec<Option<(usize, GenericSubstitution)>>,
    box_nominals: HashMap<CheckedType, NominalId>,
    /// S39 one `Box<'s, T>` nominal per (store region, referent).
    /// `arena<'r, T>` instances by (region declaration, content type): the
    /// region is part of the type's identity [OWN-3, STOR-4].
    arena_nominals: HashMap<(DeclarationId, CheckedType), NominalId>,
    /// The compiler-owned result-list nominal of a `fn_decl` that declares an
    /// ordered result list [GRAM-2, CALL-4], keyed by the ordered result
    /// binder spellings and types. Two declarations whose result lists agree
    /// share one nominal; the value a multi-result callable hands back is one
    /// value of it, and a destructuring binder list is its projection.
    result_list_nominals: HashMap<Vec<(String, CheckedType)>, NominalId>,
    /// Nominal instances a derived type named that were not interned yet.
    /// Written by the `&self` checking path and drained by the `&mut self`
    /// driver between attempts at one function.
    pending_nominals: RefCell<Vec<PendingNominal>>,
    /// [OP-10, OP-11, OP-14] instances of an operand-directed [PRE-1] row
    /// whose substitution a body reached but whose signature is not built yet.
    ///
    /// These rows write no type arguments at a call, so the syntax alone
    /// selects no instance and the discovery walk builds none. The `&self`
    /// body check derives the substitution from the operand's type and
    /// records it here; the `&mut self` driver builds the signature and
    /// retries the function, exactly as it does for a derived nominal.
    pending_instances: RefCell<Vec<(usize, generics::GenericSubstitution)>>,
    /// [PROV-1] the region an elided store brand denotes at the position
    /// being parsed: the enclosing nominal's sole region parameter while a
    /// `struct_decl` or `enum_decl` body is being read, and `None`
    /// everywhere else, where FORM-8 requires a written store argument.
    elided_store_brand: std::cell::Cell<Option<DeclarationId>>,
    /// [FN-2, OWN-1, PROV-6] whether the body now being checked is a *concrete
    /// instance* of a generic template whose spelling one symbolic instance
    /// has already judged.
    ///
    /// The template is the spelling authority: a body whose parameter lacks
    /// copy writes `move`, a `copy`-bounded body writes bare use, and the one
    /// symbolic instance decides both once. The concrete-instance recheck
    /// therefore
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
    /// [EFF-5] the pairwise comparisons of the function being checked that
    /// syntax could not settle, handed to the entailment fragment with the
    /// finished body.
    call_separations: RefCell<Vec<super::model::CheckedCallSeparation>>,
    /// Successful declaration-only FN-4 queries. A complete member check
    /// stages its batch before publishing here, and the whole checker remains
    /// failure-atomic with the prospective checked program [DIAG-2].
    contract_queries: RefCell<Vec<super::model::CheckedContractQuery>>,
    prelude_nominals: HashMap<PreludeType, NominalId>,
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
    behavior: behavior::BehaviorInventory,
}

/// Checks the currently implemented active-specification semantic family.
///
/// Unsupported language families remain explicit compiler capability results;
/// only a proved numbered-rule violation becomes [`SemanticOutcome::SourceIssue`].
#[must_use]
pub fn check_semantics<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true)
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
    check_semantics_with(resolved, false)
}

/// Legacy test helper selecting the one shipped semantic judgment. It remains
/// only while the arithmetic obligation tests are renamed around IntegerDomain.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_arithmetic_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true)
}

/// Legacy test helper selecting the one shipped semantic judgment. It remains
/// only while the division obligation tests are renamed around IntegerDomain.
#[cfg(test)]
#[must_use]
pub(crate) fn check_semantics_division_obligations<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    check_semantics_with(resolved, true)
}

fn check_semantics_with<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    reject_entailment: bool,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    let preflight = if resolved.postconditions().is_empty() {
        Ok(())
    } else {
        Checker::new(&resolved, reject_entailment).and_then(|mut checker| {
            let items = checker.item_declarations()?;
            checker.preflight_postcondition_selectors(&items)
        })
    };
    let result = preflight.and_then(|()| {
        Checker::new(&resolved, reject_entailment).and_then(|mut checker| checker.check_program())
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
        // [EFF-1] the row has two categories. Allocation carries no effect
        // entry [STOR-8], so an allocating boundary still writes `pure` where
        // it reads and writes nothing [EFF-2].
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

    /// One `effect_path` in its written spelling [EFF-1]: the parameter's own
    /// name, wrapped in `deref(...)` at each `deref` step, with every other
    /// step written as the `epsuffix` that produced it.
    ///
    /// The walk carries the selected type only as far as it can name a step:
    /// a field name needs its containing struct and a payload field needs its
    /// containing enum. A step the checked program cannot name renders as its
    /// ordinal, which is honest rather than invented.
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
        for step in &path.steps {
            match step {
                super::model::CheckedEffectStep::Field(field) => {
                    let selected = match ty {
                        Some(CheckedType::Nominal(nominal)) => match &self.nominal(nominal)?.kind {
                            CheckedNominalKind::Struct { fields } => fields
                                .get(*field as usize)
                                .map(|declared| (declared.name.clone(), declared.ty)),
                            _ => None,
                        },
                        _ => None,
                    };
                    match selected {
                        Some((name, field_type)) => {
                            rendered.push('.');
                            rendered.push_str(&name);
                            ty = Some(field_type);
                        }
                        None => {
                            rendered.push_str(&format!(".{field}"));
                            ty = None;
                        }
                    }
                }
                // `epbase := IDENT | "deref" "(" effect_path ")"` wraps the
                // path built so far instead of appending to it [EFF-1].
                super::model::CheckedEffectStep::Deref => {
                    rendered = format!("deref({rendered})");
                    ty = None;
                }
                super::model::CheckedEffectStep::Payload { variant, field } => {
                    let selected = match ty {
                        Some(CheckedType::Nominal(nominal)) => match &self.nominal(nominal)?.kind {
                            CheckedNominalKind::Enum { variants } => {
                                variants.get(*variant as usize).and_then(|declared| {
                                    declared.fields.get(*field as usize).map(|payload| {
                                        (declared.name.clone(), payload.name.clone(), payload.ty)
                                    })
                                })
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    match selected {
                        Some((variant_name, field_name, payload_type)) => {
                            rendered.push_str(&format!(".{variant_name}.{field_name}"));
                            ty = Some(payload_type);
                        }
                        None => {
                            rendered.push_str(&format!(".{variant}.{field}"));
                            ty = None;
                        }
                    }
                }
                super::model::CheckedEffectStep::Index(index) => {
                    rendered.push_str(&format!("[{}]", self.declaration_spelling(*index)?));
                    ty = None;
                }
                super::model::CheckedEffectStep::Range { start, end } => {
                    rendered.push_str(&format!(
                        "[{}..{}]",
                        self.declaration_spelling(*start)?,
                        self.declaration_spelling(*end)?
                    ));
                    ty = None;
                }
                super::model::CheckedEffectStep::Part(part) => {
                    rendered.push('.');
                    rendered.push_str(part.spelling());
                    ty = None;
                }
                super::model::CheckedEffectStep::Measure(measure) => {
                    rendered.push('.');
                    rendered.push_str(measure.spelling());
                    ty = None;
                }
            }
        }
        Ok(rendered)
    }

    /// The exhibited categories the declaration is missing, and the declared
    /// categories the body does not exhibit, each in the spelling the writer
    /// would have to add or delete.
    /// Whether `entry` is `access` or a proper prefix of it [EFF-1].
    ///
    /// [EFF-2] states the relation as "the body accesses storage at or below
    /// its path", and an effect path is a root formal plus a complete step
    /// list, so "below" is exactly the prefix order on those steps.
    fn effect_path_covers(
        entry: &super::model::CheckedStatePath,
        access: &super::model::CheckedStatePath,
    ) -> bool {
        entry.root == access.root && access.steps.starts_with(&entry.steps)
    }

    /// [EFF-2]'s two-way judgment over a complete row.
    ///
    /// "Rows are checked both ways against this complete exhibited set --
    /// every declared entry is exhibited in that sense, and every exhibited
    /// access lies under some declared entry." That is a covering relation
    /// and not equality of path sets: a row declaring a whole reference
    /// parameter covers the measure read `deref(p).len` below it [OP-15],
    /// while a row declaring only a field is not covered by an access to the
    /// whole.
    ///
    /// The two categories are not independent. [EFF-1] states that
    /// "`writes(p)` subsumes `reads(p)`, so the pair is never written for one
    /// path", so a declared write covers an exhibited read at or below its
    /// path and an exhibited write answers for a declared read. A declared
    /// write is answered only by an exhibited write: nothing subsumes a write
    /// the body never makes.
    fn effect_row_matches(declared: &EffectSet, exhibited: &EffectSet) -> bool {
        exhibited.reads.iter().all(|access| {
            declared
                .reads
                .iter()
                .chain(&declared.writes)
                .any(|entry| Self::effect_path_covers(entry, access))
        }) && exhibited.writes.iter().all(|access| {
            declared
                .writes
                .iter()
                .any(|entry| Self::effect_path_covers(entry, access))
        }) && declared.reads.iter().all(|entry| {
            exhibited
                .reads
                .iter()
                .chain(&exhibited.writes)
                .any(|access| Self::effect_path_covers(entry, access))
        }) && declared.writes.iter().all(|entry| {
            exhibited
                .writes
                .iter()
                .any(|access| Self::effect_path_covers(entry, access))
        })
    }

    fn effect_row_difference(
        &self,
        exhibited: &EffectSet,
        declared: &EffectSet,
        signature: &FunctionSignature,
    ) -> Result<(Vec<String>, Vec<String>), CheckStop> {
        let mut missing = Vec::new();
        let mut extra = Vec::new();
        // `missing` names each exhibited access lying under no declared
        // entry; `extra` names each declared entry the body never accesses at
        // or below. Both are [EFF-2]'s own two failures.
        for path in &exhibited.reads {
            if !declared
                .reads
                .iter()
                .chain(&declared.writes)
                .any(|entry| Self::effect_path_covers(entry, path))
            {
                missing.push(format!(
                    "reads({})",
                    self.render_effect_path(path, signature)?
                ));
            }
        }
        for path in &exhibited.writes {
            if !declared
                .writes
                .iter()
                .any(|entry| Self::effect_path_covers(entry, path))
            {
                missing.push(format!(
                    "writes({})",
                    self.render_effect_path(path, signature)?
                ));
            }
        }
        for entry in &declared.reads {
            if !exhibited
                .reads
                .iter()
                .chain(&exhibited.writes)
                .any(|path| Self::effect_path_covers(entry, path))
            {
                extra.push(format!(
                    "reads({})",
                    self.render_effect_path(entry, signature)?
                ));
            }
        }
        for entry in &declared.writes {
            if !exhibited
                .writes
                .iter()
                .any(|path| Self::effect_path_covers(entry, path))
            {
                extra.push(format!(
                    "writes({})",
                    self.render_effect_path(entry, signature)?
                ));
            }
        }
        Ok((missing, extra))
    }

    /// [OWN-1, FORM-1, FN-2] whether this body is the authority on the one
    /// spelling [FORM-1] keys on a value's copy/affine class.
    ///
    /// That spelling is `move p` versus a bare `p` [OWN-1]: one spelling per
    /// meaning, selected by the class. A concrete instance of a generic
    /// template is not its authority: the template's one symbolic instance
    /// judged it under the parameter's written bound, so at a copy instance a
    /// `move` of a template-affine value denotes a copy rather than reopening
    /// a judgment the template already made [PROV-6]. Every other judgment of
    /// those rules — consume-once and dead roots — is re-judged here, because
    /// each is a property of the concrete instance and not of the spelling.
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

    /// [OP-10] whether this type parameter is a compiler-owned window type
    /// parameter: the `W` or `X` of a [PRE-1] record, supplied by an operand
    /// and never written.
    ///
    /// The spelling alone cannot decide it, because a source declaration may
    /// name a parameter `W`; the rule's own sentence is that no source
    /// declaration can write such a parameter, so the prelude origin of the
    /// declaration that introduces it is the other half of the test.
    pub(in crate::semantic::check) fn is_window_type_parameter(
        &self,
        declaration: DeclarationId,
    ) -> Result<bool, CheckStop> {
        let Some(record) = self.resolved.declaration(declaration) else {
            return Ok(false);
        };
        if !matches!(record.spelling(), "W" | "X") {
            return Ok(false);
        }
        Ok(self
            .resolved
            .syntax()
            .classified_bundle()
            .source_bundle()
            .file(record.origin().coordinate().source())
            .is_some_and(|file| file.prelude().is_some()))
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
        let mut written = self
            .written_body_effect_rows
            .borrow()
            .get(&signature.declaration)
            .cloned()
            .unwrap_or_else(|| syntactic.clone());
        // [EFF-3, FN-2] the cached symbolic body fixes the source-written
        // path row for every instance, but allocation has no source entry and
        // can become known only after a function argument is concrete. Keep
        // that instance fact instead of replacing it with the symbolic
        // schema's necessarily absent metadata.
        written.allocates |= syntactic.allocates;
        written
    }

    /// [GRAM-2, STOR-8] whether this unit wrote `program no_heap;`.
    ///
    /// Resolution has already refused a second `heap_decl` and one at any
    /// later item position, so a `heap_decl` present anywhere under the root
    /// is the admitted first-item declaration.
    fn declares_no_heap(
        tree: &TreeView<'unit, 'classified, 'lexed, 'source>,
    ) -> Result<bool, CheckStop> {
        for item in tree.children(tree.root())? {
            if tree.production(*item)? != Production::Item {
                continue;
            }
            for child in tree.children(*item)? {
                if tree.production(*child)? == Production::HeapDecl {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn new(
        resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
        reject_entailment: bool,
    ) -> Result<Self, CheckStop> {
        // A semantic unit includes the fixed PRE-1 declarations before resolution.
        // A source-only parse is useful to tools but is not a complete compiler input.
        if !resolved
            .syntax()
            .finalized
            .parsed
            .classified
            .source_bundle()
            .includes_prelude()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let tree = TreeView::new(resolved)?;
        let no_heap = Self::declares_no_heap(&tree)?;
        Ok(Self {
            resolved,
            reject_entailment,
            no_heap,
            tree,
            nominals: Vec::new(),
            elements: RefCell::new(Vec::new()),
            element_ids: RefCell::new(HashMap::new()),
            nominal_nodes: Vec::new(),
            nominal_states: Vec::new(),
            source_nominal_instances: Vec::new(),
            box_nominals: HashMap::new(),
            arena_nominals: HashMap::new(),
            result_list_nominals: HashMap::new(),
            pending_nominals: RefCell::new(Vec::new()),
            pending_instances: RefCell::new(Vec::new()),
            elided_store_brand: std::cell::Cell::new(None),
            template_spelling_authority: std::cell::Cell::new(false),
            commit_read_outs: RefCell::new(Vec::new()),
            call_separations: RefCell::new(Vec::new()),
            contract_queries: RefCell::new(Vec::new()),
            prelude_nominals: HashMap::new(),
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
            written_body_effect_rows: RefCell::new(HashMap::new()),
            pending_generic_requirements: Vec::new(),
            generic_requirements: Vec::new(),
            postcondition_selectors: Vec::new(),
            postcondition_unavailable_declarations: Vec::new(),
            active_postcondition: Cell::new(None),
            active_result_datums: RefCell::new(Vec::new()),
            behavior: behavior::BehaviorInventory::default(),
        })
    }

    fn check_program(&mut self) -> Result<CheckedProgramData, CheckStop> {
        let items = self.item_declarations()?;
        self.collect_behavior_groups(&items)?;
        self.reject_instantiation_cycles(&items)?;
        self.declare_nominals(&items)?;
        self.collect_constants(&items)?;
        self.complete_nominals()?;
        self.collect_deferred_nominal_constants(&items)?;
        self.collect_function_signatures(&items)?;
        self.admit_postcondition_selectors()?;
        self.validate_generic_templates()?;
        if self
            .signatures
            .iter()
            .any(|signature| signature.formal_parameter.is_some())
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }

        // Phase A completes every reachable concrete function before any
        // acceptance-bearing entailment judgment runs. This makes forward,
        // recursive, mutually recursive, and concrete generic call summaries
        // independent of function traversal order.
        // The cursor is re-read each round because checking a body may select
        // an operand-directed [PRE-1] instance [OP-10, OP-11, OP-14] the
        // syntax could not name; those signatures are appended here and are
        // checked by this same loop before phase B reads the inventory.
        let mut function_inventory = Vec::with_capacity(self.signatures.len());
        let mut index = 0_usize;
        while index < self.signatures.len() {
            function_inventory.push(self.check_function_interning_nominals(index)?);
            index = index
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        // Body checking may first instantiate a nominal through the fields of
        // an ordinary constructor. FN-6 has already checked the written finite
        // dependency graph, and ensure_source_nominal_instance has completed
        // each discovered instance. Lowering reads this completed inventory;
        // no hidden entry-store instance is required to seed a representation.

        self.check_behavior_bindings()?;

        // Phase B reads only the completed inventory. Kill-relevant [EFF-2]
        // projections are indexed by dense function identity [ENT-5]; later
        // program-level goal summaries extend this same complete context.
        let callees = self.entailment_callees()?;
        self.install_call_requirements(&mut function_inventory)?;
        let permission_signatures = self
            .signatures
            .iter()
            .map(|signature| PermissionSignature {
                name: signature.name.clone(),
                parameter_declarations: signature
                    .parameters
                    .iter()
                    .map(|parameter| parameter.declaration)
                    .collect(),
                parameter_modes: signature
                    .parameters
                    .iter()
                    .map(|parameter| parameter.mode)
                    .collect(),
                reads: signature.declared_effects.reads.clone(),
                writes: signature.declared_effects.writes.clone(),
            })
            .collect::<Vec<_>>();
        for checked in &mut function_inventory {
            checked.function.permission_separation_queries =
                plan_permission_separations(&checked.function, &permission_signatures);
        }
        let optimistic_batch = function_inventory.iter().any(|checked| {
            !checked.function.postconditions.is_empty()
                || Self::statements_contain_value_if(
                    checked.function.body.as_deref().unwrap_or_default(),
                )
        });

        let postcondition_schedule = self.analyze_function_inventory(
            &mut function_inventory,
            &callees,
            optimistic_batch,
            None,
        )?;
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

        let executable_nominal_count = self.nominals.len();
        self.materialize_generic_requirements()?;
        let derived_consts = self.derived_consts.borrow().clone();
        for (index, derived) in derived_consts.iter().enumerate() {
            for operand in [derived.left, derived.right] {
                if matches!(operand, CheckedConst::Derived(id) if id.0 as usize >= index) {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
        }
        // Permission is a read-only legality table over the completed checked
        // program. The affine-map rule consumes a successful OP-4 disposition
        // and exact value image retained on that program; no permission rule
        // repeats a local invariant or changes source acceptance.
        let permission = analyze_permission(&functions, &permission_signatures);
        // The ledger is rendered here because only the checker still holds the
        // syntax tree the citations name. It is pure presentation over the
        // table above and reaches no decision.
        let permission_ledger = if permission
            .functions
            .iter()
            .any(|permissions| !permissions.pairs.is_empty() || !permissions.loops.is_empty())
        {
            render_ledger(&permission, &PermissionLedgerSource { tree: &self.tree })?
        } else {
            Vec::new()
        };

        Ok(CheckedProgramData {
            nominals: self.nominals.clone(),
            nominal_confinement: self
                .nominals
                .iter()
                .map(|nominal| self.confinement_regions(CheckedType::Nominal(nominal.id)))
                .collect::<Result<_, _>>()?,
            elements: self.elements.borrow().clone(),
            executable_nominal_count,
            nominal_lowering_alias: self.nominal_lowering_aliases()?,
            nominal_physical_alias: self.nominal_physical_aliases()?,
            constants: self.checked_constants.clone(),
            derived_consts,
            functions,
            contract_queries: self.contract_queries.borrow().clone(),
            postcondition_schedule,
            generic_requirements: self.generic_requirements.clone(),
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
        self.validate_formal_declarations()?;
        self.collect_concrete_function_signatures()
    }

    /// Collects every const declaration with a nominal-free type. Runs before
    /// nominal completion because a nominal field's array length may name an
    /// earlier const; constants containing nominal types [CONST-2]
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

    /// Collects constants containing nominal types, deferred by the first
    /// pass, in item order, after `complete_nominals` has filled the field
    /// inventories they are checked against. CONST-2's declaration-before-use
    /// rule is unaffected: a nominal-free constant cannot reference a value
    /// containing a nominal type (a cvalue reference has the exact expected
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
        let ty = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mut pending = vec![ty];
        while let Some(node) = pending.pop() {
            if self.tree.production(node)? == Production::Type
                && self
                    .tree
                    .direct_token_with(node, crate::TerminalPredicate::TypeIdentifier)?
                    .is_some()
            {
                return Ok(true);
            }
            pending.extend(self.tree.children(node)?.iter().copied());
        }
        Ok(false)
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
        if self.tree.direct_token_indices(value)?.len() == 1
            && self
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
        let declared_type = self.parse_type(ty_node)?;
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
            declared_type,
            ty,
            value,
        });
        self.constants.insert(declaration_id, id);
        Ok(())
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
                    let instances = std::mem::take(&mut *self.pending_instances.borrow_mut());
                    let before = self.nominals.len();
                    let before_signatures = self.signatures.len();
                    for (template, substitution) in instances {
                        self.ensure_operand_directed_instance(template, substitution)?;
                    }
                    for nominal in pending {
                        match nominal {
                            PendingNominal::Box(referent) => {
                                self.intern_box_nominal(referent)?;
                            }
                            PendingNominal::ResultList(results) => {
                                self.intern_result_list_nominal(&results)?;
                            }
                            PendingNominal::Arena(region, content) => {
                                self.intern_arena_nominal(region, content)?;
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
                    if self.nominals.len() == before && self.signatures.len() == before_signatures {
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

    /// [FN-2, OWN-1, PROV-6] one body, checked with the template's spelling
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
        let previous = self.template_spelling_authority.replace(
            signature.substitution.len() > 0
                && signature.substitution.is_concrete(&self.elements.borrow()),
        );
        let outcome = self.check_function_signature_body(signature);
        self.template_spelling_authority.set(previous);
        outcome
    }

    /// [OP-10, OP-14] whether this instance still carries a compiler-owned
    /// window type parameter unsubstituted, which is exactly the symbolic
    /// instance of a window row.
    fn has_unsupplied_window_type_parameter(
        &self,
        signature: &FunctionSignature,
    ) -> Result<bool, CheckStop> {
        for (key, argument) in signature.substitution.entries() {
            let generics::GenericParameterKey::Source(declaration) = key else {
                continue;
            };
            let generics::GenericArgument::Type(ty) = argument else {
                continue;
            };
            if !self.is_window_type_parameter(*declaration)? {
                continue;
            }
            // Once an operand has supplied the shape, `deref(window).len`
            // names a measure of one of [MSR-1]'s measured types whatever
            // that shape's element type and capacity still are. [OP-14] also
            // admits a Box holding a runtime-capacity window; its concrete W
            // is not itself measured, but the prelude clause's measure place
            // instantiates as `window.inner` [TYPE-9]. Only the unsubstituted
            // parameter of the symbolic schema instance has neither form.
            let boxed_runtime_window = matches!(
                self.box_content(*ty)?,
                Some(CheckedType::Window { capacity: None, .. })
            );
            if ty.measured().is_none() && !boxed_runtime_window {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn check_function_signature_body(
        &self,
        signature: &FunctionSignature,
    ) -> Result<CheckedFunctionInventory, CheckStop> {
        self.check_entry_formers(signature)?;
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
                self.parameter_local(parameter, binding)?,
            );
            parameters.push(CheckedParameter {
                name: parameter.name.clone(),
                declaration: parameter.declaration,
                node_path: parameter.node_path.clone(),
                binding,
                mode: parameter.mode,
                ty: parameter.ty,
                range_element: (parameter.mode == CheckedMode::Range)
                    .then(|| self.intern_element(parameter.ty))
                    .transpose()?,
            });
        }

        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
            binding_names: &mut binding_names,
        };
        let parameter_bindings = bindings.clone();
        // [OP-10] a compiler-owned window type parameter is supplied by the
        // operand and never written, so a row carrying one has no symbolic
        // instance any call can name: `deref(window).cap` names a measure of
        // the shape the operand supplies, and the unsubstituted parameter is
        // not one of [MSR-1]'s measured types. Such a row's clauses are the
        // substituted clauses of each concrete instance, which the ordinary
        // judgment below reaches once an operand fixes W. Every other generic
        // row, prelude or source, keeps its symbolic judgment.
        let unsupplied_window_row = self.has_unsupplied_window_type_parameter(signature)?;
        let requirements = if let Some(node) = self
            .tree
            .first_child_with(signature.node, Production::ContractBlock)?
            .filter(|_| !unsupplied_window_row)
        {
            let mut requires_bindings = parameter_bindings.clone();
            self.check_requires(signature, node, &mut requires_bindings, &mut counters)?
                .requirements
        } else {
            Vec::new()
        };

        let postcondition_selectors = if unsupplied_window_row {
            Vec::new()
        } else {
            self.postcondition_selectors_for_signature(signature)?
        };
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
        let mut checked = self.check_block(
            signature,
            &statements,
            &mut bindings,
            &mut counters,
            ControlScope {
                loops: &[],
                give_context: None,
            },
        )?;
        let declaration_only = self.tree.production(signature.node)? == Production::FnSig;
        if declaration_only {
            checked.can_continue = false;
            checked.effects = signature.declared_effects.clone();
        }
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
        let mut exhibited = self.written_body_effects(signature, checked.effects.clone());
        self.collect_release_effects(signature, &checked.statements, &mut exhibited)?;
        // [EFF-1] the row has exactly two categories, and [STOR-8] gives
        // allocation no entry in it, so [EFF-2]'s judgment is over `reads`
        // and `writes` alone. The allocation fact is [EFF-3] checked-program
        // metadata that no declaration can write, so comparing it here would
        // reject every `pure` function that calls a construction row against
        // a row it had no way to declare.
        //
        // Each category is judged by [EFF-2]'s own two-way covering relation
        // rather than by set equality.
        if !Self::effect_row_matches(&signature.declared_effects, &exhibited) {
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
        let postconditions = if signature.substitution.is_concrete(&self.elements.borrow()) {
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
            formal_hypothesis: signature.formal_parameter.is_some(),
            id: signature.id,
            declaration: signature.declaration,
            name: signature.name.clone(),
            symbol: signature.symbol.clone(),
            region_parameters: signature.region_parameters.clone(),
            parameters,
            result_mode: signature.result_mode,
            result: signature.result,
            declared_state_writes: signature.declared_effects.writes.clone(),
            // [EFF-3] the boundary's allocation fact is what the body
            // exhibits, not what the declaration wrote: no declaration can
            // write it [EFF-1, STOR-8], and for a body-less row the exhibited
            // set is the declared one, so a construction row still reports
            // its own allocation here.
            allocates: exhibited.allocates,
            requirements,
            postconditions,
            body: (!declaration_only).then_some(checked.statements),
            body_disposition: super::model::CheckedBodyDisposition::Inhabited,
            call_separations: {
                let mut separations = std::mem::take(&mut *self.call_separations.borrow_mut());
                separations.sort_by_key(|separation| separation.site.components().to_vec());
                separations
            },
            permission_separation_queries: Vec::new(),
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
            CheckedStatement::Loop { body, .. } | CheckedStatement::CountedRange { body, .. } => {
                Self::statements_contain_value_if(body)
            }
            CheckedStatement::Let { .. }
            | CheckedStatement::DestructuringLet { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Evaluate(_)
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
                    .map(|parameter| (parameter.declaration, parameter.mode, parameter.ty)),
                &signature.declared_effects.writes,
            ));
        }
        Ok(callees)
    }

    fn parameter_local(
        &self,
        parameter: &ParameterSignature,
        binding: BindingId,
    ) -> Result<LocalBinding, CheckStop> {
        Ok(LocalBinding {
            binding,
            declaration: parameter.declaration,
            mode: parameter.mode,
            ty: parameter.ty,
            live: true,
            loop_depth: 0,
            compiler_updated: false,
            // [REF-1] a reference parameter arrives naming the caller's path
            // by substitution [EFF-5]; inside this body the parameter name is
            // that path, so its set anchors at itself and every resolution
            // through it terminates there.
            reference: parameter.mode.is_reference().then(|| {
                ReferenceInfo::formed(
                    if parameter.mode == CheckedMode::Range {
                        references::ReferenceKind::Range
                    } else {
                        references::ReferenceKind::Single
                    },
                    ResolvedPlace::binding(binding),
                )
            }),
            refinement_witnesses: Vec::new(),
        })
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
                || Self::statements_contain_value_if(
                    checked.function.body.as_deref().unwrap_or_default(),
                )
        });
        // Only the canonical instances are judged below, and a judged body
        // reads another function's analysis solely through the postcondition
        // summaries of its callees. The other bodies of this scratch
        // inventory — every nongeneric function among them — are analyzed
        // again by the concrete phase, so analyzing them here would repeat
        // that whole cost for a result nothing reads.
        let analyzed = Self::generic_validation_scope(functions, canonical)?;
        self.analyze_function_inventory(functions, callees, optimistic_batch, Some(&analyzed))?;
        if optimistic_batch {
            for (checked, analyzed) in functions.iter_mut().zip(&analyzed) {
                if *analyzed {
                    finalize_function_entailment(&mut checked.function.entailment);
                }
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
    /// The functions whose bodies symbolic validation must analyze: the
    /// canonical instances and everything their calls reach, by the same
    /// call-graph shape the postcondition schedule is built from.
    ///
    /// The set is closed under callees, so a strongly connected component is
    /// either wholly inside it or wholly outside, and every summary an
    /// analyzed body can consume is published by an analyzed component.
    fn generic_validation_scope(
        functions: &[CheckedFunctionInventory],
        canonical: &[(usize, DeclarationId)],
    ) -> Result<Vec<bool>, CheckStop> {
        let mut analyzed = vec![false; functions.len()];
        let mut pending = canonical
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        let mut calls = Vec::new();
        while let Some(index) = pending.pop() {
            let slot = analyzed
                .get_mut(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if std::mem::replace(slot, true) {
                continue;
            }
            let function = &functions[index].function;
            calls.clear();
            collect_statement_calls(
                function.id,
                function.body.as_deref().unwrap_or_default(),
                &mut calls,
            );
            pending.extend(calls.iter().map(|call| call.callee.0 as usize));
        }
        Ok(analyzed)
    }

    /// Analyzes every function of the inventory, or only those `analyzed`
    /// marks. A caller that restricts the set must close it under callees.
    fn analyze_function_inventory(
        &self,
        functions: &mut [CheckedFunctionInventory],
        callees: &[EntailmentCallee],
        optimistic_batch: bool,
        analyzed: Option<&[bool]>,
    ) -> Result<PostconditionSchedule, CheckStop> {
        let selected = |index: usize| analyzed.is_none_or(|analyzed| analyzed[index]);
        let contract_queries = self.contract_queries.borrow().clone();
        // ENT is the single acceptance-bearing proof path for ordinary
        // obligations, call requirements, invariants and postconditions.
        let mut schedule =
            postcondition_schedule(functions.iter().map(|checked| &checked.function))
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if schedule.components.is_empty() {
            for (index, checked) in functions.iter_mut().enumerate() {
                if !selected(index) {
                    continue;
                }
                let context = EntailmentContext {
                    callees,
                    constants: &self.checked_constants,
                    constant_ids: &self.constants,
                    nominals: &self.nominals,
                    elements: &self.elements.borrow(),
                    contract_queries: &contract_queries,
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
                // Callee closure keeps components whole: skipping one skips
                // both its analysis and the summaries no analyzed body reads.
                if !component
                    .functions
                    .iter()
                    .any(|function| selected(function.0 as usize))
                {
                    continue;
                }
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
                        elements: &self.elements.borrow(),
                        contract_queries: &contract_queries,
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
            if let Some(body) = &mut checked.function.body {
                self.install_statement_call_requirements(body, &requirements)?;
            }
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
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    self.install_expression_call_requirements(value, requirements)?;
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => {
                    self.install_expression_call_requirements(scrutinee, requirements)?;
                }
                CheckedStatement::Set { target, value, .. } => {
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
                        CheckedSetTarget::RangeIndex(target) => self
                            .install_expression_call_requirements(
                                &mut target.offset,
                                requirements,
                            )?,
                        CheckedSetTarget::Storage(target) => {
                            for offset in target.offsets_mut() {
                                self.install_expression_call_requirements(offset, requirements)?;
                            }
                        }
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
                CheckedStatement::Loop { body, .. } => {
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
                formal_contract,
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
                let boundary = match formal_contract {
                    Some(boundary) => boundary.requirements.as_slice(),
                    None => requirements
                        .get(function.0 as usize)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                };
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
            CheckedExpression::IntegerOperation { arguments, .. }
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
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::BoxTake { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.install_expression_call_requirements(value, requirements)?;
            }
            CheckedExpression::ReadStorage { root, .. } => {
                for offset in root.offsets_mut() {
                    self.install_expression_call_requirements(offset, requirements)?;
                }
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::RangeIndex { offset, .. }
            | CheckedExpression::BorrowRangeIndex { offset, .. } => {
                self.install_expression_call_requirements(offset, requirements)?;
            }
            CheckedExpression::RangeElementMeasure { place, .. } => {
                self.install_expression_call_requirements(&mut place.offset, requirements)?;
                for step in &mut place.path {
                    if let super::model::CheckedPlaceStep::Subscript(subscript) = step {
                        self.install_expression_call_requirements(
                            &mut subscript.offset,
                            requirements,
                        )?;
                    }
                }
            }
            // [REF-4] both endpoints are ordinary operands evaluated at the
            // formation, and a storage source carries its own offsets.
            CheckedExpression::RangeOf {
                source, start, end, ..
            } => {
                if let super::model::CheckedRangeSource::Storage(root) = source {
                    for offset in root.offsets_mut() {
                        self.install_expression_call_requirements(offset, requirements)?;
                    }
                }
                self.install_expression_call_requirements(start, requirements)?;
                self.install_expression_call_requirements(end, requirements)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::RangeMeasure { .. }
            | CheckedExpression::BorrowAddressed { .. }
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
            if let Some(body) = &mut function.body {
                Self::install_statement_allocation_bounds(body, &bounds)?;
            }
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
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    Self::install_expression_allocation_bounds(value, bounds)?;
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => {
                    Self::install_expression_allocation_bounds(scrutinee, bounds)?;
                }
                CheckedStatement::Set { target, value, .. } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::RangeIndex(target) => {
                            Self::install_expression_allocation_bounds(&mut target.offset, bounds)?;
                        }
                        CheckedSetTarget::Storage(target) => {
                            for offset in target.offsets_mut() {
                                Self::install_expression_allocation_bounds(offset, bounds)?;
                            }
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
                CheckedStatement::Loop { body, .. } => {
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
            CheckedExpression::UserCall {
                call,
                arguments,
                allocation,
                ..
            } => {
                if let Some(allocation) = allocation
                    && let Some(upper) = bounds.get(call).copied()
                {
                    allocation.install_source_length_upper_bound(upper);
                }
                for argument in arguments {
                    Self::install_expression_allocation_bounds(argument, bounds)?;
                }
            }
            CheckedExpression::IntegerOperation { arguments, .. }
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
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::BoxTake { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                Self::install_expression_allocation_bounds(value, bounds)?;
            }
            CheckedExpression::ReadStorage { root, .. } => {
                for offset in root.offsets_mut() {
                    Self::install_expression_allocation_bounds(offset, bounds)?;
                }
            }
            CheckedExpression::ArrayIndex { offset, .. }
            | CheckedExpression::BufferIndex { offset, .. }
            | CheckedExpression::RangeIndex { offset, .. }
            | CheckedExpression::BorrowRangeIndex { offset, .. } => {
                Self::install_expression_allocation_bounds(offset, bounds)?;
            }
            CheckedExpression::RangeElementMeasure { place, .. } => {
                Self::install_expression_allocation_bounds(&mut place.offset, bounds)?;
                for step in &mut place.path {
                    if let super::model::CheckedPlaceStep::Subscript(subscript) = step {
                        Self::install_expression_allocation_bounds(&mut subscript.offset, bounds)?;
                    }
                }
            }
            CheckedExpression::RangeOf {
                source, start, end, ..
            } => {
                if let super::model::CheckedRangeSource::Storage(root) = source {
                    for offset in root.offsets_mut() {
                        Self::install_expression_allocation_bounds(offset, bounds)?;
                    }
                }
                Self::install_expression_allocation_bounds(start, bounds)?;
                Self::install_expression_allocation_bounds(end, bounds)?;
            }
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::Binding { .. }
            | CheckedExpression::ArrayMeasure { .. }
            | CheckedExpression::BufferMeasure { .. }
            | CheckedExpression::ContainerMeasure { .. }
            | CheckedExpression::RangeMeasure { .. }
            | CheckedExpression::BorrowAddressed { .. }
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
                    // [MSR-1, EFF-5] a formal-valued subscript names a value
                    // parameter of the callee, and the caller substitutes its
                    // own actual for it exactly as it substitutes a row's
                    // index positions: the offset the place is identified
                    // over is the value that argument names here.
                    let projection = match projection {
                        GoalProjection::FormalSubscript { ordinal } => GoalProjection::Subscript(
                            Self::goal_argument_offset(arguments.get(*ordinal as usize))?,
                        ),
                        other => *other,
                    };
                    image = image
                        .with_projection(projection, final_type)
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

    /// The offset one actual supplies to a formal-valued subscript [MSR-1].
    ///
    /// [ENT-2] decides a place's identity by its canonical spelling, so an
    /// argument that names a literal, a named or generic const, or a live
    /// binding supplies exactly that, and an argument that names none of
    /// them supplies the unknown offset, which no admitted family separates
    /// -- the conservative reading in both directions [OWN-7].
    fn goal_argument_offset(
        argument: Option<&GoalExpression>,
    ) -> Result<super::places::CapturedValue, CheckStop> {
        use super::places::{CapturedTerm, CapturedValue};
        let unknown = CapturedValue::unknown();
        let Some(GoalExpression::Datum(datum)) = argument else {
            return Ok(unknown);
        };
        Ok(match datum {
            GoalDatum::Literal(CheckedValue::Integer { bits, .. }) => {
                CapturedValue::new(unknown.capture, CapturedTerm::Literal(*bits))
            }
            GoalDatum::Literal(CheckedValue::ConstGeneric { declaration, .. }) => {
                CapturedValue::new(unknown.capture, CapturedTerm::Const(*declaration))
            }
            GoalDatum::Place {
                root, projections, ..
            } if projections.is_empty() => {
                CapturedValue::new(unknown.capture, CapturedTerm::Binding(*root))
            }
            _ => return Ok(unknown),
        }
        .goal_identity())
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
            GoalOperation::ArrayMeasure {
                measure,
                element,
                length,
            } => GoalOperation::ArrayMeasure {
                measure,
                element: self.instantiate_goal_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            GoalOperation::ArrayIndex { element, length } => GoalOperation::ArrayIndex {
                element: self.instantiate_goal_element(element, signature, regions)?,
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
                element: self.instantiate_goal_element(element, signature, regions)?,
                length: self.instantiate_goal_const(length, signature)?,
            },
            CheckedType::Buffer { element } => CheckedType::Buffer {
                element: self.instantiate_goal_flat_element(element, signature, regions)?,
            },
            CheckedType::Window {
                shape,
                element,
                capacity,
            } => CheckedType::Window {
                shape,
                element: self.instantiate_goal_element(element, signature, regions)?,
                capacity: capacity
                    .map(|capacity| self.instantiate_goal_const(capacity, signature))
                    .transpose()?,
            },
            CheckedType::Nominal(id) => self.instantiate_goal_nominal(id, signature, regions)?,
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => ty,
        })
    }

    /// [FN-2, S20] one nominal instance read at a caller with every formal
    /// region substituted, which is the instance the caller's own value has.
    ///
    /// The structural walk is the same one used while checking the call, so
    /// it reaches a source nominal through PRE-1 wrappers and compiler-owned
    /// store nominals as well as a source nominal's own region axis. It keeps
    /// that walk's ordinary lookup-and-defer behavior; it does not assume a
    /// result-only or nested goal type was itself a direct call argument.
    fn instantiate_goal_nominal(
        &self,
        id: NominalId,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        if signature.region_parameters.len() != regions.len() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let substitution = signature
            .region_parameters
            .iter()
            .copied()
            .zip(regions.iter().copied())
            .filter(|(formal, actual)| formal != actual)
            .collect::<Vec<_>>();
        self.substitute_type_regions(CheckedType::Nominal(id), &substitution)
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
                if self.nominal(nominal)?.is_tag_only_enum() {
                    CheckedFlatElement::TagOnlyNominal(nominal)
                } else {
                    CheckedFlatElement::Nominal(nominal)
                }
            }
            CheckedType::Generic(_)
            | CheckedType::Array { .. }
            | CheckedType::Buffer { .. }
            | CheckedType::Window { .. } => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        })
    }

    /// One run element at a caller's instance [BLK-1].
    ///
    /// Complete slot types use the ordinary recursive type substitution and
    /// are re-interned only after all formal type, const and region arguments
    /// have been instantiated.
    fn instantiate_goal_element(
        &self,
        element: CheckedElement,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedElement, CheckStop> {
        let ty = self.instantiate_goal_type(self.element_type(element)?, signature, regions)?;
        self.intern_element(ty)
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
                        super::entailment::ObligationFamily::EmptyRunRelease => SemanticRule::Prov6,
                        super::entailment::ObligationFamily::IntegerDomain => SemanticRule::Op2,
                        super::entailment::ObligationFamily::AllocationFit => SemanticRule::Op9,
                        super::entailment::ObligationFamily::RangeFormation => SemanticRule::Ref4,
                        super::entailment::ObligationFamily::RangeSeparation => SemanticRule::Eff5,
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
                            | super::entailment::SourceProofCertificateFailure::NonlinearResidual
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
                        super::entailment::SourceProofCertificateFailure::NonlinearResidual => {
                            crate::SourceProofObligation::NonlinearCertificateSum
                        }
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
                        crate::SourceProofObligation::NonlinearCertificateSum => {
                            "the multiplied operand must be one the checker holds as a single value — a parameter or a call result — because a locally derived one is expanded into its own operands and no admitted product then matches the sum; take it as a parameter, or scale the premise by a bare decimal instead"
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
                        super::entailment::ObligationFamily::EmptyRunRelease => SemanticIssue {
                            rule: SemanticRule::Prov6,
                            location,
                            kind: SemanticIssueKind::UndischargedEmptyRunRelease {
                                residual,
                                mechanical_fix: "empty the run and establish its zero length at this release point; otherwise consume or release every live element before releasing the backing",
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
                                mechanical_fix: "the allocation's own size arithmetic must stay inside u64: bound the count with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when the refusal is intended program behavior; otherwise restructure the allocation",
                            },
                        },
                        super::entailment::ObligationFamily::RangeSeparation => SemanticIssue {
                            rule: SemanticRule::Eff5,
                            location,
                            kind: SemanticIssueKind::UndischargedRangeSeparation {
                                residual,
                                mechanical_fix: "prove the two ranges disjoint by one of OWN-7's four non-strict orderings before this call, or name one range in place of the pair",
                            },
                        },
                        super::entailment::ObligationFamily::RangeFormation => SemanticIssue {
                            rule: SemanticRule::Ref4,
                            location,
                            kind: SemanticIssueKind::UndischargedRangeFormationObligation {
                                residual,
                                mechanical_fix: "establish lo <= hi and hi <= x.len with a verified requirement, a source invariant, or explicit finite proof steps; otherwise restructure the range",
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
                    // [OP-14] `free_empty` has its own site: "An undischarged
                    // obligation is a hard error citing OP-14 at the complete
                    // `call`, rendering the residual". The requirement reaches
                    // the checker on the ordinary call-requirement path, so
                    // the rule and the restructuring are selected here rather
                    // than by a second judgment of the same goal.
                    if signature.name == "free_empty" {
                        return Err(CheckStop::source_issue(SemanticIssue {
                            rule: SemanticRule::Op14,
                            location: SemanticLocation::SourceNode(
                                outcome.node_path.clone(),
                                self.tree.coordinate(node)?,
                            ),
                            kind: SemanticIssueKind::UndischargedEmptyRunRelease {
                                residual: outcome.rendered_goal.clone(),
                                mechanical_fix: "empty the window and establish its zero length at this point; otherwise take every element out and consume it",
                            },
                        }));
                    }
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

        // PRE-1 signature contracts have a declaration premise, not selected
        // WF return statements. Their ordinary aggregate was published by
        // the same entailment schedule before caller goals were checked.
        if function.body.is_none()
            || matches!(
                function.entailment.body_disposition,
                super::model::CheckedBodyDisposition::Uninhabited { .. }
            )
        {
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
