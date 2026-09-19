use std::collections::HashMap;

use crate::syntax::{FinalizedExtent, FinalizedTopology, NodeId};
use crate::{ByteOffset, CanonicalSyntaxUnit, Production, SourceId};

use super::catalog::PRELUDE_DECLARATIONS;
use super::scopes::ScopeBuild;
use super::{
    DeclarationClass, DeclarationDomain, DeclarationId, DeclarationRecord, DeclarationRole,
    DeferredUseRecord, DeferredUseRole, DependentDeclarationRecord, DependentDeclarationRole,
    LexicalUseRecord, LexicalUseRole, PostconditionCandidateRecord, PostconditionFieldRecord,
    PostconditionResolutionRecord, PostconditionSelectorClass, PostconditionSelectorUseRecord,
    PreludeDeclarationRecord, ResolutionCompilerFailure, ResolutionIssue, ResolutionIssueKind,
    ResolutionOutcome, ResolvedSyntaxUnit, ScopeId, SourceOrigin,
};

mod admission;
mod inventory;
mod lookup;
mod prelude;
mod roles;

use admission::{check_clause_blocks, check_heap_declaration};
use inventory::check_declaration_inventory;
use lookup::resolve_uses_deferred;
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
}

impl RawRoleKind {
    const fn class_ordinal(self) -> u8 {
        match self {
            Self::Declaration(_) | Self::DependentDeclaration(_) => 0,
            Self::Selector(_) => 1,
            Self::LexicalUse(_) => 2,
            Self::DeferredUse(_) => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SelectorRole {
    PlainCandidate,
    VariantField,
    VariantCandidate,
    /// The optional ordinal binder of a route, `when b is V(f: r):`
    /// [GRAM-2, CALL-4]. It names a declared result and declares nothing.
    ResultOrdinal,
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
    match build_tables(&syntax) {
        Ok(tables) => ResolutionOutcome::Complete(ResolvedSyntaxUnit {
            syntax,
            scopes: tables.scopes,
            prelude: tables.prelude,
            declarations: tables.declarations,
            dependent_declarations: tables.dependent_declarations,
            lexical_uses: tables.lexical_uses,
            deferred_uses: tables.deferred_uses,
            postconditions: tables.postconditions,
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
    let scopes = ScopeBuild::build(topology, syntax.finalized.parsed.classified.source_bundle())?;
    // [DIAG-1] fixes this order: complete unit-wide FN-8 admission precedes
    // declaration inventory, and only complete inventory permits lexical
    // resolution.
    if let Some(issue) = check_clause_blocks(topology, &scopes)? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    // [GRAM-2]'s whole-unit heap-declaration position rule. The parser is
    // table-driven and has no unit-level position judgment, so this is the
    // first stage that can make it.
    if let Some(issue) = check_heap_declaration(topology, &scopes)? {
        return Err(BuildStop::Issue(Box::new(issue)));
    }
    let roles = classify_roles(syntax, &scopes)?;
    let prelude = prelude::PreludeInventory::build(syntax, &roles)?;
    // [SET-1] "A `set` whose target name resolves to nothing declares nothing
    // and is a hard error citing SET-1 at that `place`." The v0.59 promotion
    // loop that turned such a target into a `let` declaration is gone with
    // [LIV-2], and with it resolution's only re-entrant path; the shape
    // predicate survives only to re-attribute the ordinary unresolved-name
    // rejection to the rule and location SET-1 states.
    let bare_set_targets = bare_set_target_roles(topology, &roles)?;
    {
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
                    // A named formal's members have stable function-parameter
                    // identities, but FN-3 introduces no unqualified lexical
                    // names. Only a raw function binder enters that domain.
                    // Member distinctness is the formal table's FN-3 judgment.
                    let grouped_member = declaration_role == DeclarationRole::FunctionParameter
                        && role
                            .owner_chain
                            .get(1)
                            .and_then(|owner| topology.node(*owner))
                            .is_some_and(|record| record.production == Production::FormalDecl);
                    let mut entries = if grouped_member {
                        Vec::new()
                    } else {
                        declaration_classes(declaration_role)
                    };
                    if declaration_role == DeclarationRole::Struct
                        && syntax
                            .finalized
                            .parsed
                            .classified
                            .source_bundle()
                            .file(role.origin.coordinate.source())
                            .is_some_and(|file| {
                                file.prelude() == Some(crate::source::PreludeSource::Opaque)
                            })
                    {
                        entries.retain(|class| *class != DeclarationClass::StructConstructor);
                    }
                    let record_index = declarations.len();
                    declarations.push(DeclarationRecord {
                        id,
                        role: declaration_role,
                        spelling: role.spelling.clone(),
                        origin: role.origin.clone(),
                        scope: declaration_scope(role, declaration_role, &scopes)?,
                        classes: entries.clone(),
                        diagnostic_origins: prelude.roles[role_index].clone(),
                    });
                    declaration_by_role[role_index] = Some(record_index);
                    declaration_metas.push(DeclarationMeta {
                        role_index,
                        record_index,
                        scope: declarations[record_index].scope,
                        owner: if declaration_role == DeclarationRole::FunctionParameter {
                            // The callable binder belongs to the receiving
                            // generic declaration; its own parameters and
                            // regions remain owned by the nested FnSig.
                            Some(
                                *role
                                    .owner_chain
                                    .get(1)
                                    .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?,
                            )
                        } else {
                            role.owner_chain.first().copied()
                        },
                        visibility: if matches!(
                            declaration_role,
                            DeclarationRole::Function
                                | DeclarationRole::Struct
                                | DeclarationRole::Enum
                                | DeclarationRole::Variant
                        ) && syntax
                            .finalized
                            .parsed
                            .classified
                            .source_bundle()
                            .file(role.origin.coordinate.source())
                            .is_some_and(|file| file.prelude().is_some())
                        {
                            Visibility::Always
                        } else {
                            declaration_visibility(topology, role, declaration_role)?
                        },
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
                RawRoleKind::Selector(_) => {}
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
            &prelude.builtins,
        )? {
            return Err(BuildStop::Issue(Box::new(issue)));
        }
        let (lexical_uses, unresolved) = resolve_uses_deferred(
            &scopes,
            &declarations,
            &declaration_metas,
            &declaration_index,
            &uses,
        )?;
        if let Some(issue) = unresolved {
            return Err(BuildStop::Issue(Box::new(set_target_attribution(
                topology,
                &scopes,
                &roles,
                &bare_set_targets,
                issue,
            )?)));
        }
        let postconditions = build_postcondition_records(
            topology,
            &scopes,
            &roles,
            &declarations,
            &declaration_metas,
            &declaration_index,
            &postcondition_entry_uses,
            &lexical_uses,
        )?;
        return Ok(Tables {
            scopes: scopes.records,
            prelude: prelude.records,

            declarations,
            dependent_declarations,
            lexical_uses,
            deferred_uses,
            postconditions,
        });
    }
}

/// [SET-1, GRAM-4, GRAM-5] every `pbase` role that is a bare `set` target: the
/// base of a target `place` of a `set_stmt`, written with no `psuffix` and no
/// `deref`.
///
/// Each such role index maps to its `place` node, which is the location
/// [SET-1] states for the rejection when the target name resolves to nothing.
fn bare_set_target_roles(
    topology: &FinalizedTopology,
    roles: &[ClassifiedRole],
) -> Result<HashMap<usize, NodeId>, ResolutionCompilerFailure> {
    let mut candidates = HashMap::new();
    for (index, role) in roles.iter().enumerate() {
        if !matches!(
            role.kind,
            RawRoleKind::LexicalUse(super::LexicalUseRole::PlaceBase)
        ) {
            continue;
        }
        let pbase = role.owner;
        let record = topology
            .node(pbase)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        if record.production != Production::Pbase || record.terminal_count != 1 {
            continue;
        }
        // `pbase := IDENT | "deref" "(" place ")"`: a `deref` base owns a
        // nested `place` child, and this role's spelling would then be that
        // inner base's, so a base with any child is never a bare target.
        if topology
            .node_children(pbase)
            .is_some_and(|children| !children.is_empty())
        {
            continue;
        }
        let Some(place) = record.parent else {
            continue;
        };
        let place_record = topology
            .node(place)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        if place_record.production != Production::Place {
            continue;
        }
        // `place := pbase psuffix*`: a projected or subscripted target selects
        // one component of a value that must already exist, so it declares
        // nothing.
        if topology.node_children(place).is_some_and(|children| {
            children.iter().any(|child| {
                topology
                    .node(*child)
                    .is_some_and(|record| record.production == Production::Psuffix)
            })
        }) {
            continue;
        }
        let Some(statement) = place_record.parent else {
            continue;
        };
        if topology
            .node(statement)
            .is_some_and(|record| record.production == Production::SetStmt)
        {
            candidates.insert(index, place);
        }
    }
    Ok(candidates)
}

/// [SET-1] re-attributes an unresolved bare `set` target.
///
/// [DIAG-1]'s role table sends an unresolved `pbase` IDENT to [TYPE-5] at that
/// IDENT. [SET-1] states its own rule and its own location for the one case it
/// owns — a `set` target name that resolves to nothing — and a rule's own
/// stated location wins, so the issue is rebuilt at the complete `place`.
/// Every other unresolved use passes through unchanged.
fn set_target_attribution(
    topology: &FinalizedTopology,
    scopes: &ScopeBuild,
    roles: &[ClassifiedRole],
    bare_set_targets: &HashMap<usize, NodeId>,
    issue: ResolutionIssue,
) -> Result<ResolutionIssue, ResolutionCompilerFailure> {
    if !matches!(issue.kind, ResolutionIssueKind::UnresolvedUse { .. }) {
        return Ok(issue);
    }
    let Some(place) = bare_set_targets.iter().find_map(|(index, place)| {
        roles
            .get(*index)
            .is_some_and(|role| role.origin == issue.origin)
            .then_some(*place)
    }) else {
        return Ok(issue);
    };
    let record = topology
        .node(place)
        .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
    let FinalizedExtent::Source { source, start, end } = record.extent else {
        return Err(ResolutionCompilerFailure::InvalidCanonicalTree);
    };
    Ok(ResolutionIssue {
        rule: super::ResolutionRule::Set1,
        origin: SourceOrigin {
            node: scopes.path(place)?.clone(),
            coordinate: crate::SyntaxCoordinate::new(source, start, end),
            role_ordinal: 0,
            subtoken_ordinal: 0,
        },
        kind: issue.kind,
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
        let function = function_owner(topology, block)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        // [GRAM-2] the declaration writes one result or an ordered result
        // list; every ordinal's binder is a candidate a clause may name
        // [CALL-4].
        let result_bindings = topology
            .node_children(function)
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?
            .iter()
            .copied()
            .filter(|child| {
                topology
                    .node(*child)
                    .is_some_and(|record| record.production == Production::ResultBinding)
            })
            .collect::<Vec<_>>();
        let result_binding = result_bindings
            .first()
            .copied()
            .ok_or(ResolutionCompilerFailure::InvalidCanonicalTree)?;
        let mut result_binders = Vec::with_capacity(result_bindings.len());
        for binding in &result_bindings {
            let [candidate] = roles
                .iter()
                .filter(|role| role.owner == *binding)
                .collect::<Vec<_>>()[..]
            else {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            };
            if !matches!(
                candidate.kind,
                RawRoleKind::Selector(SelectorRole::PlainCandidate)
            ) {
                return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
            }
            result_binders.push(build_postcondition_candidate(
                topology,
                scopes,
                candidate,
                None,
                block,
                declarations,
                declaration_metas,
                declaration_index,
                roles,
            )?);
        }
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
        // An unrouted clause's selector node is the first result binder, whose
        // candidate role is one of `result_binders` above; a routed clause's
        // selector node is the route, which carries its variant and its
        // optional ordinal binder [CALL-4].
        let (class, route_ordinal, variant_target) = if route.is_none() {
            (PostconditionSelectorClass::Plain, None, None)
        } else {
            let (route_ordinal, variant) = match selector_roles.as_slice() {
                [variant] => (None, *variant),
                [ordinal, variant] => {
                    if !matches!(
                        ordinal.kind,
                        RawRoleKind::Selector(SelectorRole::ResultOrdinal)
                    ) {
                        return Err(ResolutionCompilerFailure::InvalidRoleShape.into());
                    }
                    (Some(ordinal.spelling.clone()), *variant)
                }
                _ => return Err(ResolutionCompilerFailure::InvalidRoleShape.into()),
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
            (
                PostconditionSelectorClass::Variant,
                route_ordinal,
                Some(target),
            )
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

        let candidate_spellings: Vec<&str> = result_binders
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
            result_binders,
            route_ordinal,
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
        if !scopes.is_unit_scope(meta.scope)
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
                | Production::FormalDecl
                | Production::ActualDecl
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
        DeclarationRole::FunctionParameter => vec![DeclarationClass::FunctionParameter],
        DeclarationRole::Struct => vec![
            DeclarationClass::NominalType,
            DeclarationClass::StructConstructor,
        ],
        DeclarationRole::Enum => vec![DeclarationClass::NominalType],
        DeclarationRole::Variant => vec![DeclarationClass::EnumVariant],
        DeclarationRole::Formal => vec![DeclarationClass::Formal],
        DeclarationRole::Actual => vec![DeclarationClass::Actual],
        DeclarationRole::NamedConst => vec![DeclarationClass::NamedConst],
        DeclarationRole::GenericType => vec![DeclarationClass::GenericType],
        DeclarationRole::ConstGeneric => vec![DeclarationClass::ConstGeneric],
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
        DeclarationRole::Variant => scopes.node_scope(
            *role
                .owner_chain
                .first()
                .ok_or(ResolutionCompilerFailure::InvalidRoleShape)?,
        ),
        DeclarationRole::LoopLabel => scopes.declaration_scope(role.owner),
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
        | DeclarationRole::FunctionParameter
        | DeclarationRole::Parameter
        | DeclarationRole::CountedBinder
        | DeclarationRole::Invariant => node_end(topology, role.owner)?.value(),
        // [PROV-6, GRAM-4] a destructuring consume's binder is owned by its
        // `fieldbind`, and the whole statement is where it becomes visible,
        // exactly as an ordinary `let` binder's own statement is. A `set`
        // target declares nothing any more [SET-1], so `let_stmt` is the only
        // statement a `Let` declaration can be owned by.
        DeclarationRole::Let => {
            let owner = ancestor_with_production(topology, role.owner, Production::LetStmt)
                .unwrap_or(role.owner);
            node_end(topology, owner)?.value()
        }
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
        | DeclarationClass::FunctionParameter
        | DeclarationClass::NamedConst
        | DeclarationClass::ConstGeneric
        | DeclarationClass::Value => Some(DeclarationDomain::LexicalIdentifier),
        DeclarationClass::GenericType
        | DeclarationClass::NominalType
        | DeclarationClass::Formal
        | DeclarationClass::Actual => Some(DeclarationDomain::NominalType),
        DeclarationClass::StructConstructor | DeclarationClass::EnumVariant => {
            Some(DeclarationDomain::Constructor)
        }
        DeclarationClass::NumericBound => Some(DeclarationDomain::NumericBound),
        DeclarationClass::Label => Some(DeclarationDomain::Label),
        DeclarationClass::Invariant => Some(DeclarationDomain::Invariant),
        DeclarationClass::OperationFamily => None,
    }
}
