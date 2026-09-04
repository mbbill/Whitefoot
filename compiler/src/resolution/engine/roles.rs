use std::collections::HashMap;

use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::syntax::{FinalizedTopology, NodeId};
use crate::{ByteOffset, CanonicalSyntaxUnit, Production, SyntaxCoordinate};

use super::super::scopes::ScopeBuild;
use super::super::{
    DeclarationRole, DeferredUseRole, DependentDeclarationRole, LexicalUseRole,
    ResolutionCompilerFailure, SourceOrigin,
};
use super::{ClassifiedRole, EventKey, RawRole, RawRoleKind, SelectorRole, owner_chain};

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
            topology,
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
            TerminalPredicate::Identifier,
            TerminalPredicate::TypeIdentifier,
            TerminalPredicate::RegionIdentifier,
            TerminalPredicate::Label,
            TerminalPredicate::OperationName,
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
    topology: &FinalizedTopology,
    classified: &crate::ClassifiedBundle<'_, '_>,
    production: Production,
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
        Production::FnDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Function),
            roles,
            complete_counts,
        )?,
        Production::StructDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Struct),
            roles,
            complete_counts,
        )?,
        Production::EnumDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Enum),
            roles,
            complete_counts,
        )?,
        Production::Variant => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Variant),
            roles,
            complete_counts,
        )?,
        Production::ContractDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Contract),
            roles,
            complete_counts,
        )?,
        Production::ConstDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::NamedConst),
            roles,
            complete_counts,
        )?,
        Production::Gparam => {
            let Some(first) = names.first().copied() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            match name_predicate(classified, first) {
                Some(TerminalPredicate::TypeIdentifier) => {
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
                Some(TerminalPredicate::Identifier) if names.len() == 1 => {
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
        Production::RegionParams => add_all(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::RegionParameter),
            roles,
            complete_counts,
        )?,
        Production::Param => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Parameter),
            roles,
            complete_counts,
        )?,
        Production::ProgramKind if !names.is_empty() => {
            return Err(ResolutionCompilerFailure::InvalidRoleShape);
        }
        Production::InputLabel => {
            let [label] = names.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            add_complete(
                classified,
                owner,
                *label,
                RawRoleKind::TableChecked,
                roles,
                complete_counts,
            )?;
        }
        Production::InvariantStmt | Production::HeaderInvariant => {
            // The relation is a `compare_op` terminal between two affine
            // expressions, not a name; the only direct IDENT is the
            // invariant's own declaration.
            let [name] = names.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            if name_predicate(classified, *name) != Some(TerminalPredicate::Identifier) {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            }
            add_complete(
                classified,
                owner,
                *name,
                RawRoleKind::Declaration(DeclarationRole::Invariant),
                roles,
                complete_counts,
            )?;
        }
        Production::ProofUse => {
            // A relation-form use has no direct IDENT: its values sit inside
            // the affine expressions and take the `ProofValue` role below. A
            // named use has exactly one, the invariant it cites.
            let relation_form = topology
                .node_children(owner)
                .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
                .iter()
                .any(|child| {
                    topology
                        .node(*child)
                        .is_some_and(|record| record.production == Production::AffineExpr)
                });
            if relation_form {
                if !names.is_empty() {
                    return Err(ResolutionCompilerFailure::InvalidRoleShape);
                }
            } else {
                let [carrier] = names.as_slice() else {
                    return Err(ResolutionCompilerFailure::InvalidRoleShape);
                };
                if name_predicate(classified, *carrier) != Some(TerminalPredicate::Identifier) {
                    return Err(ResolutionCompilerFailure::InvalidRoleShape);
                }
                add_complete(
                    classified,
                    owner,
                    *carrier,
                    RawRoleKind::LexicalUse(LexicalUseRole::InvariantFact),
                    roles,
                    complete_counts,
                )?;
            }
        }
        Production::AffineFactor if !names.is_empty() => {
            let role = if ancestor_with_production(topology, owner, Production::ProofUse).is_some()
            {
                LexicalUseRole::ProofValue
            } else {
                LexicalUseRole::InvariantValue
            };
            add_all(
                classified,
                owner,
                &names,
                RawRoleKind::LexicalUse(role),
                roles,
                complete_counts,
            )?;
        }
        Production::LetStmt | Production::ContractDefine => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::Let),
            roles,
            complete_counts,
        )?,
        Production::ForBinding => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Declaration(DeclarationRole::CountedBinder),
            roles,
            complete_counts,
        )?,
        Production::LoopStmt | Production::ForStmt => {
            match names.as_slice() {
                [] => {}
                [label] if name_predicate(classified, *label) == Some(TerminalPredicate::Label) => {
                    add_complete(
                        classified,
                        owner,
                        *label,
                        RawRoleKind::Declaration(DeclarationRole::LoopLabel),
                        roles,
                        complete_counts,
                    )?;
                }
                _ => return Err(ResolutionCompilerFailure::InvalidRoleShape),
            }
            // [OWN-11] every loop body is a region block. The body's own
            // region is unnamed and no position can write it [FORM-8], so its
            // declaration is minted at the `loop` or `for` token under a
            // spelling no source token can form, exactly as an unnamed
            // `region_stmt`'s is.
            add_elided_region(
                classified,
                owner,
                direct,
                if production == Production::LoopStmt {
                    FixedTerminal::Loop
                } else {
                    FixedTerminal::For
                },
                DeclarationRole::LocalRegion,
                roles,
                complete_counts,
            )?;
        }
        Production::RegionStmt => {
            if names.is_empty() {
                // [FORM-8] `region { ... }`: the block still introduces one
                // local region, so the declaration is minted at the `region`
                // token under a spelling no source token can form.
                add_elided_region(
                    classified,
                    owner,
                    direct,
                    FixedTerminal::Region,
                    DeclarationRole::LocalRegion,
                    roles,
                    complete_counts,
                )?;
            } else {
                add_single(
                    classified,
                    owner,
                    &names,
                    RawRoleKind::Declaration(DeclarationRole::LocalRegion),
                    roles,
                    complete_counts,
                )?;
            }
        }
        Production::Field => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::Field),
            roles,
            complete_counts,
        )?,
        Production::Vfield => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::VariantField),
            roles,
            complete_counts,
        )?,
        Production::FnSig => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DependentDeclaration(DependentDeclarationRole::ContractMember),
            roles,
            complete_counts,
        )?,
        Production::ResultBinding => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::Selector(SelectorRole::PlainCandidate),
            roles,
            complete_counts,
        )?,
        Production::ResultRoute => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::EnsuresVariant),
            roles,
            complete_counts,
        )?,
        Production::Fieldbind => {
            if let [field, binder] = names.as_slice() {
                let selector_field =
                    ancestor_with_production(topology, owner, Production::ResultRoute).is_some();
                add_complete(
                    classified,
                    owner,
                    *field,
                    if selector_field {
                        RawRoleKind::Selector(SelectorRole::VariantField)
                    } else {
                        RawRoleKind::DeferredUse(DeferredUseRole::MatchField)
                    },
                    roles,
                    complete_counts,
                )?;
                add_complete(
                    classified,
                    owner,
                    *binder,
                    if selector_field {
                        RawRoleKind::Selector(SelectorRole::VariantCandidate)
                    } else {
                        RawRoleKind::Declaration(DeclarationRole::MatchBinder)
                    },
                    roles,
                    complete_counts,
                )?;
            } else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            }
        }
        // [FORM-8] `slice<T>` / `arena<T>`: the view still carries one region.
        Production::Type
            if !direct.is_empty()
                && (has_fixed_terminal(classified, direct, FixedTerminal::Slice)
                    || has_fixed_terminal(classified, direct, FixedTerminal::Arena))
                && !names.iter().any(|index| {
                    name_predicate(classified, *index) == Some(TerminalPredicate::RegionIdentifier)
                }) =>
        {
            let anchor = if has_fixed_terminal(classified, direct, FixedTerminal::Slice) {
                FixedTerminal::Slice
            } else {
                FixedTerminal::Arena
            };
            add_elided_region(
                classified,
                owner,
                direct,
                anchor,
                DeclarationRole::RegionParameter,
                roles,
                complete_counts,
            )?;
        }
        Production::Type => add_names_by_predicate(
            classified,
            owner,
            &names,
            TerminalPredicate::TypeIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::Type),
            TerminalPredicate::RegionIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::TypeRegion),
            roles,
            complete_counts,
        )?,
        Production::ConformDecl => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ConformanceContract),
            roles,
            complete_counts,
        )?,
        Production::Construct => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::Construct),
            roles,
            complete_counts,
        )?,
        Production::Arm => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ArmVariant),
            roles,
            complete_counts,
        )?,
        Production::Mode if !names.is_empty() => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::ModeRegion),
            roles,
            complete_counts,
        )?,
        // [FORM-8] `&T` / `&uniq T`: the borrow mode still carries one region,
        // fresh and distinct from every other region of its declaration.
        Production::Mode if has_fixed_terminal(classified, direct, FixedTerminal::Ampersand) => {
            add_elided_region(
                classified,
                owner,
                direct,
                FixedTerminal::Ampersand,
                DeclarationRole::RegionParameter,
                roles,
                complete_counts,
            )?;
        }
        Production::Targ if !names.is_empty() => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::TypeArgumentRegion),
            roles,
            complete_counts,
        )?,
        Production::EffectPath => {
            let Some((root, fields)) = names.split_first() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            add_complete(
                classified,
                owner,
                *root,
                RawRoleKind::LexicalUse(LexicalUseRole::EffectRoot),
                roles,
                complete_counts,
            )?;
            for field in fields {
                add_complete(
                    classified,
                    owner,
                    *field,
                    RawRoleKind::DeferredUse(DeferredUseRole::EffectField),
                    roles,
                    complete_counts,
                )?;
            }
        }
        Production::Effect if !names.is_empty() => add_names_by_predicate(
            classified,
            owner,
            &names,
            TerminalPredicate::RegionIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::EffectAllocationRegion),
            TerminalPredicate::RegionIdentifier,
            RawRoleKind::LexicalUse(LexicalUseRole::EffectAllocationRegion),
            roles,
            complete_counts,
        )?,
        // [FORM-8] an elided `&p` / `&uniq p` denotes the innermost enclosing
        // `region_stmt`'s region. That target is lexical rather than a name
        // lookup, so no use role is classified here and the checker resolves
        // it from the enclosing construct.
        Production::BorrowExpr if names.is_empty() => {}
        Production::BorrowExpr => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::BorrowRegion),
            roles,
            complete_counts,
        )?,
        Production::BreakStmt => match names.as_slice() {
            [] => {}
            [label] if name_predicate(classified, *label) == Some(TerminalPredicate::Label) => {
                add_complete(
                    classified,
                    owner,
                    *label,
                    RawRoleKind::LexicalUse(LexicalUseRole::BreakLabel),
                    roles,
                    complete_counts,
                )?;
            }
            _ => return Err(ResolutionCompilerFailure::InvalidRoleShape),
        },
        // Every IDENT term of a `const` expression is one Const use; the
        // candidate CONST-1 grammar admits two terms in one operation, and
        // source order is retained through the per-name role ordinal.
        Production::Const if !names.is_empty() => add_all(
            classified,
            owner,
            &names,
            RawRoleKind::LexicalUse(LexicalUseRole::Const),
            roles,
            complete_counts,
        )?,
        Production::Cvalue => {
            // The candidate CONST-2 construction cvalue owns one TYPEID (the
            // constructor) plus its direct field labels; the reference cvalue
            // owns exactly one IDENT naming an earlier const.
            let constructor = names.iter().copied().find(|index| {
                name_predicate(classified, *index) == Some(TerminalPredicate::TypeIdentifier)
            });
            if let Some(constructor) = constructor {
                add_complete(
                    classified,
                    owner,
                    constructor,
                    RawRoleKind::LexicalUse(LexicalUseRole::Construct),
                    roles,
                    complete_counts,
                )?;
                for label in names.iter().copied().filter(|index| *index != constructor) {
                    add_complete(
                        classified,
                        owner,
                        label,
                        RawRoleKind::DeferredUse(DeferredUseRole::FieldInitializer),
                        roles,
                        complete_counts,
                    )?;
                }
            } else if !names.is_empty() {
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
        Production::Pbase => {
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
        Production::Callee => {
            let [callee] = names.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape);
            };
            let use_role = match name_predicate(classified, *callee) {
                Some(TerminalPredicate::Identifier) => LexicalUseRole::IdentifierCallee,
                Some(TerminalPredicate::OperationName) => LexicalUseRole::OperationCallee,
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
        Production::FnBind => {
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
        Production::Fieldinit => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DeferredUse(DeferredUseRole::FieldInitializer),
            roles,
            complete_counts,
        )?,
        Production::Psuffix => {
            // The field alternative owns exactly one projected-field name; the
            // subscript alternative owns only bracket punctuation, and its
            // offset atom classifies through the atom's own productions.
            if !names.is_empty() {
                add_single(
                    classified,
                    owner,
                    &names,
                    RawRoleKind::DeferredUse(DeferredUseRole::ProjectedField),
                    roles,
                    complete_counts,
                )?;
            }
        }
        Production::Law => add_single(
            classified,
            owner,
            &names,
            RawRoleKind::DeferredUse(DeferredUseRole::LawName),
            roles,
            complete_counts,
        )?,
        Production::LawArg => {
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
        Production::Atom | Production::Cvalue | Production::LawArg
    ) {
        for terminal in direct {
            add_generic_suffix(classified, owner, *terminal, roles)?;
        }
    }
    Ok(())
}

fn ancestor_with_production(
    topology: &FinalizedTopology,
    mut node: NodeId,
    production: Production,
) -> Option<NodeId> {
    loop {
        let record = topology.node(node)?;
        if record.production == production {
            return Some(node);
        }
        node = record.parent?;
    }
}

fn name_predicate(
    classified: &crate::ClassifiedBundle<'_, '_>,
    terminal: usize,
) -> Option<TerminalPredicate> {
    let set = classified.tokens().get(terminal)?.terminals();
    [
        TerminalPredicate::Identifier,
        TerminalPredicate::TypeIdentifier,
        TerminalPredicate::RegionIdentifier,
        TerminalPredicate::Label,
        TerminalPredicate::OperationName,
    ]
    .into_iter()
    .find(|predicate| set.contains(*predicate))
}

/// Whether one node writes this exact fixed terminal directly.
fn has_fixed_terminal(
    classified: &crate::ClassifiedBundle<'_, '_>,
    direct: &[usize],
    terminal: FixedTerminal,
) -> bool {
    direct.iter().any(|index| {
        classified.tokens().get(*index).is_some_and(|token| {
            token
                .terminals()
                .contains(TerminalPredicate::Fixed(terminal))
        })
    })
}

/// Declares the region an elided [FORM-8] position denotes.
///
/// The position writes no REGIONID, so the declaration is anchored at the
/// construct's own introducing token and spelled `'0_<source>_<offset>`.
/// [FORM-3] admits only `'[a-z][a-z0-9_]*`, so no source token can form that
/// spelling and no lookup, redeclaration, or shadowing judgment can reach the
/// minted declaration by name; every consumer reaches it through the owning
/// node instead. The offset makes each minted region distinct, which is
/// exactly what [FORM-8] says an unnamed position denotes.
fn add_elided_region(
    classified: &crate::ClassifiedBundle<'_, '_>,
    owner: NodeId,
    direct: &[usize],
    anchor: FixedTerminal,
    role: DeclarationRole,
    roles: &mut Vec<RawRole>,
    counts: &mut [u8],
) -> Result<(), ResolutionCompilerFailure> {
    let terminal = direct
        .iter()
        .copied()
        .find(|index| {
            classified
                .tokens()
                .get(*index)
                .is_some_and(|token| token.terminals().contains(TerminalPredicate::Fixed(anchor)))
        })
        .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
    let token = classified
        .tokens()
        .get(terminal)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
        .token();
    let id = token.id();
    let count = counts
        .get_mut(terminal)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    *count = count
        .checked_add(1)
        .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
    roles.push(RawRole {
        kind: RawRoleKind::Declaration(role),
        spelling: format!("'0_{}_{}", id.source().ordinal(), id.start().value()),
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
    first_predicate: TerminalPredicate,
    first_kind: RawRoleKind,
    second_predicate: TerminalPredicate,
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
        .contains(TerminalPredicate::Literal)
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
