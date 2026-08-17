mod calls;
mod flat_storage;
mod places;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedConst, CheckedConstant, CheckedExpression, CheckedIntegerOperation, CheckedMode,
    CheckedNominalKind, CheckedProjectedDrop, CheckedSetTarget, CheckedType, CheckedValue,
    CheckedWritablePlace, FloatType, IntegerType,
};
use super::borrows::{AccessKind, ReborrowPosition, ResolvedPlace};
use super::{
    CheckStop, Checker, Constructor, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

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
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<(DeclarationId, CheckedSetTarget, EffectSet), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.has_fixed(pbase, FixedTerminal::Deref)? && self.tree.children(pbase)?.is_empty() {
            let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
            if let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } = usage.target()
                && bindings
                    .get(&declaration)
                    .is_some_and(|local| local.compiler_updated)
            {
                return self.issue_node(
                    SemanticRule::Set1,
                    node,
                    SemanticIssueKind::InvalidSetTarget {
                        root_class: "compiler-updated counted binder".to_owned(),
                        required_classes:
                            "source-writable live own storage or a live usable &uniq referent",
                    },
                );
            }
        }
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        if let Some(subscript) = self.last_subscript(&suffixes)? {
            return self.check_indexed_set_target(
                function, node, &suffixes, subscript, bindings, loop_depth,
            );
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_set_target(node, pbase, bindings);
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
                SemanticRule::Const2,
                node,
                SemanticIssueKind::ImmutableSetTarget,
            );
        }
        if class != DeclarationClass::Value {
            return self.issue_node(
                SemanticRule::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: format!("{class:?}"),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }

        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }

        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        if local.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: match local.mode {
                        CheckedMode::Shared(_) => "shared borrow",
                        CheckedMode::Unique(_) => "unique borrow holder",
                        CheckedMode::Own => "owned value",
                    }
                    .to_owned(),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }
        self.check_loan_access(
            bindings,
            None,
            &ResolvedPlace {
                root: declaration,
                fields: fields.clone(),
            },
            AccessKind::Write,
            node,
        )?;

        if !self.is_copy_type(ty)? {
            return self.issue_node(
                SemanticRule::Stor1,
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
            CheckedSetTarget::Place(CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
            }),
            EffectSet::NONE,
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
            CheckedType::Float(float) => match float {
                FloatType::F32 => "f32",
                FloatType::F64 => "f64",
            }
            .to_owned(),
            CheckedType::Generic(declaration) => {
                format!("<type-parameter:{}>", declaration.index())
            }
            CheckedType::GenericInt(declaration) => {
                format!("<Int-parameter:{}>", declaration.index())
            }
            CheckedType::GenericFloat(declaration) => {
                format!("<Float-parameter:{}>", declaration.index())
            }
            CheckedType::Nominal(id) => self.nominal(id)?.name.clone(),
            CheckedType::Array { element, length } => {
                let length = self.checked_const_name(length)?;
                format!("array<{}, {length}>", self.checked_type_name(element.ty())?)
            }
            CheckedType::Slice { region, element } => format!(
                "slice<'region#{}, {}>",
                region.index(),
                self.checked_type_name(element.ty())?
            ),
            CheckedType::Buffer { element } => {
                format!("buffer<{}>", self.checked_type_name(element.ty())?)
            }
        })
    }

    /// A field-suffix chain rooted at a struct-typed const [CONST-2
    /// candidate]. The path is resolved by the ordinary projection judgment,
    /// then folded against the constant's total value: a copy scalar
    /// selection copies out as a constant, and a composite selection keeps
    /// the whole-composite read rules.
    fn check_struct_constant_projection(
        &self,
        use_node: NodeId,
        constant: &CheckedConstant,
        suffixes: &[NodeId],
    ) -> Result<TypedExpression, CheckStop> {
        let (fields, ty) = self.resolve_struct_path(suffixes, constant.ty)?;
        let mut value = &constant.value;
        for index in &fields {
            let CheckedValue::Struct { fields: values, .. } = value else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            value = values
                .get(*index as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        }
        if value.ty() != ty {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        match value {
            CheckedValue::Struct { .. } => self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "read a const struct through its fields",
                },
            ),
            CheckedValue::Array { .. } => self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "read a const array through `index` or `len`",
                },
            ),
            scalar => Ok(TypedExpression::owned(
                CheckedExpression::Constant(scalar.clone()),
                EffectSet::NONE,
            )),
        }
    }

    pub(super) fn checked_const_name(&self, value: CheckedConst) -> Result<String, CheckStop> {
        Ok(match value {
            CheckedConst::Value(value) => value.to_string(),
            CheckedConst::Parameter(declaration) => {
                format!("<const-parameter:{}>", declaration.index())
            }
            CheckedConst::Derived(id) => {
                let derived = self.derived_const(id)?;
                format!(
                    "{} {} {}",
                    self.checked_const_name(derived.left)?,
                    derived.operation.spelling(),
                    self.checked_const_name(derived.right)?
                )
            }
        })
    }

    /// Resolves a run of field-selection suffixes over one starting type.
    /// Callers pass the suffix chain to walk — every suffix for a whole
    /// place, or the chain before a subscript for that subscript's base. A
    /// subscript suffix inside the walked run selects through a composite
    /// element value, which this version does not implement.
    pub(super) fn resolve_struct_path(
        &self,
        suffixes: &[NodeId],
        mut ty: CheckedType,
    ) -> Result<(Vec<u32>, CheckedType), CheckStop> {
        let mut fields = Vec::new();
        for &suffix in suffixes {
            if self.subscript_offset(suffix)?.is_some() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, suffix);
            }
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling();
            let CheckedType::Nominal(nominal_id) = ty else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            let CheckedNominalKind::Struct {
                fields: declared_fields,
            } = &self.nominal(nominal_id)?.kind
            else {
                return self.issue_node(
                    SemanticRule::Type5,
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
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::TypeMismatch,
                );
            };
            fields
                .push(u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?);
            ty = field.ty;
        }
        Ok((fields, ty))
    }

    pub(super) fn check_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
        )
    }

    pub(super) fn check_consuming_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Consuming,
        )
    }

    fn check_expression_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        // [GRAM-5] `expr := atom infix_tail? | call | construct`, so the only
        // shape with more than one child is the infix one.
        if let Some(tail) = self.tree.first_child_with(node, Production::InfixTail)? {
            return self.check_infix(function, node, tail, bindings, loop_depth);
        }
        let child = self.tree.only_child(node)?;
        match self.tree.production(child)? {
            Production::Atom => self.check_atom_in_context(
                function,
                child,
                bindings,
                loop_depth,
                place_context,
                ReborrowPosition::Forbidden,
            ),
            Production::Call => self.check_call(function, child, bindings, loop_depth),
            Production::Construct => self.check_construct(function, child, bindings, loop_depth),
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// [OP-1] (ii) infix resolution: the operator token selects the row.
    ///
    /// [GRAM-9] admits exactly one operation per expression, so there is no
    /// precedence to apply — the left operand is the `expr`'s own atom and
    /// the right is the tail's. The row then takes the same judgment the
    /// named spelling takes.
    fn check_infix(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        tail: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let left = self
            .tree
            .first_child_with(node, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let operator = self
            .tree
            .first_child_with(tail, Production::InfixOp)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let right = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let operation = self.infix_operation(operator)?;
        self.check_integer_operation_row(
            node,
            operation,
            &[left, right],
            function,
            bindings,
            loop_depth,
        )
    }

    /// [OP-1] the exact operator token, and the row it spells.
    ///
    /// Bare `+ - * / %` carry the trapping mode, the suffixed forms carry
    /// wrap, checked and saturating, and the four nonstrict comparisons
    /// respell here. `ilt` and `igt` keep their named spelling and have no
    /// operator token, so nothing maps to them.
    pub(super) fn infix_operation(
        &self,
        operator: NodeId,
    ) -> Result<CheckedIntegerOperation, CheckStop> {
        let [terminal] = self.tree.direct_token_indices(operator)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        Ok(match self.tree.token_bytes(*terminal)? {
            b"+" => CheckedIntegerOperation::AddTrap,
            b"+wrap" => CheckedIntegerOperation::AddWrap,
            b"+checked" => CheckedIntegerOperation::AddChecked,
            b"+sat" => CheckedIntegerOperation::AddSaturating,
            b"-" => CheckedIntegerOperation::SubtractTrap,
            b"-wrap" => CheckedIntegerOperation::SubtractWrap,
            b"-checked" => CheckedIntegerOperation::SubtractChecked,
            b"-sat" => CheckedIntegerOperation::SubtractSaturating,
            b"*" => CheckedIntegerOperation::MultiplyTrap,
            b"*wrap" => CheckedIntegerOperation::MultiplyWrap,
            b"*checked" => CheckedIntegerOperation::MultiplyChecked,
            b"*sat" => CheckedIntegerOperation::MultiplySaturating,
            b"/" => CheckedIntegerOperation::DivideTrap,
            b"/checked" => CheckedIntegerOperation::DivideChecked,
            b"%" => CheckedIntegerOperation::RemainderTrap,
            b"%checked" => CheckedIntegerOperation::RemainderChecked,
            _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        })
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
            ReborrowPosition::Forbidden,
        )
    }

    /// Checks an atom in a position whose owning rule decides whether the
    /// selected value is admissible. This delays OWN-1's bare-affine spelling
    /// rejection long enough for an earlier TYPE-7 implicit-read judgment to
    /// take exclusive ownership of a holder used for its referent.
    pub(super) fn check_consuming_atom(
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
            PlaceUseContext::Consuming,
            ReborrowPosition::Forbidden,
        )
    }

    pub(super) fn check_call_argument_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        own_result: bool,
        result_candidate: bool,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
            ReborrowPosition::CallArgument {
                own_result,
                result_candidate,
            },
        )
    }

    fn check_atom_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
        reborrow_position: ReborrowPosition,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(value) = self.postcondition_result_placeholder(node)? {
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(value),
                EffectSet::NONE,
            ));
        }
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicate::Literal)?
        {
            let bytes = self.tree.token_bytes(literal)?;
            if matches!(bytes, b"0_T" | b"1_T") {
                return self.check_generic_numeric_identity(function, node, bytes == b"1_T");
            }
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(self.parse_literal(node, bytes)?),
                EffectSet::NONE,
            ));
        }
        if let Some(place) = self.tree.first_child_with(node, Production::Place)? {
            let value = self.check_place_use(
                function,
                node,
                place,
                bindings,
                PlaceUseOptions {
                    explicit_move: self.has_fixed(node, FixedTerminal::Move)?,
                    context: place_context,
                    loop_depth,
                },
            )?;
            return Ok(value);
        }
        if let Some(borrow) = self.tree.first_child_with(node, Production::BorrowExpr)? {
            return self.check_borrow(borrow, function, bindings, loop_depth, reborrow_position);
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// The `borrow_expr` that is the complete written content of `expression`,
    /// if any: the position [OWN-14] names for the returned reborrow.
    ///
    /// An infix expression is a fresh operation result rather than a written
    /// borrow, so it answers `None` like any other non-borrow shape.
    pub(super) fn complete_borrow_expression(
        &self,
        expression: NodeId,
    ) -> Result<Option<NodeId>, CheckStop> {
        let Some(child) = self.tree.sole_expression_child(expression)? else {
            return Ok(None);
        };
        if self.tree.production(child)? != Production::Atom {
            return Ok(None);
        }
        Ok(self.tree.first_child_with(child, Production::BorrowExpr)?)
    }

    fn check_generic_numeric_identity(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        one: bool,
    ) -> Result<TypedExpression, CheckStop> {
        let usage = self.use_at(node, LexicalUseRole::GenericNumericSuffix)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::GenericType,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let ty = function
            .substitution
            .type_argument(declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let value = match ty {
            CheckedType::Integer(ty) => CheckedValue::Integer {
                ty,
                bits: u64::from(one),
            },
            CheckedType::Float(FloatType::F32) => CheckedValue::Float {
                ty: FloatType::F32,
                bits: if one { 0x3f80_0000 } else { 0 },
            },
            CheckedType::Float(FloatType::F64) => CheckedValue::Float {
                ty: FloatType::F64,
                bits: if one { 0x3ff0_0000_0000_0000 } else { 0 },
            },
            CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
                CheckedValue::NumericIdentity { ty, one }
            }
            _ => {
                return self.issue_node(SemanticRule::Form5, node, SemanticIssueKind::TypeMismatch);
            }
        };
        Ok(TypedExpression::owned(
            CheckedExpression::Constant(value),
            EffectSet::NONE,
        ))
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
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        if let Some(subscript) = self.last_subscript(&suffixes)? {
            return self.check_index_use(
                function, use_node, node, &suffixes, subscript, bindings, options,
            );
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_place_use(use_node, node, pbase, bindings, options);
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
                let local = bindings
                    .get(&declaration)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !local.live {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                if local.mode != CheckedMode::Own {
                    if !suffixes.is_empty() {
                        return self.issue_node(
                            SemanticRule::Type7,
                            use_node,
                            SemanticIssueKind::MissingDereference {
                                mechanical_fix: "write `deref(holder)`",
                            },
                        );
                    }
                    let copy = matches!(local.mode, CheckedMode::Shared(_));
                    if options.explicit_move && copy {
                        return self.issue_node(
                            SemanticRule::Own1,
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
                            SemanticRule::Own1,
                            use_node,
                            SemanticIssueKind::BareAffineUse {
                                mechanical_fix: "write `move p` for the affine place",
                            },
                        );
                    }
                    // A suspended holder admits no move, copy, or
                    // call-transfer of itself [OWN-5, OWN-13]; OWN-1's
                    // spelling judgments above are defined first and cite
                    // first at this node [DIAG-1].
                    self.check_holder_not_suspended(&local, use_node)?;
                    if !copy {
                        bindings
                            .get_mut(&declaration)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?
                            .live = false;
                    }
                    let slice = local.slice;
                    let slice_origins = slice
                        .as_ref()
                        .map(|slice| slice.origins.clone())
                        .unwrap_or_default();
                    return Ok(TypedExpression {
                        expression: CheckedExpression::Binding {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            ty: local.ty,
                            slice_origins,
                            consume_root: !copy,
                        },
                        mode: local.mode,
                        borrow: local.borrow,
                        slice,
                        holder: Some(declaration),
                        // A bare borrow holder selects the holder, not its
                        // referent [TYPE-7, SET-1].
                        reference_value: true,
                        effects: EffectSet::NONE,
                        accesses: Vec::new(),
                    });
                }
                let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
                let copy = self.is_copy_type(ty)?;
                if options.explicit_move && copy {
                    return self.issue_node(
                        SemanticRule::Own1,
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
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "write `move p` for the affine place",
                        },
                    );
                }
                if !copy && local.loop_depth < options.loop_depth {
                    return self.issue_node(
                        SemanticRule::Own11,
                        use_node,
                        SemanticIssueKind::MoveOuterBindingInLoop {
                            mechanical_fix: "move the binding before the loop or declare and consume it inside the loop body",
                        },
                    );
                }
                // OWN-1 makes an affine projection consume its whole root.
                // Its residual cleanup destroys every unselected resource
                // field, so the loan access is the root rather than only the
                // selected projection.
                let access_fields = if copy { fields.clone() } else { Vec::new() };
                let access_kind = if copy {
                    AccessKind::Read
                } else {
                    AccessKind::Move
                };
                self.check_loan_access(
                    bindings,
                    None,
                    &ResolvedPlace {
                        root: declaration,
                        fields: access_fields.clone(),
                    },
                    access_kind,
                    use_node,
                )?;
                let residual_drops = if copy || fields.is_empty() {
                    Vec::new()
                } else {
                    let paths = self.residual_drop_paths(local.ty, &fields)?;
                    self.released_paths(paths)?
                        .into_iter()
                        .map(|(fields, ty, release)| CheckedProjectedDrop {
                            fields,
                            ty,
                            release,
                        })
                        .collect()
                };
                if !copy {
                    bindings
                        .get_mut(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .live = false;
                }
                let access = ResolvedPlace {
                    root: declaration,
                    fields: access_fields,
                };
                if fields.is_empty() {
                    let slice = local.slice;
                    let slice_origins = slice
                        .as_ref()
                        .map(|slice| slice.origins.clone())
                        .unwrap_or_default();
                    let mut expression = TypedExpression::owned_with_access(
                        CheckedExpression::Binding {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            ty,
                            slice_origins,
                            consume_root: !copy,
                        },
                        EffectSet::NONE,
                        access,
                        access_kind,
                    );
                    expression.slice = slice;
                    Ok(expression)
                } else {
                    Ok(TypedExpression::owned_with_access(
                        CheckedExpression::Project {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            fields,
                            ty,
                            consume_root: !copy,
                            residual_drops,
                        },
                        EffectSet::NONE,
                        access,
                        access_kind,
                    ))
                }
            }
            DeclarationClass::NamedConst => {
                if options.explicit_move {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                let constant = self
                    .constants
                    .get(&declaration)
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let constant = self.constant(constant)?;
                if !suffixes.is_empty() {
                    // A field-suffix chain rooted at a struct-typed const
                    // [CONST-2 candidate] copies the selected value out; the
                    // selection is total at compile time, so the read folds
                    // to the selected constant.
                    if matches!(constant.value, CheckedValue::Struct { .. }) {
                        return self
                            .check_struct_constant_projection(use_node, constant, &suffixes);
                    }
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
                }
                if matches!(
                    constant.ty,
                    CheckedType::Array { .. }
                        | CheckedType::Slice { .. }
                        | CheckedType::Buffer { .. }
                ) {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "read a const array through `index` or `len`",
                        },
                    );
                }
                if matches!(constant.value, CheckedValue::Struct { .. }) {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "read a const struct through its fields",
                        },
                    );
                }
                Ok(TypedExpression::owned(
                    CheckedExpression::NamedConstant {
                        declaration,
                        value: constant.value.clone(),
                    },
                    EffectSet::NONE,
                ))
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    fn check_dereferenced_set_target(
        &self,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(DeclarationId, CheckedSetTarget, EffectSet), CheckStop> {
        let (declaration, local, borrow) =
            self.resolve_dereference_holder(node, pbase, bindings)?;
        // [SET-1] states the shared-borrow referent as an [OWN-5] violation
        // and gives that rule the citation; SET-1 owns only the residue of its
        // writability relation.
        if borrow.kind != super::borrows::BorrowKind::Unique {
            return self.issue_node(SemanticRule::Own5, node, SemanticIssueKind::BorrowConflict);
        }
        self.check_holder_not_suspended(&local, node)?;
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        let mut resolved = borrow.place;
        resolved.fields.extend_from_slice(&fields);
        self.check_loan_access(
            bindings,
            Some(declaration),
            &resolved,
            AccessKind::Write,
            node,
        )?;
        if !self.is_copy_type(ty)? {
            return self.issue_node(
                SemanticRule::Stor1,
                node,
                SemanticIssueKind::AffineSetTarget {
                    target_type: self.checked_type_name(ty)?,
                    mechanical_fix:
                        "construct a fresh owner under a new let; do not replace an affine place",
                },
            );
        }
        let mut effects = EffectSet::NONE;
        if let Some(region) = borrow.origin_region {
            effects.add_write(region);
        }
        Ok((
            declaration,
            CheckedSetTarget::Place(CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
            }),
            effects,
        ))
    }

    pub(super) fn check_match_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_consuming_expression(function, node, bindings, loop_depth)
    }

    pub(super) fn check_construct(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
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
                .first_child_with(node, Production::FieldinitList)?
                .is_some()
            {
                return self.issue_node(
                    SemanticRule::Gram8,
                    node,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: Vec::new(),
                    },
                );
            }
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(value),
                EffectSet::NONE,
            ));
        }
        let constructor = match usage.target() {
            ResolvedTarget::Source { declaration, .. } => {
                self.source_constructor(node, declaration, &function.substitution)?
            }
            ResolvedTarget::Prelude(id) => match id.ordinal() {
                // [TYPE-5] the prelude generic nominals are constructed
                // through these variant constructors, and they write the
                // nominal's arguments in every position, mandatorily:
                // `None()` has no operand to supply them and construction
                // never consults an expected nominal type [TYPE-6]. The
                // written arguments are read here exactly as
                // `generic_substitution` reads a source generic's, so both
                // classes cite TYPE-5 at the complete `construct`.
                5 | 6 => {
                    let value = self.option_type_argument_with(node, &function.substitution)?;
                    Constructor::Enum {
                        nominal: self.prelude_nominal(super::PreludeType::Option(value))?,
                        variant: u32::from(id.ordinal() == 6),
                    }
                }
                11 | 13 => {
                    let (ok, error) =
                        self.result_type_arguments_with(node, &function.substitution)?;
                    Constructor::Enum {
                        nominal: self.prelude_nominal(super::PreludeType::Result(ok, error))?,
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
                        .unsupported(UnsupportedSemanticFeature::PreludeNominalValues, node);
                }
            },
            ResolvedTarget::System(id) => {
                let index = crate::system_constructor_index(id)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let record = crate::SYSTEM_CONSTRUCTORS
                    .get(usize::from(index))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let tag = crate::SYSTEM_CONSTRUCTORS[..usize::from(index)]
                    .iter()
                    .filter(|candidate| candidate.owner == record.owner)
                    .count();
                Constructor::Enum {
                    nominal: self.system_nominal(record.owner)?,
                    variant: u32::try_from(tag)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                }
            }
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
            .first_child_with(node, Production::FieldinitList)?
        {
            self.tree.children_with(list, Production::Fieldinit)?
        } else {
            Vec::new()
        };
        let declared_field_names = declared_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<_>>();
        if written_fields.len() != declared_fields.len() {
            return self.issue_node(
                SemanticRule::Gram8,
                node,
                SemanticIssueKind::InvalidConstructionFields {
                    constructor: constructor_name,
                    declared_fields: declared_field_names,
                },
            );
        }
        let mut fields = Vec::with_capacity(written_fields.len());
        let mut effects = EffectSet::NONE;
        for (written, declared) in written_fields.into_iter().zip(&declared_fields) {
            if self
                .deferred_use_at(written, DeferredUseRole::FieldInitializer)?
                .spelling()
                != declared.name
            {
                return self.issue_node(
                    SemanticRule::Gram8,
                    written,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: declared_field_names,
                    },
                );
            }
            let atom = self
                .tree
                .first_child_with(written, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.check_atom(function, atom, bindings, loop_depth)?;
            if value.expression.ty() != declared.ty {
                return self.issue_node(SemanticRule::Type5, atom, SemanticIssueKind::TypeMismatch);
            }
            if value.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type7,
                    atom,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(holder)`",
                    },
                );
            }
            effects = effects.union(value.effects);
            fields.push(value.expression);
        }
        let expression = match constructor {
            Constructor::Struct(nominal) => CheckedExpression::ConstructStruct {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                fields,
            },
            Constructor::Enum { nominal, variant } => CheckedExpression::ConstructEnum {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                variant,
                fields,
            },
        };
        Ok(TypedExpression::owned(expression, effects))
    }
}
