//! [REF-1] resolved places and the [OWN-7] overlap relation.
//!
//! A resolved place is a root — a local variable, a parameter, or a named
//! const [CONST-2] — and the ordered steps below it: field selections,
//! `deref` of `Box` content [TYPE-7], enum payload steps, index steps, range
//! steps [REF-4], window parts [WIN-2] and measure reads [OP-15]. That is the
//! one path type in the checker; a reference variable is not storage of its
//! own, so resolving a place rooted at one replaces that root by the path the
//! reference names, recursively [REF-1].
//!
//! [OWN-7] is the single relation every consumer asks: the call-site pairwise
//! comparison [EFF-5], reference invalidation [REF-2], the [ENT-5] kills, and
//! the [PAR-1] adjacency footprints. Two places fail to overlap exactly when
//! some step of their common prefix provably selects two different storages,
//! and a pair no admitted family separates is overlapping. Nothing here may
//! grow a private second copy of that relation.
//!
//! Two of the separations [OWN-7] admits are proofs rather than syntax — two
//! index steps proved distinct by the fixed [ENT-6] families under [MSR-4]'s
//! disposition, and two range steps proved disjoint by the four non-strict
//! orderings. Those reach the entailment fragment through [`SeparationOracle`]
//! and nothing else, and every answer is memoized per function on the pair of
//! resolved paths, which is a stable key because a path's captured index and
//! endpoint values are immutable once captured [REF-1, OWN-7].

use std::cell::RefCell;
use std::collections::HashMap;

use crate::DeclarationId;

use super::model::{
    BindingId, CheckedConstantId, CheckedExpression, CheckedFunction, CheckedMatchArm,
    CheckedMeasure, CheckedMode, CheckedStatement, CheckedType, IntegerType,
};

/// One source occurrence at which an index expression or a range endpoint was
/// evaluated [REF-1].
///
/// The value that occurrence produced is immutable: the path records it, and
/// later assignments to the variables the expression used do not change it.
/// Two steps carrying one `CaptureId` therefore hold one value, whatever the
/// program did between the two places' formation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct CaptureId(pub(crate) u32);

/// How the entailment fragment reads one captured value [ENT-2].
///
/// A written literal and a named or generic const are values this relation
/// decides by itself. A binding read at the formation is a term the fragment
/// can reason about but this module cannot, and everything else is opaque to
/// both; each of the latter two is decided, if at all, by the fixed [ENT-6]
/// families through [`SeparationOracle`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CapturedTerm {
    /// A written integer literal.
    Literal(u64),
    /// The value a live `own` integer binding held at the formation.
    Binding(BindingId),
    /// An in-scope const [CONST-1, CONST-2], fixed at instantiation [FN-2].
    Const(DeclarationId),
    /// A computed value: the fragment sees the goal, not a term this module
    /// can name.
    Opaque,
}

/// The immutable value one index step or range endpoint captured [REF-1].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CapturedValue {
    pub(crate) capture: CaptureId,
    pub(crate) term: CapturedTerm,
}

impl CapturedValue {
    pub(crate) const fn new(capture: CaptureId, term: CapturedTerm) -> Self {
        Self { capture, term }
    }

    /// The offset of a write at an element position this version cannot name
    /// [MSR-3].
    ///
    /// One shared occurrence stands for every such write, which is the
    /// conservative reading in both directions: two unknown offsets compare
    /// as one storage, so a prefix test over them invalidates rather than
    /// spares, and no admitted family proves an unknown offset distinct from
    /// anything, so [OWN-7] leaves the pair overlapping.
    pub(crate) const fn unknown() -> Self {
        Self::new(CaptureId(u32::MAX), CapturedTerm::Opaque)
    }

    /// Whether the two captures hold one value on every execution.
    ///
    /// One capture occurrence is one evaluation, so an equal `CaptureId` is
    /// equality of the value itself. Across two occurrences only the two
    /// value-determined terms decide it: a binding's two reads may straddle a
    /// write to that binding, and an opaque value names no term at all.
    pub(crate) fn provably_same(self, other: Self) -> bool {
        if self.capture == other.capture {
            return true;
        }
        match (self.term, other.term) {
            (CapturedTerm::Literal(left), CapturedTerm::Literal(right)) => left == right,
            (CapturedTerm::Const(left), CapturedTerm::Const(right)) => left == right,
            _ => false,
        }
    }

    /// The half of [OWN-7]'s index separation this module settles without a
    /// proof: two written literals of different value select two slots.
    /// Every other pair is the fixed [ENT-6] families' question.
    const fn literals_distinct(self, other: Self) -> bool {
        matches!(
            (self.term, other.term),
            (CapturedTerm::Literal(left), CapturedTerm::Literal(right)) if left != right
        )
    }

    /// The support one captured value contributes to every measure term of
    /// the place it occurs in [ENT-5]: the binding it read, where it read
    /// one.
    pub(crate) const fn support(self) -> Option<BindingId> {
        match self.term {
            CapturedTerm::Binding(binding) => Some(binding),
            CapturedTerm::Literal(_) | CapturedTerm::Const(_) | CapturedTerm::Opaque => None,
        }
    }
}

/// One captured range step's two endpoints [REF-4], both evaluated at the
/// formation under the obligation `lo <= hi` and `hi <= x.len`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CapturedRange {
    pub(crate) start: CapturedValue,
    pub(crate) end: CapturedValue,
}

/// The four window parts [WIN-2]: vocabulary for effect rows and this
/// relation only, never a place a program reads, writes, or references
/// [TYPE-10].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum WindowPart {
    /// `r.next`, the append slot at index `r.len`.
    Next,
    /// `r.last`, the last filled slot at index `r.len - 1`.
    Last,
    /// `r.filled`, every slot below `r.len`.
    Filled,
    /// `r.free`, every slot from `r.len` up.
    Free,
}

impl WindowPart {
    /// The [WIN-2] member spelling of this part, which is how an effect row
    /// and every diagnostic over one name it.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Last => "last",
            Self::Filled => "filled",
            Self::Free => "free",
        }
    }
}

/// Root of a resolved place: a function-local binding — parameters, `let`
/// bindings of every right-hand form, counted binders and match binders share
/// the dense [`BindingId`] space — or a named const [CONST-2].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceRoot {
    Binding(BindingId),
    Constant(CheckedConstantId),
}

/// One step below a resolved place's root [REF-1, OWN-7].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceStep {
    /// One struct field selection, by source ordinal.
    Field(u32),
    /// `deref` of `Box` content [TYPE-7]. It is an ordinary path step and is
    /// not erased: `deref(h).value` and `deref(h.value)` are two paths.
    Deref,
    /// One enum payload step, available under the refinement fact that the
    /// enum currently holds this variant [REF-1, ENT-3.S15].
    Payload { variant: u32, field: u32 },
    /// One [OP-4] subscript. The offset is logical; the slot it selects is
    /// `(r.head + i) mod r.cap`, which is [WIN-1]'s business and no rule of
    /// the overlap relation mentions.
    Index(CapturedValue),
    /// One range step [REF-4] with both endpoints captured at formation.
    Range(CapturedRange),
    /// One of [WIN-2]'s four named window parts.
    Part(WindowPart),
    /// One measure read [OP-15, MSR-1]: descriptor storage, never a slot.
    Measure(CheckedMeasure),
}

/// What one step pair establishes about the two places below it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StepSeparation {
    /// The two steps provably select two different storages, so the complete
    /// places do not overlap however their later steps read.
    Separate,
    /// The two steps select one storage, so the walk continues and a later
    /// step may still separate them.
    Same,
    /// The two steps may select one storage and no later step can separate
    /// them: either the pair is unseparated under two different frames, or
    /// one step is a region containing the other.
    Overlapping,
}

/// The two [OWN-7] separations that are proofs rather than syntax, and
/// [WIN-2]'s one conditional answer.
///
/// [OWN-7] decides two index steps "by the fixed [ENT-6] families under
/// [MSR-4]'s disposition" and two range steps by four named non-strict
/// orderings, so the relation cannot be purely syntactic. This trait is the
/// one seam through which it reaches the entailment fragment; every
/// implementation answers from the fixed families and from nothing else, and
/// answering `false` is always sound because a pair no admitted family
/// discharges is overlapping.
///
/// An implementation's answers may not depend on the program point the
/// question is asked at, because [`PlaceMap`] memoizes them for the whole
/// function. Nothing in a resolved path can move under it: the captured index
/// and endpoint values are immutable mathematical values [OWN-7, REF-1].
pub(crate) trait SeparationOracle {
    /// Two index steps of one base whose offsets the fixed [ENT-6] families
    /// prove distinct.
    fn indices_distinct(&self, left: CapturedValue, right: CapturedValue) -> bool;

    /// Two range steps under one identical containing path proved disjoint by
    /// one of the four non-strict orderings `left.end <= right.start`,
    /// `right.end <= left.start`, `left.end <= left.start`, and
    /// `right.end <= right.start`; either empty-range ordering suffices
    /// because formation already proved start no greater than end [OWN-7].
    fn ranges_disjoint(&self, left: CapturedRange, right: CapturedRange) -> bool;

    /// [WIN-2]'s one conditional row: a live `r[i]` overlaps `r.last` unless
    /// `i != r.len - 1` is proved. `window` is the resolved place the two
    /// steps hang below, which is the `r` the relation `r.len` is read of.
    fn index_is_not_last(&self, window: &ResolvedPlace, index: CapturedValue) -> bool;
}

/// One resolved place [OWN-7]: its root and the ordered steps below it.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ResolvedPlace {
    pub(crate) root: PlaceRoot,
    pub(crate) path: Vec<PlaceStep>,
}

impl ResolvedPlace {
    /// The whole storage of one binding, with no selection below the root.
    pub(crate) const fn binding(binding: BindingId) -> Self {
        Self {
            root: PlaceRoot::Binding(binding),
            path: Vec::new(),
        }
    }

    /// One binding's storage with field selections below it, which is the
    /// shape a bare or projected place resolves to [REF-1].
    pub(crate) fn fields(binding: BindingId, fields: Vec<u32>) -> Self {
        Self {
            root: PlaceRoot::Binding(binding),
            path: fields.into_iter().map(PlaceStep::Field).collect(),
        }
    }

    /// One binding's storage with an already-resolved step path below it.
    pub(crate) const fn from_path(binding: BindingId, path: Vec<PlaceStep>) -> Self {
        Self {
            root: PlaceRoot::Binding(binding),
            path,
        }
    }

    /// One further [OP-4] subscript below this place, at the value its index
    /// expression captured [REF-1].
    pub(crate) fn push_subscript(&mut self, captured: CapturedValue) {
        self.path.push(PlaceStep::Index(captured));
    }

    /// The same place with field selections appended, for a caller that holds
    /// a plain field path.
    pub(crate) fn extend_fields(&mut self, fields: &[u32]) {
        self.path
            .extend(fields.iter().copied().map(PlaceStep::Field));
    }

    /// One place spelled as an optional leading `deref` [TYPE-7] followed by
    /// field selections, which is the shape most checked place nodes carry.
    pub(crate) fn spelled(root: PlaceRoot, deref: bool, fields: Vec<u32>) -> Self {
        let mut path = Vec::with_capacity(usize::from(deref) + fields.len());
        if deref {
            path.push(PlaceStep::Deref);
        }
        path.extend(fields.into_iter().map(PlaceStep::Field));
        Self { root, path }
    }

    /// The leading `deref`-then-fields reading of this path, where the path
    /// has exactly that shape, and `None` where any other step occurs in it.
    pub(crate) fn as_spelled(&self) -> Option<(bool, Vec<u32>)> {
        let mut steps = self.path.iter();
        let deref = matches!(steps.clone().next(), Some(PlaceStep::Deref));
        if deref {
            steps.next();
        }
        let fields = steps
            .map(|step| match step {
                PlaceStep::Field(field) => Some(*field),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        Some((deref, fields))
    }


    /// Whether this place's path positively selects the same storage as, or a
    /// storage containing, `other`'s.
    ///
    /// This is containment, not an absence of proved divergence: [REF-2] asks
    /// it of a written path against a live reference's path, and [EFF-5]
    /// clause 3 asks it of a call's substituted write paths, and both need a
    /// positive answer rather than "not proved apart".
    pub(crate) fn contains(&self, other: &Self) -> bool {
        self.root == other.root
            && self.path.len() <= other.path.len()
            && self
                .path
                .iter()
                .zip(&other.path)
                .all(|(left, right)| steps_provably_same(*left, *right))
    }

    /// [REF-2]'s own question: whether this place is a *proper* prefix of
    /// `other`, which is what invalidates a reference whose path is `other`.
    /// Writing the storage at `other` or below it is a content write and
    /// invalidates nothing.
    pub(crate) fn is_proper_prefix_of(&self, other: &Self) -> bool {
        self.path.len() < other.path.len() && self.contains(other)
    }

    /// The support every captured value in this path contributes [ENT-5].
    pub(crate) fn captured_support(&self) -> impl Iterator<Item = BindingId> + '_ {
        self.path
            .iter()
            .flat_map(|step| match step {
                PlaceStep::Index(index) => vec![index.support()],
                PlaceStep::Range(range) => vec![range.start.support(), range.end.support()],
                PlaceStep::Field(_)
                | PlaceStep::Deref
                | PlaceStep::Payload { .. }
                | PlaceStep::Part(_)
                | PlaceStep::Measure(_) => Vec::new(),
            })
            .flatten()
    }
}

/// Whether two steps positively select one storage on every execution.
fn steps_provably_same(left: PlaceStep, right: PlaceStep) -> bool {
    match (left, right) {
        (PlaceStep::Field(left), PlaceStep::Field(right)) => left == right,
        (PlaceStep::Deref, PlaceStep::Deref) => true,
        (
            PlaceStep::Payload {
                variant: left_variant,
                field: left_field,
            },
            PlaceStep::Payload {
                variant: right_variant,
                field: right_field,
            },
        ) => left_variant == right_variant && left_field == right_field,
        (PlaceStep::Index(left), PlaceStep::Index(right)) => left.provably_same(right),
        (PlaceStep::Range(left), PlaceStep::Range(right)) => {
            left.start.provably_same(right.start) && left.end.provably_same(right.end)
        }
        (PlaceStep::Part(left), PlaceStep::Part(right)) => left == right,
        (PlaceStep::Measure(left), PlaceStep::Measure(right)) => left == right,
        _ => false,
    }
}

/// [WIN-2]'s fixed answers for two window parts of one window.
///
/// `next` is the slot at index `len`, `last` the slot at index `len - 1`,
/// `filled` the slots below `len` and `free` the slots from `len` up, and
/// every cell below follows from those four definitions: `filled` and `free`
/// do not overlap, `next` is contained in `free`, `last` is contained in
/// `filled`, and the two singletons sit one index apart.
const fn parts_overlap(left: WindowPart, right: WindowPart) -> bool {
    use WindowPart::{Filled, Free, Last, Next};
    matches!(
        (left, right),
        (Next, Next)
            | (Last, Last)
            | (Filled, Filled)
            | (Free, Free)
            | (Next, Free)
            | (Free, Next)
            | (Last, Filled)
            | (Filled, Last)
    )
}

/// The [OWN-7] and [WIN-2] answer for one step pair.
fn separation(
    oracle: &dyn SeparationOracle,
    window: &ResolvedPlace,
    left: PlaceStep,
    right: PlaceStep,
) -> StepSeparation {
    match (left, right) {
        // Two field selections of different fields select two storages.
        (PlaceStep::Field(left), PlaceStep::Field(right)) => {
            if left == right {
                StepSeparation::Same
            } else {
                StepSeparation::Separate
            }
        }
        (PlaceStep::Deref, PlaceStep::Deref) => StepSeparation::Same,
        // Two payload steps of one variant selecting different fields of that
        // payload separate; two payload steps naming different variants of
        // one enum select the same storage and therefore overlap.
        (
            PlaceStep::Payload {
                variant: left_variant,
                field: left_field,
            },
            PlaceStep::Payload {
                variant: right_variant,
                field: right_field,
            },
        ) => {
            if left_variant != right_variant {
                StepSeparation::Overlapping
            } else if left_field == right_field {
                StepSeparation::Same
            } else {
                StepSeparation::Separate
            }
        }
        // Two index steps of one base. Unproved distinctness is not overlap:
        // both bases are the same storage, so a later step still separates
        // the two places under either answer, which is why the walk goes on.
        (PlaceStep::Index(left), PlaceStep::Index(right)) => {
            if left.literals_distinct(right) || oracle.indices_distinct(left, right) {
                StepSeparation::Separate
            } else {
                StepSeparation::Same
            }
        }
        // Two range steps. Every subsequent step is relative to its
        // containing range, so an unseparated pair of different frames stops
        // the walk: unequal subscripts below two different range steps never
        // establish separation by themselves.
        (PlaceStep::Range(left), PlaceStep::Range(right)) => {
            if left.start.provably_same(right.start) && left.end.provably_same(right.end) {
                StepSeparation::Same
            } else if oracle.ranges_disjoint(left, right) {
                StepSeparation::Separate
            } else {
                StepSeparation::Overlapping
            }
        }
        (PlaceStep::Part(left), PlaceStep::Part(right)) => {
            if left == right {
                StepSeparation::Same
            } else if parts_overlap(left, right) {
                StepSeparation::Overlapping
            } else {
                StepSeparation::Separate
            }
        }
        // A live `r[i]`, which has `i < r.len`, never overlaps `r.next` or
        // `r.free`, always overlaps `r.filled`, and overlaps `r.last` unless
        // `i != r.len - 1` is proved [WIN-2].
        (PlaceStep::Index(index), PlaceStep::Part(part))
        | (PlaceStep::Part(part), PlaceStep::Index(index)) => match part {
            WindowPart::Next | WindowPart::Free => StepSeparation::Separate,
            WindowPart::Filled => StepSeparation::Overlapping,
            WindowPart::Last => {
                if oracle.index_is_not_last(window, index) {
                    StepSeparation::Separate
                } else {
                    StepSeparation::Overlapping
                }
            }
        },
        // A measure is descriptor storage: `r.len` is itself a write target
        // and overlaps no slot [WIN-2, MSR-2]. Two different measures are not
        // separated by any admitted family, so they overlap.
        (PlaceStep::Measure(left), PlaceStep::Measure(right)) => {
            if left == right {
                StepSeparation::Same
            } else {
                StepSeparation::Overlapping
            }
        }
        (PlaceStep::Measure(_), PlaceStep::Index(_) | PlaceStep::Range(_) | PlaceStep::Part(_))
        | (
            PlaceStep::Index(_) | PlaceStep::Range(_) | PlaceStep::Part(_),
            PlaceStep::Measure(_),
        ) => StepSeparation::Separate,
        // Everything left is a pair no admitted family discharges, including
        // a range step against an index step or a window part on one base:
        // [OWN-7] separates two ranges and two indices and says nothing about
        // a range against either, so the pair is overlapping.
        _ => StepSeparation::Overlapping,
    }
}

/// [OWN-7] over two complete resolved paths that share a root.
fn paths_overlap(
    oracle: &dyn SeparationOracle,
    place: &ResolvedPlace,
    left: &[PlaceStep],
    right: &[PlaceStep],
) -> bool {
    for (depth, (left, right)) in left.iter().zip(right).enumerate() {
        // The window a [WIN-2] answer is read of is the place the two steps
        // hang below, which is the common prefix walked so far.
        let window = ResolvedPlace {
            root: place.root,
            path: place.path[..depth].to_vec(),
        };
        match separation(oracle, &window, *left, *right) {
            StepSeparation::Separate => return false,
            StepSeparation::Overlapping => return true,
            StepSeparation::Same => {}
        }
    }
    // One path is a prefix of the other, which is exactly [OWN-7]'s overlap.
    true
}

/// [OWN-7]: whether two resolved places overlap, without a memo.
///
/// Two places rooted at different local variables, parameters or consts are
/// two storages, which is the first separation [OWN-7] admits; below a common
/// root the walk asks each step pair in turn.
pub(crate) fn places_overlap(
    oracle: &dyn SeparationOracle,
    left: &ResolvedPlace,
    right: &ResolvedPlace,
) -> bool {
    left.root == right.root && paths_overlap(oracle, left, &left.path, &right.path)
}

/// The oracle of a consumer that holds no proof state [OWN-7].
///
/// Every answer is `false`, which is the relation's own default: a pair no
/// admitted family discharges overlaps. A consumer that needs the proved half
/// records the pair for the entailment fragment instead of guessing here.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UnprovedSeparations;

impl SeparationOracle for UnprovedSeparations {
    fn indices_distinct(&self, _left: CapturedValue, _right: CapturedValue) -> bool {
        false
    }

    fn ranges_disjoint(&self, _left: CapturedRange, _right: CapturedRange) -> bool {
        false
    }

    fn index_is_not_last(&self, _window: &ResolvedPlace, _index: CapturedValue) -> bool {
        false
    }
}

/// What one binding names when it is a reference variable [REF-1].
///
/// A reference is not storage of its own, so a binding that is one carries
/// the path set it names rather than a place of its own. The set has more
/// than one member only at a control-flow join, where [REF-1] takes the union
/// of the incoming edges' path sets and every check must hold for every
/// member.
#[derive(Clone, Debug, Default)]
pub(crate) struct BindingSummary {
    pub(crate) ty: Option<CheckedType>,
    /// The paths this binding names, empty when the binding is storage of
    /// its own.
    pub(crate) reference_paths: Vec<ResolvedPlace>,
}

/// Dense per-binding summaries for one checked function, the place resolution
/// they support, and this function's memo of the [OWN-7] answers.
#[derive(Debug, Default)]
pub(crate) struct PlaceMap {
    bindings: Vec<BindingSummary>,
    /// One answer per pair of resolved paths, for this function [OWN-7].
    ///
    /// The key is stable because a resolved path carries the immutable
    /// captured values of its index and endpoint terms, so the answer does
    /// not depend on the program point at which it is asked, and the memo is
    /// not a cross-flow fact cache: no kill and no join can change an entry.
    overlap_memo: RefCell<HashMap<(ResolvedPlace, ResolvedPlace), bool>>,
}

impl PlaceMap {
    /// Runs the binding prepass over one complete function body.
    pub(crate) fn for_function(function: &CheckedFunction) -> Self {
        let mut map = Self::default();
        for parameter in &function.parameters {
            let summary = map.summary_mut(parameter.binding);
            summary.ty = Some(parameter.ty);
            // An incoming reference parameter arrives carrying the caller's
            // path by substitution [EFF-5]; inside this body the parameter
            // name is the path, so it anchors at itself.
            if !matches!(parameter.mode, CheckedMode::Own) {
                summary.reference_paths = vec![ResolvedPlace::binding(parameter.binding)];
            }
        }
        map.collect_block_bindings(function.body.as_deref().unwrap_or_default());
        map
    }

    /// [OWN-7]: whether the two resolved places overlap.
    pub(crate) fn overlaps(
        &self,
        oracle: &dyn SeparationOracle,
        left: &ResolvedPlace,
        right: &ResolvedPlace,
    ) -> bool {
        // Two places rooted at different local variables or parameters are
        // two storages; that is the first of [OWN-7]'s separations.
        if left.root != right.root {
            return false;
        }
        let key = (left.clone(), right.clone());
        if let Some(answer) = self.overlap_memo.borrow().get(&key) {
            return *answer;
        }
        let answer = places_overlap(oracle, left, right);
        let mut memo = self.overlap_memo.borrow_mut();
        memo.insert((right.clone(), left.clone()), answer);
        memo.insert(key, answer);
        answer
    }

    pub(crate) fn summary_mut(&mut self, binding: BindingId) -> &mut BindingSummary {
        let index = binding.0 as usize;
        if self.bindings.len() <= index {
            self.bindings.resize(index + 1, BindingSummary::default());
        }
        &mut self.bindings[index]
    }

    pub(crate) fn summary(&self, binding: BindingId) -> Option<&BindingSummary> {
        self.bindings.get(binding.0 as usize)
    }

    /// Whether this binding is a reference variable rather than storage.
    pub(crate) fn is_reference(&self, binding: BindingId) -> bool {
        self.summary(binding)
            .is_some_and(|summary| !summary.reference_paths.is_empty())
    }

    /// Resolves a spelled place to the [OWN-7] resolved places it may name.
    ///
    /// Resolving a place rooted at a reference variable replaces that root
    /// with the path that reference names, recursively [REF-1], and at a join
    /// a reference variable names a set, so the result is a set: every check
    /// on the place must hold for every member.
    pub(crate) fn resolve(&self, root: PlaceRoot, steps: &[PlaceStep]) -> Vec<ResolvedPlace> {
        let PlaceRoot::Binding(binding) = root else {
            return vec![ResolvedPlace {
                root,
                path: steps.to_vec(),
            }];
        };
        let mut resolved = self.resolve_root(binding, 0);
        for place in &mut resolved {
            place.path.extend_from_slice(steps);
        }
        resolved
    }

    /// The path set one reference variable names, read through to storage.
    ///
    /// The recursion closes over the static path shapes [REF-1]: a
    /// loop-carried rebinding may change only the index values inside a path
    /// and may never extend the path through itself, so the shapes are
    /// finite. The depth guard is the [OWN-8] answer for a summary set this
    /// prepass left cyclic: the binding anchors at itself, which is the
    /// conservative place.
    fn resolve_root(&self, binding: BindingId, depth: usize) -> Vec<ResolvedPlace> {
        let anchored = vec![ResolvedPlace::binding(binding)];
        if depth > 32 {
            return anchored;
        }
        let Some(summary) = self.summary(binding) else {
            return anchored;
        };
        if summary.reference_paths.is_empty() {
            return anchored;
        }
        let mut resolved = Vec::new();
        for named in &summary.reference_paths {
            let PlaceRoot::Binding(root) = named.root else {
                resolved.push(named.clone());
                continue;
            };
            if root == binding {
                // A parameter, a match binder, or a formation this prepass
                // does not resolve anchors at its own binding.
                resolved.push(named.clone());
                continue;
            }
            for mut place in self.resolve_root(root, depth + 1) {
                place.path.extend_from_slice(&named.path);
                resolved.push(place);
            }
        }
        resolved
    }

    fn collect_block_bindings(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let reference_paths = self.reference_paths_of(value);
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.reference_paths = reference_paths;
                }
                // [CALL-4] every binder of a destructuring `let` is an
                // ordinary fresh binding of its result ordinal's type, and a
                // function returns owned values only [FN-1, REF-3].
                CheckedStatement::DestructuringLet { bindings, .. } => {
                    for (binding, ty, _) in bindings {
                        self.summary_mut(*binding).ty = Some(*ty);
                    }
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    self.summary_mut(*binding).ty = Some(*ok_type);
                }
                CheckedStatement::Replace {
                    binding, target, ..
                } => {
                    self.summary_mut(*binding).ty = Some(target.ty());
                }
                // [REF-1] a `let` binder every arm of which delivers a
                // reference is itself a reference variable naming the union
                // of the delivered path sets.
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                    let delivered = self.delivered_reference_paths(arms);
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(*result_type);
                    summary.reference_paths = delivered;
                }
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.collect_block_bindings(body);
                }
                CheckedStatement::CountedRange { binder, body, .. } => {
                    let summary = self.summary_mut(*binder);
                    summary.ty = Some(CheckedType::Integer(IntegerType::U64));
                    self.collect_block_bindings(body);
                }
                _ => {}
            }
        }
    }

    fn collect_arm_bindings(&mut self, arm: &CheckedMatchArm) {
        for binder in &arm.binders {
            let summary = self.summary_mut(binder.binding);
            summary.ty = Some(binder.ty);
            // [OWN-13] matching through a reference binds each payload as a
            // reference naming the scrutinee path extended by that payload
            // step. The scrutinee path is the checker's, not this prepass's,
            // so the binder anchors at itself and every consumer reads that
            // as the conservative place.
            if !matches!(binder.mode, CheckedMode::Own) {
                summary.reference_paths = vec![ResolvedPlace::binding(binder.binding)];
            }
        }
        self.collect_block_bindings(&arm.body);
    }

    /// The union of the path sets a value initializer's delivering arms name
    /// [REF-1, GIVE-1].
    fn delivered_reference_paths(&mut self, arms: &[CheckedMatchArm]) -> Vec<ResolvedPlace> {
        let mut union: Vec<ResolvedPlace> = Vec::new();
        for arm in arms {
            let Some(CheckedStatement::Give { value, .. }) = arm.body.last() else {
                return Vec::new();
            };
            let delivered = self.reference_paths_of(value);
            if delivered.is_empty() {
                // One arm delivering a value makes the binding storage of its
                // own; a reference binder needs every arm to deliver one.
                return Vec::new();
            }
            for place in delivered {
                if !union.contains(&place) {
                    union.push(place);
                }
            }
        }
        union
    }

    /// The path set a `let` right-hand side names, when it names one.
    ///
    /// A `borrow_expr` names the path its operand spells; a copy of a
    /// reference name copies a name for a path and therefore carries the same
    /// set. Everything else is storage of its own and names none.
    fn reference_paths_of(&self, value: &CheckedExpression) -> Vec<ResolvedPlace> {
        match value {
            CheckedExpression::BorrowAddressed { root, .. } => {
                self.resolve(root.root, &root.place_path())
            }
            CheckedExpression::BorrowBuffer { root, .. } => {
                self.resolve(PlaceRoot::Binding(root.binding), &root.place_path())
            }
            CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. } => {
                self.resolve(PlaceRoot::Binding(*binding), &[])
            }
            CheckedExpression::Binding { binding, .. } if self.is_reference(*binding) => {
                self.resolve(PlaceRoot::Binding(*binding), &[])
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CaptureId, CapturedRange, CapturedTerm, CapturedValue, PlaceMap,
        PlaceRoot, PlaceStep, ResolvedPlace, SeparationOracle, WindowPart,
    };
    use crate::semantic::model::{BindingId, CheckedMeasure};

    fn literal(capture: u32, value: u64) -> CapturedValue {
        CapturedValue::new(CaptureId(capture), CapturedTerm::Literal(value))
    }

    fn opaque(capture: u32) -> CapturedValue {
        CapturedValue::new(CaptureId(capture), CapturedTerm::Opaque)
    }

    fn place(binding: u32, path: &[PlaceStep]) -> ResolvedPlace {
        ResolvedPlace {
            root: PlaceRoot::Binding(BindingId(binding)),
            path: path.to_vec(),
        }
    }

    /// An oracle that discharges exactly the separations a test names, so a
    /// test can distinguish "the relation asked the fragment" from "the
    /// relation decided it by syntax".
    struct Proves {
        indices: bool,
        ranges: bool,
        not_last: bool,
    }

    impl SeparationOracle for Proves {
        fn indices_distinct(&self, _left: CapturedValue, _right: CapturedValue) -> bool {
            self.indices
        }

        fn ranges_disjoint(&self, _left: CapturedRange, _right: CapturedRange) -> bool {
            self.ranges
        }

        fn index_is_not_last(&self, _window: &ResolvedPlace, _index: CapturedValue) -> bool {
            self.not_last
        }
    }


    /// The oracle a consumer with no ProofContext in hand would supply
    /// [OWN-8]: every answer `false`, which denies each separation the fixed
    /// families might have discharged, so the relation degrades to
    /// "overlapping" and never to "disjoint".
    const DENIED: Proves = Proves {
        indices: false,
        ranges: false,
        not_last: false,
    };
    /// [OWN-7] two places fail to overlap when some step of their common
    /// prefix provably selects two different storages, and one place that is
    /// a prefix of the other overlaps it.
    #[test]
    fn different_roots_and_different_fields_separate() {
        let map = PlaceMap::default();
        let denied = DENIED;
        assert!(!map.overlaps(&denied, &place(0, &[]), &place(1, &[])));
        assert!(!map.overlaps(
            &denied,
            &place(0, &[PlaceStep::Field(0)]),
            &place(0, &[PlaceStep::Field(1)]),
        ));
        assert!(map.overlaps(&denied, &place(0, &[]), &place(0, &[PlaceStep::Field(1)])));
        assert!(map.overlaps(
            &denied,
            &place(0, &[PlaceStep::Field(1)]),
            &place(0, &[PlaceStep::Field(1), PlaceStep::Deref]),
        ));
    }

    /// [OWN-7] two payload steps of one variant selecting different fields of
    /// that payload separate; two payload steps naming different variants of
    /// one enum select the same storage and therefore overlap.
    #[test]
    fn payload_steps_separate_only_within_one_variant() {
        let map = PlaceMap::default();
        let denied = DENIED;
        let one = place(
            0,
            &[PlaceStep::Payload {
                variant: 0,
                field: 0,
            }],
        );
        let two = place(
            0,
            &[PlaceStep::Payload {
                variant: 0,
                field: 1,
            }],
        );
        let other = place(
            0,
            &[PlaceStep::Payload {
                variant: 1,
                field: 0,
            }],
        );
        assert!(!map.overlaps(&denied, &one, &two));
        assert!(map.overlaps(&denied, &one, &other));
    }

    /// [OWN-7] two index steps separate when two written literals differ, and
    /// otherwise only when the fixed [ENT-6] families discharge it.
    #[test]
    fn index_steps_reach_the_entailment_families() {
        let denied = DENIED;
        let proving = Proves {
            indices: true,
            ranges: false,
            not_last: false,
        };
        let left = place(0, &[PlaceStep::Index(literal(0, 3))]);
        let right = place(0, &[PlaceStep::Index(literal(1, 4))]);
        assert!(!PlaceMap::default().overlaps(&denied, &left, &right));

        let left = place(0, &[PlaceStep::Index(opaque(0))]);
        let right = place(0, &[PlaceStep::Index(opaque(1))]);
        assert!(PlaceMap::default().overlaps(&denied, &left, &right));
        assert!(!PlaceMap::default().overlaps(&proving, &left, &right));
    }

    /// [OWN-7] an unproved index pair does not stop the walk: both steps
    /// select one base, so a later field selection still separates the two
    /// complete places whichever way the index question would have gone.
    #[test]
    fn a_field_below_an_unproved_index_still_separates() {
        let map = PlaceMap::default();
        let left = place(0, &[PlaceStep::Index(opaque(0)), PlaceStep::Field(0)]);
        let right = place(0, &[PlaceStep::Index(opaque(1)), PlaceStep::Field(1)]);
        assert!(!map.overlaps(&DENIED, &left, &right));
    }

    /// [OWN-7] every subsequent step is relative to its containing range, so
    /// unequal subscripts below two different range steps never establish
    /// separation by themselves, while a proved separation of the containing
    /// ranges separates all their descendants.
    #[test]
    fn steps_below_two_unseparated_ranges_never_separate() {
        let frame = |capture: u32| {
            PlaceStep::Range(CapturedRange {
                start: opaque(capture),
                end: opaque(capture + 100),
            })
        };
        let left = place(0, &[frame(0), PlaceStep::Index(literal(1, 0))]);
        let right = place(0, &[frame(1), PlaceStep::Index(literal(2, 7))]);
        assert!(PlaceMap::default().overlaps(&DENIED, &left, &right));

        let proving = Proves {
            indices: false,
            ranges: true,
            not_last: false,
        };
        assert!(!PlaceMap::default().overlaps(&proving, &left, &right));
    }

    /// [WIN-2] a live `r[i]` never overlaps `r.next` or `r.free`, always
    /// overlaps `r.filled`, and overlaps `r.last` unless `i != r.len - 1` is
    /// proved.
    #[test]
    fn window_part_answers_are_the_fixed_table() {
        let map = PlaceMap::default();
        let denied = DENIED;
        let index = place(0, &[PlaceStep::Index(opaque(0))]);
        let part = |part| place(0, &[PlaceStep::Part(part)]);

        assert!(!map.overlaps(&denied, &index, &part(WindowPart::Next)));
        assert!(!map.overlaps(&denied, &index, &part(WindowPart::Free)));
        assert!(map.overlaps(&denied, &index, &part(WindowPart::Filled)));
        assert!(map.overlaps(&denied, &index, &part(WindowPart::Last)));
        assert!(!PlaceMap::default().overlaps(
            &Proves {
                indices: false,
                ranges: false,
                not_last: true,
            },
            &index,
            &part(WindowPart::Last),
        ));

        assert!(!map.overlaps(&denied, &part(WindowPart::Filled), &part(WindowPart::Free)));
        assert!(!map.overlaps(&denied, &part(WindowPart::Next), &part(WindowPart::Last)));
        assert!(map.overlaps(&denied, &part(WindowPart::Next), &part(WindowPart::Free)));
        assert!(map.overlaps(&denied, &part(WindowPart::Last), &part(WindowPart::Filled)));
    }

    /// [WIN-2] the measure `r.len` is itself a write target and overlaps no
    /// slot.
    #[test]
    fn a_measure_overlaps_no_slot() {
        let map = PlaceMap::default();
        let denied = DENIED;
        let length = place(0, &[PlaceStep::Measure(CheckedMeasure::Length)]);
        assert!(!map.overlaps(&denied, &length, &place(0, &[PlaceStep::Index(opaque(0))])));
        assert!(!map.overlaps(
            &denied,
            &length,
            &place(0, &[PlaceStep::Part(WindowPart::Filled)]),
        ));
        assert!(map.overlaps(&denied, &length, &length));
    }

    /// [REF-2] writing the storage at a reference's path or below it is a
    /// content write; only a proper prefix invalidates it.
    #[test]
    fn only_a_proper_prefix_invalidates() {
        let reference = place(0, &[PlaceStep::Field(0), PlaceStep::Deref]);
        assert!(place(0, &[PlaceStep::Field(0)]).is_proper_prefix_of(&reference));
        assert!(!reference.is_proper_prefix_of(&reference));
        assert!(
            !place(
                0,
                &[PlaceStep::Field(0), PlaceStep::Deref, PlaceStep::Field(2)]
            )
            .is_proper_prefix_of(&reference)
        );
        assert!(!place(0, &[PlaceStep::Field(1)]).is_proper_prefix_of(&reference));
    }
}
