use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationClass, DeclarationRole, LexicalUseRole, PreludeDeclarationId, Production,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
    UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedConst, CheckedConstant, CheckedConstantId, CheckedFlatElement, CheckedMode,
    CheckedNominalKind, CheckedType, CheckedValue, IntegerType,
};
use super::generics::GenericSubstitution;
use super::{CheckStop, Checker, EffectSet, ParameterSignature, PreludeType};

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
            if mode != CheckedMode::Own {
                let supported = matches!(ty, CheckedType::Buffer { .. })
                    || matches!(
                        ty,
                        CheckedType::Nominal(nominal)
                            if matches!(
                                self.nominal(nominal)?.kind,
                                CheckedNominalKind::Struct { .. }
                            )
                    );
                if !supported {
                    return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, node);
                }
            }
            parameters.push(ParameterSignature {
                declaration: declaration.id(),
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
        if self.has_fixed(node, FixedTerminal::F32)? || self.has_fixed(node, FixedTerminal::F64)? {
            return self.unsupported(UnsupportedSemanticFeature::FloatingPoint, node);
        }
        if self.has_fixed(node, FixedTerminal::Array)? {
            let element_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
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
            let element_type = self.parse_type_with(element_node, substitution)?;
            return Ok(CheckedType::Buffer {
                element: self.checked_flat_element(element_type, element_node)?,
            });
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
                _ => {}
            }
        }
        self.unsupported(UnsupportedSemanticFeature::CompositeValues, node)
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
        let Some(ok) = self.tree.first_child_with(*ok, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let Some(error) = self.tree.first_child_with(*error, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        Ok((
            self.parse_type_with(ok, substitution)?,
            self.parse_type_with(error, substitution)?,
        ))
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
        let Some(value) = self.tree.first_child_with(*value, Production::Type)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        self.parse_type_with(value, substitution)
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

    pub(super) fn parse_effects(&self, node: NodeId) -> Result<EffectSet, CheckStop> {
        if self.has_fixed(node, FixedTerminal::Pure)? {
            return Ok(EffectSet::NONE);
        }
        let effects = self.tree.children_with(node, Production::Effect)?;
        let mut previous = None;
        let mut declared = EffectSet::NONE;
        for effect in effects {
            let ordinal = if self.has_fixed(effect, FixedTerminal::Reads)? {
                for region in self.effect_regions(effect)? {
                    declared.add_read(region);
                }
                0
            } else if self.has_fixed(effect, FixedTerminal::Writes)? {
                for region in self.effect_regions(effect)? {
                    declared.add_write(region);
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

    pub(super) fn parse_const_expression_with(
        &self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<CheckedConst, CheckStop> {
        if let Some(digits) = self
            .tree
            .direct_token_with(node, TerminalPredicate::Digits)?
        {
            return std::str::from_utf8(self.tree.token_bytes(digits)?)
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
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::Identifier)?
            .is_some()
        {
            let usage = self.use_at(node, LexicalUseRole::Const)?;
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
            let constant = self.constant(
                *self
                    .constants
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            )?;
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
            return Ok(CheckedConst::Value(*bits));
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    pub(super) fn parse_const_value(
        &self,
        node: NodeId,
        expected: CheckedType,
    ) -> Result<CheckedValue, CheckStop> {
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
            let id = *self
                .constants
                .get(&declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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

    pub(super) fn constant(&self, id: CheckedConstantId) -> Result<&CheckedConstant, CheckStop> {
        self.checked_constants
            .get(id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn parse_const_type(&self, node: NodeId) -> Result<CheckedType, CheckStop> {
        let directly_ineligible = self
            .tree
            .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
            .is_some()
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
        let eligible = match ty {
            CheckedType::Unit | CheckedType::Integer(_) => true,
            CheckedType::Array { element, .. } => {
                matches!(
                    element,
                    CheckedFlatElement::Unit | CheckedFlatElement::Integer(_)
                )
            }
            CheckedType::Bool
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::Nominal(_) => false,
            CheckedType::Buffer { .. } => false,
        };
        if eligible {
            Ok(ty)
        } else {
            self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::InvalidConstValue,
            )
        }
    }

    fn checked_flat_element(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<CheckedFlatElement, CheckStop> {
        match ty {
            CheckedType::Unit => Ok(CheckedFlatElement::Unit),
            CheckedType::Bool => Ok(CheckedFlatElement::Bool),
            CheckedType::Integer(ty) => Ok(CheckedFlatElement::Integer(ty)),
            CheckedType::GenericInt(declaration) => Ok(CheckedFlatElement::GenericInt(declaration)),
            CheckedType::Nominal(id) if self.nominal(id)?.is_copy() => {
                Ok(CheckedFlatElement::TagOnlyNominal(id))
            }
            CheckedType::Generic(_)
            | CheckedType::Nominal(_)
            | CheckedType::Array { .. }
            | CheckedType::Buffer { .. } => {
                self.issue_node(SemanticRule::Type2, node, SemanticIssueKind::TypeMismatch)
            }
        }
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
            return self.unsupported(UnsupportedSemanticFeature::FloatingPoint, node);
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
