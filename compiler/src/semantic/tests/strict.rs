//! Ordinary compiler regressions for the CLM-3 strict partition. These are
//! intentionally independent of the protected conformance corpus.

use crate::{
    SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule,
    StrictClaimLifecycleDisposition, StrictProofView,
};

use super::super::entailment::{ProofView, StrictDerivationRootKind};
use super::super::model::StrictProgramStartDisposition;
use super::{with_semantics, with_semantics_dark};

fn rejection(source: &[u8], rule: SemanticRule, cited: &str) -> SemanticIssue {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?} rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        let SemanticLocation::SourceNode(path, coordinate) = issue.location() else {
            panic!("strict rejection must cite an existing source node");
        };
        assert!(!path.components().is_empty());
        let start = usize::try_from(coordinate.start().value()).expect("offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits usize");
        assert_eq!(&source[start..end], cited.as_bytes());
        issue
    })
}

#[test]
fn a_direct_unreachable_redundant_claim_rejects_at_the_claim_with_full_identity() {
    let source = br#"deny_claims fn main() -> own unit traps {
  let never = False();
  if never {
    let same = ieq(0_u64, 0_u64);
    claim hidden: same because "still structural";
  }
  return unit;
}
"#;
    let issue = rejection(
        source,
        SemanticRule::Clm3,
        "claim hidden: same because \"still structural\";",
    );
    let SemanticIssueKind::StrictDirectClaim(detail) = issue.kind() else {
        panic!("expected direct-claim detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "main");
    assert_eq!(detail.concrete_claim_owner, "main");
    assert_eq!(detail.name, "hidden");
    assert_eq!(detail.predicate, "same");
    assert_eq!(detail.justification, "still structural");
    assert_eq!(detail.lifecycle, StrictClaimLifecycleDisposition::Redundant);
    let SemanticLocation::SourceNode(path, _) = issue.location() else {
        unreachable!()
    };
    assert_eq!(&detail.claim, path);
}

#[test]
fn the_first_root_call_importing_a_generic_claim_carries_the_least_claim_identity() {
    let source = br#"fn claimed<T: Int>(value: own T) -> own T traps {
  let flag = True();
  claim generic_seed: flag because "one concrete seed";
  return value;
}

fn relay<T: Int>(value: own T) -> own T traps {
  let result = claimed<T>(value: value);
  return result;
}

deny_claims fn main() -> own unit traps {
  let first = relay<u64>(value: 2_u64);
  let second = relay<u8>(value: 3_u8);
  return unit;
}
"#;
    let issue = rejection(source, SemanticRule::Clm3, "relay<u64>(value: 2_u64)");
    let SemanticIssueKind::StrictImportedClaim(detail) = issue.kind() else {
        panic!("expected imported-claim detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "main");
    assert_eq!(detail.concrete_caller, "main");
    assert!(detail.concrete_callee.starts_with("relay$instance$"));
    assert!(
        detail
            .least_downstream_claim
            .concrete_function
            .starts_with("claimed$instance$")
    );
    assert_eq!(detail.least_downstream_claim.name, "generic_seed");
    let SemanticLocation::SourceNode(path, _) = issue.location() else {
        unreachable!()
    };
    assert_eq!(&detail.call, path);
}

#[test]
fn a_mutual_component_converges_one_claim_seed_to_the_root_import() {
    let source = br#"fn left(value: own u64) -> own u64 traps {
  let done = ieq(value, 0_u64);
  if done {
    let flag = True();
    claim cycle_seed: flag because "component seed";
    return value;
  } else {
    let result = right(value: value);
    return result;
  }
}

fn right(value: own u64) -> own u64 traps {
  let result = left(value: value);
  return result;
}

deny_claims fn strict_root(value: own u64) -> own u64 traps {
  let result = right(value: value);
  return result;
}

fn main() -> own unit traps {
  let result = strict_root(value: 0_u64);
  return unit;
}
"#;
    let issue = rejection(source, SemanticRule::Clm3, "right(value: value)");
    let SemanticIssueKind::StrictImportedClaim(detail) = issue.kind() else {
        panic!("expected imported claim: {issue:?}");
    };
    assert_eq!(detail.strict_root, "strict_root");
    assert_eq!(detail.concrete_callee, "right");
    assert_eq!(detail.least_downstream_claim.concrete_function, "left");
    assert_eq!(detail.least_downstream_claim.name, "cycle_seed");
}

/// No writer assertion can authorize a strict bounds query.
///
/// v0.31 stated this through the [ENT-3.S2] blinding: a body `check` was
/// legal inside a demanded closure and its fact was merely invisible in the
/// unasserted view, so the subscript stayed undischarged and OP-4 reported a
/// strict residual. v0.32 retires the body check, so the only writer
/// assertion left is a `claim`, and [CLM-3] refuses it outright at the claim
/// itself — `deny_claims` now means literally "no writer assertion in the
/// demanded closure" rather than "no *named* one". The claim here is
/// load-bearing and reachable, which is what separates this case from the
/// unreachable-redundant one above.
#[test]
fn a_load_bearing_claim_cannot_authorize_a_strict_bounds_query() {
    let asserted =
        br#"deny_claims fn read(values: own array<u8, 4>, index: own u64) -> own u8 traps {
  let room = len(values);
  let inside = ilt(index, room);
  claim body_authorization: inside because "body authorization";
  return values[index];
}

fn main() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  let value = read(values: move values, index: 0_u64);
  return unit;
}
"#;
    let issue = rejection(
        asserted,
        SemanticRule::Clm3,
        "claim body_authorization: inside because \"body authorization\";",
    );
    let SemanticIssueKind::StrictDirectClaim(detail) = issue.kind() else {
        panic!("expected direct-claim detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "read");
    assert_eq!(detail.concrete_claim_owner, "read");
    assert_eq!(detail.name, "body_authorization");
    assert_eq!(detail.lifecycle, StrictClaimLifecycleDisposition::Retained);
}

/// The same law where the obligation is a callee requirement rather than a
/// subscript: the assertion the strict root would have leaned on is refused
/// at the claim, not deferred to the U view of the call.
#[test]
fn a_load_bearing_claim_cannot_authorize_a_strict_required_call() {
    let asserted = br#"fn required(value: own u64, limit: own u64) -> own unit pure requires {
  let allowed = ilt(value, limit);
  check allowed else trap "required";
} {
  return unit;
}

deny_claims fn forward(value: own u64, limit: own u64) -> own unit traps {
  let allowed = ilt(value, limit);
  claim body_authorization: allowed because "body authorization";
  required(value: value, limit: limit);
  return unit;
}

fn main() -> own unit traps {
  forward(value: 0_u64, limit: 1_u64);
  return unit;
}
"#;
    let issue = rejection(
        asserted,
        SemanticRule::Clm3,
        "claim body_authorization: allowed because \"body authorization\";",
    );
    let SemanticIssueKind::StrictDirectClaim(detail) = issue.kind() else {
        panic!("expected direct-claim detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "forward");
    assert_eq!(detail.concrete_claim_owner, "forward");
    assert_eq!(detail.name, "body_authorization");
    assert_eq!(detail.lifecycle, StrictClaimLifecycleDisposition::Retained);
}

/// A downstream authorization is still attributed to the real leaf that
/// wrote it, not to the strict root that imported it. v0.31 saw this as a
/// blinded body check leaving the leaf's own subscript undischarged; v0.32
/// sees the same authorship through [CLM-3]'s import event, which names the
/// least downstream claim's function and the root's own call site.
#[test]
fn a_downstream_authorization_is_reported_against_the_real_leaf() {
    let source = br#"fn leaf(values: own array<u8, 4>, index: own u64) -> own u8 traps {
  let room = len(values);
  let inside = ilt(index, room);
  claim leaf_authorization: inside because "leaf authorization";
  return values[index];
}

deny_claims fn root(values: own array<u8, 4>, index: own u64) -> own u8 traps {
  let value = leaf(values: move values, index: index);
  return value;
}

fn main() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  let value = root(values: move values, index: 0_u64);
  return unit;
}
"#;
    let issue = rejection(
        source,
        SemanticRule::Clm3,
        "leaf(values: move values, index: index)",
    );
    let SemanticIssueKind::StrictImportedClaim(detail) = issue.kind() else {
        panic!("expected downstream imported-claim detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "root");
    assert_eq!(detail.concrete_caller, "root");
    assert_eq!(detail.concrete_callee, "leaf");
    assert_eq!(detail.least_downstream_claim.concrete_function, "leaf");
    assert_eq!(detail.least_downstream_claim.name, "leaf_authorization");
}

#[test]
fn the_entry_wrapper_cannot_authorize_its_own_opaque_conjunction() {
    let source = br#"deny_claims fn main() -> own unit pure requires {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let together = band(first, second);
  check together else trap "entry conjunction";
} {
  return unit;
}
"#;
    let issue = rejection(
        source,
        SemanticRule::Fn8,
        "check together else trap \"entry conjunction\";",
    );
    let SemanticIssueKind::StrictProgramStartRequirement(detail) = issue.kind() else {
        panic!("expected strict program-start detail: {issue:?}");
    };
    // Exhaustive destructuring locks the program-start payload to the fields
    // authorized by FN-8; unlike a call failure, it carries no repair advice.
    let crate::StrictProgramStartRequirementDetail {
        strict_root,
        concrete_function,
        final_check,
        instantiated_goal,
        disposition: _,
        view,
    } = detail.as_ref();
    assert_eq!(strict_root, "main");
    assert_eq!(concrete_function, "main");
    assert_eq!(*view, StrictProofView::Unasserted);
    let SemanticLocation::SourceNode(path, _) = issue.location() else {
        unreachable!()
    };
    assert_eq!(final_check, path);
    assert!(instantiated_goal.contains("Boolean(And)"));
}

#[test]
fn an_outside_caller_must_prove_a_marked_root_requirement_in_its_own_u_view() {
    let source =
        br#"deny_claims fn guarded(value: own u64, limit: own u64) -> own unit pure requires {
  let allowed = ilt(value, limit);
  check allowed else trap "guarded";
} {
  return unit;
}

fn ordinary(value: own u64, limit: own u64) -> own unit traps {
  let allowed = ilt(value, limit);
  claim ordinary_authorization: allowed because "ordinary authorization";
  guarded(value: value, limit: limit);
  return unit;
}

fn main() -> own unit traps {
  ordinary(value: 0_u64, limit: 1_u64);
  return unit;
}
"#;
    let issue = rejection(
        source,
        SemanticRule::Fn8,
        "guarded(value: value, limit: limit)",
    );
    let SemanticIssueKind::StrictUndischargedCallRequirement(detail) = issue.kind() else {
        panic!("expected outside-caller strict call detail: {issue:?}");
    };
    assert_eq!(detail.strict_root, "guarded");
    assert_eq!(detail.concrete_caller, "ordinary");
    assert_eq!(detail.concrete_callee, "guarded");
}

#[test]
fn an_outside_call_does_not_demand_its_actual_expression_obligations_in_u() {
    let source = br#"deny_claims fn sink(value: own u8) -> own unit pure requires {
  let valid = ieq(0_u64, 0_u64);
  check valid else trap "constant boundary";
} {
  return unit;
}

fn ordinary(values: own array<u8, 4>, index: own u64) -> own unit traps {
  let room = len(values);
  let inside = ilt(index, room);
  claim ordinary_actual_authorization: inside because "ordinary actual authorization";
  sink(value: values[index]);
  return unit;
}

fn main() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  ordinary(values: move values, index: 0_u64);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("outside actual U failure must not flow upward: {outcome:?}");
        };
        let ordinary = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "ordinary")
            .expect("ordinary caller exists");
        assert!(
            ordinary
                .entailment
                .unasserted
                .obligations
                .iter()
                .any(|outcome| {
                    !outcome.discharged
                        && outcome.residual.as_deref() == Some("index < len(values)")
                })
        );
        assert!(
            ordinary
                .entailment
                .strict_roots
                .iter()
                .any(|root| root.kind == StrictDerivationRootKind::CallGoal)
        );
    });
}

#[test]
fn a_real_value_branch_and_verified_result_pass_with_remapped_strict_roots() {
    let source = br#"fn relay(value: own u64) -> own u64 pure ensures result {
  check ieq(result, value) else trap "relay result";
} {
  return value;
}

fn accept(value: own u64, limit: own u64) -> own u64 pure requires {
  let allowed = ile(value, limit);
  check allowed else trap "accepted bound";
} {
  return value;
}

deny_claims fn main() -> own unit pure requires {
  let valid = ieq(0_u64, 0_u64);
  check valid else trap "entry relation";
} {
  let candidate = 7_u64;
  let prior = 0_u64;
  let limit = 8_u64;
  let fits = ile(candidate, limit);
  let bounded = if fits {
    give candidate;
  } else {
    give prior;
  }
  let relayed = relay(value: bounded);
  let accepted = accept(value: relayed, limit: limit);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("real strict path must check completely: {outcome:?}");
        };
        let strict = &program.data.strict_partition;
        assert_eq!(strict.roots.len(), 1);
        assert_eq!(
            strict.roots[0].program_start,
            StrictProgramStartDisposition::Discharged
        );
        assert!(
            strict
                .components
                .iter()
                .all(|component| component.may_claims.is_empty())
        );
        assert!(
            strict
                .components
                .iter()
                .filter(|component| component.demanded)
                .all(|component| component.disposition.is_some())
        );
        assert_eq!(strict.calls, program.data.postcondition_schedule.calls);
        for function in &program.data.functions {
            super::entailment::validate_derivations(&function.entailment);
        }
        let main = &program.data.functions[program.data.main.0 as usize];
        assert!(!main.entailment.strict_roots.is_empty());
        for root in &main.entailment.strict_roots {
            assert!(root.derivation.0 < main.entailment.derivations.nodes.len() as u32);
            assert_eq!(
                main.entailment.derivations.node_views[root.derivation.0 as usize],
                ProofView::Unasserted
            );
        }
        assert!(
            main.entailment
                .strict_roots
                .iter()
                .any(|root| { root.kind == StrictDerivationRootKind::ProgramStart })
        );
    });
}

#[test]
fn strict_closures_do_not_flow_upward_into_an_ordinary_claiming_caller() {
    let source = br#"deny_claims fn identity<T: Int>(value: own T) -> own T pure {
  return value;
}

fn ordinary() -> own u64 traps {
  let flag = True();
  claim caller_only: flag because "outside every outgoing closure";
  let small = identity<u8>(value: 3_u8);
  let large = identity<u64>(value: 4_u64);
  return large;
}

fn main() -> own unit traps {
  let value = ordinary();
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("upward near miss must remain accepted: {outcome:?}");
        };
        assert_eq!(program.data.strict_partition.roots.len(), 2);
        assert_eq!(program.data.claim_ledger.entries.len(), 1);
        assert_eq!(program.data.claim_ledger.entries[0].name, "caller_only");
    });
}

#[test]
fn removing_the_marker_preserves_the_ordinary_diagnostic_and_dark_observability() {
    let source = br#"fn read(values: own array<u8, 4>, index: own u64) -> own u8 traps {
  let room = len(values);
  let inside = ilt(index, room);
  claim ordinary_authorization: inside because "ordinary authorization";
  return values[index];
}

fn main() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  let value = read(values: move values, index: 0_u64);
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("unmarked ordinary source must retain v0.28 acceptance: {outcome:?}");
        };
        assert!(program.data.strict_partition.roots.is_empty());
        assert!(program.data.strict_partition.components.is_empty());
    });
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("dark entailment observability must remain intact: {outcome:?}");
        };
        assert!(program.data.strict_partition.roots.is_empty());
    });
}

#[test]
fn the_dark_hook_still_reports_a_refuted_direct_claim_as_failure_atomic_clm3() {
    let source = br#"deny_claims fn main() -> own unit traps {
  let same = ieq(0_u64, 0_u64);
  let different = ine(0_u64, 0_u64);
  if same {
    claim refuted_seed: different because "dark lifecycle observation";
  }
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("dark marker failure must publish no checked program: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm3);
        let SemanticIssueKind::StrictDirectClaim(detail) = issue.kind() else {
            panic!("expected direct strict claim: {issue:?}");
        };
        assert_eq!(detail.name, "refuted_seed");
        assert_eq!(detail.lifecycle, StrictClaimLifecycleDisposition::Refuted);
    });
}

#[test]
fn ordinary_fn3_contract_validation_precedes_a_marker_failure() {
    let source = br#"contract Repeated {
  fn value() -> own i32 pure;
  fn value() -> own i32 pure;
}

deny_claims fn main() -> own unit traps {
  let flag = True();
  claim later_seed: flag because "CLM-3 must run later";
  return unit;
}
"#;
    let issue = rejection(source, SemanticRule::Fn3, "fn value() -> own i32 pure;");
    assert_eq!(
        issue.kind(),
        &SemanticIssueKind::DuplicateContractMember {
            member: "value".to_owned(),
        }
    );
}

#[test]
fn ordinary_fn4_law_validation_precedes_a_marker_failure() {
    let source = br#"contract BadIdentity {
  fn combine(x: own u64, y: own u64) -> own u64 pure;
  law identity(combine, unit);
}

deny_claims fn main() -> own unit traps {
  let flag = True();
  claim later_seed: flag because "CLM-3 must run later";
  return unit;
}
"#;
    let issue = rejection(source, SemanticRule::Fn4, "law identity(combine, unit);");
    assert_eq!(issue.kind(), &SemanticIssueKind::InvalidContractLaw);
}

#[test]
fn a_dark_strict_ephemeral_failure_keeps_the_bind_first_repair() {
    let source = br#"fn positive(value: own u8) -> own unit pure requires {
  check ilt(value, 10_u8) else trap "small";
} {
  return unit;
}

deny_claims fn main() -> own unit pure {
  let values = array_new<u8, 2>(3_u8);
  positive(value: values[0_u64]);
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("dark strict ephemeral call must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::StrictUndischargedCallRequirement(detail) = issue.kind() else {
            panic!("expected strict call detail: {issue:?}");
        };
        assert!(
            detail
                .instantiated_goal
                .contains("argument #0 pre-transfer value")
        );
        assert_eq!(
            detail.mechanical_fix,
            "bind that argument or referent value non-consumingly with one preceding ordinary let, establish the complete requirement over that binding with a dominating real branch or another non-assertion fact source admitted by ENT-3, and pass the binding, borrowing it when the parameter mode requires a borrow"
        );
    });
}

#[test]
fn an_uninstantiated_generic_marker_is_retained_without_a_concrete_root() {
    let source = br#"deny_claims fn unused<T: Int>(value: own T) -> own T pure {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("unused marked generic must check: {outcome:?}");
        };
        assert_eq!(program.data.strict_partition.markers.len(), 1);
        assert!(program.data.strict_partition.roots.is_empty());
        assert!(program.data.strict_partition.components.is_empty());
    });
}
