//! Forward authority tracking for CLM-1 claim locality.
//!
//! This analysis is deliberately separate from PRV provenance.  Every user or
//! system call result starts a boundary-result authority, even when PRV proves
//! that the result has no external origin.  The pass records the reaching value
//! authority at each claim point; entailment can then query those frozen
//! snapshots with its canonical goal supports.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use super::goal::GoalProjection;
use super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedFunction, CheckedLoopId,
    CheckedMatchArm, CheckedMode, CheckedNominal, CheckedNominalKind, CheckedSetTarget,
    CheckedSliceSource, CheckedStatement, CheckedType, FunctionId, IntegerType,
};
use crate::{NodePath, SemanticCompilerFailure};

type LocalityResult<T> = Result<T, SemanticCompilerFailure>;

/// The call class that introduced one boundary-result value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryResultKind {
    UserCall(FunctionId),
    SystemCall(u8),
}

/// Stable call occurrence plus scratch call kind.  `FunctionId` is retained
/// only until the checker maps it to the callee's source declaration and name;
/// it must never be rendered or published directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundaryWitness {
    pub(crate) kind: BoundaryResultKind,
    pub(crate) call: NodePath,
}

impl BoundaryWitness {
    /// Deterministic source order.  The kind/id suffix is only an invariant
    /// tie-break for an impossible pair of different calls at one node path.
    pub(crate) fn source_cmp(&self, other: &Self) -> Ordering {
        self.call
            .components()
            .cmp(other.call.components())
            .then_with(|| boundary_kind_cmp(&self.kind, &other.kind))
    }
}

fn boundary_kind_cmp(left: &BoundaryResultKind, right: &BoundaryResultKind) -> Ordering {
    match (left, right) {
        (BoundaryResultKind::UserCall(left), BoundaryResultKind::UserCall(right)) => {
            left.0.cmp(&right.0)
        }
        (BoundaryResultKind::SystemCall(left), BoundaryResultKind::SystemCall(right)) => {
            left.cmp(right)
        }
        (BoundaryResultKind::UserCall(_), BoundaryResultKind::SystemCall(_)) => Ordering::Less,
        (BoundaryResultKind::SystemCall(_), BoundaryResultKind::UserCall(_)) => Ordering::Greater,
    }
}

fn earlier_owned(
    left: Option<BoundaryWitness>,
    right: Option<BoundaryWitness>,
) -> Option<BoundaryWitness> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left.source_cmp(&right).is_le() {
            left
        } else {
            right
        }),
        (Some(witness), None) | (None, Some(witness)) => Some(witness),
        (None, None) => None,
    }
}

fn earlier_ref<'a>(
    left: Option<&'a BoundaryWitness>,
    right: Option<&'a BoundaryWitness>,
) -> Option<&'a BoundaryWitness> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left.source_cmp(right).is_le() {
            left
        } else {
            right
        }),
        (Some(witness), None) | (None, Some(witness)) => Some(witness),
        (None, None) => None,
    }
}

/// Identity of one reaching definition of one value component.
///
/// A merge selects a component exactly when the definitions reaching it along
/// the incoming edges are different definition occurrences; a component every
/// edge reaches through one definition is unchanged by the merge, however the
/// edge itself was chosen.  Each identity is derived from the address of the
/// checked statement that produced it, so re-walking a loop body re-derives
/// the same identity and the fixed point still converges.  The address is
/// scratch: it is compared only with another identity of the same component
/// and is never rendered, published, or ordered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DefinitionId {
    site: usize,
    kind: DefinitionKind,
}

/// Which definition one checked statement address denotes.  A statement may
/// hold more than one definition occurrence, and a merge point may sit at the
/// same address as the statement that owns it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DefinitionKind {
    /// The value a function entry, an unwritten slot, or a freshly computed
    /// operand carries before any definition site claims it.
    Entry,
    /// The binding or storage component this statement writes.
    Written,
    /// The previous value a `replace` binds beside its write.
    Taken,
    /// A matching binder the arm introduces.
    Binder,
    /// The reaching definition a control-flow merge itself creates.
    Merge,
    /// The union of two reaching definitions formed by a data operation
    /// rather than by an edge choice.  Every producer stamps its own identity
    /// over this one before the value enters a state.
    Fused,
}

impl DefinitionId {
    const ENTRY: Self = Self {
        site: 0,
        kind: DefinitionKind::Entry,
    };

    const FUSED: Self = Self {
        site: 0,
        kind: DefinitionKind::Fused,
    };

    fn at(site: usize, kind: DefinitionKind) -> Self {
        Self { site, kind }
    }
}

/// One lexical path-control frame.  `site` is the stable address of the
/// checked statement during this analysis; it is never rendered or exposed.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ControlFrame {
    site: usize,
    witness: BoundaryWitness,
}

/// Boundary-dependent path conditions currently restricting execution.
/// Lexical frames retain nested controls even when they name the same boundary
/// call and reach a fixed point when a loop revisits the same selector.  A
/// frame is never removed: every merge asks which frames its own edges
/// acquired since the merge's entry state, so a frame the entry already
/// carries selects nothing at that merge.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ControlAuthority {
    frames: Vec<ControlFrame>,
}

impl ControlAuthority {
    fn with_added(&self, site: usize, witness: Option<BoundaryWitness>) -> Self {
        let mut result = self.clone();
        if let Some(witness) = witness {
            match result.frames.iter_mut().find(|frame| frame.site == site) {
                Some(frame) => {
                    frame.witness = earlier_owned(Some(frame.witness.clone()), Some(witness))
                        .expect("two present witnesses have an earliest member");
                }
                None => result.frames.push(ControlFrame { site, witness }),
            }
            result.frames.sort_by_key(|frame| frame.site);
        }
        result
    }

    /// The earliest witness among the frames these edges acquired since
    /// `self`.  This is the authority of the edge choice a merge whose entry
    /// state carried `self` performs, and nothing else: a frame `self` already
    /// carries chose the path into the merge's entry, not between its edges.
    fn acquired<'a>(&self, edges: impl IntoIterator<Item = &'a Self>) -> Option<BoundaryWitness> {
        let mut selected = None;
        for edge in edges {
            for frame in &edge.frames {
                if self.frames.iter().any(|held| held.site == frame.site) {
                    continue;
                }
                selected = earlier_owned(selected, Some(frame.witness.clone()));
            }
        }
        selected
    }

    fn join(&self, other: &Self) -> Self {
        let mut joined = self.clone();
        for frame in &other.frames {
            match joined
                .frames
                .iter_mut()
                .find(|candidate| candidate.site == frame.site)
            {
                Some(candidate) => {
                    candidate.witness =
                        earlier_owned(Some(candidate.witness.clone()), Some(frame.witness.clone()))
                            .expect("two present witnesses have an earliest member");
                }
                None => joined.frames.push(frame.clone()),
            }
        }
        joined.frames.sort_by_key(|frame| frame.site);
        joined
    }
}

/// One immediate value component.  Component trees stay sparse: a uniform
/// marker denotes every descendant until an exact strong update materializes
/// the immediate shape and replaces only the selected child.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum AuthorityStep {
    Field(u32),
    EnumTag,
    EnumPayload { variant: u32, field: u32 },
    Element,
    Length,
    Deref,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuthorityValue {
    ty: CheckedType,
    /// Identity of the definition this component currently reaches.  Only a
    /// merge reads it, and only to compare two reaching definitions of the
    /// same component.
    definition: DefinitionId,
    /// Authority of the value identity itself.  This is normally the same as
    /// `uniform`, but survives a strong write to one owned dereference so a
    /// returned box/arena holder cannot be laundered by replacing its content.
    identity: Option<BoundaryWitness>,
    /// Applies uniformly to the complete value exactly while `children` is
    /// empty.  Once a component is materialized, the children are complete for
    /// that immediate shape and this slot is cleared.
    uniform: Option<BoundaryWitness>,
    children: Vec<(AuthorityStep, AuthorityValue)>,
}

impl AuthorityValue {
    fn local(ty: CheckedType) -> Self {
        Self {
            ty,
            definition: DefinitionId::ENTRY,
            identity: None,
            uniform: None,
            children: Vec::new(),
        }
    }

    fn uniform(ty: CheckedType, witness: Option<BoundaryWitness>) -> Self {
        Self {
            ty,
            definition: DefinitionId::ENTRY,
            identity: witness.clone(),
            uniform: witness,
            children: Vec::new(),
        }
    }

    fn aggregate(&self) -> Option<BoundaryWitness> {
        if self.children.is_empty() {
            return earlier_owned(self.identity.clone(), self.uniform.clone());
        }
        self.children
            .iter()
            .fold(self.identity.clone(), |selected, (_, child)| {
                earlier_owned(selected, child.aggregate())
            })
    }

    fn aggregate_ref(&self) -> Option<&BoundaryWitness> {
        if self.children.is_empty() {
            return earlier_ref(self.identity.as_ref(), self.uniform.as_ref());
        }
        self.children
            .iter()
            .fold(self.identity.as_ref(), |selected, (_, child)| {
                earlier_ref(selected, child.aggregate_ref())
            })
    }

    /// Record that this whole value is the reaching definition `definition`
    /// creates.  Every descendant carries the same identity, so a component a
    /// later partial write does not touch still compares equal after either
    /// side has been materialized.
    fn stamp(&mut self, definition: DefinitionId) {
        self.definition = definition;
        for (_, child) in &mut self.children {
            child.stamp(definition);
        }
    }

    fn stamped(mut self, definition: DefinitionId) -> Self {
        self.stamp(definition);
        self
    }

    fn union_uniform(&mut self, witness: Option<&BoundaryWitness>) {
        let Some(witness) = witness else {
            return;
        };
        self.identity = earlier_owned(self.identity.take(), Some(witness.clone()));
        if self.children.is_empty() {
            self.uniform = earlier_owned(self.uniform.take(), Some(witness.clone()));
            return;
        }
        for (_, child) in &mut self.children {
            child.union_uniform(Some(witness));
        }
    }

    fn immediate_shape(
        ty: CheckedType,
        nominals: &[CheckedNominal],
    ) -> LocalityResult<Vec<(AuthorityStep, CheckedType)>> {
        let shape = match ty {
            CheckedType::Nominal(nominal) => {
                let nominal = nominals
                    .get(nominal.0 as usize)
                    .filter(|candidate| candidate.id == nominal)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                match &nominal.kind {
                    CheckedNominalKind::Struct { fields } => fields
                        .iter()
                        .enumerate()
                        .map(|(field, declaration)| {
                            Ok((
                                AuthorityStep::Field(
                                    u32::try_from(field)
                                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                                ),
                                declaration.ty,
                            ))
                        })
                        .collect::<LocalityResult<Vec<_>>>()?,
                    CheckedNominalKind::Enum { variants } => {
                        let mut shape = vec![(AuthorityStep::EnumTag, CheckedType::Bool)];
                        for (variant, declaration) in variants.iter().enumerate() {
                            let variant = u32::try_from(variant)
                                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                            for (field, declaration) in declaration.fields.iter().enumerate() {
                                shape.push((
                                    AuthorityStep::EnumPayload {
                                        variant,
                                        field: u32::try_from(field).map_err(|_| {
                                            SemanticCompilerFailure::CounterOverflow
                                        })?,
                                    },
                                    declaration.ty,
                                ));
                            }
                        }
                        shape
                    }
                    CheckedNominalKind::Box { referent } => {
                        vec![(AuthorityStep::Deref, *referent)]
                    }
                    CheckedNominalKind::Arena { content, .. } => {
                        vec![(AuthorityStep::Deref, *content)]
                    }
                    CheckedNominalKind::ArenaStorage
                    | CheckedNominalKind::SystemResource { .. } => Vec::new(),
                }
            }
            CheckedType::Array { element, .. }
            | CheckedType::Slice { element, .. }
            | CheckedType::Buffer { element } => vec![
                (AuthorityStep::Element, element.ty()),
                (
                    AuthorityStep::Length,
                    CheckedType::Integer(IntegerType::U64),
                ),
            ],
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_)
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_) => Vec::new(),
        };
        Ok(shape)
    }

    fn materialize(&mut self, nominals: &[CheckedNominal]) -> LocalityResult<()> {
        if !self.children.is_empty() {
            return Ok(());
        }
        let shape = Self::immediate_shape(self.ty, nominals)?;
        if shape.is_empty() {
            return Ok(());
        }
        let inherited = self.uniform.take();
        let definition = self.definition;
        self.children = shape
            .into_iter()
            .map(|(step, ty)| {
                (
                    step,
                    Self::uniform(ty, inherited.clone()).stamped(definition),
                )
            })
            .collect();
        Ok(())
    }

    fn child_type(
        &self,
        step: AuthorityStep,
        nominals: &[CheckedNominal],
    ) -> LocalityResult<CheckedType> {
        Self::immediate_shape(self.ty, nominals)?
            .into_iter()
            .find_map(|(candidate, ty)| (candidate == step).then_some(ty))
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn selected(&self, step: AuthorityStep, nominals: &[CheckedNominal]) -> LocalityResult<Self> {
        if self.children.is_empty() {
            return Ok(
                Self::uniform(self.child_type(step, nominals)?, self.uniform.clone())
                    .stamped(self.definition),
            );
        }
        self.children
            .iter()
            .find_map(|(candidate, value)| (candidate == &step).then_some(value.clone()))
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn selected_path(
        &self,
        path: &[AuthorityStep],
        nominals: &[CheckedNominal],
    ) -> LocalityResult<Self> {
        let mut selected = self.clone();
        for step in path {
            selected = selected.selected(*step, nominals)?;
        }
        Ok(selected)
    }

    fn witness_path(&self, path: &[AuthorityStep]) -> Option<&BoundaryWitness> {
        if path.is_empty() {
            return self.aggregate_ref();
        }
        if self.children.is_empty() {
            return self.uniform.as_ref();
        }
        let (first, rest) = path.split_first()?;
        self.children
            .iter()
            .find_map(|(candidate, value)| (candidate == first).then_some(value))
            .and_then(|value| value.witness_path(rest))
    }

    /// Returns the authority of the selected value identity without folding
    /// in any sibling content.  Dereference carries the holder identity into
    /// its referent, but a boundary-derived field stored beside a local field
    /// does not make the locally allocated holder itself boundary-derived.
    fn identity_path(&self, path: &[AuthorityStep]) -> Option<&BoundaryWitness> {
        if path.is_empty() {
            return self.identity.as_ref();
        }
        if self.children.is_empty() {
            return self.uniform.as_ref();
        }
        let (first, rest) = path.split_first()?;
        self.children
            .iter()
            .find_map(|(candidate, value)| (candidate == first).then_some(value))
            .and_then(|value| value.identity_path(rest))
    }

    fn replace_path(
        &mut self,
        path: &[AuthorityStep],
        replacement: Self,
        nominals: &[CheckedNominal],
    ) -> LocalityResult<()> {
        let Some((first, rest)) = path.split_first() else {
            *self = replacement;
            return Ok(());
        };
        self.materialize(nominals)?;
        let child = self
            .children
            .iter_mut()
            .find_map(|(candidate, value)| (candidate == first).then_some(value))
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        child.replace_path(rest, replacement, nominals)
    }

    /// A possible-overlap write joins rather than replaces.  The written
    /// component becomes the definition `definition` names, because a reader
    /// after this statement reaches this write and not the older one alone.
    fn union_path(
        &mut self,
        path: &[AuthorityStep],
        value: &Self,
        definition: DefinitionId,
        nominals: &[CheckedNominal],
    ) -> LocalityResult<()> {
        let current = self.selected_path(path, nominals)?;
        let joined = current.join(value, nominals)?.stamped(definition);
        self.replace_path(path, joined, nominals)
    }

    /// Union two authorities of one component as a data operation.  The result
    /// is not a reaching definition of its own; every caller stamps the
    /// identity of the definition that consumes it.
    fn join(&self, other: &Self, nominals: &[CheckedNominal]) -> LocalityResult<Self> {
        if self.ty != other.ty {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let definition = if self.definition == other.definition {
            self.definition
        } else {
            DefinitionId::FUSED
        };
        if self.children.is_empty() && other.children.is_empty() {
            return Ok(Self {
                ty: self.ty,
                definition,
                identity: earlier_owned(self.identity.clone(), other.identity.clone()),
                uniform: earlier_owned(self.uniform.clone(), other.uniform.clone()),
                children: Vec::new(),
            });
        }
        let mut left = self.clone();
        let mut right = other.clone();
        left.materialize(nominals)?;
        right.materialize(nominals)?;
        if left.children.len() != right.children.len() {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let mut children = Vec::with_capacity(left.children.len());
        for ((left_step, left), (right_step, right)) in left.children.iter().zip(&right.children) {
            if left_step != right_step {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
            children.push((*left_step, left.join(right, nominals)?));
        }
        Ok(Self {
            ty: self.ty,
            definition,
            identity: earlier_owned(left.identity, right.identity),
            uniform: None,
            children,
        })
    }

    /// Merge two reaching states of one component at a control-flow join.
    ///
    /// `selection` is the authority of the edge choice this merge performs and
    /// `merge` names the reaching definition the merge itself creates.  The
    /// selection joins exactly those components whose incoming definitions are
    /// different definition occurrences: those are the components the edge
    /// chooses.  A component both edges reach through one definition keeps that
    /// definition and takes only the ordinary authority union, so a value the
    /// selected edge never redefined stays `Local`.
    fn merge(
        &self,
        other: &Self,
        nominals: &[CheckedNominal],
        selection: Option<&BoundaryWitness>,
        merge: DefinitionId,
    ) -> LocalityResult<Self> {
        if self.ty != other.ty {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        if self.definition != other.definition {
            let mut selected = self.join(other, nominals)?;
            selected.union_uniform(selection);
            return Ok(selected.stamped(merge));
        }
        let definition = self.definition;
        if self.children.is_empty() && other.children.is_empty() {
            return Ok(Self {
                ty: self.ty,
                definition,
                identity: earlier_owned(self.identity.clone(), other.identity.clone()),
                uniform: earlier_owned(self.uniform.clone(), other.uniform.clone()),
                children: Vec::new(),
            });
        }
        let mut left = self.clone();
        let mut right = other.clone();
        left.materialize(nominals)?;
        right.materialize(nominals)?;
        if left.children.len() != right.children.len() {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let mut children = Vec::with_capacity(left.children.len());
        for ((left_step, left), (right_step, right)) in left.children.iter().zip(&right.children) {
            if left_step != right_step {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
            children.push((*left_step, left.merge(right, nominals, selection, merge)?));
        }
        Ok(Self {
            ty: self.ty,
            definition,
            identity: earlier_owned(left.identity, right.identity),
            uniform: None,
            children,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AuthorityState {
    bindings: Vec<Option<AuthorityValue>>,
    /// Boundary discriminants controlling paths reaching this point.  A claim
    /// observes their earliest witness even when all explicit support values
    /// were defined before the branch.
    control: ControlAuthority,
}

impl AuthorityState {
    fn binding(&self, binding: BindingId, ty: CheckedType) -> LocalityResult<AuthorityValue> {
        match self.bindings.get(binding.0 as usize) {
            Some(Some(value)) if value.ty == ty => Ok(value.clone()),
            Some(Some(_)) => Err(SemanticCompilerFailure::InvalidResolution),
            Some(None) | None => Ok(AuthorityValue::local(ty)),
        }
    }

    fn raw_binding(&self, binding: BindingId) -> LocalityResult<&AuthorityValue> {
        self.bindings
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    /// Install a whole-value definition.  The stamp is what later merges read:
    /// a slot every incoming edge reaches through this one definition is not
    /// selected by the edge choice.
    fn set_binding(&mut self, binding: BindingId, value: AuthorityValue, definition: DefinitionId) {
        let index = binding.0 as usize;
        if self.bindings.len() <= index {
            self.bindings.resize(index + 1, None);
        }
        self.bindings[index] = Some(value.stamped(definition));
    }

    fn merge(
        &self,
        other: &Self,
        nominals: &[CheckedNominal],
        selection: Option<&BoundaryWitness>,
        merge: DefinitionId,
    ) -> LocalityResult<Self> {
        let mut merged = Self {
            bindings: Vec::with_capacity(self.bindings.len().max(other.bindings.len())),
            control: self.control.join(&other.control),
        };
        for index in 0..self.bindings.len().max(other.bindings.len()) {
            let value = match (
                self.bindings.get(index).and_then(Option::as_ref),
                other.bindings.get(index).and_then(Option::as_ref),
            ) {
                (Some(left), Some(right)) => Some(left.merge(right, nominals, selection, merge)?),
                // A slot only one edge carries is a binding declared inside
                // that edge and out of scope at the merge.
                (Some(value), None) | (None, Some(value)) => Some(value.clone()),
                (None, None) => None,
            };
            merged.bindings.push(value);
        }
        Ok(merged)
    }
}

#[derive(Clone, Debug)]
enum HolderReferent {
    Place {
        binding: BindingId,
        fields: Vec<u32>,
    },
    Holder(BindingId),
    /// A borrow parameter or borrowed match binder.  Its binding slot models
    /// the opaque referent directly for locality updates.
    Opaque,
    /// An owning box/arena whose stored content is the value-tree `Deref`
    /// component rather than a separately named local place.
    OwnedDeref,
}

#[derive(Clone, Debug)]
struct GiveEdge {
    state: AuthorityState,
    value: AuthorityValue,
}

#[derive(Default)]
struct FlowResult {
    normal: Option<AuthorityState>,
    breaks: HashMap<CheckedLoopId, Vec<AuthorityState>>,
    gives: Vec<GiveEdge>,
}

impl FlowResult {
    fn normal(state: AuthorityState) -> Self {
        Self {
            normal: Some(state),
            ..Self::default()
        }
    }

    fn append_abrupt(&mut self, mut other: Self) {
        for (target, mut states) in other.breaks.drain() {
            self.breaks.entry(target).or_default().append(&mut states);
        }
        self.gives.append(&mut other.gives);
    }
}

/// Frozen reaching-value authority for every claim occurrence in one checked
/// function.  It is computed once for the baseline and is independent of S3
/// masks, entailment facts, callee summaries, and PRV classifications.
#[derive(Clone)]
pub(crate) struct ClaimAuthorityAnalysis {
    snapshots: Arc<HashMap<NodePath, AuthorityState>>,
    holders: Arc<Vec<Option<HolderReferent>>>,
}

impl ClaimAuthorityAnalysis {
    pub(crate) fn analyze(
        function: &CheckedFunction,
        nominals: &[CheckedNominal],
    ) -> LocalityResult<Self> {
        // Most functions contain no written claim.  Avoid collecting holders
        // and running a second semantic walk for those functions; the empty
        // analysis remains a clone-cheap inventory value for masked runs.
        if !block_contains_claim(&function.body) {
            return Ok(Self {
                snapshots: Arc::new(HashMap::new()),
                holders: Arc::new(Vec::new()),
            });
        }
        let holders = collect_holders(function, nominals)?;
        let mut pass = AuthorityPass {
            nominals,
            holders,
            snapshots: HashMap::new(),
        };
        let mut entry = AuthorityState::default();
        for parameter in &function.parameters {
            entry.set_binding(
                parameter.binding,
                AuthorityValue::local(parameter.ty),
                DefinitionId::ENTRY,
            );
        }
        let _ = pass.walk_block(&function.body, entry)?;
        Ok(Self {
            snapshots: Arc::new(pass.snapshots),
            holders: Arc::new(pass.holders),
        })
    }

    /// Returns the earliest boundary result read by one canonical goal support
    /// at `claim`.  A dereference consults both the holder value and its resolved
    /// referent, so a borrow-mode call result cannot disappear merely because
    /// OWN-6 roots it at a local actual.
    ///
    /// A claim occurrence standing on a boundary-selected edge contributes
    /// nothing by itself: the selector's witness reaches this query only
    /// through a support whose reaching definition that selector chose.
    pub(crate) fn witness(
        &self,
        claim: &NodePath,
        root: BindingId,
        projections: &[GoalProjection],
        length: bool,
    ) -> Option<&BoundaryWitness> {
        let state = self.snapshots.get(claim)?;
        let mut selected = None;
        let mut binding = root;
        let mut path = Vec::new();

        for projection in projections {
            match projection {
                GoalProjection::Field(field) => path.push(AuthorityStep::Field(*field)),
                GoalProjection::Deref => {
                    selected = earlier_ref(
                        selected,
                        state
                            .bindings
                            .get(binding.0 as usize)
                            .and_then(Option::as_ref)
                            .and_then(|value| value.identity_path(&path)),
                    );
                    if path.is_empty() {
                        let mut followed = false;
                        for _ in 0..=self.holders.len() {
                            match self
                                .holders
                                .get(binding.0 as usize)
                                .and_then(Option::as_ref)
                            {
                                Some(HolderReferent::Place {
                                    binding: referent,
                                    fields,
                                }) => {
                                    binding = *referent;
                                    path =
                                        fields.iter().copied().map(AuthorityStep::Field).collect();
                                    followed = true;
                                    break;
                                }
                                Some(HolderReferent::Holder(next)) => {
                                    binding = *next;
                                    selected = earlier_ref(
                                        selected,
                                        state
                                            .bindings
                                            .get(binding.0 as usize)
                                            .and_then(Option::as_ref)
                                            .and_then(|value| value.identity.as_ref()),
                                    );
                                }
                                Some(HolderReferent::Opaque) => {
                                    // An opaque borrow parameter stores the
                                    // referent value directly in its own slot.
                                    // A reborrow chain must therefore resume
                                    // selection at this terminal holder rather
                                    // than append an owned-indirection step.
                                    followed = true;
                                    break;
                                }
                                Some(HolderReferent::OwnedDeref) | None => break,
                            }
                        }
                        if followed {
                            continue;
                        }
                    }
                    path.push(AuthorityStep::Deref);
                }
            }
        }
        if length {
            path.push(AuthorityStep::Length);
        }
        earlier_ref(
            selected,
            state
                .bindings
                .get(binding.0 as usize)
                .and_then(Option::as_ref)
                .and_then(|value| value.witness_path(&path)),
        )
    }
}

struct AuthorityPass<'a> {
    nominals: &'a [CheckedNominal],
    holders: Vec<Option<HolderReferent>>,
    snapshots: HashMap<NodePath, AuthorityState>,
}

impl AuthorityPass<'_> {
    /// A claim inside a loop is reached by more than one state.  Merging them
    /// is not an ordinary edge choice with a dominator to measure against, so
    /// the selection is every boundary control either state stands under: a
    /// support whose reaching definition differs between two arrivals is
    /// chosen by whatever brought this occurrence there.
    fn record_snapshot(&mut self, claim: &NodePath, state: &AuthorityState) -> LocalityResult<()> {
        match self.snapshots.get_mut(claim) {
            Some(previous) => {
                let selection =
                    ControlAuthority::default().acquired([&previous.control, &state.control]);
                *previous = previous.merge(
                    state,
                    self.nominals,
                    selection.as_ref(),
                    DefinitionId::at(std::ptr::from_ref(claim).addr(), DefinitionKind::Merge),
                )?;
            }
            None => {
                self.snapshots.insert(claim.clone(), state.clone());
            }
        }
        Ok(())
    }

    fn walk_block(
        &mut self,
        statements: &[CheckedStatement],
        entry: AuthorityState,
    ) -> LocalityResult<FlowResult> {
        let mut result = FlowResult::normal(entry);
        for statement in statements {
            let Some(state) = result.normal.take() else {
                break;
            };
            let mut step = self.walk_statement(statement, state)?;
            result.normal = step.normal.take();
            result.append_abrupt(step);
        }
        Ok(result)
    }

    fn walk_statement(
        &mut self,
        statement: &CheckedStatement,
        mut state: AuthorityState,
    ) -> LocalityResult<FlowResult> {
        let control_site = std::ptr::from_ref(statement).addr();
        let written = DefinitionId::at(control_site, DefinitionKind::Written);
        match statement {
            CheckedStatement::Let { binding, value, .. } => {
                let value = self.expression(value, &state)?;
                state.set_binding(*binding, value, written);
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::PropagateLet {
                binding,
                scrutinee,
                ok_type,
                ..
            } => {
                let scrutinee = self.expression(scrutinee, &state)?;
                let mut value = scrutinee.selected(
                    AuthorityStep::EnumPayload {
                        variant: 0,
                        field: 0,
                    },
                    self.nominals,
                )?;
                if value.ty != *ok_type {
                    value = AuthorityValue::uniform(*ok_type, value.aggregate());
                }
                let selector = self.match_selector_witness(&scrutinee)?;
                // The bound payload is the delivered value the Ok edge selects.
                value.union_uniform(selector.as_ref());
                state.set_binding(*binding, value, written);
                // Only the Ok edge reaches the following statement, so that
                // continuation itself reveals the Result discriminant.
                state.control = state.control.with_added(control_site, selector);
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::Set { target, value, .. } => {
                let offset = self.target_selector_witness(target, &state)?;
                let mut value = self.expression(value, &state)?;
                value.union_uniform(offset.as_ref());
                self.write_target(&mut state, target, value, offset.as_ref(), written)?;
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::Replace {
                binding,
                target,
                value,
                ..
            } => {
                let offset = self.target_selector_witness(target, &state)?;
                let mut previous = self.read_target(target, &state)?;
                previous.union_uniform(offset.as_ref());
                let mut replacement = self.expression(value, &state)?;
                replacement.union_uniform(offset.as_ref());
                self.write_target(&mut state, target, replacement, offset.as_ref(), written)?;
                state.set_binding(
                    *binding,
                    previous,
                    DefinitionId::at(control_site, DefinitionKind::Taken),
                );
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::Evaluate(value) | CheckedStatement::DropExpression { value, .. } => {
                let _ = self.expression(value, &state)?;
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::Claim {
                condition, site, ..
            } => {
                self.record_snapshot(&site.node_path, &state)?;
                let _ = self.expression(condition, &state)?;
                Ok(FlowResult::normal(state))
            }
            CheckedStatement::Return { value, .. } => {
                let _ = self.expression(value, &state)?;
                Ok(FlowResult::default())
            }
            CheckedStatement::Give { value, .. } => {
                // The delivery merge reads this edge's own control, so a give
                // reached through a boundary selector carries that selector
                // into the initializer without tainting the value here.
                let value = self.expression(value, &state)?;
                Ok(FlowResult {
                    gives: vec![GiveEdge { state, value }],
                    ..FlowResult::default()
                })
            }
            CheckedStatement::Break { target, .. } => {
                let mut breaks = HashMap::new();
                breaks.insert(*target, vec![state]);
                Ok(FlowResult {
                    breaks,
                    ..FlowResult::default()
                })
            }
            CheckedStatement::Match {
                scrutinee, arms, ..
            } => self.walk_match(control_site, scrutinee, arms, state, None),
            CheckedStatement::ValueMatchLet {
                binding,
                result_type,
                scrutinee,
                arms,
                ..
            } => self.walk_match(
                control_site,
                scrutinee,
                arms,
                state,
                Some((*binding, *result_type)),
            ),
            CheckedStatement::Loop { id, body, .. } => {
                self.walk_loop(control_site, *id, body, state)
            }
            CheckedStatement::CountedRange {
                id,
                binder,
                lower,
                upper,
                body,
                ..
            } => self.walk_counted(control_site, *id, *binder, lower, upper, body, state),
            CheckedStatement::Region { body, .. } => self.walk_block(body, state),
        }
    }

    fn walk_match(
        &mut self,
        control_site: usize,
        scrutinee: &CheckedExpression,
        arms: &[CheckedMatchArm],
        entry: AuthorityState,
        receiver: Option<(BindingId, CheckedType)>,
    ) -> LocalityResult<FlowResult> {
        let scrutinee_value = self.expression(scrutinee, &entry)?;
        let selector = self.match_selector_witness(&scrutinee_value)?;
        let incoming_control = entry.control.clone();
        let branch_control = incoming_control.with_added(control_site, selector.clone());
        let merge = DefinitionId::at(control_site, DefinitionKind::Merge);
        let mut exits = Vec::new();
        let mut abrupt = FlowResult::default();
        let mut deliveries = Vec::new();
        for arm in arms {
            let mut arm_state = entry.clone();
            arm_state.control = branch_control.clone();
            for binder in &arm.binders {
                let mut value = scrutinee_value.selected(
                    AuthorityStep::EnumPayload {
                        variant: arm.tag,
                        field: binder.field,
                    },
                    self.nominals,
                )?;
                if value.ty != binder.ty {
                    value = AuthorityValue::uniform(binder.ty, value.aggregate());
                }
                // The binder is the payload the arm's own tag selects.
                value.union_uniform(selector.as_ref());
                arm_state.set_binding(
                    binder.binding,
                    value,
                    DefinitionId::at(control_site, DefinitionKind::Binder),
                );
            }
            let mut arm_result = self.walk_block(&arm.body, arm_state)?;
            if let Some(exit) = arm_result.normal.take() {
                exits.push(exit);
            }
            deliveries.append(&mut arm_result.gives);
            abrupt.append_abrupt(arm_result);
        }

        if let Some((binding, result_type)) = receiver {
            // Every normal value-initializer edge is a `give`; arm fallthrough
            // is deliberately ignored.  Nested initializers consume their own
            // gives before returning here.
            if deliveries.is_empty() {
                abrupt.normal = None;
                return Ok(abrupt);
            }
            // A `value_if` or `value_match` delivers the value its selector
            // chooses, so the selector joins the delivered value whatever the
            // arms deliver.  A nested selector reaching a give edge is carried
            // by that edge's own control.
            let selection = incoming_control.acquired(
                deliveries
                    .iter()
                    .map(|edge| &edge.state.control)
                    .collect::<Vec<_>>(),
            );
            let mut states = Vec::with_capacity(deliveries.len());
            let mut delivered: Option<AuthorityValue> = None;
            for edge in deliveries {
                states.push(edge.state);
                delivered = Some(match delivered {
                    Some(previous) => previous.join(&edge.value, self.nominals)?,
                    None => edge.value,
                });
            }
            let mut state = merge_states(&states, self.nominals, selection.as_ref(), merge)?
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let mut delivered = delivered.ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if delivered.ty != result_type {
                delivered = AuthorityValue::uniform(result_type, delivered.aggregate());
            }
            delivered.union_uniform(selector.as_ref());
            delivered.union_uniform(selection.as_ref());
            state.set_binding(
                binding,
                delivered,
                DefinitionId::at(control_site, DefinitionKind::Written),
            );
            abrupt.normal = Some(state);
            abrupt.gives.clear();
            return Ok(abrupt);
        }

        // The selector chooses among the arms' reaching definitions at this
        // reconvergence.  A component every arm reaches through one definition
        // is not chosen here, however the arm itself was selected.
        let selection = incoming_control.acquired(exits.iter().map(|exit| &exit.control));
        abrupt.normal = merge_states(&exits, self.nominals, selection.as_ref(), merge)?;
        abrupt.gives = deliveries;
        Ok(abrupt)
    }

    /// Only the discriminant selects a match arm.  Payload authority flows
    /// through the selected binder, but an unrelated boundary payload in a
    /// locally constructed enum must not become implicit control authority.
    fn match_selector_witness(
        &self,
        scrutinee: &AuthorityValue,
    ) -> LocalityResult<Option<BoundaryWitness>> {
        match scrutinee.ty {
            CheckedType::Bool => Ok(scrutinee.aggregate()),
            CheckedType::Nominal(nominal) => {
                let nominal = self
                    .nominals
                    .get(nominal.0 as usize)
                    .filter(|candidate| candidate.id == nominal)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if matches!(nominal.kind, CheckedNominalKind::Enum { .. }) {
                    Ok(scrutinee
                        .selected(AuthorityStep::EnumTag, self.nominals)?
                        .aggregate())
                } else {
                    Err(SemanticCompilerFailure::InvalidResolution)
                }
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution),
        }
    }

    fn walk_loop(
        &mut self,
        control_site: usize,
        id: CheckedLoopId,
        body: &[CheckedStatement],
        entry: AuthorityState,
    ) -> LocalityResult<FlowResult> {
        let merge = DefinitionId::at(control_site, DefinitionKind::Merge);
        let mut head = entry.clone();
        let final_result = loop {
            let body_result = self.walk_block(body, head.clone())?;
            let next = match &body_result.normal {
                Some(backedge) => {
                    // Whatever selected the backedge selects the loop head
                    // between the entry definition and the body's own.
                    let selection = entry.control.acquired([&backedge.control]);
                    entry.merge(backedge, self.nominals, selection.as_ref(), merge)?
                }
                None => head.clone(),
            };
            if next == head {
                break body_result;
            }
            head = next;
        };
        let mut result = final_result;
        let exits = result.breaks.remove(&id).unwrap_or_default();
        let selection = entry
            .control
            .acquired(exits.iter().map(|exit| &exit.control));
        result.normal = merge_states(&exits, self.nominals, selection.as_ref(), merge)?;
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn walk_counted(
        &mut self,
        control_site: usize,
        id: CheckedLoopId,
        binder: BindingId,
        lower: &CheckedExpression,
        upper: &CheckedExpression,
        body: &[CheckedStatement],
        entry: AuthorityState,
    ) -> LocalityResult<FlowResult> {
        let lower = self.expression(lower, &entry)?;
        let upper = self.expression(upper, &entry)?;
        let endpoint = earlier_owned(lower.aggregate(), upper.aggregate());
        let entry_control = entry.control.clone();
        let loop_control = entry.control.with_added(control_site, endpoint.clone());
        let merge = DefinitionId::at(control_site, DefinitionKind::Merge);
        let binder_definition = DefinitionId::at(control_site, DefinitionKind::Binder);
        let mut initial = entry.clone();
        initial.control = loop_control.clone();
        // The counted binder is the endpoint-selected value of this iteration.
        initial.set_binding(
            binder,
            AuthorityValue::uniform(CheckedType::Integer(IntegerType::U64), endpoint.clone()),
            binder_definition,
        );
        let mut head = initial.clone();
        let final_result = loop {
            let mut body_result = self.walk_block(body, head.clone())?;
            if let Some(backedge) = &mut body_result.normal {
                backedge.set_binding(
                    binder,
                    AuthorityValue::uniform(
                        CheckedType::Integer(IntegerType::U64),
                        endpoint.clone(),
                    ),
                    binder_definition,
                );
            }
            let next = match &body_result.normal {
                Some(backedge) => {
                    let selection = entry_control.acquired([&initial.control, &backedge.control]);
                    initial.merge(backedge, self.nominals, selection.as_ref(), merge)?
                }
                None => head.clone(),
            };
            if next == head {
                break body_result;
            }
            head = next;
        };
        let mut result = final_result;
        let has_backedge = result.normal.is_some();
        let mut exits = result.breaks.remove(&id).unwrap_or_default();
        if has_backedge {
            // A counted range is finite, so a represented backedge also has an
            // exhaustion edge after some number of iterations.
            exits.push(head);
        }

        // The false-header edge exists even when the body never runs, and the
        // endpoint chose it.  A component every exit reaches through one
        // definition is nonetheless untouched by that choice.
        let mut false_header = entry;
        false_header.control = loop_control;
        exits.push(false_header);
        let selection = entry_control.acquired(exits.iter().map(|exit| &exit.control));
        result.normal = merge_states(&exits, self.nominals, selection.as_ref(), merge)?;
        Ok(result)
    }

    fn expression(
        &self,
        expression: &CheckedExpression,
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        Ok(match expression {
            CheckedExpression::Constant(_) | CheckedExpression::NamedConstant { .. } => {
                AuthorityValue::local(expression.ty())
            }
            CheckedExpression::Binding { binding, ty, .. } => state.binding(*binding, *ty)?,
            CheckedExpression::UserCall {
                function,
                call,
                arguments,
                result,
                ..
            } => {
                let _ = self.expression_witness(arguments, state)?;
                let boundary = BoundaryWitness {
                    kind: BoundaryResultKind::UserCall(*function),
                    call: call.clone(),
                };
                AuthorityValue::uniform(*result, Some(boundary))
            }
            CheckedExpression::SystemCall {
                operation,
                call,
                arguments,
                result,
                ..
            } => {
                let _ = self.expression_witness(arguments, state)?;
                let boundary = BoundaryWitness {
                    kind: BoundaryResultKind::SystemCall(*operation),
                    call: call.clone(),
                };
                AuthorityValue::uniform(*result, Some(boundary))
            }
            CheckedExpression::IntegerOperation {
                arguments, result, ..
            } => AuthorityValue::uniform(*result, self.expression_witness(arguments, state)?),
            CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. } => {
                AuthorityValue::uniform(expression.ty(), self.expression_witness(arguments, state)?)
            }
            CheckedExpression::NumericConversion { value, result, .. } => {
                AuthorityValue::uniform(*result, self.expression(value, state)?.aggregate())
            }
            CheckedExpression::Reinterpret { value, .. } => {
                AuthorityValue::uniform(expression.ty(), self.expression(value, state)?.aggregate())
            }
            CheckedExpression::ArrayFill { ty, value, .. } => {
                let mut result = AuthorityValue::local(*ty);
                result.replace_path(
                    &[AuthorityStep::Element],
                    self.expression(value, state)?,
                    self.nominals,
                )?;
                result
            }
            CheckedExpression::ArrayLength { root, .. } => match root {
                CheckedArrayRoot::Binding { .. } => self
                    .array_root(root, state)?
                    .selected(AuthorityStep::Length, self.nominals)?,
                CheckedArrayRoot::Constant(_) => AuthorityValue::local(expression.ty()),
            },
            CheckedExpression::ArrayIndex {
                root,
                element_type,
                offset,
                ..
            } => {
                let element = match root {
                    CheckedArrayRoot::Binding { .. } => self
                        .array_root(root, state)?
                        .selected(AuthorityStep::Element, self.nominals)?,
                    CheckedArrayRoot::Constant(_) => AuthorityValue::local(*element_type),
                };
                let witness = earlier_owned(
                    element.aggregate(),
                    self.expression(offset, state)?.aggregate(),
                );
                AuthorityValue::uniform(*element_type, witness)
            }
            CheckedExpression::BufferFill {
                element,
                length,
                value,
                ..
            } => {
                let ty = CheckedType::Buffer { element: *element };
                let mut result = AuthorityValue::local(ty);
                result.replace_path(
                    &[AuthorityStep::Length],
                    AuthorityValue::uniform(
                        CheckedType::Integer(IntegerType::U64),
                        self.expression(length, state)?.aggregate(),
                    ),
                    self.nominals,
                )?;
                result.replace_path(
                    &[AuthorityStep::Element],
                    self.expression(value, state)?,
                    self.nominals,
                )?;
                result
            }
            CheckedExpression::BufferVacant {
                element, length, ..
            } => {
                let ty = CheckedType::Buffer {
                    element: super::model::CheckedFlatElement::Nominal(*element),
                };
                let mut result = AuthorityValue::local(ty);
                result.replace_path(
                    &[AuthorityStep::Length],
                    AuthorityValue::uniform(
                        CheckedType::Integer(IntegerType::U64),
                        self.expression(length, state)?.aggregate(),
                    ),
                    self.nominals,
                )?;
                result
            }
            CheckedExpression::BufferFits { length, .. } => AuthorityValue::uniform(
                CheckedType::Bool,
                self.expression(length, state)?.aggregate(),
            ),
            CheckedExpression::BufferLength { root } => self
                .buffer_root(root.binding, &root.fields, state)?
                .selected(AuthorityStep::Length, self.nominals)?,
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let element = self
                    .buffer_root(root.binding, &root.fields, state)?
                    .selected(AuthorityStep::Element, self.nominals)?;
                AuthorityValue::uniform(
                    root.element.ty(),
                    earlier_owned(
                        element.aggregate(),
                        self.expression(offset, state)?.aggregate(),
                    ),
                )
            }
            CheckedExpression::SliceOf {
                source,
                region,
                element,
                ..
            } => {
                let source = self.slice_source(source, *element, state)?;
                let mut result = AuthorityValue::local(CheckedType::Slice {
                    region: *region,
                    element: *element,
                });
                for step in [AuthorityStep::Element, AuthorityStep::Length] {
                    let selected = source.selected(step, self.nominals)?;
                    result.replace_path(&[step], selected, self.nominals)?;
                }
                result
            }
            CheckedExpression::SliceLength { root } => {
                self.read_place(root.binding, &[], state)?
                    .selected(AuthorityStep::Length, self.nominals)?
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let element = self
                    .read_place(root.binding, &[], state)?
                    .selected(AuthorityStep::Element, self.nominals)?;
                AuthorityValue::uniform(
                    root.element.ty(),
                    earlier_owned(
                        element.aggregate(),
                        self.expression(offset, state)?.aggregate(),
                    ),
                )
            }
            CheckedExpression::BoxNew { nominal, value, .. } => {
                let mut result = AuthorityValue::local(CheckedType::Nominal(*nominal));
                result.replace_path(
                    &[AuthorityStep::Deref],
                    self.expression(value, state)?,
                    self.nominals,
                )?;
                result
            }
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. } => {
                let holder = self.expression(value, state)?;
                let marker = holder.identity.clone();
                let mut content = holder.selected(AuthorityStep::Deref, self.nominals)?;
                content.union_uniform(marker.as_ref());
                content
            }
            CheckedExpression::ArenaNew { nominal, value, .. } => {
                let mut result = AuthorityValue::local(CheckedType::Nominal(*nominal));
                result.replace_path(
                    &[AuthorityStep::Deref],
                    self.expression(value, state)?,
                    self.nominals,
                )?;
                result
            }
            CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. } => AuthorityValue::local(expression.ty()),
            CheckedExpression::DerefAddressed { binding, .. } => {
                self.read_deref(*binding, &[], state)?
            }
            CheckedExpression::ConstructStruct {
                nominal, fields, ..
            } => {
                let mut result = AuthorityValue::local(CheckedType::Nominal(*nominal));
                for (field, value) in fields.iter().enumerate() {
                    result.replace_path(
                        &[AuthorityStep::Field(
                            u32::try_from(field)
                                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                        )],
                        self.expression(value, state)?,
                        self.nominals,
                    )?;
                }
                result
            }
            CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
                ..
            } => {
                let mut result = AuthorityValue::local(CheckedType::Nominal(*nominal));
                for (field, value) in fields.iter().enumerate() {
                    result.replace_path(
                        &[AuthorityStep::EnumPayload {
                            variant: *variant,
                            field: u32::try_from(field)
                                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                        }],
                        self.expression(value, state)?,
                        self.nominals,
                    )?;
                }
                result
            }
            CheckedExpression::Project {
                binding,
                fields,
                ty,
                ..
            } => {
                let value = self.read_place(*binding, fields, state)?;
                if value.ty == *ty {
                    value
                } else {
                    AuthorityValue::uniform(*ty, value.aggregate())
                }
            }
            CheckedExpression::ProjectValue {
                value, field, ty, ..
            } => {
                let value = self
                    .expression(value, state)?
                    .selected(AuthorityStep::Field(*field), self.nominals)?;
                if value.ty == *ty {
                    value
                } else {
                    AuthorityValue::uniform(*ty, value.aggregate())
                }
            }
        })
    }

    fn expression_witness(
        &self,
        expressions: &[CheckedExpression],
        state: &AuthorityState,
    ) -> LocalityResult<Option<BoundaryWitness>> {
        expressions.iter().try_fold(None, |selected, expression| {
            Ok(earlier_owned(
                selected,
                self.expression(expression, state)?.aggregate(),
            ))
        })
    }

    fn array_root(
        &self,
        root: &CheckedArrayRoot,
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        match root {
            CheckedArrayRoot::Binding { binding, fields } => {
                self.read_place(*binding, fields, state)
            }
            CheckedArrayRoot::Constant(_) => Err(SemanticCompilerFailure::InvalidResolution),
        }
    }

    fn buffer_root(
        &self,
        binding: BindingId,
        fields: &[u32],
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        self.read_place(binding, fields, state)
    }

    fn slice_source(
        &self,
        source: &CheckedSliceSource,
        element: super::model::CheckedFlatElement,
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        match source {
            CheckedSliceSource::Array { root, length } => match root {
                CheckedArrayRoot::Binding { .. } => self.array_root(root, state),
                CheckedArrayRoot::Constant(_) => Ok(AuthorityValue::local(CheckedType::Array {
                    element,
                    length: *length,
                })),
            },
            CheckedSliceSource::Buffer(root) => self.buffer_root(root.binding, &root.fields, state),
            CheckedSliceSource::ArenaContent {
                binding, fields, ..
            } => state
                .raw_binding(*binding)?
                .selected(AuthorityStep::Deref, self.nominals)?
                .selected_path(
                    &fields
                        .iter()
                        .copied()
                        .map(AuthorityStep::Field)
                        .collect::<Vec<_>>(),
                    self.nominals,
                ),
        }
    }

    fn read_place(
        &self,
        binding: BindingId,
        fields: &[u32],
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        if self
            .holders
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .is_some()
        {
            return self.read_deref(binding, fields, state);
        }
        state.raw_binding(binding)?.selected_path(
            &fields
                .iter()
                .copied()
                .map(AuthorityStep::Field)
                .collect::<Vec<_>>(),
            self.nominals,
        )
    }

    fn read_deref(
        &self,
        binding: BindingId,
        fields: &[u32],
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        let marker = self.holder_chain_marker(binding, state)?;
        let (root, mut path) = self.resolve_holder(binding)?;
        path.extend(fields.iter().copied().map(AuthorityStep::Field));
        let mut value = state
            .raw_binding(root)?
            .selected_path(&path, self.nominals)?;
        value.union_uniform(marker.as_ref());
        Ok(value)
    }

    /// Returns the earliest identity carried by any holder in one resolved
    /// chain.  A reborrow expression is locally formed, but reading through it
    /// still depends on every holder whose access capability it forwards.
    fn holder_chain_marker(
        &self,
        holder: BindingId,
        state: &AuthorityState,
    ) -> LocalityResult<Option<BoundaryWitness>> {
        let mut current = holder;
        let mut marker = None;
        for _ in 0..=self.holders.len() {
            marker = earlier_owned(marker, state.raw_binding(current)?.identity.clone());
            match self
                .holders
                .get(current.0 as usize)
                .and_then(Option::as_ref)
            {
                Some(HolderReferent::Holder(next)) => current = *next,
                Some(
                    HolderReferent::Place { .. }
                    | HolderReferent::Opaque
                    | HolderReferent::OwnedDeref,
                )
                | None => return Ok(marker),
            }
        }
        Err(SemanticCompilerFailure::InvalidResolution)
    }

    fn resolve_holder(&self, holder: BindingId) -> LocalityResult<(BindingId, Vec<AuthorityStep>)> {
        let mut current = holder;
        for _ in 0..=self.holders.len() {
            match self
                .holders
                .get(current.0 as usize)
                .and_then(Option::as_ref)
            {
                Some(HolderReferent::Place { binding, fields }) => {
                    return Ok((
                        *binding,
                        fields.iter().copied().map(AuthorityStep::Field).collect(),
                    ));
                }
                Some(HolderReferent::Holder(next)) => current = *next,
                Some(HolderReferent::Opaque) => return Ok((current, Vec::new())),
                Some(HolderReferent::OwnedDeref) => {
                    return Ok((current, vec![AuthorityStep::Deref]));
                }
                None => return Ok((current, Vec::new())),
            }
        }
        Err(SemanticCompilerFailure::InvalidResolution)
    }

    fn target_selector_witness(
        &self,
        target: &CheckedSetTarget,
        state: &AuthorityState,
    ) -> LocalityResult<Option<BoundaryWitness>> {
        Ok(match target {
            CheckedSetTarget::Place(_) => None,
            CheckedSetTarget::ArrayIndex(target) => {
                self.expression(&target.offset, state)?.aggregate()
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.expression(&target.offset, state)?.aggregate()
            }
        })
    }

    fn read_target(
        &self,
        target: &CheckedSetTarget,
        state: &AuthorityState,
    ) -> LocalityResult<AuthorityValue> {
        match target {
            CheckedSetTarget::Place(place) if self.whole_owned_holder_target(state, place)? => {
                Ok(state.raw_binding(place.binding)?.clone())
            }
            CheckedSetTarget::Place(place)
                if self
                    .holders
                    .get(place.binding.0 as usize)
                    .and_then(Option::as_ref)
                    .is_some() =>
            {
                self.read_deref(place.binding, &place.fields, state)
            }
            CheckedSetTarget::Place(place) => self.read_place(place.binding, &place.fields, state),
            CheckedSetTarget::ArrayIndex(target) => self
                .read_place(target.binding, &target.fields, state)?
                .selected(AuthorityStep::Element, self.nominals),
            CheckedSetTarget::BufferIndex(target) => self
                .read_place(target.root.binding, &target.root.fields, state)?
                .selected(AuthorityStep::Element, self.nominals),
        }
    }

    fn write_target(
        &self,
        state: &mut AuthorityState,
        target: &CheckedSetTarget,
        value: AuthorityValue,
        selector: Option<&BoundaryWitness>,
        definition: DefinitionId,
    ) -> LocalityResult<()> {
        // This function is invoked only for explicit SET-1/SET-2 commits.  Calls
        // deliberately never invoke it: call-written `&uniq` storage is outside
        // the first claim-locality batch.
        match target {
            CheckedSetTarget::Place(place) => {
                let (root, mut path) = if self.whole_owned_holder_target(state, place)? {
                    (place.binding, Vec::new())
                } else {
                    self.storage_target(place.binding)?
                };
                path.extend(place.fields.iter().copied().map(AuthorityStep::Field));
                let slot = state
                    .bindings
                    .get_mut(root.0 as usize)
                    .and_then(Option::as_mut)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                slot.replace_path(&path, value.stamped(definition), self.nominals)
            }
            CheckedSetTarget::ArrayIndex(target) => {
                let (root, mut path) = self.storage_target(target.binding)?;
                path.extend(target.fields.iter().copied().map(AuthorityStep::Field));
                path.push(AuthorityStep::Element);
                let mut value = value;
                value.union_uniform(selector);
                let slot = state
                    .bindings
                    .get_mut(root.0 as usize)
                    .and_then(Option::as_mut)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                slot.union_path(&path, &value, definition, self.nominals)
            }
            CheckedSetTarget::BufferIndex(target) => {
                let (root, mut path) = self.storage_target(target.root.binding)?;
                path.extend(target.root.fields.iter().copied().map(AuthorityStep::Field));
                path.push(AuthorityStep::Element);
                let mut value = value;
                value.union_uniform(selector);
                let slot = state
                    .bindings
                    .get_mut(root.0 as usize)
                    .and_then(Option::as_mut)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                slot.union_path(&path, &value, definition, self.nominals)
            }
        }
    }

    fn whole_owned_holder_target(
        &self,
        state: &AuthorityState,
        place: &super::model::CheckedWritablePlace,
    ) -> LocalityResult<bool> {
        Ok(matches!(
            self.holders
                .get(place.binding.0 as usize)
                .and_then(Option::as_ref),
            Some(HolderReferent::OwnedDeref)
        ) && place.fields.is_empty()
            && state.raw_binding(place.binding)?.ty == place.ty)
    }

    fn storage_target(
        &self,
        binding: BindingId,
    ) -> LocalityResult<(BindingId, Vec<AuthorityStep>)> {
        if self
            .holders
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .is_some()
        {
            self.resolve_holder(binding)
        } else {
            Ok((binding, Vec::new()))
        }
    }
}

fn merge_states(
    states: &[AuthorityState],
    nominals: &[CheckedNominal],
    selection: Option<&BoundaryWitness>,
    merge: DefinitionId,
) -> LocalityResult<Option<AuthorityState>> {
    let mut states = states.iter();
    let Some(first) = states.next() else {
        return Ok(None);
    };
    let mut merged = first.clone();
    for state in states {
        merged = merged.merge(state, nominals, selection, merge)?;
    }
    Ok(Some(merged))
}

fn block_contains_claim(statements: &[CheckedStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        CheckedStatement::Claim { .. } => true,
        CheckedStatement::Match { arms, .. } | CheckedStatement::ValueMatchLet { arms, .. } => {
            arms.iter().any(|arm| block_contains_claim(&arm.body))
        }
        CheckedStatement::Loop { body, .. }
        | CheckedStatement::CountedRange { body, .. }
        | CheckedStatement::Region { body, .. } => block_contains_claim(body),
        CheckedStatement::Let { .. }
        | CheckedStatement::PropagateLet { .. }
        | CheckedStatement::Set { .. }
        | CheckedStatement::Replace { .. }
        | CheckedStatement::Evaluate(_)
        | CheckedStatement::DropExpression { .. }
        | CheckedStatement::Return { .. }
        | CheckedStatement::Give { .. }
        | CheckedStatement::Break { .. } => false,
    })
}

fn collect_holders(
    function: &CheckedFunction,
    nominals: &[CheckedNominal],
) -> LocalityResult<Vec<Option<HolderReferent>>> {
    let mut holders = Vec::new();
    for parameter in &function.parameters {
        if matches!(parameter.mode, CheckedMode::Own)
            && is_owned_deref_type(parameter.ty, nominals)?
        {
            set_holder(&mut holders, parameter.binding, HolderReferent::OwnedDeref);
        } else if !matches!(parameter.mode, CheckedMode::Own) {
            set_holder(&mut holders, parameter.binding, HolderReferent::Opaque);
        }
    }
    collect_block_holders(&function.body, &mut holders, nominals)?;
    Ok(holders)
}

fn collect_block_holders(
    statements: &[CheckedStatement],
    holders: &mut Vec<Option<HolderReferent>>,
    nominals: &[CheckedNominal],
) -> LocalityResult<()> {
    for statement in statements {
        match statement {
            CheckedStatement::Let { binding, value, .. } => {
                let holder = match value {
                    CheckedExpression::Binding {
                        binding: source, ..
                    } => match holders.get(source.0 as usize).and_then(Option::as_ref) {
                        Some(HolderReferent::OwnedDeref) => Some(HolderReferent::OwnedDeref),
                        Some(_) => Some(HolderReferent::Holder(*source)),
                        None if is_owned_deref_type(value.ty(), nominals)? => {
                            Some(HolderReferent::OwnedDeref)
                        }
                        None => None,
                    },
                    CheckedExpression::BorrowAddressed { binding, .. }
                    | CheckedExpression::BorrowBox { binding, .. }
                    | CheckedExpression::BorrowSystemResource { binding, .. } => {
                        Some(HolderReferent::Place {
                            binding: *binding,
                            fields: Vec::new(),
                        })
                    }
                    CheckedExpression::BorrowBuffer { root, .. } => Some(HolderReferent::Place {
                        binding: root.binding,
                        fields: root.fields.clone(),
                    }),
                    CheckedExpression::ReborrowAddressed { binding, .. } => {
                        Some(HolderReferent::Holder(*binding))
                    }
                    CheckedExpression::UserCall {
                        result_borrow: Some(result),
                        ..
                    } => Some(HolderReferent::Place {
                        binding: result.binding,
                        fields: result.fields.clone(),
                    }),
                    CheckedExpression::BoxNew { .. } | CheckedExpression::ArenaNew { .. } => {
                        Some(HolderReferent::OwnedDeref)
                    }
                    _ if is_owned_deref_type(value.ty(), nominals)? => {
                        Some(HolderReferent::OwnedDeref)
                    }
                    _ => None,
                };
                if let Some(holder) = holder {
                    set_holder(holders, *binding, holder);
                }
            }
            CheckedStatement::Match { arms, .. } => {
                for arm in arms {
                    for binder in &arm.binders {
                        if matches!(binder.mode, CheckedMode::Own)
                            && is_owned_deref_type(binder.ty, nominals)?
                        {
                            set_holder(holders, binder.binding, HolderReferent::OwnedDeref);
                        } else if !matches!(binder.mode, CheckedMode::Own) {
                            set_holder(holders, binder.binding, HolderReferent::Opaque);
                        }
                    }
                    collect_block_holders(&arm.body, holders, nominals)?;
                }
            }
            CheckedStatement::ValueMatchLet {
                binding,
                result_type,
                arms,
                ..
            } => {
                if is_owned_deref_type(*result_type, nominals)? {
                    set_holder(holders, *binding, HolderReferent::OwnedDeref);
                }
                for arm in arms {
                    for binder in &arm.binders {
                        if matches!(binder.mode, CheckedMode::Own)
                            && is_owned_deref_type(binder.ty, nominals)?
                        {
                            set_holder(holders, binder.binding, HolderReferent::OwnedDeref);
                        } else if !matches!(binder.mode, CheckedMode::Own) {
                            set_holder(holders, binder.binding, HolderReferent::Opaque);
                        }
                    }
                    collect_block_holders(&arm.body, holders, nominals)?;
                }
            }
            CheckedStatement::Loop { body, .. }
            | CheckedStatement::CountedRange { body, .. }
            | CheckedStatement::Region { body, .. } => {
                collect_block_holders(body, holders, nominals)?
            }
            CheckedStatement::PropagateLet {
                binding, ok_type, ..
            } => {
                if is_owned_deref_type(*ok_type, nominals)? {
                    set_holder(holders, *binding, HolderReferent::OwnedDeref);
                }
            }
            CheckedStatement::Replace {
                binding, target, ..
            } => {
                if is_owned_deref_type(target.ty(), nominals)? {
                    set_holder(holders, *binding, HolderReferent::OwnedDeref);
                }
            }
            CheckedStatement::Set { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Claim { .. }
            | CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => {}
        }
    }
    Ok(())
}

fn is_owned_deref_type(ty: CheckedType, nominals: &[CheckedNominal]) -> LocalityResult<bool> {
    let CheckedType::Nominal(id) = ty else {
        return Ok(false);
    };
    let nominal = nominals
        .get(id.0 as usize)
        .filter(|candidate| candidate.id == id)
        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
    Ok(matches!(
        nominal.kind,
        CheckedNominalKind::Box { .. } | CheckedNominalKind::Arena { .. }
    ))
}

fn set_holder(
    holders: &mut Vec<Option<HolderReferent>>,
    binding: BindingId,
    holder: HolderReferent,
) {
    let index = binding.0 as usize;
    if holders.len() <= index {
        holders.resize(index + 1, None);
    }
    holders[index] = Some(holder);
}
