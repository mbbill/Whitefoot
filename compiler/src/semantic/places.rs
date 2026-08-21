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

/// One tracked place [ENT-2](a): a root, an optional `deref` reading through
/// a borrow or box holder, and field selections — never an index segment.
///
/// This compact form represents no deref or one leading deref followed by
/// fields. Interleaved or repeated derefs use [`ProjectedPlaceTerm`].
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
    pub(crate) fields: Vec<u32>,
}

impl ResolvedPlace {
    /// [OWN-7]: places overlap when one's path is a prefix of the other's.
    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        self.root == other.root
            && (self.fields.starts_with(&other.fields) || other.fields.starts_with(&self.fields))
    }

    /// The whole storage of one binding, with no field selection.
    pub(crate) fn binding(binding: BindingId) -> Self {
        Self {
            root: PlaceRoot::Binding(binding),
            fields: Vec::new(),
        }
    }
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
                    let summary = self.summary_mut(*binding);
                    summary.ty = Some(value.ty());
                    summary.holder = holder;
                    summary.implicit_deref = implicit_deref;
                    summary.delivery_carrier = summary.holder.is_none();
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

    /// Resolves a spelled place to its [OWN-5] resolved place, reading
    /// through let-bound borrows; opaque holders anchor at themselves.
    pub(crate) fn resolve(&self, place: &PlaceTerm) -> ResolvedPlace {
        match place.root {
            PlaceRoot::Constant(id) => ResolvedPlace {
                root: PlaceRoot::Constant(id),
                fields: place.fields.clone(),
            },
            PlaceRoot::Binding(binding) => {
                let mut resolved = if place.deref {
                    self.resolve_deref(binding, 0)
                } else {
                    ResolvedPlace::binding(binding)
                };
                resolved.fields.extend_from_slice(&place.fields);
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
            fields: Vec::new(),
        };
        for projection in &place.projections {
            match projection {
                PlaceProjection::Field(field) => resolved.fields.push(*field),
                PlaceProjection::Deref => {
                    if resolved.fields.is_empty()
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
                resolved.fields.extend_from_slice(fields);
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
                resolved.fields.extend_from_slice(fields);
                resolved
            }
            Some(HolderReferent::Holder(next)) => {
                self.resolve_deref_with_holders(*next, depth + 1, holders)
            }
            Some(HolderReferent::Opaque) | None => anchored,
        }
    }

    /// The resolved place a borrow-shaped call argument reads through, and
    /// whether a callee write through it is an element write.
    ///
    /// This is the [EFF-2] boundary projection's argument half: exactly the
    /// actual shapes through which a declared `reads`/`writes` region reaches
    /// caller storage. Every other shape is unresolved here, and each caller
    /// decides what an unresolved actual means for it.
    pub(crate) fn argument_referent(
        &self,
        argument: &CheckedExpression,
    ) -> Option<(ResolvedPlace, bool, bool)> {
        match argument {
            CheckedExpression::BorrowBuffer { root, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(root.binding),
                    deref: self.is_holder(root.binding),
                    fields: root.fields.clone(),
                };
                Some((self.resolve(&place), true, false))
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. } => {
                let place = PlaceTerm {
                    root: PlaceRoot::Binding(*binding),
                    deref: self.is_holder(*binding),
                    fields: Vec::new(),
                };
                Some((self.resolve(&place), false, false))
            }
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                Some((self.resolve_deref(*binding, 0), false, false))
            }
            CheckedExpression::Binding { binding, ty, .. } if self.is_holder(*binding) => Some((
                self.resolve_deref(*binding, 0),
                matches!(ty, CheckedType::Buffer { .. } | CheckedType::Slice { .. }),
                true,
            )),
            _ => None,
        }
    }
}

pub(crate) fn holder_from_value(value: &CheckedExpression) -> Option<HolderReferent> {
    match value {
        CheckedExpression::BorrowAddressed { binding, .. }
        | CheckedExpression::BorrowBox { binding, .. }
        | CheckedExpression::BorrowSystemResource { binding, .. } => Some(HolderReferent::Place {
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
