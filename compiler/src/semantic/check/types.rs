use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationClass, DeclarationRole, LexicalUseRole, PreludeDeclarationId, Production,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::model::{
    BindingId, CheckedCapabilityOrigins, CheckedConst, CheckedConstant, CheckedConstantId,
    CheckedExpression, CheckedFlatElement, CheckedMatchArm, CheckedMode, CheckedNominalKind,
    CheckedParameter, CheckedResultAuthorityOrigin, CheckedSetTarget, CheckedStatement,
    CheckedType, CheckedValue, ConstOperation, FloatType, IntegerType, NominalId,
    evaluate_const_operation,
};
use super::floats::parse_float_literal;
use super::generics::GenericSubstitution;
use super::{
    CheckStop, Checker, EffectSet, LocalBinding, ParameterSignature, PreludeType, TypedExpression,
};

#[derive(Clone, Debug)]
enum CapabilityOriginResolution {
    Absent,
    Finite(CheckedCapabilityOrigins),
    Unknown(crate::NodePath),
}

impl CapabilityOriginResolution {
    fn union(&mut self, other: Self) {
        match (&mut *self, other) {
            (Self::Unknown(_), _) => {}
            (_, Self::Unknown(path)) => *self = Self::Unknown(path),
            (Self::Absent, finite @ Self::Finite(_)) => *self = finite,
            (Self::Finite(left), Self::Finite(right)) => left.union(&right),
            (Self::Absent, Self::Absent) | (Self::Finite(_), Self::Absent) => {}
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn parse_parameters_with(
        &self,
        function: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<Vec<ParameterSignature>, CheckStop> {
        let Some(list) = self
            .tree
            .first_child_with(function, Production::ParamList)?
        else {
            return Ok(Vec::new());
        };
        let mut parameters = Vec::new();
        for node in self.tree.children_with(list, Production::Param)? {
            let declaration = self.declaration_at(node, DeclarationRole::Parameter)?;
            let mode = self
                .tree
                .first_child_with(node, Production::Mode)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let mode = self.parse_mode(mode)?;
            let ty_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let ty = self.parse_type_with(ty_node, substitution)?;
            if mode != CheckedMode::Own && !self.borrowable_type(ty)? {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node);
            }
            parameters.push(ParameterSignature {
                declaration: declaration.id(),
                node_path: self.tree.path(node)?.clone(),
                name: declaration.spelling().to_owned(),
                mode,
                ty,
            });
        }
        Ok(parameters)
    }

    pub(super) fn parse_rtype_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(CheckedMode, CheckedType), CheckStop> {
        let mode = self
            .tree
            .first_child_with(node, Production::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self.parse_mode(mode)?;
        let ty = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        Ok((mode, self.parse_type_with(ty, substitution)?))
    }

    pub(super) fn parse_type(&self, node: NodeId) -> Result<CheckedType, CheckStop> {
        self.parse_type_with(node, &GenericSubstitution::default())
    }

    pub(super) fn parse_type_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<CheckedType, CheckStop> {
        let targs = self.tree.first_child_with(node, Production::Targs)?;
        if let Some(ty) = self.integer_type(node)? {
            if targs.is_some() {
                return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
            }
            return Ok(CheckedType::Integer(ty));
        }
        if self.has_fixed(node, FixedTerminal::Unit)? {
            if targs.is_some() {
                return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
            }
            return Ok(CheckedType::Unit);
        }
        if self.has_fixed(node, FixedTerminal::F32)? {
            return Ok(CheckedType::Float(FloatType::F32));
        }
        if self.has_fixed(node, FixedTerminal::F64)? {
            return Ok(CheckedType::Float(FloatType::F64));
        }
        if self.has_fixed(node, FixedTerminal::Array)? {
            let element_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(element_node, substitution)?;
            let length_node = self
                .tree
                .first_child_with(node, Production::Const)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let element_type = self.parse_type_with(element_node, substitution)?;
            let element = self.checked_flat_element(element_type, element_node)?;
            return Ok(CheckedType::Array {
                element,
                length: self.parse_const_expression_with(length_node, substitution)?,
            });
        }
        if self.has_fixed(node, FixedTerminal::Buffer)? {
            let element_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(element_node, substitution)?;
            let element_type = self.parse_type_with(element_node, substitution)?;
            // [TYPE-2] v0.31 forms buffers over copy elements and over
            // region-free affine nominal elements. The structural affine
            // composites (`buffer<buffer<T>>`, `buffer<array<T, N>>`) and
            // generic elements have no implemented representation yet and
            // stop as an explicit unsupported capability rather than a
            // source rejection.
            return match self.buffer_element(element_type)? {
                Some(element) => Ok(CheckedType::Buffer { element }),
                None => self.unsupported(UnsupportedSemanticFeature::CompositeValues, element_node),
            };
        }
        if self.has_fixed(node, FixedTerminal::Arena)? {
            let content_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(content_node, substitution)?;
            let content = self.parse_type_with(content_node, substitution)?;
            let usage = self.use_at(node, LexicalUseRole::TypeRegion)?;
            let ResolvedTarget::Source {
                declaration: region,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            return self
                .arena_nominals
                .get(&(region, content))
                .copied()
                .map(CheckedType::Nominal)
                .ok_or(SemanticCompilerFailure::InvalidResolution.into());
        }
        if self.has_fixed(node, FixedTerminal::Slice)? {
            let usage = self.use_at(node, LexicalUseRole::TypeRegion)?;
            let ResolvedTarget::Source {
                declaration: region,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            let element_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let element_type = self.parse_type_with(element_node, substitution)?;
            let Some(element) = self.flat_element(element_type)? else {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, element_node);
            };
            return Ok(CheckedType::Slice { region, element });
        }
        if self.has_fixed(node, FixedTerminal::Box)? {
            let referent_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(referent_node, substitution)?;
            let referent = self.parse_type_with(referent_node, substitution)?;
            return self
                .box_nominals
                .get(&referent)
                .copied()
                .map(CheckedType::Nominal)
                .ok_or(SemanticCompilerFailure::InvalidResolution.into());
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
            .is_some()
        {
            let usage = self.use_at(node, LexicalUseRole::Type)?;
            match usage.target() {
                ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(0) => {
                    if targs.is_some() {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    return Ok(CheckedType::Bool);
                }
                ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(3) => {
                    let value = self.option_type_argument_with(node, substitution)?;
                    return self
                        .prelude_nominals
                        .get(&PreludeType::Option(value))
                        .copied()
                        .map(CheckedType::Nominal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution.into());
                }
                ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(8) => {
                    let (ok, error) = self.result_type_arguments_with(node, substitution)?;
                    return self
                        .prelude_nominals
                        .get(&PreludeType::Result(ok, error))
                        .copied()
                        .map(CheckedType::Nominal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution.into());
                }
                ResolvedTarget::Prelude(id) if matches!(id.ordinal(), 15 | 17 | 20) => {
                    if targs.is_some() {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    let ty = match id.ordinal() {
                        15 => PreludeType::Overflow,
                        17 => PreludeType::DivError,
                        20 => PreludeType::NarrowError,
                        _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                    };
                    return Ok(CheckedType::Nominal(self.prelude_nominal(ty)?));
                }
                ResolvedTarget::Prelude(_) => {
                    return self
                        .unsupported(UnsupportedSemanticFeature::PreludeNominalValues, node);
                }
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::NominalType,
                } => {
                    let template_index = *self
                        .nominal_templates_by_declaration
                        .get(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let template = self
                        .nominal_templates
                        .get(template_index)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let instance = self.nominal_generic_substitution(
                        node,
                        &template.generic_parameters,
                        substitution,
                    )?;
                    return self
                        .source_nominal_instance(declaration, &instance)
                        .map(CheckedType::Nominal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution.into());
                }
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::GenericType,
                } => {
                    if targs.is_some() {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    let Some(ty) = substitution.type_argument(declaration) else {
                        return self.unsupported(UnsupportedSemanticFeature::Generics, node);
                    };
                    return Ok(ty);
                }
                ResolvedTarget::System(id) => {
                    let index = crate::system_nominal_index(id, self.inventory())
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if targs.is_some() {
                        return self.issue_node(
                            SemanticRule::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    return Ok(CheckedType::Nominal(self.system_nominal(index)?));
                }
                _ => {}
            }
        }
        self.unsupported(UnsupportedSemanticFeature::CompositeValues, node)
    }

    pub(super) fn reject_region_bearing_generic_argument(
        &self,
        argument: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if self.type_node_is_region_bearing_with(argument, substitution)? {
            return self.issue_node(
                SemanticRule::Fn2,
                argument,
                SemanticIssueKind::RegionBearingGenericArgument {
                    mechanical_fix:
                        "make the slice or arena a direct written parameter or result instead of a generic argument",
                },
            );
        }
        Ok(())
    }

    pub(super) fn reject_region_bearing_storage_type(
        &self,
        ty: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if self.type_node_is_region_bearing_with(ty, substitution)? {
            return self.issue_node(
                SemanticRule::Stor5,
                ty,
                SemanticIssueKind::RegionBearingStorage {
                    mechanical_fix:
                        "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
                },
            );
        }
        Ok(())
    }

    fn type_node_is_region_bearing_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<bool, CheckStop> {
        if self.has_fixed(node, FixedTerminal::Slice)?
            || self.has_fixed(node, FixedTerminal::Arena)?
        {
            return Ok(true);
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
            .is_some()
        {
            let usage = self.use_at(node, LexicalUseRole::Type)?;
            if let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::GenericType,
            } = usage.target()
                && let Some(ty) = substitution.type_argument(declaration)
                && (matches!(ty, CheckedType::Slice { .. }) || self.arena_instance(ty)?.is_some())
            {
                return Ok(true);
            }
        }
        for child in self.tree.children_with(node, Production::Type)? {
            if self.type_node_is_region_bearing_with(child, substitution)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn result_type_arguments_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(CheckedType, CheckedType), CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [ok, error] = arguments.as_slice() else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        self.reject_region_bearing_generic_argument(*ok, substitution)?;
        self.reject_region_bearing_generic_argument(*error, substitution)?;
        let Some(ok) = self.tree.first_child_with(*ok, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let Some(error) = self.tree.first_child_with(*error, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let ok = self.parse_type_with(ok, substitution)?;
        let error = self.parse_type_with(error, substitution)?;
        Ok((ok, error))
    }

    pub(super) fn option_type_argument_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<CheckedType, CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [value] = arguments.as_slice() else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        self.reject_region_bearing_generic_argument(*value, substitution)?;
        let Some(value) = self.tree.first_child_with(*value, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let value = self.parse_type_with(value, substitution)?;
        Ok(value)
    }

    pub(super) fn integer_type(&self, node: NodeId) -> Result<Option<IntegerType>, CheckStop> {
        let fixed = [
            (FixedTerminal::I8, IntegerType::I8),
            (FixedTerminal::I16, IntegerType::I16),
            (FixedTerminal::I32, IntegerType::I32),
            (FixedTerminal::I64, IntegerType::I64),
            (FixedTerminal::U8, IntegerType::U8),
            (FixedTerminal::U16, IntegerType::U16),
            (FixedTerminal::U32, IntegerType::U32),
            (FixedTerminal::U64, IntegerType::U64),
        ];
        for (terminal, ty) in fixed {
            if self.has_fixed(node, terminal)? {
                return Ok(Some(ty));
            }
        }
        Ok(None)
    }

    pub(super) fn parse_effects(
        &self,
        node: NodeId,
        parameters: &[ParameterSignature],
    ) -> Result<EffectSet, CheckStop> {
        if self.has_fixed(node, FixedTerminal::Pure)? {
            return Ok(EffectSet::NONE);
        }
        let effects = self.tree.children_with(node, Production::Effect)?;
        let mut previous = None;
        let mut declared = EffectSet::NONE;
        for effect in effects {
            let ordinal = if self.has_fixed(effect, FixedTerminal::Reads)? {
                for region in self.effect_regions(effect)? {
                    declared.add_region_read(region);
                }
                for capability in self.effect_capabilities(effect, parameters)? {
                    declared.add_capability_read(capability);
                }
                0
            } else if self.has_fixed(effect, FixedTerminal::Writes)? {
                for region in self.effect_regions(effect)? {
                    declared.add_region_write(region);
                }
                for capability in self.effect_capabilities(effect, parameters)? {
                    declared.add_capability_write(capability);
                }
                1
            } else if self.has_fixed(effect, FixedTerminal::Allocates)? {
                for terminal in self.tree.direct_token_indices(effect)? {
                    if self.tree.token_bytes(*terminal)? == b"heap" {
                        declared.allocates_heap = true;
                    }
                }
                for region in self.effect_regions(effect)? {
                    declared.add_arena_allocation(region);
                }
                2
            } else if self.has_fixed(effect, FixedTerminal::Traps)? {
                declared.traps = true;
                3
            } else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            if previous.is_some_and(|last| last >= ordinal) {
                return self.issue_node(
                    SemanticRule::Eff1,
                    node,
                    SemanticIssueKind::InvalidEffectRow,
                );
            }
            previous = Some(ordinal);
        }
        Ok(declared)
    }

    fn effect_regions(&self, node: NodeId) -> Result<Vec<crate::DeclarationId>, CheckStop> {
        let path = self.tree.path(node)?;
        let mut uses = self
            .resolved
            .lexical_uses()
            .iter()
            .filter(|usage| {
                usage.role() == LexicalUseRole::EffectRegion && usage.origin().node() == path
            })
            .collect::<Vec<_>>();
        uses.sort_by_key(|usage| usage.origin().role_ordinal());
        uses.into_iter()
            .map(|usage| match usage.target() {
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Region,
                } => Ok(declaration),
                _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
            })
            .collect()
    }

    fn effect_capabilities(
        &self,
        node: NodeId,
        parameters: &[ParameterSignature],
    ) -> Result<Vec<crate::DeclarationId>, CheckStop> {
        let path = self.tree.path(node)?;
        let mut uses = self
            .resolved
            .lexical_uses()
            .iter()
            .filter(|usage| {
                usage.role() == LexicalUseRole::EffectCapability && usage.origin().node() == path
            })
            .collect::<Vec<_>>();
        uses.sort_by_key(|usage| usage.origin().role_ordinal());
        let mut capabilities = Vec::with_capacity(uses.len());
        for usage in uses {
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            let Some(parameter) = parameters
                .iter()
                .find(|parameter| parameter.declaration == declaration)
            else {
                return self.issue_node(
                    SemanticRule::Eff1,
                    node,
                    SemanticIssueKind::InvalidEffectRow,
                );
            };
            if !self.type_carries_one_capability(parameter.ty)? {
                return self.issue_node(
                    SemanticRule::Eff1,
                    node,
                    SemanticIssueKind::InvalidEffectRow,
                );
            }
            capabilities.push(declaration);
        }
        Ok(capabilities)
    }

    /// EFF-1 admits a direct formal only when its complete type carries one
    /// unambiguous system authority root. The count is capped at two because
    /// only zero, exactly one, and ambiguous-many affect this judgment.
    pub(super) fn type_carries_one_capability(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        Ok(self.type_capability_root_count(ty)? == 1)
    }

    pub(super) fn type_capability_root_count(&self, ty: CheckedType) -> Result<u8, CheckStop> {
        Ok(self.type_capability_cardinality(ty)?.1)
    }

    pub(super) fn type_capability_cardinality(
        &self,
        ty: CheckedType,
    ) -> Result<(u8, u8), CheckStop> {
        let mut visiting = HashSet::new();
        self.capability_cardinality(ty, &mut visiting)
    }

    /// Follows a checked value through moves, borrows, and closed-world call
    /// summaries to every direct formal which may supply its one runtime root.
    pub(super) fn capability_origins_of_value(
        &self,
        value: &TypedExpression,
        bindings: &HashMap<crate::DeclarationId, LocalBinding>,
    ) -> Result<Option<CheckedCapabilityOrigins>, CheckStop> {
        let (_, maximum) = self.type_capability_cardinality(value.expression.ty())?;
        if maximum != 1 {
            return Ok(None);
        }
        let root = value
            .borrow
            .as_ref()
            .map(|borrow| borrow.place.root)
            .or(value.holder)
            .or_else(|| value.accesses.first().map(|access| access.place.root));
        let rooted = root.and_then(|root| {
            bindings
                .get(&root)
                .and_then(|binding| binding.capability_origins.clone())
        });
        let origins = rooted.map_or_else(
            || self.expression_capability_origins(&value.expression),
            CapabilityOriginResolution::Finite,
        );
        self.finish_capability_origins(
            origins,
            value.expression.ty(),
            value.expression.carrier().cloned(),
        )
    }

    pub(super) fn give_capability_origins(
        &self,
        arms: &[CheckedMatchArm],
        result_type: CheckedType,
    ) -> Result<Option<CheckedCapabilityOrigins>, CheckStop> {
        let mut origins = CapabilityOriginResolution::Absent;
        let mut fallback = None;
        for arm in arms {
            self.collect_give_capability_origins(&arm.body, &mut origins, &mut fallback);
        }
        self.finish_capability_origins(origins, result_type, fallback)
    }

    fn collect_give_capability_origins(
        &self,
        statements: &[CheckedStatement],
        origins: &mut CapabilityOriginResolution,
        fallback: &mut Option<crate::NodePath>,
    ) {
        for statement in statements {
            match statement {
                CheckedStatement::Give { value, .. } => {
                    if fallback.is_none() {
                        *fallback = value.carrier().cloned();
                    }
                    origins.union(self.expression_capability_origins(value));
                }
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        self.collect_give_capability_origins(&arm.body, origins, fallback);
                    }
                }
                // A nested value initializer owns its own give set.
                CheckedStatement::ValueMatchLet { .. } => {}
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => {
                    self.collect_give_capability_origins(body, origins, fallback);
                }
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
    }

    fn finish_capability_origins(
        &self,
        origins: CapabilityOriginResolution,
        ty: CheckedType,
        fallback: Option<crate::NodePath>,
    ) -> Result<Option<CheckedCapabilityOrigins>, CheckStop> {
        let (minimum, maximum) = self.type_capability_cardinality(ty)?;
        if maximum == 0 {
            return Ok(None);
        }
        if maximum != 1 {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        match origins {
            CapabilityOriginResolution::Absent if minimum == 0 => {
                Ok(Some(CheckedCapabilityOrigins {
                    may_absent: true,
                    may_fresh: false,
                    formals: Vec::new(),
                }))
            }
            CapabilityOriginResolution::Absent => {
                let path = fallback.ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let node = self
                    .tree
                    .node_with_path(&path)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.unsupported(UnsupportedSemanticFeature::CapabilityResultOrigin, node)
            }
            CapabilityOriginResolution::Finite(mut origins) => {
                origins.may_absent &= minimum == 0;
                if minimum == 0 {
                    origins.may_absent = true;
                }
                Ok(Some(origins))
            }
            CapabilityOriginResolution::Unknown(path) if self.deriving_result_authority.get() => {
                let _ = path;
                Ok(Some(CheckedCapabilityOrigins::fresh()))
            }
            CapabilityOriginResolution::Unknown(path) => {
                let node = self
                    .tree
                    .node_with_path(&path)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.unsupported(UnsupportedSemanticFeature::CapabilityResultOrigin, node)
            }
        }
    }

    fn expression_capability_origins(
        &self,
        expression: &CheckedExpression,
    ) -> CapabilityOriginResolution {
        match expression {
            CheckedExpression::Binding {
                capability_origins, ..
            }
            | CheckedExpression::Project {
                capability_origins, ..
            }
            | CheckedExpression::BorrowSystemResource {
                capability_origins, ..
            } => capability_origins.clone().map_or(
                CapabilityOriginResolution::Absent,
                CapabilityOriginResolution::Finite,
            ),
            CheckedExpression::SystemCall {
                operation,
                arguments,
                call,
                ..
            } => match crate::SYSTEM_OPERATIONS
                .get(usize::from(*operation))
                .map(|operation| operation.result_authority)
            {
                Some(crate::SystemResultAuthority::None) => CapabilityOriginResolution::Absent,
                Some(crate::SystemResultAuthority::Fresh) => {
                    CapabilityOriginResolution::Finite(CheckedCapabilityOrigins::fresh())
                }
                Some(crate::SystemResultAuthority::Parameter(ordinal)) => {
                    arguments.get(usize::from(ordinal)).map_or_else(
                        || CapabilityOriginResolution::Unknown(call.clone()),
                        |argument| self.expression_capability_origins(argument),
                    )
                }
                None => CapabilityOriginResolution::Unknown(call.clone()),
            },
            CheckedExpression::UserCall {
                function,
                arguments,
                call,
                ..
            } => match self
                .result_authority_origins
                .borrow()
                .get(function.0 as usize)
                .cloned()
            {
                Some(CheckedResultAuthorityOrigin::NoCapability) => {
                    CapabilityOriginResolution::Absent
                }
                Some(CheckedResultAuthorityOrigin::Finite {
                    may_absent,
                    may_fresh,
                    formals,
                }) => {
                    let finite = CheckedCapabilityOrigins {
                        may_absent,
                        may_fresh,
                        formals: Vec::new(),
                    };
                    let mut resolved = CapabilityOriginResolution::Finite(finite);
                    for formal in formals {
                        let Some(argument) = arguments.get(formal as usize) else {
                            return CapabilityOriginResolution::Unknown(call.clone());
                        };
                        resolved.union(self.expression_capability_origins(argument));
                    }
                    resolved
                }
                Some(CheckedResultAuthorityOrigin::Unknown) | None => {
                    CapabilityOriginResolution::Unknown(call.clone())
                }
            },
            _ => {
                let mut origins = CapabilityOriginResolution::Absent;
                for child in super::super::model::expression_children(expression) {
                    origins.union(self.expression_capability_origins(child));
                }
                origins
            }
        }
    }

    fn capability_cardinality(
        &self,
        ty: CheckedType,
        visiting: &mut HashSet<NominalId>,
    ) -> Result<(u8, u8), CheckStop> {
        let CheckedType::Nominal(id) = ty else {
            return Ok(match ty {
                CheckedType::Buffer { element } => {
                    // A buffer can hold arbitrarily many elements. Even when
                    // its element type carries one root, the complete buffer
                    // type does not identify one direct capability.
                    if self.capability_cardinality(element.ty(), visiting)?.1 == 0 {
                        (0, 0)
                    } else {
                        (0, 2)
                    }
                }
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::Generic(_)
                | CheckedType::GenericInt(_)
                | CheckedType::GenericFloat(_)
                | CheckedType::Array { .. }
                | CheckedType::Slice { .. } => (0, 0),
                CheckedType::Nominal(_) => unreachable!(),
            });
        };
        if !visiting.insert(id) {
            let mut reachable = HashSet::new();
            return Ok(
                if self.type_reaches_system_capability(ty, &mut reachable)? {
                    (0, 2)
                } else {
                    (0, 0)
                },
            );
        }
        let cardinality = match &self.nominal(id)?.kind {
            CheckedNominalKind::SystemResource { nominal } => {
                if crate::system_resource_contract(*nominal)
                    .is_some_and(|contract| contract.resource.carries_authority())
                {
                    (1, 1)
                } else {
                    (0, 0)
                }
            }
            CheckedNominalKind::Struct { fields } => {
                self.capability_fields_cardinality(fields.iter().map(|field| field.ty), visiting)?
            }
            CheckedNominalKind::Enum { variants } => {
                let mut minimum = 2_u8;
                let mut maximum = 0_u8;
                for variant in variants {
                    let (variant_minimum, variant_maximum) = self.capability_fields_cardinality(
                        variant.fields.iter().map(|field| field.ty),
                        visiting,
                    )?;
                    minimum = minimum.min(variant_minimum);
                    maximum = maximum.max(variant_maximum);
                }
                if variants.is_empty() {
                    (0, 0)
                } else {
                    (minimum, maximum)
                }
            }
            CheckedNominalKind::Box { referent } => {
                self.capability_cardinality(*referent, visiting)?
            }
            CheckedNominalKind::Arena { content, .. } => {
                if self.capability_cardinality(*content, visiting)?.1 == 0 {
                    (0, 0)
                } else {
                    (0, 2)
                }
            }
            CheckedNominalKind::ArenaStorage => (0, 0),
        };
        visiting.remove(&id);
        Ok(cardinality)
    }

    /// Graph reachability used only to classify a recursive cardinality
    /// cycle. A pure recursive SCC contributes no capability merely because
    /// it is recursive; a cycle which can reach a real system authority stays
    /// conservatively many until per-leaf recursive origins exist.
    fn type_reaches_system_capability(
        &self,
        ty: CheckedType,
        visited: &mut HashSet<NominalId>,
    ) -> Result<bool, CheckStop> {
        if let CheckedType::Buffer { element } = ty {
            return self.type_reaches_system_capability(element.ty(), visited);
        }
        let CheckedType::Nominal(id) = ty else {
            return Ok(false);
        };
        if !visited.insert(id) {
            return Ok(false);
        }
        match &self.nominal(id)?.kind {
            CheckedNominalKind::SystemResource { nominal } => {
                Ok(crate::system_resource_contract(*nominal)
                    .is_some_and(|contract| contract.resource.carries_authority()))
            }
            CheckedNominalKind::Struct { fields } => {
                for field in fields {
                    if self.type_reaches_system_capability(field.ty, visited)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            CheckedNominalKind::Enum { variants } => {
                for field in variants.iter().flat_map(|variant| &variant.fields) {
                    if self.type_reaches_system_capability(field.ty, visited)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            CheckedNominalKind::Box { referent } => {
                self.type_reaches_system_capability(*referent, visited)
            }
            CheckedNominalKind::Arena { content, .. } => {
                self.type_reaches_system_capability(*content, visited)
            }
            CheckedNominalKind::ArenaStorage => Ok(false),
        }
    }

    fn capability_fields_cardinality(
        &self,
        fields: impl Iterator<Item = CheckedType>,
        visiting: &mut HashSet<NominalId>,
    ) -> Result<(u8, u8), CheckStop> {
        let mut minimum = 0_u8;
        let mut maximum = 0_u8;
        for field in fields {
            let (field_minimum, field_maximum) = self.capability_cardinality(field, visiting)?;
            minimum = minimum.saturating_add(field_minimum).min(2);
            maximum = maximum.saturating_add(field_maximum).min(2);
            if minimum == 2 && maximum == 2 {
                return Ok((2, 2));
            }
        }
        Ok((minimum, maximum))
    }

    /// First source carrier whose runtime value may contain several logical
    /// capability roots. This is a final capability checkpoint, not a source
    /// judgment; callers run it only after all ordinary source checks.
    pub(super) fn first_multi_capability_expression(
        &self,
        statements: &[CheckedStatement],
    ) -> Result<Option<crate::NodePath>, CheckStop> {
        for statement in statements {
            let mut expressions = Vec::new();
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Claim {
                    condition: value, ..
                }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => expressions.push(value),
                CheckedStatement::PropagateLet { scrutinee, .. }
                | CheckedStatement::Match { scrutinee, .. }
                | CheckedStatement::ValueMatchLet { scrutinee, .. } => {
                    expressions.push(scrutinee);
                }
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => {
                            expressions.push(&target.offset);
                        }
                        CheckedSetTarget::BufferIndex(target) => {
                            expressions.push(&target.offset);
                        }
                    }
                    expressions.push(value);
                }
                CheckedStatement::CountedRange { lower, upper, .. } => {
                    expressions.push(lower);
                    expressions.push(upper);
                }
                CheckedStatement::Loop { .. }
                | CheckedStatement::Break { .. }
                | CheckedStatement::Region { .. } => {}
            }
            for expression in expressions {
                if let Some(path) = self.first_multi_capability_child(expression)? {
                    return Ok(Some(path));
                }
            }
            match statement {
                CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } => {
                    for arm in arms {
                        if let Some(path) = self.first_multi_capability_expression(&arm.body)? {
                            return Ok(Some(path));
                        }
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => {
                    if let Some(path) = self.first_multi_capability_expression(body)? {
                        return Ok(Some(path));
                    }
                }
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(None)
    }

    fn first_multi_capability_child(
        &self,
        expression: &CheckedExpression,
    ) -> Result<Option<crate::NodePath>, CheckStop> {
        if self.type_capability_cardinality(expression.ty())?.1 > 1 {
            return expression
                .carrier()
                .cloned()
                .map(Some)
                .ok_or(SemanticCompilerFailure::InvalidResolution.into());
        }
        for child in super::super::model::expression_children(expression) {
            if let Some(path) = self.first_multi_capability_child(child)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// Exact binding identities whose complete runtime value may carry more
    /// than one capability root. Release attribution uses this only to keep
    /// preliminary scratch failure-atomic until the final unsupported stop;
    /// it never licenses a checked function for publication.
    pub(super) fn multi_capability_bindings(
        &self,
        parameters: &[CheckedParameter],
        statements: &[CheckedStatement],
    ) -> Result<HashSet<BindingId>, CheckStop> {
        let mut bindings = HashSet::new();
        for parameter in parameters {
            self.record_multi_capability_binding(&mut bindings, parameter.binding, parameter.ty)?;
        }
        self.collect_multi_capability_bindings(statements, &mut bindings)?;
        Ok(bindings)
    }

    fn record_multi_capability_binding(
        &self,
        bindings: &mut HashSet<BindingId>,
        binding: BindingId,
        ty: CheckedType,
    ) -> Result<(), CheckStop> {
        if self.type_capability_cardinality(ty)?.1 > 1 {
            bindings.insert(binding);
        }
        Ok(())
    }

    fn collect_multi_capability_bindings(
        &self,
        statements: &[CheckedStatement],
        bindings: &mut HashSet<BindingId>,
    ) -> Result<(), CheckStop> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    self.record_multi_capability_binding(bindings, *binding, value.ty())?;
                }
                CheckedStatement::PropagateLet {
                    binding, ok_type, ..
                } => {
                    self.record_multi_capability_binding(bindings, *binding, *ok_type)?;
                }
                CheckedStatement::Replace {
                    binding, target, ..
                } => {
                    self.record_multi_capability_binding(bindings, *binding, target.ty())?;
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    ..
                } => {
                    self.record_multi_capability_binding(bindings, *binding, *result_type)?;
                }
                CheckedStatement::Set { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. }
                | CheckedStatement::CountedRange { .. }
                | CheckedStatement::Loop { .. }
                | CheckedStatement::Match { .. }
                | CheckedStatement::Region { .. } => {}
            }
            match statement {
                CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } => {
                    for arm in arms {
                        for binder in &arm.binders {
                            self.record_multi_capability_binding(
                                bindings,
                                binder.binding,
                                binder.ty,
                            )?;
                        }
                        self.collect_multi_capability_bindings(&arm.body, bindings)?;
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => {
                    self.collect_multi_capability_bindings(body, bindings)?;
                }
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    pub(super) fn parse_const_expression_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<CheckedConst, CheckStop> {
        let digits = self
            .tree
            .direct_tokens_matching(node, &[TerminalPredicate::Digits])?;
        let identifiers = self.tree.direct_identifiers(node)?;
        let mut terms = digits
            .iter()
            .copied()
            .chain(identifiers.iter().copied())
            .collect::<Vec<_>>();
        terms.sort_unstable();
        let Some(operator) = self.tree.first_child_with(node, Production::InfixOp)? else {
            let [term] = terms.as_slice() else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            return self.parse_const_term(node, *term, &identifiers, substitution);
        };
        // The candidate CONST-1 shape: exactly one operation over two terms,
        // evaluated at monomorphization. Both terms concrete evaluates now
        // under the const-eval overflow policy; a symbolic operand interns
        // one symbolic operation instead, and every concrete instantiation
        // re-enters this path with a concrete substitution.
        let operation = self.const_operation(operator)?;
        let [left, right] = terms.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let left = self.parse_const_term(node, *left, &identifiers, substitution)?;
        let right = self.parse_const_term(node, *right, &identifiers, substitution)?;
        if let (CheckedConst::Value(left), CheckedConst::Value(right)) = (left, right) {
            return evaluate_const_operation(operation, left, right)
                .map(CheckedConst::Value)
                .ok_or_else(|| {
                    self.issue_value(
                        SemanticRule::Const1,
                        node,
                        SemanticIssueKind::ConstEvalOverflow {
                            operation: operation.spelling(),
                        },
                    )
                });
        }
        self.combine_const(operation, left, right)
            .ok_or(SemanticCompilerFailure::CounterOverflow.into())
    }

    /// The one const operation of a candidate-grammar `const` tail. The
    /// grammar reuses `infix_op`; the runtime arithmetic modes are rejected
    /// here, so const evaluation has exactly the five bare spellings and
    /// never overloads a runtime overflow mode [CONST-1].
    fn const_operation(&self, operator: NodeId) -> Result<ConstOperation, CheckStop> {
        let [terminal] = self.tree.direct_token_indices(operator)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        match self.tree.token_bytes(*terminal)? {
            b"+" => Ok(ConstOperation::Add),
            b"-" => Ok(ConstOperation::Subtract),
            b"*" => Ok(ConstOperation::Multiply),
            b"/" => Ok(ConstOperation::Divide),
            b"%" => Ok(ConstOperation::Remainder),
            b"+wrap" | b"+checked" | b"+sat" | b"-wrap" | b"-checked" | b"-sat" | b"*wrap"
            | b"*checked" | b"*sat" | b"/checked" | b"%checked" => self.issue_node(
                SemanticRule::Const1,
                operator,
                SemanticIssueKind::ConstRuntimeArithmeticMode {
                    mechanical_fix: "write the bare operator: const evaluation rejects overflow at compile time and has no runtime arithmetic modes",
                },
            ),
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// One `const` term: a bare decimal u64 literal, an integer-typed named
    /// const, or an in-scope const-generic parameter [CONST-1].
    fn parse_const_term(
        &self,
        node: NodeId,
        terminal: usize,
        identifiers: &[usize],
        substitution: &GenericSubstitution,
    ) -> Result<CheckedConst, CheckStop> {
        let Some(ordinal) = identifiers.iter().position(|entry| *entry == terminal) else {
            return std::str::from_utf8(self.tree.token_bytes(terminal)?)
                .ok()
                .and_then(|digits| digits.parse::<u64>().ok())
                .map(CheckedConst::Value)
                .ok_or_else(|| {
                    self.issue_value(
                        SemanticRule::Const1,
                        node,
                        SemanticIssueKind::InvalidConstValue,
                    )
                });
        };
        let uses = self.uses_at_ordered(node, LexicalUseRole::Const)?;
        let usage = uses
            .get(ordinal)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (declaration, named) = match usage.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::NamedConst,
            } => (declaration, true),
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::ConstGeneric,
            } => (declaration, false),
            _ => {
                return self.issue_node(
                    SemanticRule::Const1,
                    node,
                    SemanticIssueKind::InvalidConstValue,
                );
            }
        };
        if !named {
            let Some(value) = substitution.const_argument(declaration) else {
                return self.unsupported(UnsupportedSemanticFeature::Generics, node);
            };
            return Ok(value);
        }
        let Some(constant) = self.constants.get(&declaration).copied() else {
            if self.postcondition_declaration_unavailable(declaration) {
                return Err(CheckStop::PostconditionPrerequisiteUnavailable);
            }
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let constant = self.constant(constant)?;
        let CheckedValue::Integer { ty, bits } = &constant.value else {
            return self.issue_node(
                SemanticRule::Const1,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        };
        if ty.signed() && bits & (1_u64 << (ty.width() - 1)) != 0 {
            return self.issue_node(
                SemanticRule::Const1,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        Ok(CheckedConst::Value(*bits))
    }

    pub(super) fn parse_const_value(
        &self,
        node: NodeId,
        expected: CheckedType,
    ) -> Result<CheckedValue, CheckStop> {
        // The construction shape is decided first: its direct tokens include
        // the field-label IDENTs, so the single-identifier reference reader
        // below must never see it.
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
            .is_some()
        {
            return self.parse_const_construction(node, expected);
        }
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicate::Literal)?
        {
            let value = self.parse_literal(node, self.tree.token_bytes(literal)?)?;
            if value.ty() == expected {
                return Ok(value);
            }
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::Identifier)?
            .is_some()
        {
            let usage = self.use_at(node, LexicalUseRole::ConstValue)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::NamedConst,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            let Some(id) = self.constants.get(&declaration).copied() else {
                if self.postcondition_declaration_unavailable(declaration) {
                    return Err(CheckStop::PostconditionPrerequisiteUnavailable);
                }
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            let constant = self.constant(id)?;
            if constant.ty == expected {
                return Ok(constant.value.clone());
            }
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        let CheckedType::Array { element, length } = expected else {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        };
        if !self.has_fixed(node, FixedTerminal::LeftBracket)? {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let entries = self.tree.children_with(node, Production::Cvalue)?;
        let Some(length) = length.value() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if u64::try_from(entries.len()).ok() != Some(length) {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        let element_type = element.ty();
        let mut elements = Vec::with_capacity(entries.len());
        for entry in entries {
            elements.push(self.parse_const_value(entry, element_type)?);
        }
        Ok(CheckedValue::Array {
            ty: expected,
            elements,
        })
    }

    /// One construction cvalue [CONST-2 candidate]: `TYPEID(field: cvalue,
    /// ...)` totally defining a struct-typed constant. The constructor must
    /// name the expected struct, and the written fields must be the declared
    /// fields in exact declared order [GRAM-8], each field value a cvalue of
    /// the declared field type.
    fn parse_const_construction(
        &self,
        node: NodeId,
        expected: CheckedType,
    ) -> Result<CheckedValue, CheckStop> {
        if !crate::semantic::V031_CANDIDATE_SEMANTICS {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        // Written generic construction arguments in const position are not
        // implemented yet: valid under the candidate's eligibility relation
        // only through concrete instances, which this version does not intern
        // from a cvalue.
        if self
            .tree
            .first_child_with(node, Production::Targs)?
            .is_some()
        {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
        }
        let CheckedType::Nominal(id) = expected else {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        };
        let (constructor_name, declared_fields) = {
            let nominal = self.nominal(id)?;
            let super::super::model::CheckedNominalKind::Struct { fields } = &nominal.kind else {
                return self.issue_node(
                    SemanticRule::Const2,
                    node,
                    SemanticIssueKind::InvalidConstValue,
                );
            };
            (nominal.name.clone(), fields.clone())
        };
        let expected_template = self
            .source_nominal_instances
            .get(id.0 as usize)
            .and_then(|instance| instance.as_ref().map(|(template, _)| *template))
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let usage = self.use_at(node, LexicalUseRole::Construct)?;
        let ResolvedTarget::Source { declaration, .. } = usage.target() else {
            // A prelude or system constructor never names a const-eligible
            // struct.
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        };
        let written_template = match self.constructor_templates_by_declaration.get(&declaration) {
            Some(super::ConstructorTemplate::Struct { template }) => *template,
            _ => {
                return self.issue_node(
                    SemanticRule::Const2,
                    node,
                    SemanticIssueKind::InvalidConstValue,
                );
            }
        };
        if written_template != expected_template {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        let labels = self.tree.direct_identifiers(node)?;
        let values = self.tree.children_with(node, Production::Cvalue)?;
        let declared_field_names = declared_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<_>>();
        if labels.len() != declared_fields.len() || values.len() != declared_fields.len() {
            return self.issue_node(
                SemanticRule::Gram8,
                node,
                SemanticIssueKind::InvalidConstructionFields {
                    constructor: constructor_name,
                    declared_fields: declared_field_names,
                },
            );
        }
        let mut fields = Vec::with_capacity(declared_fields.len());
        for ((label, value), declared) in labels.iter().zip(&values).zip(&declared_fields) {
            if self.tree.token_bytes(*label)? != declared.name.as_bytes() {
                return self.issue_node(
                    SemanticRule::Gram8,
                    node,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: declared_field_names,
                    },
                );
            }
            fields.push(self.parse_const_value(*value, declared.ty)?);
        }
        Ok(CheckedValue::Struct {
            ty: expected,
            fields,
        })
    }

    pub(super) fn constant(&self, id: CheckedConstantId) -> Result<&CheckedConstant, CheckStop> {
        self.checked_constants
            .get(id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn parse_const_type(&self, node: NodeId) -> Result<CheckedType, CheckStop> {
        let directly_ineligible = (!crate::semantic::V031_CANDIDATE_SEMANTICS
            && self
                .tree
                .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
                .is_some())
            || self.has_fixed(node, FixedTerminal::Slice)?
            || self.has_fixed(node, FixedTerminal::Box)?
            || self.has_fixed(node, FixedTerminal::Arena)?
            || self.has_fixed(node, FixedTerminal::Buffer)?;
        if directly_ineligible {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            );
        }
        let ty = self.parse_type(node)?;
        if self.const_eligible_type(ty)? {
            Ok(ty)
        } else {
            self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            )
        }
    }

    /// The CONST-2 const-eligibility relation: a primitive, an array of
    /// const-eligible flat elements, or — under the v0.31 candidate — a
    /// source struct whose every field type is const-eligible. Enums, boxes,
    /// buffers, slices, arenas, and generics remain ineligible (a const is
    /// pure static rodata: no allocation, no region, no drop).
    fn const_eligible_type(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        Ok(match ty {
            CheckedType::Unit | CheckedType::Integer(_) | CheckedType::Float(_) => true,
            CheckedType::Array { element, .. } => {
                matches!(
                    element,
                    CheckedFlatElement::Unit
                        | CheckedFlatElement::Integer(_)
                        | CheckedFlatElement::Float(_)
                )
            }
            CheckedType::Nominal(id) if crate::semantic::V031_CANDIDATE_SEMANTICS => {
                match &self.nominal(id)?.kind {
                    super::super::model::CheckedNominalKind::Struct { fields } => {
                        let fields = fields.iter().map(|field| field.ty).collect::<Vec<_>>();
                        for field in fields {
                            if !self.const_eligible_type(field)? {
                                return Ok(false);
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }
            CheckedType::Bool
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_)
            | CheckedType::Nominal(_) => false,
            CheckedType::Slice { .. } | CheckedType::Buffer { .. } => false,
        })
    }

    pub(super) fn checked_flat_element(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<CheckedFlatElement, CheckStop> {
        match self.flat_element(ty)? {
            Some(element) => Ok(element),
            None => self.issue_node(SemanticRule::Type2, node, SemanticIssueKind::TypeMismatch),
        }
    }

    /// The [TYPE-2] buffer element domain: every flat copy element, plus a
    /// region-free affine nominal stored by value. Arrays and slices keep
    /// [`Self::flat_element`]'s copy domain.
    pub(super) fn buffer_element(
        &self,
        ty: CheckedType,
    ) -> Result<Option<CheckedFlatElement>, CheckStop> {
        if let Some(element) = self.flat_element(ty)? {
            return Ok(Some(element));
        }
        Ok(match ty {
            CheckedType::Nominal(id) => match self.nominal(id)?.kind {
                // An arena is region-bearing [STOR-5] and the region
                // allocation list is compiler-owned; neither is an element.
                super::super::model::CheckedNominalKind::Arena { .. }
                | super::super::model::CheckedNominalKind::ArenaStorage => None,
                _ => Some(CheckedFlatElement::Nominal(id)),
            },
            _ => None,
        })
    }

    pub(super) fn flat_element(
        &self,
        ty: CheckedType,
    ) -> Result<Option<CheckedFlatElement>, CheckStop> {
        Ok(match ty {
            CheckedType::Unit => Some(CheckedFlatElement::Unit),
            CheckedType::Bool => Some(CheckedFlatElement::Bool),
            CheckedType::Integer(ty) => Some(CheckedFlatElement::Integer(ty)),
            CheckedType::Float(ty) => Some(CheckedFlatElement::Float(ty)),
            CheckedType::GenericInt(declaration) => {
                Some(CheckedFlatElement::GenericInt(declaration))
            }
            CheckedType::GenericFloat(declaration) => {
                Some(CheckedFlatElement::GenericFloat(declaration))
            }
            CheckedType::Nominal(id) if self.nominal(id)?.is_copy() => {
                Some(CheckedFlatElement::TagOnlyNominal(id))
            }
            CheckedType::Generic(_)
            | CheckedType::Nominal(_)
            | CheckedType::Array { .. }
            | CheckedType::Slice { .. }
            | CheckedType::Buffer { .. } => None,
        })
    }

    pub(super) fn parse_literal(
        &self,
        node: NodeId,
        bytes: &[u8],
    ) -> Result<CheckedValue, CheckStop> {
        if bytes == b"unit" {
            return Ok(CheckedValue::Unit);
        }
        if bytes.ends_with(b"_f32") || bytes.ends_with(b"_f64") {
            return parse_float_literal(bytes).ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Form7,
                    node,
                    SemanticIssueKind::InvalidFloatLiteral,
                )
            });
        }
        parse_integer(bytes).ok_or_else(|| {
            self.issue_value(
                SemanticRule::Form7,
                node,
                SemanticIssueKind::InvalidIntegerLiteral,
            )
        })
    }

    pub(super) fn check_message(&self, node: NodeId) -> Result<String, CheckStop> {
        let terminal = self
            .tree
            .direct_token_with(node, TerminalPredicate::String)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let bytes = self.tree.token_bytes(terminal)?;
        let interior = bytes
            .strip_prefix(b"\"")
            .and_then(|bytes| bytes.strip_suffix(b"\""))
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mut decoded = Vec::with_capacity(interior.len());
        let mut cursor = 0;
        while cursor < interior.len() {
            if interior[cursor] != b'\\' {
                decoded.push(interior[cursor]);
                cursor += 1;
                continue;
            }
            let escaped = *interior
                .get(cursor + 1)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            decoded.push(match escaped {
                b'\\' => b'\\',
                b'"' => b'"',
                b'n' => b'\n',
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            });
            cursor += 2;
        }
        String::from_utf8(decoded)
            .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding.into())
    }
}

fn parse_integer(bytes: &[u8]) -> Option<CheckedValue> {
    let split = bytes.iter().rposition(|byte| *byte == b'_')?;
    let ty = match bytes.get(split + 1..)? {
        b"i8" => IntegerType::I8,
        b"i16" => IntegerType::I16,
        b"i32" => IntegerType::I32,
        b"i64" => IntegerType::I64,
        b"u8" => IntegerType::U8,
        b"u16" => IntegerType::U16,
        b"u32" => IntegerType::U32,
        b"u64" => IntegerType::U64,
        _ => return None,
    };
    let negative = bytes.first() == Some(&b'-');
    if negative && !ty.signed() {
        return None;
    }
    let digits = bytes.get(usize::from(negative)..split)?;
    if digits.is_empty()
        || (digits.len() > 1 && digits.first() == Some(&b'0'))
        || (negative && digits == b"0")
    {
        return None;
    }
    let magnitude = std::str::from_utf8(digits).ok()?.parse::<u128>().ok()?;
    let width = ty.width();
    let bits = if ty.signed() {
        let maximum = (1_u128 << (width - 1)) - 1;
        let minimum_magnitude = 1_u128 << (width - 1);
        if (!negative && magnitude > maximum) || (negative && magnitude > minimum_magnitude) {
            return None;
        }
        if negative {
            let modulus = 1_u128 << width;
            u64::try_from(modulus - magnitude).ok()?
        } else {
            u64::try_from(magnitude).ok()?
        }
    } else {
        let maximum = (1_u128 << width) - 1;
        if magnitude > maximum {
            return None;
        }
        u64::try_from(magnitude).ok()?
    };
    Some(CheckedValue::Integer { ty, bits })
}
