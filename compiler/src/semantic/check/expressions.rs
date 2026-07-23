mod arrays;
mod calls;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminalV0_14, TerminalPredicateV0_14};
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, ProductionV0_14,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_14,
    UnsupportedSemanticFeatureV0_14,
};

use super::super::model::{
    CheckedArrayRoot, CheckedExpression, CheckedNominalKind, CheckedProjectedDrop, CheckedType,
    CheckedValue, CheckedWritablePlace, IntegerType,
};
use super::{CheckStop, Checker, Constructor, FunctionSignature, LocalBinding, TypedExpression};

#[derive(Clone, Copy)]
enum PlaceUseContext {
    Ordinary,
    Consuming,
}

#[derive(Clone, Copy)]
struct PlaceUseOptions {
    explicit_move: bool,
    context: PlaceUseContext,
    loop_depth: usize,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_set_target(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(DeclarationId, CheckedWritablePlace), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, ProductionV0_14::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminalV0_14::Deref)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, pbase);
        }
        if self.has_fixed(pbase, FixedTerminalV0_14::Index)? {
            let base = self
                .tree
                .first_child_with(pbase, ProductionV0_14::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if matches!(
                self.check_array_place(base, bindings)?.root,
                CheckedArrayRoot::Constant(_)
            ) {
                return self.issue_node(
                    SemanticRuleV0_14::Const2,
                    node,
                    SemanticIssueKind::ImmutableSetTarget,
                );
            }
            return self.unsupported(UnsupportedSemanticFeatureV0_14::CompositeValues, pbase);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }

        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if class == DeclarationClass::NamedConst {
            return self.issue_node(
                SemanticRuleV0_14::Const2,
                node,
                SemanticIssueKind::ImmutableSetTarget,
            );
        }
        if class != DeclarationClass::Value {
            return self.issue_node(
                SemanticRuleV0_14::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: format!("{class:?}"),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }

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

        let mut ty = local.ty;
        let mut fields = Vec::new();
        for suffix in self.tree.children_with(node, ProductionV0_14::Psuffix)? {
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling();
            let CheckedType::Nominal(nominal_id) = ty else {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let CheckedNominalKind::Struct {
                fields: declared_fields,
            } = &self.nominal(nominal_id)?.kind
            else {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let Some((index, field)) = declared_fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == name)
            else {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            fields
                .push(u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?);
            ty = field.ty;
        }

        if !self.is_copy_type(ty)? {
            return self.issue_node(
                SemanticRuleV0_14::Stor1,
                node,
                SemanticIssueKind::AffineSetTarget {
                    target_type: self.checked_type_name(ty)?,
                    mechanical_fix:
                        "construct a fresh owner under a new let; do not replace an affine place",
                },
            );
        }

        Ok((
            declaration,
            CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
            },
        ))
    }

    pub(super) fn checked_type_name(&self, ty: CheckedType) -> Result<String, CheckStop> {
        Ok(match ty {
            CheckedType::Unit => "unit".to_owned(),
            CheckedType::Bool => "Bool".to_owned(),
            CheckedType::Integer(integer) => match integer {
                IntegerType::I8 => "i8",
                IntegerType::I16 => "i16",
                IntegerType::I32 => "i32",
                IntegerType::I64 => "i64",
                IntegerType::U8 => "u8",
                IntegerType::U16 => "u16",
                IntegerType::U32 => "u32",
                IntegerType::U64 => "u64",
            }
            .to_owned(),
            CheckedType::Nominal(id) => self.nominal(id)?.name.clone(),
            CheckedType::Array { element, length } => {
                format!("array<{}, {length}>", self.checked_type_name(element.ty())?)
            }
        })
    }

    pub(super) fn check_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_with_expected(function, node, bindings, loop_depth, None)
    }

    pub(super) fn check_expression_with_expected(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        expected: Option<CheckedType>,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            expected,
            PlaceUseContext::Ordinary,
        )
    }

    pub(super) fn check_consuming_expression_with_expected(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        expected: Option<CheckedType>,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            expected,
            PlaceUseContext::Consuming,
        )
    }

    fn check_expression_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        expected: Option<CheckedType>,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        let child = self.tree.only_child(node)?;
        match self.tree.production(child)? {
            ProductionV0_14::Atom => {
                self.check_atom_in_context(function, child, bindings, loop_depth, place_context)
            }
            ProductionV0_14::Call => self.check_call(function, child, bindings, loop_depth),
            ProductionV0_14::Construct => {
                self.check_construct(function, child, bindings, loop_depth, expected)
            }
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    pub(super) fn check_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
        )
    }

    fn check_atom_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::Literal)?
        {
            return Ok(TypedExpression {
                expression: CheckedExpression::Constant(
                    self.parse_literal(node, self.tree.token_bytes(literal)?)?,
                ),
                exhibits_traps: false,
            });
        }
        if let Some(place) = self.tree.first_child_with(node, ProductionV0_14::Place)? {
            let value = self.check_place_use(
                function,
                node,
                place,
                bindings,
                PlaceUseOptions {
                    explicit_move: self.has_fixed(node, FixedTerminalV0_14::Move)?,
                    context: place_context,
                    loop_depth,
                },
            )?;
            return Ok(value);
        }
        if let Some(borrow) = self
            .tree
            .first_child_with(node, ProductionV0_14::BorrowExpr)?
        {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, borrow);
        }
        let _ = function;
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    fn check_place_use(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, ProductionV0_14::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminalV0_14::Deref)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, pbase);
        }
        if self.has_fixed(pbase, FixedTerminalV0_14::Index)? {
            return self.check_index_use(function, use_node, node, pbase, bindings, options);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        match class {
            DeclarationClass::Value => {
                let local = *bindings
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !local.live {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        use_node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                let mut ty = local.ty;
                let mut fields = Vec::new();
                let mut residual_drops = Vec::new();
                for suffix in self.tree.children_with(node, ProductionV0_14::Psuffix)? {
                    let name = self
                        .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                        .spelling();
                    let CheckedType::Nominal(nominal_id) = ty else {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            suffix,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    let CheckedNominalKind::Struct {
                        fields: declared_fields,
                    } = &self.nominal(nominal_id)?.kind
                    else {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            suffix,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    let Some((index, field)) = declared_fields
                        .iter()
                        .enumerate()
                        .find(|(_, field)| field.name == name)
                    else {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            suffix,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    for (sibling_index, sibling) in declared_fields.iter().enumerate().rev() {
                        if sibling_index != index && !self.is_copy_type(sibling.ty)? {
                            let mut sibling_path = fields.clone();
                            sibling_path.push(
                                u32::try_from(sibling_index)
                                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                            );
                            residual_drops.push(CheckedProjectedDrop {
                                fields: sibling_path,
                                ty: sibling.ty,
                            });
                        }
                    }
                    fields.push(
                        u32::try_from(index)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                    );
                    ty = field.ty;
                }
                let copy = self.is_copy_type(ty)?;
                if options.explicit_move && copy {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                if !copy
                    && !options.explicit_move
                    && matches!(options.context, PlaceUseContext::Ordinary)
                {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "write `move p` for the affine place",
                        },
                    );
                }
                if !copy && local.loop_depth < options.loop_depth {
                    return self.issue_node(
                        SemanticRuleV0_14::Own11,
                        use_node,
                        SemanticIssueKind::MoveOuterBindingInLoop {
                            mechanical_fix: "move the binding before the loop or declare and consume it inside the loop body",
                        },
                    );
                }
                if !copy {
                    bindings
                        .get_mut(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .live = false;
                }
                if fields.is_empty() {
                    Ok(TypedExpression {
                        expression: CheckedExpression::Binding {
                            binding: local.binding,
                            ty,
                        },
                        exhibits_traps: false,
                    })
                } else {
                    Ok(TypedExpression {
                        expression: CheckedExpression::Project {
                            binding: local.binding,
                            fields,
                            ty,
                            consume_root: !copy,
                            residual_drops: if copy { Vec::new() } else { residual_drops },
                        },
                        exhibits_traps: false,
                    })
                }
            }
            DeclarationClass::NamedConst => {
                if options.explicit_move {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                if !self
                    .tree
                    .children_with(node, ProductionV0_14::Psuffix)?
                    .is_empty()
                {
                    return self
                        .unsupported(UnsupportedSemanticFeatureV0_14::CompositeValues, node);
                }
                let constant = self
                    .constants
                    .get(&declaration)
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let constant = self.constant(constant)?;
                if matches!(constant.ty, CheckedType::Array { .. }) {
                    return self.issue_node(
                        SemanticRuleV0_14::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "read a const array through `index` or `len`",
                        },
                    );
                }
                Ok(TypedExpression {
                    expression: CheckedExpression::Constant(constant.value.clone()),
                    exhibits_traps: false,
                })
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    pub(super) fn check_match_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_consuming_expression_with_expected(function, node, bindings, loop_depth, None)
    }

    pub(super) fn check_construct(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        expected: Option<CheckedType>,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(targs) = self.tree.first_child_with(node, ProductionV0_14::Targs)? {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::Generics, targs);
        }
        let usage = self.use_at(node, LexicalUseRole::Construct)?;
        let constructor_name = usage.spelling().to_owned();
        if let ResolvedTarget::Prelude(id) = usage.target()
            && matches!(id.ordinal(), 1 | 2)
        {
            let value = match id.ordinal() {
                1 => CheckedValue::Bool(true),
                2 => CheckedValue::Bool(false),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            if self
                .tree
                .first_child_with(node, ProductionV0_14::FieldinitList)?
                .is_some()
            {
                return self.issue_node(
                    SemanticRuleV0_14::Gram8,
                    node,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: Vec::new(),
                    },
                );
            }
            return Ok(TypedExpression {
                expression: CheckedExpression::Constant(value),
                exhibits_traps: false,
            });
        }
        let constructor = match usage.target() {
            ResolvedTarget::Source { declaration, .. } => *self
                .constructors_by_declaration
                .get(&declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            ResolvedTarget::Prelude(id) => match id.ordinal() {
                11 | 13 => {
                    let Some(CheckedType::Nominal(nominal)) = expected else {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    if !matches!(
                        self.prelude_type(nominal),
                        Some(super::PreludeType::Result(_, _))
                    ) {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    Constructor::Enum {
                        nominal,
                        variant: u32::from(id.ordinal() == 13),
                    }
                }
                16 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::Overflow)?,
                    variant: 0,
                },
                18 | 19 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::DivError)?,
                    variant: u32::from(id.ordinal() == 19),
                },
                21 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::NarrowError)?,
                    variant: 0,
                },
                _ => {
                    return self
                        .unsupported(UnsupportedSemanticFeatureV0_14::PreludeNominalValues, node);
                }
            },
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let declared_fields = match constructor {
            Constructor::Struct(nominal) => match &self.nominal(nominal)?.kind {
                CheckedNominalKind::Struct { fields } => fields.clone(),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            },
            Constructor::Enum { nominal, variant } => match &self.nominal(nominal)?.kind {
                CheckedNominalKind::Enum { variants } => variants
                    .get(variant as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .fields
                    .clone(),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            },
        };
        let written_fields = if let Some(list) = self
            .tree
            .first_child_with(node, ProductionV0_14::FieldinitList)?
        {
            self.tree.children_with(list, ProductionV0_14::Fieldinit)?
        } else {
            Vec::new()
        };
        let declared_field_names = declared_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<_>>();
        if written_fields.len() != declared_fields.len() {
            return self.issue_node(
                SemanticRuleV0_14::Gram8,
                node,
                SemanticIssueKind::InvalidConstructionFields {
                    constructor: constructor_name,
                    declared_fields: declared_field_names,
                },
            );
        }
        let mut fields = Vec::with_capacity(written_fields.len());
        let mut exhibits_traps = false;
        for (written, declared) in written_fields.into_iter().zip(&declared_fields) {
            if self
                .deferred_use_at(written, DeferredUseRole::FieldInitializer)?
                .spelling()
                != declared.name
            {
                return self.issue_node(
                    SemanticRuleV0_14::Gram8,
                    written,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: declared_field_names,
                    },
                );
            }
            let atom = self
                .tree
                .first_child_with(written, ProductionV0_14::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.check_atom(function, atom, bindings, loop_depth)?;
            if value.expression.ty() != declared.ty {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    atom,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            exhibits_traps |= value.exhibits_traps;
            fields.push(value.expression);
        }
        let expression = match constructor {
            Constructor::Struct(nominal) => CheckedExpression::ConstructStruct { nominal, fields },
            Constructor::Enum { nominal, variant } => CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
            },
        };
        Ok(TypedExpression {
            expression,
            exhibits_traps,
        })
    }
}
