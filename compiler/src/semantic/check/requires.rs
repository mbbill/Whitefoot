use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, PostconditionResolutionRecord, Production,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::goal::{
    CheckedRequirement, GoalDatum, GoalExpression, GoalOperation, GoalProjection, GoalTemplate,
};
use super::super::model::{
    BindingId, CheckedConst, CheckedExpression, CheckedFloatOperation, CheckedMode,
    CheckedNominalKind, CheckedStatement, CheckedType, CheckedValue,
};
use super::super::postcondition::PostconditionConstantOrigin;
use super::{CheckStop, Checker, ControlCounters, ControlScope, FunctionSignature, LocalBinding};

pub(super) struct CheckedRequires {
    pub(super) requirements: Vec<CheckedRequirement>,
}

#[derive(Clone, Copy)]
pub(super) enum ClauseKind<'record> {
    Requires,
    Postcondition(&'record PostconditionResolutionRecord),
}

/// One source-stable leaf shared by FN-8 and FN-9 alpha expansion.  The
/// symbolic result is private to this intermediate tree; conversion to a
/// GoalTemplate rejects it, so GoalDatum and GoalTemplate remain unchanged.
#[derive(Clone, Debug)]
pub(super) enum ExpandedClauseDatum {
    Parameter {
        ordinal: u32,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    NamedConst {
        declaration: DeclarationId,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    Literal {
        value: CheckedValue,
        origin: PostconditionConstantOrigin,
    },
    Result {
        ty: CheckedType,
    },
}

impl ExpandedClauseDatum {
    pub(super) const fn ty(&self) -> CheckedType {
        match self {
            Self::Parameter { ty, .. } | Self::NamedConst { ty, .. } | Self::Result { ty } => *ty,
            Self::Literal { value, .. } => value.ty(),
        }
    }

    fn with_projection(mut self, projection: GoalProjection, ty: CheckedType) -> Option<Self> {
        match &mut self {
            Self::Parameter {
                projections,
                ty: datum_ty,
                ..
            }
            | Self::NamedConst {
                projections,
                ty: datum_ty,
                ..
            } => {
                projections.push(projection);
                *datum_ty = ty;
                Some(self)
            }
            Self::Literal { .. } | Self::Result { .. } => None,
        }
    }
}

/// The one alpha-expanded expression representation used by both clause
/// families.  FN-8 converts the complete tree to GoalExpression; FN-9 admits
/// only a comparison root whose two children downcast to closed datums.
#[derive(Clone, Debug)]
pub(super) enum ExpandedClauseExpression {
    Datum(ExpandedClauseDatum),
    Operation {
        row: GoalOperation,
        type_arguments: Vec<CheckedType>,
        const_arguments: Vec<CheckedConst>,
        result: CheckedType,
        arguments: Vec<Self>,
    },
    InvalidSelectorUse {
        ty: CheckedType,
    },
}

impl ExpandedClauseExpression {
    pub(super) const fn ty(&self) -> CheckedType {
        match self {
            Self::Datum(datum) => datum.ty(),
            Self::Operation { result, .. } | Self::InvalidSelectorUse { ty: result } => *result,
        }
    }

    pub(super) fn contains_invalid_selector_use(&self) -> bool {
        match self {
            Self::InvalidSelectorUse { .. } => true,
            Self::Operation { arguments, .. } => {
                arguments.iter().any(Self::contains_invalid_selector_use)
            }
            Self::Datum(_) => false,
        }
    }

    fn with_projection(self, projection: GoalProjection, ty: CheckedType) -> Option<Self> {
        let Self::Datum(datum) = self else {
            return None;
        };
        datum.with_projection(projection, ty).map(Self::Datum)
    }

    fn into_goal_expression(self) -> Option<GoalExpression> {
        match self {
            Self::Datum(ExpandedClauseDatum::Parameter {
                ordinal,
                projections,
                ty,
            }) => Some(GoalExpression::Datum(GoalDatum::Parameter {
                ordinal,
                projections,
                ty,
            })),
            Self::Datum(ExpandedClauseDatum::NamedConst {
                declaration,
                projections,
                ty,
            }) => Some(GoalExpression::Datum(GoalDatum::NamedConst {
                declaration,
                projections,
                ty,
            })),
            Self::Datum(ExpandedClauseDatum::Literal { value, .. }) => {
                Some(GoalExpression::Datum(GoalDatum::Literal(value)))
            }
            Self::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments,
            } => Some(GoalExpression::Operation {
                row,
                type_arguments,
                const_arguments,
                result,
                arguments: arguments
                    .into_iter()
                    .map(Self::into_goal_expression)
                    .collect::<Option<Vec<_>>>()?,
            }),
            Self::Datum(ExpandedClauseDatum::Result { .. }) | Self::InvalidSelectorUse { .. } => {
                None
            }
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_requires(
        &self,
        function: &FunctionSignature,
        block: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
    ) -> Result<CheckedRequires, CheckStop> {
        let mut expanded_bindings = HashMap::new();
        for (ordinal, parameter) in function.parameters.iter().enumerate() {
            let local = bindings
                .get(&parameter.declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            expanded_bindings.insert(
                local.binding,
                ExpandedClauseExpression::Datum(ExpandedClauseDatum::Parameter {
                    ordinal,
                    projections: Vec::new(),
                    ty: parameter.ty,
                }),
            );
        }
        for definition in self.tree.children_with(block, Production::ContractDefine)? {
            let expression = self
                .tree
                .first_child_with(definition, Production::Expr)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if !self.validate_clause_computation(ClauseKind::Requires, definition, expression)? {
                return self.invalid_clause(ClauseKind::Requires, definition);
            }
            let checked = self
                .check_statement(
                    function,
                    definition,
                    bindings,
                    counters,
                    ControlScope {
                        loops: &[],
                        give_context: None,
                    },
                )
                .map_err(Self::clause_conditional_repair)?;
            if !checked.can_continue {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let CheckedStatement::Let { binding, value, .. } = &checked.statement else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            self.validate_clause_copy_local(ClauseKind::Requires, definition, *binding, bindings)?;
            let expanded =
                self.build_clause_expression(expression, value, bindings, &expanded_bindings)?;
            expanded_bindings.insert(*binding, expanded);
        }

        let mut requirements = Vec::new();
        for clause in self.tree.children_with(block, Production::RequiresClause)? {
            let expression = self
                .tree
                .first_child_with(clause, Production::ClauseExpr)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.validate_clause_condition(ClauseKind::Requires, clause, expression)?;
            let condition = self
                .check_expression(function, expression, bindings, 0)
                .map_err(Self::clause_conditional_repair)?;
            if condition.mode != CheckedMode::Own || condition.expression.ty() != CheckedType::Bool
            {
                return self.issue_node(
                    SemanticRule::Op5,
                    expression,
                    SemanticIssueKind::InvalidPredicateCondition,
                );
            }
            let root = self
                .build_clause_expression(
                    expression,
                    &condition.expression,
                    bindings,
                    &expanded_bindings,
                )?
                .into_goal_expression()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            requirements.push(CheckedRequirement {
                template: GoalTemplate::new(root),
                clause: self.tree.path(clause)?.clone(),
            });
        }
        Ok(CheckedRequires { requirements })
    }

    /// The contract-conditional OWN-1 bare-affine repair [#35]. OWN-1's
    /// ordinary mechanical fix is `write move p`, but [FN-8] rejects `move`
    /// inside a contract block, so that instruction would send the writer
    /// from one hard error to another. A definition or clause instead carries
    /// the contract-specific repair.
    /// Inert while `V031_CANDIDATE_SEMANTICS` is false.
    fn clause_conditional_repair(stop: CheckStop) -> CheckStop {
        if !crate::semantic::V031_CANDIDATE_SEMANTICS {
            return stop;
        }
        let CheckStop::Issue(mut issue) = stop else {
            return stop;
        };
        if matches!(issue.kind, SemanticIssueKind::BareAffineUse { .. })
            && matches!(issue.rule, SemanticRule::Own1)
        {
            issue.kind = SemanticIssueKind::BareAffineUse {
                mechanical_fix: "restate the definition or clause over copy operands or non-consuming admitted reads",
            };
        }
        CheckStop::Issue(issue)
    }

    /// Alpha-expands one already-checked admitted FN-8/FN-9 expression. Source
    /// atoms supply declaration/projection identity; the checked expression
    /// supplies the uniquely selected row and types.
    pub(super) fn build_clause_expression(
        &self,
        source: NodeId,
        checked: &CheckedExpression,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, ExpandedClauseExpression>,
    ) -> Result<ExpandedClauseExpression, CheckStop> {
        let atoms = self.clause_operand_atoms(source)?;
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
                ..
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
                ..
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
                ..
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
                ..
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
                ..
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
            CheckedExpression::BufferFits {
                element,
                layout_ceiling,
                length,
                ..
            } => Some((
                GoalOperation::BufferFits {
                    element: *element,
                    maximum_length: layout_ceiling.stride.allocation_limit(),
                },
                vec![*element],
                Vec::new(),
                CheckedType::Bool,
                std::slice::from_ref(length.as_ref()),
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
                    self.build_clause_operand(atom, Some(argument), bindings, expanded_bindings)
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(ExpandedClauseExpression::Operation {
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
            let argument = self.build_clause_atom(atoms[0], None, bindings, expanded_bindings)?;
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
                (CheckedExpression::BufferLength { root, .. }, CheckedType::Buffer { element })
                    if element == root.element =>
                {
                    GoalOperation::BufferLength { element }
                }
                (
                    CheckedExpression::SliceLength { root, .. },
                    CheckedType::Slice { region, element },
                ) if expanded_bindings.get(&root.binding).is_some_and(|source| {
                    source.ty() == CheckedType::Slice { region, element }
                }) =>
                {
                    GoalOperation::SliceLength { region, element }
                }
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            return Ok(ExpandedClauseExpression::Operation {
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
        self.build_clause_operand(atoms[0], Some(checked), bindings, expanded_bindings)
    }

    /// One written clause operand. An `atom` is a leaf datum; every other
    /// written form — today exactly a `call`, which is how [MSR-5] admits a
    /// measure term as an operand — is expanded by the ordinary clause walk
    /// against the row the typer already selected for it.
    fn build_clause_operand(
        &self,
        node: NodeId,
        checked: Option<&CheckedExpression>,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, ExpandedClauseExpression>,
    ) -> Result<ExpandedClauseExpression, CheckStop> {
        if self.tree.production(node)? == Production::Atom {
            return self.build_clause_atom(node, checked, bindings, expanded_bindings);
        }
        let checked = checked.ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.build_clause_expression(node, checked, bindings, expanded_bindings)
    }

    pub(super) fn clause_operand_atoms(
        &self,
        expression: NodeId,
    ) -> Result<Vec<NodeId>, CheckStop> {
        // [GRAM-5] a `clause_expr` carries its operands directly, and each
        // one is an `atom`, a `call`, or a `construct`. A single-operand
        // clause reads through to that operand, so a bare `len(P)` clause
        // operand and a `len(P)` operand of a comparison are one path.
        match self.tree.production(expression)? {
            Production::ClauseExpr => {
                return match self.tree.children(expression)? {
                    [only] => self.clause_operand_atoms(*only),
                    [left, _operator, right] => Ok(vec![*left, *right]),
                    _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
                };
            }
            Production::Atom => return Ok(vec![expression]),
            Production::Call => {
                let Some(list) = self
                    .tree
                    .first_child_with(expression, Production::AtomList)?
                else {
                    return Ok(Vec::new());
                };
                return self
                    .tree
                    .children_with(list, Production::Atom)
                    .map_err(Into::into);
            }
            Production::Construct => return Ok(Vec::new()),
            _ => {}
        }
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

    fn build_clause_atom(
        &self,
        atom: NodeId,
        checked: Option<&CheckedExpression>,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, ExpandedClauseExpression>,
    ) -> Result<ExpandedClauseExpression, CheckStop> {
        if self.postcondition_selector_use_inside(atom)? {
            let ty = self
                .active_postcondition
                .get()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .result_type;
            return Ok(if self.postcondition_selector_is_bare_atom(atom)? {
                ExpandedClauseExpression::Datum(ExpandedClauseDatum::Result { ty })
            } else {
                ExpandedClauseExpression::InvalidSelectorUse {
                    ty: checked.map_or(ty, CheckedExpression::ty),
                }
            });
        }
        if let Some(literal) = self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
        {
            let Some(CheckedExpression::Constant(value)) = checked else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let bytes = self.tree.token_bytes(literal)?;
            let origin = if matches!(bytes, b"0_T" | b"1_T") {
                let usage = self.use_at(atom, LexicalUseRole::GenericNumericSuffix)?;
                let ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::GenericType,
                } = usage.target()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                PostconditionConstantOrigin::GenericNumericIdentity {
                    type_parameter: declaration,
                    one: bytes == b"1_T",
                }
            } else {
                PostconditionConstantOrigin::Literal
            };
            return Ok(ExpandedClauseExpression::Datum(
                ExpandedClauseDatum::Literal {
                    value: value.clone(),
                    origin,
                },
            ));
        }
        let place = self
            .tree
            .first_child_with(atom, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let result = self.build_clause_place(place, bindings, expanded_bindings)?;
        if checked.is_some_and(|expression| expression.ty() != result.ty()) {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(result)
    }

    fn build_clause_place(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, ExpandedClauseExpression>,
    ) -> Result<ExpandedClauseExpression, CheckStop> {
        let (expression, holder_pending) =
            self.build_clause_place_inner(place, bindings, expanded_bindings)?;
        if holder_pending {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(expression)
    }

    /// Mirrors the already-completed TYPE-7 dereference type walk while
    /// retaining only predicate identity. A borrow-holder dereference leaves
    /// the written referent type unchanged; an own box dereference selects the
    /// box nominal's referent type.
    fn build_clause_place_inner(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        expanded_bindings: &HashMap<BindingId, ExpandedClauseExpression>,
    ) -> Result<(ExpandedClauseExpression, bool), CheckStop> {
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
                self.build_clause_place_inner(nested, bindings, expanded_bindings)?;
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
                        ExpandedClauseExpression::Datum(ExpandedClauseDatum::NamedConst {
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
    pub(super) fn validate_clause_copy_local(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        binding: BindingId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let local = bindings
            .values()
            .find(|local| local.binding == binding)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.is_copy_type(local.ty)? {
            return self.invalid_clause(clause, entry);
        }
        Ok(())
    }

    pub(super) fn validate_clause_condition(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<(), CheckStop> {
        if self.validate_clause_computation(clause, entry, expression)? {
            return Ok(());
        }
        // [FN-8] admits one further shape here that a contract definition does
        // not: a predicate may be either a Bool clause atom or one admitted
        // operation returning Bool.
        let Some(atom) = self.tree.first_child_with(expression, Production::Atom)? else {
            return self.invalid_clause(clause, entry);
        };
        self.validate_clause_atom(clause, entry, atom)
    }

    /// Validates a clause computation, reporting whether the expression was
    /// one of the two spellings [FN-8] admits for it.
    ///
    /// [FN-8] requires "an ANF [GRAM-9] call to, or infix spelling of, a
    /// non-trapping, total operation-table row with effect `pure`", and
    /// [GRAM-5] gives those two spellings distinct `expr` shapes. `Ok(false)`
    /// means the expression is neither, leaving each caller to say whether
    /// its position admits a bare atom.
    pub(super) fn validate_clause_computation(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<bool, CheckStop> {
        if let Some(tail) = self
            .tree
            .first_child_with(expression, Production::InfixTail)?
        {
            self.validate_clause_infix(clause, entry, expression, tail)?;
            return Ok(true);
        }
        if let Some(call) = self.tree.first_child_with(expression, Production::Call)? {
            self.validate_clause_operation(clause, entry, call)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Validates the infix spelling of a row against the same [FN-8] subset
    /// the named spelling faces.
    ///
    /// The operator token selects the row under [OP-1] (ii), so admission
    /// asks whether the selected row is proof-required exact rather than
    /// re-reading its spelling. Both operands are
    /// clause atoms and both are validated; [GRAM-9] admits exactly one
    /// operation per expression, so there is no deeper operand to reach.
    fn validate_clause_infix(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        expression: NodeId,
        tail: NodeId,
    ) -> Result<(), CheckStop> {
        let operator = self.infix_operator_node(tail)?;
        if self.infix_operation(operator)?.is_exact() {
            return self.invalid_clause(clause, entry);
        }
        let left = self
            .tree
            .first_child_with(expression, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_clause_atom(clause, entry, left)?;
        let right = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_clause_atom(clause, entry, right)
    }

    fn validate_clause_operation(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        call: NodeId,
    ) -> Result<(), CheckStop> {
        let callee = self
            .tree
            .first_child_with(call, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let callee_path = self.tree.path(callee)?;
        let usage = match clause {
            ClauseKind::Requires => self.resolved.lexical_uses().iter().find(|usage| {
                usage.origin().node() == callee_path
                    && matches!(
                        usage.role(),
                        LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee
                    )
            }),
            ClauseKind::Postcondition(record) => record
                .provisional_uses
                .iter()
                .chain(self.resolved.lexical_uses())
                .find(|usage| {
                    usage.origin().node() == callee_path
                        && matches!(
                            usage.role(),
                            LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee
                        )
                }),
        }
        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let ResolvedTarget::Operation(operation) = usage.target() else {
            // FN-8 admits only table-operation calls. User and system
            // callees have already resolved successfully, so they are an
            // InvalidRequires source form rather than a compiler-resolution
            // failure.
            return self.invalid_clause(clause, entry);
        };
        let spelling = crate::operation_family_spelling(operation)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if matches!(
            spelling,
            "ineg" | "iabs" | "ishl" | "ishr" | "buffer_new" | "box_new" | "arena_new"
        ) {
            return self.invalid_clause(clause, entry);
        }
        if let Some(arguments) = self.tree.first_child_with(call, Production::AtomList)? {
            for atom in self.tree.children_with(arguments, Production::Atom)? {
                self.validate_clause_atom(clause, entry, atom)?;
            }
        }
        Ok(())
    }

    fn validate_clause_atom(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        atom: NodeId,
    ) -> Result<(), CheckStop> {
        if self.has_fixed(atom, FixedTerminal::Move)?
            || self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some()
        {
            return self.invalid_clause(clause, entry);
        }
        if let Some(place) = self.tree.first_child_with(atom, Production::Place)? {
            return self.validate_clause_place(clause, entry, place);
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

    fn validate_clause_place(
        &self,
        clause: ClauseKind<'_>,
        entry: NodeId,
        place: NodeId,
    ) -> Result<(), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        for suffix in self.tree.children_with(place, Production::Psuffix)? {
            if self.subscript_offset(suffix)?.is_some() {
                return self.invalid_clause(clause, entry);
            }
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.validate_clause_place(clause, entry, nested)?;
        }
        Ok(())
    }

    pub(super) fn invalid_clause<T>(
        &self,
        clause: ClauseKind<'_>,
        node: NodeId,
    ) -> Result<T, CheckStop> {
        match clause {
            ClauseKind::Requires => {
                self.issue_node(SemanticRule::Fn8, node, SemanticIssueKind::InvalidRequires)
            }
            ClauseKind::Postcondition(_) => self.issue_node(
                SemanticRule::Fn9,
                node,
                SemanticIssueKind::InvalidPostconditionClause,
            ),
        }
    }
}
