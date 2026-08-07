use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, LexicalUseRole, Production, ResolvedTarget,
    ScopeId, SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedBufferRoot, CheckedExpression, CheckedMode, CheckedNominalKind, CheckedSliceOrigin,
    CheckedType,
};
use super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, ParameterSignature,
    TypedExpression,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BorrowKind {
    Shared,
    Unique,
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
    },
    /// The complete `expr` of a `return_stmt` [OWN-14].
    ReturnExpression,
}

/// [OWN-14]'s exact restructuring for a rejected reborrow form.
const OWN14_RESTRUCTURING: &str = "pass the reborrow as a statement-scoped child in argument position, \
     return it as the complete return expression from a parameter or let-bound holder, \
     or return the holder itself";

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
}

/// The value a position requires of its operand, for [TYPE-7]'s implicit-read
/// exclusivity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RequiredReferent {
    /// A `match` scrutinee requires an enum value [OWN-13, ERR-2].
    Enum,
    /// An `index` root requires directly indexable storage [OP-4].
    IndexableStorage,
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

    pub(super) fn effect_regions(&self) -> Vec<DeclarationId> {
        let mut regions = Vec::new();
        for origin in &self.origins {
            let region = match origin {
                CheckedSliceOrigin::SourcePlace { origin_region, .. } => *origin_region,
                CheckedSliceOrigin::FormalSlice { region, .. } => Some(*region),
                CheckedSliceOrigin::ImmutableConst => None,
            };
            if let Some(region) = region
                && !regions.contains(&region)
            {
                regions.push(region);
            }
        }
        regions
    }
}

pub(super) fn push_slice_origin(origins: &mut Vec<CheckedSliceOrigin>, origin: CheckedSliceOrigin) {
    if !origins.contains(&origin) {
        origins.push(origin);
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
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
        let path = self.tree.path(node)?;
        let mut declarations = self
            .resolved
            .declarations()
            .iter()
            .filter(|declaration| {
                declaration.role() == DeclarationRole::RegionParameter
                    && declaration.origin().node() == path
            })
            .collect::<Vec<_>>();
        declarations.sort_by_key(|declaration| declaration.origin().role_ordinal());
        Ok(declarations
            .into_iter()
            .map(|declaration| declaration.id())
            .collect())
    }

    pub(super) fn parse_mode(&self, node: NodeId) -> Result<CheckedMode, CheckStop> {
        if self.has_fixed(node, crate::FixedTerminal::Own)? {
            return Ok(CheckedMode::Own);
        }
        let usage = self.use_at(node, LexicalUseRole::ModeRegion)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Region,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
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
        let usage = self.use_at(node, LexicalUseRole::BorrowRegion)?;
        let ResolvedTarget::Source {
            declaration: region,
            class: DeclarationClass::Region,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
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
        if self.has_fixed(pbase, crate::FixedTerminal::Deref)? {
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
        if function.region_parameters.contains(&region)
            || !self.scope_is_within(
                self.region_declaration(region)?.scope(),
                self.declaration_scope(local.declaration)?,
            )?
        {
            return self.issue_node(
                SemanticRule::Own10,
                node,
                SemanticIssueKind::InvalidBorrowLifetime,
            );
        }
        let (fields, ty) = self.resolve_struct_path(place_node, local.ty)?;
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
                    binding: local.binding,
                    nominal,
                }
            }
            CheckedType::Nominal(nominal)
                if fields.is_empty()
                    && matches!(
                        self.nominal(nominal)?.kind,
                        CheckedNominalKind::SystemResource { .. }
                    ) =>
            {
                CheckedExpression::BorrowSystemResource {
                    binding: local.binding,
                    nominal,
                }
            }
            CheckedType::Slice { .. } if fields.is_empty() => CheckedExpression::Binding {
                binding: local.binding,
                ty,
                slice_origins: slice
                    .as_ref()
                    .map(|slice| slice.origins.clone())
                    .unwrap_or_default(),
            },
            _ if fields.is_empty() && self.borrow_addresses_storage(ty)? => {
                CheckedExpression::BorrowAddressed {
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
                SemanticIssueKind::InvalidBorrowLifetime,
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
            ReborrowPosition::CallArgument { own_result } if !own_result => {
                return self.issue_node(
                    SemanticRule::Own6,
                    node,
                    SemanticIssueKind::InvalidChildReborrow,
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
        if matches!(position, ReborrowPosition::CallArgument { .. }) {
            let region_declaration = self.region_declaration(region)?;
            if region_declaration.role() != DeclarationRole::LocalRegion
                || !self.child_region_is_statement_scoped(region_declaration, node)?
            {
                return self.issue_node(
                    SemanticRule::Own6,
                    node,
                    SemanticIssueKind::InvalidChildReborrow,
                );
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
                        SemanticIssueKind::InvalidChildReborrow,
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
                        SemanticIssueKind::InvalidBorrowLifetime,
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
        let (fields, ty) = self.resolve_struct_path(place_node, local.ty)?;
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
                if fields.is_empty()
                    && matches!(
                        self.nominal(nominal)?.kind,
                        CheckedNominalKind::SystemResource { .. }
                    ) =>
            {
                CheckedExpression::BorrowSystemResource {
                    binding: local.binding,
                    nominal,
                }
            }
            _ if fields.is_empty() && self.borrow_addresses_storage(ty)? => {
                CheckedExpression::ReborrowAddressed {
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
        Ok(TypedExpression {
            expression,
            mode: borrow.mode(),
            borrow: Some(borrow),
            slice: None,
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
        if self.tree.production(region_node)? != Production::RegionStmt {
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

    fn borrow_region_is_inside_current_loops(
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
        Ok(self.enclosing_loops(region_node)? == borrow_loops)
    }

    fn enclosing_loops(&self, node: NodeId) -> Result<Vec<NodeId>, CheckStop> {
        let mut loops = Vec::new();
        let mut cursor = self.tree.parent(node)?;
        while let Some(node) = cursor {
            if self.tree.production(node)? == Production::LoopStmt {
                loops.push(node);
            }
            cursor = self.tree.parent(node)?;
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
        let CheckedNominalKind::Box { referent } = self.nominal(nominal)?.kind else {
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
            RequiredReferent::Enum => match ty {
                CheckedType::Bool => true,
                CheckedType::Nominal(nominal) => {
                    matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Enum { .. })
                }
                _ => false,
            },
            RequiredReferent::IndexableStorage => matches!(
                ty,
                CheckedType::Array { .. } | CheckedType::Buffer { .. } | CheckedType::Slice { .. }
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
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let destination_region = match (destination, borrow.kind) {
            (CheckedMode::Shared(region), BorrowKind::Shared)
            | (CheckedMode::Unique(region), BorrowKind::Unique) => region,
            _ => {
                return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
            }
        };
        if !self.region_outlives(borrow.region, destination_region)? {
            return self.issue_node(
                SemanticRule::Own4,
                node,
                SemanticIssueKind::InvalidBorrowLifetime,
            );
        }
        borrow.region = destination_region;
        Ok(Some(borrow))
    }

    pub(super) fn borrow_holder_scope_supported(
        &self,
        holder: DeclarationId,
        mode: CheckedMode,
    ) -> Result<bool, CheckStop> {
        let region = match mode {
            CheckedMode::Own => return Ok(true),
            CheckedMode::Shared(region) | CheckedMode::Unique(region) => region,
        };
        let holder_scope = self.declaration_scope(holder)?;
        let holder_scope = self
            .resolved
            .scopes()
            .get(holder_scope.index())
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(holder_scope.parent() == Some(self.region_declaration(region)?.scope()))
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
                    && matches!(
                        access,
                        AccessKind::Write | AccessKind::Move | AccessKind::UniqueBorrow
                    )
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
