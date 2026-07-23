use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::model::{CheckedExpression, CheckedMode};
use super::super::super::borrows::{AccessKind, BorrowInfo, BorrowKind, places_overlap};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

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
        let mut argument_holders = Vec::with_capacity(fields.len());
        let mut call_scoped_borrows: Vec<BorrowInfo> = Vec::new();
        let mut effects = EffectSet {
            reads: Vec::new(),
            writes: Vec::new(),
            allocates_heap: signature.declared_effects.allocates_heap,
            allocates_arenas: Vec::new(),
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
            if argument.expression.ty() != parameter.ty {
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
        self.check_call_borrow_overlap(node, &checked_borrows)?;
        self.project_call_effects(
            node,
            signature,
            &actual_regions,
            &checked_borrows,
            &argument_holders,
            bindings,
            &mut effects,
        )?;
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                arguments,
                result: signature.result,
            },
            mode: signature.result_mode,
            borrow: None,
            holder: None,
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

    fn check_call_borrow_overlap(
        &self,
        node: NodeId,
        arguments: &[Option<BorrowInfo>],
    ) -> Result<(), CheckStop> {
        for (index, left) in arguments.iter().enumerate() {
            let Some(left) = left else {
                continue;
            };
            for right in arguments.iter().skip(index + 1).flatten() {
                if places_overlap(&left.place, &right.place)
                    && (left.kind == BorrowKind::Unique || right.kind == BorrowKind::Unique)
                {
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

    #[allow(clippy::too_many_arguments)]
    fn project_call_effects(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
        actual_regions: &[DeclarationId],
        arguments: &[Option<BorrowInfo>],
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
        for (parameter, (argument, holder)) in signature
            .parameters
            .iter()
            .zip(arguments.iter().zip(holders))
        {
            let Some(argument) = argument else {
                continue;
            };
            let formal_region = match parameter.mode {
                CheckedMode::Own => continue,
                CheckedMode::Shared(region) | CheckedMode::Unique(region) => region,
            };
            if signature.declared_effects.reads.contains(&formal_region) {
                self.check_loan_access(bindings, *holder, &argument.place, AccessKind::Read, node)?;
                if let Some(origin) = argument.origin_region {
                    effects.add_read(origin);
                }
            }
            if signature.declared_effects.writes.contains(&formal_region) {
                self.check_loan_access(
                    bindings,
                    *holder,
                    &argument.place,
                    AccessKind::Write,
                    node,
                )?;
                if let Some(origin) = argument.origin_region {
                    effects.add_write(origin);
                }
            }
        }
        Ok(())
    }
}
