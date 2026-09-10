//! [LIV-2] the one `set` commit.
//!
//! One rule writes places, and one function checks every written form of it:
//! `set p = e;`, `set (p, q) = f(...);` and `set (p, q) = e1, e2;` differ only
//! in how many targets they name and where each ordinal's value comes from.
//! The order is the rule's order — every target resolved and judged first,
//! then the whole right-hand side, then the three admission conditions and one
//! commit — so no shape of this statement can reach a commit the others do not.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{DeclarationId, Production, SemanticCompilerFailure, SemanticIssueKind, SemanticRule};

use super::super::super::model::{
    CheckedCommitValues, CheckedSetTarget, CheckedStatement, CheckedWritablePlace,
};
use super::super::super::places::{PlaceOffset, PlaceProjection, PlaceStep, paths_diverge};
use super::super::borrows::{ResolvedPlace, places_overlap};
use super::super::expressions::{MutationAccess, MutationTarget};
use super::super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use super::{ControlScope, StatementResult};

/// [VIEW-4]'s exact restructuring: the rule's own mechanical fix.
///
/// A view is never repaired in place, because the repair the rule names is a
/// second value: the loan the commit would displace belongs to the view that
/// holds it, so a new view is formed and bound rather than written over the
/// old one's place.
const VIEW4_NEW_BINDING: &str =
    "bind a new view under a new `let` rather than committing at this one";

/// One target of the commit being checked, with the state the three admission
/// conditions read at the commit.
struct FormedTarget {
    /// The written `place`, where every rejection about this target is located.
    node: NodeId,
    mutation: MutationTarget,
    /// [LIV-2] the target is a complete binding that was already dead when the
    /// statement resolved it, so this commit reinitializes it and revives it.
    revives: bool,
    /// [LIV-2] the right-hand side read this target's previous value out.
    read_out: bool,
}

/// [LIV-2] one target place of the commit whose right-hand side is being
/// checked, and whether that right-hand side has read it out.
///
/// `place.storage_path` retains the complete selected storage, including
/// owning Box dereferences and the final subscript. Matching and spent-place
/// liveness use this path; the conservative `place.path` still serves the
/// existing overlap and loan relations.
pub(in crate::semantic::check) struct CommitReadOut {
    place: ResolvedPlace,
    /// Whether read-out owes the affine-element judgment, including literal
    /// offset equality. A measured place can itself carry a subscript
    /// [MSR-1], so `grid[0][1]` and `grid[1][1]` differ despite agreeing in
    /// their last offset; the flag never substitutes for that complete path.
    element: bool,
    read_out: bool,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [LIV-2] a read-out spends that target for the rest of the RHS,
    /// including later scalar reads or borrows of its descendants. A measure
    /// reads only its descriptor, so reading an enclosing run's descriptor
    /// does not read a spent element [MSR-2].
    pub(in crate::semantic::check) fn check_commit_place_live(
        &self,
        place: &ResolvedPlace,
        node: NodeId,
        descriptor: bool,
    ) -> Result<(), CheckStop> {
        let spent = self.commit_read_outs.borrow().iter().any(|target| {
            target.read_out
                && target.place.root == place.root
                && !target
                    .place
                    .storage_path
                    .iter()
                    .zip(&place.storage_path)
                    .any(|(left, right)| match (left, right) {
                        (PlaceProjection::Field(left), PlaceProjection::Field(right)) => {
                            left != right
                        }
                        (PlaceProjection::Subscript(left), PlaceProjection::Subscript(right)) => {
                            left.provably_distinct(*right)
                        }
                        (PlaceProjection::Deref, PlaceProjection::Deref) => false,
                        _ => true,
                    })
                && (!descriptor || target.place.storage_path.len() <= place.storage_path.len())
        });
        if spent {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        Ok(())
    }

    /// Whether `place` is a read-out of a target of the commit now being
    /// checked, recording that read-out when it is [LIV-2].
    ///
    /// The rule's own sentence, and nothing else: the moved place is a target
    /// place, or a place reached through one. A `move` of a strict prefix of a
    /// target is an ordinary consuming use and is not answered here.
    ///
    /// One target is read out at most once, because after its read-out the
    /// target is dead for the remainder of the evaluation [LIV-2]. A second
    /// `move` of the same place is therefore an ordinary use of what that
    /// read-out consumed and is judged as one.
    pub(in crate::semantic::check) fn take_commit_read_out(&self, place: &ResolvedPlace) -> bool {
        self.take_commit_storage_read_out(place, false)
    }

    /// Whether `place[offset]` is the read-out of an element target of the
    /// commit now being checked, recording that read-out when it is [LIV-2].
    ///
    /// An element target is matched only at an offset provably the same as
    /// its own: reading one element out and reinitialising another would
    /// leave the second holding a value that never left and the first holding
    /// none, so an offset this rule cannot decide keeps [STOR-1]'s rejection.
    /// One target is read out at most once, exactly as a whole-place target
    /// is.
    pub(in crate::semantic::check) fn take_commit_element_read_out(
        &self,
        place: &ResolvedPlace,
    ) -> bool {
        self.take_commit_storage_read_out(place, true)
    }

    fn take_commit_storage_read_out(&self, place: &ResolvedPlace, element: bool) -> bool {
        let mut targets = self.commit_read_outs.borrow_mut();
        for target in targets.iter_mut() {
            if target.read_out
                || target.element != element
                || target.place.root != place.root
                || !Self::storage_path_prefix(&target.place.storage_path, &place.storage_path)
            {
                continue;
            }
            target.read_out = true;
            return true;
        }
        false
    }

    fn storage_path_prefix(target: &[PlaceProjection], read: &[PlaceProjection]) -> bool {
        target.len() <= read.len()
            && target
                .iter()
                .zip(read)
                .all(|(target, read)| match (target, read) {
                    (PlaceProjection::Field(left), PlaceProjection::Field(right)) => {
                        PlaceStep::Field(*left).provably_same(PlaceStep::Field(*right))
                    }
                    (PlaceProjection::Deref, PlaceProjection::Deref) => true,
                    (PlaceProjection::Subscript(left), PlaceProjection::Subscript(right)) => {
                        left.provably_same(*right)
                    }
                    _ => false,
                })
    }

    /// Extraction below a Box target needs an explicit account of the old
    /// target's unselected owning content. Retain that capability boundary.
    pub(in crate::semantic::check) fn is_box_descendant_read_out(
        &self,
        place: &ResolvedPlace,
    ) -> bool {
        place.storage_path.contains(&PlaceProjection::Deref)
            && self.commit_read_outs.borrow().iter().any(|target| {
                !target.read_out
                    && target.place.root == place.root
                    && target.place.storage_path.len() < place.storage_path.len()
                    && Self::storage_path_prefix(&target.place.storage_path, &place.storage_path)
            })
    }

    /// [GRAM-4, SET-1, LIV-2] one `set` statement, in every written form.
    pub(super) fn check_commit(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut super::ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let target_nodes = self.tree.children_with(node, Production::Place)?;
        let value_nodes = self.tree.children_with(node, Production::Expr)?;
        if target_nodes.is_empty() || value_nodes.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }

        // [LIV-2] every target place is resolved and judged in written order,
        // before the right-hand side is evaluated, and the resolution is not
        // re-taken at the commit. A target that declares its own binding has
        // nothing to resolve and nothing to read out: it is formed below,
        // once its ordinal has fixed its type.
        let mut targets: Vec<FormedTarget> = Vec::with_capacity(target_nodes.len());
        let mut declaring: Vec<(usize, NodeId, DeclarationId)> = Vec::new();
        let mut effects = EffectSet::NONE;
        for (ordinal, target_node) in target_nodes.iter().enumerate() {
            if let Some(declaration) = self.declaring_commit_target(*target_node)? {
                declaring.push((ordinal, *target_node, declaration));
                continue;
            }
            let revives = self.commit_revives_binding(*target_node, bindings)?;
            let mut mutation =
                self.check_set_target(function, *target_node, bindings, scope.loops.len())?;
            if revives {
                // An entry-dead complete binding has no current owner whose
                // state this initialization could write [LIV-2, EFF-2]. Its
                // bare target evaluates no offsets or other expressions;
                // ordinary target admission, the RHS and the commit kill
                // still apply. Same-statement read-out keeps its own write.
                mutation.effects = EffectSet::NONE;
            }
            for earlier in &targets {
                if self.commit_targets_overlap(&earlier.mutation, &mutation) {
                    return self.issue_node(
                        SemanticRule::Liv2,
                        *target_node,
                        SemanticIssueKind::OverlappingCommitTargets {
                            first: self.place_spelling(earlier.node)?,
                            second: self.place_spelling(*target_node)?,
                            mechanical_fix: "one commit writes pairwise non-overlapping places; \
                                             write the overlapping target in a statement of its own",
                        },
                    );
                }
            }
            effects = effects.union(mutation.effects.clone());
            targets.push(FormedTarget {
                node: *target_node,
                mutation,
                revives,
                read_out: false,
            });
        }

        // [LIV-2] each target's previous value is read out at the start of the
        // right-hand side's evaluation, and the target is dead through it.
        let (values, read_outs) = self.check_commit_values(
            function,
            &targets,
            &value_nodes,
            bindings,
            scope.loops.len(),
        )?;
        for (target, read_out) in targets.iter_mut().zip(read_outs) {
            target.read_out = read_out;
        }
        for value in &values {
            effects = effects.union(value.effects.clone());
        }

        // Condition 3, then the commit itself.
        let ordinals = self.commit_ordinal_types(node, target_nodes.len(), &values)?;
        // [LIV-2, TYPE-5] a declaring target's binding is minted here, dead,
        // with its own ordinal's type; the ordinary target formation then
        // judges it exactly as it judges a `let`-bound binding this commit
        // revives, and the commit below makes it live.
        for (ordinal, target_node, declaration) in declaring {
            let ty = *ordinals
                .get(ordinal)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let binding = Self::allocate_binding(counters.next_binding)?;
            counters
                .binding_names
                .push(self.declaration_spelling(declaration)?);
            if bindings
                .insert(
                    declaration,
                    LocalBinding {
                        binding,
                        declaration,
                        mode: crate::semantic::model::CheckedMode::Own,
                        ty,
                        state_origins: None,
                        live: false,
                        loop_depth: scope.loops.len(),
                        compiler_updated: false,
                        borrow: None,
                        slice: None,
                        slice_loans: Vec::new(),
                        suspended: false,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            // The target names no existing place, so there is nothing to
            // resolve, nothing to read out and nothing to overlap: what the
            // statement writes is this binding's own fresh storage, and
            // [EFF-2] attributes the write to that storage exactly as a `let`
            // attributes its initialization.
            let place = ResolvedPlace::fields(declaration, Vec::new());
            let mut target_effects = EffectSet::NONE;
            for path in self.effect_paths_for_place(target_node, &place, bindings)? {
                target_effects.add_write(path);
            }
            let mutation = MutationTarget {
                declaration,
                access: MutationAccess::Place {
                    holder: None,
                    place: place.clone(),
                },
                place,
                element: false,
                target: CheckedSetTarget::Place(CheckedWritablePlace {
                    binding,
                    fields: Vec::new(),
                    ty,
                    declares: true,
                }),
                effects: target_effects,
                unsupported: None,
            };
            effects = effects.union(mutation.effects.clone());
            targets.insert(
                ordinal.min(targets.len()),
                FormedTarget {
                    node: target_node,
                    mutation,
                    revives: true,
                    read_out: false,
                },
            );
        }
        for (target, ty) in targets.iter().zip(&ordinals) {
            if target.mutation.target.ty() != *ty {
                return self.issue_node(
                    SemanticRule::Type5,
                    value_nodes[0],
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(target.mutation.target.ty())?,
                        self.checked_type_name(*ty)?,
                    ),
                );
            }
        }
        for target in &targets {
            self.judge_commit_admission(target, bindings)?;
        }
        // Every source rejection of this statement is judged above; a target
        // this compiler cannot lower stops here and nowhere earlier [DIAG-1].
        for target in &targets {
            if let Some(feature) = target.mutation.unsupported {
                return self.unsupported(feature, target.node);
            }
        }
        self.commit_bindings(&targets, &values, bindings)?;

        let node_path = self.tree.path(node)?.clone();
        let statement = if targets.len() == 1 && value_nodes.len() == 1 {
            let target = targets
                .into_iter()
                .next()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let value = values
                .into_iter()
                .next()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            CheckedStatement::Set {
                node_path,
                target: target.mutation.target,
                value: value.expression,
            }
        } else {
            let commit_values = if value_nodes.len() == 1 {
                let value = values
                    .into_iter()
                    .next()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let nominal = self
                    .result_list_of(&value)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                CheckedCommitValues::ResultList {
                    nominal,
                    value: Box::new(value.expression),
                }
            } else {
                CheckedCommitValues::Written(
                    values.into_iter().map(|value| value.expression).collect(),
                )
            };
            CheckedStatement::SetList {
                node_path,
                targets: targets
                    .into_iter()
                    .map(|target| target.mutation.target)
                    .collect(),
                values: commit_values,
            }
        };
        Ok(Self::continuing_statement(statement, effects))
    }

    /// Every ordinal value, checked under the read-out context this commit
    /// installs [LIV-2].
    ///
    /// The context is removed before any rejection leaves this function, so no
    /// later statement of any function can read a stale target.
    fn check_commit_values(
        &self,
        function: &FunctionSignature,
        targets: &[FormedTarget],
        value_nodes: &[NodeId],
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<(Vec<super::super::TypedExpression>, Vec<bool>), CheckStop> {
        self.commit_read_outs.replace(
            targets
                .iter()
                .map(|target| {
                    let mut place = target.mutation.place.clone();
                    let offset = match &target.mutation.target {
                        CheckedSetTarget::ArrayIndex(target) => Some(&target.offset),
                        CheckedSetTarget::BufferIndex(target) => Some(&target.offset),
                        CheckedSetTarget::SliceIndex(target) => Some(&target.offset),
                        CheckedSetTarget::Place(_) | CheckedSetTarget::Storage(_) => None,
                    };
                    if let Some(offset) = offset {
                        place.storage_path.push(PlaceProjection::Subscript(
                            Self::place_offset_of(offset).unwrap_or(PlaceOffset::Opaque),
                        ));
                    }
                    CommitReadOut {
                        place,
                        element: target.mutation.element,
                        read_out: false,
                    }
                })
                .collect(),
        );
        let mut values = Vec::with_capacity(value_nodes.len());
        let mut outcome = Ok(());
        for value_node in value_nodes {
            match self.check_expression(function, *value_node, bindings, loop_depth) {
                Ok(value) => values.push(value),
                Err(stop) => {
                    outcome = Err(stop);
                    break;
                }
            }
        }
        let read_outs = self
            .commit_read_outs
            .take()
            .into_iter()
            .map(|target| target.read_out)
            .collect();
        outcome?;
        Ok((values, read_outs))
    }

    /// [LIV-2] condition 3's ordinal types: one call's declared result
    /// ordinals, or the written value list's own types.
    fn commit_ordinal_types(
        &self,
        node: NodeId,
        targets: usize,
        values: &[super::super::TypedExpression],
    ) -> Result<Vec<super::super::super::model::CheckedType>, CheckStop> {
        if values.len() == targets {
            return Ok(values.iter().map(|value| value.expression.ty()).collect());
        }
        // More than one target and exactly one written expression: the
        // right-hand side is the one call whose result ordinals the targets
        // name [CALL-4].
        let [value] = values else {
            return self.issue_node(
                SemanticRule::Type5,
                node,
                SemanticIssueKind::type_mismatch(
                    format!("{targets} committed values"),
                    format!("{} written values", values.len()),
                ),
            );
        };
        let call = self
            .tree
            .first_child_with(node, Production::Expr)?
            .and_then(|expr| self.tree.first_child_with(expr, Production::Call).ok()?)
            .unwrap_or(node);
        let Some(nominal) = self.result_list_of(value) else {
            return self.result_list_shape_rejection(call, targets, value);
        };
        let ordinals = self.result_list_ordinals(nominal)?;
        if ordinals.len() != targets {
            return self.result_list_shape_rejection(call, targets, value);
        }
        Ok(ordinals)
    }

    /// [LIV-2] condition 1, judged at the commit, target by target.
    fn judge_commit_admission(
        &self,
        target: &FormedTarget,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        // The root's liveness is re-established after the right-hand side
        // [SET-1, LIV-1]. A complete binding this commit reinitializes is the
        // one dead root a commit revives — because it was already dead when
        // the statement resolved it, or because this statement's own read-out
        // took its value. Every projected, dereferenced or subscripted target
        // still demands a live root, so a right-hand side that consumed the
        // root by some other path is [OWN-1]'s rejection however this target
        // was read out.
        let reinitializes =
            self.commit_reinitializes_binding(target) && (target.revives || target.read_out);
        if !reinitializes
            && !bindings
                .get(&target.mutation.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .live
        {
            return self.issue_node(
                SemanticRule::Own1,
                target.node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        self.revalidate_mutation_access(&target.mutation.access, bindings, target.node)?;
        let ty = target.mutation.target.ty();
        // [VIEW-4] a commit may not displace a live loan. A commit that
        // displaces a value of loan-bearing type is admitted exactly when the
        // displaced value is consumed by that same statement's right-hand
        // side: otherwise the displaced view survives — as the `replace`
        // binding, or simply as the loan state of a copy target [VIEW-1] that
        // [LIV-2] condition 1 makes dead at the commit with nothing consumed —
        // and its loan would outlive the descriptor whose place it was held
        // from. This is asked before the copy admission below, because the
        // copy view is exactly the target the third disjunct would admit.
        if Self::checked_type_is_loan_bearing(ty) && !target.read_out {
            return self.issue_node(
                SemanticRule::View4,
                target.node,
                SemanticIssueKind::LoanBearingCommitTarget {
                    target_type: self.checked_type_name(ty)?,
                    mechanical_fix: VIEW4_NEW_BINDING,
                },
            );
        }
        if self.is_copy_type(ty)? || target.read_out || target.revives {
            return Ok(());
        }
        // A live affine target whose previous value the right-hand side does
        // not read out is [STOR-1]'s error, kept for exactly that case.
        self.issue_node(
            SemanticRule::Stor1,
            target.node,
            SemanticIssueKind::AffineSetTarget {
                target_type: self.checked_type_name(ty)?,
                mechanical_fix: super::super::expressions::STOR1_REPLACE,
            },
        )
    }

    /// Whether this target is the complete binding its own declaration names,
    /// which is the one target shape a commit reinitializes [LIV-2].
    ///
    /// A `deref` target writes a referent the holder does not own, and a
    /// projected or subscripted target writes one component of a value, so
    /// neither is that shape.
    fn commit_reinitializes_binding(&self, target: &FormedTarget) -> bool {
        matches!(&target.mutation.target, CheckedSetTarget::Place(place) if place.fields.is_empty())
            && target.mutation.place.root == target.mutation.declaration
            && target.mutation.place.path.is_empty()
    }

    /// The commit itself: every target is live afterwards, and a complete
    /// binding takes the ordinal's own ownership identity [LIV-2, EFF-2].
    fn commit_bindings(
        &self,
        targets: &[FormedTarget],
        values: &[super::super::TypedExpression],
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        // Every RHS was evaluated before the atomic commit. In particular a
        // result list's ordinals all use the one returned image, and installing
        // one field cannot change the origin substituted for another field.
        let images = values
            .iter()
            .map(|value| self.state_origins_of_value(value, bindings))
            .collect::<Result<Vec<_>, _>>()?;
        for (index, target) in targets.iter().enumerate() {
            if self.commit_reinitializes_binding(target) {
                bindings
                    .get_mut(&target.mutation.declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .live = true;
            }
            if !self.type_carries_identity(target.mutation.target.ty())? {
                continue;
            }
            let image = if values.len() == targets.len() {
                images.get(index).cloned().flatten()
            } else {
                let field =
                    u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                images
                    .first()
                    .cloned()
                    .flatten()
                    .map(|image| image.projected(&[field]))
            };
            let Some(fields) = self.state_fields_of_target(
                &target.mutation.target,
                &target.mutation.place,
                bindings,
            )?
            else {
                // Owning indirection and indexed contents need separate
                // allocation/element identities; this slice changes neither.
                continue;
            };
            let local = bindings
                .get_mut(&target.mutation.place.root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            local.state_origins = Some(
                local
                    .state_origins
                    .take()
                    .unwrap_or_else(crate::semantic::model::CheckedStateOrigins::fresh)
                    .replace_path(&fields, image),
            );
        }
        Ok(())
    }

    /// [LIV-2] the declaration one `set` target mints, when its identifier
    /// resolved to no binding.
    ///
    /// The resolver decides this, not the checker: a target identifier with no
    /// visible binding is promoted there to an ordinary `let` declaration
    /// owned by its own `pbase`, so the question here is exactly whether this
    /// target's base owns one.
    fn declaring_commit_target(
        &self,
        target_node: NodeId,
    ) -> Result<Option<DeclarationId>, CheckStop> {
        if !self
            .tree
            .children_with(target_node, Production::Psuffix)?
            .is_empty()
        {
            return Ok(None);
        }
        let Some(pbase) = self.tree.first_child_with(target_node, Production::Pbase)? else {
            return Ok(None);
        };
        if !self.tree.children(pbase)?.is_empty() {
            return Ok(None);
        }
        Ok(self
            .declaration_at(pbase, crate::DeclarationRole::Let)
            .ok()
            .map(|declaration| declaration.id()))
    }

    /// Whether this written target is a complete binding that is already dead,
    /// which is the one shape [LIV-2] reinitializes from dead.
    fn commit_revives_binding(
        &self,
        target_node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<bool, CheckStop> {
        if !self
            .tree
            .children_with(target_node, Production::Psuffix)?
            .is_empty()
        {
            return Ok(false);
        }
        let Some(declaration) = self.complete_binding_target(target_node)? else {
            return Ok(false);
        };
        Ok(bindings.get(&declaration).is_some_and(|local| !local.live))
    }

    /// [LIV-2] condition 2 over two formed targets.
    ///
    /// Two places overlap when one is reached through the other [OWN-7]; two
    /// element writes of one place additionally overlap unless some step of
    /// their common prefix provably selects two different storages, which for
    /// two subscripts is literals with unequal values. That is the same
    /// judgment [OWN-7] states for two subscripted places, and it reads the
    /// complete path because a measured place may carry a subscript of its
    /// own: `grid[k]` and `grid[i][j]` are decided at `k` against `i`.
    fn commit_targets_overlap(&self, first: &MutationTarget, second: &MutationTarget) -> bool {
        if !places_overlap(&first.place, &second.place) {
            return false;
        }
        if first.element && second.element && first.place == second.place {
            return !paths_diverge(
                &Self::commit_target_path(&first.target),
                &Self::commit_target_path(&second.target),
            );
        }
        true
    }

    /// The complete path one element target writes below its root: the
    /// selections that reach the base, and the element the offset selects.
    ///
    /// A measured place may itself carry a subscript [MSR-1], so this is a
    /// path and never one offset: `grid[k]` and `grid[i][j]` are decided by
    /// their *first* offsets and not by their last.
    pub(in crate::semantic::check) fn commit_target_path(
        target: &CheckedSetTarget,
    ) -> Vec<PlaceStep> {
        let (mut path, offset) = match target {
            CheckedSetTarget::Place(target) => (
                target
                    .fields
                    .iter()
                    .copied()
                    .map(PlaceStep::Field)
                    .collect::<Vec<_>>(),
                None,
            ),
            CheckedSetTarget::ArrayIndex(target) => (
                target
                    .fields
                    .iter()
                    .copied()
                    .map(PlaceStep::Field)
                    .collect(),
                Some(Self::place_offset_of(&target.offset).unwrap_or(PlaceOffset::Opaque)),
            ),
            CheckedSetTarget::BufferIndex(target) => (
                target
                    .root
                    .fields
                    .iter()
                    .copied()
                    .map(PlaceStep::Field)
                    .collect(),
                Some(Self::place_offset_of(&target.offset).unwrap_or(PlaceOffset::Opaque)),
            ),
            CheckedSetTarget::Storage(target) => (target.place_path(), None),
            // A view has no field path of its own: the descriptor is a
            // direct binding [VIEW-6], so the target path is the one
            // subscript the statement wrote.
            CheckedSetTarget::SliceIndex(target) => (
                Vec::new(),
                Some(Self::place_offset_of(&target.offset).unwrap_or(PlaceOffset::Opaque)),
            ),
        };
        if let Some(offset) = offset {
            path.push(PlaceStep::Subscript(offset));
        }
        path
    }

    /// The written spelling of one `place`, rebuilt from its own tokens, for
    /// the diagnostic that must name two targets at once.
    fn place_spelling(&self, node: NodeId) -> Result<String, CheckStop> {
        let mut terminals = Vec::new();
        self.collect_terminals(node, &mut terminals)?;
        terminals.sort_unstable();
        let mut rendered = String::new();
        for terminal in terminals {
            rendered.push_str(&String::from_utf8_lossy(self.tree.token_bytes(terminal)?));
        }
        Ok(rendered)
    }

    fn collect_terminals(&self, node: NodeId, terminals: &mut Vec<usize>) -> Result<(), CheckStop> {
        terminals.extend_from_slice(self.tree.direct_token_indices(node)?);
        for child in self.tree.children(node)? {
            self.collect_terminals(*child, terminals)?;
        }
        Ok(())
    }
}
