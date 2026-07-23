use std::collections::HashSet;

use super::super::catalog::{PRELUDE_DECLARATIONS, operation_id};
use super::super::scopes::ScopeBuild;
use super::super::{
    DeclarationClass, DeclarationOrigin, DeclarationRecord, LexicalUseRecord, LexicalUseRole,
    ResolutionCompilerFailure, ResolutionIssue, ResolutionIssueKind, ResolutionRuleV0_14,
    ResolvedTarget, ScopeId,
};
use super::inventory::conflict_key;
use super::{BuildStop, DeclarationIndex, DeclarationMeta, UseMeta, is_visible};

pub(super) fn resolve_uses(
    scopes: &ScopeBuild,
    declarations: &[DeclarationRecord],
    metas: &[DeclarationMeta],
    index: &DeclarationIndex,
    uses: &[UseMeta],
) -> Result<Vec<LexicalUseRecord>, BuildStop> {
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
                    return Err(BuildStop::Issue(Box::new(ResolutionIssue {
                        rule: ResolutionRuleV0_14::Type6,
                        origin: use_record.origin.clone(),
                        kind: ResolutionIssueKind::NonEnclosingLabel {
                            spelling: use_record.spelling.clone(),
                            role: use_record.role,
                            origins: labels,
                        },
                    })));
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
                return Err(BuildStop::Issue(Box::new(ResolutionIssue {
                    rule: use_rule(use_record.role),
                    origin: use_record.origin.clone(),
                    kind: ResolutionIssueKind::InvisibleUse {
                        spelling: use_record.spelling.clone(),
                        role: use_record.role,
                        admissible,
                        origins: invisible,
                    },
                })));
            }
            [] => {
                let mut available: Vec<_> = available.into_iter().collect();
                available.sort_unstable();
                return Err(BuildStop::Issue(Box::new(ResolutionIssue {
                    rule: use_rule(use_record.role),
                    origin: use_record.origin.clone(),
                    kind: ResolutionIssueKind::UnresolvedUse {
                        spelling: use_record.spelling.clone(),
                        role: use_record.role,
                        admissible,
                        available,
                    },
                })));
            }
            _ => return Err(ResolutionCompilerFailure::AmbiguousResolution.into()),
        }
    }
    Ok(resolved)
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
        LexicalUseRole::ArmVariant => vec![DeclarationClass::EnumVariant],
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectRegion
        | LexicalUseRole::BorrowRegion => vec![DeclarationClass::Region],
        LexicalUseRole::BreakLabel => vec![DeclarationClass::Label],
        LexicalUseRole::Const => {
            vec![DeclarationClass::NamedConst, DeclarationClass::ConstGeneric]
        }
        LexicalUseRole::ConstValue => vec![DeclarationClass::NamedConst],
        LexicalUseRole::PlaceBase => {
            vec![DeclarationClass::NamedConst, DeclarationClass::Value]
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
        LexicalUseRole::Construct | LexicalUseRole::ArmVariant => vec![
            DeclarationClass::StructConstructor,
            DeclarationClass::EnumVariant,
        ],
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectRegion
        | LexicalUseRole::BorrowRegion => vec![DeclarationClass::Region],
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
    }
}

fn use_rule(role: LexicalUseRole) -> ResolutionRuleV0_14 {
    match role {
        LexicalUseRole::Type | LexicalUseRole::PlaceBase => ResolutionRuleV0_14::Type5,
        LexicalUseRole::GenericBound | LexicalUseRole::ConformanceContract => {
            ResolutionRuleV0_14::Fn3
        }
        LexicalUseRole::Construct | LexicalUseRole::ArmVariant | LexicalUseRole::BreakLabel => {
            ResolutionRuleV0_14::Type6
        }
        LexicalUseRole::TypeRegion
        | LexicalUseRole::ModeRegion
        | LexicalUseRole::TypeArgumentRegion
        | LexicalUseRole::EffectRegion
        | LexicalUseRole::BorrowRegion => ResolutionRuleV0_14::Own3,
        LexicalUseRole::Const => ResolutionRuleV0_14::Const1,
        LexicalUseRole::ConstValue => ResolutionRuleV0_14::Const2,
        LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee => {
            ResolutionRuleV0_14::Op1
        }
        LexicalUseRole::FunctionBinding => ResolutionRuleV0_14::Fn4,
        LexicalUseRole::GenericNumericSuffix => ResolutionRuleV0_14::Form5,
    }
}
