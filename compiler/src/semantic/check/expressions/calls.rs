use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, ProductionV0_14, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_14, UnsupportedSemanticFeatureV0_14,
};

use super::super::super::model::{
    CheckedBooleanOperation, CheckedExpression, CheckedIntegerOperation, CheckedNominalKind,
    CheckedType, TrapSite,
};
use super::super::{
    CheckStop, Checker, FunctionSignature, LocalBinding, PreludeType, TypedExpression,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(in crate::semantic::check) fn check_call(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let callee = self
            .tree
            .first_child_with(node, ProductionV0_14::Callee)?
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
        match usage.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Function,
            } => self.check_user_call(node, declaration, function, bindings, loop_depth),
            ResolvedTarget::Operation(operation) => {
                self.check_operation(node, operation, function, bindings, loop_depth)
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    fn check_user_call(
        &self,
        node: NodeId,
        declaration: DeclarationId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(targs) = self.tree.first_child_with(node, ProductionV0_14::Targs)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::Generics, targs);
        }
        let target = *self
            .functions_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let signature = self
            .signatures
            .get(target.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let fields = if let Some(list) = self
            .tree
            .first_child_with(node, ProductionV0_14::FieldinitList)?
        {
            self.tree.children_with(list, ProductionV0_14::Fieldinit)?
        } else {
            Vec::new()
        };
        if self
            .tree
            .first_child_with(node, ProductionV0_14::AtomList)?
            .is_some()
            || fields.len() != signature.parameters.len()
        {
            return self.issue_node(
                SemanticRuleV0_14::Gram11,
                node,
                Self::invalid_named_arguments(signature),
            );
        }
        let mut arguments = Vec::with_capacity(fields.len());
        let mut exhibits_traps = signature.declared_traps;
        for (field, parameter) in fields.into_iter().zip(&signature.parameters) {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(
                    SemanticRuleV0_14::Gram11,
                    field,
                    Self::invalid_named_arguments(signature),
                );
            }
            let atom = self
                .tree
                .first_child_with(field, ProductionV0_14::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if argument.expression.ty() != parameter.ty {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    atom,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            exhibits_traps |= argument.exhibits_traps;
            arguments.push(argument.expression);
        }
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                arguments,
                result: signature.result,
            },
            exhibits_traps,
        })
    }

    fn check_operation(
        &self,
        node: NodeId,
        operation_id: crate::OperationFamilyId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let spelling = crate::operation_family_spelling_v0_14(operation_id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if matches!(spelling, "band" | "bor" | "bxor" | "bnot") {
            return self.check_boolean_operation(node, spelling, function, bindings, loop_depth);
        }
        if matches!(spelling, "eeq" | "ene") {
            return self.check_enum_equality(
                node,
                spelling == "eeq",
                function,
                bindings,
                loop_depth,
            );
        }
        let operation = match spelling {
            "iadd.wrap" => CheckedIntegerOperation::AddWrap,
            "isub.wrap" => CheckedIntegerOperation::SubtractWrap,
            "imul.wrap" => CheckedIntegerOperation::MultiplyWrap,
            "iadd.trap" => CheckedIntegerOperation::AddTrap,
            "isub.trap" => CheckedIntegerOperation::SubtractTrap,
            "imul.trap" => CheckedIntegerOperation::MultiplyTrap,
            "iadd.checked" => CheckedIntegerOperation::AddChecked,
            "isub.checked" => CheckedIntegerOperation::SubtractChecked,
            "imul.checked" => CheckedIntegerOperation::MultiplyChecked,
            "idiv.checked" => CheckedIntegerOperation::DivideChecked,
            "irem.checked" => CheckedIntegerOperation::RemainderChecked,
            "idiv.trap" => CheckedIntegerOperation::DivideTrap,
            "irem.trap" => CheckedIntegerOperation::RemainderTrap,
            "iabs.wrap" => CheckedIntegerOperation::AbsoluteWrap,
            "iabs.trap" => CheckedIntegerOperation::AbsoluteTrap,
            "iabs.checked" => CheckedIntegerOperation::AbsoluteChecked,
            "ineg.wrap" => CheckedIntegerOperation::NegateWrap,
            "ineg.trap" => CheckedIntegerOperation::NegateTrap,
            "ineg.checked" => CheckedIntegerOperation::NegateChecked,
            "iand" => CheckedIntegerOperation::BitAnd,
            "ior" => CheckedIntegerOperation::BitOr,
            "ixor" => CheckedIntegerOperation::BitXor,
            "inot" => CheckedIntegerOperation::BitNot,
            "ishl.wrap" => CheckedIntegerOperation::ShiftLeftWrap,
            "ishr.wrap" => CheckedIntegerOperation::ShiftRightWrap,
            "ishl.trap" => CheckedIntegerOperation::ShiftLeftTrap,
            "ishr.trap" => CheckedIntegerOperation::ShiftRightTrap,
            "irotl" => CheckedIntegerOperation::RotateLeft,
            "irotr" => CheckedIntegerOperation::RotateRight,
            "ipopcount" => CheckedIntegerOperation::PopulationCount,
            "iclz" => CheckedIntegerOperation::LeadingZeros,
            "ictz" => CheckedIntegerOperation::TrailingZeros,
            "ibswap" => CheckedIntegerOperation::ByteSwap,
            "imulhi" => CheckedIntegerOperation::MultiplyHigh,
            "iadd.sat" => CheckedIntegerOperation::AddSaturating,
            "isub.sat" => CheckedIntegerOperation::SubtractSaturating,
            "imul.sat" => CheckedIntegerOperation::MultiplySaturating,
            "imin" => CheckedIntegerOperation::Minimum,
            "imax" => CheckedIntegerOperation::Maximum,
            "ieq" => CheckedIntegerOperation::Equal,
            "ine" => CheckedIntegerOperation::NotEqual,
            "ilt" => CheckedIntegerOperation::Less,
            "ile" => CheckedIntegerOperation::LessEqual,
            "igt" => CheckedIntegerOperation::Greater,
            "ige" => CheckedIntegerOperation::GreaterEqual,
            _ => {
                return self.unsupported(UnsupportedSemanticFeatureV0_14::OperationFamily, node);
            }
        };
        if self
            .tree
            .first_child_with(node, ProductionV0_14::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRuleV0_14::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: spelling.to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        let targs = self
            .tree
            .first_child_with(node, ProductionV0_14::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Fn2,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let targs = self.tree.children_with(targs, ProductionV0_14::Targ)?;
        if targs.len() != 1 {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let type_node = self
            .tree
            .first_child_with(targs[0], ProductionV0_14::Type)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let Some(operand_type) = self.integer_type(type_node)? else {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        };
        if !operation.accepts_operand_type(operand_type) {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let operand_count = operation.operand_count();
        let atoms = self.operation_atoms(node, operand_count)?;
        let mut arguments = Vec::with_capacity(operand_count);
        let mut exhibits_traps = operation.traps();
        for (index, atom) in atoms.into_iter().enumerate() {
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if Some(argument.expression.ty()) != operation.argument_type(operand_type, index) {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    atom,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            exhibits_traps |= argument.exhibits_traps;
            arguments.push(argument.expression);
        }
        let trap = if operation.traps() {
            Some(TrapSite {
                rule_id: if matches!(
                    operation,
                    CheckedIntegerOperation::ShiftLeftTrap
                        | CheckedIntegerOperation::ShiftRightTrap
                ) {
                    "OP-8"
                } else {
                    "OP-2"
                },
                message: if matches!(
                    operation,
                    CheckedIntegerOperation::AddTrap
                        | CheckedIntegerOperation::SubtractTrap
                        | CheckedIntegerOperation::MultiplyTrap
                ) {
                    "integer overflow".to_owned()
                } else {
                    String::new()
                },
                function: function.name.clone(),
                node_path: self.tree.path(node)?.clone(),
            })
        } else {
            None
        };
        let checked_error =
            match operation {
                CheckedIntegerOperation::AddChecked
                | CheckedIntegerOperation::SubtractChecked
                | CheckedIntegerOperation::MultiplyChecked => Some(PreludeType::Overflow),
                CheckedIntegerOperation::DivideChecked
                | CheckedIntegerOperation::RemainderChecked => Some(PreludeType::DivError),
                CheckedIntegerOperation::AbsoluteChecked
                | CheckedIntegerOperation::NegateChecked => Some(PreludeType::Overflow),
                _ => None,
            };
        let result = if let Some(error) = checked_error {
            CheckedType::Nominal(self.prelude_nominal(PreludeType::Result(
                CheckedType::Integer(operand_type),
                CheckedType::Nominal(self.prelude_nominal(error)?),
            ))?)
        } else {
            operation
                .scalar_result_type(operand_type)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
        };
        Ok(TypedExpression {
            expression: CheckedExpression::IntegerOperation {
                operation,
                operand_type,
                arguments,
                result,
                trap,
            },
            exhibits_traps,
        })
    }

    fn check_boolean_operation(
        &self,
        node: NodeId,
        spelling: &str,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let operation = match spelling {
            "band" => CheckedBooleanOperation::And,
            "bor" => CheckedBooleanOperation::Or,
            "bxor" => CheckedBooleanOperation::ExclusiveOr,
            "bnot" => CheckedBooleanOperation::Not,
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        if self.operation_type_argument(node, spelling)? != CheckedType::Bool {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let expected = usize::from(operation != CheckedBooleanOperation::Not) + 1;
        let atoms = self.operation_atoms(node, expected)?;
        let mut arguments = Vec::with_capacity(atoms.len());
        let mut exhibits_traps = false;
        for atom in atoms {
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if argument.expression.ty() != CheckedType::Bool {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    atom,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            exhibits_traps |= argument.exhibits_traps;
            arguments.push(argument.expression);
        }
        Ok(TypedExpression {
            expression: CheckedExpression::BooleanOperation {
                operation,
                arguments,
            },
            exhibits_traps,
        })
    }

    fn check_enum_equality(
        &self,
        node: NodeId,
        equal: bool,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let operand_type = self.operation_type_argument(node, if equal { "eeq" } else { "ene" })?;
        let tag_only = match operand_type {
            CheckedType::Bool => true,
            CheckedType::Nominal(id) => matches!(
                &self.nominal(id)?.kind,
                CheckedNominalKind::Enum { variants }
                    if variants.iter().all(|variant| variant.fields.is_empty())
            ),
            _ => false,
        };
        if !tag_only {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let atoms = self.operation_atoms(node, 2)?;
        let mut arguments = Vec::with_capacity(2);
        let mut exhibits_traps = false;
        for atom in atoms {
            let argument = self.check_atom(function, atom, bindings, loop_depth)?;
            if argument.expression.ty() != operand_type {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    atom,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            exhibits_traps |= argument.exhibits_traps;
            arguments.push(argument.expression);
        }
        Ok(TypedExpression {
            expression: CheckedExpression::EnumEquality {
                equal,
                operand_type,
                arguments,
            },
            exhibits_traps,
        })
    }

    fn operation_type_argument(
        &self,
        node: NodeId,
        spelling: &str,
    ) -> Result<CheckedType, CheckStop> {
        if self
            .tree
            .first_child_with(node, ProductionV0_14::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRuleV0_14::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: spelling.to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        let targs = self
            .tree
            .first_child_with(node, ProductionV0_14::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Fn2,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let targs = self.tree.children_with(targs, ProductionV0_14::Targ)?;
        if targs.len() != 1 {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        let ty = self
            .tree
            .first_child_with(targs[0], ProductionV0_14::Type)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        self.parse_type(ty)
    }

    fn invalid_named_arguments(signature: &FunctionSignature) -> SemanticIssueKind {
        SemanticIssueKind::InvalidNamedArguments {
            callee: signature.name.clone(),
            declared_parameters: signature
                .parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
        }
    }

    fn operation_atoms(&self, node: NodeId, expected: usize) -> Result<Vec<NodeId>, CheckStop> {
        let Some(list) = self
            .tree
            .first_child_with(node, ProductionV0_14::AtomList)?
        else {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        };
        let atoms = self.tree.children_with(list, ProductionV0_14::Atom)?;
        if atoms.len() < expected {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        }
        if atoms.len() > expected {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                atoms[expected],
                SemanticIssueKind::InvalidOperation,
            );
        }
        Ok(atoms)
    }
}
