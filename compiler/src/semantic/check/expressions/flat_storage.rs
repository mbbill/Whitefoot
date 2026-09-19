use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedArrayRoot, CheckedArraySetTarget, CheckedBufferRoot, CheckedBufferSetTarget,
    CheckedConst, CheckedContainerRoot, CheckedExpression, CheckedLayoutCeiling,
    CheckedLayoutMagnitude, CheckedMeasure, CheckedMode, CheckedNominalKind, CheckedPlaceStep,
    CheckedPlaceSubscript, CheckedSetTarget, CheckedTargetDomainObligation, CheckedType,
    IntegerType, MeasureCell, NominalId,
};
use super::super::super::places::{
    CaptureId, CapturedTerm, CapturedValue, PlaceRoot, PlaceStep, ResolvedPlace,
};
use super::super::references::{AccessKind, RequiredReferent};
use super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::{MutationTarget, PlaceUseOptions};

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedArrayPlace {
    pub(super) root: CheckedArrayRoot,
    declaration: Option<DeclarationId>,
    array_type: CheckedType,
    element_type: CheckedType,
    length: CheckedConst,
}

impl CheckedArrayPlace {
    fn resolved_place(&self) -> Option<ResolvedPlace> {
        let CheckedArrayRoot::Binding { binding, fields } = &self.root else {
            return None;
        };
        Some(ResolvedPlace::fields(*binding, fields.clone()))
    }
}

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedBufferPlace {
    root: CheckedBufferRoot,
    declaration: DeclarationId,
    element_type: CheckedType,
    resolved: ResolvedPlace,
}

#[derive(Clone)]
pub(in crate::semantic::check) enum CheckedIndexedPlace {
    Array(CheckedArrayPlace),
    Buffer(CheckedBufferPlace),
    /// One run or bump extent [BLK-1, PROV-1]: the two runs are indexable
    /// bases [OP-4] and all three have a measure-table row [MSR-1].
    Container(CheckedContainerPlace),
}

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedContainerPlace {
    root: CheckedContainerRoot,
    resolved: ResolvedPlace,
    /// The source declaration this place is rooted in, where it has one; a
    /// place rooted in a named const [CONST-2] has none.
    declaration: Option<DeclarationId>,
    /// The effects and accesses of every offset occurring inside the place
    /// [EFF-2]: an offset that reads a binding is a read of that binding,
    /// wherever in the place it occurs.
    offsets: CarriedOperands,
}

/// The effects and accesses one place's own offset operands exhibit.
#[derive(Clone, Default)]
pub(in crate::semantic::check) struct CarriedOperands {
    pub(in crate::semantic::check) effects: EffectSet,
    pub(in crate::semantic::check) accesses: Vec<PlaceAccess>,
}

fn add_layout_magnitude(
    left: CheckedLayoutMagnitude,
    right: CheckedLayoutMagnitude,
) -> CheckedLayoutMagnitude {
    match (left, right) {
        (CheckedLayoutMagnitude::Finite(left), CheckedLayoutMagnitude::Finite(right)) => {
            left.checked_add(right).map_or(
                CheckedLayoutMagnitude::AboveU64,
                CheckedLayoutMagnitude::Finite,
            )
        }
        _ => CheckedLayoutMagnitude::AboveU64,
    }
}

fn multiply_layout_magnitude(value: CheckedLayoutMagnitude, count: u64) -> CheckedLayoutMagnitude {
    if count == 0 {
        return CheckedLayoutMagnitude::Finite(0);
    }
    match value {
        CheckedLayoutMagnitude::Finite(value) => value.checked_mul(count).map_or(
            CheckedLayoutMagnitude::AboveU64,
            CheckedLayoutMagnitude::Finite,
        ),
        CheckedLayoutMagnitude::AboveU64 => CheckedLayoutMagnitude::AboveU64,
    }
}

fn round_up_layout_magnitude(value: CheckedLayoutMagnitude, align: u64) -> CheckedLayoutMagnitude {
    match value {
        CheckedLayoutMagnitude::Finite(value) => value
            .checked_add(align - 1)
            .map(|sum| sum / align * align)
            .map_or(
                CheckedLayoutMagnitude::AboveU64,
                CheckedLayoutMagnitude::Finite,
            ),
        CheckedLayoutMagnitude::AboveU64 => CheckedLayoutMagnitude::AboveU64,
    }
}

impl CheckedIndexedPlace {
    /// Mutable array owners use the same typed storage path as run slots.
    /// Keep constant arrays on their immutable-global read path.
    fn into_element_storage(self) -> Result<Self, CheckStop> {
        let Self::Array(array) = self else {
            return Ok(self);
        };
        let CheckedArrayRoot::Binding { binding, fields } = &array.root else {
            return Ok(Self::Array(array));
        };
        Ok(Self::Container(CheckedContainerPlace {
            root: CheckedContainerRoot {
                root: PlaceRoot::Binding(*binding),
                path: fields
                    .iter()
                    .copied()
                    .map(CheckedPlaceStep::Field)
                    .collect(),
                ty: array.array_type,
            },
            resolved: array
                .resolved_place()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            declaration: array.declaration,
            offsets: CarriedOperands::default(),
        }))
    }

    /// The declaration this place is rooted in, where it has one. A place
    /// rooted in a named const has none, and no proof-point admission
    /// restricts a const.
    pub(in crate::semantic::check) const fn root_declaration(&self) -> Option<DeclarationId> {
        match self {
            Self::Array(array) => array.declaration,
            Self::Buffer(buffer) => Some(buffer.declaration),
            Self::Container(container) => container.declaration,
        }
    }

    /// The complete path one element read selects below the base's root:
    /// the selections that reach the base, and the element `offset` selects
    /// [LIV-2].
    fn indexed_element_path(&self, offset: CapturedValue) -> Vec<PlaceStep> {
        let mut path = match self {
            Self::Array(array) => match &array.root {
                CheckedArrayRoot::Binding { fields, .. } => {
                    fields.iter().copied().map(PlaceStep::Field).collect()
                }
                CheckedArrayRoot::Constant(_) => Vec::new(),
            },
            Self::Buffer(buffer) => buffer
                .root
                .fields
                .iter()
                .copied()
                .map(PlaceStep::Field)
                .collect(),
            Self::Container(container) => container
                .root
                .path
                .iter()
                .map(CheckedPlaceStep::place_step)
                .collect(),
        };
        path.push(PlaceStep::Index(offset));
        path
    }

    /// The resolved place of the indexed base, for [LIV-2]'s element read-out
    /// matching. A slice indexes storage its own descriptor names and is not
    /// a commit target, so it has none here.
    fn indexed_base_place(&self) -> Option<ResolvedPlace> {
        match self {
            Self::Array(array) => array.resolved_place(),
            Self::Buffer(buffer) => Some(buffer.resolved.clone()),
            Self::Container(container) => Some(container.resolved.clone()),
        }
    }

    fn element_type(&self, checker: &Checker<'_, '_, '_, '_>) -> Result<CheckedType, CheckStop> {
        match self {
            Self::Array(array) => Ok(array.element_type),
            Self::Buffer(buffer) => Ok(buffer.element_type),
            Self::Container(container) => checker.element_type(
                container
                    .root
                    .element()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            ),
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    fn constant_storage_place(
        &self,
        constant: super::super::super::model::CheckedConstantId,
        suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
        require_named_offsets: bool,
    ) -> Result<CheckedContainerPlace, CheckStop> {
        let value = self.constant(constant)?;
        let (path, ty, offsets) = self.resolve_storage_path(
            suffixes,
            value.ty,
            bindings,
            function,
            loop_depth,
            require_named_offsets,
        )?;
        let mut resolved = ResolvedPlace {
            root: PlaceRoot::Constant(constant),
            path: Vec::new(),
        };
        resolved
            .path
            .extend(path.iter().map(CheckedPlaceStep::place_step));
        Ok(CheckedContainerPlace {
            root: CheckedContainerRoot {
                root: PlaceRoot::Constant(constant),
                path,
                ty,
            },
            resolved,
            // A place rooted in a named const [CONST-2] is immutable static
            // storage and is rooted in no writable declaration.
            declaration: None,
            offsets,
        })
    }

    pub(super) fn check_constant_storage_read(
        &self,
        node: NodeId,
        constant: super::super::super::model::CheckedConstantId,
        suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let place = self.constant_storage_place(
            constant,
            suffixes,
            bindings,
            function,
            options.loop_depth,
            false,
        )?;
        if options.explicit_move
            && place
                .root
                .path
                .iter()
                .all(|step| matches!(step, CheckedPlaceStep::Field(_)))
        {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "read the constant's copy scalar without `move`",
                },
            );
        }
        self.check_storage_read(node, place, bindings, options)
    }


    pub(in crate::semantic::check) fn layout_ceiling(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<CheckedLayoutCeiling, CheckStop> {
        let mut visiting = HashSet::new();
        self.layout_ceiling_inner(ty, &mut visiting).ok_or_else(|| {
            self.issue_value(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation)
        })
    }

    /// Recomputes the OP-9 ceiling after a generic GoalTemplate's element
    /// type has been instantiated. Keeping this calculation at the type
    /// authority prevents a symbolic template's conservative ceiling from
    /// becoming the identity of a concrete call requirement.
    pub(in crate::semantic::check) fn instantiated_layout_ceiling(
        &self,
        ty: CheckedType,
    ) -> Option<CheckedLayoutCeiling> {
        self.layout_ceiling_inner(ty, &mut HashSet::new())
    }

    fn layout_ceiling_inner(
        &self,
        ty: CheckedType,
        visiting: &mut HashSet<NominalId>,
    ) -> Option<CheckedLayoutCeiling> {
        fn finish(size: CheckedLayoutMagnitude, align: u64) -> Option<CheckedLayoutCeiling> {
            if align == 0 {
                return None;
            }
            let stride = match round_up_layout_magnitude(size, align) {
                CheckedLayoutMagnitude::Finite(0) => CheckedLayoutMagnitude::Finite(1),
                stride => stride,
            };
            Some(CheckedLayoutCeiling {
                size,
                align,
                stride,
            })
        }
        let primitive = |bytes| finish(CheckedLayoutMagnitude::Finite(bytes), bytes.max(1));
        match ty {
            CheckedType::Unit | CheckedType::Bool => primitive(1),
            CheckedType::Integer(integer) => primitive(u64::from(integer.width() / 8)),
            CheckedType::Float(float) => primitive(u64::from(float.width() / 8)),
            CheckedType::Array { element, length } => {
                let length = length.value()?;
                if length == 0 {
                    return finish(CheckedLayoutMagnitude::Finite(0), 1);
                }
                let element =
                    self.layout_ceiling_inner(self.element_type(element).ok()?, visiting)?;
                finish(
                    multiply_layout_magnitude(element.stride, length),
                    element.align,
                )
            }
            CheckedType::Buffer { .. } => finish(CheckedLayoutMagnitude::Finite(32), 16),
            // [OP-9]: a `Vector` descriptor and a provider are one
            // (32, 16) pair each; a `FixedVector` is its element pair
            // repeated `n` times followed by its two (8, 8) descriptor words,
            // so its aggregate alignment is `max(align_ceiling(T), 8)`.
            CheckedType::Vector { .. } | CheckedType::Heap { .. } | CheckedType::Extent { .. } => {
                finish(CheckedLayoutMagnitude::Finite(32), 16)
            }
            CheckedType::FixedVector { element, length } => {
                let length = length.value()?;
                let element =
                    self.layout_ceiling_inner(self.element_type(element).ok()?, visiting)?;
                let align = element.align.max(8);
                let elements = multiply_layout_magnitude(element.stride, length);
                let body = round_up_layout_magnitude(elements, 8);
                finish(
                    add_layout_magnitude(body, CheckedLayoutMagnitude::Finite(16)),
                    align,
                )
            }
            CheckedType::Slice { .. } => None,
            CheckedType::Nominal(id) => {
                if !visiting.insert(id) {
                    return None;
                }
                let nominal = self.nominal(id).ok()?;
                let result = match &nominal.kind {
                    CheckedNominalKind::Box { .. } => {
                        finish(CheckedLayoutMagnitude::Finite(16), 16)
                    }
                    CheckedNominalKind::Arena { .. } | CheckedNominalKind::ArenaStorage => None,
                    CheckedNominalKind::Opaque => finish(CheckedLayoutMagnitude::Finite(32), 16),
                    CheckedNominalKind::Struct { fields } => {
                        self.aggregate_layout_ceiling(fields.iter().map(|field| field.ty), visiting)
                    }
                    CheckedNominalKind::Enum { variants }
                        if variants.iter().all(|variant| variant.fields.is_empty()) =>
                    {
                        primitive(if variants.len() <= 2 { 1 } else { 4 })
                    }
                    CheckedNominalKind::Enum { variants } => self.aggregate_layout_ceiling(
                        std::iter::once(CheckedType::Integer(IntegerType::U32)).chain(
                            variants
                                .iter()
                                .flat_map(|variant| variant.fields.iter().map(|field| field.ty)),
                        ),
                        visiting,
                    ),
                };
                visiting.remove(&id);
                result
            }
            // Symbolic generic bodies are validated but never lowered. A
            // bound-wide ceiling lets that structural pass retain the same
            // expression shape; every concrete instance is checked again and
            // receives its exact ceiling. Int and Float are at most 64 bits.
            CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => primitive(8),
            // FN-2's symbolic pass retains an abstract upper observation for
            // an opaque parameter. Allocation-fit predicates, including a
            // direct buffer_fits::<T>, are checked again at every concrete
            // instance with that instance's exact ceiling. This does not
            // broaden the legacy buffer element domain.
            CheckedType::Generic(_) => Some(CheckedLayoutCeiling {
                size: CheckedLayoutMagnitude::AboveU64,
                align: 16,
                stride: CheckedLayoutMagnitude::AboveU64,
            }),
        }
    }

    fn aggregate_layout_ceiling(
        &self,
        fields: impl IntoIterator<Item = CheckedType>,
        visiting: &mut HashSet<NominalId>,
    ) -> Option<CheckedLayoutCeiling> {
        let mut size = CheckedLayoutMagnitude::Finite(0);
        let mut align = 1_u64;
        for ty in fields {
            let field = self.layout_ceiling_inner(ty, visiting)?;
            size = round_up_layout_magnitude(size, field.align);
            size = add_layout_magnitude(size, field.size);
            align = align.max(field.align);
        }
        size = round_up_layout_magnitude(size, align);
        let stride = match size {
            CheckedLayoutMagnitude::Finite(0) => CheckedLayoutMagnitude::Finite(1),
            size => size,
        };
        Some(CheckedLayoutCeiling {
            size,
            align,
            stride,
        })
    }

    /// One [MSR-1] measure former read as an [OP-1] row.
    ///
    /// The four spellings share one judgment because they are one operation
    /// family over one place: which measure the row reads is the selected
    /// measure, and the measure table [MSR-1] gives its value per measured
    /// type. Nothing here is keyed on the spelling beyond that selection.
    pub(in crate::semantic::check) fn check_flat_measure(
        &self,
        node: NodeId,
        measure: CheckedMeasure,
        _function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        _loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_named_operation_arguments(node, measure.spelling())?;
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, 1)?;
        // [CALL-4] a measure over an admitted result place is an operand with
        // no per-family admission. A result binder is the clause's own datum
        // rather than a place, so the former reads it here instead of through
        // the ordinary indexed place.
        if let Some((ordinal, ty)) = self.postcondition_selector_is_bare_atom(atoms[0])?
            && measured_kind_of(ty).is_some()
        {
            return Ok(TypedExpression::owned(
                CheckedExpression::PostconditionResultMeasure {
                    measure,
                    ordinal,
                    ty,
                },
                EffectSet::NONE,
            ));
        }
        // [OP-2] a measure former's selected element type is the base place's
        // own; the result is `own u64` for every row, so nothing else consults
        // it.
        let place = self.check_indexed_atom_place(atoms[0], bindings, _function, _loop_depth)?;
        let mut effects = EffectSet::NONE;
        match &place {
            CheckedIndexedPlace::Array(_) => {}
            // [MSR-2] a measure's support is the resolved place of the
            // measured value itself, so reading one is an ordinary read of
            // that place.
            CheckedIndexedPlace::Container(container) => {
                self.check_commit_place_live(&container.resolved, atoms[0], true)?;
                for path in self.effect_paths_for_descriptor(
                    atoms[0],
                    &container.resolved,
                    bindings,
                    measure,
                )? {
                    effects.add_read(path);
                }
                // [EFF-2] an offset occurring inside the measured place is
                // read where the place is formed, exactly as the operand of
                // a written subscript is.
                effects = effects.union(container.offsets.effects.clone());
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                for path in self.effect_paths_for_place(atoms[0], &buffer.resolved, bindings)? {
                    effects.add_read(path);
                }
            }
        }
        Ok(TypedExpression::owned(
            self.measure_of_indexed_place(measure, place, atoms[0])?,
            effects,
        ))
    }

    /// The [MSR-1] measure read over one already-resolved indexed place.
    ///
    /// It is the tail of the reader row above and the whole of an [INV-1]
    /// affine measure factor, which reads no storage and forms no loan and
    /// therefore reaches only this part.
    pub(in crate::semantic::check) fn measure_of_indexed_place(
        &self,
        measure: CheckedMeasure,
        place: CheckedIndexedPlace,
        operand: NodeId,
    ) -> Result<CheckedExpression, CheckStop> {
        Ok(match place {
            CheckedIndexedPlace::Array(array) => CheckedExpression::ArrayMeasure {
                measure,
                root: array.root,
                length: array.length,
            },
            CheckedIndexedPlace::Buffer(buffer) => CheckedExpression::BufferMeasure {
                measure,
                root: buffer.root,
            },
            CheckedIndexedPlace::Container(container) => {
                // [MSR-1]: a measure the table gives no row is the
                // ordinary [TYPE-5] operand rejection, carried by the
                // measured types the table does have a row for.
                let measured = container
                    .root
                    .measured()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if matches!(measure.cell(measured), MeasureCell::Absent) {
                    return self.issue_node(
                        SemanticRule::Type5,
                        operand,
                        SemanticIssueKind::type_mismatch(
                            "a measured place whose measure table has this row",
                            self.checked_type_name(container.root.ty)?,
                        ),
                    );
                }
                CheckedExpression::ContainerMeasure {
                    measure,
                    root: container.root,
                }
            }
        })
    }

    /// Extends an already resolved run base to its final selected storage.
    /// Every offset remains in source order, before a mutation's RHS.
    fn extend_storage_place(
        &self,
        mut place: CheckedContainerPlace,
        suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<CheckedContainerPlace, CheckStop> {
        let (path, ty, offsets) = self.resolve_storage_path(
            suffixes,
            place.root.ty,
            bindings,
            function,
            loop_depth,
            false,
        )?;
        place.resolved.path.extend(path.iter().map(CheckedPlaceStep::place_step));
        place.root.path.extend(path);
        place.root.ty = ty;
        place.offsets.effects = place.offsets.effects.union(offsets.effects);
        place.offsets.accesses.extend(offsets.accesses);
        Ok(place)
    }

    fn check_storage_read(
        &self,
        node: NodeId,
        place: CheckedContainerPlace,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let liveness = self.check_commit_place_live(&place.resolved, node, false);
        let copy = self.is_copy_type(place.root.ty)?;
        let read_out = liveness.is_ok()
            && options.explicit_move
            && !copy
            && self.take_commit_element_read_out(&place.resolved);
        // [DIAG-1] an explicit affine element move without an admitted
        // read-out violates TYPE-2 even when that same use is also dead under
        // OWN-1. TYPE-2 is defined first and owns their simultaneous event.
        // Liveness still prevents spending another read-out and owns later
        // scalar reads, which have no affine-element violation.
        if options.explicit_move && !copy && !read_out {
            return self.issue_node(
                SemanticRule::Type2,
                node,
                SemanticIssueKind::AffineElementMove {
                    mechanical_fix: "exchange the element with `let old = replace p = e;`",
                },
            );
        }
        liveness?;
        if !copy && !read_out {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "exchange the element with `let old = replace p = e;`",
                },
            );
        }
        if copy && options.explicit_move && self.judges_class_spelling() {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the indexed copy place without `move`",
                },
            );
        }
        let mut effects = place.offsets.effects;
        for path in self.effect_paths_for_place(node, &place.resolved, bindings)? {
            effects.add_read(path);
        }
        let mut accesses = place.offsets.accesses;
        accesses.push(PlaceAccess {
            place: place.resolved,
            kind: AccessKind::Read,
        });
        Ok(TypedExpression {
            expression: CheckedExpression::ReadStorage {
                carrier: self.tree.path(node)?.clone(),
                root: place.root,
            },
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects,
            accesses,
        })
    }

    /// [LIV-2] whether this subscript read is the read-out of an element
    /// target of the `set` whose right-hand side is being checked.
    ///
    /// The offset is read here before the ordinary judgment below reaches it,
    /// so that the admission is decided from the same written offset the
    /// target carried. An offset that does not check, or that this rule
    /// cannot decide, matches nothing and leaves every diagnostic below in
    /// its own place: no read-out is recorded and the affine rejection stands.
    fn element_read_out(
        &self,
        function: &FunctionSignature,
        indexed: &CheckedIndexedPlace,
        suffix: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<bool, CheckStop> {
        let Some(mut place) = indexed.indexed_base_place() else {
            return Ok(false);
        };
        let Some(offset_node) = self.subscript_offset(suffix)? else {
            return Ok(false);
        };
        let mut probe = bindings.clone();
        let Ok(offset) = self.check_atom(function, offset_node, &mut probe, loop_depth) else {
            return Ok(false);
        };
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || offset.mode != CheckedMode::Own
        {
            return Ok(false);
        }
        place.push_subscript(
            Self::captured_of(offset_node, &offset.expression).unwrap_or(CapturedValue::unknown()),
        );
        Ok(self.take_commit_element_read_out(&place))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_index_use(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        place: NodeId,
        suffixes: &[NodeId],
        subscript: usize,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let suffix = suffixes[subscript];
        let indexed = self
            .check_indexed_place(
                place,
                bindings,
                &suffixes[..subscript],
                suffix,
                function,
                options.loop_depth,
            )?
            .into_element_storage()?;
        if let CheckedIndexedPlace::Container(container) = indexed {
            let container = self.extend_storage_place(
                container,
                &suffixes[subscript..],
                bindings,
                function,
                options.loop_depth,
            )?;
            return self.check_storage_read(use_node, container, bindings, options);
        }
        if subscript + 1 != suffixes.len() {
            // General suffix paths are legal source. Even when the legacy
            // representation cannot carry a valid projection, selecting a
            // nonexistent field of its known element type is TYPE-5.
            self.resolve_struct_path(&suffixes[subscript + 1..], indexed.element_type(self)?)?;
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, place);
        }
        // [LIV-2, BLK-1] the one affine element read a subscript admits: a
        // `move P[i]` in the right-hand side of the `set` whose own target is
        // `P[i]`. The element leaves through the read-out and the same
        // statement's commit reinitialises the slot at one commit, so no
        // program point sees the slot empty and no second owner is minted —
        // which is exactly the ground [SET-2]'s exchange stands on. Every
        // other affine subscript read is the rejection below.
        let element_read_out = options.explicit_move
            && !self.is_copy_type(indexed.element_type(self)?)?
            && self.element_read_out(function, &indexed, suffix, bindings, options.loop_depth)?;
        // [TYPE-2] affine elements leave and enter their slots only through
        // [SET-2] replacement and are read in place through borrowed match:
        // a subscript read would mint a second owner of the stored value, so
        // both the bare and the `move` spelling reject here.
        if !element_read_out && !self.is_copy_type(indexed.element_type(self)?)? {
            if options.explicit_move {
                return self.issue_node(
                    SemanticRule::Type2,
                    use_node,
                    SemanticIssueKind::AffineElementMove {
                        mechanical_fix: "exchange the element with `let old = replace p = e;`",
                    },
                );
            }
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "exchange the element with `let old = replace p = e;`",
                },
            );
        }
        if !element_read_out && options.explicit_move && self.judges_class_spelling() {
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the indexed copy place without `move`",
                },
            );
        }
        if matches!(indexed, CheckedIndexedPlace::Container(_)) {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let offset_node = self
            .subscript_offset(suffix)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let offset = self.check_atom(function, offset_node, bindings, options.loop_depth)?;
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || offset.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRule::Type5,
                offset_node,
                SemanticIssueKind::type_mismatch(
                    "own u64",
                    self.checked_value_name(offset.mode, offset.expression.ty())?,
                ),
            );
        }
        // A subscript is not an [EFF-2] trap source: an accepted subscript
        // is discharged [OP-4] and executes no runtime check. Retain only the
        // psuffix identity that the [ENT-6] obligation judgment and [OP-4]
        // rejection cite.
        let obligation = self.tree.path(suffix)?.clone();
        let mut effects = offset.effects;
        let mut accesses = offset.accesses;
        match &indexed {
            CheckedIndexedPlace::Array(array) => {
                if let Some(place) = array.resolved_place() {
                    accesses.push(PlaceAccess {
                        place,
                        kind: AccessKind::Read,
                    });
                }
            }
            CheckedIndexedPlace::Buffer(buffer) => accesses.push(PlaceAccess {
                place: buffer.resolved.clone(),
                kind: AccessKind::Read,
            }),
            CheckedIndexedPlace::Container(_) => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }
        let expression = match indexed {
            CheckedIndexedPlace::Array(array) => CheckedExpression::ArrayIndex {
                carrier: self.tree.path(use_node)?.clone(),
                root: array.root,
                element_type: array.element_type,
                length: array.length,
                offset: Box::new(offset.expression),
                obligation,
                target_domain: CheckedTargetDomainObligation::ElementAddress,
            },
            CheckedIndexedPlace::Buffer(buffer) => {
                for path in self.effect_paths_for_place(use_node, &buffer.resolved, bindings)? {
                    effects.add_read(path);
                }
                CheckedExpression::BufferIndex {
                    carrier: self.tree.path(use_node)?.clone(),
                    root: buffer.root,
                    offset: Box::new(offset.expression),
                    obligation,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                }
            }
            CheckedIndexedPlace::Container(_) => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        };
        Ok(TypedExpression {
            expression,
            mode: CheckedMode::Own,
            reference: None,
            reference_value: false,
            effects,
            accesses,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::semantic::check) fn check_indexed_set_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        suffixes: &[NodeId],
        subscript: usize,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<MutationTarget, CheckStop> {
        let suffix = suffixes[subscript];
        let indexed = self
            .check_indexed_place(
                node,
                bindings,
                &suffixes[..subscript],
                suffix,
                function,
                loop_depth,
            )?
            .into_element_storage()?;
        if let CheckedIndexedPlace::Container(container) = indexed {
            if container.root.binding().is_none() {
                return self.issue_node(
                    SemanticRule::Const2,
                    node,
                    SemanticIssueKind::ImmutableSetTarget,
                );
            }
            let container = self.extend_storage_place(
                container,
                &suffixes[subscript..],
                bindings,
                function,
                loop_depth,
            )?;
            self.check_mutation_target_class(node, container.root.ty)?;
            let mut effects = container.offsets.effects;
            for path in self.effect_paths_for_place(node, &container.resolved, bindings)? {
                effects.add_write(path);
            }
            let Some(declaration) = container.declaration else {
                return self.issue_node(
                    SemanticRule::Const2,
                    node,
                    SemanticIssueKind::ImmutableSetTarget,
                );
            };
            return Ok(MutationTarget {
                declaration,
                place: container.resolved,
                through_reference: None,
                element: true,
                target: CheckedSetTarget::Storage(container.root),
                effects,
                unsupported: None,
            });
        }
        if subscript + 1 != suffixes.len() {
            self.resolve_struct_path(&suffixes[subscript + 1..], indexed.element_type(self)?)?;
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
        }
        if matches!(indexed, CheckedIndexedPlace::Container(_)) {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let offset_node = self
            .subscript_offset(suffix)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let offset = self.check_atom(function, offset_node, bindings, loop_depth)?;
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || offset.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRule::Type5,
                offset_node,
                SemanticIssueKind::type_mismatch(
                    "own u64",
                    self.checked_value_name(offset.mode, offset.expression.ty())?,
                ),
            );
        }
        // As in the read path, retain only the psuffix identity for [ENT-6];
        // an accepted target contributes no runtime check or trap carrier.
        let obligation = self.tree.path(suffix)?.clone();
        // [SET-1]/[SET-2] partition the selected element class exactly as
        // they partition every other final selected type.
        let element_type = match &indexed {
            CheckedIndexedPlace::Array(array) => array.element_type,
            CheckedIndexedPlace::Buffer(buffer) => buffer.root.element.ty(),
            CheckedIndexedPlace::Container(_) => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        };
        self.check_mutation_target_class(node, element_type)?;
        let offset_place = Self::captured_of(offset_node, &offset.expression).unwrap_or(CapturedValue::unknown());
        let mut effects = offset.effects;
        let (declaration, place, target) = match indexed {
            CheckedIndexedPlace::Array(array) => {
                let Some(declaration) = array.declaration else {
                    return self.issue_node(
                        SemanticRule::Const2,
                        node,
                        SemanticIssueKind::ImmutableSetTarget,
                    );
                };
                let resolved = array.resolved_place().ok_or_else(|| {
                    self.issue_value(
                        SemanticRule::Const2,
                        node,
                        SemanticIssueKind::ImmutableSetTarget,
                    )
                })?;
                let CheckedArrayRoot::Binding { binding, fields } = array.root else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                (
                    declaration,
                    resolved,
                    CheckedSetTarget::ArrayIndex(Box::new(CheckedArraySetTarget {
                        binding,
                        fields,
                        array_type: array.array_type,
                        element_type: array.element_type,
                        length: array.length,
                        offset: offset.expression,
                        obligation,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                for path in self.effect_paths_for_place(node, &buffer.resolved, bindings)? {
                    effects.add_write(path);
                }
                (
                    buffer.declaration,
                    buffer.resolved.clone(),
                    CheckedSetTarget::BufferIndex(Box::new(CheckedBufferSetTarget {
                        root: buffer.root,
                        offset: offset.expression,
                        obligation,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
            CheckedIndexedPlace::Container(_) => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        };
        // [MSR-2, LIV-2] a subscript target writes one element of `place`,
        // never the run's own storage, so disjointness and the measure kill
        // both read the element flag rather than the place alone.
        let mut place = place;
        place.push_subscript(offset_place);
        Ok(MutationTarget {
            declaration,
            place,
            through_reference: None,
            element: true,
            target,
            effects,
            unsupported: None,
        })
    }

    /// Resolves a storage place's suffixes into typed field selections and
    /// subscripts, in written order [MSR-1, OWN-7]. Measures and addressed
    /// borrows use the same path and owe the same subscript obligations.
    ///
    /// `len_of(table[i])` is a term, so a measured place is not a field path.
    /// A subscript inside one is an [OP-4] occurrence like every other: it
    /// selects the base's [BLK-1] element and owes `i < len_of(base)`, which
    /// is submitted where the place is formed [MSR-4]. Its offset must be a
    /// term the place relations can name — [OWN-7] decides two subscripts by
    /// their offsets and [ENT-5] takes each offset's own support into every
    /// enclosing measure — so a written literal, a live `own u64` binding and
    /// an in-scope const generic [MSR-6] are admitted and every other offset
    /// is the explicit unsupported capability when a tracked identity is
    /// required. Ordinary evaluated read and write offsets may instead stay
    /// opaque: their expressions still owe bounds, and establish no equal or
    /// distinct place identity.
    pub(in crate::semantic::check) fn resolve_storage_path(
        &self,
        suffixes: &[NodeId],
        mut ty: CheckedType,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
        require_named_offsets: bool,
    ) -> Result<(Vec<CheckedPlaceStep>, CheckedType, CarriedOperands), CheckStop> {
        let mut path = Vec::new();
        let mut carried = CarriedOperands::default();
        for (position, &suffix) in suffixes.iter().enumerate() {
            let Some(offset_node) = self.subscript_offset(suffix)? else {
                let (fields, selected) =
                    self.resolve_struct_path(&suffixes[position..=position], ty)?;
                path.extend(fields.into_iter().map(CheckedPlaceStep::Field));
                ty = selected;
                continue;
            };
            // [OP-4] each suffix selects the complete element type of its
            // already-typed base. Array storage can be nested in a run slot.
            let element_type = match ty {
                CheckedType::Array { element, .. }
                | CheckedType::FixedVector { element, .. }
                | CheckedType::Vector { element, .. } => self.element_type(element)?,
                CheckedType::Buffer { .. } | CheckedType::Slice { .. } => {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, suffix);
                }
                _ => {
                    return self.issue_node(
                        SemanticRule::Op4,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "an indexable base",
                            self.checked_type_name(ty)?,
                        ),
                    );
                }
            };
            let mut probe = bindings.clone();
            let offset = self.check_atom(function, offset_node, &mut probe, loop_depth)?;
            if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
                || offset.mode != CheckedMode::Own
            {
                return self.issue_node(
                    SemanticRule::Type5,
                    offset_node,
                    SemanticIssueKind::type_mismatch(
                        "own u64",
                        self.checked_value_name(offset.mode, offset.expression.ty())?,
                    ),
                );
            }
            let captured = Self::captured_of(offset_node, &offset.expression);
            if require_named_offsets && captured.is_none() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, offset_node);
            }
            let captured = captured.unwrap_or(CapturedValue::unknown());
            carried.effects = carried.effects.union(offset.effects);
            carried.accesses.extend(offset.accesses);
            path.push(CheckedPlaceStep::Subscript(Box::new(
                CheckedPlaceSubscript {
                    base_type: ty,
                    element_type,
                    offset: offset.expression,
                    obligation: self.tree.path(suffix)?.clone(),
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    captured,
                },
            )));
            ty = element_type;
        }
        Ok((path, ty, carried))
    }

    /// The immutable value one index expression produced at this occurrence
    /// [REF-1, OWN-7].
    ///
    /// `occurrence` is the source node the offset was evaluated at, which is
    /// what names the captured value: [REF-1] fixes an index inside a path at
    /// formation, and two steps carrying one occurrence hold one value
    /// whatever the program did between the two places' formation. The term
    /// beside it is how the entailment fragment reads that value, classified
    /// over the checked operand and never over its spelling: a literal or
    /// named integer const is its mathematical value, a binding read is that
    /// binding, and a const generic is fixed at instantiation [FN-2].
    pub(in crate::semantic::check) fn captured_of(
        occurrence: NodeId,
        offset: &CheckedExpression,
    ) -> Option<CapturedValue> {
        let capture = CaptureId(u32::try_from(occurrence.index()).unwrap_or(u32::MAX - 1));
        match offset {
            CheckedExpression::Constant(super::super::super::model::CheckedValue::Integer {
                bits,
                ..
            }) => Some(CapturedValue::new(capture, CapturedTerm::Literal(*bits))),
            CheckedExpression::NamedConstant {
                value: super::super::super::model::CheckedValue::Integer { bits, .. },
                ..
            } => Some(CapturedValue::new(capture, CapturedTerm::Literal(*bits))),
            CheckedExpression::Constant(
                super::super::super::model::CheckedValue::ConstGeneric { declaration, .. },
            ) => Some(CapturedValue::new(capture, CapturedTerm::Const(*declaration))),
            CheckedExpression::Binding {
                binding,
                consume_root: false,
                ..
            } => Some(CapturedValue::new(capture, CapturedTerm::Binding(*binding))),
            _ => None,
        }
    }

    pub(in crate::semantic::check) fn check_indexed_atom_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        if self.has_fixed(node, FixedTerminal::Move)? {
            return self.issue_node(
                SemanticRule::Type5,
                node,
                SemanticIssueKind::type_mismatch(
                    "a place, which a subscript indexes",
                    "a written `move`, which consumes rather than indexes",
                ),
            );
        }
        let place = self
            .tree
            .first_child_with(node, Production::Place)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::type_mismatch(
                        "a place, which a subscript indexes",
                        "an atom that is not a place",
                    ),
                )
            })?;
        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        self.check_indexed_place(place, bindings, &suffixes, place, function, loop_depth)
    }

    /// One indexable place written through an explicit `deref` [TYPE-7].
    ///
    /// The `deref` names a reference's referent, so the place is resolved by
    /// the ordinary [REF-1] walk and the written suffixes continue it. The
    /// v0.59 companion of this function also had to answer for a view
    /// descriptor reached through a holder; views are gone, so one indexable
    /// container place is the whole answer.
    fn check_dereferenced_indexed_place(
        &self,
        node: NodeId,
        pbase: NodeId,
        base_suffixes: &[NodeId],
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let inner = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let inner = self.resolve_explicit_place(node, inner, bindings)?;
        let mut place = self.resolve_explicit_dereference(node, pbase, inner, bindings)?;
        let (binding, mut path) = self.explicit_container_path(&place.expression, node)?;
        let (suffix_path, ty, offsets) = self.resolve_storage_path(
            base_suffixes,
            place.ty,
            bindings,
            function,
            loop_depth,
            true,
        )?;
        place
            .resolved
            .path
            .extend(suffix_path.iter().map(CheckedPlaceStep::place_step));
        path.extend(suffix_path);
        let fields = field_prefix(&path);
        match ty {
            CheckedType::Buffer { element } => {
                let Some(fields) = fields else {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
                };
                Ok(CheckedIndexedPlace::Buffer(CheckedBufferPlace {
                    root: CheckedBufferRoot {
                        binding,
                        fields,
                        element,
                    },
                    declaration: place.declaration,
                    element_type: element.ty(),
                    resolved: place.resolved,
                }))
            }
            // [OP-4] the indexable bases, reached through `deref` exactly as
            // an inline one is: a run is one measured place wherever it is
            // reached from [MSR-1].
            CheckedType::Array { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Extent { .. } => {
                Ok(CheckedIndexedPlace::Container(CheckedContainerPlace {
                    root: CheckedContainerRoot {
                        root: PlaceRoot::Binding(binding),
                        path,
                        ty,
                    },
                    resolved: place.resolved,
                    declaration: Some(place.declaration),
                    offsets,
                }))
            }
            _ => self.issue_node(
                SemanticRule::Op4,
                node,
                SemanticIssueKind::type_mismatch(
                    "an indexable base",
                    self.checked_type_name(ty)?,
                ),
            ),
        }
    }

    /// The binding and typed storage path one checked place expression
    /// selects, for a target or read the lowering addresses directly.
    pub(in crate::semantic::check) fn explicit_container_path(
        &self,
        expression: &CheckedExpression,
        node: NodeId,
    ) -> Result<(crate::semantic::model::BindingId, Vec<CheckedPlaceStep>), CheckStop> {
        match expression {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => Ok((*binding, Vec::new())),
            CheckedExpression::BoxDeref { nominal, value, .. } => {
                let (binding, mut path) = self.explicit_container_path(value, node)?;
                path.push(CheckedPlaceStep::BoxReferent(*nominal));
                Ok((binding, path))
            }
            CheckedExpression::ProjectValue { value, field, .. } => {
                let (binding, mut path) = self.explicit_container_path(value, node)?;
                path.push(CheckedPlaceStep::Field(*field));
                Ok((binding, path))
            }
            _ => self.unsupported(UnsupportedSemanticFeature::CompositeValues, node),
        }
    }

    /// Checks "pbase plus the given suffix run" as one place of indexable
    /// storage. A subscript passes the chain before its own `psuffix` and
    /// anchors its wrong-base judgment there [OP-4]; a `len` or `slice_of`
    /// operand passes the complete chain and anchors at the place node.
    fn check_indexed_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        base_suffixes: &[NodeId],
        anchor: NodeId,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_indexed_place(
                node,
                pbase,
                base_suffixes,
                bindings,
                function,
                loop_depth,
            );
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let (root, binding, declaration, path, ty, offsets) = match class {
            DeclarationClass::Value => {
                let local = bindings
                    .get(&declaration)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !local.live {
                    return self.issue_node(
                        SemanticRule::Own1,
                        node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                let (path, ty, offsets) = self.resolve_storage_path(
                    base_suffixes,
                    local.ty,
                    bindings,
                    function,
                    loop_depth,
                    true,
                )?;
                // A borrow holder written where its indexable referent is
                // required is the [TYPE-7] implicit read; a borrow of
                // something no `index` could reach falls through to the
                // operand's own mismatch below.
                if local.mode != CheckedMode::Own
                    && self.reads_implicitly_through_holder(
                        true,
                        ty,
                        RequiredReferent::IndexableStorage,
                    )?
                {
                    return self.issue_node(
                        SemanticRule::Type7,
                        node,
                        SemanticIssueKind::MissingDereference {
                            mechanical_fix: "write `deref(holder)`",
                        },
                    );
                }
                (
                    CheckedArrayRoot::Binding {
                        binding: local.binding,
                        fields: Vec::new(),
                    },
                    Some(local.binding),
                    Some(declaration),
                    path,
                    ty,
                    offsets,
                )
            }
            DeclarationClass::NamedConst => {
                if !base_suffixes.is_empty() {
                    let constant = *self
                        .constants
                        .get(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let place = self.constant_storage_place(
                        constant,
                        base_suffixes,
                        bindings,
                        function,
                        loop_depth,
                        true,
                    )?;
                    if place.root.measured().is_none() {
                        return self.issue_node(
                            SemanticRule::Type5,
                            anchor,
                            SemanticIssueKind::type_mismatch(
                                "an array, buffer, or slice place",
                                self.checked_type_name(place.root.ty)?,
                            ),
                        );
                    }
                    return Ok(CheckedIndexedPlace::Container(place));
                }
                let id = *self
                    .constants
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                (
                    CheckedArrayRoot::Constant(id),
                    None,
                    None,
                    Vec::new(),
                    self.constant(id)?.ty,
                    CarriedOperands::default(),
                )
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        // Every base but a run's carries a flat element [TYPE-2], so a
        // subscript inside one selects storage this version has no measured
        // place for; the field prefix is what those branches read.
        let fields = field_prefix(&path);
        match ty {
            CheckedType::Array { element, length } if fields.is_some() => {
                let fields = fields.ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let root = match root {
                    CheckedArrayRoot::Binding { binding, .. } => {
                        CheckedArrayRoot::Binding { binding, fields }
                    }
                    CheckedArrayRoot::Constant(id) => {
                        if !fields.is_empty() {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        CheckedArrayRoot::Constant(id)
                    }
                };
                Ok(CheckedIndexedPlace::Array(CheckedArrayPlace {
                    root,
                    declaration,
                    array_type: ty,
                    element_type: self.element_type(element)?,
                    length,
                }))
            }
            CheckedType::Buffer { element } => {
                let (Some(binding), Some(declaration)) = (binding, declaration) else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let Some(fields) = fields else {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, anchor);
                };
                let resolved_fields = fields.clone();
                Ok(CheckedIndexedPlace::Buffer(CheckedBufferPlace {
                    root: CheckedBufferRoot {
                        binding,
                        fields,
                        element,
                    },
                    declaration,
                    element_type: element.ty(),
                    resolved: ResolvedPlace::fields(binding, resolved_fields),
                }))
            }
            // [TYPE-7] owns the implicit-read case exclusively: a `box` holder
            // written where its indexable referent would be required is
            // rejected citing TYPE-7 with the `deref(.)` fix, and the
            // operand's wrong-type judgment forms no rejection.
            _ if self.reads_implicitly_through_holder(
                false,
                ty,
                RequiredReferent::IndexableStorage,
            )? =>
            {
                self.issue_node(
                    SemanticRule::Type7,
                    node,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(holder)`",
                    },
                )
            }
            // [MSR-1] gives the two runs and the bump extent a measure-table
            // row and [OP-4] makes the two runs indexable bases; a `Heap<'s>`
            // has neither, so it falls through to the operand rejection
            // below.
            CheckedType::Array { .. }
            | CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Extent { .. } => {
                let (Some(binding), Some(declaration)) = (binding, declaration) else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let resolved_path: Vec<PlaceStep> =
                    path.iter().map(CheckedPlaceStep::place_step).collect();
                Ok(CheckedIndexedPlace::Container(CheckedContainerPlace {
                    root: CheckedContainerRoot {
                        root: PlaceRoot::Binding(binding),
                        path,
                        ty,
                    },
                    resolved: ResolvedPlace::from_path(binding, resolved_path),
                    declaration: Some(declaration),
                    offsets,
                }))
            }
            _ => self.issue_node(
                SemanticRule::Type5,
                anchor,
                SemanticIssueKind::type_mismatch(
                    "an array, buffer, or slice place",
                    self.checked_type_name(ty)?,
                ),
            ),
        }
    }
}

/// The field selections of a path that carries no subscript, absent where it
/// does.
fn field_prefix(path: &[CheckedPlaceStep]) -> Option<Vec<u32>> {
    path.iter()
        .map(|step| match step {
            CheckedPlaceStep::Field(field) => Some(*field),
            CheckedPlaceStep::BoxReferent(_) | CheckedPlaceStep::Subscript(_) => None,
        })
        .collect()
}

/// The [MSR-1] measure-table row one type selects, if it has one.
///
/// It is the same table [`CheckedMeasure::cell`] reads; this is only the
/// mapping from a checked type to its row, which the clause path needs before
/// it has a place.
pub(in crate::semantic::check) const fn measured_kind_of(
    ty: CheckedType,
) -> Option<super::super::super::model::MeasuredKind> {
    use super::super::super::model::MeasuredKind;
    match ty {
        CheckedType::Array { .. } => Some(MeasuredKind::Array),
        CheckedType::Buffer { .. } => Some(MeasuredKind::Buffer),
        CheckedType::Slice { .. } => Some(MeasuredKind::Slice),
        CheckedType::FixedVector { .. } => Some(MeasuredKind::FixedVector),
        CheckedType::Vector { .. } => Some(MeasuredKind::Vector),
        CheckedType::Extent { .. } => Some(MeasuredKind::Extent),
        _ => None,
    }
}

#[cfg(test)]
mod layout_magnitude_tests {
    use super::{
        CheckedLayoutMagnitude, add_layout_magnitude, multiply_layout_magnitude,
        round_up_layout_magnitude,
    };

    #[test]
    fn finite_or_above_u64_preserves_every_layout_ceiling_observation() {
        let finite = CheckedLayoutMagnitude::Finite;
        assert_eq!(multiply_layout_magnitude(finite(8), 3), finite(24));
        assert_eq!(multiply_layout_magnitude(finite(8), 0), finite(0));
        assert_eq!(
            multiply_layout_magnitude(finite(8), u64::MAX),
            CheckedLayoutMagnitude::AboveU64
        );
        assert_eq!(
            multiply_layout_magnitude(CheckedLayoutMagnitude::AboveU64, 0),
            finite(0)
        );
        assert_eq!(round_up_layout_magnitude(finite(9), 8), finite(16));
        assert_eq!(
            round_up_layout_magnitude(finite(u64::MAX), 8),
            CheckedLayoutMagnitude::AboveU64
        );
        assert_eq!(
            add_layout_magnitude(finite(u64::MAX), finite(1)),
            CheckedLayoutMagnitude::AboveU64
        );
        assert_eq!(CheckedLayoutMagnitude::AboveU64.allocation_limit(), 0);
    }
}
