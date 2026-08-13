//! [ENT-4] fact states and the least-fixed-point difference-bound closure.
//!
//! A state carries the *flow* of [ENT-3]: the live established facts (plus
//! facts a join admitted, since [ENT-5] closes each joined state first). The
//! closed state at a point is computed on demand as the least set containing
//! the live and implicit facts, closed under transitivity, disequality
//! strengthening, and subsumption; every derivability answer equals that
//! least-closure answer.

use std::collections::{HashMap, HashSet};

use super::super::goal::{GoalExpression, GoalProjection};
use super::super::model::{BindingId, IntegerType};
use super::term::{LengthBound, TermId, TermKind, TermTable, ZERO, type_range};

/// One normalized source relation over interned terms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Relation {
    /// `left - right <= bound`.
    Bound {
        left: TermId,
        right: TermId,
        bound: i128,
    },
    /// `left = right`, the bound pair in both directions.
    Equal { left: TermId, right: TermId },
    /// `left != right`, one disequality.
    Distinct { left: TermId, right: TermId },
}

/// Dense identity of one finite typed expression in a concrete function's
/// [ENT-2] goal universe. Only Bool-typed members may carry signed facts;
/// non-Bool members are retained solely as ordinary-let origin expansions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GoalId(pub(crate) u32);

/// The two exact opaque facts [ENT-2] admits for one complete goal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalSign {
    Positive,
    Negative,
}

/// One place read by a complete goal. `length` records ENT-5's fixed-length
/// boundary: an element write does not invalidate a `len(P)` observation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct GoalSupport {
    pub(crate) root: BindingId,
    pub(crate) projections: Vec<GoalProjection>,
    pub(crate) length: bool,
}

/// Derived data attached to one exact typed expression.
#[derive(Clone, Debug)]
struct GoalRecord {
    expression: GoalExpression,
    projection: Option<Relation>,
    support: Vec<GoalSupport>,
}

/// Per-function interning table for [ENT-2]'s finite goal universe.
#[derive(Default)]
pub(crate) struct GoalTable {
    ids: HashMap<GoalExpression, GoalId>,
    records: Vec<GoalRecord>,
}

impl GoalTable {
    pub(crate) fn intern(
        &mut self,
        expression: GoalExpression,
        projection: Option<Relation>,
        support: Vec<GoalSupport>,
    ) -> GoalId {
        if let Some(id) = self.ids.get(&expression).copied() {
            let record = &mut self.records[id.0 as usize];
            debug_assert_eq!(record.support, support);
            if record.projection.is_none() {
                record.projection = projection;
            } else {
                debug_assert_eq!(record.projection, projection);
            }
            return id;
        }
        let id = GoalId(self.records.len() as u32);
        self.ids.insert(expression.clone(), id);
        self.records.push(GoalRecord {
            expression,
            projection,
            support,
        });
        id
    }

    pub(crate) fn expression(&self, id: GoalId) -> &GoalExpression {
        &self.records[id.0 as usize].expression
    }

    pub(crate) fn projection(&self, id: GoalId) -> Option<&Relation> {
        self.records[id.0 as usize].projection.as_ref()
    }

    pub(crate) fn support(&self, id: GoalId) -> &[GoalSupport] {
        &self.records[id.0 as usize].support
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = GoalId> + '_ {
        (0..self.records.len()).map(|index| GoalId(index as u32))
    }
}

impl Relation {
    /// [ENT-3] S1 exact negation over mathematical integers.
    pub(crate) fn negated(&self) -> Self {
        match self {
            Self::Bound { left, right, bound } => Self::Bound {
                left: *right,
                right: *left,
                bound: -bound - 1,
            },
            Self::Equal { left, right } => Self::Distinct {
                left: *left,
                right: *right,
            },
            Self::Distinct { left, right } => Self::Equal {
                left: *left,
                right: *right,
            },
        }
    }

    /// Every term occurring in the relation, for kill support tests.
    pub(crate) fn terms(&self) -> [TermId; 2] {
        match self {
            Self::Bound { left, right, .. }
            | Self::Equal { left, right }
            | Self::Distinct { left, right } => [*left, *right],
        }
    }
}

/// What one match arm's value binder gains when the scrutinee is an
/// outcome-carrying call [ENT-3] S7, S10.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutcomeRelation {
    /// S7 checked arithmetic: the binder equals the base term shifted by this
    /// constant.
    Shifted(i128),
    /// S10 boundary count: the binder is at most the base term.
    AtMost,
}

/// One pending arm fact: the relation the observing arm's value binder gains,
/// against a base term whose support must survive the path to the match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OutcomeFact {
    /// The variant whose arm observes it — `Ok` for checked arithmetic and
    /// for the `Result`-shaped boundary calls, `ReadBytes` for `read_once`.
    /// Every other arm establishes nothing [ENT-3].
    pub(crate) variant: &'static str,
    /// The term the binder is related to: p for S7, k for S10.
    pub(crate) base: TermId,
    pub(crate) relation: OutcomeRelation,
}

/// One live fact state on the structural flow [ENT-3].
#[derive(Clone, Debug, Default)]
pub(crate) struct FactState {
    /// The loop rule's empty join: the contradictory all-derivable state, in
    /// which every relation is derivable and every fact is present. Z has
    /// empty support, so `Z - Z <= -1` never dies and the flag is absorbing
    /// under kills [ENT-4, ENT-5].
    pub(crate) all_derivable: bool,
    /// Live difference bounds `left - right <= bound`, smallest bound kept.
    pub(crate) bounds: HashMap<(TermId, TermId), i128>,
    /// Live disequalities, stored with ordered term pair.
    pub(crate) distinct: HashSet<(TermId, TermId)>,
    /// [ENT-3] comparison origins (b): `own Bool` bindings whose initializer
    /// comparison is still valid on every path from initializer to here.
    pub(crate) origins: HashMap<BindingId, Relation>,
    /// [ENT-3] S7/S10 outcome origins: bindings holding the outcome of a
    /// checked-arithmetic or bounded boundary call, under the same no-kill,
    /// no-`set` path discipline the comparison origins carry.
    pub(crate) outcomes: HashMap<BindingId, OutcomeFact>,
    /// Live exact signed whole-goal facts [ENT-2..ENT-4].
    pub(crate) opaque: HashSet<(GoalId, GoalSign)>,
    /// Complete still-valid pure/total origin expansion of an ordinary let.
    /// The binding's own direct value goal is intentionally separate.
    pub(crate) goal_origins: HashMap<BindingId, GoalId>,
}

impl FactState {
    pub(crate) fn establish(&mut self, relation: &Relation) {
        if self.all_derivable {
            return;
        }
        match relation {
            Relation::Bound { left, right, bound } => self.add_bound(*left, *right, *bound),
            Relation::Equal { left, right } => {
                self.add_bound(*left, *right, 0);
                self.add_bound(*right, *left, 0);
            }
            Relation::Distinct { left, right } => {
                self.distinct.insert(ordered(*left, *right));
            }
        }
    }

    pub(crate) fn establish_goal(&mut self, goal: GoalId, sign: GoalSign) {
        if !self.all_derivable {
            self.opaque.insert((goal, sign));
        }
    }

    fn add_bound(&mut self, left: TermId, right: TermId, bound: i128) {
        let entry = self.bounds.entry((left, right)).or_insert(bound);
        if bound < *entry {
            *entry = bound;
        }
    }

    /// Removes every live fact and origin with a support member the kill
    /// predicate reaches. Closure never resurrects a killed fact: derived
    /// facts normally live in [`ClosedState`] views or join results, while
    /// S11 deliberately materializes its post-capture closure before kills.
    pub(crate) fn kill(&mut self, mut killed: impl FnMut(TermId) -> bool) {
        if self.all_derivable {
            return;
        }
        let dead: Vec<(TermId, TermId)> = self
            .bounds
            .keys()
            .filter(|(left, right)| killed(*left) || killed(*right))
            .copied()
            .collect();
        for pair in dead {
            self.bounds.remove(&pair);
        }
        self.distinct
            .retain(|(left, right)| !killed(*left) && !killed(*right));
        self.origins.retain(|_, relation| {
            let [left, right] = relation.terms();
            !killed(left) && !killed(right)
        });
        self.outcomes.retain(|_, outcome| !killed(outcome.base));
    }

    /// Removes signed facts and ordinary-let origin expansions whose exact
    /// goal support is invalidated by one ENT-5 event.
    pub(crate) fn kill_goals(&mut self, mut killed: impl FnMut(GoalId) -> bool) {
        if self.all_derivable {
            return;
        }
        self.opaque.retain(|(goal, _)| !killed(*goal));
        self.goal_origins.retain(|_, goal| !killed(*goal));
    }
}

fn ordered(left: TermId, right: TermId) -> (TermId, TermId) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

/// The closed fact state at one point: the [ENT-4] least fixed point over the
/// live facts and the implicit facts of every registered term.
pub(crate) struct ClosedState {
    all_derivable: bool,
    bounds: HashMap<(TermId, TermId), i128>,
    distinct: HashSet<(TermId, TermId)>,
    opaque: HashSet<(GoalId, GoalSign)>,
}

impl ClosedState {
    /// `left - right <= bound` is derivable.
    pub(crate) fn derives_bound(&self, left: TermId, right: TermId, bound: i128) -> bool {
        if self.all_derivable {
            return true;
        }
        self.bounds
            .get(&(left, right))
            .is_some_and(|held| *held <= bound)
    }

    /// A state is contradictory when `t - t <= -1` is derivable for any term;
    /// there every relation is derivable and every obligation is discharged.
    pub(crate) const fn contradictory(&self) -> bool {
        self.all_derivable
    }

    /// [ENT-4] exact derivability of one normalized relation: a bound by the
    /// held smaller-or-equal constant, an equality by both zero bounds, a
    /// disequality by presence or by either strict bound.
    pub(crate) fn derives(&self, relation: &Relation) -> bool {
        if self.all_derivable {
            return true;
        }
        match relation {
            Relation::Bound { left, right, bound } => self.derives_bound(*left, *right, *bound),
            Relation::Equal { left, right } => {
                self.derives_bound(*left, *right, 0) && self.derives_bound(*right, *left, 0)
            }
            Relation::Distinct { left, right } => {
                self.distinct.contains(&ordered(*left, *right))
                    || self.derives_bound(*left, *right, -1)
                    || self.derives_bound(*right, *left, -1)
            }
        }
    }

    /// Exact signed-goal derivability: a retained opaque sign or its one
    /// comparison-root projection, with no Boolean decomposition.
    pub(crate) fn derives_goal(&self, goal: GoalId, sign: GoalSign, goals: &GoalTable) -> bool {
        if self.all_derivable || self.opaque.contains(&(goal, sign)) {
            return true;
        }
        let Some(relation) = goals.projection(goal) else {
            return false;
        };
        match sign {
            GoalSign::Positive => self.derives(relation),
            GoalSign::Negative => self.derives(&relation.negated()),
        }
    }

    pub(crate) fn holds_opaque(&self, goal: GoalId, sign: GoalSign) -> bool {
        !self.all_derivable && self.opaque.contains(&(goal, sign))
    }
}

/// Computes the [ENT-4] closure of `state` over the registered terms.
pub(crate) fn close(state: &FactState, terms: &TermTable, goals: &GoalTable) -> ClosedState {
    if state.all_derivable {
        return ClosedState {
            all_derivable: true,
            bounds: HashMap::new(),
            distinct: HashSet::new(),
            opaque: HashSet::new(),
        };
    }
    let mut bounds = state.bounds.clone();
    let mut distinct = state.distinct.clone();
    let add = |map: &mut HashMap<(TermId, TermId), i128>, left, right, bound: i128| {
        let entry = map.entry((left, right)).or_insert(bound);
        if bound < *entry {
            *entry = bound;
        }
    };
    // Implicit facts [ENT-2]: reflexive bounds, fragment-type ranges, the
    // constant fold through Z, and array length equalities registered on the
    // length term itself by the flow.
    for id in terms.ids() {
        add(&mut bounds, id, id, 0);
        match terms.kind(id) {
            TermKind::Zero | TermKind::ConstParameter(_) => {}
            TermKind::Constant(value) => {
                add(&mut bounds, id, ZERO, *value);
                add(&mut bounds, ZERO, id, -value);
            }
            TermKind::Place(_, ty) | TermKind::ProjectedPlace(_, ty) => {
                let (minimum, maximum) = type_range(*ty);
                add(&mut bounds, id, ZERO, maximum);
                add(&mut bounds, ZERO, id, -minimum);
            }
            TermKind::Length(_) | TermKind::ProjectedLength(_) => {
                let (minimum, maximum) = type_range(IntegerType::U64);
                add(&mut bounds, id, ZERO, maximum);
                add(&mut bounds, ZERO, id, -minimum);
                match terms.length_bound(id) {
                    Some(LengthBound::Constant(length)) => {
                        add(&mut bounds, id, ZERO, length);
                        add(&mut bounds, ZERO, id, -length);
                    }
                    Some(LengthBound::Equal(parameter)) => {
                        add(&mut bounds, id, parameter, 0);
                        add(&mut bounds, parameter, id, 0);
                    }
                    None => {}
                }
            }
            TermKind::CountedCapture { .. } => {
                let (minimum, maximum) = type_range(IntegerType::U64);
                add(&mut bounds, id, ZERO, maximum);
                add(&mut bounds, ZERO, id, -minimum);
            }
        }
    }
    // Least fixed point of transitivity (1), disequality strengthening (2),
    // and subsumption (3). The rules are monotone over finitely many ordered
    // pairs, so iteration terminates; strengthening only lowers a bound to
    // -1, so the outer loop runs at most a handful of rounds.
    let ids: Vec<TermId> = terms.ids().collect();
    loop {
        let mut changed = false;
        for middle in &ids {
            for left in &ids {
                let Some(first) = bounds.get(&(*left, *middle)).copied() else {
                    continue;
                };
                for right in &ids {
                    let Some(second) = bounds.get(&(*middle, *right)).copied() else {
                        continue;
                    };
                    let via = first.saturating_add(second);
                    match bounds.entry((*left, *right)) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            if via < *entry.get() {
                                entry.insert(via);
                                changed = true;
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(via);
                            changed = true;
                        }
                    }
                }
            }
        }
        // ENT-4 makes every strict bound a disequality in either
        // orientation. Retain that derived fact in this same fixed point so
        // it can strengthen an available weak bound and so ENT-5 joins can
        // intersect the complete closed disequality set.
        for ((left, right), bound) in &bounds {
            if left != right && *bound <= -1 && distinct.insert(ordered(*left, *right)) {
                changed = true;
            }
        }
        for (left, right) in &distinct {
            for (from, to) in [(*left, *right), (*right, *left)] {
                if bounds.get(&(from, to)).is_some_and(|bound| *bound == 0) {
                    bounds.insert((from, to), -1);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let l0_contradictory = ids
        .iter()
        .any(|id| bounds.get(&(*id, *id)).is_some_and(|bound| *bound < 0));
    let mut closed = ClosedState {
        all_derivable: l0_contradictory,
        bounds,
        distinct,
        opaque: state.opaque.clone(),
    };
    let goal_contradictory = !closed.all_derivable
        && goals.ids().any(|goal| {
            closed.derives_goal(goal, GoalSign::Positive, goals)
                && closed.derives_goal(goal, GoalSign::Negative, goals)
        });
    if goal_contradictory {
        closed.all_derivable = true;
    }
    closed
}

/// Materializes the [ENT-4] least closure as a live flow state.
///
/// Ordinary queries can keep closure as an ephemeral view. S11 instead fixes
/// the complete post-capture closure *before* the counted loop's continuing
/// kill subtraction, so consequences whose support no longer includes a
/// mutable endpoint source must become independently live facts first.
pub(crate) fn materialize_closure(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
) -> FactState {
    let closed = close(state, terms, goals);
    if closed.all_derivable {
        return FactState {
            all_derivable: true,
            ..FactState::default()
        };
    }
    FactState {
        all_derivable: false,
        bounds: closed.bounds,
        distinct: closed.distinct,
        origins: state.origins.clone(),
        outcomes: state.outcomes.clone(),
        opaque: closed.opaque,
        goal_origins: state.goal_origins.clone(),
    }
}

/// [ENT-5] join of arm-exit states, each already taken after its scope-exit
/// kills. Each input is closed first; the join keeps, per ordered term pair,
/// the weakest bound held by all, and each disequality held by all. The empty
/// join is the contradictory all-derivable state.
pub(crate) fn join(states: &[FactState], terms: &TermTable, goals: &GoalTable) -> FactState {
    // Close before filtering: a contradiction established immediately before
    // an edge is already the absorbing all-derivable state even when no kill
    // had occasion to materialize its flag.
    let closed: Vec<ClosedState> = states
        .iter()
        .map(|state| close(state, terms, goals))
        .filter(|state| !state.contradictory())
        .collect();
    let Some((first, rest)) = closed.split_first() else {
        return FactState {
            all_derivable: true,
            ..FactState::default()
        };
    };
    let mut bounds = HashMap::new();
    for (pair, bound) in &first.bounds {
        let mut weakest = *bound;
        let held = rest.iter().all(|state| {
            state.bounds.get(pair).is_some_and(|other| {
                if *other > weakest {
                    weakest = *other;
                }
                true
            })
        });
        if held {
            bounds.insert(*pair, weakest);
        }
    }
    let mut distinct = first.distinct.clone();
    for state in rest {
        distinct.retain(|pair| state.distinct.contains(pair));
    }
    // Comparison and outcome origins are path conditions, not facts; one
    // survives a join only when every contributing path carries the same one.
    let mut opaque = first.opaque.clone();
    for state in rest {
        opaque.retain(|fact| state.opaque.contains(fact));
    }
    let contributing: Vec<&FactState> = states
        .iter()
        .filter(|state| !close(state, terms, goals).contradictory())
        .collect();
    let mut origins = contributing[0].origins.clone();
    let mut outcomes = contributing[0].outcomes.clone();
    let mut goal_origins = contributing[0].goal_origins.clone();
    for state in contributing.iter().skip(1) {
        origins.retain(|binding, relation| {
            state
                .origins
                .get(binding)
                .is_some_and(|other| other == relation)
        });
        outcomes.retain(|binding, outcome| {
            state
                .outcomes
                .get(binding)
                .is_some_and(|other| other == outcome)
        });
        goal_origins.retain(|binding, goal| {
            state
                .goal_origins
                .get(binding)
                .is_some_and(|other| other == goal)
        });
    }
    FactState {
        all_derivable: false,
        bounds,
        distinct,
        origins,
        outcomes,
        opaque,
        goal_origins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_join_remains_the_contradictory_all_derivable_state() {
        let joined = join(&[], &TermTable::new(), &GoalTable::default());
        assert!(joined.all_derivable);
    }
}
