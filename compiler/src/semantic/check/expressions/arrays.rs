use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminalV0_14;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, ProductionV0_14, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_14, UnsupportedSemanticFeatureV0_14,
};

use super::super::super::model::{
    CheckedArrayElement, CheckedArrayRoot, CheckedExpression, CheckedType, IntegerType, TrapSite,
};
use super::super::{CheckStop, Checker, FunctionSignature, LocalBinding, TypedExpression};
use super::PlaceUseOptions;

#[derive(Clone, Copy)]
pub(super) struct CheckedArrayPlace {
    pub(super) root: CheckedArrayRoot,
    element_type: CheckedType,
    length: u64,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(in crate::semantic::check) fn check_array_new(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, ProductionV0_14::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRuleV0_14::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: "array_new".to_owned(),
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
        let [element_arg, length_arg] = targs.as_slice() else {
            return self.issue_node(
                SemanticRuleV0_14::Op1,
                node,
                SemanticIssueKind::InvalidOperation,
            );
        };
        let element_node = self
            .tree
            .first_child_with(*element_arg, ProductionV0_14::Type)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Op1,
                    *element_arg,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let element_type = self.parse_type(element_node)?;
        let element = match element_type {
            CheckedType::Unit => CheckedArrayElement::Unit,
            CheckedType::Integer(ty) => CheckedArrayElement::Integer(ty),
            _ => {
                return self.issue_node(
                    SemanticRuleV0_14::Op1,
                    element_node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let length_node = self
            .tree
            .first_child_with(*length_arg, ProductionV0_14::Const)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Op1,
                    *length_arg,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let length = self.parse_const_expression(length_node)?;
        let atoms = self.operation_atoms(node, 1)?;
        let value = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if value.expression.ty() != element_type {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
        Ok(TypedExpression {
            expression: CheckedExpression::ArrayFill {
                ty: CheckedType::Array { element, length },
                value: Box::new(value.expression),
            },
            exhibits_traps: value.exhibits_traps,
        })
    }

    pub(in crate::semantic::check) fn check_array_length(
        &self,
        node: NodeId,
        _function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        _loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let element_type = self.operation_type_argument(node, "len")?;
        let atoms = self.operation_atoms(node, 1)?;
        let array = self.check_array_atom_place(atoms[0], bindings)?;
        if array.element_type != element_type {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
        Ok(TypedExpression {
            expression: CheckedExpression::ArrayLength {
                root: array.root,
                length: array.length,
            },
            exhibits_traps: false,
        })
    }

    pub(super) fn check_index_use(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        place: NodeId,
        pbase: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        if !self
            .tree
            .children_with(place, ProductionV0_14::Psuffix)?
            .is_empty()
        {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                place,
                SemanticIssueKind::TypeMismatch,
            );
        }
        if options.explicit_move {
            return self.issue_node(
                SemanticRuleV0_14::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the indexed copy place without `move`",
                },
            );
        }
        let selected = self
            .tree
            .first_child_with(pbase, ProductionV0_14::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let selected = self.parse_type(selected)?;
        let base = self
            .tree
            .first_child_with(pbase, ProductionV0_14::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let array = self.check_array_place(base, bindings)?;
        if selected != array.element_type {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                pbase,
                SemanticIssueKind::TypeMismatch,
            );
        }
        let offset = self
            .tree
            .first_child_with(pbase, ProductionV0_14::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let offset = self.check_atom(function, offset, bindings, options.loop_depth)?;
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64) {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                pbase,
                SemanticIssueKind::TypeMismatch,
            );
        }
        Ok(TypedExpression {
            expression: CheckedExpression::ArrayIndex {
                root: array.root,
                element_type: array.element_type,
                length: array.length,
                offset: Box::new(offset.expression),
                trap: TrapSite {
                    rule_id: "OP-4",
                    message: String::new(),
                    function: function.name.clone(),
                    node_path: self.tree.path(pbase)?.clone(),
                },
            },
            exhibits_traps: true,
        })
    }

    fn check_array_atom_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedArrayPlace, CheckStop> {
        if self.has_fixed(node, FixedTerminalV0_14::Move)? {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        }
        let place = self
            .tree
            .first_child_with(node, ProductionV0_14::Place)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRuleV0_14::Type5,
                    node,
                    SemanticIssueKind::TypeMismatch,
                )
            })?;
        self.check_array_place(place, bindings)
    }

    pub(super) fn check_array_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedArrayPlace, CheckStop> {
        if !self
            .tree
            .children_with(node, ProductionV0_14::Psuffix)?
            .is_empty()
        {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        }
        let pbase = self
            .tree
            .first_child_with(node, ProductionV0_14::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminalV0_14::Deref)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, pbase);
        }
        if self.has_fixed(pbase, FixedTerminalV0_14::Index)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::CompositeValues, pbase);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let (root, ty) = match class {
            DeclarationClass::Value => {
                let local = *bindings
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !local.live {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                (CheckedArrayRoot::Binding(local.binding), local.ty)
            }
            DeclarationClass::NamedConst => {
                let id = *self
                    .constants
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (CheckedArrayRoot::Constant(id), self.constant(id)?.ty)
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let CheckedType::Array { element, length } = ty else {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        };
        Ok(CheckedArrayPlace {
            root,
            element_type: element.ty(),
            length,
        })
    }
}
