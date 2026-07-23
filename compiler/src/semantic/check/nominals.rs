use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::{
    DeclarationRole, DependentDeclarationRole, PreludeDeclarationId, ProductionV0_14,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRuleV0_14, UnsupportedSemanticFeatureV0_14,
};

use super::super::model::{
    CheckedConstructor, CheckedField, CheckedNominal, CheckedNominalKind, CheckedType,
    CheckedVariant, NominalId,
};
use super::{CheckStop, Checker, Constructor, PreludeType};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn declare_nominals(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree.production(*node).is_ok_and(|production| {
                matches!(
                    production,
                    ProductionV0_14::StructDecl | ProductionV0_14::EnumDecl
                )
            })
        }) {
            if let Some(generics) = self
                .tree
                .first_child_with(node, ProductionV0_14::Generics)?
            {
                return self.unsupported(UnsupportedSemanticFeatureV0_14::Generics, generics);
            }
            let role = match self.tree.production(node)? {
                ProductionV0_14::StructDecl => DeclarationRole::Struct,
                ProductionV0_14::EnumDecl => DeclarationRole::Enum,
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            };
            let declaration = self.declaration_at(node, role)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let id = NominalId(
                u32::try_from(self.nominals.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            if self
                .nominals_by_declaration
                .insert(declaration_id, id)
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            if role == DeclarationRole::Struct
                && self
                    .constructors_by_declaration
                    .insert(declaration_id, Constructor::Struct(id))
                    .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            self.nominal_nodes.push(Some(node));
            self.prelude_types.push(None);
            self.nominals.push(CheckedNominal {
                id,
                name,
                kind: match role {
                    DeclarationRole::Struct => CheckedNominalKind::Struct { fields: Vec::new() },
                    DeclarationRole::Enum => CheckedNominalKind::Enum {
                        variants: Vec::new(),
                    },
                    _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
                },
            });
        }
        Ok(())
    }

    pub(super) fn complete_nominals(&mut self) -> Result<(), CheckStop> {
        let source_nominal_count = self.nominals.len();
        self.register_prelude_nominals()?;

        for id in 0..source_nominal_count {
            let node = *self
                .nominal_nodes
                .get(id)
                .and_then(Option::as_ref)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let kind = match self.tree.production(node)? {
                ProductionV0_14::StructDecl => CheckedNominalKind::Struct {
                    fields: self.parse_struct_fields(node)?,
                },
                ProductionV0_14::EnumDecl => CheckedNominalKind::Enum {
                    variants: self.parse_enum_variants(NominalId(id as u32), node)?,
                },
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            };
            self.nominals
                .get_mut(id)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .kind = kind;
        }
        self.reject_recursive_nominal_layouts()
    }

    fn parse_struct_fields(&self, node: NodeId) -> Result<Vec<CheckedField>, CheckStop> {
        let nodes = self.tree.children_with(node, ProductionV0_14::Field)?;
        let mut seen = HashSet::with_capacity(nodes.len());
        let mut fields = Vec::with_capacity(nodes.len());
        for field in nodes {
            let declaration =
                self.dependent_declaration_at(field, DependentDeclarationRole::Field)?;
            let name = declaration.spelling().to_owned();
            if !seen.insert(name.clone()) {
                return self.issue_node(
                    SemanticRuleV0_14::Type6,
                    field,
                    SemanticIssueKind::DuplicateFieldLabel { label: name },
                );
            }
            let ty = self
                .tree
                .first_child_with(field, ProductionV0_14::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            fields.push(CheckedField {
                name,
                ty: self.parse_type(ty)?,
            });
        }
        Ok(fields)
    }

    fn parse_enum_variants(
        &mut self,
        nominal: NominalId,
        node: NodeId,
    ) -> Result<Vec<CheckedVariant>, CheckStop> {
        let nodes = self.tree.children_with(node, ProductionV0_14::Variant)?;
        let mut variants = Vec::with_capacity(nodes.len());
        for variant_node in nodes {
            let declaration = self.declaration_at(variant_node, DeclarationRole::Variant)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let tag = u32::try_from(variants.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            if self
                .constructors_by_declaration
                .insert(
                    declaration_id,
                    Constructor::Enum {
                        nominal,
                        variant: tag,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            let mut fields = Vec::new();
            let mut seen = HashSet::new();
            if let Some(list) = self
                .tree
                .first_child_with(variant_node, ProductionV0_14::VfieldList)?
            {
                for field in self.tree.children_with(list, ProductionV0_14::Vfield)? {
                    let declaration = self
                        .dependent_declaration_at(field, DependentDeclarationRole::VariantField)?;
                    let field_name = declaration.spelling().to_owned();
                    if !seen.insert(field_name.clone()) {
                        return self.issue_node(
                            SemanticRuleV0_14::Type6,
                            field,
                            SemanticIssueKind::DuplicateFieldLabel { label: field_name },
                        );
                    }
                    let ty = self
                        .tree
                        .first_child_with(field, ProductionV0_14::Type)?
                        .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                    fields.push(CheckedField {
                        name: field_name,
                        ty: self.parse_type(ty)?,
                    });
                }
            }
            variants.push(CheckedVariant {
                name,
                constructor: CheckedConstructor::Source(declaration_id),
                tag,
                fields,
            });
        }
        Ok(variants)
    }

    fn reject_recursive_nominal_layouts(&self) -> Result<(), CheckStop> {
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
                        return self.unsupported(
                            UnsupportedSemanticFeatureV0_14::RecursiveNominalLayout,
                            node,
                        );
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
            CheckedType::Unit | CheckedType::Bool | CheckedType::Integer(_) => true,
            CheckedType::Array { .. } => false,
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

    fn register_prelude_nominals(&mut self) -> Result<(), CheckStop> {
        self.intern_prelude_nominal(PreludeType::Overflow)?;
        self.intern_prelude_nominal(PreludeType::DivError)?;
        self.intern_prelude_nominal(PreludeType::NarrowError)?;

        let mut type_nodes = self
            .tree
            .topology()
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                (record.production == ProductionV0_14::Type)
                    .then(|| NodeId::from_index(index).map(|node| (record.tree_depth, node)))
                    .flatten()
            })
            .collect::<Vec<_>>();
        type_nodes.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then(left.1.index().cmp(&right.1.index()))
        });
        for (_, node) in type_nodes {
            if self.prelude_type_ordinal(node)? == Some(8) {
                let (ok, error) = self.result_type_arguments(node)?;
                self.intern_prelude_nominal(PreludeType::Result(ok, error))?;
            }
        }

        let propagate_lets = self
            .tree
            .topology()
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                (record.production == ProductionV0_14::LetStmt)
                    .then(|| NodeId::from_index(index))
                    .flatten()
            })
            .collect::<Vec<_>>();
        for node in propagate_lets {
            if self
                .tree
                .first_child_with(node, ProductionV0_14::PropagateLetRhs)?
                .is_none()
            {
                continue;
            }
            let ok_node = self
                .tree
                .first_child_with(node, ProductionV0_14::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let ok = self.parse_type(ok_node)?;
            let function = self.enclosing_function(node)?;
            let rtype = self
                .tree
                .first_child_with(function, ProductionV0_14::Rtype)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let return_type = self
                .tree
                .first_child_with(rtype, ProductionV0_14::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let return_type = self.parse_type(return_type)?;
            if let CheckedType::Nominal(return_nominal) = return_type
                && let Some(PreludeType::Result(_, error)) = self.prelude_type(return_nominal)
            {
                self.intern_prelude_nominal(PreludeType::Result(ok, error))?;
            }
        }

        let call_nodes = self
            .tree
            .topology()
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                (record.production == ProductionV0_14::Call)
                    .then(|| NodeId::from_index(index))
                    .flatten()
            })
            .collect::<Vec<_>>();
        for node in call_nodes {
            let Some(callee) = self.tree.first_child_with(node, ProductionV0_14::Callee)? else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let spelling = self.tree.direct_spelling(callee)?;
            if !matches!(
                spelling.as_slice(),
                b"iadd.checked"
                    | b"isub.checked"
                    | b"imul.checked"
                    | b"idiv.checked"
                    | b"irem.checked"
                    | b"iabs.checked"
                    | b"ineg.checked"
            ) {
                continue;
            }
            let Some(targs) = self.tree.first_child_with(node, ProductionV0_14::Targs)? else {
                continue;
            };
            let arguments = self.tree.children_with(targs, ProductionV0_14::Targ)?;
            let [argument] = arguments.as_slice() else {
                continue;
            };
            let Some(ty_node) = self
                .tree
                .first_child_with(*argument, ProductionV0_14::Type)?
            else {
                continue;
            };
            let Some(integer) = self.integer_type(ty_node)? else {
                continue;
            };
            let error = if matches!(spelling.as_slice(), b"idiv.checked" | b"irem.checked") {
                PreludeType::DivError
            } else {
                PreludeType::Overflow
            };
            self.intern_prelude_nominal(PreludeType::Result(
                CheckedType::Integer(integer),
                CheckedType::Nominal(self.prelude_nominal(error)?),
            ))?;
        }
        Ok(())
    }

    fn enclosing_function(&self, mut node: NodeId) -> Result<NodeId, CheckStop> {
        loop {
            let record = self
                .tree
                .topology()
                .node(node)
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let Some(parent) = record.parent else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            if self.tree.production(parent)? == ProductionV0_14::FnDecl {
                return Ok(parent);
            }
            node = parent;
        }
    }

    fn intern_prelude_nominal(&mut self, ty: PreludeType) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.prelude_nominals.get(&ty) {
            return Ok(*id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let (name, variants) = match ty {
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
        self.prelude_types.push(Some(ty));
        if self.prelude_nominals.insert(ty, id).is_some() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }
}
