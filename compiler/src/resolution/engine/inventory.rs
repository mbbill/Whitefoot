use crate::ProductionV0_14;
use crate::syntax::{FinalizedExtent, FinalizedTopology};

use super::super::catalog::{PRELUDE_DECLARATIONS, reserved_name};
use super::super::scopes::ScopeBuild;
use super::super::{
    DeclarationClass, DeclarationConflict, DeclarationOrigin, DeclarationRecord, DeclarationRole,
    DeferredUseRole, DependentDeclarationRole, ReservedDeclarationRole, ResolutionCompilerFailure,
    ResolutionIssue, ResolutionIssueKind, ResolutionRuleV0_14,
};
use super::{
    ClassifiedRole, DeclarationIndex, DeclarationMeta, EventKey, RawRoleKind,
    ancestor_with_production, declaration_domain, is_visible,
};

struct InventoryTables<'a> {
    roles: &'a [ClassifiedRole],
    declarations: &'a [DeclarationRecord],
    metas: &'a [DeclarationMeta],
    index: &'a DeclarationIndex,
}

pub(super) fn check_declaration_inventory(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    roles: &[ClassifiedRole],
    declarations: &[DeclarationRecord],
    metas: &[DeclarationMeta],
    index: &DeclarationIndex,
    declaration_by_role: &[Option<usize>],
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    if declarations.len() != metas.len()
        || metas
            .iter()
            .enumerate()
            .any(|(index, meta)| meta.record_index != index)
    {
        return Err(ResolutionCompilerFailure::InvalidRoleShape);
    }
    let tables = InventoryTables {
        roles,
        declarations,
        metas,
        index,
    };
    for (role_index, role) in roles.iter().enumerate() {
        if let Some((reserved_role, checked_spelling)) = reserved_role(role)
            && let Some((class, inventory_ordinal)) = reserved_name(checked_spelling)
        {
            return Ok(Some(ResolutionIssue {
                rule: ResolutionRuleV0_14::Form3,
                origin: role.origin.clone(),
                kind: ResolutionIssueKind::ReservedName {
                    spelling: checked_spelling.to_owned(),
                    declaration_role: reserved_role,
                    class,
                    inventory_ordinal,
                },
            }));
        }
        let Some(record_index) = declaration_by_role.get(role_index).and_then(|entry| *entry)
        else {
            continue;
        };
        let meta = meta_for_record(metas, record_index)?;
        let declaration = declarations
            .get(record_index)
            .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;

        if matches!(
            declaration.role,
            DeclarationRole::RegionParameter | DeclarationRole::LocalRegion
        ) && let Some(conflicting) = index
            .with_spelling(&declaration.spelling)
            .iter()
            .copied()
            .take_while(|candidate| *candidate < meta.record_index)
            .filter_map(|candidate| metas.get(candidate))
            .find(|candidate| {
                candidate.region_owner == meta.region_owner
                    && matches!(
                        declarations[candidate.record_index].role,
                        DeclarationRole::RegionParameter | DeclarationRole::LocalRegion
                    )
            })
        {
            return Ok(Some(ResolutionIssue {
                rule: ResolutionRuleV0_14::Own3,
                origin: declaration.origin.clone(),
                kind: ResolutionIssueKind::RepeatedRegion {
                    spelling: declaration.spelling.clone(),
                    conflicting: declarations[conflicting.record_index].origin.clone(),
                },
            }));
        }

        if declaration.role == DeclarationRole::MatchBinder
            && let Some(issue) = match_binder_issue(topology, scopes, role, declaration, &tables)?
        {
            return Ok(Some(issue));
        }

        if let Some(issue) = collision_issue(scopes, declaration, meta, &tables)? {
            return Ok(Some(issue));
        }
    }
    Ok(None)
}

fn reserved_role(role: &ClassifiedRole) -> Option<(ReservedDeclarationRole, &str)> {
    let mapped = match role.kind {
        RawRoleKind::Declaration(DeclarationRole::Function) => ReservedDeclarationRole::Function,
        RawRoleKind::Declaration(DeclarationRole::NamedConst) => {
            ReservedDeclarationRole::NamedConst
        }
        RawRoleKind::Declaration(DeclarationRole::Parameter) => ReservedDeclarationRole::Parameter,
        RawRoleKind::Declaration(DeclarationRole::Let) => ReservedDeclarationRole::Let,
        RawRoleKind::Declaration(DeclarationRole::MatchBinder) => {
            ReservedDeclarationRole::MatchBinder
        }
        RawRoleKind::DependentDeclaration(DependentDeclarationRole::Field) => {
            ReservedDeclarationRole::Field
        }
        RawRoleKind::DependentDeclaration(DependentDeclarationRole::VariantField) => {
            ReservedDeclarationRole::VariantField
        }
        RawRoleKind::Declaration(DeclarationRole::RegionParameter) => {
            ReservedDeclarationRole::RegionParameter
        }
        RawRoleKind::Declaration(DeclarationRole::LocalRegion) => {
            ReservedDeclarationRole::LocalRegion
        }
        _ => return None,
    };
    let spelling = if matches!(
        mapped,
        ReservedDeclarationRole::RegionParameter | ReservedDeclarationRole::LocalRegion
    ) {
        role.spelling.strip_prefix('\'')?
    } else {
        &role.spelling
    };
    Some((mapped, spelling))
}

fn match_binder_issue(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    role: &ClassifiedRole,
    declaration: &DeclarationRecord,
    tables: &InventoryTables<'_>,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let arm = ancestor_with_production(topology, role.owner, ProductionV0_14::Arm)
        .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
    let paired_field = tables
        .roles
        .iter()
        .find(|candidate| {
            candidate.owner == role.owner
                && matches!(
                    candidate.kind,
                    RawRoleKind::DeferredUse(DeferredUseRole::MatchField)
                )
        })
        .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
    let earlier_binder = tables
        .index
        .with_spelling(&declaration.spelling)
        .iter()
        .copied()
        .take_while(|candidate| *candidate < declaration.id.index())
        .filter_map(|candidate| tables.metas.get(candidate))
        .find_map(|candidate| {
            let earlier = tables.declarations.get(candidate.record_index)?;
            (earlier.role == DeclarationRole::MatchBinder
                && earlier.spelling == declaration.spelling
                && ancestor_with_production(
                    topology,
                    tables.roles[candidate.role_index].owner,
                    ProductionV0_14::Arm,
                ) == Some(arm))
            .then(|| earlier.origin.clone())
        });
    let arm_scope = scopes.declaration_scope(arm)?;
    let entry_scope = scopes
        .records
        .get(arm_scope.index())
        .and_then(super::super::ScopeRecord::parent)
        .ok_or(ResolutionCompilerFailure::InvalidScopeTree)?;
    let arm_record = topology
        .node(arm)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    let FinalizedExtent::Source { source, start, .. } = arm_record.extent else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    let mut arm_entry_conflicts = Vec::new();
    for candidate in tables
        .index
        .with_spelling(&declaration.spelling)
        .iter()
        .filter_map(|candidate| tables.metas.get(*candidate))
    {
        let record = &tables.declarations[candidate.record_index];
        if candidate.entries.iter().any(|class| {
            matches!(
                class,
                DeclarationClass::Function
                    | DeclarationClass::NamedConst
                    | DeclarationClass::ConstGeneric
                    | DeclarationClass::Value
            )
        }) && is_visible(
            scopes,
            candidate,
            entry_scope,
            source.ordinal(),
            start.value(),
        ) {
            arm_entry_conflicts.push(record.origin.clone());
        }
    }
    arm_entry_conflicts.sort_by_key(EventKey::from_origin);
    if declaration.spelling == paired_field.spelling
        || earlier_binder.is_some()
        || !arm_entry_conflicts.is_empty()
    {
        return Ok(Some(ResolutionIssue {
            rule: ResolutionRuleV0_14::Gram10,
            origin: declaration.origin.clone(),
            kind: ResolutionIssueKind::MatchBinderFreshness {
                spelling: declaration.spelling.clone(),
                paired_field: paired_field.spelling.clone(),
                earlier_binder,
                arm_entry_conflicts,
            },
        }));
    }
    Ok(None)
}

fn collision_issue(
    scopes: &ScopeBuild,
    declaration: &DeclarationRecord,
    meta: &DeclarationMeta,
    tables: &InventoryTables<'_>,
) -> Result<Option<ResolutionIssue>, ResolutionCompilerFailure> {
    let mut prelude_conflicts = Vec::new();
    for class in &meta.entries {
        let domain =
            declaration_domain(*class).ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
        for prelude in PRELUDE_DECLARATIONS {
            if prelude.spelling == declaration.spelling
                && prelude.class.and_then(declaration_domain) == Some(domain)
            {
                prelude_conflicts.push(DeclarationConflict {
                    domain,
                    class: prelude
                        .class
                        .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?,
                    origin: DeclarationOrigin::Prelude(prelude.id),
                });
            }
        }
    }
    sort_conflicts(&mut prelude_conflicts, tables.declarations);
    if !prelude_conflicts.is_empty() {
        return Ok(Some(collision(
            declaration,
            prelude_conflicts,
            ResolutionRuleV0_14::Type6,
        )));
    }

    let mut same_scope = Vec::new();
    for candidate in tables
        .index
        .with_spelling(&declaration.spelling)
        .iter()
        .copied()
        .take_while(|candidate| *candidate < meta.record_index)
        .filter_map(|candidate| tables.metas.get(candidate))
        .filter(|candidate| candidate.scope == meta.scope)
    {
        collect_domain_conflicts(
            declaration,
            meta,
            &tables.declarations[candidate.record_index],
            candidate,
            &mut same_scope,
        );
    }
    sort_conflicts(&mut same_scope, tables.declarations);
    if !same_scope.is_empty() {
        return Ok(Some(collision(
            declaration,
            same_scope,
            ResolutionRuleV0_14::Type6,
        )));
    }

    let mut shadows = Vec::new();
    for candidate in tables
        .index
        .with_spelling(&declaration.spelling)
        .iter()
        .filter_map(|candidate| tables.metas.get(*candidate))
    {
        if candidate.record_index == meta.record_index
            || candidate.scope == meta.scope
            || !scopes.is_ancestor(candidate.scope, meta.scope)
            || !is_visible(
                scopes,
                candidate,
                meta.scope,
                declaration.origin.coordinate.source().ordinal(),
                declaration.origin.coordinate.start().value(),
            )
        {
            continue;
        }
        collect_domain_conflicts(
            declaration,
            meta,
            &tables.declarations[candidate.record_index],
            candidate,
            &mut shadows,
        );
    }
    sort_conflicts(&mut shadows, tables.declarations);
    Ok((!shadows.is_empty()).then(|| collision(declaration, shadows, ResolutionRuleV0_14::Type6)))
}

fn meta_for_record(
    metas: &[DeclarationMeta],
    record_index: usize,
) -> Result<&DeclarationMeta, ResolutionCompilerFailure> {
    metas
        .get(record_index)
        .filter(|meta| meta.record_index == record_index)
        .ok_or(ResolutionCompilerFailure::InvalidRoleShape)
}

fn collect_domain_conflicts(
    declaration: &DeclarationRecord,
    meta: &DeclarationMeta,
    candidate: &DeclarationRecord,
    candidate_meta: &DeclarationMeta,
    conflicts: &mut Vec<DeclarationConflict>,
) {
    if candidate.spelling != declaration.spelling {
        return;
    }
    for class in &meta.entries {
        let Some(domain) = declaration_domain(*class) else {
            continue;
        };
        for candidate_class in &candidate_meta.entries {
            if declaration_domain(*candidate_class) == Some(domain) {
                conflicts.push(DeclarationConflict {
                    domain,
                    class: *candidate_class,
                    origin: DeclarationOrigin::Source(candidate.origin.clone()),
                });
            }
        }
    }
}

fn collision(
    declaration: &DeclarationRecord,
    conflicts: Vec<DeclarationConflict>,
    rule: ResolutionRuleV0_14,
) -> ResolutionIssue {
    ResolutionIssue {
        rule,
        origin: declaration.origin.clone(),
        kind: ResolutionIssueKind::DeclarationCollision {
            spelling: declaration.spelling.clone(),
            conflicts,
        },
    }
}

fn sort_conflicts(conflicts: &mut [DeclarationConflict], declarations: &[DeclarationRecord]) {
    conflicts.sort_by(|left, right| {
        let left_domain = left.domain.ordinal();
        let right_domain = right.domain.ordinal();
        left_domain.cmp(&right_domain).then_with(|| {
            conflict_key(&left.origin, declarations).cmp(&conflict_key(&right.origin, declarations))
        })
    });
}

pub(super) fn conflict_key(
    origin: &DeclarationOrigin,
    declarations: &[DeclarationRecord],
) -> (u8, EventKey) {
    match origin {
        DeclarationOrigin::Prelude(id) => (
            0,
            EventKey {
                source: 0,
                start: u64::from(id.ordinal()),
                end: 0,
                path: Vec::new(),
                role: 0,
                subtoken: 0,
            },
        ),
        DeclarationOrigin::Source(origin) => {
            let _ = declarations;
            (1, EventKey::from_origin(origin))
        }
    }
}
