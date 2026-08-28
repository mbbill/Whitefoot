//! CLM-3's opt-in strict partition over the existing concrete call SCCs and
//! unasserted entailment view.

use super::super::entailment::{
    CallGoalCounterfactual, CallGoalDisposition, ClaimDisposition, PostconditionSchedule,
};
use super::super::goal::first_ephemeral_argument;
use super::super::model::{
    CheckedFunction, FunctionId, StrictClaimIdentity, StrictComponentDisposition,
    StrictComponentMetadata, StrictPartitionMetadata, StrictRootDisposition, StrictRootMetadata,
};
use super::super::{CheckStop, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticRule};
use super::Checker;
use crate::{FixedTerminal, NodePath, SemanticCompilerFailure, TerminalPredicate};

const STRICT_REPAIR: &str =
    "add a dominating real branch or another non-assertion fact source admitted by ENT-3";
const STRICT_EPHEMERAL_REPAIR: &str = "bind that argument or referent value non-consumingly with one preceding ordinary let, establish the complete requirement over that binding with a dominating real branch or another non-assertion fact source admitted by ENT-3, and pass the binding, borrowing it when the parameter mode requires a borrow";

/// A [CLM-3] strict failure. Both variants cite [FN-8]: the strict
/// obligation arms of [OP-4] and [OP-2] were retired because no source
/// program reaches them — U differs from the complete state only by S3, a
/// complete-only callee summary is exactly a claim-dependent one, and a
/// claim anywhere in a demanded closure is a CLM-3 event raised before any
/// U obligation query. `strict_closure_failures` keeps that invariant as an
/// internal-consistency guard.
#[derive(Clone)]
enum StrictFailure {
    Call {
        function: FunctionId,
        node_path: NodePath,
        requires_clause: NodePath,
    },
}

impl StrictFailure {
    fn function(&self) -> FunctionId {
        match self {
            Self::Call { function, .. } => *function,
        }
    }

    fn node_path(&self) -> &NodePath {
        match self {
            Self::Call { node_path, .. } => node_path,
        }
    }
}

#[derive(Default)]
struct StrictRegistrations {
    obligations: Vec<(FunctionId, NodePath)>,
    calls: Vec<(FunctionId, NodePath)>,
    boundary_calls: Vec<(FunctionId, NodePath)>,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Retains every written marker independently of whether a generic
    /// declaration has a reachable concrete instance.
    pub(super) fn strict_declaration_markers(&self) -> Result<Vec<NodePath>, CheckStop> {
        let mut markers = Vec::new();
        for template in &self.function_templates {
            if self
                .tree
                .direct_token_with(
                    template.node,
                    TerminalPredicate::Fixed(FixedTerminal::DenyClaims),
                )?
                .is_some()
            {
                markers.push(self.tree.path(template.node)?.clone());
            }
        }
        Ok(markers)
    }

    /// Runs CLM-3 after ordinary entailment and PRV have succeeded. Successful
    /// U roots are registered in their owning arenas before the sole finish;
    /// returned metadata contains no pre-finish derivation identity.
    pub(super) fn check_strict_partition(
        &self,
        functions: &mut [CheckedFunction],
        schedule: &PostconditionSchedule,
        markers: Vec<NodePath>,
    ) -> Result<StrictPartitionMetadata, CheckStop> {
        let roots = functions
            .iter()
            .filter(|function| function.deny_claims_marker.is_some())
            .map(|function| function.id)
            .collect::<Vec<_>>();
        if roots.is_empty() {
            return Ok(StrictPartitionMetadata {
                markers,
                ..StrictPartitionMetadata::default()
            });
        }
        self.validate_strict_schedule(functions, schedule)?;

        let mut components = schedule
            .components
            .iter()
            .map(|component| {
                let mut direct_claims =
                    component
                        .functions
                        .iter()
                        .flat_map(|function| {
                            functions[function.0 as usize].entailment.claims.iter().map(
                                move |claim| StrictClaimIdentity {
                                    function: *function,
                                    node_path: claim.node_path.clone(),
                                    name: claim.name.clone(),
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                Self::sort_claims(&mut direct_claims);
                StrictComponentMetadata {
                    ordinal: component.ordinal,
                    functions: component.functions.clone(),
                    outgoing: component.outgoing.clone(),
                    may_claims: direct_claims.clone(),
                    direct_claims,
                    demanded: false,
                    disposition: None,
                }
            })
            .collect::<Vec<_>>();

        // Component ordinals are already callee-before-caller, so every
        // outgoing MayClaims set is complete when its caller is visited.
        for ordinal in 0..components.len() {
            let outgoing = components[ordinal].outgoing.clone();
            for callee in outgoing {
                let inherited = components
                    .get(callee as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .may_claims
                    .clone();
                components[ordinal].may_claims.extend(inherited);
            }
            Self::sort_claims(&mut components[ordinal].may_claims);
            components[ordinal].may_claims.dedup_by(|left, right| {
                left.function == right.function
                    && left.node_path == right.node_path
                    && left.name == right.name
            });
        }

        let mut registrations = StrictRegistrations::default();
        let mut root_metadata = Vec::with_capacity(roots.len());
        for root in roots {
            let root_index = root.0 as usize;
            let root_function = functions
                .get(root_index)
                .filter(|function| function.id == root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let root_component = *schedule
                .function_components
                .get(root_index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let closure = Self::strict_closure(root_component, &components)?;

            if let Some(claim) = components[root_component as usize].direct_claims.first() {
                return self.reject_direct_claim(functions, root, claim);
            }
            if let Some(call) = schedule.calls.iter().find(|call| {
                schedule.function_components[call.caller.0 as usize] == root_component
                    && schedule.function_components[call.callee.0 as usize] != root_component
                    && !components[schedule.function_components[call.callee.0 as usize] as usize]
                        .may_claims
                        .is_empty()
            }) {
                let callee_component = schedule.function_components[call.callee.0 as usize];
                let least = components[callee_component as usize]
                    .may_claims
                    .first()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                return self.reject_imported_claim(functions, root, call, least);
            }

            let mut failures = self.strict_closure_failures(functions, &closure, &components)?;

            self.collect_outside_root_failures(
                functions,
                schedule,
                root,
                &closure,
                &mut failures,
                &mut registrations,
            )?;
            failures.sort_by(|left, right| {
                left.function().0.cmp(&right.function().0).then_with(|| {
                    left.node_path()
                        .components()
                        .cmp(right.node_path().components())
                })
            });
            if let Some(failure) = failures.first() {
                return self.reject_strict_failure(functions, root, failure);
            }

            for component in &closure {
                let metadata = components
                    .get_mut(*component as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                metadata.demanded = true;
                metadata.disposition = Some(StrictComponentDisposition::Succeeded);
                for function in &metadata.functions {
                    let entailment = &functions[function.0 as usize].entailment;
                    registrations.obligations.extend(
                        entailment
                            .unasserted
                            .obligations
                            .iter()
                            .map(|outcome| (*function, outcome.node_path.clone())),
                    );
                    registrations.calls.extend(
                        entailment
                            .unasserted
                            .call_goals
                            .iter()
                            .map(|outcome| (*function, outcome.node_path.clone())),
                    );
                }
            }
            root_metadata.push(StrictRootMetadata {
                function: root,
                marker: root_function
                    .deny_claims_marker
                    .clone()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                component: root_component,
                closure,
                disposition: StrictRootDisposition::Succeeded,
            });
        }

        registrations.obligations.sort_by(Self::registration_order);
        registrations.obligations.dedup();
        registrations.calls.sort_by(Self::registration_order);
        registrations.calls.dedup();
        registrations
            .boundary_calls
            .sort_by(Self::registration_order);
        registrations.boundary_calls.dedup();
        for (function, path) in registrations.obligations {
            functions[function.0 as usize]
                .entailment
                .register_strict_obligation(&path)?;
        }
        for (function, path) in registrations.calls {
            functions[function.0 as usize]
                .entailment
                .register_strict_call(&path)?;
        }
        for (function, path) in registrations.boundary_calls {
            functions[function.0 as usize]
                .entailment
                .register_strict_boundary_call(&path)?;
        }

        Ok(StrictPartitionMetadata {
            markers,
            components,
            roots: root_metadata,
            calls: schedule.calls.clone(),
        })
    }

    fn validate_strict_schedule(
        &self,
        functions: &[CheckedFunction],
        schedule: &PostconditionSchedule,
    ) -> Result<(), CheckStop> {
        if schedule.function_components.len() != functions.len()
            || schedule.components.is_empty()
            || schedule
                .components
                .iter()
                .enumerate()
                .any(|(ordinal, component)| {
                    component.ordinal as usize != ordinal
                        || component
                            .functions
                            .iter()
                            .any(|function| function.0 as usize >= functions.len())
                        || component
                            .outgoing
                            .iter()
                            .any(|callee| *callee >= component.ordinal)
                })
            || schedule.calls.iter().any(|call| {
                call.caller.0 as usize >= functions.len()
                    || call.callee.0 as usize >= functions.len()
            })
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(())
    }

    fn sort_claims(claims: &mut [StrictClaimIdentity]) {
        claims.sort_by(|left, right| {
            left.function
                .0
                .cmp(&right.function.0)
                .then_with(|| {
                    left.node_path
                        .components()
                        .cmp(right.node_path.components())
                })
                .then_with(|| left.name.cmp(&right.name))
        });
    }

    fn strict_closure(
        root: u32,
        components: &[StrictComponentMetadata],
    ) -> Result<Vec<u32>, CheckStop> {
        let mut pending = vec![root];
        let mut closure = Vec::new();
        while let Some(component) = pending.pop() {
            if closure.contains(&component) {
                continue;
            }
            let metadata = components
                .get(component as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            closure.push(component);
            pending.extend(metadata.outgoing.iter().rev().copied());
        }
        closure.sort_unstable();
        Ok(closure)
    }

    fn strict_closure_failures(
        &self,
        functions: &[CheckedFunction],
        closure: &[u32],
        components: &[StrictComponentMetadata],
    ) -> Result<Vec<StrictFailure>, CheckStop> {
        let mut failures = Vec::new();
        for component in closure {
            let metadata = components
                .get(*component as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for function in &metadata.functions {
                let checked = functions
                    .get(function.0 as usize)
                    .filter(|checked| checked.id == *function)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                // Every obligation this component owns discharged in the
                // complete state, or the ordinary OP-4/OP-2 rejection fired
                // long before CLM-3 ran. U is that same flow with S3
                // disabled, and the closure reaching here holds no claim,
                // so the two states are identical here. An undischarged U
                // obligation is therefore a checker defect, never invalid
                // source: it is reported as a compiler failure rather than
                // as a source-language rejection [DIAG-1].
                if checked
                    .entailment
                    .unasserted
                    .obligations
                    .iter()
                    .any(|outcome| !outcome.discharged)
                {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                failures.extend(
                    checked
                        .entailment
                        .unasserted
                        .call_goals
                        .iter()
                        .filter(|outcome| {
                            outcome.actual_obligations_ok
                                && outcome.goal_disposition != CallGoalDisposition::Discharged
                        })
                        .map(|outcome| StrictFailure::Call {
                            function: *function,
                            node_path: outcome.node_path.clone(),
                            requires_clause: outcome.requires_clause.clone(),
                        }),
                );
            }
        }
        Ok(failures)
    }

    fn collect_outside_root_failures(
        &self,
        functions: &[CheckedFunction],
        schedule: &PostconditionSchedule,
        root: FunctionId,
        closure: &[u32],
        failures: &mut Vec<StrictFailure>,
        registrations: &mut StrictRegistrations,
    ) -> Result<(), CheckStop> {
        if functions[root.0 as usize].requirements.is_empty() {
            return Ok(());
        }
        for call in schedule.calls.iter().filter(|call| call.callee == root) {
            let caller_component = schedule.function_components[call.caller.0 as usize];
            if closure.contains(&caller_component) {
                continue;
            }
            let caller = functions
                .get(call.caller.0 as usize)
                .filter(|function| function.id == call.caller)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let outcomes = caller
                .entailment
                .unasserted
                .call_goals
                .iter()
                .filter(|outcome| outcome.node_path == call.node_path && outcome.callee == root)
                .collect::<Vec<_>>();
            if outcomes.is_empty() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            let mut all_discharged = true;
            for outcome in outcomes {
                if outcome.goal_disposition != CallGoalDisposition::Discharged {
                    all_discharged = false;
                    failures.push(StrictFailure::Call {
                        function: call.caller,
                        node_path: call.node_path.clone(),
                        requires_clause: outcome.requires_clause.clone(),
                    });
                }
            }
            if all_discharged {
                registrations
                    .boundary_calls
                    .push((call.caller, call.node_path.clone()));
            }
        }
        Ok(())
    }

    fn reject_direct_claim(
        &self,
        functions: &[CheckedFunction],
        root: FunctionId,
        identity: &StrictClaimIdentity,
    ) -> Result<StrictPartitionMetadata, CheckStop> {
        let owner = &functions[identity.function.0 as usize];
        let claim = owner
            .entailment
            .claims
            .iter()
            .find(|claim| claim.node_path == identity.node_path && claim.name == identity.name)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let lifecycle = match claim.disposition {
            ClaimDisposition::Retained => crate::StrictClaimLifecycleDisposition::Retained,
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let location = self.strict_location(&claim.node_path)?;
        Err(CheckStop::source_issue(SemanticIssue {
            rule: SemanticRule::Clm3,
            location,
            kind: SemanticIssueKind::StrictDirectClaim(Box::new(crate::StrictDirectClaimDetail {
                strict_root: functions[root.0 as usize].symbol.clone(),
                concrete_claim_owner: owner.symbol.clone(),
                claim: claim.node_path.clone(),
                name: claim.name.clone(),
                predicate: claim.predicate.clone(),
                justification: claim.justification.raw.clone(),
                lifecycle,
            })),
        }))
    }

    fn reject_imported_claim(
        &self,
        functions: &[CheckedFunction],
        root: FunctionId,
        call: &super::super::entailment::ConcreteCallOccurrence,
        least: &StrictClaimIdentity,
    ) -> Result<StrictPartitionMetadata, CheckStop> {
        let location = self.strict_location(&call.node_path)?;
        Err(CheckStop::source_issue(SemanticIssue {
            rule: SemanticRule::Clm3,
            location,
            kind: SemanticIssueKind::StrictImportedClaim(Box::new(
                crate::StrictImportedClaimDetail {
                    strict_root: functions[root.0 as usize].symbol.clone(),
                    concrete_caller: functions[call.caller.0 as usize].symbol.clone(),
                    call: call.node_path.clone(),
                    concrete_callee: functions[call.callee.0 as usize].symbol.clone(),
                    least_downstream_claim: crate::StrictClaimIdentityDetail {
                        concrete_function: functions[least.function.0 as usize].symbol.clone(),
                        claim: least.node_path.clone(),
                        name: least.name.clone(),
                    },
                },
            )),
        }))
    }

    fn reject_strict_failure(
        &self,
        functions: &[CheckedFunction],
        root: FunctionId,
        failure: &StrictFailure,
    ) -> Result<StrictPartitionMetadata, CheckStop> {
        let strict_root = functions[root.0 as usize].symbol.clone();
        let function = &functions[failure.function().0 as usize];
        let location = self.strict_location(failure.node_path())?;
        match failure {
            StrictFailure::Call {
                node_path,
                requires_clause,
                ..
            } => {
                let outcome = Self::strict_call_outcome(function, node_path, requires_clause)?;
                let callee = &functions[outcome.callee.0 as usize];
                let mechanical_fix = if first_ephemeral_argument(&outcome.goal.root).is_some() {
                    STRICT_EPHEMERAL_REPAIR
                } else {
                    STRICT_REPAIR
                };
                Err(CheckStop::source_issue(SemanticIssue {
                    rule: SemanticRule::Fn8,
                    location,
                    kind: SemanticIssueKind::StrictUndischargedCallRequirement(Box::new(
                        crate::StrictUndischargedCallRequirementDetail {
                            strict_root,
                            concrete_caller: function.symbol.clone(),
                            concrete_callee: callee.symbol.clone(),
                            requires_clause: outcome.requires_clause.clone(),
                            instantiated_goal: outcome.rendered_goal.clone(),
                            disposition: Self::strict_disposition(outcome.goal_disposition)?,
                            view: crate::StrictProofView::Unasserted,
                            mechanical_fix,
                        },
                    )),
                }))
            }
        }
    }

    fn strict_call_outcome<'function>(
        function: &'function CheckedFunction,
        node_path: &NodePath,
        requires_clause: &NodePath,
    ) -> Result<&'function CallGoalCounterfactual, CheckStop> {
        function
            .entailment
            .unasserted
            .call_goals
            .iter()
            .find(|outcome| {
                outcome.node_path == *node_path && outcome.requires_clause == *requires_clause
            })
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    fn strict_disposition(
        disposition: CallGoalDisposition,
    ) -> Result<crate::CallRequirementDisposition, CheckStop> {
        match disposition {
            CallGoalDisposition::Discharged => {
                Err(SemanticCompilerFailure::InvalidResolution.into())
            }
            CallGoalDisposition::Refuted => Ok(crate::CallRequirementDisposition::Refuted),
            CallGoalDisposition::Unproved => Ok(crate::CallRequirementDisposition::Unproved),
        }
    }

    fn strict_location(&self, path: &NodePath) -> Result<SemanticLocation, CheckStop> {
        let node = self
            .tree
            .node_with_path(path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(SemanticLocation::SourceNode(
            path.clone(),
            self.tree.coordinate(node)?,
        ))
    }

    fn registration_order(
        left: &(FunctionId, NodePath),
        right: &(FunctionId, NodePath),
    ) -> std::cmp::Ordering {
        left.0
            .0
            .cmp(&right.0.0)
            .then_with(|| left.1.components().cmp(right.1.components()))
    }
}
