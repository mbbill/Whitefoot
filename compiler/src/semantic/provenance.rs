//! Finite dependency and subject-only requirement bridge metadata [ENT-6].
//!
//! This module deliberately implements no provenance class and makes no
//! source-acceptance decision.  It retains the finite parameter dependencies,
//! protected-leaf bridge, counterfactual ENT rewalks, and post-convergence
//! witness predecessors that the later held subject-position gate needs.

use std::cmp::Ordering;
use std::collections::HashMap;

use super::entailment::{
    CallGoalCounterfactual, CallGoalDisposition, CallGoalEvidence, EntailmentCallee,
    EntailmentContext, rewalk_function_unasserted,
};
use super::model::{
    BindingId, CheckedArrayRoot, CheckedConstant, CheckedConstantId, CheckedExpression,
    CheckedFunction, CheckedIntegerOperation, CheckedMatchArm, CheckedMode, CheckedNominal,
    CheckedNominalKind, CheckedSetTarget, CheckedSliceSource, CheckedStatement, CheckedType,
    FunctionId,
};
use crate::{DeclarationId, NodePath};

/// One finite component of a value or parameter dependency.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DatumSelector {
    Plain,
    EnumPayload { variant: u32, field: u32 },
}

/// One concrete function parameter component [ENT-6].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ParameterDatum {
    pub(crate) ordinal: u32,
    pub(crate) selector: DatumSelector,
}

/// A deterministic finite set of parameter datums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ParameterDependencies {
    pub(crate) datums: Vec<ParameterDatum>,
}

impl ParameterDependencies {
    fn singleton(datum: ParameterDatum) -> Self {
        Self {
            datums: vec![datum],
        }
    }

    fn insert(&mut self, datum: ParameterDatum) -> bool {
        match self.datums.binary_search(&datum) {
            Ok(_) => false,
            Err(index) => {
                self.datums.insert(index, datum);
                true
            }
        }
    }

    fn union(&mut self, other: &Self) -> bool {
        let mut changed = false;
        for datum in &other.datums {
            changed |= self.insert(*datum);
        }
        changed
    }
}

/// One retained component of a value binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DatumDependencies {
    pub(crate) selector: DatumSelector,
    pub(crate) parameters: ParameterDependencies,
}

/// Per-value dependency with direct enum-payload projections only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValueDependencies {
    pub(crate) components: Vec<DatumDependencies>,
}

impl ValueDependencies {
    fn empty(ty: CheckedType, nominals: &[CheckedNominal]) -> Self {
        Self {
            components: selectors(ty, nominals)
                .into_iter()
                .map(|selector| DatumDependencies {
                    selector,
                    parameters: ParameterDependencies::default(),
                })
                .collect(),
        }
    }

    fn from_aggregate(
        ty: CheckedType,
        aggregate: &ParameterDependencies,
        nominals: &[CheckedNominal],
    ) -> Self {
        let mut value = Self::empty(ty, nominals);
        for component in &mut value.components {
            component.parameters.union(aggregate);
        }
        value
    }

    fn parameter(ordinal: u32, ty: CheckedType, nominals: &[CheckedNominal]) -> Self {
        let mut value = Self::empty(ty, nominals);
        for component in &mut value.components {
            component.parameters = ParameterDependencies::singleton(ParameterDatum {
                ordinal,
                selector: component.selector,
            });
        }
        value
    }

    fn aggregate(&self) -> ParameterDependencies {
        let mut aggregate = ParameterDependencies::default();
        for component in &self.components {
            aggregate.union(&component.parameters);
        }
        aggregate
    }

    fn selected(&self, selector: DatumSelector) -> ParameterDependencies {
        self.components
            .iter()
            .find(|component| component.selector == selector)
            .map_or_else(
                || self.aggregate(),
                |component| component.parameters.clone(),
            )
    }

    fn component_mut(&mut self, selector: DatumSelector) -> Option<&mut DatumDependencies> {
        self.components
            .iter_mut()
            .find(|component| component.selector == selector)
    }

    fn union_value(&mut self, other: &Self) -> bool {
        let same_shape = self
            .components
            .iter()
            .map(|component| component.selector)
            .eq(other.components.iter().map(|component| component.selector));
        if same_shape {
            let mut changed = false;
            for (target, source) in self.components.iter_mut().zip(&other.components) {
                changed |= target.parameters.union(&source.parameters);
            }
            changed
        } else {
            let aggregate = other.aggregate();
            let mut changed = false;
            for component in &mut self.components {
                changed |= component.parameters.union(&aggregate);
            }
            changed
        }
    }
}

/// Retained dependency metadata for one concrete function instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionDependencies {
    pub(crate) function: FunctionId,
    /// Dense [`BindingId`] order. `None` is an unused dense slot.
    pub(crate) bindings: Vec<Option<ValueDependencies>>,
    /// Whole resolved storage roots only, in dense [`BindingId`] order.
    pub(crate) storage_roots: Vec<ParameterDependencies>,
    pub(crate) result: ValueDependencies,
    /// One aggregate content-write dependency per declared parameter.
    pub(crate) writes: Vec<ParameterDependencies>,
}

/// Exact checked occurrence of one concrete requirement.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RequirementOccurrence {
    pub(crate) function: FunctionId,
    pub(crate) final_check: NodePath,
    pub(crate) conjunct: u32,
}

/// Exact protected ENT-6 leaf identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProtectedLeaf {
    pub(crate) function: FunctionId,
    pub(crate) obligation: NodePath,
    pub(crate) conjunct: u32,
}

/// Deterministic predecessor of a converged structural bridge pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StructuralPredecessor {
    Local,
    Call {
        call: NodePath,
        downstream_requirement: RequirementOccurrence,
    },
}

/// Deterministic predecessor of a converged subject-datum bridge triple.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SubjectPredecessor {
    Local,
    Call {
        call: NodePath,
        argument: u32,
        downstream_requirement: RequirementOccurrence,
        downstream_subject: ParameterDatum,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StructuralBridge {
    pub(crate) requirement: RequirementOccurrence,
    pub(crate) leaf: ProtectedLeaf,
    pub(crate) predecessor: StructuralPredecessor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubjectBridge {
    pub(crate) requirement: RequirementOccurrence,
    pub(crate) subject: ParameterDatum,
    pub(crate) leaf: ProtectedLeaf,
    pub(crate) predecessor: SubjectPredecessor,
}

/// One call-goal view retained without turning a counterfactual into a call
/// acceptance judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BridgeGoalView {
    pub(crate) actual_obligations_ok: bool,
    pub(crate) goal_disposition: CallGoalDisposition,
    pub(crate) goal_evidence: Vec<CallGoalEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallSubjectComposition {
    pub(crate) argument: u32,
    pub(crate) callee_subject: ParameterDatum,
    pub(crate) caller_parameters: ParameterDependencies,
}

/// One accepted call connected to one inherited protected leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BridgeCallLink {
    pub(crate) caller: FunctionId,
    pub(crate) call: NodePath,
    pub(crate) downstream_requirement: RequirementOccurrence,
    pub(crate) leaf: ProtectedLeaf,
    pub(crate) subjects: Vec<CallSubjectComposition>,
    pub(crate) full: BridgeGoalView,
    pub(crate) unasserted: BridgeGoalView,
    pub(crate) s4_blinded: BridgeGoalView,
    /// Present only when this call is a monotone upstream bridge generator.
    pub(crate) upstream_requirement: Option<RequirementOccurrence>,
}

/// Complete metadata-only result of the stage-7 ENT-6 analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvenanceMetadata {
    pub(crate) functions: Vec<FunctionDependencies>,
    /// S2/S3-disabled rewalk with the body-entry S4 goal retained.
    pub(crate) unasserted: Vec<super::entailment::FunctionEntailmentRewalk>,
    /// The same rewalk with S4 and its exact L0 projection omitted.
    pub(crate) s4_blinded: Vec<super::entailment::FunctionEntailmentRewalk>,
    pub(crate) structural_bridges: Vec<StructuralBridge>,
    pub(crate) subject_bridges: Vec<SubjectBridge>,
    pub(crate) calls: Vec<BridgeCallLink>,
}

/// Inputs already held by the phase-B semantic inventory.
pub(crate) struct ProvenanceContext<'check> {
    pub(crate) callees: &'check [EntailmentCallee],
    pub(crate) constants: &'check [CheckedConstant],
    pub(crate) constant_ids: &'check HashMap<DeclarationId, CheckedConstantId>,
    pub(crate) nominals: &'check [CheckedNominal],
    pub(crate) binding_names: &'check [Vec<String>],
}

#[derive(Clone, Debug)]
enum HolderRoot {
    Place(BindingId),
    Holder(BindingId),
    Opaque,
}

fn selectors(ty: CheckedType, nominals: &[CheckedNominal]) -> Vec<DatumSelector> {
    let CheckedType::Nominal(nominal) = ty else {
        return vec![DatumSelector::Plain];
    };
    let Some(CheckedNominal {
        kind: CheckedNominalKind::Enum { variants },
        ..
    }) = nominals.get(nominal.0 as usize)
    else {
        return vec![DatumSelector::Plain];
    };
    let mut payloads = Vec::new();
    for (variant, shape) in variants.iter().enumerate() {
        let Ok(variant) = u32::try_from(variant) else {
            continue;
        };
        for (field, _) in shape.fields.iter().enumerate() {
            let Ok(field) = u32::try_from(field) else {
                continue;
            };
            payloads.push(DatumSelector::EnumPayload { variant, field });
        }
    }
    if payloads.is_empty() {
        vec![DatumSelector::Plain]
    } else {
        payloads
    }
}

fn occurrence_cmp(left: &RequirementOccurrence, right: &RequirementOccurrence) -> Ordering {
    left.function
        .0
        .cmp(&right.function.0)
        .then_with(|| {
            left.final_check
                .components()
                .cmp(right.final_check.components())
        })
        .then_with(|| left.conjunct.cmp(&right.conjunct))
}

fn leaf_cmp(left: &ProtectedLeaf, right: &ProtectedLeaf) -> Ordering {
    left.function
        .0
        .cmp(&right.function.0)
        .then_with(|| {
            left.obligation
                .components()
                .cmp(right.obligation.components())
        })
        .then_with(|| left.conjunct.cmp(&right.conjunct))
}

fn requirement_occurrence(function: &CheckedFunction) -> Option<RequirementOccurrence> {
    function
        .requirement
        .as_ref()
        .map(|requirement| RequirementOccurrence {
            function: function.id,
            final_check: requirement.trap.node_path.clone(),
            conjunct: 0,
        })
}

struct FunctionPass<'check> {
    function: &'check CheckedFunction,
    nominals: &'check [CheckedNominal],
    holders: Vec<Option<HolderRoot>>,
    bindings: Vec<Option<ValueDependencies>>,
    roots: Vec<ParameterDependencies>,
    result: ValueDependencies,
    writes: Vec<ParameterDependencies>,
}

impl<'check> FunctionPass<'check> {
    fn new(function: &'check CheckedFunction, nominals: &'check [CheckedNominal]) -> Self {
        let slots = binding_slot_count(function);
        let mut pass = Self {
            function,
            nominals,
            holders: vec![None; slots],
            bindings: vec![None; slots],
            roots: vec![ParameterDependencies::default(); slots],
            result: ValueDependencies::empty(function.result, nominals),
            writes: vec![ParameterDependencies::default(); function.parameters.len()],
        };
        pass.collect_holders(&function.body);
        for (ordinal, parameter) in function.parameters.iter().enumerate() {
            let Ok(ordinal) = u32::try_from(ordinal) else {
                continue;
            };
            let value = ValueDependencies::parameter(ordinal, parameter.ty, nominals);
            pass.set_binding(parameter.binding, parameter.ty, &value);
            if !matches!(parameter.mode, CheckedMode::Own) {
                pass.set_holder(parameter.binding, HolderRoot::Opaque);
            }
        }
        pass
    }

    fn from_metadata(
        function: &'check CheckedFunction,
        nominals: &'check [CheckedNominal],
        metadata: &FunctionDependencies,
    ) -> Self {
        let mut pass = Self::new(function, nominals);
        pass.bindings = metadata.bindings.clone();
        pass.roots = metadata.storage_roots.clone();
        pass.result = metadata.result.clone();
        pass.writes = metadata.writes.clone();
        pass
    }

    fn metadata(self) -> FunctionDependencies {
        FunctionDependencies {
            function: self.function.id,
            bindings: self.bindings,
            storage_roots: self.roots,
            result: self.result,
            writes: self.writes,
        }
    }

    fn set_holder(&mut self, binding: BindingId, holder: HolderRoot) {
        if let Some(slot) = self.holders.get_mut(binding.0 as usize) {
            *slot = Some(holder);
        }
    }

    fn match_holder(scrutinee: &CheckedExpression) -> Option<HolderRoot> {
        match scrutinee {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. }
            | CheckedExpression::Project { binding, .. } => Some(HolderRoot::Holder(*binding)),
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => Self::match_holder(value),
            _ => None,
        }
    }

    fn collect_holders(&mut self, statements: &[CheckedStatement]) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value } => {
                    let holder = match value {
                        CheckedExpression::BorrowAddressed { binding, .. }
                        | CheckedExpression::BorrowBox { binding, .. }
                        | CheckedExpression::BorrowSystemResource { binding, .. } => {
                            Some(HolderRoot::Place(*binding))
                        }
                        CheckedExpression::BorrowBuffer { root } => {
                            Some(HolderRoot::Place(root.binding))
                        }
                        CheckedExpression::ReborrowAddressed { binding, .. } => {
                            Some(HolderRoot::Holder(*binding))
                        }
                        CheckedExpression::BoxNew { .. } => Some(HolderRoot::Opaque),
                        _ => None,
                    };
                    if let Some(holder) = holder {
                        self.set_holder(*binding, holder);
                    }
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    let holder = Self::match_holder(scrutinee);
                    for arm in arms {
                        for binder in &arm.binders {
                            if !matches!(binder.mode, CheckedMode::Own)
                                && let Some(holder) = &holder
                            {
                                // [OWN-13] makes a borrowed payload binder a
                                // child place of the matched holder. Provenance
                                // deliberately retains the whole storage root,
                                // so the payload field itself adds no selector.
                                self.set_holder(binder.binding, holder.clone());
                            }
                        }
                        self.collect_holders(&arm.body);
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => self.collect_holders(body),
                CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Check { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
    }

    fn resolve_root(&self, binding: BindingId) -> BindingId {
        let mut current = binding;
        for _ in 0..=self.holders.len() {
            let next = match self
                .holders
                .get(current.0 as usize)
                .and_then(Option::as_ref)
            {
                Some(HolderRoot::Place(root) | HolderRoot::Holder(root)) => *root,
                Some(HolderRoot::Opaque) | None => return current,
            };
            if next == current {
                return current;
            }
            current = next;
        }
        current
    }

    fn binding(&self, binding: BindingId, ty: CheckedType) -> ValueDependencies {
        self.bindings
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .cloned()
            .unwrap_or_else(|| ValueDependencies::empty(ty, self.nominals))
    }

    fn resolved_binding(&self, binding: BindingId, ty: CheckedType) -> ValueDependencies {
        self.binding(self.resolve_root(binding), ty)
    }

    fn root(&self, binding: BindingId) -> ParameterDependencies {
        let resolved = self.resolve_root(binding);
        self.roots
            .get(resolved.0 as usize)
            .cloned()
            .unwrap_or_default()
    }

    fn set_binding(
        &mut self,
        binding: BindingId,
        ty: CheckedType,
        value: &ValueDependencies,
    ) -> bool {
        let index = binding.0 as usize;
        let mut changed = false;
        if let Some(slot) = self.bindings.get_mut(index) {
            let target = slot.get_or_insert_with(|| ValueDependencies::empty(ty, self.nominals));
            changed |= target.union_value(value);
        }
        if let Some(root) = self.roots.get_mut(index) {
            changed |= root.union(&value.aggregate());
        }
        changed
    }

    fn add_root_write(
        &mut self,
        binding: BindingId,
        dependencies: &ParameterDependencies,
        seed_every_value_component: bool,
    ) -> bool {
        let resolved = self.resolve_root(binding);
        let index = resolved.0 as usize;
        let mut changed = self
            .roots
            .get_mut(index)
            .is_some_and(|root| root.union(dependencies));
        if seed_every_value_component && let Some(Some(value)) = self.bindings.get_mut(index) {
            for component in &mut value.components {
                changed |= component.parameters.union(dependencies);
            }
        }
        for (ordinal, parameter) in self.function.parameters.iter().enumerate() {
            if matches!(parameter.mode, CheckedMode::Unique(_))
                && self.resolve_root(parameter.binding) == resolved
                && let Some(write) = self.writes.get_mut(ordinal)
            {
                changed |= write.union(dependencies);
            }
        }
        changed
    }

    fn scan_until_stable(&mut self, summaries: &[FunctionDependencies]) {
        loop {
            let before = (
                self.bindings.clone(),
                self.roots.clone(),
                self.result.clone(),
                self.writes.clone(),
            );
            self.scan_block(&self.function.body, summaries, None);
            let after = (&self.bindings, &self.roots, &self.result, &self.writes);
            if before.0 == *after.0
                && before.1 == *after.1
                && before.2 == *after.2
                && before.3 == *after.3
            {
                break;
            }
        }
    }

    fn scan_block(
        &mut self,
        statements: &[CheckedStatement],
        summaries: &[FunctionDependencies],
        mut gives: Option<&mut ValueDependencies>,
    ) {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value } => {
                    let dependencies = self.expression(value, summaries);
                    self.set_binding(*binding, value.ty(), &dependencies);
                }
                CheckedStatement::PropagateLet {
                    binding,
                    scrutinee,
                    ok_type,
                    ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries);
                    let ok = scrutinee.selected(DatumSelector::EnumPayload {
                        variant: 0,
                        field: 0,
                    });
                    let ok = ValueDependencies::from_aggregate(*ok_type, &ok, self.nominals);
                    self.set_binding(*binding, *ok_type, &ok);
                    let error = scrutinee.selected(DatumSelector::EnumPayload {
                        variant: 1,
                        field: 0,
                    });
                    if let Some(component) = self.result.component_mut(DatumSelector::EnumPayload {
                        variant: 1,
                        field: 0,
                    }) {
                        component.parameters.union(&error);
                    }
                }
                CheckedStatement::Set { target, value } => {
                    self.scan_set_target(target, summaries);
                    let value = self.expression(value, summaries);
                    let aggregate = value.aggregate();
                    let root = target.binding();
                    let whole_binding =
                        matches!(target, CheckedSetTarget::Place(place) if place.fields.is_empty());
                    self.add_root_write(root, &aggregate, !whole_binding);
                    if let CheckedSetTarget::Place(place) = target
                        && place.fields.is_empty()
                    {
                        let resolved = self.resolve_root(place.binding);
                        self.set_binding(resolved, place.ty, &value);
                    }
                }
                CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. } => {
                    self.expression(value, summaries);
                }
                CheckedStatement::Check { condition, .. }
                | CheckedStatement::Claim { condition, .. } => {
                    self.expression(condition, summaries);
                }
                CheckedStatement::Return { value, .. } => {
                    let value = self.expression(value, summaries);
                    self.result.union_value(&value);
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries);
                    for arm in arms {
                        self.seed_arm_binders(arm, &scrutinee);
                        self.scan_block(&arm.body, summaries, None);
                    }
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    scrutinee,
                    arms,
                    ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries);
                    let mut delivered = ValueDependencies::empty(*result_type, self.nominals);
                    for arm in arms {
                        self.seed_arm_binders(arm, &scrutinee);
                        self.scan_block(&arm.body, summaries, Some(&mut delivered));
                    }
                    self.set_binding(*binding, *result_type, &delivered);
                }
                CheckedStatement::Give { value, .. } => {
                    let value = self.expression(value, summaries);
                    if let Some(target) = gives.as_deref_mut() {
                        target.union_value(&value);
                    }
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.scan_block(body, summaries, None);
                }
                CheckedStatement::CountedRange {
                    binder,
                    lower,
                    upper,
                    body,
                    ..
                } => {
                    let lower = self.expression(lower, summaries);
                    self.expression(upper, summaries);
                    self.set_binding(
                        *binder,
                        CheckedType::Integer(super::model::IntegerType::U64),
                        &lower,
                    );
                    self.scan_block(body, summaries, None);
                }
                CheckedStatement::Break { .. } => {}
            }
        }
    }

    fn seed_arm_binders(&mut self, arm: &CheckedMatchArm, scrutinee: &ValueDependencies) {
        for binder in &arm.binders {
            let selected = scrutinee.selected(DatumSelector::EnumPayload {
                variant: arm.tag,
                field: binder.field,
            });
            let value = ValueDependencies::from_aggregate(binder.ty, &selected, self.nominals);
            self.set_binding(binder.binding, binder.ty, &value);
        }
    }

    fn scan_set_target(&mut self, target: &CheckedSetTarget, summaries: &[FunctionDependencies]) {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                self.expression(&target.offset, summaries);
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.expression(&target.offset, summaries);
            }
        }
    }

    fn expression(
        &mut self,
        expression: &CheckedExpression,
        summaries: &[FunctionDependencies],
    ) -> ValueDependencies {
        match expression {
            CheckedExpression::Constant(_) | CheckedExpression::NamedConstant { .. } => {
                ValueDependencies::empty(expression.ty(), self.nominals)
            }
            CheckedExpression::Binding { binding, ty, .. } => self.binding(*binding, *ty),
            CheckedExpression::UserCall {
                function,
                arguments,
                result,
                ..
            } => {
                let actuals = arguments
                    .iter()
                    .map(|argument| self.expression(argument, summaries))
                    .collect::<Vec<_>>();
                let Some(callee) = summaries.get(function.0 as usize) else {
                    return ValueDependencies::empty(*result, self.nominals);
                };
                for (ordinal, dependencies) in callee.writes.iter().enumerate() {
                    let substituted = substitute_parameters(dependencies, &actuals);
                    if let Some(argument) = arguments.get(ordinal)
                        && let Some(root) = self.argument_root(argument)
                    {
                        self.add_root_write(root, &substituted, true);
                    }
                }
                substitute_value(&callee.result, &actuals)
            }
            CheckedExpression::SystemCall {
                arguments, result, ..
            } => {
                for argument in arguments {
                    self.expression(argument, summaries);
                }
                // System provenance is deliberately the later gate's
                // separate declaration data. It adds no parameter datum.
                ValueDependencies::empty(*result, self.nominals)
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                result,
                ..
            } => {
                let operands = self.expression_aggregate(arguments, summaries);
                let mut value = ValueDependencies::empty(*result, self.nominals);
                if matches!(
                    operation,
                    CheckedIntegerOperation::AddChecked
                        | CheckedIntegerOperation::SubtractChecked
                        | CheckedIntegerOperation::MultiplyChecked
                        | CheckedIntegerOperation::DivideChecked
                        | CheckedIntegerOperation::RemainderChecked
                        | CheckedIntegerOperation::AbsoluteChecked
                        | CheckedIntegerOperation::NegateChecked
                ) {
                    if let Some(ok) = value.component_mut(DatumSelector::EnumPayload {
                        variant: 0,
                        field: 0,
                    }) {
                        ok.parameters.union(&operands);
                    }
                } else {
                    for component in &mut value.components {
                        component.parameters.union(&operands);
                    }
                }
                value
            }
            CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. } => {
                let aggregate = self.expression_aggregate(arguments, summaries);
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                result,
            } => {
                let operand = self.expression(value, summaries).aggregate();
                let mut converted = ValueDependencies::empty(*result, self.nominals);
                if source.converts_totally_to(*destination) {
                    for component in &mut converted.components {
                        component.parameters.union(&operand);
                    }
                } else if let Some(ok) = converted.component_mut(DatumSelector::EnumPayload {
                    variant: 0,
                    field: 0,
                }) {
                    ok.parameters.union(&operand);
                }
                converted
            }
            CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. } => {
                let value = self.expression(value, summaries);
                let aggregate = value.aggregate();
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ArrayFill { value, .. } => {
                let aggregate = self.expression(value, summaries).aggregate();
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceLength { .. } => {
                ValueDependencies::empty(expression.ty(), self.nominals)
            }
            CheckedExpression::ArrayIndex { root, offset, .. } => {
                let mut aggregate = self.array_root(root);
                aggregate.union(&self.expression(offset, summaries).aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                let mut aggregate = self.expression(length, summaries).aggregate();
                aggregate.union(&self.expression(value, summaries).aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let mut aggregate = self.root(root.binding);
                aggregate.union(&self.expression(offset, summaries).aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::SliceOf { source, .. } => {
                let aggregate = match source {
                    CheckedSliceSource::Array { root, .. } => self.array_root(root),
                    CheckedSliceSource::Buffer(root) => self.root(root.binding),
                };
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let mut aggregate = self.root(root.binding);
                aggregate.union(&self.expression(offset, summaries).aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BorrowBuffer { root } => {
                let aggregate = self.root(root.binding);
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => {
                self.resolved_binding(*binding, expression.ty())
            }
            CheckedExpression::ConstructStruct { fields, .. } => {
                let aggregate = self.expression_aggregate(fields, summaries);
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ConstructEnum {
                variant, fields, ..
            } => {
                let mut value = ValueDependencies::empty(expression.ty(), self.nominals);
                for (field, expression) in fields.iter().enumerate() {
                    let Ok(field) = u32::try_from(field) else {
                        continue;
                    };
                    let dependencies = self.expression(expression, summaries).aggregate();
                    if let Some(component) = value.component_mut(DatumSelector::EnumPayload {
                        variant: *variant,
                        field,
                    }) {
                        component.parameters.union(&dependencies);
                    }
                }
                value
            }
            CheckedExpression::Project { binding, ty, .. } => {
                let aggregate = self.root(*binding);
                ValueDependencies::from_aggregate(*ty, &aggregate, self.nominals)
            }
            CheckedExpression::ProjectValue { value, ty, .. } => {
                let aggregate = self.expression(value, summaries).aggregate();
                ValueDependencies::from_aggregate(*ty, &aggregate, self.nominals)
            }
        }
    }

    fn expression_aggregate(
        &mut self,
        expressions: &[CheckedExpression],
        summaries: &[FunctionDependencies],
    ) -> ParameterDependencies {
        let mut aggregate = ParameterDependencies::default();
        for expression in expressions {
            aggregate.union(&self.expression(expression, summaries).aggregate());
        }
        aggregate
    }

    fn array_root(&self, root: &CheckedArrayRoot) -> ParameterDependencies {
        match root {
            CheckedArrayRoot::Binding { binding, .. } => self.root(*binding),
            CheckedArrayRoot::Constant(_) => ParameterDependencies::default(),
        }
    }

    fn argument_root(&self, argument: &CheckedExpression) -> Option<BindingId> {
        let binding = match argument {
            CheckedExpression::Binding { binding, .. } => *binding,
            CheckedExpression::BorrowBuffer { root } => root.binding,
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. } => *binding,
            _ => return None,
        };
        Some(self.resolve_root(binding))
    }
}

fn substitute_parameters(
    dependencies: &ParameterDependencies,
    actuals: &[ValueDependencies],
) -> ParameterDependencies {
    let mut substituted = ParameterDependencies::default();
    for datum in &dependencies.datums {
        if let Some(actual) = actuals.get(datum.ordinal as usize) {
            substituted.union(&actual.selected(datum.selector));
        }
    }
    substituted
}

fn substitute_value(value: &ValueDependencies, actuals: &[ValueDependencies]) -> ValueDependencies {
    ValueDependencies {
        components: value
            .components
            .iter()
            .map(|component| DatumDependencies {
                selector: component.selector,
                parameters: substitute_parameters(&component.parameters, actuals),
            })
            .collect(),
    }
}

fn binding_slot_count(function: &CheckedFunction) -> usize {
    let mut maximum = function
        .parameters
        .iter()
        .map(|parameter| parameter.binding.0)
        .max();
    binding_maximum(&function.body, &mut maximum);
    maximum.map_or(0, |binding| binding as usize + 1)
}

fn binding_maximum(statements: &[CheckedStatement], maximum: &mut Option<u32>) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { binding, .. }
            | CheckedStatement::PropagateLet { binding, .. } => {
                include_binding(maximum, *binding);
            }
            CheckedStatement::Match { arms, .. } => {
                for arm in arms {
                    for binder in &arm.binders {
                        include_binding(maximum, binder.binding);
                    }
                    binding_maximum(&arm.body, maximum);
                }
            }
            CheckedStatement::ValueMatchLet { binding, arms, .. } => {
                include_binding(maximum, *binding);
                for arm in arms {
                    for binder in &arm.binders {
                        include_binding(maximum, binder.binding);
                    }
                    binding_maximum(&arm.body, maximum);
                }
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                binding_maximum(body, maximum);
            }
            CheckedStatement::CountedRange { binder, body, .. } => {
                include_binding(maximum, *binder);
                binding_maximum(body, maximum);
            }
            CheckedStatement::Set { .. }
            | CheckedStatement::Evaluate(_)
            | CheckedStatement::DropExpression { .. }
            | CheckedStatement::Check { .. }
            | CheckedStatement::Claim { .. }
            | CheckedStatement::Return { .. }
            | CheckedStatement::Give { .. }
            | CheckedStatement::Break { .. } => {}
        }
    }
}

fn include_binding(maximum: &mut Option<u32>, binding: BindingId) {
    *maximum = Some(maximum.map_or(binding.0, |current| current.max(binding.0)));
}

fn dependency_fixed_point(
    functions: &[CheckedFunction],
    nominals: &[CheckedNominal],
) -> Vec<FunctionDependencies> {
    let mut summaries = functions
        .iter()
        .map(|function| FunctionPass::new(function, nominals).metadata())
        .collect::<Vec<_>>();
    loop {
        let previous = summaries.clone();
        for function in functions {
            let mut pass = FunctionPass::new(function, nominals);
            pass.scan_until_stable(&previous);
            let derived = pass.metadata();
            if let Some(summary) = summaries.get_mut(function.id.0 as usize) {
                summary.result.union_value(&derived.result);
                for (target, source) in summary.writes.iter_mut().zip(&derived.writes) {
                    target.union(source);
                }
            }
        }
        if summaries == previous {
            break;
        }
    }
    functions
        .iter()
        .map(|function| {
            let mut pass = FunctionPass::new(function, nominals);
            pass.scan_until_stable(&summaries);
            pass.metadata()
        })
        .collect()
}

#[derive(Clone)]
struct LeafSite {
    leaf: ProtectedLeaf,
    offset: CheckedExpression,
}

#[derive(Clone)]
struct CallSite {
    caller: FunctionId,
    call: NodePath,
    downstream_requirement: RequirementOccurrence,
    arguments: Vec<CheckedExpression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StructuralKey {
    requirement: RequirementOccurrence,
    leaf: ProtectedLeaf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SubjectKey {
    requirement: RequirementOccurrence,
    subject: ParameterDatum,
    leaf: ProtectedLeaf,
}

#[derive(Clone)]
struct CallInventory {
    site: CallSite,
    caller_requirement: Option<RequirementOccurrence>,
    actuals: Vec<ValueDependencies>,
    full: BridgeGoalView,
    unasserted: BridgeGoalView,
    blinded: BridgeGoalView,
}

fn structural_key_cmp(left: &StructuralKey, right: &StructuralKey) -> Ordering {
    occurrence_cmp(&left.requirement, &right.requirement)
        .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
}

fn subject_key_cmp(left: &SubjectKey, right: &SubjectKey) -> Ordering {
    occurrence_cmp(&left.requirement, &right.requirement)
        .then_with(|| left.subject.cmp(&right.subject))
        .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
}

fn insert_structural(keys: &mut Vec<StructuralKey>, key: StructuralKey) -> bool {
    match keys.binary_search_by(|candidate| structural_key_cmp(candidate, &key)) {
        Ok(_) => false,
        Err(index) => {
            keys.insert(index, key);
            true
        }
    }
}

fn insert_subject(keys: &mut Vec<SubjectKey>, key: SubjectKey) -> bool {
    match keys.binary_search_by(|candidate| subject_key_cmp(candidate, &key)) {
        Ok(_) => false,
        Err(index) => {
            keys.insert(index, key);
            true
        }
    }
}

fn collect_sites(function: &CheckedFunction) -> (Vec<LeafSite>, Vec<CallSite>) {
    let mut leaves = Vec::new();
    let mut calls = Vec::new();
    collect_block_sites(function.id, &function.body, &mut leaves, &mut calls);
    (leaves, calls)
}

fn collect_block_sites(
    function: FunctionId,
    statements: &[CheckedStatement],
    leaves: &mut Vec<LeafSite>,
    calls: &mut Vec<CallSite>,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Check {
                condition: value, ..
            }
            | CheckedStatement::Claim {
                condition: value, ..
            }
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. } => {
                collect_expression_sites(function, value, leaves, calls);
            }
            CheckedStatement::PropagateLet { scrutinee, .. }
            | CheckedStatement::Match { scrutinee, .. }
            | CheckedStatement::ValueMatchLet { scrutinee, .. } => {
                collect_expression_sites(function, scrutinee, leaves, calls);
                if let CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } = statement
                {
                    for arm in arms {
                        collect_block_sites(function, &arm.body, leaves, calls);
                    }
                }
            }
            CheckedStatement::Set { target, value } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        collect_expression_sites(function, &target.offset, leaves, calls);
                        leaves.push(LeafSite {
                            leaf: ProtectedLeaf {
                                function,
                                obligation: target.trap.node_path.clone(),
                                conjunct: 0,
                            },
                            offset: target.offset.clone(),
                        });
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression_sites(function, &target.offset, leaves, calls);
                        leaves.push(LeafSite {
                            leaf: ProtectedLeaf {
                                function,
                                obligation: target.trap.node_path.clone(),
                                conjunct: 0,
                            },
                            offset: target.offset.clone(),
                        });
                    }
                }
                collect_expression_sites(function, value, leaves, calls);
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                collect_block_sites(function, body, leaves, calls);
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                collect_expression_sites(function, lower, leaves, calls);
                collect_expression_sites(function, upper, leaves, calls);
                collect_block_sites(function, body, leaves, calls);
            }
            CheckedStatement::Break { .. } => {}
        }
    }
}

fn collect_expression_sites(
    function: FunctionId,
    expression: &CheckedExpression,
    leaves: &mut Vec<LeafSite>,
    calls: &mut Vec<CallSite>,
) {
    match expression {
        CheckedExpression::UserCall {
            function: callee,
            call,
            arguments,
            requirement,
            ..
        } => {
            for argument in arguments {
                collect_expression_sites(function, argument, leaves, calls);
            }
            if let Some(requirement) = requirement {
                calls.push(CallSite {
                    caller: function,
                    call: call.clone(),
                    downstream_requirement: RequirementOccurrence {
                        function: *callee,
                        final_check: requirement.final_check.clone(),
                        conjunct: 0,
                    },
                    arguments: arguments.clone(),
                });
            }
        }
        CheckedExpression::ArrayIndex { offset, trap, .. }
        | CheckedExpression::BufferIndex { offset, trap, .. }
        | CheckedExpression::SliceIndex { offset, trap, .. } => {
            collect_expression_sites(function, offset, leaves, calls);
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: trap.node_path.clone(),
                    conjunct: 0,
                },
                offset: (**offset).clone(),
            });
        }
        _ => {
            for child in expression_children(expression) {
                collect_expression_sites(function, child, leaves, calls);
            }
        }
    }
}

fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
    match expression {
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayLength { .. }
        | CheckedExpression::BufferLength { .. }
        | CheckedExpression::SliceOf { .. }
        | CheckedExpression::SliceLength { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowAddressed { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => Vec::new(),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::SystemCall { arguments, .. }
        | CheckedExpression::IntegerOperation { arguments, .. }
        | CheckedExpression::FloatOperation { arguments, .. }
        | CheckedExpression::BooleanOperation { arguments, .. }
        | CheckedExpression::EnumEquality { arguments, .. }
        | CheckedExpression::ConstructStruct {
            fields: arguments, ..
        }
        | CheckedExpression::ConstructEnum {
            fields: arguments, ..
        } => arguments.iter().collect(),
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value],
        CheckedExpression::ArrayIndex { offset, .. }
        | CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset],
        CheckedExpression::BufferFill { length, value, .. } => vec![length, value],
    }
}

fn call_counterfactual<'outcome>(
    outcomes: &'outcome [CallGoalCounterfactual],
    call: &NodePath,
) -> Option<&'outcome CallGoalCounterfactual> {
    outcomes.iter().find(|outcome| outcome.node_path == *call)
}

fn counterfactual_view(outcome: &CallGoalCounterfactual) -> BridgeGoalView {
    BridgeGoalView {
        actual_obligations_ok: outcome.actual_obligations_ok,
        goal_disposition: outcome.goal_disposition,
        goal_evidence: outcome.goal_evidence.clone(),
    }
}

fn call_is_upstream_generator(call: &CallInventory) -> bool {
    call.unasserted.goal_disposition == CallGoalDisposition::Discharged
        && call.blinded.goal_disposition != CallGoalDisposition::Discharged
        && call.caller_requirement.is_some()
}

fn actual_values(
    function: &CheckedFunction,
    metadata: &FunctionDependencies,
    arguments: &[CheckedExpression],
    summaries: &[FunctionDependencies],
    nominals: &[CheckedNominal],
) -> Vec<ValueDependencies> {
    let mut pass = FunctionPass::from_metadata(function, nominals, metadata);
    arguments
        .iter()
        .map(|argument| pass.expression(argument, summaries))
        .collect()
}

fn build_call_inventory(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    unasserted: &[super::entailment::FunctionEntailmentRewalk],
    blinded: &[super::entailment::FunctionEntailmentRewalk],
    nominals: &[CheckedNominal],
) -> Vec<CallInventory> {
    let mut calls = Vec::new();
    for function in functions {
        let (_, sites) = collect_sites(function);
        let Some(function_dependencies) = dependencies.get(function.id.0 as usize) else {
            continue;
        };
        let Some(unasserted) = unasserted.get(function.id.0 as usize) else {
            continue;
        };
        let Some(blinded) = blinded.get(function.id.0 as usize) else {
            continue;
        };
        for site in sites {
            let Some(full) = function
                .entailment
                .call_goals
                .iter()
                .find(|outcome| outcome.node_path == site.call)
            else {
                continue;
            };
            if full.disposition != CallGoalDisposition::Discharged {
                continue;
            }
            let Some(unasserted) = call_counterfactual(&unasserted.call_goals, &site.call) else {
                continue;
            };
            let Some(blinded) = call_counterfactual(&blinded.call_goals, &site.call) else {
                continue;
            };
            let actuals = actual_values(
                function,
                function_dependencies,
                &site.arguments,
                dependencies,
                nominals,
            );
            calls.push(CallInventory {
                site,
                caller_requirement: requirement_occurrence(function),
                actuals,
                full: BridgeGoalView {
                    actual_obligations_ok: true,
                    goal_disposition: full.disposition,
                    goal_evidence: full.evidence.clone(),
                },
                unasserted: counterfactual_view(unasserted),
                blinded: counterfactual_view(blinded),
            });
        }
    }
    calls.sort_by(|left, right| {
        left.site
            .caller
            .0
            .cmp(&right.site.caller.0)
            .then_with(|| {
                left.site
                    .call
                    .components()
                    .cmp(right.site.call.components())
            })
            .then_with(|| {
                occurrence_cmp(
                    &left.site.downstream_requirement,
                    &right.site.downstream_requirement,
                )
            })
    });
    calls
}

fn local_bridge_seeds(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    unasserted: &[super::entailment::FunctionEntailmentRewalk],
    blinded: &[super::entailment::FunctionEntailmentRewalk],
    nominals: &[CheckedNominal],
) -> (Vec<StructuralKey>, Vec<SubjectKey>) {
    let mut structural = Vec::new();
    let mut subjects = Vec::new();
    for function in functions {
        let Some(requirement) = requirement_occurrence(function) else {
            continue;
        };
        let Some(function_dependencies) = dependencies.get(function.id.0 as usize) else {
            continue;
        };
        let Some(unasserted) = unasserted.get(function.id.0 as usize) else {
            continue;
        };
        let Some(blinded) = blinded.get(function.id.0 as usize) else {
            continue;
        };
        let (leaves, _) = collect_sites(function);
        for site in leaves {
            let full_discharged = function
                .entailment
                .obligations
                .iter()
                .any(|outcome| outcome.node_path == site.leaf.obligation && outcome.discharged);
            let unasserted_discharged = unasserted
                .obligations
                .iter()
                .any(|outcome| outcome.node_path == site.leaf.obligation && outcome.discharged);
            let blinded_discharged = blinded
                .obligations
                .iter()
                .any(|outcome| outcome.node_path == site.leaf.obligation && outcome.discharged);
            if !full_discharged || !unasserted_discharged || blinded_discharged {
                continue;
            }
            insert_structural(
                &mut structural,
                StructuralKey {
                    requirement: requirement.clone(),
                    leaf: site.leaf.clone(),
                },
            );
            let mut pass = FunctionPass::from_metadata(function, nominals, function_dependencies);
            let dependency = pass.expression(&site.offset, dependencies).aggregate();
            for subject in dependency.datums {
                insert_subject(
                    &mut subjects,
                    SubjectKey {
                        requirement: requirement.clone(),
                        subject,
                        leaf: site.leaf.clone(),
                    },
                );
            }
        }
    }
    (structural, subjects)
}

fn bridge_fixed_point(
    calls: &[CallInventory],
    local_structural: &[StructuralKey],
    local_subjects: &[SubjectKey],
) -> (Vec<StructuralKey>, Vec<SubjectKey>) {
    let mut structural = local_structural.to_vec();
    let mut subjects = local_subjects.to_vec();
    loop {
        let structural_before = structural.clone();
        let subjects_before = subjects.clone();
        for call in calls {
            if !call_is_upstream_generator(call) {
                continue;
            }
            let Some(caller_requirement) = call.caller_requirement.as_ref() else {
                continue;
            };
            for downstream in &structural_before {
                if downstream.requirement != call.site.downstream_requirement {
                    continue;
                }
                insert_structural(
                    &mut structural,
                    StructuralKey {
                        requirement: caller_requirement.clone(),
                        leaf: downstream.leaf.clone(),
                    },
                );
            }
            for downstream in &subjects_before {
                if downstream.requirement != call.site.downstream_requirement {
                    continue;
                }
                let Some(actual) = call.actuals.get(downstream.subject.ordinal as usize) else {
                    continue;
                };
                for subject in actual.selected(downstream.subject.selector).datums {
                    insert_subject(
                        &mut subjects,
                        SubjectKey {
                            requirement: caller_requirement.clone(),
                            subject,
                            leaf: downstream.leaf.clone(),
                        },
                    );
                }
            }
        }
        if structural == structural_before && subjects == subjects_before {
            break;
        }
    }
    (structural, subjects)
}

fn predecessor_cmp(left: &StructuralPredecessor, right: &StructuralPredecessor) -> Ordering {
    match (left, right) {
        (StructuralPredecessor::Local, StructuralPredecessor::Local) => Ordering::Equal,
        (StructuralPredecessor::Local, StructuralPredecessor::Call { .. }) => Ordering::Less,
        (StructuralPredecessor::Call { .. }, StructuralPredecessor::Local) => Ordering::Greater,
        (
            StructuralPredecessor::Call {
                call: left_call,
                downstream_requirement: left_requirement,
            },
            StructuralPredecessor::Call {
                call: right_call,
                downstream_requirement: right_requirement,
            },
        ) => left_call
            .components()
            .cmp(right_call.components())
            .then_with(|| occurrence_cmp(left_requirement, right_requirement)),
    }
}

fn subject_predecessor_cmp(left: &SubjectPredecessor, right: &SubjectPredecessor) -> Ordering {
    match (left, right) {
        (SubjectPredecessor::Local, SubjectPredecessor::Local) => Ordering::Equal,
        (SubjectPredecessor::Local, SubjectPredecessor::Call { .. }) => Ordering::Less,
        (SubjectPredecessor::Call { .. }, SubjectPredecessor::Local) => Ordering::Greater,
        (
            SubjectPredecessor::Call {
                call: left_call,
                argument: left_argument,
                downstream_requirement: left_requirement,
                downstream_subject: left_subject,
            },
            SubjectPredecessor::Call {
                call: right_call,
                argument: right_argument,
                downstream_requirement: right_requirement,
                downstream_subject: right_subject,
            },
        ) => left_call
            .components()
            .cmp(right_call.components())
            .then_with(|| left_argument.cmp(right_argument))
            .then_with(|| occurrence_cmp(left_requirement, right_requirement))
            .then_with(|| left_subject.cmp(right_subject)),
    }
}

fn structural_index(keys: &[StructuralKey], key: &StructuralKey) -> Option<usize> {
    keys.binary_search_by(|candidate| structural_key_cmp(candidate, key))
        .ok()
}

fn subject_index(keys: &[SubjectKey], key: &SubjectKey) -> Option<usize> {
    keys.binary_search_by(|candidate| subject_key_cmp(candidate, key))
        .ok()
}

fn update_structural_witness(
    distances: &mut [Option<u32>],
    predecessors: &mut [Option<StructuralPredecessor>],
    index: usize,
    distance: u32,
    predecessor: StructuralPredecessor,
) -> bool {
    let replace = match (distances[index], predecessors[index].as_ref()) {
        (None, _) => true,
        (Some(current), _) if distance < current => true,
        (Some(current), Some(existing)) if distance == current => {
            predecessor_cmp(&predecessor, existing) == Ordering::Less
        }
        _ => false,
    };
    if replace {
        distances[index] = Some(distance);
        predecessors[index] = Some(predecessor);
    }
    replace
}

fn update_subject_witness(
    distances: &mut [Option<u32>],
    predecessors: &mut [Option<SubjectPredecessor>],
    index: usize,
    distance: u32,
    predecessor: SubjectPredecessor,
) -> bool {
    let replace = match (distances[index], predecessors[index].as_ref()) {
        (None, _) => true,
        (Some(current), _) if distance < current => true,
        (Some(current), Some(existing)) if distance == current => {
            subject_predecessor_cmp(&predecessor, existing) == Ordering::Less
        }
        _ => false,
    };
    if replace {
        distances[index] = Some(distance);
        predecessors[index] = Some(predecessor);
    }
    replace
}

fn reconstruct_structural_bridges(
    converged: &[StructuralKey],
    local: &[StructuralKey],
    calls: &[CallInventory],
) -> Vec<StructuralBridge> {
    let mut distances = vec![None; converged.len()];
    let mut predecessors = vec![None; converged.len()];
    for key in local {
        if let Some(index) = structural_index(converged, key) {
            update_structural_witness(
                &mut distances,
                &mut predecessors,
                index,
                0,
                StructuralPredecessor::Local,
            );
        }
    }

    loop {
        let mut changed = false;
        for call in calls {
            if !call_is_upstream_generator(call) {
                continue;
            }
            let Some(caller_requirement) = call.caller_requirement.as_ref() else {
                continue;
            };
            for (downstream_index, downstream) in converged.iter().enumerate() {
                if downstream.requirement != call.site.downstream_requirement {
                    continue;
                }
                let Some(downstream_distance) = distances[downstream_index] else {
                    continue;
                };
                let upstream = StructuralKey {
                    requirement: caller_requirement.clone(),
                    leaf: downstream.leaf.clone(),
                };
                let Some(upstream_index) = structural_index(converged, &upstream) else {
                    continue;
                };
                changed |= update_structural_witness(
                    &mut distances,
                    &mut predecessors,
                    upstream_index,
                    downstream_distance.saturating_add(1),
                    StructuralPredecessor::Call {
                        call: call.site.call.clone(),
                        downstream_requirement: downstream.requirement.clone(),
                    },
                );
            }
        }
        if !changed {
            break;
        }
    }

    let bridges = converged
        .iter()
        .zip(predecessors)
        .filter_map(|(key, predecessor)| {
            predecessor.map(|predecessor| StructuralBridge {
                requirement: key.requirement.clone(),
                leaf: key.leaf.clone(),
                predecessor,
            })
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(bridges.len(), converged.len());
    bridges
}

fn reconstruct_subject_bridges(
    converged: &[SubjectKey],
    local: &[SubjectKey],
    calls: &[CallInventory],
) -> Vec<SubjectBridge> {
    let mut distances = vec![None; converged.len()];
    let mut predecessors = vec![None; converged.len()];
    for key in local {
        if let Some(index) = subject_index(converged, key) {
            update_subject_witness(
                &mut distances,
                &mut predecessors,
                index,
                0,
                SubjectPredecessor::Local,
            );
        }
    }

    loop {
        let mut changed = false;
        for call in calls {
            if !call_is_upstream_generator(call) {
                continue;
            }
            let Some(caller_requirement) = call.caller_requirement.as_ref() else {
                continue;
            };
            for (downstream_index, downstream) in converged.iter().enumerate() {
                if downstream.requirement != call.site.downstream_requirement {
                    continue;
                }
                let Some(downstream_distance) = distances[downstream_index] else {
                    continue;
                };
                let Some(actual) = call.actuals.get(downstream.subject.ordinal as usize) else {
                    continue;
                };
                for caller_subject in actual.selected(downstream.subject.selector).datums {
                    let upstream = SubjectKey {
                        requirement: caller_requirement.clone(),
                        subject: caller_subject,
                        leaf: downstream.leaf.clone(),
                    };
                    let Some(upstream_index) = subject_index(converged, &upstream) else {
                        continue;
                    };
                    changed |= update_subject_witness(
                        &mut distances,
                        &mut predecessors,
                        upstream_index,
                        downstream_distance.saturating_add(1),
                        SubjectPredecessor::Call {
                            call: call.site.call.clone(),
                            argument: downstream.subject.ordinal,
                            downstream_requirement: downstream.requirement.clone(),
                            downstream_subject: downstream.subject,
                        },
                    );
                }
            }
        }
        if !changed {
            break;
        }
    }

    let bridges = converged
        .iter()
        .zip(predecessors)
        .filter_map(|(key, predecessor)| {
            predecessor.map(|predecessor| SubjectBridge {
                requirement: key.requirement.clone(),
                subject: key.subject,
                leaf: key.leaf.clone(),
                predecessor,
            })
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(bridges.len(), converged.len());
    bridges
}

fn build_call_links(
    calls: &[CallInventory],
    structural: &[StructuralKey],
    subjects: &[SubjectKey],
) -> Vec<BridgeCallLink> {
    let mut links = Vec::new();
    for call in calls {
        for bridge in structural {
            if bridge.requirement != call.site.downstream_requirement {
                continue;
            }
            let mut compositions = Vec::new();
            for subject in subjects {
                if subject.requirement != bridge.requirement || subject.leaf != bridge.leaf {
                    continue;
                }
                let Some(actual) = call.actuals.get(subject.subject.ordinal as usize) else {
                    continue;
                };
                compositions.push(CallSubjectComposition {
                    argument: subject.subject.ordinal,
                    callee_subject: subject.subject,
                    caller_parameters: actual.selected(subject.subject.selector),
                });
            }
            compositions.sort_by(|left, right| {
                left.argument
                    .cmp(&right.argument)
                    .then_with(|| left.callee_subject.cmp(&right.callee_subject))
                    .then_with(|| {
                        left.caller_parameters
                            .datums
                            .cmp(&right.caller_parameters.datums)
                    })
            });
            compositions.dedup();
            links.push(BridgeCallLink {
                caller: call.site.caller,
                call: call.site.call.clone(),
                downstream_requirement: call.site.downstream_requirement.clone(),
                leaf: bridge.leaf.clone(),
                subjects: compositions,
                full: call.full.clone(),
                unasserted: call.unasserted.clone(),
                s4_blinded: call.blinded.clone(),
                upstream_requirement: call_is_upstream_generator(call)
                    .then(|| call.caller_requirement.clone())
                    .flatten(),
            });
        }
    }
    links.sort_by(|left, right| {
        left.caller
            .0
            .cmp(&right.caller.0)
            .then_with(|| left.call.components().cmp(right.call.components()))
            .then_with(|| {
                occurrence_cmp(&left.downstream_requirement, &right.downstream_requirement)
            })
            .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
    });
    links
}

/// Computes all stage-7 dependency, rewalk, bridge, and witness metadata.
///
/// This result is observational only: neither this function nor a consumer of
/// its result participates in source acceptance or lowering.
pub(crate) fn analyze_program_provenance(
    functions: &[CheckedFunction],
    context: &ProvenanceContext<'_>,
) -> ProvenanceMetadata {
    let dependencies = dependency_fixed_point(functions, context.nominals);
    let mut unasserted = Vec::with_capacity(functions.len());
    let mut blinded = Vec::with_capacity(functions.len());
    for function in functions {
        let binding_names = context
            .binding_names
            .get(function.id.0 as usize)
            .map_or(&[][..], Vec::as_slice);
        let entailment_context = EntailmentContext {
            callees: context.callees,
            constants: context.constants,
            constant_ids: context.constant_ids,
            nominals: context.nominals,
            binding_names,
        };
        unasserted.push(rewalk_function_unasserted(
            function,
            &entailment_context,
            true,
        ));
        blinded.push(rewalk_function_unasserted(
            function,
            &entailment_context,
            false,
        ));
    }

    let (local_structural, local_subjects) = local_bridge_seeds(
        functions,
        &dependencies,
        &unasserted,
        &blinded,
        context.nominals,
    );
    let calls = build_call_inventory(
        functions,
        &dependencies,
        &unasserted,
        &blinded,
        context.nominals,
    );
    let (structural, subjects) = bridge_fixed_point(&calls, &local_structural, &local_subjects);
    let structural_bridges = reconstruct_structural_bridges(&structural, &local_structural, &calls);
    let subject_bridges = reconstruct_subject_bridges(&subjects, &local_subjects, &calls);
    let calls = build_call_links(&calls, &structural, &subjects);

    ProvenanceMetadata {
        functions: dependencies,
        unasserted,
        s4_blinded: blinded,
        structural_bridges,
        subject_bridges,
        calls,
    }
}
