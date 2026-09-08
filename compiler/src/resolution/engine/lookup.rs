use std::collections::HashSet;

use super::super::catalog::{PRELUDE_DECLARATIONS, operation_id};
use super::super::scopes::ScopeBuild;
use super::super::{
    DeclarationClass, DeclarationOrigin, DeclarationRecord, LexicalUseRecord, LexicalUseRole,
    ResolutionCompilerFailure, ResolutionIssue, ResolutionIssueKind, ResolutionRule,
    ResolvedTarget, ScopeId, SystemDeclarationRecord,
};
use super::inventory::conflict_key;
use super::{DeclarationIndex, DeclarationMeta, UseMeta, is_visible};

pub(super) fn resolve_uses_deferred(
    scopes: &ScopeBuild,
    declarations: &[DeclarationRecord],
    metas: &[DeclarationMeta],
    index: &DeclarationIndex,
    uses: &[UseMeta],
    system: &[SystemDeclarationRecord],
) -> Result<(Vec<LexicalUseRecord>, Option<ResolutionIssue>), ResolutionCompilerFailure> {
    let mut resolved = Vec::with_capacity(uses.len());
    for use_record in uses {
        let admissible = admissible_classes(use_record.role, &use_record.spelling);
        let universe = universe_classes(use_record.role);
        let mut candidates = Vec::new();
        let mut invisible = Vec::new();
        let mut available = HashSet::new();
        for meta in index
            .with_spelling(&use_record.spelling)
            .iter()
            .filter_map(|candidate| metas.get(*candidate))
        {
            let declaration = &declarations[meta.record_index];
            if meta.scope != ScopeId(0)
                && meta
                    .owner
                    .is_some_and(|owner| !use_record.owner_chain.contains(&owner))
            {
                continue;
            }
            for class in &meta.entries {
                if !universe.contains(class) {
                    continue;
                }
                let visible = is_visible(
                    scopes,
                    meta,
                    use_record.scope,
                    use_record.origin.coordinate.source().ordinal(),
                    use_record.origin.coordinate.start().value(),
                );
                if visible {
                    available.insert(*class);
                }
                if admissible.contains(class) {
                    if visible {
                        candidates.push(ResolvedTarget::Source {
                            declaration: declaration.id,
                            class: *class,
                        });
                    } else {
                        invisible.push(DeclarationOrigin::Source(declaration.origin.clone()));
                    }
                }
            }
        }
        for prelude in PRELUDE_DECLARATIONS {
            let Some(class) = prelude.class else {
                continue;
            };
            if prelude.spelling == use_record.spelling && universe.contains(&class) {
                available.insert(class);
                if admissible.contains(&class) {
                    candidates.push(ResolvedTarget::Prelude(prelude.id));
                }
            }
        }
        // The third admitted declaration source [SYS-1]: every system entry
        // is a compilation-root entry of its domain in every lexical use's
        // candidate universe [SYS-3]. TYPE-6 and [SYS-2] admit a system entry
        // only at a `type` TYPEID, a `construct` or `arm` TYPEID, and a
        // `callee` IDENT — never at a `fn_bind` right IDENT, which admits
        // only a top-level source function; there the visible system entry
        // still contributes its class to the available set.
        for record in system {
            let Some(class) = record.lookup_class() else {
                continue;
            };
            if record.spelling() == use_record.spelling && universe.contains(&class) {
                available.insert(class);
                if admissible.contains(&class) && system_admissible(use_record.role) {
                    candidates.push(ResolvedTarget::System(record.id()));
                }
            }
        }
        // The fourth admitted declaration source [BLK-0], plus the [TYPE-2]
        // container and provider nominals. Both enter every unit on [SYS-3]'s
        // terms, and both are admitted at exactly the roles a system entry
        // is: a `type` TYPEID for a nominal and a `callee` IDENT for an
        // operation.
        for (ordinal, nominal) in crate::CONTAINER_NOMINALS.iter().enumerate() {
            if nominal.spelling != use_record.spelling {
                continue;
            }
            for class in crate::CONTAINER_NOMINAL_CLASSES {
                if !universe.contains(&class) {
                    continue;
                }
                available.insert(class);
                if admissible.contains(&class)
                    && system_admissible(use_record.role)
                    && let Ok(ordinal) = u8::try_from(ordinal)
                {
                    candidates.push(ResolvedTarget::Container(crate::ContainerNominalId::new(
                        ordinal,
                    )));
                }
            }
        }
        for (ordinal, operation) in crate::KERNEL_OPERATIONS.iter().enumerate() {
            if operation.spelling == use_record.spelling
                && universe.contains(&crate::KERNEL_OPERATION_CLASS)
            {
                available.insert(crate::KERNEL_OPERATION_CLASS);
                if admissible.contains(&crate::KERNEL_OPERATION_CLASS)
                    && system_admissible(use_record.role)
                    && let Ok(ordinal) = u8::try_from(ordinal)
                {
                    candidates.push(ResolvedTarget::Kernel(crate::KernelOperationId::new(
                        ordinal,
                    )));
                }
            }
        }
        if universe.contains(&DeclarationClass::OperationFamily)
            && let Some(operation) = operation_id(&use_record.spelling)
        {
            available.insert(DeclarationClass::OperationFamily);
            if admissible.contains(&DeclarationClass::OperationFamily) {
                candidates.push(ResolvedTarget::Operation(operation));
            }
        }

        if use_record.role == LexicalUseRole::BreakLabel {
            candidates.retain(|target| match target {
                ResolvedTarget::Source { declaration, .. } => metas
                    .get(declaration.index())
                    .is_some_and(|meta| meta.owner == use_record.function_owner),
                _ => false,
            });
            if candidates.is_empty() {
                let labels: Vec<_> = index
                    .with_spelling(&use_record.spelling)
                    .iter()
                    .filter_map(|candidate| metas.get(*candidate))
                    .filter(|meta| meta.owner == use_record.function_owner)
                    .filter_map(|meta| {
                        let declaration = &declarations[meta.record_index];
                        meta.entries
                            .contains(&DeclarationClass::Label)
                            .then(|| DeclarationOrigin::Source(declaration.origin.clone()))
                    })
                    .collect();
                if !labels.is_empty() {
                    return Ok((
                        resolved,
                        Some(ResolutionIssue {
                            rule: ResolutionRule::Type6,
                            origin: use_record.origin.clone(),
                            kind: ResolutionIssueKind::NonEnclosingLabel {
                                spelling: use_record.spelling.clone(),
                                role: use_record.role,
                                origins: labels,
                            },
                        }),
                    ));
                }
            }
        }

        match candidates.as_slice() {
            [target] => resolved.push(LexicalUseRecord {
                role: use_record.role,
                spelling: use_record.spelling.clone(),
                origin: use_record.origin.clone(),
                target: *target,
            }),
            [] if !invisible.is_empty() && use_record.role != LexicalUseRole::BreakLabel => {
                invisible.sort_by(|left, right| {
                    conflict_key(left, declarations).cmp(&conflict_key(right, declarations))
                });
                return Ok((
                    resolved,
                    Some(ResolutionIssue {
                        rule: use_rule(use_record.role),
                        origin: use_record.origin.clone(),
                        kind: ResolutionIssueKind::InvisibleUse {
                            spelling: use_record.spelling.clone(),
                            role: use_record.role,
                            admissible,
                            origins: invisible,
                        },
                    }),
                ));
            }
            [] => {
                let mut available: Vec<_> = available.into_iter().collect();
                available.sort_unstable();
                return Ok((
                    resolved,
                    Some(ResolutionIssue {
                        rule: use_rule(use_record.role),
                        origin: use_record.origin.clone(),
                        kind: ResolutionIssueKind::UnresolvedUse {
                            spelling: use_record.spelling.clone(),
                            role: use_record.role,
                            admissible,
                            available,
                        },
                    }),
                ));
            }
            _ => return Err(ResolutionCompilerFailure::AmbiguousResolution),
        }
    }
    Ok((resolved, None))
}

/// Whether one lexical-use role admits an admitted system entry at all.
///
/// TYPE-6's admitted-uses column and [SYS-2]'s own exclusion are
/// entry-source-specific, not only class-specific: a `callee` IDENT admits a
/// top-level function or an admitted system operation, while a `fn_bind`
/// right IDENT admits only a top-level function, so the shared `Function`
/// class cannot make that distinction by itself.
fn system_admissible(role: LexicalUseRole) -> bool {
    matches!(
        role,
        LexicalUseRole::Type
            | LexicalUseRole::Construct
            | LexicalUseRole::ArmVariant
            | LexicalUseRole::EnsuresVariant
            | LexicalUseRole::IdentifierCallee
    )
}

fn admissible_classes(role: LexicalUseRole, spelling: &str) -> Vec<DeclarationClass> {
    match role {
        LexicalUseRole::Type => vec![DeclarationClass::GenericType, DeclarationClass::NominalType],
        LexicalUseRole::GenericBound | LexicalUseRole::ConformanceContract => {
            vec![DeclarationClass::Contract]
        }
        LexicalUseRole::Construct => vec![
            DeclarationClass::StructConstructor,
            DeclarationClass::EnumVariant,
        ],
        LexicalUseRole::ArmVariant | LexicalUseRole::EnsuresVariant => {
            vec![DeclarationClass::EnumVariant]
        }
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectAllocationRegion
        | LexicalUseRole::BorrowRegion => vec![DeclarationClass::Region],
        LexicalUseRole::EffectRoot => vec![DeclarationClass::Value],
        LexicalUseRole::BreakLabel => vec![DeclarationClass::Label],
        LexicalUseRole::Const => {
            vec![DeclarationClass::NamedConst, DeclarationClass::ConstGeneric]
        }
        LexicalUseRole::ConstValue => vec![DeclarationClass::NamedConst],
        LexicalUseRole::PlaceBase => {
            // [MSR-6] a `pbase` admits an in-scope const generic beside a
            // named const: a const generic is a monomorphization-time
            // constant and already an [ENT-2] symbolic constant term, so
            // this admission adds a spelling and no fact source.
            vec![
                DeclarationClass::NamedConst,
                DeclarationClass::ConstGeneric,
                DeclarationClass::Value,
            ]
        }
        LexicalUseRole::IdentifierCallee => {
            if operation_id(spelling).is_some() {
                vec![DeclarationClass::OperationFamily]
            } else {
                vec![DeclarationClass::Function]
            }
        }
        LexicalUseRole::OperationCallee => vec![DeclarationClass::OperationFamily],
        LexicalUseRole::FunctionBinding => vec![DeclarationClass::Function],
        LexicalUseRole::GenericNumericSuffix => vec![DeclarationClass::GenericType],
        // [MSR-6, INV-1] an affine atom is one bare place whose `pbase` is an
        // IDENT, and an in-scope const generic is a value in exactly that
        // position: it is a constant of [ENT-2] clause (c) rather than a
        // tracked place, so it reaches the affine domain as an immutable atom
        // nothing kills.
        LexicalUseRole::InvariantValue | LexicalUseRole::ProofValue => {
            // A named integer const is already an [ENT-2] constant term, so it
            // denotes the same value in a proof relation that it denotes
            // everywhere else; excluding it here forced the digits to be
            // rewritten inline in every invariant that names the same limit.
            vec![
                DeclarationClass::Value,
                DeclarationClass::ConstGeneric,
                DeclarationClass::NamedConst,
            ]
        }
        LexicalUseRole::InvariantFact => vec![DeclarationClass::Invariant],
    }
}

fn universe_classes(role: LexicalUseRole) -> Vec<DeclarationClass> {
    match role {
        LexicalUseRole::Type | LexicalUseRole::GenericNumericSuffix => {
            vec![DeclarationClass::GenericType, DeclarationClass::NominalType]
        }
        LexicalUseRole::GenericBound | LexicalUseRole::ConformanceContract => {
            vec![DeclarationClass::Contract]
        }
        LexicalUseRole::Construct | LexicalUseRole::ArmVariant | LexicalUseRole::EnsuresVariant => {
            vec![
                DeclarationClass::StructConstructor,
                DeclarationClass::EnumVariant,
            ]
        }
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectAllocationRegion
        | LexicalUseRole::BorrowRegion => vec![DeclarationClass::Region],
        LexicalUseRole::EffectRoot => vec![DeclarationClass::Value],
        LexicalUseRole::BreakLabel => vec![DeclarationClass::Label],
        LexicalUseRole::Const
        | LexicalUseRole::ConstValue
        | LexicalUseRole::PlaceBase
        | LexicalUseRole::FunctionBinding => vec![
            DeclarationClass::Function,
            DeclarationClass::NamedConst,
            DeclarationClass::ConstGeneric,
            DeclarationClass::Value,
        ],
        LexicalUseRole::IdentifierCallee => vec![
            DeclarationClass::Function,
            DeclarationClass::NamedConst,
            DeclarationClass::ConstGeneric,
            DeclarationClass::Value,
            DeclarationClass::OperationFamily,
        ],
        LexicalUseRole::OperationCallee => vec![DeclarationClass::OperationFamily],
        LexicalUseRole::InvariantValue | LexicalUseRole::ProofValue => {
            // A named integer const is already an [ENT-2] constant term, so it
            // denotes the same value in a proof relation that it denotes
            // everywhere else; excluding it here forced the digits to be
            // rewritten inline in every invariant that names the same limit.
            vec![
                DeclarationClass::Value,
                DeclarationClass::ConstGeneric,
                DeclarationClass::NamedConst,
            ]
        }
        LexicalUseRole::InvariantFact => vec![DeclarationClass::Invariant],
    }
}

fn use_rule(role: LexicalUseRole) -> ResolutionRule {
    match role {
        LexicalUseRole::Type | LexicalUseRole::PlaceBase => ResolutionRule::Type5,
        LexicalUseRole::GenericBound | LexicalUseRole::ConformanceContract => ResolutionRule::Fn3,
        LexicalUseRole::Construct
        | LexicalUseRole::ArmVariant
        | LexicalUseRole::EnsuresVariant
        | LexicalUseRole::BreakLabel => ResolutionRule::Type6,
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectAllocationRegion
        | LexicalUseRole::BorrowRegion => ResolutionRule::Own3,
        LexicalUseRole::EffectRoot => ResolutionRule::Eff1,
        LexicalUseRole::Const => ResolutionRule::Const1,
        LexicalUseRole::ConstValue => ResolutionRule::Const2,
        LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee => ResolutionRule::Op1,
        LexicalUseRole::FunctionBinding => ResolutionRule::Fn4,
        LexicalUseRole::GenericNumericSuffix => ResolutionRule::Form5,
        LexicalUseRole::InvariantValue => ResolutionRule::Inv1,
        LexicalUseRole::ProofValue => ResolutionRule::Prf1,
        LexicalUseRole::InvariantFact => ResolutionRule::Inv1,
    }
}
