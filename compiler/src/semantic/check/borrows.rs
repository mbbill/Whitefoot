use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    ScopeId, SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedBufferRoot, CheckedExpression, CheckedMode, CheckedNominalKind, CheckedSliceOrigin,
    CheckedStatePath, CheckedType, LoanStrength,
};
use super::linearity::LinearityClass;
use super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, ParameterSignature,
    TypedExpression,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BorrowKind {
    Shared,
    Unique,
}

/// The owned indirection a `deref` place base reaches when its root binding
/// is own-mode rather than a borrow holder [OWN-14]. Each arm names the
/// storage class the root owns [STOR-1] and the content type the written
/// suffix chain continues from; a borrow position additionally reads the arm
/// as the [OWN-10] case that governs it.
#[derive(Clone, Copy)]
pub(super) enum OwnedContent {
    /// `box<T>` content: heap storage this binding owns [STOR-1], so
    /// [OWN-10]'s own-mode-binding case governs a borrow of it.
    Boxed(CheckedType),
    /// `arena<'r, T>` content: arena-owned storage bounded by `'r`
    /// [STOR-1, STOR-4], so [OWN-10]'s arena case governs a borrow of it
    /// with source region `'r`.
    Arena {
        source: DeclarationId,
        content: CheckedType,
    },
}

impl OwnedContent {
    pub(super) const fn ty(self) -> CheckedType {
        match self {
            Self::Boxed(content) | Self::Arena { content, .. } => content,
        }
    }
}

/// The syntactic position of a `borrow_expr`, which decides the written
/// reborrow form admitted there: the statement-scoped child in call-argument
/// position [OWN-6] and the returned reborrow as the complete return
/// expression [OWN-14]. Every other position rejects every reborrow form.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ReborrowPosition {
    /// Not a position OWN-14 admits any written reborrow form in.
    Forbidden,
    /// An argument atom of a `call` expression, judged by OWN-6 alone;
    /// `own_result` carries OWN-6's receiving-result-mode condition.
    CallArgument {
        /// Whether the receiving call's result mode is `own` or `unit`.
        own_result: bool,
        /// Under the reborrow extension only: this argument position is the
        /// receiving borrow-returning call's single provenance candidate, so
        /// a written reborrow here may outlive the statement inside the
        /// bound result and its parent holder is suspended for the remainder
        /// of its life. Always `false` with the extension off.
        result_candidate: bool,
    },
    /// The complete `expr` of a `return_stmt` [OWN-14].
    ReturnExpression,
}

// [DIAG-1] same-node judgment order inside the borrow checker: a suspended
// holder's OWN-5 rejection is asked after OWN-1's liveness and spelling
// judgments and before the reborrow admissions, and the returned reborrow
// asks OWN-10's creation obligation before OWN-14's admission, because each
// earlier-defined rule cites first at one offending use.
const _: () = {
    assert!(SemanticRule::Own1.definition_rank() < SemanticRule::Own5.definition_rank());
    assert!(SemanticRule::Own5.definition_rank() < SemanticRule::Own6.definition_rank());
    assert!(SemanticRule::Own10.definition_rank() < SemanticRule::Own14.definition_rank());
};

/// [OWN-14]'s exact restructuring for a rejected reborrow form.
const OWN14_RESTRUCTURING: &str = "pass the reborrow as a statement-scoped child in argument position, \
     return it as the complete return expression from a parameter or let-bound holder, \
     or return the holder itself";

/// [OWN-10]'s storage-duration condition for a borrow of local storage.
///
/// The rejection published nothing at all: not the region it refused, not the
/// binding whose storage the borrow views, and not where a region it would
/// accept has to be introduced. All three are in hand at every one of these
/// judgments.
const OWN10_LOCAL_STORAGE: &str = "a borrow of local storage names a region introduced inside that binding's own scope: write `region 'r { ... }` after the binding and take the borrow inside it. A caller-supplied region parameter is never admitted here, because it outlives the storage.";

/// OWN-6's statement-scoped-region condition, in the terms a writer has.
///
/// The rule reads "a locally-introduced region whose block does not extend
/// beyond the enclosing statement". A writer meets that as two facts at once:
/// the region holds one statement, and anything that statement binds is gone
/// at the closing brace — so `region 'r { let permit = reserve(...); match
/// open(permit: move permit, ...) { ... } }`, which is the shape every
/// recursive walker wants, is rejected and cannot be repaired by shortening
/// the region.
///
/// The 0099 text named two routes and neither reached a working walker: the
/// `replace` route cannot commit where the call consumed the target's root,
/// which is exactly what `move permit` does, and the helper route is only the
/// first third of the working idiom. `tests/programs/dir_walk.wf` pairs the
/// helper with two more parts, and all three are named here.
const OWN6_STATEMENT_SCOPE: &str = "a child reborrow's region admits exactly one statement, and a value that statement binds dies at the region's end, so `region 'r { let permit = reserve_handle::<'r>(factory: &uniq 'r holder); match open_...(permit: move permit, ...) { ... } }` is two statements and cannot be repaired by shortening the region. The whole idiom is three parts: move the reserve and the open into one helper that takes the holder as `&uniq 'f` and returns the opened value (`fn open_source_from_factory['f, 'd](factory: &uniq 'f HandleFactory, directory: &'d DirectoryRead) -> result: own Result<DirectorySource, IoError>`); make the single statement of the region the `match` on that helper's call; and write every statement that uses the opened value inside that `match` arm, because the opened value dies with the region (P4 linear threading, P15 recursive walker). The other route, `let stale = replace target = call(...);`, applies only where the call leaves the target's root alive: a call that consumes the target root — one taking `move permit` — rejects OWN-1 instead.";

/// OWN-6's receiver condition: which calls admit a reborrow argument at all.
const OWN6_ARGUMENT_POSITION: &str = "a reborrow is an argument only to a call returning an owned \
     value or unit, or in the one argument position a borrow-returning call takes its result \
     from; pass the holder itself, or bind the result from that position";

/// OWN-6's holder condition: which holder a child may be taken from.
const OWN6_HOLDER: &str = "reborrow only a parameter or let-bound holder, take `&uniq` only from \
     a `&uniq` holder, and introduce the child region inside the holder's own region";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResolvedPlace {
    pub(super) root: DeclarationId,
    pub(super) fields: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BorrowInfo {
    pub(super) kind: BorrowKind,
    pub(super) region: DeclarationId,
    pub(super) place: ResolvedPlace,
    pub(super) origin_region: Option<DeclarationId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SliceInfo {
    pub(super) region: DeclarationId,
    pub(super) origins: Vec<CheckedSliceOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SliceLoan {
    /// The named data region, which—not a descriptor binding—owns this claim.
    pub(super) region: DeclarationId,
    /// The exact source place protected for the complete named region.
    pub(super) place: ResolvedPlace,
    /// [VIEW-1] the strength of the loan the view value holds.
    ///
    /// It is carried because two exclusive loans on one range are what
    /// [OWN-5] 606 refuses, and because a target path through a view is
    /// admitted at exclusive strength and at no other [SET-1].
    pub(super) strength: LoanStrength,
    /// [PROV-3] the bindings that hold this loan: every view binding whose
    /// origin set names the loan's place.
    ///
    /// A loan begins where its value is formed or copied and ends where that
    /// value's own liveness ends. For a **copy** view that end is its last
    /// use, and a use is a property of a binding, so the extent this list
    /// carries is the union of its holders' remaining uses. An empty list is
    /// a loan no binding took — the formation's value was consumed inside
    /// its own statement — and such a loan keeps the conservative
    /// region-scoped extent [OWN-4], because this checker has no program
    /// point between two operands of one statement.
    pub(super) descriptors: Vec<DeclarationId>,
}

impl SliceLoan {
    /// Whether one access to the origin conflicts with this loan [OWN-5].
    ///
    /// A shared loan refuses what a shared borrow refuses: a write, a move,
    /// and the unique borrow that would carry either — which is what makes an
    /// exclusive formation over a place a shared view already views the
    /// second formation's own conflict.
    ///
    /// An exclusive loan refuses those and, being exclusive, the unique
    /// borrow a *second exclusive view* of the range would take. It admits a
    /// **shared** second view, which is [OWN-6]'s shared child reborrow of a
    /// unique loan applied to a view rather than to a place [S31]: the child
    /// carries the parent's range, the parent may not write its elements
    /// while the child lives, and the parent resumes where the child's own
    /// liveness ends [PROV-3].
    ///
    /// A read of the origin is admitted at both strengths, which is what lets
    /// a view's own element read reach the storage it views.
    pub(super) const fn refuses(&self, access: AccessKind) -> bool {
        matches!(
            access,
            AccessKind::Write | AccessKind::Move | AccessKind::UniqueBorrow
        )
    }
}

/// The value a position requires of its operand, for [TYPE-7]'s implicit-read
/// exclusivity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RequiredReferent {
    /// A position requires one exact value type, as counted endpoints require
    /// `own u64` after TYPE-7's implicit-read exclusivity [TYPE-5].
    Exact(CheckedType),
    /// A `match` scrutinee requires an enum value [OWN-13, ERR-2].
    Enum,
    /// An `index` root requires directly indexable storage [OP-4].
    IndexableStorage,
}

/// How one liveness question reads a use written in the same statement as the
/// access it is asked about.
///
/// This checker states no program point between two operands of one statement,
/// which is why a loan no binding holds keeps its whole region extent. Where
/// that reading is what the rule needs, the caller asks for it; where a rule
/// was stated over document order and its diagnostics are pinned to that
/// reading, it keeps it.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Simultaneity {
    /// Document order decides, as it did before the sibling-operand question
    /// arose.
    Sequential,
    /// A use anywhere in the access's own `let`, `set`, expression, or
    /// `return` statement is simultaneous with the access.
    OneStatementIsOneMoment,
}

#[derive(Clone, Copy)]
pub(super) enum AccessKind {
    Read,
    Write,
    Move,
    SharedBorrow,
    UniqueBorrow,
}

impl BorrowInfo {
    pub(super) const fn mode(&self) -> CheckedMode {
        match self.kind {
            BorrowKind::Shared => CheckedMode::Shared(self.region),
            BorrowKind::Unique => CheckedMode::Unique(self.region),
        }
    }
}

impl SliceInfo {
    pub(super) fn source_places(&self) -> Vec<(ResolvedPlace, Option<DeclarationId>)> {
        self.origins
            .iter()
            .filter_map(|origin| match origin {
                CheckedSliceOrigin::SourcePlace {
                    root,
                    fields,
                    origin_region,
                } => Some((
                    ResolvedPlace {
                        root: *root,
                        fields: fields.clone(),
                    },
                    *origin_region,
                )),
                CheckedSliceOrigin::ImmutableConst | CheckedSliceOrigin::FormalSlice { .. } => None,
            })
            .collect()
    }

    pub(super) fn effect_places(&self) -> Vec<ResolvedPlace> {
        let mut places = Vec::new();
        for origin in &self.origins {
            let place = match origin {
                CheckedSliceOrigin::SourcePlace { root, fields, .. } => Some(ResolvedPlace {
                    root: *root,
                    fields: fields.clone(),
                }),
                CheckedSliceOrigin::FormalSlice { parameter, .. } => Some(ResolvedPlace {
                    root: *parameter,
                    fields: Vec::new(),
                }),
                CheckedSliceOrigin::ImmutableConst => None,
            };
            if let Some(place) = place
                && !places.contains(&place)
            {
                places.push(place);
            }
        }
        places
    }
}

pub(super) fn push_slice_origin(origins: &mut Vec<CheckedSliceOrigin>, origin: CheckedSliceOrigin) {
    if !origins.contains(&origin) {
        origins.push(origin);
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Converts a resolved runtime place into the source-expressible static
    /// state path at a callable boundary. Struct fields remain precise;
    /// enum payloads and owned indirection are represented by their nearest
    /// expressible parent because EFF-1 deliberately has no such projection.
    pub(super) fn state_path(
        &self,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedStatePath, CheckStop> {
        let mut ty = bindings
            .get(&place.root)
            .map(|binding| binding.ty)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let mut fields = Vec::new();
        for field in &place.fields {
            let CheckedType::Nominal(nominal) = ty else {
                break;
            };
            let CheckedNominalKind::Struct {
                fields: declared_fields,
            } = &self.nominal(nominal)?.kind
            else {
                break;
            };
            let selected = declared_fields
                .get(*field as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            fields.push(*field);
            ty = selected.ty;
        }
        Ok(CheckedStatePath {
            root: place.root,
            fields,
        })
    }

    /// Instantiates one resolved place onto the current function's incoming
    /// formal identities. Fresh local owners yield no enclosing effect;
    /// moved affine owners retain their structural formal sources; a scalar
    /// borrow parameter falls back to its direct parameter place.
    pub(super) fn effect_paths_for_place(
        &self,
        place: &ResolvedPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Vec<CheckedStatePath>, CheckStop> {
        let canonical = self.state_path(place, bindings)?;
        let binding = bindings
            .get(&place.root)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if let Some(origins) = &binding.state_origins {
            if origins.unknown && !self.deriving_result_state_origin.get() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            let mut paths = origins
                .clone()
                .projected(&canonical.fields)
                .formals
                .into_iter()
                .map(|origin| origin.source)
                .collect::<Vec<_>>();
            paths.sort();
            paths.dedup();
            return Ok(paths);
        }
        let parameter = self.resolved.declarations().iter().any(|declaration| {
            declaration.id() == place.root && declaration.role() == DeclarationRole::Parameter
        });
        if parameter
            && (binding.mode != CheckedMode::Own || matches!(binding.ty, CheckedType::Slice { .. }))
        {
            Ok(vec![canonical])
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) fn parse_region_parameters(
        &self,
        function: NodeId,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let Some(node) = self
            .tree
            .first_child_with(function, Production::RegionParams)?
        else {
            return Ok(Vec::new());
        };
        // [GRAM-2, PROV-6] each member is one `region_param`, so the written
        // order is the child order and each child owns exactly one region.
        let mut declarations = Vec::new();
        for member in self.tree.children_with(node, Production::RegionParam)? {
            // [PROV-6, S37] a member may carry a linearity bound, read here at
            // the declaration and checked at every instantiation over the
            // store class of the region argument. `copy` is not one of the two
            // classes a store has: the bound names the class of the store the
            // region identifies, and no store is reclaimed by duplication.
            if self.written_linearity_bound(member)? == Some(LinearityClass::Copy) {
                return self.issue_node(
                    SemanticRule::Prov6,
                    member,
                    SemanticIssueKind::InvalidRegionBound {
                        mechanical_fix: "a region parameter's bound names its store: write \
                             `affine` for a bump extent and `linear` for a general store, or \
                             leave it unbounded",
                    },
                );
            }
            declarations.push(
                self.declaration_at(member, DeclarationRole::RegionParameter)?
                    .id(),
            );
        }
        Ok(declarations)
    }

    /// Whether one node writes a REGIONID of its own.
    ///
    /// [FORM-8] leaves the region unwritten wherever the surrounding text
    /// already fixes it, so every region reader asks this first.
    pub(super) fn writes_region(&self, node: NodeId) -> Result<bool, CheckStop> {
        Ok(self
            .tree
            .direct_token_with(node, TerminalPredicate::RegionIdentifier)?
            .is_some())
    }

    /// The region one construct declares at its own node.
    ///
    /// A named `region_stmt` declares it from its REGIONID; every elided
    /// [FORM-8] position has its region minted by resolution at the owning
    /// node under a spelling [FORM-3] admits from no source token, so the node
    /// is the only route to it.
    pub(super) fn region_declared_at(&self, node: NodeId) -> Result<DeclarationId, CheckStop> {
        let path = self.tree.path(node)?;
        self.resolved
            .declarations()
            .iter()
            .find(|declaration| {
                matches!(
                    declaration.role(),
                    DeclarationRole::RegionParameter | DeclarationRole::LocalRegion
                ) && declaration.origin().node() == path
            })
            .map(crate::DeclarationRecord::id)
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    /// The region an elided `borrow_expr` denotes: the innermost region block
    /// lexically enclosing it [FORM-8].
    ///
    /// A region block is a `region_stmt` or a loop body, because every
    /// `loop_stmt` and `for_stmt` body is itself one [OWN-11]. `None` means no
    /// region block encloses the borrow, where [FORM-8] requires the region to
    /// be written.
    pub(super) fn enclosing_region(
        &self,
        node: NodeId,
    ) -> Result<Option<DeclarationId>, CheckStop> {
        let mut current = node;
        while let Some(parent) = self.tree.parent(current)? {
            match self.tree.production(parent)? {
                Production::RegionStmt | Production::LoopStmt => {
                    return Ok(Some(self.region_declared_at(parent)?));
                }
                // A counted loop's endpoint atoms are written in the enclosing
                // scope, not the body, so only a statement of the body reaches
                // the body's own region.
                Production::ForStmt if self.tree.production(current)? == Production::Stmt => {
                    return Ok(Some(self.region_declared_at(parent)?));
                }
                Production::FnDecl => return Ok(None),
                _ => {}
            }
            current = parent;
        }
        Ok(None)
    }

    /// Whether one region declaration is the region a loop body introduces.
    ///
    /// [OWN-11] mints it at the owning `loop_stmt` or `for_stmt` node, so the
    /// origin production tells the two local-region kinds apart.
    pub(super) fn loop_body_region_owner(
        &self,
        region: DeclarationId,
    ) -> Result<Option<NodeId>, CheckStop> {
        let declaration = self.region_declaration(region)?;
        if declaration.role() != DeclarationRole::LocalRegion {
            return Ok(None);
        }
        let Some(node) = self.tree.node_with_path(declaration.origin().node()) else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        Ok(matches!(
            self.tree.production(node)?,
            Production::LoopStmt | Production::ForStmt
        )
        .then_some(node))
    }

    /// The region one `borrow_expr` takes.
    ///
    /// [FORM-8] writes it exactly when it is not the innermost enclosing
    /// `region_stmt`'s region, so a written name resolves by lookup and an
    /// elided one by that enclosing block. `None` means the borrow elides its
    /// region with no `region_stmt` enclosing it, which FORM-8 rejects.
    pub(super) fn borrow_expr_region(
        &self,
        node: NodeId,
    ) -> Result<Option<DeclarationId>, CheckStop> {
        if !self.writes_region(node)? {
            return self.enclosing_region(node);
        }
        let usage = self.use_at(node, LexicalUseRole::BorrowRegion)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Region,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if self.enclosing_region(node)? == Some(declaration) {
            return self.issue_node(
                SemanticRule::Form8,
                node,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "drop the region name: this borrow takes the region of the \
region block that most closely encloses it, and a loop body is one",
                },
            );
        }
        Ok(Some(declaration))
    }

    /// The region a `slice` or `arena` type carries.
    ///
    /// [FORM-8] writes it only where it relates two positions of the owning
    /// declaration or names an output-position region the caller chooses;
    /// elsewhere resolution mints the position's own distinct region.
    pub(super) fn type_region(&self, node: NodeId) -> Result<DeclarationId, CheckStop> {
        if !self.writes_region(node)? {
            return self.region_declared_at(node);
        }
        let usage = self.use_at(node, LexicalUseRole::TypeRegion)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Region,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        Ok(declaration)
    }

    /// Every formal region of one callable boundary: the written
    /// `region_params` first, then the region each parameter position leaves
    /// unwritten [FORM-8], in parameter order.
    ///
    /// An unwritten position denotes a region distinct from every other, so
    /// it occupies exactly that one position and no caller writes it; it is
    /// still a formal region of the boundary, and a call substitutes it with
    /// the region of the actual argument at that position.
    pub(super) fn append_elided_formal_regions(
        written: &mut Vec<DeclarationId>,
        parameters: &[ParameterSignature],
    ) {
        for parameter in parameters {
            for region in [
                match parameter.mode {
                    CheckedMode::Own => None,
                    CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
                },
                match parameter.ty {
                    CheckedType::Slice { region, .. } => Some(region),
                    _ => None,
                },
            ]
            .into_iter()
            .flatten()
            {
                if !written.contains(&region) {
                    written.push(region);
                }
            }
        }
    }

    /// Every REGIONID written below `root`, in source order, as the owning
    /// node and its exact spelling.
    fn written_regions_below(&self, root: NodeId) -> Result<Vec<(NodeId, String)>, CheckStop> {
        let classified = self.resolved.syntax().classified_bundle();
        let mut found = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            for terminal in self.tree.direct_token_indices(node)? {
                let token = classified
                    .tokens()
                    .get(*terminal)
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                if !token
                    .terminals()
                    .contains(TerminalPredicate::RegionIdentifier)
                {
                    continue;
                }
                found.push((
                    node,
                    *terminal,
                    std::str::from_utf8(self.tree.token_bytes(*terminal)?)
                        .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?
                        .to_owned(),
                ));
            }
            stack.extend_from_slice(self.tree.children(node)?);
        }
        found.sort_by_key(|(_, terminal, _)| *terminal);
        Ok(found
            .into_iter()
            .map(|(node, _, name)| (node, name))
            .collect())
    }

    /// [FORM-8] over one `fn_decl` or `fn_sig` boundary.
    ///
    /// A region name is written exactly where the same region is meant at two
    /// or more positions of the declaration, or where an output position names
    /// a region no parameter position names. `region_params` then lists
    /// exactly those names, once each, in order of first written occurrence.
    pub(super) fn check_declaration_region_spelling(&self, node: NodeId) -> Result<(), CheckStop> {
        let mut inputs = Vec::new();
        if let Some(parameters) = self.tree.first_child_with(node, Production::ParamList)? {
            inputs = self.written_regions_below(parameters)?;
        }
        let mut outputs = Vec::new();
        for production in [Production::ResultBinding, Production::Effects] {
            if let Some(child) = self.tree.first_child_with(node, production)? {
                outputs.extend(self.written_regions_below(child)?);
            }
        }
        // [FORM-8] every output position writes its region: either the same
        // region is meant at an input position, or no input determines it and
        // the caller chooses it. An elided output region names nothing either
        // way.
        if let Some(result) = self
            .tree
            .first_child_with(node, Production::ResultBinding)?
        {
            let mut stack = vec![result];
            while let Some(current) = stack.pop() {
                let carries_region = match self.tree.production(current)? {
                    Production::Mode => !self.has_fixed(current, crate::FixedTerminal::Own)?,
                    Production::Type => {
                        self.has_fixed(current, crate::FixedTerminal::Slice)?
                            || self.has_fixed(current, crate::FixedTerminal::Arena)?
                    }
                    _ => false,
                };
                if carries_region && !self.writes_region(current)? {
                    return self.issue_node(
                        SemanticRule::Form8,
                        current,
                        SemanticIssueKind::RegionSpelling {
                            mechanical_fix: "write this result region: name the parameter region \
the result shares, or a region parameter of its own that the caller supplies",
                        },
                    );
                }
                stack.extend_from_slice(self.tree.children(current)?);
            }
        }
        let input_names: Vec<&str> = inputs.iter().map(|(_, name)| name.as_str()).collect();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for (_, name) in inputs.iter().chain(outputs.iter()) {
            *counts.entry(name.as_str()).or_default() += 1;
        }
        let mut order: Vec<String> = Vec::new();
        for (position, name) in inputs.iter().chain(outputs.iter()) {
            let related = counts.get(name.as_str()).copied().unwrap_or_default() >= 2;
            let caller_chosen = !input_names.contains(&name.as_str());
            if !related && !caller_chosen {
                return self.issue_node(
                    SemanticRule::Form8,
                    *position,
                    SemanticIssueKind::RegionSpelling {
                        mechanical_fix: "drop the region name: no other position of this \
declaration names this region, so the position denotes one region of its own",
                    },
                );
            }
            if !order.iter().any(|written| written == name) {
                order.push(name.clone());
            }
        }
        let declared = match self.tree.first_child_with(node, Production::RegionParams)? {
            Some(list) => self
                .written_regions_below(list)?
                .into_iter()
                .map(|(_, name)| name)
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };
        if declared != order {
            let at = self
                .tree
                .first_child_with(node, Production::RegionParams)?
                .unwrap_or(node);
            return self.issue_node(
                SemanticRule::Form8,
                at,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "the region parameter list holds exactly the region names \
this declaration writes, once each, in the order of their first written occurrence, and is \
absent when it writes none",
                },
            );
        }
        Ok(())
    }

    /// Whether one region spelling occurs anywhere below `root`, ignoring the
    /// `region_stmt` binder at `root` itself.
    pub(super) fn region_is_referenced_below(
        &self,
        root: NodeId,
        spelling: &str,
    ) -> Result<bool, CheckStop> {
        let mut stack = self.tree.children(root)?.to_vec();
        while let Some(node) = stack.pop() {
            if let Some(terminal) = self
                .tree
                .direct_token_with(node, TerminalPredicate::RegionIdentifier)?
                && std::str::from_utf8(self.tree.token_bytes(terminal)?)
                    .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?
                    == spelling
            {
                return Ok(true);
            }
            stack.extend_from_slice(self.tree.children(node)?);
        }
        Ok(false)
    }

    /// Whether one `region_stmt` is the whole body of the loop enclosing it.
    ///
    /// [OWN-11] gives every loop body one region over exactly that body, so a
    /// block that is the body's only statement has exactly that block and is a
    /// second spelling of that one region [FORM-8]. A block the body writes
    /// any other statement beside has a strictly smaller block, which
    /// [OWN-6]'s statement-scope judgment tells apart from the body's own.
    pub(super) fn region_block_is_the_loop_body(&self, node: NodeId) -> Result<bool, CheckStop> {
        let Some(statement) = self.tree.parent(node)? else {
            return Ok(false);
        };
        if self.tree.production(statement)? != Production::Stmt {
            return Ok(false);
        }
        let Some(owner) = self.tree.parent(statement)? else {
            return Ok(false);
        };
        if !matches!(
            self.tree.production(owner)?,
            Production::LoopStmt | Production::ForStmt
        ) {
            return Ok(false);
        }
        Ok(self.tree.children_with(owner, Production::Stmt)?.as_slice() == [statement])
    }

    /// Whether one region name is written at a `targ` region argument below
    /// this node.
    ///
    /// [FORM-8] a retained-argument table operation and an undetermined callee
    /// region parameter are the only in-body positions that must carry a
    /// written REGIONID, and no implicit loop-body region has a name to carry
    /// there.
    pub(super) fn region_is_type_argument_below(
        &self,
        root: NodeId,
        spelling: &str,
    ) -> Result<bool, CheckStop> {
        let mut stack = self.tree.children(root)?.to_vec();
        while let Some(node) = stack.pop() {
            if self.tree.production(node)? == Production::Targ
                && let Some(terminal) = self
                    .tree
                    .direct_token_with(node, TerminalPredicate::RegionIdentifier)?
                && std::str::from_utf8(self.tree.token_bytes(terminal)?)
                    .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?
                    == spelling
            {
                return Ok(true);
            }
            stack.extend_from_slice(self.tree.children(node)?);
        }
        Ok(false)
    }

    pub(super) fn parse_mode(&self, node: NodeId) -> Result<CheckedMode, CheckStop> {
        if self.has_fixed(node, crate::FixedTerminal::Own)? {
            return Ok(CheckedMode::Own);
        }
        let declaration = if self.writes_region(node)? {
            let usage = self.use_at(node, LexicalUseRole::ModeRegion)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            declaration
        } else {
            self.region_declared_at(node)?
        };
        Ok(if self.has_fixed(node, crate::FixedTerminal::Uniq)? {
            CheckedMode::Unique(declaration)
        } else {
            CheckedMode::Shared(declaration)
        })
    }

    /// Whether `&'r T` / `&uniq 'r T` is carried for this `T`.
    ///
    /// [OWN-2] restricts no type, so this states what the checker, lowering,
    /// and backend carry today rather than a language rule: every directly
    /// stored value, plus the descriptor and opaque-handle types that are
    /// already their own borrow. `array` content and an unsubstituted generic
    /// stay explicitly unsupported instead of being misreported as invalid
    /// source.
    pub(super) fn borrowable_type(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        Ok(match ty {
            CheckedType::Buffer { .. } | CheckedType::Slice { .. } => true,
            // A store-resident run is a descriptor and a provider is its own
            // handle, so each is already the thing a borrow carries; a
            // frame-resident run is inline storage, so a borrow of it is the
            // address of that storage, exactly as a borrow of a struct is
            // [BLK-1, PROV-1]. [BLK-4] refuses only the `&uniq` of a run, so
            // the shared borrow is an ordinary one.
            CheckedType::Vector { .. } | CheckedType::Heap { .. } | CheckedType::Extent { .. } => {
                true
            }
            CheckedType::FixedVector { .. } => true,
            CheckedType::Nominal(nominal) => matches!(
                self.nominal(nominal)?.kind,
                CheckedNominalKind::Struct { .. }
                    | CheckedNominalKind::Enum { .. }
                    | CheckedNominalKind::Box { .. }
                    | CheckedNominalKind::SystemResource { .. }
            ),
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => true,
            CheckedType::Array { .. }
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_) => false,
        })
    }

    /// Whether a borrow of this type is the address of the borrowed storage.
    ///
    /// A `buffer` or `slice` value is already a descriptor, and a `box` or
    /// system-resource value is already its own borrow, so only directly
    /// stored content — scalars, structs, and enums — needs a stable address
    /// for reads and writes through the holder [OWN-5, TYPE-7].
    pub(super) fn borrow_addresses_storage(&self, ty: CheckedType) -> Result<bool, CheckStop> {
        Ok(match ty {
            CheckedType::Nominal(nominal) => matches!(
                self.nominal(nominal)?.kind,
                CheckedNominalKind::Struct { .. } | CheckedNominalKind::Enum { .. }
            ),
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => true,
            // A `Heap` or an `Arena` is a stored proof-only value whose
            // cursor state a `&uniq` holder writes, so its borrow addresses
            // that storage. Both runs [BLK-1] are borrowed the same way: an
            // inline run is storage in its owner and a store-resident run's
            // descriptor is storage in its owner's frame, so each borrow is
            // the address of the run's own storage. That is one borrow path
            // for the two runs rather than one shape each, and [BLK-4]
            // refuses the `&uniq` of either, so no borrow of a run writes
            // through it.
            CheckedType::Heap { .. }
            | CheckedType::Extent { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. } => true,
            CheckedType::Buffer { .. }
            | CheckedType::Slice { .. }
            | CheckedType::Array { .. }
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_) => false,
        })
    }

    pub(super) fn parameter_borrow(&self, parameter: &ParameterSignature) -> Option<BorrowInfo> {
        let (kind, region) = match parameter.mode {
            CheckedMode::Own => return None,
            CheckedMode::Shared(region) => (BorrowKind::Shared, region),
            CheckedMode::Unique(region) => (BorrowKind::Unique, region),
        };
        Some(BorrowInfo {
            kind,
            region,
            place: ResolvedPlace {
                root: parameter.declaration,
                fields: Vec::new(),
            },
            origin_region: Some(region),
        })
    }

    pub(super) fn parameter_slice(&self, parameter: &ParameterSignature) -> Option<SliceInfo> {
        let CheckedType::Slice { region, .. } = parameter.ty else {
            return None;
        };
        Some(SliceInfo {
            region,
            origins: vec![CheckedSliceOrigin::FormalSlice {
                parameter: parameter.declaration,
                region,
            }],
        })
    }

    pub(super) fn check_borrow(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        position: ReborrowPosition,
    ) -> Result<TypedExpression, CheckStop> {
        let carrier = self
            .tree
            .parent(node)?
            .filter(|parent| self.tree.production(*parent) == Ok(Production::Atom))
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let Some(region) = self.borrow_expr_region(node)? else {
            return self.issue_node(
                SemanticRule::Form8,
                node,
                SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "write the region this borrow takes, or place the borrow \
inside the `region` block whose region it takes",
                },
            );
        };
        let kind = if self.has_fixed(node, crate::FixedTerminal::Uniq)? {
            BorrowKind::Unique
        } else {
            BorrowKind::Shared
        };
        let place_node = self
            .tree
            .first_child_with(node, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase = self
            .tree
            .first_child_with(place_node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // A borrow of a subscripted place views one storage element; this
        // version's borrows view whole bindings and field projections only.
        for suffix in self.tree.children_with(place_node, Production::Psuffix)? {
            if self.subscript_offset(suffix)?.is_some() {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
            }
        }
        if self.has_fixed(pbase, crate::FixedTerminal::Deref)? {
            // [OWN-14] defines a reborrow form by its root binding's *mode*,
            // not by the `deref` spelling: only a place rooted at a borrow
            // holder is one. A `deref` over an own-mode `box` or `arena`
            // binding reaches content that binding owns [STOR-1], so the
            // borrow is an ordinary borrow judged by [OWN-10]'s own-mode and
            // arena-content cases. Dispatching it as a reborrow demanded a
            // borrow holder the source never wrote and reported spec-legal
            // programs as OWN-6/OWN-14/TYPE-7 violations.
            if let Some(root) = self.owned_content_deref_root(pbase, bindings)? {
                return self.check_owned_content_borrow(
                    node, place_node, region, function, loop_depth, root,
                );
            }
            return self.check_child_reborrow(
                node, place_node, pbase, region, kind, bindings, loop_depth, position,
            );
        }
        if !self.borrow_region_is_inside_current_loops(region, node, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                node,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                },
            );
        }
        if !self.tree.children(pbase)?.is_empty() {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, pbase);
        }
        let root_use = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = root_use.target()
        else {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        };
        let local = bindings
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if local.mode != CheckedMode::Own {
            // A `borrow_expr` whose place roots at a borrow-mode binding is a
            // reborrow form; outside call-argument position it is OWN-14's
            // hard error. In argument position the deref-free spelling stays
            // outside OWN-6's closed written form, an explicit capability gap.
            if !matches!(position, ReborrowPosition::CallArgument { .. }) {
                return self.issue_node(
                    SemanticRule::Own14,
                    node,
                    SemanticIssueKind::InvalidReborrowPosition {
                        mechanical_fix: OWN14_RESTRUCTURING,
                    },
                );
            }
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                place_node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        if local.compiler_updated && kind == BorrowKind::Unique {
            return self.issue_node(SemanticRule::Own11, node, SemanticIssueKind::BorrowConflict);
        }
        if function.region_parameters.contains(&region)
            || !self.scope_is_within(
                self.region_declaration(region)?.scope(),
                self.declaration_scope(local.declaration)?,
            )?
        {
            return self.issue_node(
                SemanticRule::Own10,
                node,
                SemanticIssueKind::InvalidBorrowLifetime {
                    region: self.region_phrase(region)?,
                    binder: self.declaration_spelling(local.declaration)?,
                    mechanical_fix: OWN10_LOCAL_STORAGE.to_owned(),
                },
            );
        }
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        let place = ResolvedPlace {
            root: declaration,
            fields: fields.clone(),
        };
        self.check_loan_access(
            bindings,
            None,
            &place,
            match kind {
                BorrowKind::Shared => AccessKind::SharedBorrow,
                BorrowKind::Unique => AccessKind::UniqueBorrow,
            },
            node,
        )?;
        let borrow = BorrowInfo {
            kind,
            region,
            place,
            origin_region: None,
        };
        let slice = local.slice.clone();
        let expression = match ty {
            CheckedType::Buffer { element } => CheckedExpression::BorrowBuffer {
                carrier: self.tree.path(carrier)?.clone(),
                root: CheckedBufferRoot {
                    binding: local.binding,
                    fields,
                    element,
                },
            },
            CheckedType::Nominal(nominal)
                if fields.is_empty()
                    && matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Box { .. }) =>
            {
                CheckedExpression::BorrowBox {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    nominal,
                }
            }
            // An opaque resource value is its own borrow, whether the
            // binding is that resource or a field of one. A system struct's
            // `receive` and `send` are ordinary field places [SYS-18], so a
            // borrow of one carries the field path and nothing else changes:
            // the loan `place` above already names the field, so [OWN-5]
            // decides two loans on disjoint fields exactly as it does for a
            // source struct.
            CheckedType::Nominal(nominal)
                if matches!(
                    self.nominal(nominal)?.kind,
                    CheckedNominalKind::SystemResource { .. }
                ) =>
            {
                CheckedExpression::BorrowSystemResource {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    fields: fields.clone(),
                    state_origins: local.state_origins.clone(),
                    nominal,
                }
            }
            CheckedType::Slice { .. } if fields.is_empty() => CheckedExpression::Binding {
                carrier: self.tree.path(carrier)?.clone(),
                binding: local.binding,
                state_origins: local.state_origins.clone(),
                ty,
                slice_origins: slice
                    .as_ref()
                    .map(|slice| slice.origins.clone())
                    .unwrap_or_default(),
                consume_root: false,
            },
            _ if fields.is_empty() && self.borrow_addresses_storage(ty)? => {
                CheckedExpression::BorrowAddressed {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    ty,
                }
            }
            _ => {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
            }
        };
        Ok(TypedExpression {
            expression,
            mode: borrow.mode(),
            borrow: Some(borrow),
            slice,
            holder: None,
            // A `borrow_expr` is a reference value, never the referent
            // [TYPE-7, GRAM-5].
            reference_value: true,
            effects: EffectSet::NONE,
            accesses: Vec::new(),
        })
    }

    /// The own-mode `box` or `arena` binding a `deref` place base is rooted
    /// at, when it is one. `None` means the base is not that shape — a
    /// borrow-holder root, a chained or suffixed holder place, or a nonvalue
    /// target — and the position keeps its holder disposition: [OWN-14]'s
    /// reborrow judgment for a borrow, and [SET-1]'s live usable `&uniq`
    /// referent for a mutation target.
    pub(super) fn owned_content_deref_root(
        &self,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<Option<(LocalBinding, OwnedContent)>, CheckStop> {
        let root_place = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let root_base = self
            .tree
            .first_child_with(root_place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.tree.children(root_base)?.is_empty()
            || !self
                .tree
                .children_with(root_place, Production::Psuffix)?
                .is_empty()
        {
            return Ok(None);
        }
        let usage = self.use_at(root_base, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(None);
        };
        let Some(local) = bindings.get(&declaration).cloned() else {
            return Ok(None);
        };
        if local.mode != CheckedMode::Own || local.borrow.is_some() {
            return Ok(None);
        }
        let CheckedType::Nominal(nominal) = local.ty else {
            return Ok(None);
        };
        let content = match self.nominal(nominal)?.kind {
            CheckedNominalKind::Box { referent, .. } => OwnedContent::Boxed(referent),
            CheckedNominalKind::Arena { region, content } => OwnedContent::Arena {
                source: region,
                content,
            },
            _ => return Ok(None),
        };
        Ok(Some((local, content)))
    }

    /// An ordinary borrow whose place reaches through owned indirection. The
    /// judgment order is [OWN-11]'s loop restriction, [OWN-1]'s liveness, then
    /// [OWN-10]'s storage-duration case for the root's storage class, exactly
    /// as for a borrow of the binding itself; only the region relation
    /// differs, because arena content outlives its region rather than its
    /// binding [STOR-4].
    fn check_owned_content_borrow(
        &self,
        node: NodeId,
        place_node: NodeId,
        region: DeclarationId,
        function: &FunctionSignature,
        loop_depth: usize,
        root: (LocalBinding, OwnedContent),
    ) -> Result<TypedExpression, CheckStop> {
        let (local, content) = root;
        if !self.borrow_region_is_inside_current_loops(region, node, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                node,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                },
            );
        }
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                place_node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        let admitted = match content {
            OwnedContent::Arena { source, .. } => self.region_outlives(source, region)?,
            OwnedContent::Boxed(_) => {
                !function.region_parameters.contains(&region)
                    && self.scope_is_within(
                        self.region_declaration(region)?.scope(),
                        self.declaration_scope(local.declaration)?,
                    )?
            }
        };
        if !admitted {
            return self.issue_node(
                SemanticRule::Own10,
                node,
                SemanticIssueKind::InvalidBorrowLifetime {
                    region: self.region_phrase(region)?,
                    binder: self.declaration_spelling(local.declaration)?,
                    mechanical_fix: OWN10_LOCAL_STORAGE.to_owned(),
                },
            );
        }
        // The written suffix chain still selects a real field of the content
        // type, so a wrong spelling stays a source rejection rather than being
        // masked by the capability stop below [DIAG-1].
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let (_fields, _ty) = self.resolve_struct_path(&suffixes, content.ty())?;
        // TEMPORARY capability stop, judged after the [OWN-1], [OWN-10], and
        // [OWN-11] source rejections above. No checked expression addresses
        // owned indirection content: a `box` binding lowers to the content
        // pointer with the box's own IR type and arena storage has no runtime
        // at all, so there is nothing for the IR builder to take the address
        // of. This is the same explicit stop the arena-content `slice_of`
        // path takes rather than publishing an unlowerable checked program.
        match content {
            OwnedContent::Arena { .. } => {
                self.unsupported(UnsupportedSemanticFeature::ArenaRuntime, place_node)
            }
            OwnedContent::Boxed(_) => {
                self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node)
            }
        }
    }

    pub(super) fn check_direct_slice_borrow_lifetime(
        &self,
        function: &FunctionSignature,
        region: DeclarationId,
        owner: Option<DeclarationId>,
        node: NodeId,
        loop_depth: usize,
    ) -> Result<(), CheckStop> {
        if !self.borrow_region_is_inside_current_loops(region, node, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                node,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                },
            );
        }
        let Some(owner) = owner else {
            return Ok(());
        };
        if function.region_parameters.contains(&region)
            || !self.scope_is_within(
                self.region_declaration(region)?.scope(),
                self.declaration_scope(owner)?,
            )?
        {
            return self.issue_node(
                SemanticRule::Own10,
                node,
                SemanticIssueKind::InvalidBorrowLifetime {
                    region: self.region_phrase(region)?,
                    binder: self.declaration_spelling(owner)?,
                    mechanical_fix: OWN10_LOCAL_STORAGE.to_owned(),
                },
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn check_child_reborrow(
        &self,
        node: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        kind: BorrowKind,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        position: ReborrowPosition,
    ) -> Result<TypedExpression, CheckStop> {
        let carrier = self
            .tree
            .parent(node)?
            .filter(|parent| self.tree.production(*parent) == Ok(Production::Atom))
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        match position {
            ReborrowPosition::Forbidden => {
                return self.issue_node(
                    SemanticRule::Own14,
                    node,
                    SemanticIssueKind::InvalidReborrowPosition {
                        mechanical_fix: OWN14_RESTRUCTURING,
                    },
                );
            }
            // A reborrow argument to a borrow-returning call is admitted
            // only in the call's single provenance-candidate position under
            // the reborrow extension; every other borrow-returning receiver
            // keeps OWN-6's own/unit-result condition.
            ReborrowPosition::CallArgument {
                own_result,
                result_candidate,
            } if !own_result && !result_candidate => {
                return self.issue_node(
                    SemanticRule::Own6,
                    node,
                    SemanticIssueKind::InvalidChildReborrow {
                        mechanical_fix: OWN6_ARGUMENT_POSITION,
                    },
                );
            }
            ReborrowPosition::CallArgument { .. } | ReborrowPosition::ReturnExpression => {}
        }
        if !self.borrow_region_is_inside_current_loops(region, node, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                node,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the child region inside the enclosing loop body",
                },
            );
        }
        if let ReborrowPosition::CallArgument {
            result_candidate, ..
        } = position
        {
            // In the provenance-candidate position of a borrow-returning
            // call, the child's loan survives in the bound result, so the
            // statement-scoped-region condition is replaced by the parent's
            // permanent suspension [OWN-6]; a caller-supplied region is
            // admitted there because the claim is carried by the result
            // holder, never by a resumed parent. Every other argument child
            // stays statement-scoped.
            if !result_candidate {
                let region_declaration = self.region_declaration(region)?;
                if region_declaration.role() != DeclarationRole::LocalRegion
                    || !self.child_region_is_statement_scoped(region_declaration, node)?
                {
                    return self.issue_node(
                        SemanticRule::Own6,
                        node,
                        SemanticIssueKind::InvalidChildReborrow {
                            mechanical_fix: OWN6_STATEMENT_SCOPE,
                        },
                    );
                }
            }
        }
        let (holder, local, parent) = self.resolve_dereference_holder(node, pbase, bindings)?;
        // No reborrow is created through a holder [OWN-13] suspended: its
        // live arm-scoped children's loans overlap every place it reaches,
        // so the pair would be both usable [OWN-5].
        self.check_holder_not_suspended(&local, node)?;
        let holder_role = self.declaration_record(holder)?.role();
        match position {
            ReborrowPosition::Forbidden => unreachable!("rejected above"),
            ReborrowPosition::CallArgument { .. } => {
                if !matches!(
                    holder_role,
                    DeclarationRole::Parameter | DeclarationRole::Let
                ) || (kind == BorrowKind::Unique && parent.kind != BorrowKind::Unique)
                    || !self.region_outlives(parent.region, region)?
                {
                    return self.issue_node(
                        SemanticRule::Own6,
                        node,
                        SemanticIssueKind::InvalidChildReborrow {
                            mechanical_fix: OWN6_HOLDER,
                        },
                    );
                }
            }
            ReborrowPosition::ReturnExpression => {
                // [OWN-10]'s borrow-rooted case is the creation obligation and
                // is defined before OWN-14, so its violation is cited first at
                // this node [DIAG-1].
                if !self.region_outlives(parent.region, region)? {
                    return self.issue_node(
                        SemanticRule::Own10,
                        node,
                        SemanticIssueKind::InvalidBorrowLifetime {
                            region: self.region_phrase(region)?,
                            binder: self.declaration_spelling(holder)?,
                            // The holder's own region is what a legal
                            // reborrow names. A region [FORM-8] leaves
                            // unwritten has no name to give.
                            mechanical_fix: match self.written_region_name(parent.region)? {
                                Some(name) => format!(
                                    "a returned child reborrow names a region its holder's own \
region {name} outlives; name {name} itself, or a region {name} outlives, on the returned reborrow"
                                ),
                                None => "a returned child reborrow names a region its holder's \
own region outlives; that region is unwritten here, so relate the holder's region to this result \
and name it on the returned reborrow"
                                    .to_owned(),
                            },
                        },
                    );
                }
                // [OWN-14] admission: a parameter or let-bound holder, never a
                // match binder, and mode preserved in both directions.
                if !matches!(
                    holder_role,
                    DeclarationRole::Parameter | DeclarationRole::Let
                ) || kind != parent.kind
                {
                    return self.issue_node(
                        SemanticRule::Own14,
                        node,
                        SemanticIssueKind::InvalidReborrowPosition {
                            mechanical_fix: OWN14_RESTRUCTURING,
                        },
                    );
                }
            }
        }
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        let mut place = parent.place.clone();
        place.fields.extend_from_slice(&fields);
        self.check_loan_access(
            bindings,
            Some(holder),
            &place,
            match kind {
                BorrowKind::Shared => AccessKind::SharedBorrow,
                BorrowKind::Unique => AccessKind::UniqueBorrow,
            },
            node,
        )?;
        let expression = match ty {
            CheckedType::Buffer { element } => CheckedExpression::BorrowBuffer {
                carrier: self.tree.path(carrier)?.clone(),
                root: CheckedBufferRoot {
                    binding: local.binding,
                    fields,
                    element,
                },
            },
            // An opaque resource value is its own borrow, so a child reborrow
            // of a borrow-mode holder is that same inline value: there is no
            // content to address and nothing to reload [SYS-2, OWN-6].
            CheckedType::Nominal(nominal)
                if matches!(
                    self.nominal(nominal)?.kind,
                    CheckedNominalKind::SystemResource { .. }
                ) =>
            {
                CheckedExpression::BorrowSystemResource {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    fields: fields.clone(),
                    state_origins: local.state_origins.clone(),
                    nominal,
                }
            }
            // A view value is already a descriptor, so the child reborrow a
            // helper takes of its own view holder is that same descriptor
            // read once more: there is no content to address and nothing to
            // reload, exactly as a system-resource holder's child is
            // [OWN-6, VIEW-1]. The child carries the parent's range and its
            // loan region, and [OWN-6]'s ordinary suspension freezes the
            // holder while the child lives [OWN-5].
            CheckedType::Slice { .. } if fields.is_empty() => CheckedExpression::Binding {
                carrier: self.tree.path(carrier)?.clone(),
                binding: local.binding,
                state_origins: local.state_origins.clone(),
                ty,
                slice_origins: local
                    .slice
                    .as_ref()
                    .map(|slice| slice.origins.clone())
                    .unwrap_or_default(),
                consume_root: false,
            },
            _ if fields.is_empty() && self.borrow_addresses_storage(ty)? => {
                CheckedExpression::ReborrowAddressed {
                    carrier: self.tree.path(carrier)?.clone(),
                    binding: local.binding,
                    ty,
                }
            }
            _ => {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
            }
        };
        let borrow = BorrowInfo {
            kind,
            region,
            place,
            origin_region: parent.origin_region,
        };
        // The reborrowed descriptor reaches the same storage its parent does,
        // so the child's origin set is the parent's own [VIEW-2].
        let slice = matches!(ty, CheckedType::Slice { .. })
            .then(|| local.slice.clone())
            .flatten();
        Ok(TypedExpression {
            expression,
            mode: borrow.mode(),
            borrow: Some(borrow),
            slice,
            holder: Some(holder),
            reference_value: true,
            effects: EffectSet::NONE,
            accesses: Vec::new(),
        })
    }

    fn child_region_is_statement_scoped(
        &self,
        region: &crate::DeclarationRecord,
        child: NodeId,
    ) -> Result<bool, CheckStop> {
        let Some(region_node) = self.tree.node_with_path(region.origin().node()) else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        // [OWN-3] a `region_stmt` body and a loop body are both blocks a local
        // region is introduced over, so both answer whether that block extends
        // beyond the enclosing statement [OWN-6, OWN-11].
        if !matches!(
            self.tree.production(region_node)?,
            Production::RegionStmt | Production::LoopStmt | Production::ForStmt
        ) {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let region_statements = self.tree.children_with(region_node, Production::Stmt)?;
        let [region_statement] = region_statements.as_slice() else {
            return Ok(false);
        };
        let mut cursor = Some(child);
        while let Some(node) = cursor {
            if self.tree.production(node)? == Production::Stmt {
                return Ok(node == *region_statement);
            }
            cursor = self.tree.parent(node)?;
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    pub(in crate::semantic::check) fn borrow_region_is_inside_current_loops(
        &self,
        region: DeclarationId,
        borrow: NodeId,
        loop_depth: usize,
    ) -> Result<bool, CheckStop> {
        if loop_depth == 0 {
            return Ok(true);
        }
        let declaration = self.region_declaration(region)?;
        let Some(region_node) = self.tree.node_with_path(declaration.origin().node()) else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let borrow_loops = self.enclosing_loops(borrow)?;
        if borrow_loops.len() != loop_depth {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let mut region_loops = self.enclosing_loops(region_node)?;
        // [OWN-11] a loop body's own region is introduced inside that body, so
        // the loop owning it counts as one of the loops enclosing the region
        // even though the node minting it is the loop node itself.
        if self.loop_body_region_owner(region)?.is_some() {
            region_loops.push(region_node);
        }
        Ok(region_loops == borrow_loops)
    }

    fn enclosing_loops(&self, node: NodeId) -> Result<Vec<NodeId>, CheckStop> {
        let mut loops = Vec::new();
        let mut child = node;
        let mut cursor = self.tree.parent(node)?;
        while let Some(ancestor) = cursor {
            let production = self.tree.production(ancestor)?;
            if production == Production::LoopStmt
                || (production == Production::ForStmt
                    && self.tree.production(child)? == Production::Stmt)
            {
                loops.push(ancestor);
            }
            child = ancestor;
            cursor = self.tree.parent(ancestor)?;
        }
        loops.reverse();
        Ok(loops)
    }

    pub(super) fn resolve_dereference_holder(
        &self,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(DeclarationId, LocalBinding, BorrowInfo), CheckStop> {
        let holder_place = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let holder_base = self
            .tree
            .first_child_with(holder_place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.tree.children(holder_base)?.is_empty()
            || !self
                .tree
                .children_with(holder_place, Production::Psuffix)?
                .is_empty()
        {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, holder_place);
        }
        let usage = self.use_at(holder_base, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                holder_place,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        let Some(borrow) = local.borrow.clone() else {
            return self.issue_node(
                SemanticRule::Type7,
                node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "deref requires a borrow holder",
                },
            );
        };
        Ok((declaration, local, borrow))
    }

    /// A suspended holder's own allowance is withdrawn: nothing is read,
    /// written, borrowed, or committed through it while its arm-scoped
    /// children live, which is the remainder of its region [OWN-5, OWN-13].
    /// Each position asks after its own earlier-defined judgments so the
    /// same-node citation keeps DIAG-1's first-definition rank.
    pub(super) fn check_holder_not_suspended(
        &self,
        local: &LocalBinding,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        if local.suspended {
            return self.issue_node(SemanticRule::Own5, node, SemanticIssueKind::BorrowConflict);
        }
        Ok(())
    }

    /// [TYPE-7]'s implicit read, asked once by each position that requires a
    /// referent value rather than the holder that reaches it.
    ///
    /// The rule is one rule, but the two holder shapes are represented
    /// differently here, so the predicate takes both facts. A borrow-mode
    /// value already carries its referent's checked type, so only its
    /// provenance — it was not written through `deref` — separates the holder
    /// from the referent. A `box` binding carries the holder's own type, so
    /// the question is what its referent is. Either way the answer is the
    /// same rejection with the same `deref(.)` fix, and the position's own
    /// wrong-type judgment forms no rejection.
    pub(super) fn reads_implicitly_through_holder(
        &self,
        holds_reference: bool,
        ty: CheckedType,
        required: RequiredReferent,
    ) -> Result<bool, CheckStop> {
        if holds_reference {
            return self.satisfies_requirement(ty, required);
        }
        let CheckedType::Nominal(nominal) = ty else {
            return Ok(false);
        };
        let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind else {
            return Ok(false);
        };
        self.satisfies_requirement(referent, required)
    }

    fn satisfies_requirement(
        &self,
        ty: CheckedType,
        required: RequiredReferent,
    ) -> Result<bool, CheckStop> {
        Ok(match required {
            RequiredReferent::Exact(required) => ty == required,
            RequiredReferent::Enum => match ty {
                CheckedType::Bool => true,
                CheckedType::Nominal(nominal) => {
                    matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Enum { .. })
                }
                _ => false,
            },
            // [OP-4, BLK-1] the two runs are indexable bases exactly as the
            // three flat storages are, so a run holder written where its
            // referent is required is the same [TYPE-7] missing `deref`.
            RequiredReferent::IndexableStorage => matches!(
                ty,
                CheckedType::Array { .. }
                    | CheckedType::Buffer { .. }
                    | CheckedType::Slice { .. }
                    | CheckedType::FixedVector { .. }
                    | CheckedType::Vector { .. }
            ),
        })
    }

    pub(super) fn borrow_for_destination(
        &self,
        destination: CheckedMode,
        value: &TypedExpression,
        node: NodeId,
    ) -> Result<Option<BorrowInfo>, CheckStop> {
        if destination == CheckedMode::Own {
            if value.mode == CheckedMode::Own {
                return Ok(None);
            }
            return self.issue_node(
                SemanticRule::Type7,
                node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        let Some(mut borrow) = value.borrow.clone() else {
            return self.issue_node(
                SemanticRule::Type5,
                node,
                SemanticIssueKind::type_mismatch(
                    self.checked_mode_name(destination)?,
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
            );
        };
        let destination_region = match (destination, borrow.kind) {
            (CheckedMode::Shared(region), BorrowKind::Shared)
            | (CheckedMode::Unique(region), BorrowKind::Unique) => region,
            _ => {
                return self.issue_node(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::type_mismatch(
                        self.checked_mode_name(destination)?,
                        format!(
                            "{} {}",
                            match borrow.kind {
                                BorrowKind::Shared => "&",
                                BorrowKind::Unique => "&uniq",
                            },
                            self.region_phrase(borrow.region)?
                        ),
                    ),
                );
            }
        };
        if !self.region_outlives(borrow.region, destination_region)? {
            return self.issue_node(
                SemanticRule::Own4,
                node,
                SemanticIssueKind::InvalidBorrowLifetime { region: self.region_phrase(destination_region)?, binder: self.declaration_spelling(borrow.place.root)?, mechanical_fix: format!("the value's borrow is live for {}, and {} is not inside it; store or pass it under a region {} outlives, or introduce {} inside {}'s block", self.region_phrase(borrow.region)?, self.region_phrase(destination_region)?, self.region_phrase(borrow.region)?, self.region_phrase(destination_region)?, self.region_phrase(borrow.region)?) },
            );
        }
        borrow.region = destination_region;
        Ok(Some(borrow))
    }

    /// A borrow-mode `let` holder is supported when its lexical scope lies
    /// within its borrow's region block, so the borrow value [OWN-4] is live
    /// for the holder's whole scope. The region may enclose the holder any
    /// number of blocks up (an outer region's borrow legally stored under an
    /// inner region), and a caller-supplied region encloses the entire body
    /// [OWN-3]. A holder that would outlive its borrow's region stays an
    /// explicit capability stop.
    pub(super) fn borrow_holder_scope_supported(
        &self,
        holder: DeclarationId,
        mode: CheckedMode,
    ) -> Result<bool, CheckStop> {
        let region = match mode {
            CheckedMode::Own => return Ok(true),
            CheckedMode::Shared(region) | CheckedMode::Unique(region) => region,
        };
        self.scope_is_within(
            self.declaration_scope(holder)?,
            self.region_declaration(region)?.scope(),
        )
    }

    /// Whether `declaration`'s owning lexical scope lies within `region`'s
    /// block — the [STOR-4] destination judgment for a value confined to that
    /// region. A caller-supplied region's block encloses the whole body.
    pub(super) fn declaration_is_within_region_block(
        &self,
        declaration: DeclarationId,
        region: DeclarationId,
    ) -> Result<bool, CheckStop> {
        self.scope_is_within(
            self.declaration_scope(declaration)?,
            self.region_declaration(region)?.scope(),
        )
    }

    pub(super) fn check_loan_access(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        through_holder: Option<DeclarationId>,
        place: &ResolvedPlace,
        access: AccessKind,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        // The place a suspended ancestor's loan does not confiscate: an
        // access through a holder whose own borrow extends that loan's place
        // is the arm-scoped child's use, and the child's narrower loan
        // carries the exclusivity while the ancestor is not usable
        // [OWN-5, OWN-13].
        let through_place = through_holder
            .and_then(|holder| bindings.get(&holder))
            .and_then(|holder| holder.borrow.as_ref())
            .map(|borrow| &borrow.place);
        for (declaration, local) in bindings {
            if let Some(loan) = &local.borrow
                && Some(*declaration) != through_holder
                && places_overlap(&loan.place, place)
            {
                let suspended_ancestor = local.suspended
                    && through_place.is_some_and(|child| {
                        child.root == loan.place.root
                            && child.fields.starts_with(&loan.place.fields)
                    });
                let conflicts = match access {
                    AccessKind::Read => loan.kind == BorrowKind::Unique,
                    AccessKind::Write | AccessKind::Move | AccessKind::UniqueBorrow => true,
                    AccessKind::SharedBorrow => loan.kind == BorrowKind::Unique,
                };
                if conflicts && !suspended_ancestor {
                    return self.issue_node(
                        SemanticRule::Own5,
                        node,
                        SemanticIssueKind::BorrowConflict,
                    );
                }
            }
            for loan in &local.slice_loans {
                if places_overlap(&loan.place, place)
                    && loan.refuses(access)
                    && self.slice_loan_is_live(loan, bindings, node)?
                {
                    return self.issue_node(
                        SemanticRule::Own5,
                        node,
                        SemanticIssueKind::BorrowConflict,
                    );
                }
            }
        }
        // [S31, OWN-5] the freeze a shared child reborrow puts on its parent
        // reaches the parent's *holder*, not only its elements.
        //
        // The loans above are claims on the origin storage, and `&uniq
        // writer` names the descriptor rather than that storage, so no claim
        // on the origin sees it. But a `&uniq` of an exclusive view is
        // exactly the borrow through which an element write of the frozen
        // range is made, and a `move` of one hands that write to a callee
        // outright; both are the access the element write already is, taken
        // one indirection earlier. Refusing them here is what makes the
        // freeze a property of the loan rather than of the one statement
        // form that happened to check it.
        if matches!(access, AccessKind::UniqueBorrow | AccessKind::Move) {
            let frozen: Vec<ResolvedPlace> = bindings
                .values()
                .flat_map(|local| local.slice_loans.iter())
                .filter(|loan| {
                    loan.strength == LoanStrength::Exclusive
                        && loan.descriptors.contains(&place.root)
                })
                .map(|loan| loan.place.clone())
                .collect();
            if !frozen.is_empty() {
                self.check_child_reborrow_freeze_at(
                    bindings,
                    &frozen,
                    node,
                    Simultaneity::OneStatementIsOneMoment,
                )?;
            }
        }
        Ok(())
    }

    /// [S31, PROV-3] the freeze a shared child reborrow puts on its parent:
    /// an element write through an exclusive view is refused while a shared
    /// view of the same storage is live.
    ///
    /// The write reaches the origin through the parent's own data pointer, so
    /// the parent's own exclusive loan is not the loan to ask about — this is
    /// the one access an exclusive view's holder makes that its own loan
    /// cannot answer. The only shared loan that can stand on storage an
    /// exclusive view already views is a child reborrow of that view, so the
    /// question is exactly whether such a child is still live.
    pub(super) fn check_child_reborrow_freeze(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        origins: &[ResolvedPlace],
        node: NodeId,
    ) -> Result<(), CheckStop> {
        self.check_child_reborrow_freeze_at(bindings, origins, node, Simultaneity::Sequential)
    }

    /// [S31, PROV-3] the freeze, with the caller stating how it reads a use
    /// written in the same statement as the access.
    fn check_child_reborrow_freeze_at(
        &self,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        origins: &[ResolvedPlace],
        node: NodeId,
        simultaneity: Simultaneity,
    ) -> Result<(), CheckStop> {
        for local in bindings.values() {
            for loan in &local.slice_loans {
                if loan.strength == LoanStrength::Shared
                    && origins
                        .iter()
                        .any(|origin| places_overlap(&loan.place, origin))
                    && self.slice_loan_is_live_at(loan, bindings, node, simultaneity)?
                {
                    return self.issue_node(
                        SemanticRule::Own5,
                        node,
                        SemanticIssueKind::BorrowConflict,
                    );
                }
            }
        }
        Ok(())
    }

    /// [PROV-3] whether one loan is still live at this access.
    ///
    /// A loan begins where its value is formed or copied and ends where that
    /// value's own liveness ends: for an **affine** view its consume or
    /// release, and for a **copy** view its last use. The affine case keeps
    /// [OWN-4]'s named-region extent, which is the conservative reading of a
    /// consume this checker has no separate program point for; the copy case
    /// is decided here, and is what admits an append to a run after the view
    /// of it went dead.
    ///
    /// A loan no binding holds is live for its region, because the value that
    /// held it was consumed inside its own statement and this checker states
    /// no program point between two operands of one statement.
    fn slice_loan_is_live(
        &self,
        loan: &SliceLoan,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        node: NodeId,
    ) -> Result<bool, CheckStop> {
        self.slice_loan_is_live_at(loan, bindings, node, Simultaneity::Sequential)
    }

    fn slice_loan_is_live_at(
        &self,
        loan: &SliceLoan,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        node: NodeId,
        simultaneity: Simultaneity,
    ) -> Result<bool, CheckStop> {
        if loan.strength != LoanStrength::Shared || loan.descriptors.is_empty() {
            return Ok(true);
        }
        for holder in &loan.descriptors {
            let live = bindings.get(holder).is_none_or(|binding| binding.live);
            if live && self.declaration_is_used_at_or_after(*holder, node, simultaneity)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether one value binding has a use at or after this node [ENT-5].
    ///
    /// Document order is the canonical tree's own child-ordinal path order,
    /// so a use at a later statement — in either arm of a later branch
    /// included — compares greater. Document order is not execution order in
    /// two places, and both keep a loan live.
    ///
    /// A loop body is the first: a use textually before this node but inside
    /// the innermost loop body containing it follows the node on the next
    /// iteration.
    ///
    /// One simple statement is the second: this checker states no program
    /// point between two operands of one statement, which is the very reason
    /// a loan no binding holds keeps its region extent. A use in the same
    /// `let`, `set`, expression or `return` statement as this access is
    /// therefore simultaneous with it and not before it, whichever operand
    /// the canonical order writes first.
    fn declaration_is_used_at_or_after(
        &self,
        declaration: DeclarationId,
        node: NodeId,
        simultaneity: Simultaneity,
    ) -> Result<bool, CheckStop> {
        let here = self.tree.path(node)?.components().to_vec();
        let mut repeated: Option<Vec<u32>> = None;
        let mut simultaneous: Option<Vec<u32>> = None;
        let mut current = self.tree.parent(node)?;
        while let Some(ancestor) = current {
            match self.tree.production(ancestor)? {
                Production::LetStmt
                | Production::SetStmt
                | Production::ExprStmt
                | Production::ReturnStmt
                    if simultaneous.is_none()
                        && simultaneity == Simultaneity::OneStatementIsOneMoment =>
                {
                    simultaneous = Some(self.tree.path(ancestor)?.components().to_vec());
                }
                Production::LoopStmt | Production::ForStmt => {
                    repeated = Some(self.tree.path(ancestor)?.components().to_vec());
                    break;
                }
                Production::FnDecl => break,
                _ => {}
            }
            current = self.tree.parent(ancestor)?;
        }
        for usage in self.resolved.lexical_uses() {
            let ResolvedTarget::Source {
                declaration: target,
                class: DeclarationClass::Value,
            } = usage.target()
            else {
                continue;
            };
            if target != declaration {
                continue;
            }
            let path = usage.origin().node().components();
            if path >= here.as_slice() {
                return Ok(true);
            }
            if simultaneous
                .as_ref()
                .is_some_and(|statement| path.starts_with(statement.as_slice()))
            {
                return Ok(true);
            }
            if repeated
                .as_ref()
                .is_some_and(|body| path.starts_with(body.as_slice()))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn region_declaration(
        &self,
        id: DeclarationId,
    ) -> Result<&crate::DeclarationRecord, CheckStop> {
        self.declaration_record(id)
    }

    fn declaration_record(
        &self,
        id: DeclarationId,
    ) -> Result<&crate::DeclarationRecord, CheckStop> {
        self.resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.id() == id)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    fn declaration_scope(&self, id: DeclarationId) -> Result<ScopeId, CheckStop> {
        self.resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.id() == id)
            .map(|declaration| declaration.scope())
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    fn scope_is_within(&self, mut scope: ScopeId, ancestor: ScopeId) -> Result<bool, CheckStop> {
        loop {
            if scope == ancestor {
                return Ok(true);
            }
            let record = self
                .resolved
                .scopes()
                .get(scope.index())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let Some(parent) = record.parent() else {
                return Ok(false);
            };
            scope = parent;
        }
    }

    pub(super) fn region_outlives(
        &self,
        source: DeclarationId,
        destination: DeclarationId,
    ) -> Result<bool, CheckStop> {
        if source == destination {
            return Ok(true);
        }
        let source = self.region_declaration(source)?;
        let destination = self.region_declaration(destination)?;
        if source.role() == DeclarationRole::RegionParameter {
            return Ok(destination.role() == DeclarationRole::LocalRegion);
        }
        if destination.role() == DeclarationRole::RegionParameter {
            return Ok(false);
        }
        self.scope_is_within(destination.scope(), source.scope())
    }
}

pub(super) fn places_overlap(left: &ResolvedPlace, right: &ResolvedPlace) -> bool {
    left.root == right.root
        && (left.fields.starts_with(&right.fields) || right.fields.starts_with(&left.fields))
}
