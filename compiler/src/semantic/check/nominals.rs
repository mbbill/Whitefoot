use std::collections::HashSet;

use crate::{BuiltinPreludeId, SemanticCompilerFailure, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedConst, CheckedConstructor, CheckedField, CheckedNominal, CheckedNominalKind,
    CheckedType, CheckedVariant, LoanStrength, NominalId,
};
use super::{CheckStop, Checker, PendingNominal, PreludeType};

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
            CheckedNominalKind::Box { .. }
            | CheckedNominalKind::Arena { .. }
            | CheckedNominalKind::ArenaStorage
            | CheckedNominalKind::Opaque => Vec::new(),
        };
        let mut pending: Vec<_> = fields.into_iter().map(|field| field.ty).collect();
        let mut visited = HashSet::new();
        let mut dependencies = Vec::new();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) {
                continue;
            }
            match ty {
                CheckedType::Nominal(id) => {
                    if !matches!(self.nominal(id)?.kind, CheckedNominalKind::Box { .. }) {
                        dependencies.push(id);
                    }
                }
                CheckedType::Array { element, length } if length != CheckedConst::Value(0) => {
                    pending.push(self.element_type(element)?);
                }
                CheckedType::FixedVector { element, .. } => {
                    pending.push(self.element_type(element)?);
                }
                // The target spells an empty array as [0 x i8], independently
                // of T. Fixed runs retain their payload's element layout even
                // at N0; descriptor-owned runs never embed their elements.
                _ => {}
            }
        }
        Ok(dependencies)
    }

    pub(super) fn nominal(&self, id: NominalId) -> Result<&CheckedNominal, CheckStop> {
        self.nominals
            .get(id.0 as usize)
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    /// The arena instance data behind a checked type, when it is one
    /// [STOR-1]: its region declaration and content type.
    pub(super) fn arena_instance(
        &self,
        ty: CheckedType,
    ) -> Result<Option<(crate::DeclarationId, CheckedType)>, CheckStop> {
        Ok(match ty {
            CheckedType::Nominal(id) => match self.nominal(id)?.kind {
                CheckedNominalKind::Arena { region, content } => Some((region, content)),
                _ => None,
            },
            _ => None,
        })
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
            // [S37] a type parameter standing for itself is copy exactly when
            // its written bound is `copy`: that bound is what admits the bare
            // use and the duplication its body writes [OWN-1, FN-2].
            CheckedType::Generic(declaration) => {
                self.generic_parameter_class(declaration)? == super::linearity::LinearityClass::Copy
            }
            // [VIEW-1, S27] the shared view is copy and the writable one is
            // affine. Affinity on the shared view buys no safety — a second
            // copy is a second *shared* loan, which [OWN-5] admits without
            // limit, and a loan-bearing value owns nothing [PROV-3], so it
            // has nothing to release twice. The exclusive view stays affine
            // because [OWN-5] refuses two exclusive loans on one range.
            CheckedType::Slice { strength, .. } => strength == LoanStrength::Shared,
            CheckedType::Array { .. }
            | CheckedType::Buffer { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Heap { .. }
            | CheckedType::Extent { .. } => false,
        })
    }

    /// Whether the complete type cannot occur at a proper accessible subplace.
    /// Follow type edges, including owning indirection, rather than expanding
    /// values or capacities. Recursive occurrences and unresolved generics do
    /// not establish uniqueness of the whole place.
    pub(super) fn has_no_same_typed_subplace(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        let mut pending = vec![ty];
        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            let children = match current {
                CheckedType::Nominal(id) => match &self.nominal(id)?.kind {
                    CheckedNominalKind::Struct { fields } => {
                        fields.iter().map(|field| field.ty).collect::<Vec<_>>()
                    }
                    CheckedNominalKind::Enum { variants } => variants
                        .iter()
                        .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                        .collect(),
                    CheckedNominalKind::Box { referent, .. } => vec![*referent],
                    CheckedNominalKind::Arena { content, .. } => vec![*content],
                    CheckedNominalKind::Opaque => Vec::new(),
                    CheckedNominalKind::ArenaStorage => return Ok(false),
                },
                CheckedType::Array { element, .. }
                | CheckedType::FixedVector { element, .. }
                | CheckedType::Vector { element, .. } => vec![self.element_type(element)?],
                CheckedType::Buffer { element } | CheckedType::Slice { element, .. } => {
                    vec![element.ty()]
                }
                CheckedType::Generic(_)
                | CheckedType::GenericInt(_)
                | CheckedType::GenericFloat(_) => return Ok(false),
                CheckedType::Unit
                | CheckedType::Bool
                | CheckedType::Integer(_)
                | CheckedType::Float(_)
                | CheckedType::Heap { .. }
                | CheckedType::Extent { .. } => Vec::new(),
            };
            for child in children {
                if child == ty {
                    return Ok(false);
                }
                pending.push(child);
            }
        }
        Ok(true)
    }

    pub(super) fn prelude_type(&self, id: NominalId) -> Option<PreludeType> {
        self.prelude_types.get(id.0 as usize).copied().flatten()
    }

    /// The interned prelude instance, deferring when a derived type named one
    /// that is not interned yet.
    ///
    /// The checked arithmetic rows produce `Result<T, Overflow>` and
    /// `Result<T, DivError>` for a *derived* `T`, and after the annotation is
    /// deleted nothing writes that type, so the miss is recoverable rather
    /// than an error: the driver interns it and checks the function again.
    pub(super) fn prelude_nominal(&self, ty: PreludeType) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.prelude_nominals.get(&ty) {
            return Ok(*id);
        }
        self.pending_nominals
            .borrow_mut()
            .push(PendingNominal::Prelude(ty));
        Err(CheckStop::DeferredNominal)
    }

    /// S39 the `Box<'s, T>` nominal for one (store region, referent), or
    /// the deferral that interns it.
    pub(super) fn store_box_nominal(
        &self,
        region: crate::DeclarationId,
        referent: CheckedType,
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.store_box_nominals.get(&(region, referent)) {
            return Ok(*id);
        }
        self.pending_nominals
            .borrow_mut()
            .push(PendingNominal::StoreBox(region, referent));
        Err(CheckStop::DeferredNominal)
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
            kind: CheckedNominalKind::Box {
                referent,
                region: None,
                release: super::super::model::CheckedReleaseClass::General,
            },
            linear: false,
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

    /// S39 one `Box<'s, T>` instance, interned per (store region, referent).
    ///
    /// The region is a component of the type's name exactly as a run's is
    /// [PROV-1], so two stores give two nominals, and the release class the
    /// walk selects is read off that region alone [PROV-6].
    pub(super) fn intern_store_box_nominal(
        &mut self,
        region: crate::DeclarationId,
        referent: CheckedType,
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.store_box_nominals.get(&(region, referent)) {
            return Ok(*id);
        }
        let release = self.vector_release_class(region)?;
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let name = format!("Box<{}>", self.checked_type_name(referent)?);
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: CheckedNominalKind::Box {
                referent,
                region: Some(region),
                release,
            },
            linear: false,
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        if self
            .store_box_nominals
            .insert((region, referent), id)
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }

    /// The ordered `(binder spelling, type)` rows of a `fn_decl`'s result
    /// list, or `None` when an ordinal's mode is one this compiler cannot
    /// carry in a result list yet.
    ///
    /// The mode judgment itself belongs to `build_function_signature`, which
    /// reports it at the offending `rtype`; this reader is the shared walk
    /// both the nominal pre-scan and the signature build take over the same
    /// ordinals [GRAM-2, CALL-4].
    pub(super) fn result_list_fields(
        &self,
        function: crate::syntax::NodeId,
        substitution: &super::generics::GenericSubstitution,
    ) -> Result<Option<Vec<(String, CheckedType)>>, CheckStop> {
        let mut rows = Vec::new();
        for binding in self
            .tree
            .children_with(function, crate::Production::ResultBinding)?
        {
            let [name] = self.tree.direct_identifiers(binding)?[..] else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let name = std::str::from_utf8(self.tree.token_bytes(name)?)
                .map(str::to_owned)
                .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?;
            let rtype = self
                .tree
                .first_child_with(binding, crate::Production::Rtype)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let (mode, ty) = self.parse_rtype_with(rtype, substitution)?;
            if mode != super::super::model::CheckedMode::Own {
                return Ok(None);
            }
            rows.push((name, ty));
        }
        Ok(Some(rows))
    }

    /// The compiler-owned nominal a `fn_decl`'s ordered result list denotes
    /// [GRAM-2, CALL-4].
    ///
    /// A declaration that writes two or more results hands its caller one
    /// value carrying them in written order, and every result ordinal is one
    /// field of it. Nothing else in the language changes: the callable
    /// boundary keeps exactly one result type, the ordinary transfer,
    /// ownership, and lowering rules apply to it, and a destructuring binder
    /// list is the projection that names the ordinals again at the caller.
    /// The name is spelled with parentheses so no source TYPEID can collide
    /// with it; nothing reads it but a diagnostic.
    /// The already-interned result-list nominal of one ordered result list,
    /// for the `&self` checking path that cannot intern one itself.
    pub(super) fn result_list_nominal(
        &self,
        results: &[(String, CheckedType)],
    ) -> Option<NominalId> {
        self.result_list_nominals.get(results).copied()
    }

    pub(super) fn intern_result_list_nominal(
        &mut self,
        results: &[(String, CheckedType)],
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.result_list_nominals.get(results) {
            return Ok(*id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let mut rendered = Vec::with_capacity(results.len());
        let mut fields = Vec::with_capacity(results.len());
        for (name, ty) in results {
            rendered.push(format!("{name}: {}", self.checked_type_name(*ty)?));
            fields.push(CheckedField {
                name: name.clone(),
                ty: *ty,
            });
        }
        self.nominals.push(CheckedNominal {
            id,
            name: format!("({})", rendered.join(", ")),
            kind: CheckedNominalKind::Struct { fields },
            linear: false,
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        if self
            .result_list_nominals
            .insert(results.to_vec(), id)
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }

    pub(super) fn intern_arena_nominal(
        &mut self,
        region: crate::DeclarationId,
        content: CheckedType,
    ) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.arena_nominals.get(&(region, content)) {
            return Ok(*id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let region_spelling = self
            .resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.id() == region)
            .map(|declaration| declaration.spelling().to_owned())
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let name = format!(
            "arena<{region_spelling}, {}>",
            self.checked_type_name(content)?
        );
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: CheckedNominalKind::Arena { region, content },
            linear: false,
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        if self.arena_nominals.insert((region, content), id).is_some() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(id)
    }

    /// The one compiler-owned region allocation-list nominal [STOR-3],
    /// deferring interning to the `&mut self` driver on first use.
    pub(super) fn arena_storage_nominal_or_defer(&self) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.arena_storage_nominal {
            return Ok(id);
        }
        self.pending_nominals
            .borrow_mut()
            .push(PendingNominal::ArenaStorage);
        Err(CheckStop::DeferredNominal)
    }

    pub(super) fn intern_arena_storage_nominal(&mut self) -> Result<NominalId, CheckStop> {
        if let Some(id) = self.arena_storage_nominal {
            return Ok(id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        self.nominals.push(CheckedNominal {
            id,
            name: "arena-region-storage".to_owned(),
            kind: CheckedNominalKind::ArenaStorage,
            linear: false,
        });
        self.nominal_nodes.push(None);
        self.nominal_states.push(2);
        self.source_nominal_instances.push(None);
        self.prelude_types.push(None);
        self.arena_storage_nominal = Some(id);
        Ok(id)
    }

    pub(super) fn register_prelude_nominals(&mut self) -> Result<(), CheckStop> {
        self.intern_prelude_nominal(PreludeType::Overflow)?;
        self.intern_prelude_nominal(PreludeType::DivError)?;
        self.intern_prelude_nominal(PreludeType::NarrowError)?;
        Ok(())
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
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::NONE),
                        tag: 0,
                        fields: Vec::new(),
                    },
                    CheckedVariant {
                        name: "Some".to_owned(),
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::SOME),
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
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::OK),
                        tag: 0,
                        fields: vec![CheckedField {
                            name: "value".to_owned(),
                            ty: ok,
                        }],
                    },
                    CheckedVariant {
                        name: "Err".to_owned(),
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::ERR),
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
                    constructor: CheckedConstructor::Prelude(BuiltinPreludeId::OVERFLOW),
                    tag: 0,
                    fields: Vec::new(),
                }],
            ),
            PreludeType::DivError => (
                "DivError".to_owned(),
                vec![
                    CheckedVariant {
                        name: "DivideByZero".to_owned(),
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::DIVIDE_BY_ZERO),
                        tag: 0,
                        fields: Vec::new(),
                    },
                    CheckedVariant {
                        name: "DivOverflow".to_owned(),
                        constructor: CheckedConstructor::Prelude(BuiltinPreludeId::DIV_OVERFLOW),
                        tag: 1,
                        fields: Vec::new(),
                    },
                ],
            ),
            PreludeType::NarrowError => (
                "NarrowError".to_owned(),
                vec![CheckedVariant {
                    name: "NarrowError".to_owned(),
                    constructor: CheckedConstructor::Prelude(BuiltinPreludeId::NARROW_ERROR),
                    tag: 0,
                    fields: Vec::new(),
                }],
            ),
        };
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: CheckedNominalKind::Enum { variants },
            linear: false,
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
