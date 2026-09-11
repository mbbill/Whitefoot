//! [BLK-4] confinement and the stored-position reachability closure.
//!
//! A confined value can occupy a destination only when every region in its
//! complete type outlives that destination. Exclusive parameters, including
//! generic referents, use ordinary projected-effect kills [CALL-6, ENT-5].

use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::{Production, SemanticIssueKind, SemanticRule};

use super::super::model::{CheckedNominalKind, CheckedType, NominalId};
use super::{CheckStop, Checker};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [BLK-4] a stored position whose brand resolves to the entry heap's
    /// store region, in a unit whose entry selects no `command.heap` row.
    ///
    /// The region is minted before `main` in every unit whether or not the
    /// entry holds the provider [PROV-1], so this refusal is about
    /// reachability of a provider and not existence of a region: a unit with
    /// the whole-program fact `heap-unreachable` can form no value of such a
    /// type, and the declaration is refused where it is written rather than at
    /// every use.
    pub(super) fn reject_confined_type_without_store(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if self.unit_reaches_the_general_store()? {
            return Ok(());
        }
        if !self.reaches_the_entry_heap(ty)? {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Blk4,
            node,
            SemanticIssueKind::ConfinedTypeWithoutStore {
                mechanical_fix: "give this nominal a region parameter and confine the field to \
                     it, or let the entry receive the general store with `command.heap as heap: \
                     own Heap`",
            },
        )
    }

    /// Whether this unit's entry selects [FN-7]'s `heap` standard input, which
    /// is the one route by which a provider of the general store enters a
    /// program. The scan reads `input_label` nodes, which [FN-7] admits on the
    /// entry's own parameters alone, so it is the whole-program fact.
    fn unit_reaches_the_general_store(&self) -> Result<bool, CheckStop> {
        if let Some(known) = self.general_store_reachable.get() {
            return Ok(known);
        }
        let mut reachable = false;
        for label in self.nodes_with(Production::InputLabel)? {
            let [tail] = self.tree.direct_identifiers(label)?[..] else {
                continue;
            };
            if self.decoded(tail)? == "heap" {
                reachable = true;
                break;
            }
        }
        self.general_store_reachable.set(Some(reachable));
        Ok(reachable)
    }

    /// Whether this type is, or reaches at any depth, a run branded to the
    /// entry heap's store region.
    fn reaches_the_entry_heap(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        let mut pending = vec![ty];
        let mut visited: HashSet<NominalId> = HashSet::new();
        while let Some(current) = pending.pop() {
            match current {
                CheckedType::Vector { region, .. } if region.is_entry_heap_region() => {
                    return Ok(true);
                }
                CheckedType::Array { element, .. }
                | CheckedType::Vector { element, .. }
                | CheckedType::FixedVector { element, .. } => {
                    pending.push(self.element_type(element)?)
                }
                CheckedType::Buffer { element } => {
                    pending.push(element.ty());
                }
                CheckedType::Nominal(id) => {
                    if !visited.insert(id) {
                        continue;
                    }
                    match &self.nominal(id)?.kind {
                        CheckedNominalKind::Struct { fields } => {
                            pending.extend(fields.iter().map(|field| field.ty));
                        }
                        CheckedNominalKind::Enum { variants } => {
                            pending.extend(
                                variants.iter().flat_map(|variant| {
                                    variant.fields.iter().map(|field| field.ty)
                                }),
                            );
                        }
                        // S39 a cell branded to the entry heap is the same
                        // position a run branded to it is.
                        CheckedNominalKind::Box {
                            referent,
                            region: Some(region),
                            ..
                        } => {
                            if region.is_entry_heap_region() {
                                return Ok(true);
                            }
                            pending.push(*referent);
                        }
                        CheckedNominalKind::Box { referent, .. } => pending.push(*referent),
                        CheckedNominalKind::Arena { content, .. } => pending.push(*content),
                        CheckedNominalKind::ArenaStorage
                        | CheckedNominalKind::SystemResource { .. } => {}
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }
}
