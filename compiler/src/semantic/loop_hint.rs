//! The counted-loop split hint: a developer-channel line that names the
//! parallelism a loop cannot have and a recursion can.
//!
//! The permission judgment reads a pair of adjacent *statements*, so two
//! executions of one statement are never a pair and a counted loop gets
//! nothing, however independent its iterations are. The same computation
//! written as a recursive halving of its index range is two adjacent
//! `let`-bound calls, and is eligible with no change to the judgment. That is
//! the whole content of this hint: not a transform, not a verdict, and not
//! anything an accepted program can notice. It reports one line per counted
//! loop that a split would make eligible, and the writer decides.
//!
//! ## Why the advice needs a proof and not a guess
//!
//! A split runs the two halves of the range in either order, or at the same
//! time, and combines their results. A writer who follows the advice on a loop
//! whose combine is not *exactly* associative gets a program that publishes
//! different bytes at a different worker count — the one failure this whole
//! path exists to make impossible. So the hint is emitted only when the loop's
//! carried state is a single accumulator combined under an operation whose
//! regrouping is bit-for-bit identity:
//!
//! - `+wrap` and `*wrap`, which are the ring operations of Z/2^n;
//! - `iand`, `ior`, `ixor`, which are the lattice and group operations of the
//!   bit vector;
//! - `imin` and `imax`, which are the lattice operations of the total order;
//! - boolean `and`, `or`, `xor`.
//!
//! **No float operation is ever admitted.** `fadd.strict` is the pointed
//! example: floating-point addition is not associative, so `(a + b) + c` and
//! `a + (b + c)` are different numbers, and a schedule that regrouped them
//! would move a published byte. The rule is not "floats are risky" but "the
//! admitted set is enumerated and contains no float", which is why a float
//! accumulator produces no line at all rather than a hedged one.
//!
//! The same obligation reaches through a call. A callee that writes caller
//! storage carries state across iterations exactly as a `set` in the body
//! does, and the combine is whatever its own body performs — which can be a
//! float fold one frame away, invisible to a survey that reads statements. So
//! a call whose row writes anything at all refuses the line. The enumerated
//! set has to govern all of the loop's carried state, not only the part
//! written where the survey can see it.
//!
//! There is a pleasing agreement here that is worth naming: the integer
//! operations that carry no proof obligation — the `wrap` family and the
//! bitwise and ordering operations — are exactly the ones that are exactly
//! associative. `+defined` and `+exact` are associative in Z but carry a
//! per-application obligation that regrouping moves, so they are not admitted
//! either.
//!
//! ## What the hint does not cover
//!
//! Only [`CheckedStatement::CountedRange`] — the `for` form — is considered. A
//! bare `loop` has no index range, so "split its index range" is not advice
//! that means anything about it; the compiler would be inventing a range from
//! the shape of a hand-written counter, which is exactly the kind of
//! source-shape reading this project does not do. A writer who wants the
//! advice writes the counted form, which is the form that carries the fact.

use super::model::{
    BindingId, CheckedBooleanOperation, CheckedExpression, CheckedFunction,
    CheckedIntegerOperation, CheckedLoopId, CheckedSetTarget, CheckedStatement, FunctionId,
    expression_children,
};
use super::permission::{PermissionSignature, visit_read_bindings};
use crate::NodePath;

/// One counted loop a recursive index split would make eligible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoopSplitHint {
    /// The counted-loop statement.
    pub(crate) statement: NodePath,
    /// The operations the split would regroup, in source order and without
    /// repeats. They are on the line because they are the reason the advice is
    /// safe to take: these and no others survive being reassociated.
    pub(crate) combines: Vec<&'static str>,
}

/// What the hint asks about a callee, supplied by the permission analysis
/// because it is the pass that already holds the call graph.
#[derive(Clone, Copy)]
pub(crate) struct CalleeFacts<'check> {
    /// Whether a `claim` is reachable from each function, dense by
    /// [`FunctionId`].
    pub(crate) reaches_claim: &'check [bool],
    /// The declared effect row of each function, dense by [`FunctionId`].
    pub(crate) signatures: &'check [PermissionSignature],
}

impl CalleeFacts<'_> {
    /// Whether a call to this function keeps the loop splittable: no reachable
    /// claim, an empty `writes` row, and a row carrying neither `external` nor
    /// `blocks`.
    ///
    /// The `writes` row is what makes carried state reachable through a call.
    /// [`Survey`] classifies only the `set` statements written in the loop
    /// body, so storage a callee writes through a `&uniq` parameter is state
    /// carried across iterations that no survey sees — and the line would name
    /// the combine of some *other*, admitted accumulator as though it were the
    /// loop's only one. A row that writes anything therefore refuses, rather
    /// than this analysis guessing which of the caller's places the row
    /// projects onto. The storage may well be the iteration's own, but
    /// deciding that needs the resolved places the judgment holds and a
    /// diagnostic has no business rebuilding.
    ///
    /// An unknown callee answers `false`, so a call this analysis cannot place
    /// suppresses the hint rather than being advised around.
    fn admits(&self, callee: FunctionId) -> bool {
        let index = callee.0 as usize;
        let Some(signature) = self.signatures.get(index) else {
            return false;
        };
        !self.reaches_claim.get(index).copied().unwrap_or(true)
            && signature.writes.is_empty()
            && !signature.external
            && !signature.blocks
    }
}

/// Every counted loop of one function a recursive index split would make
/// eligible, in source order.
///
/// `eligible_pairs` are the statement paths of the pairs the judgment already
/// found eligible in this function. A loop containing one of them already has
/// its parallelism and gets no line, so the hint never contradicts the ledger
/// two lines above it.
pub(crate) fn split_hints(
    function: &CheckedFunction,
    facts: CalleeFacts<'_>,
    eligible_pairs: &[NodePath],
) -> Vec<LoopSplitHint> {
    let mut hints = Vec::new();
    collect(&function.body, facts, eligible_pairs, &mut hints);
    hints
}

fn collect(
    statements: &[CheckedStatement],
    facts: CalleeFacts<'_>,
    eligible_pairs: &[NodePath],
    hints: &mut Vec<LoopSplitHint>,
) {
    for statement in statements {
        if let CheckedStatement::CountedRange {
            id,
            node_path,
            binder,
            body,
            ..
        } = statement
            && !eligible_pairs.iter().any(|pair| encloses(node_path, pair))
            && let Some(combines) = splittable(*id, *binder, body, facts)
        {
            hints.push(LoopSplitHint {
                statement: node_path.clone(),
                combines,
            });
        }
        for nested in nested_bodies(statement) {
            collect(nested, facts, eligible_pairs, hints);
        }
    }
}

/// Whether one statement lies inside another, by node path prefix.
fn encloses(outer: &NodePath, inner: &NodePath) -> bool {
    let outer = outer.components();
    let inner = inner.components();
    inner.len() > outer.len() && inner.starts_with(outer)
}

/// The combines of a counted loop a split would make eligible, or `None`.
///
/// The survey is deliberately one-sided: every form this analysis has not
/// classified refuses. A missed statement would let the hint advise a split
/// that changes what a program publishes, which is the only way this
/// diagnostic can do harm.
fn splittable(
    id: CheckedLoopId,
    binder: BindingId,
    body: &[CheckedStatement],
    facts: CalleeFacts<'_>,
) -> Option<Vec<&'static str>> {
    let mut survey = Survey {
        facts,
        outer_loop: id,
        introduced: vec![binder],
        inner_loops: Vec::new(),
        accumulators: Vec::new(),
        reads: Vec::new(),
        refused: false,
    };
    survey.introduce(body);
    survey.walk(body);
    if survey.refused || survey.accumulators.is_empty() {
        return None;
    }
    // An accumulator the loop reads anywhere but inside its own combine makes
    // the iterations order-dependent — the value a later iteration sees is the
    // running total, which a split does not produce — so the loop is not a
    // reduction however associative its combine is.
    for accumulator in &survey.accumulators {
        if survey
            .reads
            .iter()
            .filter(|read| **read == accumulator.binding)
            .count()
            != 1
        {
            return None;
        }
    }
    let mut combines = Vec::new();
    for accumulator in &survey.accumulators {
        if !combines.contains(&accumulator.combine) {
            combines.push(accumulator.combine);
        }
    }
    Some(combines)
}

/// One accepted accumulate statement.
struct Accumulator {
    binding: BindingId,
    combine: &'static str,
}

struct Survey<'check> {
    facts: CalleeFacts<'check>,
    /// The counted loop under survey, so a `break` that closes it is told
    /// apart from one that closes a loop opened inside it.
    outer_loop: CheckedLoopId,
    /// Bindings introduced anywhere inside the body, including the loop's own
    /// binder. Everything else a statement names belongs to an enclosing
    /// scope and is therefore carried across iterations if it is written.
    introduced: Vec<BindingId>,
    inner_loops: Vec<u32>,
    accumulators: Vec<Accumulator>,
    /// Every read occurrence, with multiplicity.
    reads: Vec<BindingId>,
    refused: bool,
}

impl Survey<'_> {
    /// Records every binding the body introduces, before anything is judged
    /// against that set.
    fn introduce(&mut self, statements: &[CheckedStatement]) {
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
                CheckedStatement::Loop { id, .. } => {
                    self.inner_loops.push(id.0);
                }
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

    fn walk(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            self.statement(statement);
            for nested in nested_bodies(statement) {
                self.walk(nested);
            }
        }
    }

    fn statement(&mut self, statement: &CheckedStatement) {
        match statement {
            // A claim is a trap edge out of the loop, and a split would have
            // to decide which half traps first.
            CheckedStatement::Claim { .. } => self.refused = true,
            // All three leave the loop, so the iterations are not independent.
            // `give` leaves the enclosing value initializer as well, and a
            // split has no representation for that edge at all: it would sum
            // the whole range where the loop stopped early.
            CheckedStatement::Return { .. }
            | CheckedStatement::PropagateLet { .. }
            | CheckedStatement::Give { .. } => {
                self.refused = true;
            }
            CheckedStatement::Break { target, .. } => {
                if !self.inner_loops.contains(&target.0) || target.0 == self.outer_loop.0 {
                    self.refused = true;
                }
            }
            CheckedStatement::Set { target, value, .. } => {
                self.assignment(target, value);
                self.expression(value);
            }
            // A [SET-2] replacement reads the previous value out into a fresh
            // binding, which is a read of the carried state that a split does
            // not reproduce.
            CheckedStatement::Replace { target, value, .. } => {
                if !self.introduced.contains(&target.binding()) {
                    self.refused = true;
                }
                self.expression(value);
            }
            CheckedStatement::Let { value, .. } | CheckedStatement::Evaluate(value) => {
                self.expression(value);
            }
            CheckedStatement::DropExpression { value, .. } => self.expression(value),
            CheckedStatement::Match { scrutinee, .. }
            | CheckedStatement::ValueMatchLet { scrutinee, .. } => self.expression(scrutinee),
            CheckedStatement::CountedRange { lower, upper, .. } => {
                self.expression(lower);
                self.expression(upper);
            }
            CheckedStatement::Loop { .. } | CheckedStatement::Region { .. } => {}
        }
    }

    /// Classifies one `set` against the carried-state rule.
    fn assignment(&mut self, target: &CheckedSetTarget, value: &CheckedExpression) {
        let root = target.binding();
        if self.introduced.contains(&root) {
            // Storage the body itself introduced dies with the iteration.
            return;
        }
        // The whole binding and nothing else: a write into a field, an array
        // element, or a buffer slot of enclosing storage is a parallel *map*,
        // which condition 2 denies today because two index ranges of one
        // buffer are one place. Advising a split there would advise a program
        // the judgment then refuses.
        let CheckedSetTarget::Place(place) = target else {
            self.refused = true;
            return;
        };
        if !place.fields.is_empty() {
            self.refused = true;
            return;
        }
        let Some(combine) = combine_of(root, value) else {
            self.refused = true;
            return;
        };
        self.accumulators.push(Accumulator {
            binding: root,
            combine,
        });
    }

    /// One expression tree: every read it performs, and every call it makes.
    ///
    /// The two walks are separate because [`visit_read_bindings`] descends the
    /// whole tree itself. Calling it at every level as well would count one
    /// read of an accumulator as several, and the reduction test below reads
    /// that count.
    fn expression(&mut self, expression: &CheckedExpression) {
        visit_read_bindings(expression, &mut |binding| self.reads.push(binding));
        self.calls(expression);
    }

    fn calls(&mut self, expression: &CheckedExpression) {
        match expression {
            CheckedExpression::UserCall { function, .. } => {
                if !self.facts.admits(*function) {
                    self.refused = true;
                }
            }
            // A [SYS-2] operation is the external world: it is the very thing
            // condition 3's row gate keeps out of an overlapped pair.
            CheckedExpression::SystemCall { .. } => self.refused = true,
            _ => {}
        }
        for child in expression_children(expression) {
            self.calls(child);
        }
    }
}

/// The combine of `set acc = <op>(acc, rest)`, when `op` is exactly
/// associative and `rest` does not read `acc`.
fn combine_of(accumulator: BindingId, value: &CheckedExpression) -> Option<&'static str> {
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
    // nowhere: `acc = acc + f(acc)` is not a reduction.
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

/// The exactly-associative integer operations, and their spellings.
///
/// The list is closed and every entry is here for the same stated reason:
/// regrouping its applications produces the same bits. `+exact`, `+defined`,
/// and `+checked` are associative in Z and are still absent, because each
/// application carries its own obligation and regrouping moves which
/// intermediate has to satisfy it.
fn integer_combine(operation: CheckedIntegerOperation) -> Option<&'static str> {
    Some(match operation {
        CheckedIntegerOperation::AddWrap => "+wrap",
        CheckedIntegerOperation::MultiplyWrap => "*wrap",
        CheckedIntegerOperation::BitAnd => "iand",
        CheckedIntegerOperation::BitOr => "ior",
        CheckedIntegerOperation::BitXor => "ixor",
        CheckedIntegerOperation::Minimum => "imin",
        CheckedIntegerOperation::Maximum => "imax",
        _ => return None,
    })
}

/// The exactly-associative boolean operations. `not` is unary and no combine.
fn boolean_combine(operation: CheckedBooleanOperation) -> Option<&'static str> {
    Some(match operation {
        CheckedBooleanOperation::And => "band",
        CheckedBooleanOperation::Or => "bor",
        CheckedBooleanOperation::ExclusiveOr => "bxor",
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
