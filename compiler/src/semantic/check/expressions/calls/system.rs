//! Call typing for admitted [SYS-2] system operations.
//!
//! A system operation is checked exactly like a user function call — named
//! [GRAM-11] arguments in declared order, explicit region arguments, borrow
//! formation and overlap checking, and [EFF-2] call-boundary effect
//! projection — except that its signature is the fixed catalog row rather
//! than a source declaration. System operations never exhibit `traps`.
//! Memory entries project through loans and storage origins; world entries
//! substitute directly through the written capability vector [EFF-2].

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, SystemOperation, SystemParameterMode,
    operation_region_effects,
};

use super::super::super::super::model::{CheckedExpression, CheckedMode};
use super::super::super::borrows::{AccessKind, BorrowInfo, BorrowKind, places_overlap};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

struct SystemCallProjection<'a> {
    actual_regions: &'a [DeclarationId],
    borrows: &'a [Option<BorrowInfo>],
    holders: &'a [Option<DeclarationId>],
    bindings: &'a HashMap<DeclarationId, LocalBinding>,
}

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
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        let mut effects = EffectSet {
            reads: Vec::new(),
            writes: Vec::new(),
            allocates_heap: false,
            allocates_arenas: Vec::new(),
            traps: false,
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
            let world_regions = parameter
                .world_regions
                .iter()
                .map(|index| {
                    actual_regions
                        .get(usize::from(*index))
                        .copied()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let expected_type = self.system_type_with(parameter.ty, &world_regions)?;
            if argument.expression.ty() != expected_type {
                return self.issue_node(SemanticRule::Type5, atom, SemanticIssueKind::TypeMismatch);
            }
            let passed_borrow = self.borrow_for_destination(expected_mode, &argument, atom)?;
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
            SystemCallProjection {
                actual_regions: &actual_regions,
                borrows: &checked_borrows,
                holders: &argument_holders,
                bindings,
            },
            &mut effects,
        )?;
        let result_world_regions = operation
            .result_world_regions
            .iter()
            .map(|index| {
                actual_regions
                    .get(usize::from(*index))
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = self.system_type_with(operation.result, &result_world_regions)?;
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
    pub(in crate::semantic::check) fn system_call_region_arguments(
        &self,
        node: NodeId,
        operation: &SystemOperation,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            if operation.regions.is_empty() {
                return Ok(Vec::new());
            }
            return self.issue_node(SemanticRule::Sys2, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if arguments.len() != operation.regions.len() {
            return self.issue_node(SemanticRule::Sys2, node, SemanticIssueKind::TypeMismatch);
        }
        let mut actual = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.into_iter().enumerate() {
            let usage = self.use_at(argument, LexicalUseRole::TypeArgumentRegion)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return self.issue_node(
                    SemanticRule::Sys2,
                    argument,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let kind = if operation.world_regions.contains(
                &u8::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            ) {
                super::super::super::super::model::CheckedRegionKind::World
            } else {
                super::super::super::super::model::CheckedRegionKind::Memory
            };
            self.constrain_region_kind(argument, declaration, kind)?;
            actual.push(declaration);
        }
        Ok(actual)
    }

    fn project_system_call_effects(
        &self,
        node: NodeId,
        operation: &SystemOperation,
        projection: SystemCallProjection<'_>,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        let (reads, writes) = operation_region_effects(operation);
        for (access, declared) in [(AccessKind::Read, &reads), (AccessKind::Write, &writes)] {
            for index in declared
                .iter()
                .filter(|index| operation.world_regions.contains(index))
            {
                let region = *projection
                    .actual_regions
                    .get(usize::from(*index))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                match access {
                    AccessKind::Read => effects.add_read(region),
                    AccessKind::Write => effects.add_write(region),
                    _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                }
            }
        }
        for (parameter, (borrow, holder)) in operation
            .parameters
            .iter()
            .zip(projection.borrows.iter().zip(projection.holders))
        {
            let mode_region = match parameter.mode {
                SystemParameterMode::Own => continue,
                SystemParameterMode::Borrow(region) | SystemParameterMode::UniqueBorrow(region) => {
                    region
                }
            };
            for (access, declared) in [(AccessKind::Read, &reads), (AccessKind::Write, &writes)] {
                if !declared.contains(&mode_region) {
                    continue;
                }
                if operation.world_regions.contains(&mode_region) {
                    continue;
                }
                let borrow = borrow
                    .as_ref()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.check_loan_access(projection.bindings, *holder, &borrow.place, access, node)?;
                if let Some(origin) = borrow.origin_region {
                    match access {
                        AccessKind::Read => effects.add_read(origin),
                        AccessKind::Write => effects.add_write(origin),
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
