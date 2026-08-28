use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::goal::{GoalDatum, GoalExpression, GoalProjection};
use super::super::super::super::model::{
    CheckedExpression, CheckedMode, CheckedNominalKind, CheckedResultBorrow, CheckedSliceOrigin,
    CheckedStateOrigins, CheckedType,
};
use super::super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, ResolvedPlace, SliceInfo, places_overlap, push_slice_origin,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, ResultProvenance,
    TypedExpression, borrow_result_provenance,
};

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
        let actual_regions = self.call_region_arguments(node, signature)?;
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
        // Payload-free allocation and trap capabilities transfer by presence
        // at a call boundary [EFF-2]; region entries are projected below.
        let mut effects = EffectSet {
            allocates_heap: signature.declared_effects.allocates_heap,
            traps: signature.declared_effects.traps,
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
            let expected_mode = self.substitute_mode(parameter.mode, signature, &actual_regions)?;
            let expected_type =
                self.substitute_parameter_type(parameter.ty, signature, &actual_regions)?;
            if argument.expression.ty() != expected_type {
                return self.issue_node(SemanticRule::Type5, atom, SemanticIssueKind::type_mismatch(self.checked_type_name(expected_type)?, self.checked_type_name(argument.expression.ty())?));
            }
            let passed_borrow = self.borrow_for_destination(expected_mode, &argument, atom)?;
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
        let result =
            self.substitute_parameter_type(signature.result, signature, &actual_regions)?;
        let result_mode =
            self.substitute_mode(signature.result_mode, signature, &actual_regions)?;
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
    fn call_goal_argument(
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
            return Ok(GoalExpression::Datum(GoalDatum::EphemeralActual {
                caller,
                call: call.clone(),
                argument: ordinal,
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
    pub(in crate::semantic::check) fn call_region_arguments(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            if signature.region_parameters.is_empty() {
                return Ok(Vec::new());
            }
            return self.issue_node(SemanticRule::Fn2, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(signature.region_parameters.len(), "region argument"), "no type-argument list"));
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let generic_count = signature.substitution.len();
        let expected = generic_count
            .checked_add(signature.region_parameters.len())
            .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        if arguments.len() != expected {
            return self.issue_node(SemanticRule::Fn2, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(expected, "type and region argument"), crate::semantic::written_count(arguments.len(), "argument")));
        }
        arguments
            .into_iter()
            .skip(generic_count)
            .map(|argument| {
                let usage = self.use_at(argument, LexicalUseRole::TypeArgumentRegion)?;
                match usage.target() {
                    ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::Region,
                    } => Ok(declaration),
                    _ => self.issue_node(
                        SemanticRule::Fn2,
                        argument,
                        SemanticIssueKind::type_mismatch("a region argument in this position", "an argument that does not name a region"),
                    ),
                }
            })
            .collect()
    }

    fn substitute_mode(
        &self,
        mode: CheckedMode,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Result<CheckedMode, CheckStop> {
        let (kind, formal) = match mode {
            CheckedMode::Own => return Ok(CheckedMode::Own),
            CheckedMode::Shared(region) => (BorrowKind::Shared, region),
            CheckedMode::Unique(region) => (BorrowKind::Unique, region),
        };
        let index = signature
            .region_parameters
            .iter()
            .position(|region| *region == formal)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let actual = *actual_regions
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(match kind {
            BorrowKind::Shared => CheckedMode::Shared(actual),
            BorrowKind::Unique => CheckedMode::Unique(actual),
        })
    }

    fn substitute_parameter_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        let CheckedType::Slice { region, element } = ty else {
            return Ok(ty);
        };
        let index = signature
            .region_parameters
            .iter()
            .position(|formal| *formal == region)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(CheckedType::Slice {
            region: *actual_regions
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            element,
        })
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
