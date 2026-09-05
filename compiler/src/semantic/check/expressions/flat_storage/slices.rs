use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, FixedTerminal, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::super::model::{
    CheckedExpression, CheckedMode, CheckedSliceOrigin, CheckedSliceSource, CheckedType,
    LoanStrength,
};
use super::super::super::borrows::{AccessKind, ResolvedPlace, SliceInfo, SliceLoan};
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::CheckedIndexedPlace;

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [VIEW-2] one view formation, at the strength the written row names.
    ///
    /// The two rows are one judgment: `slice_of` takes a shared borrow of the
    /// viewed storage and hands back a shared loan, `mut_slice_of` takes a
    /// `&uniq` borrow and hands back an exclusive one, and every other
    /// sentence — the region the borrow takes [OP-2], the origin the view
    /// carries [PROV-3], the element the viewed place fixes — is written
    /// once for both.
    pub(in crate::semantic::check) fn check_slice_of(
        &self,
        node: NodeId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if self
            .tree
            .first_child_with(node, Production::FieldinitList)?
            .is_some()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                SemanticIssueKind::InvalidNamedArguments {
                    callee: strength.former().to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        // [TYPE-5] `slice_of` is outside the retained-argument class, so it
        // carries no written argument: the region comes from the operand's own
        // borrow and the element from the place it views. A written argument
        // here is the rejection, not the supply.
        if self
            .tree
            .first_child_with(node, Production::Targs)?
            .is_some()
        {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        }
        let atoms = self.operation_atoms(node, 1)?;
        let borrow = self
            .tree
            .first_child_with(atoms[0], Production::BorrowExpr)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::type_mismatch(
                        "a written shared borrow of the viewed storage, `&'r place`",
                        "an atom that is not a borrow expression",
                    ),
                )
            })?;
        let written = if self.has_fixed(borrow, FixedTerminal::Uniq)? {
            LoanStrength::Exclusive
        } else {
            LoanStrength::Shared
        };
        if written != strength {
            let (expected, found) = match strength {
                LoanStrength::Shared => (
                    "a written shared borrow of the viewed storage, `&'r place`",
                    "a `&uniq` borrow, which slice_of does not take",
                ),
                LoanStrength::Exclusive => (
                    "a written unique borrow of the viewed storage, `&uniq 'r place`",
                    "a shared borrow, which mut_slice_of does not take",
                ),
            };
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(expected, found),
            );
        }
        // [OP-2] the result region is the one the operand's borrow takes,
        // written or elided [FORM-8].
        let Some(region) = self.borrow_expr_region(borrow)? else {
            return self.issue_node(
                crate::SemanticRule::Form8,
                borrow,
                crate::SemanticIssueKind::RegionSpelling {
                    mechanical_fix: "write the region this borrow takes, or place the borrow \
inside the `region` block whose region it takes",
                },
            );
        };
        let place_node = self
            .tree
            .first_child_with(borrow, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let pbase = self
            .tree
            .first_child_with(place_node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_arena_content_slice_of(
                node, borrow, place_node, pbase, region, strength, function, bindings, loop_depth,
            );
        }
        let root_use = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = root_use.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let owner = match class {
            DeclarationClass::Value => Some(declaration),
            // [CONST-2] a named const is permanently fixed storage, so it is
            // the `immutable-const` origin of a shared view and no origin an
            // exclusive one could write through.
            DeclarationClass::NamedConst if strength == LoanStrength::Shared => None,
            DeclarationClass::NamedConst => {
                return self.issue_node(
                    SemanticRule::Const2,
                    atoms[0],
                    SemanticIssueKind::ImmutableSetTarget,
                );
            }
            _ => {
                return self.issue_node(
                    SemanticRule::Type5,
                    atoms[0],
                    SemanticIssueKind::type_mismatch(
                        "a borrow of a runtime value binding or a named const",
                        "a borrow of a declaration that is neither",
                    ),
                );
            }
        };
        self.check_direct_slice_borrow_lifetime(function, region, owner, borrow, loop_depth)?;
        let suffixes = self.tree.children_with(place_node, Production::Psuffix)?;
        let indexed = self.check_indexed_place(
            place_node, bindings, &suffixes, place_node, function, loop_depth,
        )?;
        // [OP-2] the element is the viewed place's, and [STOR-4] still confines
        // a slice to flat elements — now judged on the derived one.
        //
        // DELIBERATELY UNTESTED, by the 2026-08-08 ruling. Once the element is
        // derived rather than written, no source appears to reach this arm:
        // `array<T, N>` and `buffer<T>` already require a flat T, so every
        // route tried — a non-copy struct element, a generic element, a nested
        // array element, an `array_new` of a struct — is rejected earlier by
        // TYPE-2 or by OP-1 on the array type itself, each confirmed with a
        // control that deletes the `slice_of` line and fails identically. That
        // is "not shown reachable", not "proven unreachable", so the rejection
        // stays. Widening what `array<T, N>` or `buffer<T>` admit re-opens the
        // question and owes this arm a test.
        let element_type = indexed.element_type();
        // An affine-element buffer is viewable in principle ([OP-1] states no
        // copy bound on the viewed T), but the in-place borrowed element read
        // a view would serve is not implemented, so the view stops as an
        // explicit unsupported capability rather than a source rejection.
        if let CheckedType::Nominal(id) = element_type
            && !self.nominal(id)?.is_copy()
        {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, atoms[0]);
        }
        let Some(element) = self.flat_element(element_type)? else {
            return self.issue_node(
                SemanticRule::Op1,
                atoms[0],
                SemanticIssueKind::InvalidOperation,
            );
        };
        let (source, resolved) = match indexed {
            // TEMPORARY capability stop, judged after every source rejection
            // above: an array is a value with no stable address in this
            // lowering, so the descriptor a view of one carries points at a
            // snapshot of it. A shared view is unaffected — a live shared
            // loan refuses every write to its origin, so the snapshot and the
            // array agree at every point the view is readable — while a write
            // through an exclusive view would reach the snapshot and not the
            // array. It stops here rather than lowering a write nobody can
            // observe.
            CheckedIndexedPlace::Array(_) if strength == LoanStrength::Exclusive => {
                return self
                    .unsupported(UnsupportedSemanticFeature::ExclusiveViewOverArray, atoms[0]);
            }
            CheckedIndexedPlace::Array(array) => {
                let resolved = array.resolved_place().unwrap_or(ResolvedPlace {
                    root: declaration,
                    fields: Vec::new(),
                });
                (
                    CheckedSliceSource::Array {
                        root: array.root,
                        length: array.length,
                    },
                    resolved,
                )
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                (CheckedSliceSource::Buffer(buffer.root), buffer.resolved)
            }
            // [OP-1] `slice_of` takes an array or a buffer; a view of a run
            // is [BLK-0]'s own `slice_of` row, which is DEFERRED with the
            // views batch.
            CheckedIndexedPlace::Slice(_) | CheckedIndexedPlace::Container(_) => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let origin = owner.map_or(CheckedSliceOrigin::ImmutableConst, |_| {
            CheckedSliceOrigin::SourcePlace {
                root: resolved.root,
                fields: resolved.fields.clone(),
                origin_region: None,
            }
        });
        let origins = vec![origin];
        // [PROV-3] use 1: the formation's own access to the origin is the
        // access the loan's strength names, so a second exclusive view of one
        // place meets the first loan here and is the ordinary [OWN-5]
        // conflict, while a second shared view does not.
        let taken = match strength {
            LoanStrength::Shared => AccessKind::SharedBorrow,
            LoanStrength::Exclusive => AccessKind::UniqueBorrow,
        };
        let accesses = if let Some(owner) = owner {
            self.check_loan_access(bindings, None, &resolved, taken, borrow)?;
            bindings
                .get_mut(&owner)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .push_slice_loan(SliceLoan {
                    region,
                    place: resolved.clone(),
                    strength,
                });
            vec![PlaceAccess {
                place: resolved,
                kind: taken,
            }]
        } else {
            Vec::new()
        };
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                source,
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(SliceInfo { region, origins }),
            holder: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses,
        })
    }

    /// [OWN-5] `slice_of` over a place reached in arena content: the operand
    /// `&'a deref(storage)` views the content array of an own `arena<'r, T>`
    /// binding. Its region obeys [OWN-10]'s arena case — the arena's `'r`
    /// must outlive-or-equals the borrow's region — and the created slice's
    /// origin retains the complete resolved place, so [FN-1]'s return-origin
    /// ceiling excludes it exactly as it excludes every other raw callee
    /// place: an `arena<'r, U>` parameter is not an input-slice supplier.
    #[allow(clippy::too_many_arguments)]
    fn check_arena_content_slice_of(
        &self,
        node: NodeId,
        borrow: NodeId,
        place_node: NodeId,
        pbase: NodeId,
        region: DeclarationId,
        strength: LoanStrength,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        if !self.borrow_region_is_inside_current_loops(region, borrow, loop_depth)? {
            return self.issue_node(
                SemanticRule::Own11,
                borrow,
                SemanticIssueKind::BorrowRegionOutsideLoop {
                    mechanical_fix: "introduce the borrow region inside the enclosing loop body",
                },
            );
        }
        let inner_place = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let inner_pbase = self
            .tree
            .first_child_with(inner_place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // The implemented fragment reaches content through one deref of a
        // directly named binding; deeper chains and projected content stay
        // explicit capability stops.
        if self.has_fixed(inner_pbase, FixedTerminal::Deref)?
            || !self.tree.children(inner_pbase)?.is_empty()
            || !self
                .tree
                .children_with(inner_place, Production::Psuffix)?
                .is_empty()
            || !self
                .tree
                .children_with(place_node, Production::Psuffix)?
                .is_empty()
        {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        let root_use = self.use_at(inner_pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = root_use.target()
        else {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        };
        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                place_node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        let Some((arena_region, content)) = self.arena_instance(local.ty)? else {
            // A deref over anything but an own arena binding stays a
            // capability stop rather than a fabricated source verdict.
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        };
        if local.mode != CheckedMode::Own {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, place_node);
        }
        // [OWN-10] for a place rooted in arena<'r, T> content: 'r must
        // outlive-or-equals the borrow's region.
        if !self.region_outlives(arena_region, region)? {
            return self.issue_node(
                SemanticRule::Own10,
                borrow,
                SemanticIssueKind::InvalidBorrowLifetime {
                    region: self.region_phrase(region)?,
                    binder: self.declaration_spelling(declaration)?,
                    // The arena's own region is what the view must name. A
                    // region [FORM-8] leaves unwritten has no name to give,
                    // so the repair is to relate the two positions first.
                    mechanical_fix: match self.written_region_name(arena_region)? {
                        Some(name) => format!(
                            "arena content outlives its arena's region {name}, not the arena \
binding; name {name} on this view, or a region {name} outlives"
                        ),
                        None => "arena content outlives its arena's own region, not the arena \
binding; that region is unwritten here, so write it on the arena and name it on this view, or \
take the view in a region it outlives"
                            .to_owned(),
                    },
                },
            );
        }
        // TEMPORARY capability stop, judged after the [OWN-1] and [OWN-10]
        // source rejections above: no lowering builds a slice over arena
        // content. A view over an arena *parameter* still checks on, because
        // the whole function then stops at the arena-parameter gate, and the
        // [FN-1] return-origin judgment must reach its verdict first. A view
        // over a *local* arena has no such later gate, so without this stop
        // it would publish a checked program the IR builder cannot lower.
        if !function
            .parameters
            .iter()
            .any(|parameter| parameter.declaration == declaration)
        {
            return self.unsupported(UnsupportedSemanticFeature::ArenaRuntime, place_node);
        }
        let CheckedType::Array { element, length } = content else {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place_node);
        };
        let resolved = ResolvedPlace {
            root: declaration,
            fields: Vec::new(),
        };
        let taken = match strength {
            LoanStrength::Shared => AccessKind::SharedBorrow,
            LoanStrength::Exclusive => AccessKind::UniqueBorrow,
        };
        self.check_loan_access(bindings, None, &resolved, taken, borrow)?;
        bindings
            .get_mut(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .push_slice_loan(SliceLoan {
                region,
                place: resolved.clone(),
                strength,
            });
        // The origin is the complete resolved place reached in arena content
        // [OWN-5]; reads through the formed view stay reads of storage this
        // function owns, so the formation carries no boundary effect.
        let origins = vec![CheckedSliceOrigin::SourcePlace {
            root: declaration,
            fields: Vec::new(),
            origin_region: None,
        }];
        Ok(TypedExpression {
            expression: CheckedExpression::SliceOf {
                carrier: self.tree.path(node)?.clone(),
                source: CheckedSliceSource::ArenaContent {
                    binding: local.binding,
                    fields: Vec::new(),
                    length,
                },
                region,
                element,
                strength,
                origins: origins.clone(),
            },
            mode: CheckedMode::Own,
            borrow: None,
            slice: Some(SliceInfo { region, origins }),
            holder: None,
            reference_value: false,
            effects: EffectSet::NONE,
            accesses: vec![PlaceAccess {
                place: resolved,
                kind: taken,
            }],
        })
    }
}
