//! [ENT-4] fact states and the least-fixed-point difference-bound closure.
//!
//! A state carries the *flow* of [ENT-3]: the live established facts (plus
//! facts a join admitted, since [ENT-5] closes each joined state first). The
//! closed state at a point is computed on demand as the least set containing
//! the live and implicit facts, closed under transitivity, disequality
//! strengthening, and subsumption; every derivability answer equals that
//! least-closure answer.

use std::collections::{HashMap, HashSet};

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

    fn add_bound(&mut self, left: TermId, right: TermId, bound: i128) {
        let entry = self.bounds.entry((left, right)).or_insert(bound);
        if bound < *entry {
            *entry = bound;
        }
    }

    /// Removes every live fact and origin with a support member the kill
    /// predicate reaches. Closure never resurrects a killed fact: derived
    /// facts live only inside [`ClosedState`] views and join results.
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
}

/// Computes the [ENT-4] closure of `state` over the registered terms.
pub(crate) fn close(state: &FactState, terms: &TermTable) -> ClosedState {
    if state.all_derivable {
        return ClosedState {
            all_derivable: true,
            bounds: HashMap::new(),
            distinct: HashSet::new(),
        };
    }
    let mut bounds = state.bounds.clone();
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
            TermKind::Place(_, ty) => {
                let (minimum, maximum) = type_range(*ty);
                add(&mut bounds, id, ZERO, maximum);
                add(&mut bounds, ZERO, id, -minimum);
            }
            TermKind::Length(_) => {
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
        for (left, right) in &state.distinct {
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
    let contradictory = ids
        .iter()
        .any(|id| bounds.get(&(*id, *id)).is_some_and(|bound| *bound < 0));
    ClosedState {
        all_derivable: contradictory,
        bounds,
        distinct: state.distinct.clone(),
    }
}

/// [ENT-5] join of arm-exit states, each already taken after its scope-exit
/// kills. Each input is closed first; the join keeps, per ordered term pair,
/// the weakest bound held by all, and each disequality held by all. The empty
/// join is the contradictory all-derivable state.
pub(crate) fn join(states: &[FactState], terms: &TermTable) -> FactState {
    // An all-derivable input contains every fact, so it never narrows the
    // join; the join is over the remaining states, and with none left it is
    // the empty join: the contradictory all-derivable state itself.
    let contributing: Vec<&FactState> =
        states.iter().filter(|state| !state.all_derivable).collect();
    let closed: Vec<ClosedState> = contributing
        .iter()
        .map(|state| close(state, terms))
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
    let mut origins = contributing
        .first()
        .map(|state| state.origins.clone())
        .unwrap_or_default();
    let mut outcomes = contributing
        .first()
        .map(|state| state.outcomes.clone())
        .unwrap_or_default();
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
    }
    FactState {
        all_derivable: false,
        bounds,
        distinct,
        origins,
        outcomes,
    }
}
