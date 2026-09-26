//! Acceptance of one analyzed function (`design/compiler/acceptance-records.md`):
//! every obligation record answered and discharged.
//!
//! The rule of an undischarged record was fixed where the checker formed it,
//! so the report is a table from that rule to its issue, filled in from the
//! record and the judgment that answered it. [DIAG-1] admits one rule and one
//! location, selected by the order in which obligations are decided.

use crate::syntax::NodeId;
use crate::{
    NodePath, Production, SemanticCompilerFailure, SemanticIssue, SemanticIssueKind,
    SemanticLocation, SemanticRule, StaticObligationDisposition,
};

use super::super::entailment::{
    CallGoalDisposition, FunctionEntailment, PostconditionDisposition,
    SourceProofCertificateFailure,
};
use super::super::goal::first_ephemeral_argument;
use super::super::model::CheckedFunction;
use super::super::obligations::{ObligationRecord, ObligationSubject, RecordAnswer};
use super::{CheckStop, Checker, references};

/// One position in the causal order in which obligations are decided.
///
/// `Child` is a syntax child ordinal, so a plain node path orders a failure
/// exactly where the walk reaches it. `AfterSubtree` is the position
/// immediately after everything one node encloses: it is greater than every
/// child ordinal under that node and still less than the node's following
/// siblings.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProofPosition {
    Child(u32),
    AfterSubtree,
}

impl Checker<'_, '_, '_, '_> {
    /// Accepts `function` exactly when every record is answered and
    /// discharged, and otherwise reports the undischarged record decided
    /// first.
    ///
    /// Body obligations are ordered by where they are decided and then by
    /// their rule's rank; the relations of FN-9, decided at the exits, follow
    /// in source order. A record left unanswered while another is answered
    /// undischarged is that failure's consequence: the engine stops judging
    /// what a failed obligation guards. Unanswered records alone, or a
    /// judgment answering none, are the checker and the engine disagreeing,
    /// and the function is not accepted.
    pub(super) fn entailment_rejection(&self, function: &CheckedFunction) -> Result<(), CheckStop> {
        let entailment = &function.entailment;
        if entailment.answers.len() != function.obligations.len()
            || !entailment.unrecorded.is_empty()
        {
            return Err(SemanticCompilerFailure::ObligationContract.into());
        }
        let mut body = Vec::new();
        let mut relations = Vec::new();
        for (record, answer) in function.obligations.iter().zip(&entailment.answers) {
            let Some(answer) = *answer else {
                continue;
            };
            if answer.discharged(entailment) {
                continue;
            }
            if record.rule == SemanticRule::Fn9 {
                relations.push((record, answer));
            } else {
                body.push((
                    self.proof_position(record, answer, entailment)?,
                    record,
                    answer,
                ));
            }
        }
        // `min_by` keeps the first of several equal minima, so the selection
        // depends only on this order and on record order, never on a hash.
        let selected = body
            .into_iter()
            .min_by(|left, right| {
                left.0.cmp(&right.0).then_with(|| {
                    left.1
                        .rule
                        .definition_rank()
                        .cmp(&right.1.rule.definition_rank())
                })
            })
            .map(|(_, record, answer)| (record, answer))
            .or_else(|| relations.into_iter().next());
        if let Some((record, answer)) = selected {
            return Err(CheckStop::source_issue(
                self.undischarged_issue(function, record, answer)?,
            ));
        }
        if entailment.answers.iter().any(Option::is_none) {
            return Err(SemanticCompilerFailure::ObligationContract.into());
        }
        Ok(())
    }

    /// Where one undischarged body record is decided.
    ///
    /// Every judgment but one is decided where it stands. INV-1's backedge
    /// judgment is the exception: it is proved only after the whole loop body
    /// has been walked, and a body failure that demotes a value to a fresh
    /// full-range atom is exactly what breaks it. Positioning the backedge
    /// after the body it consumes therefore reports the cause rather than the
    /// effect, while INV-1's base judgment stays at the header where it is
    /// decided. A local invariant is decided at the written `use` that owns
    /// its failure.
    fn proof_position(
        &self,
        record: &ObligationRecord,
        answer: RecordAnswer,
        entailment: &FunctionEntailment,
    ) -> Result<Vec<ProofPosition>, CheckStop> {
        let path = match answer {
            RecordAnswer::LoopInvariant(index) => {
                let outcome = entailment
                    .loop_invariants
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if outcome.proof.base && outcome.proof.step == Some(false) {
                    let node = self
                        .tree
                        .node_with_path(&outcome.node_path)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                    if let Some(loop_node) = self.enclosing_loop_node(node)? {
                        let mut components = self
                            .tree
                            .path(loop_node)?
                            .components()
                            .iter()
                            .copied()
                            .map(ProofPosition::Child)
                            .collect::<Vec<_>>();
                        components.push(ProofPosition::AfterSubtree);
                        return Ok(components);
                    }
                }
                &outcome.node_path
            }
            RecordAnswer::SourceProof(index) => entailment
                .source_proofs
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .rejection_node_path(),
            RecordAnswer::Obligation(_)
            | RecordAnswer::CallGoal(_)
            | RecordAnswer::Postcondition(_)
            | RecordAnswer::Uninhabited => &record.site,
        };
        Ok(path
            .components()
            .iter()
            .copied()
            .map(ProofPosition::Child)
            .collect())
    }

    /// The enclosing `loop_stmt` or `for_stmt` of one loop-header invariant.
    ///
    /// A header invariant is always written inside its loop statement, so the
    /// ancestor exists for every well-formed tree. The absent case keeps the
    /// caller total and simply leaves the invariant at its own position.
    fn enclosing_loop_node(&self, node: NodeId) -> Result<Option<NodeId>, SemanticCompilerFailure> {
        let mut current = node;
        loop {
            let production = self.tree.production(current)?;
            if production == Production::LoopStmt || production == Production::ForStmt {
                return Ok(Some(current));
            }
            match self.tree.parent(current)? {
                Some(parent) => current = parent,
                None => return Ok(None),
            }
        }
    }

    fn source_location(&self, path: &NodePath) -> Result<SemanticLocation, CheckStop> {
        let node = self
            .tree
            .node_with_path(path)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(SemanticLocation::SourceNode(
            path.clone(),
            self.tree.coordinate(node)?,
        ))
    }

    /// The issue one undischarged record reports: its rule selects the kind,
    /// and the record with the judgment that answered it fill it in.
    fn undischarged_issue(
        &self,
        function: &CheckedFunction,
        record: &ObligationRecord,
        answer: RecordAnswer,
    ) -> Result<SemanticIssue, CheckStop> {
        let entailment = &function.entailment;
        let obligation = |index: usize| {
            let outcome = entailment
                .obligations
                .get(index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let residual = outcome
                .residual
                .clone()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let disposition = if outcome.refuted {
                StaticObligationDisposition::Refuted
            } else {
                StaticObligationDisposition::Unproved
            };
            Ok::<_, SemanticCompilerFailure>((outcome, residual, disposition))
        };
        let issue = |rule, location, kind| SemanticIssue {
            rule,
            location,
            kind,
            request: None,
        };
        Ok(match (record.rule, answer) {
            (SemanticRule::Inv1, RecordAnswer::LoopInvariant(index)) => {
                self.undischarged_loop_invariant(entailment, index)?
            }
            (SemanticRule::Inv1 | SemanticRule::Prf1, RecordAnswer::SourceProof(index)) => {
                self.undischarged_source_proof(entailment, index)?
            }
            (SemanticRule::Op4, RecordAnswer::Obligation(index)) => {
                let (outcome, residual, _) = obligation(index)?;
                issue(
                    SemanticRule::Op4,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedBoundsObligation {
                        residual,
                        mechanical_fix: "when the relation must hold, establish the residual with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise restructure the access",
                    },
                )
            }
            (SemanticRule::Op2, RecordAnswer::Obligation(index)) => {
                let (outcome, residual, disposition) = obligation(index)?;
                issue(
                    SemanticRule::Op2,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedIntegerDomainObligation {
                        residual,
                        disposition,
                        mechanical_fix: "when the relation must hold, establish the fixed `.defined` normalization with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when its false edge is intended program behavior; otherwise use an available total non-exact row or restructure the arithmetic",
                    },
                )
            }
            (SemanticRule::Op9, RecordAnswer::Obligation(index)) => {
                let (outcome, residual, _) = obligation(index)?;
                issue(
                    SemanticRule::Op9,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedAllocationFitObligation {
                        residual,
                        mechanical_fix: "the allocation's own size arithmetic must stay inside u64: bound the count with a verified requirement, a source invariant, or explicit finite proof steps; use a dominating branch only when the refusal is intended program behavior; otherwise restructure the allocation",
                    },
                )
            }
            (SemanticRule::Op6, RecordAnswer::Obligation(index)) => {
                let (outcome, residual, disposition) = obligation(index)?;
                issue(
                    SemanticRule::Op6,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedConversionDomainObligation {
                        residual,
                        disposition,
                        mechanical_fix: "establish this cvt.defined domain with a verified requirement, an integer range invariant, or explicit finite proof steps; use a dominating cvt.defined condition when refusal is intended behavior, or use cvt.checked to return the failed conversion",
                    },
                )
            }
            (rule @ (SemanticRule::Eff5 | SemanticRule::Op11), RecordAnswer::Obligation(index)) => {
                let (outcome, residual, _) = obligation(index)?;
                issue(
                    rule,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedCallSeparation {
                        residual,
                        mechanical_fix: "prove the two positions distinct before this call, or pass one of them",
                    },
                )
            }
            (SemanticRule::Ref2, RecordAnswer::Obligation(index)) => {
                let (outcome, _, _) = obligation(index)?;
                let ObligationSubject::Source {
                    family: super::super::entailment::ObligationFamily::ReferencePreservation(query),
                    ..
                } = record.subject
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let use_site = function
                    .call_separations
                    .get(query as usize)
                    .and_then(|query| query.reference_use.as_ref())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                issue(
                    SemanticRule::Ref2,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::InvalidReferenceUse {
                        binder: use_site.binder.clone(),
                        event: use_site.event,
                        mechanical_fix: references::REF2_FORM_AGAIN,
                    },
                )
            }
            (SemanticRule::Ref4, RecordAnswer::Obligation(index)) => {
                let (outcome, residual, _) = obligation(index)?;
                issue(
                    SemanticRule::Ref4,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedRangeFormationObligation {
                        residual,
                        mechanical_fix: "establish lo <= hi and hi <= x.len with a verified requirement, a source invariant, or explicit finite proof steps; otherwise restructure the range",
                    },
                )
            }
            // [OP-14] `free_empty` has its own site: "An undischarged
            // obligation is a hard error citing OP-14 at the complete
            // `call`, rendering the residual".
            (SemanticRule::Op14, RecordAnswer::CallGoal(index)) => {
                let outcome = entailment
                    .call_goals
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                issue(
                    SemanticRule::Op14,
                    self.source_location(&outcome.node_path)?,
                    SemanticIssueKind::UndischargedEmptyRunRelease {
                        residual: outcome.rendered_goal.clone(),
                        mechanical_fix: "empty the window and establish its zero length at this point; otherwise take every element out and consume it",
                    },
                )
            }
            (SemanticRule::Fn8, RecordAnswer::CallGoal(index)) => {
                self.undischarged_call_requirement(entailment, index)?
            }
            (SemanticRule::Fn9, RecordAnswer::Postcondition(index)) => {
                self.undischarged_postcondition(function, index)?
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        })
    }

    fn undischarged_loop_invariant(
        &self,
        entailment: &FunctionEntailment,
        index: usize,
    ) -> Result<SemanticIssue, CheckStop> {
        let outcome = entailment
            .loop_invariants
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let obligation = if !outcome.proof.base {
            crate::LoopInvariantProofObligation::Base
        } else if outcome.proof.step == Some(false) {
            crate::LoopInvariantProofObligation::Backedge
        } else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let (mechanical_fix, required_relation) = match obligation {
            crate::LoopInvariantProofObligation::Base => (
                "weaken or correct this invariant, or establish the missing facts before the loop so the invariant holds at the first loop header",
                outcome.base_target.clone(),
            ),
            crate::LoopInvariantProofObligation::Backedge => (
                "strengthen the invariant prefix, weaken or correct this invariant, or establish the missing body facts so every reachable normal fallthrough preserves it at the next loop header",
                outcome.backedge_target.clone(),
            ),
        };
        Ok(SemanticIssue {
            rule: SemanticRule::Inv1,
            location: self.source_location(&outcome.node_path)?,
            kind: SemanticIssueKind::UndischargedLoopInvariant {
                name: outcome.name.clone(),
                obligation,
                required_relation,
                mechanical_fix,
            },
            request: None,
        })
    }

    fn undischarged_source_proof(
        &self,
        entailment: &FunctionEntailment,
        index: usize,
    ) -> Result<SemanticIssue, CheckStop> {
        let outcome = entailment
            .source_proofs
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let location = self.source_location(outcome.rejection_node_path())?;
        if let Some(failure) = outcome.check.target_failure {
            let (reason, mechanical_fix) = match failure {
                SourceProofCertificateFailure::ArithmeticOverflow => (
                    "the invariant target exceeds the i128 proof domain after current value images are substituted",
                    "split or rescale the invariant so its normalized current-value coefficients and constant fit i128",
                ),
                SourceProofCertificateFailure::FormationCapacity => (
                    "the invariant target exceeds a fixed affine formation capacity after current value images are substituted",
                    "split the invariant into smaller local invariants whose normalized current-value shapes fit the fixed capacities",
                ),
                SourceProofCertificateFailure::RepeatedUse { .. }
                | SourceProofCertificateFailure::UseCapacity { .. }
                | SourceProofCertificateFailure::NonlinearResidual
                | SourceProofCertificateFailure::InvalidFactor { .. } => {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            };
            return Ok(SemanticIssue {
                rule: SemanticRule::Inv1,
                location,
                kind: SemanticIssueKind::InvalidInvariant {
                    reason,
                    mechanical_fix,
                },
                request: None,
            });
        }
        if !outcome.certificate_written {
            if !outcome.check.premises.is_empty()
                || outcome.check.source_failure.is_some()
                || outcome.check.certificate_failure.is_some()
                || outcome.check.residual_failure.is_some()
                || outcome.check.redundant
                || outcome.check.combination
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            return Ok(SemanticIssue {
                rule: SemanticRule::Inv1,
                location,
                kind: SemanticIssueKind::UndischargedLocalInvariant {
                    name: outcome.name.clone(),
                    mechanical_fix: "weaken or correct this invariant, or establish the missing facts before this statement so AUTO proves its target in the entering context",
                },
                request: None,
            });
        }
        let failure_obligation = |failure| match failure {
            SourceProofCertificateFailure::RepeatedUse { first, repeated } => {
                crate::SourceProofObligation::RepeatedUse { first, repeated }
            }
            SourceProofCertificateFailure::UseCapacity { maximum, actual } => {
                crate::SourceProofObligation::UseCapacity { maximum, actual }
            }
            SourceProofCertificateFailure::ArithmeticOverflow => {
                crate::SourceProofObligation::CertificateArithmeticOverflow
            }
            SourceProofCertificateFailure::FormationCapacity => {
                crate::SourceProofObligation::CertificateFormationCapacity
            }
            SourceProofCertificateFailure::InvalidFactor { use_index } => {
                crate::SourceProofObligation::InvalidUseFactor { use_index }
            }
            SourceProofCertificateFailure::NonlinearResidual => {
                crate::SourceProofObligation::NonlinearCertificateSum
            }
        };
        let obligation = if let Some(failure) = outcome.check.source_failure {
            failure_obligation(failure)
        } else if outcome.check.redundant {
            crate::SourceProofObligation::RedundantUseBlock
        } else if let Some(failure) = outcome.check.certificate_failure {
            failure_obligation(failure)
        } else if let Some(index) = outcome.check.first_unproved_premise {
            crate::SourceProofObligation::Premise(index)
        } else if let Some(failure) = outcome.check.residual_failure {
            failure_obligation(failure)
        } else if !outcome.check.combination {
            crate::SourceProofObligation::Combination
        } else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let mechanical_fix = match obligation {
            crate::SourceProofObligation::Premise(_) => {
                "establish this use relation from facts already available before the invariant statement, or replace it with a relation AUTO can prove in that same entering context"
            }
            crate::SourceProofObligation::Combination => {
                "rewrite the invariant target, use relations, or explicit positive factors so their source-order weighted sum leaves a residual proved by the fixed direct L0 or interval rule"
            }
            crate::SourceProofObligation::RedundantUseBlock => {
                "remove the use block; AUTO already proves this invariant target from the same entering context in this specification version"
            }
            crate::SourceProofObligation::RepeatedUse { .. } => {
                "replace repeated normalized use relations with one use carrying their combined explicit positive factor"
            }
            crate::SourceProofObligation::UseCapacity { .. } => {
                "split this local certificate into named intermediate invariants so every written use list is within the fixed structural capacity"
            }
            crate::SourceProofObligation::CertificateArithmeticOverflow => {
                "split or rescale this certificate so every source-order proof-domain coefficient and constant operation fits i128"
            }
            crate::SourceProofObligation::CertificateFormationCapacity => {
                "split this certificate into smaller named intermediate invariants whose canonical affine shapes fit the fixed formation capacities"
            }
            crate::SourceProofObligation::InvalidUseFactor { .. } => {
                "write a canonical positive bare-decimal factor, or omit the factor when it is one"
            }
            crate::SourceProofObligation::NonlinearCertificateSum => {
                "the multiplied operand must be one the checker holds as a single value — a parameter or a call result — because a locally derived one is expanded into its own operands and no admitted product then matches the sum; take it as a parameter, or scale the premise by a bare decimal instead"
            }
        };
        Ok(SemanticIssue {
            rule: SemanticRule::Prf1,
            location,
            kind: SemanticIssueKind::UndischargedSourceProof {
                name: outcome.name.clone(),
                obligation,
                mechanical_fix,
            },
            request: None,
        })
    }

    fn undischarged_call_requirement(
        &self,
        entailment: &FunctionEntailment,
        index: usize,
    ) -> Result<SemanticIssue, CheckStop> {
        let outcome = entailment
            .call_goals
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let signature = self
            .signatures
            .get(outcome.callee.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let disposition = match outcome.disposition {
            CallGoalDisposition::Discharged => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            CallGoalDisposition::Refuted => crate::CallRequirementDisposition::Refuted,
            CallGoalDisposition::Unproved => crate::CallRequirementDisposition::Unproved,
        };
        let location = self.source_location(&outcome.node_path)?;
        let requires_clause = self.node_location(&outcome.requires_clause)?;
        let mechanical_fix = if first_ephemeral_argument(&outcome.goal.root).is_some() {
            "bind that argument or referent value with one preceding ordinary let, establish the entire instantiated requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow"
        } else {
            "when the call is required to succeed, establish the entire instantiated callee requirement with a verified requirement, a source invariant, or explicit finite proof steps before the call; use a dominating branch only when rejection is intended program behavior; otherwise restructure the call"
        };
        Ok(SemanticIssue {
            rule: SemanticRule::Fn8,
            location,
            kind: SemanticIssueKind::UndischargedCallRequirement(Box::new(
                crate::UndischargedCallRequirementDetail {
                    concrete_callee: self.render_function_instance(signature)?,
                    requires_clause,
                    instantiated_goal: outcome.rendered_goal.clone(),
                    disposition,
                    mechanical_fix,
                },
            )),
            request: None,
        })
    }

    /// [FN-9] the first failed exit of one relation, or its missing exit.
    fn undischarged_postcondition(
        &self,
        function: &CheckedFunction,
        index: usize,
    ) -> Result<SemanticIssue, CheckStop> {
        let proof = function
            .entailment
            .postconditions
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(exit) = proof
            .exits
            .iter()
            .find(|exit| exit.disposition != PostconditionDisposition::Discharged)
        else {
            if !proof.exits.is_empty() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            return Ok(SemanticIssue {
                rule: SemanticRule::Fn9,
                location: self.source_location(&proof.selector)?,
                kind: SemanticIssueKind::NoSelectedNormalExit {
                    residual: "no selected normal exit",
                },
                request: None,
            });
        };
        let disposition = match exit.disposition {
            PostconditionDisposition::Discharged => {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            PostconditionDisposition::Refuted => crate::PostconditionProofDisposition::Refuted,
            PostconditionDisposition::Unproved => crate::PostconditionProofDisposition::Unproved,
        };
        Ok(SemanticIssue {
            rule: SemanticRule::Fn9,
            location: self.source_location(&exit.statement)?,
            kind: SemanticIssueKind::UndischargedPostcondition(Box::new(
                crate::UndischargedPostconditionDetail {
                    concrete_function: match self
                        .signatures
                        .get(function.id.0 as usize)
                        .filter(|signature| signature.declaration == function.declaration)
                    {
                        Some(signature) => self.render_function_instance(signature)?,
                        None => function.name.clone(),
                    },
                    postcondition: self.node_location(&proof.block)?,
                    conjunct: proof.relation_ordinal,
                    selector: self.node_location(&proof.selector)?,
                    relation: exit.residual.clone(),
                    disposition,
                },
            )),
            request: None,
        })
    }
}
