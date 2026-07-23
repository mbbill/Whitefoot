use crate::syntax::NodeId;
use crate::{
    PreludeDeclarationId, Production, SemanticCompilerFailure, UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedConstructor, CheckedField, CheckedNominal, CheckedNominalKind, CheckedType,
    CheckedVariant, NominalId,
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
        };
        Ok(fields
            .into_iter()
            .filter_map(|field| match field.ty {
                CheckedType::Nominal(id) => Some(id),
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
            | CheckedType::GenericInt(_) => true,
            CheckedType::Generic(_) | CheckedType::Array { .. } | CheckedType::Buffer { .. } => {
                false
            }
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

    pub(super) fn register_prelude_nominals(&mut self) -> Result<(), CheckStop> {
        self.intern_prelude_nominal(PreludeType::Overflow)?;
        self.intern_prelude_nominal(PreludeType::DivError)?;
        self.intern_prelude_nominal(PreludeType::NarrowError)?;
        Ok(())
    }

    pub(super) fn enclosing_function(&self, mut node: NodeId) -> Result<NodeId, CheckStop> {
        loop {
            let record = self
                .tree
                .topology()
                .node(node)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let Some(parent) = record.parent else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            if self.tree.production(parent)? == Production::FnDecl {
                return Ok(parent);
            }
            node = parent;
        }
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
