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
    CheckedConst, CheckedExpression, CheckedMode, CheckedNominalKind, CheckedProjectedDrop,
    CheckedSetTarget, CheckedType, CheckedValue, CheckedWritablePlace, FloatType, IntegerType,
};
use super::borrows::{AccessKind, ResolvedPlace};
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
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_set_target(node, pbase, bindings);
        }
        if self.has_fixed(pbase, FixedTerminal::Index)? {
            return self.check_indexed_set_target(function, node, pbase, bindings, loop_depth);
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

        let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
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
            CheckedType::Nominal(id) => self.nominal(id)?.name.clone(),
            CheckedType::Array { element, length } => {
                let length = match length {
                    CheckedConst::Value(value) => value.to_string(),
                    CheckedConst::Parameter(declaration) => {
                        format!("<const-parameter:{}>", declaration.index())
                    }
                };
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

    pub(super) fn resolve_struct_path(
        &self,
        node: NodeId,
        mut ty: CheckedType,
    ) -> Result<(Vec<u32>, CheckedType), CheckStop> {
        let mut fields = Vec::new();
        for suffix in self.tree.children_with(node, Production::Psuffix)? {
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
            Production::Atom => self.check_atom_in_context(
                function,
                child,
                bindings,
                loop_depth,
                place_context,
                false,
            ),
            Production::Call => self.check_call(function, child, bindings, loop_depth),
            Production::Construct => {
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
            false,
        )
    }

    pub(super) fn check_call_argument_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        child_reborrow_allowed: bool,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
            child_reborrow_allowed,
        )
    }

    fn check_atom_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
        child_reborrow_allowed: bool,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicate::Literal)?
        {
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(
                    self.parse_literal(node, self.tree.token_bytes(literal)?)?,
                ),
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
            return self.check_borrow(
                borrow,
                function,
                bindings,
                loop_depth,
                child_reborrow_allowed,
            );
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
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_place_use(use_node, node, pbase, bindings, options);
        }
        if self.has_fixed(pbase, FixedTerminal::Index)? {
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
                    if !self
                        .tree
                        .children_with(node, Production::Psuffix)?
                        .is_empty()
                    {
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
                    if !copy {
                        bindings
                            .get_mut(&declaration)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?
                            .live = false;
                    }
                    return Ok(TypedExpression {
                        expression: CheckedExpression::Binding {
                            binding: local.binding,
                            ty: local.ty,
                        },
                        mode: local.mode,
                        borrow: local.borrow,
                        slice: None,
                        holder: Some(declaration),
                        effects: EffectSet::NONE,
                        accesses: Vec::new(),
                    });
                }
                let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
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
                self.check_loan_access(
                    bindings,
                    None,
                    &ResolvedPlace {
                        root: declaration,
                        fields: fields.clone(),
                    },
                    if copy {
                        AccessKind::Read
                    } else {
                        AccessKind::Move
                    },
                    use_node,
                )?;
                let residual_drops = if copy || fields.is_empty() {
                    Vec::new()
                } else {
                    self.residual_drop_paths(local.ty, &fields)?
                        .into_iter()
                        .map(|(fields, ty)| CheckedProjectedDrop { fields, ty })
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
                    fields: fields.clone(),
                };
                let access_kind = if copy {
                    AccessKind::Read
                } else {
                    AccessKind::Move
                };
                if fields.is_empty() {
                    let mut expression = TypedExpression::owned_with_access(
                        CheckedExpression::Binding {
                            binding: local.binding,
                            ty,
                        },
                        EffectSet::NONE,
                        access,
                        access_kind,
                    );
                    expression.slice = local.slice;
                    Ok(expression)
                } else {
                    Ok(TypedExpression::owned_with_access(
                        CheckedExpression::Project {
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
                if !self
                    .tree
                    .children_with(node, Production::Psuffix)?
                    .is_empty()
                {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
                }
                let constant = self
                    .constants
                    .get(&declaration)
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let constant = self.constant(constant)?;
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
                Ok(TypedExpression::owned(
                    CheckedExpression::Constant(constant.value.clone()),
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
        if borrow.kind != super::borrows::BorrowKind::Unique {
            return self.issue_node(
                SemanticRule::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: "shared borrow".to_owned(),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }
        let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
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
                5 | 6 => {
                    let Some(CheckedType::Nominal(nominal)) = expected else {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    if !matches!(
                        self.prelude_type(nominal),
                        Some(super::PreludeType::Option(_))
                    ) {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    Constructor::Enum {
                        nominal,
                        variant: u32::from(id.ordinal() == 6),
                    }
                }
                11 | 13 => {
                    let Some(CheckedType::Nominal(nominal)) = expected else {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    if !matches!(
                        self.prelude_type(nominal),
                        Some(super::PreludeType::Result(_, _))
                    ) {
                        return self.issue_node(
                            SemanticRule::Type5,
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
                        .unsupported(UnsupportedSemanticFeature::PreludeNominalValues, node);
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
            Constructor::Struct(nominal) => CheckedExpression::ConstructStruct { nominal, fields },
            Constructor::Enum { nominal, variant } => CheckedExpression::ConstructEnum {
                nominal,
                variant,
                fields,
            },
        };
        Ok(TypedExpression::owned(expression, effects))
    }
}
