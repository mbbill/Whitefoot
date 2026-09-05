//! [PROV-6] the release graph, the linearity predicate, the `linear`
//! modifier, and the two statements that read them.
//!
//! Everything here is derived from a type and a scope, never from a name, a
//! signature, or a statement's shape. The release graph is the one object the
//! compiler-derived release and `dispose p;` both walk; the linearity
//! predicate is that graph read against what the scope holds.

use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::{Production, SemanticIssueKind, SemanticRule, TerminalPredicate};

use super::super::model::{CheckedNominalKind, CheckedType, NominalId};
use super::{CheckStop, Checker};

/// [PROV-6, S37] the linearity class a declaration writes as a bound, and the
/// class this compiler computes for a type in a scope.
///
/// The three form the strict chain `copy < affine < linear`, ordered by what a
/// body may do with a value of the class: `copy` may duplicate it, use it bare
/// and drop it; `affine` may `move` it at most once and may drop it; `linear`
/// must consume it exactly once and may never drop it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::semantic) enum LinearityClass {
    /// Duplicated on use and dropped without any action [OWN-1].
    Copy,
    /// Reclaimed without spending a capability and unmarked.
    Affine,
    /// Marked by the modifier, or reclaimed by spending a capability this
    /// scope does not hold.
    Linear,
}

impl LinearityClass {
    pub(in crate::semantic) const fn spelling(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Affine => "affine",
            Self::Linear => "linear",
        }
    }

    /// [PROV-6, S37] satisfaction is the chain read left to right: an argument
    /// of class `self` instantiates the bound `bound` exactly when
    /// `self <= bound`. The reverse direction is the rule's hard error.
    pub(in crate::semantic) fn satisfies(self, bound: Self) -> bool {
        self <= bound
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [PROV-6] whether this type's own reclamation is a release to a store
    /// whose provider is a value.
    ///
    /// In this version the heap-backed types are exactly those types, and
    /// the ambient heap is their sole provider. Nothing here reads a name:
    /// the storage class [STOR-1] selects the answer.
    pub(in crate::semantic) fn is_capability_released(
        &self,
        ty: CheckedType,
    ) -> Result<bool, CheckStop> {
        Ok(match ty {
            CheckedType::Buffer { .. } => true,
            CheckedType::Nominal(id) => {
                matches!(self.nominal(id)?.kind, CheckedNominalKind::Box { .. })
            }
            _ => false,
        })
    }

    /// [PROV-6, STOR-5] whether this type is or reaches a view, which owns
    /// nothing and contributes no release-graph node.
    pub(in crate::semantic) fn is_loan_bearing(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        let mut visited = HashSet::new();
        self.loan_bearing_with(ty, &mut visited)
    }

    fn loan_bearing_with(
        &self,
        ty: CheckedType,
        visited: &mut HashSet<NominalId>,
    ) -> Result<bool, CheckStop> {
        match ty {
            CheckedType::Slice { .. } => Ok(true),
            CheckedType::Array { element, .. } | CheckedType::Buffer { element } => {
                self.loan_bearing_with(element.ty(), visited)
            }
            CheckedType::Nominal(id) => {
                if !visited.insert(id) {
                    return Ok(false);
                }
                for component in self.owned_components(id)? {
                    if self.loan_bearing_with(component, visited)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// The types one nominal owns directly [PROV-6]: its fields, its enum
    /// variant payloads, its `box` referent, and its `arena` content.
    fn owned_components(&self, id: NominalId) -> Result<Vec<CheckedType>, CheckStop> {
        Ok(match &self.nominal(id)?.kind {
            CheckedNominalKind::Struct { fields } => fields.iter().map(|field| field.ty).collect(),
            CheckedNominalKind::Enum { variants } => variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect(),
            CheckedNominalKind::Box { referent } => vec![*referent],
            CheckedNominalKind::Arena { content, .. } => vec![*content],
            CheckedNominalKind::ArenaStorage | CheckedNominalKind::SystemResource { .. } => {
                Vec::new()
            }
        })
    }

    /// [PROV-6] the nodes of this type's release graph, each visited once.
    ///
    /// A loan-bearing value contributes no node, which is why a view can
    /// neither be disposed nor make its holder linear. The visited set is
    /// what makes the walk terminate on a graph with a cycle, whose refusal
    /// is DEFERRED.
    pub(in crate::semantic) fn release_graph_nodes(
        &self,
        ty: CheckedType,
    ) -> Result<Vec<CheckedType>, CheckStop> {
        let mut nodes = Vec::new();
        let mut seen_nominals = HashSet::new();
        let mut pending = vec![ty];
        while let Some(current) = pending.pop() {
            if self.is_loan_bearing(current)? {
                continue;
            }
            if let CheckedType::Nominal(id) = current
                && !seen_nominals.insert(id)
            {
                continue;
            }
            if !nodes.contains(&current) {
                nodes.push(current);
            }
            match current {
                CheckedType::Array { element, .. } | CheckedType::Buffer { element } => {
                    pending.push(element.ty());
                }
                CheckedType::Nominal(id) => pending.extend(self.owned_components(id)?),
                _ => {}
            }
        }
        Ok(nodes)
    }

    /// [PROV-6] whether any node of this type's release graph carries the
    /// `linear` modifier, this type's own node included.
    pub(in crate::semantic) fn owns_modifier_linear_node(
        &self,
        ty: CheckedType,
    ) -> Result<Option<NominalId>, CheckStop> {
        for node in self.release_graph_nodes(ty)? {
            if let CheckedType::Nominal(id) = node
                && self.nominal(id)?.linear
            {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// [PROV-6] the linearity class of a value of this type in the scope now
    /// being checked.
    ///
    /// The copy half is [OWN-1]'s own classification, which [PROV-6] refines
    /// and replaces nothing of: a copy value is never affine and never linear.
    /// The capability half is stated over the provider a scope holds. In this
    /// version the only provider is the ambient heap, which every scope
    /// holds, so the capability half makes nothing linear here and the
    /// modifier is the whole of the remaining answer. The version that makes a
    /// provider a written value is where the second half first fires.
    ///
    /// A type parameter standing for itself at a symbolic instance [FN-2] has
    /// exactly the class its written bound names [S37]: the body is checked
    /// once under that bound and the bound is what the body was written for.
    pub(in crate::semantic) fn linearity_class(
        &self,
        ty: CheckedType,
    ) -> Result<LinearityClass, CheckStop> {
        if let CheckedType::Generic(declaration) = ty {
            return self.generic_parameter_class(declaration);
        }
        if self.is_copy_type(ty)? {
            return Ok(LinearityClass::Copy);
        }
        Ok(if self.owns_modifier_linear_node(ty)?.is_some() {
            LinearityClass::Linear
        } else {
            LinearityClass::Affine
        })
    }

    /// [PROV-6] a value linear in this scope may not reach a scope exit by a
    /// compiler-derived release: in this scope no derived release exists to
    /// carry it. The two routes that remain are the whole move and the whole
    /// destructuring.
    pub(in crate::semantic) fn reject_linear_value_not_consumed(
        &self,
        ty: CheckedType,
        name: &str,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        // [S37] a `linear`-bounded type parameter is linear at its symbolic
        // instance: the body is checked once under its written bound, and
        // under `linear` it must consume the value exactly once and may never
        // drop it. The obligation names the bound rather than a nominal,
        // because at the symbolic instance no nominal carries it.
        let marked = match self.owns_modifier_linear_node(ty)? {
            Some(marked) => self.nominal(marked)?.name.clone(),
            None => {
                if self.linearity_class(ty)? != LinearityClass::Linear {
                    return Ok(());
                }
                let named = match ty {
                    CheckedType::Generic(declaration) => self.declaration_spelling(declaration)?,
                    other => self.checked_type_name(other)?,
                };
                format!("the `linear` bound written on {named}")
            }
        };
        self.issue_node::<()>(
            SemanticRule::Prov6,
            node,
            SemanticIssueKind::LinearValueNotConsumed {
                binding: name.to_owned(),
                obligation: marked,
                mechanical_fix: "move the value out whole, or take it apart with \
                     let N(f: a, ...) = move v;",
            },
        )?;
        Ok(())
    }

    /// [PROV-6] a consume of a proper sub-place of a value linear in this
    /// scope, where the same statement's commit does not reinitialise that
    /// sub-place, abandons the residual the obligation was written for.
    pub(in crate::semantic) fn reject_partial_consume(
        &self,
        root: CheckedType,
        selected: &[u32],
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let Some(marked) = self.owns_modifier_linear_node(root)? else {
            return Ok(());
        };
        let obligation = self.nominal(marked)?.name.clone();
        // The residual is what the consume abandons: every part of the root
        // the selected sub-place does not carry away, named by its own type.
        let residual = match self
            .residual_drop_paths(root, selected)?
            .first()
            .map(|(_, ty)| *ty)
        {
            Some(ty) => self.checked_type_name(ty)?,
            None => self.checked_type_name(root)?,
        };
        self.issue_node::<()>(
            SemanticRule::Prov6,
            node,
            SemanticIssueKind::LinearValuePartiallyConsumed {
                obligation,
                residual,
                mechanical_fix: "destructure the whole value with let N(f: a, ...) = move v;",
            },
        )?;
        Ok(())
    }

    /// [PROV-6] the `linear` modifier is admitted only on a nominal [OWN-1]
    /// classifies as affine.
    pub(in crate::semantic) fn check_linear_modifier_admission(
        &self,
        id: NominalId,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let nominal = self.nominal(id)?;
        if !nominal.linear || !nominal.is_copy() {
            return Ok(());
        }
        self.issue_node::<()>(
            SemanticRule::Prov6,
            node,
            SemanticIssueKind::LinearModifierOnCopyNominal {
                nominal: nominal.name.clone(),
                mechanical_fix: "give a variant a payload, or put the obligation on the \
                     value the issuer hands out",
            },
        )?;
        Ok(())
    }

    /// Whether a `struct_decl` or `enum_decl` node writes the modifier.
    pub(in crate::semantic) fn declaration_is_linear(
        &self,
        node: NodeId,
    ) -> Result<bool, CheckStop> {
        Ok(self
            .tree
            .direct_token_with(node, TerminalPredicate::Fixed(crate::FixedTerminal::Linear))?
            .is_some())
    }

    /// [PROV-6, GRAM-2] the linearity bound a `gparam` or a `region_param`
    /// writes, when it writes one.
    pub(in crate::semantic) fn written_linearity_bound(
        &self,
        node: NodeId,
    ) -> Result<Option<LinearityClass>, CheckStop> {
        let Some(bound) = self
            .tree
            .first_child_with(node, Production::LinearityBound)?
        else {
            return Ok(None);
        };
        for (terminal, class) in [
            (crate::FixedTerminal::Linear, LinearityClass::Linear),
            (crate::FixedTerminal::Copy, LinearityClass::Copy),
        ] {
            if self
                .tree
                .direct_token_with(bound, TerminalPredicate::Fixed(terminal))?
                .is_some()
            {
                return Ok(Some(class));
            }
        }
        Ok(Some(LinearityClass::Affine))
    }

    /// [PROV-6, S37] the class a type parameter's written bound names.
    ///
    /// The bound is mandatory [GRAM-2], so a `gparam` that writes no
    /// `linearity_bound` writes a marker TYPEID instead — `Int` or `Float`,
    /// each of which is a copy class [OP-1, OWN-1]. The reader is over the
    /// parameter's own declaration and never over a use of it.
    pub(in crate::semantic) fn generic_parameter_class(
        &self,
        declaration: crate::DeclarationId,
    ) -> Result<LinearityClass, CheckStop> {
        let record = self
            .resolved
            .declarations()
            .iter()
            .find(|candidate| candidate.id() == declaration)
            .ok_or(crate::SemanticCompilerFailure::InvalidResolution)?;
        let node = self
            .tree
            .node_with_path(record.origin().node())
            .ok_or(crate::SemanticCompilerFailure::InvalidResolution)?;
        Ok(self
            .written_linearity_bound(node)?
            .unwrap_or(LinearityClass::Copy))
    }

    /// [PROV-1, PROV-6, S37] the store class of one region, or `None` when
    /// that region names no store.
    ///
    /// The answer is read from the region's own declaration and from the
    /// reserving occurrences of the unit, never from a type over it: the entry
    /// heap is the general store, a bounded region parameter is the class its
    /// bound names, and a `region_stmt` region is a bump extent exactly when a
    /// reserving occurrence [BLK-2] names it. Every other region — a loop
    /// body's, an unwritten borrow position's, an unbounded region parameter's
    /// — names no store, which is the answer that satisfies neither bound.
    pub(in crate::semantic) fn region_store_class(
        &self,
        region: crate::DeclarationId,
    ) -> Result<Option<LinearityClass>, CheckStop> {
        if region.is_entry_heap_region() {
            return Ok(Some(LinearityClass::Linear));
        }
        let record = self
            .resolved
            .declarations()
            .iter()
            .find(|candidate| candidate.id() == region)
            .ok_or(crate::SemanticCompilerFailure::InvalidResolution)?;
        let role = record.role();
        let Some(node) = self.tree.node_with_path(record.origin().node()) else {
            return Ok(None);
        };
        if role == crate::DeclarationRole::RegionParameter {
            return self.written_linearity_bound(node);
        }
        if role != crate::DeclarationRole::LocalRegion {
            return Ok(None);
        }
        for call in self
            .tree
            .descendants_with(self.tree.root(), Production::Call)?
        {
            if self.reserving_occurrence_names(call, region)? {
                return Ok(Some(LinearityClass::Affine));
            }
        }
        Ok(None)
    }

    /// [PROV-6, STOR-1, STOR-3] the release class of a run branded by one
    /// region, decided from that region's own declaration.
    ///
    /// It is the store class read fail-closed: an `affine`-bounded region
    /// parameter and a `region_stmt` region are bump extents, whose
    /// reclamation is the region's own reset, and every other region — the
    /// entry heap, an unbounded region parameter, a `linear`-bounded one — is
    /// a general store whose run is released by spending a provider. A
    /// misclassification in the extent direction would drop a free, so the
    /// two extent cases are the ones that must be positively identified.
    pub(in crate::semantic) fn vector_release_class(
        &self,
        region: crate::DeclarationId,
    ) -> Result<super::super::model::CheckedReleaseClass, CheckStop> {
        use super::super::model::CheckedReleaseClass;
        if region.is_entry_heap_region() {
            return Ok(CheckedReleaseClass::General);
        }
        let Some(record) = self
            .resolved
            .declarations()
            .iter()
            .find(|candidate| candidate.id() == region)
        else {
            return Ok(CheckedReleaseClass::General);
        };
        Ok(match record.role() {
            crate::DeclarationRole::LocalRegion => CheckedReleaseClass::Extent,
            crate::DeclarationRole::RegionParameter => {
                let node = self
                    .tree
                    .node_with_path(record.origin().node())
                    .ok_or(crate::SemanticCompilerFailure::InvalidResolution)?;
                match self.written_linearity_bound(node)? {
                    Some(LinearityClass::Affine) => CheckedReleaseClass::Extent,
                    _ => CheckedReleaseClass::General,
                }
            }
            _ => CheckedReleaseClass::General,
        })
    }

    /// [PROV-6, S37] an instantiation whose argument's class does not satisfy
    /// the written bound is refused at the call, naming the parameter, the
    /// bound and the argument.
    ///
    /// Satisfaction is the chain `copy < affine < linear` read left to right,
    /// so the bound is a ceiling and not an equality: `linear` accepts every
    /// class, `affine` accepts copy and affine, and `copy` accepts copy alone.
    pub(in crate::semantic) fn check_linearity_bound(
        &self,
        parameter: &str,
        bound: LinearityClass,
        argument: CheckedType,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let actual = self.linearity_class(argument)?;
        if actual.satisfies(bound) {
            return Ok(());
        }
        let argument = self.checked_type_name(argument)?;
        self.issue_node::<()>(
            SemanticRule::Prov6,
            node,
            SemanticIssueKind::LinearityBoundMismatch {
                parameter: parameter.to_owned(),
                bound: bound.spelling(),
                argument,
                actual: actual.spelling(),
            },
        )?;
        Ok(())
    }

    /// [PROV-6, S37] the region axis of the same check.
    ///
    /// A region argument's class is `affine` when the store it names is a bump
    /// extent, whose reclamation is its own region reset, and `linear` when it
    /// names a general store, whose reclamation spends a provider capability
    /// [PROV-1]. A region that names no store — a loan region, or a region a
    /// `region_stmt` introduced that no reserving occurrence names — has no
    /// store class and satisfies neither bound.
    pub(in crate::semantic) fn check_region_linearity_bound(
        &self,
        parameter: &str,
        bound: LinearityClass,
        argument: crate::DeclarationId,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let actual = self.region_store_class(argument)?;
        if actual.is_some_and(|actual| actual.satisfies(bound)) {
            return Ok(());
        }
        self.issue_node::<()>(
            SemanticRule::Prov6,
            node,
            SemanticIssueKind::LinearityBoundMismatch {
                parameter: parameter.to_owned(),
                bound: bound.spelling(),
                argument: if argument.is_entry_heap_region() {
                    "the entry heap's store region".to_owned()
                } else {
                    self.region_phrase(argument)?
                },
                actual: actual.map_or("a region that names no store", LinearityClass::spelling),
            },
        )?;
        Ok(())
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [PROV-6] `dispose p;`'s admission over `p`'s release graph, judged
    /// before the operand's own ownership consume.
    pub(in crate::semantic::check) fn dispose_admission(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if self.is_loan_bearing(ty)? {
            return self.issue_node(
                SemanticRule::Prov6,
                node,
                SemanticIssueKind::DisposeOfLoanBearingOperand {
                    ty: self.checked_type_name(ty)?,
                    mechanical_fix: "a view owns nothing and has no release action of its own; \
                         release the value it views",
                },
            );
        }
        if let Some(marked) = self.owns_modifier_linear_node(ty)? {
            return self.issue_node(
                SemanticRule::Prov6,
                node,
                SemanticIssueKind::DisposeOfLinearNode {
                    nominal: self.nominal(marked)?.name.clone(),
                    mechanical_fix: "take the value apart with let N(f: a, ...) = move v; \
                         and discharge the marked component",
                },
            );
        }
        let mut capability_leaf = false;
        for node_ty in self.release_graph_nodes(ty)? {
            capability_leaf |= self.is_capability_released(node_ty)?;
        }
        if !capability_leaf {
            return self.issue_node(
                SemanticRule::Prov6,
                node,
                SemanticIssueKind::DisposeWithoutCapabilityLeaf {
                    ty: self.checked_type_name(ty)?,
                    mechanical_fix: "this value's release action reclaims no capability; \
                         let the scope exit run it",
                },
            );
        }
        // Every capability-released leaf in this version names the ambient
        // heap, which every scope holds, so the resolution admits every
        // otherwise-admitted operand and contributes no provider write.
        Ok(())
    }
}
