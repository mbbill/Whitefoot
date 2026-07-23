use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminalV0_14, TerminalPredicateV0_14};
use crate::{
    DeclarationClass, DeclarationRole, LexicalUseRole, PreludeDeclarationId, ProductionV0_14,
    ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_14,
    UnsupportedSemanticFeatureV0_14,
};

use super::super::model::{CheckedType, CheckedValue, IntegerType};
use super::{CheckStop, Checker, ParameterSignature, PreludeType};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn parse_parameters(
        &self,
        function: NodeId,
    ) -> Result<Vec<ParameterSignature>, CheckStop> {
        let Some(list) = self
            .tree
            .first_child_with(function, ProductionV0_14::ParamList)?
        else {
            return Ok(Vec::new());
        };
        let mut parameters = Vec::new();
        for node in self.tree.children_with(list, ProductionV0_14::Param)? {
            let declaration = self.declaration_at(node, DeclarationRole::Parameter)?;
            let mode = self
                .tree
                .first_child_with(node, ProductionV0_14::Mode)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.require_own_mode(mode)?;
            let ty_node = self
                .tree
                .first_child_with(node, ProductionV0_14::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            parameters.push(ParameterSignature {
                declaration: declaration.id(),
                name: declaration.spelling().to_owned(),
                ty: self.parse_type(ty_node)?,
            });
        }
        Ok(parameters)
    }

    pub(super) fn parse_rtype(&self, node: NodeId) -> Result<CheckedType, CheckStop> {
        let mode = self
            .tree
            .first_child_with(node, ProductionV0_14::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.require_own_mode(mode)?;
        let ty = self
            .tree
            .first_child_with(node, ProductionV0_14::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.parse_type(ty)
    }

    pub(super) fn require_own_mode(&self, node: NodeId) -> Result<(), CheckStop> {
        if self.has_fixed(node, FixedTerminalV0_14::Own)? {
            Ok(())
        } else {
            self.unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, node)
        }
    }

    pub(super) fn parse_type(&self, node: NodeId) -> Result<CheckedType, CheckStop> {
        let targs = self.tree.first_child_with(node, ProductionV0_14::Targs)?;
        if let Some(ty) = self.integer_type(node)? {
            if targs.is_some() {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    node,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            return Ok(CheckedType::Integer(ty));
        }
        if self.has_fixed(node, FixedTerminalV0_14::Unit)? {
            if targs.is_some() {
                return self.issue_node(
                    SemanticRuleV0_14::Type5,
                    node,
                    SemanticIssueKind::TypeMismatch,
                );
            }
            return Ok(CheckedType::Unit);
        }
        if self.has_fixed(node, FixedTerminalV0_14::F32)?
            || self.has_fixed(node, FixedTerminalV0_14::F64)?
        {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::FloatingPoint, node);
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::TypeIdentifier)?
            .is_some()
        {
            let usage = self.use_at(node, LexicalUseRole::Type)?;
            match usage.target() {
                ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(0) => {
                    if targs.is_some() {
                        return self.issue_node(
                            SemanticRuleV0_14::Type5,
                            node,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    return Ok(CheckedType::Bool);
                }
                ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(8) => {
                    let (ok, error) = self.result_type_arguments(node)?;
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
                            SemanticRuleV0_14::Type5,
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
                        .unsupported(UnsupportedSemanticFeatureV0_14::PreludeNominalValues, node);
                }
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::NominalType,
                } => {
                    return self
                        .nominals_by_declaration
                        .get(&declaration)
                        .copied()
                        .map(CheckedType::Nominal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution.into())
                        .and_then(|ty| {
                            if let Some(targs) = targs {
                                self.unsupported(UnsupportedSemanticFeatureV0_14::Generics, targs)
                            } else {
                                Ok(ty)
                            }
                        });
                }
                _ => {}
            }
        }
        self.unsupported(UnsupportedSemanticFeatureV0_14::CompositeValues, node)
    }

    pub(super) fn prelude_type_ordinal(&self, node: NodeId) -> Result<Option<u8>, CheckStop> {
        if self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::TypeIdentifier)?
            .is_none()
        {
            return Ok(None);
        }
        let usage = self.use_at(node, LexicalUseRole::Type)?;
        Ok(match usage.target() {
            ResolvedTarget::Prelude(id) => Some(id.ordinal()),
            _ => None,
        })
    }

    pub(super) fn result_type_arguments(
        &self,
        node: NodeId,
    ) -> Result<(CheckedType, CheckedType), CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, ProductionV0_14::Targs)? else {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        };
        let arguments = self.tree.children_with(targs, ProductionV0_14::Targ)?;
        let [ok, error] = arguments.as_slice() else {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        };
        let Some(ok) = self.tree.first_child_with(*ok, ProductionV0_14::Type)? else {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        };
        let Some(error) = self.tree.first_child_with(*error, ProductionV0_14::Type)? else {
            return self.issue_node(
                SemanticRuleV0_14::Type5,
                node,
                SemanticIssueKind::TypeMismatch,
            );
        };
        Ok((self.parse_type(ok)?, self.parse_type(error)?))
    }

    pub(super) fn integer_type(&self, node: NodeId) -> Result<Option<IntegerType>, CheckStop> {
        let fixed = [
            (FixedTerminalV0_14::I8, IntegerType::I8),
            (FixedTerminalV0_14::I16, IntegerType::I16),
            (FixedTerminalV0_14::I32, IntegerType::I32),
            (FixedTerminalV0_14::I64, IntegerType::I64),
            (FixedTerminalV0_14::U8, IntegerType::U8),
            (FixedTerminalV0_14::U16, IntegerType::U16),
            (FixedTerminalV0_14::U32, IntegerType::U32),
            (FixedTerminalV0_14::U64, IntegerType::U64),
        ];
        for (terminal, ty) in fixed {
            if self.has_fixed(node, terminal)? {
                return Ok(Some(ty));
            }
        }
        Ok(None)
    }

    pub(super) fn parse_effects(&self, node: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(node, FixedTerminalV0_14::Pure)? {
            return Ok(false);
        }
        let effects = self.tree.children_with(node, ProductionV0_14::Effect)?;
        let mut previous = None;
        let mut has_traps = false;
        let mut unsupported = None;
        for effect in effects {
            let (ordinal, feature) = if self.has_fixed(effect, FixedTerminalV0_14::Reads)? {
                (0, Some(UnsupportedSemanticFeatureV0_14::EffectFamily))
            } else if self.has_fixed(effect, FixedTerminalV0_14::Writes)? {
                (1, Some(UnsupportedSemanticFeatureV0_14::EffectFamily))
            } else if self.has_fixed(effect, FixedTerminalV0_14::Allocates)? {
                (2, Some(UnsupportedSemanticFeatureV0_14::EffectFamily))
            } else if self.has_fixed(effect, FixedTerminalV0_14::Traps)? {
                has_traps = true;
                (3, None)
            } else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            if previous.is_some_and(|last| last >= ordinal) {
                return self.issue_node(
                    SemanticRuleV0_14::Eff1,
                    node,
                    SemanticIssueKind::InvalidEffectRow,
                );
            }
            previous = Some(ordinal);
            if feature.is_some() && unsupported.is_none() {
                unsupported = Some(effect);
            }
        }
        if let Some(effect) = unsupported {
            return self.unsupported(UnsupportedSemanticFeatureV0_14::EffectFamily, effect);
        }
        Ok(has_traps)
    }

    pub(super) fn parse_const_value(&self, node: NodeId) -> Result<CheckedValue, CheckStop> {
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::Literal)?
        {
            return self.parse_literal(node, self.tree.token_bytes(literal)?);
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::Identifier)?
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
            return self
                .constants
                .get(&declaration)
                .copied()
                .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into());
        }
        self.unsupported(UnsupportedSemanticFeatureV0_14::CompositeValues, node)
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
            return self.unsupported(UnsupportedSemanticFeatureV0_14::FloatingPoint, node);
        }
        parse_integer(bytes).ok_or_else(|| {
            self.issue_value(
                SemanticRuleV0_14::Form7,
                node,
                SemanticIssueKind::InvalidIntegerLiteral,
            )
        })
    }

    pub(super) fn check_message(&self, node: NodeId) -> Result<String, CheckStop> {
        let terminal = self
            .tree
            .direct_token_with(node, TerminalPredicateV0_14::String)?
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
