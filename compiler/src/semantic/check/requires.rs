use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::goal::{
    CheckedRequirement, GoalDatum, GoalExpression, GoalOperation, GoalProjection, GoalTemplate,
};
use super::super::model::{
    BindingId, CheckedExpression, CheckedFloatOperation, CheckedMode, CheckedNominalKind,
    CheckedStatement, CheckedType,
};
use super::{CheckStop, Checker, ControlCounters, ControlScope, FunctionSignature, LocalBinding};

pub(super) struct CheckedRequires {
    pub(super) requirement: CheckedRequirement,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_requires(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
    ) -> Result<CheckedRequires, CheckStop> {
        let entries = self.tree.children_with(node, Production::RequiresEntry)?;
        let mut expanded_bindings = HashMap::new();
        for (ordinal, parameter) in function.parameters.iter().enumerate() {
            let local = bindings
                .get(&parameter.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            expanded_bindings.insert(
                local.binding,
                GoalExpression::Datum(GoalDatum::Parameter {
                    ordinal,
                    projections: Vec::new(),
                    ty: parameter.ty,
                }),
            );
        }
        let mut requirement = None;
        for entry in entries {
            let wrapper = self.tree.only_child(entry)?;
            if self.tree.production(wrapper)? != Production::Stmt {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let statement = self.tree.only_child(wrapper)?;
            self.validate_requires_statement(entry, statement)?;
            let checked = self.check_statement(
                function,
                statement,
                bindings,
                counters,
                ControlScope {
                    loops: &[],
                    give_context: None,
                },
            )?;
            if !checked.can_continue {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            // The clause local's copy judgment waits for `check_statement`
            // because [FN-8] describes a derived property: the local is an
            // "own copy value", and after the v0.23 annotation deletion the
            // type comes from the [TYPE-5] derivation just performed. Reading
            // the checker's own answer keeps one derivation; the earlier
            // subset pass still reports a malformed clause first.
            let expression = self.requires_statement_expression(statement)?;
            match &checked.statement {
                CheckedStatement::Let { binding, value } => {
                    self.validate_requires_copy_local(entry, *binding, bindings)?;
                    let expanded = self.build_goal_expression(
                        expression,
                        value,
                        bindings,
                        &expanded_bindings,
                    )?;
                    expanded_bindings.insert(*binding, expanded);
                }
                CheckedStatement::Check { condition, trap } => {
                    let root = self.build_goal_expression(
                        expression,
                        condition,
                        bindings,
                        &expanded_bindings,
                    )?;
                    if root.ty() != CheckedType::Bool || requirement.is_some() {
                        return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
                    }
                    requirement = Some(CheckedRequirement {
                        template: GoalTemplate::new(root),
                        trap: trap.clone(),
                    });
                }
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            }
        }
        Ok(CheckedRequires {
            requirement: requirement.ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?,
        })
    }

    fn requires_statement_expression(&self, statement: NodeId) -> Result<NodeId, CheckStop> {
        let owner = if self.tree.production(statement)? == Production::LetStmt {
            self.tree
                .first_child_with(statement, Production::OrdinaryLetRhs)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?
        } else {
            statement
        };
        self.tree
            .first_child_with(owner, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// Converts one already-checked admitted FN-8 expression into its typed
    /// predicate identity. Source atoms supply declaration/projection identity;
    /// the checked expression supplies the uniquely selected row and types.
    fn build_goal_expression(
        &self,
        source: NodeId,
        checked: &CheckedExpression,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, GoalExpression>,
    ) -> Result<GoalExpression, CheckStop> {
        let atoms = self.goal_operand_atoms(source)?;
        let operation = match checked {
            CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                result,
                ..
            } => Some((
                GoalOperation::Integer {
                    operation: *operation,
                    operand_type: *operand_type,
                },
                Vec::new(),
                Vec::new(),
                *result,
                arguments.as_slice(),
            )),
            CheckedExpression::FloatOperation {
                operation,
                operand_type,
                arguments,
            } => Some((
                GoalOperation::Float {
                    operation: *operation,
                    operand_type: *operand_type,
                },
                if matches!(
                    operation,
                    CheckedFloatOperation::Infinity | CheckedFloatOperation::Nan
                ) {
                    vec![*operand_type]
                } else {
                    Vec::new()
                },
                Vec::new(),
                operation.result_type(*operand_type),
                arguments.as_slice(),
            )),
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                result,
            } => Some((
                GoalOperation::NumericConversion {
                    source: *source,
                    destination: *destination,
                },
                vec![source.ty(), destination.ty()],
                Vec::new(),
                *result,
                std::slice::from_ref(value.as_ref()),
            )),
            CheckedExpression::Reinterpret {
                source,
                destination,
                value,
            } => Some((
                GoalOperation::Reinterpret {
                    source: *source,
                    destination: *destination,
                },
                vec![source.ty(), destination.ty()],
                Vec::new(),
                destination.ty(),
                std::slice::from_ref(value.as_ref()),
            )),
            CheckedExpression::BooleanOperation {
                operation,
                arguments,
            } => Some((
                GoalOperation::Boolean(*operation),
                Vec::new(),
                Vec::new(),
                CheckedType::Bool,
                arguments.as_slice(),
            )),
            CheckedExpression::EnumEquality {
                equal,
                operand_type,
                arguments,
            } => Some((
                GoalOperation::EnumEquality {
                    equal: *equal,
                    operand_type: *operand_type,
                },
                Vec::new(),
                Vec::new(),
                CheckedType::Bool,
                arguments.as_slice(),
            )),
            _ => None,
        };
        if let Some((row, type_arguments, const_arguments, result, checked_arguments)) = operation {
            if atoms.len() != checked_arguments.len() {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let arguments = atoms
                .into_iter()
                .zip(checked_arguments)
                .map(|(atom, argument)| {
                    self.build_goal_atom(atom, Some(argument), bindings, expanded_bindings)
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            });
        }

        if matches!(
            checked,
            CheckedExpression::ArrayLength { .. }
                | CheckedExpression::BufferLength { .. }
                | CheckedExpression::SliceLength { .. }
        ) {
            if atoms.len() != 1 {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let argument = self.build_goal_atom(atoms[0], None, bindings, expanded_bindings)?;
            let row = match (checked, argument.ty()) {
                (
                    CheckedExpression::ArrayLength { length, .. },
                    CheckedType::Array {
                        element,
                        length: argument_length,
                    },
                ) if argument_length == *length => GoalOperation::ArrayLength {
                    element,
                    length: *length,
                },
                (CheckedExpression::BufferLength { root }, CheckedType::Buffer { element })
                    if element == root.element =>
                {
                    GoalOperation::BufferLength { element }
                }
                (
                    CheckedExpression::SliceLength { root },
                    CheckedType::Slice { region, element },
                ) if expanded_bindings.get(&root.binding).is_some_and(|source| {
                    source.ty() == CheckedType::Slice { region, element }
                }) =>
                {
                    GoalOperation::SliceLength { region, element }
                }
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            return Ok(GoalExpression::Operation {
                row,
                type_arguments: Vec::new(),
                const_arguments: Vec::new(),
                result: CheckedType::Integer(super::super::model::IntegerType::U64),
                arguments: vec![argument],
            });
        }

        if atoms.len() != 1 {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        self.build_goal_atom(atoms[0], Some(checked), bindings, expanded_bindings)
    }

    fn goal_operand_atoms(&self, expression: NodeId) -> Result<Vec<NodeId>, CheckStop> {
        if let Some(tail) = self
            .tree
            .first_child_with(expression, Production::InfixTail)?
        {
            let left = self
                .tree
                .first_child_with(expression, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let right = self
                .tree
                .first_child_with(tail, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            return Ok(vec![left, right]);
        }
        if let Some(call) = self.tree.first_child_with(expression, Production::Call)? {
            let Some(list) = self.tree.first_child_with(call, Production::AtomList)? else {
                return Ok(Vec::new());
            };
            return self
                .tree
                .children_with(list, Production::Atom)
                .map_err(Into::into);
        }
        self.tree
            .first_child_with(expression, Production::Atom)?
            .map_or_else(|| Ok(Vec::new()), |atom| Ok(vec![atom]))
    }

    fn build_goal_atom(
        &self,
        atom: NodeId,
        checked: Option<&CheckedExpression>,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, GoalExpression>,
    ) -> Result<GoalExpression, CheckStop> {
        if self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
            .is_some()
        {
            let Some(CheckedExpression::Constant(value)) = checked else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            return Ok(GoalExpression::Datum(GoalDatum::Literal(value.clone())));
        }
        let place = self
            .tree
            .first_child_with(atom, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let result = self.build_goal_place(place, bindings, expanded_bindings)?;
        if checked.is_some_and(|expression| expression.ty() != result.ty()) {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(result)
    }

    fn build_goal_place(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, GoalExpression>,
    ) -> Result<GoalExpression, CheckStop> {
        let (expression, holder_pending) =
            self.build_goal_place_inner(place, bindings, expanded_bindings)?;
        if holder_pending {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(expression)
    }

    /// Mirrors the already-completed TYPE-7 dereference type walk while
    /// retaining only predicate identity. A borrow-holder dereference leaves
    /// the written referent type unchanged; an own box dereference selects the
    /// box nominal's referent type.
    fn build_goal_place_inner(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, GoalExpression>,
    ) -> Result<(GoalExpression, bool), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (mut expression, holder_pending) = if self.has_fixed(pbase, FixedTerminal::Deref)? {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let (nested, nested_holder_pending) =
                self.build_goal_place_inner(nested, bindings, expanded_bindings)?;
            let ty = if nested_holder_pending {
                nested.ty()
            } else {
                let CheckedType::Nominal(nominal) = nested.ty() else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let CheckedNominalKind::Box { referent } = self.nominal(nominal)?.kind else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                referent
            };
            (
                nested
                    .with_projection(GoalProjection::Deref, ty)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                false,
            )
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
                    (
                        expanded_bindings
                            .get(&local.binding)
                            .cloned()
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                        local.mode != CheckedMode::Own,
                    )
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

    /// Holds a clause local to [FN-8]'s "own copy value", judged on the type
    /// the checker derived for it.
    ///
    /// The mode half of that phrase needs no check — the grammar admits no
    /// written mode and [FN-8] fixes it — but the copy half is a real
    /// restriction that the deleted annotation used to carry. The admitted-row
    /// filter does not imply it: `array_new` and the `checked` arithmetic rows
    /// are pure, total and non-trapping, and yield an `array<T, N>` and a
    /// `Result<T, Overflow>` respectively.
    fn validate_requires_copy_local(
        &self,
        entry: NodeId,
        binding: BindingId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let local = bindings
            .values()
            .find(|local| local.binding == binding)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.is_copy_type(local.ty)? {
            return self.invalid_requires(entry);
        }
        Ok(())
    }

    fn validate_requires_statement(
        &self,
        entry: NodeId,
        statement: NodeId,
    ) -> Result<(), CheckStop> {
        match self.tree.production(statement)? {
            Production::LetStmt => self.validate_requires_let(entry, statement),
            Production::CheckStmt => {
                let expression = self
                    .tree
                    .first_child_with(statement, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                self.validate_requires_condition(entry, expression)
            }
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// Validates one clause `let` against the admitted [FN-8] subset.
    ///
    /// A clause `let` carries no written mode or type, so neither is read
    /// here. [FN-8] fixes the mode itself — "each let introduces a fresh
    /// clause-local own copy value visible to later clause statements" — and
    /// the type is derived from the initializer by the ordinary [TYPE-5]
    /// derivation that a function-body `let` uses, in `check_statement`
    /// immediately after this pass. What remains written, and what this pass
    /// examines, is the right-hand side: an ordinary initializer whose
    /// expression is a call to an admitted operation-table row.
    fn validate_requires_let(&self, entry: NodeId, node: NodeId) -> Result<(), CheckStop> {
        let rhs = self
            .tree
            .first_child_with(node, Production::OrdinaryLetRhs)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression = self
            .tree
            .first_child_with(rhs, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // A clause `let` initializer is a computation, so only [FN-8]'s two
        // operation spellings are admitted. A bare atom is not a computation,
        // which is what keeps `let candidate = x;` rejected.
        if self.validate_requires_computation(entry, expression)? {
            return Ok(());
        }
        self.invalid_requires(entry)
    }

    fn validate_requires_condition(
        &self,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<(), CheckStop> {
        if self.validate_requires_computation(entry, expression)? {
            return Ok(());
        }
        // [FN-8] admits one further shape here that a clause `let` does not:
        // "the final check condition is either a Bool clause atom or one such
        // operation returning Bool".
        let Some(atom) = self.tree.first_child_with(expression, Production::Atom)? else {
            return self.invalid_requires(entry);
        };
        self.validate_requires_atom(entry, atom)
    }

    /// Validates a clause computation, reporting whether the expression was
    /// one of the two spellings [FN-8] admits for it.
    ///
    /// [FN-8] requires "an ANF [GRAM-9] call to, or infix spelling of, a
    /// non-trapping, total operation-table row with effect `pure`", and
    /// [GRAM-5] gives those two spellings distinct `expr` shapes. `Ok(false)`
    /// means the expression is neither, leaving each caller to say whether
    /// its position admits a bare atom.
    fn validate_requires_computation(
        &self,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<bool, CheckStop> {
        if let Some(tail) = self
            .tree
            .first_child_with(expression, Production::InfixTail)?
        {
            self.validate_requires_infix(entry, expression, tail)?;
            return Ok(true);
        }
        if let Some(call) = self.tree.first_child_with(expression, Production::Call)? {
            self.validate_requires_operation(entry, call)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Validates the infix spelling of a row against the same [FN-8] subset
    /// the named spelling faces.
    ///
    /// The operator token selects the row under [OP-1] (ii), so admission
    /// asks the same `traps` predicate the ordinary checker asks rather than
    /// re-reading the spelling: the bare `+ - * / %` forms carry the trapping
    /// mode and no `.trap` suffix, so the named-spelling filter in
    /// `validate_requires_operation` cannot see them. Both operands are
    /// clause atoms and both are validated; [GRAM-9] admits exactly one
    /// operation per expression, so there is no deeper operand to reach.
    fn validate_requires_infix(
        &self,
        entry: NodeId,
        expression: NodeId,
        tail: NodeId,
    ) -> Result<(), CheckStop> {
        let operator = self
            .tree
            .first_child_with(tail, Production::InfixOp)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.infix_operation(operator)?.traps() {
            return self.invalid_requires(entry);
        }
        let left = self
            .tree
            .first_child_with(expression, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_requires_atom(entry, left)?;
        let right = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_requires_atom(entry, right)
    }

    fn validate_requires_operation(&self, entry: NodeId, call: NodeId) -> Result<(), CheckStop> {
        let callee = self
            .tree
            .first_child_with(call, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let callee_path = self.tree.path(callee)?;
        let usage = self
            .resolved
            .lexical_uses()
            .iter()
            .find(|usage| {
                usage.origin().node() == callee_path
                    && matches!(
                        usage.role(),
                        LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee
                    )
            })
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let ResolvedTarget::Operation(operation) = usage.target() else {
            // FN-8 admits only table-operation calls. User and system
            // callees have already resolved successfully, so they are an
            // InvalidRequires source form rather than a compiler-resolution
            // failure.
            return self.invalid_requires(entry);
        };
        let spelling = crate::operation_family_spelling(operation)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if spelling.ends_with(".trap") || matches!(spelling, "buffer_new" | "box_new" | "arena_new")
        {
            return self.invalid_requires(entry);
        }
        if let Some(arguments) = self.tree.first_child_with(call, Production::AtomList)? {
            for atom in self.tree.children_with(arguments, Production::Atom)? {
                self.validate_requires_atom(entry, atom)?;
            }
        }
        Ok(())
    }

    fn validate_requires_atom(&self, entry: NodeId, atom: NodeId) -> Result<(), CheckStop> {
        if self.has_fixed(atom, FixedTerminal::Move)?
            || self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some()
        {
            return self.invalid_requires(entry);
        }
        if let Some(place) = self.tree.first_child_with(atom, Production::Place)? {
            return self.validate_requires_place(entry, place);
        }
        if self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
            .is_some()
        {
            return Ok(());
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    fn validate_requires_place(&self, entry: NodeId, place: NodeId) -> Result<(), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        for suffix in self.tree.children_with(place, Production::Psuffix)? {
            if self.subscript_offset(suffix)?.is_some() {
                return self.invalid_requires(entry);
            }
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.validate_requires_place(entry, nested)?;
        }
        Ok(())
    }

    fn invalid_requires<T>(&self, node: NodeId) -> Result<T, CheckStop> {
        self.issue_node(SemanticRule::Fn8, node, SemanticIssueKind::InvalidRequires)
    }
}
