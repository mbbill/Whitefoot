//! The permission judgment P: whether two sibling call statements may be
//! executed with their evaluations overlapped.
//!
//! P is a compiler-internal legality judgment. It refuses nothing, changes no
//! acceptance, and grants no lowering by itself; it records, per analyzed
//! site, whether overlapping the two statements is permitted and whether the
//! permitted overlap is actualizable.
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
//! 3. **Row gate.** Neither callee's row carries `external` or `blocks`, and
//!    neither does the row of any call written between them. Rows gate; places
//!    prove. No disjointness is ever derived from a row.
//! 4. **No skipping exit.** No exit edge of s1 bypasses s2, and no statement
//!    between them carries an exit edge at all: s1's only continuation is s2.
//!    A `propagate` right-hand side has an `Err` edge to the function-return
//!    sink [ERR-3], so it is never a first member and never an interposed one;
//!    a `claim` has a trap edge to the [DIAG-3] sink, which the eligibility
//!    closure below cannot see because it inspects callees and not the
//!    caller's own block. This is not merely a condition about differing
//!    observables: under either schedule a hand-out is outstanding at every
//!    interposed statement, so an exit taken there abandons an unjoined lane
//!    still reading the caller's frame.
//!
//! Two schedules are realizable for one window — hand s1 to a lane and run
//! T1…Tk then s2 on the calling thread, or run s1 then hoist s2's operands,
//! hand s2 out, and run T1…Tk — and [PAR-1] forbids stating any rule in terms
//! of the schedule. The conditions above are therefore the **intersection** of
//! what the two admit, never the weaker set the current backend alone would
//! survive.
//!
//! Actualization additionally requires eligibility: the transitive call
//! closure of both callees reaches zero `claim` sites. Claims are the only
//! writer-reachable runtime checks, so a claim-free closure has no trap site
//! and therefore no trap-selection question.
//!
//! **Invariant.** P consults typing, declared effect rows, resolved places
//! [OWN-5, OWN-7], the statement graph's exit edges, and the monomorphized
//! call graph — and never the entailment fact state. Nothing here reads a
//! derived fact, an obligation disposition, a claim disposition, or any
//! optimizer fact; the only thing it asks about a claim is that its statement
//! exists. Facts-on and facts-off compilation therefore produce the same
//! permission table by construction, and permission can never turn an
//! accepted program into a rejected one or move a required check.

use std::collections::VecDeque;

use super::entailment::collect_statement_calls;
use super::loop_permission::LoopPermission;
use super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedFunction, CheckedMode, CheckedSetTarget,
    CheckedSliceSource, CheckedStatement, CheckedType, FunctionId, expression_children,
};
use super::places::{PlaceMap, PlaceRoot, PlaceTerm, ResolvedPlace};
use crate::{DeclarationId, NodePath};

/// The declared effect row and region parameters of one concrete function, as
/// P reads them. This is the callable boundary only: no body fact enters.
#[derive(Clone, Debug, Default)]
pub(crate) struct PermissionSignature {
    /// Formal region parameters in declaration order.
    pub(crate) region_parameters: Vec<DeclarationId>,
    pub(crate) reads: Vec<DeclarationId>,
    pub(crate) writes: Vec<DeclarationId>,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
    pub(crate) external: bool,
    pub(crate) blocks: bool,
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
    /// A `claim`'s trap edge to the [DIAG-3] sink [CLM-1]. Eligibility cannot
    /// stand in for this one: `claim_closure` walks *callees*, so a claim the
    /// writer put in the caller's own block between the two calls is invisible
    /// to it.
    ClaimTrap,
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
    /// slice region: a callee row that declares `writes('r)` projects nothing
    /// onto it, and only the handle's consumed place is recorded. Every arena
    /// program stops today at `UnsupportedSemanticFeature::ArenaRuntime`, so
    /// nothing reaches this gap; the arena lane must close it rather than
    /// inherit the projection.
    Arena {
        region: DeclarationId,
        call: NodePath,
    },
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConflictKind {
    /// Both statements write.
    WriteWrite,
    /// The earlier writes and the later's callee reads through an actual.
    WriteRead,
    /// The earlier reads and the later writes.
    ReadWrite,
    /// The earlier writes and the later reads the same storage on the calling
    /// thread while evaluating its own operands.
    WriteOperandRead,
    /// The earlier reads storage on the calling thread while evaluating its
    /// own operands, and the later writes it.
    OperandReadWrite,
}

impl ConflictKind {
    /// The two footprint halves this conflict joins, earlier first, as the
    /// ledger names them.
    pub(crate) const fn halves(self) -> (&'static str, &'static str) {
        match self {
            Self::WriteWrite => ("write", "write"),
            Self::WriteRead => ("write", "read"),
            Self::ReadWrite => ("read", "write"),
            Self::WriteOperandRead => ("write", "operand read"),
            Self::OperandReadWrite => ("operand read", "write"),
        }
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
    /// Condition 3: a callee row carries `external` or `blocks`.
    Row {
        side: PairSide,
        external: bool,
        blocks: bool,
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
            | Self::UnresolvedFootprint { .. }
            | Self::InterposedForm { .. } => 2,
            Self::Row { .. } => 3,
            Self::SkippingExit { .. } => 4,
        }
    }
}

/// One reachable `claim` site that keeps a permitted pair from being
/// actualized, with the call path that reaches it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimWitness {
    /// Analyzed-callee-to-claim-bearing-callee path, root first.
    pub(crate) path: Vec<FunctionId>,
    /// The claim-bearing function's own name, for citation.
    pub(crate) function: String,
    /// The claim's written name.
    pub(crate) claim: String,
    pub(crate) node_path: NodePath,
}

/// The judgment's outcome for one analyzed pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PermissionVerdict {
    /// P holds and both call closures are claim-free.
    PermittedEligible,
    /// P holds, but a reachable `claim` site keeps the overlap out of reach
    /// of this version's actualization. Each checker improvement that turns a
    /// claim into a proof widens the eligible set [ENT-1].
    PermittedNotActualizable {
        claim_sites: usize,
        witness: ClaimWitness,
    },
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
            Self::PermittedEligible | Self::PermittedNotActualizable { .. } => None,
        }
    }
}

/// One analyzed call statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionSite {
    /// The owning `let_stmt` node.
    pub(crate) statement: NodePath,
    /// The binding the statement defines. A join must complete before the
    /// first use of it.
    pub(crate) binding: BindingId,
    /// The call occurrence inside it.
    pub(crate) call: NodePath,
    pub(crate) callee: FunctionId,
    pub(crate) callee_name: String,
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

/// Every analyzed pair and eligible chain of one concrete function, in source
/// order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FunctionPermissions {
    pub(crate) function: String,
    pub(crate) pairs: Vec<PermissionPair>,
    pub(crate) runs: Vec<PermissionRun>,
    /// The [PAR-2] verdict of every counted loop of this function, in source
    /// order. The pair judgment above is computed exactly as it was before
    /// these existed, and nothing here is lowered by this version.
    pub(crate) loops: Vec<LoopPermission>,
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
    let callees = direct_callees(functions);
    let claims = functions
        .iter()
        .map(|function| {
            let mut sites = Vec::new();
            collect_claim_sites(&function.body, &mut sites);
            sites
        })
        .collect::<Vec<_>>();
    let program = Program {
        functions,
        signatures,
        reaches_claim: claim_reachability(&claims, &callees),
        callees,
        claims,
    };
    PermissionMetadata {
        functions: functions
            .iter()
            .map(|function| program.analyze_function(function))
            .collect(),
    }
}

/// One `claim` statement, by written name and node.
struct ClaimRecord {
    name: String,
    node_path: NodePath,
}

pub(super) struct Program<'check> {
    functions: &'check [CheckedFunction],
    signatures: &'check [PermissionSignature],
    /// Direct monomorphized callees per function, ascending and deduplicated.
    callees: Vec<Vec<FunctionId>>,
    /// `claim` statements per function, in source order.
    claims: Vec<Vec<ClaimRecord>>,
    /// Whether some function reachable from each function carries a `claim`,
    /// dense by [`FunctionId`]. Eligibility asks this of every judged window,
    /// and a chain of m calls is judged over O(m²) ordered pairs, so answering
    /// it by a whole-program search each time made the search the dominant
    /// cost of the analysis. One reverse walk answers it for every function.
    reaches_claim: Vec<bool>,
}

/// Whether a `claim` is reachable from each function, by one walk of the call
/// graph backwards from the functions that carry one.
///
/// This is exactly `claim_closure(&[f]).is_some()` for every f, computed
/// together. It decides only *whether* to search; the count and the witness
/// still come from the forward search, so nothing a ledger or a test reads is
/// derived from here.
fn claim_reachability(claims: &[Vec<ClaimRecord>], callees: &[Vec<FunctionId>]) -> Vec<bool> {
    let mut callers: Vec<Vec<usize>> = vec![Vec::new(); claims.len()];
    for (caller, called) in callees.iter().enumerate() {
        for callee in called {
            if let Some(entry) = callers.get_mut(callee.0 as usize) {
                entry.push(caller);
            }
        }
    }
    let mut reaches = vec![false; claims.len()];
    let mut pending = claims
        .iter()
        .enumerate()
        .filter(|(_, sites)| !sites.is_empty())
        .map(|(ordinal, _)| ordinal)
        .collect::<Vec<_>>();
    while let Some(ordinal) = pending.pop() {
        let Some(seen) = reaches.get_mut(ordinal) else {
            continue;
        };
        if *seen {
            continue;
        }
        *seen = true;
        if let Some(calling) = callers.get(ordinal) {
            pending.extend(calling.iter().copied());
        }
    }
    reaches
}

/// One candidate statement: a `let` whose right-hand side is exactly one
/// named-function call.
struct Candidate<'check> {
    statement: NodePath,
    /// Position of this statement in its own block. Two candidates and the
    /// statements between their positions are one judged window.
    index: usize,
    /// The binding the statement defines.
    binding: BindingId,
    call: CallProjection<'check>,
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
    pub(super) callee: FunctionId,
    pub(super) arguments: &'check [CheckedExpression],
    pub(super) argument_nodes: &'check [NodePath],
    pub(super) regions: &'check [DeclarationId],
}

/// The call one expression is, or `None` for every other expression form.
pub(super) fn call_projection(value: &CheckedExpression) -> Option<CallProjection<'_>> {
    let CheckedExpression::UserCall {
        function,
        call,
        argument_nodes,
        arguments,
        goal_regions,
        ..
    } = value
    else {
        return None;
    };
    Some(CallProjection {
        call,
        callee: *function,
        arguments,
        argument_nodes,
        regions: goal_regions,
    })
}

/// One statement written between the two judged calls, reduced to what the
/// window rule asks of it.
struct Interposed {
    /// The binding it defines, when it defines one.
    defines: Option<BindingId>,
    /// Every binding its own operands mention.
    uses: Vec<BindingId>,
    footprint: Footprint,
    /// Its callee, when the statement is one call. A call between the two
    /// members faces the same row gate they do and joins the eligibility
    /// roots.
    callee: Option<FunctionId>,
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
            loops: Vec::new(),
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
        PermissionSite {
            statement: candidate.statement.clone(),
            binding: candidate.binding,
            call: candidate.call.call.clone(),
            callee: candidate.call.callee,
            callee_name: self
                .functions
                .get(candidate.call.callee.0 as usize)
                .map(|function| function.name.clone())
                .unwrap_or_default(),
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
        let mut interposed = Vec::with_capacity(second.index - first.index);
        for (offset, classified) in windows
            .statements
            .get(first.index + 1..second.index)
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
        if used.contains(&first.binding) {
            return PermissionVerdict::Denied(Denial::Dataflow {
                binding: first.binding,
                definer: PairSide::First,
                reader: PairSide::Second,
            });
        }
        for (offset, record) in interposed.iter().enumerate() {
            // Where s1 takes the lane its value does not exist until the join,
            // so nothing between them may read it.
            if record.uses.contains(&first.binding) {
                return PermissionVerdict::Denied(Denial::Dataflow {
                    binding: first.binding,
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

        // Condition 3: the row gate, over both members and every call between
        // them. A row that gates the members gates a call written between them
        // for the same reason: nothing about its reach is proved by places.
        for (side, callee) in [
            (PairSide::First, first.call.callee),
            (PairSide::Second, second.call.callee),
        ]
        .into_iter()
        .chain(
            interposed
                .iter()
                .enumerate()
                .filter_map(|(offset, record)| {
                    record
                        .callee
                        .map(|callee| (PairSide::Between(offset), callee))
                }),
        ) {
            let Some(signature) = self.signatures.get(callee.0 as usize) else {
                return PermissionVerdict::Denied(Denial::Row {
                    side,
                    external: true,
                    blocks: true,
                });
            };
            if signature.external || signature.blocks {
                return PermissionVerdict::Denied(Denial::Row {
                    side,
                    external: signature.external,
                    blocks: signature.blocks,
                });
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

        // Eligibility: every call closure of the window reaches zero claim
        // sites. A call between the members can be actualized only alongside
        // them, so its claims count exactly as a member's do.
        let mut roots = vec![first.call.callee, second.call.callee];
        roots.extend(interposed.iter().filter_map(|record| record.callee));
        // The precomputed reachability answers the common case — every root
        // claim-free — without walking the call graph at all. Only a root that
        // does reach a claim pays for the search, which exists to produce the
        // count and the witness.
        if roots.iter().all(|root| {
            !self
                .reaches_claim
                .get(root.0 as usize)
                .copied()
                .unwrap_or(true)
        }) {
            return PermissionVerdict::PermittedEligible;
        }
        match self.claim_closure(&roots) {
            Some((claim_sites, witness)) => PermissionVerdict::PermittedNotActualizable {
                claim_sites,
                witness,
            },
            None => PermissionVerdict::PermittedEligible,
        }
    }

    /// One statement between the two members, reduced to what the window rule
    /// judges, or the reason it cannot be.
    ///
    /// The match is exhaustive on purpose, for the same reason
    /// [`collect_claim_sites`] is: a statement form this analysis does not
    /// classify would otherwise contribute an empty footprint and *widen*
    /// permission, which is the one direction the judgment must never fail in.
    /// Every form is either given a footprint here or refused here.
    fn interposed_of(
        &self,
        places: &PlaceMap,
        index: usize,
        statement: &'check CheckedStatement,
    ) -> Result<Interposed, InterposedRefusal> {
        match statement {
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
                        callee: Some(candidate.call.callee),
                    });
                }
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                Ok(Interposed {
                    defines: Some(*binding),
                    uses,
                    footprint: value_footprint(places, value, node_path),
                    callee: None,
                })
            }
            CheckedStatement::Set {
                node_path,
                target,
                value,
            } => {
                let mut footprint = value_footprint(places, value, node_path);
                set_target_place(places, target, node_path, &mut footprint, false);
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                collect_set_target_bindings(target, &mut uses);
                Ok(Interposed {
                    defines: None,
                    uses,
                    footprint,
                    callee: None,
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
                let mut footprint = value_footprint(places, value, node_path);
                set_target_place(places, target, node_path, &mut footprint, true);
                let mut uses = Vec::new();
                collect_used_bindings(value, &mut uses);
                collect_set_target_bindings(target, &mut uses);
                Ok(Interposed {
                    defines: Some(*binding),
                    uses,
                    footprint,
                    callee: None,
                })
            }
            // Exit-bearing forms: condition 4.
            CheckedStatement::PropagateLet { .. } => {
                Err(InterposedRefusal::Exit(ExitKind::PropagateError))
            }
            CheckedStatement::Claim { .. } => Err(InterposedRefusal::Exit(ExitKind::ClaimTrap)),
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
        let mut footprint = Footprint::default();
        let (Some(signature), Some(callee)) = (
            self.signatures.get(candidate.callee.0 as usize),
            self.functions.get(candidate.callee.0 as usize),
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
            let mode_region = match parameter.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
            };
            let slice_region = match parameter.ty {
                CheckedType::Slice { region, .. } => Some(region),
                _ => None,
            };
            let carries = |declared: &[DeclarationId]| {
                mode_region.is_some_and(|region| declared.contains(&region))
                    || slice_region.is_some_and(|region| declared.contains(&region))
            };
            let written = carries(&signature.writes);
            let read = carries(&signature.reads);
            if written || read {
                match argument_place(places, argument) {
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
                continue;
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

    /// The declared row of one concrete function, or `None` when this
    /// analysis does not hold one. A caller that cannot read a row denies.
    pub(super) fn signature(&self, function: FunctionId) -> Option<&PermissionSignature> {
        self.signatures.get(function.0 as usize)
    }

    /// One concrete function's source name, for citation.
    pub(super) fn function_name(&self, function: FunctionId) -> String {
        self.functions
            .get(function.0 as usize)
            .map(|function| function.name.clone())
            .unwrap_or_default()
    }

    /// Whether a `claim` is reachable from one function. An unknown function
    /// answers `true`, so a callee this analysis cannot place keeps the
    /// overlap out of reach rather than being read as claim-free.
    pub(super) fn reaches_claim(&self, function: FunctionId) -> bool {
        self.reaches_claim
            .get(function.0 as usize)
            .copied()
            .unwrap_or(true)
    }

    /// Total reachable `claim` sites over the transitive call closure of the
    /// given roots, with the first witness in deterministic order, or `None`
    /// when the closure is claim-free. One breadth-first walk covers both
    /// roots, so a function reachable from both is counted once.
    pub(super) fn claim_closure(&self, roots: &[FunctionId]) -> Option<(usize, ClaimWitness)> {
        let mut total = 0;
        let mut witness = None;
        let mut visited = vec![false; self.functions.len()];
        let mut queue = VecDeque::new();
        let mut parents: Vec<(FunctionId, Option<FunctionId>)> = Vec::new();
        for root in roots {
            let Some(seen) = visited.get_mut(root.0 as usize) else {
                continue;
            };
            if !*seen {
                *seen = true;
                parents.push((*root, None));
                queue.push_back(*root);
            }
        }
        while let Some(current) = queue.pop_front() {
            let claims = self
                .claims
                .get(current.0 as usize)
                .map(Vec::as_slice)
                .unwrap_or_default();
            total += claims.len();
            if witness.is_none()
                && let Some(claim) = claims.first()
            {
                witness = Some(ClaimWitness {
                    path: witness_path(&parents, current),
                    function: self
                        .functions
                        .get(current.0 as usize)
                        .map(|function| function.name.clone())
                        .unwrap_or_default(),
                    claim: claim.name.clone(),
                    node_path: claim.node_path.clone(),
                });
            }
            let callees = self
                .callees
                .get(current.0 as usize)
                .map(Vec::as_slice)
                .unwrap_or_default();
            for callee in callees {
                let Some(seen) = visited.get_mut(callee.0 as usize) else {
                    continue;
                };
                if !*seen {
                    *seen = true;
                    parents.push((*callee, Some(current)));
                    queue.push_back(*callee);
                }
            }
        }
        witness.map(|witness| (total, witness))
    }
}

/// The breadth-first path from a root to `target`, root first.
fn witness_path(
    parents: &[(FunctionId, Option<FunctionId>)],
    target: FunctionId,
) -> Vec<FunctionId> {
    let mut path = vec![target];
    let mut current = target;
    while let Some((_, Some(parent))) = parents.iter().find(|(node, _)| *node == current) {
        path.push(*parent);
        current = *parent;
    }
    path.reverse();
    path
}

#[derive(Debug, Default)]
pub(super) struct Footprint {
    pub(super) writes: Vec<Access>,
    pub(super) reads: Vec<Access>,
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
    for write in &earlier.writes {
        for (kind, access) in later
            .writes
            .iter()
            .map(|access| (ConflictKind::WriteWrite, access))
            .chain(
                later
                    .reads
                    .iter()
                    .map(|access| (ConflictKind::WriteRead, access)),
            )
            .chain(
                later
                    .operand_reads
                    .iter()
                    .map(|access| (ConflictKind::WriteOperandRead, access)),
            )
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
            .map(|access| (ConflictKind::ReadWrite, access))
            .chain(
                earlier_operands
                    .then(|| {
                        earlier
                            .operand_reads
                            .iter()
                            .map(|access| (ConflictKind::OperandReadWrite, access))
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

/// The footprint of a window statement that is not a call: the places its
/// consumed `own` operands transfer away, and the places its operands read.
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
fn candidate_of(index: usize, statement: &CheckedStatement) -> Option<Candidate<'_>> {
    let (node_path, binding, value, exit) = match statement {
        CheckedStatement::Let {
            node_path,
            binding,
            value,
        } => (node_path, binding, value, None),
        CheckedStatement::PropagateLet {
            node_path,
            binding,
            scrutinee,
            ..
        } => (
            node_path,
            binding,
            scrutinee,
            Some(ExitKind::PropagateError),
        ),
        _ => return None,
    };
    Some(Candidate {
        statement: node_path.clone(),
        index,
        binding: *binding,
        call: call_projection(value)?,
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
        | CheckedStatement::PropagateLet { .. }
        | CheckedStatement::Set { .. }
        | CheckedStatement::Replace { .. }
        | CheckedStatement::DropExpression { .. }
        | CheckedStatement::Evaluate(_)
        | CheckedStatement::Claim { .. }
        | CheckedStatement::Return { .. }
        | CheckedStatement::Give { .. }
        | CheckedStatement::Break { .. } => {}
    }
}

/// Every `claim` statement of one body, in source order.
///
/// The match is exhaustive on purpose: a future body-bearing statement form
/// that this walk did not descend into would hide the claims inside it, and a
/// hidden claim *widens* eligibility — the one direction this judgment must
/// never fail in. Every other axis of P denies when it cannot see.
fn collect_claim_sites(statements: &[CheckedStatement], out: &mut Vec<ClaimRecord>) {
    for statement in statements {
        match statement {
            CheckedStatement::Claim { name, site, .. } => out.push(ClaimRecord {
                name: name.clone(),
                node_path: site.node_path.clone(),
            }),
            CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
                for arm in arms {
                    collect_claim_sites(&arm.body, out);
                }
            }
            CheckedStatement::Loop { body, .. }
            | CheckedStatement::Region { body, .. }
            | CheckedStatement::CountedRange { body, .. } => collect_claim_sites(body, out),
            CheckedStatement::Let { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => {}
        }
    }
}

/// The direct monomorphized callees of every function, from the shared
/// concrete call-occurrence collector. This is call-graph shape only.
fn direct_callees(functions: &[CheckedFunction]) -> Vec<Vec<FunctionId>> {
    functions
        .iter()
        .map(|function| {
            let mut calls = Vec::new();
            collect_statement_calls(function.id, &function.body, &mut calls);
            let mut callees = calls
                .into_iter()
                .map(|call| call.callee)
                .collect::<Vec<_>>();
            callees.sort_unstable_by_key(|callee| callee.0);
            callees.dedup();
            callees
        })
        .collect()
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
        | CheckedExpression::BufferLength { root }
        | CheckedExpression::BufferIndex { root, .. } => note(root.binding),
        CheckedExpression::SliceLength { root } | CheckedExpression::SliceIndex { root, .. } => {
            note(root.binding)
        }
        CheckedExpression::ArrayLength { root, .. }
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
fn collect_operand_reads(
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
        CheckedExpression::BufferLength { root } | CheckedExpression::BufferIndex { root, .. } => {
            read(
                footprint,
                node,
                rooted_place(places, root.binding, &root.fields),
            );
        }
        CheckedExpression::ArrayLength { root, .. }
        | CheckedExpression::ArrayIndex { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => {
                read(footprint, node, rooted_place(places, *binding, fields));
            }
            CheckedArrayRoot::Constant(id) => read(
                footprint,
                node,
                ResolvedPlace {
                    root: PlaceRoot::Constant(*id),
                    fields: Vec::new(),
                },
            ),
        },
        CheckedExpression::SliceOf { source, .. } => {
            read(footprint, node, slice_source_place(places, source));
        }
        // A slice descriptor names storage this analysis does not resolve, so
        // reading through one fails closed.
        CheckedExpression::SliceLength { .. } | CheckedExpression::SliceIndex { .. } => {
            footprint.operand_unresolved = Some(node.clone());
        }
        // [GRAM-9] forbids a call in argument position; if one ever reaches
        // here its whole footprint is unaccounted for.
        CheckedExpression::UserCall { .. } | CheckedExpression::SystemCall { .. } => {
            footprint.operand_unresolved = Some(node.clone());
        }
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
fn slice_source_place(places: &PlaceMap, source: &CheckedSliceSource) -> ResolvedPlace {
    match source {
        CheckedSliceSource::Array { root, .. } => match root {
            CheckedArrayRoot::Binding { binding, fields } => rooted_place(places, *binding, fields),
            CheckedArrayRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(*id),
                fields: Vec::new(),
            },
        },
        CheckedSliceSource::Buffer(root) => rooted_place(places, root.binding, &root.fields),
        CheckedSliceSource::ArenaContent {
            binding, fields, ..
        } => rooted_place(places, *binding, fields),
    }
}

fn rooted_place(places: &PlaceMap, binding: BindingId, fields: &[u32]) -> ResolvedPlace {
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
