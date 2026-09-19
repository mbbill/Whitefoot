//! [REF-1, REF-2, REF-3, REF-4] references: the checker side.
//!
//! A reference is a local name for a path [REF-1]. It is not storage of its
//! own, it carries no permission marker, no region and no loan, and it never
//! escapes the function that formed it [REF-3]. What the checker therefore
//! has to carry for a reference binding is exactly three things: the set of
//! paths it names, whether it is a `&T` or a `&[T]` [REF-4], and whether it
//! is still valid [REF-2].
//!
//! The validity fact is threaded through the statement walk rather than
//! recomputed on demand, because [REF-2]'s closing sentence puts it outside
//! the entailment fragment [ENT-1] and its invalidation set is exactly
//! enumerated: a proper prefix of the path is written, moved out of, or
//! released, by a statement, by a call [EFF-5], or by a compiler-derived
//! release at scope exit [STOR-3]; the scope of the local variable the path
//! starts at ends; or a refinement fact a payload step depends on is
//! invalidated. Writing the storage at the path or below it is a content
//! write and invalidates nothing.
//!
//! Every path question — prefix, overlap, index and range separation — is
//! [OWN-7]'s, asked through [`super::super::places`] and answered nowhere
//! else.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, FixedTerminal, LexicalUseRole, Production,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::model::{
    BindingId, CheckedContainerRoot, CheckedEffectStep, CheckedExpression, CheckedMeasure,
    CheckedMode, CheckedNominalKind, CheckedPlaceStep, CheckedRangeRoot, CheckedRangeSource,
    CheckedStatePath, CheckedType, WindowShape,
};
use super::super::places::{CapturedRange, CapturedValue, PlaceRoot, PlaceStep, ResolvedPlace};
use super::{
    CheckStop, Checker, EffectPath, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};

// [DIAG-1] same-node judgment order at a reference use: OWN-1's liveness and
// spelling judgments are asked before REF-2's validity judgment, and REF-1
// fixes the path a reference names before REF-2 judges that path's validity,
// because each earlier-defined rule cites first at one offending use.
//
// This replaces the v0.59 ordering over OWN-5, OWN-6, OWN-10 and OWN-14,
// whose subject was the loan-and-reborrow apparatus; none of those four rules
// exists in the active specification, so the pinned order had nothing left to
// pin. The successor ordering is the one a reference use asks in.
const _: () = {
    assert!(SemanticRule::Own1.definition_rank() < SemanticRule::Ref1.definition_rank());
    assert!(SemanticRule::Ref1.definition_rank() < SemanticRule::Ref2.definition_rank());
    assert!(SemanticRule::Ref2.definition_rank() < SemanticRule::Ref3.definition_rank());
    assert!(SemanticRule::Ref3.definition_rank() < SemanticRule::Ref4.definition_rank());
};

/// [REF-1]'s restructuring for a `&` over a reference variable.
pub(super) const REF1_NAME_THE_PATH: &str = "name the path the reference names";

/// [REF-1]'s restructuring for a loop-carried rebinding that extends a path
/// through itself.
pub(super) const REF1_STATIC_SHAPE: &str =
    "walk owned links by recursion, or keep the nodes in one storage and iterate an index";

/// [REF-2]'s restructuring for a use of an invalidated reference.
pub(super) const REF2_FORM_AGAIN: &str = "form the reference again after that event";

/// [REF-3]'s restructuring for an escaping reference.
pub(super) const REF3_RETURN_AN_INDEX: &str =
    "return an index and let the caller form the reference";

/// [REF-4]'s restructuring for a range reference over a `Ring`.
pub(super) const REF4_RING: &str =
    "a ring hands out single slots; take the elements one at a time";

/// [WIN-3]'s restructuring for a move out of a window slot or array element.
pub(super) const WIN3_NO_TAKE: &str = "use take_back, remove_at, or swap [OP-10, OP-11]";

/// [OWN-1]'s restructuring for a `move` of a place reached through a `deref`.
pub(super) const OWN1_ROOTED_CONSUME: &str =
    "consume a place rooted in a live own-mode binding of this function";

/// [WIN-3]'s restructuring for a remaining linear part of a consumed owner.
pub(super) const WIN3_DESTRUCTURE: &str =
    "take it in the same destructuring: let N(f: a, ..) = move v;";

/// What one use does to the storage at a place.
///
/// There are four, and no fifth: v0.59's shared/unique borrow split had the
/// loan apparatus as its subject and [REF-1] states that there is no
/// permission marker on a reference, so forming one is one kind of access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AccessKind {
    /// The place's value is observed [EFF-1].
    Read,
    /// The storage at the place, and everything below it, is written
    /// [EFF-1, SET-1].
    Write,
    /// The place is consumed [OWN-1], which kills the whole binding rooting
    /// it.
    Move,
    /// A reference is formed to the place [REF-1].
    Reference,
}

impl AccessKind {
    /// Whether this access invalidates every live reference whose path it is
    /// a proper prefix of [REF-2].
    pub(super) const fn invalidates_references(self) -> bool {
        match self {
            Self::Write | Self::Move => true,
            Self::Read | Self::Reference => false,
        }
    }
}

/// Which reference kind a binding names [GRAM-3, REF-4].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReferenceKind {
    /// `&T`: one place.
    Single,
    /// `&[T]`: a range of elements, whose one measure is `len` [REF-4].
    Range,
}

/// The exactly enumerated events [REF-2] admits as invalidating, carried into
/// the diagnostic so the rejection names the event rather than only the use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum InvalidationEvent {
    /// A proper prefix of the path was written by a statement [SET-1].
    PrefixWritten,
    /// A proper prefix of the path was moved out of [OWN-1, WIN-3].
    PrefixMoved,
    /// A proper prefix of the path was released [STOR-3].
    PrefixReleased,
    /// A call's substituted row writes a proper prefix of the path
    /// [EFF-5 clause 3].
    CallWrite,
    /// The scope of the local variable the path starts at ended.
    RootScopeEnded,
    /// A refinement fact a payload step in the path depends on was
    /// invalidated [REF-1, ENT-3.S15].
    RefinementLost,
    /// A window operation moved the boundary or the logical origin the
    /// reference's bound was formed under [OP-10, REF-4].
    ///
    /// This is not [EFF-5]'s prefix rule: `take_back` writes `r.len`, which
    /// is no prefix of `r[i]`, and yet [OP-10] states that such a reference
    /// dies there, because the bound `i < r.len` its formation established no
    /// longer holds. `place_back`'s `ensures` carries that bound across the
    /// call and `insert_at`'s occupant change leaves the slot itself, so
    /// neither is one of these events.
    WindowBoundaryMoved,
}

impl InvalidationEvent {
    /// The exact event phrase the [REF-2] diagnostic carries.
    pub(super) const fn phrase(&self) -> &'static str {
        match self {
            Self::PrefixWritten => "a proper prefix of the reference's path was written",
            Self::PrefixMoved => "a proper prefix of the reference's path was moved out of",
            Self::PrefixReleased => "a proper prefix of the reference's path was released",
            Self::CallWrite => "a call wrote a proper prefix of the reference's path",
            Self::RootScopeEnded => "the scope of the local variable the path starts at ended",
            Self::RefinementLost => {
                "the refinement fact the reference's payload step depends on was invalidated"
            }
            Self::WindowBoundaryMoved => {
                "a window operation moved the boundary or origin the reference was formed under"
            }
        }
    }
}

/// [REF-2] "p is valid" is a fact like any other.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ReferenceValidity {
    Valid,
    /// Re-established only by forming the reference again.
    Invalid(InvalidationEvent),
}

/// What one reference binding names [REF-1].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReferenceInfo {
    pub(super) kind: ReferenceKind,
    /// The path set. It has more than one member only after a control-flow
    /// join, where [REF-1] takes the union of the incoming edges' sets and
    /// every check must hold for every member.
    pub(super) paths: Vec<ResolvedPlace>,
    pub(super) validity: ReferenceValidity,
    /// The enum places whose refinement facts the payload steps of `paths`
    /// depend on, with the variant each step selects [REF-1, ENT-3.S15].
    pub(super) refinements: Vec<(ResolvedPlace, u32)>,
}

impl ReferenceInfo {
    /// One freshly formed reference naming one path.
    pub(super) fn formed(kind: ReferenceKind, path: ResolvedPlace) -> Self {
        let refinements = refinement_dependencies(&path);
        Self {
            kind,
            paths: vec![path],
            validity: ReferenceValidity::Valid,
            refinements,
        }
    }

    pub(super) const fn is_valid(&self) -> bool {
        matches!(self.validity, ReferenceValidity::Valid)
    }

    /// [REF-1] one further step below every path this reference names, which
    /// is how a payload binder names the scrutinee path extended by its own
    /// payload step [OWN-13].
    ///
    /// The refinement dependencies are recomputed over the extended paths, so
    /// a payload step carries the fact its availability rests on [ENT-3.S15].
    pub(super) fn extend(&mut self, step: PlaceStep) {
        for path in &mut self.paths {
            path.path.push(step);
        }
        self.refinements = self
            .paths
            .iter()
            .flat_map(refinement_dependencies)
            .collect();
    }

    /// [REF-2] invalidation is a fact about the whole reference: a path set
    /// with one invalidated member is invalid, because every check on the
    /// binding must hold for every member of the set [REF-1].
    pub(super) fn invalidate(&mut self, event: InvalidationEvent) {
        if self.is_valid() {
            self.validity = ReferenceValidity::Invalid(event);
        }
    }

    /// [REF-1] the join of two incoming edges: the union of the path sets,
    /// and the meet of the validity facts, because a check must hold on every
    /// incoming edge.
    pub(super) fn join(&mut self, other: &Self) {
        for path in &other.paths {
            if !self.paths.contains(path) {
                self.paths.push(path.clone());
            }
        }
        for dependency in &other.refinements {
            if !self.refinements.contains(dependency) {
                self.refinements.push(dependency.clone());
            }
        }
        if let ReferenceValidity::Invalid(event) = &other.validity {
            self.invalidate(event.clone());
        }
    }

    /// [REF-1] the static path shape: a loop-carried rebinding may change
    /// only the index values inside the path and may never extend the path
    /// through itself.
    ///
    /// Two shapes agree when their roots agree and their step kinds agree
    /// step for step; the captured index and endpoint values are what a
    /// rebinding is allowed to move.
    pub(super) fn shape_agrees_with(&self, other: &Self) -> bool {
        self.paths.iter().all(|left| {
            other
                .paths
                .iter()
                .any(|right| path_shapes_agree(left, right))
        })
    }
}

/// Whether two resolved paths have one static shape [REF-1].
fn path_shapes_agree(left: &ResolvedPlace, right: &ResolvedPlace) -> bool {
    left.root == right.root
        && left.path.len() == right.path.len()
        && left
            .path
            .iter()
            .zip(&right.path)
            .all(|(left, right)| step_shapes_agree(*left, *right))
}

const fn step_shapes_agree(left: PlaceStep, right: PlaceStep) -> bool {
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
        // An index or a range step is exactly what a loop-carried rebinding
        // may move, so two of them agree in shape whatever they captured.
        (PlaceStep::Index(_), PlaceStep::Index(_)) | (PlaceStep::Range(_), PlaceStep::Range(_)) => {
            true
        }
        (PlaceStep::Part(left), PlaceStep::Part(right)) => matches!(
            (left, right),
            (super::super::places::WindowPart::Next, super::super::places::WindowPart::Next)
                | (
                    super::super::places::WindowPart::Last,
                    super::super::places::WindowPart::Last
                )
                | (
                    super::super::places::WindowPart::Filled,
                    super::super::places::WindowPart::Filled
                )
                | (
                    super::super::places::WindowPart::Free,
                    super::super::places::WindowPart::Free
                )
        ),
        (PlaceStep::Measure(left), PlaceStep::Measure(right)) => left as u8 == right as u8,
        _ => false,
    }
}

/// The enum places whose refinement facts one path's payload steps depend on
/// [REF-1].
///
/// A payload step is available only under the fact that the enum currently
/// holds that variant, so the base the step hangs below is the place the fact
/// is about.
fn refinement_dependencies(path: &ResolvedPlace) -> Vec<(ResolvedPlace, u32)> {
    let mut dependencies = Vec::new();
    for (position, step) in path.path.iter().enumerate() {
        if let PlaceStep::Payload { variant, .. } = step {
            dependencies.push((
                ResolvedPlace {
                    root: path.root,
                    path: path.path[..position].to_vec(),
                },
                *variant,
            ));
        }
    }
    dependencies
}

/// The value a position requires of its operand [TYPE-5, TYPE-7].
///
/// A reference is read bare [TYPE-7], so a position that needs the referent
/// states what it needs and takes the path the reference names; there is no
/// read-through operation to insert.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RequiredReferent {
    /// A position requires one exact value type, as counted endpoints require
    /// `own u64` [TYPE-5].
    Exact(CheckedType),
    /// A `match` scrutinee requires an enum value [OWN-13, ERR-2].
    Enum,
    /// An index root requires directly indexable storage [OP-4].
    IndexableStorage,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [GRAM-3] the written mode of a `param`: `own`, `&`, or the `&[T]`
    /// range-reference kind, which is written without a `mode` node.
    pub(super) fn parse_mode(&self, node: NodeId) -> Result<CheckedMode, CheckStop> {
        let Some(mode) = self.tree.first_child_with(node, Production::Mode)? else {
            // `param := IDENT ":" (mode type | "&" "[" type "]")`: the second
            // alternative writes no `mode` node at all [GRAM-2].
            return Ok(CheckedMode::Range);
        };
        if self.has_fixed(mode, crate::FixedTerminal::Own)? {
            return Ok(CheckedMode::Own);
        }
        if self.has_fixed(mode, crate::FixedTerminal::Ampersand)? {
            return Ok(CheckedMode::Reference);
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// [REF-1] the path set a spelled place root names, read through every
    /// reference variable in the chain.
    ///
    /// Resolving a place rooted at a reference variable replaces that root
    /// with the path that reference names, recursively, and every [OWN-7]
    /// judgment reads the result.
    pub(super) fn resolve_reference_root(
        &self,
        root: DeclarationId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Vec<ResolvedPlace>, CheckStop> {
        let Some(local) = bindings.get(&root) else {
            // A named const [CONST-2] is storage of its own.
            let constant = *self
                .constants
                .get(&root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            return Ok(vec![ResolvedPlace {
                root: PlaceRoot::Constant(constant),
                path: Vec::new(),
            }]);
        };
        let Some(reference) = &local.reference else {
            return Ok(vec![ResolvedPlace::binding(local.binding)]);
        };
        // A parameter's reference arrives carrying the caller's path by
        // substitution [EFF-5]; inside this body the parameter name *is* the
        // path, so its set anchors at itself and the recursion terminates
        // there. Every other reference's set was resolved when it was formed.
        Ok(reference.paths.clone())
    }

    /// [REF-1] the complete resolved place set of one spelled place: its
    /// resolved root, with the written steps appended.
    pub(super) fn resolve_place_set(
        &self,
        root: DeclarationId,
        steps: &[PlaceStep],
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Vec<ResolvedPlace>, CheckStop> {
        let mut resolved = self.resolve_reference_root(root, bindings)?;
        for place in &mut resolved {
            place.path.extend_from_slice(steps);
        }
        Ok(resolved)
    }

    /// [REF-2] the validity judgment at one use of a reference binding.
    pub(super) fn check_reference_valid(
        &self,
        local: &LocalBinding,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let Some(reference) = &local.reference else {
            return Ok(());
        };
        let ReferenceValidity::Invalid(event) = &reference.validity else {
            return Ok(());
        };
        self.issue_node(
            SemanticRule::Ref2,
            node,
            SemanticIssueKind::InvalidReferenceUse {
                binder: self.declaration_spelling(local.declaration)?,
                event: event.phrase(),
                mechanical_fix: REF2_FORM_AGAIN,
            },
        )
    }

    /// [REF-2] applies one access's invalidation to every live reference in
    /// scope.
    ///
    /// The prefix question is [OWN-7]'s, asked conservatively: two indexed
    /// positions on one storage are taken to overlap unless proved distinct,
    /// which is exactly what [`ResolvedPlace::is_proper_prefix_of`] answers
    /// without a proof and what the separation oracle answers with one.
    /// Writing the storage at a reference's own path, or below it, is a
    /// content write and invalidates nothing.
    pub(super) fn invalidate_references(
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        written: &ResolvedPlace,
        event: &InvalidationEvent,
    ) {
        for local in bindings.values_mut() {
            let Some(reference) = &mut local.reference else {
                continue;
            };
            if reference
                .paths
                .iter()
                .any(|path| written.is_proper_prefix_of(path))
            {
                reference.invalidate(event.clone());
            }
        }
    }

    /// [OP-10, REF-2] every reference into one window becomes invalid.
    ///
    /// A reference into a window is formed under a bound and stays valid
    /// while that bound holds: `&r[i]` under `i < r.len` and `&r[lo..hi]`
    /// under `hi <= r.len` [REF-4]. The operations that move the boundary
    /// down, move a run between two windows, shift every logical index, or
    /// remake the block whole end that bound, and the references formed under
    /// it die with it.
    pub(super) fn invalidate_window_references(
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        window: &ResolvedPlace,
    ) {
        for local in bindings.values_mut() {
            let Some(reference) = &mut local.reference else {
                continue;
            };
            if reference.paths.iter().any(|path| window.contains(path)) {
                reference.invalidate(InvalidationEvent::WindowBoundaryMoved);
            }
        }
    }

    /// [REF-2] the scope-exit half: every reference whose path starts at a
    /// local variable whose scope ends here becomes invalid, and so does
    /// every reference through a place that scope's compiler-derived release
    /// reaches [STOR-3].
    pub(super) fn invalidate_references_leaving_scope(
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        leaving: &[BindingId],
    ) {
        for local in bindings.values_mut() {
            let Some(reference) = &mut local.reference else {
                continue;
            };
            if reference.paths.iter().any(|path| match path.root {
                PlaceRoot::Binding(binding) => leaving.contains(&binding),
                PlaceRoot::Constant(_) => false,
            }) {
                reference.invalidate(InvalidationEvent::RootScopeEnded);
            }
        }
    }

    /// [REF-1, ENT-3.S15] the arm-exit half: a payload reference is valid
    /// exactly while the arm's refinement fact holds [OWN-13].
    pub(super) fn invalidate_refinement_dependents(
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scrutinee: &ResolvedPlace,
    ) {
        for local in bindings.values_mut() {
            let Some(reference) = &mut local.reference else {
                continue;
            };
            if reference
                .refinements
                .iter()
                .any(|(place, _)| place == scrutinee)
            {
                reference.invalidate(InvalidationEvent::RefinementLost);
            }
        }
    }

    /// [REF-1] the join of a reference binding over two incoming edges.
    pub(super) fn join_reference(
        target: &mut LocalBinding,
        incoming: &LocalBinding,
    ) -> Result<(), CheckStop> {
        match (&mut target.reference, &incoming.reference) {
            (Some(left), Some(right)) => {
                left.join(right);
                Ok(())
            }
            (None, None) => Ok(()),
            // A binding that is a reference on one edge and storage on the
            // other cannot exist: a `let` binder's kind is derived once from
            // its initializer [TYPE-5, REF-1] and a `set` of a reference
            // variable rebinds the same name.
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    /// [REF-3] a reference never escapes: it may not be assigned into any
    /// aggregate, returned, or captured by a stored function value.
    pub(super) fn reject_escaping_reference(
        &self,
        value: &TypedExpression,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if value.mode == CheckedMode::Own {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Ref3,
            node,
            SemanticIssueKind::EscapingReference {
                mechanical_fix: REF3_RETURN_AN_INDEX,
            },
        )
    }

    /// [REF-1] `&p` where `p` is a reference variable, and [REF-4] the whole
    /// `borrow_expr` judgment.
    ///
    /// `borrow_expr := "&" place` [GRAM-5]: there is no permission marker, no
    /// region, and no other qualifier, so the judgment is exactly the path
    /// resolution plus [REF-4]'s range-formation obligations.
    pub(super) fn check_borrow(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let carrier = self
            .tree
            .parent(node)?
            .filter(|parent| self.tree.production(*parent) == Ok(Production::Atom))
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let place_node = self
            .tree
            .first_child_with(node, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase = self
            .tree
            .first_child_with(place_node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // `deref` is an ordinary path step [TYPE-7], so a `borrow_expr` over
        // one is an ordinary formation; the root of the complete path is what
        // decides the judgment.
        //
        // [REF-1] a place that goes through a reference variable is written
        // under that step — `&deref(p)`, `&deref(p)[i]` — and resolving the
        // step replaces it with the path the reference names. The root of the
        // complete path is therefore that reference variable itself, which is
        // what the reference-root replacement below already reads, so the two
        // spellings reach one judgment and one representation.
        let written_deref = self.has_fixed(pbase, FixedTerminal::Deref)?;
        let pbase = if written_deref {
            let inner = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if !self
                .tree
                .children_with(inner, Production::Psuffix)?
                .is_empty()
            {
                // A reference is never stored in an aggregate [REF-3], so the
                // only place a `deref` step names is a bare reference
                // variable; anything else is a form this walk cannot root.
                return self
                    .unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
            }
            self.tree
                .first_child_with(inner, Production::Pbase)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
        } else {
            pbase
        };
        let root_use = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let (root, root_type, root_binding) = match root_use.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::NamedConst,
            } => {
                let constant = *self
                    .constants
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (
                    PlaceRoot::Constant(constant),
                    self.constant(constant)?.ty,
                    None,
                )
            }
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } => {
                let local = bindings
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                // [REF-1] a reference variable is not storage of its own, so
                // `&r` where `r` is one has no path to name that `r` does not
                // already name.
                if local.reference.is_some()
                    && !written_deref
                    && self.tree.children(pbase)?.is_empty()
                {
                    let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
                    if suffixes.is_empty() {
                        return self.issue_node(
                            SemanticRule::Ref1,
                            node,
                            SemanticIssueKind::ReferenceToReferenceVariable {
                                mechanical_fix: REF1_NAME_THE_PATH,
                            },
                        );
                    }
                }
                if !local.live {
                    return self.issue_node(
                        SemanticRule::Own1,
                        place_node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                self.check_reference_valid(local, place_node)?;
                (
                    PlaceRoot::Binding(local.binding),
                    local.ty,
                    Some(local.clone()),
                )
            }
            _ => {
                return self.unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
            }
        };
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        // [REF-4] `&x[lo..hi]` forms a range reference. The range step is the
        // whole of the formation and selects no element place, so it is
        // resolved here rather than by the ordinary storage walk, and it is
        // the last written `psuffix`: a range reference is not storage
        // [TYPE-8], so nothing below it is written.
        if let Some(position) = self.range_suffix_position(&suffixes)? {
            if position + 1 != suffixes.len() {
                return self
                    .unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
            }
            return self.check_range_formation(
                carrier,
                place_node,
                suffixes[position],
                &suffixes[..position],
                root,
                root_type,
                root_binding.as_ref(),
                written_deref,
                function,
                bindings,
                loop_depth,
            );
        }
        let (path, ty, carried) =
            self.resolve_storage_path(&suffixes, root_type, bindings, function, loop_depth, true)?;
        let mut place = ResolvedPlace {
            root,
            path: path.iter().map(CheckedPlaceStep::place_step).collect(),
        };
        // [REF-1] the written steps continue the path the root names, so a
        // reference root is replaced by what it names before anything reads
        // the place.
        if let Some(local) = &root_binding
            && let Some(reference) = &local.reference
            && let Some(named) = reference.paths.first()
        {
            let mut resolved = named.clone();
            resolved.path.extend(place.path.iter().copied());
            place = resolved;
        }
        let kind = ReferenceKind::Single;
        let expression = CheckedExpression::BorrowAddressed {
            carrier: self.tree.path(carrier)?.clone(),
            root: CheckedContainerRoot {
                root,
                path,
                ty,
            },
        };
        let mut accesses = carried.accesses;
        accesses.push(PlaceAccess {
            place: place.clone(),
            kind: AccessKind::Reference,
        });
        Ok(TypedExpression {
            expression,
            mode: match kind {
                ReferenceKind::Single => CheckedMode::Reference,
                ReferenceKind::Range => CheckedMode::Range,
            },
            reference: Some(ReferenceInfo::formed(kind, place)),
            reference_value: true,
            effects: carried.effects,
            accesses,
        })
    }

    /// The position of the one `psuffix` written as a range step, if any
    /// [GRAM-5, REF-4].
    fn range_suffix_position(&self, suffixes: &[NodeId]) -> Result<Option<usize>, CheckStop> {
        for (position, &suffix) in suffixes.iter().enumerate() {
            if self
                .tree
                .first_child_with(suffix, Production::RangeTail)?
                .is_some()
            {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }

    /// [REF-4] `&x[lo..hi]`, over an indexable place or over another range
    /// reference, under `lo <= hi` and `hi <= x.len`.
    #[allow(clippy::too_many_arguments)]
    fn check_range_formation(
        &self,
        carrier: NodeId,
        place_node: NodeId,
        suffix: NodeId,
        base_suffixes: &[NodeId],
        root: PlaceRoot,
        root_type: CheckedType,
        root_binding: Option<&LocalBinding>,
        written_deref: bool,
        function: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let start_node = self
            .subscript_offset(suffix)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let tail = self
            .tree
            .first_child_with(suffix, Production::RangeTail)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let end_node = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // [REF-4] re-slicing: the base is the run another range names, whose
        // element type the `deref` already selected [TYPE-7].
        let range_base = written_deref
            && root_binding.is_some_and(|local| local.mode == CheckedMode::Range);
        let mut carried = super::expressions::flat_storage::CarriedOperands::default();
        let (source, base_place, element_type) = if range_base {
            if !base_suffixes.is_empty() {
                return self
                    .unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
            }
            let local = root_binding.ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let Some(element) = self.flat_element(local.ty)? else {
                return self
                    .unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
            };
            let named = local
                .reference
                .as_ref()
                .and_then(|reference| reference.paths.first().cloned())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            (
                CheckedRangeSource::Range(CheckedRangeRoot {
                    binding: local.binding,
                    element,
                }),
                named,
                local.ty,
            )
        } else {
            let (path, ty, offsets) = self.resolve_storage_path(
                base_suffixes,
                root_type,
                bindings,
                function,
                loop_depth,
                true,
            )?;
            carried = offsets;
            // [REF-4] a range over a `Ring` is refused: a wrapped window is
            // two extents and `&[T]` has one `len`.
            if matches!(
                ty,
                CheckedType::Window {
                    shape: WindowShape::Ring,
                    ..
                }
            ) {
                return self.issue_node(
                    SemanticRule::Ref4,
                    suffix,
                    SemanticIssueKind::RangeOverRing {
                        mechanical_fix: REF4_RING,
                    },
                );
            }
            let element = match ty {
                CheckedType::Array { element, .. } | CheckedType::Window { element, .. } => {
                    self.element_type(element)?
                }
                // [OP-4] a runtime-capacity `Array<T>` is an indexable base
                // exactly as the constant-capacity one is [TYPE-9].
                CheckedType::Buffer { element } => element.ty(),
                _ => {
                    return self.issue_node(
                        SemanticRule::Op4,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "an indexable base",
                            self.checked_type_name(ty)?,
                        ),
                    );
                }
            };
            let mut base = ResolvedPlace {
                root,
                path: path.iter().map(CheckedPlaceStep::place_step).collect(),
            };
            if let Some(local) = root_binding
                && let Some(reference) = &local.reference
                && let Some(named) = reference.paths.first()
            {
                let mut resolved = named.clone();
                resolved.path.extend(base.path.iter().copied());
                base = resolved;
            }
            (
                CheckedRangeSource::Storage(CheckedContainerRoot { root, path, ty }),
                base,
                element,
            )
        };
        let Some(element) = self.flat_element(element_type)? else {
            return self.unsupported(UnsupportedSemanticFeature::ReferenceFormation, place_node);
        };
        let mut endpoints = Vec::with_capacity(2);
        for node in [start_node, end_node] {
            let mut probe = bindings.clone();
            let value = self.check_atom(function, node, &mut probe, loop_depth)?;
            if value.expression.ty() != CheckedType::Integer(crate::semantic::model::IntegerType::U64)
                || value.mode != CheckedMode::Own
            {
                return self.issue_node(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::type_mismatch(
                        "own u64",
                        self.checked_value_name(value.mode, value.expression.ty())?,
                    ),
                );
            }
            carried.effects = carried.effects.union(value.effects.clone());
            carried.accesses.extend(value.accesses.clone());
            endpoints.push(value);
        }
        let end = endpoints.pop().ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let start = endpoints.pop().ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let captured_start = Checker::captured_of(start_node, &start.expression)
            .unwrap_or(CapturedValue::unknown());
        let captured_end =
            Checker::captured_of(end_node, &end.expression).unwrap_or(CapturedValue::unknown());
        // [OWN-7] the formed reference names the base path extended by its
        // own range step; every later separation question reads that step.
        let mut place = base_place;
        place.path.push(PlaceStep::Range(CapturedRange {
            start: captured_start,
            end: captured_end,
        }));
        let expression = CheckedExpression::RangeOf {
            carrier: self.tree.path(carrier)?.clone(),
            source,
            element,
            start: Box::new(start.expression),
            end: Box::new(end.expression),
            obligation: self.tree.path(suffix)?.clone(),
        };
        let mut accesses = carried.accesses;
        accesses.push(PlaceAccess {
            place: place.clone(),
            kind: AccessKind::Reference,
        });
        Ok(TypedExpression {
            expression,
            mode: CheckedMode::Range,
            reference: Some(ReferenceInfo::formed(ReferenceKind::Range, place)),
            reference_value: true,
            effects: carried.effects,
            accesses,
        })
    }

    /// Converts a resolved runtime place into the static state path a
    /// callable boundary declares [EFF-1, EFF-2].
    ///
    /// The path is complete wherever [EFF-1] can express it: field
    /// selections, `deref`, payload steps, window parts and measures map one
    /// for one, and an index or range position maps only when the value it
    /// captured is a value parameter of the same callable, which is the only
    /// index spelling a signature admits. Anything else is a dynamic element
    /// and maps to its nearest statically nameable enclosing path, which is
    /// the prefix above the step [EFF-2].
    pub(super) fn state_path(
        &self,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Option<CheckedStatePath>, CheckStop> {
        let PlaceRoot::Binding(binding) = place.root else {
            // A const root contributes no effect [EFF-2, CONST-2].
            return Ok(None);
        };
        let Some(local) = bindings.values().find(|local| local.binding == binding) else {
            return Ok(None);
        };
        let mut steps = Vec::new();
        for step in &place.path {
            let mapped = match step {
                PlaceStep::Field(field) => CheckedEffectStep::Field(*field),
                PlaceStep::Deref => CheckedEffectStep::Deref,
                PlaceStep::Payload { variant, field } => CheckedEffectStep::Payload {
                    variant: *variant,
                    field: *field,
                },
                PlaceStep::Part(part) => CheckedEffectStep::Part(*part),
                PlaceStep::Measure(measure) => CheckedEffectStep::Measure(*measure),
                PlaceStep::Index(captured) => {
                    let Some(parameter) = self.captured_parameter(*captured, bindings) else {
                        break;
                    };
                    CheckedEffectStep::Index(parameter)
                }
                PlaceStep::Range(range) => {
                    let (Some(start), Some(end)) = (
                        self.captured_parameter(range.start, bindings),
                        self.captured_parameter(range.end, bindings),
                    ) else {
                        break;
                    };
                    CheckedEffectStep::Range { start, end }
                }
            };
            steps.push(mapped);
        }
        Ok(Some(CheckedStatePath {
            root: local.declaration,
            steps,
        }))
    }

    /// The value parameter one captured index or endpoint read, when it read
    /// one [EFF-1]: a signature never contains an index expression, so this
    /// is the only index spelling a declared row admits.
    fn captured_parameter(
        &self,
        captured: super::super::places::CapturedValue,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Option<DeclarationId> {
        let binding = captured.support()?;
        let local = bindings.values().find(|local| local.binding == binding)?;
        self.resolved
            .declarations()
            .iter()
            .find(|declaration| {
                declaration.id() == local.declaration
                    && declaration.role() == DeclarationRole::Parameter
            })
            .map(|declaration| declaration.id())
    }

    /// [EFF-2] the enclosing formal-rooted effect of one resolved access.
    ///
    /// An access rooted only in local storage contributes no enclosing path,
    /// including through a local reference; the checked access and its
    /// ordinary footprint remain. A const root contributes no read.
    pub(super) fn effect_paths_for_place(
        &self,
        _node: NodeId,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Vec<EffectPath>, CheckStop> {
        let PlaceRoot::Binding(binding) = place.root else {
            return Ok(Vec::new());
        };
        let Some(local) = bindings.values().find(|local| local.binding == binding) else {
            return Ok(Vec::new());
        };
        // [EFF-1] every `effect_path` is rooted at one reference parameter of
        // the same callable; a by-value parameter has no effect entry at all.
        if local.mode == CheckedMode::Own {
            return Ok(Vec::new());
        }
        let is_parameter = self.resolved.declarations().iter().any(|declaration| {
            declaration.id() == local.declaration && declaration.role() == DeclarationRole::Parameter
        });
        if !is_parameter {
            return Ok(Vec::new());
        }
        Ok(self
            .state_path(place, bindings)?
            .into_iter()
            .map(EffectPath::from)
            .collect())
    }

    /// A callee's declared formal effect covers the complete selected actual
    /// [EFF-5].
    pub(super) fn effect_paths_for_whole_place(
        &self,
        node: NodeId,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Vec<EffectPath>, CheckStop> {
        self.effect_paths_for_place(node, place, bindings)
    }

    /// A measure reads the selected run's descriptor storage and no slot
    /// [MSR-2, WIN-2].
    pub(super) fn effect_paths_for_descriptor(
        &self,
        node: NodeId,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        measure: CheckedMeasure,
    ) -> Result<Vec<EffectPath>, CheckStop> {
        let mut descriptor = place.clone();
        descriptor.path.push(PlaceStep::Measure(measure));
        self.effect_paths_for_place(node, &descriptor, bindings)
    }

    /// The struct-field prefix of a resolved place, for a diagnostic or a
    /// drop path that has fields and nothing else.
    pub(super) fn field_prefix(place: &ResolvedPlace) -> Vec<u32> {
        place
            .path
            .iter()
            .map_while(|step| match step {
                PlaceStep::Field(field) => Some(*field),
                _ => None,
            })
            .collect()
    }

    /// [TYPE-7] whether this operand would be read through a reference or a
    /// cell to satisfy the position it stands in.
    ///
    /// There is no implicit read through a reference or through a cell, so a
    /// reference or `Box` binding written where a value of its referent type
    /// is expected is a hard error whose mechanical fix is `deref(.)`.
    pub(super) fn reads_implicitly_through_holder(
        &self,
        holds_reference: bool,
        ty: CheckedType,
        required: RequiredReferent,
    ) -> Result<bool, CheckStop> {
        if holds_reference {
            return self.satisfies_referent_requirement(ty, required);
        }
        let CheckedType::Nominal(nominal) = ty else {
            return Ok(false);
        };
        let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind else {
            return Ok(false);
        };
        self.satisfies_referent_requirement(referent, required)
    }

    pub(super) fn satisfies_referent_requirement(
        &self,
        ty: CheckedType,
        required: RequiredReferent,
    ) -> Result<bool, CheckStop> {
        Ok(match required {
            RequiredReferent::Exact(required) => ty == required,
            RequiredReferent::Enum => match ty {
                CheckedType::Bool => true,
                CheckedType::Nominal(nominal) => {
                    matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Enum { .. })
                }
                _ => false,
            },
            // [OP-4, TYPE-9] a storage shape is an indexable base, so a shape
            // holder written where its referent is required is the same
            // [TYPE-7] missing `deref`.
            RequiredReferent::IndexableStorage => matches!(
                ty,
                CheckedType::Array { .. }
                    | CheckedType::Buffer { .. }
                    | CheckedType::Window { .. }
            ),
        })
    }

    /// [REF-2] the bindings one scope exit ends, which are the bindings of
    /// the inner state the enclosing state does not declare.
    pub(super) fn bindings_leaving_scope(
        inner: &HashMap<DeclarationId, LocalBinding>,
        enclosing: &[DeclarationId],
    ) -> Vec<BindingId> {
        inner
            .iter()
            .filter(|(declaration, _)| !enclosing.contains(declaration))
            .map(|(_, local)| local.binding)
            .collect()
    }

    /// Whether a type is a nominal whose kind is a struct, for the field
    /// walks above.
    pub(super) fn struct_field_type(
        &self,
        ty: CheckedType,
        field: u32,
    ) -> Result<Option<CheckedType>, CheckStop> {
        let CheckedType::Nominal(nominal) = ty else {
            return Ok(None);
        };
        let CheckedNominalKind::Struct { fields } = &self.nominal(nominal)?.kind else {
            return Ok(None);
        };
        Ok(fields.get(field as usize).map(|field| field.ty))
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidationEvent, ReferenceInfo, ReferenceKind, ReferenceValidity};
    use crate::semantic::model::BindingId;
    use crate::semantic::places::{PlaceStep, ResolvedPlace};

    /// This module is the only place [REF-2]'s validity lattice can be read
    /// directly: `ReferenceInfo` is `pub(super)` inside
    /// `crate::semantic::check`, so `crate::semantic::tests` cannot name it.
    /// The source-level half of the same judgment — which programs the
    /// invalidation events actually reject — is in
    /// `crate::semantic::tests::references`.
    fn reference(binding: u32) -> ReferenceInfo {
        ReferenceInfo::formed(
            ReferenceKind::Single,
            ResolvedPlace::binding(BindingId(binding)),
        )
    }

    /// [REF-2] a freshly formed reference is valid, and validity is
    /// re-established only by forming the reference again: a second
    /// invalidating event does not overwrite the first, because the
    /// diagnostic names the event that made the reference invalid.
    #[test]
    fn invalidation_is_sticky_and_keeps_the_first_event() {
        let mut info = reference(0);
        assert!(info.is_valid());
        info.invalidate(InvalidationEvent::PrefixWritten);
        assert_eq!(
            info.validity,
            ReferenceValidity::Invalid(InvalidationEvent::PrefixWritten)
        );
        info.invalidate(InvalidationEvent::RootScopeEnded);
        assert_eq!(
            info.validity,
            ReferenceValidity::Invalid(InvalidationEvent::PrefixWritten),
            "a second event must not relabel an already invalid reference"
        );
    }

    /// [REF-2] every one of the seven enumerated events, and no other, carries
    /// its own phrase into the diagnostic. The phrases are distinct, so a
    /// rejection names which event happened.
    #[test]
    fn every_invalidation_event_has_its_own_phrase() {
        let events = [
            InvalidationEvent::PrefixWritten,
            InvalidationEvent::PrefixMoved,
            InvalidationEvent::PrefixReleased,
            InvalidationEvent::CallWrite,
            InvalidationEvent::RootScopeEnded,
            InvalidationEvent::RefinementLost,
            InvalidationEvent::WindowBoundaryMoved,
        ];
        let mut phrases: Vec<&'static str> = events.iter().map(InvalidationEvent::phrase).collect();
        let written = phrases.len();
        phrases.sort_unstable();
        phrases.dedup();
        assert_eq!(phrases.len(), written, "two events share one phrase");
    }

    /// [REF-1] the join of two incoming edges is the union of the path sets
    /// and the meet of the validity facts, because every check on the binding
    /// must hold on every incoming edge.
    #[test]
    fn a_join_unions_the_paths_and_meets_the_validity() {
        let mut left = reference(0);
        let right = reference(1);
        left.join(&right);
        assert_eq!(
            left.paths.len(),
            2,
            "the join is the union of the path sets"
        );
        assert!(left.is_valid(), "two valid edges join to a valid reference");

        let mut valid = reference(0);
        let mut invalid = reference(1);
        invalid.invalidate(InvalidationEvent::CallWrite);
        valid.join(&invalid);
        assert_eq!(
            valid.validity,
            ReferenceValidity::Invalid(InvalidationEvent::CallWrite),
            "one invalid edge makes the joined reference invalid"
        );
    }

    /// [REF-1] a join does not duplicate a path both edges already name.
    #[test]
    fn a_join_keeps_one_copy_of_a_shared_path() {
        let mut left = reference(0);
        let right = reference(0);
        left.join(&right);
        assert_eq!(left.paths.len(), 1);
    }

    /// [REF-1] a path has a static shape: a loop-carried rebinding may change
    /// only the index values inside the path, and may never extend the path
    /// through itself.
    #[test]
    fn the_static_shape_admits_a_moved_index_and_refuses_a_longer_path() {
        let indexed = |capture: u32, value: u64| {
            let mut place = ResolvedPlace::binding(BindingId(0));
            place.push_subscript(crate::semantic::places::CapturedValue::new(
                crate::semantic::places::CaptureId(capture),
                crate::semantic::places::CapturedTerm::Literal(value),
            ));
            ReferenceInfo::formed(ReferenceKind::Single, place)
        };
        let first = indexed(0, 1);
        let moved = indexed(1, 2);
        assert!(
            first.shape_agrees_with(&moved),
            "a rebinding may move the captured index value"
        );

        let mut longer = ResolvedPlace::binding(BindingId(0));
        longer.push_subscript(crate::semantic::places::CapturedValue::new(
            crate::semantic::places::CaptureId(2),
            crate::semantic::places::CapturedTerm::Literal(1),
        ));
        longer.path.push(PlaceStep::Field(0));
        let extended = ReferenceInfo::formed(ReferenceKind::Single, longer);
        assert!(
            !first.shape_agrees_with(&extended),
            "a rebinding may not extend the path through itself"
        );
    }
}
