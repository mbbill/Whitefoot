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
//! 2. **No shared writable footprint.** Every place B writes is iteration-own
//!    or that accumulator. An element write, a field of enclosing storage, a
//!    callee row projecting onto enclosing storage, an arena append, and every
//!    write this judgment cannot resolve all deny.
//! 3. **Row gate.** No call in B carries `external` or `blocks`, and no
//!    [SYS-2] operation is written in B. Rows gate; places prove.
//! 4. **No exit edge.** No `return`, `give`, `propagate` `Err` edge, or
//!    `break` naming L or an enclosing loop leaves the loop, so every
//!    iteration of the whole range runs.
//!
//! There is no fifth condition. An earlier version of this judgment also asked
//! that B and the transitive call closure of its calls reach zero `claim`
//! sites, so that no schedule could choose which iteration traps first. That
//! condition is gone, on the ground the window judgment's module doc derives:
//! a false executed claim is a contract violation [SCOPE-4], so an execution
//! reaching one is erroneous, the observable-identity guarantee is conditional
//! on contract compliance, and a correct loop — one no iteration of which
//! trips a claim — keeps that guarantee whole under every schedule. An
//! erroneous one traps with exactly one well-formed [DIAG-3] record, of a
//! claim the schedule may choose among those that evaluated false, because
//! `wf_trap` latches. A `claim` in B is therefore an ordinary statement here:
//! its predicate is read like any other expression and it constrains nothing.
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
//! edges — and never the entailment fact state. The quantification over
//! iterations is *structural*: it comes from [FN-1]'s unit-increment
//! recurrence over a compiler-owned binder, never from a derived fact about
//! two indices. Facts-on and facts-off compilation therefore produce the same
//! loop permission table by construction.
//!
//! # Why counting an accumulator's reads by binding is complete
//!
//! Condition 1 counts the read occurrences of the accumulator *binding*, and a
//! read spelled `deref(h)` names the holder rather than the storage. Two facts
//! close that gap without a second relation. A holder bound inside the body
//! names the accumulator where it is formed, and forming a borrow is itself a
//! counted read occurrence, so such a loop is refused by the count. A holder
//! bound outside the body leaves only two spellings: writing the accumulator
//! by name while that borrow is live is an [OWN-5] borrow conflict and the
//! program is rejected, and writing it *through* the holder makes the target
//! root a holder, which condition 2 refuses because an accumulator must be a
//! whole binding named directly. Asking instead whether any holder in the
//! function reaches the accumulator would be flow-insensitive and would refuse
//! a sound reduction whose result is borrowed after the loop.
//!
//! # The one-sided reading
//!
//! Every form this judgment has not classified refuses, and the statement
//! match is exhaustive for that reason: a missed statement would contribute an
//! empty footprint and *widen* permission, which is the one direction it must
//! never fail in. A form refusal is reported ahead of the numbered conditions,
//! because a statement whose footprint is unknown has no condition-1 or
//! condition-2 answer to give.

use super::model::{
    BindingId, CheckedBooleanOperation, CheckedExpression, CheckedFunction,
    CheckedIntegerOperation, CheckedLoopId, CheckedSetTarget, CheckedStatement,
    expression_children,
};
use super::permission::{
    Access, Footprint, Program, call_projection, collect_consumed_places, set_target_place,
    visit_read_bindings,
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
    /// What actualizing this permission needs from the judgment, present
    /// exactly for a permitted, eligible loop carrying one accumulator.
    ///
    /// The judgment does not decide that anything is emitted: lowering reads
    /// this, applies its own emission conditions, and may still decline. The
    /// verdict above is the same either way.
    pub(crate) actualization: Option<LoopActualization>,
}

/// The identities a synthesized range split needs: which binding carries the
/// fold, and the operation the combination tree applies.
///
/// Everything else the split needs — which values the body reads from the
/// enclosing scope, what the frame costs, what the body weighs — is a property
/// of the emitted shape rather than of the judgment, and lowering computes it
/// from the IR it built.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoopActualization {
    pub(crate) accumulator: BindingId,
    pub(crate) combine: LoopCombine,
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
    /// accumulator recombined. A `claim` in the body or in a callee is not a
    /// reason to refuse; the module doc carries why.
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
    /// Condition 2, fail closed: a written place this judgment cannot
    /// resolve. An unresolved element overlaps every place, so it denies.
    UnresolvedWrite { argument: NodePath },
    /// Condition 2, fail closed: a body statement form whose footprint this
    /// judgment does not compute.
    BodyForm { form: &'static str },
    /// Condition 3: a call in the body carries `external` or `blocks`.
    Row {
        function: String,
        external: bool,
        blocks: bool,
    },
    /// Condition 3: a [SYS-2] operation is the external world itself.
    SystemCall { call: NodePath },
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
            Self::SharedWrite { .. } | Self::UnresolvedWrite { .. } | Self::BodyForm { .. } => 2,
            Self::Row { .. } | Self::SystemCall { .. } => 3,
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
    collect(program, places, &function.body, &mut judged);
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
                node_path.clone(),
                *id,
                *binder,
                body,
            ));
        }
        for nested in nested_bodies(statement) {
            collect(program, places, nested, judged);
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
    statement: NodePath,
    id: CheckedLoopId,
    binder: BindingId,
    body: &'check [CheckedStatement],
) -> LoopPermission {
    let mut survey = Survey {
        program,
        places,
        outer_loop: id,
        introduced: vec![binder],
        inner_loops: Vec::new(),
        reads: Vec::new(),
        accumulates: Vec::new(),
        carried: None,
        shared: None,
        unresolved: None,
        form: None,
        row: None,
        system_call: None,
        exit: None,
    };
    survey.introduce(body);
    survey.walk(body, 0);
    survey.finish(statement)
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
    /// The counted loop under judgment, so a `break` that closes it is told
    /// apart from one that closes a loop opened inside it.
    outer_loop: CheckedLoopId,
    /// Bindings introduced anywhere inside the body, including the loop's own
    /// binder. Storage rooted in one of these is created fresh by every
    /// iteration and dies with it; everything else outlives the iteration.
    introduced: Vec<BindingId>,
    inner_loops: Vec<u32>,
    /// Every read occurrence, with multiplicity.
    reads: Vec<BindingId>,
    accumulates: Vec<Accumulate>,
    carried: Option<NodePath>,
    shared: Option<NodePath>,
    unresolved: Option<NodePath>,
    form: Option<&'static str>,
    row: Option<(String, bool, bool)>,
    system_call: Option<NodePath>,
    exit: Option<&'static str>,
}

impl<'check> Survey<'check, '_> {
    /// Records every binding the body introduces, before anything is judged
    /// against that set.
    fn introduce(&mut self, statements: &'check [CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, .. }
                | CheckedStatement::PropagateLet { binding, .. }
                | CheckedStatement::Replace { binding, .. }
                | CheckedStatement::ValueMatchLet { binding, .. } => {
                    self.introduced.push(*binding);
                }
                CheckedStatement::CountedRange { id, binder, .. } => {
                    self.introduced.push(*binder);
                    self.inner_loops.push(id.0);
                }
                CheckedStatement::Loop { id, .. } => self.inner_loops.push(id.0),
                _ => {}
            }
            if let CheckedStatement::Match { arms, .. }
            | CheckedStatement::ValueMatchLet { arms, .. } = statement
            {
                for arm in arms {
                    for arm_binder in &arm.binders {
                        self.introduced.push(arm_binder.binding);
                    }
                }
            }
            for nested in nested_bodies(statement) {
                self.introduce(nested);
            }
        }
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
                self.moved_places(value, node_path);
                self.expression(value);
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
                self.written_target(target, node_path, combine);
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
                self.written_target(target, node_path, None);
                self.moved_places(value, node_path);
                self.expression(value);
            }
            // A claim writes nothing, and it leaves the loop only when it is
            // false — an erroneous execution, which the module doc accounts
            // for rather than refuses. Its predicate is an ordinary read of the
            // body, counted here, and that is the whole of its contribution.
            CheckedStatement::Claim { condition, .. } => self.expression(condition),
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
    ) {
        // The target's place is formed exactly as the window judgment forms
        // one, so both judgments read one place relation and neither grows a
        // private copy of it.
        let mut footprint = Footprint::default();
        set_target_place(self.places, target, node, &mut footprint, false);
        for write in &footprint.writes {
            match write {
                Access::Place { place, .. } if self.is_iteration_own(place) => {}
                Access::Place { place, .. } => self.enclosing_write(target, place, node, combine),
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
            // An element or field of enclosing storage: two iterations write
            // one place, because a resolved place carries no index segment
            // [ENT-2]. That is the fail-closed direction, and the parallel map
            // it refuses is deferred rather than granted.
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
        visit_read_bindings(expression, &mut |binding| self.reads.push(binding));
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
                match self.program.signature(*function) {
                    Some(signature) if signature.external || signature.blocks => {
                        self.row.get_or_insert((
                            self.program.function_name(*function),
                            signature.external,
                            signature.blocks,
                        ));
                    }
                    // A callee this analysis cannot place has no readable row,
                    // so it gates rather than being assumed clean.
                    None => {
                        self.row
                            .get_or_insert((self.program.function_name(*function), true, true));
                    }
                    Some(_) => {}
                }
            }
            // A [SYS-2] operation is the external world: it is the very thing
            // condition 3's row gate keeps out of an overlapped execution.
            CheckedExpression::SystemCall { call, .. } => {
                self.system_call.get_or_insert(call.clone());
            }
            _ => {}
        }
        for child in expression_children(expression) {
            self.calls(child);
        }
    }

    /// Every write of one projected footprint, against condition 2.
    ///
    /// Only the written half is judged. The read half is unconstrained
    /// because condition 2 leaves exactly one place of enclosing storage
    /// written — the accumulator — and condition 1 counts every read
    /// occurrence of it by binding, which no unresolved read can hide: a read
    /// through a borrow needs a holder, and an accumulator any holder reaches
    /// is refused outright.
    fn record_writes(&mut self, footprint: &Footprint) {
        if let Some(argument) = &footprint.unresolved {
            self.unresolved.get_or_insert(argument.clone());
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
        // The one accumulator this version's split carries. Condition 1 has
        // already refused every loop with more than one, so the payload exists
        // exactly where the verdict below is `PermittedEligible` and the body
        // combines something: a permitted loop that carries nothing has no fold
        // to distribute.
        let actualization = match (&denial, self.accumulates.first()) {
            (None, Some(accumulate)) => Some(LoopActualization {
                accumulator: accumulate.binding,
                combine: accumulate.combine,
            }),
            _ => None,
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
        if let Some(argument) = &self.shared {
            return Some(LoopDenial::SharedWrite {
                argument: argument.clone(),
            });
        }
        if let Some(argument) = &self.unresolved {
            return Some(LoopDenial::UnresolvedWrite {
                argument: argument.clone(),
            });
        }
        if let Some((function, external, blocks)) = &self.row {
            return Some(LoopDenial::Row {
                function: function.clone(),
                external: *external,
                blocks: *blocks,
            });
        }
        if let Some(call) = &self.system_call {
            return Some(LoopDenial::SystemCall { call: call.clone() });
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
            .filter(|read| *read == accumulator)
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

/// Every block a statement owns.
fn nested_bodies(statement: &CheckedStatement) -> Vec<&[CheckedStatement]> {
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
