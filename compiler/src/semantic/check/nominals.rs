use crate::{PreludeDeclarationId, SemanticCompilerFailure, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedConstructor, CheckedField, CheckedFlatElement, CheckedNominal, CheckedNominalKind,
    CheckedType, CheckedVariant, IntegerType, NominalId,
};
use super::{CheckStop, Checker, PreludeType};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn reject_recursive_nominal_layouts(&self) -> Result<(), CheckStop> {
        let mut colors = vec![0_u8; self.nominals.len()];
        for root in 0..self.nominals.len() {
            if colors[root] != 0 {
                continue;
            }
            colors[root] = 1;
            let mut stack = vec![(root, 0_usize, self.nominal_dependencies(root)?)];
            while let Some((current, next, dependencies)) = stack.last_mut() {
                if *next == dependencies.len() {
                    colors[*current] = 2;
                    stack.pop();
                    continue;
                }
                let dependency = dependencies[*next].0 as usize;
                *next += 1;
                match colors.get(dependency).copied() {
                    Some(0) => {
                        colors[dependency] = 1;
                        stack.push((dependency, 0, self.nominal_dependencies(dependency)?));
                    }
                    Some(1) => {
                        let node = stack
                            .iter()
                            .filter_map(|(index, _, _)| {
                                self.nominal_nodes.get(*index).copied().flatten()
                            })
                            .next()
                            .or_else(|| self.nominal_nodes.get(dependency).copied().flatten())
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        return self
                            .unsupported(UnsupportedSemanticFeature::RecursiveNominalLayout, node);
                    }
                    Some(2) => {}
                    _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                }
            }
        }
        Ok(())
    }

    fn nominal_dependencies(&self, index: usize) -> Result<Vec<NominalId>, CheckStop> {
        let nominal = self
            .nominals
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let fields: Vec<&CheckedField> = match &nominal.kind {
            CheckedNominalKind::Struct { fields } => fields.iter().collect(),
            CheckedNominalKind::Enum { variants } => variants
                .iter()
                .flat_map(|variant| variant.fields.iter())
                .collect(),
            CheckedNominalKind::Box { .. } | CheckedNominalKind::SystemResource { .. } => {
                Vec::new()
            }
        };
        Ok(fields
            .into_iter()
            .filter_map(|field| match field.ty {
                CheckedType::Nominal(id)
                    if !matches!(
                        self.nominals
                            .get(id.0 as usize)
                            .map(|nominal| &nominal.kind),
                        Some(CheckedNominalKind::Box { .. })
                    ) =>
                {
                    Some(id)
                }
                _ => None,
            })
            .collect())
    }

    pub(super) fn nominal(&self, id: NominalId) -> Result<&CheckedNominal, CheckStop> {
        self.nominals
            .get(id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn is_copy_type(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        Ok(match ty {
            CheckedType::Nominal(id) => self.nominal(id)?.is_copy(),
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_) => true,
            CheckedType::Generic(_)
            | CheckedType::Array { .. }
            | CheckedType::Slice { .. }
            | CheckedType::Buffer { .. } => false,
        })
    }

    pub(super) fn prelude_type(&self, id: NominalId) -> Option<PreludeType> {
        self.prelude_types.get(id.0 as usize).copied().flatten()
    }

    pub(super) fn prelude_nominal(&self, ty: PreludeType) -> Result<NominalId, CheckStop> {
        self.prelude_nominals
            .get(&ty)
            .copied()
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn intern_box_nominal(
        &mut self,
        referent: CheckedType,
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.box_nominals.get(&referent) {
            return Ok(*id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let name = format!("box<{}>", self.checked_type_name(referent)?);
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: CheckedNominalKind::Box { referent },
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        if self.box_nominals.insert(referent, id).is_some() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }

    pub(super) fn register_prelude_nominals(&mut self) -> Result<(), CheckStop> {
        self.intern_prelude_nominal(PreludeType::Overflow)?;
        self.intern_prelude_nominal(PreludeType::DivError)?;
        self.intern_prelude_nominal(PreludeType::NarrowError)?;
        Ok(())
    }

    /// Returns the already-interned instance of one [SYS-2] nominal type.
    pub(super) fn system_nominal(&self, index: u8) -> Result<NominalId, CheckStop> {
        self.system_nominals
            .get(&index)
            .copied()
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    /// Interns one [SYS-2] nominal type: an opaque resource becomes a
    /// [`CheckedNominalKind::SystemResource`]; an outcome enum becomes an
    /// ordinary checked enum whose variants and typed fields come from the
    /// catalog, so matching, affinity, and drops use the one normal path.
    pub(super) fn intern_system_nominal(&mut self, index: u8) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.system_nominals.get(&index) {
            return Ok(*id);
        }
        let nominal = crate::SYSTEM_NOMINALS
            .get(usize::from(index))
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let kind = if nominal.opaque {
            CheckedNominalKind::SystemResource { nominal: index }
        } else {
            let mut variants = Vec::new();
            for (constructor_index, constructor) in crate::SYSTEM_CONSTRUCTORS.iter().enumerate() {
                if constructor.owner != index {
                    continue;
                }
                let mut fields = Vec::with_capacity(constructor.fields.len());
                for field in constructor.fields {
                    fields.push(CheckedField {
                        name: field.name.to_owned(),
                        ty: self.ensure_system_type(field.ty)?,
                    });
                }
                let declaration = u8::try_from(constructor_index)
                    .ok()
                    .and_then(crate::system_constructor_declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let tag = u32::try_from(variants.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                variants.push(CheckedVariant {
                    name: constructor.spelling.to_owned(),
                    constructor: CheckedConstructor::System(declaration),
                    tag,
                    fields,
                });
            }
            CheckedNominalKind::Enum { variants }
        };
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        self.nominals.push(CheckedNominal {
            id,
            name: nominal.spelling.to_owned(),
            kind,
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        if self.system_nominals.insert(index, id).is_some() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }

    /// Maps one written [SYS-2] table type to its checked type, interning
    /// every system nominal and prelude `Result` instantiation it needs.
    pub(super) fn ensure_system_type(
        &mut self,
        ty: crate::SystemTypeRef,
    ) -> Result<CheckedType, CheckStop> {
        Ok(match ty {
            crate::SystemTypeRef::U8 => CheckedType::Integer(IntegerType::U8),
            crate::SystemTypeRef::U32 => CheckedType::Integer(IntegerType::U32),
            crate::SystemTypeRef::U64 => CheckedType::Integer(IntegerType::U64),
            crate::SystemTypeRef::BufferU8 => CheckedType::Buffer {
                element: CheckedFlatElement::Integer(IntegerType::U8),
            },
            crate::SystemTypeRef::Nominal(index) => {
                CheckedType::Nominal(self.intern_system_nominal(index)?)
            }
            crate::SystemTypeRef::Result { ok, err } => {
                let ok = match ok {
                    crate::SystemResultPayload::U64 => CheckedType::Integer(IntegerType::U64),
                    crate::SystemResultPayload::Nominal(index) => {
                        CheckedType::Nominal(self.intern_system_nominal(index)?)
                    }
                };
                let error = CheckedType::Nominal(self.intern_system_nominal(err)?);
                CheckedType::Nominal(self.intern_prelude_nominal(PreludeType::Result(ok, error))?)
            }
        })
    }

    /// Read-only variant of [`Self::ensure_system_type`] for use during
    /// function checking, after the mutable pre-pass interned every needed
    /// instance.
    pub(super) fn system_type(&self, ty: crate::SystemTypeRef) -> Result<CheckedType, CheckStop> {
        Ok(match ty {
            crate::SystemTypeRef::U8 => CheckedType::Integer(IntegerType::U8),
            crate::SystemTypeRef::U32 => CheckedType::Integer(IntegerType::U32),
            crate::SystemTypeRef::U64 => CheckedType::Integer(IntegerType::U64),
            crate::SystemTypeRef::BufferU8 => CheckedType::Buffer {
                element: CheckedFlatElement::Integer(IntegerType::U8),
            },
            crate::SystemTypeRef::Nominal(index) => {
                CheckedType::Nominal(self.system_nominal(index)?)
            }
            crate::SystemTypeRef::Result { ok, err } => {
                let ok = match ok {
                    crate::SystemResultPayload::U64 => CheckedType::Integer(IntegerType::U64),
                    crate::SystemResultPayload::Nominal(index) => {
                        CheckedType::Nominal(self.system_nominal(index)?)
                    }
                };
                let error = CheckedType::Nominal(self.system_nominal(err)?);
                CheckedType::Nominal(self.prelude_nominal(PreludeType::Result(ok, error))?)
            }
        })
    }

    pub(super) fn intern_prelude_nominal(
        &mut self,
        ty: PreludeType,
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.prelude_nominals.get(&ty) {
            return Ok(*id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let (name, variants) = match ty {
            PreludeType::Option(value) => (
                format!("Option<{}>", self.checked_type_name(value)?),
                vec![
                    CheckedVariant {
                        name: "None".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(5)),
                        tag: 0,
                        fields: Vec::new(),
                    },
                    CheckedVariant {
                        name: "Some".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(6)),
                        tag: 1,
                        fields: vec![CheckedField {
                            name: "value".to_owned(),
                            ty: value,
                        }],
                    },
                ],
            ),
            PreludeType::Result(ok, error) => (
                format!(
                    "Result<{}, {}>",
                    self.checked_type_name(ok)?,
                    self.checked_type_name(error)?
                ),
                vec![
                    CheckedVariant {
                        name: "Ok".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(11)),
                        tag: 0,
                        fields: vec![CheckedField {
                            name: "value".to_owned(),
                            ty: ok,
                        }],
                    },
                    CheckedVariant {
                        name: "Err".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(13)),
                        tag: 1,
                        fields: vec![CheckedField {
                            name: "error".to_owned(),
                            ty: error,
                        }],
                    },
                ],
            ),
            PreludeType::Overflow => (
                "Overflow".to_owned(),
                vec![CheckedVariant {
                    name: "Overflow".to_owned(),
                    constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(16)),
                    tag: 0,
                    fields: Vec::new(),
                }],
            ),
            PreludeType::DivError => (
                "DivError".to_owned(),
                vec![
                    CheckedVariant {
                        name: "DivideByZero".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(18)),
                        tag: 0,
                        fields: Vec::new(),
                    },
                    CheckedVariant {
                        name: "DivOverflow".to_owned(),
                        constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(19)),
                        tag: 1,
                        fields: Vec::new(),
                    },
                ],
            ),
            PreludeType::NarrowError => (
                "NarrowError".to_owned(),
                vec![CheckedVariant {
                    name: "NarrowError".to_owned(),
                    constructor: CheckedConstructor::Prelude(PreludeDeclarationId::new(21)),
                    tag: 0,
                    fields: Vec::new(),
                }],
            ),
        };
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: CheckedNominalKind::Enum { variants },
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(Some(ty));
        if self.prelude_nominals.insert(ty, id).is_some() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }
}
