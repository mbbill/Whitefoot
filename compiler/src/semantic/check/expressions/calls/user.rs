use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::goal::{
    EvaluatedValueOccurrence, GoalDatum, GoalExpression, GoalProjection,
};
use super::super::super::super::model::{
    CheckedElement, CheckedExpression, CheckedMode, CheckedNominalKind, CheckedResultBorrow,
    CheckedSliceOrigin, CheckedStateOrigins, CheckedType,
};
use super::super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, ResolvedPlace, SliceInfo, places_overlap, push_slice_origin,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, ResultProvenance,
    TypedExpression, borrow_result_provenance,
};

/// One formal region's binding at one call site.
///
/// [FORM-8] writes only the region parameters no parameter position
/// determines; every other formal region starts unbound and takes the least
/// region of the actual arguments at the positions naming it.
#[derive(Clone, Copy)]
pub(in crate::semantic::check) struct RegionBinding {
    /// Whether the caller wrote this region argument.
    written: bool,
    /// The substituted region: fixed for a written argument, and the least
    /// actual region observed so far for an inferred one.
    region: Option<DeclarationId>,
    /// [PROV-1] the actual this formal took at its first *store* position.
    ///
    /// A store region is invariant: two values have the same store exactly
    /// when their types name the same region, decided by exact identity. A
    /// loan region relates two positions by outlives and takes the least
    /// region observed; a store region takes the first and every later
    /// position of the same formal must name it exactly, which is the
    /// ordinary [TYPE-5] argument mismatch where it does not.
    store: Option<DeclarationId>,
}

impl RegionBinding {
    const INFERRED: Self = Self {
        written: false,
        region: None,
        store: None,
    };
}

/// What one parameter position requires of its actual argument's mode.
#[derive(Clone, Copy)]
pub(in crate::semantic::check) enum ModeExpectation {
    /// An owned value.
    Own,
    /// A borrow of this kind. `region` is `None` where [FORM-8] leaves the
    /// formal region for the actual to determine, and the actual's own region
    /// is then the substituted one.
    Borrow {
        kind: BorrowKind,
        region: Option<DeclarationId>,
    },
}

struct CallAccessClaim {
    kind: BorrowKind,
    origin: CallClaimOrigin,
}

enum CallClaimOrigin {
    Place(ResolvedPlace),
    FormalSlice,
}

impl CallClaimOrigin {
    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Place(left), Self::Place(right)) => places_overlap(left, right),
            (Self::FormalSlice, _) | (_, Self::FormalSlice) => true,
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// The single provenance-candidate parameter position of a
    /// borrow-returning callee signature under the reborrow extension: the
    /// one parameter written as a borrow of the result's kind in the
    /// result's formal region, admitted only when no other parameter
    /// mentions that formal region in its mode or type. Distinct formal
    /// regions are incomparable inside the callee [OWN-3] and storage is
    /// borrow- and region-free [STOR-5], so every borrow an accepted callee
    /// can deliver in the result region is rooted in this parameter's actual
    /// or in immutable named-const storage; the candidate's resolved place
    /// therefore covers every mutable storage the result can reach.
    ///
    /// Every other disposition forms no candidate here. Under the
    /// declaration-provenance candidate, FN-1 has already rejected the
    /// ambiguous boundary at its `rtype`, so only the const-storage
    /// disposition still reaches this call site.
    fn result_borrow_candidate(&self, signature: &FunctionSignature) -> Option<usize> {
        if !self.reborrow_extension {
            return None;
        }
        match borrow_result_provenance(
            &signature.parameters,
            signature.result_mode,
            signature.result,
        ) {
            Some(ResultProvenance::Candidate(index)) => Some(index),
            Some(
                ResultProvenance::Unjudgeable
                | ResultProvenance::ConstStorage
                | ResultProvenance::Ambiguous,
            )
            | None => None,
        }
    }

    pub(super) fn check_user_call(
        &self,
        node: NodeId,
        declaration: DeclarationId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let target = self.concrete_function_for_call(node, declaration, &function.substitution)?;
        let signature = self
            .signatures
            .get(target.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let mut region_bindings = self.call_region_arguments(node, signature)?;
        let fields = if let Some(list) = self
            .tree
            .first_child_with(node, Production::FieldinitList)?
        {
            self.tree.children_with(list, Production::Fieldinit)?
        } else {
            Vec::new()
        };
        if self
            .tree
            .first_child_with(node, Production::AtomList)?
            .is_some()
            || fields.len() != signature.parameters.len()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                Self::invalid_named_arguments(signature),
            );
        }
        let mut arguments = Vec::with_capacity(fields.len());
        let mut checked_borrows = Vec::with_capacity(fields.len());
        let mut checked_slices = Vec::with_capacity(fields.len());
        let mut argument_holders = Vec::with_capacity(fields.len());
        let mut state_origins = Vec::with_capacity(fields.len());
        let mut argument_places = Vec::with_capacity(fields.len());
        let mut argument_nodes = Vec::with_capacity(fields.len());
        let mut goal_arguments = Vec::with_capacity(fields.len());
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        let call = self.tree.path(node)?.clone();
        // Payload-free heap allocation transfers by presence at a call
        // boundary [EFF-2]; region entries are projected below.
        let mut effects = EffectSet {
            allocates_heap: signature.declared_effects.allocates_heap,
            ..EffectSet::NONE
        };
        let result_candidate = self.result_borrow_candidate(signature);
        for (ordinal, (field, parameter)) in
            fields.into_iter().zip(&signature.parameters).enumerate()
        {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(
                    SemanticRule::Gram11,
                    field,
                    Self::invalid_named_arguments(signature),
                );
            }
            let atom = self
                .tree
                .first_child_with(field, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let explicit_borrow = self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some();
            let argument = self.check_call_argument_atom(
                function,
                atom,
                bindings,
                loop_depth,
                signature.result_mode == CheckedMode::Own,
                result_candidate == Some(ordinal),
            )?;
            for access in &argument.accesses {
                for borrow in &call_scoped_borrows {
                    if places_overlap(&access.place, &borrow.place)
                        && match access.kind {
                            AccessKind::Read => borrow.kind == BorrowKind::Unique,
                            AccessKind::Write
                            | AccessKind::Move
                            | AccessKind::SharedBorrow
                            | AccessKind::UniqueBorrow => true,
                        }
                    {
                        return self.issue_node(
                            SemanticRule::Own12,
                            atom,
                            SemanticIssueKind::BorrowConflict,
                        );
                    }
                }
            }
            let expectation = self.substitute_mode(parameter.mode, signature, &region_bindings)?;
            let expected_type = self.substitute_parameter_type(
                parameter.ty,
                signature,
                &region_bindings,
                argument.expression.ty(),
            )?;
            if argument.expression.ty() != expected_type {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(expected_type)?,
                        self.checked_type_name(argument.expression.ty())?,
                    ),
                );
            }
            let (passed_borrow, expected_mode) =
                self.call_argument_borrow(expectation, &argument, atom)?;
            // [FORM-8] every inferred formal region this position names takes
            // the actual it just observed into its running least region.
            for (formal, actual) in [
                (
                    match parameter.mode {
                        CheckedMode::Own => None,
                        CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
                    },
                    match expected_mode {
                        CheckedMode::Own => None,
                        CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
                    },
                ),
                (
                    self.written_type_region(parameter.ty)?,
                    self.written_type_region(expected_type)?,
                ),
            ] {
                let (Some(formal), Some(actual)) = (formal, actual) else {
                    continue;
                };
                let index = Self::formal_region_index(signature, formal)?;
                let mut binding = *region_bindings
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !binding.written {
                    self.observe_actual_region(&mut binding, actual, atom)?;
                    // [PROV-1] a store region is invariant, so the first
                    // position that names it fixes it and every later one is
                    // substituted with that region rather than with its own
                    // actual.
                    if binding.store.is_none()
                        && self.written_store_type_region(parameter.ty)? == Some(formal)
                    {
                        binding.store = Some(actual);
                    }
                    *region_bindings
                        .get_mut(index)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)? = binding;
                }
            }
            state_origins.push(self.state_origins_of_value(&argument, bindings)?);
            argument_places.push(
                argument
                    .accesses
                    .iter()
                    .map(|access| access.place.clone())
                    .collect::<Vec<_>>(),
            );
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            goal_arguments.push(self.call_goal_argument(
                function.id,
                &call,
                ordinal,
                atom,
                expected_mode,
                expected_type,
                &argument,
                passed_borrow.as_ref(),
                bindings,
            )?);
            argument_nodes.push(self.tree.path(atom)?.clone());
            if explicit_borrow && let Some(borrow) = &argument.borrow {
                call_scoped_borrows.push(borrow.clone());
            }
            checked_borrows.push(passed_borrow);
            checked_slices.push(argument.slice.clone());
            argument_holders.push(argument.holder);
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        let actual_regions = Self::resolved_regions(&region_bindings)?;
        self.check_region_parameter_bounds(node, signature, &actual_regions)?;
        self.check_call_borrow_overlap(node, &checked_borrows, &checked_slices)?;
        self.project_call_effects(
            node,
            function,
            signature,
            &actual_regions,
            &checked_borrows,
            &checked_slices,
            &argument_holders,
            &state_origins,
            &argument_places,
            bindings,
            &mut effects,
        )?;
        let result = self.substitute_result_type(signature.result, signature, &actual_regions)?;
        let result_mode =
            self.substituted_mode(signature.result_mode, signature, &actual_regions)?;
        let slice = self.substitute_slice_result(signature, result, &checked_slices)?;
        let slice_origins = slice
            .as_ref()
            .map(|slice| slice.origins.clone())
            .unwrap_or_default();
        // Reborrow extension: a borrow-mode result with a provenance
        // candidate carries the candidate actual's complete resolved place
        // as its own claim, so binding the result creates an ordinary holder
        // over caller storage [OWN-5, OWN-6]. Creating that claim through a
        // still-usable `&uniq` parent holder suspends the parent for the
        // remainder of its life: the claim may outlive the statement inside
        // the bound result, so statement-end resumption would leave two
        // usable paths to one place.
        let result_borrow_info = result_candidate
            .and_then(|index| checked_borrows.get(index))
            .cloned()
            .flatten();
        let result_borrow = if let Some(borrow) = &result_borrow_info {
            if let Some(holder) = result_candidate
                .and_then(|index| argument_holders.get(index))
                .copied()
                .flatten()
            {
                let parent_is_unique = bindings
                    .get(&holder)
                    .and_then(|local| local.borrow.as_ref())
                    .is_some_and(|parent| parent.kind == BorrowKind::Unique);
                if parent_is_unique {
                    bindings
                        .get_mut(&holder)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .suspended = true;
                }
            }
            let root = bindings
                .get(&borrow.place.root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .binding;
            Some(CheckedResultBorrow {
                binding: root,
                fields: borrow.place.fields.clone(),
            })
        } else {
            None
        };
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                call,
                argument_nodes,
                arguments,
                goal_arguments,
                goal_regions: actual_regions,
                requirements: Vec::new(),
                result,
                slice_origins,
                result_borrow,
            },
            mode: result_mode,
            borrow: result_borrow_info,
            slice,
            holder: None,
            // A reference-returning call still yields a reference value; the
            // referent is reached only through an explicit holder [TYPE-7].
            reference_value: result_mode != CheckedMode::Own,
            effects,
            accesses: Vec::new(),
        })
    }

    /// Captures one already-checked actual's pre-transfer goal image.
    ///
    /// This runs after the actual expression has acquired all of its checked
    /// obligations and after borrow feasibility succeeds. It never rechecks or
    /// reevaluates the source expression. A borrow destination is represented
    /// by the resolved ultimate referent captured in `passed_borrow` before
    /// that transient checker metadata disappears.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::semantic::check) fn call_goal_argument(
        &self,
        caller: super::super::super::super::model::FunctionId,
        call: &crate::NodePath,
        ordinal: u32,
        atom: NodeId,
        expected_mode: CheckedMode,
        expected_type: CheckedType,
        argument: &super::super::super::TypedExpression,
        passed_borrow: Option<&BorrowInfo>,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<GoalExpression, CheckStop> {
        if expected_mode != CheckedMode::Own {
            let borrow = passed_borrow.ok_or(SemanticCompilerFailure::InvalidResolution)?;
            return self.goal_referent_image(&borrow.place, expected_type, bindings);
        }

        if self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
            .is_some()
        {
            let CheckedExpression::Constant(value) = &argument.expression else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            return Ok(GoalExpression::Datum(GoalDatum::Literal(value.clone())));
        }

        let place = self
            .tree
            .first_child_with(atom, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.call_goal_place_contains_subscript(place)? {
            return Ok(GoalExpression::Datum(GoalDatum::EvaluatedValue {
                function: caller,
                occurrence: EvaluatedValueOccurrence::CallArgument {
                    call: call.clone(),
                    argument: ordinal,
                },
                captured_type: expected_type,
                projections: Vec::new(),
                ty: expected_type,
            }));
        }
        let (image, holder_pending) = self.call_goal_place_inner(place, bindings)?;
        if holder_pending || image.ty() != expected_type {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(image)
    }

    /// A place may nest another place under a `deref` pbase. Search the whole
    /// source place, not only its outer suffix list, so a future admitted
    /// `deref(boxes[i])` actual receives the same ephemeral treatment and is
    /// never misidentified as a rereadable place.
    fn call_goal_place_contains_subscript(&self, place: NodeId) -> Result<bool, CheckStop> {
        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        if self.last_subscript(&suffixes)?.is_some() {
            return Ok(true);
        }
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let Some(nested) = self.tree.first_child_with(pbase, Production::Place)? else {
            return Ok(false);
        };
        self.call_goal_place_contains_subscript(nested)
    }

    /// Forms a caller-visible referent datum. A root that is itself one of the
    /// caller's borrow parameters remains opaque and therefore retains one
    /// `Deref`; a local borrow/reborrow has already resolved through its holder
    /// to an own root and adds no such projection.
    fn goal_referent_image(
        &self,
        place: &ResolvedPlace,
        ty: CheckedType,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<GoalExpression, CheckStop> {
        let mut projections = Vec::new();
        if bindings
            .get(&place.root)
            .is_some_and(|binding| binding.mode != CheckedMode::Own)
        {
            projections.push(GoalProjection::Deref);
        }
        projections.extend(place.fields.iter().copied().map(GoalProjection::Field));
        let datum = if self.constants.contains_key(&place.root) {
            GoalDatum::NamedConst {
                declaration: place.root,
                projections,
                ty,
            }
        } else {
            let binding = bindings
                .get(&place.root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            GoalDatum::Place {
                root: binding.binding,
                projections,
                ty,
            }
        };
        Ok(GoalExpression::Datum(datum))
    }

    /// Resolves one non-indexed own actual to its concrete caller datum while
    /// preserving own-box dereference and field order. Dereferencing a borrow
    /// holder consumes the holder boundary exactly once and leaves the
    /// ultimate referent image produced above.
    fn call_goal_place_inner(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(GoalExpression, bool), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (mut expression, holder_pending) = if self
            .has_fixed(pbase, crate::FixedTerminal::Deref)?
        {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let (nested, nested_holder_pending) = self.call_goal_place_inner(nested, bindings)?;
            if nested_holder_pending {
                (nested, false)
            } else {
                let CheckedType::Nominal(nominal) = nested.ty() else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let CheckedNominalKind::Box { referent } = self.nominal(nominal)?.kind else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                (
                    nested
                        .with_projection(GoalProjection::Deref, referent)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                    false,
                )
            }
        } else {
            let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
            let ResolvedTarget::Source { declaration, class } = usage.target() else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            match class {
                DeclarationClass::Value => {
                    let local = bindings
                        .get(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if let Some(borrow) = &local.borrow {
                        (
                            self.goal_referent_image(&borrow.place, local.ty, bindings)?,
                            true,
                        )
                    } else {
                        (
                            GoalExpression::Datum(GoalDatum::Place {
                                root: local.binding,
                                projections: Vec::new(),
                                ty: local.ty,
                            }),
                            false,
                        )
                    }
                }
                DeclarationClass::NamedConst => {
                    let constant = self
                        .constants
                        .get(&declaration)
                        .copied()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    (
                        GoalExpression::Datum(GoalDatum::NamedConst {
                            declaration,
                            projections: Vec::new(),
                            ty: self.constant(constant)?.ty,
                        }),
                        false,
                    )
                }
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            }
        };

        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        if holder_pending && !suffixes.is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if !suffixes.is_empty() {
            let (fields, final_ty) = self.resolve_struct_path(&suffixes, expression.ty())?;
            for field in fields {
                expression = expression
                    .with_projection(GoalProjection::Field(field), final_ty)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            }
        }
        Ok((expression, holder_pending))
    }

    /// The written type, const, and region arguments of a user-generic call.
    ///
    /// [DIAG-1] selects the cited rule by the callee's class rather than by
    /// the kind of argument problem, and for a user-generic call that rule is
    /// FN-2 — "a missing, wrong-kind, wrong-count, or wrong-domain argument".
    /// TYPE-5 governs whether an argument's *type* matches its parameter, one
    /// step later and at the offending atom; it does not own the argument list
    /// itself.
    /// Whether one formal region occupies a parameter position of this
    /// callable: a `param` mode or a `slice` parameter type [FORM-8].
    fn formal_region_is_determined(
        &self,
        signature: &FunctionSignature,
        formal: DeclarationId,
    ) -> Result<bool, CheckStop> {
        for parameter in &signature.parameters {
            if matches!(
                parameter.mode,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) if region == formal
            ) || self.written_type_region(parameter.ty)? == Some(formal)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The one region a type writes [FORM-8, PROV-1].
    ///
    /// A view names its loan region and each store-branded type names its
    /// store region; every other type names none. This is the region axis of
    /// substitution: a parameter whose type names a formal region determines
    /// that region from its actual, exactly as a borrow mode does, so the
    /// caller does not write it.
    ///
    /// [BLK-1]'s one-level lift puts a second place a store region can be
    /// written: a frame-resident run of store-backed runs names its store in
    /// its element position and nowhere else, and that region is determined by
    /// the actual exactly as a top-level one is. Where both levels name a
    /// region — `Vector<'s, Vector<'t, u8>>` — this reports the outer one
    /// alone, so the inner is not substituted and the position is the ordinary
    /// [TYPE-5] region mismatch: fail-closed, and an explicit gap rather than a
    /// silent second binding.
    /// [S20] a source nominal instance carrying exactly one region argument
    /// names that region on the same ground: its region parameter is a
    /// component of its type name [TYPE-2, PROV-1], and a parameter of that
    /// type therefore determines the region from its actual. An instance
    /// carrying two or more region arguments names none, which is the same
    /// fail-closed reading `Vector<'s, Vector<'t, u8>>` already takes.
    fn written_type_region(&self, ty: CheckedType) -> Result<Option<DeclarationId>, CheckStop> {
        if let CheckedType::Nominal(id) = ty {
            return Ok(match self.nominal_region_axis(id)? {
                Some([(_, region)]) => Some(*region),
                _ => None,
            });
        }
        Ok(Self::written_container_type_region(ty))
    }

    /// The one *store* region a type writes [PROV-1]: the same relation minus
    /// a view's loan region, which names no store and relates two positions
    /// by outlives rather than by identity.
    fn written_store_type_region(
        &self,
        ty: CheckedType,
    ) -> Result<Option<DeclarationId>, CheckStop> {
        if matches!(ty, CheckedType::Slice { .. }) {
            return Ok(None);
        }
        self.written_type_region(ty)
    }

    const fn written_container_type_region(ty: CheckedType) -> Option<DeclarationId> {
        match ty {
            CheckedType::Slice { region, .. }
            | CheckedType::Vector { region, .. }
            | CheckedType::Heap { region }
            | CheckedType::Extent { region, .. } => Some(region),
            CheckedType::FixedVector {
                element: CheckedElement::Vector { region, .. },
                ..
            } => Some(region),
            _ => None,
        }
    }

    /// The same type with its written region replaced [FN-2, OWN-12].
    ///
    /// A run's release class is a function of its region's declaration alone
    /// [PROV-6], and [PROV-6]'s own instantiation check makes the actual's
    /// store class equal the formal bound's, so the class the formal carried
    /// is the class the actual has and the substitution preserves it.
    /// The same type with its written region replaced, over a source nominal
    /// [S20].
    ///
    /// A nominal instance is a nominal-arena identity rather than a structure,
    /// so the substituted type is the instance of the same declaration with
    /// the same type and const arguments and this region — which is exactly
    /// the actual's own instance when the two agree, and no instance at all
    /// when they do not. Reading it off the actual mints nothing during
    /// checking and rejects a genuine mismatch through the ordinary [TYPE-5]
    /// equality below.
    fn with_nominal_type_region(
        &self,
        ty: CheckedType,
        region: DeclarationId,
        actual: CheckedType,
    ) -> Result<CheckedType, CheckStop> {
        let (CheckedType::Nominal(formal_id), CheckedType::Nominal(actual_id)) = (ty, actual)
        else {
            return Ok(ty);
        };
        let (Some((_, formal)), Some((_, actual))) = (
            self.source_nominal_instance_entry(formal_id)?,
            self.source_nominal_instance_entry(actual_id)?,
        ) else {
            return Ok(ty);
        };
        let substituted = formal
            .region_arguments()
            .iter()
            .map(|(parameter, _)| (*parameter, region))
            .collect::<Vec<_>>();
        if substituted != actual.region_arguments() {
            return Ok(ty);
        }
        // The two instances must be one representation as well as one
        // declaration: a formal region whose store class differs from the
        // actual's gives its runs a different release action [PROV-6], and
        // that difference is a [TYPE-5] mismatch at this argument rather than
        // a substitution.
        if !self.nominals_differ_only_in_region(actual_id, formal_id)? {
            return Ok(ty);
        }
        Ok(CheckedType::Nominal(actual_id))
    }

    const fn with_type_region(ty: CheckedType, region: DeclarationId) -> CheckedType {
        match ty {
            CheckedType::Slice { element, .. } => CheckedType::Slice { region, element },
            CheckedType::Vector {
                element, release, ..
            } => CheckedType::Vector {
                region,
                element,
                release,
            },
            CheckedType::Heap { .. } => CheckedType::Heap { region },
            CheckedType::Extent { bytes, align, .. } => CheckedType::Extent {
                region,
                bytes,
                align,
            },
            CheckedType::FixedVector {
                element:
                    CheckedElement::Vector {
                        element, release, ..
                    },
                length,
            } => CheckedType::FixedVector {
                element: CheckedElement::Vector {
                    region,
                    element,
                    release,
                },
                length,
            },
            other => other,
        }
    }

    /// The formal regions a caller writes: exactly those the callee's own
    /// parameter positions leave undetermined [FORM-8].
    fn caller_chosen_regions(
        &self,
        signature: &FunctionSignature,
    ) -> Result<Vec<usize>, CheckStop> {
        let mut chosen = Vec::new();
        for index in 0..signature.written_regions {
            let Some(formal) = signature.region_parameters.get(index) else {
                continue;
            };
            if !self.formal_region_is_determined(signature, *formal)? {
                chosen.push(index);
            }
        }
        Ok(chosen)
    }

    /// Binds each formal region of the callee for one call.
    ///
    /// [FORM-8] writes exactly the region parameters no parameter position
    /// determines; every other formal region — a written one a parameter
    /// position names, and every region a parameter position leaves unwritten
    /// — is determined by the actual arguments at those positions and is
    /// filled in as they are checked.
    ///
    /// [DIAG-1] selects the cited rule by the callee's class rather than by
    /// the kind of argument problem, and for a user-generic call that rule is
    /// FN-2 — "a missing, wrong-kind, wrong-count, or wrong-domain argument".
    /// TYPE-5 governs whether an argument's *type* matches its parameter, one
    /// step later and at the offending atom; it does not own the argument list
    /// itself.
    pub(in crate::semantic::check) fn call_region_arguments(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
    ) -> Result<Vec<RegionBinding>, CheckStop> {
        let generic_count = signature.substitution.len();
        let chosen = self.caller_chosen_regions(signature)?;
        let written = match self.tree.first_child_with(node, Production::Targs)? {
            Some(targs) => {
                let arguments = self.tree.children_with(targs, Production::Targ)?;
                if arguments.len() < generic_count {
                    return self.issue_node(
                        SemanticRule::Fn2,
                        node,
                        SemanticIssueKind::type_mismatch(
                            crate::semantic::written_count(
                                generic_count
                                    .checked_add(chosen.len())
                                    .ok_or(SemanticCompilerFailure::CounterOverflow)?,
                                "type and region argument",
                            ),
                            crate::semantic::written_count(arguments.len(), "argument"),
                        ),
                    );
                }
                arguments
                    .into_iter()
                    .skip(generic_count)
                    .collect::<Vec<_>>()
            }
            None => {
                if generic_count > 0 {
                    return self.issue_node(
                        SemanticRule::Fn2,
                        node,
                        SemanticIssueKind::type_mismatch(
                            crate::semantic::written_count(
                                generic_count
                                    .checked_add(chosen.len())
                                    .ok_or(SemanticCompilerFailure::CounterOverflow)?,
                                "type and region argument",
                            ),
                            "no type-argument list",
                        ),
                    );
                }
                Vec::new()
            }
        };
        if written.len() != chosen.len() {
            // A member that is not a bare REGIONID makes this an [FN-2]
            // arity fault over the type and const arguments; only a list
            // whose type part is right and whose region part is wrong is
            // [FORM-8]'s.
            let mut regional = true;
            for argument in &written {
                if self
                    .tree
                    .first_child_with(*argument, Production::Type)?
                    .is_some()
                    || self
                        .tree
                        .first_child_with(*argument, Production::Const)?
                        .is_some()
                {
                    regional = false;
                    break;
                }
            }
            if !regional {
                return self.issue_node(
                    SemanticRule::Fn2,
                    node,
                    SemanticIssueKind::type_mismatch(
                        crate::semantic::written_count(
                            generic_count
                                .checked_add(chosen.len())
                                .ok_or(SemanticCompilerFailure::CounterOverflow)?,
                            "type and region argument",
                        ),
                        crate::semantic::written_count(
                            generic_count
                                .checked_add(written.len())
                                .ok_or(SemanticCompilerFailure::CounterOverflow)?,
                            "argument",
                        ),
                    ),
                );
            }
            return self.issue_node(
                SemanticRule::Form8,
                node,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "write exactly the callee's region parameters that occur in \
no parameter type, in their declared order; every other region argument is determined by this \
call's own arguments and is not written",
                },
            );
        }
        let slots = chosen;
        let mut bindings = vec![RegionBinding::INFERRED; signature.region_parameters.len()];
        for (slot, argument) in slots.into_iter().zip(written) {
            let usage = self.use_at(argument, LexicalUseRole::TypeArgumentRegion)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return self.issue_node(
                    SemanticRule::Fn2,
                    argument,
                    SemanticIssueKind::type_mismatch(
                        "a region argument in this position",
                        "an argument that does not name a region",
                    ),
                );
            };
            let Some(binding) = bindings.get_mut(slot) else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            *binding = RegionBinding {
                written: true,
                region: Some(declaration),
                store: Some(declaration),
            };
        }
        Ok(bindings)
    }

    /// [PROV-6, S37] the region axis of the bound check, at one call.
    ///
    /// A written region parameter may carry a linearity bound, and that bound
    /// is a claim about the *store* the region names: `affine` is a bump
    /// extent, `linear` a general store [PROV-1]. The region argument this
    /// call binds to it is the one every other region judgment uses, whether
    /// the caller wrote it in the `::` list or a parameter position determined
    /// it, so the check is over the resolved binding and never over a
    /// spelling. Elided formal regions [FORM-8] carry no bound and are not
    /// written, so only the written prefix is read.
    fn check_region_parameter_bounds(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Result<(), CheckStop> {
        for (index, formal) in signature
            .region_parameters
            .iter()
            .take(signature.written_regions)
            .enumerate()
        {
            let record = self
                .resolved
                .declarations()
                .iter()
                .find(|candidate| candidate.id() == *formal)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let Some(formal_node) = self.tree.node_with_path(record.origin().node()) else {
                continue;
            };
            let Some(bound) = self.written_linearity_bound(formal_node)? else {
                continue;
            };
            let spelling = record.spelling().to_owned();
            let actual = *actual_regions
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.check_region_linearity_bound(&spelling, bound, actual, node)?;
        }
        Ok(())
    }

    /// The index of one formal region in the callee's formal-region list.
    fn formal_region_index(
        signature: &FunctionSignature,
        formal: DeclarationId,
    ) -> Result<usize, CheckStop> {
        signature
            .region_parameters
            .iter()
            .position(|region| *region == formal)
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    /// Records one actual region observed at a position naming an inferred
    /// formal region, keeping the least region every observation outlives
    /// [OWN-3, OWN-4]. Two incomparable actual regions leave the formal with
    /// no legal substitution and reject exactly as an unsatisfiable written
    /// region argument does.
    fn observe_actual_region(
        &self,
        binding: &mut RegionBinding,
        actual: DeclarationId,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let Some(current) = binding.region else {
            binding.region = Some(actual);
            return Ok(());
        };
        if self.region_outlives(actual, current)? {
            return Ok(());
        }
        if self.region_outlives(current, actual)? {
            binding.region = Some(actual);
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Own4,
            node,
            SemanticIssueKind::InvalidBorrowLifetime {
                region: self.region_phrase(current)?,
                binder: self.region_phrase(actual)?,
                mechanical_fix: format!(
                    "this parameter position and an earlier one share one region, but {} and {} \
are incomparable; pass borrows whose regions are nested, or give the parameters distinct regions",
                    self.region_phrase(actual)?,
                    self.region_phrase(current)?
                ),
            },
        )
    }

    /// Every formal region's substituted actual after the argument list is
    /// checked.
    fn resolved_regions(bindings: &[RegionBinding]) -> Result<Vec<DeclarationId>, CheckStop> {
        bindings
            .iter()
            .map(|binding| {
                binding
                    .region
                    .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
            })
            .collect()
    }

    /// What one formal mode requires of its actual, after region binding.
    fn substitute_mode(
        &self,
        mode: CheckedMode,
        signature: &FunctionSignature,
        bindings: &[RegionBinding],
    ) -> Result<ModeExpectation, CheckStop> {
        let (kind, formal) = match mode {
            CheckedMode::Own => return Ok(ModeExpectation::Own),
            CheckedMode::Shared(region) => (BorrowKind::Shared, region),
            CheckedMode::Unique(region) => (BorrowKind::Unique, region),
        };
        let index = Self::formal_region_index(signature, formal)?;
        let binding = bindings
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(ModeExpectation::Borrow {
            kind,
            region: if binding.written {
                binding.region
            } else {
                None
            },
        })
    }

    /// One formal mode substituted with regions already resolved.
    fn substituted_mode(
        &self,
        mode: CheckedMode,
        signature: &FunctionSignature,
        regions: &[DeclarationId],
    ) -> Result<CheckedMode, CheckStop> {
        let (kind, formal) = match mode {
            CheckedMode::Own => return Ok(CheckedMode::Own),
            CheckedMode::Shared(region) => (BorrowKind::Shared, region),
            CheckedMode::Unique(region) => (BorrowKind::Unique, region),
        };
        let index = Self::formal_region_index(signature, formal)?;
        let actual = *regions
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(match kind {
            BorrowKind::Shared => CheckedMode::Shared(actual),
            BorrowKind::Unique => CheckedMode::Unique(actual),
        })
    }

    /// Checks one actual argument against its parameter's mode expectation and
    /// returns the loan it passes together with the exact mode that position
    /// carries after region binding.
    pub(in crate::semantic::check) fn call_argument_borrow(
        &self,
        expectation: ModeExpectation,
        argument: &TypedExpression,
        atom: NodeId,
    ) -> Result<(Option<BorrowInfo>, CheckedMode), CheckStop> {
        let (kind, region) = match expectation {
            ModeExpectation::Own => {
                let borrow = self.borrow_for_destination(CheckedMode::Own, argument, atom)?;
                return Ok((borrow, CheckedMode::Own));
            }
            ModeExpectation::Borrow { kind, region } => (kind, region),
        };
        if let Some(region) = region {
            let mode = match kind {
                BorrowKind::Shared => CheckedMode::Shared(region),
                BorrowKind::Unique => CheckedMode::Unique(region),
            };
            let borrow = self.borrow_for_destination(mode, argument, atom)?;
            return Ok((borrow, mode));
        }
        // [FORM-8] the parameter leaves its region to this actual, so the
        // position constrains the borrow kind only and the actual's own
        // region is the substituted one. No [OWN-4] order is required here
        // because the loan is not being shortened to a written region.
        let Some(borrow) = argument.borrow.clone() else {
            return self.issue_node(
                SemanticRule::Type5,
                atom,
                SemanticIssueKind::type_mismatch(
                    match kind {
                        BorrowKind::Shared => "a shared borrow".to_owned(),
                        BorrowKind::Unique => "a `uniq` borrow".to_owned(),
                    },
                    self.checked_value_name(argument.mode, argument.expression.ty())?,
                ),
            );
        };
        if borrow.kind != kind {
            return self.issue_node(
                SemanticRule::Type5,
                atom,
                SemanticIssueKind::type_mismatch(
                    match kind {
                        BorrowKind::Shared => "a shared borrow".to_owned(),
                        BorrowKind::Unique => "a `uniq` borrow".to_owned(),
                    },
                    self.checked_value_name(argument.mode, argument.expression.ty())?,
                ),
            );
        }
        let mode = match kind {
            BorrowKind::Shared => CheckedMode::Shared(borrow.region),
            BorrowKind::Unique => CheckedMode::Unique(borrow.region),
        };
        Ok((Some(borrow), mode))
    }

    /// One formal parameter type with its formal region substituted.
    ///
    /// A written region argument fixes the region; a region [FORM-8] leaves
    /// for the actual to determine takes the actual slice's own region, so
    /// this position constrains the element type alone.
    fn substitute_parameter_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        bindings: &[RegionBinding],
        actual: CheckedType,
    ) -> Result<CheckedType, CheckStop> {
        let Some(formal) = self.written_type_region(ty)? else {
            return Ok(ty);
        };
        let Ok(index) = Self::formal_region_index(signature, formal) else {
            // A region this declaration does not parameterize — the entry
            // heap's store region [PROV-1] is the one such region a parameter
            // type can name — is not substituted at a call.
            return Ok(ty);
        };
        let binding = bindings
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let region = match (binding.written, binding.region) {
            (true, Some(region)) => region,
            // [PROV-1] a store region already fixed by an earlier position of
            // this call is the substitution here too, so a second argument
            // naming a second store is the ordinary [TYPE-5] mismatch and not
            // a second binding.
            (false, _)
                if binding.store.is_some()
                    && self.written_store_type_region(ty)? == Some(formal) =>
            {
                binding
                    .store
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
            }
            (false, _) => match self.written_type_region(actual)? {
                Some(region) => region,
                None => return Ok(ty),
            },
            _ => return Ok(ty),
        };
        if matches!(ty, CheckedType::Nominal(_)) {
            return self.with_nominal_type_region(ty, region, actual);
        }
        Ok(Self::with_type_region(ty, region))
    }

    /// One formal type with every formal region already resolved.
    fn substitute_result_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        let Some(formal) = self.written_type_region(ty)? else {
            return Ok(ty);
        };
        let Ok(index) = Self::formal_region_index(signature, formal) else {
            return Ok(ty);
        };
        // [S20] a nominal result keeps the declaration's own region: no
        // instance of it at the actual region need exist at this call, and the
        // caller's next transfer substitutes it from the actual it holds.
        if matches!(ty, CheckedType::Nominal(_)) {
            return Ok(ty);
        }
        Ok(Self::with_type_region(
            ty,
            *actual_regions
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
        ))
    }

    fn substitute_slice_result(
        &self,
        signature: &FunctionSignature,
        result: CheckedType,
        arguments: &[Option<SliceInfo>],
    ) -> Result<Option<SliceInfo>, CheckStop> {
        let CheckedType::Slice { region, .. } = result else {
            return Ok(None);
        };
        let mut origins = Vec::new();
        for origin in &signature.slice_return_ceiling {
            match origin {
                CheckedSliceOrigin::ImmutableConst => {
                    push_slice_origin(&mut origins, CheckedSliceOrigin::ImmutableConst);
                }
                CheckedSliceOrigin::FormalSlice { parameter, .. } => {
                    let index = signature
                        .parameters
                        .iter()
                        .position(|candidate| candidate.declaration == *parameter)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let actual = arguments
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    for actual_origin in &actual.origins {
                        push_slice_origin(&mut origins, actual_origin.clone());
                    }
                }
                CheckedSliceOrigin::SourcePlace { .. } => {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
        }
        Ok(Some(SliceInfo { region, origins }))
    }

    pub(super) fn check_call_borrow_overlap(
        &self,
        node: NodeId,
        borrows: &[Option<BorrowInfo>],
        slices: &[Option<SliceInfo>],
    ) -> Result<(), CheckStop> {
        let claims = borrows
            .iter()
            .zip(slices)
            .map(|(borrow, slice)| Self::call_claims(borrow.as_ref(), slice.as_ref()))
            .collect::<Vec<_>>();
        for (index, left_claims) in claims.iter().enumerate() {
            for right_claims in claims.iter().skip(index + 1) {
                if left_claims.iter().any(|left| {
                    right_claims.iter().any(|right| {
                        (left.kind == BorrowKind::Unique || right.kind == BorrowKind::Unique)
                            && left.origin.overlaps(&right.origin)
                    })
                }) {
                    return self.issue_node(
                        SemanticRule::Own12,
                        node,
                        SemanticIssueKind::BorrowConflict,
                    );
                }
            }
        }
        Ok(())
    }

    fn call_claims(borrow: Option<&BorrowInfo>, slice: Option<&SliceInfo>) -> Vec<CallAccessClaim> {
        let mut claims = Vec::new();
        if let Some(borrow) = borrow {
            claims.push(CallAccessClaim {
                kind: borrow.kind,
                origin: CallClaimOrigin::Place(borrow.place.clone()),
            });
        }
        if let Some(slice) = slice {
            for origin in &slice.origins {
                let origin = match origin {
                    CheckedSliceOrigin::SourcePlace { root, fields, .. } => {
                        CallClaimOrigin::Place(ResolvedPlace {
                            root: *root,
                            fields: fields.clone(),
                        })
                    }
                    CheckedSliceOrigin::FormalSlice { .. } => CallClaimOrigin::FormalSlice,
                    CheckedSliceOrigin::ImmutableConst => continue,
                };
                claims.push(CallAccessClaim {
                    kind: BorrowKind::Shared,
                    origin,
                });
            }
        }
        claims
    }

    #[allow(clippy::too_many_arguments)]
    fn project_call_effects(
        &self,
        node: NodeId,
        caller: &FunctionSignature,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
        borrows: &[Option<BorrowInfo>],
        slices: &[Option<SliceInfo>],
        holders: &[Option<DeclarationId>],
        state_origins: &[Option<CheckedStateOrigins>],
        argument_places: &[Vec<ResolvedPlace>],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        for formal_region in &signature.declared_effects.allocates_arenas {
            let index = signature
                .region_parameters
                .iter()
                .position(|region| region == formal_region)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            effects.add_arena_allocation(
                *actual_regions
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            );
        }
        for (access, declared) in [
            (AccessKind::Read, &signature.declared_effects.reads),
            (AccessKind::Write, &signature.declared_effects.writes),
        ] {
            for formal in declared {
                let index = signature
                    .parameters
                    .iter()
                    .position(|parameter| parameter.declaration == formal.root)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let parameter = signature
                    .parameters
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut actual_paths = Vec::new();

                if parameter.mode != CheckedMode::Own {
                    let borrow = borrows
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let mut place = borrow.place.clone();
                    place.fields.extend_from_slice(&formal.fields);
                    self.check_loan_access(
                        bindings,
                        holders.get(index).copied().flatten(),
                        &place,
                        access,
                        node,
                    )?;
                    for path in self.effect_paths_for_place(&place, bindings)? {
                        actual_paths.push(path);
                    }
                } else if matches!(parameter.ty, CheckedType::Slice { .. }) {
                    let slice = slices
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    for (mut place, _) in slice.source_places() {
                        place.fields.extend_from_slice(&formal.fields);
                        self.check_loan_access(
                            bindings,
                            holders.get(index).copied().flatten(),
                            &place,
                            access,
                            node,
                        )?;
                    }
                    for mut place in slice.effect_places() {
                        place.fields.extend_from_slice(&formal.fields);
                        actual_paths.extend(self.effect_paths_for_place(&place, bindings)?);
                    }
                }

                for place in argument_places.get(index).into_iter().flatten() {
                    let mut path = self.state_path(place, bindings)?;
                    path.fields.extend_from_slice(&formal.fields);
                    actual_paths.push(path);
                }
                if let Some(origins) = state_origins.get(index).and_then(Option::as_ref) {
                    if origins.unknown && !self.deriving_result_state_origin.get() {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                    for origin in origins.clone().projected(&formal.fields).formals {
                        actual_paths.push(origin.source);
                    }
                }

                for path in actual_paths {
                    if !caller
                        .parameters
                        .iter()
                        .any(|parameter| parameter.declaration == path.root)
                    {
                        continue;
                    }
                    match access {
                        AccessKind::Read => effects.add_read(path),
                        AccessKind::Write => effects.add_write(path),
                        _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                    }
                }
            }
        }
        Ok(())
    }
}
