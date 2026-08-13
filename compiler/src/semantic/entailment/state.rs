//! [ENT-4] fact states and the least-fixed-point difference-bound closure.
//!
//! A state carries the *flow* of [ENT-3]: the live established facts (plus
//! facts a join admitted, since [ENT-5] closes each joined state first). The
//! closed state at a point is computed on demand as the least set containing
//! the live and implicit facts, closed under transitivity, disequality
//! strengthening, and subsumption; every derivability answer equals that
//! least-closure answer.

use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use super::super::goal::{GoalExpression, GoalProjection};
use super::super::model::{BindingId, IntegerType};
use super::term::{LengthBound, TermId, TermKind, TermTable, ZERO, type_range};
use crate::NodePath;

/// One normalized source relation over interned terms.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum GoalSign {
    Positive,
    Negative,
}

/// Dense function-local identity of one retained ENT-4 derivation node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct DerivationId(pub(crate) u32);

/// Dense function-local identity of one proof-producing flow event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FlowEventId(pub(crate) u32);

/// Proof-producing phase of one event in the existing ENT flow.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FlowEventKind {
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S9,
    S10,
    S11,
    Join,
    Snapshot,
}

/// One deterministic flow event. A source path is retained exactly when the
/// checked tree already carries one; synthetic joins and loop edges need only
/// their dense event identity and predecessor ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlowEvent {
    pub(crate) kind: FlowEventKind,
    pub(crate) node_path: Option<NodePath>,
}

/// Canonical retained identity for one dense goal ID after analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedGoal {
    pub(crate) expression: GoalExpression,
    pub(crate) projection: Option<Relation>,
}

/// The exact canonical identities referenced by retained dense term and goal
/// IDs. The analyzer moves these inventories into the checked function; it
/// does not duplicate them or invent a portable encoding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationInventory {
    pub(crate) terms: Vec<TermKind>,
    pub(crate) length_bounds: Vec<Option<LengthBound>>,
    pub(crate) goals: Vec<RetainedGoal>,
}

/// Why an implicit bound exists independently of writer facts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ImplicitBoundKind {
    Reflexive,
    Constant,
    TypeMinimum,
    TypeMaximum,
    ArrayLength,
}

/// One reaching predecessor named by a join derivation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct JoinParent {
    pub(crate) ordinal: u32,
    pub(crate) parent: DerivationId,
}

/// The closed set of proof steps emitted by the existing entailment flow.
/// Parent IDs always precede their child in the arena.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DerivationNode {
    SourceBound {
        relation: Relation,
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
    },
    SourceDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
    },
    SourceGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
    },
    ImplicitBound {
        left: TermId,
        right: TermId,
        bound: i128,
        kind: ImplicitBoundKind,
    },
    TransitiveBound {
        left: TermId,
        middle: TermId,
        right: TermId,
        bound: i128,
        first: DerivationId,
        second: DerivationId,
    },
    StrengthenedBound {
        left: TermId,
        right: TermId,
        bound: i128,
        weak: DerivationId,
        distinct: DerivationId,
    },
    SubsumedBound {
        left: TermId,
        right: TermId,
        held: i128,
        requested: i128,
        parent: DerivationId,
    },
    Equality {
        left: TermId,
        right: TermId,
        forward: DerivationId,
        reverse: DerivationId,
    },
    DisequalityFromStrictBound {
        left: TermId,
        right: TermId,
        parent: DerivationId,
    },
    GoalProjection {
        goal: GoalId,
        sign: GoalSign,
        relation: Relation,
        parent: DerivationId,
    },
    L0Contradiction {
        term: TermId,
        parent: DerivationId,
    },
    GoalContradiction {
        goal: GoalId,
        positive: DerivationId,
        negative: DerivationId,
    },
    JoinBound {
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    JoinContradiction {
        event: FlowEventId,
        parents: Vec<JoinParent>,
    },
    MaterializedBound {
        left: TermId,
        right: TermId,
        bound: i128,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedDistinct {
        left: TermId,
        right: TermId,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedGoal {
        goal: GoalId,
        sign: GoalSign,
        event: FlowEventId,
        parent: DerivationId,
    },
    MaterializedContradiction {
        event: FlowEventId,
        parent: DerivationId,
    },
}

impl DerivationNode {
    fn for_each_parent(&self, mut visit: impl FnMut(DerivationId)) {
        match self {
            Self::TransitiveBound { first, second, .. } => {
                visit(*first);
                visit(*second);
            }
            Self::StrengthenedBound { weak, distinct, .. } => {
                visit(*weak);
                visit(*distinct);
            }
            Self::SubsumedBound { parent, .. }
            | Self::DisequalityFromStrictBound { parent, .. }
            | Self::GoalProjection { parent, .. }
            | Self::L0Contradiction { parent, .. }
            | Self::MaterializedBound { parent, .. }
            | Self::MaterializedDistinct { parent, .. }
            | Self::MaterializedGoal { parent, .. }
            | Self::MaterializedContradiction { parent, .. } => visit(*parent),
            Self::Equality {
                forward, reverse, ..
            } => {
                visit(*forward);
                visit(*reverse);
            }
            Self::GoalContradiction {
                positive, negative, ..
            } => {
                visit(*positive);
                visit(*negative);
            }
            Self::JoinBound { parents, .. }
            | Self::JoinDistinct { parents, .. }
            | Self::JoinGoal { parents, .. }
            | Self::JoinContradiction { parents, .. } => {
                for parent in parents {
                    visit(parent.parent);
                }
            }
            Self::SourceBound { .. }
            | Self::SourceDistinct { .. }
            | Self::SourceGoal { .. }
            | Self::ImplicitBound { .. } => {}
        }
    }

    #[cfg(test)]
    pub(crate) fn parent_ids(&self) -> Vec<DerivationId> {
        let mut parents = Vec::with_capacity(self.parent_count());
        self.for_each_parent(|parent| parents.push(parent));
        parents
    }

    fn parent_count(&self) -> usize {
        match self {
            Self::TransitiveBound { .. }
            | Self::StrengthenedBound { .. }
            | Self::Equality { .. }
            | Self::GoalContradiction { .. } => 2,
            Self::SubsumedBound { .. }
            | Self::DisequalityFromStrictBound { .. }
            | Self::GoalProjection { .. }
            | Self::L0Contradiction { .. }
            | Self::MaterializedBound { .. }
            | Self::MaterializedDistinct { .. }
            | Self::MaterializedGoal { .. }
            | Self::MaterializedContradiction { .. } => 1,
            Self::JoinBound { parents, .. }
            | Self::JoinDistinct { parents, .. }
            | Self::JoinGoal { parents, .. }
            | Self::JoinContradiction { parents, .. } => parents.len(),
            Self::SourceBound { .. }
            | Self::SourceDistinct { .. }
            | Self::SourceGoal { .. }
            | Self::ImplicitBound { .. } => 0,
        }
    }

    fn maximum_parent_depth(&self, depths: &[u32]) -> Option<u32> {
        let mut maximum = None;
        self.for_each_parent(|parent| {
            let depth = depths[parent.0 as usize];
            maximum = Some(maximum.map_or(depth, |current: u32| current.max(depth)));
        });
        maximum
    }

    fn rank(&self) -> u8 {
        match self {
            Self::SourceBound { .. } => 0,
            Self::SourceDistinct { .. } => 1,
            Self::SourceGoal { .. } => 2,
            Self::ImplicitBound { .. } => 3,
            Self::TransitiveBound { .. } => 4,
            Self::StrengthenedBound { .. } => 5,
            Self::SubsumedBound { .. } => 6,
            Self::Equality { .. } => 7,
            Self::DisequalityFromStrictBound { .. } => 8,
            Self::GoalProjection { .. } => 9,
            Self::L0Contradiction { .. } => 10,
            Self::GoalContradiction { .. } => 11,
            Self::JoinBound { .. } => 12,
            Self::JoinDistinct { .. } => 13,
            Self::JoinGoal { .. } => 14,
            Self::JoinContradiction { .. } => 15,
            Self::MaterializedBound { .. } => 16,
            Self::MaterializedDistinct { .. } => 17,
            Self::MaterializedGoal { .. } => 18,
            Self::MaterializedContradiction { .. } => 19,
        }
    }
}

/// Root class counts are kept explicit so task 0056 can measure the frozen
/// corpus without parsing debug output or inventing a persistent format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationMetrics {
    pub(crate) bounds_roots: u32,
    pub(crate) opaque_goal_roots: u32,
    pub(crate) projected_goal_roots: u32,
    pub(crate) contradiction_roots: u32,
    pub(crate) unique_nodes: u32,
    pub(crate) parent_edges: u32,
    pub(crate) maximum_depth: u32,
    pub(crate) retained_bytes: usize,
}

/// Which mandatory checked-program query owns a retained root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DerivationRootKind {
    BoundsObligation(u32),
    CallGoal(u32),
    CountedS11 {
        occurrence: u32,
        atom: CountedRootAtom,
    },
}

/// The fixed eight directed atomic bounds in one normative S11 group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountedRootAtom {
    LowerCaptureToEndpoint,
    LowerEndpointToCapture,
    UpperCaptureToEndpoint,
    UpperEndpointToCapture,
    BinderToLowerCapture,
    LowerCaptureToBinder,
    LowerCaptureLeBinder,
    BinderLtUpperCapture,
}

/// One mandatory query root into the function-local arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DerivationRoot {
    pub(crate) kind: DerivationRootKind,
    pub(crate) node: DerivationId,
}

/// Private, lifetime-bound derivation storage for one concrete checked
/// function. It is intentionally neither serializable nor independently
/// verifiable; the checked program remains the only authority.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DerivationLedger {
    pub(crate) events: Vec<FlowEvent>,
    pub(crate) nodes: Vec<DerivationNode>,
    pub(crate) roots: Vec<DerivationRoot>,
    depths: Vec<u32>,
    interned: HashMap<DerivationNode, DerivationId>,
    pub(crate) metrics: DerivationMetrics,
}

impl DerivationLedger {
    pub(crate) fn event(
        &mut self,
        kind: FlowEventKind,
        node_path: Option<NodePath>,
    ) -> FlowEventId {
        let id = FlowEventId(
            u32::try_from(self.events.len())
                .expect("ENT flow event inventory exceeds the u32 identity space"),
        );
        self.events.push(FlowEvent { kind, node_path });
        id
    }

    pub(crate) fn intern(&mut self, node: DerivationNode) -> DerivationId {
        if let Some(id) = self.interned.get(&node).copied() {
            return id;
        }
        let id = DerivationId(
            u32::try_from(self.nodes.len())
                .expect("ENT derivation inventory exceeds the u32 identity space"),
        );
        let mut parents_precede = true;
        node.for_each_parent(|parent| parents_precede &= parent.0 < id.0);
        debug_assert!(parents_precede);
        let depth = node
            .maximum_parent_depth(&self.depths)
            .map_or(0, |maximum| maximum.saturating_add(1));
        self.nodes.push(node.clone());
        self.depths.push(depth);
        self.interned.insert(node, id);
        id
    }

    pub(crate) fn depth(&self, id: DerivationId) -> u32 {
        self.depths[id.0 as usize]
    }

    fn better(&self, candidate: DerivationId, current: DerivationId) -> bool {
        let candidate_depth = self.depth(candidate);
        let current_depth = self.depth(current);
        candidate_depth < current_depth
            || (candidate_depth == current_depth
                && compare_node_ties(
                    &self.nodes[candidate.0 as usize],
                    &self.nodes[current.0 as usize],
                )
                .is_lt())
    }

    fn candidate_better(&self, candidate: &DerivationNode, current: DerivationId) -> bool {
        let candidate_depth = candidate
            .maximum_parent_depth(&self.depths)
            .map_or(0, |maximum| maximum.saturating_add(1));
        let current_depth = self.depth(current);
        candidate_depth < current_depth
            || (candidate_depth == current_depth
                && compare_node_ties(candidate, &self.nodes[current.0 as usize]).is_lt())
    }

    pub(crate) fn add_root(&mut self, kind: DerivationRootKind, node: DerivationId) {
        self.roots.push(DerivationRoot { kind, node });
    }

    pub(crate) fn finish(&mut self) -> Vec<Option<DerivationId>> {
        let old_len = self.nodes.len();
        let mut keep = vec![false; old_len];
        let mut stack: Vec<DerivationId> = self.roots.iter().map(|root| root.node).collect();
        while let Some(id) = stack.pop() {
            let index = id.0 as usize;
            if keep[index] {
                continue;
            }
            keep[index] = true;
            self.nodes[index].for_each_parent(|parent| stack.push(parent));
        }
        let mut remap = vec![None; old_len];
        let mut nodes = Vec::with_capacity(keep.iter().filter(|kept| **kept).count());
        for (index, node) in self.nodes.iter().enumerate() {
            if keep[index] {
                let id = DerivationId(
                    u32::try_from(nodes.len())
                        .expect("retained ENT derivations exceed the u32 identity space"),
                );
                remap[index] = Some(id);
                nodes.push(node.clone());
            }
        }
        for node in &mut nodes {
            remap_node(node, &remap);
        }
        for root in &mut self.roots {
            root.node = remap[root.node.0 as usize].expect("root retained");
        }
        self.nodes = nodes;
        let mut depths = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let depth = node
                .maximum_parent_depth(&depths)
                .map_or(0, |maximum| maximum.saturating_add(1));
            depths.push(depth);
        }
        self.depths = depths;
        self.interned.clear();
        self.interned.shrink_to_fit();
        self.prune_events();
        self.metrics = self.measure();
        remap
    }

    fn prune_events(&mut self) {
        let mut keep = vec![false; self.events.len()];
        for node in &self.nodes {
            if let Some(event) = node_event(node) {
                keep[event.0 as usize] = true;
            }
        }
        let mut remap = vec![None; self.events.len()];
        let mut events = Vec::with_capacity(keep.iter().filter(|kept| **kept).count());
        for (index, event) in self.events.iter().enumerate() {
            if keep[index] {
                let id = FlowEventId(
                    u32::try_from(events.len())
                        .expect("retained ENT events exceed the u32 identity space"),
                );
                remap[index] = Some(id);
                events.push(event.clone());
            }
        }
        for node in &mut self.nodes {
            if let Some(event) = node_event_mut(node) {
                *event = remap[event.0 as usize].expect("retained node event retained");
            }
        }
        self.events = events;
    }

    fn measure(&self) -> DerivationMetrics {
        let mut metrics = DerivationMetrics::default();
        for root in &self.roots {
            match &self.nodes[root.node.0 as usize] {
                DerivationNode::SourceGoal { .. }
                | DerivationNode::JoinGoal { .. }
                | DerivationNode::MaterializedGoal { .. } => metrics.opaque_goal_roots += 1,
                DerivationNode::GoalProjection { .. } => metrics.projected_goal_roots += 1,
                DerivationNode::L0Contradiction { .. }
                | DerivationNode::GoalContradiction { .. }
                | DerivationNode::JoinContradiction { .. }
                | DerivationNode::MaterializedContradiction { .. } => {
                    metrics.contradiction_roots += 1;
                }
                _ => metrics.bounds_roots += 1,
            }
        }
        metrics.unique_nodes = u32::try_from(self.nodes.len())
            .expect("retained ENT derivations exceed the u32 metric space");
        let parent_edges: usize = self.nodes.iter().map(DerivationNode::parent_count).sum();
        metrics.parent_edges = u32::try_from(parent_edges)
            .expect("retained ENT parent edges exceed the u32 metric space");
        metrics.maximum_depth = self.depths.iter().copied().max().unwrap_or(0);
        metrics.retained_bytes = self.nodes.capacity() * size_of::<DerivationNode>()
            + self.events.capacity() * size_of::<FlowEvent>()
            + self.roots.capacity() * size_of::<DerivationRoot>()
            + self.depths.capacity() * size_of::<u32>()
            + self
                .nodes
                .iter()
                .map(|node| match node {
                    DerivationNode::JoinBound { parents, .. }
                    | DerivationNode::JoinDistinct { parents, .. }
                    | DerivationNode::JoinGoal { parents, .. }
                    | DerivationNode::JoinContradiction { parents, .. } => {
                        parents.capacity() * size_of::<JoinParent>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
            + self
                .events
                .iter()
                .filter_map(|event| event.node_path.as_ref())
                .map(|path| path.components.capacity() * size_of::<u32>())
                .sum::<usize>();
        metrics
    }
}

fn compare_node_ties(left: &DerivationNode, right: &DerivationNode) -> std::cmp::Ordering {
    let rank = left.rank().cmp(&right.rank());
    if !rank.is_eq() {
        return rank;
    }
    let mut index = 0;
    loop {
        match (tie_component(left, index), tie_component(right, index)) {
            (Some(left), Some(right)) => {
                let ordering = left.cmp(&right);
                if !ordering.is_eq() {
                    return ordering;
                }
            }
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
        }
        index += 1;
    }
}

fn tie_component(node: &DerivationNode, index: usize) -> Option<u32> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. } => (index == 0).then_some(event.0),
        DerivationNode::ImplicitBound { kind, .. } => (index == 0).then_some(match kind {
            ImplicitBoundKind::Reflexive => 0,
            ImplicitBoundKind::Constant => 1,
            ImplicitBoundKind::TypeMinimum => 2,
            ImplicitBoundKind::TypeMaximum => 3,
            ImplicitBoundKind::ArrayLength => 4,
        }),
        DerivationNode::TransitiveBound { first, second, .. } => {
            [first.0, second.0].get(index).copied()
        }
        DerivationNode::StrengthenedBound { weak, distinct, .. } => {
            [weak.0, distinct.0].get(index).copied()
        }
        DerivationNode::Equality {
            forward, reverse, ..
        } => [forward.0, reverse.0].get(index).copied(),
        DerivationNode::GoalContradiction {
            positive, negative, ..
        } => [positive.0, negative.0].get(index).copied(),
        DerivationNode::SubsumedBound { parent, .. }
        | DerivationNode::DisequalityFromStrictBound { parent, .. }
        | DerivationNode::GoalProjection { parent, .. }
        | DerivationNode::L0Contradiction { parent, .. } => (index == 0).then_some(parent.0),
        DerivationNode::JoinBound { event, parents, .. }
        | DerivationNode::JoinDistinct { event, parents, .. }
        | DerivationNode::JoinGoal { event, parents, .. }
        | DerivationNode::JoinContradiction { event, parents, .. } => {
            if index == 0 {
                Some(event.0)
            } else {
                let parent_index = (index - 1) / 2;
                let parent = parents.get(parent_index)?;
                if (index - 1).is_multiple_of(2) {
                    Some(parent.ordinal)
                } else {
                    Some(parent.parent.0)
                }
            }
        }
        DerivationNode::MaterializedBound { event, parent, .. }
        | DerivationNode::MaterializedDistinct { event, parent, .. }
        | DerivationNode::MaterializedGoal { event, parent, .. }
        | DerivationNode::MaterializedContradiction { event, parent, .. } => {
            [parent.0, event.0].get(index).copied()
        }
    }
}

fn node_event(node: &DerivationNode) -> Option<FlowEventId> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. }
        | DerivationNode::JoinBound { event, .. }
        | DerivationNode::JoinDistinct { event, .. }
        | DerivationNode::JoinGoal { event, .. }
        | DerivationNode::JoinContradiction { event, .. }
        | DerivationNode::MaterializedBound { event, .. }
        | DerivationNode::MaterializedDistinct { event, .. }
        | DerivationNode::MaterializedGoal { event, .. }
        | DerivationNode::MaterializedContradiction { event, .. } => Some(*event),
        _ => None,
    }
}

fn node_event_mut(node: &mut DerivationNode) -> Option<&mut FlowEventId> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. }
        | DerivationNode::JoinBound { event, .. }
        | DerivationNode::JoinDistinct { event, .. }
        | DerivationNode::JoinGoal { event, .. }
        | DerivationNode::JoinContradiction { event, .. }
        | DerivationNode::MaterializedBound { event, .. }
        | DerivationNode::MaterializedDistinct { event, .. }
        | DerivationNode::MaterializedGoal { event, .. }
        | DerivationNode::MaterializedContradiction { event, .. } => Some(event),
        _ => None,
    }
}

fn remap_id(id: &mut DerivationId, remap: &[Option<DerivationId>]) {
    *id = remap[id.0 as usize].expect("retained node parent retained");
}

fn remap_node(node: &mut DerivationNode, remap: &[Option<DerivationId>]) {
    match node {
        DerivationNode::TransitiveBound { first, second, .. } => {
            remap_id(first, remap);
            remap_id(second, remap);
        }
        DerivationNode::StrengthenedBound { weak, distinct, .. } => {
            remap_id(weak, remap);
            remap_id(distinct, remap);
        }
        DerivationNode::Equality {
            forward, reverse, ..
        } => {
            remap_id(forward, remap);
            remap_id(reverse, remap);
        }
        DerivationNode::GoalContradiction {
            positive, negative, ..
        } => {
            remap_id(positive, remap);
            remap_id(negative, remap);
        }
        DerivationNode::JoinBound { parents, .. }
        | DerivationNode::JoinDistinct { parents, .. }
        | DerivationNode::JoinGoal { parents, .. }
        | DerivationNode::JoinContradiction { parents, .. } => {
            for parent in parents {
                remap_id(&mut parent.parent, remap);
            }
        }
        DerivationNode::SubsumedBound { parent, .. }
        | DerivationNode::DisequalityFromStrictBound { parent, .. }
        | DerivationNode::GoalProjection { parent, .. }
        | DerivationNode::L0Contradiction { parent, .. }
        | DerivationNode::MaterializedBound { parent, .. }
        | DerivationNode::MaterializedDistinct { parent, .. }
        | DerivationNode::MaterializedGoal { parent, .. }
        | DerivationNode::MaterializedContradiction { parent, .. } => remap_id(parent, remap),
        DerivationNode::SourceBound { .. }
        | DerivationNode::SourceDistinct { .. }
        | DerivationNode::SourceGoal { .. }
        | DerivationNode::ImplicitBound { .. } => {}
    }
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
        let id = GoalId(
            u32::try_from(self.records.len())
                .expect("ENT goal inventory exceeds the u32 identity space"),
        );
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
        (0..self.records.len()).map(|index| {
            GoalId(u32::try_from(index).expect("ENT goal inventory exceeds the u32 identity space"))
        })
    }

    pub(crate) fn into_inventory(self) -> Vec<RetainedGoal> {
        self.records
            .into_iter()
            .map(|record| RetainedGoal {
                expression: record.expression,
                projection: record.projection,
            })
            .collect()
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
    pub(crate) event_kind: FlowEventKind,
}

/// One live fact state on the structural flow [ENT-3].
#[derive(Clone, Debug, Default)]
pub(crate) struct FactState {
    /// The loop rule's empty join: the contradictory all-derivable state, in
    /// which every relation is derivable and every fact is present. Z has
    /// empty support, so `Z - Z <= -1` never dies and the flag is absorbing
    /// under kills [ENT-4, ENT-5].
    pub(crate) all_derivable: bool,
    /// Exact reason the state is all-derivable. Every transition that sets
    /// the flag sets this handle at the same time.
    pub(crate) contradiction: Option<DerivationId>,
    /// Live difference bounds `left - right <= bound`, smallest bound kept.
    pub(crate) bounds: HashMap<(TermId, TermId), i128>,
    pub(crate) bound_proofs: HashMap<(TermId, TermId), DerivationId>,
    /// Live disequalities, stored with ordered term pair.
    pub(crate) distinct: HashSet<(TermId, TermId)>,
    pub(crate) distinct_proofs: HashMap<(TermId, TermId), DerivationId>,
    /// [ENT-3] comparison origins (b): `own Bool` bindings whose initializer
    /// comparison is still valid on every path from initializer to here.
    pub(crate) origins: HashMap<BindingId, Relation>,
    /// [ENT-3] S7/S10 outcome origins: bindings holding the outcome of a
    /// checked-arithmetic or bounded boundary call, under the same no-kill,
    /// no-`set` path discipline the comparison origins carry.
    pub(crate) outcomes: HashMap<BindingId, OutcomeFact>,
    /// Live exact signed whole-goal facts [ENT-2..ENT-4].
    pub(crate) opaque: HashSet<(GoalId, GoalSign)>,
    pub(crate) opaque_proofs: HashMap<(GoalId, GoalSign), DerivationId>,
    /// Complete still-valid pure/total origin expansion of an ordinary let.
    /// The binding's own direct value goal is intentionally separate.
    pub(crate) goal_origins: HashMap<BindingId, GoalId>,
}

impl FactState {
    /// Establishes one normalized source bound and returns that exact source
    /// parent even when a stronger fact was already live. S11 uses this at a
    /// true body entry so the retained root names the executed header proof
    /// point instead of whichever equivalent bound won fact-state
    /// canonicalization.
    pub(crate) fn establish_bound_with_proof(
        &mut self,
        left: TermId,
        right: TermId,
        bound: i128,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) -> DerivationId {
        let relation = Relation::Bound { left, right, bound };
        let proof = ledger.intern(DerivationNode::SourceBound {
            relation,
            left,
            right,
            bound,
            event,
        });
        if !self.all_derivable {
            self.add_bound(left, right, bound, proof, ledger);
        }
        proof
    }

    /// Returns the live parent proving this directed bound without rerunning
    /// closure. Immediately after S11 materializes its preheader snapshot,
    /// this is either the exact materialized bound or the materialized
    /// contradiction that proves every requested relation.
    pub(crate) fn bound_parent(
        &self,
        left: TermId,
        right: TermId,
        requested: i128,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        self.bounds
            .get(&(left, right))
            .is_some_and(|held| *held <= requested)
            .then(|| self.bound_proofs[&(left, right)])
    }

    pub(crate) fn establish(
        &mut self,
        relation: &Relation,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) {
        if self.all_derivable {
            return;
        }
        match relation {
            Relation::Bound { left, right, bound } => {
                self.establish_bound_with_proof(*left, *right, *bound, ledger, event);
            }
            Relation::Equal { left, right } => {
                let forward = ledger.intern(DerivationNode::SourceBound {
                    relation: relation.clone(),
                    left: *left,
                    right: *right,
                    bound: 0,
                    event,
                });
                self.add_bound(*left, *right, 0, forward, ledger);
                let reverse = ledger.intern(DerivationNode::SourceBound {
                    relation: relation.clone(),
                    left: *right,
                    right: *left,
                    bound: 0,
                    event,
                });
                self.add_bound(*right, *left, 0, reverse, ledger);
            }
            Relation::Distinct { left, right } => {
                let pair = ordered(*left, *right);
                let proof = ledger.intern(DerivationNode::SourceDistinct {
                    left: pair.0,
                    right: pair.1,
                    event,
                });
                if self.distinct.insert(pair) || ledger.better(proof, self.distinct_proofs[&pair]) {
                    self.distinct_proofs.insert(pair, proof);
                }
            }
        }
    }

    pub(crate) fn establish_goal(
        &mut self,
        goal: GoalId,
        sign: GoalSign,
        ledger: &mut DerivationLedger,
        event: FlowEventId,
    ) {
        if !self.all_derivable {
            let fact = (goal, sign);
            let proof = ledger.intern(DerivationNode::SourceGoal { goal, sign, event });
            if self.opaque.insert(fact) || ledger.better(proof, self.opaque_proofs[&fact]) {
                self.opaque_proofs.insert(fact, proof);
            }
        }
    }

    fn add_bound(
        &mut self,
        left: TermId,
        right: TermId,
        bound: i128,
        proof: DerivationId,
        ledger: &DerivationLedger,
    ) {
        let pair = (left, right);
        let entry = self.bounds.entry((left, right)).or_insert(bound);
        if bound < *entry
            || (bound == *entry
                && self
                    .bound_proofs
                    .get(&pair)
                    .is_none_or(|current| ledger.better(proof, *current)))
        {
            *entry = bound;
            self.bound_proofs.insert(pair, proof);
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
            self.bound_proofs.remove(&pair);
        }
        self.distinct
            .retain(|(left, right)| !killed(*left) && !killed(*right));
        self.distinct_proofs
            .retain(|(left, right), _| !killed(*left) && !killed(*right));
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
        self.opaque_proofs.retain(|(goal, _), _| !killed(*goal));
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
    contradiction: Option<DerivationId>,
    bounds: HashMap<(TermId, TermId), i128>,
    bound_proofs: HashMap<(TermId, TermId), DerivationId>,
    distinct: HashSet<(TermId, TermId)>,
    distinct_proofs: HashMap<(TermId, TermId), DerivationId>,
    opaque: HashSet<(GoalId, GoalSign)>,
    opaque_proofs: HashMap<(GoalId, GoalSign), DerivationId>,
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

    pub(crate) fn contradiction_proof(&self) -> Option<DerivationId> {
        self.contradiction
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

    pub(crate) fn opaque_proof(&self, goal: GoalId, sign: GoalSign) -> Option<DerivationId> {
        (!self.all_derivable)
            .then(|| self.opaque_proofs.get(&(goal, sign)).copied())
            .flatten()
    }

    pub(crate) fn bound_proof(
        &self,
        left: TermId,
        right: TermId,
        requested: i128,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let held = *self.bounds.get(&(left, right))?;
        if held > requested {
            return None;
        }
        let parent = self.bound_proofs[&(left, right)];
        if held == requested {
            Some(parent)
        } else {
            Some(ledger.intern(DerivationNode::SubsumedBound {
                left,
                right,
                held,
                requested,
                parent,
            }))
        }
    }

    pub(crate) fn relation_proof(
        &self,
        relation: &Relation,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        match relation {
            Relation::Bound { left, right, bound } => {
                self.bound_proof(*left, *right, *bound, ledger)
            }
            Relation::Equal { left, right } => {
                let forward = self.bound_proof(*left, *right, 0, ledger)?;
                let reverse = self.bound_proof(*right, *left, 0, ledger)?;
                Some(ledger.intern(DerivationNode::Equality {
                    left: *left,
                    right: *right,
                    forward,
                    reverse,
                }))
            }
            Relation::Distinct { left, right } => {
                let pair = ordered(*left, *right);
                let mut best = self.distinct_proofs.get(&pair).copied();
                for (from, to) in [(*left, *right), (*right, *left)] {
                    if let Some(parent) = self.bound_proof(from, to, -1, ledger) {
                        let candidate = ledger.intern(DerivationNode::DisequalityFromStrictBound {
                            left: pair.0,
                            right: pair.1,
                            parent,
                        });
                        if best.is_none_or(|current| ledger.better(candidate, current)) {
                            best = Some(candidate);
                        }
                    }
                }
                best
            }
        }
    }

    pub(crate) fn goal_projection_proof(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        if self.all_derivable {
            return self.contradiction;
        }
        let mut relation = goals.projection(goal)?.clone();
        if sign == GoalSign::Negative {
            relation = relation.negated();
        }
        let parent = self.relation_proof(&relation, ledger)?;
        Some(ledger.intern(DerivationNode::GoalProjection {
            goal,
            sign,
            relation,
            parent,
        }))
    }

    fn goal_proof(
        &self,
        goal: GoalId,
        sign: GoalSign,
        goals: &GoalTable,
        ledger: &mut DerivationLedger,
    ) -> Option<DerivationId> {
        self.opaque_proof(goal, sign)
            .or_else(|| self.goal_projection_proof(goal, sign, goals, ledger))
    }
}

/// Computes the [ENT-4] closure of `state` over the registered terms.
pub(crate) fn close(
    state: &FactState,
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
) -> ClosedState {
    if state.all_derivable {
        return ClosedState {
            all_derivable: true,
            contradiction: state.contradiction,
            bounds: HashMap::new(),
            bound_proofs: HashMap::new(),
            distinct: HashSet::new(),
            distinct_proofs: HashMap::new(),
            opaque: HashSet::new(),
            opaque_proofs: HashMap::new(),
        };
    }
    let mut bounds = state.bounds.clone();
    let mut bound_proofs = state.bound_proofs.clone();
    let mut distinct = state.distinct.clone();
    let mut distinct_proofs = state.distinct_proofs.clone();
    {
        let mut add = |left: TermId,
                       right: TermId,
                       bound: i128,
                       kind: ImplicitBoundKind,
                       ledger: &mut DerivationLedger| {
            let node = DerivationNode::ImplicitBound {
                left,
                right,
                bound,
                kind,
            };
            insert_closed_candidate(
                &mut bounds,
                &mut bound_proofs,
                left,
                right,
                bound,
                node,
                ledger,
            );
        };
        // Implicit facts [ENT-2]: reflexive bounds, fragment-type ranges, the
        // constant fold through Z, and array length equalities registered on the
        // length term itself by the flow.
        for id in terms.ids() {
            add(id, id, 0, ImplicitBoundKind::Reflexive, ledger);
            match terms.kind(id) {
                TermKind::Zero | TermKind::ConstParameter(_) => {}
                TermKind::Constant(value) => {
                    add(id, ZERO, *value, ImplicitBoundKind::Constant, ledger);
                    add(ZERO, id, -value, ImplicitBoundKind::Constant, ledger);
                }
                TermKind::Place(_, ty) | TermKind::ProjectedPlace(_, ty) => {
                    let (minimum, maximum) = type_range(*ty);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
                }
                TermKind::Length(_) | TermKind::ProjectedLength(_) => {
                    let (minimum, maximum) = type_range(IntegerType::U64);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
                    match terms.length_bound(id) {
                        Some(LengthBound::Constant(length)) => {
                            add(id, ZERO, length, ImplicitBoundKind::ArrayLength, ledger);
                            add(ZERO, id, -length, ImplicitBoundKind::ArrayLength, ledger);
                        }
                        Some(LengthBound::Equal(parameter)) => {
                            add(id, parameter, 0, ImplicitBoundKind::ArrayLength, ledger);
                            add(parameter, id, 0, ImplicitBoundKind::ArrayLength, ledger);
                        }
                        None => {}
                    }
                }
                TermKind::CountedCapture { .. } => {
                    let (minimum, maximum) = type_range(IntegerType::U64);
                    add(id, ZERO, maximum, ImplicitBoundKind::TypeMaximum, ledger);
                    add(ZERO, id, -minimum, ImplicitBoundKind::TypeMinimum, ledger);
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
                let first_proof = bound_proofs[&(*left, *middle)];
                for right in &ids {
                    let Some(second) = bounds.get(&(*middle, *right)).copied() else {
                        continue;
                    };
                    let second_proof = bound_proofs[&(*middle, *right)];
                    let via = first.saturating_add(second);
                    let node = DerivationNode::TransitiveBound {
                        left: *left,
                        middle: *middle,
                        right: *right,
                        bound: via,
                        first: first_proof,
                        second: second_proof,
                    };
                    changed |= insert_closed_candidate(
                        &mut bounds,
                        &mut bound_proofs,
                        *left,
                        *right,
                        via,
                        node,
                        ledger,
                    );
                }
            }
        }
        // ENT-4 makes every strict bound a disequality in either
        // orientation. Retain that derived fact in this same fixed point so
        // it can strengthen an available weak bound and so ENT-5 joins can
        // intersect the complete closed disequality set.
        for left in &ids {
            for right in &ids {
                if left == right
                    || !bounds
                        .get(&(*left, *right))
                        .is_some_and(|bound| *bound <= -1)
                {
                    continue;
                }
                let pair = ordered(*left, *right);
                let node = DerivationNode::DisequalityFromStrictBound {
                    left: pair.0,
                    right: pair.1,
                    parent: bound_proofs[&(*left, *right)],
                };
                let accepted = distinct_proofs
                    .get(&pair)
                    .is_none_or(|current| ledger.candidate_better(&node, *current));
                if accepted {
                    let proof = ledger.intern(node);
                    distinct.insert(pair);
                    distinct_proofs.insert(pair, proof);
                    changed = true;
                }
            }
        }
        let mut distinct_pairs: Vec<_> = distinct.iter().copied().collect();
        distinct_pairs.sort_unstable();
        for (left, right) in distinct_pairs {
            for (from, to) in [(left, right), (right, left)] {
                if bounds.get(&(from, to)).is_some_and(|bound| *bound == 0) {
                    let node = DerivationNode::StrengthenedBound {
                        left: from,
                        right: to,
                        bound: -1,
                        weak: bound_proofs[&(from, to)],
                        distinct: distinct_proofs[&ordered(left, right)],
                    };
                    changed |= insert_closed_candidate(
                        &mut bounds,
                        &mut bound_proofs,
                        from,
                        to,
                        -1,
                        node,
                        ledger,
                    );
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut contradiction = None;
    for id in &ids {
        if bounds.get(&(*id, *id)).is_some_and(|bound| *bound < 0) {
            let candidate = ledger.intern(DerivationNode::L0Contradiction {
                term: *id,
                parent: bound_proofs[&(*id, *id)],
            });
            if contradiction.is_none_or(|current| ledger.better(candidate, current)) {
                contradiction = Some(candidate);
            }
        }
    }
    let mut closed = ClosedState {
        all_derivable: contradiction.is_some(),
        contradiction,
        bounds,
        bound_proofs,
        distinct,
        distinct_proofs,
        opaque: state.opaque.clone(),
        opaque_proofs: state.opaque_proofs.clone(),
    };
    if !closed.all_derivable {
        for goal in goals.ids() {
            if !closed.derives_goal(goal, GoalSign::Positive, goals)
                || !closed.derives_goal(goal, GoalSign::Negative, goals)
            {
                continue;
            }
            let positive = closed.goal_proof(goal, GoalSign::Positive, goals, ledger);
            let negative = closed.goal_proof(goal, GoalSign::Negative, goals, ledger);
            if let (Some(positive), Some(negative)) = (positive, negative) {
                let candidate = ledger.intern(DerivationNode::GoalContradiction {
                    goal,
                    positive,
                    negative,
                });
                if closed
                    .contradiction
                    .is_none_or(|current| ledger.better(candidate, current))
                {
                    closed.contradiction = Some(candidate);
                }
            }
        }
        closed.all_derivable = closed.contradiction.is_some();
    }
    closed
}

fn insert_closed_candidate(
    bounds: &mut HashMap<(TermId, TermId), i128>,
    proofs: &mut HashMap<(TermId, TermId), DerivationId>,
    left: TermId,
    right: TermId,
    bound: i128,
    node: DerivationNode,
    ledger: &mut DerivationLedger,
) -> bool {
    let pair = (left, right);
    let accepted = match bounds.get(&pair).copied() {
        None => true,
        Some(current) if bound < current => true,
        Some(current) if bound == current => ledger.candidate_better(&node, proofs[&pair]),
        Some(_) => false,
    };
    if !accepted {
        return false;
    }
    let proof = ledger.intern(node);
    bounds.insert(pair, bound);
    proofs.insert(pair, proof);
    true
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
    ledger: &mut DerivationLedger,
) -> FactState {
    let closed = close(state, terms, goals, ledger);
    let event = ledger.event(FlowEventKind::Snapshot, None);
    if closed.all_derivable {
        let parent = closed.contradiction.expect("contradictory closure proof");
        let proof = ledger.intern(DerivationNode::MaterializedContradiction { event, parent });
        return FactState {
            all_derivable: true,
            contradiction: Some(proof),
            ..FactState::default()
        };
    }
    let mut bound_proofs = HashMap::new();
    let mut bound_keys: Vec<_> = closed.bounds.keys().copied().collect();
    bound_keys.sort_unstable();
    for (left, right) in bound_keys {
        let bound = closed.bounds[&(left, right)];
        let proof = ledger.intern(DerivationNode::MaterializedBound {
            left,
            right,
            bound,
            event,
            parent: closed.bound_proofs[&(left, right)],
        });
        bound_proofs.insert((left, right), proof);
    }
    let mut distinct_proofs = HashMap::new();
    let mut distinct_keys: Vec<_> = closed.distinct.iter().copied().collect();
    distinct_keys.sort_unstable();
    for (left, right) in distinct_keys {
        let proof = ledger.intern(DerivationNode::MaterializedDistinct {
            left,
            right,
            event,
            parent: closed.distinct_proofs[&(left, right)],
        });
        distinct_proofs.insert((left, right), proof);
    }
    let mut opaque_proofs = HashMap::new();
    let mut opaque_keys: Vec<_> = closed.opaque.iter().copied().collect();
    opaque_keys.sort_unstable();
    for (goal, sign) in opaque_keys {
        let proof = ledger.intern(DerivationNode::MaterializedGoal {
            goal,
            sign,
            event,
            parent: closed.opaque_proofs[&(goal, sign)],
        });
        opaque_proofs.insert((goal, sign), proof);
    }
    FactState {
        all_derivable: false,
        contradiction: None,
        bounds: closed.bounds,
        bound_proofs,
        distinct: closed.distinct,
        distinct_proofs,
        origins: state.origins.clone(),
        outcomes: state.outcomes.clone(),
        opaque: closed.opaque,
        opaque_proofs,
        goal_origins: state.goal_origins.clone(),
    }
}

/// [ENT-5] join of arm-exit states, each already taken after its scope-exit
/// kills. Each input is closed first; the join keeps, per ordered term pair,
/// the weakest bound held by all, and each disequality held by all. The empty
/// join is the contradictory all-derivable state.
pub(crate) fn join(
    states: &[FactState],
    terms: &TermTable,
    goals: &GoalTable,
    ledger: &mut DerivationLedger,
) -> FactState {
    let event = ledger.event(FlowEventKind::Join, None);
    // Close before filtering: a contradiction established immediately before
    // an edge is already the absorbing all-derivable state even when no kill
    // had occasion to materialize its flag.
    let closed: Vec<ClosedState> = states
        .iter()
        .map(|state| close(state, terms, goals, ledger))
        .collect();
    let contributing: Vec<usize> = closed
        .iter()
        .enumerate()
        .filter_map(|(index, state)| (!state.contradictory()).then_some(index))
        .collect();
    let Some((&first_index, rest_indices)) = contributing.split_first() else {
        let parents = closed
            .iter()
            .enumerate()
            .map(|(ordinal, state)| JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent: state
                    .contradiction
                    .expect("every noncontributing join edge is contradictory"),
            })
            .collect();
        let proof = ledger.intern(DerivationNode::JoinContradiction { event, parents });
        return FactState {
            all_derivable: true,
            contradiction: Some(proof),
            ..FactState::default()
        };
    };
    let first = &closed[first_index];
    let mut bounds = HashMap::new();
    let mut bound_proofs = HashMap::new();
    let mut first_bound_keys: Vec<_> = first.bounds.keys().copied().collect();
    first_bound_keys.sort_unstable();
    for pair in first_bound_keys {
        let bound = first.bounds[&pair];
        let mut weakest = bound;
        let held = rest_indices.iter().all(|index| {
            closed[*index].bounds.get(&pair).is_some_and(|other| {
                if *other > weakest {
                    weakest = *other;
                }
                true
            })
        });
        if held {
            let mut parents = Vec::with_capacity(states.len());
            for (ordinal, state) in closed.iter().enumerate() {
                let parent = if state.contradictory() {
                    state
                        .contradiction
                        .expect("contradictory predecessor proof")
                } else {
                    state
                        .bound_proof(pair.0, pair.1, weakest, ledger)
                        .expect("contributing predecessor proves joined bound")
                };
                parents.push(JoinParent {
                    ordinal: u32::try_from(ordinal)
                        .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                    parent,
                });
            }
            let proof = ledger.intern(DerivationNode::JoinBound {
                left: pair.0,
                right: pair.1,
                bound: weakest,
                event,
                parents,
            });
            bounds.insert(pair, weakest);
            bound_proofs.insert(pair, proof);
        }
    }
    let mut distinct = first.distinct.clone();
    for index in rest_indices {
        distinct.retain(|pair| closed[*index].distinct.contains(pair));
    }
    let mut distinct_proofs = HashMap::new();
    let mut distinct_keys: Vec<_> = distinct.iter().copied().collect();
    distinct_keys.sort_unstable();
    for pair in distinct_keys {
        let mut parents = Vec::with_capacity(states.len());
        for (ordinal, state) in closed.iter().enumerate() {
            let parent = if state.contradictory() {
                state
                    .contradiction
                    .expect("contradictory predecessor proof")
            } else {
                state.distinct_proofs[&pair]
            };
            parents.push(JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent,
            });
        }
        let proof = ledger.intern(DerivationNode::JoinDistinct {
            left: pair.0,
            right: pair.1,
            event,
            parents,
        });
        distinct_proofs.insert(pair, proof);
    }
    // Comparison and outcome origins are path conditions, not facts; one
    // survives a join only when every contributing path carries the same one.
    let mut opaque = first.opaque.clone();
    for index in rest_indices {
        opaque.retain(|fact| closed[*index].opaque.contains(fact));
    }
    let mut opaque_proofs = HashMap::new();
    let mut opaque_keys: Vec<_> = opaque.iter().copied().collect();
    opaque_keys.sort_unstable();
    for (goal, sign) in opaque_keys {
        let mut parents = Vec::with_capacity(states.len());
        for (ordinal, state) in closed.iter().enumerate() {
            let parent = if state.contradictory() {
                state
                    .contradiction
                    .expect("contradictory predecessor proof")
            } else {
                state.opaque_proofs[&(goal, sign)]
            };
            parents.push(JoinParent {
                ordinal: u32::try_from(ordinal)
                    .expect("ENT join predecessor ordinal exceeds the u32 identity space"),
                parent,
            });
        }
        let proof = ledger.intern(DerivationNode::JoinGoal {
            goal,
            sign,
            event,
            parents,
        });
        opaque_proofs.insert((goal, sign), proof);
    }
    let contributing_states: Vec<&FactState> =
        contributing.iter().map(|index| &states[*index]).collect();
    let mut origins = contributing_states[0].origins.clone();
    let mut outcomes = contributing_states[0].outcomes.clone();
    let mut goal_origins = contributing_states[0].goal_origins.clone();
    for state in contributing_states.iter().skip(1) {
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
        contradiction: None,
        bounds,
        bound_proofs,
        distinct,
        distinct_proofs,
        origins,
        outcomes,
        opaque,
        opaque_proofs,
        goal_origins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_join_remains_the_contradictory_all_derivable_state() {
        let mut ledger = DerivationLedger::default();
        let joined = join(&[], &TermTable::new(), &GoalTable::default(), &mut ledger);
        assert!(joined.all_derivable);
        let proof = joined
            .contradiction
            .expect("the empty join has one exact contradiction proof");
        assert!(matches!(
            ledger.nodes[proof.0 as usize],
            DerivationNode::JoinContradiction { ref parents, event }
                if parents.is_empty()
                    && ledger.events[event.0 as usize].kind == FlowEventKind::Join
        ));
        ledger.add_root(DerivationRootKind::BoundsObligation(0), proof);
        let remap = ledger.finish();
        assert_eq!(ledger.nodes.len(), 1);
        assert_eq!(ledger.roots.len(), 1);
        assert_eq!(remap[proof.0 as usize], Some(DerivationId(0)));
    }
}
