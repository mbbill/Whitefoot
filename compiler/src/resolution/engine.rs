use std::collections::HashMap;

use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{ByteOffset, CanonicalSyntaxUnit, ProductionV0_15, SourceId};

use super::catalog::PRELUDE_DECLARATIONS;
use super::scopes::ScopeBuild;
use super::{
    DeclarationClass, DeclarationDomain, DeclarationId, DeclarationRecord, DeclarationRole,
    DeferredUseRecord, DeferredUseRole, DependentDeclarationRecord, DependentDeclarationRole,
    LexicalUseRecord, LexicalUseRole, PreludeDeclarationRecord, ResolutionCompilerFailure,
    ResolutionIssue, ResolutionOutcome, ResolvedSyntaxUnit, ScopeId, SourceOrigin,
};

mod admission;
mod inventory;
mod lookup;
mod roles;

use admission::check_requires_blocks;
use inventory::check_declaration_inventory;
use lookup::resolve_uses;
use roles::classify_roles;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EventKey {
    source: u32,
    start: u64,
    end: u64,
    path: Vec<u32>,
    role: u32,
    subtoken: u32,
}

impl EventKey {
    fn from_origin(origin: &SourceOrigin) -> Self {
        let coordinate = origin.coordinate;
        Self {
            source: coordinate.source().ordinal(),
            start: coordinate.start().value(),
            end: coordinate.end().value(),
            path: origin.node.components().to_vec(),
            role: origin.role_ordinal,
            subtoken: origin.subtoken_ordinal,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RawRoleKind {
    Declaration(DeclarationRole),
    DependentDeclaration(DependentDeclarationRole),
    LexicalUse(LexicalUseRole),
    DeferredUse(DeferredUseRole),
}

impl RawRoleKind {
    const fn class_ordinal(self) -> u8 {
        match self {
            Self::Declaration(_) | Self::DependentDeclaration(_) => 0,
            Self::LexicalUse(_) => 1,
            Self::DeferredUse(_) => 2,
        }
    }
}

struct RawRole {
    kind: RawRoleKind,
    spelling: String,
    owner: NodeId,
    source: SourceId,
    carrier_start: ByteOffset,
    carrier_end: ByteOffset,
    role_start: ByteOffset,
    role_end: ByteOffset,
    subtoken_ordinal: u32,
}

struct ClassifiedRole {
    kind: RawRoleKind,
    spelling: String,
    owner: NodeId,
    origin: SourceOrigin,
    scope: ScopeId,
    owner_chain: Vec<NodeId>,
}

#[derive(Clone, Copy)]
enum Visibility {
    Always,
    After { source: u32, byte: u64 },
}

struct DeclarationMeta {
    role_index: usize,
    record_index: usize,
    scope: ScopeId,
    owner: Option<NodeId>,
    region_owner: Option<NodeId>,
    visibility: Visibility,
    entries: Vec<DeclarationClass>,
}

struct DeclarationIndex {
    by_spelling: HashMap<String, Vec<usize>>,
}

impl DeclarationIndex {
    fn build(declarations: &[DeclarationRecord]) -> Self {
        let mut by_spelling = HashMap::new();
        for declaration in declarations {
            by_spelling
                .entry(declaration.spelling.clone())
                .or_insert_with(Vec::new)
                .push(declaration.id.index());
        }
        Self { by_spelling }
    }

    fn with_spelling(&self, spelling: &str) -> &[usize] {
        self.by_spelling.get(spelling).map_or(&[], Vec::as_slice)
    }
}

struct UseMeta {
    role: LexicalUseRole,
    spelling: String,
    origin: SourceOrigin,
    scope: ScopeId,
    owner_chain: Vec<NodeId>,
    function_owner: Option<NodeId>,
}

struct Tables {
    scopes: Vec<super::ScopeRecord>,
    prelude: Vec<PreludeDeclarationRecord>,
    declarations: Vec<DeclarationRecord>,
    dependent_declarations: Vec<DependentDeclarationRecord>,
    lexical_uses: Vec<LexicalUseRecord>,
    deferred_uses: Vec<DeferredUseRecord>,
}

enum BuildStop {
    Issue(Box<ResolutionIssue>),
    Compiler(ResolutionCompilerFailure),
}

impl From<ResolutionCompilerFailure> for BuildStop {
    fn from(value: ResolutionCompilerFailure) -> Self {
        Self::Compiler(value)
    }
}

/// Resolves every exact-v0.15 declaration and lexical use in canonical syntax.
#[must_use]
pub fn resolve_v0_15<'classified, 'lexed, 'source>(
    syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
) -> ResolutionOutcome<'classified, 'lexed, 'source> {
    match build_tables(&syntax) {
        Ok(tables) => ResolutionOutcome::Complete(ResolvedSyntaxUnit {
            syntax,
            scopes: tables.scopes,
            prelude: tables.prelude,
            declarations: tables.declarations,
            dependent_declarations: tables.dependent_declarations,
            lexical_uses: tables.lexical_uses,
            deferred_uses: tables.deferred_uses,
        }),
        Err(BuildStop::Issue(issue)) => ResolutionOutcome::SourceIssue {
            syntax,
            issue: *issue,
        },
        Err(BuildStop::Compiler(failure)) => ResolutionOutcome::CompilerFailure { syntax, failure },
    }
}

fn build_tables(syntax: &CanonicalSyntaxUnit<'_, '_, '_>) -> Result<Tables, BuildStop> {
    let topology = &syntax.finalized.topology;
    let scopes = ScopeBuild::build(topology)?;
    if let Some(issue) = check_requires_blocks(topology, &scopes)? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    let roles = classify_roles(syntax, &scopes)?;
    let mut declarations = Vec::new();
    let mut dependent_declarations = Vec::new();
    let mut deferred_uses = Vec::new();
    let mut declaration_metas = Vec::new();
    let mut declaration_by_role = vec![None; roles.len()];
    let mut uses = Vec::new();

    for (role_index, role) in roles.iter().enumerate() {
        match role.kind {
            RawRoleKind::Declaration(declaration_role) => {
                let id = DeclarationId::from_index(declarations.len())
                    .ok_or(ResolutionCompilerFailure::CounterOverflow)?;
                let entries = declaration_classes(declaration_role);
                let record_index = declarations.len();
                declarations.push(DeclarationRecord {
                    id,
                    role: declaration_role,
                    spelling: role.spelling.clone(),
                    origin: role.origin.clone(),
                    scope: declaration_scope(role, declaration_role, &scopes)?,
                    classes: entries.clone(),
                });
                declaration_by_role[role_index] = Some(record_index);
                declaration_metas.push(DeclarationMeta {
                    role_index,
                    record_index,
                    scope: declarations[record_index].scope,
                    owner: role.owner_chain.first().copied(),
                    region_owner: function_owner(topology, role.owner),
                    visibility: declaration_visibility(topology, role, declaration_role)?,
                    entries,
                });
            }
            RawRoleKind::DependentDeclaration(dependent_role) => {
                dependent_declarations.push(DependentDeclarationRecord {
                    role: dependent_role,
                    spelling: role.spelling.clone(),
                    origin: role.origin.clone(),
                });
            }
            RawRoleKind::LexicalUse(use_role) => uses.push(UseMeta {
                role: use_role,
                spelling: role.spelling.clone(),
                origin: role.origin.clone(),
                scope: role.scope,
                owner_chain: role.owner_chain.clone(),
                function_owner: function_owner(topology, role.owner),
            }),
            RawRoleKind::DeferredUse(deferred_role) => deferred_uses.push(DeferredUseRecord {
                role: deferred_role,
                spelling: role.spelling.clone(),
                origin: role.origin.clone(),
            }),
        }
    }

    let declaration_index = DeclarationIndex::build(&declarations);

    if let Some(issue) = check_declaration_inventory(
        topology,
        &scopes,
        &roles,
        &declarations,
        &declaration_metas,
        &declaration_index,
        &declaration_by_role,
    )? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    let lexical_uses = resolve_uses(
        &scopes,
        &declarations,
        &declaration_metas,
        &declaration_index,
        &uses,
    )?;
    Ok(Tables {
        scopes: scopes.records,
        prelude: PRELUDE_DECLARATIONS.to_vec(),
        declarations,
        dependent_declarations,
        lexical_uses,
        deferred_uses,
    })
}

fn owner_chain(
    topology: &FinalizedTopology,
    mut node: NodeId,
) -> Result<Vec<NodeId>, ResolutionCompilerFailure> {
    let mut owners = Vec::new();
    loop {
        let record = topology
            .node(node)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        if matches!(
            record.production,
            ProductionV0_15::FnSig
                | ProductionV0_15::FnDecl
                | ProductionV0_15::StructDecl
                | ProductionV0_15::EnumDecl
                | ProductionV0_15::ContractDecl
        ) {
            owners.push(node);
        }
        let Some(parent) = record.parent else {
            break;
        };
        node = parent;
    }
    Ok(owners)
}

fn function_owner(topology: &FinalizedTopology, mut node: NodeId) -> Option<NodeId> {
    loop {
        let record = topology.node(node)?;
        if matches!(
            record.production,
            ProductionV0_15::FnDecl | ProductionV0_15::FnSig
        ) {
            return Some(node);
        }
        node = record.parent?;
    }
}

fn declaration_classes(role: DeclarationRole) -> Vec<DeclarationClass> {
    match role {
        DeclarationRole::Function => vec![DeclarationClass::Function],
        DeclarationRole::Struct => vec![
            DeclarationClass::NominalType,
            DeclarationClass::StructConstructor,
        ],
        DeclarationRole::Enum => vec![DeclarationClass::NominalType],
        DeclarationRole::Variant => vec![DeclarationClass::EnumVariant],
        DeclarationRole::Contract => vec![DeclarationClass::Contract],
        DeclarationRole::NamedConst => vec![DeclarationClass::NamedConst],
        DeclarationRole::GenericType => vec![DeclarationClass::GenericType],
        DeclarationRole::ConstGeneric => vec![DeclarationClass::ConstGeneric],
        DeclarationRole::RegionParameter | DeclarationRole::LocalRegion => {
            vec![DeclarationClass::Region]
        }
        DeclarationRole::Parameter | DeclarationRole::Let | DeclarationRole::MatchBinder => {
            vec![DeclarationClass::Value]
        }
        DeclarationRole::LoopLabel => vec![DeclarationClass::Label],
    }
}

fn declaration_scope(
    role: &ClassifiedRole,
    declaration_role: DeclarationRole,
    scopes: &ScopeBuild,
) -> Result<ScopeId, ResolutionCompilerFailure> {
    match declaration_role {
        DeclarationRole::Variant => Ok(ScopeId(0)),
        DeclarationRole::LoopLabel | DeclarationRole::LocalRegion => {
            scopes.declaration_scope(role.owner)
        }
        _ => Ok(role.scope),
    }
}

fn declaration_visibility(
    topology: &FinalizedTopology,
    role: &ClassifiedRole,
    declaration_role: DeclarationRole,
) -> Result<Visibility, ResolutionCompilerFailure> {
    if declaration_role == DeclarationRole::Function {
        return Ok(Visibility::Always);
    }
    let coordinate = role.origin.coordinate;
    let byte = match declaration_role {
        DeclarationRole::NamedConst
        | DeclarationRole::ConstGeneric
        | DeclarationRole::Parameter
        | DeclarationRole::Let => node_end(topology, role.owner)?.value(),
        DeclarationRole::MatchBinder => {
            let list =
                ancestor_with_production(topology, role.owner, ProductionV0_15::FieldbindList)
                    .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
            node_end(topology, list)?.value()
        }
        _ => coordinate.end().value(),
    };
    Ok(Visibility::After {
        source: coordinate.source().ordinal(),
        byte,
    })
}

fn node_end(
    topology: &FinalizedTopology,
    node: NodeId,
) -> Result<ByteOffset, ResolutionCompilerFailure> {
    let record = topology
        .node(node)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    let FinalizedExtent::Source { end, .. } = record.extent else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    Ok(end)
}

fn ancestor_with_production(
    topology: &FinalizedTopology,
    mut node: NodeId,
    production: ProductionV0_15,
) -> Option<NodeId> {
    loop {
        let record = topology.node(node)?;
        if record.production == production {
            return Some(node);
        }
        node = record.parent?;
    }
}

fn is_visible(
    scopes: &ScopeBuild,
    declaration: &DeclarationMeta,
    use_scope: ScopeId,
    use_source: u32,
    use_byte: u64,
) -> bool {
    if !scopes.is_ancestor(declaration.scope, use_scope) {
        return false;
    }
    match declaration.visibility {
        Visibility::Always => true,
        Visibility::After { source, byte } => {
            use_source > source || (use_source == source && use_byte >= byte)
        }
    }
}

fn declaration_domain(class: DeclarationClass) -> Option<DeclarationDomain> {
    match class {
        DeclarationClass::Function
        | DeclarationClass::NamedConst
        | DeclarationClass::ConstGeneric
        | DeclarationClass::Value => Some(DeclarationDomain::LexicalIdentifier),
        DeclarationClass::GenericType | DeclarationClass::NominalType => {
            Some(DeclarationDomain::NominalType)
        }
        DeclarationClass::StructConstructor | DeclarationClass::EnumVariant => {
            Some(DeclarationDomain::Constructor)
        }
        DeclarationClass::Contract => Some(DeclarationDomain::Contract),
        DeclarationClass::Region => Some(DeclarationDomain::Region),
        DeclarationClass::Label => Some(DeclarationDomain::Label),
        DeclarationClass::OperationFamily => None,
    }
}
