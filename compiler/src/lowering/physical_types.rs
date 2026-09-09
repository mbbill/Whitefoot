//! Contextual storage types for already accepted region-polymorphic functions.
//!
//! Nominal identity is the checked source family plus its complete reclamation
//! graph. Equal layouts alone never merge types. Pair visitation makes cyclic
//! graphs finite without a depth limit or an acceptance heuristic.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::semantic::{CheckedNominalKind, CheckedProgramData, CheckedReleaseClass};
use crate::{DeclarationId, NominalId};

use super::*;

type Releases = Vec<(DeclarationId, CheckedReleaseClass)>;

pub(super) struct PhysicalTypeMap {
    pub(super) nominals: Vec<IrNominalId>,
    pub(super) elements: Vec<Option<IrElement>>,
}

/// Lower only elements reached by executable types. Generic proof replay can
/// leave symbolic entries in the checked table; they have no physical type.
/// Structural children are interned before parents, so ascending handle order
/// is a topological order even when nominal references close an owning graph.
pub(super) fn base_elements(
    data: &CheckedProgramData,
    nominals: &[IrNominalId],
) -> Result<(Vec<IrType>, Vec<Option<IrElement>>), LoweringFailure> {
    let mut pending = data
        .functions
        .iter()
        .flat_map(specialize::executable_types)
        .collect::<Vec<_>>();
    pending.extend(data.constants.iter().map(|constant| constant.ty));
    for nominal in data.nominals.iter().take(data.executable_nominal_count) {
        match &nominal.kind {
            CheckedNominalKind::Struct { fields } => {
                pending.extend(fields.iter().map(|field| field.ty))
            }
            CheckedNominalKind::Enum { variants } => pending.extend(
                variants
                    .iter()
                    .flat_map(|variant| &variant.fields)
                    .map(|field| field.ty),
            ),
            CheckedNominalKind::Box { referent, .. } => pending.push(*referent),
            CheckedNominalKind::Arena { content, .. } => pending.push(*content),
            CheckedNominalKind::ArenaStorage | CheckedNominalKind::SystemResource { .. } => {}
        }
    }
    let mut needed = BTreeSet::new();
    while let Some(ty) = pending.pop() {
        if let CheckedType::FixedVector { element, .. } | CheckedType::Vector { element, .. } = ty
            && needed.insert(element.index())
        {
            pending.push(
                *data
                    .elements
                    .get(element.index())
                    .ok_or(LoweringFailure::InvalidCheckedProgram)?,
            );
        }
    }
    let mut map = vec![None; data.elements.len()];
    let mut elements = Vec::new();
    let mut interned = HashMap::new();
    for index in needed {
        let ty = lower_type(
            TypeLowering {
                nominals,
                elements: &map,
                releases: &[],
            },
            data.elements[index],
        )?;
        map[index] = Some(intern_element(&mut elements, &mut interned, ty)?);
    }
    Ok((elements, map))
}

fn intern_element(
    elements: &mut Vec<IrType>,
    interned: &mut HashMap<IrType, IrElement>,
    ty: IrType,
) -> Result<IrElement, LoweringFailure> {
    if let Some(id) = interned.get(&ty) {
        return Ok(*id);
    }
    let id =
        IrElement(u32::try_from(elements.len()).map_err(|_| LoweringFailure::CounterOverflow)?);
    elements.push(ty);
    interned.insert(ty, id);
    Ok(id)
}

struct Instance {
    source: NominalId,
    releases: Releases,
    id: IrNominalId,
}

pub(super) struct PhysicalTypes<'a> {
    data: &'a CheckedProgramData,
    pub(super) nominals: Vec<IrNominal>,
    pub(super) elements: Vec<IrType>,
    base_elements: Vec<Option<IrElement>>,
    interned_elements: HashMap<IrType, IrElement>,
    element_instances: HashMap<(CheckedElement, Releases), IrElement>,
    instances: Vec<Instance>,
}

impl<'a> PhysicalTypes<'a> {
    pub(super) fn new(
        data: &'a CheckedProgramData,
        nominals: Vec<IrNominal>,
        elements: Vec<IrType>,
        base_elements: Vec<Option<IrElement>>,
    ) -> Self {
        let instances = data
            .nominal_lowering_alias
            .iter()
            .enumerate()
            .take(data.executable_nominal_count)
            .filter(|(index, alias)| *index == alias.0 as usize)
            .map(|(_, alias)| Instance {
                source: *alias,
                releases: Vec::new(),
                id: IrNominalId(alias.0),
            })
            .collect();
        Self {
            data,
            nominals,
            interned_elements: elements
                .iter()
                .enumerate()
                .map(|(index, ty)| (*ty, IrElement(index as u32)))
                .collect(),
            elements,
            base_elements,
            element_instances: HashMap::new(),
            instances,
        }
    }

    pub(super) fn map(
        &mut self,
        releases: &[(DeclarationId, CheckedReleaseClass)],
    ) -> Result<PhysicalTypeMap, LoweringFailure> {
        let nominals = (0..self.data.executable_nominal_count)
            .map(|index| {
                self.nominal(
                    NominalId(u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?),
                    releases,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut elements = vec![None; self.data.elements.len()];
        for (index, mapped) in elements.iter_mut().enumerate() {
            if self.base_elements[index].is_some() {
                *mapped = Some(self.element(
                    CheckedElement(
                        u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?,
                    ),
                    releases,
                )?);
            }
        }
        Ok(PhysicalTypeMap { nominals, elements })
    }

    fn element(
        &mut self,
        source: CheckedElement,
        releases: &[(DeclarationId, CheckedReleaseClass)],
    ) -> Result<IrElement, LoweringFailure> {
        let key = (source, releases.to_vec());
        if let Some(id) = self.element_instances.get(&key) {
            return Ok(*id);
        }
        let ty = *self
            .data
            .elements
            .get(source.index())
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        let ty = self.ty(ty, releases)?;
        let id = intern_element(&mut self.elements, &mut self.interned_elements, ty)?;
        self.element_instances.insert(key, id);
        Ok(id)
    }

    fn family(&self, source: NominalId) -> Result<NominalId, LoweringFailure> {
        self.data
            .nominal_physical_alias
            .get(source.0 as usize)
            .copied()
            .ok_or(LoweringFailure::InvalidCheckedProgram)
    }

    fn nominal(
        &mut self,
        source: NominalId,
        releases: &[(DeclarationId, CheckedReleaseClass)],
    ) -> Result<IrNominalId, LoweringFailure> {
        let family = self.family(source)?;
        for candidate in &self.instances {
            if self.family(candidate.source)? == family
                && self.same_graph(source, releases, candidate.source, &candidate.releases)?
            {
                return Ok(candidate.id);
            }
        }
        let id = IrNominalId(
            u32::try_from(self.nominals.len()).map_err(|_| LoweringFailure::CounterOverflow)?,
        );
        let mut nominal = self
            .nominals
            .get(source.0 as usize)
            .cloned()
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        nominal.id = id;
        // Publish the identity before visiting children, so recursive fields
        // refer to this same instance. The placeholder is closed before use.
        self.nominals.push(nominal);
        self.instances.push(Instance {
            source,
            releases: releases.to_vec(),
            id,
        });
        let kind = self
            .data
            .nominals
            .get(source.0 as usize)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?
            .kind
            .clone();
        let lowered = match kind {
            CheckedNominalKind::Struct { fields } => IrNominalKind::Struct {
                fields: fields
                    .iter()
                    .map(|field| {
                        Ok(IrField {
                            ty: self.ty(field.ty, releases)?,
                        })
                    })
                    .collect::<Result<_, LoweringFailure>>()?,
            },
            CheckedNominalKind::Enum { variants } => IrNominalKind::Enum {
                variants: variants
                    .iter()
                    .map(|variant| {
                        Ok(IrVariant {
                            tag: variant.tag,
                            fields: variant
                                .fields
                                .iter()
                                .map(|field| {
                                    Ok(IrField {
                                        ty: self.ty(field.ty, releases)?,
                                    })
                                })
                                .collect::<Result<_, LoweringFailure>>()?,
                        })
                    })
                    .collect::<Result<_, LoweringFailure>>()?,
            },
            CheckedNominalKind::Box {
                referent,
                region,
                release,
            } => IrNominalKind::Box {
                referent: self.ty(referent, releases)?,
                release: lower_release_class(effective_release(releases, region, release)),
            },
            CheckedNominalKind::Arena { content, .. } => IrNominalKind::Arena {
                content: self.ty(content, releases)?,
            },
            CheckedNominalKind::ArenaStorage | CheckedNominalKind::SystemResource { .. } => {
                self.nominals[id.index()].kind.clone()
            }
        };
        self.nominals[id.index()].kind = lowered;
        Ok(id)
    }

    fn ty(
        &mut self,
        ty: CheckedType,
        releases: &[(DeclarationId, CheckedReleaseClass)],
    ) -> Result<IrType, LoweringFailure> {
        match ty {
            CheckedType::FixedVector { element, length } => {
                return Ok(IrType::FixedVector {
                    element: self.element(element, releases)?,
                    length: length
                        .value()
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                });
            }
            CheckedType::Vector {
                region,
                element,
                release,
            } => {
                return Ok(IrType::Vector {
                    element: self.element(element, releases)?,
                    release: lower_release_class(effective_release(
                        releases,
                        Some(region),
                        release,
                    )),
                });
            }
            _ => {}
        }
        let mut map = self
            .data
            .nominal_lowering_alias
            .iter()
            .map(|id| IrNominalId(id.0))
            .collect::<Vec<_>>();
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            match ty {
                CheckedType::Nominal(id) => {
                    map[id.0 as usize] = self.nominal(id, releases)?;
                }
                CheckedType::Array { element, .. }
                | CheckedType::Buffer { element }
                | CheckedType::Slice { element, .. } => pending.push(element.ty()),
                _ => {}
            }
        }
        lower_type(
            TypeLowering {
                nominals: &map,
                elements: &[],
                releases,
            },
            ty,
        )
    }

    fn same_graph(
        &self,
        left: NominalId,
        left_releases: &[(DeclarationId, CheckedReleaseClass)],
        right: NominalId,
        right_releases: &[(DeclarationId, CheckedReleaseClass)],
    ) -> Result<bool, LoweringFailure> {
        let mut pending = vec![(CheckedType::Nominal(left), CheckedType::Nominal(right))];
        let mut visited = HashSet::new();
        while let Some((left, right)) = pending.pop() {
            if !visited.insert((left, right)) {
                continue;
            }
            match (left, right) {
                (CheckedType::Nominal(left), CheckedType::Nominal(right)) => {
                    if self.family(left)? != self.family(right)? {
                        return Ok(false);
                    }
                    let left = &self
                        .data
                        .nominals
                        .get(left.0 as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .kind;
                    let right = &self
                        .data
                        .nominals
                        .get(right.0 as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .kind;
                    match (left, right) {
                        (
                            CheckedNominalKind::Box {
                                referent: lt,
                                region: lr,
                                release: lc,
                            },
                            CheckedNominalKind::Box {
                                referent: rt,
                                region: rr,
                                release: rc,
                            },
                        ) => {
                            if effective_release(left_releases, *lr, *lc)
                                != effective_release(right_releases, *rr, *rc)
                            {
                                return Ok(false);
                            }
                            pending.push((*lt, *rt));
                        }
                        (
                            CheckedNominalKind::Arena { content: left, .. },
                            CheckedNominalKind::Arena { content: right, .. },
                        ) => pending.push((*left, *right)),
                        (
                            CheckedNominalKind::Struct { fields: left },
                            CheckedNominalKind::Struct { fields: right },
                        ) => {
                            if left.len() != right.len() {
                                return Ok(false);
                            }
                            pending.extend(
                                left.iter()
                                    .zip(right)
                                    .map(|(left, right)| (left.ty, right.ty)),
                            );
                        }
                        (
                            CheckedNominalKind::Enum { variants: left },
                            CheckedNominalKind::Enum { variants: right },
                        ) => {
                            if left.len() != right.len() {
                                return Ok(false);
                            }
                            for (left, right) in left.iter().zip(right) {
                                if left.tag != right.tag || left.fields.len() != right.fields.len()
                                {
                                    return Ok(false);
                                }
                                pending.extend(
                                    left.fields
                                        .iter()
                                        .zip(&right.fields)
                                        .map(|(left, right)| (left.ty, right.ty)),
                                );
                            }
                        }
                        (CheckedNominalKind::ArenaStorage, CheckedNominalKind::ArenaStorage) => {}
                        (
                            CheckedNominalKind::SystemResource { nominal: left },
                            CheckedNominalKind::SystemResource { nominal: right },
                        ) if left == right => {}
                        _ => return Ok(false),
                    }
                }
                (
                    CheckedType::Vector {
                        region: lr,
                        element: le,
                        release: lc,
                    },
                    CheckedType::Vector {
                        region: rr,
                        element: re,
                        release: rc,
                    },
                ) => {
                    if effective_release(left_releases, Some(lr), lc)
                        != effective_release(right_releases, Some(rr), rc)
                    {
                        return Ok(false);
                    }
                    pending.push((
                        *self
                            .data
                            .elements
                            .get(le.index())
                            .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                        *self
                            .data
                            .elements
                            .get(re.index())
                            .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                    ));
                }
                (
                    CheckedType::FixedVector {
                        element: left,
                        length: ln,
                    },
                    CheckedType::FixedVector {
                        element: right,
                        length: rn,
                    },
                ) if ln == rn => pending.push((
                    *self
                        .data
                        .elements
                        .get(left.index())
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                    *self
                        .data
                        .elements
                        .get(right.index())
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?,
                )),
                (
                    CheckedType::Array {
                        element: left,
                        length: ln,
                    },
                    CheckedType::Array {
                        element: right,
                        length: rn,
                    },
                ) if ln == rn => pending.push((left.ty(), right.ty())),
                (CheckedType::Buffer { element: left }, CheckedType::Buffer { element: right })
                | (
                    CheckedType::Slice { element: left, .. },
                    CheckedType::Slice { element: right, .. },
                ) => pending.push((left.ty(), right.ty())),
                (CheckedType::Heap { .. }, CheckedType::Heap { .. }) => {}
                (
                    CheckedType::Extent {
                        bytes: lb,
                        align: la,
                        ..
                    },
                    CheckedType::Extent {
                        bytes: rb,
                        align: ra,
                        ..
                    },
                ) if lb == rb && la == ra => {}
                _ if left == right => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
}

fn effective_release(
    releases: &[(DeclarationId, CheckedReleaseClass)],
    region: Option<DeclarationId>,
    fallback: CheckedReleaseClass,
) -> CheckedReleaseClass {
    region.map_or(fallback, |region| {
        TypeLowering {
            nominals: &[],
            elements: &[],
            releases,
        }
        .release(region, fallback)
    })
}
