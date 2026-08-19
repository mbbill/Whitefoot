//! Finite explicit-dataflow provenance and constrained-subject gate [PRV-1/2/3].
//!
//! Component pairs converge and freeze before protected demands are derived.
//! The second stratum then composes direct demands and exact requirement
//! bridges without feeding either result back into provenance classification.

use std::cmp::Ordering;

use super::entailment::{
    CallGoalCounterfactual, CallGoalDisposition, CallGoalEvidence, CallGoalOutcome,
};
use super::model::{
    BindingId, CheckedArrayRoot, CheckedExpression, CheckedFunction, CheckedIntegerOperation,
    CheckedMatchArm, CheckedMode, CheckedNominal, CheckedNominalKind, CheckedSetTarget,
    CheckedSliceSource, CheckedStatement, CheckedType, FunctionId,
};
use crate::{NodePath, SemanticCompilerFailure};

type ProvenanceResult<T> = Result<T, SemanticCompilerFailure>;

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

/// One PRV-1 component pair.  The bit is an unconditional origin, never a
/// synthetic parameter datum, so substitution preserves bit-only terminals.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProvenanceDependency {
    pub(crate) unconditional_external: bool,
    pub(crate) parameters: ParameterDependencies,
}

impl ProvenanceDependency {
    fn parameter(datum: ParameterDatum) -> Self {
        Self {
            unconditional_external: false,
            parameters: ParameterDependencies::singleton(datum),
        }
    }

    fn external() -> Self {
        Self {
            unconditional_external: true,
            parameters: ParameterDependencies::default(),
        }
    }

    fn union(&mut self, other: &Self) -> bool {
        let changed = !self.unconditional_external && other.unconditional_external;
        self.unconditional_external |= other.unconditional_external;
        self.parameters.union(&other.parameters) | changed
    }
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
    pub(crate) dependency: ProvenanceDependency,
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
                    dependency: ProvenanceDependency::default(),
                })
                .collect(),
        }
    }

    fn from_aggregate(
        ty: CheckedType,
        aggregate: &ProvenanceDependency,
        nominals: &[CheckedNominal],
    ) -> Self {
        let mut value = Self::empty(ty, nominals);
        for component in &mut value.components {
            component.dependency.union(aggregate);
        }
        value
    }

    fn parameter(ordinal: u32, ty: CheckedType, nominals: &[CheckedNominal]) -> Self {
        let mut value = Self::empty(ty, nominals);
        for component in &mut value.components {
            component.dependency = ProvenanceDependency::parameter(ParameterDatum {
                ordinal,
                selector: component.selector,
            });
        }
        value
    }

    fn external(ty: CheckedType, nominals: &[CheckedNominal]) -> Self {
        let mut value = Self::empty(ty, nominals);
        for component in &mut value.components {
            component.dependency = ProvenanceDependency::external();
        }
        value
    }

    fn aggregate(&self) -> ProvenanceDependency {
        let mut aggregate = ProvenanceDependency::default();
        for component in &self.components {
            aggregate.union(&component.dependency);
        }
        aggregate
    }

    pub(crate) fn selected(
        &self,
        selector: DatumSelector,
    ) -> ProvenanceResult<ProvenanceDependency> {
        self.components
            .iter()
            .find(|component| component.selector == selector)
            .map(|component| component.dependency.clone())
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn component_mut(&mut self, selector: DatumSelector) -> Option<&mut DatumDependencies> {
        self.components
            .iter_mut()
            .find(|component| component.selector == selector)
    }

    fn union_value(&mut self, other: &Self) -> ProvenanceResult<bool> {
        let same_shape = self
            .components
            .iter()
            .map(|component| component.selector)
            .eq(other.components.iter().map(|component| component.selector));
        if !same_shape {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        let mut changed = false;
        for (target, source) in self.components.iter_mut().zip(&other.components) {
            changed |= target.dependency.union(&source.dependency);
        }
        Ok(changed)
    }
}

/// Retained dependency metadata for one concrete function instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionDependencies {
    pub(crate) function: FunctionId,
    /// Dense [`BindingId`] order. `None` is an unused dense slot.
    pub(crate) bindings: Vec<Option<ValueDependencies>>,
    /// Whole resolved storage roots only, in dense [`BindingId`] order.
    pub(crate) storage_roots: Vec<ProvenanceDependency>,
    pub(crate) result: ValueDependencies,
    /// One aggregate content-write dependency per declared parameter.
    pub(crate) writes: Vec<ProvenanceDependency>,
}

/// Exact checked occurrence of one concrete ordered requirement set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RequirementOccurrence {
    pub(crate) function: FunctionId,
    pub(crate) clauses: Vec<NodePath>,
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
    pub(crate) boundaries: Vec<DemandBoundary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectDemand {
    pub(crate) function: FunctionId,
    pub(crate) subject: ParameterDatum,
    pub(crate) leaf: ProtectedLeaf,
    pub(crate) boundaries: Vec<DemandBoundary>,
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
    pub(crate) caller_dependency: ProvenanceDependency,
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProvenanceDemandKind {
    Direct,
    Bridge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvenanceTarget {
    pub(crate) kind: ProvenanceDemandKind,
    pub(crate) callee_subject: ParameterDatum,
    pub(crate) requirement: Option<RequirementOccurrence>,
    pub(crate) leaf: ProtectedLeaf,
    /// Nonpropagated companion datums beside the terminating true bit.
    pub(crate) companions: ParameterDependencies,
    /// Complete shortest PRV-1 carrier suffix from the rejecting actual to
    /// its labelled-entry or system origin.
    pub(crate) boundaries: Vec<DemandBoundary>,
    pub(crate) carrier: CarrierRoute,
}

/// One coalesced PRV-2 event per accepted call and argument ordinal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvenanceCallEvent {
    pub(crate) caller: FunctionId,
    pub(crate) call: NodePath,
    pub(crate) argument: u32,
    pub(crate) argument_node: NodePath,
    pub(crate) targets: Vec<ProvenanceTarget>,
    pub(crate) selected_target: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProvenanceGoalObservation {
    NotApplicable,
    Evaluated(BridgeGoalView),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallArgumentProvenanceDisposition {
    NoEvent,
}

/// Explicit success-side disposition for every argument of every accepted
/// ordinary call. Absence is never used to mean success.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallArgumentDisposition {
    pub(crate) caller: FunctionId,
    pub(crate) call: NodePath,
    pub(crate) argument: u32,
    pub(crate) argument_node: NodePath,
    pub(crate) complete: ProvenanceGoalObservation,
    pub(crate) unasserted: ProvenanceGoalObservation,
    pub(crate) s4_blinded: ProvenanceGoalObservation,
    pub(crate) disposition: CallArgumentProvenanceDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalLeafProvenanceDisposition {
    BlindedDischarged,
    Internal,
    DirectDemand,
    RequirementBridge,
}

/// Explicit success-side disposition for every complete-state-discharged
/// protected leaf. Rejecting PRV-3 witnesses remain outside this table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalLeafDisposition {
    pub(crate) leaf: ProtectedLeaf,
    pub(crate) complete_discharged: bool,
    pub(crate) unasserted_discharged: bool,
    pub(crate) s4_blinded_discharged: bool,
    pub(crate) disposition: LocalLeafProvenanceDisposition,
}

/// Acceptance-bearing PRV scratch plus the success metadata retained by the
/// checked program. Lowering and optimization do not read this table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvenanceMetadata {
    pub(crate) functions: Vec<FunctionDependencies>,
    /// S3-disabled view with the body-entry S4 goal retained.
    pub(crate) unasserted: Vec<super::entailment::FunctionEntailmentView>,
    /// The same view with S4 and its exact L0 projection omitted.
    pub(crate) s4_blinded: Vec<super::entailment::FunctionEntailmentView>,
    pub(crate) structural_bridges: Vec<StructuralBridge>,
    pub(crate) subject_bridges: Vec<SubjectBridge>,
    pub(crate) calls: Vec<BridgeCallLink>,
    pub(crate) direct_demands: Vec<DirectDemand>,
    pub(crate) call_argument_dispositions: Vec<CallArgumentDisposition>,
    pub(crate) local_leaf_dispositions: Vec<LocalLeafDisposition>,
}

/// Failure-atomic PRV scratch. This value is consumed before constructing a
/// checked program and is never lowering authority.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProvenanceFailures {
    pub(crate) local_rejections: Vec<(
        ProtectedLeaf,
        ProvenanceDependency,
        Option<RequirementOccurrence>,
        CarrierRoute,
    )>,
    pub(crate) call_events: Vec<ProvenanceCallEvent>,
}

pub(crate) struct ProvenanceAnalysis {
    pub(crate) metadata: ProvenanceMetadata,
    pub(crate) failures: ProvenanceFailures,
}

/// The converged PRV-1 component pairs frozen before optimistic S12 facts are
/// formed. The second provenance stratum consumes this value without
/// recomputing the component fixed point from candidate entailment metadata.
pub(crate) struct FrozenProvenanceDependencies {
    functions: Vec<FunctionDependencies>,
}

impl ProvenanceAnalysis {
    /// Replaces the temporary pre-finish proof views with the authoritative
    /// remapped views after the optimistic program batch has passed PRV-2/3.
    pub(crate) fn refresh_entailment_views(&mut self, functions: &[CheckedFunction]) {
        self.metadata.unasserted = functions
            .iter()
            .map(|function| function.entailment.unasserted.clone())
            .collect();
        self.metadata.s4_blinded = functions
            .iter()
            .map(|function| function.entailment.s4_blinded.clone())
            .collect();
    }
}

/// Inputs already held by the phase-B semantic inventory.
pub(crate) struct ProvenanceContext<'check> {
    pub(crate) nominals: &'check [CheckedNominal],
    /// The kind-declaring command entry; all of its labelled parameters are
    /// unconditional PRV-1 origins and have no caller-substitutable datum.
    pub(crate) external_entry: Option<FunctionId>,
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

/// Which result components one system operation's [SYS-2] `wf-prov` row
/// classifies as external.
///
/// The row is declaration data, so the classification is data too, named once
/// here rather than spread through the dependency construction below. An
/// extraction lock compares each case against the specification's own cell;
/// before it existed the whole PRV-1 provenance table was hand-transcribed as
/// bare numeric ordinals with nothing checking it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum SystemResultProvenance {
    /// Every component of the result carries an external class: a plain
    /// external result, or both `Ok` and `Err` payloads.
    AllExternal,
    /// `Ok(value:)` depends on the concrete call's `start` actual, while
    /// `Err(error:)` is unconditionally external.
    OkDependent,
    /// The first variant's endpoint payload depends on the concrete call's
    /// `start` actual, the third variant's error payload is external, and all
    /// other payloads (including `ListBytes(entries:)`) are internal.
    EndpointDependent,
    /// No component is external.
    NoneExternal,
}

#[cfg(test)]
impl SystemResultProvenance {
    // Compatibility names for ordinary tests still being migrated on their
    // isolated branch. They are removed when that test slice is integrated.
    #[allow(non_upper_case_globals)]
    pub(super) const ErrorPayloadOnly: Self = Self::OkDependent;
    #[allow(non_upper_case_globals)]
    pub(super) const ReadFailedPayloadOnly: Self = Self::EndpointDependent;
}

/// The `wf-prov` result-component class of each [SYS-2] operation, by its
/// index in `SYSTEM_OPERATIONS`.
///
/// `None` for an index no operation occupies; the caller turns that into a
/// compiler failure, because the checked model cannot normally contain one.
pub(super) const fn system_result_provenance(operation: u8) -> Option<SystemResultProvenance> {
    Some(match operation {
        // args_count, arg_get, host_bytes_len, host_utf8_len, relative_path,
        // open_read, open_directory, open_list, and the candidate open_file:
        // every component of an opened capability or handle comes from
        // outside.
        0 | 1 | 2 | 4 | 6 | 7 | 11 | 12 | 14 => SystemResultProvenance::AllExternal,
        // host_copy_bytes, host_copy_utf8, write_once.
        3 | 5 | 9 => SystemResultProvenance::OkDependent,
        // read_once, and the candidate list_once.
        8 | 13 => SystemResultProvenance::EndpointDependent,
        // exit_status.
        10 => SystemResultProvenance::NoneExternal,
        _ => return None,
    })
}

fn system_result_dependencies(
    operation: u8,
    ty: CheckedType,
    nominals: &[CheckedNominal],
) -> ProvenanceResult<ValueDependencies> {
    let mut value = ValueDependencies::empty(ty, nominals);
    let mut mark = |variant, field| -> ProvenanceResult<()> {
        value
            .component_mut(DatumSelector::EnumPayload { variant, field })
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .dependency
            .unconditional_external = true;
        Ok(())
    };
    match system_result_provenance(operation).ok_or(SemanticCompilerFailure::InvalidResolution)? {
        SystemResultProvenance::AllExternal => {
            for component in &mut value.components {
                component.dependency.unconditional_external = true;
            }
        }
        SystemResultProvenance::OkDependent => mark(1, 0)?,
        SystemResultProvenance::EndpointDependent => mark(2, 0)?,
        SystemResultProvenance::NoneExternal => {}
    }
    Ok(value)
}

fn system_endpoint_start(operation: u8) -> ProvenanceResult<Option<usize>> {
    let row = crate::SYSTEM_OPERATIONS
        .get(usize::from(operation))
        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
    if !matches!(
        system_result_provenance(operation),
        Some(SystemResultProvenance::OkDependent | SystemResultProvenance::EndpointDependent)
    ) {
        return Ok(None);
    }
    Ok(Some(
        row.parameters
            .iter()
            .position(|parameter| parameter.name == "start")
            .ok_or(SemanticCompilerFailure::InvalidResolution)?,
    ))
}

/// The `wf-prov` writable-`&uniq`-parameter column: the parameter ordinals one
/// operation writes with an external class.
pub(super) fn system_external_writes(operation: u8) -> ProvenanceResult<&'static [usize]> {
    Ok(match operation {
        3 | 5 => &[1],
        // read_once, and the candidate list_once: the handle advances and the
        // destination receives host bytes.
        8 | 13 => &[0, 1],
        9 => &[0],
        // The candidate open_file, like every other open, writes no parameter.
        0..=2 | 4 | 6 | 7 | 10..=12 | 14 => &[],
        _ => return Err(SemanticCompilerFailure::InvalidResolution),
    })
}

fn occurrence_cmp(left: &RequirementOccurrence, right: &RequirementOccurrence) -> Ordering {
    left.function.0.cmp(&right.function.0).then_with(|| {
        left.clauses
            .iter()
            .map(NodePath::components)
            .cmp(right.clauses.iter().map(NodePath::components))
    })
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
    (!function.requirements.is_empty()).then(|| RequirementOccurrence {
        function: function.id,
        clauses: function
            .requirements
            .iter()
            .map(|requirement| requirement.clause.clone())
            .collect(),
    })
}

struct FunctionPass<'check> {
    function: &'check CheckedFunction,
    nominals: &'check [CheckedNominal],
    holders: Vec<Option<HolderRoot>>,
    bindings: Vec<Option<ValueDependencies>>,
    roots: Vec<ProvenanceDependency>,
    result: ValueDependencies,
    writes: Vec<ProvenanceDependency>,
}

impl<'check> FunctionPass<'check> {
    fn new(
        function: &'check CheckedFunction,
        nominals: &'check [CheckedNominal],
        entry_external: bool,
    ) -> ProvenanceResult<Self> {
        let slots = binding_slot_count(function);
        let mut pass = Self {
            function,
            nominals,
            holders: vec![None; slots],
            bindings: vec![None; slots],
            roots: vec![ProvenanceDependency::default(); slots],
            result: ValueDependencies::empty(function.result, nominals),
            writes: vec![ProvenanceDependency::default(); function.parameters.len()],
        };
        pass.collect_holders(&function.body)?;
        for (ordinal, parameter) in function.parameters.iter().enumerate() {
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let value = if entry_external {
                ValueDependencies::external(parameter.ty, nominals)
            } else {
                ValueDependencies::parameter(ordinal, parameter.ty, nominals)
            };
            pass.set_binding(parameter.binding, parameter.ty, &value)?;
            if !matches!(parameter.mode, CheckedMode::Own) {
                pass.set_holder(parameter.binding, HolderRoot::Opaque)?;
            }
        }
        Ok(pass)
    }

    fn from_metadata(
        function: &'check CheckedFunction,
        nominals: &'check [CheckedNominal],
        metadata: &FunctionDependencies,
    ) -> ProvenanceResult<Self> {
        let mut pass = Self::new(function, nominals, false)?;
        pass.bindings = metadata.bindings.clone();
        pass.roots = metadata.storage_roots.clone();
        pass.result = metadata.result.clone();
        pass.writes = metadata.writes.clone();
        Ok(pass)
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

    fn set_holder(&mut self, binding: BindingId, holder: HolderRoot) -> ProvenanceResult<()> {
        let slot = self
            .holders
            .get_mut(binding.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        *slot = Some(holder);
        Ok(())
    }

    fn match_holder(scrutinee: &CheckedExpression) -> Option<HolderRoot> {
        match scrutinee {
            CheckedExpression::Binding { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. }
            | CheckedExpression::Project { binding, .. } => Some(HolderRoot::Holder(*binding)),
            CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => Self::match_holder(value),
            _ => None,
        }
    }

    fn collect_holders(&mut self, statements: &[CheckedStatement]) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let holder = match value {
                        CheckedExpression::BorrowAddressed { binding, .. }
                        | CheckedExpression::BorrowBox { binding, .. }
                        | CheckedExpression::BorrowSystemResource { binding, .. } => {
                            Some(HolderRoot::Place(*binding))
                        }
                        CheckedExpression::BorrowBuffer { root, .. } => {
                            Some(HolderRoot::Place(root.binding))
                        }
                        CheckedExpression::ReborrowAddressed { binding, .. } => {
                            Some(HolderRoot::Holder(*binding))
                        }
                        // A bound borrow-mode call result is a holder over
                        // the provenance-candidate actual's storage root
                        // [OWN-6]; provenance retains the whole root exactly
                        // as it does for a matched holder's payload binder.
                        CheckedExpression::UserCall {
                            result_borrow: Some(result_borrow),
                            ..
                        } => Some(HolderRoot::Place(result_borrow.binding)),
                        CheckedExpression::BoxNew { .. } | CheckedExpression::ArenaNew { .. } => {
                            Some(HolderRoot::Opaque)
                        }
                        _ => None,
                    };
                    if let Some(holder) = holder {
                        self.set_holder(*binding, holder)?;
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
                                self.set_holder(binder.binding, holder.clone())?;
                            }
                        }
                        self.collect_holders(&arm.body)?;
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => self.collect_holders(body)?,
                CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn resolve_root(&self, binding: BindingId) -> ProvenanceResult<BindingId> {
        let mut current = binding;
        for _ in 0..=self.holders.len() {
            let holder = self
                .holders
                .get(current.0 as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let next = match holder.as_ref() {
                Some(HolderRoot::Place(root) | HolderRoot::Holder(root)) => *root,
                Some(HolderRoot::Opaque) | None => return Ok(current),
            };
            if next == current {
                return Ok(current);
            }
            current = next;
        }
        Err(SemanticCompilerFailure::InvalidResolution)
    }

    fn binding(&self, binding: BindingId, ty: CheckedType) -> ProvenanceResult<ValueDependencies> {
        let value = self
            .bindings
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let expected = ValueDependencies::empty(ty, self.nominals);
        if !value
            .components
            .iter()
            .map(|component| component.selector)
            .eq(expected
                .components
                .iter()
                .map(|component| component.selector))
        {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        Ok(value.clone())
    }

    fn resolved_binding(
        &self,
        binding: BindingId,
        ty: CheckedType,
    ) -> ProvenanceResult<ValueDependencies> {
        // Holder resolution identifies the storage affected by writes. The
        // holder binding itself retains the exact value component selected by
        // a borrow or match binder, and every fixed-point rescan refreshes it
        // after a root write. Reading the resolved root here would replace a
        // borrowed payload with its enclosing aggregate shape.
        self.binding(binding, ty)
    }

    fn root(&self, binding: BindingId) -> ProvenanceResult<ProvenanceDependency> {
        let resolved = self.resolve_root(binding)?;
        self.roots
            .get(resolved.0 as usize)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn set_binding(
        &mut self,
        binding: BindingId,
        ty: CheckedType,
        value: &ValueDependencies,
    ) -> ProvenanceResult<bool> {
        let index = binding.0 as usize;
        let mut changed = false;
        let slot = self
            .bindings
            .get_mut(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let target = slot.get_or_insert_with(|| ValueDependencies::empty(ty, self.nominals));
        changed |= target.union_value(value)?;
        let root = self
            .roots
            .get_mut(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        changed |= root.union(&value.aggregate());
        Ok(changed)
    }

    fn add_root_write(
        &mut self,
        binding: BindingId,
        dependencies: &ProvenanceDependency,
        seed_every_value_component: bool,
    ) -> ProvenanceResult<bool> {
        let resolved = self.resolve_root(binding)?;
        let index = resolved.0 as usize;
        let mut changed = self
            .roots
            .get_mut(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .union(dependencies);
        if seed_every_value_component {
            let value = self
                .bindings
                .get_mut(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .as_mut()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for component in &mut value.components {
                changed |= component.dependency.union(dependencies);
            }
        }
        for (ordinal, parameter) in self.function.parameters.iter().enumerate() {
            if matches!(parameter.mode, CheckedMode::Unique(_))
                && self.resolve_root(parameter.binding)? == resolved
            {
                let write = self
                    .writes
                    .get_mut(ordinal)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                changed |= write.union(dependencies);
            }
        }
        Ok(changed)
    }

    fn set_seeds_every_value_component(&self, target: &CheckedSetTarget) -> ProvenanceResult<bool> {
        let binding = target.binding();
        let whole_place =
            matches!(target, CheckedSetTarget::Place(place) if place.fields.is_empty());
        // A direct whole-owner assignment has an exact componentwise value
        // transfer below. A projected/element write, or a whole-place write
        // through an alias whose apparent whole is only a child of the
        // resolved owner, conservatively widens every owner component.
        Ok(!whole_place || self.resolve_root(binding)? != binding)
    }

    fn scan_until_stable(&mut self, summaries: &[FunctionDependencies]) -> ProvenanceResult<()> {
        loop {
            let before = (
                self.bindings.clone(),
                self.roots.clone(),
                self.result.clone(),
                self.writes.clone(),
            );
            self.scan_block(&self.function.body, summaries, None)?;
            let after = (&self.bindings, &self.roots, &self.result, &self.writes);
            if before.0 == *after.0
                && before.1 == *after.1
                && before.2 == *after.2
                && before.3 == *after.3
            {
                break;
            }
        }
        Ok(())
    }

    fn scan_block(
        &mut self,
        statements: &[CheckedStatement],
        summaries: &[FunctionDependencies],
        mut gives: Option<&mut ValueDependencies>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { binding, value, .. } => {
                    let dependencies = self.expression(value, summaries)?;
                    self.set_binding(*binding, value.ty(), &dependencies)?;
                }
                CheckedStatement::PropagateLet {
                    binding,
                    scrutinee,
                    ok_type,
                    ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries)?;
                    let ok = scrutinee.selected(DatumSelector::EnumPayload {
                        variant: 0,
                        field: 0,
                    })?;
                    let ok = ValueDependencies::from_aggregate(*ok_type, &ok, self.nominals);
                    self.set_binding(*binding, *ok_type, &ok)?;
                    let error = scrutinee.selected(DatumSelector::EnumPayload {
                        variant: 1,
                        field: 0,
                    })?;
                    let component = self
                        .result
                        .component_mut(DatumSelector::EnumPayload {
                            variant: 1,
                            field: 0,
                        })
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    component.dependency.union(&error);
                }
                CheckedStatement::Set { target, value, .. } => {
                    self.scan_set_target(target, summaries)?;
                    let value = self.expression(value, summaries)?;
                    let aggregate = value.aggregate();
                    let root = target.binding();
                    let seed_every_value_component =
                        self.set_seeds_every_value_component(target)?;
                    self.add_root_write(root, &aggregate, seed_every_value_component)?;
                    if let CheckedSetTarget::Place(place) = target
                        && place.fields.is_empty()
                    {
                        let resolved = self.resolve_root(place.binding)?;
                        if resolved == place.binding {
                            self.set_binding(resolved, place.ty, &value)?;
                        }
                    }
                }
                CheckedStatement::Replace {
                    binding,
                    target,
                    value,
                    ..
                } => {
                    // [SET-2]: the write half is exactly a Set commit's
                    // dependency flow; the fresh binding additionally owns
                    // the target's previous value, represented conservatively
                    // by the complete resolved-root dependency (union-only,
                    // so the fixed point stays monotone and fail-closed).
                    self.scan_set_target(target, summaries)?;
                    let value = self.expression(value, summaries)?;
                    let aggregate = value.aggregate();
                    let root = target.binding();
                    let seed_every_value_component =
                        self.set_seeds_every_value_component(target)?;
                    self.add_root_write(root, &aggregate, seed_every_value_component)?;
                    if let CheckedSetTarget::Place(place) = target
                        && place.fields.is_empty()
                    {
                        let resolved = self.resolve_root(place.binding)?;
                        if resolved == place.binding {
                            self.set_binding(resolved, place.ty, &value)?;
                        }
                    }
                    let previous = self.root(root)?;
                    let previous =
                        ValueDependencies::from_aggregate(target.ty(), &previous, self.nominals);
                    self.set_binding(*binding, target.ty(), &previous)?;
                }
                CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. } => {
                    self.expression(value, summaries)?;
                }
                CheckedStatement::Claim { condition, .. } => {
                    self.expression(condition, summaries)?;
                }
                CheckedStatement::Return { value, .. } => {
                    let value = self.expression(value, summaries)?;
                    self.result.union_value(&value)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries)?;
                    for arm in arms {
                        self.seed_arm_binders(arm, &scrutinee)?;
                        self.scan_block(&arm.body, summaries, gives.as_deref_mut())?;
                    }
                }
                CheckedStatement::ValueMatchLet {
                    binding,
                    result_type,
                    scrutinee,
                    arms,
                    ..
                } => {
                    let scrutinee = self.expression(scrutinee, summaries)?;
                    let mut delivered = ValueDependencies::empty(*result_type, self.nominals);
                    for arm in arms {
                        self.seed_arm_binders(arm, &scrutinee)?;
                        self.scan_block(&arm.body, summaries, Some(&mut delivered))?;
                    }
                    self.set_binding(*binding, *result_type, &delivered)?;
                }
                CheckedStatement::Give { value, .. } => {
                    let value = self.expression(value, summaries)?;
                    let target = gives
                        .as_deref_mut()
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    target.union_value(&value)?;
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.scan_block(body, summaries, gives.as_deref_mut())?;
                }
                CheckedStatement::CountedRange {
                    binder,
                    lower,
                    upper,
                    body,
                    ..
                } => {
                    let lower = self.expression(lower, summaries)?;
                    self.expression(upper, summaries)?;
                    self.set_binding(
                        *binder,
                        CheckedType::Integer(super::model::IntegerType::U64),
                        &lower,
                    )?;
                    self.scan_block(body, summaries, gives.as_deref_mut())?;
                }
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn seed_arm_binders(
        &mut self,
        arm: &CheckedMatchArm,
        scrutinee: &ValueDependencies,
    ) -> ProvenanceResult<()> {
        for binder in &arm.binders {
            let selected = scrutinee.selected(DatumSelector::EnumPayload {
                variant: arm.tag,
                field: binder.field,
            })?;
            let value = ValueDependencies::from_aggregate(binder.ty, &selected, self.nominals);
            self.set_binding(binder.binding, binder.ty, &value)?;
        }
        Ok(())
    }

    fn scan_set_target(
        &mut self,
        target: &CheckedSetTarget,
        summaries: &[FunctionDependencies],
    ) -> ProvenanceResult<()> {
        match target {
            CheckedSetTarget::Place(_) => {}
            CheckedSetTarget::ArrayIndex(target) => {
                self.expression(&target.offset, summaries)?;
            }
            CheckedSetTarget::BufferIndex(target) => {
                self.expression(&target.offset, summaries)?;
            }
        }
        Ok(())
    }

    fn expression(
        &mut self,
        expression: &CheckedExpression,
        summaries: &[FunctionDependencies],
    ) -> ProvenanceResult<ValueDependencies> {
        Ok(match expression {
            CheckedExpression::Constant(_) | CheckedExpression::NamedConstant { .. } => {
                ValueDependencies::empty(expression.ty(), self.nominals)
            }
            CheckedExpression::Binding { binding, ty, .. } => self.binding(*binding, *ty)?,
            CheckedExpression::UserCall {
                function,
                arguments,
                ..
            } => {
                let actuals = arguments
                    .iter()
                    .map(|argument| self.expression(argument, summaries))
                    .collect::<ProvenanceResult<Vec<_>>>()?;
                let callee = summaries
                    .get(function.0 as usize)
                    .filter(|candidate| candidate.function == *function)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if callee.writes.len() != arguments.len() {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                }
                for (ordinal, dependencies) in callee.writes.iter().enumerate() {
                    if !dependencies.unconditional_external
                        && dependencies.parameters.datums.is_empty()
                    {
                        continue;
                    }
                    let substituted = substitute_dependency(dependencies, &actuals)?;
                    let argument = arguments
                        .get(ordinal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let root = self
                        .argument_root(argument)?
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    self.add_root_write(root, &substituted, true)?;
                }
                substitute_value(&callee.result, &actuals)?
            }
            CheckedExpression::SystemCall {
                operation,
                arguments,
                result,
                ..
            } => {
                let actuals = arguments
                    .iter()
                    .map(|argument| self.expression(argument, summaries))
                    .collect::<ProvenanceResult<Vec<_>>>()?;
                let external = ProvenanceDependency::external();
                for ordinal in system_external_writes(*operation)? {
                    let argument = arguments
                        .get(*ordinal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let root = self
                        .argument_root(argument)?
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    self.add_root_write(root, &external, true)?;
                }
                let mut value = system_result_dependencies(*operation, *result, self.nominals)?;
                if let Some(start) = system_endpoint_start(*operation)? {
                    let dependency = actuals
                        .get(start)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .aggregate();
                    value
                        .component_mut(DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        })
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .dependency
                        .union(&dependency);
                }
                value
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                result,
                ..
            } => {
                let operands = self.expression_aggregate(arguments, summaries)?;
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
                    let ok = value
                        .component_mut(DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        })
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    ok.dependency.union(&operands);
                } else {
                    for component in &mut value.components {
                        component.dependency.union(&operands);
                    }
                }
                value
            }
            CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. } => {
                let aggregate = self.expression_aggregate(arguments, summaries)?;
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                result,
                ..
            } => {
                let operand = self.expression(value, summaries)?.aggregate();
                let mut converted = ValueDependencies::empty(*result, self.nominals);
                if source.converts_totally_to(*destination) {
                    for component in &mut converted.components {
                        component.dependency.union(&operand);
                    }
                } else {
                    let ok = converted
                        .component_mut(DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        })
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    ok.dependency.union(&operand);
                }
                converted
            }
            CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaNew { value, .. }
            | CheckedExpression::ArenaDeref { value, .. } => {
                let value = self.expression(value, summaries)?;
                let aggregate = value.aggregate();
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ArrayFill { value, .. } => {
                let aggregate = self.expression(value, summaries)?.aggregate();
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceLength { .. } => {
                ValueDependencies::empty(expression.ty(), self.nominals)
            }
            CheckedExpression::ArrayIndex { root, offset, .. } => {
                let mut aggregate = self.array_root(root)?;
                aggregate.union(&self.expression(offset, summaries)?.aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                let mut aggregate = self.expression(length, summaries)?.aggregate();
                aggregate.union(&self.expression(value, summaries)?.aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
                let aggregate = self.expression(length, summaries)?.aggregate();
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let mut aggregate = self.root(root.binding)?;
                aggregate.union(&self.expression(offset, summaries)?.aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::SliceOf { source, .. } => {
                let aggregate = match source {
                    CheckedSliceSource::Array { root, .. } => self.array_root(root)?,
                    CheckedSliceSource::Buffer(root) => self.root(root.binding)?,
                    CheckedSliceSource::ArenaContent { binding, .. } => self.root(*binding)?,
                };
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let mut aggregate = self.root(root.binding)?;
                aggregate.union(&self.expression(offset, summaries)?.aggregate());
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BorrowBuffer { root, .. } => {
                let aggregate = self.root(root.binding)?;
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => {
                self.resolved_binding(*binding, expression.ty())?
            }
            CheckedExpression::ConstructStruct { fields, .. } => {
                let aggregate = self.expression_aggregate(fields, summaries)?;
                ValueDependencies::from_aggregate(expression.ty(), &aggregate, self.nominals)
            }
            CheckedExpression::ConstructEnum {
                variant, fields, ..
            } => {
                let mut value = ValueDependencies::empty(expression.ty(), self.nominals);
                for (field, expression) in fields.iter().enumerate() {
                    let field = u32::try_from(field)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                    let dependencies = self.expression(expression, summaries)?.aggregate();
                    let component = value
                        .component_mut(DatumSelector::EnumPayload {
                            variant: *variant,
                            field,
                        })
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    component.dependency.union(&dependencies);
                }
                value
            }
            CheckedExpression::Project { binding, ty, .. } => {
                let aggregate = self.root(*binding)?;
                ValueDependencies::from_aggregate(*ty, &aggregate, self.nominals)
            }
            CheckedExpression::ProjectValue { value, ty, .. } => {
                let aggregate = self.expression(value, summaries)?.aggregate();
                ValueDependencies::from_aggregate(*ty, &aggregate, self.nominals)
            }
        })
    }

    fn expression_aggregate(
        &mut self,
        expressions: &[CheckedExpression],
        summaries: &[FunctionDependencies],
    ) -> ProvenanceResult<ProvenanceDependency> {
        let mut aggregate = ProvenanceDependency::default();
        for expression in expressions {
            aggregate.union(&self.expression(expression, summaries)?.aggregate());
        }
        Ok(aggregate)
    }

    fn array_root(&self, root: &CheckedArrayRoot) -> ProvenanceResult<ProvenanceDependency> {
        Ok(match root {
            CheckedArrayRoot::Binding { binding, .. } => self.root(*binding)?,
            CheckedArrayRoot::Constant(_) => ProvenanceDependency::default(),
        })
    }

    fn argument_root(&self, argument: &CheckedExpression) -> ProvenanceResult<Option<BindingId>> {
        let binding = match argument {
            CheckedExpression::Binding { binding, .. } => *binding,
            CheckedExpression::BorrowBuffer { root, .. } => root.binding,
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. } => *binding,
            _ => return Ok(None),
        };
        Ok(Some(self.resolve_root(binding)?))
    }
}

fn substitute_dependency(
    dependencies: &ProvenanceDependency,
    actuals: &[ValueDependencies],
) -> ProvenanceResult<ProvenanceDependency> {
    let mut substituted = ProvenanceDependency {
        unconditional_external: dependencies.unconditional_external,
        parameters: ParameterDependencies::default(),
    };
    for datum in &dependencies.parameters.datums {
        let actual = actuals
            .get(datum.ordinal as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        substituted.union(&actual.selected(datum.selector)?);
    }
    Ok(substituted)
}

fn substitute_value(
    value: &ValueDependencies,
    actuals: &[ValueDependencies],
) -> ProvenanceResult<ValueDependencies> {
    Ok(ValueDependencies {
        components: value
            .components
            .iter()
            .map(|component| {
                Ok(DatumDependencies {
                    selector: component.selector,
                    dependency: substitute_dependency(&component.dependency, actuals)?,
                })
            })
            .collect::<ProvenanceResult<Vec<_>>>()?,
    })
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
            | CheckedStatement::PropagateLet { binding, .. }
            | CheckedStatement::Replace { binding, .. } => {
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
    external_entry: Option<FunctionId>,
) -> ProvenanceResult<Vec<FunctionDependencies>> {
    let mut summaries = functions
        .iter()
        .map(|function| {
            Ok(
                FunctionPass::new(function, nominals, external_entry == Some(function.id))?
                    .metadata(),
            )
        })
        .collect::<ProvenanceResult<Vec<_>>>()?;
    loop {
        let previous = summaries.clone();
        for function in functions {
            let mut pass =
                FunctionPass::new(function, nominals, external_entry == Some(function.id))?;
            pass.scan_until_stable(&previous)?;
            let derived = pass.metadata();
            let summary = summaries
                .get_mut(function.id.0 as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            summary.result.union_value(&derived.result)?;
            if summary.writes.len() != derived.writes.len() {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
            for (target, source) in summary.writes.iter_mut().zip(&derived.writes) {
                target.union(source);
            }
        }
        if summaries == previous {
            break;
        }
    }
    functions
        .iter()
        .map(|function| {
            let mut pass =
                FunctionPass::new(function, nominals, external_entry == Some(function.id))?;
            pass.scan_until_stable(&summaries)?;
            Ok(pass.metadata())
        })
        .collect()
}

#[derive(Clone)]
struct LeafSite {
    leaf: ProtectedLeaf,
    subjects: Vec<CheckedExpression>,
}

#[derive(Clone)]
struct CallSite {
    caller: FunctionId,
    call: NodePath,
    downstream_requirement: RequirementOccurrence,
    argument_nodes: Vec<NodePath>,
    arguments: Vec<CheckedExpression>,
}

/// Every accepted user call participates in direct-demand composition,
/// including callees with no FN-8 requirement.
#[derive(Clone)]
struct DirectCallSite {
    caller: FunctionId,
    callee: FunctionId,
    call: NodePath,
    argument_nodes: Vec<NodePath>,
    arguments: Vec<CheckedExpression>,
    has_requirement: bool,
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

/// One caller-visible direct demand state.  `function` is the current
/// boundary owner; `leaf` remains the concrete downstream protected site.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectKey {
    function: FunctionId,
    subject: ParameterDatum,
    leaf: ProtectedLeaf,
}

/// One complete second-stratum demand-state identity.  Keeping direct and
/// exact requirement-bridge states distinct is what cuts recursive witness
/// cycles without collapsing their normative identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DemandState {
    Direct {
        function: FunctionId,
        subject: ParameterDatum,
        leaf: ProtectedLeaf,
    },
    Bridge {
        requirement: RequirementOccurrence,
        subject: ParameterDatum,
        leaf: ProtectedLeaf,
    },
}

impl DemandState {
    fn direct(key: &DirectKey) -> Self {
        Self::Direct {
            function: key.function,
            subject: key.subject,
            leaf: key.leaf.clone(),
        }
    }

    fn bridge(key: &SubjectKey) -> Self {
        Self::Bridge {
            requirement: key.requirement.clone(),
            subject: key.subject,
            leaf: key.leaf.clone(),
        }
    }
}

/// One complete call boundary in a shortest post-convergence demand route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DemandBoundary {
    pub(crate) call: NodePath,
    pub(crate) argument_node: NodePath,
    pub(crate) argument: u32,
    pub(crate) callee: DemandState,
    pub(crate) caller_continuation: Option<DemandState>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DemandRoute {
    boundaries: Vec<DemandBoundary>,
}

impl DemandRoute {
    fn append(&self, boundary: DemandBoundary) -> Self {
        let mut boundaries = self.boundaries.clone();
        boundaries.push(boundary);
        Self { boundaries }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalGateCandidate {
    leaf: ProtectedLeaf,
    subject: ProvenanceDependency,
    entry_requirement: Option<RequirementOccurrence>,
    carrier: CarrierRoute,
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

#[derive(Clone)]
struct DirectCallInventory {
    site: DirectCallSite,
    actuals: Vec<ValueDependencies>,
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

fn direct_key_cmp(left: &DirectKey, right: &DirectKey) -> Ordering {
    left.function
        .0
        .cmp(&right.function.0)
        .then_with(|| left.subject.cmp(&right.subject))
        .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
}

fn demand_state_route_cmp(left: &DemandState, right: &DemandState) -> Ordering {
    match (left, right) {
        (DemandState::Direct { .. }, DemandState::Bridge { .. }) => Ordering::Less,
        (DemandState::Bridge { .. }, DemandState::Direct { .. }) => Ordering::Greater,
        (
            DemandState::Direct {
                function: left_function,
                subject: left_subject,
                ..
            },
            DemandState::Direct {
                function: right_function,
                subject: right_subject,
                ..
            },
        ) => left_subject
            .cmp(right_subject)
            .then_with(|| left_function.0.cmp(&right_function.0)),
        (
            DemandState::Bridge {
                requirement: left_requirement,
                subject: left_subject,
                ..
            },
            DemandState::Bridge {
                requirement: right_requirement,
                subject: right_subject,
                ..
            },
        ) => left_requirement
            .clauses
            .iter()
            .map(NodePath::components)
            .cmp(right_requirement.clauses.iter().map(NodePath::components))
            .then_with(|| left_subject.cmp(right_subject))
            .then_with(|| {
                left_requirement
                    .function
                    .0
                    .cmp(&right_requirement.function.0)
            }),
    }
}

fn demand_boundary_cmp(left: &DemandBoundary, right: &DemandBoundary) -> Ordering {
    left.call
        .components()
        .cmp(right.call.components())
        .then_with(|| {
            left.argument_node
                .components()
                .cmp(right.argument_node.components())
        })
        .then_with(|| demand_state_route_cmp(&left.callee, &right.callee))
        .then_with(
            || match (&left.caller_continuation, &right.caller_continuation) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(left), Some(right)) => demand_state_route_cmp(left, right),
            },
        )
}

fn demand_route_cmp(left: &DemandRoute, right: &DemandRoute) -> Ordering {
    demand_boundaries_cmp(&left.boundaries, &right.boundaries)
}

fn demand_boundaries_cmp(left: &[DemandBoundary], right: &[DemandBoundary]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| {
        left.iter()
            .zip(right)
            .find_map(|(left, right)| {
                let ordering = demand_boundary_cmp(left, right);
                (ordering != Ordering::Equal).then_some(ordering)
            })
            .unwrap_or(Ordering::Equal)
    })
}

fn choose_demand_route(current: &mut Option<DemandRoute>, candidate: DemandRoute) -> bool {
    if current
        .as_ref()
        .is_none_or(|existing| demand_route_cmp(&candidate, existing) == Ordering::Less)
    {
        *current = Some(candidate);
        true
    } else {
        false
    }
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

fn insert_direct(keys: &mut Vec<DirectKey>, key: DirectKey) -> bool {
    match keys.binary_search_by(|candidate| direct_key_cmp(candidate, &key)) {
        Ok(_) => false,
        Err(index) => {
            keys.insert(index, key);
            true
        }
    }
}

fn collect_sites(
    function: &CheckedFunction,
) -> (Vec<LeafSite>, Vec<CallSite>, Vec<DirectCallSite>) {
    let mut leaves = Vec::new();
    let mut calls = Vec::new();
    let mut direct_calls = Vec::new();
    collect_block_sites(
        function.id,
        &function.body,
        &mut leaves,
        &mut calls,
        &mut direct_calls,
    );
    (leaves, calls, direct_calls)
}

fn collect_block_sites(
    function: FunctionId,
    statements: &[CheckedStatement],
    leaves: &mut Vec<LeafSite>,
    calls: &mut Vec<CallSite>,
    direct_calls: &mut Vec<DirectCallSite>,
) {
    for statement in statements {
        match statement {
            CheckedStatement::Let { value, .. }
            | CheckedStatement::Evaluate(value)
            | CheckedStatement::DropExpression { value, .. }
            | CheckedStatement::Claim {
                condition: value, ..
            }
            | CheckedStatement::Return { value, .. }
            | CheckedStatement::Give { value, .. } => {
                collect_expression_sites(function, value, leaves, calls, direct_calls);
            }
            CheckedStatement::PropagateLet { scrutinee, .. }
            | CheckedStatement::Match { scrutinee, .. }
            | CheckedStatement::ValueMatchLet { scrutinee, .. } => {
                collect_expression_sites(function, scrutinee, leaves, calls, direct_calls);
                if let CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } = statement
                {
                    for arm in arms {
                        collect_block_sites(function, &arm.body, leaves, calls, direct_calls);
                    }
                }
            }
            CheckedStatement::Set { target, value, .. }
            | CheckedStatement::Replace { target, value, .. } => {
                match target {
                    CheckedSetTarget::Place(_) => {}
                    CheckedSetTarget::ArrayIndex(target) => {
                        collect_expression_sites(
                            function,
                            &target.offset,
                            leaves,
                            calls,
                            direct_calls,
                        );
                        leaves.push(LeafSite {
                            leaf: ProtectedLeaf {
                                function,
                                obligation: target.trap.node_path.clone(),
                                conjunct: 0,
                            },
                            subjects: vec![target.offset.clone()],
                        });
                    }
                    CheckedSetTarget::BufferIndex(target) => {
                        collect_expression_sites(
                            function,
                            &target.offset,
                            leaves,
                            calls,
                            direct_calls,
                        );
                        leaves.push(LeafSite {
                            leaf: ProtectedLeaf {
                                function,
                                obligation: target.trap.node_path.clone(),
                                conjunct: 0,
                            },
                            subjects: vec![target.offset.clone()],
                        });
                    }
                }
                collect_expression_sites(function, value, leaves, calls, direct_calls);
            }
            CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                collect_block_sites(function, body, leaves, calls, direct_calls);
            }
            CheckedStatement::CountedRange {
                lower, upper, body, ..
            } => {
                collect_expression_sites(function, lower, leaves, calls, direct_calls);
                collect_expression_sites(function, upper, leaves, calls, direct_calls);
                collect_block_sites(function, body, leaves, calls, direct_calls);
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
    direct_calls: &mut Vec<DirectCallSite>,
) {
    match expression {
        CheckedExpression::UserCall {
            function: callee,
            call,
            argument_nodes,
            arguments,
            requirements,
            ..
        } => {
            for argument in arguments {
                collect_expression_sites(function, argument, leaves, calls, direct_calls);
            }
            direct_calls.push(DirectCallSite {
                caller: function,
                callee: *callee,
                call: call.clone(),
                argument_nodes: argument_nodes.clone(),
                arguments: arguments.clone(),
                has_requirement: !requirements.is_empty(),
            });
            if !requirements.is_empty() {
                calls.push(CallSite {
                    caller: function,
                    call: call.clone(),
                    downstream_requirement: RequirementOccurrence {
                        function: *callee,
                        clauses: requirements
                            .iter()
                            .map(|requirement| requirement.requires_clause.clone())
                            .collect(),
                    },
                    argument_nodes: argument_nodes.clone(),
                    arguments: arguments.clone(),
                });
            }
        }
        CheckedExpression::SystemCall {
            operation,
            call,
            arguments,
            ..
        } => {
            for argument in arguments {
                collect_expression_sites(function, argument, leaves, calls, direct_calls);
            }
            let Some(row) = crate::SYSTEM_OPERATIONS.get(usize::from(*operation)) else {
                return;
            };
            let (Some(start), Some(end)) = (
                row.parameters
                    .iter()
                    .position(|parameter| parameter.name == "start"),
                row.parameters
                    .iter()
                    .position(|parameter| parameter.name == "end"),
            ) else {
                return;
            };
            let (Some(start), Some(end)) = (arguments.get(start), arguments.get(end)) else {
                return;
            };
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: call.clone(),
                    conjunct: 0,
                },
                subjects: vec![start.clone(), end.clone()],
            });
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: call.clone(),
                    conjunct: 1,
                },
                subjects: vec![end.clone()],
            });
        }
        CheckedExpression::BufferFill {
            carrier,
            length,
            value,
            ..
        } => {
            collect_expression_sites(function, length, leaves, calls, direct_calls);
            collect_expression_sites(function, value, leaves, calls, direct_calls);
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: carrier.clone(),
                    conjunct: 0,
                },
                subjects: vec![(**length).clone()],
            });
        }
        CheckedExpression::BufferVacant {
            carrier, length, ..
        } => {
            collect_expression_sites(function, length, leaves, calls, direct_calls);
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: carrier.clone(),
                    conjunct: 0,
                },
                subjects: vec![(**length).clone()],
            });
        }
        CheckedExpression::ArrayIndex { offset, trap, .. }
        | CheckedExpression::BufferIndex { offset, trap, .. }
        | CheckedExpression::SliceIndex { offset, trap, .. } => {
            collect_expression_sites(function, offset, leaves, calls, direct_calls);
            leaves.push(LeafSite {
                leaf: ProtectedLeaf {
                    function,
                    obligation: trap.node_path.clone(),
                    conjunct: 0,
                },
                subjects: vec![(**offset).clone()],
            });
        }
        _ => {
            for child in expression_children(expression) {
                collect_expression_sites(function, child, leaves, calls, direct_calls);
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
        | CheckedExpression::ArenaNew { value, .. }
        | CheckedExpression::ArenaDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value],
        CheckedExpression::ArrayIndex { offset, .. }
        | CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset],
        CheckedExpression::BufferFill { length, value, .. } => vec![length, value],
        CheckedExpression::BufferVacant { length, .. }
        | CheckedExpression::BufferFits { length, .. } => vec![length.as_ref()],
    }
}

/// One post-convergence PRV-1 carrier edge.  Paths and selector tags remain
/// outside the component lattice and participate only in deterministic
/// witness selection after the unconditional bits have frozen.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CarrierCallRole {
    SystemResult,
    SystemWrite,
    UserResult,
    UserWrite,
    UserSubstitution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CarrierWriteContext {
    /// Exact writable formal ordinal at this call boundary.
    pub(crate) parameter: u32,
    /// Exact caller actual atom for the writable formal.
    pub(crate) actual: NodePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CarrierStep {
    pub(crate) path: NodePath,
    pub(crate) selector: DatumSelector,
    /// Call-edge identity; absent on non-call carriers.
    pub(crate) call_role: Option<CarrierCallRole>,
    /// Explanation-only identity attached to a single system/user call edge.
    /// It does not add another PRV-1 predecessor edge.
    pub(crate) write_context: Option<CarrierWriteContext>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CarrierRoute {
    steps: Vec<CarrierStep>,
}

impl CarrierRoute {
    fn terminal(path: NodePath, selector: DatumSelector) -> Self {
        Self {
            steps: vec![CarrierStep {
                path,
                selector,
                call_role: None,
                write_context: None,
            }],
        }
    }

    pub(crate) fn call_terminal(
        path: NodePath,
        selector: DatumSelector,
        call_role: CarrierCallRole,
        write_context: Option<CarrierWriteContext>,
    ) -> Self {
        Self {
            steps: vec![CarrierStep {
                path,
                selector,
                call_role: Some(call_role),
                write_context,
            }],
        }
    }

    fn prepend(mut self, path: NodePath, selector: DatumSelector) -> Self {
        self.steps.insert(
            0,
            CarrierStep {
                path,
                selector,
                call_role: None,
                write_context: None,
            },
        );
        self
    }

    fn prepend_call(
        mut self,
        path: NodePath,
        selector: DatumSelector,
        call_role: CarrierCallRole,
        write_context: Option<CarrierWriteContext>,
    ) -> Self {
        self.steps.insert(
            0,
            CarrierStep {
                path,
                selector,
                call_role: Some(call_role),
                write_context,
            },
        );
        self
    }

    fn append(mut self, suffix: Self) -> Self {
        self.steps.extend(suffix.steps);
        self
    }

    pub(crate) fn paths(&self) -> Vec<NodePath> {
        self.steps.iter().map(|step| step.path.clone()).collect()
    }

    pub(crate) fn steps(&self) -> &[CarrierStep] {
        &self.steps
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CarrierGoal {
    External,
    Parameter(ParameterDatum),
}

impl CarrierGoal {
    fn is_present(self, dependency: &ProvenanceDependency) -> bool {
        match self {
            Self::External => dependency.unconditional_external,
            Self::Parameter(datum) => dependency.parameters.datums.binary_search(&datum).is_ok(),
        }
    }
}

pub(crate) fn carrier_route_cmp(left: &CarrierRoute, right: &CarrierRoute) -> Ordering {
    left.steps.len().cmp(&right.steps.len()).then_with(|| {
        left.steps
            .iter()
            .zip(&right.steps)
            .find_map(|(left, right)| {
                let ordering = left
                    .path
                    .components()
                    .cmp(right.path.components())
                    .then_with(|| left.selector.cmp(&right.selector));
                (ordering != Ordering::Equal).then_some(ordering)
            })
            .unwrap_or(Ordering::Equal)
    })
}

fn choose_carrier_route(current: &mut Option<CarrierRoute>, candidate: Option<CarrierRoute>) {
    let Some(candidate) = candidate else {
        return;
    };
    if current
        .as_ref()
        .is_none_or(|existing| carrier_route_cmp(&candidate, existing) == Ordering::Less)
    {
        *current = Some(candidate);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CarrierState {
    Binding {
        function: FunctionId,
        binding: BindingId,
        selector: DatumSelector,
        goal: CarrierGoal,
    },
    Storage {
        function: FunctionId,
        binding: BindingId,
        goal: CarrierGoal,
    },
    Result {
        function: FunctionId,
        selector: DatumSelector,
        goal: CarrierGoal,
    },
    Write {
        function: FunctionId,
        parameter: u32,
        goal: CarrierGoal,
    },
}

struct CarrierReconstructor<'check> {
    functions: &'check [CheckedFunction],
    summaries: &'check [FunctionDependencies],
    nominals: &'check [CheckedNominal],
    external_entry: Option<FunctionId>,
}

impl<'check> CarrierReconstructor<'check> {
    fn function(&self, function: FunctionId) -> ProvenanceResult<&'check CheckedFunction> {
        self.functions
            .get(function.0 as usize)
            .filter(|candidate| candidate.id == function)
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn summary(&self, function: FunctionId) -> ProvenanceResult<&'check FunctionDependencies> {
        self.summaries
            .get(function.0 as usize)
            .filter(|candidate| candidate.function == function)
            .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn pass(&self, function: FunctionId) -> ProvenanceResult<FunctionPass<'check>> {
        FunctionPass::from_metadata(
            self.function(function)?,
            self.nominals,
            self.summary(function)?,
        )
    }

    fn expression_dependency(
        &self,
        function: FunctionId,
        expression: &CheckedExpression,
        selector: DatumSelector,
    ) -> ProvenanceResult<ProvenanceDependency> {
        self.pass(function)?
            .expression(expression, self.summaries)?
            .selected(selector)
    }

    fn external_expression_route(
        &self,
        function: FunctionId,
        expression: &CheckedExpression,
        selector: DatumSelector,
    ) -> ProvenanceResult<CarrierRoute> {
        let dependency = self.expression_dependency(function, expression, selector)?;
        if !dependency.unconditional_external {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        self.route_expression(
            function,
            expression,
            selector,
            CarrierGoal::External,
            &mut Vec::new(),
        )?
        .ok_or(SemanticCompilerFailure::InvalidResolution)
    }

    fn route_expression_aggregate(
        &self,
        function: FunctionId,
        expression: &CheckedExpression,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let mut route = None;
        for selector in selectors(expression.ty(), self.nominals) {
            choose_carrier_route(
                &mut route,
                self.route_expression(function, expression, selector, goal, visited)?,
            );
        }
        Ok(route)
    }

    fn route_expression_list(
        &self,
        function: FunctionId,
        expressions: &[CheckedExpression],
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let mut route = None;
        for expression in expressions {
            choose_carrier_route(
                &mut route,
                self.route_expression_aggregate(function, expression, goal, visited)?,
            );
        }
        Ok(route)
    }

    fn prepend_expression_carrier(
        expression: &CheckedExpression,
        selector: DatumSelector,
        route: Option<CarrierRoute>,
    ) -> Option<CarrierRoute> {
        let carrier = expression.carrier()?;
        route.map(|route| route.prepend(carrier.clone(), selector))
    }

    fn route_expression(
        &self,
        function: FunctionId,
        expression: &CheckedExpression,
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        if !goal.is_present(&self.expression_dependency(function, expression, selector)?) {
            return Ok(None);
        }
        let source = match expression {
            CheckedExpression::Constant(_)
            | CheckedExpression::NamedConstant { .. }
            | CheckedExpression::ArrayLength { .. }
            | CheckedExpression::BufferLength { .. }
            | CheckedExpression::SliceLength { .. } => return Ok(None),
            CheckedExpression::SystemCall {
                operation,
                call,
                result,
                arguments,
                ..
            } => {
                let selected = system_result_dependencies(*operation, *result, self.nominals)?
                    .selected(selector)?;
                let mut route = (matches!(goal, CarrierGoal::External)
                    && selected.unconditional_external)
                    .then(|| {
                        CarrierRoute::call_terminal(
                            call.clone(),
                            selector,
                            CarrierCallRole::SystemResult,
                            None,
                        )
                    });
                if selector
                    == (DatumSelector::EnumPayload {
                        variant: 0,
                        field: 0,
                    })
                    && let Some(start) = system_endpoint_start(*operation)?
                {
                    choose_carrier_route(
                        &mut route,
                        self.route_expression_aggregate(
                            function,
                            arguments
                                .get(start)
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                            goal,
                            visited,
                        )?,
                    );
                }
                return Ok(route);
            }
            CheckedExpression::UserCall {
                function: callee,
                call,
                arguments,
                ..
            } => {
                return self.route_user_result(
                    function, *callee, call, arguments, selector, goal, visited,
                );
            }
            CheckedExpression::Binding { binding, .. } => {
                self.route_binding(function, *binding, selector, goal, visited)?
            }
            CheckedExpression::Project { binding, .. } => {
                self.route_storage(function, *binding, goal, visited)?
            }
            CheckedExpression::BorrowBuffer { root, .. } => {
                self.route_storage(function, root.binding, goal, visited)?
            }
            CheckedExpression::BorrowAddressed { binding, .. }
            | CheckedExpression::BorrowBox { binding, .. }
            | CheckedExpression::BorrowSystemResource { binding, .. }
            | CheckedExpression::ReborrowAddressed { binding, .. }
            | CheckedExpression::DerefAddressed { binding, .. } => {
                // These edges preserve the exact value component carried by
                // the holder binding. Storage-root resolution is reserved for
                // writes and storage-wide reads; using it here would skip the
                // holder's let/borrow edges and apply a payload selector to
                // its enclosing aggregate.
                self.route_binding(function, *binding, selector, goal, visited)?
            }
            CheckedExpression::IntegerOperation {
                operation,
                arguments,
                ..
            } => {
                let checked = matches!(
                    operation,
                    CheckedIntegerOperation::AddChecked
                        | CheckedIntegerOperation::SubtractChecked
                        | CheckedIntegerOperation::MultiplyChecked
                        | CheckedIntegerOperation::DivideChecked
                        | CheckedIntegerOperation::RemainderChecked
                        | CheckedIntegerOperation::AbsoluteChecked
                        | CheckedIntegerOperation::NegateChecked
                );
                if checked
                    && selector
                        != (DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        })
                {
                    return Ok(None);
                }
                self.route_expression_list(function, arguments, goal, visited)?
            }
            CheckedExpression::FloatOperation { arguments, .. }
            | CheckedExpression::BooleanOperation { arguments, .. }
            | CheckedExpression::EnumEquality { arguments, .. }
            | CheckedExpression::ConstructStruct {
                fields: arguments, ..
            } => self.route_expression_list(function, arguments, goal, visited)?,
            CheckedExpression::NumericConversion {
                source,
                destination,
                value,
                ..
            } => {
                if !source.converts_totally_to(*destination)
                    && selector
                        != (DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        })
                {
                    return Ok(None);
                }
                self.route_expression_aggregate(function, value, goal, visited)?
            }
            CheckedExpression::Reinterpret { value, .. }
            | CheckedExpression::ArrayFill { value, .. }
            | CheckedExpression::BoxNew { value, .. }
            | CheckedExpression::BoxDeref { value, .. }
            | CheckedExpression::ArenaNew { value, .. }
            | CheckedExpression::ArenaDeref { value, .. }
            | CheckedExpression::ProjectValue { value, .. } => {
                self.route_expression_aggregate(function, value, goal, visited)?
            }
            CheckedExpression::ArrayIndex { root, offset, .. } => {
                let mut route = match root {
                    CheckedArrayRoot::Binding { binding, .. } => {
                        self.route_storage(function, *binding, goal, visited)?
                    }
                    CheckedArrayRoot::Constant(_) => None,
                };
                choose_carrier_route(
                    &mut route,
                    self.route_expression_aggregate(function, offset, goal, visited)?,
                );
                route
            }
            CheckedExpression::BufferFill { length, value, .. } => {
                let mut route = self.route_expression_aggregate(function, length, goal, visited)?;
                choose_carrier_route(
                    &mut route,
                    self.route_expression_aggregate(function, value, goal, visited)?,
                );
                route
            }
            CheckedExpression::BufferVacant { length, .. }
            | CheckedExpression::BufferFits { length, .. } => {
                self.route_expression_aggregate(function, length, goal, visited)?
            }
            CheckedExpression::BufferIndex { root, offset, .. } => {
                let mut route = self.route_storage(function, root.binding, goal, visited)?;
                choose_carrier_route(
                    &mut route,
                    self.route_expression_aggregate(function, offset, goal, visited)?,
                );
                route
            }
            CheckedExpression::SliceIndex { root, offset, .. } => {
                let mut route = self.route_storage(function, root.binding, goal, visited)?;
                choose_carrier_route(
                    &mut route,
                    self.route_expression_aggregate(function, offset, goal, visited)?,
                );
                route
            }
            CheckedExpression::SliceOf { source, .. } => match source {
                CheckedSliceSource::Array { root, .. } => match root {
                    CheckedArrayRoot::Binding { binding, .. } => {
                        self.route_storage(function, *binding, goal, visited)?
                    }
                    CheckedArrayRoot::Constant(_) => None,
                },
                CheckedSliceSource::Buffer(root) => {
                    self.route_storage(function, root.binding, goal, visited)?
                }
                CheckedSliceSource::ArenaContent { binding, .. } => {
                    self.route_storage(function, *binding, goal, visited)?
                }
            },
            CheckedExpression::ConstructEnum {
                variant, fields, ..
            } => {
                let DatumSelector::EnumPayload {
                    variant: selected_variant,
                    field,
                } = selector
                else {
                    return Ok(None);
                };
                if selected_variant != *variant {
                    return Ok(None);
                }
                let field = fields
                    .get(field as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                self.route_expression_aggregate(function, field, goal, visited)?
            }
        };
        Ok(Self::prepend_expression_carrier(
            expression, selector, source,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn route_user_result(
        &self,
        caller: FunctionId,
        callee: FunctionId,
        call: &NodePath,
        arguments: &[CheckedExpression],
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let selected = self.summary(callee)?.result.selected(selector)?;
        let mut route = None;
        if matches!(goal, CarrierGoal::External) && selected.unconditional_external {
            choose_carrier_route(
                &mut route,
                self.route_result(callee, selector, CarrierGoal::External, visited)?,
            );
        }
        for datum in selected.parameters.datums {
            let argument = arguments
                .get(datum.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let Some(callee_route) =
                self.route_result(callee, selector, CarrierGoal::Parameter(datum), visited)?
            else {
                continue;
            };
            let Some(actual_route) =
                self.route_expression(caller, argument, datum.selector, goal, visited)?
            else {
                continue;
            };
            let substitution = CarrierRoute::call_terminal(
                call.clone(),
                datum.selector,
                CarrierCallRole::UserSubstitution,
                None,
            );
            choose_carrier_route(
                &mut route,
                Some(callee_route.append(substitution).append(actual_route)),
            );
        }
        Ok(route.map(|route| {
            route.prepend_call(call.clone(), selector, CarrierCallRole::UserResult, None)
        }))
    }

    fn route_binding(
        &self,
        function: FunctionId,
        binding: BindingId,
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let state = CarrierState::Binding {
            function,
            binding,
            selector,
            goal,
        };
        if visited.contains(&state) {
            return Ok(None);
        }
        let value = self
            .summary(function)?
            .bindings
            .get(binding.0 as usize)
            .and_then(Option::as_ref)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !goal.is_present(&value.selected(selector)?) {
            return Ok(None);
        }
        visited.push(state);
        let checked = self.function(function)?;
        let mut route = None;
        if let Some((ordinal, parameter)) = checked
            .parameters
            .iter()
            .enumerate()
            .find(|(_, parameter)| parameter.binding == binding)
        {
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let is_seed = match goal {
                CarrierGoal::External => self.external_entry == Some(function),
                CarrierGoal::Parameter(datum) => {
                    self.external_entry != Some(function)
                        && datum == (ParameterDatum { ordinal, selector })
                }
            };
            if is_seed {
                choose_carrier_route(
                    &mut route,
                    Some(CarrierRoute::terminal(
                        parameter.node_path.clone(),
                        selector,
                    )),
                );
            }
        }
        self.scan_binding_block(
            function,
            &checked.body,
            binding,
            selector,
            goal,
            visited,
            &mut route,
        )?;
        // Only projected/element writes and call writes seed every value
        // component. Ordinary initialization and whole-value assignment also
        // join the aggregate storage root, but must not manufacture a sibling
        // payload predecessor for this exact selected component.
        if self.pass(function)?.resolve_root(binding)? == binding {
            let mut pass = self.pass(function)?;
            self.scan_write_block(
                function,
                &checked.body,
                binding,
                &mut pass,
                goal,
                true,
                visited,
                &mut route,
            )?;
        }
        visited.pop();
        Ok(route)
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_binding_block(
        &self,
        function: FunctionId,
        statements: &[CheckedStatement],
        binding: BindingId,
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Let {
                    node_path,
                    binding: target,
                    value,
                } if *target == binding => choose_carrier_route(
                    route,
                    self.route_expression(function, value, selector, goal, visited)?
                        .map(|route| route.prepend(node_path.clone(), selector)),
                ),
                CheckedStatement::PropagateLet {
                    node_path,
                    binding: target,
                    scrutinee,
                    ..
                } if *target == binding => choose_carrier_route(
                    route,
                    self.route_expression(
                        function,
                        scrutinee,
                        DatumSelector::EnumPayload {
                            variant: 0,
                            field: 0,
                        },
                        goal,
                        visited,
                    )?
                    .map(|route| route.prepend(node_path.clone(), selector)),
                ),
                CheckedStatement::Set {
                    node_path,
                    target: CheckedSetTarget::Place(place),
                    value,
                } if place.binding == binding && place.fields.is_empty() => {
                    let resolved = self.pass(function)?.resolve_root(place.binding)?;
                    if resolved == binding {
                        choose_carrier_route(
                            route,
                            self.route_expression(function, value, selector, goal, visited)?
                                .map(|route| route.prepend(node_path.clone(), selector)),
                        );
                    }
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                } => {
                    for arm in arms {
                        for binder in &arm.binders {
                            if binder.binding == binding {
                                let source_selector = DatumSelector::EnumPayload {
                                    variant: arm.tag,
                                    field: binder.field,
                                };
                                choose_carrier_route(
                                    route,
                                    self.route_expression(
                                        function,
                                        scrutinee,
                                        source_selector,
                                        goal,
                                        visited,
                                    )?
                                    .map(|route| route.prepend(binder.node_path.clone(), selector)),
                                );
                            }
                        }
                        self.scan_binding_block(
                            function, &arm.body, binding, selector, goal, visited, route,
                        )?;
                    }
                }
                CheckedStatement::ValueMatchLet {
                    node_path,
                    binding: target,
                    scrutinee,
                    arms,
                    ..
                } => {
                    if *target == binding {
                        let delivered =
                            self.route_gives(function, arms, selector, goal, visited)?;
                        choose_carrier_route(
                            route,
                            delivered.map(|route| route.prepend(node_path.clone(), selector)),
                        );
                    }
                    for arm in arms {
                        for binder in &arm.binders {
                            if binder.binding == binding {
                                let source_selector = DatumSelector::EnumPayload {
                                    variant: arm.tag,
                                    field: binder.field,
                                };
                                choose_carrier_route(
                                    route,
                                    self.route_expression(
                                        function,
                                        scrutinee,
                                        source_selector,
                                        goal,
                                        visited,
                                    )?
                                    .map(|route| route.prepend(binder.node_path.clone(), selector)),
                                );
                            }
                        }
                        self.scan_binding_block(
                            function, &arm.body, binding, selector, goal, visited, route,
                        )?;
                    }
                }
                CheckedStatement::CountedRange {
                    binder,
                    lower,
                    body,
                    ..
                } => {
                    if *binder == binding {
                        choose_carrier_route(
                            route,
                            self.route_expression_aggregate(function, lower, goal, visited)?,
                        );
                    }
                    self.scan_binding_block(
                        function, body, binding, selector, goal, visited, route,
                    )?;
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.scan_binding_block(
                        function, body, binding, selector, goal, visited, route,
                    )?;
                }
                // A replace-bound value's origin is storage, not an
                // expression; yielding no carrier route here is the
                // fail-closed disposition [PRV-1].
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn route_gives(
        &self,
        function: FunctionId,
        arms: &[CheckedMatchArm],
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let mut route = None;
        for arm in arms {
            self.scan_give_block(function, &arm.body, selector, goal, visited, &mut route)?;
        }
        Ok(route)
    }

    fn scan_give_block(
        &self,
        function: FunctionId,
        statements: &[CheckedStatement],
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Give {
                    node_path, value, ..
                } => choose_carrier_route(
                    route,
                    self.route_expression(function, value, selector, goal, visited)?
                        .map(|route| route.prepend(node_path.clone(), selector)),
                ),
                CheckedStatement::Match { arms, .. } => {
                    for arm in arms {
                        self.scan_give_block(function, &arm.body, selector, goal, visited, route)?;
                    }
                }
                // A nested value initializer owns its own delivery set.
                CheckedStatement::ValueMatchLet { .. } => {}
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => {
                    self.scan_give_block(function, body, selector, goal, visited, route)?;
                }
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Return { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn route_storage(
        &self,
        function: FunctionId,
        binding: BindingId,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let mut pass = self.pass(function)?;
        let resolved = pass.resolve_root(binding)?;
        let state = CarrierState::Storage {
            function,
            binding: resolved,
            goal,
        };
        if visited.contains(&state) {
            return Ok(None);
        }
        let dependency = self
            .summary(function)?
            .storage_roots
            .get(resolved.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !goal.is_present(dependency) {
            return Ok(None);
        }
        visited.push(state);
        let checked = self.function(function)?;
        let mut route = None;
        if let Some((ordinal, parameter)) = checked
            .parameters
            .iter()
            .enumerate()
            .find(|(_, parameter)| parameter.binding == resolved)
        {
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            match goal {
                CarrierGoal::External if self.external_entry == Some(function) => {
                    for selector in selectors(parameter.ty, self.nominals) {
                        choose_carrier_route(
                            &mut route,
                            Some(CarrierRoute::terminal(
                                parameter.node_path.clone(),
                                selector,
                            )),
                        );
                    }
                }
                CarrierGoal::Parameter(datum)
                    if self.external_entry != Some(function) && datum.ordinal == ordinal =>
                {
                    choose_carrier_route(
                        &mut route,
                        Some(CarrierRoute::terminal(
                            parameter.node_path.clone(),
                            datum.selector,
                        )),
                    );
                }
                CarrierGoal::External | CarrierGoal::Parameter(_) => {}
            }
        }
        self.scan_storage_block(
            function,
            &checked.body,
            resolved,
            &mut pass,
            goal,
            visited,
            &mut route,
        )?;
        visited.pop();
        Ok(route)
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_storage_match_arms(
        &self,
        function: FunctionId,
        scrutinee: &CheckedExpression,
        arms: &[CheckedMatchArm],
        root: BindingId,
        pass: &mut FunctionPass<'check>,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for arm in arms {
            for binder in &arm.binders {
                if binder.binding == root {
                    choose_carrier_route(
                        route,
                        self.route_expression(
                            function,
                            scrutinee,
                            DatumSelector::EnumPayload {
                                variant: arm.tag,
                                field: binder.field,
                            },
                            goal,
                            visited,
                        )?
                        .map(|route| route.prepend(binder.node_path.clone(), DatumSelector::Plain)),
                    );
                }
            }
            self.scan_storage_block(function, &arm.body, root, pass, goal, visited, route)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_storage_block(
        &self,
        function: FunctionId,
        statements: &[CheckedStatement],
        root: BindingId,
        pass: &mut FunctionPass<'check>,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Let {
                    node_path,
                    binding,
                    value,
                } => {
                    if *binding == root {
                        choose_carrier_route(
                            route,
                            self.route_expression_aggregate(function, value, goal, visited)?
                                .map(|route| {
                                    route.prepend(node_path.clone(), DatumSelector::Plain)
                                }),
                        );
                    }
                    self.scan_expression_writes(function, value, root, pass, goal, visited, route)?;
                }
                CheckedStatement::PropagateLet {
                    node_path,
                    binding,
                    scrutinee,
                    ..
                } => {
                    if *binding == root {
                        choose_carrier_route(
                            route,
                            self.route_expression(
                                function,
                                scrutinee,
                                DatumSelector::EnumPayload {
                                    variant: 0,
                                    field: 0,
                                },
                                goal,
                                visited,
                            )?
                            .map(|route| route.prepend(node_path.clone(), DatumSelector::Plain)),
                        );
                    }
                    self.scan_expression_writes(
                        function, scrutinee, root, pass, goal, visited, route,
                    )?;
                }
                CheckedStatement::Set {
                    node_path,
                    target,
                    value,
                }
                | CheckedStatement::Replace {
                    node_path,
                    target,
                    value,
                    ..
                } => {
                    if pass.resolve_root(target.binding())? == root {
                        choose_carrier_route(
                            route,
                            self.route_expression_aggregate(function, value, goal, visited)?
                                .map(|route| {
                                    route.prepend(node_path.clone(), DatumSelector::Plain)
                                }),
                        );
                    }
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => self.scan_expression_writes(
                            function,
                            &target.offset,
                            root,
                            pass,
                            goal,
                            visited,
                            route,
                        )?,
                        CheckedSetTarget::BufferIndex(target) => self.scan_expression_writes(
                            function,
                            &target.offset,
                            root,
                            pass,
                            goal,
                            visited,
                            route,
                        )?,
                    }
                    self.scan_expression_writes(function, value, root, pass, goal, visited, route)?;
                }
                CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Claim {
                    condition: value, ..
                }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    self.scan_expression_writes(function, value, root, pass, goal, visited, route)?;
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                } => {
                    self.scan_expression_writes(
                        function, scrutinee, root, pass, goal, visited, route,
                    )?;
                    self.scan_storage_match_arms(
                        function, scrutinee, arms, root, pass, goal, visited, route,
                    )?;
                }
                CheckedStatement::ValueMatchLet {
                    node_path,
                    binding,
                    result_type,
                    scrutinee,
                    arms,
                    ..
                } => {
                    self.scan_expression_writes(
                        function, scrutinee, root, pass, goal, visited, route,
                    )?;
                    if *binding == root {
                        let mut delivered = None;
                        for selector in selectors(*result_type, self.nominals) {
                            choose_carrier_route(
                                &mut delivered,
                                self.route_gives(function, arms, selector, goal, visited)?,
                            );
                        }
                        choose_carrier_route(
                            route,
                            delivered.map(|route| {
                                route.prepend(node_path.clone(), DatumSelector::Plain)
                            }),
                        );
                    }
                    self.scan_storage_match_arms(
                        function, scrutinee, arms, root, pass, goal, visited, route,
                    )?;
                }
                CheckedStatement::CountedRange {
                    node_path,
                    binder,
                    lower,
                    upper,
                    body,
                    ..
                } => {
                    if *binder == root {
                        choose_carrier_route(
                            route,
                            self.route_expression_aggregate(function, lower, goal, visited)?,
                        );
                        // The compiler-owned increment is dependency
                        // preserving; its self-edge is longer than the lower
                        // seed but remains represented by this exact carrier.
                        if let Some(existing) = route.clone() {
                            choose_carrier_route(
                                route,
                                Some(existing.prepend(node_path.clone(), DatumSelector::Plain)),
                            );
                        }
                    }
                    self.scan_expression_writes(function, lower, root, pass, goal, visited, route)?;
                    self.scan_expression_writes(function, upper, root, pass, goal, visited, route)?;
                    self.scan_storage_block(function, body, root, pass, goal, visited, route)?;
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.scan_storage_block(function, body, root, pass, goal, visited, route)?;
                }
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_expression_writes(
        &self,
        function: FunctionId,
        expression: &CheckedExpression,
        root: BindingId,
        pass: &mut FunctionPass<'check>,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        match expression {
            CheckedExpression::UserCall {
                function: callee,
                call,
                argument_nodes,
                arguments,
                ..
            } => {
                let callee_summary = self.summary(*callee)?;
                if callee_summary.writes.len() != arguments.len() {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                }
                for (ordinal, argument) in arguments.iter().enumerate() {
                    if pass.argument_root(argument)? == Some(root) {
                        let parameter = u32::try_from(ordinal)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                        choose_carrier_route(
                            route,
                            self.route_user_write(
                                function,
                                *callee,
                                call,
                                argument_nodes,
                                arguments,
                                parameter,
                                goal,
                                visited,
                            )?,
                        );
                    }
                }
            }
            CheckedExpression::SystemCall {
                operation,
                call,
                argument_nodes,
                arguments,
                ..
            } => {
                for ordinal in system_external_writes(*operation)? {
                    let argument = arguments
                        .get(*ordinal)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if matches!(goal, CarrierGoal::External)
                        && pass.argument_root(argument)? == Some(root)
                    {
                        let argument_node = argument_nodes
                            .get(*ordinal)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                        let system_parameter = u32::try_from(*ordinal)
                            .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                        let candidate = CarrierRoute::call_terminal(
                            call.clone(),
                            DatumSelector::Plain,
                            CarrierCallRole::SystemWrite,
                            Some(CarrierWriteContext {
                                parameter: system_parameter,
                                actual: argument_node.clone(),
                            }),
                        );
                        choose_carrier_route(route, Some(candidate));
                    }
                }
            }
            _ => {}
        }
        for child in expression_children(expression) {
            self.scan_expression_writes(function, child, root, pass, goal, visited, route)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn route_user_write(
        &self,
        caller: FunctionId,
        callee: FunctionId,
        call: &NodePath,
        argument_nodes: &[NodePath],
        arguments: &[CheckedExpression],
        parameter: u32,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let dependency = self
            .summary(callee)?
            .writes
            .get(parameter as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let mut route = None;
        if matches!(goal, CarrierGoal::External) && dependency.unconditional_external {
            choose_carrier_route(
                &mut route,
                self.route_write(callee, parameter, CarrierGoal::External, visited)?,
            );
        }
        for datum in &dependency.parameters.datums {
            let argument = arguments
                .get(datum.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let Some(callee_route) =
                self.route_write(callee, parameter, CarrierGoal::Parameter(*datum), visited)?
            else {
                continue;
            };
            let Some(actual_route) =
                self.route_expression(caller, argument, datum.selector, goal, visited)?
            else {
                continue;
            };
            let substitution = CarrierRoute::call_terminal(
                call.clone(),
                datum.selector,
                CarrierCallRole::UserSubstitution,
                None,
            );
            choose_carrier_route(
                &mut route,
                Some(callee_route.append(substitution).append(actual_route)),
            );
        }
        let argument_node = argument_nodes
            .get(parameter as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(route) = route else {
            return Ok(None);
        };
        Ok(Some(route.prepend_call(
            call.clone(),
            DatumSelector::Plain,
            CarrierCallRole::UserWrite,
            Some(CarrierWriteContext {
                parameter,
                actual: argument_node.clone(),
            }),
        )))
    }

    fn route_result(
        &self,
        function: FunctionId,
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let state = CarrierState::Result {
            function,
            selector,
            goal,
        };
        if visited.contains(&state) {
            return Ok(None);
        }
        if !goal.is_present(&self.summary(function)?.result.selected(selector)?) {
            return Ok(None);
        }
        visited.push(state);
        let mut route = None;
        self.scan_result_block(
            function,
            &self.function(function)?.body,
            selector,
            goal,
            visited,
            &mut route,
        )?;
        visited.pop();
        Ok(route)
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_result_block(
        &self,
        function: FunctionId,
        statements: &[CheckedStatement],
        selector: DatumSelector,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Return {
                    node_path, value, ..
                } => choose_carrier_route(
                    route,
                    self.route_expression(function, value, selector, goal, visited)?
                        .map(|route| route.prepend(node_path.clone(), selector)),
                ),
                CheckedStatement::PropagateLet {
                    node_path,
                    scrutinee,
                    ..
                } if selector
                    == (DatumSelector::EnumPayload {
                        variant: 1,
                        field: 0,
                    }) =>
                {
                    choose_carrier_route(
                        route,
                        self.route_expression(function, scrutinee, selector, goal, visited)?
                            .map(|route| route.prepend(node_path.clone(), selector)),
                    )
                }
                CheckedStatement::Match { arms, .. }
                | CheckedStatement::ValueMatchLet { arms, .. } => {
                    for arm in arms {
                        self.scan_result_block(
                            function, &arm.body, selector, goal, visited, route,
                        )?;
                    }
                }
                CheckedStatement::Loop { body, .. }
                | CheckedStatement::CountedRange { body, .. }
                | CheckedStatement::Region { body, .. } => {
                    self.scan_result_block(function, body, selector, goal, visited, route)?
                }
                CheckedStatement::Let { .. }
                | CheckedStatement::PropagateLet { .. }
                | CheckedStatement::Set { .. }
                | CheckedStatement::Replace { .. }
                | CheckedStatement::Evaluate(_)
                | CheckedStatement::DropExpression { .. }
                | CheckedStatement::Claim { .. }
                | CheckedStatement::Give { .. }
                | CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }

    fn route_write(
        &self,
        function: FunctionId,
        parameter: u32,
        goal: CarrierGoal,
        visited: &mut Vec<CarrierState>,
    ) -> ProvenanceResult<Option<CarrierRoute>> {
        let state = CarrierState::Write {
            function,
            parameter,
            goal,
        };
        if visited.contains(&state) {
            return Ok(None);
        }
        let dependency = self
            .summary(function)?
            .writes
            .get(parameter as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !goal.is_present(dependency) {
            return Ok(None);
        }
        let checked = self.function(function)?;
        let formal = checked
            .parameters
            .get(parameter as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let mut pass = self.pass(function)?;
        let root = pass.resolve_root(formal.binding)?;
        visited.push(state);
        let mut route = None;
        self.scan_write_block(
            function,
            &checked.body,
            root,
            &mut pass,
            goal,
            false,
            visited,
            &mut route,
        )?;
        visited.pop();
        Ok(route)
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_write_block(
        &self,
        function: FunctionId,
        statements: &[CheckedStatement],
        root: BindingId,
        pass: &mut FunctionPass<'check>,
        goal: CarrierGoal,
        widening_only: bool,
        visited: &mut Vec<CarrierState>,
        route: &mut Option<CarrierRoute>,
    ) -> ProvenanceResult<()> {
        for statement in statements {
            match statement {
                CheckedStatement::Let { value, .. }
                | CheckedStatement::Evaluate(value)
                | CheckedStatement::DropExpression { value, .. }
                | CheckedStatement::Claim {
                    condition: value, ..
                }
                | CheckedStatement::Return { value, .. }
                | CheckedStatement::Give { value, .. } => {
                    self.scan_expression_writes(function, value, root, pass, goal, visited, route)?
                }
                CheckedStatement::PropagateLet { scrutinee, .. } => self.scan_expression_writes(
                    function, scrutinee, root, pass, goal, visited, route,
                )?,
                CheckedStatement::Set {
                    node_path,
                    target,
                    value,
                }
                | CheckedStatement::Replace {
                    node_path,
                    target,
                    value,
                    ..
                } => {
                    let resolved = pass.resolve_root(target.binding())?;
                    let seeds_every_value_component =
                        pass.set_seeds_every_value_component(target)?;
                    if resolved == root && (!widening_only || seeds_every_value_component) {
                        choose_carrier_route(
                            route,
                            self.route_expression_aggregate(function, value, goal, visited)?
                                .map(|route| {
                                    route.prepend(node_path.clone(), DatumSelector::Plain)
                                }),
                        );
                    }
                    self.scan_expression_writes(function, value, root, pass, goal, visited, route)?;
                    match target {
                        CheckedSetTarget::Place(_) => {}
                        CheckedSetTarget::ArrayIndex(target) => self.scan_expression_writes(
                            function,
                            &target.offset,
                            root,
                            pass,
                            goal,
                            visited,
                            route,
                        )?,
                        CheckedSetTarget::BufferIndex(target) => self.scan_expression_writes(
                            function,
                            &target.offset,
                            root,
                            pass,
                            goal,
                            visited,
                            route,
                        )?,
                    }
                }
                CheckedStatement::Match {
                    scrutinee, arms, ..
                }
                | CheckedStatement::ValueMatchLet {
                    scrutinee, arms, ..
                } => {
                    self.scan_expression_writes(
                        function, scrutinee, root, pass, goal, visited, route,
                    )?;
                    for arm in arms {
                        self.scan_write_block(
                            function,
                            &arm.body,
                            root,
                            pass,
                            goal,
                            widening_only,
                            visited,
                            route,
                        )?;
                    }
                }
                CheckedStatement::CountedRange {
                    lower, upper, body, ..
                } => {
                    self.scan_expression_writes(function, lower, root, pass, goal, visited, route)?;
                    self.scan_expression_writes(function, upper, root, pass, goal, visited, route)?;
                    self.scan_write_block(
                        function,
                        body,
                        root,
                        pass,
                        goal,
                        widening_only,
                        visited,
                        route,
                    )?;
                }
                CheckedStatement::Loop { body, .. } | CheckedStatement::Region { body, .. } => {
                    self.scan_write_block(
                        function,
                        body,
                        root,
                        pass,
                        goal,
                        widening_only,
                        visited,
                        route,
                    )?;
                }
                CheckedStatement::Break { .. } => {}
            }
        }
        Ok(())
    }
}

fn call_counterfactuals<'outcome>(
    outcomes: &'outcome [CallGoalCounterfactual],
    call: &NodePath,
    clauses: &[NodePath],
) -> Option<Vec<&'outcome CallGoalCounterfactual>> {
    let selected = outcomes
        .iter()
        .filter(|outcome| outcome.node_path == *call)
        .collect::<Vec<_>>();
    (selected.len() == clauses.len()
        && selected
            .iter()
            .zip(clauses)
            .all(|(outcome, clause)| outcome.requires_clause == *clause))
    .then_some(selected)
}

fn call_outcomes<'outcome>(
    outcomes: &'outcome [CallGoalOutcome],
    call: &NodePath,
    clauses: &[NodePath],
) -> Option<Vec<&'outcome CallGoalOutcome>> {
    let selected = outcomes
        .iter()
        .filter(|outcome| outcome.node_path == *call)
        .collect::<Vec<_>>();
    (selected.len() == clauses.len()
        && selected
            .iter()
            .zip(clauses)
            .all(|(outcome, clause)| outcome.requires_clause == *clause))
    .then_some(selected)
}

fn call_actuals_have_failed_obligation(
    function: &CheckedFunction,
    argument_nodes: &[NodePath],
) -> bool {
    function.entailment.obligations.iter().any(|outcome| {
        !outcome.discharged
            && argument_nodes.iter().any(|argument| {
                outcome
                    .node_path
                    .components()
                    .starts_with(argument.components())
            })
    })
}

fn counterfactual_view(outcomes: &[&CallGoalCounterfactual]) -> BridgeGoalView {
    BridgeGoalView {
        actual_obligations_ok: outcomes.iter().all(|outcome| outcome.actual_obligations_ok),
        goal_disposition: outcomes
            .iter()
            .map(|outcome| outcome.goal_disposition)
            .find(|disposition| *disposition != CallGoalDisposition::Discharged)
            .unwrap_or(CallGoalDisposition::Discharged),
        goal_evidence: outcomes
            .iter()
            .flat_map(|outcome| outcome.goal_evidence.iter().copied())
            .collect(),
    }
}

fn call_is_upstream_generator(call: &CallInventory, external_entry: Option<FunctionId>) -> bool {
    goal_view_discharged(&call.unasserted)
        && !goal_view_discharged(&call.blinded)
        && call.caller_requirement.is_some()
        && external_entry != Some(call.site.caller)
}

fn goal_view_discharged(view: &BridgeGoalView) -> bool {
    view.actual_obligations_ok && view.goal_disposition == CallGoalDisposition::Discharged
}

fn actual_values(
    function: &CheckedFunction,
    metadata: &FunctionDependencies,
    arguments: &[CheckedExpression],
    summaries: &[FunctionDependencies],
    nominals: &[CheckedNominal],
) -> ProvenanceResult<Vec<ValueDependencies>> {
    let mut pass = FunctionPass::from_metadata(function, nominals, metadata)?;
    arguments
        .iter()
        .map(|argument| pass.expression(argument, summaries))
        .collect()
}

fn build_call_inventory(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    unasserted: &[super::entailment::FunctionEntailmentView],
    blinded: &[super::entailment::FunctionEntailmentView],
    nominals: &[CheckedNominal],
) -> ProvenanceResult<Vec<CallInventory>> {
    let mut calls = Vec::new();
    for function in functions {
        let (_, sites, _) = collect_sites(function);
        let function_dependencies = dependencies
            .get(function.id.0 as usize)
            .filter(|metadata| metadata.function == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let unasserted = unasserted
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let blinded = blinded
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        for site in sites {
            let full = call_outcomes(
                &function.entailment.call_goals,
                &site.call,
                &site.downstream_requirement.clauses,
            );
            let Some(full) = full else {
                // ENT-6 publishes no FN-8 outcome after an actual's own OP-4
                // failure. That base failure is a normal inapplicable PRV
                // premise; absence without such a descendant failure is a
                // checked-model inconsistency and remains fail-closed.
                if call_actuals_have_failed_obligation(function, &site.argument_nodes) {
                    continue;
                }
                return Err(SemanticCompilerFailure::InvalidResolution);
            };
            if full
                .iter()
                .any(|outcome| outcome.disposition != CallGoalDisposition::Discharged)
            {
                continue;
            }
            let unasserted = call_counterfactuals(
                &unasserted.call_goals,
                &site.call,
                &site.downstream_requirement.clauses,
            )
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let blinded = call_counterfactuals(
                &blinded.call_goals,
                &site.call,
                &site.downstream_requirement.clauses,
            )
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let actuals = actual_values(
                function,
                function_dependencies,
                &site.arguments,
                dependencies,
                nominals,
            )?;
            calls.push(CallInventory {
                site,
                caller_requirement: requirement_occurrence(function),
                actuals,
                full: BridgeGoalView {
                    actual_obligations_ok: true,
                    goal_disposition: CallGoalDisposition::Discharged,
                    goal_evidence: full
                        .iter()
                        .flat_map(|outcome| outcome.evidence.iter().copied())
                        .collect(),
                },
                unasserted: counterfactual_view(&unasserted),
                blinded: counterfactual_view(&blinded),
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
    Ok(calls)
}

fn build_direct_call_inventory(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    nominals: &[CheckedNominal],
) -> ProvenanceResult<Vec<DirectCallInventory>> {
    let mut calls = Vec::new();
    for function in functions {
        let (_, _, sites) = collect_sites(function);
        let function_dependencies = dependencies
            .get(function.id.0 as usize)
            .filter(|metadata| metadata.function == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        for site in sites {
            if site.has_requirement {
                let outcomes = function
                    .entailment
                    .call_goals
                    .iter()
                    .filter(|outcome| outcome.node_path == site.call)
                    .collect::<Vec<_>>();
                let Some(first) = outcomes.first() else {
                    if call_actuals_have_failed_obligation(function, &site.argument_nodes) {
                        continue;
                    }
                    return Err(SemanticCompilerFailure::InvalidResolution);
                };
                // The dark checker deliberately retains base-failing calls so
                // entailment tests can inspect them; ordinary accepted source
                // never reaches provenance with this disposition.
                if first.disposition != CallGoalDisposition::Discharged
                    || outcomes
                        .iter()
                        .any(|outcome| outcome.disposition != CallGoalDisposition::Discharged)
                {
                    continue;
                }
            }
            calls.push(DirectCallInventory {
                actuals: actual_values(
                    function,
                    function_dependencies,
                    &site.arguments,
                    dependencies,
                    nominals,
                )?,
                site,
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
            .then_with(|| left.site.callee.0.cmp(&right.site.callee.0))
    });
    Ok(calls)
}

fn local_bridge_seeds(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    unasserted: &[super::entailment::FunctionEntailmentView],
    blinded: &[super::entailment::FunctionEntailmentView],
    nominals: &[CheckedNominal],
) -> ProvenanceResult<(Vec<StructuralKey>, Vec<SubjectKey>)> {
    let mut structural = Vec::new();
    let mut subjects = Vec::new();
    for function in functions {
        let Some(requirement) = requirement_occurrence(function) else {
            continue;
        };
        let function_dependencies = dependencies
            .get(function.id.0 as usize)
            .filter(|metadata| metadata.function == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let unasserted = unasserted
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let blinded = blinded
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (leaves, _, _) = collect_sites(function);
        for site in leaves {
            let full_discharged = function.entailment.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            let unasserted_discharged = unasserted.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            let blinded_discharged = blinded.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            if !full_discharged || !unasserted_discharged || blinded_discharged {
                continue;
            }
            let mut pass = FunctionPass::from_metadata(function, nominals, function_dependencies)?;
            let mut dependency = ProvenanceDependency::default();
            for subject in &site.subjects {
                dependency.union(&pass.expression(subject, dependencies)?.aggregate());
            }
            // A true unconditional bit terminates locally under PRV-3.  Its
            // companion parameter datums are diagnostic explanations only.
            if dependency.unconditional_external {
                continue;
            }
            insert_structural(
                &mut structural,
                StructuralKey {
                    requirement: requirement.clone(),
                    leaf: site.leaf.clone(),
                },
            );
            for subject in dependency.parameters.datums {
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
    Ok((structural, subjects))
}

fn local_gate_seeds(
    functions: &[CheckedFunction],
    dependencies: &[FunctionDependencies],
    unasserted: &[super::entailment::FunctionEntailmentView],
    blinded: &[super::entailment::FunctionEntailmentView],
    nominals: &[CheckedNominal],
    reconstructor: &CarrierReconstructor<'_>,
) -> ProvenanceResult<(
    Vec<DirectKey>,
    Vec<LocalGateCandidate>,
    Vec<LocalLeafDisposition>,
)> {
    let mut direct = Vec::new();
    let mut local = Vec::new();
    let mut dispositions = Vec::new();
    for function in functions {
        let function_dependencies = dependencies
            .get(function.id.0 as usize)
            .filter(|metadata| metadata.function == function.id)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let unasserted = unasserted
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let blinded = blinded
            .get(function.id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (leaves, _, _) = collect_sites(function);
        for site in leaves {
            let full_discharged = function.entailment.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            if !full_discharged {
                continue;
            }
            let blinded_discharged = blinded.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            let unasserted_discharged = unasserted.obligations.iter().any(|outcome| {
                outcome.node_path == site.leaf.obligation
                    && u32::from(outcome.conjunct) == site.leaf.conjunct
                    && outcome.discharged
            });
            if blinded_discharged {
                dispositions.push(LocalLeafDisposition {
                    leaf: site.leaf,
                    complete_discharged: true,
                    unasserted_discharged,
                    s4_blinded_discharged: true,
                    disposition: LocalLeafProvenanceDisposition::BlindedDischarged,
                });
                continue;
            }
            let mut pass = FunctionPass::from_metadata(function, nominals, function_dependencies)?;
            let mut subject = ProvenanceDependency::default();
            for expression in &site.subjects {
                subject.union(&pass.expression(expression, dependencies)?.aggregate());
            }
            if subject.unconditional_external {
                let entry_requirement =
                    if reconstructor.external_entry == Some(function.id) && unasserted_discharged {
                        Some(
                            requirement_occurrence(function)
                                .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                        )
                    } else {
                        None
                    };
                local.push(LocalGateCandidate {
                    leaf: site.leaf,
                    subject,
                    entry_requirement,
                    carrier: {
                        let mut route = None;
                        for expression in &site.subjects {
                            if pass
                                .expression(expression, dependencies)?
                                .aggregate()
                                .unconditional_external
                            {
                                choose_carrier_route(
                                    &mut route,
                                    Some(reconstructor.external_expression_route(
                                        function.id,
                                        expression,
                                        DatumSelector::Plain,
                                    )?),
                                );
                            }
                        }
                        route.ok_or(SemanticCompilerFailure::InvalidResolution)?
                    },
                });
                continue;
            }
            let disposition = if subject.parameters.datums.is_empty() {
                LocalLeafProvenanceDisposition::Internal
            } else if unasserted_discharged {
                LocalLeafProvenanceDisposition::RequirementBridge
            } else {
                for datum in &subject.parameters.datums {
                    insert_direct(
                        &mut direct,
                        DirectKey {
                            function: function.id,
                            subject: *datum,
                            leaf: site.leaf.clone(),
                        },
                    );
                }
                LocalLeafProvenanceDisposition::DirectDemand
            };
            dispositions.push(LocalLeafDisposition {
                leaf: site.leaf,
                complete_discharged: true,
                unasserted_discharged,
                s4_blinded_discharged: false,
                disposition,
            });
        }
    }
    dispositions.sort_by(|left, right| leaf_cmp(&left.leaf, &right.leaf));
    local.sort_by(|left, right| leaf_cmp(&left.leaf, &right.leaf));
    Ok((direct, local, dispositions))
}

fn build_call_argument_dispositions(
    calls: &[DirectCallInventory],
    required_calls: &[CallInventory],
    events: &[ProvenanceCallEvent],
) -> ProvenanceResult<Vec<CallArgumentDisposition>> {
    let mut dispositions = Vec::new();
    for call in calls {
        let required = if call.site.has_requirement {
            Some(
                required_calls
                    .iter()
                    .find(|required| {
                        required.site.caller == call.site.caller
                            && required.site.call == call.site.call
                    })
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
            )
        } else {
            None
        };
        if call.site.argument_nodes.len() != call.site.arguments.len()
            || call.actuals.len() != call.site.arguments.len()
        {
            return Err(SemanticCompilerFailure::InvalidResolution);
        }
        for (argument, argument_node) in call.site.argument_nodes.iter().enumerate() {
            let argument =
                u32::try_from(argument).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            if events.iter().any(|event| {
                event.caller == call.site.caller
                    && event.call == call.site.call
                    && event.argument == argument
            }) {
                continue;
            }
            let observation = |view: Option<&BridgeGoalView>| match view {
                Some(view) => ProvenanceGoalObservation::Evaluated(view.clone()),
                None => ProvenanceGoalObservation::NotApplicable,
            };
            dispositions.push(CallArgumentDisposition {
                caller: call.site.caller,
                call: call.site.call.clone(),
                argument,
                argument_node: argument_node.clone(),
                complete: observation(required.map(|required| &required.full)),
                unasserted: observation(required.map(|required| &required.unasserted)),
                s4_blinded: observation(required.map(|required| &required.blinded)),
                disposition: CallArgumentProvenanceDisposition::NoEvent,
            });
        }
    }
    Ok(dispositions)
}

fn bridge_fixed_point(
    calls: &[CallInventory],
    local_structural: &[StructuralKey],
    local_subjects: &[SubjectKey],
    external_entry: Option<FunctionId>,
) -> ProvenanceResult<(Vec<StructuralKey>, Vec<SubjectKey>)> {
    let mut structural = local_structural.to_vec();
    let mut subjects = local_subjects.to_vec();
    loop {
        let structural_before = structural.clone();
        let subjects_before = subjects.clone();
        for call in calls {
            if !call_is_upstream_generator(call, external_entry) {
                continue;
            }
            let caller_requirement = call
                .caller_requirement
                .as_ref()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
                let actual = call
                    .actuals
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let selected = actual.selected(downstream.subject.selector)?;
                if selected.unconditional_external {
                    continue;
                }
                for subject in selected.parameters.datums {
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
    Ok((structural, subjects))
}

fn direct_demand_fixed_point(
    calls: &[DirectCallInventory],
    bridge_calls: &[CallInventory],
    bridges: &[SubjectKey],
    local: &[DirectKey],
) -> ProvenanceResult<Vec<DirectKey>> {
    let mut direct = local.to_vec();

    // A bridge that fails in U becomes an ordinary direct demand in its
    // caller.  A true unconditional bit terminates at this call and is
    // retained only as a PRV-2 target below.
    for call in bridge_calls {
        if goal_view_discharged(&call.blinded) || goal_view_discharged(&call.unasserted) {
            continue;
        }
        for bridge in bridges {
            if bridge.requirement != call.site.downstream_requirement {
                continue;
            }
            let actual = call
                .actuals
                .get(bridge.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let selected = actual.selected(bridge.subject.selector)?;
            if selected.unconditional_external {
                continue;
            }
            for subject in selected.parameters.datums {
                insert_direct(
                    &mut direct,
                    DirectKey {
                        function: call.site.caller,
                        subject,
                        leaf: bridge.leaf.clone(),
                    },
                );
            }
        }
    }

    loop {
        let previous = direct.clone();
        for call in calls {
            for downstream in &previous {
                if downstream.function != call.site.callee {
                    continue;
                }
                let actual = call
                    .actuals
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let selected = actual.selected(downstream.subject.selector)?;
                if selected.unconditional_external {
                    continue;
                }
                for subject in selected.parameters.datums {
                    insert_direct(
                        &mut direct,
                        DirectKey {
                            function: call.site.caller,
                            subject,
                            leaf: downstream.leaf.clone(),
                        },
                    );
                }
            }
        }
        if direct == previous {
            break;
        }
    }
    Ok(direct)
}

fn reconstruct_subject_routes(
    converged: &[SubjectKey],
    local: &[SubjectKey],
    calls: &[CallInventory],
    external_entry: Option<FunctionId>,
) -> ProvenanceResult<Vec<DemandRoute>> {
    let mut routes = vec![None; converged.len()];
    for key in local {
        let index =
            subject_index(converged, key).ok_or(SemanticCompilerFailure::InvalidResolution)?;
        choose_demand_route(&mut routes[index], DemandRoute::default());
    }

    loop {
        let mut changed = false;
        for call in calls {
            if !call_is_upstream_generator(call, external_entry) {
                continue;
            }
            let caller_requirement = call
                .caller_requirement
                .as_ref()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for (downstream_index, downstream) in converged.iter().enumerate() {
                if downstream.requirement != call.site.downstream_requirement {
                    continue;
                }
                let Some(downstream_route) = routes[downstream_index].clone() else {
                    continue;
                };
                let actual = call
                    .actuals
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let selected = actual.selected(downstream.subject.selector)?;
                if selected.unconditional_external {
                    continue;
                }
                let argument_node = call
                    .site
                    .argument_nodes
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                for caller_subject in selected.parameters.datums {
                    let upstream = SubjectKey {
                        requirement: caller_requirement.clone(),
                        subject: caller_subject,
                        leaf: downstream.leaf.clone(),
                    };
                    let upstream_index = subject_index(converged, &upstream)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    let candidate = downstream_route.append(DemandBoundary {
                        call: call.site.call.clone(),
                        argument_node: argument_node.clone(),
                        argument: downstream.subject.ordinal,
                        callee: DemandState::bridge(downstream),
                        caller_continuation: Some(DemandState::bridge(&upstream)),
                    });
                    changed |= choose_demand_route(&mut routes[upstream_index], candidate);
                }
            }
        }
        if !changed {
            break;
        }
    }

    routes
        .into_iter()
        .map(|route| route.ok_or(SemanticCompilerFailure::InvalidResolution))
        .collect()
}

fn reconstruct_direct_routes(
    converged: &[DirectKey],
    local: &[DirectKey],
    direct_calls: &[DirectCallInventory],
    bridge_calls: &[CallInventory],
    bridges: &[SubjectKey],
    bridge_routes: &[DemandRoute],
) -> ProvenanceResult<Vec<DemandRoute>> {
    if bridges.len() != bridge_routes.len() {
        return Err(SemanticCompilerFailure::InvalidResolution);
    }
    let mut routes = vec![None; converged.len()];
    for key in local {
        let index = converged
            .binary_search_by(|candidate| direct_key_cmp(candidate, key))
            .map_err(|_| SemanticCompilerFailure::InvalidResolution)?;
        choose_demand_route(&mut routes[index], DemandRoute::default());
    }

    loop {
        let mut changed = false;

        // A B-failing bridge whose caller U also fails converts to a direct
        // state at this exact boundary.
        for call in bridge_calls {
            if goal_view_discharged(&call.blinded) || goal_view_discharged(&call.unasserted) {
                continue;
            }
            for (bridge, bridge_route) in bridges.iter().zip(bridge_routes) {
                if bridge.requirement != call.site.downstream_requirement {
                    continue;
                }
                let actual = call
                    .actuals
                    .get(bridge.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let selected = actual.selected(bridge.subject.selector)?;
                if selected.unconditional_external {
                    continue;
                }
                let argument_node = call
                    .site
                    .argument_nodes
                    .get(bridge.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                for caller_subject in selected.parameters.datums {
                    let upstream = DirectKey {
                        function: call.site.caller,
                        subject: caller_subject,
                        leaf: bridge.leaf.clone(),
                    };
                    let upstream_index = converged
                        .binary_search_by(|candidate| direct_key_cmp(candidate, &upstream))
                        .map_err(|_| SemanticCompilerFailure::InvalidResolution)?;
                    let candidate = bridge_route.append(DemandBoundary {
                        call: call.site.call.clone(),
                        argument_node: argument_node.clone(),
                        argument: bridge.subject.ordinal,
                        callee: DemandState::bridge(bridge),
                        caller_continuation: Some(DemandState::direct(&upstream)),
                    });
                    changed |= choose_demand_route(&mut routes[upstream_index], candidate);
                }
            }
        }

        let previous = routes.clone();
        for call in direct_calls {
            for (downstream_index, downstream) in converged.iter().enumerate() {
                if downstream.function != call.site.callee {
                    continue;
                }
                let Some(downstream_route) = previous[downstream_index].as_ref() else {
                    continue;
                };
                let actual = call
                    .actuals
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let selected = actual.selected(downstream.subject.selector)?;
                if selected.unconditional_external {
                    continue;
                }
                let argument_node = call
                    .site
                    .argument_nodes
                    .get(downstream.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                for caller_subject in selected.parameters.datums {
                    let upstream = DirectKey {
                        function: call.site.caller,
                        subject: caller_subject,
                        leaf: downstream.leaf.clone(),
                    };
                    let upstream_index = converged
                        .binary_search_by(|candidate| direct_key_cmp(candidate, &upstream))
                        .map_err(|_| SemanticCompilerFailure::InvalidResolution)?;
                    let candidate = downstream_route.append(DemandBoundary {
                        call: call.site.call.clone(),
                        argument_node: argument_node.clone(),
                        argument: downstream.subject.ordinal,
                        callee: DemandState::direct(downstream),
                        caller_continuation: Some(DemandState::direct(&upstream)),
                    });
                    changed |= choose_demand_route(&mut routes[upstream_index], candidate);
                }
            }
        }
        if !changed {
            break;
        }
    }

    routes
        .into_iter()
        .map(|route| route.ok_or(SemanticCompilerFailure::InvalidResolution))
        .collect()
}

fn target_cmp(left: &ProvenanceTarget, right: &ProvenanceTarget) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.callee_subject.cmp(&right.callee_subject))
        .then_with(|| match (&left.requirement, &right.requirement) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(left), Some(right)) => occurrence_cmp(left, right),
        })
        .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
        .then_with(|| left.companions.datums.cmp(&right.companions.datums))
}

fn target_witness_cmp(left: &ProvenanceTarget, right: &ProvenanceTarget) -> Ordering {
    demand_boundaries_cmp(&left.boundaries, &right.boundaries)
        .then_with(|| leaf_cmp(&left.leaf, &right.leaf))
        .then_with(|| carrier_route_cmp(&left.carrier, &right.carrier))
}

fn insert_call_target(
    events: &mut Vec<ProvenanceCallEvent>,
    caller: FunctionId,
    call: &NodePath,
    argument: u32,
    argument_node: &NodePath,
    target: ProvenanceTarget,
) {
    let event = events
        .iter_mut()
        .find(|event| event.caller == caller && event.call == *call && event.argument == argument);
    let event = match event {
        Some(event) => event,
        None => {
            events.push(ProvenanceCallEvent {
                caller,
                call: call.clone(),
                argument,
                argument_node: argument_node.clone(),
                targets: Vec::new(),
                selected_target: 0,
            });
            events.last_mut().expect("a just-pushed event exists")
        }
    };
    match event
        .targets
        .binary_search_by(|candidate| target_cmp(candidate, &target))
    {
        Ok(index) => {
            if target_witness_cmp(&target, &event.targets[index]) == Ordering::Less {
                event.targets[index] = target;
            }
        }
        Err(index) => event.targets.insert(index, target),
    }
}

fn build_call_events(
    calls: &[DirectCallInventory],
    bridge_calls: &[CallInventory],
    direct: &[DirectKey],
    direct_routes: &[DemandRoute],
    bridges: &[SubjectKey],
    bridge_routes: &[DemandRoute],
    reconstructor: &CarrierReconstructor<'_>,
) -> ProvenanceResult<Vec<ProvenanceCallEvent>> {
    if direct.len() != direct_routes.len() || bridges.len() != bridge_routes.len() {
        return Err(SemanticCompilerFailure::InvalidResolution);
    }
    let mut events = Vec::new();
    for call in calls {
        for (demand, route) in direct.iter().zip(direct_routes) {
            if demand.function != call.site.callee {
                continue;
            }
            let selected = call
                .actuals
                .get(demand.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .selected(demand.subject.selector)?;
            if !selected.unconditional_external {
                continue;
            }
            let argument_node = call
                .site
                .argument_nodes
                .get(demand.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let argument = call
                .site
                .arguments
                .get(demand.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let carrier = reconstructor.external_expression_route(
                call.site.caller,
                argument,
                demand.subject.selector,
            )?;
            insert_call_target(
                &mut events,
                call.site.caller,
                &call.site.call,
                demand.subject.ordinal,
                argument_node,
                ProvenanceTarget {
                    kind: ProvenanceDemandKind::Direct,
                    callee_subject: demand.subject,
                    requirement: None,
                    leaf: demand.leaf.clone(),
                    companions: selected.parameters,
                    boundaries: route
                        .append(DemandBoundary {
                            call: call.site.call.clone(),
                            argument_node: argument_node.clone(),
                            argument: demand.subject.ordinal,
                            callee: DemandState::direct(demand),
                            caller_continuation: None,
                        })
                        .boundaries,
                    carrier,
                },
            );
        }
    }
    for call in bridge_calls {
        if goal_view_discharged(&call.blinded) {
            continue;
        }
        for (bridge, route) in bridges.iter().zip(bridge_routes) {
            if bridge.requirement != call.site.downstream_requirement {
                continue;
            }
            let selected = call
                .actuals
                .get(bridge.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .selected(bridge.subject.selector)?;
            if !selected.unconditional_external {
                continue;
            }
            let argument_node = call
                .site
                .argument_nodes
                .get(bridge.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let argument = call
                .site
                .arguments
                .get(bridge.subject.ordinal as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let carrier = reconstructor.external_expression_route(
                call.site.caller,
                argument,
                bridge.subject.selector,
            )?;
            insert_call_target(
                &mut events,
                call.site.caller,
                &call.site.call,
                bridge.subject.ordinal,
                argument_node,
                ProvenanceTarget {
                    kind: ProvenanceDemandKind::Bridge,
                    callee_subject: bridge.subject,
                    requirement: Some(bridge.requirement.clone()),
                    leaf: bridge.leaf.clone(),
                    companions: selected.parameters,
                    boundaries: route
                        .append(DemandBoundary {
                            call: call.site.call.clone(),
                            argument_node: argument_node.clone(),
                            argument: bridge.subject.ordinal,
                            callee: DemandState::bridge(bridge),
                            caller_continuation: None,
                        })
                        .boundaries,
                    carrier,
                },
            );
        }
    }
    events.sort_by(|left, right| {
        left.caller
            .0
            .cmp(&right.caller.0)
            .then_with(|| left.call.components().cmp(right.call.components()))
            .then_with(|| left.argument.cmp(&right.argument))
    });
    for event in &mut events {
        let selected = event
            .targets
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                target_witness_cmp(left, right).then_with(|| target_cmp(left, right))
            })
            .map(|(index, _)| index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        event.selected_target =
            u32::try_from(selected).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
    }
    Ok(events)
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

fn reconstruct_structural_bridges(
    converged: &[StructuralKey],
    local: &[StructuralKey],
    calls: &[CallInventory],
    external_entry: Option<FunctionId>,
) -> ProvenanceResult<Vec<StructuralBridge>> {
    let mut distances = vec![None; converged.len()];
    let mut predecessors = vec![None; converged.len()];
    for key in local {
        let index =
            structural_index(converged, key).ok_or(SemanticCompilerFailure::InvalidResolution)?;
        update_structural_witness(
            &mut distances,
            &mut predecessors,
            index,
            0,
            StructuralPredecessor::Local,
        );
    }

    loop {
        let mut changed = false;
        for call in calls {
            if !call_is_upstream_generator(call, external_entry) {
                continue;
            }
            let caller_requirement = call
                .caller_requirement
                .as_ref()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
                let upstream_index = structural_index(converged, &upstream)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                changed |= update_structural_witness(
                    &mut distances,
                    &mut predecessors,
                    upstream_index,
                    downstream_distance
                        .checked_add(1)
                        .ok_or(SemanticCompilerFailure::CounterOverflow)?,
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
    if bridges.len() != converged.len() {
        return Err(SemanticCompilerFailure::InvalidResolution);
    }
    Ok(bridges)
}

fn subject_bridges_from_routes(
    converged: &[SubjectKey],
    routes: &[DemandRoute],
    local: &[SubjectKey],
) -> ProvenanceResult<Vec<SubjectBridge>> {
    if converged.len() != routes.len() {
        return Err(SemanticCompilerFailure::InvalidResolution);
    }
    converged
        .iter()
        .zip(routes)
        .map(|(key, route)| {
            let predecessor = if let Some(boundary) = route.boundaries.last() {
                if boundary.caller_continuation.as_ref() != Some(&DemandState::bridge(key)) {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                }
                let DemandState::Bridge {
                    requirement,
                    subject,
                    leaf,
                } = &boundary.callee
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                };
                if *leaf != key.leaf || boundary.argument != subject.ordinal {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                }
                SubjectPredecessor::Call {
                    call: boundary.call.clone(),
                    argument: boundary.argument,
                    downstream_requirement: requirement.clone(),
                    downstream_subject: *subject,
                }
            } else {
                if subject_index(local, key).is_none() {
                    return Err(SemanticCompilerFailure::InvalidResolution);
                }
                SubjectPredecessor::Local
            };
            Ok(SubjectBridge {
                requirement: key.requirement.clone(),
                subject: key.subject,
                leaf: key.leaf.clone(),
                predecessor,
                boundaries: route.boundaries.clone(),
            })
        })
        .collect()
}

fn build_call_links(
    calls: &[CallInventory],
    structural: &[StructuralKey],
    subjects: &[SubjectKey],
    external_entry: Option<FunctionId>,
) -> ProvenanceResult<Vec<BridgeCallLink>> {
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
                let actual = call
                    .actuals
                    .get(subject.subject.ordinal as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                compositions.push(CallSubjectComposition {
                    argument: subject.subject.ordinal,
                    callee_subject: subject.subject,
                    caller_dependency: actual.selected(subject.subject.selector)?,
                });
            }
            compositions.sort_by(|left, right| {
                left.argument
                    .cmp(&right.argument)
                    .then_with(|| left.callee_subject.cmp(&right.callee_subject))
                    .then_with(|| {
                        left.caller_dependency
                            .unconditional_external
                            .cmp(&right.caller_dependency.unconditional_external)
                            .then_with(|| {
                                left.caller_dependency
                                    .parameters
                                    .datums
                                    .cmp(&right.caller_dependency.parameters.datums)
                            })
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
                upstream_requirement: call_is_upstream_generator(call, external_entry)
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
    Ok(links)
}

/// Computes both frozen PRV strata, failure-atomic gate scratch, and explicit
/// success dispositions. The checker consumes failures for source acceptance;
/// only the success metadata enters the checked program, and lowering does not
/// read it.
pub(crate) fn freeze_program_provenance(
    functions: &[CheckedFunction],
    context: &ProvenanceContext<'_>,
) -> ProvenanceResult<FrozenProvenanceDependencies> {
    Ok(FrozenProvenanceDependencies {
        functions: dependency_fixed_point(functions, context.nominals, context.external_entry)?,
    })
}

/// Runs both provenance strata on an ordinary unit. FN-9 units instead freeze
/// PRV-1 before their optimistic entailment batch and call
/// [`analyze_program_provenance_with_frozen`].
pub(crate) fn analyze_program_provenance(
    functions: &[CheckedFunction],
    context: &ProvenanceContext<'_>,
) -> ProvenanceResult<ProvenanceAnalysis> {
    let dependencies = freeze_program_provenance(functions, context)?;
    analyze_program_provenance_with_frozen(functions, context, dependencies)
}

/// Finalizes PRV-2/3 from the already-frozen ordinary component pairs and the
/// fixed optimistic complete/U/B fact batch.
pub(crate) fn analyze_program_provenance_with_frozen(
    functions: &[CheckedFunction],
    context: &ProvenanceContext<'_>,
    frozen: FrozenProvenanceDependencies,
) -> ProvenanceResult<ProvenanceAnalysis> {
    let dependencies = frozen.functions;
    let reconstructor = CarrierReconstructor {
        functions,
        summaries: &dependencies,
        nominals: context.nominals,
        external_entry: context.external_entry,
    };
    let unasserted = functions
        .iter()
        .map(|function| function.entailment.unasserted.clone())
        .collect::<Vec<_>>();
    let blinded = functions
        .iter()
        .map(|function| function.entailment.s4_blinded.clone())
        .collect::<Vec<_>>();

    let (local_structural, local_subjects) = local_bridge_seeds(
        functions,
        &dependencies,
        &unasserted,
        &blinded,
        context.nominals,
    )?;
    let calls = build_call_inventory(
        functions,
        &dependencies,
        &unasserted,
        &blinded,
        context.nominals,
    )?;
    let (structural, subjects) = bridge_fixed_point(
        &calls,
        &local_structural,
        &local_subjects,
        context.external_entry,
    )?;
    let subject_routes =
        reconstruct_subject_routes(&subjects, &local_subjects, &calls, context.external_entry)?;
    let direct_calls = build_direct_call_inventory(functions, &dependencies, context.nominals)?;
    let (local_direct, local_rejections, local_leaf_dispositions) = local_gate_seeds(
        functions,
        &dependencies,
        &unasserted,
        &blinded,
        context.nominals,
        &reconstructor,
    )?;
    let direct = direct_demand_fixed_point(&direct_calls, &calls, &subjects, &local_direct)?;
    let direct_routes = reconstruct_direct_routes(
        &direct,
        &local_direct,
        &direct_calls,
        &calls,
        &subjects,
        &subject_routes,
    )?;
    let call_events = build_call_events(
        &direct_calls,
        &calls,
        &direct,
        &direct_routes,
        &subjects,
        &subject_routes,
        &reconstructor,
    )?;
    let call_argument_dispositions =
        build_call_argument_dispositions(&direct_calls, &calls, &call_events)?;
    let structural_bridges = reconstruct_structural_bridges(
        &structural,
        &local_structural,
        &calls,
        context.external_entry,
    )?;
    let subject_bridges = subject_bridges_from_routes(&subjects, &subject_routes, &local_subjects)?;
    let calls = build_call_links(&calls, &structural, &subjects, context.external_entry)?;

    Ok(ProvenanceAnalysis {
        metadata: ProvenanceMetadata {
            functions: dependencies,
            unasserted,
            s4_blinded: blinded,
            structural_bridges,
            subject_bridges,
            calls,
            direct_demands: direct
                .into_iter()
                .zip(direct_routes)
                .map(|(demand, route)| DirectDemand {
                    function: demand.function,
                    subject: demand.subject,
                    leaf: demand.leaf,
                    boundaries: route.boundaries,
                })
                .collect(),
            call_argument_dispositions,
            local_leaf_dispositions,
        },
        failures: ProvenanceFailures {
            local_rejections: local_rejections
                .into_iter()
                .map(|candidate| {
                    (
                        candidate.leaf,
                        candidate.subject,
                        candidate.entry_requirement,
                        candidate.carrier,
                    )
                })
                .collect(),
            call_events,
        },
    })
}
