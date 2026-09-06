mod borrowed;
mod slices;

use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedArrayRoot, CheckedArraySetTarget, CheckedBufferRoot, CheckedBufferSetTarget,
    CheckedConst, CheckedContainerRoot, CheckedExpression, CheckedFlatElement,
    CheckedLayoutCeiling, CheckedLayoutMagnitude, CheckedMeasure, CheckedMode, CheckedNominalKind,
    CheckedPlaceStep, CheckedPlaceSubscript, CheckedRunSetTarget, CheckedRuntimeTargetObligations,
    CheckedSetTarget, CheckedSliceRoot, CheckedSliceSetTarget, CheckedTargetDomainObligation,
    CheckedType, IntegerType, LoanStrength, MeasureCell, NominalId,
};
use super::super::super::places::{PlaceOffset, PlaceStep};
use super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, RequiredReferent, ResolvedPlace, SliceInfo,
};
use super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::{MutationForm, MutationTarget, PlaceUseOptions};

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
        let declaration = self.declaration?;
        let CheckedArrayRoot::Binding { fields, .. } = &self.root else {
            return None;
        };
        Some(ResolvedPlace {
            root: declaration,
            fields: fields.clone(),
        })
    }
}

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedBufferPlace {
    root: CheckedBufferRoot,
    declaration: DeclarationId,
    element_type: CheckedType,
    holder: Option<DeclarationId>,
    resolved: ResolvedPlace,
    borrow_kind: Option<BorrowKind>,
}

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedSlicePlace {
    root: CheckedSliceRoot,
    declaration: DeclarationId,
    descriptor: Option<BorrowInfo>,
    slice: SliceInfo,
}

#[derive(Clone)]
pub(in crate::semantic::check) enum CheckedIndexedPlace {
    Array(CheckedArrayPlace),
    Buffer(CheckedBufferPlace),
    Slice(CheckedSlicePlace),
    /// One run or bump extent [BLK-1, PROV-1]: the two runs are indexable
    /// bases [OP-4] and all three have a measure-table row [MSR-1].
    Container(CheckedContainerPlace),
}

#[derive(Clone)]
pub(in crate::semantic::check) struct CheckedContainerPlace {
    root: CheckedContainerRoot,
    resolved: ResolvedPlace,
    holder: Option<DeclarationId>,
    /// The effects and accesses of every offset occurring inside the place
    /// [EFF-2]: an offset that reads a binding is a read of that binding,
    /// wherever in the place it occurs.
    offsets: CarriedOperands,
}

/// The effects and accesses one place's own offset operands exhibit.
#[derive(Clone, Default)]
pub(in crate::semantic::check) struct CarriedOperands {
    effects: EffectSet,
    accesses: Vec<PlaceAccess>,
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
    /// The declaration this place is rooted in, where it has one. A place
    /// rooted in a named const has none, and no proof-point admission
    /// restricts a const.
    pub(in crate::semantic::check) const fn root_declaration(&self) -> Option<DeclarationId> {
        match self {
            Self::Array(array) => array.declaration,
            Self::Buffer(buffer) => Some(buffer.declaration),
            Self::Slice(slice) => Some(slice.declaration),
            Self::Container(container) => Some(container.resolved.root),
        }
    }

    /// The complete path one element read selects below the base's root:
    /// the selections that reach the base, and the element `offset` selects
    /// [LIV-2].
    fn indexed_element_path(&self, offset: PlaceOffset) -> Vec<PlaceStep> {
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
            Self::Slice(_) => Vec::new(),
            Self::Container(container) => container
                .root
                .path
                .iter()
                .map(|step| match step {
                    CheckedPlaceStep::Field(field) => PlaceStep::Field(*field),
                    CheckedPlaceStep::Subscript(subscript) => {
                        PlaceStep::Subscript(subscript.place_offset)
                    }
                })
                .collect(),
        };
        path.push(PlaceStep::Subscript(offset));
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
            Self::Slice(_) => None,
        }
    }

    fn element_type(&self) -> CheckedType {
        match self {
            Self::Array(array) => array.element_type,
            Self::Buffer(buffer) => buffer.element_type,
            Self::Slice(slice) => slice.root.element.ty(),
            // A bump extent is measured and not indexable, so an element type
            // is asked of it only after [OP-4] has already refused it.
            Self::Container(container) => container
                .root
                .element()
                .map_or(CheckedType::Unit, |element| element.ty()),
        }
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(in crate::semantic::check) fn check_array_new(
        &self,
        node: NodeId,
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
                    callee: "array_new".to_owned(),
                    declared_parameters: Vec::new(),
                },
            );
        }
        self.reject_region_bearing_storage_operation_argument(node, "array_new", function, 2, 0)?;
        // [DIAG-1] a table operation cites the rule [OP-2] selects, never FN-2,
        // which belongs to a user-generic call; [TYPE-5] mandates `array_new`'s
        // element and length, so their absence is its violation.
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type5,
                    node,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let targs = self.tree.children_with(targs, Production::Targ)?;
        let [element_arg, length_arg] = targs.as_slice() else {
            return self.issue_node(SemanticRule::Op1, node, SemanticIssueKind::InvalidOperation);
        };
        let element_node = self
            .tree
            .first_child_with(*element_arg, Production::Type)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Op1,
                    *element_arg,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let element_type = self.parse_type_with(element_node, &function.substitution)?;
        let element = match element_type {
            CheckedType::Unit => CheckedFlatElement::Unit,
            CheckedType::Integer(ty) => CheckedFlatElement::Integer(ty),
            CheckedType::Float(ty) => CheckedFlatElement::Float(ty),
            CheckedType::GenericInt(declaration) => CheckedFlatElement::GenericInt(declaration),
            CheckedType::GenericFloat(declaration) => CheckedFlatElement::GenericFloat(declaration),
            _ => {
                return self.issue_node(
                    SemanticRule::Op1,
                    element_node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let length_node = self
            .tree
            .first_child_with(*length_arg, Production::Const)?
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Op1,
                    *length_arg,
                    SemanticIssueKind::InvalidOperation,
                )
            })?;
        let length = self.parse_const_expression_with(length_node, &function.substitution)?;
        let atoms = self.operation_atoms(node, 1)?;
        let value = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if value.expression.ty() != element_type || value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(element_type)?),
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
            );
        }
        Ok(TypedExpression::owned(
            CheckedExpression::ArrayFill {
                carrier: self.tree.path(node)?.clone(),
                ty: CheckedType::Array { element, length },
                value: Box::new(value.expression),
                target_domain: CheckedTargetDomainObligation::ElementAddress,
            },
            value.effects,
        ))
    }

    pub(in crate::semantic::check) fn check_buffer_new(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_named_operation_arguments(node, "buffer_new")?;
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, 2)?;
        let length = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if length.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || length.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    "own u64",
                    self.checked_value_name(length.mode, length.expression.ty())?,
                ),
            );
        }
        // [OP-9] `buffer_new(n, v)` is the one deleted-class row whose
        // selected type comes from its *second* operand: the first is the
        // u64 element count, and the fill value supplies T.
        let value = self.check_atom(function, atoms[1], bindings, loop_depth)?;
        if value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[1],
                SemanticIssueKind::type_mismatch(
                    format!("own {}", self.checked_type_name(value.expression.ty())?),
                    self.checked_value_name(value.mode, value.expression.ty())?,
                ),
            );
        }
        let element_type = value.expression.ty();
        let element = match element_type {
            CheckedType::Unit => CheckedFlatElement::Unit,
            CheckedType::Integer(ty) => CheckedFlatElement::Integer(ty),
            CheckedType::Float(ty) => CheckedFlatElement::Float(ty),
            CheckedType::GenericInt(declaration) => CheckedFlatElement::GenericInt(declaration),
            CheckedType::GenericFloat(declaration) => CheckedFlatElement::GenericFloat(declaration),
            _ => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let layout_ceiling = self.layout_ceiling(element.ty(), node)?;
        Ok(TypedExpression::owned(
            CheckedExpression::BufferFill {
                carrier: self.tree.path(node)?.clone(),
                element,
                length: Box::new(length.expression),
                value: Box::new(value.expression),
                layout_ceiling,
                target_domains: CheckedRuntimeTargetObligations::new(),
            },
            length
                .effects
                .union(value.effects)
                .union(EffectSet::ALLOCATES_HEAP),
        ))
    }

    /// The all-`None` affine-element constructor [OP-1, OP-9]: the written
    /// element payload type is [TYPE-5] retained because no operand can
    /// supply it, the one operand is the `own u64` length, and the result is
    /// `own buffer<Option<T>>` over the interned `Option<T>` instance.
    pub(in crate::semantic::check) fn check_buffer_vacant(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_region_bearing_storage_operation_argument(
            node,
            "buffer_vacant",
            function,
            1,
            0,
        )?;
        let payload = self.retained_operation_type_argument(node, function)?;
        if !payload.is_concrete() {
            // A generic payload defers to the concrete instantiation; the
            // template-side judgment is not implemented yet.
            return self.unsupported(UnsupportedSemanticFeature::Generics, node);
        }
        let element = self.prelude_nominal(super::super::PreludeType::Option(payload))?;
        let layout_ceiling = self.layout_ceiling(CheckedType::Nominal(element), node)?;
        let atoms = self.operation_atoms(node, 1)?;
        let length = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if length.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || length.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    "own u64",
                    self.checked_value_name(length.mode, length.expression.ty())?,
                ),
            );
        }
        Ok(TypedExpression::owned(
            CheckedExpression::BufferVacant {
                carrier: self.tree.path(node)?.clone(),
                element,
                length: Box::new(length.expression),
                layout_ceiling,
                target_domains: CheckedRuntimeTargetObligations::new(),
            },
            length.effects.union(EffectSet::ALLOCATES_HEAP),
        ))
    }

    /// The total OP-9 predicate over the exact retained buffer element type.
    pub(in crate::semantic::check) fn check_buffer_fits(
        &self,
        node: NodeId,
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_region_bearing_storage_operation_argument(node, "buffer_fits", function, 1, 0)?;
        let ty = self.retained_operation_type_argument(node, function)?;
        let element = match self.buffer_element(ty)? {
            Some(_) => ty,
            None if matches!(ty, CheckedType::Buffer { .. }) => ty,
            None => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
        let atoms = self.operation_atoms(node, 1)?;
        let length = self.check_atom(function, atoms[0], bindings, loop_depth)?;
        if length.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || length.mode != CheckedMode::Own
        {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::type_mismatch(
                    "own u64",
                    self.checked_value_name(length.mode, length.expression.ty())?,
                ),
            );
        }
        let layout_ceiling = self.layout_ceiling(ty, node)?;
        Ok(TypedExpression::owned(
            CheckedExpression::BufferFits {
                carrier: self.tree.path(node)?.clone(),
                element,
                layout_ceiling,
                length: Box::new(length.expression),
            },
            length.effects,
        ))
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
                let element = self.layout_ceiling_inner(element.ty(), visiting)?;
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
                let element = self.layout_ceiling_inner(element.ty(), visiting)?;
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
                    CheckedNominalKind::SystemResource { .. } => {
                        finish(CheckedLayoutMagnitude::Finite(32), 16)
                    }
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
            // An unbounded parameter has no buffer-storable bound. Retain an
            // exact abstract upper observation for nested symbolic layout;
            // operations that require a buffer element still reject the type
            // before reaching this calculation.
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
                self.check_loan_access(
                    bindings,
                    container.holder,
                    &container.resolved,
                    AccessKind::Read,
                    atoms[0],
                )?;
                for path in self.effect_paths_for_place(&container.resolved, bindings)? {
                    effects.add_read(path);
                }
                // [EFF-2] an offset occurring inside the measured place is
                // read where the place is formed, exactly as the operand of
                // a written subscript is.
                effects = effects.union(container.offsets.effects.clone());
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                self.check_loan_access(
                    bindings,
                    buffer.holder,
                    &buffer.resolved,
                    AccessKind::Read,
                    atoms[0],
                )?;
                for path in self.effect_paths_for_place(&buffer.resolved, bindings)? {
                    effects.add_read(path);
                }
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(descriptor) = &slice.descriptor {
                    self.check_loan_access(
                        bindings,
                        Some(slice.declaration),
                        &descriptor.place,
                        AccessKind::Read,
                        atoms[0],
                    )?;
                    for path in self.effect_paths_for_place(&descriptor.place, bindings)? {
                        effects.add_read(path);
                    }
                }
                for (place, _) in slice.slice.source_places() {
                    self.check_loan_access(
                        bindings,
                        Some(slice.declaration),
                        &place,
                        AccessKind::Read,
                        atoms[0],
                    )?;
                }
                for place in slice.slice.effect_places() {
                    for path in self.effect_paths_for_place(&place, bindings)? {
                        effects.add_read(path);
                    }
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
            CheckedIndexedPlace::Slice(slice) => CheckedExpression::SliceMeasure {
                measure,
                root: slice.root,
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
        let Some(place) = indexed.indexed_base_place() else {
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
        let path = indexed.indexed_element_path(
            Self::place_offset_of(&offset.expression).unwrap_or(PlaceOffset::Opaque),
        );
        Ok(self.take_commit_element_read_out(&place, &path))
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
        if subscript + 1 != suffixes.len() {
            return self.issue_node(
                SemanticRule::Type5,
                place,
                SemanticIssueKind::type_mismatch(
                    "a subscript as the last suffix of the place",
                    "a subscript followed by another suffix",
                ),
            );
        }
        let indexed = self.check_indexed_place(
            place,
            bindings,
            &suffixes[..subscript],
            suffix,
            function,
            options.loop_depth,
        )?;
        // [LIV-2, BLK-1] the one affine element read a subscript admits: a
        // `move P[i]` in the right-hand side of the `set` whose own target is
        // `P[i]`. The element leaves through the read-out and the same
        // statement's commit reinitialises the slot at one commit, so no
        // program point sees the slot empty and no second owner is minted —
        // which is exactly the ground [SET-2]'s exchange stands on. Every
        // other affine subscript read is the rejection below.
        let element_read_out = options.explicit_move
            && !self.is_copy_type(indexed.element_type())?
            && self.element_read_out(function, &indexed, suffix, bindings, options.loop_depth)?;
        // [TYPE-2] affine elements leave and enter their slots only through
        // [SET-2] replacement and are read in place through borrowed match:
        // a subscript read would mint a second owner of the stored value, so
        // both the bare and the `move` spelling reject here.
        if !element_read_out && !self.is_copy_type(indexed.element_type())? {
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
        match &indexed {
            CheckedIndexedPlace::Array(array) => {
                if let Some(resolved) = array.resolved_place() {
                    self.check_loan_access(bindings, None, &resolved, AccessKind::Read, suffix)?;
                }
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                self.check_loan_access(
                    bindings,
                    buffer.holder,
                    &buffer.resolved,
                    AccessKind::Read,
                    suffix,
                )?;
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(descriptor) = &slice.descriptor {
                    self.check_loan_access(
                        bindings,
                        Some(slice.declaration),
                        &descriptor.place,
                        AccessKind::Read,
                        suffix,
                    )?;
                }
                for (place, _) in slice.slice.source_places() {
                    self.check_loan_access(
                        bindings,
                        Some(slice.declaration),
                        &place,
                        AccessKind::Read,
                        suffix,
                    )?;
                }
            }
            CheckedIndexedPlace::Container(container) => {
                // [OP-4] admits exactly the two runs as indexable bases; a
                // bump extent is measured and is not one.
                if container.root.element().is_none() {
                    return self.issue_node(
                        SemanticRule::Op4,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "an array, slice, buffer, or run base",
                            self.checked_type_name(container.root.ty)?,
                        ),
                    );
                }
                self.check_loan_access(
                    bindings,
                    container.holder,
                    &container.resolved,
                    AccessKind::Read,
                    suffix,
                )?;
            }
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
            CheckedIndexedPlace::Container(container) => {
                accesses.push(PlaceAccess {
                    place: container.resolved.clone(),
                    kind: AccessKind::Read,
                });
                accesses.extend(container.offsets.accesses.iter().cloned());
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(descriptor) = &slice.descriptor {
                    accesses.push(PlaceAccess {
                        place: descriptor.place.clone(),
                        kind: AccessKind::Read,
                    });
                }
                accesses.extend(slice.slice.source_places().into_iter().map(|(place, _)| {
                    PlaceAccess {
                        place,
                        kind: AccessKind::Read,
                    }
                }));
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
                for path in self.effect_paths_for_place(&buffer.resolved, bindings)? {
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
            CheckedIndexedPlace::Container(container) => {
                for path in self.effect_paths_for_place(&container.resolved, bindings)? {
                    effects.add_read(path);
                }
                effects = effects.union(container.offsets.effects);
                let element_type = container
                    .root
                    .element()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .ty();
                CheckedExpression::RunIndex {
                    carrier: self.tree.path(use_node)?.clone(),
                    root: container.root,
                    element_type,
                    offset: Box::new(offset.expression),
                    obligation,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                }
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(descriptor) = &slice.descriptor {
                    for path in self.effect_paths_for_place(&descriptor.place, bindings)? {
                        effects.add_read(path);
                    }
                }
                for place in slice.slice.effect_places() {
                    for path in self.effect_paths_for_place(&place, bindings)? {
                        effects.add_read(path);
                    }
                }
                CheckedExpression::SliceIndex {
                    carrier: self.tree.path(use_node)?.clone(),
                    root: slice.root,
                    offset: Box::new(offset.expression),
                    obligation,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                }
            }
        };
        Ok(TypedExpression {
            expression,
            mode: CheckedMode::Own,
            borrow: None,
            slice: None,
            holder: None,
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
        form: MutationForm,
    ) -> Result<MutationTarget, CheckStop> {
        let suffix = suffixes[subscript];
        if subscript + 1 != suffixes.len() {
            return self.issue_node(
                SemanticRule::Type5,
                node,
                SemanticIssueKind::type_mismatch(
                    "a subscript as the last suffix of the place",
                    "a subscript followed by another suffix",
                ),
            );
        }
        let indexed = self.check_indexed_place(
            node,
            bindings,
            &suffixes[..subscript],
            suffix,
            function,
            loop_depth,
        )?;
        match &indexed {
            CheckedIndexedPlace::Array(array) => {
                if let Some(resolved) = array.resolved_place() {
                    self.check_loan_access(bindings, None, &resolved, AccessKind::Write, node)?;
                }
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                if buffer.borrow_kind == Some(BorrowKind::Shared) {
                    return self.issue_node(
                        SemanticRule::Set1,
                        node,
                        SemanticIssueKind::InvalidSetTarget {
                            root_class: "shared borrow".to_owned(),
                            required_classes: "live own storage or a live usable &uniq referent",
                        },
                    );
                }
                self.check_loan_access(
                    bindings,
                    buffer.holder,
                    &buffer.resolved,
                    AccessKind::Write,
                    node,
                )?;
            }
            // [SET-1] as [PROV-3] amends it: a target path may traverse a
            // view exactly when that view's loan strength on its resolved
            // origin set is exclusive. A `MutSlice` root is admitted here and
            // a `Slice` root is the refusal the rule states.
            CheckedIndexedPlace::Slice(slice) => {
                if slice.root.strength != LoanStrength::Exclusive {
                    return self.issue_node(
                        SemanticRule::Set1,
                        node,
                        SemanticIssueKind::InvalidSetTarget {
                            root_class: "shared view".to_owned(),
                            required_classes:
                                "live own storage, a live usable &uniq referent, or an exclusive \
view",
                        },
                    );
                }
            }
            // [BLK-3] element access over a run is the ordinary surface and
            // needs no row: `set v[i] = e;` writes a copy element and
            // `replace v[i] = e;` exchanges an affine one.
            CheckedIndexedPlace::Container(container) => {
                self.check_loan_access(
                    bindings,
                    container.holder,
                    &container.resolved,
                    AccessKind::Write,
                    node,
                )?;
            }
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
        // [SET-1]/[SET-2] partition the element class exactly as they
        // partition every other final selected type; every v0-constructible
        // element is copy, so an element-position `replace` rejects here
        // until an affine-element constructor exists.
        let element_type = match &indexed {
            CheckedIndexedPlace::Array(array) => array.element_type,
            CheckedIndexedPlace::Buffer(buffer) => buffer.root.element.ty(),
            // [OP-4] a bump extent is no indexable base, so a `Container`
            // place reaching a subscript target is one of the two runs.
            CheckedIndexedPlace::Container(container) => match container.root.element() {
                Some(element) => element.ty(),
                None => {
                    return self.issue_node(
                        SemanticRule::Op4,
                        node,
                        SemanticIssueKind::type_mismatch(
                            "an indexable base, which a run is",
                            "a bump extent, which has no element",
                        ),
                    );
                }
            },
            CheckedIndexedPlace::Slice(slice) => slice.root.element.ty(),
        };
        self.check_mutation_target_class(node, element_type, form)?;
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
                for path in self.effect_paths_for_place(&buffer.resolved, bindings)? {
                    effects.add_write(path.clone());
                    if form.is_replace() {
                        // [SET-2, EFF-2]: one read and one write of the
                        // target's ultimate storage origin.
                        effects.add_read(path);
                    }
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
            // [BLK-1, MSR-2] one element-position store into a run. The
            // effect row is the run's own storage, exactly as a buffer's is,
            // and the [SET-2] read-out adds the read the exchange performs.
            CheckedIndexedPlace::Container(container) => {
                for path in self.effect_paths_for_place(&container.resolved, bindings)? {
                    effects.add_write(path.clone());
                    if form.is_replace() {
                        effects.add_read(path);
                    }
                }
                (
                    container.resolved.root,
                    container.resolved.clone(),
                    CheckedSetTarget::RunIndex(Box::new(CheckedRunSetTarget {
                        root: container.root,
                        element_type,
                        place_offset: Self::place_offset_of(&offset.expression)
                            .unwrap_or(PlaceOffset::Opaque),
                        offset: offset.expression,
                        obligation,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
            // [SET-1, PROV-3] one element-position store through an exclusive
            // view. The storage written is the origin's, so the effect row
            // names every place the view's origin set resolves to [EFF-1]
            // 1386, while the target place stays the descriptor the statement
            // writes through.
            CheckedIndexedPlace::Slice(slice) => {
                // [S31] the parent may not write its elements while a shared
                // child reborrow of it lives.
                self.check_child_reborrow_freeze(bindings, &slice.slice.effect_places(), node)?;
                for origin in slice.slice.effect_places() {
                    for path in self.effect_paths_for_place(&origin, bindings)? {
                        effects.add_write(path.clone());
                        if form.is_replace() {
                            effects.add_read(path);
                        }
                    }
                }
                let resolved = ResolvedPlace {
                    root: slice.declaration,
                    fields: Vec::new(),
                };
                (
                    slice.declaration,
                    resolved,
                    CheckedSetTarget::SliceIndex(Box::new(CheckedSliceSetTarget {
                        root: slice.root,
                        offset: offset.expression,
                        obligation,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
        };
        // [MSR-2, LIV-2] a subscript target writes one element of `place`,
        // never the run's own storage, so disjointness and the measure kill
        // both read the element flag rather than the place alone.
        Ok(MutationTarget {
            declaration,
            place,
            element: true,
            target,
            effects,
            unsupported: None,
        })
    }

    /// Resolves a run of place suffixes into one [MSR-1] measured place's
    /// path: field selections and subscripts, in written order.
    ///
    /// `len_of(table[i])` is a term, so a measured place is not a field path.
    /// A subscript inside one is an [OP-4] occurrence like every other: it
    /// selects the base's [BLK-1] element and owes `i < len_of(base)`, which
    /// is submitted where the place is formed [MSR-4]. Its offset must be a
    /// term the place relations can name — [OWN-7] decides two subscripts by
    /// their offsets and [ENT-5] takes each offset's own support into every
    /// enclosing measure — so a written literal, a live `own u64` binding and
    /// an in-scope const generic [MSR-6] are admitted and every other offset
    /// is the explicit unsupported capability rather than a place whose
    /// identity no relation can decide.
    fn resolve_measured_path(
        &self,
        suffixes: &[NodeId],
        mut ty: CheckedType,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
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
            // [OP-4] a subscript inside a place indexes one of the two runs:
            // every other indexable base has a flat element [TYPE-2], which
            // no measure table row and no further subscript reaches.
            let element = match ty {
                CheckedType::FixedVector { element, .. } | CheckedType::Vector { element, .. } => {
                    element
                }
                _ => {
                    return self.issue_node(
                        SemanticRule::Op4,
                        suffix,
                        SemanticIssueKind::type_mismatch(
                            "a run base, whose element a subscript inside a place selects",
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
                    SemanticRule::Op4,
                    offset_node,
                    SemanticIssueKind::type_mismatch(
                        "own u64",
                        self.checked_value_name(offset.mode, offset.expression.ty())?,
                    ),
                );
            }
            let Some(place_offset) = Self::place_offset_of(&offset.expression) else {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, offset_node);
            };
            carried.effects = carried.effects.union(offset.effects);
            carried.accesses.extend(offset.accesses);
            path.push(CheckedPlaceStep::Subscript(Box::new(
                CheckedPlaceSubscript {
                    base_type: ty,
                    element_type: element.ty(),
                    offset: offset.expression,
                    obligation: self.tree.path(suffix)?.clone(),
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    place_offset,
                },
            )));
            ty = element.ty();
        }
        Ok((path, ty, carried))
    }

    /// One admitted offset as the place relations read it [OWN-7, ENT-5].
    ///
    /// The classification is over the checked operand and never over its
    /// spelling: a literal is its own value, a binding read is that binding,
    /// and a const generic is fixed at instantiation [FN-2].
    pub(in crate::semantic::check) fn place_offset_of(
        offset: &CheckedExpression,
    ) -> Option<PlaceOffset> {
        match offset {
            CheckedExpression::Constant(super::super::super::model::CheckedValue::Integer {
                bits,
                ..
            }) => Some(PlaceOffset::Literal(*bits)),
            CheckedExpression::Constant(
                super::super::super::model::CheckedValue::ConstGeneric { declaration, .. },
            ) => Some(PlaceOffset::Const(*declaration)),
            CheckedExpression::Binding {
                binding,
                consume_root: false,
                ..
            } => Some(PlaceOffset::Binding(*binding)),
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
            return self.check_dereferenced_buffer_place(node, pbase, base_suffixes, bindings);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let (root, binding, declaration, path, ty, slice, offsets) = match class {
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
                let (path, ty, offsets) = self.resolve_measured_path(
                    base_suffixes,
                    local.ty,
                    bindings,
                    function,
                    loop_depth,
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
                    local.slice,
                    offsets,
                )
            }
            DeclarationClass::NamedConst => {
                if !base_suffixes.is_empty() {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
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
                    None,
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
            CheckedType::Array { element, length } => {
                let Some(fields) = fields else {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, anchor);
                };
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
                    element_type: element.ty(),
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
                    holder: None,
                    resolved: ResolvedPlace {
                        root: declaration,
                        fields: resolved_fields,
                    },
                    borrow_kind: None,
                }))
            }
            CheckedType::Slice {
                region,
                element,
                strength,
            } => {
                let Some(fields) = fields else {
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, anchor);
                };
                if !fields.is_empty() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let (Some(binding), Some(declaration), Some(slice)) = (binding, declaration, slice)
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                if slice.region != region {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                Ok(CheckedIndexedPlace::Slice(CheckedSlicePlace {
                    root: CheckedSliceRoot {
                        binding,
                        element,
                        strength,
                    },
                    declaration,
                    descriptor: None,
                    slice,
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
            CheckedType::FixedVector { .. }
            | CheckedType::Vector { .. }
            | CheckedType::Extent { .. } => {
                let (Some(binding), Some(declaration)) = (binding, declaration) else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                // [OWN-5] the loan judgment reads the storage the place is
                // reached through; a subscript selects inside that storage,
                // so the prefix before the first one is what a loan protects.
                let resolved_fields = loan_prefix(&path);
                Ok(CheckedIndexedPlace::Container(CheckedContainerPlace {
                    root: CheckedContainerRoot { binding, path, ty },
                    resolved: ResolvedPlace {
                        root: declaration,
                        fields: resolved_fields,
                    },
                    offsets,
                    holder: None,
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
            CheckedPlaceStep::Subscript(_) => None,
        })
        .collect()
}

/// The field selections before the first subscript of a path [OWN-5].
fn loan_prefix(path: &[CheckedPlaceStep]) -> Vec<u32> {
    path.iter()
        .map_while(|step| match step {
            CheckedPlaceStep::Field(field) => Some(*field),
            CheckedPlaceStep::Subscript(_) => None,
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
