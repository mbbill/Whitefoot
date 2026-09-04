//! The loop permission judgment: whether the iterations of one counted `for`
//! may be executed with overlapping execution, and whether the one value they
//! carry may be recombined across them.
//!
//! The window judgment next door reads a pair of *statements*, so two
//! executions of one statement are never a pair and a counted loop gets
//! nothing however independent its iterations are. This judgment reads the
//! loop itself. It refuses nothing, changes no acceptance, and grants no
//! lowering by itself; it records, per counted loop, whether overlapping its
//! iterations is permitted. A permitted loop is actualizable, full stop.
//!
//! # What is judged
//!
//! The unit is one `for_stmt` L with body B. Writing an *iteration-own* place
//! for one rooted in a binding B itself introduces, permission holds exactly
//! when all four conditions hold:
//!
//! 1. **One accumulator, or none.** The only storage outliving an iteration
//!    that B writes as a whole binding is a single accumulator `a`, every
//!    write of which is `set a = a (+) e` or `set a = e (+) a` for one
//!    exactly-associative operation, and `a` is read nowhere else in B.
//! 2. **No shared writable footprint.** Every place B writes is iteration-own,
//!    that accumulator, or one element selected by L's own counted binder at
//!    an already-discharged [OP-4] site whose retained exact value is one
//!    nonconstant affine map `a*i+b` of L's binder. Since `a != 0`, distinct
//!    iterations select distinct elements. Every other element
//!    write. A same-map element read of that root is also refined to the same
//!    per-iteration cell; a different map or whole-root read still denies. A
//!    field of enclosing storage, a callee row projecting onto enclosing
//!    storage, an arena append, and every write this judgment cannot resolve
//!    all deny. The mapped root may be owned directly or reached through the
//!    live usable `&uniq` holder that made the `set` target writable.
//! 3. **Complete target summaries.** Every call and derived release in B
//!    identifies its target action. Ordinary effects and loans have already
//!    denied every conflicting cross-iteration access. A may-suspend target
//!    keeps the permission but requires a completion-capable loop actualizer;
//!    this version otherwise leaves the loop sequential.
//! 4. **No exit edge.** No `return`, `give`, `propagate` `Err` edge, or
//!    `break` naming L or an enclosing loop leaves the loop, so every
//!    iteration of the whole range runs.
//!
//! Source proof statements do not add a fifth condition. Their predicates have
//! already been checked against the facts at their source positions before
//! this judgment runs. They are erased before lowering, so they have no
//! runtime evaluation, footprint, exit edge, or scheduler-visible event. A
//! failed proof rejects the program rather than introducing a runtime path.
//!
//! # Why regrouping is admissible here and nowhere wider
//!
//! The window rule never regroups anything: the combination tree of a pair is
//! written in the source, and permission only overlaps its two independent
//! halves. A loop rule is categorically stronger, because it lets the
//! implementation choose the tree. Two facts carry it:
//!
//! - Each admitted operation is a *total* function on its type's complete
//!   value set, carries no per-application obligation, and is associative
//!   there: `+wrap` and `*wrap` are the ring operations of the integers modulo
//!   two to the width, `iand`, `ior`, and `ixor` are the meet, join, and group
//!   operations of the bit vector, `imin` and `imax` are the meet and join of
//!   that type's total order, and `band`, `bor`, and `bxor` are the
//!   two-element cases of the same three. Fixing the leaf order and the leaf
//!   multiset therefore fixes the value of *every* binary tree over them.
//!   **No float operation is admitted.** `fadd.strict` is the pointed example:
//!   floating-point addition is not associative, so a schedule that regrouped
//!   it would move a published byte. `+`, `+defined`, `+checked`, and `+sat`
//!   are associative over the integers and are still absent, because each
//!   application carries an obligation, a `Result` route, or a clamp that
//!   regrouping moves.
//! - No entailment fact established inside one counted iteration survives to a
//!   later head or to the continuation, so a regrouped accumulator can falsify
//!   no surviving proof.
//!
//! **Invariant.** Like the window judgment, this one consults typing, declared
//! effect rows, resolved places [OWN-5, OWN-7], and the statement graph's exit
//! edges. For the affine-map form it additionally consumes the successful
//! [OP-4] disposition and exact affine value image already retained on the
//! checked function; it never repeats the bounds proof or reconstructs a value
//! from parser shape. Injectivity is a fixed check: [FN-1] gives every
//! iteration a distinct compiler-owned binder, and multiplication by one
//! retained nonzero integer coefficient preserves distinctness.
//!
//! # Why counting an accumulator's reads by binding is complete
//!
//! Condition 1 counts the read occurrences of the accumulator *binding*, and a
//! read spelled `deref(h)` names the holder rather than the storage. Two facts
//! close that gap without a second relation. A holder bound inside the body is
//! refused before any count runs: a written borrow's loan strength is erased
//! from the checked tree, so the body admits no borrow-forming statement at
//! all and no in-body holder ever exists to read through. A holder bound
//! outside the body leaves only two spellings: writing the accumulator by name
//! while that borrow is live is an [OWN-5] borrow conflict and the program is
//! rejected, and writing *through* the holder is either a proved element map
//! under condition 2 or a non-accumulator shared write. Asking instead whether
//! any holder in the function reaches the accumulator would be flow-insensitive
//! and would refuse a sound reduction whose result is borrowed after the loop.
//!
//! # The one-sided reading
//!
//! Every form this judgment has not classified refuses, and the statement
//! match is exhaustive for that reason: a missed statement would contribute an
//! empty footprint and *widen* permission, which is the one direction it must
//! never fail in. A form refusal is reported ahead of the numbered conditions,
//! because a statement whose footprint is unknown has no condition-1 or
//! condition-2 answer to give.

use super::entailment::{ObligationFamily, ObligationOutcome, ProvedAffineIndexMap};
use super::model::{
    BindingId, CheckedArrayRoot, CheckedBooleanOperation, CheckedExpression, CheckedFunction,
    CheckedIntegerOperation, CheckedLoopId, CheckedSetTarget, CheckedSliceSource, CheckedStatement,
    expression_children,
};
use super::permission::{
    Access, Footprint, LoanStrength, Program, call_projection, collect_consumed_places,
    rooted_place, set_target_place, slice_source_place, visit_read_bindings,
};
use super::places::{PlaceMap, PlaceRoot, ResolvedPlace};
use crate::NodePath;

/// The judgment's outcome for one counted loop, and the advice that outlives
/// a refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoopPermission {
    /// The `for_stmt` node.
    pub(crate) statement: NodePath,
    pub(crate) verdict: LoopVerdict,
    /// The operations the body's accumulators combine under, in source order
    /// and without repeats. A loop with no carried value has none.
    pub(crate) combines: Vec<&'static str>,
    /// Whether a recursive split of this loop's index range, hand-written by
    /// the writer, would be eligible where the loop itself is refused. This is
    /// the advice the ledger prints for a refused loop; it is never true for a
    /// permitted one, which needs no rewrite.
    pub(crate) advises_split: bool,
    /// What actualizing this permission needs from the judgment, present for a
    /// permitted non-suspending independent map or one-accumulator reduction.
    ///
    /// The judgment does not decide that anything is emitted: lowering reads
    /// this, applies its own emission conditions, and may still decline. The
    /// verdict above is the same either way.
    pub(crate) actualization: Option<LoopActualization>,
}

/// The two disjoint actualization shapes produced by the counted judgment.
///
/// An independent map carries no invented accumulator or combine. A reduction
/// names the one real source binding and the closed operation which recombines
/// it. Everything else lowering needs — captures, frame cost, and body weight
/// — is a property of the emitted shape rather than of the judgment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopActualization {
    IndependentMap,
    Reduction {
        accumulator: BindingId,
        combine: LoopCombine,
    },
}

/// The closed set of operations an accumulator may be combined under: exactly
/// the exactly-associative ones, named once so the judgment, the ledger, and
/// the emitted combination tree cannot hold three drifting copies of it.
///
/// This is its own enumeration rather than a checked operation carried around,
/// because every consumer needs the set to be *closed*: a total spelling, a
/// total identity element, and no arm that could be reached by an operation the
/// rule does not admit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopCombine {
    AddWrap,
    MultiplyWrap,
    BitAnd,
    BitOr,
    BitXor,
    Minimum,
    Maximum,
    And,
    Or,
    ExclusiveOr,
}

impl LoopCombine {
    /// The [OP-1] spelling the ledger prints.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::AddWrap => "+wrap",
            Self::MultiplyWrap => "*wrap",
            Self::BitAnd => "iand",
            Self::BitOr => "ior",
            Self::BitXor => "ixor",
            Self::Minimum => "imin",
            Self::Maximum => "imax",
            Self::And => "band",
            Self::Or => "bor",
            Self::ExclusiveOr => "bxor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LoopVerdict {
    /// Permission holds, so the loop's iterations may be overlapped and its
    /// accumulator recombined. Source proof statements are already checked and
    /// erase before this permission can be actualized.
    PermittedEligible,
    Denied(LoopDenial),
}

impl LoopVerdict {
    pub(crate) const fn is_permitted(&self) -> bool {
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

/// Why permission does not hold for one counted loop. Each variant names
/// exactly one condition of the judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LoopDenial {
    /// Condition 1: a write into storage outliving the iteration whose value
    /// is not that storage combined under an admitted operation.
    NotAReduction { statement: NodePath },
    /// Condition 1: the body carries more than one accumulator. The split
    /// advice survives this one: a hand-written recursion may return an
    /// aggregate, which this version does not synthesize.
    ManyAccumulators { accumulators: usize },
    /// Condition 1: the accumulator is read outside its own combine, so what
    /// a later iteration sees is the running total.
    AccumulatorRead { statement: NodePath, reads: usize },
    /// Condition 2: a written place that is neither iteration-own storage nor
    /// the accumulator.
    SharedWrite { argument: NodePath },
    /// Condition 2, the loans half: an iteration's argument borrow holds an
    /// exclusive [OWN-5] loan on storage the iteration does not introduce, so
    /// two overlapped iterations would hold two usable `&uniq` borrows of one
    /// place, whatever the callee's row declares.
    Loan { argument: NodePath },
    /// Condition 2, fail closed: a written place this judgment cannot
    /// resolve. An unresolved element overlaps every place, so it denies.
    UnresolvedWrite { argument: NodePath },
    /// Condition 2, fail closed: a body statement form whose footprint this
    /// judgment does not compute.
    BodyForm { form: &'static str },
    /// Condition 4: an edge leaves the loop.
    Exit { edge: &'static str },
}

impl LoopDenial {
    /// The judgment condition this denial cites. The permission ledger prints
    /// it and the judgment tests assert it; acceptance never reads it.
    pub(crate) const fn condition(&self) -> u8 {
        match self {
            Self::NotAReduction { .. }
            | Self::ManyAccumulators { .. }
            | Self::AccumulatorRead { .. } => 1,
            Self::SharedWrite { .. }
            | Self::Loan { .. }
            | Self::UnresolvedWrite { .. }
            | Self::BodyForm { .. } => 2,
            Self::Exit { .. } => 4,
        }
    }
}

/// The verdict of every counted loop of one function, in source order.
///
/// `eligible_pairs` are the statement paths of the pairs the window judgment
/// already found eligible in this function. A loop containing one of them
/// already has parallelism a writer can see, so it is never additionally told
/// to become a recursion. The *verdict* does not read them: [PAR-2] judges the
/// loop, not what a writer could put inside it.
pub(crate) fn judge_loops<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    function: &'check CheckedFunction,
    eligible_pairs: &[NodePath],
) -> Vec<LoopPermission> {
    let mut judged = Vec::new();
    collect(
        program,
        places,
        &function.entailment.obligations,
        &function.body,
        &mut judged,
    );
    for loop_permission in &mut judged {
        if eligible_pairs
            .iter()
            .any(|pair| encloses(&loop_permission.statement, pair))
        {
            loop_permission.advises_split = false;
        }
    }
    judged
}

fn collect<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    obligations: &'check [ObligationOutcome],
    statements: &'check [CheckedStatement],
    judged: &mut Vec<LoopPermission>,
) {
    for statement in statements {
        if let CheckedStatement::CountedRange {
            id,
            node_path,
            binder,
            body,
            ..
        } = statement
        {
            judged.push(judge(
                program,
                places,
                obligations,
                node_path.clone(),
                *id,
                *binder,
                body,
            ));
        }
        for nested in nested_bodies(statement) {
            collect(program, places, obligations, nested, judged);
        }
    }
}

/// Whether one statement lies inside another, by node path prefix.
fn encloses(outer: &NodePath, inner: &NodePath) -> bool {
    let outer = outer.components();
    let inner = inner.components();
    inner.len() > outer.len() && inner.starts_with(outer)
}

fn judge<'check>(
    program: &Program<'check>,
    places: &PlaceMap,
    obligations: &'check [ObligationOutcome],
    statement: NodePath,
    id: CheckedLoopId,
    binder: BindingId,
    body: &'check [CheckedStatement],
) -> LoopPermission {
    let mut survey = Survey {
        program,
        places,
        obligations,
        outer_loop: id,
        introduced: vec![binder],
        inner_loops: Vec::new(),
        reads: Vec::new(),
        accumulates: Vec::new(),
        carried: None,
        shared: None,
        loan: None,
        call_loans: Vec::new(),
        unresolved: None,
        element_ranges: Vec::new(),
        element_reads: Vec::new(),
        form: None,
        may_suspend: false,
        exit: None,
    };
    survey.introduce(body);
    survey.walk(body, 0);
    survey.finish(statement)
}

/// One write whose already-checked subscript value is `a*i+b`, where `i` is
/// the counted binder of the loop under judgment and `a != 0`. The successful
/// OP-4 outcome establishes that the selected element is inside its
/// collection; the affine image establishes that two iterations select
/// distinct elements.
struct ProvenElementRange {
    /// The binding written at the subscript root. Keeping source identity
    /// beside resolved-place identity makes an alias read fail closed even
    /// when it reaches the same collection.
    binding: BindingId,
    place: ResolvedPlace,
    statement: NodePath,
    map: ProvedAffineIndexMap,
}

/// One already-proved element read whose exact offset is the counted loop's
/// affine map. Unlike a published source fact this is only consumer evidence:
/// permission uses it once to compare read and write ranges, then lowering
/// forgets it.
struct ProvenElementRead {
    binding: BindingId,
    place: ResolvedPlace,
    map: ProvedAffineIndexMap,
}

/// One source read occurrence and the place reached by that spelling.
///
/// The binding remains the condition-1 accumulator identity. The resolved
/// place is condition 2's collection identity: sibling fields of one struct
/// are distinct, while a whole-parent or alias access overlaps the mapped
/// collection and must be accounted for.
struct ReadOccurrence {
    binding: BindingId,
    place: ResolvedPlace,
}

/// One accepted accumulate statement: `set a = a (+) e` with `(+)` admitted.
struct Accumulate {
    binding: BindingId,
    combine: LoopCombine,
    statement: NodePath,
}

struct Survey<'check, 'run> {
    program: &'run Program<'check>,
    places: &'run PlaceMap,
    /// Successful source obligations already computed by ENT. Permission
    /// consumes their disposition by source-node identity and never reruns
    /// the underlying proof.
    obligations: &'run [ObligationOutcome],
    /// The counted loop under judgment, so a `break` that closes it is told
    /// apart from one that closes a loop opened inside it.
    outer_loop: CheckedLoopId,
    /// Bindings introduced anywhere inside the body, including the loop's own
    /// binder. Storage rooted in one of these is created fresh by every
    /// iteration and dies with it; everything else outlives the iteration.
    introduced: Vec<BindingId>,
    inner_loops: Vec<u32>,
    /// Every read occurrence, with multiplicity and resolved place.
    reads: Vec<ReadOccurrence>,
    accumulates: Vec<Accumulate>,
    carried: Option<NodePath>,
    shared: Option<NodePath>,
    loan: Option<NodePath>,
    /// Every loan formed by a body call, including shared loans. The ordinary
    /// external-loan rule below already refuses every exclusive one; exact
    /// maps additionally refuse either strength when it overlaps their root.
    call_loans: Vec<super::permission::Loan>,
    unresolved: Option<NodePath>,
    /// Proven injective affine element writes admitted as disjoint ranges
    /// rather than as whole-collection writes.
    element_ranges: Vec<ProvenElementRange>,
    /// Proven affine element reads. Every ordinary read occurrence of a
    /// mapped write root must have one matching entry; this count makes a
    /// same-index read-modify-write admissible while a stencil or whole-root
    /// read still fails closed.
    element_reads: Vec<ProvenElementRead>,
    form: Option<&'static str>,
    /// Permission remains recorded, but this loop actualizer has no stackless
    /// continuation path yet and therefore must stay sequential.
    may_suspend: bool,
    exit: Option<&'static str>,
}

impl<'check> Survey<'check, '_> {
    /// Records every binding the body introduces, and every loop it opens,
    /// before anything is judged against those sets.
    fn introduce(&mut self, statements: &'check [CheckedStatement]) {
        collect_introduced(statements, &mut self.introduced);
        collect_inner_loops(statements, &mut self.inner_loops);
    }

    /// Walks one block, carrying how many value initializers the *body* opens
    /// around it.
    ///
    /// A `give` delivers to the innermost value initializer enclosing it
    /// [GIVE-1]. When that initializer is written inside B the `give` reaches
    /// a binding of this same iteration and leaves nothing, so it is an
    /// ordinary delivery; when the loop is written inside the initializer
    /// instead, the `give` leaves the loop *and* the initializer, and a
    /// combination tree over the whole range has no representation for that
    /// edge. The count tells the two apart, and it is reckoned from L's body
    /// rather than from the function, which is why judging a nested loop
    /// starts it again at zero.
    fn walk(&mut self, statements: &'check [CheckedStatement], initializers: usize) {
        for statement in statements {
            self.statement(statement, initializers);
            let inside = initializers
                + usize::from(matches!(statement, CheckedStatement::ValueMatchLet { .. }));
            for nested in nested_bodies(statement) {
                self.walk(nested, inside);
            }
        }
    }

    /// One body statement. The match is exhaustive on purpose: every form is
    /// either given a footprint here or refused here.
    fn statement(&mut self, statement: &'check CheckedStatement, initializers: usize) {
        match statement {
            CheckedStatement::Let {
                node_path, value, ..
            } => {
                // A call's argument borrows carry their loans through the
                // parameter modes of the [EFF-2] projection below. Any other
                // value may form a borrow only of iteration-own storage,
                // where no loan is needed; `admits_borrow_forms` states why.
                if !matches!(
                    value,
                    CheckedExpression::UserCall { .. } | CheckedExpression::SystemCall { .. }
                ) && !self.admits_borrow_forms(value)
                {
                    self.refuse_form("a statement that forms a borrow of storage the iteration does not introduce");
                    return;
                }
                self.moved_places(value, node_path);
                self.expression(value);
            }
            // [CALL-4] a binder or target list writes more than one place in
            // one statement. The iteration footprint below describes one
            // written target per statement, so this form is refused rather
            // than given a footprint that does not describe it.
            CheckedStatement::DestructuringLet { .. } | CheckedStatement::SetList { .. } => {
                self.refuse_form("a statement that binds an ordered result list");
            }
            CheckedStatement::Set {
                node_path,
                target,
                value,
            } => {
                let combine = match target {
                    CheckedSetTarget::Place(place) if place.fields.is_empty() => {
                        combine_of(place.binding, value)
                    }
                    _ => None,
                };
                if !self.admits_borrow_forms(value) {
                    self.refuse_form("a statement that forms a borrow of storage the iteration does not introduce");
                    return;
                }
                self.written_target(target, node_path, combine, true);
                self.moved_places(value, node_path);
                self.expression(value);
            }
            // [SET-2] reads the previous value out into the fresh binding,
            // which is a read of whatever the target carries. On enclosing
            // storage that read is the running value and no operation
            // combines it, so the target is offered no combine.
            CheckedStatement::Replace {
                node_path,
                target,
                value,
                ..
            } => {
                if !self.admits_borrow_forms(value) {
                    self.refuse_form("a statement that forms a borrow of storage the iteration does not introduce");
                    return;
                }
                self.written_target(target, node_path, None, false);
                self.moved_places(value, node_path);
                self.expression(value);
            }
            // A source proof is checked before permission and erased before
            // lowering. It has no runtime footprint or exit edge.
            CheckedStatement::Proof(_) => {}
            CheckedStatement::Return { .. } => self.leaves("a return"),
            CheckedStatement::Give {
                node_path, value, ..
            } => {
                if initializers == 0 {
                    self.leaves("a give");
                } else {
                    self.moved_places(value, node_path);
                    self.expression(value);
                }
            }
            CheckedStatement::PropagateLet { .. } => self.leaves("a propagate"),
            CheckedStatement::Break { target, .. } => {
                if !self.inner_loops.contains(&target.0) || target.0 == self.outer_loop.0 {
                    self.leaves("a break");
                }
            }
            CheckedStatement::Match { scrutinee, .. }
            | CheckedStatement::ValueMatchLet { scrutinee, .. } => self.expression(scrutinee),
            // A nested loop is judged on its own terms elsewhere; here its
            // endpoint atoms are two ordinary reads this iteration performs.
            // No rule joins two index ranges into one iteration space.
            CheckedStatement::CountedRange { lower, upper, .. } => {
                self.expression(lower);
                self.expression(upper);
            }
            CheckedStatement::Loop { .. } | CheckedStatement::Region { .. } => {}
            // An expression statement is a call [GRAM-4] whose reach no row
            // projects onto an actual, and a discarded one carries its own
            // [STOR-3] release. Admitting these needs that release classified
            // first, so today they deny, exactly as they do in a window.
            CheckedStatement::Evaluate(_) => self.refuse_form("an expression statement"),
            CheckedStatement::DropExpression { .. } => {
                self.refuse_form("a discarded expression statement");
            }
        }
    }

    /// The place one `set` or `replace` writes, against condition 2 and, when
    /// the target is a whole binding of an enclosing scope, condition 1.
    fn written_target(
        &mut self,
        target: &'check CheckedSetTarget,
        node: &NodePath,
        combine: Option<LoopCombine>,
        admits_element_map: bool,
    ) {
        // The target's place is formed exactly as the window judgment forms
        // one, so both judgments read one place relation and neither grows a
        // private copy of it.
        let mut footprint = Footprint::default();
        set_target_place(self.places, target, node, &mut footprint, false);
        let affine_map = if admits_element_map {
            self.proven_affine_map(target)
        } else {
            None
        };
        for write in &footprint.writes {
            match write {
                Access::Place { place, .. } if self.is_iteration_own(place) => {}
                Access::Place { place, .. } => {
                    if let Some(map) = affine_map {
                        if self
                            .element_ranges
                            .iter()
                            .any(|range| range.place == *place && range.map != map)
                        {
                            // Distinct affine maps can cross between iterations
                            // even when each one is injective by itself. This
                            // deliberately fixed rule admits one map per root and
                            // performs no pairwise range search.
                            self.shared.get_or_insert(node.clone());
                        }
                        self.element_ranges.push(ProvenElementRange {
                            binding: target.binding(),
                            place: place.clone(),
                            statement: node.clone(),
                            map,
                        });
                    } else {
                        self.enclosing_write(target, place, node, combine);
                    }
                }
                Access::Arena { call, .. } => {
                    self.shared.get_or_insert(call.clone());
                }
            }
        }
        // [GRAM-9] makes a subscript an atom, so the offset reads storage and
        // calls nothing; it is walked anyway, because a read of the running
        // total spelled in a subscript is a read like any other.
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => self.expression(&target.offset),
            CheckedSetTarget::BufferIndex(target) => self.expression(&target.offset),
        }
    }

    /// Reads the exact single-binder affine image ENT retained beside this
    /// subscript's successful OP-4 outcome. Permission neither evaluates the
    /// source expression nor reruns proof: absence of this checked evidence
    /// fails closed.
    fn proven_affine_map(&self, target: &CheckedSetTarget) -> Option<ProvedAffineIndexMap> {
        let (root, obligation) = match target {
            CheckedSetTarget::ArrayIndex(target) => (target.binding, &target.obligation),
            CheckedSetTarget::BufferIndex(target) => (target.root.binding, &target.obligation),
            CheckedSetTarget::Place(_) => return None,
        };
        self.proven_affine_map_at(root, obligation)
    }

    /// Reads the retained OP-4 map for one subscript occurrence. Whether the
    /// root is own storage or a live usable `&uniq` holder was already decided
    /// when semantic checking formed a writable target; permission needs only
    /// the proved range and exact affine image.
    fn proven_affine_map_at(
        &self,
        _root: BindingId,
        obligation: &NodePath,
    ) -> Option<ProvedAffineIndexMap> {
        self.obligations
            .iter()
            .find(|outcome| {
                outcome.family == ObligationFamily::Bounds
                    && outcome.node_path == *obligation
                    && outcome.discharged
            })?
            .affine_index_maps
            .iter()
            .copied()
            .find(|map| map.loop_id == self.outer_loop)
    }

    /// Retains every ordinary source read and the proved-map detail of each
    /// direct array/buffer element read. The two records are emitted by this
    /// one recursive walk so a proved read can never exist without its
    /// corresponding ordinary occurrence.
    fn record_reads(&mut self, expression: &CheckedExpression) {
        let occurrence = match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => {
                Some((*binding, rooted_place(self.places, *binding, &[])))
            }
            CheckedExpression::Project {
                binding, fields, ..
            } => Some((*binding, rooted_place(self.places, *binding, fields))),
            CheckedExpression::BorrowBuffer { root, .. }
            | CheckedExpression::BufferMeasure { root, .. } => Some((
                root.binding,
                rooted_place(self.places, root.binding, &root.fields),
            )),
            CheckedExpression::BufferIndex {
                root, obligation, ..
            } => {
                let place = rooted_place(self.places, root.binding, &root.fields);
                if let Some(map) = self.proven_affine_map_at(root.binding, obligation) {
                    self.element_reads.push(ProvenElementRead {
                        binding: root.binding,
                        place: place.clone(),
                        map,
                    });
                }
                Some((root.binding, place))
            }
            CheckedExpression::SliceMeasure { root, .. }
            | CheckedExpression::SliceIndex { root, .. } => {
                Some((root.binding, rooted_place(self.places, root.binding, &[])))
            }
            CheckedExpression::ArrayMeasure {
                root: CheckedArrayRoot::Binding { binding, fields },
                ..
            } => Some((*binding, rooted_place(self.places, *binding, fields))),
            CheckedExpression::ArrayIndex {
                root: CheckedArrayRoot::Binding { binding, fields },
                obligation,
                ..
            } => {
                let place = rooted_place(self.places, *binding, fields);
                if let Some(map) = self.proven_affine_map_at(*binding, obligation) {
                    self.element_reads.push(ProvenElementRead {
                        binding: *binding,
                        place: place.clone(),
                        map,
                    });
                }
                Some((*binding, place))
            }
            CheckedExpression::ArrayMeasure {
                root: CheckedArrayRoot::Constant(_),
                ..
            }
            | CheckedExpression::ArrayIndex {
                root: CheckedArrayRoot::Constant(_),
                ..
            } => None,
            CheckedExpression::SliceOf { source, .. } => match source {
                CheckedSliceSource::Array {
                    root: CheckedArrayRoot::Binding { binding, .. },
                    ..
                } => Some((*binding, slice_source_place(self.places, source))),
                CheckedSliceSource::Buffer(root) => {
                    Some((root.binding, slice_source_place(self.places, source)))
                }
                CheckedSliceSource::ArenaContent { binding, .. } => {
                    Some((*binding, slice_source_place(self.places, source)))
                }
                CheckedSliceSource::Array {
                    root: CheckedArrayRoot::Constant(_),
                    ..
                } => None,
            },
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::UserCall { .. }
            | CheckedExpression::SystemCall { .. }
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
            | CheckedExpression::BoxDeref { .. }
            | CheckedExpression::ArenaNew { .. }
            | CheckedExpression::ArenaDeref { .. }
            | CheckedExpression::ConstructStruct { .. }
            | CheckedExpression::ConstructEnum { .. }
            | CheckedExpression::ProjectValue { .. } => None,
        };
        if let Some((binding, place)) = occurrence {
            self.reads.push(ReadOccurrence { binding, place });
        }
        for child in expression_children(expression) {
            self.record_reads(child);
        }
    }

    /// One write into storage that outlives the iteration.
    ///
    /// An accumulator is a whole binding of an enclosing scope, named
    /// directly: a target spelled through a borrow holder resolves to storage
    /// whose reads name the holder instead, and this judgment counts reads by
    /// binding, so such a target is refused rather than accumulated. That
    /// refusal is deliberate and not an artifact of the combine test.
    fn enclosing_write(
        &mut self,
        target: &CheckedSetTarget,
        place: &ResolvedPlace,
        node: &NodePath,
        combine: Option<LoopCombine>,
    ) {
        let named = match target {
            CheckedSetTarget::Place(target) if target.fields.is_empty() => Some(target.binding),
            // Every element form not consumed as an affine-map range above,
            // and every field of enclosing storage, remains one unresolved
            // shared place and fails closed.
            _ => None,
        };
        let Some(binding) = named.filter(|binding| {
            !self.places.is_holder(*binding) && *place == ResolvedPlace::binding(*binding)
        }) else {
            self.shared.get_or_insert(node.clone());
            return;
        };
        match combine {
            Some(combine) => self.accumulates.push(Accumulate {
                binding,
                combine,
                statement: node.clone(),
            }),
            None => {
                self.carried.get_or_insert(node.clone());
            }
        }
    }

    /// Whether a resolved place is storage this iteration introduced. Storage
    /// rooted in a binding the body opens is created and released inside the
    /// iteration, so no two iterations reach one of them.
    /// Whether every borrow formed inside this expression is a borrow of
    /// iteration-own storage, by the shared walk below.
    fn admits_borrow_forms(&self, expression: &CheckedExpression) -> bool {
        borrows_only_iteration_own(self.places, expression, &|place| {
            self.is_iteration_own(place)
        })
    }

    fn is_iteration_own(&self, place: &ResolvedPlace) -> bool {
        match place.root {
            PlaceRoot::Binding(binding) => self.introduced.contains(&binding),
            // A named const [CONST-2] is enclosing storage. Nothing writes
            // one, so this arm exists to keep the classification total.
            PlaceRoot::Constant(_) => false,
        }
    }

    /// The caller places one expression transfers away by consuming an `own`
    /// value. A move out of enclosing storage is a write of it.
    fn moved_places(&mut self, value: &CheckedExpression, node: &NodePath) {
        let mut footprint = Footprint::default();
        collect_consumed_places(self.places, value, node, &mut footprint);
        self.record_writes(&footprint);
    }

    /// One expression tree: every read it performs, and every call it makes.
    ///
    /// The two walks are separate because [`visit_read_bindings`] descends the
    /// whole tree itself. Calling it at every level as well would count one
    /// read of an accumulator as several, and condition 1 reads that count.
    fn expression(&mut self, expression: &CheckedExpression) {
        self.record_reads(expression);
        self.calls(expression);
    }

    /// Every call one expression tree makes, with its [EFF-2] projection.
    ///
    /// The walk reaches a call wherever it is written rather than only as the
    /// whole right-hand side of a `let`, so no call's row escapes the
    /// footprint by the shape of the statement that holds it.
    fn calls(&mut self, expression: &CheckedExpression) {
        match expression {
            CheckedExpression::UserCall { function, .. } => {
                if let Some(projection) = call_projection(expression) {
                    let footprint = self.program.footprint(self.places, &projection);
                    self.record_writes(&footprint);
                }
                self.may_suspend |= self.program.target_action(*function).may_suspend();
            }
            CheckedExpression::SystemCall { target_action, .. } => {
                if let Some(projection) = call_projection(expression) {
                    let footprint = self.program.footprint(self.places, &projection);
                    self.record_writes(&footprint);
                }
                self.may_suspend |= target_action.may_suspend();
            }
            _ => {}
        }
        for child in expression_children(expression) {
            self.calls(child);
        }
    }

    /// Every write of one projected footprint, against condition 2.
    ///
    /// The written half is judged directly. A call read that reaches a mapped
    /// collection necessarily travels through a borrow actual and therefore
    /// carries a loan checked below; an `own` consume is already a write. For
    /// the only other enclosing write, the accumulator, condition 1 counts
    /// source read occurrences by binding.
    fn record_writes(&mut self, footprint: &Footprint) {
        if let Some(argument) = &footprint.unresolved {
            self.unresolved.get_or_insert(argument.clone());
        }
        // The loans half of condition 2 [OWN-5, OWN-12]. Two overlapped
        // iterations both hold every loan their body forms, so an exclusive
        // loan on storage outliving the iteration always denies. Shared loans
        // remain admissible for read-only enclosing storage, but every loan is
        // retained so `denial` can reject either strength against a mapped
        // write root.
        for loan in &footprint.loans {
            self.call_loans.push(loan.clone());
            if loan.strength == LoanStrength::Exclusive && !self.is_iteration_own(&loan.place) {
                self.loan.get_or_insert(loan.argument.clone());
            }
        }
        for write in &footprint.writes {
            match write {
                Access::Place { place, .. } if self.is_iteration_own(place) => {}
                Access::Place { argument, .. } => {
                    self.shared.get_or_insert(argument.clone());
                }
                // Two iterations allocating into one enclosing region would
                // both append to its allocation list, which is one place with
                // no actual to project onto. A region the body opens is
                // iteration-own and reaches the arm above.
                Access::Arena { call, .. } => {
                    self.shared.get_or_insert(call.clone());
                }
            }
        }
    }

    fn leaves(&mut self, edge: &'static str) {
        self.exit.get_or_insert(edge);
    }

    fn refuse_form(&mut self, form: &'static str) {
        self.form.get_or_insert(form);
    }

    /// The four conditions in their numbered order, then eligibility.
    ///
    /// A form refusal is reported ahead of all four: a statement whose
    /// footprint this judgment does not compute has no condition-1 or
    /// condition-2 answer to give, so a loop with several defects reports the
    /// unclassified form first, which is the honest report.
    fn finish(self, statement: NodePath) -> LoopPermission {
        let mut carried = Vec::new();
        for accumulate in &self.accumulates {
            if !carried.contains(&accumulate.combine) {
                carried.push(accumulate.combine);
            }
        }
        let combines = carried.iter().map(|combine| combine.spelling()).collect();
        let denial = self.denial();
        // The advice outlives exactly one refusal: a loop this version
        // declines only because it carries several accumulators is still one a
        // hand-written recursion returning an aggregate can split. Every other
        // refusal is a reason the split would be refused too, or unsound.
        let advises_split =
            matches!(denial, Some(LoopDenial::ManyAccumulators { .. })) && !carried.is_empty();
        // A stateless loop is not a map merely because it is permitted. The
        // exact range is the positive witness which selects IndependentMap;
        // an accumulator selects Reduction, including a reduction whose body
        // also contains independently proved element maps.
        let actualization = if denial.is_some() || self.may_suspend {
            None
        } else if let Some(accumulate) = self.accumulates.first() {
            Some(LoopActualization::Reduction {
                accumulator: accumulate.binding,
                combine: accumulate.combine,
            })
        } else if self.element_ranges.is_empty() {
            None
        } else {
            Some(LoopActualization::IndependentMap)
        };
        let verdict = match denial {
            Some(denial) => LoopVerdict::Denied(denial),
            None => LoopVerdict::PermittedEligible,
        };
        LoopPermission {
            statement,
            verdict,
            combines,
            advises_split,
            actualization,
        }
    }

    fn denial(&self) -> Option<LoopDenial> {
        if let Some(form) = self.form {
            return Some(LoopDenial::BodyForm { form });
        }
        if let Some(denial) = self.carried_state() {
            return Some(denial);
        }
        if let Some(argument) = &self.loan {
            return Some(LoopDenial::Loan {
                argument: argument.clone(),
            });
        }
        if let Some(loan) = self.call_loans.iter().find(|loan| {
            self.element_ranges
                .iter()
                .any(|range| loan.place.overlaps(&range.place))
        }) {
            // A mapped write is disjoint from another iteration's mapped
            // write, not from a whole-root loan held by that iteration. Both
            // shared and exclusive loans therefore deny the map explicitly.
            return Some(LoopDenial::Loan {
                argument: loan.argument.clone(),
            });
        }
        if let Some(argument) = &self.shared {
            return Some(LoopDenial::SharedWrite {
                argument: argument.clone(),
            });
        }
        if let Some(range) = self.element_ranges.iter().find(|range| {
            let reads = self
                .reads
                .iter()
                .filter(|read| read.place.overlaps(&range.place))
                .count();
            let matching = self
                .element_reads
                .iter()
                .filter(|read| {
                    read.binding == range.binding
                        && read.place == range.place
                        && read.map == range.map
                })
                .count();
            reads != matching
        }) {
            // A matching read and write affine image selects the same element
            // in each iteration. Any whole-root read, different image, or
            // unproved subscript leaves the counts unequal and fails closed.
            return Some(LoopDenial::SharedWrite {
                argument: range.statement.clone(),
            });
        }
        if let Some(argument) = &self.unresolved {
            return Some(LoopDenial::UnresolvedWrite {
                argument: argument.clone(),
            });
        }
        self.exit.map(|edge| LoopDenial::Exit { edge })
    }

    /// Condition 1: the body carries at most one value across iterations, and
    /// that value is a reduction.
    ///
    /// The read count is per body rather than per accumulate, so a loop that
    /// combines one accumulator under two branches is refused although its
    /// combines are admitted. That is a conservatism, not an unsoundness —
    /// and it is what makes a single accumulate statement, and therefore a
    /// single fixed combine per accumulator, an invariant here. Widening the
    /// count would require testing that every accumulate of one binding
    /// carries the same operation, which today's shape makes vacuous.
    fn carried_state(&self) -> Option<LoopDenial> {
        if let Some(statement) = &self.carried {
            return Some(LoopDenial::NotAReduction {
                statement: statement.clone(),
            });
        }
        let mut bindings: Vec<BindingId> = Vec::new();
        for accumulate in &self.accumulates {
            if !bindings.contains(&accumulate.binding) {
                bindings.push(accumulate.binding);
            }
        }
        let [accumulator] = bindings.as_slice() else {
            return (bindings.len() > 1).then_some(LoopDenial::ManyAccumulators {
                accumulators: bindings.len(),
            });
        };
        let first = self
            .accumulates
            .iter()
            .find(|accumulate| accumulate.binding == *accumulator)
            .expect("the accumulator came from one of these");
        let reads = self
            .reads
            .iter()
            .filter(|read| read.binding == *accumulator)
            .count();
        (reads != 1).then(|| LoopDenial::AccumulatorRead {
            statement: first.statement.clone(),
            reads,
        })
    }
}

/// The combine of `set acc = <op>(acc, rest)`, when `op` is exactly
/// associative and `rest` does not read `acc`.
fn combine_of(accumulator: BindingId, value: &CheckedExpression) -> Option<LoopCombine> {
    let (combine, arguments) = match value {
        CheckedExpression::IntegerOperation {
            operation,
            arguments,
            ..
        } => (integer_combine(*operation)?, arguments),
        CheckedExpression::BooleanOperation {
            operation,
            arguments,
            ..
        } => (boolean_combine(*operation)?, arguments),
        _ => return None,
    };
    let [left, right] = arguments.as_slice() else {
        return None;
    };
    // Exactly one operand is the accumulator read, and the other reaches it
    // nowhere: `acc = acc + f(acc)` is not a reduction. Both operand
    // positions are accepted, which is sound only because every admitted
    // operation is commutative as well as associative; admitting a
    // non-commutative associative operation would silently turn a right fold
    // into a left fold and must fix the position instead.
    let carried = [(left, right), (right, left)]
        .into_iter()
        .find(|(operand, _)| reads_only(operand, accumulator))?;
    let mut mentioned = false;
    visit_read_bindings(carried.1, &mut |binding| {
        mentioned |= binding == accumulator;
    });
    if mentioned {
        return None;
    }
    Some(combine)
}

/// Whether one operand is exactly a read of this binding.
fn reads_only(operand: &CheckedExpression, binding: BindingId) -> bool {
    matches!(operand, CheckedExpression::Binding { binding: read, .. } if *read == binding)
}

/// The exactly-associative integer operations.
///
/// The list is closed and every entry is here for the same stated reason:
/// regrouping its applications produces the same bits. `+exact`, `+defined`,
/// and `+checked` are associative in Z and are still absent, because each
/// application carries its own obligation and regrouping moves which
/// intermediate has to satisfy it. `+sat` fails associativity outright.
const fn integer_combine(operation: CheckedIntegerOperation) -> Option<LoopCombine> {
    Some(match operation {
        CheckedIntegerOperation::AddWrap => LoopCombine::AddWrap,
        CheckedIntegerOperation::MultiplyWrap => LoopCombine::MultiplyWrap,
        CheckedIntegerOperation::BitAnd => LoopCombine::BitAnd,
        CheckedIntegerOperation::BitOr => LoopCombine::BitOr,
        CheckedIntegerOperation::BitXor => LoopCombine::BitXor,
        CheckedIntegerOperation::Minimum => LoopCombine::Minimum,
        CheckedIntegerOperation::Maximum => LoopCombine::Maximum,
        _ => return None,
    })
}

/// The exactly-associative boolean operations. `not` is unary and no combine.
const fn boolean_combine(operation: CheckedBooleanOperation) -> Option<LoopCombine> {
    Some(match operation {
        CheckedBooleanOperation::And => LoopCombine::And,
        CheckedBooleanOperation::Or => LoopCombine::Or,
        CheckedBooleanOperation::ExclusiveOr => LoopCombine::ExclusiveOr,
        CheckedBooleanOperation::Not => return None,
    })
}

/// Every binding one block and its nested blocks introduce, appended in source
/// order.
///
/// Storage rooted in one of these is created fresh by every iteration of the
/// loop that owns the block and dies with it; everything else outlives the
/// iteration. The staged judgment next door asks the same question of the same
/// body, so both read this one walk rather than growing two drifting copies.
pub(super) fn collect_introduced(statements: &[CheckedStatement], out: &mut Vec<BindingId>) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { binding, .. }
            | CheckedStatement::PropagateLet { binding, .. }
            | CheckedStatement::Replace { binding, .. }
            | CheckedStatement::ValueMatchLet { binding, .. } => out.push(*binding),
            CheckedStatement::CountedRange { binder, .. } => out.push(*binder),
            _ => {}
        }
        if let CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } =
            statement
        {
            for arm in arms {
                for arm_binder in &arm.binders {
                    out.push(arm_binder.binding);
                }
            }
        }
        for nested in nested_bodies(statement) {
            collect_introduced(nested, out);
        }
    }
}

/// Every loop one block opens inside itself, by loop identity.
fn collect_inner_loops(statements: &[CheckedStatement], out: &mut Vec<u32>) {
    for statement in statements {
        if let CheckedStatement::CountedRange { id, .. } | CheckedStatement::Loop { id, .. } =
            statement
        {
            out.push(id.0);
        }
        for nested in nested_bodies(statement) {
            collect_inner_loops(nested, out);
        }
    }
}

/// Whether every borrow formed inside one expression is a borrow of storage the
/// caller's predicate accepts as iteration-own.
///
/// A written borrow's shared-or-uniq mode is erased from the checked tree, so
/// the [OWN-5] loan such a borrow would hold cannot be stated. A borrow of
/// iteration-own storage needs none: each iteration borrows its own instance,
/// and nothing that instance reaches outlives the iteration. Every other
/// borrow — outer storage, or a place this walk cannot resolve — is
/// inadmissible, and the caller refuses the body, which is the fail-closed
/// direction. Both loop judgments ask exactly this, so both read this one
/// implementation.
pub(super) fn borrows_only_iteration_own(
    places: &PlaceMap,
    expression: &CheckedExpression,
    is_iteration_own: &impl Fn(&ResolvedPlace) -> bool,
) -> bool {
    let is_borrow_form = matches!(
        expression,
        CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
    );
    if is_borrow_form {
        match places.argument_referent(expression) {
            Some((place, _, _)) if is_iteration_own(&place) => {}
            _ => return false,
        }
    }
    expression_children(expression)
        .into_iter()
        .all(|child| borrows_only_iteration_own(places, child, is_iteration_own))
}

/// Every block a statement owns.
pub(super) fn nested_bodies(statement: &CheckedStatement) -> Vec<&[CheckedStatement]> {
    match statement {
        CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
            arms.iter().map(|arm| arm.body.as_slice()).collect()
        }
        CheckedStatement::Loop { body, .. }
        | CheckedStatement::Region { body, .. }
        | CheckedStatement::CountedRange { body, .. } => vec![body.as_slice()],
        _ => Vec::new(),
    }
}
