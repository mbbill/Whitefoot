mod borrowed;
mod slices;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedArrayRoot, CheckedArraySetTarget, CheckedBufferRoot, CheckedBufferSetTarget,
    CheckedConst, CheckedExpression, CheckedFlatElement, CheckedMode,
    CheckedRuntimeTargetObligations, CheckedSetTarget, CheckedSliceRoot,
    CheckedTargetDomainObligation, CheckedType, IntegerType, TrapSite,
};
use super::super::borrows::{
    AccessKind, BorrowInfo, BorrowKind, RequiredReferent, ResolvedPlace, SliceInfo,
};
use super::super::{
    CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding, PlaceAccess, TypedExpression,
};
use super::PlaceUseOptions;

#[derive(Clone)]
pub(super) struct CheckedArrayPlace {
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
struct CheckedBufferPlace {
    root: CheckedBufferRoot,
    declaration: DeclarationId,
    element_type: CheckedType,
    holder: Option<DeclarationId>,
    resolved: ResolvedPlace,
    origin_region: Option<DeclarationId>,
    borrow_kind: Option<BorrowKind>,
}

#[derive(Clone)]
struct CheckedSlicePlace {
    root: CheckedSliceRoot,
    declaration: DeclarationId,
    descriptor: Option<BorrowInfo>,
    slice: SliceInfo,
}

#[derive(Clone)]
enum CheckedIndexedPlace {
    Array(CheckedArrayPlace),
    Buffer(CheckedBufferPlace),
    Slice(CheckedSlicePlace),
}

impl CheckedIndexedPlace {
    const fn element_type(&self) -> CheckedType {
        match self {
            Self::Array(array) => array.element_type,
            Self::Buffer(buffer) => buffer.element_type,
            Self::Slice(slice) => slice.root.element.ty(),
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
                SemanticIssueKind::TypeMismatch,
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
                SemanticIssueKind::TypeMismatch,
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
                SemanticIssueKind::TypeMismatch,
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
        Ok(TypedExpression::owned(
            CheckedExpression::BufferFill {
                carrier: self.tree.path(node)?.clone(),
                element,
                length: Box::new(length.expression),
                value: Box::new(value.expression),
                trap: TrapSite {
                    rule_id: "OP-9",
                    message: String::new(),
                    function: function.name.clone(),
                    node_path: self.tree.path(node)?.clone(),
                },
                target_domains: CheckedRuntimeTargetObligations::new(),
            },
            length
                .effects
                .union(value.effects)
                .union(EffectSet::ALLOCATES_HEAP_AND_TRAPS),
        ))
    }

    pub(in crate::semantic::check) fn check_flat_length(
        &self,
        node: NodeId,
        _function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        _loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.reject_named_operation_arguments(node, "len")?;
        self.reject_written_operation_type_argument(node)?;
        let atoms = self.operation_atoms(node, 1)?;
        // [OP-2] `len`'s selected element type is the base place's own; the
        // result is `own u64` for every row, so nothing else consults it.
        let place = self.check_indexed_atom_place(atoms[0], bindings)?;
        let mut effects = EffectSet::NONE;
        match &place {
            CheckedIndexedPlace::Array(_) => {}
            CheckedIndexedPlace::Buffer(buffer) => {
                self.check_loan_access(
                    bindings,
                    buffer.holder,
                    &buffer.resolved,
                    AccessKind::Read,
                    atoms[0],
                )?;
                if let Some(region) = buffer.origin_region {
                    effects.add_read(region);
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
                    if let Some(region) = descriptor.origin_region {
                        effects.add_read(region);
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
                for region in slice.slice.effect_regions() {
                    effects.add_read(region);
                }
            }
        }
        Ok(TypedExpression::owned(
            match place {
                CheckedIndexedPlace::Array(array) => CheckedExpression::ArrayLength {
                    root: array.root,
                    length: array.length,
                },
                CheckedIndexedPlace::Buffer(buffer) => {
                    CheckedExpression::BufferLength { root: buffer.root }
                }
                CheckedIndexedPlace::Slice(slice) => {
                    CheckedExpression::SliceLength { root: slice.root }
                }
            },
            effects,
        ))
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
            return self.issue_node(SemanticRule::Type5, place, SemanticIssueKind::TypeMismatch);
        }
        if options.explicit_move {
            return self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::MoveOfCopy {
                    mechanical_fix: "use the indexed copy place without `move`",
                },
            );
        }
        let indexed = self.check_indexed_place(place, bindings, &suffixes[..subscript], suffix)?;
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
                SemanticIssueKind::TypeMismatch,
            );
        }
        // A subscript is not an [EFF-2] trap source: an accepted subscript
        // is discharged [OP-4] and executes no runtime check. The retained
        // TrapSite carries the psuffix node identity the [ENT-6] obligation
        // judgment and the [OP-4] rejection cite; it never reaches runtime.
        let trap = TrapSite {
            rule_id: "OP-4",
            message: String::new(),
            function: function.name.clone(),
            node_path: self.tree.path(suffix)?.clone(),
        };
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
                trap,
                target_domain: CheckedTargetDomainObligation::ElementAddress,
            },
            CheckedIndexedPlace::Buffer(buffer) => {
                if let Some(region) = buffer.origin_region {
                    effects.add_read(region);
                }
                CheckedExpression::BufferIndex {
                    carrier: self.tree.path(use_node)?.clone(),
                    root: buffer.root,
                    offset: Box::new(offset.expression),
                    trap,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                }
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(descriptor) = &slice.descriptor
                    && let Some(region) = descriptor.origin_region
                {
                    effects.add_read(region);
                }
                for region in slice.slice.effect_regions() {
                    effects.add_read(region);
                }
                CheckedExpression::SliceIndex {
                    carrier: self.tree.path(use_node)?.clone(),
                    root: slice.root,
                    offset: Box::new(offset.expression),
                    trap,
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

    pub(in crate::semantic::check) fn check_indexed_set_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        suffixes: &[NodeId],
        subscript: usize,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<(DeclarationId, CheckedSetTarget, EffectSet), CheckStop> {
        let suffix = suffixes[subscript];
        if subscript + 1 != suffixes.len() {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        }
        let indexed = self.check_indexed_place(node, bindings, &suffixes[..subscript], suffix)?;
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
            CheckedIndexedPlace::Slice(_) => {
                return self.issue_node(
                    SemanticRule::Set1,
                    node,
                    SemanticIssueKind::InvalidSetTarget {
                        root_class: "slice view".to_owned(),
                        required_classes: "live own storage or a live usable &uniq referent",
                    },
                );
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
                SemanticIssueKind::TypeMismatch,
            );
        }
        // As in the read path: no [EFF-2] trap contribution; the TrapSite
        // carries only the psuffix node identity for the [ENT-6] judgment.
        let trap = TrapSite {
            rule_id: "OP-4",
            message: String::new(),
            function: function.name.clone(),
            node_path: self.tree.path(suffix)?.clone(),
        };
        let mut effects = offset.effects;
        let (declaration, target) = match indexed {
            CheckedIndexedPlace::Array(array) => {
                let Some(declaration) = array.declaration else {
                    return self.issue_node(
                        SemanticRule::Const2,
                        node,
                        SemanticIssueKind::ImmutableSetTarget,
                    );
                };
                let CheckedArrayRoot::Binding { binding, fields } = array.root else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                (
                    declaration,
                    CheckedSetTarget::ArrayIndex(Box::new(CheckedArraySetTarget {
                        binding,
                        fields,
                        array_type: array.array_type,
                        element_type: array.element_type,
                        length: array.length,
                        offset: offset.expression,
                        trap,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                if let Some(region) = buffer.origin_region {
                    effects.add_write(region);
                }
                (
                    buffer.declaration,
                    CheckedSetTarget::BufferIndex(Box::new(CheckedBufferSetTarget {
                        root: buffer.root,
                        offset: offset.expression,
                        trap,
                        target_domain: CheckedTargetDomainObligation::ElementAddress,
                    })),
                )
            }
            CheckedIndexedPlace::Slice(_) => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        };
        Ok((declaration, target, effects))
    }

    fn check_indexed_atom_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        if self.has_fixed(node, FixedTerminal::Move)? {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        }
        let place = self
            .tree
            .first_child_with(node, Production::Place)?
            .ok_or_else(|| {
                self.issue_value(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch)
            })?;
        let suffixes = self.tree.children_with(place, Production::Psuffix)?;
        self.check_indexed_place(place, bindings, &suffixes, place)
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
        let (root, binding, declaration, fields, ty, slice) = match class {
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
                let (fields, ty) = self.resolve_struct_path(base_suffixes, local.ty)?;
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
                    fields,
                    ty,
                    local.slice,
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
                )
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        match ty {
            CheckedType::Array { element, length } => {
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
                    origin_region: None,
                    borrow_kind: None,
                }))
            }
            CheckedType::Slice { region, element } => {
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
                    root: CheckedSliceRoot { binding, element },
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
            _ => self.issue_node(SemanticRule::Type5, anchor, SemanticIssueKind::TypeMismatch),
        }
    }
}
