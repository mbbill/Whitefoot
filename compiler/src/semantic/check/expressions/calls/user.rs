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
    CheckedExpression, CheckedMode, CheckedNominalKind, CheckedResultBorrow,
    CheckedResultStateOrigin, CheckedSliceOrigin, CheckedStateOrigins, CheckedType, LoanStrength,
};
use super::super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, ResolvedPlace, SliceInfo, TemporaryLoan, places_overlap,
    push_slice_origin,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, ResultProvenance,
    TypedExpression, borrow_result_provenance,
};

/// One formal region's binding at one call site.
///
/// [FORM-8] writes only the region parameters no parameter position
/// determines. Invariant input positions fix a brand; only formals without
/// an invariant input take the least region of their loan actuals.
#[derive(Clone, Copy)]
pub(in crate::semantic::check) struct RegionBinding {
    /// Whether the caller wrote this region argument.
    written: bool,
    /// The explicit argument or the least actual of a loan-only formal.
    /// An invariant input instead fixes `store` and is never shortened.
    region: Option<DeclarationId>,
    /// [TYPE-2, PROV-1] the first invariant nominal or store brand.
    ///
    /// This includes phantom nominal parameters: equality is decided by
    /// the exact region arguments of the type, not its fields. A
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

/// One access an argument position claims for the duration of the call: the
/// resolved place it reaches and the strength it reaches it with [OWN-12].
///
/// A view argument claims the storage its origin names. Where that origin is
/// the enclosing declaration's own view parameter, the place is that
/// parameter's binding: inside the callee a formal view's region is a region
/// of its own and is incomparable with every other formal region [OWN-3,
/// FORM-8], so the storage it reaches is exactly what that one binding
/// reaches. A concrete overlap with another argument is decided where the
/// view was formed over a written place, which is the caller that supplied
/// it — the origin ceiling carries each formal origin down to that caller.
struct CallAccessClaim {
    kind: BorrowKind,
    place: ResolvedPlace,
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

    /// FN-1 confines a mutable result to its unique candidate; const storage
    /// cannot supply it. With the same complete type and no same-typed proper
    /// subplace, only the entire candidate can be returned. This declaration
    /// judgment changes neither the loan ceiling nor an inexact actual.
    pub(in crate::semantic::check) fn whole_result_borrow_candidate(
        &self,
        signature: &FunctionSignature,
    ) -> Result<Option<usize>, CheckStop> {
        if !matches!(signature.result_mode, CheckedMode::Unique(_)) {
            return Ok(None);
        }
        let Some(candidate) = self.result_borrow_candidate(signature) else {
            return Ok(None);
        };
        if signature.parameters[candidate].ty != signature.result
            || !self.has_no_same_typed_subplace(signature.result)?
        {
            return Ok(None);
        }
        Ok(Some(candidate))
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
        let mut loan_observations = Vec::new();
        let mut parameter_atoms = Vec::new();
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
        let mut call_scoped_borrows: Vec<TemporaryLoan> = Vec::new();
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
            self.check_call_argument_loans(
                bindings,
                &argument,
                explicit_borrow,
                &call_scoped_borrows,
                atom,
            )?;
            let expectation = self.substitute_mode(parameter.mode, signature, &region_bindings)?;
            let type_regions =
                self.match_type_regions(&parameter.region_shape, argument.expression.ty())?;
            for (position, actual) in &type_regions {
                let Some(index) = Self::formal_region_index(signature, position.formal) else {
                    continue;
                };
                if position.invariant {
                    // The first brand wins; exact whole-type comparison
                    // below checks every repeated occurrence, including
                    // multiple positions inside this same operand.
                    region_bindings[index].store.get_or_insert(*actual);
                } else {
                    loan_observations.push((index, *actual, atom));
                }
            }
            let expected_type = self.substitute_parameter_type(
                parameter.ty,
                signature,
                &region_bindings,
                &type_regions,
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
            parameter_atoms.push(atom);
            if let (
                CheckedMode::Shared(formal) | CheckedMode::Unique(formal),
                CheckedMode::Shared(actual) | CheckedMode::Unique(actual),
            ) = (parameter.mode, expected_mode)
                && let Some(index) = Self::formal_region_index(signature, formal)
            {
                loan_observations.push((index, actual, atom));
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
                call_scoped_borrows.push(TemporaryLoan::new(borrow.clone(), &argument));
            }
            checked_borrows.push(passed_borrow);
            checked_slices.push(argument.slice.clone());
            argument_holders.push(argument.holder);
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        let actual_regions = self.resolved_regions(&mut region_bindings, &loan_observations)?;
        for ((parameter, actual), atom) in signature
            .parameters
            .iter()
            .zip(&arguments)
            .zip(&parameter_atoms)
        {
            let expected = self.substitute_result_type(parameter.ty, signature, &actual_regions)?;
            if actual.ty() != expected {
                return self.issue_node(
                    SemanticRule::Type5,
                    *atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(expected)?,
                        self.checked_type_name(actual.ty())?,
                    ),
                );
            }
        }
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
        // [FN-1] all components read one entry snapshot. Effects were projected
        // above; the returned value keeps this image after referents change.
        let result_state = self
            .result_state_origins
            .borrow()
            .get(target.0 as usize)
            .cloned()
            .unwrap_or(CheckedResultStateOrigin::Unknown);
        let result_origins = self
            .type_carries_identity(result)?
            .then(|| CheckedStateOrigins::instantiate(&result_state, &state_origins));
        if !self.deriving_result_state_origin.get() {
            let summaries = self
                .borrowed_state_origins
                .borrow()
                .get(target.0 as usize)
                .cloned()
                .unwrap_or_default();
            let mut updates = Vec::new();
            for summary in summaries {
                let ordinal = summary.parameter as usize;
                let image = CheckedStateOrigins::instantiate(&summary.origin, &state_origins);
                let before = state_origins.get(ordinal).and_then(Option::as_ref);
                if !image.unknown && before == Some(&image) {
                    continue;
                }
                let borrow = checked_borrows
                    .get(ordinal)
                    .and_then(Option::as_ref)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let Some(fields) = self
                    .state_fields_of_place(&borrow.place, bindings)?
                    .filter(|_| borrow.exact_place && !image.unknown)
                else {
                    return self
                        .unsupported(crate::UnsupportedSemanticFeature::OwnerStateRouting, node);
                };
                updates.push((borrow.place.root, fields, image));
            }
            // OWN-5 already established disjoint exclusive actuals. Resolve
            // every image before installing any of them into caller storage.
            for (root, fields, image) in updates {
                let local = bindings
                    .get_mut(&root)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                local.state_origins = Some(
                    local
                        .state_origins
                        .take()
                        .unwrap_or_else(CheckedStateOrigins::fresh)
                        .replace_value_path(&fields, Some(image)),
                );
            }
        }
        let slice = self.substitute_slice_result(signature, result, &checked_slices, bindings)?;
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
        let whole_candidate = self.whole_result_borrow_candidate(signature)?;
        let result_borrow_info = result_candidate
            .and_then(|index| checked_borrows.get(index))
            .cloned()
            .flatten()
            .map(|mut borrow| {
                borrow.exact_place &= whole_candidate == result_candidate;
                borrow
            });
        let result_borrow =
            if let Some((argument, borrow)) = result_candidate.zip(result_borrow_info.as_ref()) {
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
                let root = if let Some(local) = bindings.get(&borrow.place.root) {
                    crate::semantic::CheckedPlaceRoot::Binding(local.binding)
                } else if let Some(constant) = self.constants.get(&borrow.place.root) {
                    crate::semantic::CheckedPlaceRoot::Constant(*constant)
                } else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                Some(CheckedResultBorrow {
                    argument,
                    root,
                    path: borrow.place.path.clone(),
                })
            } else {
                None
            };
        self.statement_loans
            .borrow_mut()
            .extend(call_scoped_borrows);
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                state_origins: result_origins.map(Box::new),
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
            if borrow.exact_place {
                return self.goal_referent_image(&borrow.place, expected_type, bindings);
            }
            // FN-1's candidate protects every mutable origin a returned
            // borrow may reach, including when the delivered value is a
            // different immutable constant. ENT-2's value identity is the
            // actual holder, never that conservative loan ceiling.
            let place_parent = self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .unwrap_or(atom);
            let place = self
                .tree
                .first_child_with(place_parent, Production::Place)?
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
            let (image, _) = self.call_goal_place_inner(place, bindings)?;
            if image.ty() != expected_type {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            return Ok(image);
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
        projections.extend(place.path.iter().map(|step| match step {
            crate::semantic::places::PlaceStep::Field(field) => GoalProjection::Field(*field),
            crate::semantic::places::PlaceStep::Subscript(index) => {
                GoalProjection::Subscript(*index)
            }
        }));
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
                let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind else {
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
                            if borrow.exact_place {
                                self.goal_referent_image(&borrow.place, local.ty, bindings)?
                            } else {
                                GoalExpression::Datum(GoalDatum::Place {
                                    root: local.binding,
                                    projections: vec![GoalProjection::Deref],
                                    ty: local.ty,
                                })
                            },
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
    /// Whether one formal occupies a parameter mode or an explicit type
    /// position of this callable [FORM-8].
    fn formal_region_is_determined(
        &self,
        signature: &FunctionSignature,
        formal: DeclarationId,
    ) -> Result<bool, CheckStop> {
        for parameter in &signature.parameters {
            if matches!(
                parameter.mode,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) if region == formal
            ) || parameter.region_shape.determines(formal)
            {
                return Ok(true);
            }
        }
        Ok(false)
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
                let leading_regions = self.user_call_region_prefix(&arguments)?;
                if arguments.len() - leading_regions != generic_count {
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
                    .take(leading_regions)
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
    fn formal_region_index(signature: &FunctionSignature, formal: DeclarationId) -> Option<usize> {
        signature
            .region_parameters
            .iter()
            .position(|region| *region == formal)
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
    fn resolved_regions(
        &self,
        bindings: &mut [RegionBinding],
        observations: &[(usize, DeclarationId, NodeId)],
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        // Do not take a loan meet before all brands have been seen: loans
        // preceding a brand constrain that same fixed brand too, independent
        // of argument order. Only a formal with no invariant input takes
        // the ordinary least actual loan region.
        for (index, actual, node) in observations {
            let binding = &mut bindings[*index];
            if let Some(brand) = binding.store {
                if !self.region_outlives(*actual, brand)? {
                    return self.issue_node(SemanticRule::Own4, *node, SemanticIssueKind::InvalidBorrowLifetime {
                        region: self.region_phrase(brand)?,
                        binder: self.region_phrase(*actual)?,
                        mechanical_fix: "the actual loan must outlive the region fixed by this formal's invariant type positions; use a sufficiently long loan or distinct formal regions".to_owned(),
                    });
                }
            } else {
                self.observe_actual_region(binding, *actual, *node)?;
            }
        }
        bindings
            .iter()
            .map(|binding| {
                binding
                    .store
                    .or(binding.region)
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
        // A mode region belongs to the callable declaration; captured type
        // argument regions occur inside its type, never in this mode slot.
        let index = Self::formal_region_index(signature, formal)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
        let index = Self::formal_region_index(signature, formal)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
        // region is the observed actual. [OWN-4] is checked after all input
        // brands and loan regions have been collected, so a later brand
        // constrains this loan in the same way as an earlier one.
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

    /// Substitute all invariant positions together. A loan-only position is
    /// compared at its actual type here, then contributes to the final loan
    /// substitution. The final pass enforces whole-type equality after the
    /// complete substitution, including loan-only formals in view types;
    /// this preliminary check does not authorize a view coercion.
    fn substitute_parameter_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        bindings: &[RegionBinding],
        positions: &[(
            super::super::super::type_regions::RegionPosition,
            DeclarationId,
        )],
    ) -> Result<CheckedType, CheckStop> {
        let mut substitution = Self::fixed_call_regions(signature, bindings);
        for (position, actual) in positions {
            if position.invariant {
                continue;
            }
            if Self::formal_region_index(signature, position.formal).is_none() {
                continue;
            }
            substitution.retain(|(formal, _)| *formal != position.formal);
            substitution.push((position.formal, *actual));
        }
        self.substitute_type_regions(ty, &substitution)
    }

    /// One formal type with every formal region already resolved.
    ///
    /// [FN-2] substitutes a call's region arguments into every position of the
    /// callee's signature, and a result position is one of them: the type the
    /// caller receives names the store the call's own operands and written
    /// `::` members determined [FORM-8], never the declaration's formal
    /// region. Two calls of `fn f['s: affine](store: &uniq Arena<'s, ...>) ->
    /// made: own BlockPool<'s>` at two extents therefore produce two types,
    /// which is [PROV-1]'s invariant store region read at the position where
    /// the value is produced.
    ///
    /// The substitution is over the complete checked type, so it reaches a
    /// region under `Option`, inside a nominal instance, in a run's element
    /// position, and in the result-list nominal a multi-result callable hands
    /// back. A region this declaration does not parameterize — the entry
    /// heap or a brand captured inside a concrete type argument — occurs in
    /// no formal pair and is left alone.
    fn substitute_result_type(
        &self,
        ty: CheckedType,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Result<CheckedType, CheckStop> {
        let substitution = Self::call_region_substitution(signature, actual_regions);
        self.substitute_type_regions(ty, &substitution)
    }

    /// Every formal region this call has already fixed, paired with the
    /// region it fixed it to.
    ///
    /// A region the caller wrote in its `::` list is fixed by that member,
    /// and a store region an earlier operand position named is fixed by that
    /// operand and invariant afterwards [PROV-1]. A region no position has
    /// bound yet is deliberately absent: [FORM-8] leaves it to the actual at
    /// the position that first names it, which is the judgment
    /// [`Self::substitute_parameter_type`] makes at its own top level.
    fn fixed_call_regions(
        signature: &FunctionSignature,
        bindings: &[RegionBinding],
    ) -> Vec<(DeclarationId, DeclarationId)> {
        signature
            .region_parameters
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, formal)| {
                let binding = bindings.get(index)?;
                let region = if binding.written {
                    binding.region
                } else {
                    binding.store
                }?;
                (region != formal).then_some((formal, region))
            })
            .collect()
    }

    /// Every formal region of one call that its actual differs from, paired
    /// with that actual [FORM-8].
    fn call_region_substitution(
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
    ) -> Vec<(DeclarationId, DeclarationId)> {
        signature
            .region_parameters
            .iter()
            .copied()
            .zip(actual_regions.iter().copied())
            .filter(|(formal, actual)| formal != actual)
            .collect()
    }

    fn substitute_slice_result(
        &self,
        signature: &FunctionSignature,
        result: CheckedType,
        arguments: &[Option<SliceInfo>],
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Option<SliceInfo>, CheckStop> {
        let CheckedType::Slice { region, .. } = result else {
            return Ok(None);
        };
        let mut origins = Vec::new();
        let mut loans = Vec::new();
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
                    // [VIEW-6] an own-view formal relays the actual's
                    // existing claims. A borrowed-view formal can supply
                    // only a shared child, whose new claim has the result
                    // region and inherits those exact parent routes.
                    let child;
                    let actual = if signature.parameters[index].mode == CheckedMode::Own {
                        actual
                    } else {
                        child = actual.child(region);
                        Self::publish_slice_loans(&child, bindings)?;
                        &child
                    };
                    for key in &actual.loans {
                        if !loans.contains(key) {
                            loans.push(key.clone());
                        }
                    }
                }
                CheckedSliceOrigin::SourcePlace { .. } => {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
        }
        Ok(Some(SliceInfo {
            region,
            origins,
            loans,
        }))
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
                            && places_overlap(&left.place, &right.place)
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
                place: borrow.place.clone(),
            });
        }
        if let Some(slice) = slice {
            for origin in &slice.origins {
                let place = match origin {
                    CheckedSliceOrigin::SourcePlace { root, path, .. } => {
                        ResolvedPlace::from_path(*root, path.clone())
                    }
                    CheckedSliceOrigin::FormalSlice { parameter, .. } => {
                        ResolvedPlace::fields(*parameter, Vec::new())
                    }
                    CheckedSliceOrigin::ImmutableConst => continue,
                };
                claims.push(CallAccessClaim {
                    kind: BorrowKind::Shared,
                    place,
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
        // [S23] the third category projects exactly as the other two do. Its
        // loan access is the write an allocator already performs on the same
        // provider path [EFF-1], so the check is the write check and no second
        // access is attributed.
        for (access, declared, allocation) in [
            (AccessKind::Read, &signature.declared_effects.reads, false),
            (AccessKind::Write, &signature.declared_effects.writes, false),
            (
                AccessKind::Write,
                &signature.declared_effects.allocates,
                true,
            ),
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
                    place.extend_fields(&formal.fields);
                    self.check_call_loan_access(
                        bindings,
                        holders.get(index).copied().flatten(),
                        &place,
                        access,
                        node,
                    )?;
                    // Borrowing a view still checks the descriptor's loan.
                    // Its callee effects refer to the viewed backing below,
                    // just as they do when that same view is passed by value.
                    if !matches!(parameter.ty, CheckedType::Slice { .. }) {
                        actual_paths
                            .extend(self.effect_paths_for_whole_place(node, &place, bindings)?);
                    }
                }
                if let CheckedType::Slice { strength, .. } = parameter.ty {
                    let slice = slices
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    // [PROV-3] use 1: an access the callee makes through this
                    // view is *that view's own* access to its origin, judged
                    // at its own strength, and not a second access the view's
                    // loan should refuse. The skip is exact rather than
                    // conservative: no other loan can stand on a place an
                    // exclusive view already views, because a second view of
                    // either strength and every ordinary write are refused
                    // there [OWN-5], so the only loan the check would find is
                    // the one this actual holds. A shared actual keeps the
                    // check, where the callee's own row admits reads alone.
                    let judged = strength != LoanStrength::Exclusive;
                    for (mut place, _) in slice.source_places() {
                        place.extend_fields(&formal.fields);
                        if judged {
                            self.check_call_loan_access(
                                bindings,
                                holders.get(index).copied().flatten(),
                                &place,
                                access,
                                node,
                            )?;
                        }
                    }
                    for mut place in slice.effect_places() {
                        place.extend_fields(&formal.fields);
                        actual_paths
                            .extend(self.effect_paths_for_whole_place(node, &place, bindings)?);
                    }
                }

                for place in argument_places
                    .get(index)
                    .into_iter()
                    .flatten()
                    .filter(|_| {
                        parameter.mode == CheckedMode::Own
                            && state_origins.get(index).and_then(Option::as_ref).is_none()
                    })
                {
                    let mut path = self.state_path(place, bindings)?;
                    path.fields.extend_from_slice(&formal.fields);
                    actual_paths.push(path.into());
                }
                if let Some(origins) = state_origins.get(index).and_then(Option::as_ref) {
                    let origins = origins.clone().projected(&formal.fields);
                    actual_paths
                        .extend(self.effect_paths_for_origins(node, &origins, bindings, true)?);
                }

                for path in actual_paths {
                    if !caller
                        .parameters
                        .iter()
                        .any(|parameter| parameter.declaration == path.path.root)
                    {
                        continue;
                    }
                    match (access, allocation) {
                        (_, true) => effects.add_allocation(path),
                        (AccessKind::Read, false) => effects.add_read(path),
                        (AccessKind::Write, false) => effects.add_write(path),
                        _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                    }
                }
            }
        }
        Ok(())
    }
}
