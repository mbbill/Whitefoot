use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::model::{
    CheckedExpression, CheckedMode, CheckedSliceOrigin, CheckedType,
};
use super::super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, ResolvedPlace, SliceInfo, places_overlap, push_slice_origin,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
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
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        // The payload-free categories transfer by presence at a call
        // boundary [EFF-2]; only region entries are projected below.
        let mut effects = EffectSet {
            reads: Vec::new(),
            writes: Vec::new(),
            allocates_heap: signature.declared_effects.allocates_heap,
            allocates_arenas: Vec::new(),
            external: signature.declared_effects.external,
            blocks: signature.declared_effects.blocks,
            traps: signature.declared_effects.traps,
        };
        for (field, parameter) in fields.into_iter().zip(&signature.parameters) {
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
                return self.issue_node(SemanticRule::Type5, atom, SemanticIssueKind::TypeMismatch);
            }
            let passed_borrow = self.borrow_for_destination(expected_mode, &argument, atom)?;
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
            signature,
            &actual_regions,
            &checked_borrows,
            &checked_slices,
            &argument_holders,
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
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                arguments,
                result,
                slice_origins,
            },
            mode: result_mode,
            borrow: None,
            slice,
            holder: None,
            // A reference-returning call still yields a reference value; the
            // referent is reached only through an explicit holder [TYPE-7].
            reference_value: result_mode != CheckedMode::Own,
            effects,
            accesses: Vec::new(),
        })
    }

    fn call_region_arguments(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            if signature.region_parameters.is_empty() {
                return Ok(Vec::new());
            }
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let generic_count = signature.substitution.len();
        let expected = generic_count
            .checked_add(signature.region_parameters.len())
            .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        if arguments.len() != expected {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
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
                        SemanticRule::Type5,
                        argument,
                        SemanticIssueKind::TypeMismatch,
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
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
        borrows: &[Option<BorrowInfo>],
        slices: &[Option<SliceInfo>],
        holders: &[Option<DeclarationId>],
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
        for (parameter, ((borrow, slice), holder)) in signature
            .parameters
            .iter()
            .zip(borrows.iter().zip(slices).zip(holders))
        {
            let mode_region = match parameter.mode {
                CheckedMode::Own => None,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
            };
            let slice_region = match parameter.ty {
                CheckedType::Slice { region, .. } => Some(region),
                _ => None,
            };
            for (access, declared) in [
                (AccessKind::Read, &signature.declared_effects.reads),
                (AccessKind::Write, &signature.declared_effects.writes),
            ] {
                if mode_region.is_some_and(|region| declared.contains(&region)) {
                    let borrow = borrow
                        .as_ref()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    self.check_loan_access(bindings, *holder, &borrow.place, access, node)?;
                    if let Some(origin) = borrow.origin_region {
                        match access {
                            AccessKind::Read => effects.add_read(origin),
                            AccessKind::Write => effects.add_write(origin),
                            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                        }
                    }
                }
                if slice_region.is_some_and(|region| declared.contains(&region)) {
                    let slice = slice
                        .as_ref()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    for (place, _) in slice.source_places() {
                        self.check_loan_access(bindings, *holder, &place, access, node)?;
                    }
                    for origin in slice.effect_regions() {
                        match access {
                            AccessKind::Read => effects.add_read(origin),
                            AccessKind::Write => effects.add_write(origin),
                            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
