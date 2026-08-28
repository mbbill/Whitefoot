//! Call typing for admitted [SYS-2] system operations.
//!
//! A system operation is checked exactly like a user function call — named
//! [GRAM-11] arguments in declared order, explicit region arguments, borrow
//! formation and overlap checking, and [EFF-2] call-boundary effect
//! projection — except that its signature is the fixed catalog row rather
//! than a source declaration. Its exact parameter-rooted state paths are
//! projected through each actual borrow or moved state's ordinary origin, so
//! invocation-local state frames out of the enclosing row. Compiler-owned
//! target-action metadata stays outside the source effect row.

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, SystemOperation, SystemParameterMode,
    operation_state_effects,
};

use super::super::super::super::model::{CheckedExpression, CheckedMode, CheckedStateOrigins};
use super::super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, ResolvedPlace, places_overlap,
};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_system_call(
        &self,
        node: NodeId,
        operation_index: u8,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let operation = crate::SYSTEM_OPERATIONS
            .get(usize::from(operation_index))
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let actual_regions = self.system_call_region_arguments(node, operation)?;
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
            || fields.len() != operation.parameters.len()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                Self::invalid_system_arguments(operation),
            );
        }
        let mut arguments = Vec::with_capacity(fields.len());
        let mut argument_nodes = Vec::with_capacity(fields.len());
        let mut checked_borrows = Vec::with_capacity(fields.len());
        let mut argument_holders = Vec::with_capacity(fields.len());
        let mut state_origins = Vec::with_capacity(fields.len());
        let mut argument_places = Vec::with_capacity(fields.len());
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        let mut effects = EffectSet {
            allocates_heap: false,
            traps: false,
            ..EffectSet::NONE
        };
        for (field, parameter) in fields.into_iter().zip(operation.parameters) {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(
                    SemanticRule::Gram11,
                    field,
                    Self::invalid_system_arguments(operation),
                );
            }
            let atom = self
                .tree
                .first_child_with(field, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            argument_nodes.push(self.tree.path(atom)?.clone());
            let explicit_borrow = self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some();
            // Every [SYS-2] result mode is `own`, so child reborrows are
            // admitted exactly as for an own-result user callee.
            let argument =
                self.check_call_argument_atom(function, atom, bindings, loop_depth, true, false)?;
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
            let expected_mode = match parameter.mode {
                SystemParameterMode::Own => CheckedMode::Own,
                SystemParameterMode::Borrow(region) => CheckedMode::Shared(
                    *actual_regions
                        .get(usize::from(region))
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                ),
                SystemParameterMode::UniqueBorrow(region) => CheckedMode::Unique(
                    *actual_regions
                        .get(usize::from(region))
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                ),
            };
            let expected_type = self.system_type(parameter.ty)?;
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
            if explicit_borrow && let Some(borrow) = &argument.borrow {
                call_scoped_borrows.push(borrow.clone());
            }
            checked_borrows.push(passed_borrow);
            argument_holders.push(argument.holder);
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        let no_slices = vec![None; checked_borrows.len()];
        self.check_call_borrow_overlap(node, &checked_borrows, &no_slices)?;
        self.project_system_call_effects(
            node,
            operation,
            SystemEffectActuals {
                borrows: &checked_borrows,
                holders: &argument_holders,
                state_origins: &state_origins,
                argument_places: &argument_places,
            },
            function,
            bindings,
            &mut effects,
        )?;
        let result = self.system_type(operation.result)?;
        Ok(TypedExpression::owned(
            CheckedExpression::SystemCall {
                operation: operation_index,
                target_action: operation.target_action,
                call: self.tree.path(node)?.clone(),
                regions: actual_regions,
                argument_nodes,
                arguments,
                result,
            },
            effects,
        ))
    }

    /// The written region arguments of a system operation's call.
    ///
    /// [DIAG-1] selects the cited rule by the callee's class, and [TYPE-5]
    /// names the three classes: "type, region, and const arguments for user
    /// generics [FN-2]; region arguments for system operations [SYS-2]; and,
    /// for exactly the retained-argument table operations … the written
    /// arguments their rows fix". This is the system class, so its argument
    /// list is SYS-2's — the third clause, which had no representable rule
    /// until 2026-08-08 and therefore cited TYPE-5.
    fn system_call_region_arguments(
        &self,
        node: NodeId,
        operation: &SystemOperation,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            if operation.regions.is_empty() {
                return Ok(Vec::new());
            }
            return self.issue_node(SemanticRule::Sys2, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(operation.regions.len(), "region argument"), "no type-argument list"));
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if arguments.len() != operation.regions.len() {
            return self.issue_node(SemanticRule::Sys2, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(operation.regions.len(), "region argument"), crate::semantic::written_count(arguments.len(), "argument")));
        }
        arguments
            .into_iter()
            .map(|argument| {
                let usage = self.use_at(argument, LexicalUseRole::TypeArgumentRegion)?;
                match usage.target() {
                    ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::Region,
                    } => Ok(declaration),
                    _ => self.issue_node(
                        SemanticRule::Sys2,
                        argument,
                        SemanticIssueKind::type_mismatch("a region argument in this position", "an argument that does not name a region"),
                    ),
                }
            })
            .collect()
    }

    fn project_system_call_effects(
        &self,
        node: NodeId,
        operation: &SystemOperation,
        actuals: SystemEffectActuals<'_>,
        caller: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        let (reads, writes) = operation_state_effects(operation);
        for (access, declared) in [(AccessKind::Read, reads), (AccessKind::Write, writes)] {
            for ordinal in declared {
                let index = usize::from(ordinal);
                let parameter = operation
                    .parameters
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut paths = Vec::new();
                if !matches!(parameter.mode, SystemParameterMode::Own) {
                    let borrow = actuals
                        .borrows
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    self.check_loan_access(
                        bindings,
                        actuals.holders.get(index).copied().flatten(),
                        &borrow.place,
                        access,
                        node,
                    )?;
                    paths.extend(self.effect_paths_for_place(&borrow.place, bindings)?);
                }
                for place in actuals.argument_places.get(index).into_iter().flatten() {
                    paths.push(self.state_path(place, bindings)?);
                }
                if let Some(origins) = actuals.state_origins.get(index).and_then(Option::as_ref) {
                    if origins.unknown && !self.deriving_result_state_origin.get() {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                    for origin in &origins.formals {
                        paths.push(origin.source.clone());
                    }
                }
                for path in paths {
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

    fn invalid_system_arguments(operation: &SystemOperation) -> SemanticIssueKind {
        SemanticIssueKind::InvalidNamedArguments {
            callee: operation.spelling.to_owned(),
            declared_parameters: operation
                .parameters
                .iter()
                .map(|parameter| parameter.name.to_owned())
                .collect(),
        }
    }
}

struct SystemEffectActuals<'a> {
    borrows: &'a [Option<BorrowInfo>],
    holders: &'a [Option<DeclarationId>],
    state_origins: &'a [Option<CheckedStateOrigins>],
    argument_places: &'a [Vec<ResolvedPlace>],
}
