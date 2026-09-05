//! The permission judgment P: whether two sibling call statements may be
//! executed with their evaluations overlapped.
//!
//! P is a compiler-internal legality judgment. It refuses nothing, changes no
//! acceptance, and grants no lowering by itself; it records, per analyzed
//! site, whether overlapping the two statements is permitted. A permitted
//! window is actualizable, full stop.
//!
//! # The window
//!
//! The judged unit is a *window*: an ordered pair (s1, s2) of `let x = f(...);`
//! statements in one block, both binding the result of one named-function
//! call, together with every statement of that block strictly between them.
//! [PAR-1] says "s1 **precedes** s2 in one block", not that it immediately
//! precedes it, and permission may not turn on accidental statement adjacency
//! — one builtin written between two calls must not decide whether they may
//! overlap, when the same operation wrapped in a pure function would leave
//! them adjacent. Every condition below therefore quantifies over the
//! interposed statements as well as over the two calls, and an interposed
//! statement this analysis cannot account for **denies with a report** rather
//! than silently ending the enumeration.
//!
//! P(s1, s2) holds exactly when all four conditions hold. Writing T for an
//! interposed statement, D(u) for the binding u defines, U(u) for the bindings
//! its operands mention, and W/R/O(u) for u's written, read, and caller-side
//! operand-read footprints:
//!
//! 1. **No dataflow.** No argument of s2 mentions a binding s1 defines; no
//!    interposed statement's operands mention a binding s1 defines; and no
//!    argument of s2 mentions a binding an interposed statement defines. The
//!    two added clauses are the two schedules speaking: where s1 is handed
//!    out, its value does not exist until the join, so no T may read it; where
//!    s2 is handed out, its operands are evaluated before T1…Tk run, so s2 may
//!    not read what they define. T1…Tk keep their mutual order on the calling
//!    thread under both schedules, so there is no clause between them.
//! 2. **Disjoint footprints.** Projecting each callee's declared `reads` and
//!    `writes` regions onto its argument resolved places — the same [EFF-2]
//!    boundary projection [ENT-5] kills use — gives W(s) and R(s). P requires
//!    W(s1) disjoint from W(s2) and R(s2), and W(s2) disjoint from R(s1),
//!    under [OWN-7]'s overlap relation over resolved places. An actual whose
//!    caller place this analysis cannot resolve fails closed.
//!
//!    The callee projection is not the whole footprint. A statement also
//!    reaches storage *before* its call, on the calling thread, while it
//!    evaluates its own operands, and an overlap moves that evaluation across
//!    the other member's call — so each member's writes must also be disjoint
//!    from the other's caller-side operand reads. Without this the pair
//!    `let a = bump(slot: &uniq 'r cell); let b = take(v: cell);` is permitted
//!    while `take`'s operand reads the storage `bump` writes, which is both a
//!    changed result and, on a granted lane, a data race. Both directions are
//!    judged because which member's operands move is the implementation's
//!    choice of which member takes the lane, which permission may not depend
//!    on.
//!
//!    Each interposed T carries the same obligations against both members,
//!    with **one asymmetry that must not be lost**: T is judged against s2
//!    exactly as an ordinary earlier member is — including `W(T)` against
//!    `O(s2)`, because the schedule that hands s2 out hoists its operand
//!    evaluation above T — while against s1 the mirror obligation `W(T)`
//!    against `O(s1)` does **not** arise, because the schedule that hands s1
//!    out evaluates its operands before the fork and the schedule that hands
//!    s2 out has already completed s1. The operand half is one-sided for s1
//!    and two-sided for s2. Getting this wrong conservatively costs only
//!    denials; getting it wrong permissively is a race.
//! 3. **Target support does not alter legality.** A may-suspend target does not
//!    deny permission: it selects completion lowering when one exists,
//!    otherwise the permitted window stays sequential. State conflicts and
//!    lifetime exclusion have already been decided by conditions 2 and the
//!    ordinary loans above.
//! 4. **No skipping exit.** No exit edge of s1 bypasses s2, and no statement
//!    between them carries an exit edge at all: s1's only continuation is s2.
//!    A `propagate` right-hand side has an `Err` edge to the function-return
//!    sink [ERR-3], so it is never a first member and never an interposed one.
//!    This is not merely a condition about differing observables: under either
//!    schedule a hand-out is outstanding at every interposed statement, so an
//!    exit taken there abandons an unjoined lane still reading the caller's
//!    frame. Source proof statements are erased before lowering and therefore
//!    introduce no runtime exit edge.
//!
//! Two schedules are realizable for one window — hand s1 to a lane and run
//! T1…Tk then s2 on the calling thread, or run s1 then hoist s2's operands,
//! hand s2 out, and run T1…Tk — and [PAR-1] forbids stating any rule in terms
//! of the schedule. The conditions above are therefore the **intersection** of
//! what the two admit, never the weaker set the current backend alone would
//! survive.
//!
//! # Proof statements do not add a fifth condition
//!
//! Nothing beyond those four conditions is required. Every source proof
//! statement has already been checked against its control-flow facts before
//! permission metadata is built. It is then erased before lowering: it has no
//! runtime evaluation, effect, exit edge, or scheduler-visible event. A failed
//! proof rejects the program instead of creating a runtime fallback. The
//! permission judgment therefore neither rechecks proofs nor models a proof
//! failure path.
//!
//! **Invariant.** The window and staged judgments consult typing, declared
//! effect rows, resolved places [OWN-5, OWN-7], and statement-graph exit edges.
//! The counted-loop judgment additionally consumes an already-successful
//! [OP-4] disposition and its retained single-binder affine value image; it
//! does not repeat that proof or inspect unrelated entailment facts. Permission remains a
//! read-only lowering judgment: it cannot turn an accepted program into a
//! rejected one or move a required check.

use super::loop_permission::LoopPermission;
use super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedFunction, CheckedMode, CheckedSetTarget,
    CheckedSliceSource, CheckedStatePath, CheckedStatement, FunctionId, expression_children,
};
use super::places::{PlaceMap, PlaceRoot, PlaceTerm, ResolvedPlace};
use super::staged_permission::StagedPermission;
use crate::{
    DeclarationId, NodePath, SYSTEM_OPERATIONS, SystemParameterMode, TargetAction,
    operation_state_effects,
};

/// The declared effect row and region parameters of one concrete function, as
/// P reads them. This is the callable boundary only: no body fact enters.
#[derive(Clone, Debug, Default)]
pub(crate) struct PermissionSignature {
    /// Formal region parameters in declaration order.
    pub(crate) region_parameters: Vec<DeclarationId>,
    pub(crate) reads: Vec<CheckedStatePath>,
    pub(crate) writes: Vec<CheckedStatePath>,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
}

/// Which statement of an analyzed window a denial cites: one of the two
/// judged calls, or one of the statements written between them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairSide {
    First,
    Second,
    /// The interposed statement at this zero-based position in the window.
    Between(usize),
}

/// One exit edge of a window statement that does not reach the statement's
/// ordinary successor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExitKind {
    /// A `propagate` right-hand side's `Err` edge to the function-return
    /// sink [ERR-3].
    PropagateError,
    /// A `return`, `give`, or `break` edge, which leaves the enclosing block
    /// or function without reaching s2.
    BlockExit,
}

/// One footprint element: a resolved caller place, or one arena region whose
/// allocation list the callee appends to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Access {
    /// Storage the callee reaches through one actual.
    Place {
        place: ResolvedPlace,
        /// The actual's source node, for citation.
        argument: NodePath,
    },
    /// The caller region an `allocates(arena 'r)` row appends into. Two
    /// overlapped calls allocating into one region would both mutate that
    /// region's allocation list, so the region is a written footprint element
    /// of its own, with no actual to project onto.
    ///
    /// The other half of the arena boundary is **not** covered and must be
    /// before any arena program compiles. An `arena<'r, T>` is a
    /// [`CheckedType::Nominal`], not a variant [`Footprint`] derives a region
    /// from, so an `own arena<'r, T>` parameter carries no mode region and no
    /// slice state: a callee row that declares `writes(arena)` projects nothing
    /// onto it, and only the handle's consumed place is recorded. Every arena
    /// program stops today at `UnsupportedSemanticFeature::ArenaRuntime`, so
    /// nothing reaches this gap; the arena lane must close it rather than
    /// inherit the projection.
    Arena {
        region: DeclarationId,
        call: NodePath,
    },
}

/// One [OWN-5] loan an argument borrow holds over a caller place for the
/// duration of its call [OWN-12].
///
/// A loan is not a use. The callee's declared row says what the callee *does*
/// through the borrow; the loan says what the borrow *forbids everyone else*
/// while it is live. The two are independent: `fn peek(c: &uniq 'c u64)
/// reads(cell)` projects a read and holds an exclusive loan, and a `pure`
/// callee projects nothing and still holds one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Loan {
    pub(crate) strength: LoanStrength,
    pub(crate) place: ResolvedPlace,
    /// The actual's source node, for citation.
    pub(crate) argument: NodePath,
}

/// The two borrow modes [OWN-2], as [OWN-5] grades their exclusion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoanStrength {
    Shared,
    Exclusive,
}

impl LoanStrength {
    const fn half(self) -> FootprintHalf {
        match self {
            Self::Shared => FootprintHalf::SharedLoan,
            Self::Exclusive => FootprintHalf::ExclusiveLoan,
        }
    }

    /// [OWN-5]'s matrix, read as this loan against one overlapping use of the
    /// other statement. This is `check_loan_access`'s loan/access table, the
    /// same word for the same thing: an exclusive loan excludes every access,
    /// a shared loan excludes writes and admits reads.
    const fn excludes_use(self, use_half: FootprintHalf) -> bool {
        match self {
            Self::Exclusive => true,
            Self::Shared => matches!(use_half, FootprintHalf::Write),
        }
    }

    /// The same matrix against the other statement's loan: two loans
    /// conflict exactly when at least one is exclusive [OWN-5, OWN-12].
    const fn excludes_loan(self, other: Self) -> bool {
        matches!(self, Self::Exclusive) || matches!(other, Self::Exclusive)
    }
}

impl Access {
    fn conflicts(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Place { place: left, .. }, Self::Place { place: right, .. }) => {
                left.overlaps(right)
            }
            (Self::Arena { region: left, .. }, Self::Arena { region: right, .. }) => left == right,
            (Self::Place { .. }, Self::Arena { .. }) | (Self::Arena { .. }, Self::Place { .. }) => {
                false
            }
        }
    }
}

/// Which two footprint halves a condition-2 conflict joins. The ledger states
/// it, so a denial names the access it actually found rather than calling
/// every conflict a write/write one.
///
/// The halves are named from the *earlier* and *later* statement of the two
/// the conflict joins, which the denial carries alongside as a [`PairSide`]
/// each: for the judged pair those are s1 and s2, and for an interposed
/// statement they are that statement and whichever member it was judged
/// against.
/// One half of a statement's [OWN-5] access set.
///
/// The first three are *uses*: what the callee's declared row does through an
/// actual, and what the caller's own operand evaluation touches. The last two
/// are *loans*: what an argument borrow forbids everyone else for the
/// duration of the call [OWN-12], whatever its callee's row does or does not
/// declare. A row-less `&uniq` argument is the pointed case — it reads
/// nothing, writes nothing, and still excludes every overlapping access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FootprintHalf {
    Write,
    Read,
    OperandRead,
    SharedLoan,
    ExclusiveLoan,
}

impl FootprintHalf {
    const fn name(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Read => "read",
            Self::OperandRead => "operand read",
            Self::SharedLoan => "shared loan",
            Self::ExclusiveLoan => "exclusive loan",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConflictKind {
    pub(crate) earlier: FootprintHalf,
    pub(crate) later: FootprintHalf,
}

impl ConflictKind {
    const fn new(earlier: FootprintHalf, later: FootprintHalf) -> Self {
        Self { earlier, later }
    }

    /// The two footprint halves this conflict joins, earlier first, as the
    /// ledger names them.
    pub(crate) const fn halves(self) -> (&'static str, &'static str) {
        (self.earlier.name(), self.later.name())
    }
}

/// Why P does not hold for one analyzed window. Each variant names exactly one
/// condition of the judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Denial {
    /// Condition 1: the operands of one window statement read a binding an
    /// earlier one defines.
    Dataflow {
        binding: BindingId,
        /// The statement that defines the binding.
        definer: PairSide,
        /// The statement whose operands read it.
        reader: PairSide,
    },
    /// Condition 2: two accesses of two window footprints conflict.
    Footprint {
        kind: ConflictKind,
        left: Access,
        right: Access,
        /// The two statements the accesses belong to, earlier first.
        sides: (PairSide, PairSide),
    },
    /// Condition 2: one statement's [OWN-5] loan excludes an overlapping
    /// loan or use of the other. Carried apart from `Footprint` only because
    /// a loan cites its borrow's actual rather than a row entry; it is the
    /// same condition and the ledger renders it the same way.
    Loan {
        kind: ConflictKind,
        left: NodePath,
        right: NodePath,
        sides: (PairSide, PairSide),
    },
    /// Condition 2, fail-closed: the row projects an access through an actual
    /// whose caller place this analysis cannot resolve, or an operand reads
    /// storage it cannot resolve.
    UnresolvedFootprint { side: PairSide, argument: NodePath },
    /// Condition 2, fail-closed: a statement written between s1 and s2 has a
    /// form whose footprint this analysis does not compute. [PAR-1] says an
    /// unresolved element overlaps every place, so such a statement denies
    /// rather than ending the enumeration silently — which is the whole point
    /// of judging a window instead of an adjacent pair.
    InterposedForm {
        side: PairSide,
        /// The form, as the ledger names it to the writer.
        form: &'static str,
    },
    /// Condition 4: an exit edge of s1, or of a statement between the two,
    /// does not reach s2.
    SkippingExit { side: PairSide, kind: ExitKind },
}

impl Denial {
    /// The judgment condition this denial cites. The permission ledger prints
    /// it and the judgment tests assert it; acceptance never reads it.
    pub(crate) const fn condition(&self) -> u8 {
        match self {
            Self::Dataflow { .. } => 1,
            Self::Footprint { .. }
            | Self::Loan { .. }
            | Self::UnresolvedFootprint { .. }
            | Self::InterposedForm { .. } => 2,
            Self::SkippingExit { .. } => 4,
        }
    }
}

/// The judgment's outcome for one analyzed pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PermissionVerdict {
    /// P holds, so the window may be overlapped. Source proof statements were
    /// already checked and erase before this permission can be actualized.
    PermittedEligible,
    Denied(Denial),
}

impl PermissionVerdict {
    pub(crate) const fn is_eligible(&self) -> bool {
        matches!(self, Self::PermittedEligible)
    }

    /// The cited condition of a denial, or `None` for a permitted verdict.
    #[allow(dead_code)]
    pub(crate) const fn denied_condition(&self) -> Option<u8> {
        match self {
            Self::Denied(denial) => Some(denial.condition()),
            Self::PermittedEligible => None,
        }
    }
}

/// One analyzed call statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionSite {
    /// The statement node that owns the call: the `let_stmt` node, or the call
    /// occurrence where the statement form carries no node of its own.
    pub(crate) statement: NodePath,
    /// The binding the statement defines, or `None` for a call whose result
    /// its own statement consumes. A join must complete before the first use
    /// of it.
    pub(crate) binding: Option<BindingId>,
    /// The call occurrence inside it. This is the site's identity: it exists
    /// in every written call position, where a defining binding does not.
    pub(crate) call: NodePath,
    pub(crate) callee_name: String,
    /// Compiler-owned execution summary selected for this call. The
    /// permission judgment does not use it as an alias fact.
    pub(crate) target_action: TargetAction,
}

/// One ordered pair of adjacent call statements and its verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionPair {
    pub(crate) first: PermissionSite,
    pub(crate) second: PermissionSite,
    pub(crate) verdict: PermissionVerdict,
}

/// A maximal chain of at least two adjacent call statements every ordered
/// pair of which is permitted and eligible. A chain is not implied by its
/// adjacent pairs, so every ordered pair inside it is judged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionRun {
    pub(crate) sites: Vec<PermissionSite>,
}

/// One source-ordered call step in a dependency-driven completion schedule.
///
/// The sites in one schedule are consecutive call statements. `wait_for` names
/// only earlier calls whose ordinary value, memory access, or loan conflicts
/// with this call, by their call occurrences.  It contains no resource family
/// or target operation identity; lowering later decides which direct
/// may-suspend calls have an executable completion route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionCompletionStep {
    pub(crate) site: PermissionSite,
    pub(crate) wait_for: Vec<NodePath>,
    /// At least one immediately following call may run before this call's
    /// result and loans return. The final call of a schedule is false.
    pub(crate) has_later_independent_call: bool,
}

/// Every analyzed pair and eligible chain of one concrete function, in source
/// order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionPermissions {
    pub(crate) function: String,
    pub(crate) pairs: Vec<PermissionPair>,
    pub(crate) runs: Vec<PermissionRun>,
    /// Consecutive call schedules whose waits are ordinary dependency edges.
    /// This table is intentionally absent from the developer pair ledger: it
    /// is a lowering view of the same verdicts, not another permission rule.
    pub(crate) completion_steps: Vec<PermissionCompletionStep>,
    /// The [PAR-2] verdict of every counted loop of this function, in source
    /// order. The pair judgment above is computed exactly as it was before
    /// these existed, and nothing here is lowered by this version.
    pub(crate) loops: Vec<LoopPermission>,
    /// The [PAR-3] staged verdict of every loop of this function whose body
    /// performs I/O, in source order. It is a second judgment of the same
    /// bodies and shares none of the counted permission's apparatus: no
    /// accumulator, no combination tree, no index range. A loop with no
    /// `may-suspend` action has no entry.
    pub(crate) staged: Vec<StagedPermission>,
}

/// The whole-program permission table, dense by [`FunctionId`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PermissionMetadata {
    pub(crate) functions: Vec<FunctionPermissions>,
}

impl PermissionMetadata {
    /// The table of one concrete function by its dense identity. A function
    /// the judgment never reached has no analyzed pair and therefore no
    /// entry, which reads the same as an empty one.
    pub(crate) fn of(&self, function: FunctionId) -> Option<&FunctionPermissions> {
        self.functions.get(function.0 as usize)
    }

    /// The table of one concrete function by its source name. Ledger and test
    /// convenience; the dense index is the identity.
    #[allow(dead_code)]
    pub(crate) fn named(&self, name: &str) -> Option<&FunctionPermissions> {
        self.functions
            .iter()
            .find(|entry| entry.function.as_str() == name)
    }
}

/// Runs P over every concrete function of one checked program.
///
/// `signatures` is dense by [`FunctionId`] and carries only callable-boundary
/// data. Nothing in this call graph, statement walk, or place resolution reads
/// a derived fact.
pub(crate) fn analyze_permission(
    functions: &[CheckedFunction],
    signatures: &[PermissionSignature],
) -> PermissionMetadata {
    let program = Program {
        functions,
        signatures,
    };
    PermissionMetadata {
        functions: functions
            .iter()
            .map(|function| program.analyze_function(function))
            .collect(),
    }
}

pub(super) struct Program<'check> {
    functions: &'check [CheckedFunction],
    signatures: &'check [PermissionSignature],
}

/// One candidate statement: a statement whose call position holds exactly one
/// named-function call.
///
/// A call reaches this analysis in two written positions: as the whole
/// right-hand side of a `let`, and as the scrutinee of a `match`. The two are
/// the same call and get the same judgment; what differs is who reads the
/// result. A `let` names it, so nothing reads it until a later statement does.
/// A scrutinee is read by its own statement's dispatch, so every statement
/// after that one already stands behind a read of it — which is what
/// `result_read_by_own_statement` records and what
/// [`Program::judge`] turns into the window that refuses the pair.
struct Candidate<'check> {
    /// The statement node that owns the call. A `match` statement carries no
    /// node of its own in the checked model, so a scrutinee candidate names
    /// its call occurrence here; both are inside the same statement and both
    /// sort, locate, and enclose identically.
    statement: NodePath,
    /// Position of this statement in its own block. Two candidates and the
    /// statements between their positions are one judged window.
    index: usize,
    /// The binding the statement defines, or `None` where the call's result is
    /// consumed by the statement itself and never named.
    binding: Option<BindingId>,
    call: CallProjection<'check>,
    /// The statement's own remainder reads this call's result — a scrutinee
    /// dispatch and the arms it selects. The result is therefore live before
    /// any later statement runs.
    result_read_by_own_statement: bool,
    /// An exit edge of this statement that does not reach its successor.
    exit: Option<ExitKind>,
}

/// One call occurrence, reduced to what the [EFF-2] boundary projection reads.
///
/// The window judgment reaches a call as the whole right-hand side of a
/// `let_stmt`; the loop judgment reaches one wherever it is written in a body.
/// Both project the same boundary, so both build it from this and neither
/// grows a second copy of the projection.
pub(super) struct CallProjection<'check> {
    pub(super) call: &'check NodePath,
    pub(super) target: CallTarget,
    pub(super) arguments: &'check [CheckedExpression],
    pub(super) argument_nodes: &'check [NodePath],
    pub(super) regions: &'check [DeclarationId],
}

/// The closed callable classes the overlap judgment can project.
#[derive(Clone, Copy)]
pub(super) enum CallTarget {
    User(FunctionId),
    System(u8),
}

/// The call one expression is, or `None` for every other expression form.
pub(super) fn call_projection(value: &CheckedExpression) -> Option<CallProjection<'_>> {
    match value {
        CheckedExpression::UserCall {
            function,
            call,
            argument_nodes,
            arguments,
            goal_regions,
            ..
        } => Some(CallProjection {
            call,
            target: CallTarget::User(*function),
            arguments,
            argument_nodes,
            regions: goal_regions,
        }),
        CheckedExpression::SystemCall {
            operation,
            call,
            regions,
            argument_nodes,
            arguments,
            ..
        } => Some(CallProjection {
            call,
            target: CallTarget::System(*operation),
            arguments,
            argument_nodes,
            regions,
        }),
        _ => None,
    }
}

/// One statement written between the two judged calls, reduced to what the
/// window rule asks of it.
struct Interposed {
    /// The binding it defines, when it defines one.
    defines: Option<BindingId>,
    /// Every binding its own operands mention.
    uses: Vec<BindingId>,
    footprint: Footprint,
}

/// Why one interposed statement cannot be judged as written.
enum InterposedRefusal {
    /// It carries an exit edge — condition 4.
    Exit(ExitKind),
    /// Its footprint is not computed for this form — condition 2.
    Form(&'static str),
}

/// One block, prepared once for every window judged inside it.
///
/// Every statement is classified and every candidate's footprint projected a
/// single time per block, not once per judged window. A block with m calls
/// among n statements is judged over O(m²) ordered pairs by [`collect_runs`],
/// and each window spans up to n statements, so classifying inside `judge`
/// made the analysis O(m²·n) in place resolution and allocation. On the
/// frozen real-source fixtures — whose entry blocks carry tens of calls — that
/// was the difference between a suite that finishes and one that does not.
struct BlockWindows<'check> {
    /// Every statement of the block, in source order, classified as an
    /// interposed member. Candidate statements are classified too: they are
    /// interposed members of the non-adjacent windows [`collect_runs`] judges.
    statements: Vec<Result<Interposed, InterposedRefusal>>,
    /// The call candidates of the block, in source order.
    candidates: Vec<Candidate<'check>>,
    /// Each candidate's [EFF-2] projection, positionally with `candidates`.
    footprints: Vec<Footprint>,
}

impl<'check> Program<'check> {
    fn analyze_function(&self, function: &'check CheckedFunction) -> FunctionPermissions {
        let places = PlaceMap::for_function(function);
        let mut permissions = FunctionPermissions {
            function: function.name.clone(),
            pairs: Vec::new(),
            runs: Vec::new(),
            completion_steps: Vec::new(),
            loops: Vec::new(),
            staged: Vec::new(),
        };
        let mut blocks = vec![function.body.as_slice()];
        while let Some(block) = blocks.pop() {
            self.analyze_block(&places, block, &mut permissions);
            for statement in block {
                push_nested_blocks(statement, &mut blocks);
            }
        }
        permissions.pairs.sort_by(|left, right| {
            left.first
                .statement
                .components()
                .cmp(right.first.statement.components())
        });
        permissions.runs.sort_by(|left, right| {
            left.sites[0]
                .statement
                .components()
                .cmp(right.sites[0].statement.components())
        });
        permissions.completion_steps.sort_by(|left, right| {
            left.site
                .statement
                .components()
                .cmp(right.site.statement.components())
        });
        // The loop judgment runs last and reads the finished verdicts, so a
        // loop that already holds an eligible pair is never told to become
        // one. Its own verdict does not read them: [PAR-2] is a judgment of
        // the loop, not of what a writer could put inside it.
        let eligible = permissions
            .pairs
            .iter()
            .filter(|pair| pair.verdict.is_eligible())
            .map(|pair| pair.first.statement.clone())
            .collect::<Vec<_>>();
        permissions.loops = super::loop_permission::judge_loops(self, &places, function, &eligible);
        // The staged judgment is its own rule over the same bodies. It reads
        // no [PAR-2] verdict and no pair verdict, so nothing here can move an
        // existing line of the table.
        permissions.staged = super::staged_permission::judge_staged(self, &places, function);
        permissions
    }

    /// Judges every adjacent pair of *calls* in one block, each over the
    /// window of statements written between them.
    ///
    /// The enumeration is over call candidates rather than over runs of
    /// adjacent candidate statements. A statement of any other form no longer
    /// ends the enumeration; it becomes an interposed member of the window and
    /// is judged, so a pair separated by one builtin gets a verdict and a
    /// ledger line where it previously got neither. The reported pairs stay
    /// adjacent-call pairs, so the ledger's volume is unchanged.
    fn analyze_block(
        &self,
        places: &PlaceMap,
        block: &'check [CheckedStatement],
        permissions: &mut FunctionPermissions,
    ) {
        let candidates = block
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| candidate_of(index, statement))
            .collect::<Vec<_>>();
        if candidates.len() < 2 {
            return;
        }
        let windows = BlockWindows {
            statements: block
                .iter()
                .enumerate()
                .map(|(index, statement)| self.interposed_of(places, index, statement))
                .collect(),
            footprints: candidates
                .iter()
                .map(|candidate| self.footprint(places, &candidate.call))
                .collect(),
            candidates,
        };
        for ordinal in 0..windows.candidates.len() - 1 {
            let verdict = self.judge(&windows, ordinal, ordinal + 1);
            permissions.pairs.push(PermissionPair {
                first: self.site(&windows.candidates[ordinal]),
                second: self.site(&windows.candidates[ordinal + 1]),
                verdict,
            });
        }
        self.collect_runs(&windows, permissions);
        self.collect_completion_steps(&windows, permissions);
    }

    /// Builds maximal schedules of consecutive plain call statements.
    ///
    /// Adjacent eligibility is enough to keep the writer moving from one site
    /// to the next. Every later site then records all earlier members whose
    /// ordinary pair verdict is denied. For `A(out), B(err), C(out)`, this
    /// yields no wait before B and exactly A before C.
    fn collect_completion_steps(
        &self,
        windows: &BlockWindows<'check>,
        permissions: &mut FunctionPermissions,
    ) {
        let candidates = &windows.candidates;
        let mut start = 0;
        while start + 1 < candidates.len() {
            let first = &candidates[start];
            let second = &candidates[start + 1];
            if first.exit.is_some()
                || second.exit.is_some()
                || first.index + 1 != second.index
                || !self.judge(windows, start, start + 1).is_eligible()
            {
                start += 1;
                continue;
            }

            let mut end = start + 1;
            while end + 1 < candidates.len()
                && candidates[end].exit.is_none()
                && candidates[end + 1].exit.is_none()
                && candidates[end].index + 1 == candidates[end + 1].index
                && self.judge(windows, end, end + 1).is_eligible()
            {
                end += 1;
            }

            for current in start..=end {
                let wait_for = (start..current)
                    .filter(|earlier| !self.judge(windows, *earlier, current).is_eligible())
                    .map(|earlier| candidates[earlier].call.call.clone())
                    .collect();
                permissions.completion_steps.push(PermissionCompletionStep {
                    site: self.site(&candidates[current]),
                    wait_for,
                    has_later_independent_call: current < end,
                });
            }
            start = end + 1;
        }
    }

    /// Grows maximal chains whose every ordered pair is permitted and
    /// eligible.
    ///
    /// Every ordered pair is judged over its own window, so a statement
    /// between two chain members is judged against every member of the chain:
    /// for members mi and mj bracketing an interposed T, the pair (mi, mj)
    /// covers T against both of them, and for any further member mk the pair
    /// that brackets both T and mk covers T against mk. A chain requires all
    /// its ordered pairs, so nothing in the window escapes judgment — which is
    /// [PAR-1]'s "permission for a chain is exactly permission for every
    /// ordered pair it contains", read over windows.
    fn collect_runs(&self, windows: &BlockWindows<'check>, permissions: &mut FunctionPermissions) {
        let group = &windows.candidates;
        let mut start = 0;
        while start + 1 < group.len() {
            let mut end = start;
            while end + 1 < group.len()
                && (start..=end).all(|earlier| self.judge(windows, earlier, end + 1).is_eligible())
            {
                end += 1;
            }
            if end > start {
                permissions.runs.push(PermissionRun {
                    sites: group[start..=end]
                        .iter()
                        .map(|candidate| self.site(candidate))
                        .collect(),
                });
                start = end + 1;
            } else {
                start += 1;
            }
        }
    }

    fn site(&self, candidate: &Candidate<'check>) -> PermissionSite {
        let (callee_name, target_action) = match candidate.call.target {
            CallTarget::User(function) => self
                .functions
                .get(function.0 as usize)
                .map(|function| (function.name.clone(), function.target_action))
                .unwrap_or_else(|| (String::new(), TargetAction::CONSERVATIVE)),
            CallTarget::System(operation) => SYSTEM_OPERATIONS
                .get(usize::from(operation))
                .map(|row| (row.spelling.to_owned(), row.target_action))
                .unwrap_or_else(|| (String::new(), TargetAction::CONSERVATIVE)),
        };
        PermissionSite {
            statement: candidate.statement.clone(),
            binding: candidate.binding,
            call: candidate.call.call.clone(),
            callee_name,
            target_action,
        }
    }

    /// The four conditions in their numbered order, then eligibility, over the
    /// window (s1, T1…Tk, s2).
    ///
    /// An interposed statement is classified before any condition is
    /// evaluated, because a form whose footprint this analysis does not
    /// compute has no condition-1 or condition-2 answer to give: it denies on
    /// the spot, citing the condition its form violates. A window with several
    /// defects therefore reports the interposed form ahead of a lower-numbered
    /// defect elsewhere, which is the honest report — nothing else about that
    /// statement is known.
    fn judge(
        &self,
        windows: &BlockWindows<'check>,
        first_ordinal: usize,
        second_ordinal: usize,
    ) -> PermissionVerdict {
        let first = &windows.candidates[first_ordinal];
        let second = &windows.candidates[second_ordinal];
        // Where s1's own statement reads s1's result, the rest of that
        // statement — the dispatch and the arm it selects — runs between the
        // call and everything after it, so the statement is itself the
        // window's first interposed member. It is classified exactly as any
        // other statement of its form is, which is what makes a scrutinee
        // candidate deny as a first member without a rule of its own.
        let window_start = first.index + usize::from(!first.result_read_by_own_statement);
        let mut interposed = Vec::with_capacity(second.index - window_start);
        for (offset, classified) in windows
            .statements
            .get(window_start..second.index)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let side = PairSide::Between(offset);
            match classified {
                Ok(record) => interposed.push(record),
                Err(InterposedRefusal::Exit(kind)) => {
                    return PermissionVerdict::Denied(Denial::SkippingExit { side, kind: *kind });
                }
                Err(InterposedRefusal::Form(form)) => {
                    return PermissionVerdict::Denied(Denial::InterposedForm { side, form });
                }
            }
        }

        // Condition 1: ordinary def-use, over the whole window.
        let mut used = Vec::new();
        for argument in second.call.arguments {
            collect_used_bindings(argument, &mut used);
        }
        if let Some(defined) = first.binding
            && used.contains(&defined)
        {
            return PermissionVerdict::Denied(Denial::Dataflow {
                binding: defined,
                definer: PairSide::First,
                reader: PairSide::Second,
            });
        }
        for (offset, record) in interposed.iter().enumerate() {
            // Where s1 takes the lane its value does not exist until the join,
            // so nothing between them may read it.
            if let Some(defined) = first.binding
                && record.uses.contains(&defined)
            {
                return PermissionVerdict::Denied(Denial::Dataflow {
                    binding: defined,
                    definer: PairSide::First,
                    reader: PairSide::Between(offset),
                });
            }
            // Where s2 takes the lane its operands are evaluated before the
            // interposed statements run, so it may not read what they define.
            if let Some(defined) = record.defines
                && used.contains(&defined)
            {
                return PermissionVerdict::Denied(Denial::Dataflow {
                    binding: defined,
                    definer: PairSide::Between(offset),
                    reader: PairSide::Second,
                });
            }
        }

        // Condition 2: disjoint footprints under OWN-7, fail closed.
        let left = &windows.footprints[first_ordinal];
        let right = &windows.footprints[second_ordinal];
        for (side, footprint) in [(PairSide::First, left), (PairSide::Second, right)]
            .into_iter()
            .chain(
                interposed
                    .iter()
                    .enumerate()
                    .map(|(offset, record)| (PairSide::Between(offset), &record.footprint)),
            )
        {
            // Operand evaluation is part of the statement, so an overlap moves
            // it too. Which statement's operands move depends on which member
            // takes the lane, so an unresolved operand read anywhere in the
            // window denies just as an unresolved row projection does.
            if let Some(argument) = footprint
                .unresolved
                .clone()
                .or_else(|| footprint.operand_unresolved.clone())
            {
                return PermissionVerdict::Denied(Denial::UnresolvedFootprint { side, argument });
            }
        }
        if let Some(denial) =
            footprint_conflict(left, PairSide::First, right, PairSide::Second, true)
        {
            return PermissionVerdict::Denied(denial);
        }
        for (offset, record) in interposed.iter().enumerate() {
            let side = PairSide::Between(offset);
            // Against s1 the mirror operand obligation is dropped: the
            // schedule that hands s1 out evaluates O(s1) before the fork, and
            // the schedule that hands s2 out has already completed s1, so no
            // interposed write can reach O(s1). The operand half is one-sided
            // for s1 and two-sided for s2, and that asymmetry is the whole
            // difference between this rule and judging T as an ordinary member.
            if let Some(denial) =
                footprint_conflict(left, PairSide::First, &record.footprint, side, false)
            {
                return PermissionVerdict::Denied(denial);
            }
            if let Some(denial) =
                footprint_conflict(&record.footprint, side, right, PairSide::Second, true)
            {
                return PermissionVerdict::Denied(denial);
            }
        }

        // Condition 4: no exit edge of s1 bypasses s2. An exit between them
        // denied during classification above.
        if let Some(kind) = first.exit {
            return PermissionVerdict::Denied(Denial::SkippingExit {
                side: PairSide::First,
                kind,
            });
        }

        // All four conditions hold. Source proof statements were checked
        // before this analysis and have no runtime exit or footprint.
        PermissionVerdict::PermittedEligible
    }

    /// One statement between the two members, reduced to what the window rule
    /// judges, or the reason it cannot be.
    ///
    /// The match is exhaustive on purpose: a statement form this analysis does
    /// not classify would otherwise contribute an empty footprint and *widen*
    /// permission, which is the one direction the judgment must never fail in.
    /// Every form is either given a footprint here or refused here.
    fn interposed_of(
        &self,
        places: &PlaceMap,
        index: usize,
        statement: &'check CheckedStatement,
    ) -> Result<Interposed, InterposedRefusal> {
        match statement {
            CheckedStatement::Proof(_) => Ok(Interposed {
                defines: None,
                uses: Vec::new(),
                footprint: Footprint::default(),
            }),
            CheckedStatement::Let {
                node_path,
                binding,
                value,
            } => {
                // A call between the members is judged exactly as a member is,
                // with its full [EFF-2] projection rather than the operand
                // reads of an ordinary value.
                if let Some(candidate) = candidate_of(index, statement) {
                    let mut uses = Vec::new();
                    for argument in candidate.call.arguments {
                        collect_used_bindings(argument, &mut uses);
                    }
                    return Ok(Interposed {
                        defines: Some(*binding),
                        uses,
                        footprint: self.footprint(places, &candidate.call),
                    });
                }
                // A written borrow's shared-or-uniq mode is erased from the
                // checked expression, so the [OWN-5] loan this statement
                // would hold across the window cannot be formed here. Refusal
                // is the fail-closed direction: an unloaned borrow would
                // contribute an empty footprint and widen permission.
                if expression_forms_borrow(value) {
                    return Err(InterposedRefusal::Form("a statement that forms a borrow"));
                }
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                Ok(Interposed {
                    defines: Some(*binding),
                    uses,
                    footprint: value_footprint(places, value, node_path),
                })
            }
            // [CALL-4] a binder or target list defines more than one place in
            // one statement, and this window admits exactly one definition
            // per statement. Refusal is the fail-closed direction: nothing
            // here widens a permission it cannot describe.
            CheckedStatement::DestructuringLet { .. } | CheckedStatement::SetList { .. } => Err(
                InterposedRefusal::Form("a statement that binds an ordered result list"),
            ),
            CheckedStatement::Set {
                node_path,
                target,
                value,
            } => {
                if expression_forms_borrow(value) {
                    return Err(InterposedRefusal::Form("a statement that forms a borrow"));
                }
                let mut footprint = value_footprint(places, value, node_path);
                set_target_place(places, target, node_path, &mut footprint, false);
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                collect_set_target_bindings(target, &mut uses);
                Ok(Interposed {
                    defines: None,
                    uses,
                    footprint,
                })
            }
            // [SET-2]: one read of the previous value into the fresh binding
            // and one write of the replacement into the target, so the target
            // place is both halves of the footprint.
            CheckedStatement::Replace {
                node_path,
                binding,
                target,
                value,
            } => {
                // Dead today: [SET-2] makes a region-bearing target type a
                // hard error, and a borrow type is region-bearing, so a
                // replace value can never be a borrow. The guard stands so
                // the window invariant — every loan live inside a permitted
                // window is one an argument of a judged call holds — rests
                // on written checks in all three admitted forms rather than
                // on that rule staying put.
                if expression_forms_borrow(value) {
                    return Err(InterposedRefusal::Form("a statement that forms a borrow"));
                }
                let mut footprint = value_footprint(places, value, node_path);
                set_target_place(places, target, node_path, &mut footprint, true);
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                collect_set_target_bindings(target, &mut uses);
                Ok(Interposed {
                    defines: Some(*binding),
                    uses,
                    footprint,
                })
            }
            // Exit-bearing forms: condition 4.
            CheckedStatement::PropagateLet { .. } => {
                Err(InterposedRefusal::Exit(ExitKind::PropagateError))
            }
            CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => Err(InterposedRefusal::Exit(ExitKind::BlockExit)),
            // An expression statement is a call [GRAM-4], which may be a
            // system call whose reach no row projects, and a discarded one
            // carries its own [STOR-3] release. Admitting these needs that
            // release classified first, so today they deny.
            CheckedStatement::Evaluate(_) => {
                Err(InterposedRefusal::Form("an expression statement"))
            }
            CheckedStatement::DropExpression { .. } => {
                Err(InterposedRefusal::Form("a discarded expression statement"))
            }
            // [PROV-6] `dispose p;` runs a release walk of its own, which is
            // the same classification an interposed drop needs and does not
            // have yet.
            CheckedStatement::Dispose { .. } => Err(InterposedRefusal::Form("a dispose statement")),
            // Forms carrying their own control flow and their own drops. The
            // lowering already refuses them by splitting the block, so the
            // checker refusing them keeps the two in agreement.
            CheckedStatement::Match { .. } => Err(InterposedRefusal::Form("a match statement")),
            CheckedStatement::ValueMatchLet { .. } => {
                Err(InterposedRefusal::Form("a value match or value if"))
            }
            CheckedStatement::Loop { .. } => Err(InterposedRefusal::Form("a loop")),
            CheckedStatement::CountedRange { .. } => Err(InterposedRefusal::Form("a for loop")),
            CheckedStatement::Region { .. } => Err(InterposedRefusal::Form("a region")),
        }
    }

    /// The written and read footprints of one call, by [EFF-2] boundary
    /// projection onto the actuals' resolved places.
    pub(super) fn footprint(&self, places: &PlaceMap, candidate: &CallProjection<'_>) -> Footprint {
        match candidate.target {
            CallTarget::User(callee) => self.user_call_footprint(places, candidate, callee),
            CallTarget::System(operation) => {
                self.system_call_footprint(places, candidate, operation)
            }
        }
    }

    fn user_call_footprint(
        &self,
        places: &PlaceMap,
        candidate: &CallProjection<'_>,
        callee_id: FunctionId,
    ) -> Footprint {
        let mut footprint = Footprint::default();
        let (Some(signature), Some(callee)) = (
            self.signatures.get(callee_id.0 as usize),
            self.functions.get(callee_id.0 as usize),
        ) else {
            footprint.unresolved = Some(candidate.call.clone());
            return footprint;
        };

        // An `allocates(arena 'r)` row appends to the caller region's
        // allocation list, which is written storage with no actual of its own.
        for formal in &signature.allocates_arenas {
            match signature
                .region_parameters
                .iter()
                .position(|region| region == formal)
                .and_then(|index| candidate.regions.get(index))
            {
                Some(region) => footprint.writes.push(Access::Arena {
                    region: *region,
                    call: candidate.call.clone(),
                }),
                None => footprint.unresolved = Some(candidate.call.clone()),
            }
        }

        for (index, parameter) in callee.parameters.iter().enumerate() {
            let Some(argument) = candidate.arguments.get(index) else {
                footprint.unresolved = Some(candidate.call.clone());
                return footprint;
            };
            let node = candidate
                .argument_nodes
                .get(index)
                .unwrap_or(candidate.call);
            // The loans half [OWN-5, OWN-12]. A borrow-mode parameter's
            // actual is an argument borrow live for the whole call, so it
            // holds a loan on its resolved place whatever the row declares. A
            // slice parameter is deliberately not read here: a slice's shared
            // loan belongs to the named data region its `slice_of`
            // established, not to the statement that passes the descriptor,
            // and the borrow checker holds it for that whole region already.
            let strength = match parameter.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(_) => Some(LoanStrength::Shared),
                CheckedMode::Unique(_) => Some(LoanStrength::Exclusive),
            };
            if let Some(strength) = strength {
                match argument_place(places, argument) {
                    Some(place) => footprint.loans.push(Loan {
                        strength,
                        place,
                        argument: node.clone(),
                    }),
                    None => footprint.unresolved = Some(node.clone()),
                }
            }
            // A consumed `own` actual transfers caller storage into the
            // callee. The affine discipline already forbids two consumers of
            // one place; the footprint states it rather than assuming it.
            if parameter.mode == CheckedMode::Own
                && let Some(place) = consumed_place(places, argument)
            {
                footprint.writes.push(Access::Place {
                    place,
                    argument: node.clone(),
                });
            }
        }

        for (written, declared) in [(false, &signature.reads), (true, &signature.writes)] {
            for path in declared {
                let Some(index) = callee
                    .parameters
                    .iter()
                    .position(|parameter| parameter.declaration == path.root)
                else {
                    footprint.unresolved = Some(candidate.call.clone());
                    continue;
                };
                let (Some(argument), Some(node)) = (
                    candidate.arguments.get(index),
                    candidate.argument_nodes.get(index),
                ) else {
                    footprint.unresolved = Some(candidate.call.clone());
                    continue;
                };
                match argument_place(places, argument).or_else(|| consumed_place(places, argument))
                {
                    Some(mut place) => {
                        place.extend_fields(&path.fields);
                        let access = Access::Place {
                            place,
                            argument: node.clone(),
                        };
                        if written {
                            footprint.writes.push(access);
                        } else {
                            footprint.reads.push(access);
                        }
                    }
                    None => footprint.unresolved = Some(node.clone()),
                }
            }
        }

        // The caller-side half: what this statement's own operand evaluation
        // touches before the call. An overlap moves it across the earlier
        // call, so it is part of the footprint even though no row mentions it.
        for (index, argument) in candidate.arguments.iter().enumerate() {
            let node = candidate
                .argument_nodes
                .get(index)
                .unwrap_or(candidate.call);
            collect_operand_reads(places, argument, node, &mut footprint);
        }
        footprint
    }

    /// Projects one direct system call through the same caller-place model as
    /// a user call. The catalog's ordinary parameter paths and borrow modes
    /// are complete: there is no synthetic global-world access.
    fn system_call_footprint(
        &self,
        places: &PlaceMap,
        candidate: &CallProjection<'_>,
        operation_index: u8,
    ) -> Footprint {
        let mut footprint = Footprint::default();
        let Some(operation) = SYSTEM_OPERATIONS.get(usize::from(operation_index)) else {
            footprint.unresolved = Some(candidate.call.clone());
            return footprint;
        };
        if operation.regions.len() != candidate.regions.len()
            || operation.parameters.len() != candidate.arguments.len()
            || operation.parameters.len() != candidate.argument_nodes.len()
        {
            footprint.unresolved = Some(candidate.call.clone());
            return footprint;
        }
        let (reads, writes) = operation_state_effects(operation);

        for (index, parameter) in operation.parameters.iter().enumerate() {
            let Some(argument) = candidate.arguments.get(index) else {
                footprint.unresolved = Some(candidate.call.clone());
                return footprint;
            };
            let Some(node) = candidate.argument_nodes.get(index) else {
                footprint.unresolved = Some(candidate.call.clone());
                return footprint;
            };
            let strength = match parameter.mode {
                SystemParameterMode::Own => None,
                SystemParameterMode::Borrow(_) => Some(LoanStrength::Shared),
                SystemParameterMode::UniqueBorrow(_) => Some(LoanStrength::Exclusive),
            };

            if let Some(strength) = strength {
                match argument_place(places, argument) {
                    Some(place) => footprint.loans.push(Loan {
                        strength,
                        place,
                        argument: node.clone(),
                    }),
                    None => footprint.unresolved = Some(node.clone()),
                }
            }

            // A consumed `own` actual transfers caller storage into the
            // operation, and [PAR-1] puts the place it names in the written
            // footprint whatever the row says about it. This is the same
            // unconditional push `user_call_footprint` makes, and it is
            // unconditional for the same reason: reading the row instead would
            // leave a parameter the row mentions only under `reads` with no
            // written footprint element, so another statement could read the
            // consumed place alongside the consume.
            if matches!(parameter.mode, SystemParameterMode::Own)
                && let Some(place) = consumed_place(places, argument)
            {
                footprint.writes.push(Access::Place {
                    place,
                    argument: node.clone(),
                });
            }

            let Ok(ordinal) = u8::try_from(index) else {
                footprint.unresolved = Some(node.clone());
                continue;
            };
            let written = writes.contains(&ordinal);
            let read = reads.contains(&ordinal);
            if written || read {
                match argument_place(places, argument).or_else(|| consumed_place(places, argument))
                {
                    Some(place) => {
                        let access = Access::Place {
                            place,
                            argument: node.clone(),
                        };
                        if written {
                            footprint.writes.push(access.clone());
                        }
                        if read {
                            footprint.reads.push(access);
                        }
                    }
                    None => footprint.unresolved = Some(node.clone()),
                }
            }
        }

        for (argument, node) in candidate.arguments.iter().zip(candidate.argument_nodes) {
            collect_operand_reads(places, argument, node, &mut footprint);
        }
        footprint
    }

    /// Conservative target summary of one concrete callee.
    pub(super) fn target_action(&self, function: FunctionId) -> TargetAction {
        self.functions
            .get(function.0 as usize)
            .map(|function| function.target_action)
            .unwrap_or(TargetAction::CONSERVATIVE)
    }
}

#[derive(Debug, Default)]
pub(super) struct Footprint {
    pub(super) writes: Vec<Access>,
    pub(super) reads: Vec<Access>,
    /// The [OWN-5] loans this statement's argument borrows hold for the
    /// duration of its call, independent of what its callee's row declares.
    pub(super) loans: Vec<Loan>,
    /// Storage this statement's own operand expressions read on the calling
    /// thread, before the call. An overlap moves some statement's operand
    /// evaluation across another's call, and which one moves is the
    /// implementation's choice of lane, so this half is judged in both
    /// directions for the pair — and, for a statement between the two, against
    /// s2's writes in both directions but against s1's writes only one way.
    /// The module doc derives that asymmetry.
    pub(super) operand_reads: Vec<Access>,
    /// Set when the row projects an access this analysis cannot resolve to a
    /// caller place. Every such statement is denied.
    pub(super) unresolved: Option<NodePath>,
    /// Set when an operand expression reads storage this analysis cannot
    /// resolve to a caller place. Denies wherever this statement sits in the
    /// window.
    pub(super) operand_unresolved: Option<NodePath>,
}

/// The condition-2 obligations of one ordered pair of window footprints.
///
/// The earlier statement's writes are judged against every half of the later
/// one — its callee's reads and writes and its caller-side operand reads —
/// and the later one's writes against the earlier one's reads.
///
/// `earlier_operands` selects whether the later statement's writes are also
/// judged against the earlier one's *operand* reads. It holds for the judged
/// pair and for an interposed statement against s2, whose operand evaluation
/// the hand-out hoists above it. It does not hold for s1 against an interposed
/// statement: under the schedule that hands s1 out, O(s1) is evaluated before
/// the fork, and under the schedule that hands s2 out, s1 has already
/// completed — so nothing written between them can reach it. Dropping the
/// obligation there is the one place this rule is weaker than judging every
/// window statement as an ordinary member, and it is derived, not assumed.
fn footprint_conflict(
    earlier: &Footprint,
    earlier_side: PairSide,
    later: &Footprint,
    later_side: PairSide,
    earlier_operands: bool,
) -> Option<Denial> {
    if let Some(denial) = loan_conflict(earlier, earlier_side, later, later_side, earlier_operands)
    {
        return Some(denial);
    }
    for write in &earlier.writes {
        for (kind, access) in later
            .writes
            .iter()
            .map(|access| {
                (
                    ConflictKind::new(FootprintHalf::Write, FootprintHalf::Write),
                    access,
                )
            })
            .chain(later.reads.iter().map(|access| {
                (
                    ConflictKind::new(FootprintHalf::Write, FootprintHalf::Read),
                    access,
                )
            }))
            .chain(later.operand_reads.iter().map(|access| {
                (
                    ConflictKind::new(FootprintHalf::Write, FootprintHalf::OperandRead),
                    access,
                )
            }))
        {
            if write.conflicts(access) {
                return Some(Denial::Footprint {
                    kind,
                    left: write.clone(),
                    right: access.clone(),
                    sides: (earlier_side, later_side),
                });
            }
        }
    }
    for write in &later.writes {
        for (kind, read) in earlier
            .reads
            .iter()
            .map(|access| {
                (
                    ConflictKind::new(FootprintHalf::Read, FootprintHalf::Write),
                    access,
                )
            })
            .chain(
                earlier_operands
                    .then(|| {
                        earlier.operand_reads.iter().map(|access| {
                            (
                                ConflictKind::new(FootprintHalf::OperandRead, FootprintHalf::Write),
                                access,
                            )
                        })
                    })
                    .into_iter()
                    .flatten(),
            )
        {
            if write.conflicts(read) {
                return Some(Denial::Footprint {
                    kind,
                    left: read.clone(),
                    right: write.clone(),
                    sides: (earlier_side, later_side),
                });
            }
        }
    }
    None
}

/// The loans half of the same condition: each statement's [OWN-5] loans
/// against the other statement's loans and uses.
///
/// The matrix is [OWN-5]'s own, the one `check_loan_access` applies to a live
/// loan: an exclusive loan excludes every overlapping access and every
/// overlapping loan, a shared loan excludes overlapping writes and
/// overlapping exclusive loans, and two shared loans coexist. Both
/// directions are judged because an overlap may move either statement.
///
/// `earlier_operands` gates the earlier statement's operand reads exactly as
/// it does above, for the same derivation: no loan of the later statement
/// can reach an operand evaluation that either happened before the fork or
/// after the earlier statement completed.
fn loan_conflict(
    earlier: &Footprint,
    earlier_side: PairSide,
    later: &Footprint,
    later_side: PairSide,
    earlier_operands: bool,
) -> Option<Denial> {
    fn uses(
        footprint: &Footprint,
        operands: bool,
    ) -> Vec<(FootprintHalf, &ResolvedPlace, &NodePath)> {
        footprint
            .writes
            .iter()
            .map(|access| (FootprintHalf::Write, access))
            .chain(
                footprint
                    .reads
                    .iter()
                    .map(|access| (FootprintHalf::Read, access)),
            )
            .chain(
                operands
                    .then(|| {
                        footprint
                            .operand_reads
                            .iter()
                            .map(|access| (FootprintHalf::OperandRead, access))
                    })
                    .into_iter()
                    .flatten(),
            )
            .filter_map(|(half, access)| match access {
                Access::Place { place, argument } => Some((half, place, argument)),
                // An arena region is not a place a borrow can name.
                Access::Arena { .. } => None,
            })
            .collect::<Vec<_>>()
    }

    for loan in &earlier.loans {
        for other in &later.loans {
            if loan.strength.excludes_loan(other.strength) && loan.place.overlaps(&other.place) {
                return Some(Denial::Loan {
                    kind: ConflictKind::new(loan.strength.half(), other.strength.half()),
                    left: loan.argument.clone(),
                    right: other.argument.clone(),
                    sides: (earlier_side, later_side),
                });
            }
        }
        for (half, place, argument) in uses(later, true) {
            if loan.strength.excludes_use(half) && loan.place.overlaps(place) {
                return Some(Denial::Loan {
                    kind: ConflictKind::new(loan.strength.half(), half),
                    left: loan.argument.clone(),
                    right: argument.clone(),
                    sides: (earlier_side, later_side),
                });
            }
        }
    }
    for loan in &later.loans {
        for (half, place, argument) in uses(earlier, earlier_operands) {
            if loan.strength.excludes_use(half) && loan.place.overlaps(place) {
                return Some(Denial::Loan {
                    kind: ConflictKind::new(half, loan.strength.half()),
                    left: argument.clone(),
                    right: loan.argument.clone(),
                    sides: (earlier_side, later_side),
                });
            }
        }
    }
    None
}

/// The footprint of a window statement that is not a call: the places its
/// consumed `own` operands transfer away, and the places its operands read.
/// Whether an expression forms a borrow anywhere inside it.
///
/// The checked tree erases a written borrow's shared-or-uniq mode (the mode
/// lives only in `CheckedMode`, which a non-argument borrow never meets), so
/// a window statement that forms one cannot be given its [OWN-5] loan and is
/// refused instead. Call arguments never reach this: their loans key on the
/// parameter's mode.
pub(super) fn expression_forms_borrow(expression: &CheckedExpression) -> bool {
    matches!(
        expression,
        CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
    ) || expression_children(expression)
        .into_iter()
        .any(expression_forms_borrow)
}

fn value_footprint(places: &PlaceMap, value: &CheckedExpression, node: &NodePath) -> Footprint {
    let mut footprint = Footprint::default();
    collect_consumed_places(places, value, node, &mut footprint);
    collect_operand_reads(places, value, node, &mut footprint);
    footprint
}

/// Records the storage a `set` or `replace` target names, and the operands its
/// subscript reads. A `replace` also reads the target, which `reads_target`
/// selects [SET-2].
///
/// An element target resolves to the whole collection, because a resolved
/// place carries no index segment [ENT-2]. That is the fail-closed direction:
/// one element write conflicts with any access to the collection.
pub(super) fn set_target_place(
    places: &PlaceMap,
    target: &CheckedSetTarget,
    node: &NodePath,
    footprint: &mut Footprint,
    reads_target: bool,
) {
    let place = match target {
        CheckedSetTarget::Place(target) => rooted_place(places, target.binding, &target.fields),
        CheckedSetTarget::ArrayIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            rooted_place(places, target.binding, &target.fields)
        }
        CheckedSetTarget::BufferIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            rooted_place(places, target.root.binding, &target.root.fields)
        }
        CheckedSetTarget::RunIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            rooted_container_place(places, &target.root)
        }
        // [PAR-2] a view element store writes the origin, which the view's
        // descriptor place stands for here exactly as a buffer's does: a
        // resolved place carries no index segment, so one element write
        // conflicts with any access to the view.
        CheckedSetTarget::SliceIndex(target) => {
            collect_operand_reads(places, &target.offset, node, footprint);
            rooted_place(places, target.root.binding, &[])
        }
    };
    if reads_target {
        footprint.reads.push(Access::Place {
            place: place.clone(),
            argument: node.clone(),
        });
    }
    footprint.writes.push(Access::Place {
        place,
        argument: node.clone(),
    });
}

/// Every binding a `set` or `replace` target mentions: the root it writes
/// through, whose value the calling thread reads to reach the storage, and any
/// binding its subscript reads.
fn collect_set_target_bindings(target: &CheckedSetTarget, out: &mut Vec<BindingId>) {
    let root = target.binding();
    if !out.contains(&root) {
        out.push(root);
    }
    match target {
        CheckedSetTarget::Place(_) => {}
        CheckedSetTarget::ArrayIndex(target) => collect_used_bindings(&target.offset, out),
        CheckedSetTarget::BufferIndex(target) => collect_used_bindings(&target.offset, out),
        CheckedSetTarget::RunIndex(target) => collect_used_bindings(&target.offset, out),
        CheckedSetTarget::SliceIndex(target) => collect_used_bindings(&target.offset, out),
    }
}

/// Every caller place one expression transfers away by consuming an `own`
/// value.
///
/// A move is recorded as a *write* of the storage it empties, for the reason
/// the call-boundary projection records a consumed actual that way: the affine
/// discipline already forbids a second consumer, and the footprint states it
/// rather than assuming it.
pub(super) fn collect_consumed_places(
    places: &PlaceMap,
    expression: &CheckedExpression,
    node: &NodePath,
    footprint: &mut Footprint,
) {
    if let Some(place) = consumed_place(places, expression) {
        footprint.writes.push(Access::Place {
            place,
            argument: node.clone(),
        });
    }
    for child in expression_children(expression) {
        collect_consumed_places(places, child, node, footprint);
    }
}

/// One candidate statement, or `None` for every other statement shape.
/// The call one statement holds in call position, whatever the position is.
///
/// The enumeration is over written call positions, not over statement kinds
/// that happen to be convenient: a `match` scrutinee is the same call as a
/// `let` right-hand side, gets the same [EFF-2] projection, and is judged by
/// the same four conditions. What the position changes is one recorded fact —
/// whether the statement itself reads the result — and `judge` derives the
/// window from that fact rather than from the statement's spelling.
fn candidate_of(index: usize, statement: &CheckedStatement) -> Option<Candidate<'_>> {
    let (node_path, binding, value, read_by_own_statement, exit) = match statement {
        CheckedStatement::Let {
            node_path,
            binding,
            value,
        } => (Some(node_path), Some(*binding), value, false, None),
        CheckedStatement::PropagateLet {
            node_path,
            binding,
            scrutinee,
            ..
        } => (
            Some(node_path),
            Some(*binding),
            scrutinee,
            false,
            Some(ExitKind::PropagateError),
        ),
        // A scrutinee call. `Match` carries no statement node in the checked
        // model and `ValueMatchLet`'s binding names the match's result rather
        // than the call's, so neither supplies a defining binding here.
        CheckedStatement::Match { scrutinee, .. }
        | CheckedStatement::ValueMatchLet { scrutinee, .. } => (None, None, scrutinee, true, None),
        _ => return None,
    };
    let call = call_projection(value)?;
    Some(Candidate {
        statement: node_path.unwrap_or(call.call).clone(),
        index,
        binding,
        call,
        result_read_by_own_statement: read_by_own_statement,
        exit,
    })
}

/// Every block nested inside one statement, for the whole-body walk.
fn push_nested_blocks<'check>(
    statement: &'check CheckedStatement,
    blocks: &mut Vec<&'check [CheckedStatement]>,
) {
    match statement {
        CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
            for arm in arms {
                blocks.push(arm.body.as_slice());
            }
        }
        CheckedStatement::Loop { body, .. }
        | CheckedStatement::Region { body, .. }
        | CheckedStatement::CountedRange { body, .. } => blocks.push(body.as_slice()),
        CheckedStatement::Let { .. }
        | CheckedStatement::DestructuringLet { .. }
        | CheckedStatement::PropagateLet { .. }
        | CheckedStatement::Set { .. }
        | CheckedStatement::SetList { .. }
        | CheckedStatement::Replace { .. }
        | CheckedStatement::Proof(_)
        | CheckedStatement::Dispose { .. }
        | CheckedStatement::DropExpression { .. }
        | CheckedStatement::Evaluate(_)
        | CheckedStatement::Return { .. }
        | CheckedStatement::Give { .. }
        | CheckedStatement::Break { .. } => {}
    }
}

/// Every binding one expression tree mentions, for the ordinary def-use test.
fn collect_used_bindings(expression: &CheckedExpression, out: &mut Vec<BindingId>) {
    visit_read_bindings(expression, &mut |binding| {
        if !out.contains(&binding) {
            out.push(binding);
        }
    });
}

/// Calls `note` once per binding occurrence one expression tree reads.
///
/// The dedup belongs to the caller, because a caller that has to tell one
/// occurrence of a binding from two — the loop-split hint asks exactly that of
/// an accumulator — cannot recover the count from a deduplicated list. Which
/// expression form reads which binding is classified here, beside the other
/// exhaustive matches that keep this analysis from missing a read.
pub(crate) fn visit_read_bindings(
    expression: &CheckedExpression,
    note: &mut impl FnMut(BindingId),
) {
    match expression {
        CheckedExpression::Binding { binding, .. }
        | CheckedExpression::Project { binding, .. }
        | CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::BorrowSystemResource { binding, .. }
        | CheckedExpression::ReborrowAddressed { binding, .. }
        | CheckedExpression::DerefAddressed { binding, .. } => note(*binding),
        CheckedExpression::BorrowBuffer { root, .. }
        | CheckedExpression::BufferMeasure { root, .. }
        | CheckedExpression::BufferIndex { root, .. } => note(root.binding),
        CheckedExpression::SliceMeasure { root, .. }
        | CheckedExpression::SliceIndex { root, .. } => note(root.binding),
        CheckedExpression::ArrayMeasure { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => {
            if let CheckedArrayRoot::Binding { binding, .. } = root {
                note(*binding);
            }
        }
        CheckedExpression::SliceOf { source, .. } => match source {
            CheckedSliceSource::Array { root, .. } => {
                if let CheckedArrayRoot::Binding { binding, .. } = root {
                    note(*binding);
                }
            }
            CheckedSliceSource::Buffer(root) => note(root.binding),
            CheckedSliceSource::ArenaContent { binding, .. } => note(*binding),
        },
        _ => {}
    }
    for child in expression_children(expression) {
        visit_read_bindings(child, note);
    }
}

/// Every caller place one operand expression reads on the calling thread,
/// with an unresolved read failing closed.
///
/// This is deliberately not the [EFF-2] callee projection. It is the storage
/// the *caller* touches while building an actual: a value read out of a
/// binding, a field, a `deref`, a buffer or array element. Forming a borrow
/// takes an address and reads no content, so it contributes nothing here — the
/// callee's declared row already covers whatever it reaches through that
/// borrow. Reading through a slice descriptor cannot be resolved to the
/// storage it views, so it denies rather than resolving to the descriptor.
///
/// The match is exhaustive on purpose. A future expression form that reads
/// caller storage must be classified here rather than silently contributing
/// nothing, because a missing operand read widens permission.
pub(super) fn collect_operand_reads(
    places: &PlaceMap,
    expression: &CheckedExpression,
    node: &NodePath,
    footprint: &mut Footprint,
) {
    fn read(footprint: &mut Footprint, node: &NodePath, place: ResolvedPlace) {
        footprint.operand_reads.push(Access::Place {
            place,
            argument: node.clone(),
        });
    }
    match expression {
        // Reads no caller storage of its own.
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::IntegerOperation { .. }
        | CheckedExpression::FloatOperation { .. }
        | CheckedExpression::NumericConversion { .. }
        | CheckedExpression::Reinterpret { .. }
        | CheckedExpression::BooleanOperation { .. }
        | CheckedExpression::EnumEquality { .. }
        | CheckedExpression::ArrayFill { .. }
        | CheckedExpression::BufferFill { .. }
        | CheckedExpression::BufferVacant { .. }
        | CheckedExpression::BufferFits { .. }
        | CheckedExpression::BoxNew { .. }
        | CheckedExpression::ArenaNew { .. }
        | CheckedExpression::ConstructStruct { .. }
        | CheckedExpression::ConstructEnum { .. }
        | CheckedExpression::ProjectValue { .. } => {}
        // Address formation: no content is read on this thread.
        CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowAddressed { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. } => {}
        // The handle itself is the recursed child, and its resolved place is
        // where an opaque referent anchors, so the child walk covers both.
        CheckedExpression::BoxDeref { .. } | CheckedExpression::ArenaDeref { .. } => {}
        CheckedExpression::Binding { binding, .. } => {
            read(footprint, node, rooted_place(places, *binding, &[]));
        }
        CheckedExpression::Project {
            binding, fields, ..
        } => read(footprint, node, rooted_place(places, *binding, fields)),
        CheckedExpression::DerefAddressed { binding, .. } => {
            read(footprint, node, places.resolve_deref(*binding, 0));
        }
        CheckedExpression::BufferMeasure { root, .. }
        | CheckedExpression::BufferIndex { root, .. } => {
            read(
                footprint,
                node,
                rooted_place(places, root.binding, &root.fields),
            );
        }
        CheckedExpression::ContainerMeasure { root, .. }
        | CheckedExpression::RunIndex { root, .. } => {
            read(footprint, node, rooted_container_place(places, root));
        }
        CheckedExpression::ArrayMeasure { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => {
                read(footprint, node, rooted_place(places, *binding, fields));
            }
            CheckedArrayRoot::Constant(id) => read(
                footprint,
                node,
                ResolvedPlace {
                    root: PlaceRoot::Constant(*id),
                    path: Vec::new(),
                },
            ),
        },
        CheckedExpression::SliceOf { source, .. } => {
            read(footprint, node, slice_source_place(places, source));
        }
        // A slice descriptor names storage this analysis does not resolve, so
        // reading through one fails closed.
        CheckedExpression::SliceMeasure { .. } | CheckedExpression::SliceIndex { .. } => {
            footprint.operand_unresolved = Some(node.clone());
        }
        // [GRAM-9] forbids a call in argument position; if one ever reaches
        // here its whole footprint is unaccounted for.
        CheckedExpression::UserCall { .. }
        | CheckedExpression::SystemCall { .. }
        | CheckedExpression::KernelCall { .. } => {
            footprint.operand_unresolved = Some(node.clone());
        }
        // One clause-only datum; no executable statement carries one.
        CheckedExpression::PostconditionResultMeasure { .. } => {}
    }
    for child in expression_children(expression) {
        collect_operand_reads(places, child, node, footprint);
    }
}

/// The caller place one actual reaches, for a parameter whose row projects an
/// access through it.
fn argument_place(places: &PlaceMap, argument: &CheckedExpression) -> Option<ResolvedPlace> {
    if let Some((place, _element, _entry_image)) = places.argument_referent(argument) {
        return Some(place);
    }
    match argument {
        CheckedExpression::SliceOf { source, .. } => Some(slice_source_place(places, source)),
        _ => None,
    }
}

/// The storage a direct slice value views.
pub(super) fn slice_source_place(places: &PlaceMap, source: &CheckedSliceSource) -> ResolvedPlace {
    match source {
        CheckedSliceSource::Array { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => rooted_place(places, *binding, fields),
            CheckedArrayRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(*id),
                path: Vec::new(),
            },
        },
        CheckedSliceSource::Buffer(root) => rooted_place(places, root.binding, &root.fields),
        CheckedSliceSource::ArenaContent {
            binding, fields, ..
        } => rooted_place(places, *binding, fields),
    }
}

/// The [OWN-5] place one measured or subscripted root names [MSR-1, MSR-2].
///
/// A run's path may carry subscripts of its own, so it is resolved from the
/// same source-order path the proof engine reads and never from a field list.
pub(super) fn rooted_container_place(
    places: &PlaceMap,
    root: &super::model::CheckedContainerRoot,
) -> ResolvedPlace {
    let mut projections = Vec::new();
    if places.is_holder(root.binding) {
        projections.push(super::places::PlaceProjection::Deref);
    }
    projections.extend(root.path.iter().map(|step| match step {
        super::model::CheckedPlaceStep::Field(field) => {
            super::places::PlaceProjection::Field(*field)
        }
        super::model::CheckedPlaceStep::Subscript(subscript) => {
            super::places::PlaceProjection::Subscript(subscript.place_offset)
        }
    }));
    places.resolve_projected(&super::places::ProjectedPlaceTerm {
        root: PlaceRoot::Binding(root.binding),
        projections,
    })
}

pub(super) fn rooted_place(places: &PlaceMap, binding: BindingId, fields: &[u32]) -> ResolvedPlace {
    places.resolve(&PlaceTerm {
        root: PlaceRoot::Binding(binding),
        deref: places.is_holder(binding),
        fields: fields.to_vec(),
    })
}

/// The caller place a consuming `own` actual transfers away, when the actual
/// names one.
fn consumed_place(places: &PlaceMap, argument: &CheckedExpression) -> Option<ResolvedPlace> {
    match argument {
        CheckedExpression::Binding {
            binding,
            consume_root: true,
            ..
        } => Some(rooted_place(places, *binding, &[])),
        CheckedExpression::Project {
            binding,
            consume_root: true,
            fields,
            ..
        } => Some(rooted_place(places, *binding, fields)),
        _ => None,
    }
}
