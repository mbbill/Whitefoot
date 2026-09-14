use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, FixedTerminal, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::super::model::{
    CheckedContainerRoot, CheckedExpression, CheckedMode, CheckedSliceOrigin, CheckedSliceRange,
    CheckedSliceSource, CheckedType, IntegerType, LoanStrength, MeasuredKind,
};
use super::super::super::borrows::{
    AccessKind, BorrowKind, ReborrowPosition, ResolvedPlace, SliceInfo, SliceLoanKey, TemporaryLoan,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::CheckedIndexedPlace;
use crate::semantic::places::{PlaceStep, RangeId};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [VIEW-2] one view formation, at the strength the written row names.
    ///
    /// The two rows are one judgment: `slice_of` takes a shared borrow of the
    /// viewed storage and hands back a shared loan, `mut_slice_of` takes a
    /// `&uniq` borrow and hands back an exclusive one, and every other
    /// sentence — the region the borrow takes [OP-2], the origin the view
    /// carries [PROV-3], the element the viewed place fixes — is written
    /// once for both.
    pub(in crate::semantic::check) fn check_slice_of(
        &self,
        node: NodeId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: strength.former().to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        // [TYPE-5] `slice_of` is outside the retained-argument class, so it
        // carries no written argument: the region comes from the operand's own
        // borrow and the element from the place it views. A written argument
        // here is the rejection, not the supply.
        if self.tree.argument_list(node)?.is_some() {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let count = match self.tree.first_child_with(node, Production::AtomList)? {
            Some(list) => self.tree.children_with(list, Production::Atom)?.len(),
            None => 0,
        };
        let row = match strength {
            LoanStrength::Shared => crate::KernelRow::SliceOf,
            LoanStrength::Exclusive => crate::KernelRow::MutSliceOf,
        };
        let signature = crate::semantic::kernel::kernel_signature(row);
        let contract = signature
            .range
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let range_arity = signature.parameters.len() + contract.endpoint_parameters.len();
        let atoms =
            self.operation_atoms(node, if count == range_arity { range_arity } else { 1 })?;
        let mut range_effects = EffectSet::NONE;
        let mut range_accesses = Vec::new();
        let range = if count == range_arity {
            let mut endpoints = Vec::new();
            for atom in [atoms[contract.start], atoms[contract.end]] {
                let value = self.check_atom(function, atom, bindings, loop_depth)?;
                if value.mode != CheckedMode::Own
                    || value.expression.ty() != CheckedType::Integer(IntegerType::U64)
                {
                    return self.issue_node(
                        SemanticRule::Type5,
                        atom,
                        SemanticIssueKind::type_mismatch(
                            "an own u64 range endpoint",
                            "a value of another mode or type",
                        ),
                    );
                }
                endpoints.push(value.expression);
                range_effects = range_effects.union(value.effects);
                range_accesses.extend(value.accesses);
            }
            let end = endpoints
                .pop()
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let start = endpoints
                .pop()
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            Some(CheckedSliceRange {
                start: Box::new(start),
                end: Box::new(end),
            })
        } else {
            None
        };
        let mut formed = self.check_slice_source(
            node,
            strength,
            function,
            bindings,
            loop_depth,
            atoms[contract.source],
            range.is_some(),
        )?;
        let CheckedExpression::SliceOf {
            range: destination, ..
        } = &mut formed.expression
        else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        *destination = range;
        formed.effects = formed.effects.union(range_effects);
        formed.accesses.extend(range_accesses);
        Ok(formed)
    }

    #[allow(clippy::too_many_arguments)]
    fn check_slice_source(
        &self,
        node: NodeId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        operand: NodeId,
        bounded: bool,
    ) -> Result<TypedExpression, CheckStop> {
        let atoms = [operand];
        let borrow = self
            .tree
            .first_child_with(atoms[0], Production::BorrowExpr)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::type_mismatch(
                        "a written shared borrow of the viewed storage, `&'r place`",
                        "an atom that is not a borrow expression",
                    ),
                )
            })?;
        let written = if self.has_fixed(borrow, FixedTerminal::Uniq)? {
            LoanStrength::Exclusive
        } else {
            LoanStrength::Shared
        };
        if written != strength {
            let (expected, found) = match strength {
                LoanStrength::Shared => (
                    "a written shared borrow of the viewed storage, `&'r place`",
                    "a `&uniq` borrow, which slice_of does not take",
                ),
                LoanStrength::Exclusive => (
                    "a written unique borrow of the viewed storage, `&uniq 'r place`",
                    "a shared borrow, which mut_slice_of does not take",
                ),
            };
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(expected, found),
            );
        }
        // [OP-2] the result region is the one the operand's borrow takes,
        // written or elided [FORM-8].
        let Some(region) = self.borrow_expr_region(borrow)? else {
            return self.issue_node(
                crate::SemanticRule::Form8,
                borrow,
                crate::SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "write the region this borrow takes, or place the borrow \
inside the `region` block whose region it takes",
                },
            );
        };
        let place_node = self
            .tree
            .first_child_with(borrow, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase = self
            .tree
            .first_child_with(place_node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if let Some(formed) = self.check_view_holder_slice_of(
            node, borrow, place_node, pbase, region, strength, bindings, loop_depth, atoms[0],
        )? {
            return Ok(formed);
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            if let Some(formed) = self.check_storage_holder_slice_of(
                node, borrow, place_node, pbase, region, strength, function, bindings, loop_depth,
                bounded,
            )? {
                return Ok(formed);
            }
            return self.check_arena_content_slice_of(
                node, borrow, place_node, pbase, region, strength, function, bindings, loop_depth,
                bounded,
            );
        }
        let root_use = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = root_use.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let owner = match class {
            DeclarationClass::Value => Some(declaration),
            // [CONST-2] a named const is permanently fixed storage, so it is
            // the `immutable-const` origin of a shared view and no origin an
            // exclusive one could write through.
            DeclarationClass::NamedConst if strength == LoanStrength::Shared => None,
            DeclarationClass::NamedConst => {
                return self.issue_node(
                    SemanticRule::Const2,
                    atoms[0],
                    SemanticIssueKind::ImmutableSetTarget,
                );
            }
            _ => {
                return self.issue_node(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::type_mismatch(
                        "a borrow of a runtime value binding or a named const",
                        "a borrow of a declaration that is neither",
                    ),
                );
            }
        };
        self.check_direct_slice_borrow_lifetime(function, region, owner, borrow, loop_depth)?;
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let indexed = self
            .check_indexed_place(
                place_node, bindings, &suffixes, place_node, function, loop_depth,
            )?
            .into_element_storage()?;
        // [OP-2] derives the element from the viewed place. Full arrays and
        // runs admit owning elements; general element views remain a
        // capability gap, not a source-language rejection.
        let element_type = indexed.element_type(self)?;
        // An affine-element buffer is viewable in principle ([OP-1] states no
        // copy bound on the viewed T), but the in-place borrowed element read
        // a view would serve is not implemented, so the view stops as an
        // explicit unsupported capability rather than a source rejection.
        if let CheckedType::Nominal(id) = element_type
            && !self.nominal(id)?.is_copy()
        {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, atoms[0]);
        }
        let Some(element) = self.flat_element(element_type)? else {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, atoms[0]);
        };
        let offsets = match &indexed {
            CheckedIndexedPlace::Container(container) => container.offsets.clone(),
            _ => super::CarriedOperands::default(),
        };
        let (source, mut resolved) = match indexed {
            // Immutable globals retain their direct constant-array source.
            // Mutable owners were converted to typed storage above, so both
            // strengths view the owner's slots rather than an array snapshot.
            CheckedIndexedPlace::Array(array) => {
                let resolved = array
                    .resolved_place()
                    .unwrap_or(ResolvedPlace::fields(declaration, Vec::new()));
                (
                    CheckedSliceSource::Array {
                        root: array.root,
                        length: array.length,
                    },
                    resolved,
                )
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                (CheckedSliceSource::Buffer(buffer.root), buffer.resolved)
            }
            // [VIEW-2] typed storage shares the non-wrap requirement. Runs
            // discharge it from their current window; full arrays discharge
            // it from their type's zero head and room [MSR-1].
            CheckedIndexedPlace::Container(container) => {
                let Some(measured) = container.root.measured() else {
                    return self.issue_node(
                        SemanticRule::Op1,
                        node,
                        SemanticIssueKind::InvalidOperation,
                    );
                };
                if !matches!(
                    measured,
                    MeasuredKind::FixedVector | MeasuredKind::Vector | MeasuredKind::Array
                ) {
                    return self.issue_node(
                        SemanticRule::Op1,
                        node,
                        SemanticIssueKind::InvalidOperation,
                    );
                }
                let resolved = container.resolved.clone();
                (CheckedSliceSource::Run(container.root.clone()), resolved)
            }
            CheckedIndexedPlace::Slice(_) => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let loan = self.slice_loan_id(node)?;
        if bounded {
            resolved.path.push(PlaceStep::Range(loan));
        }
        let origin = owner.map_or(CheckedSliceOrigin::ImmutableConst, |_| {
            CheckedSliceOrigin::SourcePlace {
                root: resolved.root,
                path: resolved.path.clone(),
                origin_region: None,
            }
        });
        let origins = vec![origin];
        self.check_commit_place_live(&resolved, borrow, false)?;
        // [PROV-3] use 1: the formation's own access to the origin is the
        // access the loan's strength names, so a second exclusive view of one
        // place meets the first loan here and is the ordinary [OWN-5]
        // conflict, while a second shared view does not.
        let taken = match strength {
            LoanStrength::Shared => AccessKind::SharedBorrow,
            LoanStrength::Exclusive => AccessKind::UniqueBorrow,
        };
        let mut loans = Vec::new();
        let mut accesses = if owner.is_some() {
            self.check_loan_access(bindings, None, &resolved, taken, borrow)?;
            loans.push(SliceLoanKey {
                region,
                place: resolved.clone(),
                parent: None,
                strength,
            });
            vec![PlaceAccess {
                place: resolved,
                kind: taken,
            }]
        } else {
            Vec::new()
        };
        accesses.extend(offsets.accesses);
        let slice = SliceInfo {
            region,
            origins: origins.clone(),
            loans,
        };
        Self::publish_slice_loans(&slice, bindings)?;
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                loan,
                source,
                range: None,
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(slice),
            holder: None,
            reference_value: false,
            effects: offsets.effects,
            accesses,
        })
    }

    /// [VIEW-2, OWN-6] form a storage view through the ordinary checked child
    /// borrow. Its physical address and ultimate origin are separate results
    /// of that judgment; neither is inferred from a holder's spelling.
    #[allow(clippy::too_many_arguments)]
    fn check_storage_holder_slice_of(
        &self,
        node: NodeId,
        borrow: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        bounded: bool,
    ) -> Result<Option<TypedExpression>, CheckStop> {
        let Some(inner) = self.tree.first_child_with(pbase, Production::Place)? else {
            return Ok(None);
        };
        let Some(base) = self.tree.first_child_with(inner, Production::Pbase)? else {
            return Ok(None);
        };
        if self.has_fixed(base, FixedTerminal::Deref)?
            || !self.tree.children(base)?.is_empty()
            || !self
                .tree
                .children_with(inner, Production::Psuffix)?
                .is_empty()
        {
            return Ok(None);
        }
        let usage = self.use_at(base, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(None);
        };
        let Some(local) = bindings.get(&declaration) else {
            return Ok(None);
        };
        if local.mode == CheckedMode::Own {
            return Ok(None);
        }
        let range_id = self.slice_loan_id(node)?;
        let argument = self.check_child_reborrow(
            borrow,
            place_node,
            pbase,
            region,
            match strength {
                LoanStrength::Shared => BorrowKind::Shared,
                LoanStrength::Exclusive => BorrowKind::Unique,
            },
            bindings,
            function,
            loop_depth,
            ReborrowPosition::CallArgument {
                own_result: true,
                result_candidate: false,
            },
            bounded.then_some(range_id),
        )?;
        let ty = argument.expression.ty();
        let element_type = match ty {
            CheckedType::Array { element, .. }
            | CheckedType::FixedVector { element, .. }
            | CheckedType::Vector { element, .. } => self.element_type(element)?,
            CheckedType::Buffer { element } => element.ty(),
            _ => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        if let CheckedType::Nominal(id) = element_type
            && !self.nominal(id)?.is_copy()
        {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place_node);
        }
        let Some(element) = self.flat_element(element_type)? else {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place_node);
        };
        let source = match argument.expression.clone() {
            CheckedExpression::ReborrowAddressed { binding, .. } => {
                CheckedSliceSource::Run(CheckedContainerRoot {
                    root: crate::semantic::CheckedPlaceRoot::Binding(binding),
                    path: Vec::new(),
                    ty,
                })
            }
            CheckedExpression::BorrowAddressed { root, .. } => CheckedSliceSource::Run(root),
            CheckedExpression::BorrowBuffer { root, .. } => CheckedSliceSource::Buffer(root),
            _ => {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
            }
        };
        let loan = argument
            .borrow
            .as_ref()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let mut loans = Vec::new();
        let origin = if self.constants.contains_key(&loan.place.root) {
            CheckedSliceOrigin::ImmutableConst
        } else {
            loans.push(SliceLoanKey {
                region,
                place: loan.place.clone(),
                parent: argument.holder,
                strength,
            });
            CheckedSliceOrigin::SourcePlace {
                root: loan.place.root,
                path: loan.place.path.clone(),
                origin_region: loan.origin_region,
            }
        };
        self.statement_loans
            .borrow_mut()
            .push(TemporaryLoan::new(loan.clone(), &argument));
        let origins = vec![origin];
        let slice = SliceInfo {
            region,
            origins: origins.clone(),
            loans,
        };
        Self::publish_slice_loans(&slice, bindings)?;
        let mut accesses = argument.accesses;
        accesses.push(PlaceAccess {
            place: loan.place.clone(),
            kind: match strength {
                LoanStrength::Shared => AccessKind::SharedBorrow,
                LoanStrength::Exclusive => AccessKind::UniqueBorrow,
            },
        });
        Ok(Some(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                loan: range_id,
                source,
                range: None,
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(slice),
            holder: None,
            reference_value: false,
            effects: argument.effects,
            accesses,
        }))
    }

    /// [VIEW-2, OWN-6] Reborrow a directly owned view or the view reached by
    /// one dereference of a holder. A child retains every parent origin and
    /// appends its relative range. Its region and strength cannot exceed the
    /// parent's; incompatible parent and sibling accesses remain subject to
    /// [OWN-5]. `None` leaves other operand forms to their existing checker.
    #[allow(clippy::too_many_arguments)]
    fn check_view_holder_slice_of(
        &self,
        node: NodeId,
        borrow: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        strength: LoanStrength,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        operand: NodeId,
    ) -> Result<Option<TypedExpression>, CheckStop> {
        let holder = self.has_fixed(pbase, FixedTerminal::Deref)?;
        let (inner_place, inner_pbase) = if holder {
            let Some(inner_place) = self.tree.first_child_with(pbase, Production::Place)? else {
                return Ok(None);
            };
            let Some(inner_pbase) = self.tree.first_child_with(inner_place, Production::Pbase)?
            else {
                return Ok(None);
            };
            (inner_place, inner_pbase)
        } else {
            (place_node, pbase)
        };
        // One deref of a directly named holder, no suffix chain on either
        // half: a deeper chain keeps its existing disposition.
        if self.has_fixed(inner_pbase, FixedTerminal::Deref)?
            || !self.tree.children(inner_pbase)?.is_empty()
            || !self
                .tree
                .children_with(inner_place, Production::Psuffix)?
                .is_empty()
            || !self
                .tree
                .children_with(place_node, Production::Psuffix)?
                .is_empty()
        {
            return Ok(None);
        }
        let root_use = self.use_at(inner_pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = root_use.target()
        else {
            return Ok(None);
        };
        let Some(local) = bindings.get(&declaration).cloned() else {
            return Ok(None);
        };
        let CheckedType::Slice {
            region: parent_region,
            element,
            strength: parent_strength,
        } = local.ty
        else {
            return Ok(None);
        };
        if holder == (local.mode == CheckedMode::Own) {
            return Ok(None);
        }
        // Every rejection below is a source verdict about a program this arm
        // owns, so it is reported rather than handed on.
        if !self.borrow_region_is_inside_current_loops(region, borrow, loop_depth)? {
            return self
                .issue_node(
                    SemanticRule::Own11,
                    borrow,
                    SemanticIssueKind::BorrowRegionOutsideLoop {
                        mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                    },
                )
                .map(Some);
        }
        if !local.live {
            return self
                .issue_node(
                    SemanticRule::Own1,
                    place_node,
                    SemanticIssueKind::UseAfterMove {
                        mechanical_fix: "introduce a new `let` binding before reuse",
                    },
                )
                .map(Some);
        }
        self.check_holder_not_suspended(&local, place_node)?;
        if strength == LoanStrength::Exclusive
            && (parent_strength == LoanStrength::Shared
                || matches!(local.mode, CheckedMode::Shared(_)))
        {
            return self
                .issue_node(
                    SemanticRule::Own5,
                    operand,
                    SemanticIssueKind::BorrowConflict,
                )
                .map(Some);
        }
        // [OWN-10] the child's loan may not outlive the parent's own.
        if !self.region_outlives(parent_region, region)? {
            return self
                .issue_node(
                    SemanticRule::Own10,
                    borrow,
                    SemanticIssueKind::InvalidBorrowLifetime {
                        region: self.region_phrase(region)?,
                        binder: self.declaration_spelling(declaration)?,
                        mechanical_fix: "a child of a view names a region the view's own \
region outlives; name that region, or one it outlives, on this borrow"
                            .to_owned(),
                    },
                )
                .map(Some);
        }
        let Some(parent) = local.slice.clone() else {
            return Ok(None);
        };
        let holder_place = ResolvedPlace::fields(declaration, Vec::new());
        self.check_loan_access(
            bindings,
            Some(declaration),
            &holder_place,
            AccessKind::SharedBorrow,
            borrow,
        )?;
        let loan = self.slice_loan_id(node)?;
        let mut slice = parent;
        slice.region = region;
        for origin in &mut slice.origins {
            match origin {
                CheckedSliceOrigin::SourcePlace { path, .. }
                | CheckedSliceOrigin::FormalSlice { path, .. } => path.push(PlaceStep::Range(loan)),
                CheckedSliceOrigin::ImmutableConst => {}
            }
        }
        // Keep the checked parent and physical storage identity while the
        // continuing claim narrows to the child's captured extent.
        for key in &mut slice.loans {
            key.region = region;
            key.strength = strength;
            key.place.path.push(PlaceStep::Range(loan));
        }
        let taken = match strength {
            LoanStrength::Shared => AccessKind::SharedBorrow,
            LoanStrength::Exclusive => AccessKind::UniqueBorrow,
        };
        for key in &slice.loans {
            self.check_loan_access(bindings, Some(declaration), &key.place, taken, borrow)?;
        }
        Self::publish_slice_loans(&slice, bindings)?;
        let origins = slice.origins.clone();
        Ok(Some(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                loan,
                source: CheckedSliceSource::ViewHolder {
                    binding: local.binding,
                    element,
                },
                range: None,
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(slice),
            holder: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses: Vec::new(),
        }))
    }

    /// [OWN-5] `slice_of` over a place reached in arena content: the operand
    /// `&'a deref(storage)` views the content array of an own `arena<'r, T>`
    /// binding. Its region obeys [OWN-10]'s arena case — the arena's `'r`
    /// must outlive-or-equals the borrow's region — and the created slice's
    /// origin retains the complete resolved place, so [FN-1]'s return-origin
    /// ceiling excludes it exactly as it excludes every other raw callee
    /// place: an `arena<'r, U>` parameter is not an input-slice supplier.
    #[allow(clippy::too_many_arguments)]
    fn check_arena_content_slice_of(
        &self,
        node: NodeId,
        borrow: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        bounded: bool,
    ) -> Result<TypedExpression, CheckStop> {
        if !self.borrow_region_is_inside_current_loops(region, borrow, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                borrow,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                },
            );
        }
        let inner_place = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let inner_pbase = self
            .tree
            .first_child_with(inner_place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // The implemented fragment reaches content through one deref of a
        // directly named binding; deeper chains and projected content stay
        // explicit capability stops.
        if self.has_fixed(inner_pbase, FixedTerminal::Deref)?
            || !self.tree.children(inner_pbase)?.is_empty()
            || !self
                .tree
                .children_with(inner_place, Production::Psuffix)?
                .is_empty()
            || !self
                .tree
                .children_with(place_node, Production::Psuffix)?
                .is_empty()
        {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        let root_use = self.use_at(inner_pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = root_use.target()
        else {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        };
        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                place_node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        let Some((arena_region, content)) = self.arena_instance(local.ty)? else {
            // A deref over anything but an own arena binding stays a
            // capability stop rather than a fabricated source verdict.
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        };
        if local.mode != CheckedMode::Own {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        // [OWN-10] for a place rooted in arena<'r, T> content: 'r must
        // outlive-or-equals the borrow's region.
        if !self.region_outlives(arena_region, region)? {
            return self.issue_node(
                SemanticRule::Own10,
                borrow,
                SemanticIssueKind::InvalidBorrowLifetime {
                    region: self.region_phrase(region)?,
                    binder: self.declaration_spelling(declaration)?,
                    // The arena's own region is what the view must name. A
                    // region [FORM-8] leaves unwritten has no name to give,
                    // so the repair is to relate the two positions first.
                    mechanical_fix: match self.written_region_name(arena_region)? {
                        Some(name) => format!(
                            "arena content outlives its arena's region {name}, not the arena \
binding; name {name} on this view, or a region {name} outlives"
                        ),
                        None => "arena content outlives its arena's own region, not the arena \
binding; that region is unwritten here, so write it on the arena and name it on this view, or \
take the view in a region it outlives"
                            .to_owned(),
                    },
                },
            );
        }
        // TEMPORARY capability stop, judged after the [OWN-1] and [OWN-10]
        // source rejections above: no lowering builds a slice over arena
        // content. A view over an arena *parameter* still checks on, because
        // the whole function then stops at the arena-parameter gate, and the
        // [FN-1] return-origin judgment must reach its verdict first. A view
        // over a *local* arena has no such later gate, so without this stop
        // it would publish a checked program the IR builder cannot lower.
        if !function
            .parameters
            .iter()
            .any(|parameter| parameter.declaration == declaration)
        {
            return self.unsupported(UnsupportedSemanticFeature::ArenaRuntime, place_node);
        }
        let CheckedType::Array { element, length } = content else {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place_node);
        };
        let Some(element) = self.flat_element(self.element_type(element)?)? else {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place_node);
        };
        let range_id = self.slice_loan_id(node)?;
        let mut resolved = ResolvedPlace::fields(declaration, Vec::new());
        if bounded {
            resolved.path.push(PlaceStep::Range(range_id));
        }
        let taken = match strength {
            LoanStrength::Shared => AccessKind::SharedBorrow,
            LoanStrength::Exclusive => AccessKind::UniqueBorrow,
        };
        self.check_loan_access(bindings, None, &resolved, taken, borrow)?;
        // The origin is the complete resolved place reached in arena content
        // [OWN-5]; reads through the formed view stay reads of storage this
        // function owns, so the formation carries no boundary effect.
        let origins = vec![CheckedSliceOrigin::SourcePlace {
            root: declaration,
            path: resolved.path.clone(),
            origin_region: None,
        }];
        let slice = SliceInfo {
            region,
            origins: origins.clone(),
            loans: vec![SliceLoanKey {
                region,
                place: resolved.clone(),
                parent: None,
                strength,
            }],
        };
        Self::publish_slice_loans(&slice, bindings)?;
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                loan: range_id,
                source: CheckedSliceSource::ArenaContent {
                    binding: local.binding,
                    fields: Vec::new(),
                    length,
                },
                range: None,
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(slice),
            holder: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses: vec![PlaceAccess {
                place: resolved,
                kind: taken,
            }],
        })
    }

    fn slice_loan_id(&self, node: NodeId) -> Result<RangeId, CheckStop> {
        Ok(RangeId(
            u32::try_from(node.index()).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        ))
    }
}
