//! [MOD-8] What one module verdict reads of other modules: the declarations
//! of their interface records its check reached, each named by module, role
//! and spelling and identified by a digest of its meaning.
//!
//! A module's check reads every interface record of its dependency closure,
//! and it needs each of them to hold its own judgments, since a defect in
//! any of them rejects the check. Beyond that, it reads only the
//! declarations its own records name and, transitively, the declarations
//! their items name: a signature's types, a type's fields, a contract's
//! constants. An edit to an interface that changes none of those, such as a
//! reworded `doc` entry or an unused declaration's body of text, leaves the
//! verdict as it was, and the verdict's record says so without the check
//! running again [MOD-8].
//!
//! A declaration's digest is the canonical text of its item with every
//! `doc` string emptied, preceded by its record's alias header, whose
//! bindings give the item's names their targets [MOD-4]. The text is
//! canonical [FORM-2], so a digest moves with any written change to the
//! item except a documentation string, and with any change to an alias.

use std::collections::{BTreeMap, BTreeSet};

use super::{CompilerLimits, with_canonical_syntax};
use crate::semantic::CheckedProgram;
use crate::source::SourceBundle;
use crate::syntax::terminal::TerminalPredicate;
use crate::syntax::{FinalizedExtent, NodeId};
use crate::{DeclarationRole, Production, ResolvedTarget, SourceInput};

/// One declaration by its item's role and spelling, as a record declares it.
pub(super) type ItemName = (String, String);

/// The role name of an item's declaration production.
const fn item_role(production: Production) -> Option<&'static str> {
    Some(match production {
        Production::FnDecl => "fn",
        Production::StructDecl => "struct",
        Production::EnumDecl => "enum",
        Production::InterfaceDecl => "interface",
        Production::BindingDecl => "binding",
        Production::ConstDecl => "const",
        _ => return None,
    })
}

/// The role name of a declaration that heads an item.
const fn declaration_role(role: DeclarationRole) -> Option<&'static str> {
    Some(match role {
        DeclarationRole::Function => "fn",
        DeclarationRole::Struct => "struct",
        DeclarationRole::Enum => "enum",
        DeclarationRole::Interface => "interface",
        DeclarationRole::Binding => "binding",
        DeclarationRole::NamedConst => "const",
        _ => return None,
    })
}

/// The digest of each declaration of one record, by role and spelling.
pub(super) type Digests = BTreeMap<ItemName, [u8; 32]>;

thread_local! {
    /// [`declaration_digests`] by the SHA-256 of the record's bytes: a pure
    /// function of those bytes, which one invocation's checks ask again for
    /// each module whose closure holds the record.
    static DIGESTS: std::cell::RefCell<std::collections::HashMap<[u8; 32], Option<Digests>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// The digest of every top-level declaration one record writes, or `None`
/// when the record does not pass the syntax stages, which no verdict that
/// read it can then stand on.
pub(super) fn declaration_digests(bytes: &[u8], limits: CompilerLimits) -> Option<Digests> {
    let key = crate::spec::sha256::digest(bytes);
    if let Some(known) = DIGESTS.with(|digests| digests.borrow().get(&key).cloned()) {
        return known;
    }
    let digests = record_digests(bytes, limits);
    DIGESTS.with(|known| known.borrow_mut().insert(key, digests.clone()));
    digests
}

fn record_digests(bytes: &[u8], limits: CompilerLimits) -> Option<Digests> {
    let input = [SourceInput::new("interface.wf", bytes)];
    let bundle = SourceBundle::with_limits(&input, limits.source).ok()?;
    with_canonical_syntax(&bundle, limits, false, |canonical| {
        let topology = &canonical.finalized.topology;
        let tokens = canonical.classified_bundle().tokens();
        let mut direct = vec![Vec::new(); topology.nodes.len()];
        for (index, terminal) in topology.terminals.iter().enumerate() {
            if let Some(owner) = terminal.owner
                && let Some(slot) = direct.get_mut(owner.index())
            {
                slot.push(index);
            }
        }
        let extent = |node: NodeId| match topology.node(node).map(|record| record.extent) {
            Some(FinalizedExtent::Source { start, end, .. }) => Some((
                usize::try_from(start.value()).ok()?,
                usize::try_from(end.value()).ok()?,
            )),
            _ => None,
        };
        let mut header = Vec::new();
        let mut items = Vec::new();
        for item in topology.node_children(topology.root).unwrap_or(&[]) {
            let Some(&declaration) = topology.node_children(*item).and_then(<[NodeId]>::first)
            else {
                continue;
            };
            let Some((start, end)) = extent(*item) else {
                continue;
            };
            let production = topology.node(declaration).map(|record| record.production);
            let Some(role) = production.and_then(item_role) else {
                // An alias or heap declaration binds the record's names.
                header.extend_from_slice(bytes.get(start..end).unwrap_or_default());
                header.push(b'\n');
                continue;
            };
            let spelling = direct
                .get(declaration.index())
                .into_iter()
                .flatten()
                .find_map(|terminal| {
                    let token = tokens.get(*terminal)?;
                    (token.terminals().contains(TerminalPredicate::Identifier)
                        || token
                            .terminals()
                            .contains(TerminalPredicate::TypeIdentifier))
                    .then(|| std::str::from_utf8(token.token().span().bytes()).ok())
                    .flatten()
                    .map(str::to_owned)
                });
            let Some(spelling) = spelling else {
                continue;
            };
            // Every `doc` entry of the item, whose string no judgment reads.
            let mut docs = Vec::new();
            let mut pending = vec![*item];
            while let Some(node) = pending.pop() {
                let children = topology.node_children(node).unwrap_or(&[]);
                for child in children {
                    if topology.node(*child).map(|record| record.production)
                        == Some(Production::Doc)
                    {
                        if let Some(range) = extent(*child) {
                            docs.push(range);
                        }
                    } else {
                        pending.push(*child);
                    }
                }
            }
            docs.sort_unstable();
            let mut text = Vec::with_capacity(end - start);
            let mut at = start;
            for (doc_start, doc_end) in docs {
                text.extend_from_slice(bytes.get(at..doc_start).unwrap_or_default());
                text.extend_from_slice(b"doc \"\";");
                at = doc_end;
            }
            text.extend_from_slice(bytes.get(at..end).unwrap_or_default());
            items.push(((role.to_owned(), spelling), text));
        }
        let mut digests = BTreeMap::new();
        for (name, text) in items {
            let mut material = b"declaration 1\nheader\n".to_vec();
            material.extend_from_slice(&header);
            material.extend_from_slice(b"item\n");
            material.extend_from_slice(&text);
            digests.insert(name, crate::spec::sha256::digest(&material));
        }
        Ok(digests)
    })
    .ok()
}

/// The declarations of other modules that one check of `target` reached:
/// every declaration its own records name, and every declaration the items
/// of reached declarations name, by module, role and spelling. PRE-1
/// declarations are the compiler's own and belong to no module.
pub(super) fn read_declarations(
    checked: &CheckedProgram<'_, '_, '_>,
    target: crate::ModuleId,
) -> Option<BTreeSet<(crate::ModuleId, ItemName)>> {
    let resolved = &checked._resolved;
    let syntax = resolved.syntax();
    let topology = &syntax.finalized.topology;
    let bundle = syntax.classified_bundle().source_bundle();
    let items = topology.node_children(topology.root)?;
    // Each item's module, when a module's writer record holds it.
    let modules = items
        .iter()
        .map(
            |item| match topology.node(*item).map(|record| record.extent) {
                Some(FinalizedExtent::Source { source, .. }) => bundle
                    .file(source)
                    .filter(|file| file.prelude().is_none())
                    .map(crate::source::SourceFile::module),
                _ => None,
            },
        )
        .collect::<Vec<_>>();
    // Each item's heading declaration.
    let mut heads: Vec<Option<ItemName>> = vec![None; items.len()];
    for declaration in resolved.declarations() {
        let Some(role) = declaration_role(declaration.role()) else {
            continue;
        };
        let Some(&item) = declaration.origin().node().components().first() else {
            continue;
        };
        if let Some(slot) = heads.get_mut(item as usize)
            && slot.is_none()
        {
            *slot = Some((role.to_owned(), declaration.spelling().to_owned()));
        }
    }
    // The items each item's uses name.
    let mut named = vec![BTreeSet::new(); items.len()];
    let uses = resolved.lexical_uses().iter().chain(
        resolved
            .postconditions()
            .iter()
            .flat_map(|record| &record.provisional_uses),
    );
    for record in uses {
        let ResolvedTarget::Source { declaration, .. } = record.target() else {
            continue;
        };
        let Some(&from) = record.origin().node().components().first() else {
            continue;
        };
        let to = *resolved
            .declaration(declaration)?
            .origin()
            .node()
            .components()
            .first()?;
        if from != to {
            named.get_mut(from as usize)?.insert(to);
        }
    }
    let mut reached = BTreeSet::new();
    let mut visited = vec![false; items.len()];
    let mut pending = (0..items.len())
        .filter(|item| modules.get(*item).copied().flatten() == Some(target))
        .collect::<Vec<_>>();
    for item in &pending {
        visited[*item] = true;
    }
    while let Some(item) = pending.pop() {
        for next in named.get(item)?.iter().map(|next| *next as usize) {
            if std::mem::replace(visited.get_mut(next)?, true) {
                continue;
            }
            // A PRE-1 item is the compiler's own; its meaning is the
            // compiler's identity, which scopes every record.
            let Some(module) = modules.get(next).copied().flatten() else {
                continue;
            };
            reached.insert((module, heads.get(next)?.clone()?));
            pending.push(next);
        }
    }
    Some(reached)
}
