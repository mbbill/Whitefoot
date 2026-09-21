//! [PROV-6] the release graph, the linearity predicate, the `nodrop` and
//! `nocopy` modifiers, the capability bounds, and the statements that read
//! them.
//!
//! Everything here is derived from a type and a scope, never from a name, a
//! signature, or a statement's shape. The release graph is the one object the
//! compiler-derived release and `dispose p;` both walk; the linearity
//! predicate is that graph read against what the scope holds.

use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::{Production, SemanticIssueKind, SemanticRule, TerminalPredicate};

use super::super::model::{CheckedNominalKind, CheckedReleaseMode, CheckedType, NominalId};
use super::{CheckStop, Checker};

/// [OWN-1, PROV-6] the class read from a type's two capabilities, copy and
/// drop, and the class a type parameter's bound grants its body.
///
/// A type with both capabilities is copy, a type with drop alone is affine,
/// and a type with neither is linear; no type has copy without drop. The
/// three form the strict chain `copy < affine < linear`, ordered by what a
/// body may do with a value of the class: `copy` may duplicate it, use it bare
/// and drop it; `affine` may `move` it at most once and may drop it; `linear`
/// must consume it exactly once and may never drop it. A parameter written
/// `T: copy` is copy, `T: drop` is affine, and one written with no bound is
/// linear.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::semantic) enum LinearityClass {
    /// Duplicated on use and dropped without any action [OWN-1].
    Copy,
    /// Reclaimed without spending a capability and unmarked.
    Affine,
    /// Marked `nodrop`, or owning a part that is, or a type parameter whose
    /// declaration writes no bound.
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

    /// [PROV-6] satisfaction is the filter itself: `T: copy` accepts copy
    /// arguments only, `T: drop` accepts copy and affine arguments, and a
    /// parameter with no bound accepts every class. With the bound read as
    /// the class it grants, that is `self <= bound`.
    pub(in crate::semantic) fn satisfies(self, bound: Self) -> bool {
        self <= bound
    }

    /// [GRAM-2] the `capability_bound` spelling that grants this class, which
    /// a rejection naming the written bound prints. A linear parameter writes
    /// no bound, and no argument fails to satisfy it.
    pub(in crate::semantic) const fn bound_spelling(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Affine => "drop",
            Self::Linear => "no bound",
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
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
            CheckedType::Buffer { element } => self.loan_bearing_with(element.ty(), visited),
            CheckedType::Array { element, .. } | CheckedType::Window { element, .. } => {
                self.loan_bearing_with(self.element_type(element)?, visited)
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
    /// variant payloads and its `box` referent.
    fn owned_components(&self, id: NominalId) -> Result<Vec<CheckedType>, CheckStop> {
        Ok(match &self.nominal(id)?.kind {
            CheckedNominalKind::Struct { fields } => fields.iter().map(|field| field.ty).collect(),
            CheckedNominalKind::Enum { variants } => variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect(),
            CheckedNominalKind::Box { referent, .. } => vec![*referent],
            CheckedNominalKind::Opaque => Vec::new(),
        })
    }

    /// [STOR-3, FN-10] whether the release graph can execute an action.
    /// The capability class alone is insufficient: a nocopy aggregate of
    /// scalars and a statically empty run both have empty releases.
    pub(super) fn has_nonempty_release(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        let mut pending = vec![ty];
        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            match current {
                CheckedType::Generic(_) if !self.is_copy_type(current)? => return Ok(true),
                CheckedType::Nominal(id) => {
                    if matches!(self.nominal(id)?.kind, CheckedNominalKind::Box { .. }) {
                        return Ok(true);
                    }
                    pending.extend(self.owned_components(id)?);
                }
                CheckedType::Array { element, length } if length.value() != Some(0) => {
                    pending.push(self.element_type(element)?);
                }
                CheckedType::Window {
                    element, capacity, ..
                } if capacity.and_then(super::super::model::CheckedConst::value) != Some(0) => {
                    pending.push(self.element_type(element)?);
                }
                CheckedType::Buffer { element } => pending.push(element.ty()),
                _ => {}
            }
        }
        Ok(false)
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
                CheckedType::Buffer { element } => {
                    pending.push(element.ty());
                }
                // A run owns the elements of its window [BLK-1], so its
                // element is a sub-node exactly as a field is.
                CheckedType::Array { element, .. } | CheckedType::Window { element, .. } => {
                    pending.push(self.element_type(element)?);
                }
                CheckedType::Nominal(id) => pending.extend(self.owned_components(id)?),
                _ => {}
            }
        }
        Ok(nodes)
    }

    /// [PROV-6] the release graph selected at one release point. Proving a
    /// direct run empty removes only its element edge; its own backing node
    /// remains, so provider and effect obligations for that storage survive.
    pub(in crate::semantic) fn release_graph_nodes_for(
        &self,
        ty: CheckedType,
        release: CheckedReleaseMode,
    ) -> Result<Vec<CheckedType>, CheckStop> {
        if release == CheckedReleaseMode::EmptyRun && matches!(ty, CheckedType::Window { .. }) {
            return Ok(vec![ty]);
        }
        self.release_graph_nodes(ty)
    }

    /// [PROV-6] whether any node of this type's release graph carries the
    /// `nodrop` modifier, this type's own node included.
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
    /// The copy half is [OWN-1]'s structural capability: every owned part is
    /// copy and the declaration removes nothing. The drop half is [PROV-6]'s
    /// closure: a type lacks drop exactly when its declaration carries
    /// `nodrop`, or it owns at any depth a type that lacks it, a type
    /// parameter written with no bound included.
    ///
    /// A type parameter standing for itself at a symbolic instance [FN-2] has
    /// exactly the class its written bound grants: the body is checked once
    /// under that bound and the bound is what the body was written for.
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
        Ok(if self.linear_release_obligation(ty)?.is_some() {
            LinearityClass::Linear
        } else {
            LinearityClass::Affine
        })
    }

    /// [PROV-6, D3] every general store whose capability a value of this type
    /// spends at its release, named by that store's own region.
    ///
    /// A run branded to a general store is the one capability-released node
    /// this version has whose provider is a value [PROV-1]: `box<T>` and
    /// `buffer<T>` name the ambient heap, which is no value and which every
    /// scope therefore holds. The regions come back in release-graph order
    /// with no duplicates.
    pub(in crate::semantic) fn capability_released_stores(
        &self,
        ty: CheckedType,
    ) -> Result<Vec<crate::DeclarationId>, CheckStop> {
        self.capability_released_stores_for(ty, CheckedReleaseMode::Full)
    }

    pub(in crate::semantic) fn capability_released_stores_for(
        &self,
        ty: CheckedType,
        release: CheckedReleaseMode,
    ) -> Result<Vec<crate::DeclarationId>, CheckStop> {
        // [STOR-8] one heap, provided by the trusted base: no value provides
        // storage, so no release spends a provider a parameter supplies and
        // no release-graph node names one. The graph is still walked, so a
        // type whose release graph this version cannot build is reported here
        // rather than answered with an empty set.
        self.release_graph_nodes_for(ty, release)?;
        Ok(Vec::new())
    }

    pub(super) fn linear_release_obligation(
        &self,
        ty: CheckedType,
    ) -> Result<Option<String>, CheckStop> {
        if let Some(marked) = self.owns_modifier_linear_node(ty)? {
            return Ok(Some(self.nominal(marked)?.name.clone()));
        }
        // A symbolic type parameter carries its class in the written bound,
        // not on a nominal node, so inspect each graph node explicitly.
        for node in self.release_graph_nodes(ty)? {
            if let CheckedType::Generic(declaration) = node
                && self.generic_parameter_class(declaration)? == LinearityClass::Linear
            {
                return Ok(Some(format!(
                    "the absent bound of {}, which grants its body no drop capability",
                    self.declaration_spelling(declaration)?
                )));
            }
        }
        Ok(None)
    }

    /// Chooses the graph for a compiler-derived release. A direct run whose
    /// complete graph is blocked may use the element-free graph only when its
    /// own backing obligations remain satisfiable; entailment separately
    /// proves the run empty before the edge removes its facts.
    pub(in crate::semantic::check) fn scope_release_mode(
        &self,
        ty: CheckedType,
        name: &str,
        bindings: &std::collections::HashMap<crate::DeclarationId, super::LocalBinding>,
        node: NodeId,
    ) -> Result<CheckedReleaseMode, CheckStop> {
        let linear = self.linear_release_obligation(ty)?;
        let missing = self
            .capability_released_stores(ty)?
            .into_iter()
            .find(|store| !self.scope_holds_store_capability(bindings, *store));
        if linear.is_none() && missing.is_none() {
            return Ok(CheckedReleaseMode::Full);
        }
        // [WIN-3] "No operation releases a linear element: a storage whose
        // element type is linear is itself linear [PROV-6] and the program
        // must take every element out and consume it, and then, with the
        // storage proved empty, call `free_empty` [OP-14]." A proved-empty
        // run therefore takes no compiler-derived release of its own either:
        // `free_empty` is a written call that consumes the window, so a run
        // reaching a scope exit is refused here whatever its length, and the
        // v0.59 element-free derived release is gone with the capability
        // leaves it was written for.
        if linear.is_some() {
            self.reject_linear_value_not_consumed(ty, name, node)?;
        }
        self.reject_release_without_capability(ty, name, bindings, node)?;
        Ok(CheckedReleaseMode::Full)
    }

    /// [PROV-6, D3] whether a live binding of this store's provider type
    /// stands in this scope, reached directly or through a borrow.
    ///
    /// A provider enters a function only as a parameter or as an entry input
    /// [PROV-2, FN-7], so this is a question about the bindings that stand at
    /// the point, and the mode of the binding is immaterial: a `&uniq
    /// Heap<'s>` parameter holds the capability exactly as the entry's own
    /// `own Heap<'s>` does.
    pub(in crate::semantic) fn scope_holds_store_capability(
        &self,
        bindings: &std::collections::HashMap<crate::DeclarationId, super::LocalBinding>,
        _store: crate::DeclarationId,
    ) -> bool {
        bindings.values().any(|local| local.live && false)
    }

    /// [PROV-6, D3] the refusal of a value whose release spends a capability
    /// this scope does not hold. The rejection names the binding, the scope's
    /// own edge, and the absent capability.
    pub(in crate::semantic) fn reject_release_without_capability(
        &self,
        ty: CheckedType,
        name: &str,
        bindings: &std::collections::HashMap<crate::DeclarationId, super::LocalBinding>,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        for store in self.capability_released_stores(ty)? {
            if self.scope_holds_store_capability(bindings, store) {
                continue;
            }
            let phrase = self.region_phrase(store)?;
            return self
                .issue_node::<()>(
                    SemanticRule::Prov6,
                    node,
                    SemanticIssueKind::LinearValueNotConsumed {
                        binding: name.to_owned(),
                        obligation: format!(
                            "the provider capability of {phrase}, which no live binding of this scope holds"
                        ),
                        mechanical_fix: "move the value out whole, take it apart with let N(f: a, ...) = move v;, or receive this store's provider as a parameter so the scope holds its capability",
                    },
                )
                .map(|_| ());
        }
        Ok(())
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
        // [PROV-6] a type parameter with no bound is linear at its symbolic
        // instance: the body is checked once under what its bound grants, and
        // with no bound it must consume the value exactly once and may never
        // drop it. The obligation names the absent bound rather than a
        // nominal, because at the symbolic instance no nominal carries it.
        let Some(marked) = self.linear_release_obligation(ty)? else {
            return Ok(());
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
        bindings: &std::collections::HashMap<crate::DeclarationId, super::LocalBinding>,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let obligation = if let Some(obligation) = self.linear_release_obligation(root)? {
            obligation
        } else if let Some(store) = self
            .capability_released_stores(root)?
            .into_iter()
            .find(|store| !self.scope_holds_store_capability(bindings, *store))
        {
            format!(
                "the provider capability of {}, which no live binding of this scope holds",
                self.region_phrase(store)?
            )
        } else {
            return Ok(());
        };
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

    /// Whether a `struct_decl` or `enum_decl` node writes `nodrop`, the
    /// modifier that removes the drop capability and copy with it [OWN-1].
    pub(in crate::semantic) fn declaration_is_linear(
        &self,
        node: NodeId,
    ) -> Result<bool, CheckStop> {
        Ok(self
            .tree
            .direct_token_with(node, TerminalPredicate::Fixed(crate::FixedTerminal::Nodrop))?
            .is_some())
    }

    /// Whether a `struct_decl` or `enum_decl` node writes `nocopy`, the
    /// modifier that removes the copy capability alone [OWN-1, GRAM-2].
    pub(in crate::semantic) fn declaration_is_nocopy(
        &self,
        node: NodeId,
    ) -> Result<bool, CheckStop> {
        Ok(self
            .tree
            .direct_token_with(node, TerminalPredicate::Fixed(crate::FixedTerminal::Nocopy))?
            .is_some())
    }

    /// [PROV-6, GRAM-2] the class a `gparam`'s written capability bound
    /// grants, when it writes one: `copy` grants both capabilities and `drop`
    /// grants drop alone, which are the copy and affine classes read at the
    /// parameter.
    pub(in crate::semantic) fn written_linearity_bound(
        &self,
        node: NodeId,
    ) -> Result<Option<LinearityClass>, CheckStop> {
        let Some(bound) = self
            .tree
            .first_child_with(node, Production::CapabilityBound)?
        else {
            return Ok(None);
        };
        for (terminal, class) in [
            (crate::FixedTerminal::Copy, LinearityClass::Copy),
            (crate::FixedTerminal::Drop, LinearityClass::Affine),
        ] {
            if self
                .tree
                .direct_token_with(bound, TerminalPredicate::Fixed(terminal))?
                .is_some()
            {
                return Ok(Some(class));
            }
        }
        Err(crate::SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// [PROV-6, FN-2] the class a type parameter's bound grants its body.
    ///
    /// A `gparam` writes a `capability_bound`, a numeric marker TYPEID —
    /// `Int` or `Float`, each of which implies copy [OP-1, OWN-1] — or no
    /// bound at all, which grants no capability and is the linear class. The
    /// reader is over the parameter's own declaration and never over a use of
    /// it.
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
        if self.tree.production(node)? == Production::Type {
            let mut application = node;
            while self.tree.production(application)? != Production::PackUse {
                application = self
                    .tree
                    .parent(application)?
                    .ok_or(crate::SemanticCompilerFailure::InvalidResolution)?;
            }
            for parameter in self.expand_formal_parameters(application)? {
                if let super::generics::GenericParameter::Type {
                    declaration: candidate,
                    bound,
                } = parameter
                    && candidate == declaration
                {
                    return Ok(match bound {
                        super::generics::GenericBound::Class(class) => class,
                        super::generics::GenericBound::Int
                        | super::generics::GenericBound::Float => LinearityClass::Copy,
                    });
                }
            }
            return Err(crate::SemanticCompilerFailure::InvalidResolution.into());
        }
        if let Some(class) = self.written_linearity_bound(node)? {
            return Ok(class);
        }
        // A `gparam` with no `capability_bound` child writes a numeric marker
        // after its colon, or nothing: the colon tells the two apart.
        Ok(
            if self
                .tree
                .direct_token_with(node, TerminalPredicate::Fixed(crate::FixedTerminal::Colon))?
                .is_some()
            {
                LinearityClass::Copy
            } else {
                LinearityClass::Linear
            },
        )
    }

    /// [PROV-6] an instantiation whose argument's class does not satisfy the
    /// written bound is refused at the call, naming the parameter, the bound
    /// and the argument.
    ///
    /// The bound is a capability filter, so it is a ceiling and not an
    /// equality: no bound accepts every class, `drop` accepts copy and
    /// affine, and `copy` accepts copy alone.
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
                bound: bound.bound_spelling(),
                argument,
                actual: actual.spelling(),
            },
        )?;
        Ok(())
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [PROV-6, D3] the provider place each general store reached by `ty`'s
    /// release graph spends, resolved against this function's own parameters.
    ///
    /// A general-store provider enters a function as an ordinary parameter
    /// [TYPE-2], so the parameter list is the complete candidate set,
    /// and the write this returns is what makes a derived or early release of
    /// store-backed storage visible in the declared row [EFF-2].
    pub(in crate::semantic) fn resolved_provider_writes(
        &self,
        function: &super::FunctionSignature,
        ty: CheckedType,
    ) -> Result<Vec<super::super::model::CheckedStatePath>, CheckStop> {
        self.resolved_provider_writes_for(function, ty, CheckedReleaseMode::Full)
    }

    pub(in crate::semantic) fn resolved_provider_writes_for(
        &self,
        function: &super::FunctionSignature,
        ty: CheckedType,
        release: CheckedReleaseMode,
    ) -> Result<Vec<super::super::model::CheckedStatePath>, CheckStop> {
        let mut writes = Vec::new();
        for _store in self.capability_released_stores_for(ty, release)? {
            if let Some(parameter) = function.parameters.iter().find(|_parameter| false) {
                writes.push(super::super::model::CheckedStatePath {
                    root: parameter.declaration,
                    steps: Vec::new(),
                });
            }
        }
        Ok(writes)
    }
}
