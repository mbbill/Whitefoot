use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::super::goal::{
    EvaluatedValueOccurrence, GoalDatum, GoalExpression, GoalProjection,
};
use super::super::super::super::model::{
    CheckedCallSeparation, CheckedEffectStep, CheckedExpression, CheckedMode, CheckedNominalKind,
    CheckedStatePath, CheckedType,
};
use super::super::super::super::places::{
    CapturedValue, PlaceRoot, PlaceStep, ResolvedPlace, UnprovedSeparations, places_overlap,
};
use super::super::super::references::InvalidationEvent;
use super::super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

/// One entry of a call's substituted effect row [EFF-5].
///
/// Each declared `effect_path` rooted at reference parameter i takes actual
/// argument i's path, and each IDENT index or range endpoint takes the value
/// its own argument supplies. A by-value argument contributes a consumption
/// (`move`) or a read (copy) of its own place to the same comparison, which
/// is why an entry carries a resolved place and a category and nothing about
/// how it was spelled.
struct SubstitutedEntry {
    place: ResolvedPlace,
    /// Whether this entry writes, replaces, moves out of, or frees the
    /// storage at its path and everything below it [EFF-1].
    write: bool,
    /// The argument ordinal this entry came from, so the pairwise comparison
    /// can skip a pair of entries belonging to one argument.
    argument: usize,
    /// The rendered path the [EFF-5] diagnostic carries.
    spelling: String,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_user_call(
        &self,
        node: NodeId,
        declaration: DeclarationId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let target = self.concrete_function_for_call(node, declaration, &function.substitution)?;
        let signature = self
            .signatures
            .get(target.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.check_selected_user_call(node, signature, None, function, bindings, loop_depth)
    }

    pub(in crate::semantic::check) fn check_behavior_call(
        &self,
        node: NodeId,
        key: crate::semantic::check::generics::GenericParameterKey,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let argument = function
            .substitution
            .function_argument(key)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let target = self.function_argument_instance(argument)?;
        let actual = self
            .signatures
            .get(target.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let formal = self.formal_signature(key, &function.substitution, target)?;
        let binding_site = self.behavior_binding_site(node, key, &function.substitution)?;
        let effective = self.behavior_call_signature(binding_site, &formal, actual)?;
        let effects = Some(super::super::super::super::model::CheckedEffects {
            reads: effective.declared_effects.reads.clone(),
            writes: effective.declared_effects.writes.clone(),
            allocates: effective.declared_effects.allocates,
        });
        self.check_selected_user_call(node, &effective, effects, function, bindings, loop_depth)
    }

    fn check_selected_user_call(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
        formal_effects: Option<super::super::super::super::model::CheckedEffects>,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let target = signature.id;
        let fields = if let Some(list) = self
            .tree
            .first_child_with(node, Production::FieldinitList)?
        {
            self.tree.children_with(list, Production::Fieldinit)?
        } else {
            Vec::new()
        };
        if self
            .tree
            .first_child_with(node, Production::AtomList)?
            .is_some()
            || fields.len() != signature.parameters.len()
        {
            return self.issue_node(
                SemanticRule::Gram11,
                node,
                Self::invalid_named_arguments(signature),
            );
        }
        let mut arguments = Vec::with_capacity(fields.len());
        let mut argument_nodes = Vec::with_capacity(fields.len());
        let mut goal_arguments = Vec::with_capacity(fields.len());
        // [EFF-5] each actual's resolved path set, in parameter order. The set
        // has more than one member only where the actual is a reference a
        // control-flow join gave more than one path [REF-1]; every check must
        // then hold for every member.
        let mut actual_paths: Vec<Vec<ResolvedPlace>> = Vec::with_capacity(fields.len());
        let mut actual_modes = Vec::with_capacity(fields.len());
        let call = self.tree.path(node)?.clone();
        let mut effects = EffectSet::NONE;
        for (ordinal, (field, parameter)) in
            fields.into_iter().zip(&signature.parameters).enumerate()
        {
            if self.identifier(field)? != parameter.name {
                return self.issue_node(
                    SemanticRule::Gram11,
                    field,
                    Self::invalid_named_arguments(signature),
                );
            }
            let atom = self
                .tree
                .first_child_with(field, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let argument =
                self.check_call_argument_atom(function, atom, bindings, loop_depth)?;
            if argument.expression.ty() != parameter.ty {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(parameter.ty)?,
                        self.checked_type_name(argument.expression.ty())?,
                    ),
                );
            }
            if argument.mode != parameter.mode {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_value_name(parameter.mode, parameter.ty)?,
                        self.checked_value_name(argument.mode, argument.expression.ty())?,
                    ),
                );
            }
            // [REF-2] a reference handed to a call must be valid at the call.
            if let Some(reference) = &argument.reference
                && !reference.is_valid()
            {
                return self.issue_node(
                    SemanticRule::Ref2,
                    atom,
                    SemanticIssueKind::InvalidReferenceUse {
                        binder: parameter.name.clone(),
                        event: match &reference.validity {
                            super::super::super::references::ReferenceValidity::Invalid(event) => {
                                event.phrase()
                            }
                            super::super::super::references::ReferenceValidity::Valid => "",
                        },
                        mechanical_fix: super::super::super::references::REF2_FORM_AGAIN,
                    },
                );
            }
            let paths = argument
                .reference
                .as_ref()
                .map(|reference| reference.paths.clone())
                .unwrap_or_else(|| {
                    argument
                        .accesses
                        .iter()
                        .map(|access| access.place.clone())
                        .collect()
                });
            let ordinal_index =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            goal_arguments.push(self.call_goal_argument(
                function.id,
                &call,
                ordinal_index,
                atom,
                parameter.mode,
                parameter.ty,
                &argument,
                paths.first(),
                bindings,
            )?);
            argument_nodes.push(self.tree.path(atom)?.clone());
            actual_paths.push(paths);
            actual_modes.push(parameter.mode);
            effects = effects.union(argument.effects);
            arguments.push(argument.expression);
        }
        // [STOR-8] a unit carrying the no-heap declaration cannot call an
        // allocating prelude row; [OP-11] refuses a `swap` over a copy place.
        self.reject_allocating_call_under_no_heap(node, signature)?;
        self.reject_swap_over_copy(node, signature)?;
        // [EFF-5] substitute, compare pairwise, then project the surviving
        // footprint onto the caller's own row [EFF-2].
        // [EFF-3] a call inherits its callee's allocation fact.
        if signature.declared_effects.allocates {
            effects.add_allocation();
        }
        let substituted = self.substitute_call_row(node, signature, &actual_paths, &actual_modes)?;
        self.check_call_pairwise_disjointness(node, signature, &substituted)?;
        self.invalidate_call_bystanders(&substituted, &call, bindings);
        Self::invalidate_window_operation_references(signature, &substituted, bindings);
        self.project_call_effects(node, function, &substituted, bindings, &mut effects)?;
        let result = signature.result;
        let result_mode = signature.result_mode;
        Ok(TypedExpression {
            expression: CheckedExpression::UserCall {
                function: target,
                formal_effects: formal_effects.map(Box::new),
                call,
                argument_nodes,
                arguments,
                goal_arguments,
                goal_regions: Vec::new(),
                requirements: Vec::new(),
                result,
                slice_origins: Vec::new(),
                result_borrow: None,
            },
            mode: result_mode,
            // [REF-3] no call delivers a reference: FN-1 returns owned values
            // and the declaration boundary already refused any other result
            // mode, so the result names no path.
            reference: None,
            reference_value: false,
            effects,
            accesses: Vec::new(),
        })
    }

    /// One resolved place in the spelling an [EFF-5] diagnostic renders.
    fn render_resolved_place(&self, place: &ResolvedPlace) -> Result<String, CheckStop> {
        let mut rendered = match place.root {
            PlaceRoot::Binding(binding) => format!("<binding:{}>", binding.0),
            PlaceRoot::Constant(constant) => self.constant(constant)?.name.clone(),
        };
        for step in &place.path {
            match step {
                PlaceStep::Field(field) => rendered.push_str(&format!(".{field}")),
                PlaceStep::Deref => rendered = format!("deref({rendered})"),
                PlaceStep::Payload { variant, field } => {
                    rendered.push_str(&format!(".{variant}.{field}"));
                }
                PlaceStep::Index(_) => rendered.push_str("[.]"),
                PlaceStep::Range(_) => rendered.push_str("[...]"),
                PlaceStep::Part(part) => rendered.push_str(&format!(".{}", part.spelling())),
                PlaceStep::Measure(measure) => {
                    rendered.push_str(&format!(".{}", measure.spelling()));
                }
            }
        }
        Ok(rendered)
    }

    /// [EFF-5] the substituted row of one call.
    ///
    /// Each `effect_path` rooted at reference parameter i takes actual
    /// argument i's path and appends its own `epsuffix*`; a by-value argument
    /// contributes a consumption or a read of its place to the same list. An
    /// index or range position names a value parameter of the same callable,
    /// and the value that parameter's argument supplies is what replaces it.
    fn substitute_call_row(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
        actual_paths: &[Vec<ResolvedPlace>],
        actual_modes: &[CheckedMode],
    ) -> Result<Vec<SubstitutedEntry>, CheckStop> {
        let mut entries = Vec::new();
        for (write, declared) in [
            (false, &signature.declared_effects.reads),
            (true, &signature.declared_effects.writes),
        ] {
            for formal in declared {
                let Some(index) = signature
                    .parameters
                    .iter()
                    .position(|parameter| parameter.declaration == formal.root)
                else {
                    // [EFF-1] a row is rooted at a parameter of the same
                    // callable; the declaration boundary already refused
                    // anything else.
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let steps = self.substitute_effect_steps(signature, formal, actual_paths)?;
                for base in actual_paths.get(index).into_iter().flatten() {
                    let mut place = base.clone();
                    place.path.extend_from_slice(&steps);
                    entries.push(SubstitutedEntry {
                        spelling: self.render_resolved_place(&place)?,
                        place,
                        write,
                        argument: index,
                    });
                }
            }
        }
        // [EFF-5] clause 2: a by-value argument contributes a consumption
        // (`move`) or a read (copy) of its place to this same comparison.
        for (index, mode) in actual_modes.iter().enumerate() {
            if *mode != CheckedMode::Own {
                continue;
            }
            for place in actual_paths.get(index).into_iter().flatten() {
                entries.push(SubstitutedEntry {
                    spelling: self.render_resolved_place(place)?,
                    place: place.clone(),
                    // A `move` empties the place, which [EFF-1] classes with
                    // the writes; a copy argument observes it.
                    write: self.is_copy_place_type(signature, index).map_or(true, |copy| !copy),
                    argument: index,
                });
            }
        }
        let _ = node;
        Ok(entries)
    }

    /// One declared `epsuffix*`, with its index and range positions replaced
    /// by the values their own arguments supply [EFF-5].
    fn substitute_effect_steps(
        &self,
        signature: &FunctionSignature,
        formal: &CheckedStatePath,
        actual_paths: &[Vec<ResolvedPlace>],
    ) -> Result<Vec<PlaceStep>, CheckStop> {
        let captured = |parameter: DeclarationId| -> CapturedValue {
            // The index position names a value parameter; the value its
            // argument supplied is the immutable captured value of that
            // argument's own place, where the argument is a place, and the
            // unknown offset otherwise, which no family separates.
            signature
                .parameters
                .iter()
                .position(|candidate| candidate.declaration == parameter)
                .and_then(|index| actual_paths.get(index))
                .and_then(|paths| paths.first())
                .and_then(|place| {
                    place.path.iter().rev().find_map(|step| match step {
                        PlaceStep::Index(value) => Some(*value),
                        _ => None,
                    })
                })
                .unwrap_or_else(CapturedValue::unknown)
        };
        Ok(formal
            .steps
            .iter()
            .map(|step| match step {
                CheckedEffectStep::Field(field) => PlaceStep::Field(*field),
                CheckedEffectStep::Deref => PlaceStep::Deref,
                CheckedEffectStep::Payload { variant, field } => PlaceStep::Payload {
                    variant: *variant,
                    field: *field,
                },
                CheckedEffectStep::Index(parameter) => PlaceStep::Index(captured(*parameter)),
                CheckedEffectStep::Range { start, end } => {
                    PlaceStep::Range(super::super::super::super::places::CapturedRange {
                        start: captured(*start),
                        end: captured(*end),
                    })
                }
                CheckedEffectStep::Part(part) => PlaceStep::Part(*part),
                CheckedEffectStep::Measure(measure) => PlaceStep::Measure(*measure),
            })
            .collect())
    }

    /// Whether the by-value parameter at `index` has a copy type, so its
    /// argument contributes a read rather than a consumption [EFF-5].
    fn is_copy_place_type(&self, signature: &FunctionSignature, index: usize) -> Option<bool> {
        let parameter = signature.parameters.get(index)?;
        self.is_copy_type(parameter.ty).ok()
    }

    /// [EFF-5] clause 1: two effects on overlapping paths where at least one
    /// is a write must be proved disjoint.
    ///
    /// The checker holds the actual spellings and the live reference state,
    /// so it owns this comparison; what it cannot do is discharge the index
    /// or range goal the fixed [ENT-6] families own, so a pair the syntax
    /// leaves open is recorded for the entailment fragment and refused there
    /// if it stays undischarged. `swap` is the one operation whose two
    /// arguments may name the same place [OP-11].
    fn check_call_pairwise_disjointness(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
        entries: &[SubstitutedEntry],
    ) -> Result<(), CheckStop> {
        if self.is_swap_row(signature) {
            return Ok(());
        }
        let oracle = UnprovedSeparations;
        for (index, left) in entries.iter().enumerate() {
            for right in entries.iter().skip(index + 1) {
                if left.argument == right.argument || !(left.write || right.write) {
                    continue;
                }
                if !places_overlap(&oracle, &left.place, &right.place) {
                    continue;
                }
                // A pair whose only unseparated steps are index or range
                // positions is the fixed families' question; every other
                // overlap is refused here and now.
                if Self::separable_by_position(&left.place, &right.place) {
                    self.call_separations
                        .borrow_mut()
                        .push(CheckedCallSeparation {
                            site: self.tree.path(node)?.clone(),
                            left: left.place.clone(),
                            right: right.place.clone(),
                            left_spelling: left.spelling.clone(),
                            right_spelling: right.spelling.clone(),
                        });
                    continue;
                }
                return self.issue_node(
                    SemanticRule::Eff5,
                    node,
                    SemanticIssueKind::OverlappingCallEffects {
                        first: left.spelling.clone(),
                        second: right.spelling.clone(),
                        mechanical_fix: "prove the two positions distinct, or pass one of them",
                    },
                );
            }
        }
        Ok(())
    }

    /// Whether the first step at which the two paths disagree is an index or
    /// a range position, which is the only disagreement an admitted [OWN-7]
    /// family can still separate.
    fn separable_by_position(left: &ResolvedPlace, right: &ResolvedPlace) -> bool {
        left.path
            .iter()
            .zip(&right.path)
            .find(|(left, right)| left != right)
            .is_some_and(|(left, right)| {
                matches!(
                    (left, right),
                    (PlaceStep::Index(_), PlaceStep::Index(_))
                        | (PlaceStep::Range(_), PlaceStep::Range(_))
                )
            })
    }

    /// [OP-10] the window operations that end the bound a reference into the
    /// window was formed under.
    ///
    /// `place_back`'s `ensures` carries `i < r.len` across the call and
    /// `insert_at` only changes a slot's occupant, so neither appears here.
    /// The rest move a boundary down, move a run between two windows, shift
    /// every logical index, or remake the block whole, and every reference
    /// into the operand dies [REF-2, REF-4].
    fn invalidate_window_operation_references(
        signature: &FunctionSignature,
        entries: &[SubstitutedEntry],
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) {
        const BOUND_ENDING_ROWS: [&str; 7] = [
            "take_back",
            "remove_at",
            "append",
            "split_off",
            "place_front",
            "take_front",
            "grow",
        ];
        if !BOUND_ENDING_ROWS.contains(&signature.name.as_str()) {
            return;
        }
        for entry in entries {
            Self::invalidate_window_references(bindings, &entry.place);
        }
    }

    /// [STOR-8] a compilation unit carrying the no-heap declaration cannot
    /// call an allocating prelude row.
    ///
    /// The five rows the rule names are exactly the allocating [OP-13] and
    /// [OP-10] records that take a cell from the heap; `slots_new`,
    /// `ring_new` and `array_filled` build frame-resident shapes and are not
    /// among them.
    fn reject_allocating_call_under_no_heap(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
    ) -> Result<(), CheckStop> {
        const HEAP_ROWS: [&str; 5] = [
            "box_new",
            "box_array_filled",
            "box_slots_new",
            "box_ring_new",
            "grow",
        ];
        if !self.no_heap || !HEAP_ROWS.contains(&signature.name.as_str()) {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Stor8,
            node,
            SemanticIssueKind::HeapTypeUnderNoHeap {
                spelling: signature.name.clone(),
                mechanical_fix: super::super::super::types::STOR8_NO_HEAP,
            },
        )
    }

    /// [OP-11] a `swap` over a copy place is a hard error at the first
    /// `borrow_expr`, with the restructuring `read the two values and assign
    /// them back`.
    ///
    /// The rule judges the refusal once at the written bound, exactly as
    /// [OWN-1]'s spelling judgment is judged, and does not re-make it at a
    /// concrete instance [FN-2]; `judges_class_spelling` is that same gate.
    /// The rule locates it at the first `borrow_expr`; this locates it at the
    /// complete `call`, whose written arguments are exactly those two
    /// `borrow_expr` nodes, because the refusal is about the operation and
    /// not about one of its two symmetric operands.
    fn reject_swap_over_copy(
        &self,
        node: NodeId,
        signature: &FunctionSignature,
    ) -> Result<(), CheckStop> {
        if !self.is_swap_row(signature) || !self.judges_class_spelling() {
            return Ok(());
        }
        let Some(first) = signature.parameters.first() else {
            return Ok(());
        };
        if !self.is_copy_type(first.ty)? {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Op11,
            node,
            SemanticIssueKind::SwapOverCopyPlace {
                place_type: self.checked_type_name(first.ty)?,
                mechanical_fix: "read the two values and assign them back",
            },
        )
    }

    /// [OP-11] the one operation whose two reference arguments may name the
    /// same place.
    fn is_swap_row(&self, signature: &FunctionSignature) -> bool {
        signature.name == "swap"
    }

    /// [EFF-5] clause 3: a live reference outside the call whose path has a
    /// proper prefix among the call's substituted write paths becomes invalid
    /// after the call [REF-2].
    ///
    /// A reference that is itself an argument is the thing being accessed,
    /// not a bystander, and does not invalidate itself; the reference whose
    /// own path the write names is exactly the one whose path the write is a
    /// prefix of, which [`Checker::invalidate_references`] already decides.
    fn invalidate_call_bystanders(
        &self,
        entries: &[SubstitutedEntry],
        call: &crate::NodePath,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) {
        let _ = call;
        for entry in entries.iter().filter(|entry| entry.write) {
            Self::invalidate_references(
                bindings,
                &entry.place,
                &InvalidationEvent::CallWrite,
            );
        }
    }

    /// [EFF-2] the caller's own row: each projected entry rooted in a current
    /// formal contributes that formal's corresponding path, and an entry
    /// rooted only in local storage contributes none.
    fn project_call_effects(
        &self,
        node: NodeId,
        caller: &FunctionSignature,
        entries: &[SubstitutedEntry],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        effects: &mut EffectSet,
    ) -> Result<(), CheckStop> {
        for entry in entries {
            for path in self.effect_paths_for_place(node, &entry.place, bindings)? {
                if !caller
                    .parameters
                    .iter()
                    .any(|parameter| parameter.declaration == path.path.root)
                {
                    continue;
                }
                if entry.write {
                    effects.add_write(path);
                } else {
                    effects.add_read(path);
                }
            }
        }
        Ok(())
    }

    /// Captures one already-checked actual's pre-transfer goal image.
    ///
    /// This runs after the actual expression has acquired all of its checked
    /// obligations and after borrow feasibility succeeds. It never rechecks or
    /// reevaluates the source expression. A borrow destination is represented
    /// by the resolved ultimate referent captured in `passed_borrow` before
    /// that transient checker metadata disappears.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::semantic::check) fn call_goal_argument(
        &self,
        caller: super::super::super::super::model::FunctionId,
        call: &crate::NodePath,
        ordinal: u32,
        atom: NodeId,
        expected_mode: CheckedMode,
        expected_type: CheckedType,
        argument: &super::super::super::TypedExpression,
        passed_place: Option<&ResolvedPlace>,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<GoalExpression, CheckStop> {
        if expected_mode != CheckedMode::Own {
            if let Some(place) = passed_place {
                return self.goal_referent_image(place, expected_type, bindings);
            }
            // FN-1's candidate protects every mutable origin a returned
            // borrow may reach, including when the delivered value is a
            // different immutable constant. ENT-2's value identity is the
            // actual holder, never that conservative loan ceiling.
            let place_parent = self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .unwrap_or(atom);
            let place = self
                .tree
                .first_child_with(place_parent, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            if self.call_goal_place_contains_subscript(place)? {
                return Ok(GoalExpression::Datum(GoalDatum::EvaluatedValue {
                    function: caller,
                    occurrence: EvaluatedValueOccurrence::CallArgument {
                        call: call.clone(),
                        argument: ordinal,
                    },
                    captured_type: expected_type,
                    projections: Vec::new(),
                    ty: expected_type,
                }));
            }
            let (image, _) = self.call_goal_place_inner(place, bindings)?;
            if image.ty() != expected_type {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            return Ok(image);
        }

        if self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
            .is_some()
        {
            let CheckedExpression::Constant(value) = &argument.expression else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            return Ok(GoalExpression::Datum(GoalDatum::Literal(value.clone())));
        }

        let place = self
            .tree
            .first_child_with(atom, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.call_goal_place_contains_subscript(place)? {
            return Ok(GoalExpression::Datum(GoalDatum::EvaluatedValue {
                function: caller,
                occurrence: EvaluatedValueOccurrence::CallArgument {
                    call: call.clone(),
                    argument: ordinal,
                },
                captured_type: expected_type,
                projections: Vec::new(),
                ty: expected_type,
            }));
        }
        let (image, holder_pending) = self.call_goal_place_inner(place, bindings)?;
        if holder_pending || image.ty() != expected_type {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(image)
    }

    /// A place may nest another place under a `deref` pbase. Search the whole
    /// source place, not only its outer suffix list, so a future admitted
    /// `deref(boxes[i])` actual receives the same ephemeral treatment and is
    /// never misidentified as a rereadable place.
    fn call_goal_place_contains_subscript(&self, place: NodeId) -> Result<bool, CheckStop> {
        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        if self.last_subscript(&suffixes)?.is_some() {
            return Ok(true);
        }
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let Some(nested) = self.tree.first_child_with(pbase, Production::Place)? else {
            return Ok(false);
        };
        self.call_goal_place_contains_subscript(nested)
    }

    /// Forms a caller-visible referent datum. A root that is itself one of the
    /// caller's borrow parameters remains opaque and therefore retains one
    /// `Deref`; a local borrow/reborrow has already resolved through its holder
    /// to an own root and adds no such projection. Owning indirection remains
    /// in the typed storage path: overlap may identify a Box with its content,
    /// but a measure must still select the content's value. This applies to
    /// direct borrows and forwarded holders alike.
    fn goal_referent_image(
        &self,
        place: &ResolvedPlace,
        ty: CheckedType,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<GoalExpression, CheckStop> {
        let mut projections = Vec::new();
        for step in &place.path {
            match step {
                PlaceStep::Field(field) => projections.push(GoalProjection::Field(*field)),
                PlaceStep::Deref => projections.push(GoalProjection::Deref),
                PlaceStep::Index(index) => projections.push(GoalProjection::Subscript(*index)),
                // [ENT-2] a goal datum's place carries field selections,
                // `deref` wrappings and subscripts; a payload, range, part or
                // measure step is no datum spelling, so the image stops here.
                PlaceStep::Payload { .. }
                | PlaceStep::Range(_)
                | PlaceStep::Part(_)
                | PlaceStep::Measure(_) => break,
            }
        }
        let _ = bindings;
        let datum = match place.root {
            PlaceRoot::Constant(constant) => GoalDatum::NamedConst {
                declaration: self
                    .constants
                    .iter()
                    .find_map(|(declaration, id)| (*id == constant).then_some(*declaration))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                projections,
                ty,
            },
            PlaceRoot::Binding(binding) => GoalDatum::Place {
                root: binding,
                projections,
                ty,
            },
        };
        Ok(GoalExpression::Datum(datum))
    }

    /// Resolves one non-indexed own actual to its concrete caller datum while
    /// preserving own-box dereference and field order. Dereferencing a borrow
    /// holder consumes the holder boundary exactly once and leaves the
    /// ultimate referent image produced above.
    fn call_goal_place_inner(
        &self,
        place: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(GoalExpression, bool), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (mut expression, holder_pending) = if self
            .has_fixed(pbase, crate::FixedTerminal::Deref)?
        {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let (nested, nested_holder_pending) = self.call_goal_place_inner(nested, bindings)?;
            if nested_holder_pending {
                (nested, false)
            } else {
                let CheckedType::Nominal(nominal) = nested.ty() else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                (
                    nested
                        .with_projection(GoalProjection::Deref, referent)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                    false,
                )
            }
        } else {
            let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
            let ResolvedTarget::Source { declaration, class } = usage.target() else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            match class {
                DeclarationClass::Value => {
                    let local = bindings
                        .get(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if let Some(reference) = &local.reference {
                        (
                            match reference.paths.first() {
                                Some(path) => {
                                    self.goal_referent_image(path, local.ty, bindings)?
                                }
                                None => GoalExpression::Datum(GoalDatum::Place {
                                    root: local.binding,
                                    projections: vec![GoalProjection::Deref],
                                    ty: local.ty,
                                }),
                            },
                            true,
                        )
                    } else {
                        (
                            GoalExpression::Datum(GoalDatum::Place {
                                root: local.binding,
                                projections: Vec::new(),
                                ty: local.ty,
                            }),
                            false,
                        )
                    }
                }
                DeclarationClass::NamedConst => {
                    let constant = self
                        .constants
                        .get(&declaration)
                        .copied()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    (
                        GoalExpression::Datum(GoalDatum::NamedConst {
                            declaration,
                            projections: Vec::new(),
                            ty: self.constant(constant)?.ty,
                        }),
                        false,
                    )
                }
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            }
        };

        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        if holder_pending && !suffixes.is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if !suffixes.is_empty() {
            let (fields, final_ty) = self.resolve_struct_path(&suffixes, expression.ty())?;
            for field in fields {
                expression = expression
                    .with_projection(GoalProjection::Field(field), final_ty)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            }
        }
        Ok((expression, holder_pending))
    }

}
