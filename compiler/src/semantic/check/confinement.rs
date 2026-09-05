//! [BLK-4] confinement: the position closure and the `&uniq` parameter
//! refusal.
//!
//! A confined type is one whose complete type after substitution names a
//! region, and confinement is a set rather than a flag: a value may be moved,
//! returned, or bound only where every member of that set outlives the
//! destination. The half this module carries is the one [STOR-5] deferred —
//! the refusal of a `&uniq` parameter of a source-declared `fn` whose
//! referent reaches a run, a view, or a type parameter — together with the
//! reachability closure [PROV-4] states and [PROV-6]'s release graph reads.
//!
//! Nothing here reads a name or a signature shape: the closure is over
//! fields, enum variant payloads, run elements, and written type arguments,
//! and the verdict is one per declaration.

use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::{Production, SemanticIssueKind, SemanticRule};

use super::super::model::{CheckedNominalKind, CheckedType, NominalId};
use super::{CheckStop, Checker};

/// What a `&uniq` referent reached that [BLK-4] refuses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReachedSurface {
    /// One of the two runs [BLK-1], whose boundary operations move exactly
    /// the measures a caller would otherwise retain across the call [MSR-3].
    Run,
    /// A view [VIEW-1], whose origin set a stored or borrowed position would
    /// hide [PROV-3].
    View,
    /// A type parameter, which is opaque at the declaration where this
    /// verdict is reached, and which no [S37] bound excludes both of the
    /// above at: the runs are affine and `MutSlice` is affine, while `Slice`
    /// is copy, so every one of the three classes admits one of them.
    TypeParameter,
}

impl ReachedSurface {
    const fn phrase(self) -> &'static str {
        match self {
            Self::Run => "a container nominal",
            Self::View => "a loan-bearing type",
            Self::TypeParameter => "a type parameter no linearity bound excludes both at",
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [BLK-4] the `&uniq` parameter refusal of one source-declared `fn`.
    ///
    /// A provider parameter is the one `&uniq` this rule does not refuse
    /// [PROV-2]: no operation changes a provider's identity, only its
    /// measures, and every row that hands a provider's post-state back is a
    /// compiler-owned record whose relations are complete.
    ///
    /// The two legacy flat containers `array<T, N>` and `buffer<T>` are
    /// outside the refusal: each has one measure, fixed at its formation and
    /// unmovable by any operation, so a callee holding one `&uniq` changes no
    /// measure its caller retained. The two runs are refused because their
    /// four boundary operations [BLK-3] move exactly those measures. When the
    /// flat containers retire into the runs, the position retires with them.
    pub(super) fn check_unique_parameter_confinement(
        &self,
        mode: super::CheckedMode,
        ty: CheckedType,
        name: &str,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if !matches!(mode, super::CheckedMode::Unique(_)) {
            return Ok(());
        }
        let Some(reached) = self.reaches_confined_surface(ty)? else {
            return Ok(());
        };
        self.issue_node(
            SemanticRule::Blk4,
            node,
            SemanticIssueKind::UniqueParameterReachesContainer {
                parameter: name.to_owned(),
                reached: reached.phrase(),
                mechanical_fix: "take the run by value and return it, or take a view of it",
            },
        )
    }

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
                CheckedType::Vector { element, .. } | CheckedType::FixedVector { element, .. } => {
                    pending.push(element.ty())
                }
                CheckedType::Array { element, .. } | CheckedType::Buffer { element } => {
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
                        CheckedNominalKind::Box { referent } => pending.push(*referent),
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

    /// The [PROV-4] reachability closure over one type, answering with the
    /// first refused surface it reaches: through fields, enum variant
    /// payloads, run elements, and written type arguments, at any depth.
    fn reaches_confined_surface(
        &self,
        ty: CheckedType,
    ) -> Result<Option<ReachedSurface>, CheckStop> {
        let mut pending = vec![ty];
        let mut visited: HashSet<NominalId> = HashSet::new();
        while let Some(current) = pending.pop() {
            match current {
                CheckedType::FixedVector { element, .. } => {
                    let _ = element;
                    return Ok(Some(ReachedSurface::Run));
                }
                CheckedType::Vector { .. } => return Ok(Some(ReachedSurface::Run)),
                CheckedType::Slice { .. } => return Ok(Some(ReachedSurface::View)),
                CheckedType::Generic(_) => return Ok(Some(ReachedSurface::TypeParameter)),
                CheckedType::Array { element, .. } | CheckedType::Buffer { element } => {
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
                        CheckedNominalKind::Box { referent } => pending.push(*referent),
                        CheckedNominalKind::Arena { content, .. } => pending.push(*content),
                        CheckedNominalKind::ArenaStorage
                        | CheckedNominalKind::SystemResource { .. } => {}
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }
}
