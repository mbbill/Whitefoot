use std::collections::HashMap;

use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{ByteOffset, CanonicalSyntaxUnit, Production, SourceId};

use super::catalog::{PRELUDE_DECLARATIONS, system_declarations};
use super::scopes::ScopeBuild;
use super::{
    DeclarationClass, DeclarationDomain, DeclarationId, DeclarationRecord, DeclarationRole,
    DeferredUseRecord, DeferredUseRole, DependentDeclarationRecord, DependentDeclarationRole,
    LexicalUseRecord, LexicalUseRole, PostconditionCandidateRecord, PostconditionFieldRecord,
    PostconditionResolutionRecord, PostconditionSelectorClass, PostconditionSelectorUseRecord,
    PreludeDeclarationRecord, ResolutionCompilerFailure, ResolutionIssue, ResolutionOutcome,
    ResolvedSyntaxUnit, ScopeId, SourceOrigin, SystemDeclarationRecord,
};

mod admission;
mod inventory;
mod lookup;
mod roles;

use admission::check_clause_blocks;
use inventory::check_declaration_inventory;
use lookup::{resolve_uses, resolve_uses_deferred};
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
    Selector(SelectorRole),
    LexicalUse(LexicalUseRole),
    DeferredUse(DeferredUseRole),
    /// A DIAG-1 table-checked carrier: the `program_kind` IDENT and both
    /// IDENTs of an `input_label`. It declares nothing and enters no lexical
    /// name domain; its FN-7 kind-table judgment is an unimplemented compiler
    /// capability, so classification produces no retained record yet.
    TableChecked,
}

impl RawRoleKind {
    const fn class_ordinal(self) -> u8 {
        match self {
            Self::Declaration(_) | Self::DependentDeclaration(_) => 0,
            Self::Selector(_) => 1,
            Self::LexicalUse(_) => 2,
            Self::DeferredUse(_) => 3,
            Self::TableChecked => 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SelectorRole {
    PlainCandidate,
    VariantField,
    VariantCandidate,
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

#[derive(Clone)]
struct UseMeta {
    role: LexicalUseRole,
    spelling: String,
    owner: NodeId,
    origin: SourceOrigin,
    scope: ScopeId,
    owner_chain: Vec<NodeId>,
    function_owner: Option<NodeId>,
}

struct Tables {
    scopes: Vec<super::ScopeRecord>,
    prelude: Vec<PreludeDeclarationRecord>,
    system: Vec<SystemDeclarationRecord>,
    declarations: Vec<DeclarationRecord>,
    dependent_declarations: Vec<DependentDeclarationRecord>,
    lexical_uses: Vec<LexicalUseRecord>,
    deferred_uses: Vec<DeferredUseRecord>,
    postconditions: Vec<PostconditionResolutionRecord>,
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

/// Resolves every active-specification declaration and lexical use in canonical syntax.
#[must_use]
pub fn resolve<'classified, 'lexed, 'source>(
    syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
) -> ResolutionOutcome<'classified, 'lexed, 'source> {
    resolve_with_inventory(syntax, crate::Inventory::ACTIVE)
}

/// [`resolve`] against one named [SYS-2] inventory state.
///
/// `inventory` selects which prefix of the [SYS-2] tables this unit resolves
/// against; [`crate::Inventory::ACTIVE`] is the active specification's and is
/// what [`resolve`] passes outside a candidate path.
#[must_use]
pub fn resolve_with_inventory<'classified, 'lexed, 'source>(
    syntax: CanonicalSyntaxUnit<'classified, 'lexed, 'source>,
    inventory: crate::Inventory,
) -> ResolutionOutcome<'classified, 'lexed, 'source> {
    match build_tables(&syntax, inventory) {
        Ok(tables) => ResolutionOutcome::Complete(ResolvedSyntaxUnit {
            syntax,
            scopes: tables.scopes,
            prelude: tables.prelude,
            system: tables.system,
            declarations: tables.declarations,
            dependent_declarations: tables.dependent_declarations,
            lexical_uses: tables.lexical_uses,
            deferred_uses: tables.deferred_uses,
            postconditions: tables.postconditions,
            inventory,
        }),
        Err(BuildStop::Issue(issue)) => ResolutionOutcome::SourceIssue {
            syntax,
            issue: *issue,
        },
        Err(BuildStop::Compiler(failure)) => ResolutionOutcome::CompilerFailure { syntax, failure },
    }
}

fn build_tables(
    syntax: &CanonicalSyntaxUnit<'_, '_, '_>,
    inventory: crate::Inventory,
) -> Result<Tables, BuildStop> {
    let topology = &syntax.finalized.topology;
    let scopes = ScopeBuild::build(topology)?;
    // [DIAG-1] fixes this order: complete unit-wide FN-8 admission precedes
    // declaration inventory, and only complete inventory permits lexical
    // resolution.
    if let Some(issue) = check_clause_blocks(topology, &scopes)? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    // [SYS-3] admits the complete [SYS-2] inventory into every compilation
    // unit as the third declaration source [SYS-1]. Entry-form validation is
    // deliberately later and cannot change which system names exist.
    let system = system_declarations(inventory);
    let roles = classify_roles(syntax, &scopes)?;
    let mut declarations = Vec::new();
    let mut dependent_declarations = Vec::new();
    let mut deferred_uses = Vec::new();
    let mut declaration_metas = Vec::new();
    let mut declaration_by_role = vec![None; roles.len()];
    let mut uses = Vec::new();
    let mut postcondition_entry_uses = Vec::new();

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
            RawRoleKind::LexicalUse(use_role) => {
                let use_meta = UseMeta {
                    role: use_role,
                    spelling: role.spelling.clone(),
                    owner: role.owner,
                    origin: role.origin.clone(),
                    scope: role.scope,
                    owner_chain: role.owner_chain.clone(),
                    function_owner: function_owner(topology, role.owner),
                };
                if use_role != LexicalUseRole::EnsuresVariant
                    && ancestor_with_production(topology, role.owner, Production::EnsuresClause)
                        .is_some()
                {
                    postcondition_entry_uses.push(use_meta);
                } else {
                    uses.push(use_meta);
                }
            }
            RawRoleKind::DeferredUse(deferred_role) => deferred_uses.push(DeferredUseRecord {
                role: deferred_role,
                spelling: role.spelling.clone(),
                origin: role.origin.clone(),
            }),
            // Table-checked carriers await the FN-7 kind-table capability, and
            RawRoleKind::Selector(_) | RawRoleKind::TableChecked => {}
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
        &system,
    )? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    let lexical_uses = resolve_uses(
        &scopes,
        &declarations,
        &declaration_metas,
        &declaration_index,
        &uses,
        &system,
    )?;
    let postconditions = build_postcondition_records(
        topology,
        &scopes,
        &roles,
        &declarations,
        &declaration_metas,
        &declaration_index,
        &postcondition_entry_uses,
        &lexical_uses,
        &system,
    )?;
    Ok(Tables {
        scopes: scopes.records,
        prelude: PRELUDE_DECLARATIONS.to_vec(),
        system,
        declarations,
        dependent_declarations,
        lexical_uses,
        deferred_uses,
        postconditions,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_postcondition_records(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    roles: &[ClassifiedRole],
    declarations: &[DeclarationRecord],
    declaration_metas: &[DeclarationMeta],
    declaration_index: &DeclarationIndex,
    entry_uses: &[UseMeta],
    lexical_uses: &[LexicalUseRecord],
    system: &[SystemDeclarationRecord],
) -> Result<Vec<PostconditionResolutionRecord>, BuildStop> {
    let mut blocks = Vec::new();
    for (index, record) in topology.nodes.iter().enumerate() {
        if record.production == Production::EnsuresClause {
            blocks
                .push(NodeId::from_index(index).ok_or(ResolutionCompilerFailure::CounterOverflow)?);
        }
    }
    blocks.sort_by_key(|block| {
        let record = &topology.nodes[block.index()];
        let (source, start, end) = match record.extent {
            FinalizedExtent::Source { source, start, end } => {
                (source.ordinal(), start.value(), end.value())
            }
            FinalizedExtent::BundleRoot => (u32::MAX, u64::MAX, u64::MAX),
        };
        let path = scopes
            .path(*block)
            .map_or_else(|_| Vec::new(), |path| path.components().to_vec());
        (source, start, end, path)
    });

    let mut out = Vec::with_capacity(blocks.len());
    for block in blocks {
        let function = ancestor_with_production(topology, block, Production::FnDecl)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let result_binding = topology
            .node_children(function)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .iter()
            .copied()
            .find(|child| {
                topology
                    .node(*child)
                    .is_some_and(|record| record.production == Production::ResultBinding)
            })
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let route = topology
            .node_children(block)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .iter()
            .copied()
            .find(|child| {
                topology
                    .node(*child)
                    .is_some_and(|record| record.production == Production::ResultRoute)
            });
        let selector = route.unwrap_or(result_binding);
        let selector_path = scopes.path(selector)?;
        let selector_roles: Vec<_> = roles.iter().filter(|role| role.owner == selector).collect();
        let (class, plain_candidate, variant_target) = if route.is_none() {
            let [candidate] = selector_roles.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            };
            if !matches!(
                candidate.kind,
                RawRoleKind::Selector(SelectorRole::PlainCandidate)
            ) {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            }
            (
                PostconditionSelectorClass::Plain,
                Some(build_postcondition_candidate(
                    topology,
                    scopes,
                    candidate,
                    None,
                    block,
                    declarations,
                    declaration_metas,
                    declaration_index,
                    roles,
                )?),
                None,
            )
        } else {
            let [variant] = selector_roles.as_slice() else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            };
            if !matches!(
                variant.kind,
                RawRoleKind::LexicalUse(LexicalUseRole::EnsuresVariant)
            ) {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            }
            let target = lexical_uses
                .iter()
                .find(|usage| {
                    usage.role() == LexicalUseRole::EnsuresVariant
                        && usage.origin().node() == selector_path
                })
                .map(LexicalUseRecord::target)
                .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?;
            (PostconditionSelectorClass::Variant, None, Some(target))
        };

        let mut field_roles: Vec<_> = roles
            .iter()
            .filter(|role| {
                matches!(
                    role.kind,
                    RawRoleKind::Selector(SelectorRole::VariantField)
                        | RawRoleKind::Selector(SelectorRole::VariantCandidate)
                ) && ancestor_with_production(topology, role.owner, Production::EnsuresClause)
                    == Some(block)
            })
            .collect();
        field_roles.sort_by_key(|role| EventKey::from_origin(&role.origin));
        let mut fields = Vec::new();
        for pair in field_roles.chunks(2) {
            let [field, candidate] = pair else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            };
            if field.owner != candidate.owner
                || !matches!(
                    field.kind,
                    RawRoleKind::Selector(SelectorRole::VariantField)
                )
                || !matches!(
                    candidate.kind,
                    RawRoleKind::Selector(SelectorRole::VariantCandidate)
                )
            {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            }
            fields.push(PostconditionFieldRecord {
                spelling: field.spelling.clone(),
                origin: field.origin.clone(),
                candidate: build_postcondition_candidate(
                    topology,
                    scopes,
                    candidate,
                    Some(field.spelling.clone()),
                    block,
                    declarations,
                    declaration_metas,
                    declaration_index,
                    roles,
                )?,
            });
        }
        if class == PostconditionSelectorClass::Plain && !fields.is_empty() {
            return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
        }

        let candidate_spellings: Vec<&str> = plain_candidate
            .iter()
            .map(|candidate| candidate.spelling.as_str())
            .chain(fields.iter().map(|field| field.candidate.spelling.as_str()))
            .collect();
        let mut ordinary_entry_uses = Vec::new();
        let mut selector_uses = Vec::new();
        for use_record in entry_uses.iter().filter(|use_record| {
            ancestor_with_production(topology, use_record.owner, Production::EnsuresClause)
                == Some(block)
        }) {
            if use_record.role == LexicalUseRole::PlaceBase
                && candidate_spellings.contains(&use_record.spelling.as_str())
            {
                selector_uses.push(PostconditionSelectorUseRecord {
                    spelling: use_record.spelling.clone(),
                    origin: use_record.origin.clone(),
                });
            } else {
                ordinary_entry_uses.push(use_record.clone());
            }
        }
        let entry_inventory_issue = None;
        let (provisional_uses, entry_resolution_issue) = if ordinary_entry_uses.is_empty() {
            (Vec::new(), None)
        } else {
            let (resolved, issue) = resolve_uses_deferred(
                scopes,
                declarations,
                declaration_metas,
                declaration_index,
                &ordinary_entry_uses,
                system,
            )?;
            if issue.is_some() {
                (Vec::new(), issue)
            } else {
                (resolved, None)
            }
        };
        out.push(PostconditionResolutionRecord {
            function: scopes.path(function)?.clone(),
            block: scopes.path(block)?.clone(),
            selector: selector_path.clone(),
            class,
            plain_candidate,
            fields,
            variant_target,
            provisional_uses,
            selector_uses,
            entry_inventory_issue,
            entry_resolution_issue,
        });
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn build_postcondition_candidate(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    role: &ClassifiedRole,
    paired_field: Option<String>,
    block: NodeId,
    declarations: &[DeclarationRecord],
    metas: &[DeclarationMeta],
    index: &DeclarationIndex,
    roles: &[ClassifiedRole],
) -> Result<PostconditionCandidateRecord, ResolutionCompilerFailure> {
    let mut live_conflicts = Vec::new();
    for meta in index
        .with_spelling(&role.spelling)
        .iter()
        .filter_map(|candidate| metas.get(*candidate))
    {
        if meta.scope != ScopeId(0)
            && meta
                .owner
                .is_some_and(|owner| !role.owner_chain.contains(&owner))
        {
            continue;
        }
        if !meta.entries.iter().any(|class| {
            matches!(
                class,
                DeclarationClass::Function
                    | DeclarationClass::NamedConst
                    | DeclarationClass::ConstGeneric
                    | DeclarationClass::Value
            )
        }) || !is_visible(
            scopes,
            meta,
            role.scope,
            role.origin.coordinate.source().ordinal(),
            role.origin.coordinate.start().value(),
        ) {
            continue;
        }
        live_conflicts.push(declarations[meta.record_index].origin.clone());
    }
    live_conflicts.sort_by_key(EventKey::from_origin);
    let later_local_collision = metas
        .iter()
        .filter_map(|meta| {
            let declaration = declarations.get(meta.record_index)?;
            let classified = roles.get(meta.role_index)?;
            Some((declaration, classified.owner))
        })
        .filter(|(declaration, _)| {
            declaration.role == DeclarationRole::Let
                && declaration.spelling == role.spelling
                && EventKey::from_origin(&declaration.origin) > EventKey::from_origin(&role.origin)
        })
        .find(|(_, owner)| {
            ancestor_with_production(topology, *owner, Production::ContractBlock)
                == ancestor_with_production(topology, block, Production::ContractBlock)
        })
        .map(|(declaration, _)| declaration.origin.clone());
    Ok(PostconditionCandidateRecord {
        spelling: role.spelling.clone(),
        origin: role.origin.clone(),
        paired_field,
        live_conflicts,
        later_local_collision,
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
            Production::FnSig
                | Production::FnDecl
                | Production::StructDecl
                | Production::EnumDecl
                | Production::ContractDecl
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
        if matches!(record.production, Production::FnDecl | Production::FnSig) {
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
        DeclarationRole::Parameter
        | DeclarationRole::Let
        | DeclarationRole::MatchBinder
        | DeclarationRole::CountedBinder => {
            vec![DeclarationClass::Value]
        }
        DeclarationRole::LoopLabel => vec![DeclarationClass::Label],
        DeclarationRole::Invariant => vec![DeclarationClass::Invariant],
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
        | DeclarationRole::Let
        | DeclarationRole::CountedBinder
        | DeclarationRole::Invariant => node_end(topology, role.owner)?.value(),
        DeclarationRole::MatchBinder => {
            let list = ancestor_with_production(topology, role.owner, Production::FieldbindList)
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
        DeclarationClass::Invariant => Some(DeclarationDomain::Invariant),
        DeclarationClass::OperationFamily => None,
    }
}
