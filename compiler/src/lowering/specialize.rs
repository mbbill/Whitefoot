//! Finite physical call inventory after source acceptance.
//!
//! Semantic function identities, proof summaries, and permission tables stay
//! canonical. Only code emission distinguishes the release classes of stores
//! reachable from each function's executable types and transitive calls.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::semantic::{
    CheckedBodyDisposition, CheckedContainerRoot, CheckedEnumType, CheckedExpression,
    CheckedFunction, CheckedNominalKind, CheckedPlaceStep, CheckedProgramData, CheckedReleaseClass,
    CheckedSetTarget, CheckedStatement, CheckedType, FunctionId, NominalId, expression_children,
};
use crate::{DeclarationId, NodePath};

use super::LoweringFailure;

#[derive(Debug)]
pub(super) struct PhysicalVariant {
    pub(super) source: FunctionId,
    /// Closed class assignment in declaration order, including captured store
    /// brands and local stores. Source region identity never enters a key.
    pub(super) releases: Vec<(DeclarationId, CheckedReleaseClass)>,
    pub(super) calls: Vec<(NodePath, u32)>,
}

#[derive(Debug)]
pub(super) struct PhysicalFunctions {
    pub(super) main: u32,
    pub(super) variants: Vec<PhysicalVariant>,
}

#[derive(Clone)]
struct CallEdge {
    source: FunctionId,
    path: NodePath,
    regions: Vec<DeclarationId>,
}

#[derive(Default)]
struct FunctionDependencies {
    types: Vec<CheckedType>,
    calls: Vec<CallEdge>,
}

impl PhysicalFunctions {
    pub(super) fn build(program: &CheckedProgramData) -> Result<Self, LoweringFailure> {
        let dependencies = program
            .functions
            .iter()
            .map(FunctionDependencies::collect)
            .collect::<Vec<_>>();
        let mut defaults = program.region_release_defaults.clone();
        let mut regions = Vec::with_capacity(dependencies.len());
        for dependency in &dependencies {
            let mut selected = BTreeSet::new();
            let mut visited = HashSet::new();
            for ty in &dependency.types {
                collect_regions(program, *ty, &mut selected, &mut visited, &mut defaults)?;
            }
            regions.push(selected);
        }
        defaults.sort_unstable_by_key(|(region, _)| *region);
        defaults.dedup_by_key(|(region, _)| *region);

        // A caller needs every store on which a callee's physical body depends,
        // even if that store appears only inside an instantiated type argument.
        // Every iteration adds members of the finite checked declaration set.
        loop {
            let mut changed = false;
            for (index, dependency) in dependencies.iter().enumerate() {
                for call in &dependency.calls {
                    let callee = source_function(program, call.source)?;
                    validate_call_regions(callee, call)?;
                    let required = regions
                        .get(call.source.0 as usize)
                        .ok_or(LoweringFailure::InvalidCheckedProgram)?
                        .iter()
                        .copied()
                        .map(|region| actual_region(callee, call, region))
                        .collect::<Vec<_>>();
                    for region in required {
                        changed |= regions[index].insert(region);
                    }
                }
            }
            if !changed {
                break;
            }
        }

        let regions = regions
            .into_iter()
            .map(|regions| regions.into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut plan = Self {
            main: 0,
            variants: Vec::new(),
        };
        let mut interned = HashMap::new();
        let mut represented = vec![false; program.functions.len()];
        let main_regions = regions
            .get(program.main.0 as usize)
            .ok_or(LoweringFailure::InvalidCheckedProgram)?;
        plan.main = plan.intern(
            program.main,
            default_environment(main_regions, &defaults),
            &mut interned,
            &mut represented,
        )?;
        let mut next = 0;
        loop {
            while next < plan.variants.len() {
                let source = plan.variants[next].source;
                let caller_releases = plan.variants[next].releases.clone();
                let mut calls = Vec::new();
                for call in &dependencies[source.0 as usize].calls {
                    let callee = source_function(program, call.source)?;
                    let releases = regions[call.source.0 as usize]
                        .iter()
                        .map(|region| {
                            let actual = actual_region(callee, call, *region);
                            (*region, release_at(&caller_releases, actual, &defaults))
                        })
                        .collect();
                    let target =
                        plan.intern(call.source, releases, &mut interned, &mut represented)?;
                    calls.push((call.path.clone(), target));
                }
                plan.variants[next].calls = calls;
                next += 1;
            }
            // Retain emission of otherwise unreferenced canonical definitions,
            // closing each default's calls before selecting the next definition.
            let Some(index) = represented.iter().position(|present| !present) else {
                return plan.order_by_source();
            };
            let source = program.functions[index].id;
            plan.intern(
                source,
                default_environment(&regions[index], &defaults),
                &mut interned,
                &mut represented,
            )?;
        }
    }

    fn order_by_source(mut self) -> Result<Self, LoweringFailure> {
        let mut order = (0..self.variants.len()).collect::<Vec<_>>();
        order.sort_by(|left, right| {
            let left = &self.variants[*left];
            let right = &self.variants[*right];
            left.source.0.cmp(&right.source.0).then_with(|| {
                left.releases
                    .iter()
                    .map(|(_, class)| matches!(class, CheckedReleaseClass::Extent))
                    .cmp(
                        right
                            .releases
                            .iter()
                            .map(|(_, class)| matches!(class, CheckedReleaseClass::Extent)),
                    )
            })
        });
        let mut remapping = vec![0; order.len()];
        for (new, old) in order.iter().enumerate() {
            remapping[*old] = u32::try_from(new).map_err(|_| LoweringFailure::CounterOverflow)?;
        }
        self.main = remapping[self.main as usize];
        for variant in &mut self.variants {
            for (_, callee) in &mut variant.calls {
                *callee = remapping[*callee as usize];
            }
        }
        let mut variants = self.variants.into_iter().map(Some).collect::<Vec<_>>();
        self.variants = order
            .into_iter()
            .map(|index| {
                variants[index]
                    .take()
                    .ok_or(LoweringFailure::InvalidCheckedProgram)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self)
    }

    fn intern(
        &mut self,
        source: FunctionId,
        releases: Vec<(DeclarationId, CheckedReleaseClass)>,
        interned: &mut HashMap<(FunctionId, Vec<CheckedReleaseClass>), u32>,
        represented: &mut [bool],
    ) -> Result<u32, LoweringFailure> {
        let classes = releases.iter().map(|(_, class)| *class).collect();
        let key = (source, classes);
        if let Some(id) = interned.get(&key) {
            return Ok(*id);
        }
        let id =
            u32::try_from(self.variants.len()).map_err(|_| LoweringFailure::CounterOverflow)?;
        *represented
            .get_mut(source.0 as usize)
            .ok_or(LoweringFailure::InvalidCheckedProgram)? = true;
        self.variants.push(PhysicalVariant {
            source,
            releases,
            calls: Vec::new(),
        });
        interned.insert(key, id);
        Ok(id)
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

fn validate_call_regions(callee: &CheckedFunction, call: &CallEdge) -> Result<(), LoweringFailure> {
    if callee.region_parameters.len() == call.regions.len() {
        Ok(())
    } else {
        Err(LoweringFailure::InvalidCheckedProgram)
    }
}

fn actual_region(
    callee: &CheckedFunction,
    call: &CallEdge,
    region: DeclarationId,
) -> DeclarationId {
    callee
        .region_parameters
        .iter()
        .position(|formal| *formal == region)
        .map_or(region, |index| call.regions[index])
}

fn release_at(
    environment: &[(DeclarationId, CheckedReleaseClass)],
    region: DeclarationId,
    defaults: &[(DeclarationId, CheckedReleaseClass)],
) -> CheckedReleaseClass {
    environment
        .binary_search_by_key(&region, |(region, _)| *region)
        .ok()
        .map(|index| environment[index].1)
        .or_else(|| {
            defaults
                .binary_search_by_key(&region, |(region, _)| *region)
                .ok()
                .map(|index| defaults[index].1)
        })
        .unwrap_or(CheckedReleaseClass::General)
}

fn default_environment(
    regions: &[DeclarationId],
    defaults: &[(DeclarationId, CheckedReleaseClass)],
) -> Vec<(DeclarationId, CheckedReleaseClass)> {
    regions
        .iter()
        .map(|region| (*region, release_at(&[], *region, defaults)))
        .collect()
}

fn collect_regions(
    program: &CheckedProgramData,
    ty: CheckedType,
    regions: &mut BTreeSet<DeclarationId>,
    visited: &mut HashSet<NominalId>,
    defaults: &mut Vec<(DeclarationId, CheckedReleaseClass)>,
) -> Result<(), LoweringFailure> {
    match ty {
        CheckedType::Nominal(id) => {
            if !visited.insert(id) {
                return Ok(());
            }
            let nominal = program
                .nominals
                .get(id.0 as usize)
                .ok_or(LoweringFailure::InvalidCheckedProgram)?;
            match &nominal.kind {
                CheckedNominalKind::Struct { fields } => {
                    for field in fields {
                        collect_regions(program, field.ty, regions, visited, defaults)?;
                    }
                }
                CheckedNominalKind::Enum { variants } => {
                    for variant in variants {
                        for field in &variant.fields {
                            collect_regions(program, field.ty, regions, visited, defaults)?;
                        }
                    }
                }
                CheckedNominalKind::Box {
                    referent,
                    region,
                    release,
                } => {
                    if let Some(region) = region {
                        regions.insert(*region);
                        insert_default(defaults, *region, *release);
                    }
                    collect_regions(program, *referent, regions, visited, defaults)?;
                }
                CheckedNominalKind::Arena { content, .. } => {
                    collect_regions(program, *content, regions, visited, defaults)?;
                }
                CheckedNominalKind::ArenaStorage | CheckedNominalKind::SystemResource { .. } => {}
            }
        }
        CheckedType::Vector {
            region,
            element,
            release,
        } => {
            regions.insert(region);
            insert_default(defaults, region, release);
            collect_regions(program, element.ty(), regions, visited, defaults)?;
        }
        CheckedType::Heap { region } => {
            regions.insert(region);
            insert_default(defaults, region, CheckedReleaseClass::General);
        }
        CheckedType::Extent { region, .. } => {
            regions.insert(region);
            insert_default(defaults, region, CheckedReleaseClass::Extent);
        }
        CheckedType::FixedVector { element, .. } => {
            collect_regions(program, element.ty(), regions, visited, defaults)?;
        }
        CheckedType::Array { element, .. }
        | CheckedType::Buffer { element }
        | CheckedType::Slice { element, .. } => {
            collect_regions(program, element.ty(), regions, visited, defaults)?;
        }
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Generic(_)
        | CheckedType::GenericInt(_)
        | CheckedType::GenericFloat(_) => {}
    }
    Ok(())
}

fn insert_default(
    defaults: &mut Vec<(DeclarationId, CheckedReleaseClass)>,
    region: DeclarationId,
    class: CheckedReleaseClass,
) {
    if !defaults.iter().any(|(existing, _)| *existing == region) {
        defaults.push((region, class));
    }
}

impl FunctionDependencies {
    fn collect(function: &CheckedFunction) -> Self {
        let mut dependencies = Self::default();
        dependencies.types.push(function.result);
        dependencies
            .types
            .extend(function.parameters.iter().map(|parameter| parameter.ty));
        if matches!(function.body_disposition, CheckedBodyDisposition::Inhabited) {
            dependencies.statements(&function.body);
        }
        dependencies
    }

    fn statements(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. } => self.expression(value),
                CheckedStatement::DestructuringLet {
                    bindings,
                    nominal,
                    value,
                    ..
                } => {
                    self.types.push(CheckedType::Nominal(*nominal));
                    self.types.extend(bindings.iter().map(|(_, ty)| *ty));
                    self.expression(value);
                }
                CheckedStatement::SetList {
                    targets, values, ..
                } => {
                    for target in targets {
                        self.target(target);
                    }
                    for value in values.expressions() {
                        self.expression(value);
                    }
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
                CheckedStatement::Set { target, value, .. }
                | CheckedStatement::Replace { target, value, .. } => {
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
                    if let CheckedStatement::ValueMatchLet { result_type, .. } = statement {
                        self.types.push(*result_type);
                    }
                    if let CheckedEnumType::Nominal(nominal) = enum_type {
                        self.types.push(CheckedType::Nominal(*nominal));
                    }
                    self.expression(scrutinee);
                    for arm in arms {
                        self.types
                            .extend(arm.binders.iter().map(|binder| binder.ty));
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
                CheckedStatement::Dispose { value, drops, .. } => {
                    self.expression(value);
                    self.types.extend(drops.iter().map(|drop| drop.ty));
                }
                CheckedStatement::Region {
                    body,
                    fallthrough_drops,
                    ..
                } => {
                    self.statements(body);
                    self.types
                        .extend(fallthrough_drops.iter().map(|drop| drop.ty));
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
                self.calls.push(CallEdge {
                    source: *function,
                    path: call.clone(),
                    regions: goal_regions.clone(),
                });
            }
            CheckedExpression::KernelCall { instance, .. } => {
                self.types.push(instance.element);
                self.types.extend(instance.run);
            }
            CheckedExpression::BoxDeref { nominal, .. }
            | CheckedExpression::ArenaDeref { nominal, .. }
            | CheckedExpression::ProjectValue { nominal, .. } => {
                self.types.push(CheckedType::Nominal(*nominal));
            }
            CheckedExpression::Project { residual_drops, .. } => {
                self.types.extend(residual_drops.iter().map(|drop| drop.ty));
            }
            CheckedExpression::ContainerMeasure { root, .. }
            | CheckedExpression::ReadStorage { root, .. }
            | CheckedExpression::BorrowAddressed { root, .. } => self.root_types(root),
            CheckedExpression::SliceOf {
                source: crate::semantic::CheckedSliceSource::Run(root),
                ..
            } => {
                self.root_types(root);
            }
            CheckedExpression::BufferFits { element, .. } => self.types.push(*element),
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
            CheckedSetTarget::ArrayIndex(target) => {
                self.types.push(target.array_type);
                self.expression(&target.offset);
            }
            CheckedSetTarget::BufferIndex(target) => self.expression(&target.offset),
            CheckedSetTarget::SliceIndex(target) => self.expression(&target.offset),
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
        for step in &root.path {
            match step {
                CheckedPlaceStep::Field(_) => {}
                CheckedPlaceStep::BoxReferent(nominal) => {
                    self.types.push(CheckedType::Nominal(*nominal));
                }
                CheckedPlaceStep::Subscript(subscript) => {
                    self.types
                        .extend([subscript.base_type, subscript.element_type]);
                }
            }
        }
    }
}
