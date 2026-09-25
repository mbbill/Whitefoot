//! Finite physical call inventory after source acceptance.
//!
//! Semantic function identities, proof summaries, and permission tables stay
//! canonical. [STOR-8] gives the language one heap and no region parameters,
//! so each emitted source function has exactly one physical variant, and the
//! inventory is the set of functions a build emits with each call resolved to
//! its callee's variant.

use std::collections::HashSet;

use crate::NodePath;
use crate::semantic::{
    CheckedBodyDisposition, CheckedContainerRoot, CheckedElement, CheckedEnumType,
    CheckedExpression, CheckedFunction, CheckedNominalKind, CheckedPlaceStep, CheckedProgramData,
    CheckedSetTarget, CheckedStatement, CheckedType, FunctionId, NominalId, expression_children,
};

use super::LoweringFailure;

#[derive(Debug)]
pub(super) struct PhysicalVariant {
    pub(super) source: FunctionId,
    pub(super) calls: Vec<(NodePath, u32)>,
}

#[derive(Debug)]
pub(super) struct PhysicalFunctions {
    pub(super) variants: Vec<PhysicalVariant>,
}

#[derive(Clone)]
struct CallEdge {
    source: FunctionId,
    path: NodePath,
}

#[derive(Default)]
struct FunctionDependencies {
    types: Vec<CheckedType>,
    elements: Vec<CheckedElement>,
    calls: Vec<CallEdge>,
}

impl PhysicalFunctions {
    /// Every checked definition, each closing over its calls.
    #[cfg(test)]
    pub(super) fn build(program: &CheckedProgramData) -> Result<Self, LoweringFailure> {
        Self::build_from(program, None)
    }

    /// With `roots`, only what those functions reach through their calls:
    /// a module program entry's build [MOD-9] emits the code its run can
    /// execute and no definition outside it. Without, every checked
    /// definition.
    pub(super) fn build_from(
        program: &CheckedProgramData,
        roots: Option<&[FunctionId]>,
    ) -> Result<Self, LoweringFailure> {
        let dependencies = program
            .functions
            .iter()
            .map(FunctionDependencies::collect)
            .collect::<Vec<_>>();
        debug_assert!(
            program
                .functions
                .iter()
                .all(|function| function.region_parameters.is_empty()),
            "[STOR-8] no checked function takes a region parameter"
        );
        for dependency in &dependencies {
            for call in &dependency.calls {
                source_function(program, call.source)?;
            }
        }
        // With roots, the functions they reach through calls; without, every
        // checked definition.
        let mut emitted = vec![roots.is_none(); program.functions.len()];
        let mut pending = Vec::new();
        for root in roots.unwrap_or_default() {
            let slot = emitted
                .get_mut(root.0 as usize)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            if !*slot {
                *slot = true;
                pending.push(*root);
            }
        }
        while let Some(function) = pending.pop() {
            for call in &dependencies[function.0 as usize].calls {
                let slot = &mut emitted[call.source.0 as usize];
                if !*slot {
                    *slot = true;
                    pending.push(call.source);
                }
            }
        }
        // A variant's ordinal is its source's rank among the emitted
        // functions, so ordinals follow source order.
        let mut ordinals = vec![None; program.functions.len()];
        let mut next = 0u32;
        for (index, emit) in emitted.iter().enumerate() {
            if *emit {
                ordinals[index] = Some(next);
                next = next
                    .checked_add(1)
                    .ok_or(LoweringFailure::CounterOverflow)?;
            }
        }
        let variants = program
            .functions
            .iter()
            .zip(&dependencies)
            .enumerate()
            .filter(|(index, _)| emitted[*index])
            .map(|(index, (function, dependency))| {
                if function.id.0 as usize != index {
                    return Err(LoweringFailure::InvalidCheckedProgram);
                }
                let calls = dependency
                    .calls
                    .iter()
                    .map(|call| {
                        ordinals[call.source.0 as usize]
                            .map(|ordinal| (call.path.clone(), ordinal))
                            .ok_or(LoweringFailure::InvalidCheckedProgram)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(PhysicalVariant {
                    source: function.id,
                    calls,
                })
            })
            .collect::<Result<Vec<_>, LoweringFailure>>()?;
        Ok(Self { variants })
    }
}

fn source_function(
    program: &CheckedProgramData,
    source: FunctionId,
) -> Result<&CheckedFunction, LoweringFailure> {
    program
        .functions
        .get(source.0 as usize)
        .filter(|function| function.id == source)
        .ok_or(LoweringFailure::InvalidCheckedProgram)
}

/// [STOR-8] whether a function's own concrete layout holds heap storage: the
/// type of one of its parameters, its result, or a value its body evaluates,
/// binds or releases holds a `Box` or a runtime-capacity shape [TYPE-9], as
/// itself or as a field, a payload field or an element. Such a value's
/// release frees heap storage, so its function needs the heap whether or not
/// it allocates.
pub(crate) fn holds_heap_storage(program: &CheckedProgramData, function: &CheckedFunction) -> bool {
    let dependencies = FunctionDependencies::collect(function);
    let mut visited = HashSet::new();
    dependencies
        .types
        .iter()
        .copied()
        .chain(
            dependencies
                .elements
                .iter()
                .filter_map(|element| program.elements.get(element.index()).copied()),
        )
        .any(|ty| type_holds_heap(program, ty, &mut visited))
}

fn type_holds_heap(
    program: &CheckedProgramData,
    ty: CheckedType,
    visited: &mut HashSet<NominalId>,
) -> bool {
    match ty {
        CheckedType::Buffer { .. } | CheckedType::Window { capacity: None, .. } => true,
        CheckedType::Array { element, .. } | CheckedType::Window { element, .. } => program
            .elements
            .get(element.index())
            .is_some_and(|element| type_holds_heap(program, *element, visited)),
        CheckedType::Nominal(id) => {
            if !visited.insert(id) {
                return false;
            }
            match program
                .nominals
                .get(id.0 as usize)
                .map(|nominal| &nominal.kind)
            {
                Some(CheckedNominalKind::Box { .. }) => true,
                Some(CheckedNominalKind::Struct { fields }) => fields
                    .iter()
                    .any(|field| type_holds_heap(program, field.ty, visited)),
                Some(CheckedNominalKind::Enum { variants }) => variants
                    .iter()
                    .flat_map(|variant| &variant.fields)
                    .any(|field| type_holds_heap(program, field.ty, visited)),
                Some(CheckedNominalKind::Opaque) | None => false,
            }
        }
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Generic(_)
        | CheckedType::GenericInt(_)
        | CheckedType::GenericFloat(_) => false,
    }
}

pub(super) fn executable_storage(
    function: &CheckedFunction,
) -> (Vec<CheckedType>, Vec<CheckedElement>) {
    let dependencies = FunctionDependencies::collect(function);
    (dependencies.types, dependencies.elements)
}

impl FunctionDependencies {
    fn collect(function: &CheckedFunction) -> Self {
        let mut dependencies = Self::default();
        dependencies.types.push(function.result);
        dependencies
            .types
            .extend(function.parameters.iter().map(|parameter| parameter.ty));
        if matches!(function.body_disposition, CheckedBodyDisposition::Inhabited) {
            dependencies.statements(function.body.as_deref().unwrap_or_default());
        }
        dependencies
    }

    fn statements(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::Evaluate { value, .. }
                | CheckedStatement::DropExpression { value, .. } => self.expression(value),
                CheckedStatement::DestructuringLet {
                    bindings,
                    nominal,
                    value,
                    ..
                } => {
                    self.types.push(CheckedType::Nominal(*nominal));
                    self.types.extend(bindings.iter().map(|(_, ty, _)| *ty));
                    self.expression(value);
                }
                CheckedStatement::PropagateLet {
                    scrutinee,
                    result_nominal,
                    return_nominal,
                    ok_type,
                    error_type,
                    error_drops,
                    ..
                } => {
                    self.types.extend([
                        CheckedType::Nominal(*result_nominal),
                        CheckedType::Nominal(*return_nominal),
                        *ok_type,
                        *error_type,
                    ]);
                    self.types.extend(error_drops.iter().map(|drop| drop.ty));
                    self.expression(scrutinee);
                }
                CheckedStatement::Set { target, value, .. } => {
                    self.target(target);
                    self.expression(value);
                }
                CheckedStatement::Proof(_) => {}
                CheckedStatement::Return { value, drops, .. }
                | CheckedStatement::Give { value, drops, .. } => {
                    self.expression(value);
                    self.types.extend(drops.iter().map(|drop| drop.ty));
                }
                CheckedStatement::Match {
                    scrutinee,
                    enum_type,
                    arms,
                    ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee,
                    enum_type,
                    arms,
                    ..
                } => {
                    if let CheckedStatement::ValueMatchLet {
                        result_type,
                        result_range_element,
                        ..
                    } = statement
                    {
                        self.types.push(*result_type);
                        self.elements.extend(result_range_element.iter().copied());
                    }
                    if let CheckedEnumType::Nominal(nominal) = enum_type {
                        self.types.push(CheckedType::Nominal(*nominal));
                    }
                    self.expression(scrutinee);
                    for arm in arms {
                        self.types
                            .extend(arm.binders.iter().map(|binder| binder.ty));
                        self.types.extend(arm.covered.iter().map(|drop| drop.ty));
                        self.types
                            .extend(arm.fallthrough_drops.iter().map(|drop| drop.ty));
                        self.statements(&arm.body);
                    }
                }
                CheckedStatement::Loop {
                    body,
                    backedge_drops,
                    ..
                }
                | CheckedStatement::CountedRange {
                    body,
                    backedge_drops,
                    ..
                } => {
                    if let CheckedStatement::CountedRange { lower, upper, .. } = statement {
                        self.expression(lower);
                        self.expression(upper);
                    }
                    self.types.extend(backedge_drops.iter().map(|drop| drop.ty));
                    self.statements(body);
                }
                CheckedStatement::Break { drops, .. } => {
                    self.types.extend(drops.iter().map(|drop| drop.ty));
                }
            }
        }
    }

    fn expression(&mut self, expression: &CheckedExpression) {
        self.types.push(expression.ty());
        match expression {
            CheckedExpression::UserCall {
                function,
                call,
                goal_regions,
                ..
            } => {
                debug_assert!(
                    goal_regions.is_empty(),
                    "[STOR-8] no call carries a region argument"
                );
                self.calls.push(CallEdge {
                    source: *function,
                    path: call.clone(),
                });
            }
            CheckedExpression::BoxDeref { nominal, .. }
            | CheckedExpression::ProjectValue { nominal, .. } => {
                self.types.push(CheckedType::Nominal(*nominal));
            }
            CheckedExpression::BoxTake { path, cleanup, .. } => {
                self.steps(path);
                for action in cleanup {
                    match action {
                        crate::semantic::CheckedOwnedTakeCleanup::Drop { path, ty } => {
                            self.types.push(*ty);
                            self.steps(path);
                        }
                        crate::semantic::CheckedOwnedTakeCleanup::BoxShell {
                            path,
                            nominal,
                            referent,
                        } => {
                            self.types
                                .extend([CheckedType::Nominal(*nominal), *referent]);
                            self.steps(path);
                        }
                    }
                }
            }
            CheckedExpression::Project { residual_drops, .. } => {
                self.types.extend(residual_drops.iter().map(|drop| drop.ty));
            }
            CheckedExpression::ContainerMeasure { root, .. }
            | CheckedExpression::ReadStorage { root, .. }
            | CheckedExpression::BorrowAddressed { root, .. } => self.root_types(root),
            CheckedExpression::BorrowRangeIndex { place, .. }
            | CheckedExpression::RangeIndex { place, .. } => {
                self.types.push(place.root.element_type);
                self.steps(&place.path);
            }
            CheckedExpression::RangeElementMeasure { place, .. } => {
                self.types.push(place.root.element_type);
                self.types.push(place.ty);
                self.steps(&place.path);
            }
            _ => {}
        }
        for child in expression_children(expression) {
            self.expression(child);
        }
    }

    fn target(&mut self, target: &CheckedSetTarget) {
        self.types.push(target.ty());
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::RangeIndex(target) => {
                self.types.push(target.root.element_type);
                for offset in target.offsets() {
                    self.expression(offset);
                }
                self.steps(&target.path);
            }
            CheckedSetTarget::Storage(root) => {
                self.root_types(root);
                for offset in root.offsets() {
                    self.expression(offset);
                }
            }
        }
    }

    fn root_types(&mut self, root: &CheckedContainerRoot) {
        self.types.push(root.ty);
        self.steps(&root.path);
    }

    fn steps(&mut self, steps: &[CheckedPlaceStep]) {
        for step in steps {
            match step {
                CheckedPlaceStep::Field(_) => {}
                CheckedPlaceStep::BoxReferent(nominal) => {
                    self.types.push(CheckedType::Nominal(*nominal));
                }
                CheckedPlaceStep::Subscript(subscript) => {
                    self.types
                        .extend([subscript.base_type, subscript.element_type]);
                    self.expression(&subscript.offset);
                }
            }
        }
    }
}
