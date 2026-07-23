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
use super::super::borrows::{AccessKind, BorrowKind, ResolvedPlace};
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
    resolved: ResolvedPlace,
    origin_region: Option<DeclarationId>,
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
        let targs = self
            .tree
            .first_child_with(node, Production::Targs)?
            .ok_or_else(|| {
                self.issue_value(SemanticRule::Fn2, node, SemanticIssueKind::InvalidOperation)
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
        let element_type = self.operation_type_argument(node, "buffer_new", function)?;
        let element = match element_type {
            CheckedType::Unit => CheckedFlatElement::Unit,
            CheckedType::Integer(ty) => CheckedFlatElement::Integer(ty),
            CheckedType::Float(ty) => CheckedFlatElement::Float(ty),
            CheckedType::GenericInt(declaration) => CheckedFlatElement::GenericInt(declaration),
            _ => {
                return self.issue_node(
                    SemanticRule::Op1,
                    node,
                    SemanticIssueKind::InvalidOperation,
                );
            }
        };
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
        let value = self.check_atom(function, atoms[1], bindings, loop_depth)?;
        if value.expression.ty() != element_type || value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[1],
                SemanticIssueKind::TypeMismatch,
            );
        }
        Ok(TypedExpression::owned(
            CheckedExpression::BufferFill {
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
        function: &FunctionSignature,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        _loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let element_type = self.operation_type_argument(node, "len", function)?;
        let atoms = self.operation_atoms(node, 1)?;
        let place = self.check_indexed_atom_place(atoms[0], bindings)?;
        if place.element_type() != element_type {
            return self.issue_node(
                SemanticRule::Type5,
                atoms[0],
                SemanticIssueKind::TypeMismatch,
            );
        }
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
                self.check_loan_access(
                    bindings,
                    Some(slice.declaration),
                    &slice.resolved,
                    AccessKind::Read,
                    atoms[0],
                )?;
                if let Some(region) = slice.origin_region {
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

    pub(super) fn check_index_use(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        place: NodeId,
        pbase: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        if !self
            .tree
            .children_with(place, Production::Psuffix)?
            .is_empty()
        {
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
        let selected = self
            .tree
            .first_child_with(pbase, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let selected = self.parse_type_with(selected, &function.substitution)?;
        let base = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let indexed = self.check_indexed_place(base, bindings)?;
        if selected != indexed.element_type() {
            return self.issue_node(SemanticRule::Type5, pbase, SemanticIssueKind::TypeMismatch);
        }
        match &indexed {
            CheckedIndexedPlace::Array(array) => {
                if let Some(resolved) = array.resolved_place() {
                    self.check_loan_access(bindings, None, &resolved, AccessKind::Read, pbase)?;
                }
            }
            CheckedIndexedPlace::Buffer(buffer) => {
                self.check_loan_access(
                    bindings,
                    buffer.holder,
                    &buffer.resolved,
                    AccessKind::Read,
                    pbase,
                )?;
            }
            CheckedIndexedPlace::Slice(slice) => {
                self.check_loan_access(
                    bindings,
                    Some(slice.declaration),
                    &slice.resolved,
                    AccessKind::Read,
                    pbase,
                )?;
            }
        }
        let offset = self
            .tree
            .first_child_with(pbase, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let offset = self.check_atom(function, offset, bindings, options.loop_depth)?;
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || offset.mode != CheckedMode::Own
        {
            return self.issue_node(SemanticRule::Type5, pbase, SemanticIssueKind::TypeMismatch);
        }
        let trap = TrapSite {
            rule_id: "OP-4",
            message: String::new(),
            function: function.name.clone(),
            node_path: self.tree.path(pbase)?.clone(),
        };
        let mut effects = offset.effects.union(EffectSet::TRAPS);
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
            CheckedIndexedPlace::Slice(slice) => accesses.push(PlaceAccess {
                place: slice.resolved.clone(),
                kind: AccessKind::Read,
            }),
        }
        let expression = match indexed {
            CheckedIndexedPlace::Array(array) => CheckedExpression::ArrayIndex {
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
                    root: buffer.root,
                    offset: Box::new(offset.expression),
                    trap,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                }
            }
            CheckedIndexedPlace::Slice(slice) => {
                if let Some(region) = slice.origin_region {
                    effects.add_read(region);
                }
                CheckedExpression::SliceIndex {
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
            effects,
            accesses,
        })
    }

    pub(in crate::semantic::check) fn check_indexed_set_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        pbase: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<(DeclarationId, CheckedSetTarget, EffectSet), CheckStop> {
        if !self
            .tree
            .children_with(node, Production::Psuffix)?
            .is_empty()
        {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        }
        let selected_node = self
            .tree
            .first_child_with(pbase, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let selected = self.parse_type_with(selected_node, &function.substitution)?;
        let base = self
            .tree
            .first_child_with(pbase, Production::Place)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let indexed = self.check_indexed_place(base, bindings)?;
        if selected != indexed.element_type() {
            return self.issue_node(SemanticRule::Type5, pbase, SemanticIssueKind::TypeMismatch);
        }
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
            .tree
            .first_child_with(pbase, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let offset = self.check_atom(function, offset_node, bindings, loop_depth)?;
        if offset.expression.ty() != CheckedType::Integer(IntegerType::U64)
            || offset.mode != CheckedMode::Own
        {
            return self.issue_node(SemanticRule::Type5, pbase, SemanticIssueKind::TypeMismatch);
        }
        let trap = TrapSite {
            rule_id: "OP-4",
            message: String::new(),
            function: function.name.clone(),
            node_path: self.tree.path(pbase)?.clone(),
        };
        let mut effects = offset.effects.union(EffectSet::TRAPS);
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
        self.check_indexed_place(place, bindings)
    }

    fn check_indexed_place(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedIndexedPlace, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_buffer_place(node, pbase, bindings);
        }
        if self.has_fixed(pbase, FixedTerminal::Index)? {
            return self.unsupported(UnsupportedSemanticFeature::CompositeValues, pbase);
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
                let (fields, ty) = self.resolve_struct_path(node, local.ty)?;
                if local.mode != CheckedMode::Own {
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
                if !self
                    .tree
                    .children_with(node, Production::Psuffix)?
                    .is_empty()
                {
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
                    resolved: slice.place,
                    origin_region: slice.origin_region,
                }))
            }
            _ => self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch),
        }
    }
}
