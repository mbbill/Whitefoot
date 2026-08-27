//! The staged loop permission judgment [PAR-3]: whether the body of one loop
//! may be cut at its first `may-suspend` submission, so that the *remainder*
//! of one iteration executes with overlapping execution against the *prologue*
//! of a later one.
//!
//! Like the two judgments next door this one refuses nothing, changes no
//! acceptance, and grants no lowering by itself. It records, per loop that
//! performs I/O, whether the staged schedule is permitted and what disposition
//! each place the body touches carries. Nothing here is lowered by this
//! version.
//!
//! # The unit
//!
//! The unit is one `for_stmt` or `loop_stmt` L with body B. [PAR-2]'s unit is
//! an *index subrange*, which is why it needs a trip count and a compiler-owned
//! binder and why it admits only `for_stmt`. This judgment's unit is **the
//! iteration**, which the statement graph already gives, so it asks nothing
//! about indices and admits `loop_stmt` on the same terms.
//!
//! Write S for the first `may-suspend` call of B in program order, c for the
//! statement that performs S's argument evaluation and submission, P for the
//! statements up to and including c (the *prologue*) and E for the rest (the
//! *remainder*). The judgment applies exactly when B holds a `may-suspend`
//! call; a loop that performs no I/O gets no staged verdict at all, and the
//! [PAR-2] counted permission next door is the only judgment of it.
//!
//! # The seven conditions
//!
//! 1. **The cut exists.** There is one program point c such that every
//!    statement of B either executes before c on every path through B or is
//!    reached only through c. This is computed as a real dominator and
//!    post-dominator query on the body's own control-flow graph ([`Flow`]),
//!    never as a statement-index heuristic: the natural body nests four
//!    `region`/`match` levels deep, and getting this wrong in the permissive
//!    direction breaks condition 2. Post-dominance is taken with respect to
//!    the body's *normal* completion only, so a statement whose every
//!    continuation leaves the loop is in P — which is what admits an early
//!    typed exit written in the prologue. A cut written inside a loop of B
//!    refuses outright: the submission would then run several times per
//!    iteration and the single-entry single-exit shape this condition asks for
//!    does not hold.
//! 2. **Every edge that leaves B leaves from P.** No `return_stmt`, no
//!    `give_stmt` delivering outside B, no `break_stmt` naming L or a loop
//!    enclosing L, and no `let_stmt` selecting `propagate_let_rhs` occurs in E.
//!    With K iterations in flight, an iteration's decision to leave is taken
//!    after later iterations have already submitted operations that the
//!    source-order execution never performs, and an submitted target operation
//!    is an externally observable transition that is not rolled back.
//! 3. **Retained borrows are safe.** Every borrow a `may-suspend` call of B
//!    retains past its own submission is on a place rooted in a binding B
//!    itself introduces, on a place this judgment replicates, or on a place no
//!    footprint of B writes. Every retained borrow is read as retained to its
//!    `terminal` milestone, which is what [SYS-2] publishes today; a borrow
//!    released at submission is a milestone this judgment does not yet have and
//!    reading one early would be the unsound direction.
//! 4. **Exclusive loans in E are safe.** Every exclusive loan a call of E holds
//!    is on a place rooted in a binding B itself introduces or on a place this
//!    judgment replicates. Two remainders coexist, so an exclusive loan on
//!    enclosing storage would put two usable `&uniq` borrows on one place.
//! 5. **Every place rooted outside L that B touches carries a disposition**,
//!    and there are exactly four (see [`Disposition`]). A place with none
//!    denies.
//! 6. **Replicated storage has copy elements.** A place this judgment
//!    replicates, and a construction whose storage an implementation reuses
//!    across iterations, has a copy element type. An element type whose [OWN-1]
//!    class this judgment does not resolve denies.
//! 7. **Fail closed.** An unresolved footprint element, an unresolved loan, an
//!    unresolved operand read, or a body statement form this judgment does not
//!    classify denies permission rather than granting it. This is the same
//!    one-sided reading the counted judgment states: a missed statement would
//!    contribute an empty footprint and *widen* permission, which is the one
//!    direction it must never fail in.
//!
//! # Why exactly these
//!
//! The schedule the conditions admit is: P(0), P(1), … in index order, never
//! two at once; E(i)'s stages executed after P(i+1..i+K) may already have run;
//! E(i)'s writes to places rooted outside L committed in the order of i. So the
//! only pairs that ever coexist are E(i) against P(j) for j > i, and E(i)
//! against E(j) for j != i. Conditions 3 and 5 make the first non-interfering,
//! conditions 4 and 5 the second, and condition 2 means no iteration ever
//! leaves the loop from a segment that could coexist with a later iteration's
//! work.
//!
//! **Prologues never overlap one another.** That is a restriction on the
//! schedule and not a derived fact, and it is what admits `reserve_file`'s
//! short unique loan of an enclosing `FileFactory` with no replication and no
//! loan exemption: at every program point exactly one unique loan of the
//! factory is live, so [OWN-5] is not relaxed.
//!
//! # This version replicates only iteration-own storage
//!
//! The replicated disposition here admits exactly the constructions B itself
//! introduces — the ring slots an implementation may allocate once at loop
//! entry and restore per iteration. An *enclosing* scratch buffer that the body
//! writes and reads is denied, and the denial names it. Admitting one needs a
//! derived byte-range analysis proving that every byte an iteration reads out
//! of it was written earlier in that same iteration; that analysis consumes the
//! entailment fact state and is a later batch. The admitted writer form until
//! then is the one this judgment grants: allocate the scratch inside the loop
//! body, so each iteration owns its own.
//!
//! **Invariant.** Like the two judgments next door, this one consults typing,
//! declared effect rows, resolved places [OWN-5, OWN-7], and the statement
//! graph's edges — and never the entailment fact state. Facts-on and facts-off
//! compilation therefore produce the same staged permission table by
//! construction.

use super::loop_permission::{borrows_only_iteration_own, collect_introduced, nested_bodies};
use super::model::{
    BindingId, CheckedExpression, CheckedFlatElement, CheckedFunction, CheckedLoopId,
    CheckedStatement, CheckedType,
};
use super::permission::{
    Access, Footprint, LoanStrength, Program, call_projection, collect_consumed_places,
    collect_operand_reads, set_target_place,
};
use super::places::{PlaceMap, PlaceRoot, ResolvedPlace};
use crate::NodePath;

/// The staged verdict of one loop whose body performs I/O.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StagedPermission {
    /// The loop's spelling: `for` or `loop`. A `loop_stmt` carries no node path
    /// of its own in the checked tree, so the line the ledger prints is anchored
    /// at the cut instead of at the loop head.
    pub(crate) form: &'static str,
    /// The submission this judgment cut the body at: the source call node of
    /// the first `may-suspend` action of B in program order.
    pub(crate) cut: NodePath,
    pub(crate) verdict: StagedVerdict,
    /// Every place the judgment classified, in the source order of the node
    /// that cites it. This is the teaching channel: a reader sees what each
    /// place cost, not only which one refused the loop.
    pub(crate) dispositions: Vec<PlaceDisposition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StagedVerdict {
    /// Permission holds for the staged schedule.
    Permitted,
    Denied(StagedDenial),
}

impl StagedVerdict {
    pub(crate) const fn is_permitted(&self) -> bool {
        matches!(self, Self::Permitted)
    }
}

/// One classified place, as the ledger prints it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaceDisposition {
    /// The source node that cites the place: an argument, a `set` target, or
    /// the construction that introduced it.
    pub(crate) citation: NodePath,
    pub(crate) disposition: Disposition,
    /// Why the place landed there, in one clause.
    pub(crate) reason: &'static str,
}

/// The four dispositions of condition 5. A place satisfying none denies, and
/// `Denied` is that fourth case made explicit so the table shows it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Disposition {
    /// No footprint of B writes it and every loan on it is shared.
    ReadOnly,
    /// Every footprint element and every loan touching it belongs to one
    /// ordered segment, and no loan on it is retained past the cut. Prologues
    /// run in index order and never overlap; writes the remainder performs to
    /// storage rooted outside L commit in the order of the iterations that
    /// perform them. Either segment therefore serializes the place.
    Serialized(Segment),
    /// An implementation may give each concurrently executing iteration its own
    /// storage of the same length.
    Replicated,
    Denied,
}

impl Disposition {
    /// The word the ledger prints.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::Serialized(Segment::Prologue) => "serialized-P",
            Self::Serialized(Segment::Remainder) => "serialized-E",
            Self::Replicated => "replicated",
            Self::Denied => "denied",
        }
    }
}

/// Which of the two ordered segments of B a statement belongs to. The cut
/// itself is the last statement of the prologue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Prologue,
    Remainder,
}

/// Why the staged permission does not hold. Each variant names exactly one
/// condition of the judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StagedDenial {
    /// Condition 1: the body has no single-entry single-exit cut at its first
    /// submission.
    NoCut {
        /// What broke the shape.
        reason: &'static str,
        /// A statement the cut neither dominates nor post-dominates. A
        /// `loop_stmt`, a `region_stmt`, and a `break_stmt` carry no node path
        /// of their own in the checked tree, so this is absent rather than
        /// citing a node the writer did not write there.
        statement: Option<NodePath>,
    },
    /// Condition 2: an edge leaves the loop from the remainder.
    ExitInRemainder {
        edge: &'static str,
        /// Absent for a `break_stmt`, which carries no node path.
        statement: Option<NodePath>,
    },
    /// Condition 3: a `may-suspend` call retains a borrow on enclosing storage
    /// the body writes.
    RetainedBorrow {
        argument: NodePath,
        /// Whether storage of that place's type could be replicated at all.
        /// A buffer of copy elements could, once the coverage proof exists; an
        /// opaque system nominal never can, and telling its writer to allocate
        /// one per iteration would be wrong advice.
        replicable_shape: bool,
    },
    /// Condition 4: a call of the remainder holds an exclusive loan on
    /// enclosing storage.
    RemainderExclusiveLoan {
        argument: NodePath,
        replicable_shape: bool,
    },
    /// Condition 5: a place rooted outside the loop that no disposition covers.
    NoDisposition { argument: NodePath },
    /// Condition 6: storage an implementation would replicate whose element
    /// type is not copy, or whose [OWN-1] class this judgment does not resolve.
    NotReplicable { statement: NodePath },
    /// Condition 7, fail closed: a body statement form whose footprint this
    /// judgment does not compute.
    BodyForm { form: &'static str },
    /// Condition 7, fail closed: a footprint element, loan, or operand read
    /// whose caller place this judgment does not resolve.
    Unresolved { argument: NodePath },
}

impl StagedDenial {
    /// The judgment condition this denial cites. The ledger prints it and the
    /// judgment tests assert it; acceptance never reads it.
    pub(crate) const fn condition(&self) -> u8 {
        match self {
            Self::NoCut { .. } => 1,
            Self::ExitInRemainder { .. } => 2,
            Self::RetainedBorrow { .. } => 3,
            Self::RemainderExclusiveLoan { .. } => 4,
            Self::NoDisposition { .. } => 5,
            Self::NotReplicable { .. } => 6,
            Self::BodyForm { .. } | Self::Unresolved { .. } => 7,
        }
    }

    /// The writer form this judgment admits in place of the refused one, in one
    /// sentence. A denial that named no admitted form would teach nothing, and
    /// teaching the writer is the whole reason this judgment reports a table
    /// rather than a bit.
    pub(crate) const fn writer_form(&self) -> &'static str {
        match self {
            Self::NoCut { .. } => {
                "write the body so its first I/O submission is reached on every path through it and everything else is reached only through it"
            }
            Self::ExitInRemainder { .. } => {
                "take every early return, break, or propagate in the prologue, before the body's first I/O submission"
            }
            Self::RetainedBorrow {
                replicable_shape: true,
                ..
            }
            | Self::RemainderExclusiveLoan {
                replicable_shape: true,
                ..
            } => {
                "allocate the scratch storage inside the loop body, so each iteration owns the buffer it reads and writes"
            }
            // Storage with one position — an enumeration cursor is the pointed
            // case — cannot be given to two iterations at once at any element
            // type, so the per-iteration form is not advice a writer can take.
            Self::RetainedBorrow { .. } | Self::RemainderExclusiveLoan { .. } => {
                "give each iteration its own resource, or leave this loop sequential: storage that carries one position cannot be held by two iterations at once"
            }
            Self::NoDisposition { .. } => {
                "keep each place the body touches on one side of the cut: read it only, or reach it only before the submission, or give the iteration its own"
            }
            Self::NotReplicable { .. } => {
                "give the per-iteration storage a copy element type; an affine or opaque element cannot be replicated"
            }
            Self::BodyForm { .. } | Self::Unresolved { .. } => {
                "write the body out of statement forms whose footprint the judgment resolves: calls, `set`, `replace`, `match`, `region`, and value initializers"
            }
        }
    }
}

/// The staged verdict of every loop of one function whose body performs I/O,
/// in source order.
pub(crate) fn judge_staged<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    function: &'check CheckedFunction,
) -> Vec<StagedPermission> {
    let mut judged = Vec::new();
    collect(program, places, &function.body, &mut judged);
    judged
}

fn collect<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    statements: &'check [CheckedStatement],
    judged: &mut Vec<StagedPermission>,
) {
    for statement in statements {
        // Both loop forms are judged, and on the same terms: the unit is the
        // iteration the statement graph gives, never an index subrange.
        let loop_body = match statement {
            CheckedStatement::CountedRange {
                id, binder, body, ..
            } => Some(("for", *id, vec![*binder], body)),
            CheckedStatement::Loop { id, body, .. } => Some(("loop", *id, Vec::new(), body)),
            _ => None,
        };
        if let Some((form, id, seed, body)) = loop_body
            && let Some(judgement) = judge(program, places, form, id, seed, body)
        {
            judged.push(judgement);
        }
        for nested in nested_bodies(statement) {
            collect(program, places, nested, judged);
        }
    }
}

fn judge<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    form: &'static str,
    id: CheckedLoopId,
    seed: Vec<BindingId>,
    body: &'check [CheckedStatement],
) -> Option<StagedPermission> {
    // No `may-suspend` action means no cut and therefore no staged schedule to
    // permit. Such a loop is judged by [PAR-2] alone and gets no line here.
    let cut = first_may_suspend(program, body)?;
    let mut introduced = seed;
    collect_introduced(body, &mut introduced);
    let flow = Flow::build(id, body);
    let mut survey = StagedSurvey {
        program,
        places,
        introduced,
        cut: cut.call.clone(),
        segments: Vec::new(),
        dispositions: Vec::new(),
        touched: Vec::new(),
        replicated: Vec::new(),
        form_refusal: None,
        unresolved: None,
        not_replicable: None,
        exit_in_remainder: None,
    };
    let cut_denial = survey.classify_segments(&flow, cut.statement);
    if let Some(denial) = cut_denial {
        return Some(StagedPermission {
            form,
            cut: cut.call.clone(),
            verdict: StagedVerdict::Denied(denial),
            dispositions: Vec::new(),
        });
    }
    survey.walk(&flow);
    Some(survey.finish(form, cut.call.clone()))
}

/// The first `may-suspend` call of one body in program order, with the
/// statement that performs it.
struct Cut<'check> {
    statement: &'check CheckedStatement,
    /// The source call node, which is what the ledger anchors on.
    call: NodePath,
}

fn first_may_suspend<'check>(
    program: &Program<'check>,
    statements: &'check [CheckedStatement],
) -> Option<Cut<'check>> {
    for statement in statements {
        for expression in statement_expressions(statement) {
            if let Some(call) = first_may_suspend_call(program, expression) {
                return Some(Cut { statement, call });
            }
        }
        for nested in nested_bodies(statement) {
            if let Some(cut) = first_may_suspend(program, nested) {
                return Some(cut);
            }
        }
    }
    None
}

/// The first `may-suspend` call one expression tree performs, in evaluation
/// order.
fn first_may_suspend_call(
    program: &Program<'_>,
    expression: &CheckedExpression,
) -> Option<NodePath> {
    let suspends = match expression {
        CheckedExpression::UserCall { function, call, .. } => program
            .target_action(*function)
            .may_suspend()
            .then(|| call.clone()),
        CheckedExpression::SystemCall {
            target_action,
            call,
            ..
        } => target_action.may_suspend().then(|| call.clone()),
        _ => None,
    };
    if suspends.is_some() {
        return suspends;
    }
    super::model::expression_children(expression)
        .into_iter()
        .find_map(|child| first_may_suspend_call(program, child))
}

/// Every expression one statement evaluates as its own, outside its nested
/// blocks.
fn statement_expressions(statement: &CheckedStatement) -> Vec<&CheckedExpression> {
    match statement {
        CheckedStatement::Let { value, .. }
        | CheckedStatement::Evaluate(value)
        | CheckedStatement::DropExpression { value, .. }
        | CheckedStatement::Return { value, .. }
        | CheckedStatement::Give { value, .. } => vec![value],
        CheckedStatement::Set { value, .. } | CheckedStatement::Replace { value, .. } => {
            vec![value]
        }
        CheckedStatement::PropagateLet { scrutinee, .. }
        | CheckedStatement::Match { scrutinee, .. }
        | CheckedStatement::ValueMatchLet { scrutinee, .. } => vec![scrutinee],
        CheckedStatement::Claim { condition, .. } => vec![condition],
        CheckedStatement::CountedRange { lower, upper, .. } => vec![lower, upper],
        CheckedStatement::Loop { .. }
        | CheckedStatement::Region { .. }
        | CheckedStatement::Break { .. } => Vec::new(),
    }
}

// ----------------------------------------------------------------------
// The body's control-flow graph
// ----------------------------------------------------------------------

type NodeId = usize;

/// The body completes normally and takes the loop's back edge.
const NORMAL_EXIT: NodeId = 0;
/// Control leaves the loop, or the function, without completing the body.
const LEAVES: NodeId = 1;

/// One statement of B, with the edges it takes.
struct FlowNode<'check> {
    statement: &'check CheckedStatement,
    successors: Vec<NodeId>,
    /// The edge this statement takes out of B, when it takes one. Computed
    /// where the enclosing loop and value-initializer stacks are in hand, so
    /// condition 2 reads a fact rather than re-deriving it.
    leaves: Option<&'static str>,
}

/// The body's own control-flow graph, over statements.
struct Flow<'check> {
    nodes: Vec<Option<FlowNode<'check>>>,
    entry: NodeId,
}

struct FlowBuilder<'check> {
    nodes: Vec<Option<FlowNode<'check>>>,
    /// The exit node of each loop opened inside B, innermost last.
    loop_exits: Vec<(u32, NodeId)>,
    /// The exit node of each value initializer of B enclosing the walk,
    /// innermost last. A `give` reaching one of these delivers inside B.
    initializer_exits: Vec<NodeId>,
    outer_loop: CheckedLoopId,
}

impl<'check> Flow<'check> {
    fn build(outer_loop: CheckedLoopId, body: &'check [CheckedStatement]) -> Self {
        let mut builder = FlowBuilder {
            // Two sinks, in fixed positions, so every edge has a target.
            nodes: vec![None, None],
            loop_exits: Vec::new(),
            initializer_exits: Vec::new(),
            outer_loop,
        };
        let entry = builder.block(body, NORMAL_EXIT);
        Self {
            nodes: builder.nodes,
            entry,
        }
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn successors(&self, node: NodeId) -> &[NodeId] {
        match &self.nodes[node] {
            Some(node) => &node.successors,
            None => &[],
        }
    }

    fn statement_nodes(&self) -> impl Iterator<Item = (NodeId, &FlowNode<'check>)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| node.as_ref().map(|node| (id, node)))
    }
}

impl<'check> FlowBuilder<'check> {
    fn new_node(&mut self, statement: &'check CheckedStatement) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Some(FlowNode {
            statement,
            successors: Vec::new(),
            leaves: None,
        }));
        id
    }

    fn set(&mut self, node: NodeId, successors: Vec<NodeId>, leaves: Option<&'static str>) {
        let entry = self.nodes[node]
            .as_mut()
            .expect("a statement node was just created here");
        entry.successors = successors;
        entry.leaves = leaves;
    }

    /// Wires one block back to front and returns the node control enters it at.
    fn block(&mut self, statements: &'check [CheckedStatement], next: NodeId) -> NodeId {
        let mut entry = next;
        for statement in statements.iter().rev() {
            entry = self.statement(statement, entry);
        }
        entry
    }

    fn statement(&mut self, statement: &'check CheckedStatement, next: NodeId) -> NodeId {
        match statement {
            CheckedStatement::Loop { id, body, .. } => {
                let node = self.new_node(statement);
                self.loop_exits.push((id.0, next));
                let entry = self.block(body, node);
                self.loop_exits.pop();
                self.set(node, vec![entry], None);
                node
            }
            CheckedStatement::CountedRange { id, body, .. } => {
                let node = self.new_node(statement);
                self.loop_exits.push((id.0, next));
                let entry = self.block(body, node);
                self.loop_exits.pop();
                // A counted range may execute zero iterations, so the loop head
                // also reaches its continuation directly.
                self.set(node, vec![entry, next], None);
                node
            }
            CheckedStatement::Region { body, .. } => {
                let node = self.new_node(statement);
                let entry = self.block(body, next);
                self.set(node, vec![entry], None);
                node
            }
            CheckedStatement::Match { arms, .. } => {
                let node = self.new_node(statement);
                let mut successors = Vec::new();
                for arm in arms {
                    successors.push(self.block(&arm.body, next));
                }
                if successors.is_empty() {
                    successors.push(next);
                }
                self.set(node, successors, None);
                node
            }
            CheckedStatement::ValueMatchLet { arms, .. } => {
                let node = self.new_node(statement);
                self.initializer_exits.push(next);
                let mut successors = Vec::new();
                for arm in arms {
                    successors.push(self.block(&arm.body, next));
                }
                self.initializer_exits.pop();
                if successors.is_empty() {
                    successors.push(next);
                }
                self.set(node, successors, None);
                node
            }
            CheckedStatement::Return { .. } => {
                let node = self.new_node(statement);
                self.set(node, vec![LEAVES], Some("a return"));
                node
            }
            // [GIVE-1] delivers to the innermost value initializer enclosing
            // the `give`. When that initializer is written inside B the
            // delivery reaches a binding of this same iteration and leaves
            // nothing; when the loop is written inside the initializer, the
            // `give` leaves the loop.
            CheckedStatement::Give { .. } => {
                let node = self.new_node(statement);
                match self.initializer_exits.last() {
                    Some(exit) => self.set(node, vec![*exit], None),
                    None => self.set(node, vec![LEAVES], Some("a give")),
                }
                node
            }
            // [ERR-3]: the `Err` edge reaches the function-return sink.
            CheckedStatement::PropagateLet { .. } => {
                let node = self.new_node(statement);
                self.set(node, vec![next, LEAVES], Some("a propagate"));
                node
            }
            CheckedStatement::Break { target, .. } => {
                let node = self.new_node(statement);
                let inner = (target.0 != self.outer_loop.0)
                    .then(|| {
                        self.loop_exits
                            .iter()
                            .rev()
                            .find(|(id, _)| *id == target.0)
                            .map(|(_, exit)| *exit)
                    })
                    .flatten();
                match inner {
                    Some(exit) => self.set(node, vec![exit], None),
                    None => self.set(node, vec![LEAVES], Some("a break")),
                }
                node
            }
            // A `claim` leaves the loop only when it is false, which is an
            // erroneous execution the rule accounts for rather than refuses, so
            // its trap edge is not a control edge here.
            CheckedStatement::Let { .. }
            | CheckedStatement::Set { .. }
            | CheckedStatement::Replace { .. }
            | CheckedStatement::Claim { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. } => {
                let node = self.new_node(statement);
                self.set(node, vec![next], None);
                node
            }
        }
    }
}

/// One node set, dense over the flow graph.
type NodeSet = Vec<bool>;

fn full(len: usize) -> NodeSet {
    vec![true; len]
}

fn intersect(into: &mut NodeSet, other: &NodeSet) {
    for (slot, keep) in into.iter_mut().zip(other) {
        *slot &= *keep;
    }
}

/// Dominators from the body's entry: `dom[n][d]` holds when every path from the
/// entry to n passes through d.
fn dominators(flow: &Flow<'_>) -> Vec<NodeSet> {
    let len = flow.len();
    let mut predecessors: Vec<Vec<NodeId>> = vec![Vec::new(); len];
    for node in 0..len {
        for successor in flow.successors(node) {
            predecessors[*successor].push(node);
        }
    }
    let mut dom = vec![full(len); len];
    dom[flow.entry] = vec![false; len];
    dom[flow.entry][flow.entry] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for node in 0..len {
            if node == flow.entry {
                continue;
            }
            let mut next = full(len);
            for predecessor in &predecessors[node] {
                let carried = dom[*predecessor].clone();
                intersect(&mut next, &carried);
            }
            next[node] = true;
            if next != dom[node] {
                dom[node] = next;
                changed = true;
            }
        }
    }
    dom
}

/// Post-dominators with respect to the body's *normal* completion only:
/// `pdom[n][d]` holds when every path from n to the back edge passes through d.
///
/// A node whose every continuation leaves the loop reaches the normal exit on
/// no path, so the intersection over its successors is the whole node set and
/// every node post-dominates it. That is the reading condition 1 needs: an
/// early typed exit written before the submission belongs to the prologue, and
/// condition 2 is what decides whether it is admitted.
fn post_dominators(flow: &Flow<'_>) -> Vec<NodeSet> {
    let len = flow.len();
    let mut pdom = vec![full(len); len];
    pdom[NORMAL_EXIT] = vec![false; len];
    pdom[NORMAL_EXIT][NORMAL_EXIT] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for node in 0..len {
            if node == NORMAL_EXIT {
                continue;
            }
            let mut next = full(len);
            for successor in flow.successors(node) {
                let carried = pdom[*successor].clone();
                intersect(&mut next, &carried);
            }
            next[node] = true;
            if next != pdom[node] {
                pdom[node] = next;
                changed = true;
            }
        }
    }
    pdom
}

// ----------------------------------------------------------------------
// The survey
// ----------------------------------------------------------------------

/// One place the body touches, with everything the disposition test reads.
struct Touched {
    place: ResolvedPlace,
    /// The first node that cites it, in source order of the walk.
    citation: NodePath,
    written: bool,
    /// The segments any footprint element or loan on it belongs to.
    in_prologue: bool,
    in_remainder: bool,
    /// Whether storage of this place's type could be replicated at all: a copy
    /// scalar, or a buffer or array whose element type is copy [OWN-1]. An
    /// opaque system nominal, a slice descriptor, and an affine aggregate
    /// cannot, whatever a coverage proof later establishes.
    replicable_shape: bool,
    /// A loan on it held by a `may-suspend` call, which this version reads as
    /// retained to `terminal`.
    retained_borrow: Option<NodePath>,
    /// An exclusive loan on it held by a call of the remainder.
    remainder_exclusive_loan: Option<NodePath>,
    exclusive_loan: bool,
}

/// One construction of B whose storage an implementation may reuse across
/// iterations.
struct Replicated {
    citation: NodePath,
    /// `None` when the element's [OWN-1] copy class is not resolved here.
    copy_elements: Option<bool>,
}

struct StagedSurvey<'check, 'run> {
    program: &'run Program<'check>,
    places: &'run PlaceMap,
    /// Bindings introduced anywhere inside the body, including the loop's own
    /// binder. Storage rooted in one of these is created fresh by every
    /// iteration and dies with it; everything else outlives the iteration.
    introduced: Vec<BindingId>,
    cut: NodePath,
    /// Which segment each flow node belongs to, dense by node id.
    segments: Vec<Option<Segment>>,
    dispositions: Vec<PlaceDisposition>,
    touched: Vec<Touched>,
    replicated: Vec<Replicated>,
    form_refusal: Option<&'static str>,
    unresolved: Option<NodePath>,
    not_replicable: Option<NodePath>,
    /// The first edge that leaves the loop from the remainder, which is
    /// condition 2's whole content.
    exit_in_remainder: Option<(&'static str, Option<NodePath>)>,
}

impl<'check> StagedSurvey<'check, '_> {
    /// Condition 1, as a dominator and post-dominator query on the body's own
    /// graph.
    fn classify_segments(
        &mut self,
        flow: &Flow<'check>,
        cut_statement: &'check CheckedStatement,
    ) -> Option<StagedDenial> {
        let Some(cut) = flow
            .statement_nodes()
            .find(|(_, node)| core::ptr::eq(node.statement, cut_statement))
            .map(|(id, _)| id)
        else {
            return Some(StagedDenial::NoCut {
                reason: "the body's first submission is not a statement of its control-flow graph",
                statement: Some(self.cut.clone()),
            });
        };
        // A submission written inside a loop of B runs several times per
        // iteration, so the body has no single cut and the shape this condition
        // asks for does not hold.
        if self.is_inside_inner_loop(flow, cut) {
            return Some(StagedDenial::NoCut {
                reason: "the body's first submission is written inside a loop of the body",
                statement: Some(self.cut.clone()),
            });
        }
        let dom = dominators(flow);
        let pdom = post_dominators(flow);
        self.segments = vec![None; flow.len()];
        for (id, node) in flow.statement_nodes() {
            let segment = if id == cut {
                Segment::Prologue
            } else if dom[id][cut] {
                Segment::Remainder
            } else if pdom[id][cut] {
                Segment::Prologue
            } else {
                return Some(StagedDenial::NoCut {
                    reason: "a statement of the body neither executes before the submission on every path nor is reached only through it",
                    statement: statement_citation(node.statement),
                });
            };
            self.segments[id] = Some(segment);
        }
        None
    }

    /// Whether one node lies inside a loop written in the body. The graph's own
    /// answer: some loop head of B dominates it and is reached again from it.
    fn is_inside_inner_loop(&self, flow: &Flow<'check>, node: NodeId) -> bool {
        flow.statement_nodes().any(|(head, entry)| {
            matches!(
                entry.statement,
                CheckedStatement::Loop { .. } | CheckedStatement::CountedRange { .. }
            ) && head != node
                && reaches(flow, head, node)
                && reaches(flow, node, head)
        })
    }

    /// Every statement of the body, in its segment.
    fn walk(&mut self, flow: &Flow<'check>) {
        // Source order is the order the ledger reads best, and the flow builder
        // walks blocks back to front, so the nodes are visited by their
        // statements' citation order instead of by node id.
        let mut nodes: Vec<(NodeId, &FlowNode<'check>)> = flow.statement_nodes().collect();
        nodes.sort_by_key(|(id, node)| {
            (
                statement_citation(node.statement)
                    .map(|path| path.components().to_vec())
                    .unwrap_or_default(),
                *id,
            )
        });
        for (id, node) in nodes {
            let Some(segment) = self.segments[id] else {
                continue;
            };
            self.statement(node, segment);
        }
    }

    fn statement(&mut self, node: &FlowNode<'check>, segment: Segment) {
        // Condition 2 reads the edge the flow builder already classified.
        if let (Segment::Remainder, Some(edge)) = (segment, node.leaves)
            && self.exit_in_remainder.is_none()
        {
            self.exit_in_remainder = Some((edge, statement_citation(node.statement)));
        }
        let statement = node.statement;
        let citation = statement_citation(statement).unwrap_or_else(|| self.cut.clone());
        match statement {
            CheckedStatement::Let { value, .. } => {
                self.construction(value, &citation);
                self.value(value, &citation, segment);
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                let mut footprint = Footprint::default();
                set_target_place(self.places, target, &citation, &mut footprint, false);
                self.record(&footprint, segment, false);
                self.value(value, &citation, segment);
            }
            CheckedStatement::PropagateLet { scrutinee, .. }
            | CheckedStatement::Match { scrutinee, .. }
            | CheckedStatement::ValueMatchLet { scrutinee, .. } => {
                self.value(scrutinee, &citation, segment);
            }
            CheckedStatement::Return { value, .. } | CheckedStatement::Give { value, .. } => {
                self.value(value, &citation, segment);
            }
            CheckedStatement::Claim { condition, .. } => {
                self.value(condition, &citation, segment);
            }
            CheckedStatement::CountedRange { lower, upper, .. } => {
                self.value(lower, &citation, segment);
                self.value(upper, &citation, segment);
            }
            CheckedStatement::Loop { .. }
            | CheckedStatement::Region { .. }
            | CheckedStatement::Break { .. } => {}
            // An expression statement is a call [GRAM-4] whose reach no row
            // projects onto an actual, and a discarded one carries its own
            // [STOR-3] release; admitting either needs that release classified
            // first, so both deny here exactly as they do in a window.
            CheckedStatement::Evaluate(_) => self.refuse_form("an expression statement"),
            CheckedStatement::DropExpression { .. } => {
                self.refuse_form("a discarded expression statement");
            }
        }
    }

    /// One value expression: its call footprints, the caller storage its own
    /// operands read, and the places it moves out of.
    fn value(&mut self, value: &'check CheckedExpression, citation: &NodePath, segment: Segment) {
        // A call's argument borrows carry their loans through the parameter
        // modes of the [EFF-2] projection below, and [GRAM-9] makes those
        // arguments atoms, so a value that is one call hides no bare borrow.
        // Every other value may form a borrow only of iteration-own storage,
        // where no loan is needed: a written borrow's shared-or-uniq mode is
        // erased from the checked tree, so the [OWN-5] loan it would hold
        // cannot be stated, and admitting one unstated would widen permission.
        if call_projection(value).is_none() {
            let introduced = &self.introduced;
            let admitted =
                borrows_only_iteration_own(self.places, value, &|place: &ResolvedPlace| {
                    is_iteration_own(introduced, place)
                });
            if !admitted {
                self.refuse_form(
                    "a statement that forms a borrow of storage the iteration does not introduce",
                );
                return;
            }
            let mut footprint = Footprint::default();
            collect_operand_reads(self.places, value, citation, &mut footprint);
            self.record(&footprint, segment, false);
        }
        self.calls(value, segment);
        let mut moved = Footprint::default();
        collect_consumed_places(self.places, value, citation, &mut moved);
        self.record(&moved, segment, false);
    }

    /// Every call one expression tree makes, with its [EFF-2] projection, and
    /// whether that call may suspend.
    fn calls(&mut self, expression: &'check CheckedExpression, segment: Segment) {
        let may_suspend = match expression {
            CheckedExpression::UserCall { function, .. } => {
                Some(self.program.target_action(*function).may_suspend())
            }
            CheckedExpression::SystemCall { target_action, .. } => {
                Some(target_action.may_suspend())
            }
            _ => None,
        };
        if let Some(may_suspend) = may_suspend
            && let Some(projection) = call_projection(expression)
        {
            let footprint = self.program.footprint(self.places, &projection);
            self.record(&footprint, segment, may_suspend);
        }
        for child in super::model::expression_children(expression) {
            self.calls(child, segment);
        }
    }

    /// One projected footprint, against every condition that reads places.
    fn record(&mut self, footprint: &Footprint, segment: Segment, may_suspend: bool) {
        for argument in [&footprint.unresolved, &footprint.operand_unresolved]
            .into_iter()
            .flatten()
        {
            self.unresolved.get_or_insert(argument.clone());
        }
        for loan in &footprint.loans {
            let exclusive = loan.strength == LoanStrength::Exclusive;
            let entry = self.touch(&loan.place, &loan.argument, segment);
            let Some(entry) = entry else { continue };
            entry.exclusive_loan |= exclusive;
            if may_suspend {
                entry.retained_borrow.get_or_insert(loan.argument.clone());
            }
            if exclusive && matches!(segment, Segment::Remainder) {
                entry
                    .remainder_exclusive_loan
                    .get_or_insert(loan.argument.clone());
            }
        }
        for (written, accesses) in [
            (true, &footprint.writes),
            (false, &footprint.reads),
            (false, &footprint.operand_reads),
        ] {
            for access in accesses {
                match access {
                    Access::Place { place, argument } => {
                        if let Some(entry) = self.touch(place, argument, segment) {
                            entry.written |= written;
                        }
                    }
                    // Two iterations allocating into one enclosing region both
                    // append to its allocation list, which is one place with no
                    // actual to project onto and no disposition to carry.
                    Access::Arena { call, .. } => {
                        self.unresolved.get_or_insert(call.clone());
                    }
                }
            }
        }
    }

    /// Records one touch of a place rooted outside the loop, returning its
    /// entry. Iteration-own storage carries no disposition and returns `None`.
    fn touch(
        &mut self,
        place: &ResolvedPlace,
        citation: &NodePath,
        segment: Segment,
    ) -> Option<&mut Touched> {
        if is_iteration_own(&self.introduced, place) {
            return None;
        }
        let index = match self.touched.iter().position(|entry| entry.place == *place) {
            Some(index) => index,
            None => {
                self.touched.push(Touched {
                    replicable_shape: self.is_replicable_shape(place),
                    place: place.clone(),
                    citation: citation.clone(),
                    written: false,
                    in_prologue: false,
                    in_remainder: false,
                    retained_borrow: None,
                    remainder_exclusive_loan: None,
                    exclusive_loan: false,
                });
                self.touched.len() - 1
            }
        };
        let entry = &mut self.touched[index];
        match segment {
            Segment::Prologue => entry.in_prologue = true,
            Segment::Remainder => entry.in_remainder = true,
        }
        Some(entry)
    }

    /// Whether storage of one place's type could be replicated at all.
    ///
    /// A whole binding is read through the place map's own type record; a field
    /// selection is not, because this judgment holds no field type table, and
    /// the fail-closed answer for an unresolved type is that it cannot be
    /// replicated.
    fn is_replicable_shape(&self, place: &ResolvedPlace) -> bool {
        if !place.fields.is_empty() {
            return false;
        }
        let PlaceRoot::Binding(binding) = place.root else {
            return false;
        };
        let Some(ty) = self.places.summary(binding).and_then(|summary| summary.ty) else {
            return false;
        };
        match ty {
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => true,
            CheckedType::Array { element, .. } | CheckedType::Buffer { element } => {
                is_copy_element(element)
            }
            CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_)
            | CheckedType::Nominal(_)
            | CheckedType::Slice { .. } => false,
        }
    }

    /// Condition 6's other half: one construction of B whose storage an
    /// implementation may reuse across iterations.
    fn construction(&mut self, value: &'check CheckedExpression, citation: &NodePath) {
        let copy_elements = match value {
            CheckedExpression::BufferFill { element, .. } => Some(is_copy_element(*element)),
            CheckedExpression::ArrayFill { ty, .. } => match ty {
                CheckedType::Array { element, .. } => Some(is_copy_element(*element)),
                _ => None,
            },
            // `buffer_vacant` fills an interned `Option<T>` nominal whose
            // [OWN-1] class this judgment does not resolve, so it fails closed.
            CheckedExpression::BufferVacant { .. } => Some(false),
            _ => return,
        };
        self.replicated.push(Replicated {
            citation: citation.clone(),
            copy_elements,
        });
        if copy_elements != Some(true) {
            self.not_replicable.get_or_insert(citation.clone());
        }
    }

    fn refuse_form(&mut self, form: &'static str) {
        self.form_refusal.get_or_insert(form);
    }

    /// The conditions in their numbered order, with the fail-closed form
    /// refusal ahead of all of them.
    ///
    /// A statement whose footprint this judgment does not compute has no
    /// condition-3, condition-4, or condition-5 answer to give, so a body with
    /// several defects reports the unclassified form first, which is the honest
    /// report.
    fn finish(mut self, form: &'static str, cut: NodePath) -> StagedPermission {
        for entry in &self.touched {
            let disposition = disposition_of(entry);
            self.dispositions.push(PlaceDisposition {
                citation: entry.citation.clone(),
                disposition,
                reason: disposition_reason(entry, disposition),
            });
        }
        for entry in &self.replicated {
            let replicable = entry.copy_elements == Some(true);
            self.dispositions.push(PlaceDisposition {
                citation: entry.citation.clone(),
                disposition: if replicable {
                    Disposition::Replicated
                } else {
                    Disposition::Denied
                },
                reason: if replicable {
                    "iteration-own storage with copy elements, which an implementation may give each in-flight iteration its own of"
                } else {
                    "iteration-own storage whose element type is not a resolved copy type, so it cannot be replicated"
                },
            });
        }
        let verdict = match self.denial() {
            Some(denial) => StagedVerdict::Denied(denial),
            None => StagedVerdict::Permitted,
        };
        StagedPermission {
            form,
            cut,
            verdict,
            dispositions: self.dispositions,
        }
    }

    fn denial(&self) -> Option<StagedDenial> {
        if let Some(form) = self.form_refusal {
            return Some(StagedDenial::BodyForm { form });
        }
        if let Some((edge, statement)) = &self.exit_in_remainder {
            return Some(StagedDenial::ExitInRemainder {
                edge,
                statement: statement.clone(),
            });
        }
        for entry in &self.touched {
            if entry.written
                && let Some(argument) = &entry.retained_borrow
            {
                return Some(StagedDenial::RetainedBorrow {
                    argument: argument.clone(),
                    replicable_shape: entry.replicable_shape,
                });
            }
        }
        for entry in &self.touched {
            if let Some(argument) = &entry.remainder_exclusive_loan {
                return Some(StagedDenial::RemainderExclusiveLoan {
                    argument: argument.clone(),
                    replicable_shape: entry.replicable_shape,
                });
            }
        }
        for entry in &self.touched {
            if disposition_of(entry) == Disposition::Denied {
                return Some(StagedDenial::NoDisposition {
                    argument: entry.citation.clone(),
                });
            }
        }
        if let Some(statement) = &self.not_replicable {
            return Some(StagedDenial::NotReplicable {
                statement: statement.clone(),
            });
        }
        self.unresolved
            .as_ref()
            .map(|argument| StagedDenial::Unresolved {
                argument: argument.clone(),
            })
    }
}

/// Condition 5's table, over one place rooted outside the loop.
fn disposition_of(entry: &Touched) -> Disposition {
    // Conditions 3 and 4 are stated over places, so their failures are the
    // fourth disposition rather than a separate column: a place a `may-suspend`
    // call retains a borrow on and the body writes, or one a call of the
    // remainder holds an exclusive loan on, has no safe disposition in this
    // version.
    if entry.written && entry.retained_borrow.is_some() {
        return Disposition::Denied;
    }
    if entry.remainder_exclusive_loan.is_some() {
        return Disposition::Denied;
    }
    if !entry.written && !entry.exclusive_loan {
        return Disposition::ReadOnly;
    }
    // A loan a `may-suspend` call holds outlives the submission, so a place
    // carrying one is not confined to the prologue however its footprint reads.
    let retained_past_cut = entry.retained_borrow.is_some();
    if entry.in_prologue && !entry.in_remainder && !retained_past_cut {
        return Disposition::Serialized(Segment::Prologue);
    }
    if entry.in_remainder && !entry.in_prologue {
        return Disposition::Serialized(Segment::Remainder);
    }
    Disposition::Denied
}

fn disposition_reason(entry: &Touched, disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::ReadOnly => {
            "no footprint of the body writes it and every loan on it is shared"
        }
        Disposition::Serialized(Segment::Prologue) => {
            "every footprint element and loan touching it belongs to the prologue, and prologues run in index order without overlapping"
        }
        Disposition::Serialized(Segment::Remainder) => {
            "every footprint element and loan touching it belongs to the remainder, whose writes to storage rooted outside the loop commit in index order"
        }
        Disposition::Replicated => {
            "iteration-own storage with copy elements, which an implementation may give each in-flight iteration its own of"
        }
        Disposition::Denied => {
            if entry.written && entry.retained_borrow.is_some() {
                if entry.replicable_shape {
                    "the body writes it and a may-suspend call retains a borrow of it past its own submission"
                } else {
                    "the body writes it through a retained borrow and its type carries one position, so no iteration can be given its own"
                }
            } else if entry.remainder_exclusive_loan.is_some() {
                "a call of the remainder holds an exclusive loan on it, and two remainders coexist"
            } else if entry.in_prologue && entry.in_remainder {
                "the body reaches it on both sides of the cut, so no single segment serializes it"
            } else {
                "the body writes it and no disposition of this rule covers it"
            }
        }
    }
}

/// Whether a resolved place is storage this iteration introduced.
fn is_iteration_own(introduced: &[BindingId], place: &ResolvedPlace) -> bool {
    match place.root {
        PlaceRoot::Binding(binding) => introduced.contains(&binding),
        // A named const [CONST-2] is enclosing storage. Nothing writes one, so
        // this arm exists to keep the classification total.
        PlaceRoot::Constant(_) => false,
    }
}

/// [OWN-1]'s copy classification over one flat element domain: primitives and
/// tag-only enums copy; an affine aggregate element does not.
const fn is_copy_element(element: CheckedFlatElement) -> bool {
    match element {
        CheckedFlatElement::Unit
        | CheckedFlatElement::Bool
        | CheckedFlatElement::Integer(_)
        | CheckedFlatElement::Float(_)
        | CheckedFlatElement::GenericInt(_)
        | CheckedFlatElement::GenericFloat(_)
        | CheckedFlatElement::TagOnlyNominal(_) => true,
        CheckedFlatElement::Nominal(_) => false,
    }
}

/// The source node one statement is cited at, when the checked tree carries
/// one. A `loop_stmt`, a `region_stmt`, a `break_stmt`, and a `match_stmt` keep
/// no node path of their own; a `match` is cited at its scrutinee instead, and
/// the remaining three are cited at the loop's cut by the caller.
fn statement_citation(statement: &CheckedStatement) -> Option<NodePath> {
    match statement {
        CheckedStatement::Let { node_path, .. }
        | CheckedStatement::PropagateLet { node_path, .. }
        | CheckedStatement::Set { node_path, .. }
        | CheckedStatement::Replace { node_path, .. }
        | CheckedStatement::Return { node_path, .. }
        | CheckedStatement::Give { node_path, .. }
        | CheckedStatement::ValueMatchLet { node_path, .. }
        | CheckedStatement::CountedRange { node_path, .. } => Some(node_path.clone()),
        CheckedStatement::Match { scrutinee, .. } => expression_citation(scrutinee),
        CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
            expression_citation(value)
        }
        CheckedStatement::Claim { condition, .. } => expression_citation(condition),
        CheckedStatement::Loop { .. }
        | CheckedStatement::Break { .. }
        | CheckedStatement::Region { .. } => None,
    }
}

/// The first source node one expression tree carries, in evaluation order.
fn expression_citation(expression: &CheckedExpression) -> Option<NodePath> {
    let own = match expression {
        CheckedExpression::UserCall { call, .. } | CheckedExpression::SystemCall { call, .. } => {
            Some(call.clone())
        }
        CheckedExpression::Binding { carrier, .. }
        | CheckedExpression::Project { carrier, .. }
        | CheckedExpression::IntegerOperation { carrier, .. }
        | CheckedExpression::FloatOperation { carrier, .. }
        | CheckedExpression::NumericConversion { carrier, .. }
        | CheckedExpression::Reinterpret { carrier, .. }
        | CheckedExpression::BooleanOperation { carrier, .. }
        | CheckedExpression::EnumEquality { carrier, .. }
        | CheckedExpression::ArrayFill { carrier, .. }
        | CheckedExpression::ArrayIndex { carrier, .. }
        | CheckedExpression::BufferFill { carrier, .. }
        | CheckedExpression::BufferVacant { carrier, .. }
        | CheckedExpression::BufferFits { carrier, .. }
        | CheckedExpression::BufferIndex { carrier, .. }
        | CheckedExpression::SliceOf { carrier, .. }
        | CheckedExpression::SliceIndex { carrier, .. }
        | CheckedExpression::BorrowBuffer { carrier, .. }
        | CheckedExpression::BorrowAddressed { carrier, .. }
        | CheckedExpression::BorrowBox { carrier, .. }
        | CheckedExpression::BorrowSystemResource { carrier, .. }
        | CheckedExpression::ReborrowAddressed { carrier, .. }
        | CheckedExpression::DerefAddressed { carrier, .. }
        | CheckedExpression::BoxNew { carrier, .. }
        | CheckedExpression::BoxDeref { carrier, .. }
        | CheckedExpression::ArenaNew { carrier, .. }
        | CheckedExpression::ArenaDeref { carrier, .. } => Some(carrier.clone()),
        _ => None,
    };
    if own.is_some() {
        return own;
    }
    super::model::expression_children(expression)
        .into_iter()
        .find_map(expression_citation)
}

/// Whether one node reaches another over the flow graph's edges.
fn reaches(flow: &Flow<'_>, from: NodeId, to: NodeId) -> bool {
    let mut seen = vec![false; flow.len()];
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        for successor in flow.successors(node) {
            if *successor == to {
                return true;
            }
            if !seen[*successor] {
                seen[*successor] = true;
                stack.push(*successor);
            }
        }
    }
    false
}
