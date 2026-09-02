use crate::{
    LoopInvariantProofObligation, SemanticIssueKind, SemanticLocation, SemanticOutcome,
    SemanticRule,
};

use super::super::entailment::{
    CallGoalDisposition, CallGoalEvidence, DerivationNode, ObligationFamily,
    PostconditionDisposition, SourceAffineFactRef,
};
use super::{with_semantics, with_semantics_dark};

fn assert_invariant_issue(source: &[u8], expected: LoopInvariantProofObligation) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("expected an INV-1 source rejection, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Inv1);
        let SemanticIssueKind::UndischargedLoopInvariant {
            name, obligation, ..
        } = issue.kind()
        else {
            panic!(
                "expected an undischarged loop invariant, got {:?}",
                issue.kind()
            );
        };
        assert_eq!(name, "limit");
        assert_eq!(*obligation, expected);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("INV-1 must cite the source invariant statement");
        };
        let start = usize::try_from(coordinate.start().value()).expect("source offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("source offset fits usize");
        let cited = std::str::from_utf8(&source[start..end]).expect("invariant source is UTF-8");
        assert!(
            cited.starts_with("invariant limit: ile("),
            "INV-1 cited {cited:?} instead of the complete invariant statement"
        );
        assert!(cited.ends_with(");"));
    });
}

#[test]
fn source_invariant_is_checked_at_base_and_arbitrary_backedge() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant limit: ile(i, 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("proved source invariant must check: {outcome:?}");
        };
        let invariants = &checked.data.functions[0].entailment.loop_invariants;
        assert_eq!(invariants.len(), 1);
        assert_eq!(invariants[0].name, "limit");
        assert!(invariants[0].proof.base);
        assert_eq!(invariants[0].proof.step, Some(true));
    });
}

#[test]
fn an_invariant_must_be_in_the_direct_counted_body_prefix() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    let value = i;
    invariant limit: ile(i, 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("a non-prefix invariant must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Inv1);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::InvalidLoopInvariant {
                reason: "an invariant appears after an executable statement in its loop body",
                mechanical_fix: "move every invariant into one contiguous prefix at the start of the loop body",
            }
        );
    });
}

#[test]
fn ordered_invariant_roots_have_exact_integer_normalization() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant nonstrict_forward: ile(i, 1_u64);
    invariant nonstrict_reverse: ige(1_u64, i);
    invariant strict_forward: ilt(i, 2_u64);
    invariant strict_reverse: igt(2_u64, i);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("all four ordered invariant roots must normalize: {outcome:?}");
        };
        let invariants = &checked.data.functions[0].entailment.loop_invariants;
        assert_eq!(invariants.len(), 4);
        assert!(invariants.iter().all(|invariant| invariant.proof.base));
        assert!(
            invariants
                .iter()
                .all(|invariant| invariant.proof.step == Some(true))
        );
    });

    for source in [
        br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant limit: ilt(i, 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant limit: igt(1_u64, i);
  }
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        with_semantics(source, |outcome| {
            let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("strict order must reject equality at the next header: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Inv1);
            assert!(matches!(
                issue.kind(),
                SemanticIssueKind::UndischargedLoopInvariant {
                    name,
                    obligation: LoopInvariantProofObligation::Backedge,
                    ..
                } if name == "limit"
            ));
        });
    }
}

#[test]
fn equality_and_disequality_are_not_invariant_roots() {
    for source in [
        br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant same: ieq(i, i);
  }
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
        br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant different: ine(i, 2_u64);
  }
  return exit_status(code: 0_u8);
}
"#
        .as_slice(),
    ] {
        with_semantics(source, |outcome| {
            let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("a non-ordered invariant root must be rejected: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Inv1);
            assert_eq!(
                issue.kind(),
                &SemanticIssueKind::InvalidLoopInvariant {
                    reason: "the invariant root is not an admitted ordered integer relation",
                    mechanical_fix: "write `ile`, `ilt`, `ige`, or `igt` at the invariant root; equality and disequality are not invariant roots",
                }
            );
        });
    }
}

#[test]
fn ordinary_loop_invariant_is_inductive_at_an_arbitrary_header() {
    let source = br#"fn repeat(leave: own Bool) -> result: own unit pure {
  let value = 0_u64;
  loop {
    invariant limit: ile(value, 0_u64);
    if leave {
      break;
    } else {
      set value = 0_u64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the ordinary-loop induction must check: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "repeat")
            .expect("repeat function exists");
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("repeat retains one source invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, Some(true));
    });
}

#[test]
fn ordinary_loop_invariant_must_precede_executable_statements() {
    let source = br#"fn misplaced() -> result: own unit pure {
  loop {
    let value = 0_u64;
    invariant limit: ile(value, 0_u64);
    break;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an ordinary-loop invariant after code must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Inv1);
        assert_eq!(
            issue.kind(),
            &SemanticIssueKind::InvalidLoopInvariant {
                reason: "an invariant appears after an executable statement in its loop body",
                mechanical_fix: "move every invariant into one contiguous prefix at the start of the loop body",
            }
        );
    });
}

#[test]
fn ordinary_loop_write_must_preserve_the_next_header_invariant() {
    assert_invariant_issue(
        br#"fn repeat(leave: own Bool) -> result: own unit pure {
  let value = 0_u64;
  loop {
    invariant limit: ile(value, 0_u64);
    if leave {
      break;
    } else {
      set value = 1_u64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Backedge,
    );
}

#[test]
fn ordinary_loop_break_does_not_export_its_header_invariant() {
    let source = br#"fn leave_loop(leave: own Bool) -> result: own unit pure {
  let value = 0_u64;
  loop {
    invariant limit: ile(value, 0_u64);
    if leave {
      break;
    } else {
      set value = 0_u64;
    }
  }
  let not_proved = 0_u64 - value;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an ordinary break must not export its header invariant: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
}

#[test]
fn ordinary_loop_batch_uses_all_invariants_for_each_backedge() {
    let source = br#"fn preserve_pair(a: own u64, b: own u64, c: own u64, d: own u64, leave: own Bool) -> result: own unit pure contract {
  requires ile(a, b);
  requires ile(c, d);
} {
  let first = a;
  let first_limit = b;
  let second = c;
  let second_limit = d;
  let combined_left = 0_u64;
  let combined_right = 0_u64;
  let combined_left_limit = 0_u64;
  let combined_right_limit = 0_u64;
  loop {
    invariant combined: ile(combined_left + combined_right, combined_left_limit + combined_right_limit);
    invariant first_order: ile(first, first_limit);
    invariant second_order: ile(second, second_limit);
    if leave {
      break;
    } else {
      set combined_left = first;
      set combined_right = second;
      set combined_left_limit = first_limit;
      set combined_right_limit = second_limit;
      set first = first;
      set first_limit = first_limit;
      set second = second;
      set second_limit = second_limit;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the complete invariant batch must prove the combined backedge: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve_pair")
            .expect("preserve_pair function exists");
        let invariants = &function.entailment.loop_invariants;
        assert_eq!(invariants.len(), 3);
        assert!(invariants.iter().all(|invariant| invariant.proof.base));
        assert!(
            invariants
                .iter()
                .all(|invariant| invariant.proof.step == Some(true))
        );
    });

    let source = std::str::from_utf8(source).expect("the source fixture is UTF-8");
    let without_second = source.replacen(
        "    invariant second_order: ile(second, second_limit);\n",
        "",
        1,
    );
    with_semantics(without_second.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!(
                "the missing second premise must leave the combined backedge unproved: {outcome:?}"
            );
        };
        assert_eq!(issue.rule(), SemanticRule::Inv1);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedLoopInvariant {
                name,
                obligation: LoopInvariantProofObligation::Backedge,
                ..
            } if name == "combined"
        ));
    });
}

#[test]
fn a_failed_base_batch_grants_no_ordinary_header_assumption() {
    let source = br#"fn unknown_order(left: own u64, right: own u64, leave: own Bool) -> result: own unit pure {
  loop {
    invariant first: ile(left, right);
    invariant second: ile(left, right);
    if leave {
      break;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark checking must retain the failed batch: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "unknown_order")
            .expect("unknown_order function exists");
        let [first, second] = function.entailment.loop_invariants.as_slice() else {
            panic!("unknown_order retains both invariants");
        };
        for invariant in [first, second] {
            assert!(!invariant.proof.base);
            assert_eq!(invariant.proof.step, Some(false));
        }
    });
}

#[test]
fn zero_trip_range_still_requires_the_invariant_base_case() {
    assert_invariant_issue(
        br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..0_u64 {
    invariant limit: ile(1_u64, i);
  }
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Base,
    );
}

#[test]
fn normal_body_fallthrough_must_preserve_the_invariant() {
    assert_invariant_issue(
        br#"command fn main() -> status: own ExitStatus pure {
  let sum = 0_u64;
  for i in 0_u64..1_u64 {
    invariant limit: ile(sum, i);
    set sum = 2_u64;
  }
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Backedge,
    );
}

#[test]
fn a_conditional_unit_step_preserves_the_invariant_through_an_affine_join() {
    let source = br#"fn advance(flag: own Bool) -> result: own unit pure {
  let completed = 0_u64;
  for i in 0_u64..4_u64 {
    invariant limit: ile(completed, i);
    if flag {
      set completed = completed + 1_u64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the conditional unit step must preserve its invariant: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "advance")
            .expect("advance function exists");
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("advance retains one source invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, Some(true));
    });
}

#[test]
fn an_affine_join_does_not_hide_a_branch_that_advances_too_far() {
    assert_invariant_issue(
        br#"fn advance(flag: own Bool) -> result: own unit pure {
  let completed = 0_u64;
  for i in 0_u64..4_u64 {
    invariant limit: ile(completed, i);
    if flag {
      set completed = completed + 2_u64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Backedge,
    );
}

#[test]
fn an_affine_join_retains_a_negative_constant_delta() {
    let source = br#"fn select_nonpositive(flag: own Bool) -> result: own unit pure {
  let offset = 0_i32;
  for i in 0_u64..2_u64 {
    invariant limit: ile(offset, 0_i32);
    if flag {
      set offset = -1_i32;
    } else {
      set offset = 0_i32;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the signed join delta must retain its negative lower endpoint: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "select_nonpositive")
            .expect("select_nonpositive function exists");
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("select_nonpositive retains one source invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, Some(true));
    });
}

#[test]
fn separate_joined_bindings_do_not_share_one_delta_atom() {
    assert_invariant_issue(
        br#"fn select_pair(flag: own Bool) -> result: own unit pure {
  let left = 0_u64;
  let right = 0_u64;
  for i in 0_u64..1_u64 {
    invariant limit: ile(left, right);
    if flag {
      set left = 1_u64;
      set right = 0_u64;
    } else {
      set left = 0_u64;
      set right = 1_u64;
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Backedge,
    );
}

#[test]
fn a_matching_break_is_not_a_backedge() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  for i in 0_u64..1_u64 {
    invariant limit: ile(i, 0_u64);
    break;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("an invariant with no reachable backedge must check: {outcome:?}");
        };
        let invariant = &checked.data.functions[0].entailment.loop_invariants[0];
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, None);
    });
}

#[test]
fn requirement_facts_seed_the_originating_invariant_context() {
    let source = br#"fn bounded(start: own u64) -> result: own unit pure contract {
  requires ile(start, 0_u64);
} {
  for i in start..start {
    invariant limit: ile(i, 0_u64);
    break;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the source requirement must prove the invariant base: {outcome:?}");
        };
        let bounded = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "bounded")
            .expect("bounded function exists");
        let [invariant] = bounded.entailment.loop_invariants.as_slice() else {
            panic!("bounded retains one source invariant");
        };
        assert!(invariant.proof.base);
    });

    assert_invariant_issue(
        br#"fn bounded(start: own u64) -> result: own unit pure {
  for i in start..start {
    invariant limit: ile(i, 0_u64);
    break;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Base,
    );
}

#[test]
fn source_invariant_discharges_the_weigh_addition_domain() {
    let source = br#"fn weigh['w](weights: &'w buffer<u8>, count: own u64) -> total: own u32 reads(weights) contract {
  define capacity = len(deref(weights));
  requires ile(count, capacity);
  requires ile(count, 1000_u64);
  ensures ile(total, 255000_u32);
} {
  let sum = 0_u32;
  for i in 0_u64..count {
    invariant per_byte: ile(sum, 255_u32 * i);
    let w = deref(weights)[i];
    let wide = cvt<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}

fn add_one['w](weights: &'w buffer<u8>, count: own u64) -> result: own u32 reads(weights) contract {
  define capacity = len(deref(weights));
  requires ile(count, capacity);
  requires ile(count, 1000_u64);
} {
  let total = weigh<'w>(weights: weights, count: count);
  let incremented = total + 1_u32;
  return incremented;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("weigh body must check from its source invariant: {outcome:?}");
        };
        let weigh = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "weigh")
            .expect("weigh function exists");
        super::entailment::validate_derivations(&weigh.entailment);
        let [postcondition] = weigh.entailment.postconditions.as_slice() else {
            panic!("weigh retains its one source postcondition proof");
        };
        assert!(postcondition.aggregate.discharged);
        let [exit] = postcondition.exits.as_slice() else {
            panic!("weigh has one selected return");
        };
        assert_eq!(exit.disposition, PostconditionDisposition::Discharged);
        let root = exit
            .derivation
            .expect("the accepted affine FN-9 exit has one derivation root");
        let DerivationNode::PostconditionExit { parent, .. } =
            &weigh.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the accepted FN-9 root is a postcondition exit");
        };
        assert!(matches!(
            weigh.entailment.derivations.nodes[parent.0 as usize],
            DerivationNode::AffineConsequence {
                ref premises,
                ..
            } if !premises.is_empty()
        ));
        let domain = weigh
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .expect("exact addition has one integer-domain obligation");
        assert!(domain.discharged);
        let root = domain
            .derivation
            .expect("accepted OP-2 has a derivation root");
        let DerivationNode::IntegerDomain { parents, .. } =
            &weigh.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("OP-2 root keeps its ordinary integer-domain conclusion");
        };
        assert!(parents.iter().any(|parent| matches!(
            weigh.entailment.derivations.nodes[parent.0 as usize],
            DerivationNode::AffineConsequence {
                ref premises,
                ..
            } if !premises.is_empty()
        )));

        let published = postcondition
            .summary
            .as_ref()
            .expect("weigh publishes its invariant-derived verified summary");
        assert_eq!(published.function, weigh.id);
        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&weigh.id))
            .expect("weigh belongs to one ordinary-call component");
        assert!(component.summaries.contains(published));

        let add_one = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "add_one")
            .expect("add_one function exists");
        super::entailment::validate_derivations(&add_one.entailment);
        let caller_domains = add_one
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .collect::<Vec<_>>();
        let [caller_domain] = caller_domains.as_slice() else {
            panic!("add_one retains exactly one integer-domain obligation");
        };
        assert!(caller_domain.discharged);
        let root = caller_domain
            .derivation
            .expect("the accepted caller addition has one OP-2 root");
        let mut seen = vec![false; add_one.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut reaches_weigh_summary = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &add_one.entailment.derivations.nodes[index];
            reaches_weigh_summary |= matches!(
                retained,
                DerivationNode::PostconditionCall { detail }
                    if detail.summary.summary.function == weigh.id
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            reaches_weigh_summary,
            "the caller OP-2 proof must descend from weigh's ordinary S12 relation"
        );
    });
}

#[test]
fn later_invariant_backedge_can_use_an_earlier_invariant() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let x = 0_u64;
  let y = 0_u64;
  for i in 0_u64..10_u64 {
    invariant x_tracks_i: ile(x, i);
    invariant limit: ile(y, i);
    set y = x;
    set x = i;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the later invariant may use the whole invariant prefix: {outcome:?}");
        };
        let invariants = &checked.data.functions[0].entailment.loop_invariants;
        assert_eq!(invariants.len(), 2);
        assert_eq!(invariants[0].name, "x_tracks_i");
        assert_eq!(invariants[1].name, "limit");
        assert_eq!(invariants[1].proof.step, Some(true));
    });

    assert_invariant_issue(
        br#"command fn main() -> status: own ExitStatus pure {
  let x = 0_u64;
  let y = 0_u64;
  for i in 0_u64..10_u64 {
    invariant limit: ile(y, i);
    set y = x;
    set x = i;
  }
  return exit_status(code: 0_u8);
}
"#,
        LoopInvariantProofObligation::Backedge,
    );
}

#[test]
fn descending_range_does_not_publish_a_false_exhaustion_substitution() {
    let source = br#"fn descending(value: own u64) -> result: own u64 pure contract {
  requires ile(value, 2_u64);
  ensures ile(result, 1_u64);
} {
  for i in 2_u64..1_u64 {
    invariant limit: ile(value, i);
  }
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark checking retains the failed postcondition: {outcome:?}");
        };
        let descending = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "descending")
            .expect("descending function exists");
        super::entailment::validate_derivations(&descending.entailment);
        let [invariant] = descending.entailment.loop_invariants.as_slice() else {
            panic!("descending retains one source invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, Some(true));
        let [proof] = descending.entailment.postconditions.as_slice() else {
            panic!("descending retains its one postcondition proof");
        };
        let [exit] = proof.exits.as_slice() else {
            panic!("descending has one selected return");
        };
        assert_eq!(exit.disposition, PostconditionDisposition::Unproved);
        assert!(!proof.aggregate.discharged);
    });
}

#[test]
fn matching_break_removes_false_header_exhaustion_facts_at_the_join() {
    let source = br#"fn may_stop(stop: own Bool) -> result: own u64 pure contract {
  ensures ile(result, 1_u64);
} {
  let value = 0_u64;
  for i in 0_u64..1_u64 {
    invariant limit: ile(value, i);
    if stop {
      set value = 2_u64;
      break;
    }
  }
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark checking retains the break-path postcondition: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "may_stop")
            .expect("may_stop function exists");
        super::entailment::validate_derivations(&function.entailment);
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("may_stop retains one source invariant");
        };
        assert!(invariant.proof.discharged());
        let [proof] = function.entailment.postconditions.as_slice() else {
            panic!("may_stop retains one postcondition proof");
        };
        let [exit] = proof.exits.as_slice() else {
            panic!("may_stop has one selected return");
        };
        assert_ne!(exit.disposition, PostconditionDisposition::Discharged);
        assert!(!proof.aggregate.discharged);
    });
}

#[test]
fn no_backedge_invariant_can_finish_with_a_safe_false_header_exit() {
    let source = br#"fn zero_trip(value: own u64) -> result: own u64 pure contract {
  requires ile(value, 0_u64);
  ensures ile(result, 0_u64);
} {
  for i in 0_u64..0_u64 {
    invariant limit: ile(value, i);
    return 0_u64;
  }
  set value = value;
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the zero-trip false-header exit must check: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "zero_trip")
            .expect("zero_trip function exists");
        super::entailment::validate_derivations(&function.entailment);
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("zero_trip retains one source invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, None);
        let [proof] = function.entailment.postconditions.as_slice() else {
            panic!("zero_trip retains one postcondition proof");
        };
        assert!(proof.aggregate.discharged);
        let [body_exit, false_header_exit] = proof.exits.as_slice() else {
            panic!("zero_trip has its body return and false-header return");
        };
        assert_eq!(body_exit.disposition, PostconditionDisposition::Discharged);
        let root = false_header_exit
            .derivation
            .expect("the accepted no-backedge FN-9 exit has one derivation root");
        let DerivationNode::PostconditionExit { parent, .. } =
            &function.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the accepted FN-9 root is a postcondition exit");
        };
        assert!(matches!(
            function.entailment.derivations.nodes[parent.0 as usize],
            DerivationNode::AffineConsequence {
                ref premises,
                ..
            } if !premises.is_empty()
        ));
    });
}

#[test]
fn failed_invariant_withholds_other_summaries_in_the_same_scc() {
    let source = br#"fn left(value: own i32) -> result: own i32 pure contract {
  ensures ieq(result, value);
} {
  let ignored = right(value: value);
  return value;
}

fn right(value: own i32) -> result: own i32 pure {
  let ignored = left(value: value);
  for i in 0_u64..1_u64 {
    invariant limit: ile(1_u64, i);
    break;
  }
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark checking retains the failed invariant metadata: {outcome:?}");
        };
        let left = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "left")
            .expect("left function exists");
        let right = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "right")
            .expect("right function exists");
        super::entailment::validate_derivations(&left.entailment);
        super::entailment::validate_derivations(&right.entailment);
        let [left_postcondition] = left.entailment.postconditions.as_slice() else {
            panic!("left retains its one postcondition proof");
        };
        assert!(left_postcondition.aggregate.discharged);
        assert!(left_postcondition.summary.is_none());
        assert!(right.entailment.postconditions.is_empty());
        let [failed] = right.entailment.loop_invariants.as_slice() else {
            panic!("right retains its failing source invariant");
        };
        assert!(!failed.proof.base);

        let component = checked
            .data
            .postcondition_schedule
            .components
            .iter()
            .find(|component| component.functions.contains(&left.id))
            .expect("left belongs to one ordinary-call component");
        assert_eq!(component.functions.len(), 2);
        assert!(component.functions.contains(&right.id));
        assert!(component.summaries.is_empty());
    });
    assert_invariant_issue(source, LoopInvariantProofObligation::Base);
}

#[test]
fn active_invariant_proves_a_real_array_index_obligation() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new<u8, 4>(0_u8);
  let at = 0_u64;
  for i in 0_u64..4_u64 {
    invariant position: ile(at, i);
    let value = values[at];
    set at = at + 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the source invariant must prove the array index: {outcome:?}");
        };
        let function = &checked.data.functions[0];
        let bounds = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::Bounds)
            .collect::<Vec<_>>();
        let [index] = bounds.as_slice() else {
            panic!("the loop body retains one OP-4 array-index obligation");
        };
        assert!(index.discharged);
        let root = index
            .derivation
            .expect("the accepted OP-4 index retains a derivation root");
        let mut seen = vec![false; function.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut used_invariant = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &function.entailment.derivations.nodes[index];
            used_invariant |= matches!(
                retained,
                DerivationNode::AffineConsequence {
                    premises,
                    ..
                } if !premises.is_empty()
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            used_invariant,
            "the OP-4 root must descend from the source invariant"
        );
    });
}

#[test]
fn one_exhaustion_fact_proves_a_requirement_and_postcondition() {
    let source = br#"fn accept_total(value: own u32) -> result: own unit pure contract {
  requires ile(value, 255000_u32);
} {
  return unit;
}

fn finish(count: own u64) -> result: own u32 pure contract {
  requires ile(count, 1000_u64);
  ensures ile(result, 255000_u32);
} {
  let total = 0_u32;
  for i in 0_u64..count {
    invariant per_item: ile(total, 255_u32 * i);
    set total = total + 255_u32;
  }
  let ignored = accept_total(value: total);
  return total;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the exhaustion fact must prove the ordinary requirement: {outcome:?}");
        };
        let finish = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "finish")
            .expect("finish function exists");
        super::entailment::validate_derivations(&finish.entailment);
        let [call] = finish.entailment.call_goals.as_slice() else {
            panic!("finish retains one ordinary function requirement");
        };
        assert_eq!(call.disposition, CallGoalDisposition::Discharged);
        assert_eq!(call.evidence, vec![CallGoalEvidence::AffinePositive]);

        let [postcondition] = finish.entailment.postconditions.as_slice() else {
            panic!("finish retains one source postcondition proof");
        };
        let [exit] = postcondition.exits.as_slice() else {
            panic!("finish has one selected return");
        };
        assert_eq!(exit.disposition, PostconditionDisposition::Discharged);

        let root = call
            .derivation
            .expect("the accepted function requirement retains a derivation root");
        let mut seen = vec![false; finish.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut used_exhaustion = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &finish.entailment.derivations.nodes[index];
            used_exhaustion |= matches!(
                retained,
                DerivationNode::AffineConsequence {
                    premises,
                    ..
                } if !premises.is_empty()
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            used_exhaustion,
            "the FN-8 root must descend from the exported source invariant"
        );
        let root = exit
            .derivation
            .expect("the accepted FN-9 exit retains one derivation");
        let DerivationNode::PostconditionExit { parent, .. } =
            &finish.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the FN-9 root must be a postcondition exit");
        };
        assert!(matches!(
            finish.entailment.derivations.nodes[parent.0 as usize],
            DerivationNode::AffineConsequence {
                ref premises,
                ..
            } if !premises.is_empty()
        ));
    });
}

#[test]
fn matching_break_does_not_publish_an_exhaustion_fact_to_a_later_call() {
    let source = br#"fn accept_small(value: own u64) -> result: own unit pure contract {
  requires ile(value, 1_u64);
} {
  return unit;
}

fn finish_or_stop(stop: own Bool) -> result: own unit pure {
  let value = 0_u64;
  for i in 0_u64..1_u64 {
    invariant position: ile(value, i);
    if stop {
      set value = 2_u64;
      break;
    }
  }
  let ignored = accept_small(value: value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("dark checking must retain the break-join call judgment: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "finish_or_stop")
            .expect("finish_or_stop function exists");
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("finish_or_stop retains one source invariant");
        };
        assert!(invariant.proof.discharged());
        let [call] = function.entailment.call_goals.as_slice() else {
            panic!("finish_or_stop retains one ordinary function requirement");
        };
        assert_eq!(call.disposition, CallGoalDisposition::Unproved);
        assert!(call.derivation.is_none());
    });
}

#[test]
fn active_invariant_proves_a_dynamic_buffer_index_obligation() {
    let source = br#"fn read_prefix['v](values: &'v buffer<u8>, count: own u64) -> result: own unit reads(values) contract {
  define capacity = len(deref(values));
  requires ile(count, capacity);
} {
  let index = 0_u64;
  for i in 0_u64..count {
    invariant position: ile(index, i);
    let value = deref(values)[index];
    set index = index + 1_u64;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the active invariant must prove the dynamic buffer index: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "read_prefix")
            .expect("read_prefix function exists");
        super::entailment::validate_derivations(&function.entailment);
        let index = function
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::Bounds)
            .expect("the buffer read retains one OP-4 obligation");
        assert!(index.discharged);

        let root = index
            .derivation
            .expect("the accepted OP-4 retains a derivation root");
        let mut seen = vec![false; function.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut used_invariant = false;
        while let Some(node) = stack.pop() {
            let position = node.0 as usize;
            if seen[position] {
                continue;
            }
            seen[position] = true;
            let retained = &function.entailment.derivations.nodes[position];
            used_invariant |= matches!(
                retained,
                DerivationNode::AffineConsequence {
                    premises,
                    ..
                } if !premises.is_empty()
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            used_invariant,
            "the OP-4 proof must descend from the active source invariant"
        );
    });
}

#[test]
fn exhaustion_fact_proves_filled_and_vacant_buffer_allocation_fit() {
    let source =
        br#"fn allocate_prefix(count: own u64) -> result: own unit allocates(heap) contract {
  requires ile(count, 1000_u64);
} {
  let length = 0_u64;
  for i in 0_u64..count {
    invariant produced: ile(length, i);
    set length = length + 1_u64;
  }
  let filled = buffer_new(length, 0_u16);
  let vacant = buffer_vacant<u8>(length);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the exhaustion fact must prove both allocation fits: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "allocate_prefix")
            .expect("allocate_prefix function exists");
        super::entailment::validate_derivations(&function.entailment);
        let allocations = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::AllocationFit)
            .collect::<Vec<_>>();
        assert_eq!(allocations.len(), 2, "both allocation forms retain OP-9");
        for allocation in allocations {
            assert!(allocation.discharged);

            let root = allocation
                .derivation
                .expect("the accepted OP-9 retains a derivation root");
            assert!(matches!(
                function.entailment.derivations.nodes[root.0 as usize],
                DerivationNode::GoalNormalization {
                    sign: super::super::entailment::GoalSign::Positive,
                    ..
                }
            ));
            let mut seen = vec![false; function.entailment.derivations.nodes.len()];
            let mut stack = vec![root];
            let mut used_exhaustion = false;
            while let Some(node) = stack.pop() {
                let position = node.0 as usize;
                if seen[position] {
                    continue;
                }
                seen[position] = true;
                let retained = &function.entailment.derivations.nodes[position];
                used_exhaustion |= matches!(
                    retained,
                    DerivationNode::AffineConsequence {
                        premises,
                        ..
                    } if !premises.is_empty()
                );
                stack.extend(retained.parent_ids());
            }
            assert!(
                used_exhaustion,
                "each OP-9 proof must descend from the exported source invariant"
            );
        }
    });
}

#[test]
fn exhaustion_facts_prove_both_system_range_components() {
    let source = br#"fn publish_prefix['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, limit: own u64) -> result: own unit reads(output, source), writes(output) contract {
  define capacity = len(deref(source));
  requires ile(limit, capacity);
} {
  let start = 0_u64;
  let end = 0_u64;
  for i in 0_u64..limit {
    invariant ordered: ile(start, end);
    invariant within_prefix: ile(end, i);
    set start = end;
    set end = end + 1_u64;
  }
  region 'attempt {
    let outcome = write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, start: start, end: end);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the exported invariants must prove both system ranges: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "publish_prefix")
            .expect("publish_prefix function exists");
        super::entailment::validate_derivations(&function.entailment);
        let ranges = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::SystemRange)
            .collect::<Vec<_>>();
        assert_eq!(ranges.len(), 2, "write_once retains both SYS-8 components");
        assert_eq!(ranges[0].conjunct, 0);
        assert_eq!(ranges[1].conjunct, 1);
        assert_eq!(ranges[0].node_path, ranges[1].node_path);
        for range in &ranges {
            assert!(range.discharged);
            let root = range
                .derivation
                .expect("each accepted SYS-8 component retains a derivation root");
            let mut seen = vec![false; function.entailment.derivations.nodes.len()];
            let mut stack = vec![root];
            let mut used_expected_invariant = false;
            while let Some(node) = stack.pop() {
                let position = node.0 as usize;
                if seen[position] {
                    continue;
                }
                seen[position] = true;
                let retained = &function.entailment.derivations.nodes[position];
                if let DerivationNode::AffineConsequence { premises, .. } = retained {
                    used_expected_invariant |= premises.iter().any(|premise| {
                        matches!(
                            premise.source,
                            SourceAffineFactRef::LoopInvariant(source)
                                if source.source_ordinal == u32::from(range.conjunct)
                        )
                    });
                }
                stack.extend(retained.parent_ids());
            }
            assert!(
                used_expected_invariant,
                "each SYS-8 component must descend from its corresponding exported invariant"
            );
        }
    });
}

#[test]
fn independent_invariant_intervals_discharge_two_operand_exact_multiplication() {
    let source = br#"fn bounded_product(count: own u64) -> result: own u64 pure contract {
  requires ile(count, 1000_u64);
} {
  let left = 0_u64;
  let right = 0_u64;
  for i in 0_u64..count {
    invariant left_at_head: ile(left, i);
    invariant right_at_head: ile(right, i);
    let product = left * right;
    set left = i;
    set right = i;
  }
  return left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("independent invariant intervals must prove the product: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "bounded_product")
            .expect("bounded_product exists");
        super::entailment::validate_derivations(&function.entailment);
        let domains = function
            .entailment
            .obligations
            .iter()
            .filter(|outcome| outcome.family == ObligationFamily::IntegerDomain)
            .collect::<Vec<_>>();
        let [domain] = domains.as_slice() else {
            panic!("the source product must retain one OP-2 obligation: {domains:?}");
        };
        assert!(domain.discharged);

        let root = domain
            .derivation
            .expect("the accepted product retains an integer-domain root");
        let DerivationNode::IntegerDomain { parents, .. } =
            &function.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the product root must be an integer-domain conclusion");
        };
        assert_eq!(parents.len(), 4, "one proof per closed-interval endpoint");
        let mut premise_ordinals = parents
            .iter()
            .filter_map(|parent| {
                let DerivationNode::AffineConsequence { premises, .. } =
                    &function.entailment.derivations.nodes[parent.0 as usize]
                else {
                    panic!("each product-domain parent must prove one affine endpoint");
                };
                premises.iter().find_map(|premise| match premise.source {
                    SourceAffineFactRef::LoopInvariant(source) => Some(source.source_ordinal),
                    SourceAffineFactRef::SourceProof { .. } => None,
                })
            })
            .collect::<Vec<_>>();
        premise_ordinals.sort_unstable();
        premise_ordinals.dedup();
        assert_eq!(
            premise_ordinals,
            vec![0, 1],
            "both independent source invariants must supply their operand upper bound"
        );
    });
}

#[test]
fn interval_product_checks_the_two_cross_endpoint_pairs() {
    let source = br#"fn mixed(left: own i8, right: own i8) -> result: own unit pure contract {
  requires ile(left, 1_i8);
  requires ile(0_i8, right);
  requires ile(right, 2_i8);
} {
  for i in 0_u64..1_u64 {
    invariant left_upper: ile(left, 1_i8);
    invariant right_lower: ile(0_i8, right);
    invariant right_upper: ile(right, 2_i8);
    let product = left * right;
    break;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the -128 * 2 cross endpoint must keep the product unproved: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op2);
    });
}

/// The outer invariant proves the inner base, the inner exhaustion export
/// proves the outer step, and the outer exhaustion export proves the function
/// postcondition. The caller then uses that verified postcondition to satisfy
/// a different function's requirement.
/// The written proof is the exact algebraic bridge from inner exhaustion to
/// the arbitrary outer backedge; its target belongs in that body proof context.
#[test]
fn nested_invariants_publish_a_postcondition_consumed_by_a_later_requirement() {
    let source = br#"fn accept_total(value: own u64) -> result: own unit pure contract {
  requires ile(value, 12_u64);
} {
  return unit;
}

fn count_cells(rows: own u64) -> result: own u64 pure contract {
  requires ile(rows, 3_u64);
  ensures ile(result, 12_u64);
} {
  let total = 0_u64;
  for row in 0_u64..rows {
    invariant completed_rows: ile(total, 4_u64 * row);
    for column in 0_u64..4_u64 {
      invariant completed_cells: ile(total, 4_u64 * row + column);
      set total = total + 1_u64;
    }
    prove completed_row: ile(total, 4_u64 *(row + 1_u64)) {
      use ile(total, 4_u64 * row + 4_u64);
    }
  }
  return total;
}

fn caller(rows: own u64) -> result: own unit pure contract {
  requires ile(rows, 3_u64);
} {
  let total = count_cells(rows: rows);
  let ignored = accept_total(value: total);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("nested invariant and contract composition must check: {outcome:?}");
        };
        let count_cells = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "count_cells")
            .expect("count_cells function exists");
        super::entailment::validate_derivations(&count_cells.entailment);
        let completed_rows = count_cells
            .entailment
            .loop_invariants
            .iter()
            .find(|invariant| invariant.name == "completed_rows")
            .expect("outer invariant is retained");
        let completed_cells = count_cells
            .entailment
            .loop_invariants
            .iter()
            .find(|invariant| invariant.name == "completed_cells")
            .expect("inner invariant is retained");
        assert_ne!(completed_rows.loop_id, completed_cells.loop_id);
        for invariant in [completed_rows, completed_cells] {
            assert!(invariant.proof.base);
            assert_eq!(invariant.proof.step, Some(true));
        }

        let [postcondition] = count_cells.entailment.postconditions.as_slice() else {
            panic!("count_cells retains one postcondition proof");
        };
        assert!(postcondition.aggregate.discharged);
        let [exit] = postcondition.exits.as_slice() else {
            panic!("count_cells has one selected return");
        };
        assert_eq!(exit.disposition, PostconditionDisposition::Discharged);
        let exit_root = exit
            .derivation
            .expect("the nested-loop postcondition retains a derivation");
        let mut seen = vec![false; count_cells.entailment.derivations.nodes.len()];
        let mut stack = vec![exit_root];
        let mut used_outer_exhaustion = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &count_cells.entailment.derivations.nodes[index];
            used_outer_exhaustion |= matches!(
                retained,
                DerivationNode::AffineConsequence {
                    premises,
                    ..
                } if premises.iter().any(|premise| matches!(
                    premise.source,
                    SourceAffineFactRef::LoopInvariant(source)
                        if source.loop_id == completed_rows.loop_id
                ))
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            used_outer_exhaustion,
            "the postcondition must descend from the outer loop's normal-exhaustion fact"
        );

        let accept_total = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "accept_total")
            .expect("accept_total function exists");
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function exists");
        super::entailment::validate_derivations(&caller.entailment);
        assert!(
            caller
                .entailment
                .call_goals
                .iter()
                .all(|goal| goal.disposition == CallGoalDisposition::Discharged)
        );
        let accepted_total = caller
            .entailment
            .call_goals
            .iter()
            .find(|goal| goal.callee == accept_total.id)
            .expect("accept_total retains one caller requirement");
        let root = accepted_total
            .derivation
            .expect("the discharged accept_total call retains a derivation");
        let mut seen = vec![false; caller.entailment.derivations.nodes.len()];
        let mut stack = vec![root];
        let mut used_count_cells_summary = false;
        while let Some(node) = stack.pop() {
            let index = node.0 as usize;
            if seen[index] {
                continue;
            }
            seen[index] = true;
            let retained = &caller.entailment.derivations.nodes[index];
            used_count_cells_summary |= matches!(
                retained,
                DerivationNode::PostconditionCall { detail }
                    if detail.summary.summary.function == count_cells.id
            );
            stack.extend(retained.parent_ids());
        }
        assert!(
            used_count_cells_summary,
            "the later requirement must descend from count_cells' verified postcondition"
        );
    });
}

/// The four assignments replace every value mentioned by the invariant. The
/// fixed residual rule composes the two still-live L0 requirements in
/// canonical term-pair order, so the explicit proof remains valid but is no
/// longer required for this common two-bound backedge.
#[test]
fn automatic_residual_reduction_composes_two_live_l0_facts() {
    let source = br#"fn preserve_pair_bounds(first: own u64, first_limit: own u64, second: own u64, second_limit: own u64) -> result: own unit pure contract {
  requires ile(first, first_limit);
  requires ile(second, second_limit);
} {
  let left = 0_u64;
  let left_limit = 0_u64;
  let right = 0_u64;
  let right_limit = 0_u64;
  for i in 0_u64..2_u64 {
    invariant combined: ile(left + right, left_limit + right_limit);
    set left = first;
    set left_limit = first_limit;
    set right = second;
    set right_limit = second_limit;
    prove restored: ile(left + right, left_limit + right_limit) {
      use ile(left, left_limit);
      use ile(right, right_limit);
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the source proof target must establish the outer backedge: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve_pair_bounds")
            .expect("preserve_pair_bounds function exists");
        super::entailment::validate_derivations(&function.entailment);

        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("preserve_pair_bounds retains one invariant");
        };
        assert!(invariant.proof.base);
        assert_eq!(invariant.proof.step, Some(true));

        let [proof] = function.entailment.source_proofs.as_slice() else {
            panic!("preserve_pair_bounds retains one source proof");
        };
        assert_eq!(proof.name, "restored");
        assert_eq!(proof.check.premises, [true, true]);
        assert!(proof.check.combination);
        assert!(proof.check.discharged());
    });

    const PROOF: &str = "    prove restored: ile(left + right, left_limit + right_limit) {\n      use ile(left, left_limit);\n      use ile(right, right_limit);\n    }\n";
    let source = std::str::from_utf8(source).expect("the source fixture is UTF-8");
    assert_eq!(source.matches(PROOF).count(), 1);
    let without_proof = source.replacen(PROOF, "", 1);
    with_semantics(without_proof.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the fixed two-premise reduction must prove the backedge: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve_pair_bounds")
            .expect("preserve_pair_bounds function exists");
        let [invariant] = function.entailment.loop_invariants.as_slice() else {
            panic!("the no-proof form retains one invariant");
        };
        assert_eq!(invariant.proof.step, Some(true));
        assert!(function.entailment.source_proofs.is_empty());
    });

    let missing_second = without_proof.replacen("  requires ile(second, second_limit);\n", "", 1);
    with_semantics(missing_second.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("one missing bound must leave the backedge unproved: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Inv1);
        let SemanticIssueKind::UndischargedLoopInvariant {
            name, obligation, ..
        } = issue.kind()
        else {
            panic!("the incomplete premise set must fail at the invariant backedge");
        };
        assert_eq!(name, "combined");
        assert_eq!(*obligation, LoopInvariantProofObligation::Backedge);
    });
}

/// A break from the inner loop reaches the statement after that loop without
/// passing through its normal-exhaustion edge. The completed-prefix relation
/// therefore cannot satisfy the later requirement on every path.
#[test]
fn an_inner_break_does_not_export_its_exhaustion_fact_to_a_later_requirement() {
    let source = br#"fn accept_total(value: own u64) -> result: own unit pure contract {
  requires ile(value, 4_u64);
} {
  return unit;
}

fn count_or_stop(stop: own Bool) -> result: own unit pure {
  let total = 0_u64;
  for row in 0_u64..1_u64 {
    invariant outer_range: ile(row, 1_u64);
    for column in 0_u64..4_u64 {
      invariant completed: ile(total, column);
      if stop {
        set total = 5_u64;
        break;
      }
      set total = total + 1_u64;
    }
    let ignored = accept_total(value: total);
    break;
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("inner-break proof state must remain inspectable: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "count_or_stop")
            .expect("count_or_stop function exists");
        super::entailment::validate_derivations(&function.entailment);
        let completed = function
            .entailment
            .loop_invariants
            .iter()
            .find(|invariant| invariant.name == "completed")
            .expect("inner invariant is retained");
        assert!(completed.proof.base);
        assert_eq!(completed.proof.step, Some(true));
        let [call] = function.entailment.call_goals.as_slice() else {
            panic!("count_or_stop retains one ordinary requirement");
        };
        assert_eq!(call.disposition, CallGoalDisposition::Unproved);
        assert!(call.derivation.is_none());
    });
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the unproved post-loop requirement must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("FN-8 must cite the call whose requirement is unproved");
        };
        let start = usize::try_from(coordinate.start().value()).expect("source offset fits usize");
        let end = usize::try_from(coordinate.end().value()).expect("source offset fits usize");
        assert_eq!(
            std::str::from_utf8(&source[start..end]).expect("call source is UTF-8"),
            "accept_total(value: total)"
        );
    });
}
