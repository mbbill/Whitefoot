//! Structural [OWN-5] place resolution and the [OWN-7] overlap relation.
//!
//! A resolved place is a declaration-anchored root plus a field path, reached
//! by reading `let`-bound borrows through to the storage they name. The
//! prepass that builds the per-binding holder summaries is purely syntactic:
//! it reads the checked statement tree and nothing else, so every consumer
//! sees the same relation regardless of what any later analysis derives.
//!
//! Two consumers share this module. [ENT-5] kills project a callee's declared
//! `writes` onto its actual argument places and kill every fact whose support
//! overlaps one [`super::entailment`]. The permission judgment
//! [`super::permission`] projects the same boundary and asks whether two
//! sibling calls' footprints are disjoint. Neither may grow a private copy of
//! the overlap relation.

use crate::DeclarationId;

use super::model::{
    BindingId, CheckedConstantId, CheckedExpression, CheckedFunction, CheckedMatchArm, CheckedMode,
    CheckedStatement, CheckedType, IntegerType,
};

/// Root of a tracked place: a function-local binding (parameters, `let`
/// bindings of every right-hand form, and match binders share the dense
/// [`BindingId`] space) or a named const [CONST-2].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceRoot {
    Binding(BindingId),
    Constant(CheckedConstantId),
}

/// One [OP-4] subscript offset occurring inside a tracked place [MSR-1].
///
/// [OWN-7] and [LIV-2] condition 2 decide two offsets by one relation: two
/// written literals are provably distinct exactly when their values differ,
/// and every other offset is opaque to it — provably distinct from nothing,
/// itself included. A live `own` integer binding and an in-scope const
/// generic are retained rather than collapsed to that opacity because they
/// are places and terms in their own right: [ENT-5] takes the support of
/// every offset occurring in P into the support of a measure over P, and a
/// write to the binding an offset reads therefore kills at every level.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceOffset {
    /// A written integer literal.
    Literal(u64),
    /// A live `own` integer binding read at the subscript.
    Binding(BindingId),
    /// An in-scope const generic [CONST-1], fixed at instantiation [FN-2].
    Const(DeclarationId),
    /// An offset no relation of this document decides: a computed value, or
    /// a callee's projected element write, which names no offset at all.
    Opaque,
}

impl PlaceOffset {
    /// [OWN-7, LIV-2] whether the two offsets name two elements on every
    /// execution.
    pub(crate) const fn provably_distinct(self, other: Self) -> bool {
        match (self, other) {
            (Self::Literal(left), Self::Literal(right)) => left != right,
            _ => false,
        }
    }

    /// [LIV-2] whether the two offsets name one element on every execution,
    /// which is what a read-out of an element target needs.
    pub(crate) const fn provably_same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Literal(left), Self::Literal(right)) => left == right,
            _ => false,
        }
    }

    /// The support one offset contributes to every measure term of the place
    /// it occurs in [ENT-5]: the binding it reads, where it reads one.
    pub(crate) const fn support(self) -> Option<BindingId> {
        match self {
            Self::Binding(binding) => Some(binding),
            Self::Literal(_) | Self::Const(_) | Self::Opaque => None,
        }
    }
}

/// One step of a tracked place's path below its root: a field selection, or
/// one subscript of an indexable base [OP-4].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceStep {
    Field(u32),
    Subscript(PlaceOffset),
}

impl PlaceStep {
    /// [LIV-2] whether the two steps select one storage on every execution.
    pub(crate) const fn provably_same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Subscript(left), Self::Subscript(right)) => left.provably_same(right),
            (Self::Field(left), Self::Field(right)) => left == right,
            _ => false,
        }
    }

    /// Whether the two steps select two storages on every execution, which
    /// for two subscripts is [OWN-7]'s own offset relation and for everything
    /// else is inequality.
    pub(crate) const fn provably_distinct(self, other: Self) -> bool {
        match (self, other) {
            (Self::Subscript(left), Self::Subscript(right)) => left.provably_distinct(right),
            (Self::Field(left), Self::Field(right)) => left != right,
            _ => true,
        }
    }
}

/// One tracked place [ENT-2](a): a root, an optional `deref` reading through
/// a borrow or box holder, and field selections — never an index segment.
///
/// This compact form represents no deref or one leading deref followed by
/// fields. Interleaved or repeated derefs, and every subscripted place, use
/// [`ProjectedPlaceTerm`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlaceTerm {
    pub(crate) root: PlaceRoot,
    pub(crate) deref: bool,
    pub(crate) fields: Vec<u32>,
}

/// One source-order projection in a tracked place whose spelling cannot be
/// represented by [`PlaceTerm`]'s legacy "one leading deref, then fields"
/// shape. Keeping the order makes `deref(h.value)` distinct from
/// `deref(h).value`, as [ENT-2] requires.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PlaceProjection {
    Field(u32),
    Deref,
    /// One subscript of the base reached so far [OP-4, MSR-1]. The offset is
    /// a logical one; the storage it selects is `(head_of + i) mod cap_of`,
    /// which no source rule mentions [BLK-1].
    Subscript(PlaceOffset),
}

/// The exact source-order path of a tracked place with interleaved field and
/// deref projections. The root remains declaration-anchored; projections are
/// finite because the checked expression tree is finite.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProjectedPlaceTerm {
    pub(crate) root: PlaceRoot,
    pub(crate) projections: Vec<PlaceProjection>,
}

/// One [OWN-5] resolved place, for the [OWN-7] overlap relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedPlace {
    pub(crate) root: PlaceRoot,
    pub(crate) path: Vec<PlaceStep>,
}

impl ResolvedPlace {
    /// [OWN-7]: places overlap when one's path is a prefix of the other's.
    ///
    /// Two subscripts of one base are one step of that prefix unless their
    /// offsets are provably distinct, which is the sentence [OWN-7] states
    /// for two subscripted places and [LIV-2]'s second condition reads at a
    /// commit.
    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        self.root == other.root && !paths_diverge(&self.path, &other.path)
    }

    /// Whether one place's path reaches the other's storage: `prefix` selects
    /// the same storage as, or a storage containing, `path`.
    pub(crate) fn is_prefix_of(&self, other: &Self) -> bool {
        self.root == other.root
            && self.path.len() <= other.path.len()
            && !paths_diverge(&self.path, &other.path)
    }

    /// The whole storage of one binding, with no selection below the root.
    pub(crate) fn binding(binding: BindingId) -> Self {
        Self {
            root: PlaceRoot::Binding(binding),
            path: Vec::new(),
        }
    }

    /// The same place with one field selection appended, for a caller that
    /// holds a plain field path.
    pub(crate) fn extend_fields(&mut self, fields: &[u32]) {
        self.path
            .extend(fields.iter().copied().map(PlaceStep::Field));
    }
}

/// Whether two paths select two disjoint storages: some step in their common
/// prefix provably selects two different storages [OWN-7].
pub(crate) fn paths_diverge(left: &[PlaceStep], right: &[PlaceStep]) -> bool {
    left.iter()
        .zip(right)
        .any(|(left, right)| left.provably_distinct(*right))
}

/// What one binding reads through by `deref`, for place resolution.
#[derive(Clone, Debug)]
pub(crate) enum HolderReferent {
    /// A borrow of a known local place.
    Place {
        binding: BindingId,
        fields: Vec<u32>,
    },
    /// A reborrow: reads through another holder.
    Holder(BindingId),
    /// A parameter or match-binder borrow, or an owning box: the referent has
    /// no caller-visible local place, so the binding itself anchors identity.
    Opaque,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingSummary {
    pub(crate) ty: Option<CheckedType>,
    pub(crate) holder: Option<HolderReferent>,
    /// The checked expression for a read through a borrow holder may retain
    /// only the referent value. Reconstructing its [ENT-2] source spelling
    /// must restore that explicit `deref`; an owning box already retains a
    /// `BoxDeref` node and therefore does not use this flag.
    pub(crate) implicit_deref: bool,
    /// Exact [GIVE-1] source class admitted as a bounded-delivery carrier.
    pub(crate) delivery_carrier: bool,
    /// The storage a bound view value was formed over [VIEW-1, VIEW-2].
    ///
    /// A view is a value that reaches storage it does not own, and the loan it
    /// holds is on the range of its **origin** place, not on the descriptor
    /// word the binding occupies. Every judgment that asks what storage an
    /// argument reaches therefore has to read through the binding to that
    /// origin: `mut_slice_of(&uniq report)` bound to `window` and then handed
    /// on as `&uniq window` reaches `report`, exactly as the inline formation
    /// written at the call does.
    ///
    /// `None` means this binding is not a view, or is a view whose origin this
    /// prepass does not resolve — a view parameter, or one a callee returned.
    /// Every consumer reads that as unresolved and fails closed.
    pub(crate) view_origin: Option<ResolvedPlace>,
}

/// Dense per-binding structural summaries for one checked function, and the
/// place resolution they support.
#[derive(Debug, Default)]
pub(crate) struct PlaceMap {
    /// Dense per-binding summaries indexed by [`BindingId`].
    bindings: Vec<BindingSummary>,
}

impl PlaceMap {
    /// Runs the binding prepass over one complete function body.
    pub(crate) fn for_function(function: &CheckedFunction) -> Self {
        let mut map = Self::default();
        for parameter in &function.parameters {
            let (holder, implicit_deref) = match parameter.mode {
                CheckedMode::Own => (None, false),
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => {
                    (Some(HolderReferent::Opaque), true)
                }
            };
            let summary = map.summary_mut(parameter.binding);
            summary.ty = Some(parameter.ty);
            summary.holder = holder;
            summary.implicit_deref = implicit_deref;
            summary.delivery_carrier = matches!(parameter.mode, CheckedMode::Own);
        }
        map.collect_block_bindings(&function.body);
        map
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

    fn collect_block_bindings(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let (holder, implicit_deref) = match value {
                        CheckedExpression::Binding {
                            binding: source, ..
                        } if self.is_holder(*source) => {
                            (Some(HolderReferent::Holder(*source)), true)
                        }
                        _ => (holder_from_value(value), value_has_implicit_deref(value)),
                    };
                    // Resolved before the summary is taken, because resolving
                    // the origin reads the summaries of the bindings the
                    // formation names, and each of those was declared earlier.
                    let view_origin = self.view_origin_of(value);
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.holder = holder;
                    summary.implicit_deref = implicit_deref;
                    summary.delivery_carrier = summary.holder.is_none();
                    summary.view_origin = view_origin;
                }
                // [CALL-4] every binder of a destructuring `let` is an
                // ordinary fresh binding of its result ordinal's type.
                CheckedStatement::DestructuringLet { bindings, .. } => {
                    for (binding, ty) in bindings {
                        let summary = self.summary_mut(*binding);
                        summary.ty = Some(*ty);
                        summary.delivery_carrier = true;
                    }
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(*ok_type);
                    summary.delivery_carrier = true;
                }
                CheckedStatement::Replace {
                    binding, target, ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(target.ty());
                    summary.delivery_carrier = true;
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    arms,
                    ..
                } => {
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(*result_type);
                    summary.delivery_carrier = true;
                    for arm in arms {
                        self.collect_arm_bindings(arm);
                    }
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
                    summary.delivery_carrier = true;
                    self.collect_block_bindings(body);
                }
                _ => {}
            }
        }
    }

    fn collect_arm_bindings(&mut self, arm: &CheckedMatchArm) {
        for binder in &arm.binders {
            let (holder, implicit_deref) = match binder.mode {
                CheckedMode::Own => (None, false),
                CheckedMode::Shared(_) | CheckedMode::Unique(_) => {
                    (Some(HolderReferent::Opaque), true)
                }
            };
            let summary = self.summary_mut(binder.binding);
            summary.ty = Some(binder.ty);
            summary.holder = holder;
            summary.implicit_deref = implicit_deref;
            summary.delivery_carrier = matches!(binder.mode, CheckedMode::Own);
        }
        self.collect_block_bindings(&arm.body);
    }

    pub(crate) fn is_holder(&self, binding: BindingId) -> bool {
        self.summary(binding)
            .is_some_and(|summary| summary.holder.is_some())
    }

    /// The storage one view binding was formed over [VIEW-1, VIEW-2], when
    /// this prepass resolves it.
    pub(crate) fn view_origin(&self, binding: BindingId) -> Option<ResolvedPlace> {
        self.summary(binding)
            .and_then(|summary| summary.view_origin.clone())
    }

    /// The origin a `let` right-hand side gives the view it binds.
    ///
    /// Exactly two right-hand sides carry one. A formation [VIEW-2] views the
    /// storage its operand names, which is the place [`super::permission::
    /// slice_source_place`] already resolves for an inline formation; and a
    /// copy of a shared view is a second loan on the same range [VIEW-1], so
    /// it carries its source's origin unchanged. Everything else — a view
    /// parameter, a view a callee returned — leaves this `None`, and the
    /// judgments that read it deny rather than guess.
    fn view_origin_of(&self, value: &CheckedExpression) -> Option<ResolvedPlace> {
        match value {
            CheckedExpression::SliceOf { source, .. } => {
                Some(super::permission::slice_source_place(self, source))
            }
            CheckedExpression::Binding {
                binding,
                ty: CheckedType::Slice { .. },
                ..
            } => self.view_origin(*binding),
            _ => None,
        }
    }

    /// Resolves a spelled place to its [OWN-5] resolved place, reading
    /// through let-bound borrows; opaque holders anchor at themselves.
    pub(crate) fn resolve(&self, place: &PlaceTerm) -> ResolvedPlace {
        match place.root {
            PlaceRoot::Constant(id) => {
                let mut resolved = ResolvedPlace {
                    root: PlaceRoot::Constant(id),
                    path: Vec::new(),
                };
                resolved.extend_fields(&place.fields);
                resolved
            }
            PlaceRoot::Binding(binding) => {
                let mut resolved = if place.deref {
                    self.resolve_deref(binding, 0)
                } else {
                    ResolvedPlace::binding(binding)
                };
                resolved.extend_fields(&place.fields);
                resolved
            }
        }
    }

    /// Resolves an exact interleaved field/deref spelling for [ENT-5] kills.
    /// A deref of a direct holder follows the existing holder summary. A
    /// deref after a field remains anchored at that selected storage path;
    /// replacing any prefix therefore conservatively kills the fact.
    pub(crate) fn resolve_projected(&self, place: &ProjectedPlaceTerm) -> ResolvedPlace {
        let mut resolved = ResolvedPlace {
            root: place.root,
            path: Vec::new(),
        };
        for projection in &place.projections {
            match projection {
                PlaceProjection::Field(field) => resolved.path.push(PlaceStep::Field(*field)),
                PlaceProjection::Subscript(offset) => {
                    resolved.path.push(PlaceStep::Subscript(*offset));
                }
                PlaceProjection::Deref => {
                    if resolved.path.is_empty()
                        && let PlaceRoot::Binding(binding) = resolved.root
                    {
                        resolved = self.resolve_deref(binding, 0);
                    }
                }
            }
        }
        resolved
    }

    pub(crate) fn resolve_deref(&self, holder: BindingId, depth: usize) -> ResolvedPlace {
        let anchored = ResolvedPlace::binding(holder);
        if depth > 32 {
            return anchored;
        }
        match self
            .summary(holder)
            .and_then(|summary| summary.holder.as_ref())
        {
            Some(HolderReferent::Place { binding, fields }) => {
                let mut resolved = if self.is_holder(*binding) {
                    self.resolve_deref(*binding, depth + 1)
                } else {
                    ResolvedPlace::binding(*binding)
                };
                resolved.extend_fields(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => self.resolve_deref(*next, depth + 1),
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    pub(crate) fn resolve_deref_with_holders(
        &self,
        holder: BindingId,
        depth: usize,
        holders: &mut Vec<BindingId>,
    ) -> ResolvedPlace {
        holders.push(holder);
        let anchored = ResolvedPlace::binding(holder);
        if depth > 32 {
            return anchored;
        }
        match self
            .summary(holder)
            .and_then(|summary| summary.holder.as_ref())
        {
            Some(HolderReferent::Place { binding, fields }) => {
                let mut resolved = if self.is_holder(*binding) {
                    self.resolve_deref_with_holders(*binding, depth + 1, holders)
                } else {
                    ResolvedPlace::binding(*binding)
                };
                resolved.extend_fields(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => {
                self.resolve_deref_with_holders(*next, depth + 1, holders)
            }
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    /// The place a callee writes through one view actual [CALL-3, VIEW-4].
    ///
    /// A view is handed to a callee as itself, because the descriptor is what
    /// a borrow of one carries, and [`Self::argument_referent`] leaves that
    /// shape unresolved: the permission judgment fails closed on it. The kill
    /// projection has an exact answer for it, though — [VIEW-4] forbids
    /// replacing a view through such a borrow, so what the callee writes
    /// reaches element storage only, and the kill is the element write over
    /// the view's own place, exactly as a `set` through a view subscript is.
    /// Every measure of the view survives it.
    pub(crate) fn viewed_write_referent(
        &self,
        argument: &CheckedExpression,
    ) -> Option<ResolvedPlace> {
        let CheckedExpression::Binding {
            binding,
            ty: CheckedType::Slice { .. },
            ..
        } = argument
        else {
            return None;
        };
        Some(self.resolve(&PlaceTerm {
            root: PlaceRoot::Binding(*binding),
            deref: self.is_holder(*binding),
            fields: Vec::new(),
        }))
    }

    /// The resolved place a borrow-shaped call argument reads through, and
    /// whether that shape is a directly transferred holder.
    ///
    /// This is the [EFF-2] boundary projection's argument half: exactly the
    /// actual shapes through which a declared `reads`/`writes` region reaches
    /// caller storage. Every other shape is unresolved here, and each caller
    /// decides what an unresolved actual means for it.
    ///
    /// How far such a write *reaches* is not read from these shapes: [CALL-5]
    /// selects the transport from the callee's declared parameter mode and
    /// type, so that classification belongs to the callee's declaration and
    /// never to the argument's spelling [CALL-1, CALL-2, CALL-3].
    pub(crate) fn argument_referent(
        &self,
        argument: &CheckedExpression,
    ) -> Option<(ResolvedPlace, bool)> {
        match argument {
            CheckedExpression::BorrowBuffer { root, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                Some((self.resolve(&place), false))
            }
            // A borrowed system resource that is a struct field names that
            // field, so a write through it kills the facts on the field and
            // not on its siblings [SYS-18, ENT-5].
            CheckedExpression::BorrowSystemResource {
                binding, fields, ..
            } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: fields.clone(),
                };
                Some((self.resolve(&place), false))
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: Vec::new(),
                };
                Some((self.resolve(&place), false))
            }
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                Some((self.resolve_deref(*binding, 0), false))
            }
            CheckedExpression::Binding { binding, .. } if self.is_holder(*binding) => {
                Some((self.resolve_deref(*binding, 0), true))
            }
            _ => None,
        }
    }
}

pub(crate) fn holder_from_value(value: &CheckedExpression) -> Option<HolderReferent> {
    match value {
        CheckedExpression::BorrowSystemResource {
            binding, fields, ..
        } => Some(HolderReferent::Place {
            binding: *binding,
            fields: fields.clone(),
        }),
        CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. } => Some(HolderReferent::Place {
            binding: *binding,
            fields: Vec::new(),
        }),
        CheckedExpression::BorrowBuffer { root, .. } => Some(HolderReferent::Place {
            binding: root.binding,
            fields: root.fields.clone(),
        }),
        CheckedExpression::ReborrowAddressed { binding, .. } => {
            Some(HolderReferent::Holder(*binding))
        }
        // A bound borrow-mode call result reads and writes through the
        // provenance-candidate actual's storage, so a write through it must
        // kill exactly the facts on that storage [ENT-5, OWN-6].
        CheckedExpression::UserCall {
            result_borrow: Some(result_borrow),
            ..
        } => Some(HolderReferent::Place {
            binding: result_borrow.binding,
            fields: result_borrow.fields.clone(),
        }),
        CheckedExpression::BoxNew { .. } | CheckedExpression::ArenaNew { .. } => {
            Some(HolderReferent::Opaque)
        }
        _ => None,
    }
}

/// Whether a `let` right-hand side binds a holder whose reads are spelled
/// with an implicit `deref` in the checked tree.
pub(crate) const fn value_has_implicit_deref(value: &CheckedExpression) -> bool {
    matches!(
        value,
        CheckedExpression::BorrowAddressed { .. }
            | CheckedExpression::BorrowBuffer { .. }
            | CheckedExpression::BorrowBox { .. }
            | CheckedExpression::BorrowSystemResource { .. }
            | CheckedExpression::ReborrowAddressed { .. }
            | CheckedExpression::UserCall {
                result_borrow: Some(_),
                ..
            }
    )
}
