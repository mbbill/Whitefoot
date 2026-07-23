use std::collections::HashMap;

use crate::syntax::terminal::TerminalPredicateV0_15;
use crate::syntax::{FinalizedTopology, NodeId};
use crate::{ByteOffset, CanonicalSyntaxUnit, ProductionV0_15, SyntaxCoordinate};

use super::super::scopes::ScopeBuild;
use super::super::{
    DeclarationRole, DeferredUseRole, DependentDeclarationRole, LexicalUseRole,
    ResolutionCompilerFailure, SourceOrigin,
};
use super::{ClassifiedRole, EventKey, RawRole, RawRoleKind, owner_chain};

pub(super) fn classify_roles(
    syntax: &CanonicalSyntaxUnit<'_, '_, '_>,
    scopes: &ScopeBuild,
) -> Result<Vec<ClassifiedRole>, ResolutionCompilerFailure> {
    let topology = &syntax.finalized.topology;
    let classified = syntax.classified_bundle();
    let mut raw = Vec::new();
    let mut complete_role_counts = vec![0_u8; topology.terminals.len()];
    let direct_terminals = direct_terminals_by_owner(topology)?;
    for index in 0..topology.nodes.len() {
        let node = NodeId::from_index(index).ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        let record = topology
            .node(node)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let direct = direct_terminals
            .get(index)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        classify_node(
            classified,
            record.production,
            node,
            direct,
            &mut raw,
            &mut complete_role_counts,
        )?;
    }
    for (index, terminal) in topology.terminals.iter().enumerate() {
        let token = classified
            .tokens()
            .get(index)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        if terminal.owner.is_none() {
            return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
        }
        let is_name = [
            TerminalPredicateV0_15::Identifier,
            TerminalPredicateV0_15::TypeIdentifier,
            TerminalPredicateV0_15::RegionIdentifier,
            TerminalPredicateV0_15::Label,
            TerminalPredicateV0_15::OperationName,
        ]
        .iter()
        .any(|predicate| token.terminals().contains(*predicate));
        if is_name && complete_role_counts.get(index) != Some(&1) {
            return Err(ResolutionCompilerFailure::UnclassifiedNameRole);
        }
    }

    let mut carrier_ordinals = HashMap::new();
    let mut carrier_rows: Vec<_> = raw
        .iter()
        .enumerate()
        .map(|(index, role)| {
            (
                role.owner.index(),
                role.source.ordinal(),
                role.carrier_start.value(),
                role.carrier_end.value(),
                role.kind.class_ordinal(),
                index,
            )
        })
        .collect();
    carrier_rows.sort_unstable();
    let mut previous = None;
    let mut ordinal = 0_u32;
    for (owner, source, start, end, _, raw_index) in carrier_rows {
        let carrier = (owner, source, start, end);
        if previous.is_some_and(|last| last != carrier) {
            ordinal = if previous.is_some_and(|last: (usize, u32, u64, u64)| last.0 == owner) {
                ordinal
                    .checked_add(1)
                    .ok_or(ResolutionCompilerFailure::CounterOverflow)?
            } else {
                0
            };
        } else if previous.is_none() {
            ordinal = 0;
        }
        carrier_ordinals.insert(raw_index, ordinal);
        previous = Some(carrier);
    }

    let mut roles = Vec::with_capacity(raw.len());
    for (index, role) in raw.into_iter().enumerate() {
        let role_ordinal = *carrier_ordinals
            .get(&index)
            .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
        let scope = scopes.node_scope(role.owner)?;
        roles.push(ClassifiedRole {
            kind: role.kind,
            spelling: role.spelling,
            owner: role.owner,
            origin: SourceOrigin {
                node: scopes.path(role.owner)?.clone(),
                coordinate: SyntaxCoordinate::new(role.source, role.role_start, role.role_end),
                role_ordinal,
                subtoken_ordinal: role.subtoken_ordinal,
            },
            scope,
            owner_chain: owner_chain(topology, role.owner)?,
        });
    }
    roles.sort_by_key(|role| EventKey::from_origin(&role.origin));
    Ok(roles)
}

fn direct_terminals_by_owner(
    topology: &FinalizedTopology,
) -> Result<Vec<Vec<usize>>, ResolutionCompilerFailure> {
    let mut direct = vec![Vec::new(); topology.nodes.len()];
    for (terminal_index, terminal) in topology.terminals.iter().enumerate() {
        let owner = terminal
            .owner
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        direct
            .get_mut(owner.index())
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .push(terminal_index);
    }
    Ok(direct)
}

fn classify_node(
    classified: &crate::ClassifiedBundle<'_, '_>,
    production: ProductionV0_15,
    owner: NodeId,
    direct: &[usize],
    roles: &mut Vec<RawRole>,
    complete_counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    let names: Vec<_> = direct
        .iter()
        .copied()
        .filter(|index| name_predicate(classified, *index).is_some())
        .collect();
    match production {
        ProductionV0_15::FnDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Function),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::StructDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Struct),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::EnumDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Enum),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Variant => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Variant),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::ContractDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Contract),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::ConstDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::NamedConst),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Gparam => {
            let Some(first) = names.first().copied() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            match name_predicate(classified, first) {
                Some(TerminalPredicateV0_15::TypeIdentifier) => {
                    add_complete(
                        classified,
                        owner,
                        first,
                        RawRoleKind::Declaration(DeclarationRole::GenericType),
                        roles,
                        complete_counts,
                    )?;
                    if let Some(bound) = names.get(1).copied() {
                        add_complete(
                            classified,
                            owner,
                            bound,
                            RawRoleKind::LexicalUse(LexicalUseRole::GenericBound),
                            roles,
                            complete_counts,
                        )?;
                    }
                    if names.len() > 2 {
                        return Err(ResolutionCompilerFailure::InvalidRoleShape);
                    }
                }
                Some(TerminalPredicateV0_15::Identifier) if names.len() == 1 => {
                    add_complete(
                        classified,
                        owner,
                        first,
                        RawRoleKind::Declaration(DeclarationRole::ConstGeneric),
                        roles,
                        complete_counts,
                    )?;
                }
                _ => return Err(ResolutionCompilerFailure::InvalidRoleShape),
            }
        }
        ProductionV0_15::RegionParams => add_all(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::RegionParameter),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Param => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Parameter),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::LetStmt => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Let),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::LoopStmt => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::LoopLabel),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::RegionStmt => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::LocalRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Field => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::Field),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Vfield => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::VariantField),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::FnSig => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::ContractMember),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Fieldbind => {
            if let [field, binder] = names.as_slice() {
                add_complete(
                    classified,
                    owner,
                    *field,
                    RawRoleKind::DeferredUse(DeferredUseRole::MatchField),
                    roles,
                    complete_counts,
                )?;
                add_complete(
                    classified,
                    owner,
                    *binder,
                    RawRoleKind::Declaration(DeclarationRole::MatchBinder),
                    roles,
                    complete_counts,
                )?;
            } else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            }
        }
        ProductionV0_15::Type => add_names_by_predicate(
            classified,
            owner,
            &names,
            TerminalPredicateV0_15::TypeIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::Type),
            TerminalPredicateV0_15::RegionIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::TypeRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::ConformDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ConformanceContract),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Construct => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::Construct),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Arm => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ArmVariant),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Mode if !names.is_empty() => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ModeRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Targ if !names.is_empty() => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::TypeArgumentRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Effect if !names.is_empty() => add_all(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::EffectRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::BorrowExpr => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::BorrowRegion),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::BreakStmt => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::BreakLabel),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Const if !names.is_empty() => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::Const),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Cvalue => {
            if !names.is_empty() {
                add_single(
                    classified,
                    owner,
                    &names,
                    RawRoleKind::LexicalUse(LexicalUseRole::ConstValue),
                    roles,
                    complete_counts,
                )?;
            }
        }
        ProductionV0_15::Pbase => {
            if !names.is_empty() {
                add_single(
                    classified,
                    owner,
                    &names,
                    RawRoleKind::LexicalUse(LexicalUseRole::PlaceBase),
                    roles,
                    complete_counts,
                )?;
            }
        }
        ProductionV0_15::Callee => {
            let [callee] = names.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            let use_role = match name_predicate(classified, *callee) {
                Some(TerminalPredicateV0_15::Identifier) => LexicalUseRole::IdentifierCallee,
                Some(TerminalPredicateV0_15::OperationName) => LexicalUseRole::OperationCallee,
                _ => return Err(ResolutionCompilerFailure::InvalidRoleShape),
            };
            add_complete(
                classified,
                owner,
                *callee,
                RawRoleKind::LexicalUse(use_role),
                roles,
                complete_counts,
            )?;
        }
        ProductionV0_15::FnBind => {
            if let [member, function] = names.as_slice() {
                add_complete(
                    classified,
                    owner,
                    *member,
                    RawRoleKind::DeferredUse(DeferredUseRole::ContractBinding),
                    roles,
                    complete_counts,
                )?;
                add_complete(
                    classified,
                    owner,
                    *function,
                    RawRoleKind::LexicalUse(LexicalUseRole::FunctionBinding),
                    roles,
                    complete_counts,
                )?;
            } else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            }
        }
        ProductionV0_15::Fieldinit => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DeferredUse(DeferredUseRole::FieldInitializer),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Psuffix => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DeferredUse(DeferredUseRole::ProjectedField),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::Law => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DeferredUse(DeferredUseRole::LawName),
            roles,
            complete_counts,
        )?,
        ProductionV0_15::LawArg => {
            let [argument] = direct else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            add_complete(
                classified,
                owner,
                *argument,
                RawRoleKind::DeferredUse(DeferredUseRole::LawArgument),
                roles,
                complete_counts,
            )?;
        }
        _ => {}
    }
    if matches!(
        production,
        ProductionV0_15::Atom | ProductionV0_15::Cvalue | ProductionV0_15::LawArg
    ) {
        for terminal in direct {
            add_generic_suffix(classified, owner, *terminal, roles)?;
        }
    }
    Ok(())
}

fn name_predicate(
    classified: &crate::ClassifiedBundle<'_, '_>,
    terminal: usize,
) -> Option<TerminalPredicateV0_15> {
    let set = classified.tokens().get(terminal)?.terminals();
    [
        TerminalPredicateV0_15::Identifier,
        TerminalPredicateV0_15::TypeIdentifier,
        TerminalPredicateV0_15::RegionIdentifier,
        TerminalPredicateV0_15::Label,
        TerminalPredicateV0_15::OperationName,
    ]
    .into_iter()
    .find(|predicate| set.contains(*predicate))
}

fn add_single(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    terminals: &[usize],
    kind: RawRoleKind,
    roles: &mut Vec<RawRole>,
    counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    let [terminal] = terminals else {
        return Err(ResolutionCompilerFailure::InvalidRoleShape);
    };
    add_complete(classified, owner, *terminal, kind, roles, counts)
}

fn add_all(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    terminals: &[usize],
    kind: RawRoleKind,
    roles: &mut Vec<RawRole>,
    counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    if terminals.is_empty() {
        return Err(ResolutionCompilerFailure::InvalidRoleShape);
    }
    for terminal in terminals {
        add_complete(classified, owner, *terminal, kind, roles, counts)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add_names_by_predicate(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    terminals: &[usize],
    first_predicate: TerminalPredicateV0_15,
    first_kind: RawRoleKind,
    second_predicate: TerminalPredicateV0_15,
    second_kind: RawRoleKind,
    roles: &mut Vec<RawRole>,
    counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    for terminal in terminals {
        let predicate = name_predicate(classified, *terminal)
            .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
        let kind = if predicate == first_predicate {
            first_kind
        } else if predicate == second_predicate {
            second_kind
        } else {
            return Err(ResolutionCompilerFailure::InvalidRoleShape);
        };
        add_complete(classified, owner, *terminal, kind, roles, counts)?;
    }
    Ok(())
}

fn add_complete(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    terminal: usize,
    kind: RawRoleKind,
    roles: &mut Vec<RawRole>,
    counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    let token = classified
        .tokens()
        .get(terminal)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
        .token();
    let id = token.id();
    let spelling = std::str::from_utf8(token.span().bytes())
        .map_err(|_| ResolutionCompilerFailure::InvalidNameEncoding)?
        .to_owned();
    let count = counts
        .get_mut(terminal)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    *count = count
        .checked_add(1)
        .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
    roles.push(RawRole {
        kind,
        spelling,
        owner,
        source: id.source(),
        carrier_start: id.start(),
        carrier_end: id.end(),
        role_start: id.start(),
        role_end: id.end(),
        subtoken_ordinal: 0,
    });
    Ok(())
}

fn add_generic_suffix(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    terminal: usize,
    roles: &mut Vec<RawRole>,
) -> Result<(), ResolutionCompilerFailure> {
    let classified_token = classified
        .tokens()
        .get(terminal)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    if !classified_token
        .terminals()
        .contains(TerminalPredicateV0_15::Literal)
    {
        return Ok(());
    }
    let token = classified_token.token();
    let bytes = token.span().bytes();
    if bytes.len() < 3 || !matches!(&bytes[..2], b"0_" | b"1_") {
        return Ok(());
    }
    let suffix = std::str::from_utf8(&bytes[2..])
        .map_err(|_| ResolutionCompilerFailure::InvalidNameEncoding)?;
    if !suffix
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_uppercase)
    {
        return Ok(());
    }
    let start = token
        .id()
        .start()
        .value()
        .checked_add(2)
        .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
    roles.push(RawRole {
        kind: RawRoleKind::LexicalUse(LexicalUseRole::GenericNumericSuffix),
        spelling: suffix.to_owned(),
        owner,
        source: token.id().source(),
        carrier_start: token.id().start(),
        carrier_end: token.id().end(),
        role_start: ByteOffset::new(start),
        role_end: token.id().end(),
        subtoken_ordinal: 1,
    });
    Ok(())
}
